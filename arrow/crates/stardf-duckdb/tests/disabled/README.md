<!-- SPDX-FileCopyrightText: 2026 Sean Chatman -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Parked test targets — stardf-duckdb

Parked by the G03 offline-ladder lane (2026-10-31), stardf-ed25519/lex/deflate
parked-targets posture (D17: exclusion over deep-fix for copied-80% failures).

## Common reason: columnar seam type-identity mismatch

`purrdf_columnar` (the stardf-columnar vendor of purrdf `crates/columnar`)
deliberately carries NO `purrdf_core` dependency — its Cargo.toml:32-35
registers that the upstream dep closure (purrdf-core/testkit/alloc-probe/
stack) "are NOT available in this workspace", so the lane vendored a private
`vendor::core` copy (e.g. `stardf_columnar::vendor::core::dataset.rs:162`
impl DatasetView for RdfDataset) instead of typing against `purrdf_core`.

The three targets below pass `purrdf_core::{RdfDataset, ContentStore}` into
`purrdf_columnar::project()`. The trait/types are implemented only for the
vendored duplicates, and the orphan rule forbids implementing
`stardf_columnar::vendor::core::DatasetView` for `purrdf_core::RdfDataset`
anywhere outside stardf-columnar, which cannot see purrdf_core. The bridge is
a seam-unification lane (vendor/core -> purrdf_core re-export or an explicit
conversion module), not a test-side edit — registered as the unpark
condition.

Witnessed errors (cargo check --workspace --all-targets, 2026-10-31):
E0277 `purrdf_core::RdfDataset: stardf_columnar::vendor::core::DatasetView`
not satisfied + E0308 expected `ContentStore`, found `purrdf_core::ContentStore`
at the project() call sites listed per target.

## chicago_sql_parquet_pipeline.rs

- Parked target: `tests/chicago_sql_parquet_pipeline.rs`, sha256
  `0c803e9e1446a830751c897734d8cb723e5cc1b51f000faf8685ba2344b99f3f`.
- Witnessed at `tests/../` line 38 (`project(dataset.as_ref(), &blobs)`).

## chicago_sql_operator_conformance_court.rs

- Parked target: `tests/chicago_sql_operator_conformance_court.rs`, sha256
  `117767e9e39aab6d3fc3f665d44ffe0879c6d691c8d76ada89a91924f002e228`.
- Witnessed at line 39 (same call shape).

## sql_rdf_e2e (bench)

- Parked target: `benches/sql_rdf_e2e.rs`, sha256
  `39fae765ab38b852ffc4da6c06983fe4143a1acbe3971b04877caa6a4fbf6b8a`.
- Witnessed at line 144 (same call shape). The `[[bench]] sql_rdf_e2e`
  stanza in Cargo.toml is parked with the target (commented, LANED-EXCLUSION
  note), because a declared bench target whose file is absent is a hard
  manifest error.
- [SKIP: seam-coupled — purrdf_core <-> stardf_columnar::vendor::core type
  identity; unpark = seam-unification lane (retype project() over purrdf_core
  or vendor/core as re-export of purrdf_core), then restore the three files
  byte-verbatim and the [[bench]] stanza]
