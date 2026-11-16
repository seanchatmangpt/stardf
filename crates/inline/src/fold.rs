//! The fold: a quoted triple folds to one discriminator word.
//!
//! Layout law (named): bit 63 is the STRUCTURAL-FOLD discriminator — set
//! iff the word carries a folded quoted triple. Bits 0..62 carry a 63-bit
//! structural fingerprint: `truncate64(BLAKE3(FOLD_DOMAIN || canonical))`
//! with bit 63 masked off. A plain (unfolded) word never has bit 63 set,
//! so folded and unfolded words occupy disjoint halves of the u64 space
//! and a folded word can be told from any plain term id with one shift.

use purrdf_hash::blake3;

/// Domain separator for the fold preimage. Distinct from stardf-star's
/// `QUOTED_DOMAIN` so an inline fold and a star id never agree by accident.
pub const FOLD_DOMAIN: &[u8] = b"stardf-inline:fold:v1";

/// Discriminator bit index. Set iff the word is a folded quoted triple.
pub const DISCRIMINATOR_BIT: u32 = 63;

/// The folded word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Folded(pub u64);

impl Folded {
    /// Bit 63 set?
    #[inline]
    pub fn is_folded(&self) -> bool {
        self.0 >> DISCRIMINATOR_BIT == 1
    }

    /// The 63-bit structural fingerprint (discriminator masked off).
    #[inline]
    pub fn fingerprint(&self) -> u64 {
        self.0 & !(1u64 << DISCRIMINATOR_BIT)
    }
}

/// 63-bit fingerprint = truncate64(BLAKE3(FOLD_DOMAIN || canonical)) with
/// the discriminator bit masked off.
#[must_use]
pub fn fold_fingerprint(canonical: &[u8]) -> u64 {
    let mut preimage = Vec::with_capacity(FOLD_DOMAIN.len() + canonical.len());
    preimage.extend_from_slice(FOLD_DOMAIN);
    preimage.extend_from_slice(canonical);
    let digest = blake3::hash(&preimage);
    let word = u64::from_le_bytes(*digest.as_bytes().first_chunk().expect("BLAKE3 is 32 bytes"));
    word & !(1u64 << DISCRIMINATOR_BIT)
}

/// The full folded word: discriminator set, fingerprint in bits 0..62.
#[must_use]
pub fn fold_word(canonical: &[u8]) -> Folded {
    Folded(fold_fingerprint(canonical) | (1u64 << DISCRIMINATOR_BIT))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminator_layout() {
        let f = fold_word(b"x");
        assert!(f.is_folded());
        assert_eq!(f.0 >> 63, 1);
        assert_eq!(f.fingerprint() >> 63, 0);
        // Unfolded words never carry the bit.
        assert!(!Folded(42).is_folded());
        // Deterministic and payload-sensitive.
        assert_eq!(fold_word(b"x"), fold_word(b"x"));
        assert_ne!(fold_word(b"x"), fold_word(b"y"));
    }
}
