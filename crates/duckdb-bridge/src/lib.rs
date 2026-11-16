// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native `stardf_bridge_scan()` DuckDB table function over the closed
//! five-table columnar profile, WITH projection pushdown (lane RDF-8,
//! 2026-11-01).
//!
//! # Path choice (recorded per the lane dispatch)
//!
//! Two documented paths exist for making stardf decoded columns
//! DuckDB-queryable (docs/jira/v26.11.1/waves/research-duckdb-arrow.md S4):
//! (a) an Arrow C Data Interface producer consumed via `duckdb_arrow_scan`,
//! or (b) a native table function with
//! `duckdb_table_function_supports_projection_pushdown`. This crate
//! implements (b), because (i) the frozen-duckdb 2.0.4 SDK exposes the full
//! projection-pushdown surface in-process
//! (`VTab::supports_pushdown` registers the C call;
//! `InitInfo::get_column_indices` reads the projected list), (ii) the
//! research receipt names projection pushdown as the measured driver of
//! DuckDB-Arrow performance, and (iii) the sibling `stardf-scan-duckdb`
//! crate registers `stardf_scan()` with pushdown FALSE — the gap this crate
//! closes. Path (a) is not implemented here: the vendored C API surface for
//! `duckdb_arrow_scan` was not verified present in the frozen-duckdb-sys
//! bindgen allowlist this session, and path (a) inherits
//! materialize-then-filter semantics for RecordBatchReader-class producers
//! (research receipt S1). Row-FILTER pushdown to custom table functions is
//! NOT part of the documented C table-function contract — recorded as
//! UNKNOWN, never claimed.
//!
//! # Surface
//!
//! `stardf_bridge_scan(root VARCHAR, "table" VARCHAR)` — identical shape to
//! `stardf_scan`, plus: DuckDB decides per query which of the bound table's
//! columns reach `func`. Bind declares the full closed-schema relation
//! (physical INT64 -> BIGINT, BYTE_ARRAY+UTF8 -> VARCHAR, BYTE_ARRAY ->
//! BLOB) from [`stardf_columnar::schema`]; init captures the projected
//! column indices; func emits ONLY those columns. Rows are materialized at
//! bind from the receipted validated reader ([`stardf_columnar::read`]) —
//! full physical + semantic profile validation runs before any row is
//! emitted (fail-closed). The projection pushdown delta is at EMISSION: the
//! chunk carries only projected vectors, and the global cell counter
//! ([`bridge_cells_written`]) witnesses the pruning.
//!
//! # Divergences (disclosed)
//!
//! - Bind-time materialization duplicates the sibling's disclosed posture:
//!   the codec's raw row surfaces are `pub(crate)`, so rows are materialized
//!   through the public reader; the projection delta is emission-side only.
//! - The named parameter `table` is REQUIRED: a one-relation table function
//!   binds exactly one closed-schema relation per invocation; the absent
//!   default is a typed refusal enumerating the five names.
//! - `#![forbid(unsafe_code)]`: the only unsafe edge is the SDK's own
//!   extern "C" shims inside frozen-duckdb.
//! - Parallel emission is deferred (single-threaded claim cursor, same as
//!   the sibling); the cursor is already Send+Sync.

#![forbid(unsafe_code)]
#![allow(
    // Public constructors are marked #[must_use] by hand where they guard
    // bugs; a blanket candidate pass would demand it on every getter.
    clippy::must_use_candidate,
    // `stardf_bridge_*` names repeat the crate name by contract.
    clippy::module_name_repetitions,
    // Spec vocabulary (DuckDB, Parquet, RDF 1.2, BIGINT/VARCHAR/BLOB) is
    // deliberate prose.
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
/// capped by the output chunk's own vector capacity.
const SLAB_ROWS: usize = 2048;

/// The closed five-table logical names, in canonical order.
const TABLE_NAMES: [&str; 5] = ["terms", "quads", "reifiers", "annotations", "blobs"];

/// The registered SQL name.
pub const BRIDGE_FUNCTION_NAME: &str = "stardf_bridge_scan";

// ---------------------------------------------------------------------------
// Observable pushdown counter (real state, not a mock: the courts read the
// exact cell count DuckDB made the producer write)
// ---------------------------------------------------------------------------

/// Total cells written into DuckDB chunks since process start.
static BRIDGE_CELLS_WRITTEN: AtomicUsize = AtomicUsize::new(0);

/// Cells written into DuckDB output chunks so far.
///
/// The projection-pushdown observable: with
/// `duckdb_table_function_supports_projection_pushdown` registered, a query
/// projecting K of a table's C columns drives exactly N x K cell writes for
/// N scanned rows; `SELECT *` drives N x C. A producer that ignores the
/// projected list (or registers with pushdown disabled, as `stardf_scan`
/// does) cannot produce the K-pruned count.
pub fn bridge_cells_written() -> usize {
    BRIDGE_CELLS_WRITTEN.load(Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// Typed scan-surface refusal
// ---------------------------------------------------------------------------

/// A typed scan-surface refusal: every bind/func refusal carries a stable
/// context and a structured reason, never a panic across the FFI.
///
/// The SDK converts the trait error channel into `set_error(string)`;
/// [`Display`] renders `stardf_bridge_scan <context>: <reason>`.
#[derive(Debug)]
pub struct BridgeError {
    /// The surface that refused (e.g. `bind`, `bind.table`, `func.quads`).
    context: &'static str,
    /// The structured reason.
    reason: BridgeReason,
}

/// The per-surface refusal reasons.
#[derive(Debug)]
pub enum BridgeReason {
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
    /// An internal invariant failed while rendering a row. Fail-closed: the
    /// query errors, it never emits a wrong row.
    Render(String),
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stardf_bridge_scan {}: ", self.context)?;
        match &self.reason {
            BridgeReason::Parameter(message) | BridgeReason::Render(message) => {
                write!(f, "{message}")
            }
            BridgeReason::TableRequired { available } => write!(
                f,
                "the required named parameter `table` is absent; one of {}",
                available.join(", ")
            ),
            BridgeReason::TableUnknown {
                requested,
                available,
            } => write!(
                f,
                "unknown table '{requested}'; one of {}",
                available.join(", ")
            ),
            BridgeReason::DatasetIo { file, .. } => write!(f, "reading {file} failed"),
            BridgeReason::Profile(error) => write!(f, "profile validation refused: {error}"),
            BridgeReason::InteriorNul { column } => write!(
                f,
                "VARCHAR cell in {column} carries an interior NUL byte; \
                 refusing instead of an SDK CString panic"
            ),
        }
    }
}

impl std::error::Error for BridgeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.reason {
            BridgeReason::DatasetIo { source, .. } => Some(source),
            // ColumnarError implements Display but not std::error::Error; it
            // is carried by value and rendered through Display above.
            _ => None,
        }
    }
}

impl BridgeError {
    /// A bridge refusal from its parts.
    pub const fn new(context: &'static str, reason: BridgeReason) -> Self {
        Self { context, reason }
    }
}

// ---------------------------------------------------------------------------
// Materialized rows (bind-time, through the receipted public reader)
// ---------------------------------------------------------------------------

/// One normalized cell value, decoded at bind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellVal {
    /// A BIGINT cell.
    I64(i64),
    /// A VARCHAR cell.
    Text(String),
    /// A BLOB cell.
    Blob(Vec<u8>),
    /// SQL NULL.
    Null,
}

/// The bind data: the validated dataset, derived index surfaces, and the
/// materialized rows of the selected table.
#[derive(Debug)]
pub struct BridgeBindData {
    /// The bound table.
    table: Table,
    /// Exact row count of the bound table.
    cardinality: usize,
    /// The materialized rows, row-major, schema-column order.
    rows: Vec<Vec<CellVal>>,
}

/// The init data: the claim cursor plus DuckDB's projected column list.
#[derive(Debug)]
pub struct BridgeInitData {
    /// The next unclaimed row.
    next_row: AtomicUsize,
    /// DuckDB's requested columns, as bind-schema indices, in chunk order.
    /// Populated only because supports_pushdown() registers projection
    /// pushdown; a false flag leaves this empty and the projection pruned
    /// count unreachable.
    column_indices: Vec<usize>,
}

/// Materializes the bound table's rows, row-major, schema-column order,
/// through the validated reader — the exact cell semantics the closed
/// schema defines (terms render from the dense term table with 1-based
/// wire ids; reifiers decompose bound triple terms through the canonical
/// term index; blobs pair lowercase hex digests with payloads).
// The terms arm renders the closed schema's full 11-column layout inline;
// splitting it would scatter the column contract across helpers.
#[allow(clippy::too_many_lines)]
fn materialize(
    table: Table, read: &ColumnarRead, term_index: &BTreeMap<TermValue, u32>,
    named_graphs: &HashSet<u32>,
) -> Result<(Vec<Vec<CellVal>>, usize), BridgeError> {
    let push_i = |row: &mut Vec<CellVal>, v: i64| row.push(CellVal::I64(v));
    let push_opt_i = |row: &mut Vec<CellVal>, v: Option<i64>| {
        row.push(v.map_or(CellVal::Null, CellVal::I64));
    };
    let push_t = |row: &mut Vec<CellVal>, v: String| row.push(CellVal::Text(v));
    let rows: Vec<Vec<CellVal>> = match table {
        Table::Terms => {
            let mut rows = Vec::new();
            for (index, value) in read.dataset.terms().iter().enumerate() {
                let mut row = Vec::with_capacity(11);
                let Ok(wire_id) = i64::try_from(index + 1) else {
                    return Err(BridgeError::new(
                        "bind.materialize.terms",
                        BridgeReason::Render(
                            "terms row exceeds the i64 id space".into(),
                        ),
                    ));
                };
                push_i(&mut row, wire_id);
                match value {
                    TermValue::Iri(iri) => {
                        push_i(&mut row, 0);
                        push_t(&mut row, iri.clone());
                        for _ in 0..7 {
                            row.push(CellVal::Null);
                        }
                    }
                    TermValue::Literal {
                        lexical_form,
                        datatype,
                        language,
                        direction,
                    } => {
                        push_i(&mut row, 1);
                        push_t(&mut row, lexical_form.clone());
                        push_opt_i(
                            &mut row,
                            term_index
                                .get(&TermValue::Iri(datatype.clone()))
                                .copied()
                                .map(i64::from),
                        );
                        push_opt_t(&mut row, language.clone());
                        push_opt_i(
                            &mut row,
                            direction.map(|d| match d {
                                RdfTextDirection::Ltr => 0,
                                RdfTextDirection::Rtl => 1,
                            }),
                        );
                        for _ in 0..4 {
                            row.push(CellVal::Null);
                        }
                    }
                    TermValue::Blank { label, scope } => {
                        push_i(&mut row, 2);
                        push_t(&mut row, label.clone());
                        row.push(CellVal::Null);
                        row.push(CellVal::Null);
                        row.push(CellVal::Null);
                        push_i(&mut row, i64::from(scope.0));
                        for _ in 0..3 {
                            row.push(CellVal::Null);
                        }
                    }
                    TermValue::Triple { s, p, o } => {
                        push_i(&mut row, 3);
                        row.push(CellVal::Null); // lex is always NULL
                        for _ in 0..4 {
                            row.push(CellVal::Null);
                        }
                        for component in [s.value(), p.value(), o.value()] {
                            match term_index.get(component).copied() {
                                Some(id) => push_i(&mut row, i64::from(id)),
                                None => {
                                    return Err(BridgeError::new(
                                        "bind.materialize.terms",
                                        BridgeReason::Render(format!(
                                            "triple term component {component:?} is absent \
                                             from the canonical term index"
                                        )),
                                    ));
                                }
                            }
                        }
                    }
                }
                let wire_u32 = u32::try_from(wire_id)
                    .expect("wire ids are u32 by contract");
                push_i(
                    &mut row,
                    i64::from(named_graphs.contains(&wire_u32)),
                );
                rows.push(row);
            }
            rows
        }
        Table::Quads => read
            .dataset
            .quads()
            .map(|q| {
                vec![
                    CellVal::I64(i64::from(q.s)),
                    CellVal::I64(i64::from(q.p)),
                    CellVal::I64(i64::from(q.o)),
                    q.g.map_or(CellVal::Null, |g| CellVal::I64(i64::from(g))),
                ]
            })
            .collect(),
        Table::Reifiers => {
            let mut rows = Vec::new();
            for quad in read.dataset.reifier_quads() {
                let (s, p, o) = triple_components(term_index, read, quad.o)?;
                rows.push(vec![
                    CellVal::I64(i64::from(quad.s)),
                    CellVal::I64(i64::from(s)),
                    CellVal::I64(i64::from(p)),
                    CellVal::I64(i64::from(o)),
                    quad.g.map_or(CellVal::Null, |g| CellVal::I64(i64::from(g))),
                ]);
            }
            rows
        }
        Table::Annotations => read
            .dataset
            .annotation_quads()
            .map(|q| {
                vec![
                    CellVal::I64(i64::from(q.s)),
                    CellVal::I64(i64::from(q.p)),
                    CellVal::I64(i64::from(q.o)),
                    q.g.map_or(CellVal::Null, |g| CellVal::I64(i64::from(g))),
                ]
            })
            .collect(),
        Table::Blobs => read
            .blobs
            .iter()
            .map(|(digest, payload)| {
                vec![
                    CellVal::Text(digest.to_hex()),
                    CellVal::Blob(payload.clone()),
                ]
            })
            .collect(),
    };
    let cardinality = rows.len();
    Ok((rows, cardinality))
}

/// Decomposes a triple-term wire id into component wire ids through the
/// canonical term index.
fn triple_components(
    term_index: &BTreeMap<TermValue, u32>, read: &ColumnarRead, triple_id: u32,
) -> Result<(u32, u32, u32), BridgeError> {
    const CONTEXT: &str = "bind.materialize.reifiers";
    let index = usize::try_from(triple_id).map_err(|_| {
        BridgeError::new(
            CONTEXT,
            BridgeReason::Render(format!("triple term id {triple_id} is out of range")),
        )
    })? - 1;
    let value = read.dataset.terms().get(index);
    match value {
        Some(TermValue::Triple { s, p, o }) => {
            let lookup = |component: &TermValue| {
                term_index.get(component).copied().ok_or_else(|| {
                    BridgeError::new(
                        CONTEXT,
                        BridgeReason::Render(format!(
                            "triple term component {component:?} is absent \
                             from the canonical term index"
                        )),
                    )
                })
            };
            Ok((lookup(s.value())?, lookup(p.value())?, lookup(o.value())?))
        }
        _ => Err(BridgeError::new(
            CONTEXT,
            BridgeReason::Render(format!(
                "reifier binding names id {triple_id}, which is not a triple term"
            )),
        )),
    }
}

/// Pushes an Option<String> as Text-or-Null (terms lang column).
fn push_opt_t(row: &mut Vec<CellVal>, value: Option<String>) {
    row.push(value.map_or(CellVal::Null, CellVal::Text));
}

// ---------------------------------------------------------------------------
// The VTab (projection pushdown registered)
// ---------------------------------------------------------------------------

/// The `stardf_bridge_scan()` table function.
#[derive(Debug)]
pub struct StardfBridgeVTab;

impl VTab for StardfBridgeVTab {
    type InitData = BridgeInitData;
    type BindData = BridgeBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn std::error::Error>> {
        // Declare the FULL closed-schema relation: with projection pushdown,
        // bind always declares every column; DuckDB then chooses which reach
        // init/func.
        let table = bind_table(bind)?;
        for column in table.schema().columns {
            let logical = match column.physical_type {
                PhysicalType::Int64 => LogicalTypeId::Bigint,
                PhysicalType::ByteArray if column.utf8 => LogicalTypeId::Varchar,
                PhysicalType::ByteArray => LogicalTypeId::Blob,
            };
            bind.add_result_column(column.name, LogicalTypeHandle::from(logical));
        }
        let root = PathBuf::from(bind.get_parameter(0).to_string());
        let mut files = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        for (slot, member) in files.iter_mut().zip(Table::ALL) {
            *slot = std::fs::read(root.join(member.file_name())).map_err(|source| {
                BridgeError::new(
                    "bind.files",
                    BridgeReason::DatasetIo {
                        file: member.file_name(),
                        source,
                    },
                )
            })?;
        }
        let files = ParquetFiles::from_array(files);
        let read = read(&files).map_err(|error| {
            BridgeError::new("bind.profile", BridgeReason::Profile(error))
        })?;
        let named_graphs: HashSet<u32> = read.dataset.named_graph_ids().collect();
        let term_index: BTreeMap<TermValue, u32> = read
            .dataset
            .terms()
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let id = u32::try_from(index + 1)
                    .expect("term ids are u32 by contract");
                (value.clone(), id)
            })
            .collect();
        let (rows, cardinality) =
            materialize(table, &read, &term_index, &named_graphs)?;
        bind.set_cardinality(
            u64::try_from(cardinality).unwrap_or(u64::MAX),
            true,
        );
        Ok(BridgeBindData {
            table,
            cardinality,
            rows,
        })
    }

    fn init(init: &InitInfo) -> Result<Self::InitData, Box<dyn std::error::Error>> {
        // Projection pushdown: DuckDB populated the projected column list
        // because supports_pushdown() = true registered
        // duckdb_table_function_supports_projection_pushdown. Empty only if
        // the flag did not reach the engine (a court-observable failure).
        let column_indices = init
            .get_column_indices()
            .into_iter()
            .map(|index| usize::try_from(index).unwrap_or(usize::MAX))
            .collect();
        Ok(BridgeInitData {
            next_row: AtomicUsize::new(0),
            column_indices,
        })
    }

    fn func(
        func: &TableFunctionInfo<Self>, output: &mut DataChunkHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bind_data = func.get_bind_data();
        let init_data = func.get_init_data();
        // A zero-column projection (e.g. count(*)) still scans rows: claim
        // the slab and emit length only. Any NON-empty projection must list
        // columns DuckDB actually allocated vectors for.
        let indices = &init_data.column_indices;
        let capacity = if indices.is_empty() {
            SLAB_ROWS
        } else {
            output.flat_vector(0).capacity().clamp(1, SLAB_ROWS)
        };
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
        let mut columns: Vec<_> = (0..indices.len())
            .map(|index| output.flat_vector(index))
            .collect();
        let count = end - start;
        for (local, row) in (start..end).enumerate() {
            let cells = bind_data
                .rows
                .get(row)
                .ok_or_else(|| {
                    BridgeError::new(
                        "func",
                        BridgeReason::Render(format!(
                            "row {row} is out of range (cardinality {})",
                            bind_data.cardinality
                        )),
                    )
                })?;
            for (chunk_slot, &schema_index) in indices.iter().enumerate() {
                let cell = cells.get(schema_index).ok_or_else(|| {
                    BridgeError::new(
                        "func",
                        BridgeReason::Render(format!(
                            "projected schema index {schema_index} is out of range \
                             for table {:?} ({} columns)",
                            bind_data.table,
                            cells.len()
                        )),
                    )
                })?;
                write_cell(
                    &mut columns[chunk_slot],
                    local,
                    cell,
                    "func.emission",
                )?;
                BRIDGE_CELLS_WRITTEN.fetch_add(1, Ordering::Relaxed);
            }
        }
        output.set_len(count);
        Ok(())
    }

    /// Projection pushdown REGISTERED: this flips to
    /// duckdb_table_function_supports_projection_pushdown(true) at
    /// registration time (vtab/mod.rs line 144 passes T::supports_pushdown()
    /// into TableFunction::supports_pushdown, which calls the C function).
    fn supports_pushdown() -> bool {
        true
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

/// Writes one BIGINT cell.
fn write_i64_cell(column: &mut FlatVector, slot: usize, value: i64) {
    column.as_mut_slice::<i64>()[slot] = value;
}

/// Writes one VARCHAR cell, refusing interior NUL bytes with a typed error
/// instead of the SDK's CString panic.
// `&mut FlatVector` is kept for surface symmetry: the SDK's `Inserter`
// insert takes the vector by shared ref, but the write is logically
// exclusive.
#[allow(clippy::needless_pass_by_ref_mut)]
fn write_text_cell(
    column: &mut FlatVector, slot: usize, text: &str, context: &'static str,
) -> Result<(), BridgeError> {
    if text.as_bytes().contains(&0) {
        return Err(BridgeError::new("text-cell", BridgeReason::InteriorNul { column: context }));
    }
    column.insert(slot, text);
    Ok(())
}

/// Writes one BLOB cell.
// See write_text_cell: &mut kept for surface symmetry with the other
// writers.
#[allow(clippy::needless_pass_by_ref_mut)]
fn write_blob_cell(column: &mut FlatVector, slot: usize, bytes: &[u8]) {
    column.insert(slot, bytes);
}

/// Writes one SQL NULL cell.
fn write_null_cell(column: &mut FlatVector, slot: usize) {
    column.set_null(slot);
}

/// Dispatches one normalized cell into its chunk vector writer; the VARCHAR
/// arm is the only fallible one (interior NUL refusal).
fn write_cell(
    column: &mut FlatVector, slot: usize, cell: &CellVal, context: &'static str,
) -> Result<(), BridgeError> {
    match cell {
        CellVal::I64(value) => {
            write_i64_cell(column, slot, *value);
            Ok(())
        }
        CellVal::Text(text) => write_text_cell(column, slot, text, context),
        CellVal::Blob(bytes) => {
            write_blob_cell(column, slot, bytes);
            Ok(())
        }
        CellVal::Null => {
            write_null_cell(column, slot);
            Ok(())
        }
    }
}

/// Registers `stardf_bridge_scan` on the connection (the in-process path,
/// `Connection::register_table_function`).
///
/// # Errors
///
/// Returns the SDK's registration refusal (e.g. a duplicate function name on
/// the same connection).
pub fn register_stardf_bridge_scan(
    connection: &Connection,
) -> frozen_duckdb::duckdb::Result<()> {
    connection.register_table_function::<StardfBridgeVTab>(BRIDGE_FUNCTION_NAME)
}

/// Resolves the required `table` named parameter against the closed
/// five-table set, absent -> typed [`BridgeReason::TableRequired`].
fn bind_table(bind: &BindInfo) -> Result<Table, Box<dyn std::error::Error>> {
    if bind.get_parameter_count() != 1 {
        return Err(BridgeError::new(
            "bind",
            BridgeReason::Parameter(format!(
                "expected exactly 1 positional parameter (the dataset root \
                 directory), got {}",
                bind.get_parameter_count()
            )),
        )
        .into());
    }
    let Some(value) = bind.get_named_parameter("table") else {
        return Err(BridgeError::new(
            "bind.table",
            BridgeReason::TableRequired {
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
            BridgeError::new(
                "bind.table",
                BridgeReason::TableUnknown {
                    requested,
                    available: TABLE_NAMES.to_vec(),
                },
            )
        })
        .map_err(Into::into)
}

// ---------------------------------------------------------------------------
// Courts (real engine, real files, real SQL — no doubles)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use stardf_columnar::vendor::core::{BlankScope, ContentStore, RdfDatasetBuilder, RdfLiteral};
    use stardf_columnar::{ColumnarProjection, Compression, ProjectionCell, project, write};
    use std::sync::Arc;

    /// Test-local temp-root uniquifier.
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// Courts run against ONE global emission counter; every query-driving
    /// court holds this lock so concurrent test threads cannot pollute a
    /// delta measurement.
    static COURT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// A temp directory removed on drop (test-local lease, never leaked).
    struct TempDir(PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// One compared cell.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Cell {
        I64(i64),
        Text(String),
        Blob(Vec<u8>),
        Null,
    }

    /// The known fixture (same shape as the sibling scan crate's, kept
    /// structurally independent): two quads (one graph-scoped, one
    /// default-graph), one reifier binding, one annotation, two blobs, two
    /// named graphs (one empty), a language-tagged directional literal, a
    /// typed literal and a quoted triple term.
    fn fixture_with(mutation: Option<&str>) -> (Arc<stardf_columnar::vendor::core::RdfDataset>, ContentStore) {
        let mut builder = RdfDatasetBuilder::new();
        let _ = builder
            .intern_iri(RdfLiteral::language_datatype_iri(Some(
                RdfTextDirection::Ltr,
            )))
            .unwrap();
        let _ = builder
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
                lexical_form: mutation.unwrap_or("hi").to_owned(),
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
        builder.push_quad(subject, predicate, bonjour, Some(graph)).unwrap();
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
    /// Returns the root, the canonical projection, and the raw file bytes
    /// per table (for the byte-identity restore proof).
    fn fixture_dir_with(
        mutation: Option<&str>,
    ) -> (
        TempDir,
        ColumnarProjection,
        Vec<(&'static str, Vec<u8>)>,
    ) {
        let (dataset, blobs) = fixture_with(mutation);
        let encoded = write(&*dataset, &blobs, Compression::Uncompressed).unwrap();
        let projection = project(&*dataset, &blobs).unwrap();
        let dir = std::env::temp_dir().join(format!(
            "stardf-duckdb-bridge-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let mut raw = Vec::new();
        for (table, bytes) in encoded.files.iter() {
            std::fs::write(dir.join(table.file_name()), bytes).unwrap();
            raw.push((table.file_name(), bytes.to_vec()));
        }
        (TempDir(dir), projection, raw)
    }

    fn fixture_dir() -> (TempDir, ColumnarProjection) {
        let (dir, projection, _) = fixture_dir_with(None);
        (dir, projection)
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

    /// The rows `stardf_bridge_scan` returns through a real connection,
    /// normalized to [`Cell`] per the closed schema's column types.
    fn scan_rows(connection: &Connection, root: &std::path::Path, table: Table) -> Vec<Vec<Cell>> {
        let sql = format!(
            "SELECT * FROM stardf_bridge_scan('{}', \"table\" = '{}')",
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

    /// The values of ONE projected BIGINT column through a real query.
    fn projected_i64s(
        connection: &Connection, root: &std::path::Path, table: Table, column: &str,
    ) -> Vec<Option<i64>> {
        let sql = format!(
            "SELECT {column} FROM stardf_bridge_scan('{}', \"table\" = '{}')",
            root.display(),
            table.name()
        );
        let mut statement = connection.prepare(&sql).unwrap();
        let mut rows = statement.query([]).unwrap();
        let mut out = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            out.push(row.get::<_, Option<i64>>(0).unwrap());
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

    /// A real connection with `stardf_bridge_scan` registered.
    fn bridge_connection() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        register_stardf_bridge_scan(&connection).unwrap();
        connection
    }

    // -- Court 1: SELECT via DuckDB == direct stardf decode (all five tables)

    #[test]
    fn five_table_counts_and_values_match_the_canonical_projection() {
        let _court = COURT_LOCK.lock().unwrap();
        let connection = bridge_connection();
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

    // -- Court 2: a projected SELECT matches the direct decode of the SAME
    // columns, in the SAME row order (DuckDB-side projection correctness).

    #[test]
    fn projected_select_matches_direct_decode_of_the_same_columns() {
        let _court = COURT_LOCK.lock().unwrap();
        let connection = bridge_connection();
        let (root, projection) = fixture_dir();
        // quads.(s, o): the direct-decode side, normalized.
        let expected: Vec<(Option<i64>, Option<i64>)> = expected_rows(&projection, Table::Quads)
            .into_iter()
            .map(|row| match (row[0].clone(), row[2].clone()) {
                (Cell::I64(s), Cell::I64(o)) => (Some(s), Some(o)),
                _ => panic!("quads s/o are required BIGINTs"),
            })
            .collect();
        let sql = format!(
            "SELECT s, o FROM stardf_bridge_scan('{}', \"table\" = 'quads')",
            root.0.display()
        );
        let mut statement = connection.prepare(&sql).unwrap();
        let mut rows = statement.query([]).unwrap();
        let mut got = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            got.push((
                row.get::<_, Option<i64>>(0).unwrap(),
                row.get::<_, Option<i64>>(1).unwrap(),
            ));
        }
        assert_eq!(got, expected, "projected quads.(s,o) parity");
    }

    // -- Court 3: projection pushdown is OBSERVABLE — a K-column projection
    // writes exactly N x K cells, SELECT * writes N x C. A producer that
    // ignores DuckDB's projected list cannot produce the pruned count.

    #[test]
    fn projection_pushdown_prunes_cells_written() {
        let _court = COURT_LOCK.lock().unwrap();
        let connection = bridge_connection();
        let (root, projection) = fixture_dir();
        let n = projection.row_count(Table::Quads);
        assert_eq!(n, 2, "fixture quads count (fixed by construction)");
        let before_star = bridge_cells_written();
        let _ = scan_rows(&connection, &root.0, Table::Quads);
        let star_delta = bridge_cells_written() - before_star;
        let before_pruned = bridge_cells_written();
        let _ = projected_i64s(&connection, &root.0, Table::Quads, "o");
        let pruned_delta = bridge_cells_written() - before_pruned;
        // DuckDB may rescan the function internally (optimizer/execution
        // passes); both deltas then scale by the SAME pass count k, so the
        // pruning law is exact: star = k*N*C, pruned = k*N*1.
        assert_eq!(
            star_delta % (n * 4),
            0,
            "SELECT * cells must be a multiple of N*C: {star_delta}"
        );
        let k = star_delta / (n * 4);
        assert!(k >= 1, "at least one scan pass: k={k}");
        assert_eq!(
            pruned_delta,
            n * k,
            "SELECT o writes exactly k*N cells (k={k} shared with SELECT *)"
        );
    }

    // -- Court 4 (anti-vacuity flip): corrupting a decoded column value
    // pre-exposure (a literal lexical form, before encode/write) MUST
    // diverge on the DuckDB side. The court flips, then RESTORES by
    // rebuilding the unmutated fixture and re-proving byte identity.

    #[test]
    fn corrupted_decoded_column_diverges_on_the_duckdb_side() {
        let _court = COURT_LOCK.lock().unwrap();
        let connection = bridge_connection();
        // Baseline (raw bytes kept for the restore witness).
        let (root, projection, base_raw) = fixture_dir_with(None);
        let baseline = scan_rows(&connection, &root.0, Table::Terms);
        assert_eq!(baseline, expected_rows(&projection, Table::Terms));
        // FLIP: the literal "hi" -> "corrupted-in-court" (pre-exposure:
        // the mutation happens before encode/write, i.e. before the producer
        // ever sees the column).
        let (mutated_root, mutated_projection, mutated_raw) =
            fixture_dir_with(Some("corrupted-in-court"));
        // The flip must be real: the mutated ENCODING differs from baseline.
        let flipped = mutated_raw
            .iter()
            .zip(base_raw.iter())
            .any(|((mn, mb), (bn, bb))| mn == bn && mb != bb);
        assert!(flipped, "the corruption must change the encoded bytes");
        // RESTORE: rebuild the unmutated fixture; require byte-identical
        // files (determinism = fail-closed restore) AND baseline behavior.
        let (restored_root, _restored_projection, restored_raw) = fixture_dir_with(None);
        for ((mn, mb), (bn, bb)) in restored_raw.iter().zip(base_raw.iter()) {
            assert_eq!(*mn, *bn);
            assert_eq!(mb, bb, "restore rebuild must be byte-identical: {mn}");
        }
        assert_eq!(
            scan_rows(&connection, &restored_root.0, Table::Terms),
            baseline,
            "restored rebuild must reproduce the baseline scan"
        );
        let mutated = scan_rows(&connection, &mutated_root.0, Table::Terms);
        assert_eq!(
            mutated,
            expected_rows(&mutated_projection, Table::Terms),
            "mutated fixture still decodes cleanly through DuckDB"
        );
        assert_ne!(
            mutated, baseline,
            "the corruption MUST flip the DuckDB-side result (anti-vacuity)"
        );
        // The divergence is witnessed in the lex column: the corrupted
        // literal's lexical form reaches the DuckDB side verbatim.
        let lexes: Vec<&Cell> = mutated.iter().map(|row| &row[2]).collect();
        assert!(
            lexes.iter().any(|cell| **cell == Cell::Text("corrupted-in-court".into())),
            "the corrupted lexical form must appear on the DuckDB side: {lexes:?}"
        );
        assert_eq!(baseline.len(), mutated.len(), "shape preserved, values flipped");
    }

    // -- Court 5: byte-level corruption of an encoded page is a typed
    // refusal surfaced through DuckDB's error channel (fail-closed, never a
    // wrong row).

    #[test]
    fn byte_corrupted_parquet_is_a_typed_refusal() {
        let _court = COURT_LOCK.lock().unwrap();
        let connection = bridge_connection();
        let (root, _, raw) = fixture_dir_with(None);
        // Corrupt a byte INSIDE the quads payload, mid-file (header bytes
        // stay parseable; the reader's validation must still refuse).
        let (name, bytes) = &raw[1];
        assert_eq!(*name, "quads.parquet");
        let mut corrupted = bytes.clone();
        let mid = corrupted.len() / 2;
        corrupted[mid] ^= 0xFF;
        std::fs::write(root.0.join(name), &corrupted).unwrap();
        let sql = format!(
            "SELECT * FROM stardf_bridge_scan('{}', \"table\" = 'quads')",
            root.0.display()
        );
        let error = connection
            .query_row::<i64, _, _>(&sql, [], |row| row.get(0))
            .unwrap_err();
        let rendered = error.to_string();
        assert!(
            rendered.contains("profile validation refused"),
            "typed profile refusal through the DuckDB error channel: {rendered}"
        );
    }

    // -- Typed refusal courts (bind surface).

    #[test]
    fn absent_table_parameter_is_a_typed_refusal() {
        let _court = COURT_LOCK.lock().unwrap();
        let connection = bridge_connection();
        let (root, _) = fixture_dir();
        let sql = format!(
            "SELECT count(*) FROM stardf_bridge_scan('{}')",
            root.0.display()
        );
        let error = connection
            .query_row::<i64, _, _>(&sql, [], |row| row.get(0))
            .unwrap_err();
        assert!(
            error.to_string().contains("table"),
            "refusal must name the absent named parameter: {error}"
        );
    }

    #[test]
    fn unknown_table_is_a_typed_refusal() {
        let _court = COURT_LOCK.lock().unwrap();
        let connection = bridge_connection();
        let (root, _) = fixture_dir();
        let sql = format!(
            "SELECT count(*) FROM stardf_bridge_scan('{}', \"table\" = 'nonesuch')",
            root.0.display()
        );
        let error = connection
            .query_row::<i64, _, _>(&sql, [], |row| row.get(0))
            .unwrap_err();
        assert!(
            error.to_string().contains("unknown table 'nonesuch'"),
            "typed unknown-table refusal: {error}"
        );
    }

    // -- The pushdown flag itself (a court that reads registered intent).

    #[test]
    fn pushdown_is_registered() {
        assert!(
            <StardfBridgeVTab as VTab>::supports_pushdown(),
            "the bridge exists to register projection pushdown; false is the \
             sibling's divergence, not this crate's"
        );
    }
}
