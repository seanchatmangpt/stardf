//! LANE 8 — ArrowArrayStream C ABI surface (`stream_c.rs`).
//!
//! Bridges `stardf_arrow_core::stream` (pure-Rust producer + vtable) to raw
//! C: every callback crossing the membrane is `extern "C"` and guarded by
//! `catch_unwind` — a Rust panic inside a callback is converted to a nonzero
//! error flag, never unwinding across FFI.
//!
//! End-of-stream follows the Arrow C Stream Interface spec (and the lib.rs
//! consumer test): `get_next` returns 0 and writes a zeroed (release-less)
//! array, not a nonzero flag. Errors return the core `STREAM_ERR` flag (1).
//!
//! API consumed by `lib.rs` (unchanged from the lane-7 fallback contract):
//!
//! ```text
//! #[repr(C)] pub struct ArrowArrayStream {
//!     get_schema, get_next, get_last_error, release, private_data
//! }
//! pub fn export_stream(batch: &StardfBatch) -> *mut ArrowArrayStream;
//! pub fn stream_free(stream: *mut ArrowArrayStream);
//! ```

#![allow(dead_code)]

use crate::StardfBatch;
use core::ffi::{c_char, c_void};
use std::alloc::Layout; // LANE 10: ownership-tag layout
use std::boxed::Box;

// LANE 09 decouple (2026-10-31): was `use cgap_arrow::c_abi::{ArrowArray,
// ArrowSchema}` — the callbacks now speak stardf-arrow-core's spec-fixed
// stream struct shapes directly (crate root re-exports the same types), so
// the cgap git crate left this file entirely.
use stardf_arrow_core::stream as core_stream;
use stardf_arrow_core::stream::{ArrowArray, ArrowSchema};

// The capi callbacks and the core stream now share ONE struct definition
// (spec-fixed); these asserts pin the C Data Interface ABI shape on the
// LP64 targets this membrane ships for, plus the fn-pointer/pointer-slot
// pun the release path relies on.
const _: () = assert!(core::mem::size_of::<ArrowArray>() == 80);
const _: () = assert!(core::mem::size_of::<ArrowSchema>() == 72);
const _: () = assert!(
    core::mem::size_of::<Option<unsafe extern "C" fn()>>() == core::mem::size_of::<*mut c_void>()
);

/// struct ArrowArrayStream (C Stream Interface spec). Function pointers are
/// `extern "C"` — C consumers call these through the struct, so the plain
/// Rust fn pointers the core vtable carries must never leak through.
#[repr(C)]
pub struct ArrowArrayStream {
    /// Fill `out` with the batch schema; 0 on success.
    pub get_schema: Option<unsafe extern "C" fn(*mut ArrowArrayStream, *mut ArrowSchema) -> i32>,
    /// Fill `out` with the next batch; 0 on success, zeroed `out` at EOS.
    pub get_next: Option<unsafe extern "C" fn(*mut ArrowArrayStream, *mut ArrowArray) -> i32>,
    /// Last error on this stream; NULL when none is pending.
    pub get_last_error: Option<unsafe extern "C" fn(*mut ArrowArrayStream) -> *const c_char>,
    /// Release the stream (CAS-swap discipline downstream); NULL once done.
    pub release: Option<unsafe extern "C" fn(*mut ArrowArrayStream)>,
    /// Opaque owner state.
    pub private_data: *mut c_void,
}

/// Private state boxed behind `ArrowArrayStream.private_data`: the core
/// stream value (its own vtable + producer) moved across the bridge.
struct StreamPriv {
    core: core_stream::ArrowArrayStream,
}

/// SAFETY: `stream` is null-checked by the caller. Returns None for a
/// released stream (private_data consumed to NULL by [`stream_release`]):
/// the typed dead state, never dereferenced.
unsafe fn core_of<'a>(stream: *mut ArrowArrayStream) -> Option<&'a mut StreamPriv> {
    unsafe {
        let p = (*stream).private_data;
        if p.is_null() {
            None
        } else {
            Some(&mut *(p as *mut StreamPriv))
        }
    }
}

/// Panics become error flags: [`run_guarded`] never lets an unwind cross the
/// FFI boundary (note: the workspace profile builds with `panic = "abort"`,
/// where the guard is vacuous by construction; it is load-bearing for rlib
/// consumers linked with the default unwind profile).
fn run_guarded(f: impl FnOnce() -> i32) -> i32 {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(flag) => flag,
        Err(_) => core_stream::STREAM_ERR,
    }
}

/// `extern "C"` get_schema: delegate to the core vtable through the bridge.
unsafe extern "C" fn stream_get_schema(
    stream: *mut ArrowArrayStream, out: *mut ArrowSchema,
) -> i32 {
    if stream.is_null() || out.is_null() {
        return core_stream::STREAM_ERR;
    }
    run_guarded(|| {
        // A released stream has private_data == NULL: typed dead, error flag.
        let core = match unsafe { core_of(stream) } {
            Some(c) => c,
            None => return core_stream::STREAM_ERR,
        };
        let get_schema = core.core.get_schema;
        // SAFETY: layout equality is const-asserted; `stream`/`out` are the
        // caller's live structs and the core vtable owns the same contract.
        unsafe { get_schema(&mut core.core, out) }
    })
}

/// `extern "C"` get_next: delegate to the core vtable, then translate the
/// core's `STREAM_EOF` flag into the spec's zeroed-array end-of-stream.
unsafe extern "C" fn stream_get_next(stream: *mut ArrowArrayStream, out: *mut ArrowArray) -> i32 {
    if stream.is_null() || out.is_null() {
        return core_stream::STREAM_ERR;
    }
    run_guarded(|| {
        // A released stream has private_data == NULL: typed dead, error flag.
        let core = match unsafe { core_of(stream) } {
            Some(c) => c,
            None => return core_stream::STREAM_ERR,
        };
        let get_next = core.core.get_next;
        // SAFETY: layout equality is const-asserted; spec contract as above.
        match unsafe { get_next(&mut core.core, out) } {
            core_stream::STREAM_OK => {
                if unsafe { (*out).release.is_none() } || unsafe { (*out).length == 0 } {
                    // Either the core yielded an unreleased (EOS) array, or
                    // it yielded a zero-length batch (the CursorProducer
                    // emits one for an empty slab instead of EOS). Both are
                    // translated to the spec end-of-stream: flag 0 with a
                    // zeroed release-less array — an empty yield carries no
                    // data on this fixed-64 surface, so the translation
                    // never discards rows.
                    unsafe {
                        std::ptr::write_bytes(out as *mut u8, 0, core::mem::size_of::<ArrowArray>())
                    };
                }
                core_stream::STREAM_OK
            }
            core_stream::STREAM_EOF => {
                // Spec end-of-stream: flag 0, release-less zeroed array.
                unsafe {
                    std::ptr::write_bytes(out as *mut u8, 0, core::mem::size_of::<ArrowArray>())
                };
                core_stream::STREAM_OK
            }
            flag => flag,
        }
    })
}

/// `extern "C"` get_last_error: delegate to the core vtable.
unsafe extern "C" fn stream_get_last_error(stream: *mut ArrowArrayStream) -> *const c_char {
    if stream.is_null() {
        return core::ptr::null();
    }
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // A released stream has private_data == NULL: typed dead, NULL error.
        let core = match unsafe { core_of(stream) } {
            Some(c) => c,
            None => return core::ptr::null(),
        };
        let get_last_error = core.core.get_last_error;
        // SAFETY: core vtable contract; NULL when no error is pending.
        unsafe { get_last_error(&mut core.core) }
    })) {
        Ok(p) => p,
        Err(_) => core::ptr::null(),
    }
}

/// SAFETY: single release. CAS-swaps the release slot to NULL BEFORE any
/// drop, so a double release is a typed no-op, never a double free (house
/// discipline: cgap-arrow c_abi.rs:70-77).
unsafe extern "C" fn stream_release(stream: *mut ArrowArrayStream) {
    if stream.is_null() {
        return;
    }
    // Option<unsafe extern "C" fn> is a nullable pointer with a niche;
    // AtomicPtr<c_void> is repr(transparent) over *mut c_void, so an
    // in-place atomic view of the release slot is a pointer pun.
    let slot: &core::sync::atomic::AtomicPtr<c_void> = unsafe {
        &*(core::ptr::addr_of_mut!((*stream).release) as *mut core::sync::atomic::AtomicPtr<c_void>)
    };
    let prev = slot.swap(core::ptr::null_mut(), core::sync::atomic::Ordering::AcqRel);
    if prev.is_null() {
        return; // already released: typed no-op
    }
    let raw = unsafe { (*stream).private_data };
    if !raw.is_null() {
        unsafe { (*stream).private_data = core::ptr::null_mut() };
        // SAFETY: recreated exactly as boxed in export_stream.
        let mut priv_box = unsafe { Box::from_raw(raw as *mut StreamPriv) };
        // Release the inner core stream exactly once (its own release slot
        // is still armed: the outer slot was live until the swap above).
        if let Some(release) = priv_box.core.release {
            // SAFETY: inner stream is live; its release is CAS-guarded.
            unsafe { release(&mut priv_box.core as *mut _) };
        }
        drop(priv_box);
    }
    // Ownership law (model: cgap-arrow c_abi.rs release_array): release
    // consumes ONLY private_data. The stream shell stays allocated — its
    // disposal belongs solely to stream_free / stardf_stream_free, whose
    // internal release re-fires as the typed no-op above before the shell
    // is dropped. Dropping the shell here double-frees every consumer
    // that finishes with stardf_stream_free.
}

/// Export `batch` as an owned `ArrowArrayStream` over fixed 64-quad batches.
/// Builds the `Quad64` slab from the batch's term-id rows and hands it to
/// `stardf_arrow_core::stream::CursorProducer`; the returned handle speaks
/// raw `extern "C"` and never unwinds across the boundary.
pub fn export_stream(batch: &StardfBatch) -> *mut ArrowArrayStream {
    let slab: std::vec::Vec<stardf_arrow_core::layout::Quad64> = batch
        .quads
        .iter()
        .map(|q| {
            stardf_arrow_core::layout::Quad64::new(
                q[0], q[1], q[2], q[3], 0,         // t_ns: batch rows carry term ids only
                0,         // vector_clock
                0,         // policy_mask
                [0u8; 32], // inline_digest: unset (invalid per spec, never read)
            )
        })
        .collect();
    let core = core_stream::export_stream(core_stream::CursorProducer::new(slab));
    let priv_ = Box::into_raw(Box::new(StreamPriv { core }));
    // LANE 10 (disclosed minimal edit, re-applied after lane-9 rewrite): mint
    // the stream's ownership tag so the cross-allocator court can verify the
    // release chain claims it before dealloc.
    let stream_ptr = Box::into_raw(Box::new(ArrowArrayStream {
        get_schema: Some(stream_get_schema),
        get_next: Some(stream_get_next),
        get_last_error: Some(stream_get_last_error),
        release: Some(stream_release),
        private_data: priv_ as *mut c_void,
    }));
    crate::ownership::tag_mint(
        stream_ptr as *mut c_void,
        Layout::for_value(unsafe { &*stream_ptr }),
    );
    stream_ptr
}

/// Test shim: invoke the stream's own get_schema callback.
///
/// # Safety
/// `stream` must be a live exported stream; `out` a valid writable schema
/// slot.
pub unsafe fn stream_get_schema_shim(stream: *mut ArrowArrayStream, out: *mut ArrowSchema) -> i32 {
    unsafe { ((*stream).get_schema.unwrap())(stream, out) }
}

/// Test shim: invoke the stream's own get_next callback.
///
/// # Safety
/// `stream` must be a live exported stream; `out` a valid writable array
/// slot.
pub unsafe fn stream_get_next_shim(stream: *mut ArrowArrayStream, out: *mut ArrowArray) -> i32 {
    unsafe { ((*stream).get_next.unwrap())(stream, out) }
}

/// Free a stream previously handed out by [`export_stream`]. Typed no-op on
/// null; runs the release callback so a foreign-held stream is released
/// exactly once, then drops the shell (sole owner of shell disposal).
///
/// # Safety
/// `stream` must be null or a live exported-stream handle freed exactly
/// once.
pub unsafe fn stream_free(stream: *mut ArrowArrayStream) {
    if stream.is_null() {
        return;
    }
    // LANE 10 (re-applied after lane-9 rewrite): ownership-tag claim BEFORE
    // dealloc — tagged pointer discipline. Under `mutant-cross-alloc` the
    // claim is bypassed (untagged foreign free — the defect shape), detected
    // via `ownership::pending()`; the dealloc itself stays layout-correct
    // System dealloc, so the mutant is observable without UB.
    #[cfg(not(feature = "mutant-cross-alloc"))]
    {
        let claimed = crate::ownership::tag_claim(stream as *mut c_void);
        debug_assert!(
            claimed.is_some(),
            "stream_free on a handle with no ownership tag (foreign or double free)"
        );
    }
    unsafe {
        if let Some(release) = (*stream).release {
            release(stream);
        }
        drop(Box::from_raw(stream));
    }
}

// ---------------------------------------------------------------------------
// Tests (Chicago: real stream, real callbacks, assert final state).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StardfBatch;

    fn batch_of(n: usize) -> StardfBatch {
        let mut b = StardfBatch::new();
        for i in 0..n {
            b.quads.push([i as u32 + 1, 2, 3, 4]);
        }
        b
    }

    /// Pull through the real extern "C" fn pointer; returns (flag, length,
    /// released?).
    unsafe fn pull(stream: *mut ArrowArrayStream) -> (i32, i64, bool) {
        let mut array: ArrowArray = core::mem::zeroed();
        let flag = unsafe { ((*stream).get_next.unwrap())(stream, &mut array) };
        let released = array.release.is_some();
        let len = array.length;
        if released {
            unsafe { (array.release.unwrap())(&mut array) };
        }
        (flag, len, released)
    }

    #[test]
    fn batches_of_64_then_spec_eof() {
        for n in [0usize, 1, 63, 64, 65, 129] {
            let stream = export_stream(&batch_of(n));
            let expected = n.div_ceil(64);
            let mut got = 0usize;
            loop {
                // SAFETY: live stream from export_stream.
                let (flag, len, released) = unsafe { pull(stream) };
                assert_eq!(flag, 0, "get_next must never fail on a live stream (n={n})");
                if !released {
                    // Spec end-of-stream: zeroed array, flag 0.
                    assert_eq!(len, 0, "EOS array must be zeroed (n={n})");
                    break;
                }
                got += 1;
                assert!(len > 0, "yielded batch must be non-empty (n={n})");
                assert!(
                    got <= expected,
                    "yielded more than ceil(n/64) batches (n={n})"
                );
            }
            assert_eq!(got, expected, "exactly ceil(n/64) batches (n={n})");
            // SAFETY: live exported stream freed exactly once.
            unsafe { stream_free(stream) };
        }
    }

    #[test]
    fn get_schema_is_extern_c_and_valid() {
        let stream = export_stream(&batch_of(4));
        // SAFETY: zeroed is a valid all-NULL ArrowSchema shell.
        let mut schema: ArrowSchema = unsafe { core::mem::zeroed() };
        // SAFETY: live stream; shim invokes the armed callback.
        let flag = unsafe { stream_get_schema_shim(stream, &mut schema) };
        assert_eq!(flag, 0);
        assert!(schema.release.is_some(), "handed-out schema must be armed");
        assert!(!schema.format.is_null(), "format string must be readable");
        unsafe {
            if let Some(rel) = schema.release {
                rel(&mut schema);
            }
        }
        // SAFETY: live exported stream freed exactly once.
        unsafe { stream_free(stream) };
    }

    #[test]
    fn release_exactly_once_double_is_typed_noop() {
        let stream = export_stream(&batch_of(0));
        // SAFETY: live stream. The court is the double RELEASE CALLBACK: the
        // second invocation must observe the CAS-nulled slot and be a typed
        // no-op, never a double free of private data. The shell is owned by
        // stream_free (sole disposer) and is freed exactly once below.
        unsafe {
            if let Some(release) = (*stream).release {
                release(stream);
                release(stream);
            }
            stream_free(stream);
        }
    }

    #[test]
    fn null_arguments_are_error_flags_not_crashes() {
        // SAFETY: zeroed is a valid all-NULL shell for both structs.
        let mut schema: ArrowSchema = unsafe { core::mem::zeroed() };
        let mut array: ArrowArray = unsafe { core::mem::zeroed() };
        // SAFETY: deliberately null arguments.
        unsafe {
            assert_ne!(stream_get_schema(core::ptr::null_mut(), &mut schema), 0);
            assert_ne!(stream_get_next(core::ptr::null_mut(), &mut array), 0);
            assert_ne!(
                stream_get_schema(core::ptr::null_mut(), core::ptr::null_mut()),
                0
            );
            assert!(stream_get_last_error(core::ptr::null_mut()).is_null());
        }
        // SAFETY: typed no-op on null.
        unsafe { stream_free(core::ptr::null_mut()) };
    }
}
