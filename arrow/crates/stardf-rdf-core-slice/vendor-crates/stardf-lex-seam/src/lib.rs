// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Seam: purrdf-lex allocation(+submodules)+constructors+text_out+walk surfaces
//! needed by the stardf term-model slice. All files VERBATIM from purrdf
//! rev 0da05783 (crates/lex/src/...). Pending replacement by full W1 stardf-lex.

pub mod allocation;
pub mod constructors;
pub mod text_out;
pub mod walk;

pub use purrdf_hash::error;
