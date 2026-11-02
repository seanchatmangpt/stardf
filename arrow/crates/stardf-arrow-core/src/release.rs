//! Shared release-slot discipline for the stardf Arrow substrate.
//!
//! Single kernel, many bindings: every exported FFI struct (`ArrowArray`,
//! `ArrowSchema`, `ArrowArrayStream`) carries one `ReleaseSlot`-shaped
//! release/private pair and binds this one kernel, instead of each binding
//! restating the CAS-swap discipline (the two-BLAKE3 duplication defect
//! class). Model: `cgap-arrow/src/c_abi.rs:54-98`.
//!
//! Ordering law (disarm-then-drop): the atomic swap-to-NULL completes BEFORE
//! the drop callback runs, so any concurrent observer that reads the slot
//! after the swap sees a disarmed struct; the drop callback itself sees the
//! private data live until the swap has already completed.
//!
//! Exactly-once law: the first `release` wins the swap; every later call
//! observes NULL and is a typed no-op (`ReleaseOutcome::DoubleRelease`),
//! never a double free.

extern crate alloc;

use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// Drop callback for armed private data. Receives the raw private pointer;
/// must recreate the boxed shell exactly as armed.
pub type DropFn = unsafe extern "C" fn(*mut core::ffi::c_void);

/// Outcome of a [`ReleaseSlot::release`] call. The double-release case is a
/// typed value, not an error or an abort — C hosts may legally release twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseOutcome {
    /// This call was the first-release winner; the drop callback ran.
    Released,
    /// Already released (or never armed): typed no-op, nothing dropped.
    DoubleRelease,
}

/// Exactly-once release slot: an atomic release-callback pointer plus the
/// private-data pointer it drops.
///
/// Layout mirrors the FFI pair (`release: Option<unsafe fn>` +
/// `private_data: *mut c_void`): the release field is NULL after release
/// (disarmed) and holds the callback while armed.
#[repr(C)]
pub struct ReleaseSlot {
    /// When armed: the [`DropFn`] (fn-ptr value stored as a raw pointer).
    /// NULL = disarmed. The swap on this field IS the exactly-once election.
    slot: AtomicPtr<core::ffi::c_void>,
    /// Private data, live until the release winner's callback drops it.
    private: AtomicPtr<core::ffi::c_void>,
}

// SAFETY: the slot is an arm-once/release-once election over atomics; the
// caller upholds the aliasing contract on `private` (see `private()`).
unsafe impl Send for ReleaseSlot {}
unsafe impl Sync for ReleaseSlot {}

impl ReleaseSlot {
    /// A disarmed, empty slot.
    pub const fn new() -> Self {
        Self {
            slot: AtomicPtr::new(core::ptr::null_mut()),
            private: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    /// Arm the slot with a drop callback and its private data.
    ///
    /// # Safety
    /// Call exactly once per slot, before the struct is shared with any other
    /// thread or handed to a C host. `private` must remain valid until a
    /// winning `release` runs `release_fn` on it.
    pub unsafe fn arm(&self, release_fn: DropFn, private: *mut core::ffi::c_void) {
        // SAFETY: caller guarantees arm-once-before-share; fn ptr and raw
        // pointer are both pointer-sized with a nullable niche.
        let fn_as_ptr: *mut core::ffi::c_void = release_fn as *mut core::ffi::c_void;
        self.private.store(private, Ordering::Release);
        // Release ordering: arming store last, so any observer that reads the
        // armed callback also sees the private pointer fully stored.
        self.slot.store(fn_as_ptr, Ordering::Release);
    }

    /// Exactly-once release. On the winning call: swap-to-NULL first
    /// (disarm), THEN run the drop callback on the private data
    /// (disarm-then-drop). Every subsequent call is
    /// [`ReleaseOutcome::DoubleRelease`] and touches no memory.
    pub fn release(&self) -> ReleaseOutcome {
        // AcqRel: acquire pairs with the arm store (private data visible),
        // release publishes the disarm to concurrent losers.
        let prev = self.slot.swap(core::ptr::null_mut(), Ordering::AcqRel);
        if prev.is_null() {
            return ReleaseOutcome::DoubleRelease;
        }
        // SAFETY: prev was stored by `arm` from a valid `DropFn`.
        let drop_fn: DropFn = unsafe { core::mem::transmute(prev) };
        let private = self.private.load(Ordering::Acquire);
        // Disarm-then-drop: the swap above has already completed; the
        // callback observes a disarmed slot and live private data.
        // SAFETY: armed exactly once; this is the single winning call.
        unsafe { drop_fn(private) };
        ReleaseOutcome::Released
    }

    /// True once the slot has been released (or never armed).
    pub fn is_disarmed(&self) -> bool {
        self.slot.load(Ordering::Acquire).is_null()
    }

    /// Read the private-data pointer while the slot is still armed.
    ///
    /// # Safety
    /// Caller guarantees no concurrent or past release (e.g. the exporter
    /// reading its own still-armed slot).
    pub unsafe fn private(&self) -> *mut core::ffi::c_void {
        self.private.load(Ordering::Acquire)
    }
}

/// Default: a disarmed slot is freely constructible.
impl Default for ReleaseSlot {
    fn default() -> Self {
        Self::new()
    }
}

// Ensure the pun transmutes stay pointer-sized.
const _: () = assert!(core::mem::size_of::<DropFn>() == core::mem::size_of::<*mut ()>());

/// Reference-counted private-data shell for stream+array shared ownership.
///
/// The same payload backs an `ArrowArrayStream` and its yielded arrays: each
/// FFI struct holds the shell pointer as its private data and one refcount;
/// every release callback decrements once. The payload drops exactly when the
/// last reference releases.
pub struct RefCounted<T> {
    refs: AtomicUsize,
    data: core::mem::ManuallyDrop<T>,
}

impl<T> RefCounted<T> {
    /// Box the shell with one reference and hand out the raw private pointer.
    pub fn new(data: T) -> *mut Self {
        alloc::boxed::Box::into_raw(alloc::boxed::Box::new(Self {
            refs: AtomicUsize::new(1),
            data: core::mem::ManuallyDrop::new(data),
        }))
    }

    /// Acquire one reference.
    ///
    /// # Safety
    /// The shell is live (refs >= 1).
    pub unsafe fn retain(&self) {
        // Relaxed is the established refcount-increment ordering (the owner
        // of the new ref already has proven access); AcqRel on the decrement
        // below carries the synchronization.
        self.refs.fetch_add(1, Ordering::Relaxed);
    }

    /// Release one reference; drops the payload exactly on the last one.
    /// Returns the remaining count after the decrement (0 = this call won).
    ///
    /// # Safety
    /// The shell is live (refs >= 1); each release call corresponds to one
    /// previously-acquired reference.
    pub unsafe fn release(&self) -> usize {
        let prev = self.refs.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            // SAFETY: last reference; the box was created in `new`.
            drop(unsafe { alloc::boxed::Box::from_raw(self as *const Self as *mut Self) });
        }
        prev - 1
    }

    /// Current reference count (diagnostic; racy under concurrency).
    pub fn ref_count(&self) -> usize {
        self.refs.load(Ordering::Acquire)
    }

    /// Shared payload access.
    ///
    /// # Safety
    /// refs >= 1 and no &mut alias.
    pub unsafe fn data(&self) -> &T {
        &self.data
    }
}

impl<T> Drop for RefCounted<T> {
    fn drop(&mut self) {
        // SAFETY: Drop runs exactly once, after the last `release`.
        unsafe { core::mem::ManuallyDrop::drop(&mut self.data) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use alloc::{boxed::Box, vec::Vec};
    use core::ffi::c_void;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;

    /// Witness payload: records drops, carries a value the drop callback
    /// reads to prove the data is live until after the NULL swap.
    struct Witness {
        value: u32,
        drops: &'static AtomicUsize,
    }

    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn drop_witness(private: *mut c_void) {
        // SAFETY: armed as Box<Witness> in the test.
        let w = unsafe { Box::from_raw(private as *mut Witness) };
        assert_eq!(w.value, 0xDEAD_BEEF);
        w.drops.fetch_add(1, Ordering::SeqCst);
        drop(w);
    }

    fn arm_witness(slot: &ReleaseSlot) -> *mut Witness {
        let w = Box::into_raw(Box::new(Witness {
            value: 0xDEAD_BEEF,
            drops: &DROP_COUNT,
        }));
        // SAFETY: arm-once, test-scoped.
        unsafe { slot.arm(drop_witness, w as *mut c_void) };
        w
    }

    #[test]
    fn exactly_once_under_repeated_release() {
        DROP_COUNT.store(0, Ordering::SeqCst);
        let slot = ReleaseSlot::new();
        let w = arm_witness(&slot);
        assert!(!slot.is_disarmed());

        assert_eq!(slot.release(), ReleaseOutcome::Released);
        assert!(slot.is_disarmed());
        for _ in 0..100 {
            assert_eq!(slot.release(), ReleaseOutcome::DoubleRelease);
        }
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1);
        let _ = w; // already dropped by the winning release; raw ptr retained
                   // only as the arm-time witness value.
    }

    #[test]
    fn drop_callback_sees_disarmed_slot_and_live_data() {
        DROP_COUNT.store(0, Ordering::SeqCst);
        // Const-constructible slot, so the callback itself can probe it.
        static SLOT: ReleaseSlot = ReleaseSlot::new();
        static CB_RAN: AtomicUsize = AtomicUsize::new(0);

        unsafe extern "C" fn probe_drop(private: *mut c_void) {
            // Disarm-then-drop law witnessed from inside the callback:
            // the NULL swap has completed BEFORE we run...
            assert!(SLOT.is_disarmed(), "slot must disarm before drop runs");
            // ...yet the private data is still live and readable.
            let p = unsafe { SLOT.private() };
            assert_eq!(p, private, "private pointer stable until after swap");
            // SAFETY: armed as Box<Witness>.
            let w = unsafe { Box::from_raw(private as *mut Witness) };
            assert_eq!(w.value, 0xDEAD_BEEF);
            w.drops.fetch_add(1, Ordering::SeqCst);
            drop(w);
            CB_RAN.store(1, Ordering::SeqCst);
        }

        let w = Box::into_raw(Box::new(Witness {
            value: 0xDEAD_BEEF,
            drops: &DROP_COUNT,
        }));
        // SAFETY: arm-once, test-scoped static.
        unsafe { SLOT.arm(probe_drop, w as *mut c_void) };
        assert_eq!(SLOT.release(), ReleaseOutcome::Released);
        assert_eq!(CB_RAN.load(Ordering::SeqCst), 1);
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn concurrent_release_from_8_threads_drops_exactly_once() {
        DROP_COUNT.store(0, Ordering::SeqCst);
        let slot = Arc::new(ReleaseSlot::new());
        let w = arm_witness(&slot);

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let slot = Arc::clone(&slot);
                std::thread::spawn(move || {
                    let mut winners = 0;
                    for _ in 0..64 {
                        if slot.release() == ReleaseOutcome::Released {
                            winners += 1;
                        }
                    }
                    winners
                })
            })
            .collect();
        let total_winners: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        assert_eq!(total_winners, 1, "exactly one thread wins the release");
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1);
        assert!(slot.is_disarmed());
        let _ = w;
    }

    #[test]
    fn unarmed_slot_release_is_typed_noop() {
        let slot = ReleaseSlot::new();
        assert!(slot.is_disarmed());
        assert_eq!(slot.release(), ReleaseOutcome::DoubleRelease);
    }

    #[test]
    fn refcounted_shell_shared_stream_array_ownership() {
        static SHELL_DROPS: AtomicUsize = AtomicUsize::new(0);
        struct Payload(u64);
        impl Drop for Payload {
            fn drop(&mut self) {
                SHELL_DROPS.fetch_add(1, Ordering::SeqCst);
            }
        }

        let shell = RefCounted::<Payload>::new(Payload(42));
        // SAFETY: refs == 1 after new.
        let payload0 = unsafe { RefCounted::data(&*shell).0 };
        assert_eq!(payload0, 42);
        assert_eq!(unsafe { &*shell }.ref_count(), 1);

        // Stream retains; array retains; three owners total.
        // SAFETY: live shell.
        unsafe { RefCounted::retain(&*shell) };
        unsafe { RefCounted::retain(&*shell) };
        assert_eq!(unsafe { &*shell }.ref_count(), 3);

        // Three releases (any interleaving); payload drops on the last.
        // SAFETY: live shell, owned refs.
        assert_eq!(unsafe { RefCounted::release(&*shell) }, 2);
        assert_eq!(unsafe { RefCounted::release(&*shell) }, 1);
        assert_eq!(unsafe { RefCounted::release(&*shell) }, 0);
        assert_eq!(SHELL_DROPS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn refcounted_concurrent_releases_drop_exactly_once() {
        static SHELL_DROPS: AtomicUsize = AtomicUsize::new(0);
        struct Payload;
        impl Drop for Payload {
            fn drop(&mut self) {
                SHELL_DROPS.fetch_add(1, Ordering::SeqCst);
            }
        }
        let shell = RefCounted::<Payload>::new(Payload);
        let shell: &'static RefCounted<Payload> = Box::leak(unsafe { Box::from_raw(shell) });
        for _ in 0..15 {
            // SAFETY: live shell.
            unsafe { RefCounted::retain(shell) };
        }
        assert_eq!(shell.ref_count(), 16);
        // SAFETY: 16 owned refs released exactly once each.
        let handles: Vec<_> = (0..8)
            .map(|_| {
                std::thread::spawn(move || {
                    // SAFETY: owned refs, 2 per thread.
                    for _ in 0..2 {
                        unsafe { RefCounted::release(shell) };
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(SHELL_DROPS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn release_slot_plus_refcounted_end_to_end() {
        // Stream slot + array slot sharing one RefCounted shell: the exact
        // stream+array ownership shape the substrate map calls for.
        static SHELL_DROPS: AtomicUsize = AtomicUsize::new(0);
        struct Payload(#[allow(dead_code)] u32);
        impl Drop for Payload {
            fn drop(&mut self) {
                SHELL_DROPS.fetch_add(1, Ordering::SeqCst);
            }
        }
        unsafe extern "C" fn shell_drop(private: *mut c_void) {
            // SAFETY: armed as Box<RefCounted<Payload>>.
            unsafe { RefCounted::release(&*(private as *const RefCounted<Payload>)) };
        }

        let shell_ptr = RefCounted::<Payload>::new(Payload(7));
        let stream_slot = ReleaseSlot::new();
        let array_slot = ReleaseSlot::new();
        // LANE 7 FIX (disclosed): each armed slot owns one reference. The
        // shell is born with refs=1 (stream slot); the array slot must
        // retain its own ref before arming, or the first release drops the
        // shell while the second slot still holds a dangling pointer
        // (use-after-free; heap corruption surfaced as SIGTRAP in any test
        // scheduled after this one).
        unsafe {
            RefCounted::retain(&*shell_ptr); // array slot's reference
        }
        // SAFETY: arm-once per slot, distinct slots, shell live.
        unsafe {
            stream_slot.arm(shell_drop, shell_ptr as *mut c_void);
            array_slot.arm(shell_drop, shell_ptr as *mut c_void);
        }

        // Either order; both callbacks run; shell drops on the second.
        assert_eq!(stream_slot.release(), ReleaseOutcome::Released);
        assert_eq!(stream_slot.release(), ReleaseOutcome::DoubleRelease);
        assert_eq!(array_slot.release(), ReleaseOutcome::Released);
        assert_eq!(array_slot.release(), ReleaseOutcome::DoubleRelease);
        assert_eq!(SHELL_DROPS.load(Ordering::SeqCst), 1);
    }
}
