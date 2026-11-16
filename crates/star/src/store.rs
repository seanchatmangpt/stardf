
//! Triple store: ordinary triples, quoted ids as first-class slot terms.
//!
//! A quoted triple's payload (s, p, o) is registered once under its
//! content-addressed id; edge metadata is then ordinary triples whose
//! subject is `Term::Quoted(id)`. Querying metadata is a plain pattern
//! match on the subject term — no dictionary, no reification vocabulary
//! needed on the store path.

use crate::term::{quoted_id_hex, quoted_id, Term, Triple};
use std::collections::BTreeMap;

/// Payload of one quoted triple, recoverable from the registry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuotedPayload {
    pub s: Term,
    pub p: String,
    pub o: Term,
}

/// The store.
#[derive(Default)]
pub struct Store {
    triples: Vec<Triple>,
    /// id -> ⟨⟨s p o⟩⟩ payload; content-addressed, so equal quoted triples
    /// dedupe by construction (same id, one entry).
    quoted: BTreeMap<u64, QuotedPayload>,
}

impl Store {
    pub fn new() -> Self {
        Store::default()
    }

    /// Register a quoted triple ⟨⟨s p o⟩⟩ and return its id term.
    /// Equal payloads map to equal ids (content addressing), so a repeated
    /// registration is idempotent.
    pub fn quote(&mut self, s: Term, p: &str, o: Term) -> Term {
        let id = quoted_id(&s, p, &o);
        self.quoted.entry(id).or_insert(QuotedPayload {
            s, p: p.to_string(), o,
        });
        Term::Quoted(id)
    }

    /// Add an ordinary triple (subject may be a quoted id for metadata).
    pub fn add(&mut self, triple: Triple) {
        self.triples.push(triple);
    }

    pub fn triples(&self) -> &[Triple] {
        &self.triples
    }

    pub fn quoted_count(&self) -> usize {
        self.quoted.len()
    }

    pub fn quoted_payload(&self, id: u64) -> Option<&QuotedPayload> {
        self.quoted.get(&id)
    }

    pub fn quoted_payloads(&self) -> impl Iterator<Item = (u64, &QuotedPayload)> {
        self.quoted.iter().map(|(k, v)| (*k, v))
    }

    /// Integrity check BY CONSTRUCTION: every registered payload must still
    /// derive its own id. Returns the count of id mismatches (0 = sound).
    pub fn verify_integrity(&self) -> usize {
        self.quoted
            .iter()
            .filter(|(id, q)| quoted_id(&q.s, &q.p, &q.o) != **id)
            .count()
    }

    /// Edge metadata: all triples asserted ON the quoted id
    /// (subject = Term::Quoted(id)).
    pub fn metadata_of(&self, id: u64) -> Vec<&Triple> {
        self.triples
            .iter()
            .filter(|t| matches!(&t.s, Term::Quoted(qid) if *qid == id))
            .collect()
    }

    /// Uses-of: triples in which the quoted id appears in object position.
    pub fn used_as_object(&self, id: u64) -> Vec<&Triple> {
        self.triples
            .iter()
            .filter(|t| matches!(&t.o, Term::Quoted(qid) if *qid == id))
            .collect()
    }

    /// Flat projection: read every quoted triple as an ordinary reified
    /// triple set — `qid node`, `rdf:subject`, `rdf:predicate`, `rdf:object`
    /// — the differential baseline the courts compare against. Quoted ids
    /// nested inside payloads expand recursively.
    pub fn flat_projection(&self) -> Vec<Triple> {
        let node = |id: u64| Term::Iri(format!("stardf-star:qid:{}", quoted_id_hex(id)));
        let mut out = Vec::new();
        for (id, q) in &self.quoted {
            let n = node(*id);
            out.push(Triple::new(n.clone(), "http://www.w3.org/1999/02/22-rdf-syntax-ns#subject", q.s.clone()));
            out.push(Triple::new(n.clone(), "http://www.w3.org/1999/02/22-rdf-syntax-ns#predicate", Term::Iri(q.p.clone())));
            out.push(Triple::new(n.clone(), "http://www.w3.org/1999/02/22-rdf-syntax-ns#object", q.o.clone()));
            for t in self.metadata_of(*id) {
                out.push(Triple::new(n.clone(), &t.p, t.o.clone()));
            }
        }
        out
    }
}
