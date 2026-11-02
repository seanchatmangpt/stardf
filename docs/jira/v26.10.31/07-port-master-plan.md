# Port Master Plan — three ports (columnar / SQL / duckdb-scan)

Companion to `06-phase0-columnar-sql-duckdb.md` (§9 wave plan, §10 operator
register) and `02-wave-contracts.md` (W1–W5 shape). This file sequences the
three ports into per-target W-cycles with per-wave gates, ONE real
integration test per wave, and the blocker chain. No new evidence is
claimed here; every grounding claim cites 06 §n.

## 0. Status update — truth-pass (2026-10-31, lane 6)

This section records what is WITNESSED as of 2026-10-31 vs still OPEN;
the rest of the file remains the original plan text.

Witnessed (receipts on disk, cited not gated):

- Columnar crate vendored as workspace member:
  `stardf/arrow/crates/stardf-columnar/` exists in-tree (§3a W2 gate
  target reached at the vendoring level).
- Golden-bytes court in-tree: `crates/stardf-columnar/tests/golden_bytes.rs`
  (5 golden fixtures + 1 negative case = 5+1, §3a W4 fixture set).
- Seam courts 4/4 in-tree: `crates/stardf-columnar/tests/seam_courts.rs`.
- Integration gate `columnar_integration` GREEN twice independently
  (two separate witnessed runs, non-consecutive). This is the §3a W2
  ONE-test exit evidence.
- Oracle gate: PENDING — F5 oracle evidence not yet landed; W4's
  byte-parity claim is court-armed but not oracle-closed.
- **G_COL_EXIT is not yet discharged** — it remains the final arbiter
  and requires the courts to be witnessed as a set, not inferred from
  individual greens.

SQL / duckdb cycles: planned state unchanged (§3b/§3c), but their
dispatch preconditions advanced:

- **D12 probe verdict witnessed** (06 §2 header: algebra crate is
  standalone today) — SQL scope decision input is no longer UNKNOWN.
- **D13 registry-resolve datum witnessed** (frozen-duckdb resolves via
  registry metadata, not just machine-local path).
- Freeze ticket (B0) drafted.
- R1 gate landed **red** — gate exists, receipt not yet admitted
  (red-pending-receipt).

Blocker chain statuses (supersedes §2 prose):

| Blocker | Was | Now |
|---|---|---|
| B0 freeze ticket | DRAFT | **landed-ticket** (drafted; enforcement court still to wire) |
| B1 R1 gate | missing | **landed-red** (gate exists, fails pending receipt) |
| B2 BLAKE3 arbitration | recommendation only | **gate landed** |
| B3 frozen-duckdb rehome | PENDING-OPERATOR | **datum landed** (D13 registry-resolve witnessed); rename-court still OPEN |

### 0.1 Terminal reconciliation (2026-10-31, lane 7)

Additive only: the lane-6 truth-pass above is retained verbatim as
history; this subsection supersedes its OPEN lines.

Witnessed terminal state:

- **G_COL_EXIT witnessed** — the four-target courts ran green as a set
  and the workspace check exited 0. The lane-6 line "G_COL_EXIT is not
  yet discharged" is superseded: the columnar seam gate is discharged.
- **One-package posture supersedes the 6-tier per-crate structure.**
  The executed port posture is one package, not the per-crate tier
  sequencing of §1–§3; those sections remain as historical plan text,
  not a description of the executed structure.
- **Blocker chain B0–B3 fully resolved** (supersedes the §0 table and
  §2 prose; lane-6 states cited from the table above):

| Blocker | Lane-6 state | Terminal state |
|---|---|---|
| B0 freeze ticket | landed-ticket | **resolved** — =2.0.4 ratified |
| B1 R1 license gate | landed-red | **resolved** — license gate green |
| B2 BLAKE3 arbitration | gate landed | **resolved** — blake3 gate landed |
| B3 frozen-duckdb rehome | datum landed | **resolved** — datum witnessed + court PASS at =2.0.4 |

B3's lane-6 open item (rename-court) closed with that court PASS at
=2.0.4.

Consequence for §5: its non-claims ("G_COL_EXIT is NOT discharged",
"W3–W5 remain targets", OPEN-7 oracle-pending) predate this
reconciliation and are superseded by it; §5 is retained verbatim per
lane scope (§0-only edits).

## 1. Dependency order (layering law, 06 §9 / P0-20)

Order is forced, not chosen: **columnar → SQL → duckdb-scan**.
Columnar consumes quads only via the generic `DatasetView`/`ViewTermId`
seam (06 §2); SQL's integration slice consumes columnar; `stardf_scan()`
consumes columnar tables as its storage (06 §6) and is scoped as the SQL
cycle's integration slice (06 §9). avec's first unmet need is the
columnar seam (06 §1, §9) — zero SQL/duckdb consumer need stated
anywhere.

Inter-port gates:

- **G_COL_EXIT**: the columnar seam is real (bridge edges compile,
  digest courts pass). Gates the SQL cycle's start.
- **G_SQL_EXIT**: one real purrdf golden class ported end-to-end over
  the columnar tables. Gates the duckdb-scan cycle's start.
- duckdb-scan starts on code, not docs — its courts are PROPOSAL until
  code exists (06 §6).

## 2. Blocker chain (resolution order)

```
B0 freeze ticket -> B1 R1 gate -> B2 BLAKE3 arbitration -> B3 frozen-duckdb rehome
```

- **B0 — ABI freeze-expansion ticket** (06 §8.2, BLOCKER-CLASS).
  Cut FIRST, enumerating the 21 query/serialize exports + carrier
  structs (06 §5); enforcement is `nm` diff vs ABI-FREEZE.md. Blocks
  ALL SQL-cycle export work and the W1 census additions (11 anon
  structs, 02-wave-contracts W1).
- **B1 — R1 license-dependency gate** (06 §8.5; registered in
  04-falsifiers.md:115, gate missing). Must exist before any avec
  boundary crossing; repo-wide precondition, not port-specific.
- **B2 — KD-08 BLAKE3 single-impl arbitration** (06 §7, §10.3).
  Recommendation: purrdf-hash native (zero-dep/no_std parity) +
  `cargo tree` single-identity gate + digest-parity court. Blocks the
  columnar W4 — digest courts cannot run under a double-hash-impl
  tree.
- **B3 — frozen-duckdb re-homing** (06 §7, §10.5). Machine-local
  absolute-path pin (`/Users/sac/frozen-duckdb`); falsifier 06 §8.3.
  Blocks only the duckdb-scan W5 portable receipt. PENDING-OPERATOR;
  no planning-lane authority to re-home.

Placement: B0/B1 precede all three cycles; B2 precedes columnar W4;
B3 precedes duckdb-scan W5. B0–B2 are engineering-resolvable; B3 is
PENDING-OPERATOR.

## 3. Per-target W-cycles

Each cycle reuses the W1–W5 shape from `02-wave-contracts.md`; each
wave names ONE real integration test (Chicago: real collaborator, real
file, real digest — no mocks). "ONE" = exactly one integration-court
commit per wave; unit/golden courts ride along but are not the wave's
exit evidence. Governor charge deltas (06 §5) ride as unit courts
throughout the SQL cycle.

### 3a. Columnar cycle

- **W1** — Gate: TTL regen with columnar export/type census;
  `--mutate drop-type` refuses.
  ONE test: contract court — regenerated TTL vs hand-checked
  11-anon-struct list (02-wave-contracts W1). Needs B0.
- **W2** — Gate: vendored tier leaves + bridge query returns non-empty
  rows. ONE test: bridge query runs against the 4951-triple contract,
  >0 rows (02-wave-contracts W2). STATUS: vendoring WITNESSED (crate
  in-tree, workspace member); `columnar_integration` GREEN twice
  independently.
- **W3** — Gate: avec-ffi consumer court compiles + passes. ONE test:
  **avec-ffi path-dep court** — stardf-columnar serialize → avec-ffi
  release with allocator-identity assert (02-wave-contracts W3 court;
  06 §4). Needs B1 before this avec crossing.
- **W4** — Gate: digest-equality court purrdf-vs-stardf (06 §8.4) +
  `.sha256` manifest (F4b, 06 §3). ONE test: same quads →
  byte-identical Parquet both sides. Needs B2. Decides OPEN-7.
  STATUS: B2 gate landed; golden_bytes 5+1 + seam_courts 4/4 in-tree;
  oracle gate PENDING (F5) — OPEN-7 still UNKNOWN.
- **W5** — Gate: CI + receipts; `check-layers.py` + license gate +
  G_COL_EXIT recorded. ONE test: CI runs the W4 digest court as a real
  workflow step.

### 3b. SQL cycle

Scope: FULL (all 4 features GO, 06 §5) is the stated direction, but
W1b's census + W3's single golden class keep wave budget fixed until
OPEN-2 resolves. Wave order: aggregation first, then Update, paths,
federation (06 §5 GO list).

- **W1** — Gate: freeze ticket (B0) cut, export set enumerated,
  `nm`-diff enforcement live. ONE test: `nm` diff vs the ABI freeze
  list REFUSES a rogue export. STATUS: B0 landed-ticket (drafted);
  enforcement court not yet wired — OPEN.
- **W1b** — Gate: 21 exports + carrier census in TTL (06 §5). ONE
  test: census court vs 06 §5's own export list.
- **W2** — Gate: governor bind via `Admission`+`GovernorState`
  adapter scaffold. ONE test: adapter compiles against real
  `PurrdfQueryGovernors`/`PurrdfResourceVector` structs.
- **W3** — Gate: one real purrdf golden class ported end-to-end.
  ONE test: **one golden SQL class (aggregation first)** run through
  stardf SQL → results byte-parity vs purrdf golden. Exit = G_SQL_EXIT.
  STATUS: blocked on B1 (landed-red, receipt pending) and D12 verdict
  (witnessed) — dispatch precondition met, receipt gate not.
- **W4** — Gate: full golden-set parity + governor monotone-budget
  invariant (06 §5). ONE test: raising-a-ceiling-never-loses-answers
  court on real governor state.
- **W5** — Gate: CI + receipts; G_SQL_EXIT recorded. ONE test: CI runs
  the W3 golden-parity court.

### 3c. duckdb-scan cycle

Type mapping frozen at W1: int64→BIGINT (UINTEGER is PROPOSAL, 06 §6);
BYTE_ARRAY→VARCHAR; PLAIN-only, no dictionary scans (06 §6). avec
zero-copy expectations are design-only (06 §6) — copy precedents
`avec-beam/ops_scan.rs` and `avec-parquet/scan.rs`.

- **W1** — Gate: stardf_scan contract (5-table schema + type mapping,
  INT64 not u32, 06 §6) frozen in TTL. ONE test: contract court vs the
  06 §6 type-mapping table.
- **W2** — Gate: VTab impl (~6 methods) + in-process registration. ONE
  test: VTab registers against real
  `Connection::register_table_function` (06 §1).
- **W3** — Gate: one real stardf_scan VTab test (06 §9). ONE test:
  **count-parity court** — stardf_scan vs stream `ceil(n/64)`
  cross-check, real queries over real columnar tables.
- **W4** — Gate: all 6 courts pass — value parity, pushdown flip,
  empty-input, typed error, row-count mutant `--mutate drop-one-row`,
  count parity (06 §6). ONE test: the row-count mutant court —
  mutation must be caught.
- **W5** — Gate: CI + portable receipt. Requires B3. ONE test: CI runs
  count-parity + mutant court; portable receipt minted. STATUS: B3
  datum landed (D13 registry-resolve witnessed); rename-court still
  OPEN — W5 stays blocked until that court closes.

## 4. OPEN / PENDING-OPERATOR register

- **OPEN-1** — Which target first (rec: columnar, 06 §10.1) —
  PENDING-OPERATOR.
- **OPEN-2** — SQL scope: full 270K-line SPARQL stack vs subset
  (06 §10.2) — PENDING-OPERATOR.
- **OPEN-3** — BLAKE3 single impl, KD-08 (rec: purrdf-hash native,
  06 §10.3) — PENDING-OPERATOR arbitration.
- **OPEN-4** — ABI freeze-expansion ticket text (06 §10.4) — OPEN,
  engineering-resolvable (= B0).
- **OPEN-5** — frozen-duckdb re-home (06 §10.5) — PENDING-OPERATOR
  (= B3).
- **OPEN-6** — stardf_scan mechanism: in-process VTab (grounded) vs
  loadable dylib (unverified, 06 §10.6) — PENDING-OPERATOR; this plan
  assumes in-process.
- **OPEN-7** — Columnar id encoding Int64-widened vs Int32 golden
  regen (06 §10.7) — decided only by columnar W4 digest court.

## 5. Non-claims

- §3a W1–W2 gate targets are witnessed (see §0); W3–W5 remain targets,
  not results. No wave is declared fully exited except as cited in §0.
- G_COL_EXIT is NOT discharged — it is the final arbiter and stands
  until the court set is witnessed as a whole.
- duckdb courts remain PROPOSAL until code exists (06 §6 verbatim).
- OPEN-7 id-encoding decision is UNKNOWN; only the W4 digest court
  decides it (oracle gate pending F5).
- No git mutations performed; only file updated is this one.
