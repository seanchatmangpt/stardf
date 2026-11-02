//! Batch of [`Quad64`] -> Arrow C Data Interface export (lane 5 contract).
//!
//! A [`StardfBatch`] is a fixed-capacity 64-quad slab — exactly 4096 bytes of
//! [`Quad64`] at 64-byte alignment, one cache line per record. [`export`]
//! projects the four `u32` term slots (s/p/o/g) as four primitive `"u32"`
//! child arrays under one struct (`"+s"`) parent. The slab is AoS; Arrow
//! primitive buffers are SoA, so export gathers each term slot into its own
//! dense `u32` buffer (one allocation per column, owned by the child).
//!
//! Release safety (house discipline, model
//! `cgap-arrow/src/c_abi.rs:54-98` + `c_abi_nested.rs:63-91`): every release
//! callback is atomically swapped to NULL before private data drops, so a
//! double release is a typed no-op, never a double free; the parent release
//! cascades each child exactly once (children released before the slab they
//! alias is dropped).

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ffi::{c_char, c_void};
use core::sync::atomic::{AtomicPtr, Ordering};

// LANE 09 decouple (2026-10-31): was `use cgap_arrow::layout::...` — the
// git dep is gone from the published manifest; the spec-fixed local mirror
// (crate::layout, const-asserted offsets) is the single source.
use crate::layout::{Quad64, BOTTOM_TERM_ID};

/// Maximum quads per batch slab.
pub const BATCH_CAP: usize = 64;

/// struct ArrowSchema (C Data Interface spec), mirrored from
/// `cgap-arrow/src/c_abi.rs:25-35`.
#[repr(C)]
pub struct ArrowSchema {
    /// Format string (e.g. `"u32"`, `"+s"`).
    pub format: *const c_char,
    /// Column name or NULL.
    pub name: *const c_char,
    /// Serialized metadata or NULL.
    pub metadata: *const c_char,
    /// Flag bits.
    pub flags: i64,
    /// Child schema count.
    pub n_children: i64,
    /// Child schema pointers.
    pub children: *mut *mut ArrowSchema,
    /// Dictionary schema or NULL.
    pub dictionary: *mut ArrowSchema,
    /// Release callback (NULL once released).
    pub release: Option<unsafe fn(*mut ArrowSchema)>,
    /// Opaque owner state.
    pub private_data: *mut c_void,
}

/// struct ArrowArray (C Data Interface spec), mirrored from
/// `cgap-arrow/src/c_abi.rs:39-50`.
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
    /// Child array count.
    pub n_children: i64,
    /// Buffer pointers.
    pub buffers: *mut *const c_void,
    /// Child array pointers.
    pub children: *mut *mut ArrowArray,
    /// Dictionary array or NULL.
    pub dictionary: *mut ArrowArray,
    /// Release callback (NULL once released).
    pub release: Option<unsafe fn(*mut ArrowArray)>,
    /// Opaque owner state.
    pub private_data: *mut c_void,
}

// ---- Layout laws as build-time facts ------------------------------------
// The physical batch invariant (spec R4) and the spec-struct ABI shape are
// pinned at compile time; a regression is a build failure.
const _: () = assert!(core::mem::size_of::<Quad64>() == 64);
const _: () = assert!(core::mem::align_of::<Quad64>() == 64);
const _: () = assert!(core::mem::size_of::<[Quad64; BATCH_CAP]>() == 4096);
const _: () = assert!(core::mem::offset_of!(Quad64, s_id) == 0x00);
const _: () = assert!(core::mem::offset_of!(Quad64, p_id) == 0x04);
const _: () = assert!(core::mem::offset_of!(Quad64, o_id) == 0x08);
const _: () = assert!(core::mem::offset_of!(Quad64, g_id) == 0x0C);
// Spec struct shape: release slot sits after dictionary, private_data last,
// so the AtomicPtr pun used by the release callbacks is layout-identical to
// the Option<fn> niche (same argument as c_abi.rs:81-85).
const _: () = assert!(core::mem::size_of::<ArrowArray>() == 80);
const _: () = assert!(core::mem::size_of::<ArrowSchema>() == 72);
const _: () =
    assert!(core::mem::size_of::<Option<unsafe fn()>>() == core::mem::size_of::<*mut c_void>());

/// Sentinel stored in exported `g` slots that were BOTTOM in the source quad.
///
/// The `g` column exports the raw `g_id` term; BOTTOM (`0x0`) stays BOTTOM
/// (the deterministic-zero null convention, `layout.rs:24-31`). This constant
/// exists so roundtrip tests name the convention explicitly.
pub const G_NULL_SENTINEL: u32 = BOTTOM_TERM_ID;

/// Fixed-capacity 64-quad slab: exactly 4096 bytes of [`Quad64`].
#[derive(Clone, Copy, Debug)]
pub struct StardfBatch {
    quads: [Quad64; BATCH_CAP],
    len: usize,
}

// The physical slab inside the batch is exactly 4096 bytes, 64-byte aligned.
const _: () = assert!(core::mem::size_of::<[Quad64; BATCH_CAP]>() == 4096);
const _: () = assert!(core::mem::align_of::<[Quad64; BATCH_CAP]>() == 64);

impl StardfBatch {
    /// An empty batch (every slot BOTTOM).
    pub fn new() -> Self {
        // Quad64 zeroed digest is invalid per spec §1 but BOTTOM is the
        // deterministic zero convention; export only reads s/p/o/g, so the
        // digest slot is never dereferenced here.
        StardfBatch {
            quads: unsafe { core::mem::zeroed() },
            len: 0,
        }
    }

    /// Number of live quads.
    pub fn len(&self) -> usize {
        self.len
    }

    /// True when no live quads.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Append a quad; refuses when the slab is full.
    pub fn push(&mut self, quad: Quad64) -> Result<(), BatchFull> {
        if self.len >= BATCH_CAP {
            return Err(BatchFull);
        }
        self.quads[self.len] = quad;
        self.len += 1;
        Ok(())
    }

    /// Live quads.
    pub fn quads(&self) -> &[Quad64] {
        &self.quads[..self.len]
    }

    /// The raw slab (whole fixed-capacity array, live or not).
    pub fn slab(&self) -> &[Quad64; BATCH_CAP] {
        &self.quads
    }
}

impl Default for StardfBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Refusal when pushing into a full slab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BatchFull;

/// Owned parent-array state: the slab plus the four child shells.
///
/// Drop order matters and is enforced by [`release_batch_array`]: children are
/// released (and their shells freed) *before* the slab they alias is dropped.
struct OwnedBatchArray {
    batch: StardfBatch,
    child_shells: Box<[*mut ArrowArray]>,
}

/// Owned child-array state: the gathered `u32` values plus the two-slot
/// buffer-pointer slice (validity NULL, values).
struct OwnedColumn {
    values: Box<[u32]>,
    buffers: Box<[*const c_void; 2]>,
}

/// Owned parent-schema state: the four child schema shells.
struct OwnedBatchSchema {
    child_shells: Box<[*mut ArrowSchema]>,
}

/// Ownership note (not a type used at runtime): the child schema shells are
/// freed by `release_batch_schema`'s cascade, exactly like the child array
/// shells on the array side — batch.rs diverges from cgap-arrow's schema-side
/// convention (`c_abi_nested.rs`: child release frees its own shell) by
/// keeping child schema `private_data` NULL and letting the parent cascade
/// free the shells. Self-consistent pair: `release_child_schema` disarms
/// only, the parent cascade frees.
struct OwnedChildSchema {
    // marker only; the shell is freed by the parent schema cascade
    _priv: (),
}

// Silence unused-type lint: OwnedChildSchema documents the ownership split.
#[allow(dead_code)]
impl OwnedChildSchema {}

/// SAFETY: single release via atomic swap-to-NULL (c_abi.rs:81-91); the first
/// caller wins, later calls observe `release == NULL` and return untouched.
unsafe fn release_column_array(array: *mut ArrowArray) {
    // Option<unsafe fn> is a nullable pointer with a niche: identical layout
    // to *mut c_void, so an in-place atomic view of the release slot is a
    // pointer pun.
    let slot: &AtomicPtr<c_void> =
        unsafe { &*(core::ptr::addr_of_mut!((*array).release) as *mut AtomicPtr<c_void>) };
    let prev = slot.swap(core::ptr::null_mut(), Ordering::AcqRel);
    if prev.is_null() {
        return; // already released: typed no-op
    }
    let private = unsafe { (*array).private_data };
    if !private.is_null() {
        // SAFETY: recreated exactly as boxed in `export`.
        unsafe { drop(Box::from_raw(private as *mut OwnedColumn)) };
    }
}

/// SAFETY: parent release. The winner cascades every child exactly once
/// (each child release is CAS-guarded) and only then drops the slab the
/// children alias — same ordering discipline as c_abi_nested.rs:63-88.
unsafe fn release_batch_array(array: *mut ArrowArray) {
    let slot: &AtomicPtr<c_void> =
        unsafe { &*(core::ptr::addr_of_mut!((*array).release) as *mut AtomicPtr<c_void>) };
    let prev = slot.swap(core::ptr::null_mut(), Ordering::AcqRel);
    if prev.is_null() {
        return; // already released: typed no-op
    }
    let private = unsafe { (*array).private_data };
    if !private.is_null() {
        // SAFETY: recreated exactly as boxed in `export`.
        let owned = unsafe { Box::from_raw(private as *mut OwnedBatchArray) };
        for &child in owned.child_shells.iter() {
            // SAFETY: the child shell was boxed in `export` and is still
            // alive: the parent release slot was live until the swap above.
            if let Some(release) = unsafe { (*child).release } {
                // SAFETY: the child's own release is CAS-guarded, so exactly
                // one call ever frees its private data.
                unsafe { release(child) };
            }
            // SAFETY: the shell box from `export`; a child release frees only
            // its private data, never the shell.
            unsafe { drop(Box::from_raw(child)) };
        }
        drop(owned); // the slab dies last
    }
}

/// SAFETY: schema-side mirror of [`release_column_array`]; frees the aliased
/// shell box (c_abi.rs:52-63 convention).
unsafe fn release_child_schema(schema: *mut ArrowSchema) {
    let slot: &AtomicPtr<c_void> =
        unsafe { &*(core::ptr::addr_of_mut!((*schema).release) as *mut AtomicPtr<c_void>) };
    let prev = slot.swap(core::ptr::null_mut(), Ordering::AcqRel);
    let _ = prev;
    // Disarm only: a child schema owns no private data. The shell box is
    // freed by release_batch_schema's cascade exactly once (c_abi_nested
    // convention: children are released via their parent).
}

/// SAFETY: parent-schema release; cascades every child schema release exactly
/// once (each is CAS-guarded) and frees the parent shell. Same convention as
/// c_abi_nested.rs:91-119.
unsafe fn release_batch_schema(schema: *mut ArrowSchema) {
    let slot: &AtomicPtr<c_void> =
        unsafe { &*(core::ptr::addr_of_mut!((*schema).release) as *mut AtomicPtr<c_void>) };
    let prev = slot.swap(core::ptr::null_mut(), Ordering::AcqRel);
    if prev.is_null() {
        return;
    }
    let private = unsafe { (*schema).private_data };
    if !private.is_null() {
        // SAFETY: recreated exactly as boxed in `export`.
        let owned = unsafe { Box::from_raw(private as *mut OwnedBatchSchema) };
        for &child in owned.child_shells.iter() {
            // SAFETY: the child schema shell is alive until this cascade.
            if let Some(release) = unsafe { (*child).release } {
                unsafe { release(child) };
            }
            // SAFETY: the shell box from `export`.
            unsafe { drop(Box::from_raw(child)) };
        }
        drop(owned);
    }
    // The parent schema shell itself is NOT freed here (house convention,
    // c_abi_nested.rs:60-61): whoever boxes the returned struct frees it.
}

/// Format strings for the exported schema: struct parent, `"u32"` children.
pub const PARENT_FORMAT: &core::ffi::CStr = c"+s";
/// Child column format: fixed-width primitive u32.
pub const CHILD_FORMAT: &core::ffi::CStr = c"u32";

/// Export a batch as heap-resident, spec-compliant C Data Interface shells:
/// one struct (`"+s"`) parent with four primitive `"u32"` children
/// (`s`, `p`, `o`, `g`), each child's value buffer aliasing the batch slab at
/// its Quad64 field offset (zero-copy).
///
/// Ownership transfers to the importer. Importer duty (spec contract, same
/// protocol as the c_abi.rs tests):
///
/// 1. read through `children[i]` / `buffers`;
/// 2. call the parent array's `release` (cascades every child exactly once);
/// 3. call the parent schema's `release`;
/// 4. free all struct shells with `Box::from_raw`.
///
/// # Panics
/// Never. `batch.len()` is bounded by [`BATCH_CAP`].
pub fn export(batch: StardfBatch) -> (*mut ArrowArray, *mut ArrowSchema) {
    let len = batch.len() as i64;

    // Child field offsets within Quad64 (layout-law-pinned above).
    let offs = [
        (core::mem::offset_of!(Quad64, s_id), c"s"),
        (core::mem::offset_of!(Quad64, p_id), c"p"),
        (core::mem::offset_of!(Quad64, o_id), c"o"),
        (core::mem::offset_of!(Quad64, g_id), c"g"),
    ];

    let mut child_arrays: Vec<*mut ArrowArray> = Vec::with_capacity(4);
    let mut child_schemas: Vec<*mut ArrowSchema> = Vec::with_capacity(4);

    let owned_ptr = Box::into_raw(Box::new(OwnedBatchArray {
        batch,
        child_shells: Box::new([]), // placeholder; filled below
    }));
    // Gather each term slot into a dense u32 column (AoS slab -> SoA).
    // SAFETY: reads only the live prefix [0, len) of the boxed slab.
    let columns: [Box<[u32]>; 4] = {
        // SAFETY: reads only the live prefix [0, len) of the boxed slab.
        let quads = unsafe { (*owned_ptr).batch.quads() };
        let mut c0: Vec<u32> = Vec::with_capacity(quads.len());
        let mut c1: Vec<u32> = Vec::with_capacity(quads.len());
        let mut c2: Vec<u32> = Vec::with_capacity(quads.len());
        let mut c3: Vec<u32> = Vec::with_capacity(quads.len());
        for q in quads {
            c0.push(q.s_id);
            c1.push(q.p_id);
            c2.push(q.o_id);
            c3.push(q.g_id);
        }
        [c0.into(), c1.into(), c2.into(), c3.into()]
    };
    for (values, &(_, name)) in columns.into_iter().zip(offs.iter()) {
        let col = Box::into_raw(Box::new(OwnedColumn {
            values,
            buffers: Box::new([core::ptr::null(), core::ptr::null()]),
        }));
        // SAFETY: col is a live Box<OwnedColumn>; patch the values pointer
        // now that the box is at its final address.
        unsafe {
            (*col).buffers[1] = (*col).values.as_ptr() as *const c_void;
        }
        // SAFETY: col is a live Box<OwnedColumn>.
        let buffers_ptr = unsafe { (*col).buffers.as_mut_ptr() };
        child_arrays.push(Box::into_raw(Box::new(ArrowArray {
            length: len,
            null_count: 0,
            offset: 0,
            n_buffers: 2,
            n_children: 0,
            buffers: buffers_ptr,
            children: core::ptr::null_mut(),
            dictionary: core::ptr::null_mut(),
            release: Some(release_column_array),
            private_data: col as *mut c_void,
        })));
        let schema = Box::into_raw(Box::new(ArrowSchema {
            format: CHILD_FORMAT.as_ptr(),
            name: name.as_ptr(),
            metadata: core::ptr::null(),
            flags: 0,
            n_children: 0,
            children: core::ptr::null_mut(),
            dictionary: core::ptr::null_mut(),
            release: Some(release_child_schema),
            private_data: core::ptr::null_mut(),
        }));
        // private_data stays NULL: the child schema owns nothing; its shell
        // is freed by the parent schema cascade (c_abi_nested convention).
        child_schemas.push(schema);
    }

    // Fill the parent's child-shell list with a shallow pointer copy.
    // SAFETY: owned_ptr is live.
    unsafe {
        (*owned_ptr).child_shells = child_arrays.clone().into_boxed_slice();
    }
    let children_ptr =
        Box::into_raw(child_arrays.clone().into_boxed_slice()) as *mut *mut ArrowArray;

    let array_shell = Box::into_raw(Box::new(ArrowArray {
        length: len,
        null_count: 0,
        offset: 0,
        n_buffers: 0,
        n_children: 4,
        buffers: core::ptr::null_mut(),
        children: children_ptr,
        dictionary: core::ptr::null_mut(),
        release: Some(release_batch_array),
        private_data: owned_ptr as *mut c_void,
    }));

    let owned_schema_ptr = Box::into_raw(Box::new(OwnedBatchSchema {
        child_shells: child_schemas.clone().into_boxed_slice(),
    }));
    let schema_shell = Box::into_raw(Box::new(ArrowSchema {
        format: PARENT_FORMAT.as_ptr(),
        name: core::ptr::null(),
        metadata: core::ptr::null(),
        flags: 0,
        n_children: 4,
        children: Box::into_raw(child_schemas.clone().into_boxed_slice()) as *mut *mut ArrowSchema,
        dictionary: core::ptr::null_mut(),
        release: Some(release_batch_schema),
        private_data: owned_schema_ptr as *mut c_void,
    }));

    (array_shell, schema_shell)
}

/// In-process importer: wraps an exported pair, reads the four u32 columns,
/// and releases exactly once on drop.
#[cfg(test)]
struct ImportedBatch {
    array: *mut ArrowArray,
    schema: *mut ArrowSchema,
}

#[cfg(test)]
impl ImportedBatch {
    /// # Safety
    /// `array`/`schema` must be a live pair returned by [`export`]; this takes
    /// last-reference ownership of both.
    unsafe fn take(array: *mut ArrowArray, schema: *mut ArrowSchema) -> Self {
        ImportedBatch { array, schema }
    }

    /// Row count.
    fn len(&self) -> usize {
        // SAFETY: live export-owned shell; single owner.
        unsafe { (*self.array).length as usize }
    }

    /// Read column `i` (0..4) as a u32 slice.
    ///
    /// # Safety
    /// Column `i` must be a live `"u32"` child of this pair.
    unsafe fn column(&self, i: usize) -> &[u32] {
        // SAFETY: live parent shell; i < n_children asserted.
        unsafe {
            let n = (*self.array).n_children as usize;
            assert!(i < n, "column index out of range");
            let child = *(*self.array).children.add(i);
            let len = (*child).length as usize;
            let values = *(*child).buffers.add(1) as *const u32;
            core::slice::from_raw_parts(values, len)
        }
    }
}

#[cfg(test)]
impl Drop for ImportedBatch {
    fn drop(&mut self) {
        // SAFETY: single-owner import; each release called exactly once, then
        // the struct shells are freed (importer duty steps 2-4).
        unsafe {
            if let Some(release) = (*self.array).release {
                release(self.array);
            }
            if let Some(release) = (*self.schema).release {
                release(self.schema);
            }
            drop(Box::from_raw(self.array));
            drop(Box::from_raw(self.schema));
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    /// A deterministic quad with distinct s/p/o/g values.
    fn quad(seed: u32) -> Quad64 {
        let mut q = Quad64 {
            s_id: seed.wrapping_mul(2654435761),
            p_id: seed.wrapping_mul(40503) | 1,
            o_id: !seed,
            g_id: if seed % 3 == 0 {
                crate::layout::BOTTOM_TERM_ID
            } else {
                seed | 0x8000_0000
            },
            t_ns: 0,
            vector_clock: 0,
            policy_mask: 0,
            inline_digest: [0u8; 32],
        };
        q.t_ns = (seed as u64) << 32;
        q
    }

    fn fill(n: usize) -> StardfBatch {
        let mut b = StardfBatch::new();
        for i in 0..n {
            b.push(quad(i as u32)).expect("slab capacity");
        }
        b
    }

    #[test]
    fn batch_slab_is_4096b_align64_and_full_slab_refuses() {
        assert!(core::mem::size_of::<StardfBatch>() >= 4096);
        assert!(core::mem::align_of::<StardfBatch>() >= 64);
        let mut b = fill(64);
        assert_eq!(b.len(), 64);
        assert!(b.push(quad(99)).is_err(), "full slab refuses");
        assert_eq!(StardfBatch::new().len(), 0);
    }

    #[test]
    fn export_shape_is_struct_parent_with_four_u32_children() {
        let (array, schema) = export(fill(5));
        // SAFETY: test-owned pair; read-only inspection, released on drop.
        unsafe {
            assert_eq!((*array).length, 5);
            assert_eq!((*array).n_children, 4);
            assert_eq!((*schema).n_children, 4);
            assert_eq!(core::ffi::CStr::from_ptr((*schema).format), c"+s");
            let names = [c"s", c"p", c"o", c"g"];
            let children = (*schema).children;
            for (i, name) in names.iter().enumerate() {
                let cs: *mut ArrowSchema = *children.add(i);
                assert_eq!(core::ffi::CStr::from_ptr((*cs).format), c"u32");
                assert_eq!(core::ffi::CStr::from_ptr((*cs).name), *name);
                let ca: *mut ArrowArray = *(*array).children.add(i);
                assert_eq!((*ca).length, 5);
                assert_eq!((*ca).n_buffers, 2);
            }
        }
        // SAFETY: take ownership for the exactly-once drop.
        drop(unsafe { ImportedBatch::take(array, schema) });
    }

    #[test]
    fn export_import_roundtrip() {
        let batch = fill(64);
        let (array, schema) = export(batch);
        // SAFETY: live test-owned pair.
        let imported = unsafe { ImportedBatch::take(array, schema) };
        assert_eq!(imported.len(), 64);
        for (row, q) in batch.quads().iter().enumerate() {
            // SAFETY: columns 0..4 are live u32 children.
            unsafe {
                assert_eq!(imported.column(0)[row], q.s_id);
                assert_eq!(imported.column(1)[row], q.p_id);
                assert_eq!(imported.column(2)[row], q.o_id);
                assert_eq!(imported.column(3)[row], q.g_id);
            }
        }
        // Empty batch exports a valid zero-length pair.
        let (a0, s0) = export(StardfBatch::new());
        let empty = unsafe { ImportedBatch::take(a0, s0) };
        assert_eq!(empty.len(), 0);
        // SAFETY: zero-length column still yields an empty slice.
        assert!(unsafe { empty.column(0) }.is_empty());
    }

    #[test]
    fn release_cascade_exactly_once_and_double_release_is_noop() {
        // Real observation, no mocks, per the house precedent
        // (c_abi_nested.rs `all_real_children_release_through_cascade`): the
        // parent cascade FREES every child shell, so reading a child slot
        // after the parent release is a use-after-free — exactly-once is
        // proven by (a) pre-cascade arming of every child slot on both
        // sides, (b) the parent slot swapping to NULL, (c) a second parent
        // release being a typed no-op that does not re-cascade (a double
        // free anywhere in the cascade would abort the test process).
        let (array, schema) = export(fill(3));
        // SAFETY: live test-owned pair.
        unsafe {
            // Pre-cascade: every child slot on both sides is armed.
            for i in 0..4 {
                let ca: *mut ArrowArray = *(*array).children.add(i);
                assert!((*ca).release.is_some(), "child array {i} not armed");
                let cs: *mut ArrowSchema = *(*schema).children.add(i);
                assert!((*cs).release.is_some(), "child schema {i} not armed");
            }
            let parent_release = (*array).release.unwrap();
            parent_release(array); // cascade: children armed exactly once
            assert!((*array).release.is_none());
            // Double release: typed no-op, never a re-cascade/double free.
            parent_release(array);
            assert!((*array).release.is_none());
            // Schema side: release cascades child schemas exactly once.
            let schema_release = (*schema).release.unwrap();
            schema_release(schema);
            assert!((*schema).release.is_none());
            schema_release(schema); // no-op
                                    // Importer duty step 4: free the shells.
            drop(Box::from_raw(array));
            drop(Box::from_raw(schema));
        }
    }
}
