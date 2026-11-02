//! Court 1 (R2 falsifier, LANE 10) — allocation-side identity over the
//! stardf-arrow-capi membrane.
//!
//! Lifecycle under court: mint (arena_new → intern → append_quad) →
//! export (stardf_avec_export_stream) → consume (get_schema/get_next,
//! releasing the handed-out array) → release (stream_free + arena_free).
//!
//! ## Why a test-local counting allocator (Chicago, real collaborators)
//!
//! The cgap-audit CountingAlloc counts allocs only — dealloc is a bare
//! pass-through (cgap/crates/audit/src/alloc_counter.rs :65-67, "frees are
//! not allocations"), so it cannot prove the FREE side of identity. This
//! court mints its own #[global_allocator] counting alloc AND dealloc
//! separately.
//!
//! ## Attribution: pointer-set, not bare counters
//!
//! A bare process-global alloc==dealloc equality is NOT a lawful identity
//! assertion: lazy frees of pre-window allocations (std/thread teardown,
//! sibling tests) pollute it — witnessed during this lane as ±1 deltas that
//! flip direction run-to-run. The court therefore attributes each free to
//! the window that minted it: allocations made while a window is open are
//! recorded in a fixed allocation-free pointer table; a free counts toward
//! the window only when its pointer is found in the table. Identity law:
//!
//! 1. mint allocs > 0 (not vacuous — the path really allocates)
//! 2. release window frees ALL minted pointers: freed == minted (every
//!    byte allocated inside stardf's allocator was freed inside it)
//! 3. table empty after the window (no window allocation survives)
//!
//! ## Flip contract (a court that cannot flip is FAIL)
//!
//! Under `--features mutant-cross-alloc` the release chains bypass tag
//! claims (see cross_allocator_court.rs): the export leaks by construction,
//! so freed < minted at window close — the identity law detects the mutant.
//! A clean build passes; the mutant build must FAIL here (positive control
//! documented; the primary detection court is cross_allocator_court.rs,
//! which asserts detection without failing the binary). This file asserts
//! the honest path on clean builds only; under the mutant feature the
//! identity test is compiled out (its law is inverted in
//! cross_allocator_court.rs) to keep each build's courts single-valued.
//!
//! Single-threaded by construction: the tracking window is process-global,
//! so lifecycle tests serialize on LIFECYCLE (SLOTS mutex idiom,
//! cgap/crates/cgap-ffi/tests/alloc_courts.rs :30); warm-up runs OUTSIDE
//! the window (:73-97 idiom).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
use std::sync::Mutex;

use stardf_arrow_capi::{
    stardf_arena_free, stardf_arena_intern, stardf_arena_new, stardf_batch_append_quad,
    stardf_avec_export_stream, stardf_stream_free, ArrowArray, ArrowArrayStream, ArrowSchema,
    StardfArena, StardfBatch,
};

/// Process-global lifecycle gate (SLOTS idiom).
static LIFECYCLE: Mutex<()> = Mutex::new(());

static WINDOW: AtomicBool = AtomicBool::new(false);
static MINTED: AtomicUsize = AtomicUsize::new(0);
static FREED: AtomicUsize = AtomicUsize::new(0);

/// Fixed allocation-free live-pointer table with linear scan.
const SLOTS_N: usize = 1024;
static LIVE: [AtomicUsize; SLOTS_N] = {
    #[allow(clippy::declare_interior_mutable_const)]
    const Z: AtomicUsize = AtomicUsize::new(0);
    [Z; SLOTS_N]
};
static SIZES: [AtomicUsize; SLOTS_N] = {
    #[allow(clippy::declare_interior_mutable_const)]
    const Z: AtomicUsize = AtomicUsize::new(0);
    [Z; SLOTS_N]
};
const EMPTY: usize = 0;

/// Open a tracking window (caller holds LIFECYCLE).
fn window_open() {
    for s in LIVE.iter() {
        s.store(EMPTY, Relaxed);
    }
    MINTED.store(0, Relaxed);
    FREED.store(0, Relaxed);
    WINDOW.store(true, Relaxed);
}

/// Close the window; returns (minted, freed, surviving).
fn window_close() -> (usize, usize, Vec<(usize, usize)>) {
    WINDOW.store(false, Relaxed);
    let minted = MINTED.load(Relaxed);
    let freed = FREED.load(Relaxed);
    let mut surviving: Vec<(usize, usize)> = Vec::new();
    for i in 0..SLOTS_N {
        let p = LIVE[i].load(Relaxed);
        if p != EMPTY {
            surviving.push((p, SIZES[i].load(Relaxed)));
        }
    }
    for i in 0..SLOTS_N {
        LIVE[i].store(EMPTY, Relaxed);
        SIZES[i].store(0, Relaxed);
    }
    (minted, freed, surviving)
}

/// Record an allocation (with its size) in the open window.
fn track(ptr: *mut u8, size: usize) {
    if !WINDOW.load(Relaxed) || (ptr as usize) <= EMPTY {
        return;
    }
    for i in 0..SLOTS_N {
        match LIVE[i].compare_exchange(EMPTY, ptr as usize, Relaxed, Relaxed) {
            Ok(_) => {
                SIZES[i].store(size, Relaxed);
                MINTED.fetch_add(1, Relaxed);
                return;
            }
            Err(cur) if cur == ptr as usize => return,
            Err(_) => continue,
        }
    }
}

/// Attribute a free to the window when its pointer is live-tracked.
fn untrack(ptr: *mut u8) {
    let p = ptr as usize;
    // LANE-2 minimal clippy unblock (disclosed): `p <= EMPTY` is absurd
    // (EMPTY is the minimum); equality is the intent.
    if p == EMPTY {
        return;
    }
    for i in 0..SLOTS_N {
        if LIVE[i].load(Relaxed) == p {
            LIVE[i].store(EMPTY, Relaxed);
            SIZES[i].store(0, Relaxed);
            FREED.fetch_add(1, Relaxed);
            return;
        }
    }
}

struct CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            track(ptr, layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        untrack(ptr);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            untrack(ptr);
            track(new_ptr, new_size);
        }
        new_ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            track(ptr, layout.size());
        }
        ptr
    }
}

#[global_allocator]
static COUNTING: CountingAlloc = CountingAlloc;

/// Mint a batch of `quads` quads in a fresh arena and export it as an owned
/// stream. Returns (arena, stream); caller releases both.
///
/// # Safety
/// Plain allocations; returned handles are live and owned.
unsafe fn mint_and_export(quads: usize) -> (*mut StardfArena, *mut ArrowArrayStream) {
    let arena = stardf_arena_new(1 << 16);
    assert!(!arena.is_null(), "arena_new failed");
    let mut ids: Vec<u32> = Vec::with_capacity(4);
    for i in 0..4u32 {
        let lexical = format!("ex:term{i}");
        let id = stardf_arena_intern(arena, lexical.as_ptr(), lexical.len());
        assert_ne!(id, 0, "intern failed for {lexical}");
        ids.push(id);
    }
    let mut batch: *mut StardfBatch = core::ptr::null_mut();
    for i in 0..quads {
        // SAFETY: live arena handle; ids are in range by construction.
        batch = stardf_batch_append_quad(arena, ids[0], ids[1], ids[2], ids[3]);
        assert!(!batch.is_null(), "append_quad failed on quad {i}");
    }
    let mut stream: *mut ArrowArrayStream = core::ptr::null_mut();
    // SAFETY: live batch handle, valid out slot.
    let status = stardf_avec_export_stream(batch, &mut stream);
    assert_eq!(status, 0, "export_stream failed: status {status}");
    assert!(!stream.is_null());
    (arena, stream)
}

/// Consume the stream: get_schema + get_next until end-of-stream, releasing
/// each handed-out array (frees stardf-owned buffers inside stardf's
/// allocator).
///
/// # Safety
/// `stream` must be a live exported stream.
unsafe fn consume_stream(stream: *mut ArrowArrayStream) -> usize {
    let mut schema_out = core::mem::MaybeUninit::<ArrowSchema>::uninit();
    // SAFETY: live stream, valid out slot.
    let status = stardf_arrow_capi::stream_get_schema_shim(stream, schema_out.as_mut_ptr());
    assert_eq!(status, 0, "get_schema failed");
    let schema = schema_out.assume_init();
    let _ = schema;
    let mut arrays = 0usize;
    loop {
        let mut arr = core::mem::MaybeUninit::<ArrowArray>::uninit();
        // SAFETY: live stream, valid out slot.
        let status = stardf_arrow_capi::stream_get_next_shim(stream, arr.as_mut_ptr());
        assert_eq!(status, 0, "get_next failed");
        let mut arr = arr.assume_init();
        if arr.release.is_none() {
            return arrays;
        }
        arrays += 1;
        // SAFETY: live handed-out array we own; released exactly once.
        let release = arr.release.unwrap();
        release(&mut arr as *mut _);
    }
}

/// Run one full lifecycle inside a fresh tracking window. Returns
/// (minted, freed, surviving).
unsafe fn tracked_lifecycle(quads: usize) -> (usize, usize, Vec<(usize, usize)>) {
    window_open();
    let (arena, stream) = mint_and_export(quads);
    let arrays = consume_stream(stream);
    assert_eq!(arrays, 1, "single-shot stream yields exactly one array");
    stardf_stream_free(stream);
    stardf_arena_free(arena);
    window_close()
}

#[test]
#[cfg(not(feature = "mutant-cross-alloc"))]
fn mint_export_consume_release_is_allocation_side_identical() {
    let _g = LIFECYCLE.lock().unwrap();
    // Warm-up (setup outside the window, alloc_courts.rs :73-97 idiom):
    // absorbs one-time lazy init/teardown so the window is steady-state.
    // SAFETY: owned lifecycle, fully released.
    unsafe {
        let (a, s) = mint_and_export(1);
        consume_stream(s);
        stardf_stream_free(s);
        stardf_arena_free(a);
    }

    let (minted, freed, surviving) = unsafe { tracked_lifecycle(8) };
    assert!(
        minted > 0,
        "mint performed zero allocations in the tracked window — \
         the lifecycle never exercised stardf's allocator (vacuous court)"
    );
    assert_eq!(
        freed, minted,
        "allocation-side identity violated: {minted} allocations minted, \
         {freed} freed inside stardf's allocator, surviving: {surviving:?}"
    );
    assert!(
        surviving.is_empty(),
        "window-minted allocations survived the full lifecycle: {surviving:?}"
    );
    assert_eq!(
        stardf_arrow_capi::ownership::pending(),
        0,
        "ownership tag must be claimed by the honest release chain"
    );
}

#[test]
#[cfg(not(feature = "mutant-cross-alloc"))]
fn lifecycle_is_repeatable_without_leak() {
    let _g = LIFECYCLE.lock().unwrap();
    // SAFETY: owned lifecycle, fully released.
    unsafe {
        let (a, s) = mint_and_export(1);
        consume_stream(s);
        stardf_stream_free(s);
        stardf_arena_free(a);
    }
    for round in 0..3 {
        // SAFETY: tracked owned lifecycle.
        let (minted, freed, surviving) = unsafe { tracked_lifecycle(4) };
        assert!(minted > 0, "round {round}: vacuous window");
        assert_eq!(
            freed, minted,
            "round {round}: identity violated, surviving: {surviving:?}"
        );
        assert!(surviving.is_empty(), "round {round}: leak: {surviving:?}");
        assert_eq!(stardf_arrow_capi::ownership::pending(), 0);
    }
}
