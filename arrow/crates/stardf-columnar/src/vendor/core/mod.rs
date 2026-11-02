// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! In-crate stand-ins for the purrdf substrate crates (see PROVENANCE.md).

pub mod backend;
pub mod bits;
pub mod content;
pub mod dataset;
pub mod diagnostic;
pub mod langtag;
pub mod model;
pub mod term_fixture;
pub mod vocab;

/// Upstream path parity shim: `purrdf_core::ir::pack::bits`.
pub mod ir {
    /// Pack layer.
    pub mod pack {
        pub use super::super::bits;
    }
}

pub use content::{ContentDigest, ContentStore, ContentStoreError};
pub use dataset::{DatasetView, LossLedger, QuadIds, RdfDataset, RdfDatasetBuilder, ViewTermId};
pub use diagnostic::RdfDiagnostic;
pub use model::{
    BlankScope, RdfLiteral, RdfTextDirection, TermBox, TermFactory, TermGuard, TermRef, TermValue,
};

/// `purrdf_core::langtag` seam.
pub mod langtag_seam {
    pub use super::langtag::{is_identity_folded, is_well_formed_with, Profile};
}
