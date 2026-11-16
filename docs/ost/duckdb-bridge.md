# DuckDB bridge receipt — stardf-duckdb-bridge (lane RDF-8, 2026-11-01)

## Subject

`crates/stardf-duckdb-bridge` — a native DuckDB table function
`stardf_bridge_scan(root VARCHAR, "table" VARCHAR)` over the closed
five-table columnar profile, WITH projection pushdown registered
(`duckdb_table_function_supports_projection_pushdown`).

## Path decision (recorded per dispatch)

Two documented paths (research receipt
`docs/jira/v26.11.1/waves/research-duckdb-arrow.md` S4):

- **(a) C Data Interface producer + `duckdb_arrow_scan`** — NOT implemented
  this lane. Reasons: (1) `duckdb_arrow_scan` availability in the vendored
  frozen-duckdb-sys bindgen allowlist was not verifiable this session
  (bindgen-generated bindings; not confirmed), (2) path (a) inherits
  materialize-then-filter semantics for RecordBatchReader-class producers
  (research receipt S1), so the pushdown driver of DuckDB-Arrow performance
  is unreachable there.
- **(b) native table function with projection pushdown** — IMPLEMENTED.
  Reasons: the frozen-duckdb 2.0.4 SDK exposes the exact documented contract
  (`VTab::supports_pushdown` -> `TableFunction::supports_pushdown` ->
  `duckdb_table_function_supports_projection_pushdown`, vtab/mod.rs:144;
  `InitInfo::get_column_indices` reading
  `duckdb_init_get_column_index`, vtab/function.rs:165-174), and the
  sibling `stardf-scan-duckdb` crate registers `stardf_scan()` with
  `supports_pushdown() = false` — the exact in-tree gap.

## What is consumed read-only (no sibling modified)

- `stardf_columnar::{read, ParquetFiles, Table, schema::PhysicalType}` —
  the receipted validated five-table decode path (full physical + semantic
  validation, fail-closed; byte-corrupted pages are a typed refusal).
- `frozen_duckdb::duckdb::{Connection, vtab::{BindInfo, InitInfo, VTab,
  TableFunctionInfo}, core::{DataChunkHandle, FlatVector, Inserter,
  LogicalTypeHandle, LogicalTypeId}}` — in-process registration.

## Courts (all in `src/lib.rs` tests; real engine, real files, real SQL)

Run: `CARGO_TARGET_DIR=/tmp/r8m-target cargo test -p stardf-duckdb-bridge`
— 8 passed / 0 failed, exit 0, this session.

1. **Equivalence** (`five_table_counts_and_values_match_the_canonical_
   projection`): DuckDB `SELECT *` over `stardf_bridge_scan` vs direct
   stardf decode via `stardf_columnar::project` — identical rows for all
   five tables (terms/quads/reifiers/annotations/blobs). Two independent
   decode paths (reader vs writer-side projection), non-circular.
2. **Projected-select parity** (`projected_select_matches_direct_decode_...`):
   `SELECT s, o FROM stardf_bridge_scan(...)` == direct decode of the same
   columns, same row order.
3. **Projection pushdown is observable and pruned** (`projection_pushdown_
   prunes_cells_written`): a global cell counter ([`bridge_cells_written()])
   reads the exact cells DuckDB made the producer write. Under a shared
   COURT_LOCK (concurrent test threads pollute the global counter — a real
   race the court caught), `SELECT *` writes k·N·C cells and `SELECT o`
   writes k·N cells (k = DuckDB internal scan passes, asserted equal). A
   producer ignoring the projected list cannot produce the pruned count.
4. **Anti-vacuity flip + fail-closed restore** (`corrupted_decoded_column_
   diverges_on_the_duckdb_side`): the literal `"hi"` is corrupted to
   `"corrupted-in-court"` BEFORE encode/write (pre-exposure). The DuckDB-side
   terms scan must (and does) carry the corrupted lexical form verbatim
   (baseline did not change quads ids — dense interning preserves ids, the
   lex cell is where the corruption lands; first draft compared quads and
   the court corrected the lane). Restore = rebuild the unmutated fixture:
   byte-identical files (determinism proof) and byte-identical scan. Flip is
   loud, restore is byte-verified.
5. **Byte-level page corruption** (`byte_corrupted_parquet_is_a_typed_
   refusal`): flipping a mid-file byte in quads.parquet is a typed
   `profile validation refused` through DuckDB's error channel — fail-closed,
   never a wrong row.
6. **Typed bind refusals** (absent/unknown `table` parameter): enumerated
   five-name refusals through the FFI error channel.
7. **Flag court** (`pushdown_is_registered`): `supports_pushdown() == true`
   (the sibling's documented divergence is `false`).

## Contract cites (documented, fetched live by the research receipt 2026-11-01)

- `duckdb_table_function_supports_projection_pushdown` —
  https://duckdb.org/docs/2.0/clients/c/table_functions (bind/init/main
  contract; `duckdb_init_get_column_count`/`duckdb_init_get_column_index`
  "only populated if ... projection pushdown ... set"; default false).
- `duckdb_arrow_scan` / `duckdb_arrow_array_scan` (unimplemented path (a)) —
  https://duckdb.org/docs/2.0/clients/c/api.
- Pushdown tiers: https://duckdb.org/docs/2.0/guides/python/sql_on_arrow
  (RecordBatchReader = materialize-then-filter; projection pushdown is the
  documented performance driver).
- Rust surface: frozen-duckdb 2.0.4 `src/duckdb/vtab/mod.rs:144` and
  `src/duckdb/vtab/function.rs:165-174,229-241` (registry sources listed in
  the receipt; vendored SDK consumed as a crates.io dep, unmodified).

## Divergences / UNKNOWN (honest)

- **Row-filter pushdown to custom producers: UNKNOWN** — not part of the
  documented C table-function contract; never claimed. Projection-only.
- **Bind-time materialization**: rows are materialized at bind through the
  public reader (codec raw row surfaces are `pub(crate)`); the projection
  delta is emission-side. Zero-copy/decode-side pruning is future work.
- `duckdb_arrow_scan` presence in frozen-duckdb-sys: UNVERIFIED (path (a)
  unimplemented for this reason, not silently skipped).
- Parallel emission deferred (single-threaded claim cursor, as the sibling).

## Standing

ALIVE (path (b)): built, courts run and pass this session; clippy-clean for
the crate; workspace `cargo check --workspace` exit 0 with the member added.
Workspace-lint warnings about missing `[lints]` inheritance on OTHER crates
(stardf-star, cgap-*, stardf-shapes, stardf-arrow root manifest) are
pre-existing, not this lane's.

## Files

- `crates/stardf-duckdb-bridge/Cargo.toml` (new)
- `crates/stardf-duckdb-bridge/src/lib.rs` (new, ~1190 lines)
- `Cargo.toml` (root manifest: member entry + workspace dep entry — required
  by workspace membership, disclosed)
- `docs/ost/duckdb-bridge.md` (this receipt)
