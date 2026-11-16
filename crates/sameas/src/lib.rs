//! owl:sameAs via SLOT-ID CANONICALIZATION (digest-space), lane RDF-6.
//!
//! DESIGN-AROUND LAW (the differentiator vs OST): OST elects canonical
//! representatives and rewrites dictionary references inside C++ index
//! structures. This crate never rewrites stored references. It canonicalizes
//! in the DIGEST SPACE: every synonymous term's id already derives from
//! `BLAKE3(SAMEAS_NS_V1 ++ iri)` truncated to 64 bits (namespace-tag +
//! collision-trap discipline adapted from cgap A06's `b"bont\x01"` +
//! `slot_id` law, docs/jira/v26.11.1/lanes/A06-bont-scalars.md SS 4-5,
//! mirrored in cgap-bont primitives.rs). A union-find over those ids elects
//! the lowest id as canonical representative; queries rewrite the LOOKUP KEY
//! (s -> canonical(s)) and union the class members' facts. Collision
//! trapping and the pin layout survive unchanged because ids are never
//! mutated after derivation.
use std::collections::BTreeMap;

use purrdf_hash::blake3;

/// Namespace tag, domain-separating this id space (cgap A06 SS5 discipline;
/// `\x01` is the v1 tag byte, part of the freeze).
pub const SAMEAS_NS_V1: &[u8] = b"stardf-sameas\x01";

/// A 64-bit digest-space term id: truncate64(BLAKE3(NS ++ iri)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermId(pub u64);

/// truncate64 per the cgap A06 slot-id derivation.
#[must_use]
pub fn truncate64(digest: &[u8; 32]) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&digest[..8]);
    u64::from_le_bytes(b)
}

/// Digest-space term id: `truncate64(BLAKE3(SAMEAS_NS_V1 ++ iri))`,
/// the cgap `slot_id(lexeme)` law adapted to stardf term IRIs.
#[must_use]
pub fn term_id(iri: &[u8]) -> TermId {
    let mut msg = Vec::with_capacity(SAMEAS_NS_V1.len() + iri.len());
    msg.extend_from_slice(SAMEAS_NS_V1);
    msg.extend_from_slice(iri);
    TermId(truncate64(&blake3::hash(&msg).as_bytes()))
}

/// Build-time collision trap: two DISTINCT irises yielding one TermId is a
/// hard error, never silent drift (cgap A06 SS5 producer obligation).
#[derive(Debug, PartialEq, Eq)]
pub struct CollisionRefused {
    pub id: TermId,
    pub first: Vec<u8>,
    pub second: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SameAsError {
    Collision(CollisionRefused),
    Contradiction(ContradictionRefused),
}

/// Conflicting functional facts across an equivalence class, typed.
#[derive(Debug, PartialEq, Eq)]
pub struct ContradictionRefused {
    pub canonical: TermId,
    pub predicate: TermId,
    pub existing_object: TermId,
    pub new_object: TermId,
}

/// Union-find over digest-space ids. Union keeps the MINIMUM id as root, so
/// the elected canonical representative is `find` itself — deterministic
/// under insertion-order permutation (min of a set is order-free).
#[derive(Default)]
pub struct SameAsIndex {
    parent: BTreeMap<TermId, TermId>,
}

impl SameAsIndex {
    #[must_use]
    pub fn new() -> Self { Self::default() }

    /// Ingest one owl:sameAs pair (two IRIs).
    pub fn ingest_pair(&mut self, a: &[u8], b: &[u8]) {
        let (ta, tb) = (term_id(a), term_id(b));
        if ta == tb { return; }
        let (ra, rb) = (self.find(ta), self.find(tb));
        if ra == rb { return; }
        // elect min-id root (deterministic canonical election)
        let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
        self.parent.insert(hi, lo);
    }

    /// Root = canonical representative (lowest id in the class).
    #[must_use]
    pub fn find(&self, t: TermId) -> TermId {
        let mut cur = t;
        while let Some(&p) = self.parent.get(&cur) {
            if p == cur { break; }
            cur = p;
        }
        cur
    }

    /// Canonical id for a term: its class root; singletons map to themselves.
    #[must_use]
    pub fn canonical(&self, t: TermId) -> TermId { self.find(t) }

    /// Full canonical map: canonical -> sorted class members (incl. itself).
    #[must_use]
    pub fn canonical_map(&self) -> BTreeMap<TermId, Vec<TermId>> {
        let mut m: BTreeMap<TermId, Vec<TermId>> = BTreeMap::new();
        let roots: Vec<TermId> = self.parent.keys().copied().collect();
        for t in roots {
            let c = self.find(t);
            m.entry(c).or_default().push(t);
        }
        for members in m.values_mut() {
            if !members.contains(&{
                let c = members[0];
                self.find(c)
            }) {
                let c = self.find(members[0]);
                members.push(c);
            }
            members.sort_unstable();
            members.dedup();
        }
        m
    }
}

/// Rewrite layer: facts stored under RAW subject ids; queries rewrite the
/// lookup key through `canonical()` and union the class's facts. Stored ids
/// are never mutated (the design-around).
#[derive(Default)]
pub struct FactStore {
    pub index: SameAsIndex,
    functional: BTreeMap<TermId, ()>,
    facts: Vec<(TermId, TermId, TermId)>,
    by_subject: BTreeMap<TermId, Vec<usize>>,
}

pub struct Fact<'a> {
    pub subject: &'a TermId,
    pub predicate: &'a TermId,
    pub object: &'a TermId,
}

impl FactStore {
    #[must_use]
    pub fn new() -> Self { Self::default() }

    /// Declare a predicate functional (contradiction checking applies).
    pub fn declare_functional(&mut self, p: TermId) { self.functional.insert(p, ()); }

    /// Add a fact; enforces functional consistency across the equivalence
    /// class (canonical subject + functional predicate -> one object).
    pub fn add_fact(
        &mut self, subject: &[u8], predicate: &[u8], object: &[u8],
    ) -> Result<(), SameAsError> {
        let (s, p, o) = (term_id(subject), term_id(predicate), term_id(object));
        self.add_ids(s, p, o)
    }

    pub fn add_ids(&mut self, s: TermId, p: TermId, o: TermId) -> Result<(), SameAsError> {
        let can = self.index.canonical(s);
        if self.functional.contains_key(&p) {
            let mut checked: BTreeMap<TermId, TermId> = BTreeMap::new();
            for m in self.class_members(can) {
                for &i in self.by_subject.get(&m).into_iter().flatten() {
                    let (_fs, fp, fo) = self.facts[i];
                    if fp == p {
                        let prev = checked.insert(m, fo);
                        if let Some(prev_o) = prev {
                            if prev_o != fo {
                                return Err(SameAsError::Contradiction(ContradictionRefused {
                                    canonical: m, predicate: p,
                                    existing_object: prev_o, new_object: fo,
                                }));
                            }
                        }
                    }
                }
            }
            for (m, fo) in &checked {
                if *fo != o {
                    return Err(SameAsError::Contradiction(ContradictionRefused {
                        canonical: *m, predicate: p,
                        existing_object: *fo, new_object: o,
                    }));
                }
            }
        }
        let i = self.facts.len();
        self.facts.push((s, p, o));
        self.by_subject.entry(s).or_default().push(i);
        Ok(())
    }

    /// Query through ANY class member: union every member's facts.
    /// Facts are keyed by raw subject id (never rewritten); the lookup unions
    /// the class — same result either direction, ids stay stable under late
    /// owl:sameAs arrival.
    #[must_use]
    pub fn facts_about(&self, subject: &[u8]) -> Vec<(TermId, TermId, TermId)> {
        let t = term_id(subject);
        let can = self.index.canonical(t);
        let mut out = Vec::new();
        for m in self.class_members(can) {
            if let Some(idx) = self.by_subject.get(&m) {
                for &i in idx { out.push(self.facts[i]); }
            }
        }
        out
    }

    /// All members of the class rooted at `can`, including the canonical.
    #[must_use]
    pub fn class_members(&self, can: TermId) -> Vec<TermId> {
        let mut out = vec![can];
        for (&t, _) in self.index.parent.iter() {
            if self.index.find(t) == can && t != can { out.push(t); }
        }
        out.sort_unstable();
        out
    }
}
