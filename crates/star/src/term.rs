
//! Term model and content-addressed quoted-triple ids.
//!
//! DESIGN-AROUND (named law): OST removed RDF-star's ~400% reification
//! overhead by embedding compound terms inside a dictionary encoding.
//! stardf removes it differently: a quoted triple ⟨⟨s p o⟩⟩ is
//! CONTENT-ADDRESSED — its identity is truncate64(BLAKE3(canonical(s,p,o))),
//! so the id IS the digest: first-class, integrity-checkable by
//! construction. No dictionary machinery, no reification inflation. Edge
//! metadata attaches as ordinary triples whose subject is the quoted id.
//!
//! Truncation discipline mirrors stardf's own law
//! (arrow/crates/stardf-gts/src/dict.rs `derive_seed`: BLAKE3 output, first
//! 8 bytes LE as u64). stardf-lex carries no collision/trap handling for
//! truncated digests, so the 64-bit truncation gets the cgap A06 trap
//! treatment adapted: domain-separated preimage, and courts assert
//! injectivity on the live corpus + a 400-triple adversarial set (birthday
//! bound for 64 bits is ~5 billion pairs; a failing injectivity assertion
//! here would indicate a broken derivation, not a birthday hit).

use purrdf_hash::blake3;

/// Domain separator for quoted-triple id preimages.
pub const QUOTED_DOMAIN: &[u8] = b"stardf-star:quoted-triple:v1";

/// A content-addressed quoted-triple identifier:
/// `truncate64(BLAKE3(QUOTED_DOMAIN || canonical(s, p, o)))`.
pub type QuotedId = u64;

/// An RDF term. `Quoted` is a first-class term: it may appear in any
/// subject or object position and carries no side dictionary.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Term {
    Iri(String),
    Blank(String),
    Literal {
        value: String,
        lang: Option<String>,
        datatype: Option<String>,
    },
    /// ⟨⟨s p o⟩⟩ — identified by content, not by storage position.
    Quoted(QuotedId),
}

/// A triple. Subject/predicate/object are plain RDF terms; quoted ids slot
/// in as ordinary terms.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Triple {
    pub s: Term,
    pub p: String,
    pub o: Term,
}

impl Triple {
    pub fn new(s: impl Into<Term>, p: &str, o: impl Into<Term>) -> Self {
        Triple { s: s.into(), p: p.to_string(), o: o.into() }
    }
}

impl From<&str> for Term {
    fn from(v: &str) -> Self {
        Term::Iri(v.to_string())
    }
}

impl From<String> for Term {
    fn from(v: String) -> Self {
        Term::Iri(v)
    }
}

/// Length-prefixed field append — unambiguous framing for canonical bytes.
fn push_field(out: &mut Vec<u8>, field: &[u8]) {
    out.extend_from_slice(&(field.len() as u64).to_le_bytes());
    out.extend_from_slice(field);
}

/// Canonical byte serialization of a term. Quoted terms recurse into their
/// s/p/o so the id of an outer quoted triple covers its whole subtree.
pub fn canonical_term(term: &Term) -> Vec<u8> {
    let mut out = Vec::new();
    match term {
        Term::Iri(v) => {
            out.push(b'I');
            push_field(&mut out, v.as_bytes());
        }
        Term::Blank(v) => {
            out.push(b'B');
            push_field(&mut out, v.as_bytes());
        }
        Term::Literal { value, lang, datatype } => {
            out.push(b'L');
            push_field(&mut out, value.as_bytes());
            push_field(&mut out, lang.as_deref().unwrap_or("").as_bytes());
            push_field(&mut out, datatype.as_deref().unwrap_or("").as_bytes());
        }
        Term::Quoted(id) => {
            // Nested quoted terms canonicalize to their id. Sound because
            // the id is itself content-addressed over the payload preimage
            // (truncate64 over BLAKE3), so id determines payload.
            out.push(b'Q');
            out.extend_from_slice(&id.to_le_bytes());
        }
    }
    out
}

/// Canonical bytes for a quoted triple (s, p, o), domain-separated.
pub fn canonical_quoted(s: &Term, p: &str, o: &Term) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(QUOTED_DOMAIN);
    push_field(&mut out, &canonical_term(s));
    push_field(&mut out, p.as_bytes());
    push_field(&mut out, &canonical_term(o));
    out
}

/// `truncate64(BLAKE3(preimage))` — first 8 bytes, little-endian, per the
/// stardf `derive_seed` discipline cited in the module header.
pub fn quoted_id(s: &Term, p: &str, o: &Term) -> QuotedId {
    let digest = blake3::hash(&canonical_quoted(s, p, o));
    u64::from_le_bytes(*digest.as_bytes().first_chunk().expect("BLAKE3 is 32 bytes"))
}

/// Build a `Term::Quoted` for ⟨⟨s p o⟩⟩. Nested quoted terms inside `s`/`o`
/// are allowed; their ids were fixed when the inner term was built, and the
/// outer id covers the whole subtree because `canonical_quoted` recurses.
pub fn quote(s: Term, p: &str, o: Term) -> Term {
    Term::Quoted(quoted_id(&s, p, &o))
}

/// The human-facing hex of a quoted id (display/debug surfaces).
pub fn quoted_id_hex(id: QuotedId) -> String {
    purrdf_hash::hex::encode(&id.to_le_bytes())
}
