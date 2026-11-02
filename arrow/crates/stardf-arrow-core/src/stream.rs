//! Arrow C Stream Interface (`ArrowArrayStream`) — structural mirror of the
//! cgap-arrow release discipline, zero cgap dependency.
//!
//! Spec flag convention: `0` = OK, any nonzero = error, `10` (ENOENT) = end
//! of stream. Release safety (same FMEA as `cgap-arrow/src/c_abi.rs:70-127`):
//! the release callback atomically CAS-swaps the release slot to NULL *before*
//! dropping private data, so a double release is a typed no-op, never a double
//! free.

use crate::layout::{Quad64, QUAD_SIZE};
extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ffi::{c_char, c_int, c_void};
use core::sync::atomic::{AtomicPtr, Ordering};

// ---------------------------------------------------------------------------
// C Data Interface value structs (mirror of cgap-arrow c_abi.rs — duplicated
// here only because this crate has no cgap dependency; layout is spec-fixed).
// ---------------------------------------------------------------------------

/// struct ArrowSchema (C Data Interface spec).
#[repr(C)]
pub struct ArrowSchema {
    /// Format string per the C Data Interface spec.
    pub format: *const c_char,
    /// Column name or NULL.
    pub name: *const c_char,
    /// Serialized metadata or NULL.
    pub metadata: *const c_char,
    /// Flag bits.
    pub flags: i64,
    /// Child count.
    pub n_children: i64,
    /// Child pointers.
    pub children: *mut *mut ArrowSchema,
    /// Dictionary, or NULL.
    pub dictionary: *mut ArrowSchema,
    /// Release callback (NULL once released).
    pub release: Option<unsafe fn(*mut ArrowSchema)>,
    /// Opaque owner state.
    pub private_data: *mut c_void,
}

/// struct ArrowArray (C Data Interface spec).
#[repr(C)]
pub struct ArrowArray {
    /// Logical row count (including offset).
    pub length: i64,
    /// Number of null rows (-1 when not computed).
    pub null_count: i64,
    /// Logical offset into the buffers.
    pub offset: i64,
    /// Buffer pointer count.
    pub n_buffers: i64,
    /// Child count.
    pub n_children: i64,
    /// Buffer pointers.
    pub buffers: *mut *const c_void,
    /// Child pointers.
    pub children: *mut *mut ArrowArray,
    /// Dictionary, or NULL.
    pub dictionary: *mut ArrowArray,
    /// Release callback (NULL once released).
    pub release: Option<unsafe fn(*mut ArrowArray)>,
    /// Opaque owner state.
    pub private_data: *mut c_void,
}

/// `get_schema` / `get_next` returned OK.
pub const STREAM_OK: c_int = 0;
/// `get_next` returned end-of-stream (ENOENT, per the C Stream Interface
/// spec). Internal flag only: the capi bridge (`stardf-arrow-capi
/// stream_c.rs`) translates this into the consumer-facing end-of-stream —
/// return flag `0` with a zeroed, release-less `ArrowArray` — before it
/// crosses the C ABI.
pub const STREAM_EOF: c_int = 10;
/// Generic error flag (any nonzero other than 10 per spec).
pub const STREAM_ERR: c_int = 1;

// ---------------------------------------------------------------------------
// Fixed batch shape.
// ---------------------------------------------------------------------------

/// Fixed batch shape: 64 quads per batch.
pub const QUADS_PER_BATCH: usize = 64;
/// 64 quads x 64 bytes = 4096 bytes per batch.
pub const BATCH_BYTES: usize = QUADS_PER_BATCH * QUAD_SIZE;

const _: () = assert!(BATCH_BYTES == 4096);

// ---------------------------------------------------------------------------
// Pure-Rust producer side.
// ---------------------------------------------------------------------------

/// Pure-Rust stream abstraction: yields owned batches of quads.
pub trait StardfStream {
    /// Pull the next batch; `None` = end of stream.
    fn next_batch(&mut self) -> Option<&[Quad64]>;
    /// Last error on this stream, if any.
    fn last_error(&self) -> Option<&'static str> {
        None
    }
}

/// Cursor-driven producer over an owned quad slab: slices it into fixed
/// [`QUADS_PER_BATCH`] batches.
pub struct CursorProducer {
    slab: Vec<Quad64>,
    cursor: usize,
}

impl CursorProducer {
    /// Wrap a quad slab. Fixed-batch callers should pass a slab whose length
    /// is a multiple of [`QUADS_PER_BATCH`].
    pub fn new(slab: Vec<Quad64>) -> Self {
        Self { slab, cursor: 0 }
    }

    /// Number of full batches this producer still holds.
    pub fn remaining_batches(&self) -> usize {
        self.slab.len().saturating_sub(self.cursor) / QUADS_PER_BATCH
    }
}

impl StardfStream for CursorProducer {
    fn next_batch(&mut self) -> Option<&[Quad64]> {
        if self.cursor >= self.slab.len() {
            return None;
        }
        let end = core::cmp::min(self.cursor + QUADS_PER_BATCH, self.slab.len());
        let batch = &self.slab[self.cursor..end];
        self.cursor = end;
        Some(batch)
    }
}

// ---------------------------------------------------------------------------
// C ABI side.
// ---------------------------------------------------------------------------

/// Private state boxed behind `ArrowArrayStream.private_data`.
struct StreamPrivate {
    producer: CursorProducer,
}

/// SAFETY: first-release winner swaps the slot to NULL before dropping the
/// boxed private data; later calls observe NULL and return (typed no-op).
unsafe fn release_stream(stream: *mut ArrowArrayStream) {
    if stream.is_null() {
        return;
    }
    // Option<unsafe fn> is a nullable pointer with a niche; AtomicPtr<c_void>
    // is repr(transparent) over *mut c_void — an in-place atomic view of the
    // release slot is a pointer pun (same as cgap-arrow c_abi.rs:70-77).
    let slot: &AtomicPtr<c_void> =
        unsafe { &*(core::ptr::addr_of_mut!((*stream).release) as *mut AtomicPtr<c_void>) };
    let prev = slot.swap(core::ptr::null_mut(), Ordering::AcqRel);
    if prev.is_null() {
        return;
    }
    let private = unsafe { (*stream).private_data };
    if !private.is_null() {
        drop(unsafe { Box::from_raw(private as *mut StreamPrivate) });
    }
}

/// The exported schema shell: format string is a `'static` literal, no
/// children/dictionary, so release only needs to mark the slot released.
static QUAD_SCHEMA_FORMAT: &[u8] = b"w:64\0";

/// SAFETY: `out` was filled by [`get_schema`] (or is already released).
unsafe fn release_schema(schema: *mut ArrowSchema) {
    let slot: &AtomicPtr<c_void> =
        unsafe { &*(core::ptr::addr_of_mut!((*schema).release) as *mut AtomicPtr<c_void>) };
    slot.swap(core::ptr::null_mut(), Ordering::AcqRel);
}

/// SAFETY: `stream` is a live, unreleased `ArrowArrayStream` (spec contract);
/// `out` is a caller-provided destination struct.
unsafe fn get_schema(stream: *mut ArrowArrayStream, out: *mut ArrowSchema) -> c_int {
    if stream.is_null() || out.is_null() || unsafe { (*stream).release.is_none() } {
        return STREAM_ERR;
    }
    unsafe {
        *out = ArrowSchema {
            format: QUAD_SCHEMA_FORMAT.as_ptr() as *const c_char,
            name: core::ptr::null(),
            metadata: core::ptr::null(),
            flags: 0,
            n_children: 0,
            children: core::ptr::null_mut(),
            dictionary: core::ptr::null_mut(),
            release: Some(release_schema),
            private_data: core::ptr::null_mut(),
        };
    }
    STREAM_OK
}

/// Owner tuple boxed into the array's private_data: the boxed quad slice plus
/// the boxed buffers array it points into.
struct PackagedBatch {
    batch: *mut [Quad64],
    buffers: *mut [*const c_void],
}

/// SAFETY: first-release winner; drops the boxed batch + buffers exactly once.
unsafe fn release_array(array: *mut ArrowArray) {
    if array.is_null() {
        return;
    }
    let slot: &AtomicPtr<c_void> =
        unsafe { &*(core::ptr::addr_of_mut!((*array).release) as *mut AtomicPtr<c_void>) };
    let prev = slot.swap(core::ptr::null_mut(), Ordering::AcqRel);
    if prev.is_null() {
        return;
    }
    let private = unsafe { (*array).private_data as *mut PackagedBatch };
    if !private.is_null() {
        let packaged = unsafe { Box::from_raw(private) };
        if !packaged.batch.is_null() {
            drop(unsafe { Box::from_raw(packaged.batch) });
        }
        if !packaged.buffers.is_null() {
            drop(unsafe { Box::from_raw(packaged.buffers) });
        }
    }
}

/// SAFETY: `stream` is a live, unreleased `ArrowArrayStream` (spec contract);
/// `out` is a caller-provided destination struct.
unsafe fn get_next(stream: *mut ArrowArrayStream, out: *mut ArrowArray) -> c_int {
    if stream.is_null() || out.is_null() || unsafe { (*stream).release.is_none() } {
        return STREAM_ERR;
    }
    let private = unsafe { (*stream).private_data as *mut StreamPrivate };
    if private.is_null() {
        return STREAM_ERR;
    }
    // SAFETY: private is a live Box<StreamPrivate>; sole owner is the stream.
    let producer = unsafe { &mut (*private).producer };
    match producer.next_batch() {
        Some(batch) => {
            let batch: *mut [Quad64] = Box::into_raw(Box::from(batch));
            let buffers = Box::into_raw(Box::new([core::ptr::null(), batch.cast::<c_void>()]))
                as *mut [*const c_void];
            // SAFETY: batch is a live Box<[Quad64]>.
            let len = batch.len();
            unsafe {
                *out = ArrowArray {
                    length: len as i64,
                    null_count: 0,
                    offset: 0,
                    n_buffers: 2,
                    n_children: 0,
                    buffers: buffers as *mut *const c_void,
                    children: core::ptr::null_mut(),
                    dictionary: core::ptr::null_mut(),
                    release: Some(release_array),
                    private_data: Box::into_raw(Box::new(PackagedBatch { batch, buffers }))
                        as *mut c_void,
                };
            }
            STREAM_OK
        }
        None => STREAM_EOF,
    }
}

/// SAFETY: `stream` is a live, unreleased `ArrowArrayStream` (spec contract).
/// The returned pointer is valid until the next call on this stream; it is
/// NULL when no error is pending (empty on success paths, per contract).
unsafe fn get_last_error(stream: *mut ArrowArrayStream) -> *const c_char {
    if stream.is_null() || unsafe { (*stream).release.is_none() } {
        return core::ptr::null();
    }
    // CursorProducer cannot fail: success paths carry no error string, so the
    // empty-on-success contract is satisfied by NULL.
    core::ptr::null()
}

/// struct ArrowArrayStream (C Stream Interface spec).
#[repr(C)]
pub struct ArrowArrayStream {
    /// Schema callback per the C Stream Interface spec.
    pub get_schema: unsafe fn(*mut ArrowArrayStream, *mut ArrowSchema) -> c_int,
    /// Next-batch callback per the C Stream Interface spec.
    pub get_next: unsafe fn(*mut ArrowArrayStream, *mut ArrowArray) -> c_int,
    /// Last-error callback per the C Stream Interface spec.
    pub get_last_error: unsafe fn(*mut ArrowArrayStream) -> *const c_char,
    /// Release callback (NULL once released).
    pub release: Option<unsafe fn(*mut ArrowArrayStream)>,
    /// Opaque owner state.
    pub private_data: *mut c_void,
}

/// Export a pure-Rust [`CursorProducer`] as a C `ArrowArrayStream`.
///
/// Ownership transfers to the consumer: when done it calls `release` (spec
/// contract), which frees the boxed private data exactly once.
pub fn export_stream(producer: CursorProducer) -> ArrowArrayStream {
    let private = Box::into_raw(Box::new(StreamPrivate { producer }));
    ArrowArrayStream {
        get_schema,
        get_next,
        get_last_error,
        release: Some(release_stream),
        private_data: private as *mut c_void,
    }
}

// ---------------------------------------------------------------------------
// Tests (Chicago: real struct, real calls, assert final state).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_slab(n: usize) -> Vec<Quad64> {
        (0..n)
            .map(|i| Quad64 {
                s_id: i as u32,
                p_id: 0,
                o_id: i as u32,
                g_id: 0,
                t_ns: i as u64,
                vector_clock: 1,
                policy_mask: 0,
                inline_digest: [0u8; 32],
            })
            .collect()
    }

    /// Pull one batch through the real fn pointer; returns (flag, length).
    /// Releases the produced array per the spec consumer contract.
    unsafe fn pull(stream: *mut ArrowArrayStream) -> (c_int, i64) {
        let mut array: ArrowArray = core::mem::zeroed();
        let flag = unsafe { ((*stream).get_next)(stream, &mut array) };
        let len = if flag == STREAM_OK { array.length } else { 0 };
        if array.release.is_some() {
            unsafe { (array.release.unwrap())(&mut array) };
        }
        (flag, len)
    }

    #[test]
    fn get_next_returns_64_then_eof() {
        let mut stream = export_stream(CursorProducer::new(test_slab(64)));
        let s = &mut stream as *mut ArrowArrayStream;
        let (flag, len) = unsafe { pull(s) };
        assert_eq!(flag, STREAM_OK);
        assert_eq!(len, 64);
        let (flag, _) = unsafe { pull(s) };
        assert_eq!(flag, STREAM_EOF);
        // EOF is stable across repeated pulls.
        let (flag, _) = unsafe { pull(s) };
        assert_eq!(flag, STREAM_EOF);
        unsafe { release_stream(s) };
    }

    #[test]
    fn release_exactly_once_double_is_typed_noop() {
        let mut stream = export_stream(CursorProducer::new(test_slab(0)));
        let s = &mut stream as *mut ArrowArrayStream;
        unsafe { release_stream(s) };
        // Second release must be a no-op, not a double free.
        unsafe { release_stream(s) };
    }

    #[test]
    fn get_last_error_empty_on_success() {
        let mut stream = export_stream(CursorProducer::new(test_slab(64)));
        let s = &mut stream as *mut ArrowArrayStream;
        let err = unsafe { ((*s).get_last_error)(s) };
        assert!(err.is_null());
        let (flag, _) = unsafe { pull(s) };
        assert_eq!(flag, STREAM_OK);
        let err = unsafe { ((*s).get_last_error)(s) };
        assert!(err.is_null());
        unsafe { release_stream(s) };
    }

    #[test]
    fn batch_bytes_is_4096() {
        assert_eq!(
            QUADS_PER_BATCH * core::mem::size_of::<Quad64>(),
            BATCH_BYTES
        );
        assert_eq!(BATCH_BYTES, 4096);
    }
}
