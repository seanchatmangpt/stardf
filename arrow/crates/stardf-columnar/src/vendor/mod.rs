// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! In-crate stand-ins for the purrdf substrate crates. See PROVENANCE.md for
//! the exact provenance and the disclosed divergences.
//!
//! The shims carry deliberate unused surface (vocabulary constants, visitor
//! accessors kept for upstream call-shape parity); that dead code is declared
//! here rather than scattered per-item.
#![allow(dead_code)]

#[macro_use]
pub mod hash;
#[macro_use]
pub mod lex;
// The crate's single sanctioned unsafe surface (the counting GlobalAlloc)
// lives here; see src/lib.rs and PROVENANCE.md for the disclosed
// forbid->deny divergence.
#[allow(unsafe_code)]
pub mod alloc_probe;
pub mod core;
pub mod testkit;
