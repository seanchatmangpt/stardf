// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! PurRDF's first-party, byte-deterministic Parquet bridge.
//!
//! The codec projects an RDF 1.2 [`crate::vendor::core::DatasetView`] plus a
//! [`crate::vendor::core::ContentStore`] into the fixed five-table schema exposed by
//! [`schema`]. Its deliberately narrow Parquet subset keeps the implementation
//! wasm-clean and auditable: `INT64`/`BYTE_ARRAY` columns, flat `OPTIONAL` fields,
//! PLAIN values, RLE definition levels, Data Page V2, and UNCOMPRESSED or ZSTD
//! page bodies.
//!
//! No vocabulary is built into this crate. Every IRI is carried from the caller's
//! dataset verbatim.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/Blackcat-Informatics/purrdf/main/docs/purrdf-logo.svg"
)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/Blackcat-Informatics/purrdf/main/docs/purrdf-logo.svg"
)]
// Lane 1 (2026-10-31), disclosed divergence: `deny` rather than `forbid` so
// vendor/alloc_probe's real counting GlobalAlloc (over System) can carry the
// one unavoidable unsafe impl, scoped behind a module-level allow. Every new
// unsafe site still fails the build; see PROVENANCE.md.
#![deny(unsafe_code)]

#[cfg(feature = "upstream-tests")]
mod upstream_tests_note {
    //! Upstream test modules are preserved behind `--features upstream-tests`
    //! and additionally require the purrdf substrate crates (unavailable in
    //! this workspace); see PROVENANCE.md.
}
// LANE-14 UNBLOCK (disclosed): `pub` so external integration tests can
// implement the vendored `DatasetView` trait over their own storage
// (`stardf-arrow-capi/tests/columnar_integration.rs` needs to name
// `stardf_columnar::vendor::core::DatasetView` and its associated types).
mod column;
mod compact;
pub mod error;
mod files;
mod parquet;
#[doc(hidden)]
pub mod plain_bench;
mod projection;
mod reader;
// Lane 16 (2026-10-31): the materialized SPARQL result model (SparqlResult,
// DatasetHandle, SolutionRow) + the RdfDatasetBuilder, vendored per the D12
// slice admission; see src/result.rs for provenance + divergences.
pub mod result;
pub mod schema;
#[cfg(test)]
mod smoke;
pub mod vendor;
mod writer;

pub use error::{
    ColumnarError, ColumnarWriteError, CompactReason, ConstructionReason, MetadataReason,
    PageBodyReason, ProjectionReason, ReaderReason, ValueBodyReason,
};
pub use files::ParquetFiles;
pub use parquet::Compression;
pub use projection::{ColumnarProjection, ProjectionCell, ProjectionRows};
pub use reader::{read, ColumnarRead};
pub use result::{DatasetHandle, RdfDatasetBuilder, SolutionRow, SparqlResult};
pub use schema::{ColumnSchema, PhysicalType, Repetition, Table, TableSchema};
pub use writer::{project, write, ColumnarWrite};
