// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! stardf-rdf-core-slice: verbatim term-model slice of PurRDF `crates/rdf-core`
//! at rev `0da0578306983da851a9073337d57e2e2c41ab3a`.
//!
//! Mechanical copy-and-rename extraction per `08-vendoring-plan.md`: package
//! renames only (`purrdf-*` -> `stardf-*`), every vendored file byte-identical
//! to the pinned rev (sha256 receipts in `PROVENANCE.md`). The crate skeleton
//! (this file) is the only new source surface: it declares the vendored modules,
//! carries the root re-exports the vendored files expect from `crate::`, and
//! hosts the `ir` compatibility module that preserves the upstream
//! `crate::ir::term` paths inside the slice layout.
//!
//! Build standing: PARTIAL — the residual closure (crate-internal modules the
//! vendored files reference that are outside the term-model + self-contained
//! closure set) is listed in `PROVENANCE.md`; this crate is a standalone
//! workspace root until the closure lands and it joins the stardf/arrow
//! workspace members.

pub mod blank_label;
pub mod diagnostic;
pub mod model;
pub mod small;
pub mod term;
pub mod term_walk;
// xml_escape lane (2026-10-31): upstream `crates/rdf-core/src/lib.rs` wires
// `pub mod xml_escape;` (its long-standing path); the vendored module file is
// byte-verbatim from purrdf rev 0da05783 (sha256
// 77cde60f6236f461faafeb102f05130896f0beead60d11ac78d3b3ece4fd60d8).
pub mod xml_escape;

/// Upstream path compatibility: upstream these types live at
/// `crate::ir::term`; in the slice the term model is at `crate::term`, and the
/// vendored files' `crate::ir::term::...` paths resolve through this module.
pub mod ir {
    pub use crate::term;
}

pub use diagnostic::RdfLocation;
pub use model::{RdfLiteral, RdfTerm, RdfTermKind, RdfTextDirection, RdfTriple};
pub use purrdf_lex::walk::Nested;
pub use term::{BlankScope, TermId, TermValue};
pub use term_walk::{TermBox, try_fold_nested, visit_nested};
