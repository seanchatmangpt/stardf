//! Consumer courts for `stardf_avec_export_stream` over the C ABI.
//!
//! Contract: `docs/jira/v26.10.31/02-wave-contracts.md` W3; authority:
//! `authority/stardf/stardf_contract.ttl` `fn_avec_export_stream`
//! (downstreamSubstrate cgap). Style model:
//! `/Users/sac/cgap/crates/cgap-ffi/tests/zero_copy.rs` — real FFI (extern
//! "C" declarations bound against the same-process rlib), no mocks, assert
//! final state.
//!
//! ## Courts
//!
//! 1. `get_schema` returns a valid schema (status 0, armed release, readable
//!    format string) on first call.
//! 2. `get_next` yields exactly `ceil(n/64)` batches, then end-of-stream.
//! 3. A partial final batch has the correct row count.
//! 4. Release exactly-once: the second release-callback invocation is a typed
//!    no-op, never a double free.
//! 5. `get_next` after release returns an error flag, not a crash.
//! 6. `get_last_error` is empty (NULL) when no error; the crate-level
//!    `stardf_last_error` serves a non-NULL, readable message after a forced
//!    error.
//!
//! ## Status vs the current `stream_c.rs` (lane 8 implementation)
//!
//! The suite is ALIVE (compiles and runs) but the courts KILL the current
//! implementation: `stream_release` frees the stream SHELL itself
//! (`drop(Box::from_raw(stream))` in stream_c.rs) while `stream_free` also
//! calls release then drops the shell — a double free on every consumer
//! `stardf_stream_free` call (courts 1-4 and 6 die with SIGTRAP in the test
//! process; court 5 passes via the nilled-slot typed-dead arm). This header
//! will be updated when the owner repairs stream_c.rs.
//!
/// Tests serialized: the crate-global LAST_ERROR slot and process-lifetime
/// resources make parallel runs race (same idiom as the model suite's SLOTS).
/// Poison-resilient by design: a failed court must not cascade-poison the
/// other five — each court reports its own verdict.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serial_lock() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

use stardf_arrow_capi as capi;

// LANE 09 (2026-10-31): ArrowArray joined the capi re-export import — the
// test no longer names the cgap git crate (crates.io publish law).
use capi::{ArrowArray, ArrowArrayStream, StardfArena, StardfBatch};

/// Consumer-side view of `struct ArrowSchema` (the capi crate re-exports the
/// spec-fixed core ArrowSchema, and this view keeps the test honest about
/// consumer-side reads: layout is spec-fixed, and `release` sits at the same
/// slot so the handed-out callback can be invoked through this view).
#[repr(C)]
struct ArrowSchemaView {
    format: *const core::ffi::c_char,
    name: *const core::ffi::c_char,
    metadata: *const core::ffi::c_char,
    flags: i64,
    n_children: i64,
    children: *mut core::ffi::c_void,
    dictionary: *mut core::ffi::c_void,
    release: Option<unsafe fn(*mut ArrowSchemaView)>,
    private_data: *mut core::ffi::c_void,
}

// WAIVED(improper_ctypes): these declarations mirror the capi crate's own
// same-process Rust functions (signatures copied from lib.rs); only opaque
// handle pointers cross the boundary, which are ABI-identical regardless of
// the referent's Rust layout. No real C boundary is involved.
#[allow(improper_ctypes)]
extern "C" {
    // Signatures mirror lib.rs exactly (u32 x4 for append_quad).
    fn stardf_arena_new(capacity_bytes: usize) -> *mut StardfArena;
    fn stardf_arena_intern(arena: *mut StardfArena, lexical: *const u8, len: usize) -> u32;
    fn stardf_arena_free(arena: *mut StardfArena);
    fn stardf_batch_append_quad(
        arena: *mut StardfArena, s: u32, p: u32, o: u32, g: u32,
    ) -> *mut StardfBatch;
    fn stardf_avec_export_stream(
        batch: *mut StardfBatch, out_stream: *mut *mut ArrowArrayStream,
    ) -> i32;
    fn stardf_stream_free(stream: *mut ArrowArrayStream);
    fn stardf_last_error() -> *const core::ffi::c_char;
}

const STATUS_OK: i32 = 0;
/// End-of-stream is flag 0 (OK) with a release-less array, per the Arrow C
/// Stream Interface spec; a nonzero flag other than 10 is an error.
const STREAM_EOF_FLAG: i32 = 10;
const QUADS_PER_BATCH: usize = 64;

const fn batches_for(n: usize) -> usize {
    n.div_ceil(QUADS_PER_BATCH)
}

/// Build a real batch of `n` quads through the FFI (arena + intern + appends)
/// and export it as an owned stream handle. Returns (arena, stream).
///
/// # Safety
/// FFI contract per lib.rs; arena and stream are caller-owned.
unsafe fn make_stream(n: usize) -> (*mut StardfArena, *mut ArrowArrayStream) {
    let arena = stardf_arena_new(1 << 20);
    assert!(!arena.is_null(), "arena_new failed: {:?}", err_str());
    // One distinct term is enough; ids start at 1.
    let id = stardf_arena_intern(arena, b"ex:s".as_ptr(), 4);
    assert_eq!(id, 1, "intern must mint id 1");
    // LANE-8 REPAIR (disclosed, compile/test-freeze SLA): the unconditional
    // first append made n=0 carry 1 quad, so batches_for(0)=0 could never
    // match the yielded batch count. n quads means exactly n appends.
    let mut batch: *mut StardfBatch = core::ptr::null_mut();
    for _ in 0..n {
        let b = stardf_batch_append_quad(arena, 1, 1, 1, 1);
        assert!(!b.is_null(), "append failed (n={n}): {:?}", err_str());
        if batch.is_null() {
            batch = b;
        } else {
            assert_eq!(b, batch, "batch handle must be pointer-stable");
        }
    }
    let mut stream: *mut ArrowArrayStream = std::ptr::null_mut();
    let st = stardf_avec_export_stream(batch, &mut stream);
    assert_eq!(st, STATUS_OK, "export_stream failed: {:?}", err_str());
    assert!(!stream.is_null(), "stream handle must be non-null on OK");
    (arena, stream)
}

/// # Safety
/// Message pointer from the crate is valid until the next failing call.
unsafe fn err_str() -> Option<String> {
    let p = stardf_last_error();
    if p.is_null() {
        return None;
    }
    // SAFETY: NUL-terminated message owned by the crate.
    Some(
        unsafe { core::ffi::CStr::from_ptr(p) }
            .to_string_lossy()
            .into_owned(),
    )
}

/// Pull one batch; returns (flag, length, release-armed-at-handout?).
///
/// # Safety
/// `stream` must be live and unreleased.
unsafe fn pull(stream: *mut ArrowArrayStream) -> (i32, i64, bool) {
    let mut array: ArrowArray = core::mem::zeroed();
    let get_next = (*stream).get_next.expect("get_next must be armed");
    let flag = get_next(stream, &mut array);
    let out = (flag, array.length, array.release.is_some());
    // Consumer contract: release every yielded array before the next pull.
    if let Some(rel) = array.release {
        rel(&mut array);
    }
    out
}

#[test]
fn court_1_get_schema_valid_on_first_call() {
    let _g = serial_lock();
    // SAFETY: FFI contract per lib.rs; resources freed below.
    unsafe {
        let (arena, stream) = make_stream(4);
        let get_schema = (*stream)
            .get_schema
            .expect("get_schema fn slot must be armed on a live stream");
        let mut schema: ArrowSchemaView = core::mem::zeroed();
        let flag = get_schema(stream, &mut schema as *mut _ as *mut _);
        assert_eq!(flag, STATUS_OK, "get_schema must return OK on first call");
        assert!(
            schema.release.is_some(),
            "handed-out schema must have an armed release callback"
        );
        assert!(!schema.format.is_null(), "schema format must be readable");
        // Real C-string walk of the format through the raw pointer.
        let mut len = 0usize;
        let fp = schema.format;
        while *fp.add(len) != 0 {
            len += 1;
        }
        assert!(len > 0, "format string must be non-empty");
        // Consumer contract: release the handed-out schema via its callback.
        if let Some(rel) = schema.release {
            rel(&mut schema as *mut _ as *mut _);
        }
        stardf_stream_free(stream);
        stardf_arena_free(arena);
    }
}

#[test]
fn court_2_exact_batch_count_then_eof() {
    let _g = serial_lock();
    // SAFETY: FFI contract per lib.rs; resources freed below.
    unsafe {
        // LANE-8 NOTE (disclosed): n=0 was dropped here — the current C ABI
        // cannot mint an EMPTY batch handle (stardf_batch_append_quad creates
        // the batch on first append), so export for n=0 is NULL_POINTER by
        // contract. The zero-batch/immediate-EOF edge is courted in
        // stream_c.rs's own unit test (n=0 -> immediate spec EOF, PASS).
        for n in [1usize, 63, 64, 65, 128, 129, 200] {
            let (arena, stream) = make_stream(n);
            let mut got = 0usize;
            loop {
                let (flag, len, released) = pull(stream);
                if flag != STATUS_OK {
                    assert_eq!(
                        flag, STREAM_EOF_FLAG,
                        "only EOF (10) may terminate the stream (n={n})"
                    );
                }
                // LANE-2 REPAIR (disclosed): the break was inverted — it
                // fired on the first ARMED (data) array, so `got` counted
                // only EOS pulls and no conforming implementation could
                // reach ceil(n/64). Spec convention (per this file's own
                // header): data batches are release-armed, EOS is the
                // zeroed release-less array with flag 0. Break on EOS.
                if !released || flag != STATUS_OK {
                    break;
                }
                got += 1;
                assert!(len > 0, "yielded batch must be non-empty (n={n})");
                assert!(
                    got <= batches_for(n),
                    "stream yielded more than ceil(n/64) batches (n={n}, got={got})"
                );
            }
            assert_eq!(
                got,
                batches_for(n),
                "exactly ceil(n/64) batches must be yielded before EOF (n={n})"
            );
            // EOF must be stable across repeated pulls.
            let (flag, _, released) = pull(stream);
            assert!(
                flag != STATUS_OK || !released,
                "EOF must be stable across repeated pulls (n={n})"
            );
            stardf_stream_free(stream);
            stardf_arena_free(arena);
        }
    }
}

#[test]
fn court_3_partial_final_batch_length() {
    let _g = serial_lock();
    // SAFETY: FFI contract per lib.rs; resources freed below.
    unsafe {
        // 65 quads: one full batch of 64 + a partial final batch of 1 quad
        // (4 flattened i32 values per quad).
        let n = 65usize;
        let (arena, stream) = make_stream(n);
        let get_next = (*stream).get_next.expect("get_next must be armed");
        let mut array: ArrowArray = core::mem::zeroed();
        let flag = get_next(stream, &mut array);
        assert_eq!(flag, STATUS_OK, "first batch must succeed");
        assert_eq!(array.length, 64, "first batch carries exactly 64 quad rows");
        if let Some(rel) = array.release {
            rel(&mut array);
        }
        let mut final_array: ArrowArray = core::mem::zeroed();
        let flag = get_next(stream, &mut final_array);
        assert_eq!(flag, STATUS_OK, "partial final batch must succeed");
        assert_eq!(
            final_array.length,
            (n % QUADS_PER_BATCH) as i64,
            "partial final batch must carry exactly the leftover quad rows"
        );
        if let Some(rel) = final_array.release {
            rel(&mut final_array);
        }
        stardf_stream_free(stream);
        stardf_arena_free(arena);
    }
}

#[test]
fn court_4_release_exactly_once_second_is_typed_noop() {
    let _g = serial_lock();
    // SAFETY: FFI contract per lib.rs; resources freed below.
    unsafe {
        let (arena, stream) = make_stream(4);
        let release = (*stream).release.expect("release slot must be armed");
        // First release: consumes the private data.
        release(stream);
        assert!(
            (*stream).private_data.is_null(),
            "first release must consume the private data"
        );
        // Second release: typed no-op — must not double-free.
        release(stream);
        assert!(
            (*stream).private_data.is_null(),
            "second release must leave the stream a consumed no-op"
        );
        // Shell disposal: stream_free's internal release re-fires as a no-op,
        // then drops the shell — this line aborting is a double-free defect.
        stardf_stream_free(stream);
        stardf_arena_free(arena);
    }
}

#[test]
fn court_5_use_after_release_returns_error_not_crash() {
    let _g = serial_lock();
    // SAFETY: FFI contract per lib.rs; resources freed below.
    unsafe {
        let (arena, stream) = make_stream(4);
        let release = (*stream).release.expect("release slot must be armed");
        release(stream);
        // Use-after-release must be a TYPED dead state, never a crash:
        // either the vtable slot is nilled (structurally dead) or a call
        // through the surviving slot returns a nonzero error flag.
        match (*stream).get_next {
            None => {} // nilled slot: the release marked the stream dead.
            Some(get_next) => {
                let mut array: ArrowArray = core::mem::zeroed();
                let flag = get_next(stream, &mut array);
                assert_ne!(
                    flag, STATUS_OK,
                    "get_next on a released stream must return an error flag"
                );
            }
        }
        // Shell disposal after the use-after-release probe.
        stardf_stream_free(stream);
        stardf_arena_free(arena);
    }
}

#[test]
fn court_6_last_error_empty_on_success_non_null_after_forced_error() {
    let _g = serial_lock();
    // SAFETY: FFI contract per lib.rs; resources freed below.
    unsafe {
        let (arena, stream) = make_stream(4);
        let get_last_error = (*stream)
            .get_last_error
            .expect("get_last_error slot must be armed");
        let p = get_last_error(stream);
        assert!(
            p.is_null(),
            "stream-level last error must be empty (NULL) on success paths"
        );
        stardf_stream_free(stream);
        stardf_arena_free(arena);

        // Forced error at the crate level: stardf_arena_new(0) is
        // INVALID_ARGUMENT per lib.rs.
        let bad = stardf_arena_new(0);
        assert!(bad.is_null(), "arena_new(0) must return NULL");
        let p = stardf_last_error();
        assert!(
            !p.is_null(),
            "crate-level last error must be non-NULL after a forced error"
        );
        // The message must be a real, readable, non-empty C string.
        let msg = core::ffi::CStr::from_ptr(p);
        assert!(!msg.is_empty(), "error message must be non-empty");
        assert!(!msg.to_bytes().is_empty());
    }
}
