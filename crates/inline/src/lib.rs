//! stardf-inline: bit-63 structural folding of RDF 1.2 quoted triples.
//!
//! DESIGN-AROUND (named law): the paper proposes folding a quoted triple
//! into one machine word; the WITNESSED representation in this repo is
//! stardf-star's content-addressed `truncate64(BLAKE3(...))` ids. This
//! crate implements the folding variant for real: bit 63 = discriminator
//! (folded vs plain word), bits 0..62 = structural fingerprint, payload
//! in an arena, and a MANDATORY collision trap on any fingerprint clash.
//! See `docs/ost/inline-folding.md` for the collision analysis.

pub mod arena;
pub mod fold;
pub mod store;

pub use arena::{Arena, Payload, Slot};
pub use fold::{fold_fingerprint, fold_word, Folded, DISCRIMINATOR_BIT, FOLD_DOMAIN};
pub use store::{FoldError, InlineStore, Trap};
