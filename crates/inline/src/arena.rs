//! Arena payload storage with per-slot integrity checksums.
//!
//! The folded word carries only a 63-bit fingerprint, not the payload. The
//! payload lives in the arena; each slot stores `checksum =
//! fold_fingerprint(canonical payload)` alongside the payload so that ANY
//! read path can fail closed: a mutated payload, a mutated index, or a
//! fold collision between two distinct payloads all surface as typed
//! traps, never silent corruption.

use stardf_star::term::{canonical_quoted, Term};

/// One arena slot: payload plus its fold checksum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub payload: Payload,
    pub checksum: u64,
}

/// ⟨⟨s p o⟩⟩ payload held in the arena.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Payload {
    pub s: Term,
    pub p: String,
    pub o: Term,
}

impl Payload {
    /// Canonical bytes of the payload (star's framing, inline's domain is
    /// prepended by the fold itself).
    pub fn canonical(&self) -> Vec<u8> {
        canonical_quoted(&self.s, &self.p, &self.o)
    }

    /// Fold checksum of the payload.
    pub fn checksum(&self) -> u64 {
        crate::fold::fold_fingerprint(&self.canonical())
    }
}

/// Dense arena. Index widths: `u32` (4 billion quoted triples — the
/// corpus bound in the collision receipt, orders above intended corpora).
#[derive(Default)]
pub struct Arena {
    slots: Vec<Slot>,
}

impl Arena {
    pub fn new() -> Self {
        Arena { slots: Vec::new() }
    }

    /// Append a slot, return its index.
    pub fn push(&mut self, slot: Slot) -> u32 {
        let idx = self.slots.len() as u32;
        self.slots.push(slot);
        idx
    }

    /// Indexed access WITH integrity: `Some` only if the slot's stored
    /// checksum still matches its payload's recomputed checksum.
    pub fn get_checked(&self, idx: u32) -> Option<&Slot> {
        let slot = self.slots.get(idx as usize)?;
        let live = slot.payload.checksum();
        (live == slot.checksum).then_some(slot)
    }

    /// Unchecked access — the table path uses `get_checked` instead.
    pub fn get(&self, idx: u32) -> Option<&Slot> {
        self.slots.get(idx as usize)
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }

    /// Mutable slot access (integrity courts).
    pub fn slot_mut(&mut self, idx: u32) -> Option<&mut Slot> {
        self.slots.get_mut(idx as usize)
    }
}
