
//! stardf-star: RDF 1.2 quoted triples ⟨⟨s p o⟩⟩ as CONTENT-ADDRESSED
//! NESTED SLOTS — a different design-around from OST's inline-dictionary
//! compound terms. See `term.rs` for the named law and `docs/ost/
//! quoted-triples.md` for the receipt.

pub mod parser;
pub mod store;
pub mod term;

pub use parser::parse;
pub use store::{QuotedPayload, Store};
pub use term::{quote, quoted_id, quoted_id_hex, canonical_quoted, canonical_term, Term, Triple, QuotedId, QUOTED_DOMAIN};
