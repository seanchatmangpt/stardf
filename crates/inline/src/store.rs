//! The inline store: fingerprint table + arena + collision trap.
//!
//! `fold_quote` computes the 63-bit fingerprint of the payload; if the
//! fingerprint is already present AND the existing slot's checksum
//! differs, the two payloads collided in the 63-bit space — typed trap,
//! payload NOT stored (fail-closed; see docs/ost/inline-folding.md for
//! the probability math). Equal payloads dedupe idempotently.

use crate::arena::{Arena, Payload, Slot};
use crate::fold::{fold_fingerprint, Folded};
use stardf_star::term::Term;
use std::collections::HashMap;

/// Typed trap reasons. A trap is SOUND behavior: detected divergence, no
/// silent corruption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trap {
    /// Two distinct payloads folded to the same 63-bit fingerprint.
    FoldCollision,
    /// Arena slot at the mapped index no longer matches its checksum.
    ArenaCorruption,
    /// Mapped index is out of the arena's bounds.
    IndexOutOfBounds,
}

/// Fold errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldError {
    /// `Trap::FoldCollision` surfaced through the fold path.
    Trapped,
}

/// The store.
#[derive(Default)]
pub struct InlineStore {
    /// 63-bit fingerprint -> arena index.
    table: HashMap<u64, u32>,
    arena: Arena,
    /// Witnessed traps (observable final state for courts).
    traps: Vec<(Trap, u64)>,
}

impl InlineStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fold ⟨⟨s p o⟩⟩ into a discriminator word and register the payload.
    pub fn fold_quote(&mut self, s: Term, p: &str, o: Term) -> Result<Folded, FoldError> {
        let payload = Payload { s, p: p.to_string(), o };
        let fp = fold_fingerprint(&payload.canonical());
        if let Some(&idx) = self.table.get(&fp) {
            let stored = self
                .arena
                .get(idx)
                .ok_or(FoldError::Trapped)
                .and_then(|slot| {
                    (slot.payload == payload).then_some(()).ok_or(FoldError::Trapped)
                });
            if stored.is_err() {
                self.traps.push((Trap::FoldCollision, fp));
                return Err(FoldError::Trapped);
            }
            // Equal payload: idempotent dedupe.
            return Ok(fold_word_from_fp(fp));
        }
        let idx = self.arena.push(Slot { checksum: fp, payload });
        self.table.insert(fp, idx);
        Ok(fold_word_from_fp(fp))
    }

    /// Lookup WITH trap: fingerprint -> index -> checked arena read.
    pub fn lookup(&self, folded: Folded) -> Result<&Payload, Trap> {
        let fp = folded.fingerprint();
        let idx = self.table.get(&fp).copied().ok_or(Trap::IndexOutOfBounds)?;
        let slot = self.arena.get_checked(idx).ok_or(Trap::ArenaCorruption)?;
        Ok(&slot.payload)
    }

    /// The arena index mapped for a fingerprint (court surface).
    pub fn index_of(&self, fp: u64) -> Option<u32> {
        self.table.get(&fp).copied()
    }

    /// Witnessed trap log (courts assert on this final state).
    pub fn traps(&self) -> &[(Trap, u64)] {
        &self.traps
    }

    pub fn quoted_count(&self) -> usize {
        self.arena.len()
    }

    pub fn arena(&self) -> &Arena {
        &self.arena
    }

    /// Mutable arena access (integrity courts corrupt real slots through
    /// this and assert the trap fires on the next read).
    pub fn arena_mut(&mut self) -> &mut Arena {
        &mut self.arena
    }

    /// Whole-store integrity walk: recompute every payload checksum.
    /// Returns the fingerprints of corrupted slots (empty = sound).
    pub fn verify_integrity(&self) -> Vec<u64> {
        self.arena
            .slots()
            .iter()
            .filter(|slot| slot.payload.checksum() != slot.checksum)
            .map(|slot| slot.checksum)
            .collect()
    }
}

fn fold_word_from_fp(fp: u64) -> Folded {
    use crate::fold::DISCRIMINATOR_BIT;
    Folded(fp | (1u64 << DISCRIMINATOR_BIT))
}
