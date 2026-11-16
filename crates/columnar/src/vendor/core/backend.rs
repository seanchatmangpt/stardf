// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Path-parity shim for `purrdf_core::backend`.
//!
//! The interning algebra already lives in the term-model shim
//! ([`super::model::TermFactory`], disclosed there as a divergence); this
//! module preserves the upstream module path `backend::TermFactory` so the
//! vendored call sites import through their original route.

pub use super::model::TermFactory;
