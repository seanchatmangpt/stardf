// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Umbrella Rust API for PurRDF.
//!
//! # STARDF RE-DERIVATION BANNER (slice 11, rev 0da05783 base)
//!
//! This is the re-derived copy-sweep facade: the package is `stardf`, the lib
//! name stays verbatim `purrdf`. The upstream module map at the pinned rev
//! carries 27 modules over ~31 sub-crates; this copy carries the modules whose
//! backing crates are resolvable members of the copy-sweep workspace, and only
//! those. The authoritative upstream map is the pinned rev's
//! `crates/purrdf/src/lib.rs`; everything this copy does not yet carry is
//! parked with a port precondition in `crates/stardf/VENDOR-PROVENANCE.md`
//! (never dropped silently).
//!
//! ## Carried now (4 modules + the root error binding)
//!
//! | Module | Sub-crate(s) |
//! |---|---|
//! | [`sparql`] | [`purdfe_sparql_algebra`] — parser + algebra leg |
//! | [`xsd`] | [`purdff_xsd`] |
//! | [`iri`] | [`purdfe_iri`] |
//! | [`events`] | [`purdff_events`] |
//!
//! [`purdff_xsd`]: purrdf_xsd
//! [`purdff_events`]: purrdf_events

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/Blackcat-Informatics/purrdf/main/docs/purrdf-logo.svg"
)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/Blackcat-Informatics/purrdf/main/docs/purrdf-logo.svg"
)]
#![forbid(unsafe_code)]

/// Borrowed typed error inspection, available without a sub-crate dependency.
pub use purrdf_lex::error;

/// SPARQL 1.1/1.2: parser + algebra ([`purdfe_sparql_algebra`]) — the parser
/// and algebra leg of the upstream facade's [`sparql`] module. The evaluator
/// ([`purdfe_sparql_eval`]) and results serialization
/// ([`purdfe_sparql_results`]) legs are parked until their crates land
/// (VENDOR-PROVENANCE.md item 5).
pub mod sparql {
    pub use purrdf_sparql_algebra::*;
    // The algebra crate exposes an `error` module; bind it explicitly so the
    // bare name survives the glob (the upstream two-glob ambiguity note is
    // parked with the eval/results legs).
    pub use purrdf_sparql_algebra::error;
    pub use purrdf_sparql_algebra::TermPattern;
}

/// XSD datatype value spaces and operations.
pub mod xsd {
    pub use purrdf_xsd::*;
}

/// IRI parsing, resolution, and CURIE expansion/contraction.
pub mod iri {
    pub use purrdf_iri::*;
}

/// The zero-dependency streaming RDF event model.
pub mod events {
    pub use purrdf_events::*;
}

// First-party court for the re-derived module map — NOT vendored upstream
// text. The upstream `#[cfg(test)] mod tests` exercised the full upstream
// surface (rdf, shapes, shex, markdown, text, retrieval, gts, columnar) and is
// parked with its modules (VENDOR-PROVENANCE.md item 4); this court keeps the
// umbrella's test gate non-vacuous at the re-derived surface: every carried
// module must resolve through the facade, and the foundations must answer.
// Falsifier: a module dropped from the map without re-deriving this file fails
// to compile; a carried module whose surface does not answer fails the
// assertions below.
#[cfg(test)]
mod rederived_facade_court {
    #[test]
    fn rederived_module_map_resolves_and_answers() {
        assert!(
            crate::iri::parse("https://example.org/x").is_ok(),
            "the iri module answers through the facade"
        );
        assert_ne!(
            format!("{:?}", crate::events::TextDirection::Ltr),
            "",
            "the events module answers through the facade"
        );
        // Compile-time map proof for the remaining carried bindings: these
        // paths resolve only while the re-derived map carries them.
        #[allow(unused_imports)]
        use crate::{error, sparql, xsd};
    }
}
