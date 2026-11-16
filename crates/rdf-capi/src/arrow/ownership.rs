//! Ownership-tag registry — LANE 10 (R2 falsifier court substrate).
//!
//! Every stardf-owned FFI handle minted for export registers a tag here at
//! mint time; the release chain must CLAIM the tag before deallocating.
//! `pending()` after a full release is the cross-allocator court's detection
//! hook: a pending tag for freed memory proves the free escaped stardf's
//! allocator discipline (with System-on-both-sides this is observable, not
//! UB).
//!
//! Mutant posture: under `mutant-cross-alloc`, the release chain skips the
//! claim (the free goes out untagged — the defect shape). See
//! `tests/cross_allocator_court.rs` for the flip contract.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Mint-time ownership tag: the layout the handle was boxed with, so the
/// release chain deallocates with the layout the allocation used (tagged
/// pointer discipline).
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)] // size/align consumed by debug_assert path and future layout-check courts
pub struct Tag {
    size: usize,
    align: usize,
}

static OWNERSHIP: OnceLock<Mutex<HashMap<usize, Tag>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<usize, Tag>> {
    OWNERSHIP.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Register an ownership tag for `ptr` boxed with `layout`. Called at mint
/// time, before the handle leaves the crate.
pub fn tag_mint(ptr: *mut core::ffi::c_void, layout: Layout) {
    registry().lock().unwrap().insert(
        ptr as usize,
        Tag {
            size: layout.size(),
            align: layout.align(),
        },
    );
}

/// Claim (remove) the ownership tag for `ptr`. Returns None when the pointer
/// is foreign or already released.
pub fn tag_claim(ptr: *mut core::ffi::c_void) -> Option<Tag> {
    registry().lock().unwrap().remove(&(ptr as usize))
}

/// Pending (unclaimed) tags — the court's detection hook.
pub fn pending() -> usize {
    registry().lock().unwrap().len()
}

use std::alloc::Layout;
