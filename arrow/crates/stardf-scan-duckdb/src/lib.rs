// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native in-process `stardf_scan()` DuckDB table function over the closed
//! five-table columnar profile.
//!
//! Lane 02-scan-scaffold (2026-10-31): W2a–W2d of `docs/SCAN-COMPLETION.md`
//! §4 — `bind` / `init` / `func` + in-process registration over the
//! frozen-duckdb `VTab` trait. Result columns are derived mechanically from
//! the closed five-table schema ([`stardf_columnar::schema`]): INT64 →
//! BIGINT, BYTE_ARRAY+UTF8 → VARCHAR, BYTE_ARRAY → BLOB. Rows are
//! materialized at bind from the receipted validated reader
//! ([`stardf_columnar::read`]) — full physical + semantic profile validation
//! runs before any row is emitted.
//!
//! # Surface
//!
//! `stardf_scan(root VARCHAR, table VARCHAR)`:
//!
//! - positional parameter: the dataset root directory holding the five
//!   Parquet files (canonical filenames per the columnar schema);
//! - named parameter `table` (VARCHAR): selects one of the closed five-table
//!   set `terms | quads | reifiers | annotations | blobs`.
//!
//! # Divergences (disclosed)
//!
//! - `supports_pushdown()` stays `false` — the frozen-duckdb 2.0.4 SDK
//!   exposes no complex-filter pushdown surface; registered per F05. The C3
//!   flip court (SCAN-COMPLETION §5) owns the before/after when it lands.
//! - The named parameter `table` is REQUIRED: a one-relation table function
//!   binds exactly one closed-schema relation per invocation, so the absent
//!   `table` default is a typed refusal enumerating the five names, never a
//!   silent invented union surface.
//! - Rows are materialized at bind from the public reader because the
//!   codec's raw row surfaces (`read_table`/`TableData`) are `pub(crate)`.
//!   Bind-time materialization duplicates blob payloads once; the zero-copy
//!   emission path is the later allocation-identity rider, not a W2 gate.
//! - Parallel emission (`set_max_threads` + per-thread split, W2b) is
//!   deferred; the [`AtomicUsize`] cursor is already Send+Sync and
//!   claim-based, so the per-thread split is an additive change.
//! - `#![forbid(unsafe_code)]`: the only unsafe edge is the SDK's own
//!   `extern "C"` shims inside frozen-duckdb. VARCHAR cells carrying an
//!   interior NUL byte take a typed [`ScanError`] refusal through the
//!   trait's error channel, never an SDK `CString` panic across the FFI.

#![forbid(unsafe_code)]
// Root [workspace.lints] carries clippy pedantic/nursery as warnings (which
// the CI ladder promotes via -D warnings). Targeted opt-outs with reasons,
// the same posture as the vendored first-class manifests:
#![allow(
    // Public constructors are marked #[must_use] by hand where they guard
    // bugs; a blanket candidate pass would demand it on every getter.
    clippy::must_use_candidate,
    // `stardf_scan_*` names repeat the crate name by contract, not by
    // accident.
    clippy::module_name_repetitions,
    // Spec vocabulary (DuckDB, Parquet, RDF 1.2, BIGINT/VARCHAR/BLOB) is
    // deliberate prose, not identifiers; the vendored first-class manifests
    // carry the same opt-out with the same reason.
    clippy::doc_markdown
)]

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use frozen_duckdb::duckdb::Connection;
use frozen_duckdb::duckdb::core::{
    DataChunkHandle, FlatVector, Inserter, LogicalTypeHandle, LogicalTypeId,
};
use frozen_duckdb::duckdb::vtab::{BindInfo, InitInfo, TableFunctionInfo, VTab};
use stardf_columnar::schema::{PhysicalType, Table};
use stardf_columnar::vendor::core::{RdfTextDirection, TermValue};
use stardf_columnar::{ColumnarError, ColumnarRead, ParquetFiles, read};

/// Bounded emission slab: rows claimed per `func` invocation, additionally
/// capped by the output chunk's own vector capacity (`duckdb_vector_size()`).
const SLAB_ROWS: usize = 2048;

/// The closed five-table logical names, in canonical order.
const TABLE_NAMES: [&str; 5] = ["terms", "quads", "reifiers", "annotations", "blobs"];

// ---------------------------------------------------------------------------
// Typed scan-surface refusal (the `ColumnarError` reason-enum posture)
// ---------------------------------------------------------------------------

/// A typed scan-surface refusal: every bind/func refusal carries a stable
/// context and a structured reason, never a panic across the C ABI.
///
/// The SDK converts the trait error channel into `set_error(string)`;
/// [`Display`](fmt::Display) renders `stardf_scan <context>: <reason>` so
/// the typed reason survives the FFI as a prefix-coded message.
#[derive(Debug)]
pub struct ScanError {
    /// The surface that refused (e.g. `bind`, `bind.table`, `func.terms`).
    context: &'static str,
    /// The structured reason.
    reason: ScanReason,
}

/// The per-surface refusal reasons.
#[derive(Debug)]
pub enum ScanReason {
    /// The positional root parameter is absent or malformed.
    Parameter(String),
    /// The required `table` named parameter is absent.
    TableRequired {
        /// The closed five-table names, canonical order.
        available: Vec<&'static str>,
    },
    /// A table name outside the closed five-table set.
    TableUnknown {
        /// The requested name.
        requested: String,
        /// The closed five-table names, canonical order.
        available: Vec<&'static str>,
    },
    /// A dataset file could not be read from the root directory.
    DatasetIo {
        /// The Parquet file name.
        file: &'static str,
        /// The OS error.
        source: std::io::Error,
    },
    /// The five-table profile failed the receipted reader validation.
    Profile(ColumnarError),
    /// A VARCHAR cell carried an interior NUL byte.
    InteriorNul {
        /// The logical `table.column` context.
        column: &'static str,
    },
    /// An internal invariant failed while rendering a row (e.g. a
    /// reconstructed triple term's component id absent from the term index).
    /// Fail-closed: the query errors, it never emits a wrong row.
    Render(String),
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stardf_scan {}: ", self.context)?;
        match &self.reason {
            ScanReason::Parameter(message) | ScanReason::Render(message) => {
                write!(f, "{message}")
            }
            ScanReason::TableRequired { available } => write!(
                f,
                "the required named parameter `table` is absent; one of {}",
                available.join(", ")
            ),
            ScanReason::TableUnknown {
                requested,
                available,
            } => write!(
                f,
                "unknown table '{requested}'; one of {}",
                available.join(", ")
            ),
            ScanReason::DatasetIo { file, .. } => write!(f, "reading {file} failed"),
            ScanReason::Profile(error) => write!(f, "profile validation refused: {error}"),
            ScanReason::InteriorNul { column } => write!(
                f,
                "VARCHAR cell in {column} carries an interior NUL byte; \
                 refusing instead of an SDK CString panic"
            ),
        }
    }
}

impl std::error::Error for ScanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.reason {
            ScanReason::DatasetIo { source, .. } => Some(source),
            // ColumnarError implements Display but not std::error::Error, so
            // it is carried by value and rendered through Display above.
            _ => None,
        }
    }
}

impl ScanError {
    /// A scan-surface refusal from its parts.
    pub const fn new(context: &'static str, reason: ScanReason) -> Self {
        Self { context, reason }
    }
}

// ---------------------------------------------------------------------------
// Materialized scan rows
// ---------------------------------------------------------------------------

/// The materialized rows of one bound table.
///
/// Everything is resolved at bind into plain owned data, so `func` is a pure
/// slab copy: quad/reifier/annotation id rows are `Copy`, the terms table
/// renders from the dataset's dense term table (id `r + 1` is Parquet
/// terms.id, per the reader's dense-id contract), and blob payloads are
/// paired with their lowercase hex digests once.
#[derive(Debug)]
enum ScanRows {
    /// Render from `dataset.terms()` by row index.
    Terms,
    /// Quads in canonical (decoded) row order.
    Quads(Vec<QuadRow>),
    /// Reifiers in canonical (decoded) row order, triple-term components
    /// already decomposed to wire ids.
    Reifiers(Vec<ReifierRow>),
    /// Annotations in canonical (decoded) row order.
    Annotations(Vec<QuadRow>),
    /// Blobs in digest order: `(lowercase hex digest, payload)`.
    Blobs(Vec<(String, Vec<u8>)>),
}

/// A plain id row `(s, p, o, graph)`.
type QuadRow = (u32, u32, u32, Option<u32>);

/// A reifier row `(reifier, s, p, o, graph)` with the bound triple term's
/// components already decomposed to wire ids.
type ReifierRow = (u32, u32, u32, u32, Option<u32>);

/// The bind data: the validated dataset, the per-table derived index
/// surfaces, and the materialized rows of the selected table.
#[derive(Debug)]
pub struct StardfScanBindData {
    /// The bound table.
    table: Table,
    /// Exact row count of the bound table.
    cardinality: usize,
    /// The validated dataset + verified blob store.
    read: ColumnarRead,
    /// Dense ids (1-based) declared as named graphs.
    named_graphs: HashSet<u32>,
    /// Term value → wire id (1-based), over the canonical dictionary.
    term_index: BTreeMap<TermValue, u32>,
    /// The materialized rows.
    rows: ScanRows,
}

/// The init data: a shared claim cursor for bounded slab emission.
///
/// Global across threads (`Send + Sync`): `func` claims a bounded row slab
/// through `compare_exchange`, so a later per-thread split needs no cursor
/// change.
#[derive(Debug)]
pub struct StardfScanInitData {
    /// The next unclaimed row.
    next_row: AtomicUsize,
}

// ---------------------------------------------------------------------------
// The VTab
// ---------------------------------------------------------------------------

/// The `stardf_scan()` table function.
#[derive(Debug)]
pub struct StardfScanVTab;

impl VTab for StardfScanVTab {
    type InitData = StardfScanInitData;
    type BindData = StardfScanBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn std::error::Error>> {
        // Emit the result columns mechanically from the closed columnar
        // schema: physical INT64 → BIGINT, BYTE_ARRAY+UTF8 → VARCHAR,
        // BYTE_ARRAY → BLOB.
        let table = bind_table(bind)?;
        for column in table.schema().columns {
            let logical = match column.physical_type {
                PhysicalType::Int64 => LogicalTypeId::Bigint,
                PhysicalType::ByteArray if column.utf8 => LogicalTypeId::Varchar,
                PhysicalType::ByteArray => LogicalTypeId::Blob,
            };
            bind.add_result_column(column.name, LogicalTypeHandle::from(logical));
        }
        let bind_data = bind_inner(bind, table)?;
        bind.set_cardinality(
            u64::try_from(bind_data.cardinality).unwrap_or(u64::MAX),
            true,
        );
        Ok(bind_data)
    }

    fn init(_init: &InitInfo) -> Result<Self::InitData, Box<dyn std::error::Error>> {
        Ok(StardfScanInitData {
            next_row: AtomicUsize::new(0),
        })
    }

    fn func(
        func: &TableFunctionInfo<Self>, output: &mut DataChunkHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bind_data = func.get_bind_data();
        let init_data = func.get_init_data();
        let capacity = output.flat_vector(0).capacity().clamp(1, SLAB_ROWS);
        // Claim a bounded slab; DuckDB re-invokes `func` until a 0-length
        // chunk is returned.
        let (start, end) = loop {
            let claimed = init_data.next_row.load(Ordering::Relaxed);
            if claimed >= bind_data.cardinality {
                output.set_len(0);
                return Ok(());
            }
            let end = (claimed + capacity).min(bind_data.cardinality);
            if init_data
                .next_row
                .compare_exchange(claimed, end, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break (claimed, end);
            }
        };
        let mut columns: Vec<_> = (0..bind_data.table.schema().columns.len())
            .map(|index| output.flat_vector(index))
            .collect();
        let count = end - start;
        for (local, row) in (start..end).enumerate() {
            match &bind_data.rows {
                ScanRows::Terms => render_terms(bind_data, row, &mut columns, local)?,
                ScanRows::Quads(rows) | ScanRows::Annotations(rows) => {
                    render_quad_row(rows[row], &mut columns, local);
                }
                ScanRows::Reifiers(rows) => {
                    render_reifier_row(rows[row], &mut columns, local);
                }
                ScanRows::Blobs(rows) => render_blob_row(&rows[row], &mut columns, local)?,
            }
        }
        output.set_len(count);
        Ok(())
    }

    /// W2e divergence: stays `false` — the 2.0.4 SDK exposes no
    /// complex-filter pushdown; the C3 flip court owns the flip (F05).
    fn supports_pushdown() -> bool {
        false
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        Some(vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)])
    }

    fn named_parameters() -> Option<Vec<(String, LogicalTypeHandle)>> {
        Some(vec![(
            "table".to_owned(),
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        )])
    }
}

/// Registers `stardf_scan` on the connection (the W2d in-process path,
/// `vtab/mod.rs` `register_table_function`).
///
/// # Errors
///
/// Returns the SDK's registration refusal (e.g. a duplicate function name on
/// the same connection).
pub fn register_stardf_scan(connection: &Connection) -> frozen_duckdb::duckdb::Result<()> {
    connection.register_table_function::<StardfScanVTab>(SCAN_FUNCTION_NAME)
}

/// The registered SQL name.
pub const SCAN_FUNCTION_NAME: &str = "stardf_scan";

/// Resolves the required `table` named parameter against the closed
/// five-table set, absent → typed [`ScanReason::TableRequired`].
fn bind_table(bind: &BindInfo) -> Result<Table, Box<dyn std::error::Error>> {
    if bind.get_parameter_count() != 1 {
        return Err(ScanError::new(
            "bind",
            ScanReason::Parameter(format!(
                "expected exactly 1 positional parameter (the dataset root \
                 directory), got {}",
                bind.get_parameter_count()
            )),
        )
        .into());
    }
    let Some(value) = bind.get_named_parameter("table") else {
        return Err(ScanError::new(
            "bind.table",
            ScanReason::TableRequired {
                available: TABLE_NAMES.to_vec(),
            },
        )
        .into());
    };
    let requested = value.to_string();
    Table::ALL
        .into_iter()
        .find(|table| table.name() == requested)
        .ok_or_else(|| {
            ScanError::new(
                "bind.table",
                ScanReason::TableUnknown {
                    requested,
                    available: TABLE_NAMES.to_vec(),
                },
            )
        })
        .map_err(Into::into)
}

/// Loads the five Parquet files from the root directory, validates the full
/// profile through the receipted reader, and materializes the bound table's
/// rows.
fn bind_inner(
    bind: &BindInfo, table: Table,
) -> Result<StardfScanBindData, Box<dyn std::error::Error>> {
    let root = PathBuf::from(bind.get_parameter(0).to_string());
    let mut files = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for (slot, member) in files.iter_mut().zip(Table::ALL) {
        *slot = std::fs::read(root.join(member.file_name())).map_err(|source| {
            ScanError::new(
                "bind.files",
                ScanReason::DatasetIo {
                    file: member.file_name(),
                    source,
                },
            )
        })?;
    }
    let files = ParquetFiles::from_array(files);
    let read =
        read(&files).map_err(|error| ScanError::new("bind.profile", ScanReason::Profile(error)))?;

    let named_graphs: HashSet<u32> = read.dataset.named_graph_ids().collect();
    let term_index: BTreeMap<TermValue, u32> = read
        .dataset
        .terms()
        .iter()
        .enumerate()
        .map(|(index, value)| {
            // Term ids are u32 by contract (the wire id space); a term table
            // larger than that cannot exist.
            (
                value.clone(),
                u32::try_from(index + 1).expect("term ids are u32 by contract"),
            )
        })
        .collect();
    let cardinality;
    let rows = match table {
        Table::Terms => {
            cardinality = read.dataset.terms().len();
            ScanRows::Terms
        }
        Table::Quads => {
            let rows: Vec<QuadRow> = read
                .dataset
                .quads()
                .map(|quad| (quad.s, quad.p, quad.o, quad.g))
                .collect();
            cardinality = rows.len();
            ScanRows::Quads(rows)
        }
        Table::Reifiers => {
            // The dataset stores `(reifier, triple-term, graph)`; the Parquet
            // reifiers row is `(reifier, s, p, o, g)` — decompose each bound
            // triple term through the term index.
            let mut rows = Vec::new();
            for quad in read.dataset.reifier_quads() {
                let (s, p, o) = triple_components(&term_index, &read, quad.o, "reifiers")?;
                rows.push((quad.s, s, p, o, quad.g));
            }
            cardinality = rows.len();
            ScanRows::Reifiers(rows)
        }
        Table::Annotations => {
            let rows: Vec<QuadRow> = read
                .dataset
                .annotation_quads()
                .map(|quad| (quad.s, quad.p, quad.o, quad.g))
                .collect();
            cardinality = rows.len();
            ScanRows::Annotations(rows)
        }
        Table::Blobs => {
            let rows: Vec<(String, Vec<u8>)> = read
                .blobs
                .iter()
                .map(|(digest, payload)| (digest.to_hex(), payload.clone()))
                .collect();
            cardinality = rows.len();
            ScanRows::Blobs(rows)
        }
    };
    Ok(StardfScanBindData {
        table,
        cardinality,
        read,
        named_graphs,
        term_index,
        rows,
    })
}

/// Decomposes a triple-term wire id into its component wire ids through the
/// canonical term index.
fn triple_components(
    term_index: &BTreeMap<TermValue, u32>, read: &ColumnarRead, triple_id: u32,
    context: &'static str,
) -> Result<(u32, u32, u32), Box<dyn std::error::Error>> {
    let value = read.dataset.terms().get(
        usize::try_from(triple_id).map_err(|_| {
            ScanError::new(
                context,
                ScanReason::Render(format!("triple term id {triple_id} is out of range")),
            )
        })? - 1,
    );
    match value {
        Some(TermValue::Triple { s, p, o }) => {
            let lookup = |component: &TermValue| {
                term_index.get(component).copied().ok_or_else(|| {
                    ScanError::new(
                        context,
                        ScanReason::Render(format!(
                            "triple term component {component:?} is absent \
                             from the canonical term index"
                        )),
                    )
                })
            };
            Ok((lookup(s.value())?, lookup(p.value())?, lookup(o.value())?))
        }
        _ => Err(ScanError::new(
            context,
            ScanReason::Render(format!(
                "reifier binding names id {triple_id}, which is not a triple term"
            )),
        )
        .into()),
    }
}

// ---------------------------------------------------------------------------
// Row rendering into the output chunk
// ---------------------------------------------------------------------------

/// Writes one BIGINT cell.
fn put_bigint(column: &mut FlatVector, slot: usize, value: i64) {
    column.as_mut_slice::<i64>()[slot] = value;
}

/// Writes one nullable BIGINT cell.
fn put_opt_bigint(column: &mut FlatVector, slot: usize, value: Option<i64>) {
    match value {
        Some(value) => put_bigint(column, slot, value),
        None => column.set_null(slot),
    }
}

/// Writes one nullable VARCHAR cell, refusing interior NUL bytes with a
/// typed error instead of the SDK's `CString` panic.
fn put_opt_text(
    column: &mut FlatVector, slot: usize, value: Option<&str>, context: &'static str,
) -> Result<(), ScanError> {
    let Some(text) = value else {
        column.set_null(slot);
        return Ok(());
    };
    if text.as_bytes().contains(&0) {
        return Err(ScanError::new(
            context,
            ScanReason::InteriorNul { column: context },
        ));
    }
    column.insert(slot, text);
    Ok(())
}

/// Writes one nullable BLOB cell.
fn put_opt_bytes(column: &mut FlatVector, slot: usize, value: Option<&[u8]>) {
    match value {
        Some(bytes) => column.insert(slot, bytes),
        None => column.set_null(slot),
    }
}

/// Renders one terms row.
///
/// Terms columns, in schema order: id, kind, lex, datatype, lang, direction,
/// scope, triple_s, triple_p, triple_o, named_graph. Ids are 1-based dense
/// wire ids (`terms.id == row + 1`, the reader's dense-id contract); the
/// optional columns are nulled first and filled per variant.
fn render_terms(
    bind_data: &StardfScanBindData, row: usize, columns: &mut [FlatVector], slot: usize,
) -> Result<(), ScanError> {
    const CONTEXT: &str = "func.terms";
    let terms = bind_data.read.dataset.terms();
    let Some(value) = terms.get(row) else {
        return Err(ScanError::new(
            CONTEXT,
            ScanReason::Render(format!("terms row {row} is out of range")),
        ));
    };
    let wire_id = u32::try_from(row + 1).map_err(|_| {
        ScanError::new(
            CONTEXT,
            ScanReason::Render("terms row exceeds the u32 id space".into()),
        )
    })?;
    put_bigint(&mut columns[0], slot, i64::from(wire_id));
    // Every optional cell is written exactly once per row through the
    // put_opt_* helpers (write clears nothing — `set_null` is the ONLY way a
    // cell reads NULL — so a null-first pass would mask written cells). The
    // triple-term row's lex is the one cell that is always NULL.
    match value {
        TermValue::Iri(iri) => {
            put_bigint(&mut columns[1], slot, 0);
            put_opt_text(&mut columns[2], slot, Some(iri), CONTEXT)?;
            for index in [3_usize, 4, 5, 6, 7, 8, 9] {
                columns[index].set_null(slot);
            }
        }
        TermValue::Literal {
            lexical_form,
            datatype,
            language,
            direction,
        } => {
            put_bigint(&mut columns[1], slot, 1);
            put_opt_text(&mut columns[2], slot, Some(lexical_form), CONTEXT)?;
            put_opt_bigint(
                &mut columns[3],
                slot,
                bind_data
                    .term_index
                    .get(&TermValue::Iri(datatype.clone()))
                    .copied()
                    .map(i64::from),
            );
            put_opt_text(&mut columns[4], slot, language.as_deref(), CONTEXT)?;
            put_opt_bigint(
                &mut columns[5],
                slot,
                direction.map(|direction| match direction {
                    RdfTextDirection::Ltr => 0,
                    RdfTextDirection::Rtl => 1,
                }),
            );
            for index in [6_usize, 7, 8, 9] {
                columns[index].set_null(slot);
            }
        }
        TermValue::Blank { label, scope } => {
            put_bigint(&mut columns[1], slot, 2);
            put_opt_text(&mut columns[2], slot, Some(label), CONTEXT)?;
            put_bigint(&mut columns[6], slot, i64::from(scope.0));
            for index in [3_usize, 4, 5, 7, 8, 9] {
                columns[index].set_null(slot);
            }
        }
        TermValue::Triple { s, p, o } => {
            put_bigint(&mut columns[1], slot, 3);
            columns[2].set_null(slot);
            for index in [3_usize, 4, 5, 6] {
                columns[index].set_null(slot);
            }
            for (index, component) in [s.value(), p.value(), o.value()].into_iter().enumerate() {
                match bind_data.term_index.get(component).copied() {
                    Some(component_id) => {
                        put_bigint(&mut columns[7 + index], slot, i64::from(component_id));
                    }
                    None => {
                        return Err(ScanError::new(
                            CONTEXT,
                            ScanReason::Render(format!(
                                "triple term component {component:?} is absent \
                                 from the canonical term index"
                            )),
                        ));
                    }
                }
            }
        }
    }
    put_bigint(
        &mut columns[10],
        slot,
        i64::from(bind_data.named_graphs.contains(&wire_id)),
    );
    Ok(())
}

/// Renders one quad-class id row (quads: `s/p/o/g`; annotations:
/// `reifier/predicate/value/g`).
fn render_quad_row(row: QuadRow, columns: &mut [FlatVector], slot: usize) {
    let (first, second, third, graph) = row;
    put_bigint(&mut columns[0], slot, i64::from(first));
    put_bigint(&mut columns[1], slot, i64::from(second));
    put_bigint(&mut columns[2], slot, i64::from(third));
    put_opt_bigint(&mut columns[3], slot, graph.map(i64::from));
}

/// Renders one reifiers row `(reifier, s, p, o, g)`.
fn render_reifier_row(row: ReifierRow, columns: &mut [FlatVector], slot: usize) {
    let (reifier, first, second, third, graph) = row;
    put_bigint(&mut columns[0], slot, i64::from(reifier));
    put_bigint(&mut columns[1], slot, i64::from(first));
    put_bigint(&mut columns[2], slot, i64::from(second));
    put_bigint(&mut columns[3], slot, i64::from(third));
    put_opt_bigint(&mut columns[4], slot, graph.map(i64::from));
}

/// Renders one blobs row `(lowercase hex digest, payload)`.
fn render_blob_row(
    row: &(String, Vec<u8>), columns: &mut [FlatVector], slot: usize,
) -> Result<(), ScanError> {
    const CONTEXT: &str = "func.blobs";
    let (digest, payload) = row;
    put_opt_text(&mut columns[0], slot, Some(digest), CONTEXT)?;
    put_opt_bytes(&mut columns[1], slot, Some(payload.as_slice()));
    Ok(())
}

// ---------------------------------------------------------------------------
// W2 registration tests (real engine, real files, real SQL — no doubles)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use stardf_columnar::vendor::core::{BlankScope, ContentStore, RdfDatasetBuilder, RdfLiteral};
    use stardf_columnar::{ColumnarProjection, Compression, ProjectionCell, project, write};
    use std::sync::Arc;

    /// Test-local temp-root uniquifier.
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// A temp directory removed on drop (test-local lease, never leaked).
    struct TempDir(PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// One compared cell: the DuckDB side and the projection side are both
    /// normalized into this shape before equality.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Cell {
        I64(i64),
        Text(String),
        Blob(Vec<u8>),
        Null,
    }

    /// The known fixture: two quads (one graph-scoped, one default-graph),
    /// one reifier binding, one annotation, two blobs, two named graphs
    /// (one empty), a language-tagged directional literal, a typed literal
    /// and a quoted triple term. Counts are fixed by construction:
    /// terms = 9 (including the freeze-minted `rdf:reifies`), quads = 2,
    /// reifiers = 1, annotations = 1, blobs = 2.
    fn fixture() -> (Arc<stardf_columnar::vendor::core::RdfDataset>, ContentStore) {
        let mut builder = RdfDatasetBuilder::new();
        // The shim builder does NOT auto-intern a literal's datatype IRI: the
        // projection resolves a literal's datatype through the term index, so
        // both datatype IRIs are interned as terms first (disclosed fixture
        // contract, unlike the kernel builder).
        // (returned ids unused — interning itself is the fixture contract)
        let _dir_lang_string = builder
            .intern_iri(RdfLiteral::language_datatype_iri(Some(
                RdfTextDirection::Ltr,
            )))
            .unwrap();
        let _xsd_string = builder
            .intern_iri("http://www.w3.org/2001/XMLSchema#string")
            .unwrap();
        let subject = builder.intern_blank("subject", BlankScope(3)).unwrap();
        let predicate = builder.intern_iri("https://example.org/p").unwrap();
        let bonjour = builder
            .intern_value(&TermValue::Literal {
                lexical_form: "bonjour".to_owned(),
                datatype: RdfLiteral::language_datatype_iri(Some(RdfTextDirection::Ltr)).to_owned(),
                language: Some("fr".to_owned()),
                direction: Some(RdfTextDirection::Ltr),
            })
            .unwrap();
        let hi = builder
            .intern_value(&TermValue::Literal {
                lexical_form: "hi".to_owned(),
                datatype: "http://www.w3.org/2001/XMLSchema#string".to_owned(),
                language: None,
                direction: None,
            })
            .unwrap();
        let graph = builder.intern_iri("https://example.org/g").unwrap();
        let empty_graph = builder.intern_iri("https://example.org/empty").unwrap();
        builder.declare_named_graph(graph).unwrap();
        builder.declare_named_graph(empty_graph).unwrap();
        let triple = builder.intern_triple(subject, predicate, bonjour).unwrap();
        let reifier = builder.intern_blank("reifier", BlankScope(3)).unwrap();
        builder
            .push_quad(subject, predicate, bonjour, Some(graph))
            .unwrap();
        builder.push_quad(subject, predicate, hi, None).unwrap();
        builder
            .push_reifier_in_graph(reifier, triple, Some(graph))
            .unwrap();
        builder
            .push_annotation_in_graph(reifier, predicate, hi, None)
            .unwrap();
        let mut blobs = ContentStore::new();
        blobs.insert(b"first".to_vec());
        blobs.insert(b"second".to_vec());
        (builder.freeze().unwrap(), blobs)
    }

    /// Writes the fixture's five Parquet files into a fresh temp root.
    fn fixture_dir() -> (TempDir, ColumnarProjection) {
        let (dataset, blobs) = fixture();
        let encoded = write(&*dataset, &blobs, Compression::Uncompressed).unwrap();
        let projection = project(&*dataset, &blobs).unwrap();
        let dir = std::env::temp_dir().join(format!(
            "stardf-scan-duckdb-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for (table, bytes) in encoded.files.iter() {
            std::fs::write(dir.join(table.file_name()), bytes).unwrap();
        }
        (TempDir(dir), projection)
    }

    /// The canonical projection rows, normalized to [`Cell`].
    fn expected_rows(projection: &ColumnarProjection, table: Table) -> Vec<Vec<Cell>> {
        projection
            .rows(table)
            .map(|row| {
                row.into_iter()
                    .zip(table.schema().columns)
                    .map(|(cell, column)| match cell {
                        ProjectionCell::Null => Cell::Null,
                        ProjectionCell::Int64(value) => Cell::I64(value),
                        ProjectionCell::Bytes(bytes) if column.utf8 => {
                            Cell::Text(String::from_utf8(bytes.to_vec()).unwrap())
                        }
                        ProjectionCell::Bytes(bytes) => Cell::Blob(bytes.to_vec()),
                    })
                    .collect()
            })
            .collect()
    }

    /// The rows `stardf_scan` returns through a real connection, normalized
    /// to [`Cell`] per the closed schema's column types.
    fn scan_rows(connection: &Connection, root: &std::path::Path, table: Table) -> Vec<Vec<Cell>> {
        let sql = format!(
            "SELECT * FROM stardf_scan('{}', \"table\" = '{}')",
            root.display(),
            table.name()
        );
        let mut statement = connection
            .prepare(&sql)
            .unwrap_or_else(|error| panic!("prepare {table:?} failed: {error}"));
        let mut rows = statement.query([]).unwrap();
        let mut out = Vec::new();
        while let Some(row) = rows
            .next()
            .unwrap_or_else(|error| panic!("fetch {table:?} failed: {error}"))
        {
            let mut cells = Vec::new();
            for (index, column) in table.schema().columns.iter().enumerate() {
                let cell = match column.physical_type {
                    PhysicalType::Int64 => {
                        Cell::from_opt_i64(row.get::<_, Option<i64>>(index).unwrap())
                    }
                    PhysicalType::ByteArray if column.utf8 => {
                        Cell::from_opt_text(row.get::<_, Option<String>>(index).unwrap())
                    }
                    PhysicalType::ByteArray => {
                        Cell::from_opt_blob(row.get::<_, Option<Vec<u8>>>(index).unwrap())
                    }
                };
                cells.push(cell);
            }
            out.push(cells);
        }
        out
    }

    impl Cell {
        fn from_opt_i64(value: Option<i64>) -> Self {
            value.map_or(Self::Null, Self::I64)
        }

        fn from_opt_text(value: Option<String>) -> Self {
            value.map_or(Self::Null, Self::Text)
        }

        fn from_opt_blob(value: Option<Vec<u8>>) -> Self {
            value.map_or(Self::Null, Self::Blob)
        }
    }

    /// A real connection with `stardf_scan` registered.
    fn scan_connection() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        register_stardf_scan(&connection).unwrap();
        connection
    }

    #[test]
    fn five_table_counts_and_values_match_the_canonical_projection() {
        let connection = scan_connection();
        let (root, projection) = fixture_dir();
        for table in Table::ALL {
            let scanned = scan_rows(&connection, &root.0, table);
            assert_eq!(
                scanned.len(),
                projection.row_count(table),
                "count parity for {table:?}"
            );
            assert_eq!(
                scanned,
                expected_rows(&projection, table),
                "value parity for {table:?}"
            );
        }
    }

    #[test]
    fn terms_rows_carry_the_dense_ids_kinds_and_named_graph_flags() {
        let connection = scan_connection();
        let (root, projection) = fixture_dir();
        let scanned = scan_rows(&connection, &root.0, Table::Terms);
        assert_eq!(scanned.len(), projection.row_count(Table::Terms));
        // Dense ids: terms.id == row + 1 (the reader's dense-id contract).
        for (row, cells) in scanned.iter().enumerate() {
            let expected = i64::try_from(row).unwrap_or(i64::MAX) + 1;
            assert_eq!(cells[0], Cell::I64(expected), "terms.id density");
        }
        // The two named graphs are flagged; every other term is not.
        let named_flags: Vec<i64> = scanned
            .iter()
            .map(|cells| match cells[10] {
                Cell::I64(value) => value,
                ref other => panic!("named_graph is a required BIGINT, got {other:?}"),
            })
            .collect();
        assert_eq!(named_flags.iter().filter(|&&flag| flag == 1).count(), 2);
        // The blank subject and the quoted triple kind are present.
        let kinds: Vec<i64> = scanned
            .iter()
            .map(|cells| match cells[1] {
                Cell::I64(value) => value,
                ref other => panic!("kind is a required BIGINT, got {other:?}"),
            })
            .collect();
        assert!(kinds.contains(&2), "blank-node kind present: {kinds:?}");
        assert!(kinds.contains(&3), "triple-term kind present: {kinds:?}");
        assert!(kinds.contains(&1), "literal kind present: {kinds:?}");
        assert!(kinds.contains(&0), "iri kind present: {kinds:?}");
    }

    #[test]
    fn absent_table_parameter_is_a_typed_refusal() {
        let connection = scan_connection();
        let (root, _) = fixture_dir();
        let sql = format!("SELECT count(*) FROM stardf_scan('{}')", root.0.display());
        let error = connection
            .query_row::<i64, _, _>(&sql, [], |row| row.get(0))
            .unwrap_err();
        let rendered = error.to_string();
        assert!(
            rendered.contains("table"),
            "refusal must name the absent named parameter: {rendered}"
        );
    }

    #[test]
    fn unknown_table_is_a_typed_refusal() {
        let connection = scan_connection();
        let (root, _) = fixture_dir();
        let unknown = format!(
            "SELECT count(*) FROM stardf_scan('{}', \"table\" = 'nonesuch')",
            root.0.display()
        );
        let error = connection
            .query_row::<i64, _, _>(&unknown, [], |row| row.get(0))
            .unwrap_err();
        assert!(
            error.to_string().contains("unknown table 'nonesuch'"),
            "typed unknown-table refusal: {error}"
        );
    }

    #[test]
    fn missing_files_are_a_typed_refusal() {
        let connection = scan_connection();
        let missing = "SELECT count(*) FROM stardf_scan(\
                       '/nonexistent-stardf-scan-root', \"table\" = 'terms')";
        let error = connection
            .query_row::<i64, _, _>(missing, [], |row| row.get(0))
            .unwrap_err();
        assert!(
            error.to_string().contains("reading terms.parquet failed"),
            "typed dataset-io refusal: {error}"
        );
    }

    #[test]
    fn pushdown_stays_disabled_per_f05() {
        assert!(
            !<StardfScanVTab as VTab>::supports_pushdown(),
            "supports_pushdown must stay false until the C3 flip court lands"
        );
    }
}
