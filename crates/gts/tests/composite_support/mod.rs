// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Dedicated composite fixture keys shared by public integration tests.

// LINT-GATE (G03 offline ladder, 2026-10-31, D17): this shared fixture module
// is compiled into several test crates outside the stardf-gts lib gate.
#[allow(clippy::redundant_pub_crate)]
pub(crate) use purrdf_gts::fixture::fixed_composite_key as composite_key;
