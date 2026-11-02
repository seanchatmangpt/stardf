//! Court 2 (R2 falsifier, LANE 10) — cross-allocator positive-control court
//! over the stardf-arrow-capi release chain.
//!
//! ## The defect class under court
//!
//! A stardf-owned FFI handle freed OUTSIDE stardf's allocator discipline.
//! With genuinely distinct allocators this is UB (allocator mismatch on
//! free); with System-on-both-sides it is observable, not UB: the
//! ownership tag minted at export is never claimed before the dealloc.
//!
//! ## Positive-control mutant structure
//!
//! `#[cfg(feature = "mutant-cross-alloc")]` code paths sit IN the release
//! chains:
//! - `stream_c::stream_free` skips the `ownership::tag_claim` before
//!   dealloc (the free goes out untagged — the foreign path).
//! - `lib::stardf_release` returns early without running the release
//!   callbacks and without claiming the tag (untagged foreign free).
//!
//! ## How `--features mutant-cross-alloc` flips the court
//!
//! The two builds run DIFFERENT tests:
//!
//! - Clean build (`cargo test`): `honest_release_claims_every_tag` runs —
//!   full lifecycles on BOTH release chains (stream + boxed-array export)
//!   must claim every minted tag: `pending() == 0` after release. This is
//!   the no-false-positive half of the court.
//! - Mutant build (`cargo test --features mutant-cross-alloc`):
//!   `court_detects_untagged_foreign_release` runs instead — after the
//!   release the tag MUST still be pending (`pending() == 1` per chain):
//!   the court DETECTS the cross-allocator release via the ownership tag,
//!   no UB involved. A mutant build that passes the honest test, or a
//!   clean build that fires the detection, is a FAIL (vacuous court).
//!
//! Detection hook: `stardf_arrow_capi::ownership::pending()` (stream
//! chain) and `stardf_pending_tags()` (boxed-array chain). Test-idiom
//! credit: SLOTS mutex (cgap/crates/cgap-ffi/tests/alloc_courts.rs :30),
//! setup-outside-tracking (:73-97).

use std::sync::Mutex;

use stardf_arrow_capi::{
    stardf_array_addr, stardf_batch_rows, stardf_i32_batch, stardf_pending_tags, stardf_release,
    ArrowArrayStream,
};

/// Process-global gate (SLOTS idiom): export slots and tag counters are
/// process-global, so the lifecycles serialize here.
static SLOTS: Mutex<()> = Mutex::new(());

/// Mint the arena→stream lifecycle and release it. Returns rows consumed.
///
/// # Safety
/// Plain allocations and owned handles throughout.
unsafe fn stream_lifecycle(quads: usize) {
    let arena = stardf_arrow_capi::stardf_arena_new(1 << 16);
    assert!(!arena.is_null());
    let mut ids: Vec<u32> = Vec::with_capacity(4);
    for i in 0..4u32 {
        let lexical = format!("ex:term{i}");
        let id = stardf_arrow_capi::stardf_arena_intern(arena, lexical.as_ptr(), lexical.len());
        assert_ne!(id, 0);
        ids.push(id);
    }
    let mut batch: *mut stardf_arrow_capi::StardfBatch = core::ptr::null_mut();
    for _ in 0..quads {
        batch = stardf_arrow_capi::stardf_batch_append_quad(arena, ids[0], ids[1], ids[2], ids[3]);
        assert!(!batch.is_null());
    }
    let mut stream: *mut ArrowArrayStream = core::ptr::null_mut();
    // SAFETY: live batch handle, valid out slot.
    assert_eq!(
        stardf_arrow_capi::stardf_avec_export_stream(batch, &mut stream),
        0
    );
    // Consume: get_next hands out the array; release it through its own
    // callback (frees the stardf-owned buffers inside stardf's allocator).
    let mut arr = core::mem::MaybeUninit::<stardf_arrow_capi::ArrowArray>::uninit();
    // SAFETY: live stream, valid out slot.
    assert_eq!(
        stardf_arrow_capi::stream_get_next_shim(stream, arr.as_mut_ptr()),
        0
    );
    let mut arr = arr.assume_init();
    if let Some(release) = arr.release {
        // SAFETY: live handed-out array we own.
        release(&mut arr as *mut _);
    }
    // SAFETY: live owned stream.
    stardf_arrow_capi::stream_free(stream);
    // SAFETY: live owned arena.
    stardf_arrow_capi::stardf_arena_free(arena);
}

/// Mint the boxed-array export lifecycle and release it.
///
/// # Safety
/// Plain allocations and owned handles throughout.
unsafe fn boxed_array_lifecycle(n: i64, null_every: i64) {
    let ptr = stardf_i32_batch(n, null_every);
    assert!(!ptr.is_null());
    assert_eq!(stardf_array_addr(), ptr as usize, "zero-copy slot identity");
    assert_eq!(stardf_batch_rows(), n);
    // SAFETY: live stardf-owned export.
    let outcome = stardf_release(ptr);
    #[cfg(not(feature = "mutant-cross-alloc"))]
    assert_eq!(outcome, stardf_arrow_capi::ReleaseOutcome::Released);
    #[cfg(feature = "mutant-cross-alloc")]
    assert_eq!(
        outcome,
        stardf_arrow_capi::ReleaseOutcome::ReleasedViaForeign
    );
}

/// CLEAN BUILD ONLY: honest release chains must claim every minted tag —
/// the court does not false-positive on an honest lifecycle.
#[test]
#[cfg(not(feature = "mutant-cross-alloc"))]
fn honest_release_claims_every_tag() {
    let _g = SLOTS.lock().unwrap();
    // SAFETY: owned lifecycles, fully released.
    unsafe {
        stream_lifecycle(8);
        boxed_array_lifecycle(64, 7);
    }
    assert_eq!(
        stardf_arrow_capi::ownership::pending(),
        0,
        "stream release chain must claim its ownership tag"
    );
    assert_eq!(
        stardf_pending_tags(),
        0,
        "boxed-array release chain must claim its tag"
    );
}

/// MUTANT BUILD ONLY: the court must DETECT the untagged foreign release —
/// the ownership tag stays pending after the release, no UB involved.
#[test]
#[cfg(feature = "mutant-cross-alloc")]
fn court_detects_untagged_foreign_release() {
    let _g = SLOTS.lock().unwrap();
    // SAFETY: owned lifecycles (the mutant leaks them by construction).
    unsafe {
        stream_lifecycle(8);
    }
    assert_eq!(
        stardf_arrow_capi::ownership::pending(),
        1,
        "court must detect the untagged stream release via the pending ownership tag"
    );

    // SAFETY: owned lifecycle.
    unsafe {
        boxed_array_lifecycle(64, 7);
    }
    assert!(
        stardf_pending_tags() >= 1,
        "court must detect the untagged boxed-array release via the pending tag counter"
    );
}

/// BOTH BUILDS: the court substrate itself must be live — tags mint on
/// export and are observable through the public hook (zero-information
/// check would carry no bits; this pins the hook to a real mint).
#[test]
fn tag_hooks_are_live_on_real_mints() {
    let _g = SLOTS.lock().unwrap();
    let before = stardf_pending_tags();
    // SAFETY: plain allocation; slots are process-global by design.
    let ptr = unsafe { stardf_i32_batch(16, 0) };
    assert!(!ptr.is_null());
    assert_eq!(
        stardf_pending_tags(),
        before + 1,
        "each mint must mint exactly one ownership tag"
    );
    // SAFETY: live stardf-owned export.
    let outcome = unsafe { stardf_release(ptr) };
    #[cfg(not(feature = "mutant-cross-alloc"))]
    assert_eq!(outcome, stardf_arrow_capi::ReleaseOutcome::Released);
    #[cfg(feature = "mutant-cross-alloc")]
    assert_eq!(
        outcome,
        stardf_arrow_capi::ReleaseOutcome::ReleasedViaForeign
    );
    if cfg!(not(feature = "mutant-cross-alloc")) {
        assert_eq!(
            stardf_pending_tags(),
            before,
            "honest release claims the tag"
        );
    } else {
        assert!(
            stardf_pending_tags() > before,
            "mutant release must leave the tag pending (detection fuel)"
        );
    }
}
