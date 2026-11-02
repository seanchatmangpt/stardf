// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Seam: purrdf-hash digest+frame surfaces needed by the stardf term-model slice.
//! digest.rs + frame.rs are VERBATIM from purrdf rev 0da05783
//! (crates/hash/src/{digest,frame}.rs). Pending replacement by full W1 stardf-hash.

pub mod digest;
pub mod error;
mod impls;
pub mod frame;

pub use digest::{Digest, MAX_OUTPUT_LEN};
