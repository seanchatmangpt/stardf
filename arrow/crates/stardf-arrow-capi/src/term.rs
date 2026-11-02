//! LANE-OWNED FILE (lane 9: TermId divergence bridge + term/arena internals).
//!
//! `StardfTermView` / [`purrdf_view_to_stardf`] bridge the purrdf TermId
//! encoding to the stardf/cgap u32 encoding. The DIVERGENCE register below is
//! the enumerated contract for every semantic difference the bridge must
//! handle; each row has a named conversion edge and a unit test.
//!
//! The `StardfArena` API consumed by `lib.rs` is unchanged from the earlier
//! fallback:
//!
//! ```text
//! struct StardfArena { /* lane 8's own fields */ }
//! impl StardfArena {
//!     fn with_capacity_bytes(cap: usize) -> Option<Self>;
//!     fn intern(&mut self, lexical: &[u8]) -> Option<u32>;
//!     fn lookup(&self, id: u32) -> Option<&[u8]>;
//!     fn len(&self) -> u32;
//!     fn charge_quad(&mut self) -> bool;
//!     fn batch_mut(&mut self) -> &mut crate::StardfBatch;
//! }
//! ```
//!
//! The arena also owns the append-only `StardfBatch` (lib.rs's type) so the
//! batch handle stays pointer-stable for the arena's lifetime.

#![allow(dead_code)]

use std::collections::HashMap;
use std::vec::Vec;

/// Per-term accounting overhead (id slot + hash entry), pinned so the
/// budget law is deterministic across lanes.
pub const TERM_OVERHEAD: usize = 16;

/// Term arena: byte-budgeted lexical interner.
pub struct StardfArena {
    cap: usize,
    used: usize,
    terms: Vec<Vec<u8>>,
    index: HashMap<Vec<u8>, u32>,
    batch: crate::StardfBatch,
}

impl StardfArena {
    /// New arena with a total byte budget `cap` (0 => None, invalid).
    pub fn with_capacity_bytes(cap: usize) -> Option<Self> {
        if cap == 0 {
            return None;
        }
        Some(Self {
            cap,
            used: 0,
            terms: Vec::new(),
            index: HashMap::new(),
            batch: crate::StardfBatch::new(),
        })
    }

    /// Intern `lexical`, returning a stable term id (ids start at 1; 0 is
    /// reserved as the null/not-found sentinel so no valid term is id 0).
    pub fn intern(&mut self, lexical: &[u8]) -> Option<u32> {
        if let Some(&id) = self.index.get(lexical) {
            return Some(id);
        }
        if self.used + lexical.len() + TERM_OVERHEAD > self.cap {
            return None; // ARENA_FULL
        }
        let id = self.terms.len() as u32 + 1;
        let key = lexical.to_vec();
        self.terms.push(key.clone());
        self.index.insert(key, id);
        self.used += lexical.len() + TERM_OVERHEAD;
        Some(id)
    }

    /// Lexical form of `id`, or None when out of range.
    pub fn lookup(&self, id: u32) -> Option<&[u8]> {
        if id == 0 {
            return None;
        }
        self.terms.get((id - 1) as usize).map(|v| v.as_slice())
    }

    /// Number of interned terms.
    pub fn len(&self) -> u32 {
        self.terms.len() as u32
    }

    /// True when no terms interned.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// Charge the byte cost of one quad (4 x u32) against the budget;
    /// false when the budget would overflow (ARENA_FULL).
    pub fn charge_quad(&mut self) -> bool {
        const QUAD_BYTES: usize = 16;
        if self.used + QUAD_BYTES > self.cap {
            return false;
        }
        self.used += QUAD_BYTES;
        true
    }

    /// Bytes of the budget consumed so far.
    pub fn used(&self) -> usize {
        self.used
    }

    /// Mutable access to the arena-owned append-only quad batch.
    pub fn batch_mut(&mut self) -> &mut crate::StardfBatch {
        &mut self.batch
    }
}

// ---------------------------------------------------------------------------
// TermId divergence bridge (lane 9)
// ---------------------------------------------------------------------------

// DIVERGENCE REGISTER — purrdf TermId <-> stardf/cgap TermId
//
// This bridge converts between two INCOMPATIBLE u32 id encodings. Every row
// below names a semantic difference, the conversion edge that handles it, and
// the unit test that pins it. If you add a difference, add a row + edge + test.
//
// | #  | axis        | purrdf (rdf-core/src/ir/term.rs:44-122)            | stardf/cgap (cgap-arrow/src/layout.rs:20-31)     | edge + test |
// |----|-------------|----------------------------------------------------|--------------------------------------------------|-------------|
// | D1 | type        | `TermId(NonZeroU32)` `#[repr(transparent)]`        | `pub type TermId = u32`                          | (structural) |
// | D2 | offset      | inner = `dense_index + 1`; `index()` = inner - 1   | no offset; id IS the arena slot + 1 (ids start at 1) | identity on the RAW word (`purrdf_raw_to_stardf`); `bridge_view_roundtrip_preserves_dense_index` |
// | D3 | 0 sentinel  | `0` = niche sentinel ("none"), never minted        | `0` = `BOTTOM_TERM_ID` (unbound/null), never minted | `purrdf_raw_zero_maps_to_bottom` |
// | D4 | u32::MAX    | refused by `from_index` (index+1 would overflow); max raw word IS u32::MAX (index u32::MAX-1) | `0xFFFF_FFFF` = `WILDCARD_TERM_ID` (match-anything), never minted | `purrdf_raw_u32_max_refused` |
// | D5 | scope       | dataset-local; not merge-stable, never serialized (C0.8) | arena-local; stable only within one `StardfArena` | (documented here; both sides refuse cross-scope use by convention) |
// | D6 | pattern role| n/a — `QuadProbePlan` (purrdf dataset.rs:722) uses `0` as UNBOUND with a *probe* split | `WILDCARD` matches any term including `BOTTOM`; `BOTTOM` matches only itself | `bottom_vs_wildcard_semantics` |
// | D7 | hashing     | hand-impl hashes `inner - 1` (the dense index), NOT the raw word | derived `Hash` on the plain u32 | (conversion preserves the dense index, so hash-equality follows) |
//
// Consequence of D2+D3: the purrdf `+1` offset and the stardf 1-based id
// convention CANCEL — carrying the raw purrdf word verbatim into stardf is a
// dense-index-preserving identity for every mintable purrdf id. Any code that
// "strips the offset" by subtracting 1 produces a BOTTOM collision for purrdf
// raw id 1 and is therefore FORBIDDEN through this bridge.

/// Mirrored purrdf term view: the RDF-term surface of a purrdf term plus its
/// raw `u32` word (`TermId::raw()`, i.e. `dense_index + 1`). `kind`, `lexical`,
/// `datatype`, `language` and `blank_scope` carry the term's RDF value so the
/// receiving arena can re-intern it; `term_id` is the identity word this
/// bridge converts.
///
/// `blank_scope` is the purrdf dataset identity the blank node is local to
/// (D5); it is carried for re-interning only and never folded into the id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StardfTermView {
    /// RDF term kind.
    pub kind: TermKind,
    /// Lexical form (IRI bytes, literal lexical form, or blank-node label).
    pub lexical: Vec<u8>,
    /// Datatype IRI for typed literals (None for IRIs/blanks; callers owning
    /// a language-tagged literal should use `rdf:langString` per RDF 1.2).
    pub datatype: Option<Vec<u8>>,
    /// Language tag for language-tagged literals.
    pub language: Option<Vec<u8>>,
    /// Dataset/blank-node scope identifier (D5), when the term is a blank node.
    pub blank_scope: Option<u64>,
    /// Raw purrdf id word: `dense_index + 1`; `0` = absent (niche), per D3.
    pub term_id: u32,
}

/// RDF term kind of a [`StardfTermView`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermKind {
    /// IRI.
    Iri,
    /// Literal (typed and/or language-tagged).
    Literal,
    /// Blank node, scoped to `blank_scope`.
    Blank,
}

/// Typed failure of [`purrdf_view_to_stardf`] / [`purrdf_raw_to_stardf`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermViewError {
    /// purrdf raw word `u32::MAX` collides with the stardf
    /// `WILDCARD_TERM_ID` (D4). No stardf encoding exists; refuse loudly.
    WildcardCollision,
}

/// Explicit total conversion: raw purrdf id word -> stardf `TermId`.
///
/// - `0` -> `BOTTOM_TERM_ID` (D3; purrdf "none" and stardf "unbound" agree).
/// - `u32::MAX` -> `Err(WildcardCollision)` (D4).
/// - any other raw word `w` (= purrdf `dense_index + 1`) -> `Ok(w)`: the
///   offsets cancel (D2), so the dense index survives as `w - 1`.
// LANE 09 (2026-10-31): sentinel constants now come from stardf-arrow-core's
// spec-fixed layout mirror (value-identical: 0 and 0xFFFF_FFFF, both
// const-asserted there); the cgap git crate left this file.
#[inline]
pub const fn purrdf_raw_to_stardf(raw: u32) -> Result<u32, TermViewError> {
    match raw {
        0 => Ok(stardf_arrow_core::layout::BOTTOM_TERM_ID),
        0xFFFF_FFFF => Err(TermViewError::WildcardCollision),
        // D2: purrdf inner (index+1) == stardf 1-based id; identity.
        w => Ok(w),
    }
}

/// Explicit total conversion of a full [`StardfTermView`] to a stardf
/// [`purrdf_view_to_stardf::TermId`] word, for use when re-interning the
/// term's RDF value into a `StardfArena`.
#[inline]
pub fn purrdf_view_to_stardf(view: &StardfTermView) -> Result<u32, TermViewError> {
    purrdf_raw_to_stardf(view.term_id)
}

/// Inverse edge: stardf `TermId` word -> raw purrdf word. `BOTTOM` maps back
/// to `0` (purrdf "none"); `WILDCARD` has no purrdf encoding and is refused
/// (D4, dual of `purrdf_raw_u32_max_refused`).
#[inline]
pub const fn stardf_to_purrdf_raw(id: u32) -> Result<u32, TermViewError> {
    match id {
        stardf_arrow_core::layout::BOTTOM_TERM_ID => Ok(0),
        stardf_arrow_core::layout::WILDCARD_TERM_ID => Err(TermViewError::WildcardCollision),
        w => Ok(w),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_is_stable_and_starts_at_1() {
        let mut a = StardfArena::with_capacity_bytes(1 << 16).unwrap();
        let x = a.intern(b"ex:s").unwrap();
        let y = a.intern(b"ex:s").unwrap();
        assert_eq!(x, y);
        assert_eq!(x, 1);
        assert_eq!(a.lookup(x), Some(&b"ex:s"[..]));
    }

    #[test]
    fn budget_exhaustion_returns_none() {
        let mut a = StardfArena::with_capacity_bytes(TERM_OVERHEAD + 3).unwrap();
        assert!(a.intern(b"abc").is_some());
        assert_eq!(a.intern(b"toobig"), None);
    }

    #[test]
    fn zero_capacity_is_invalid() {
        assert!(StardfArena::with_capacity_bytes(0).is_none());
    }

    #[test]
    fn quad_charging_hits_budget() {
        // Budget exactly fits one interned 3-byte term (3 + TERM_OVERHEAD)
        // plus one 16-byte quad charge; the second charge must refuse.
        // (The previous constants, TERM_OVERHEAD + 16, were below the
        // intern+quad floor, so even the FIRST charge refused — the test
        // could never pass.)
        let mut a = StardfArena::with_capacity_bytes(TERM_OVERHEAD + 3 + 16).unwrap();
        assert_eq!(a.intern(b"abc"), Some(1));
        assert_eq!(a.used(), TERM_OVERHEAD + 3);
        assert!(a.charge_quad());
        assert_eq!(a.used(), TERM_OVERHEAD + 3 + 16);
        assert!(!a.charge_quad());
    }

    // -- TermId divergence-bridge edges (lane 9) ----------------------------

    use crate::term::{TermKind, TermViewError};

    fn view(raw: u32) -> StardfTermView {
        StardfTermView {
            kind: TermKind::Iri,
            lexical: b"http://example.org/s".to_vec(),
            datatype: None,
            language: None,
            blank_scope: None,
            term_id: raw,
        }
    }

    /// D3: purrdf raw `0` (niche "none") -> stardf `BOTTOM_TERM_ID`.
    #[test]
    fn purrdf_raw_zero_maps_to_bottom() {
        assert_eq!(
            purrdf_raw_to_stardf(0),
            Ok(stardf_arrow_core::layout::BOTTOM_TERM_ID)
        );
        assert_eq!(stardf_arrow_core::layout::BOTTOM_TERM_ID, 0x0000_0000);
        assert_eq!(purrdf_view_to_stardf(&view(0)), Ok(0x0000_0000));
    }

    /// D4: raw `u32::MAX` is stardf `WILDCARD` — refused, both edges, both
    /// directions.
    #[test]
    fn purrdf_raw_u32_max_refused() {
        assert_eq!(
            purrdf_raw_to_stardf(u32::MAX),
            Err(TermViewError::WildcardCollision)
        );
        assert_eq!(
            purrdf_view_to_stardf(&view(u32::MAX)),
            Err(TermViewError::WildcardCollision)
        );
        assert_eq!(
            stardf_to_purrdf_raw(stardf_arrow_core::layout::WILDCARD_TERM_ID),
            Err(TermViewError::WildcardCollision)
        );
        assert_eq!(stardf_arrow_core::layout::WILDCARD_TERM_ID, 0xFFFF_FFFF);
    }

    /// D2: for every mintable purrdf id the raw word (inner = index+1)
    /// converts identically and the stardf arena's own 1-based convention
    /// cancels the offset — dense_index = converted - 1 is preserved.
    #[test]
    fn bridge_view_roundtrip_preserves_dense_index() {
        for dense_index in [0u32, 1, 2, 42, 0xFFFE_FFFE, u32::MAX - 2] {
            let raw = dense_index + 1; // purrdf TermId::raw()
            let stardf = purrdf_raw_to_stardf(raw).expect("mintable id converts");
            assert_eq!(stardf, raw);
            // The inverse recovers the same raw word, hence the same index.
            assert_eq!(stardf_to_purrdf_raw(stardf), Ok(raw));
            assert_eq!(stardf - 1, dense_index);
        }
        // Max mintable purrdf word (index u32::MAX-1) lands just under WILDCARD.
        assert_eq!(purrdf_raw_to_stardf(u32::MAX - 1), Ok(u32::MAX - 1));
    }

    /// D3 inverse: stardf `BOTTOM` maps back to purrdf raw `0`.
    #[test]
    fn bottom_maps_back_to_purrdf_none() {
        assert_eq!(
            stardf_to_purrdf_raw(stardf_arrow_core::layout::BOTTOM_TERM_ID),
            Ok(0)
        );
    }

    /// D6:.BOTTOM matches only itself; WILDCARD matches everything (incl.
    /// BOTTOM) — the pattern-role split the bridge refuses to blur by
    /// remapping either sentinel.
    #[test]
    fn bottom_vs_wildcard_semantics() {
        let matches =
            |pat: u32, term: u32| pat == stardf_arrow_core::layout::WILDCARD_TERM_ID || pat == term;
        assert!(matches(
            stardf_arrow_core::layout::BOTTOM_TERM_ID,
            stardf_arrow_core::layout::BOTTOM_TERM_ID
        ));
        assert!(!matches(stardf_arrow_core::layout::BOTTOM_TERM_ID, 7));
        for term in [0u32, 1, 7, u32::MAX] {
            assert!(matches(stardf_arrow_core::layout::WILDCARD_TERM_ID, term));
        }
    }

    /// Full-view conversion for a literal/blank-shaped view; term_id is the
    /// only field feeding the id conversion, the RDF-value fields ride along
    /// for re-interning (D5: scope stays out of the id).
    #[test]
    fn view_conversion_ignores_value_fields_for_identity() {
        let lit = StardfTermView {
            kind: TermKind::Literal,
            lexical: b"42".to_vec(),
            datatype: Some(b"http://www.w3.org/2001/XMLSchema#integer".to_vec()),
            language: Some(b"en".to_vec()),
            blank_scope: None,
            term_id: 9,
        };
        assert_eq!(purrdf_view_to_stardf(&lit), Ok(9));
        let blank = StardfTermView {
            kind: TermKind::Blank,
            lexical: b"b0".to_vec(),
            datatype: None,
            language: None,
            blank_scope: Some(7),
            term_id: 10,
        };
        assert_eq!(purrdf_view_to_stardf(&blank), Ok(10));
    }
}
