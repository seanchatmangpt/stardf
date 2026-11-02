// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Seam: purrdf-cdt TextDirection re-export needed by the stardf term-model slice.
//! term.rs carries the VERBATIM upstream line (crates/cdt/src/term.rs:33).
//! Pending replacement by full W1 stardf-cdt.

pub mod term;

pub use term::TextDirection;
