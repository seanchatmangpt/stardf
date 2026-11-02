# SCAN-COMPLETION

Lane 17 (read-only design assembly), 2026-10-31. Subject: completing
`stardf-scan-duckdb` — the native in-process `stardf_scan()` DuckDB table
function over the five columnar tables. This note assembles the stopped
campaign's W5 scan designs from their receipted sources; no scan code was
written or executed in this lane. Rows without in-tree evidence are marked
PROPOSAL.

## 1. Ground truth (re-read this session, file:line)

| Subject | State |
|---|---|
| `crates/stardf-scan-duckdb/` | ABSENT — greenfield |
| `stardf-duckdb` (lib `purrdf_duckdb`) | EXISTS, member, vendored-verbatim; engine only — no VTab |
| `frozen-duckdb` pin | req `2.0.3` caret; lock resolves **2.0.4 registry** — float real here |
| `[patch.crates-io]` | ABSENT from the stardf/arrow root manifest |
| VTab SDK surface | proven, in-process |
| Five-table schema | closed set, Int64 physical ids |
| stardf-columnar lib target | `write`/`read`/`Table`/`TableSchema` exported |
| External DuckDB oracle | green: counts=6,3,0,0,2; terms=BIGINT,VARCHAR; blobs=BLOB |
| `/Users/sac/frozen-duckdb` | present at session start; ENOENT minutes later (§6.4) |

Evidence, per row (all re-read this session):

- Scan crate absent: not in `ls crates/`; not a member of
  `stardf/arrow/Cargo.toml`; the only tree-wide `stardf_scan` hit is
  `docs/OPERATOR-DECISIONS.md:426` (prose).
- stardf-duckdb: `crates/stardf-duckdb/Cargo.toml:17-21` (lib-name law);
  `src/` = `lib.rs` + `eval.rs` (engine + row projection).
- frozen-duckdb pin: `stardf/arrow/Cargo.toml:244`;
  `stardf/arrow/Cargo.lock:993-1033` (`frozen-duckdb`/`-sys`/`-builder`
  all `2.0.4`, source `registry+…`).
- `[patch.crates-io]`: grep this session (only a comment at line 231).
- VTab SDK: registry `frozen-duckdb-2.0.4/src/duckdb/vtab/mod.rs:41-81`;
  registration `register_table_function` `mod.rs:140-149`.
- Five-table schema: `crates/stardf-columnar/src/schema.rs:92-98`
  (`Table::ALL`); `:48` (`PhysicalType::Int64`).
- stardf-columnar target: `crates/stardf-columnar/src/lib.rs:63-66`.
- Oracle gate: `scripts/check-columnar-oracle.sh`; RECEIPTS LANE-8 row
  (exit 0).
- frozen-duckdb checkout: observation registered in §6.4; the trait was
  read from the registry source instead.

## 2. Harvested design inputs (receipted sources)

- **06-phase0 §6** (P0-4/15/16/17): greenfield VTab (~6 methods) over the
  five columnar tables; columnar term ids are INT64 not u32; PLAIN-only
  (no dictionary scans); mapping int64→BIGINT, BYTE_ARRAY→VARCHAR;
  `supports_pushdown()` for s/p/o pruning; avec zero-copy expectations are
  design-only; precedents `avec-beam/ops_scan.rs` + `avec-parquet/scan.rs`;
  6-court proposal. **All PROPOSAL until code exists.**
- **07-port-master-plan §3c**: cycle shape — W1 contract freeze (5-table
  schema + type mapping, INT64 not u32, frozen in TTL) → W2 VTab impl +
  in-process registration → W3 one real count-parity court → W4 all six
  courts + mutant → W5 CI + portable receipt (requires B3).
- **13-v26111-target-ledger**: "Native in-process `stardf_scan()` VTab" is
  the W5 in-flight row (rehome court → scan scaffold → 6 courts + mutant);
  errata #4: the stardf VTab courts assert **allocation-identity, not a
  zero-delta constant** — the `{0,0}` delta is the avec ops_scan precedent
  and must not be imported as a stardf claim.
- **10-duckdb-rehome-plan §5**: mechanism = in-process VTab (OPEN-6,
  grounded; loadable dylib UNSUPPORTED until a `LOAD`-based court exists) —
  operator ratification only. §1: the B3 rename-court script shape
  (rename both machine-local roots away, build must still succeed,
  fail-closed restore digests).
- **VTab trait (frozen-duckdb 2.0.4, registry)**: `type InitData` and
  `type BindData` (`Sized + Send + Sync`), `bind`, `init`, `func`,
  `supports_pushdown()` (default false), `parameters()` (default None)
  — `vtab/mod.rs:41-81`. `BindInfo`: `add_result_column`, `set_error`,
  `get_parameter`, `get_named_parameter`, `set_cardinality`
  — `vtab/function.rs:35-124`. Live registration examples:
  `vtab/arrow.rs:1335+` (`register_table_function::<ArrowVTab>("arrow")`).
- **Zero-copy precedents** (design-only): `avec-beam/ops_scan.rs` — view
  aliases the parent payload, pointer identity `data_ptr == base + start`,
  counted-site alloc delta `{0,0}`; `avec-parquet/scan.rs` — PLAIN payload
  bytes alias the mapping, fail-closed typed `ScanError`, bounded slice
  arithmetic, rep/def levels present = typed refusal.

## 3. Crate shape (PROPOSAL — placement is engineering-discretionary)

New crate `crates/stardf-scan-duckdb/` as a workspace member. Rationale:
`stardf-duckdb` is vendored-verbatim (`purrdf_duckdb`, upstream purrdf
rev 0da05783); `stardf_scan()` is greenfield first-party code with no
upstream counterpart — grafting it into the vendored crate would break the
byte-verbatim provenance posture and re-vendor diffing. A separate crate
keeps the vendored member's diff at zero and gives the scan surface its
own gate target (`-p stardf-scan-duckdb`).

- **Deps** (all through `[workspace.dependencies]`, single-pin law):
  `stardf-columnar` (tables + `write`/`read`), `frozen-duckdb` (VTab
  trait + connection). Dev-dep for the fixture: `stardf-columnar`'s
  `write` already covers fixture generation; `stardf-arrow-core` only if
  the splitmix64 term fixture needs the kernel side.
- **Module layout**: `src/lib.rs` (registration facade), `src/bind.rs`,
  `src/init.rs`, `src/func.rs`, `src/types.rs` (the frozen W1 type map),
  `src/error.rs` (typed per-surface reason enums, `ColumnarError` shape).
- **Crate law**: `#![forbid(unsafe_code)]` in stardf code; the VTab trait
  machinery's unsafe lives inside `frozen-duckdb` (the only unsafe edge is
  the SDK's own `extern "C"` shims — same posture as columnar: FFI unsafe
  stays in the boundary crate, not first-party scan logic).
- **Naming**: function registers as `stardf_scan` — one parameter: the
  dataset root dir (VARCHAR path holding the five Parquet files); named
  parameter `table` (VARCHAR, optional) to bind a single table. Type map
  per §4; naming beyond this is PROPOSAL.

## 4. VTab implementation order (exact)

Frozen at W1 first — the type map is contract, not code:

| Order | Step | Gate |
|---|---|---|
| W1 | Freeze stardf_scan contract in TTL | contract court vs 06 §6 type-map |
| W2a | `bind` (detail below) | registration test |
| W2b | `init` (detail below) | registration test |
| W2c | `func` (detail below) | registration test |
| W2d | in-process registration (detail below) | real registration vs real engine |
| W2e | pushdown LAST (detail below) | C3 flip |
| W3 | count-parity court | court green |
| W4 | full court set + mutant | all six green + C6 flips |
| W5 | CI + portable receipt | receipt minted (requires B3) |

Step detail:

- **W1**: freeze the five-table schema, type mapping (INT64 ids → BIGINT;
  BYTE_ARRAY→VARCHAR; PLAIN-only), parameter surface, and error classes in
  the contract TTL; a `--mutate drop-type`-class refusal court rides along
  (F1a pattern).
- **W2a — `bind`**: parse the root-dir param; open + validate the
  five-table profile via `stardf_columnar` reader validation
  (fail-closed); emit result columns via `BindInfo::add_result_column`
  (terms: id BIGINT, lex/lang/datatype VARCHAR, …; quads: s/p/o BIGINT
  required, g BIGINT optional → nullable); set exact cardinality when
  known (`set_cardinality(n, true)`).
- **W2b — `init`**: build `InitData` (Send+Sync) — open reader handles,
  row-group cursor, per-thread split; `set_max_threads` when parallel.
- **W2c — `func`**: populate the output `DataChunkHandle` in bounded
  chunks from PLAIN columns; INT64 widen/narrow with checked conversion
  (fail-closed on out-of-range words); EMPTY chunk when done.
- **W2d**: `Connection::register_table_function::<StardfScanVTab>(
  "stardf_scan")` — the mod.rs:140-149 path.
- **W2e**: `supports_pushdown()` → true + s/p/o/g BIGINT filter
  consumption LAST — default false until C1–C3 pass without it, so the
  pushdown flip (C3) has a real before/after.

Order rationale: bind/init/func is the trait's own dependency order
(bind defines the column surface init/func emit); registration before
pushdown keeps the C3 flip meaningful (a court that only ever saw
pushdown-on cannot witness a flip); the type map freezes first because
every later court asserts against it.

## 5. Court inventory (06 §6 six-court proposal, concretized)

All six are PROPOSAL until code exists (06 §6 verbatim). Anti-vacuity law
applies: each court ships with its positive control; a skip is loud; a
mutation a court accepts is VACUOUS GATE = FAIL.

- **C1 — count parity (the W3 court)**: `SELECT count(*)` per table
  through `stardf_scan()` == `read_parquet` on the same files == the
  stream-side arithmetic cross-check `ceil(n/64)` over the fixture's
  64-row slabs. Fixture: `stardf_columnar::write` of the splitmix64
  fixture (state `0x6012`) — retargeting `scripts/check-columnar-oracle.sh`'s
  fixture at the columnar writer, exactly as that script's own comment
  anticipates (it currently builds via stardf-arrow-core; counts 6,3,0,0,2
  are the standing floor witness).
- **C2 — value parity**: `SELECT *` through `stardf_scan()` vs
  `read_parquet` — row-set equality on every column, both orderings of the
  fixture. Pins the INT64→BIGINT and BYTE_ARRAY→VARCHAR mapping at the
  value level, not just counts.
- **C3 — pushdown flip**: with `supports_pushdown()=true`, a query with an
  s/p/o BIGINT equality filter must return the filtered row set AND a
  mutant that ignores the pushed filter must be caught. Before W2e lands,
  this court is loud-SKIP with its reason (pushdown not yet enabled), never
  a faked pass.
- **C4 — empty-input**: fixture dataset with zero quads → zero rows, exit
  clean, no error surfaced through the FFI boundary.
- **C5 — typed error**: missing table file / invalid profile at bind →
  typed error via `BindInfo::set_error` (the `ColumnarError` reason-enum
  shape), never a panic across the C ABI.
- **C6 — row-count mutant (anti-vacuity positive control)**:
  `--mutate drop-one-row` removes one fixture row; the court MUST catch it
  (count parity breaks). A court that passes under the mutation is a
  vacuous gate = FAIL; loud `[SKIP]` only with a stated reason.
- **Allocation-identity rider (not a seventh court)**: scan courts assert
  allocation-identity per the ledger errata — released pointers were
  minted by the allocator that owns the release path — not a `{0,0}`
  zero-delta constant (that is the avec ops_scan precedent, release errata
  #4 in 13-v26111-target-ledger).

## 6. Gate dependencies

### 6.1 Workspace-green gate (the dispatch gate)

The scan lane adds a root-manifest member + a new crate — shared-file
edits on one canonical checkout. Dispatch and landing conditions:

```bash
cargo test --workspace --no-fail-fast          # → 0
cargo fmt --all -- --check                     # → 0
cargo clippy --workspace --all-targets -- -D warnings   # → 0
python3 scripts/verify.py                      # ×2, byte-identical
python3 scripts/verify.py --mutate drop-export # → 1 (flip witnessed)
```

- Ladder per the RECEIPTS LANE-11 row (the F7+C2 wave-finish ladder, last
  witnessed green at parent HEAD `c9f8028`). The scan lane lands only on a
  tree where the ladder is green at start and leaves it green — the scan
  courts join the ladder once landed (pass + flip under C6).
- Compile-freeze SLA: the root-manifest edit must fix-or-revert within
  ~10 minutes if it breaks the shared build (same-checkout law); the new
  crate must compile clean under `clippy --all-targets -D warnings`
  (workspace pedantic/nursery lints are warn-only, capi gate is not).
- One write-campaign at a time on the tree (12-full-parity-campaign launch
  precondition): the scan lane dispatches only when the completion
  campaign's arbiter/coordinator releases the tree.

### 6.2 B3 — frozen-duckdb re-home (blocks W5 only)

- **Precondition already met on this tree**: no `[patch.crates-io]`; lock
  sources are `registry+…` (re-witnessed this session, Cargo.lock:993-1033)
  — the registry-only posture (rehome option c shape) is effectively in
  force at the workspace level.
- **The rename court itself is UNRUN**: port `10-duckdb-rehome-plan §1`
  court (`mv` both machine-local roots away → `cargo test -p
  stardf-scan-duckdb` must stay green → byte-exact restore, fail-closed)
  as e.g. `scripts/court_frozen_duckdb_rehome.sh` (PROPOSAL). 07 §3c W5:
  W1–W4 are NOT blocked by B3; only the portable receipt is.

### 6.3 Pin decision (PENDING-OPERATOR, small)

Manifest req `2.0.3` (caret) vs lock 2.0.4: the family floats today. The
W1 contract freeze should pin `=2.0.x` exactly (10-duckdb-rehome-plan §6
item 2) so the courts' substrate (duckdb.h 2.0.0 C API per 06 §1) cannot
move between court runs. Golden-regen exposure is nil for the scan surface
itself (scan courts assert counts/values, not Parquet bytes) but the
engine binary must not drift mid-ladder.

### 6.4 Session observation (registered, no conclusion)

`/Users/sac/frozen-duckdb` was readable at this session's start (fork
sources grepped successfully) and ENOENT minutes later — consistent with
a concurrent re-home actuation (D13 option a is the recommended posture).
No git/filesystem operation in this lane; the trait surface was read from
the registry source, which is what the lock actually compiles against.

## 7. Standing

| Subject | Standing | Basis |
|---|---|---|
| This design note | ALIVE (landed this session; every citation re-read on disk) | the file itself |
| `stardf-scan-duckdb` crate | UNKNOWN (absent; nothing started) | §1 row 1 |
| W1–W5 scan implementation | NOT STARTED (courts PROPOSAL) | 06 §6 verbatim; 07 §5 |
| B3 rename court | UNVERIFIED (court unrun; precondition met) | §6.2 |
| Mechanism (in-process VTab) | PENDING-OPERATOR ratification (OPEN-6) | rehome-plan §5 |
| frozen-duckdb `=` pin | PENDING-OPERATOR | §6.3 |

## See Also

- `stardf/docs/jira/v26.10.31/06-phase0-columnar-sql-duckdb.md` §6 — the scan design source
- `stardf/docs/jira/v26.10.31/07-port-master-plan.md` §3c — the W-cycle gates
- `stardf/docs/jira/v26.10.31/10-duckdb-rehome-plan.md` §1/§5/§6 — B3 court + mechanism
- `stardf/docs/jira/v26.10.31/13-v26111-target-ledger.md` — in-flight row + errata #4
- `stardf/arrow/docs/RECEIPTS.md` — the workspace-green ladder witness
- `stardf/arrow/scripts/check-columnar-oracle.sh` — the external-oracle precedent
- `/Users/sac/avec/crates/avec-beam/src/ops_scan.rs` — zero-copy scan precedent
- `/Users/sac/avec/crates/avec-parquet/src/scan.rs` — mmap PLAIN-scan precedent
