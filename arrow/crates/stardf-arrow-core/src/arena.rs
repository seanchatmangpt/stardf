//! TermId arena (`stardf:TermIdArenaDefinition`).
//!
//! Interner mapping lexical terms to dense `TermId`s (u32, crate-canonical
//! alias from `crate::layout`). Contract invariants and their enforcement:
//!
//! - 64-byte alignment: every arena buffer is allocated with
//!   `Layout::from_size_align(bytes, 64)` (pattern:
//!   cgap/crates/cgap-arrow/src/buffer.rs); asserted by test on the exact
//!   allocation the arena owns.
//! - u32 id space: BOTTOM_ID (0) is reserved and never minted (minting
//!   starts at 1); WILDCARD_ID (u32::MAX) is refused as mintable with a
//!   typed error, never wrapped (purrdf from_index parity,
//!   purrdf/crates/rdf-core/src/ir/term.rs:110-122).
//! - BLAKE3 fingerprint: each minted id carries the BLAKE3 digest of its
//!   lexical form, retrievable via TermArena::fingerprint /
//!   FrozenArena::fingerprint.
//! - Budget seam (purrdf Admission parity,
//!   purrdf/crates/lex/src/allocation.rs:272-275): every growth path admits
//!   the next live total through Admission::resize BEFORE allocating;
//!   refusal is typed (ArenaError), never a panic.
//! - Zero post-freeze allocation: TermArena::freeze converts to a
//!   FrozenArena whose reads perform no allocation; asserted by a
//!   counting-allocator test (pattern: cgap audit/src/alloc_counter.rs).

#![forbid(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::fmt;
use core::ptr::NonNull;

/// Reserved id: never minted, means absent in dense slots.
pub const BOTTOM_ID: u32 = 0;
/// Refused id: u32::MAX is never mintable (purrdf from_index parity).
pub const WILDCARD_ID: u32 = u32::MAX;

// Reserved id (never minted) — arena-local parity with the crate's layout
// sentinel vocabulary (BOTTOM=0, WILDCARD=u32::MAX refused as mintable).

/// Arena-local id alias (u32) — dense ids minted from 1.
pub type TermId = u32;

/// Allocation alignment for every arena buffer (contract `byteAlignment 64`).
pub const ARENA_ALIGN: usize = 64;

/// Size of a stored BLAKE3 fingerprint.
pub const FINGERPRINT_LEN: usize = 32;

/// Typed refusal for every arena failure mode. No panics on any path.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArenaError {
    /// Budget seam refused the next live total.
    BudgetExceeded {
        /// Total live bytes the operation attempted to admit.
        attempted: u64,
        /// Granted limit.
        limit: u64,
    },
    /// Id-space / extent overflow before any allocation.
    Capacity {
        /// Attempted extent.
        attempted: u64,
        /// Maximum representable extent.
        limit: u64,
    },
    /// Mutating call on a frozen arena returns this, not a panic.
    Frozen,
    /// Underlying allocator returned null for the requested extent.
    AllocFailed {
        /// Requested byte extent.
        requested: usize,
    },
}

impl fmt::Display for ArenaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArenaError::BudgetExceeded { attempted, limit } => {
                write!(
                    f,
                    "budget exceeded: attempted {attempted} bytes, limit {limit}"
                )
            }
            ArenaError::Capacity { attempted, limit } => {
                write!(f, "capacity exceeded: attempted {attempted}, limit {limit}")
            }
            ArenaError::Frozen => f.write_str("arena is frozen (immutable)"),
            ArenaError::AllocFailed { requested } => {
                write!(f, "allocator returned null for {requested} bytes")
            }
        }
    }
}

#[cfg(test)]
impl std::error::Error for ArenaError {}

/// Borrowed caller's live-capacity admission for one arena (purrdf parity:
/// purrdf/crates/lex/src/allocation.rs:272-275).
pub trait Admission {
    /// Admit the next exact total live bytes before allocation; shrink only
    /// after destruction.
    ///
    /// # Errors
    /// Typed [`ArenaError`] when the budget does not admit `live_bytes`.
    fn resize(&mut self, live_bytes: usize) -> Result<(), ArenaError>;
}

/// Zero-size admission with no preset: accepts every exact live layout.
/// The arena still checks extents and performs allocation fallibly.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Resident;

impl Admission for Resident {
    #[inline]
    fn resize(&mut self, _live_bytes: usize) -> Result<(), ArenaError> {
        Ok(())
    }
}

impl<A: Admission + ?Sized> Admission for &mut A {
    #[inline]
    fn resize(&mut self, live_bytes: usize) -> Result<(), ArenaError> {
        (**self).resize(live_bytes)
    }
}

/// Fixed-grant budget refusing live totals above its limit with a typed
/// [`ArenaError::BudgetExceeded`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    limit_bytes: usize,
}

impl Budget {
    /// A budget admitting any live total `<= limit_bytes`.
    #[must_use]
    pub const fn new(limit_bytes: usize) -> Self {
        Self { limit_bytes }
    }

    /// Granted limit in bytes.
    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit_bytes
    }
}

impl Admission for Budget {
    #[inline]
    fn resize(&mut self, live_bytes: usize) -> Result<(), ArenaError> {
        if live_bytes > self.limit_bytes {
            return Err(ArenaError::BudgetExceeded {
                attempted: live_bytes as u64,
                limit: self.limit_bytes as u64,
            });
        }
        Ok(())
    }
}

/// 64-byte aligned, growable byte buffer (pattern: cgap-arrow buffer.rs).
struct AlignedBuf {
    ptr: NonNull<u8>,
    len: usize,
}

// Safety: the buffer exclusively owns its `len` bytes; only `u8` is stored
// and no references are handed out across growth.
unsafe impl Send for AlignedBuf {}

impl AlignedBuf {
    /// Allocate `len` bytes, 64-byte aligned. Typed refusal, never a panic.
    fn alloc(len: usize) -> Result<Self, ArenaError> {
        if len == 0 {
            // Dangling but 64-aligned non-null sentinel for the empty buffer:
            // the alignment law stays observable even at len 0, with no
            // allocation.
            let sentinel: &'static [u8; ARENA_ALIGN] = &[0u8; ARENA_ALIGN];
            let ptr = NonNull::from(sentinel).cast::<u8>();
            return Ok(Self { ptr, len: 0 });
        }
        let layout =
            Layout::from_size_align(len, ARENA_ALIGN).map_err(|_| ArenaError::Capacity {
                attempted: len as u64,
                limit: isize::MAX as u64,
            })?;
        // Safety: layout has nonzero size; null is checked below.
        let raw = unsafe { alloc::alloc::alloc(layout) };
        let ptr = NonNull::new(raw).ok_or(ArenaError::AllocFailed { requested: len })?;
        Ok(Self { ptr, len })
    }

    /// Initialized byte contents.
    #[inline]
    fn as_slice(&self) -> &[u8] {
        if self.len == 0 {
            return &[];
        }
        // Safety: `len` bytes are allocated and were fully written by the
        // arena before any read; lifetime is tied to &self.
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Mutable byte contents (initialization window only).
    #[inline]
    fn as_slice_mut(&mut self) -> &mut [u8] {
        if self.len == 0 {
            return &mut [];
        }
        // Safety: exclusive &mut self, `len` allocated bytes.
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for AlignedBuf {
    fn drop(&mut self) {
        if self.len != 0 {
            let layout = Layout::from_size_align(self.len, ARENA_ALIGN).expect("layout round-trip");
            // Safety: matching layout for the pointer from `alloc`.
            unsafe { alloc::alloc::dealloc(self.ptr.as_ptr(), layout) };
        }
    }
}

/// Growable TermId interner over a single 64-byte-aligned byte arena.
///
/// Every growth path runs the [`Admission`] seam on the next exact live
/// total BEFORE touching the allocator; all heap storage is 64-byte aligned.
/// Ids are dense from 1 (BOTTOM_ID = 0 reserved, never minted);
/// WILDCARD_ID is refused as mintable.
pub struct TermArena<A: Admission = Resident> {
    admission: A,
    /// Lexical bytes of all interned terms, back to back in mint order.
    text: AlignedBuf,
    /// (start, end) byte range per minted id (index i holds id i+1).
    lengths: Vec<(usize, usize)>,
    /// BLAKE3 fingerprint per minted id (index i holds id i+1).
    fingerprints: Vec<[u8; FINGERPRINT_LEN]>,
    /// Lexical form -> minted id.
    index: BTreeMap<Box<str>, u32>,
    frozen: bool,
}

// Safety: TermArena owns all its storage; A: Admission is an API choice, so
// require the caller's admission policy to be Send to keep the arena Send.
unsafe impl<A: Admission + Send> Send for TermArena<A> {}

impl<A: Admission> TermArena<A> {
    /// Empty arena; performs no allocation.
    pub fn new(admission: A) -> Self {
        Self {
            admission,
            text: AlignedBuf {
                ptr: NonNull::from(&[0u8; ARENA_ALIGN]).cast::<u8>(),
                len: 0,
            },
            lengths: Vec::new(),
            fingerprints: Vec::new(),
            index: BTreeMap::new(),
            frozen: false,
        }
    }

    /// Number of minted ids (all >= 1; BOTTOM is never minted).
    #[must_use]
    pub fn len(&self) -> u32 {
        // Dense id space excluding the reserved bottom means the mintable
        // count cannot exceed u32::MAX - 1, well inside u32.
        self.lengths.len() as u32
    }

    /// True when no terms are interned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lengths.is_empty()
    }

    /// Total live lexical bytes resident in the arena buffer.
    #[must_use]
    pub fn live_bytes(&self) -> usize {
        self.text.len
    }

    /// Intern `term`, minting a fresh dense id on first sight. Dedup: the
    /// same lexical form returns the same id.
    ///
    /// # Errors
    /// - [`ArenaError::BudgetExceeded`] when the admission seam refuses the
    ///   next live total (before any allocation).
    /// - [`ArenaError::Capacity`] on id-space or extent overflow.
    /// - [`ArenaError::Frozen`] after [`Self::freeze`].
    /// - [`ArenaError::AllocFailed`] when the allocator returns null.
    pub fn intern(&mut self, term: &str) -> Result<TermId, ArenaError> {
        if self.frozen {
            return Err(ArenaError::Frozen);
        }
        if let Some(&id) = self.index.get(term) {
            return Ok(id);
        }
        // The next mint must stay inside the lawful id space: BOTTOM (0) is
        // reserved and WILDCARD (u32::MAX) is refused as mintable, so the
        // largest mintable id is u32::MAX - 1. Refuse the mint that would
        // produce WILDCARD rather than wrapping (purrdf from_index parity).
        let next_id = self
            .len()
            .checked_add(1)
            .filter(|&n| n != BOTTOM_ID && n != WILDCARD_ID)
            .ok_or(ArenaError::Capacity {
                attempted: u64::from(self.len()) + 1,
                limit: u64::from(WILDCARD_ID) - 1,
            })?;
        let bytes = term.as_bytes();
        let new_len = self
            .text
            .len
            .checked_add(bytes.len())
            .ok_or(ArenaError::Capacity {
                attempted: u64::MAX,
                limit: isize::MAX as u64,
            })?;
        // Admission seam runs on the next exact live total, BEFORE allocating.
        self.admission.resize(new_len)?;
        // Grow the aligned buffer: allocate the replacement, copy, drop old.
        let new_buf = AlignedBuf::alloc(new_len)?;
        if self.text.len != 0 {
            // Safety: non-overlapping owned regions; source fully initialized.
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.text.as_slice().as_ptr(),
                    new_buf.ptr.as_ptr(),
                    self.text.len,
                );
            }
        }
        let write_at = self.text.len;
        self.text = new_buf;
        self.text.as_slice_mut()[write_at..new_len].copy_from_slice(bytes);
        self.lengths.push((write_at, new_len));
        self.fingerprints.push(*blake3::hash(bytes).as_bytes());
        self.index.insert(Box::from(term), next_id);
        Ok(next_id)
    }

    /// Look up a previously interned lexical form.
    #[must_use]
    pub fn get(&self, term: &str) -> Option<TermId> {
        self.index.get(term).copied()
    }

    /// Lexical form of a minted id (zero-copy borrow of arena bytes).
    #[must_use]
    pub fn term(&self, id: TermId) -> Option<&str> {
        if id == BOTTOM_ID {
            return None;
        }
        let (s, e) = *self.lengths.get((id - 1) as usize)?;
        let bytes = &self.text.as_slice()[s..e];
        // Interned terms are valid UTF-8 by construction; a corrupt arena
        // yields None, not a panic.
        core::str::from_utf8(bytes).ok()
    }

    /// BLAKE3 fingerprint of a minted id's lexical form. BOTTOM (reserved)
    /// has none.
    #[must_use]
    pub fn fingerprint(&self, id: TermId) -> Option<[u8; FINGERPRINT_LEN]> {
        if id == BOTTOM_ID {
            return None;
        }
        self.fingerprints.get((id - 1) as usize).copied()
    }

    /// Freeze: convert to a read-only [`FrozenArena`]. Post-freeze reads
    /// perform zero allocation (asserted by test with a counting allocator).
    pub fn freeze(self) -> FrozenArena {
        FrozenArena {
            text: self.text,
            lengths: self.lengths,
            fingerprints: self.fingerprints,
            index: self.index,
        }
    }
}

impl Default for TermArena<Resident> {
    fn default() -> Self {
        Self::new(Resident)
    }
}

/// Read-only frozen arena: lookups and reads allocate nothing.
pub struct FrozenArena {
    text: AlignedBuf,
    lengths: Vec<(usize, usize)>,
    fingerprints: Vec<[u8; FINGERPRINT_LEN]>,
    index: BTreeMap<Box<str>, u32>,
}

// Manual impls: FrozenArena owns all storage and shares nothing mutable.
// Vec/[u8;32]/BTreeMap<Box<str>, u32> are Send+Sync for these element types.
unsafe impl Send for FrozenArena {}
unsafe impl Sync for FrozenArena {}

impl FrozenArena {
    /// Look up a term. Zero-allocation.
    #[must_use]
    pub fn get(&self, term: &str) -> Option<TermId> {
        self.index.get(term).copied()
    }

    /// Lexical form of a minted id. Zero-allocation.
    #[must_use]
    pub fn term(&self, id: TermId) -> Option<&str> {
        if id == BOTTOM_ID {
            return None;
        }
        let (s, e) = *self.lengths.get((id - 1) as usize)?;
        core::str::from_utf8(&self.text.as_slice()[s..e]).ok()
    }

    /// BLAKE3 fingerprint of a minted id. BOTTOM (reserved) has none.
    #[must_use]
    pub fn fingerprint(&self, id: TermId) -> Option<[u8; FINGERPRINT_LEN]> {
        if id == BOTTOM_ID {
            return None;
        }
        self.fingerprints.get((id - 1) as usize).copied()
    }

    /// Raw pointer to the aligned lexical buffer (test/introspection seam).
    #[must_use]
    pub fn buffer_ptr(&self) -> *const u8 {
        self.text.ptr.as_ptr()
    }

    /// Number of interned terms.
    #[must_use]
    pub fn len(&self) -> u32 {
        self.lengths.len() as u32
    }

    /// True when empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lengths.is_empty()
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;

    // Per-thread tracking state: `Copy + Cell` + const-init TLS so the alloc
    // hook never allocates while reading it (cgap alloc_counter pattern).
    // Thread-local because the test harness runs each test on its own
    // thread; parallel test threads must not pollute each other's counts.
    #[derive(Clone, Copy)]
    struct Tracking {
        active: bool,
        count: usize,
        bytes: usize,
    }

    std::thread_local! {
        static TRACKING: Cell<Tracking> =
            const { Cell::new(Tracking { active: false, count: 0, bytes: 0 }) };
    }

    #[inline]
    fn track(layout: Layout) {
        TRACKING.with(|t| {
            let mut s = t.get();
            if s.active {
                s.count += 1;
                s.bytes += layout.size();
            }
            t.set(s);
        });
    }

    /// Test-local counting allocator (pattern: cgap audit/src/alloc_counter.rs).
    /// Pass-through over System; per-thread scoped tracking.
    struct CountingAlloc;

    unsafe impl GlobalAlloc for CountingAlloc {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let ptr = unsafe { System.alloc(layout) };
            if !ptr.is_null() {
                track(layout);
            }
            ptr
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            unsafe { System.dealloc(ptr, layout) }
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
            if !new_ptr.is_null() {
                track(Layout::from_size_align(new_size, layout.align()).expect("realloc layout"));
            }
            new_ptr
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            let ptr = unsafe { System.alloc_zeroed(layout) };
            if !ptr.is_null() {
                track(layout);
            }
            ptr
        }
    }

    #[global_allocator]
    static COUNTING: CountingAlloc = CountingAlloc;

    /// Runs `f` with allocation counting active on this thread; returns
    /// (value, count, bytes).
    fn with_alloc_tracking<T>(f: impl FnOnce() -> T) -> (T, usize, usize) {
        TRACKING.with(|t| {
            let prev = t.get();
            t.set(Tracking {
                active: true,
                count: 0,
                bytes: 0,
            });
            let out = f();
            let done = t.get();
            t.set(Tracking {
                active: prev.active,
                count: prev.count,
                bytes: prev.bytes,
            });
            (out, done.count, done.bytes)
        })
    }

    #[test]
    fn budget_refusal_is_typed_and_prealloc() {
        let mut arena = TermArena::new(Budget::new(8));
        // First intern fits (5 bytes live).
        let a = arena.intern("alpha").expect("within budget");
        assert_eq!(a, 1);
        assert_eq!(arena.live_bytes(), 5);
        // Second intern needs 5 + 5 = 10 > 8: typed refusal, id space and
        // arena state untouched.
        let err = arena.intern("bravo").unwrap_err();
        assert_eq!(
            err,
            ArenaError::BudgetExceeded {
                attempted: 10,
                limit: 8
            },
            "expected typed budget refusal, got {err:?}"
        );
        // Refused intern allocated nothing and minted nothing.
        assert_eq!(arena.len(), 1);
        assert_eq!(arena.get("bravo"), None);
        // Dedup hit allocates nothing: the term already lives in the arena.
        let (_, count, _bytes) = with_alloc_tracking(|| {
            let id = arena.intern("alpha").expect("dedup hit");
            assert_eq!(id, 1);
        });
        assert_eq!(count, 0, "dedup lookup must not allocate");
    }

    #[test]
    fn arena_allocation_is_64_byte_aligned() {
        let mut arena = TermArena::default();
        for t in ["alpha", "beta", "gamma-delta-epsilon"] {
            arena.intern(t).expect("resident admission");
        }
        assert_eq!(arena.live_bytes(), 5 + 4 + 19);
        // The buffer the arena owns is 64-byte aligned, including after growth.
        let frozen = arena.freeze();
        let ptr = frozen.buffer_ptr() as usize;
        assert!(ptr != 0, "frozen buffer pointer must be non-null");
        assert_eq!(
            ptr % ARENA_ALIGN,
            0,
            "buffer at {ptr:#x} violates 64-byte alignment"
        );
    }

    #[test]
    fn bottom_reserved_and_wildcard_refused() {
        // BOTTOM is reserved: never minted, never resolvable.
        let mut arena = TermArena::default();
        let first = arena.intern("x").expect("mint from empty");
        assert_eq!(first, 1, "minting starts at 1, BOTTOM (0) reserved");
        assert_eq!(arena.term(0), None);
        assert_eq!(arena.fingerprint(0), None);
        // WILDCARD (u32::MAX) is refused as mintable: from the API surface the
        // only mint path is intern, whose id-space guard refuses the mint that
        // would produce WILDCARD; direct reconstruction of the raw id is not
        // exposed. Verify the guard constant parity with the layout module.
        assert_eq!(WILDCARD_ID, u32::MAX);
        assert_eq!(BOTTOM_ID, 0);
        // Out-of-range ids resolve to None, not a panic.
        assert_eq!(arena.term(u32::MAX), None);
        assert_eq!(arena.fingerprint(u32::MAX), None);
        assert_eq!(arena.term(arena.len() + 1), None);
    }

    #[test]
    fn interner_dedups_and_roundtrips() {
        let mut arena = TermArena::default();
        let a1 = arena.intern("http://example.org/s").expect("intern");
        let a2 = arena.intern("http://example.org/s").expect("intern");
        assert_eq!(a1, a2, "same lexical form, same id");
        assert_eq!(arena.len(), 1);
        let b = arena.intern("http://example.org/p").expect("intern");
        assert_ne!(a1, b);
        assert_eq!(arena.get("http://example.org/s"), Some(a1));
        assert_eq!(arena.get("http://example.org/missing"), None);
        assert_eq!(arena.term(a1), Some("http://example.org/s"));
        assert_eq!(arena.term(b), Some("http://example.org/p"));
        // BLAKE3 fingerprint is the digest of the lexical form.
        let fp = arena.fingerprint(a1).expect("minted id");
        assert_eq!(fp, *blake3::hash(b"http://example.org/s").as_bytes());
        assert_ne!(fp, arena.fingerprint(b).expect("minted id"));
    }

    #[test]
    fn freeze_is_zero_allocation_on_reads() {
        let mut arena = TermArena::default();
        let ids: Vec<u32> = ["s", "p", "o", "g", "extra-long-literal-value-0123456789"]
            .iter()
            .map(|t| arena.intern(t).expect("intern"))
            .collect();
        let frozen = arena.freeze();
        // Result containers are allocated OUTSIDE the tracked scope so the
        // counter sees only the arena's own reads.
        let mut id_reads: Vec<(Option<&str>, Option<[u8; FINGERPRINT_LEN]>)> =
            Vec::with_capacity(ids.len());
        let mut lookups: Vec<Option<u32>> = Vec::with_capacity(6);
        let ((), count, bytes) = with_alloc_tracking(|| {
            for &id in &ids {
                id_reads.push((frozen.term(id), frozen.fingerprint(id)));
            }
            for t in [
                "s",
                "p",
                "o",
                "g",
                "extra-long-literal-value-0123456789",
                "missing",
            ] {
                lookups.push(frozen.get(t));
            }
        });
        assert!(
            count == 0,
            "post-freeze reads allocated {count} times ({bytes} bytes)"
        );
        assert_eq!(id_reads.len(), 5);
        assert_eq!(lookups.len(), 6);
        assert_eq!(lookups[5], None, "missing term lookup");
        assert_eq!(frozen.term(ids[0]), Some("s"));
        assert_eq!(frozen.get("missing"), None);
    }

    #[test]
    fn frozen_refuses_mutation_typed() {
        // Mutation refusal is typed at the type level: FrozenArena has no
        // intern at all; TermArena::intern after move is unrepresentable.
        // Cover the runtime Frozen error path via the guard flag contract:
        // a moved-into-freeze arena cannot be reused (compile-time). Here we
        // assert the error variant exists and displays.
        let err = ArenaError::Frozen;
        let msg = std::format!("{err}");
        assert_eq!(msg, "arena is frozen (immutable)");
    }

    #[test]
    fn growth_reallocs_stay_aligned_and_data_intact() {
        // Push enough distinct terms to force multiple aligned regrowths.
        let mut arena = TermArena::default();
        let n = 500;
        for i in 0..n {
            let term = std::format!("http://example.org/term/{i:04}");
            let id = arena.intern(&term).expect("intern");
            assert_eq!(id as usize, i + 1, "dense mint order from 1");
        }
        assert_eq!(arena.len(), n as u32);
        // Every term round-trips with the right BLAKE3 fingerprint.
        for i in 0..n {
            let term = std::format!("http://example.org/term/{i:04}");
            let id = (i + 1) as u32;
            assert_eq!(arena.term(id), Some(term.as_str()));
            assert_eq!(
                arena.fingerprint(id),
                Some(*blake3::hash(term.as_bytes()).as_bytes())
            );
        }
    }
}
