// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Seam: purrdf-testkit's `rng` surface needed by the xml_escape module's
//! vendored tests (`SplitMix64` deterministic stream). rng.rs is VERBATIM from
//! purrdf rev `0da0578306983da851a9073337d57e2e2c41ab3a`
//! (`crates/testkit/src/rng.rs`). Pending replacement by the full W1-vendored
//! stardf-testkit (its jsonschema metaschema include_str! surface references a
//! not-yet-vendored `crates/jsonschema` tree — 17 include_str! failures
//! witnessed 2026-10-31, see the xml_escape lane receipt).

pub mod rng;
