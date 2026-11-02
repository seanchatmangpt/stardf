# Phase-0 Synthesis — columnar / SQL / DuckDB ports (v26.10.31)

## 0. Post-Phase-0 outcome (2026-10-31, lane 5 plan-update)

Phase-0 inputs consumed and landed:

- **Columnar vendored** per the §2/§3 seam analysis — u32 dictionary id space
  (§2 D3/D4: graph slot `Option<i64>`→u32 BOTTOM=absent, cap `u32::MAX-2`),
  Int64-widened physical encoding (§10 decision 7, golden-stable option),
  with the **BLAKE3 divergence disclosed in PROVENANCE** (§7 KD-08: stardf
  blake3 crate carried as a registered divergence, not silently merged).
- **D12 probe landed** (W1-7): the algebra crate is standalone TODAY; the
  results crate is deeply-coupled (~25 symbols) — confirming the §1 scale
  verdict that any SQL port must name its subset.
- **DuckDB re-home datum landed** (W1-9): `frozen-duckdb` 2.0.3
  registry-resolves; the sys/builder crate floats — closing §10 decision 5's
  precondition (portable receipts no longer blocked on the local path pin).
- **Freeze ticket GL-STARDF-002 drafted** per §8 R2 (freeze-expansion ticket
  FIRST, before any query export work).
- **R1 gate landed red** per §7/§8 R5 — the dependency-edge license gate is
  in place and currently failing, which is the correct loud state prior to
  avec boundary crossing.

Everything below §0 is the historical Phase-0 receipt, unchanged.

## 0a. Final-outcome addendum (2026-10-31, doc lane 6)

§0 is the lane-5 mid-state. The final state supersedes it on four points, all
re-read on disk this lane (no git operations; run receipts cited from the
landed evidence files, not re-executed):

1. **duckdb — `=2.0.4` ratified, rehome registry-resolves.** §0's W1-9 datum
   ("`frozen-duckdb` 2.0.3 registry-resolves; sys/builder floats") is
   superseded by D13 RESOLVED / D19 DECIDED (`docs/OPERATOR-DECISIONS.md:648`,
   `:715`): operator ratified `frozen-duckdb`/`-sys`/`-builder` all `=2.0.4`,
   landed at workspace `Cargo.toml:282-284`. The F04 rehome falsifier FIRED
   cache-absent on `=2.0.3` (fail-closed court path, two-arm
   `scripts/verify_duckdb_rehome.sh`), so the `=2.0.3` failure class is moot
   under the ratified pins; the resolve is registry-only (lock source
   `registry+…`, no `[patch.crates-io]`, no machine-local path). Nonclaim:
   the `=2.0.4` cache-absent court re-run is NOT witnessed (D13/D19 blocks).
2. **scan VTab landed with 6 courts, oracle through `stardf_scan`.** §0 named
   `stardf_scan()` greenfield; final: `crates/stardf-scan-duckdb` is a
   first-class member (publish=true, avec-decoupled, `vtab` feature under the
   `=2.0.4` pin) with the in-process VTab (`register_stardf_scan`,
   `crates/stardf-scan-duckdb/src/lib.rs:357`) and exactly six courts c1–c6
   (`tests/scan_courts.rs`). G10 re-witness
   (`evidence/g10-scan-courts-rewitness-2026-10-31.json`, standing ALIVE,
   postdating the V26-11-1-VERDICT "courts UNKNOWN-as-receipt" row):
   `cargo test -p stardf-scan-duckdb --locked` exit 0, 12/12 across two
   witnessing runs — c1 count parity (scan == read_parquet == slab
   arithmetic, five tables), c2 value parity (ORDER BY pinned), c3 LOUD SKIP
   by landed design (`supports_pushdown()=false`, the W2e divergence;
   filtered-row-set + mutant-catch legs arm only when pushdown flips), c4
   empty-input clean, c5 three typed-refusal legs, c6 mutant flip witnessed
   in-run. The external oracle gate ran in tier-3 in-process registration
   mode **through `stardf_scan`** (`just columnar-oracle` exit 0;
   standing-fixture counts 1030/400/108/108/9, engine == generator
   projection, types BIGINT/VARCHAR).
3. **D12 — resolved whole-vendor per one-package; slice coexistence.** §0's
   probe verdict ("results crate deeply-coupled; any SQL port must name its
   subset") is superseded at tree-shape level by D15's one-package rule
   (OPERATOR-DECISIONS.md, 80/20 re-scope): whole-vendor straight-copied the
   entire purrdf crate set including `stardf-sparql-eval` (member,
   `Cargo.toml:168`) and `stardf-sql` (member). The slice artifacts COEXIST
   rather than deleted: `stardf-slice` is a member (`Cargo.toml:165`) and
   `crates/stardf-rdf-core-slice` is retained on disk but loudly excluded
   from members (registered PARTIAL standing, escalated to the W1 vendoring
   queue, `Cargo.toml:148-155`). The slice remains the admitted PARITY scope
   only (12-full-parity-campaign GAP 1; D17: copied crates carry no courts).
4. **columnar vendored** (§0 held) — `stardf-columnar` is a member with
   `src/vendor/` leaves and is the physical substrate the scan surface reads
   (G10 c1/c2 and the tier-3 oracle run real `stardf_columnar::write` bytes
   through `stardf_scan`).

---

Input: 20-lane read-only explore wave (2026-10-31, P0-1..P0-20). Companion to
01-master-plan / 02-wave-contracts / 03-subsystem-map / 04-falsifiers / 05-next-steps.
All claims carry file:line from the lane receipts; UNKNOWN/PROPOSAL marked where ungrounded.

## 1. Ground truth (P0-1, P0-4, P0-15)

The named targets exist as real crates — the premise "barely touched" was right, the
assumed shape was wrong:

| Crate | Reality | Size/Tests |
|---|---|---|
| `purrdf-sql` | EXISTS — SQL-in-RDF AST + deterministic compiler (NOT an engine) | 6,064 ln / 4 golden |
| `purrdf-duckdb` | EXISTS — embedded engine over `frozen-duckdb` 2.0.3 (local fork, pinned `/Users/sac/frozen-duckdb`), `CREATE VIEW ... read_parquet` over 5 tables. **Not an extension.** | 477 ln / 38 |
| `stardf_scan()` DuckDB extension | **DOCS-ONLY, greenfield** — zero code in all three trees; the avec mapping doc says so itself (`03-subsystem-mapping.md:1-7`) | — |
| SPARQL stack | algebra 35K/640 · **eval 215K/2,678** · results 10K/223 · conformance 9K/184 | ~270K ln total |
| `purrdf-columnar` | hand-rolled Parquet subset, `#![forbid(unsafe_code)]`, byte-deterministic | 8,697 ln / 37 |

DuckDB SDK: **RESOLVED, not UNKNOWN** — `frozen-duckdb/prebuilt/duckdb.h` (2.0.0 C API) +
Rust `VTab` trait with proven in-process registration (`Connection::register_table_function`,
live examples). No dynamic-load entry macro locally; recommend in-process table function.

Scale verdict: the SPARQL stack is a different reconstitution magnitude from stardf-arrow
(13 exports). Any SQL port plan must say which subset, or it is not a plan.

## 2. The seam (P0-5, P0-11)

**Columnar consumes quads ONLY via the generic `DatasetView`/`ViewTermId` trait** — never
raw purrdf TermIds. All term identity is re-minted as local i64 dictionary indices
(`writer.rs:315-376`); `QuadRow = (i64,i64,i64,Option<i64>)` are dictionary indices, not
backend ids. The stardf seam is therefore exactly two layers: the Dictionary id space and
the physical `Int64Column`. `Option<i64>` graph slot collapses to `u32` with BOTTOM=absent
(D3); dictionary cap becomes `u32::MAX - 2` (D4); D2's offset cancellation makes carrying
`D::Id::raw()` words straight through legal, deleting the TermValue re-mint for id purposes.

**Evaluator never uses integer sentinels** — unbound is `Option<TermId>`/`GraphMatch`/
`SolutionTerm` enums; raw words appear only inside `QuadKey` masks (`dataset.rs:722,745`).
Port = `ViewTermId` impl + 5 named bridge edges (`QuadKey::new/misses`, `ProbePattern`
construction in `bgp.rs:647/716`, `SolutionTerm::join_key` `scratch.rs:112`,
`ViewTermId::encode` `dataset_view.rs:415`). Cursors are already id-generic. Secondary id
space: `ScratchId` (0-based u32, bit-32 disjointness via `encode_computed` must survive).

## 3. Byte-determinism (P0-2, P0-6, P0-9)

Law: total-order dictionary assignment from `BTreeSet<TermValue>` (never ingest order),
tuple-set row ordering, digest-ordered blobs, exact PLAIN/RLE/DataPageV2 recipe, Zstd
level-3 single-frame pinned `structured-zstd 0.0.49`. **Determinism survives u32** — ids
serialize as PLAIN INT64; carry-points: `i32::MAX` dictionary cap → `u32::MAX-2`, checked
conversion, golden regeneration with recorded reason. Fixtures are splitmix64-seeded
(`state=0x6012`) — regeneration is F2a-safe by construction. Zstd digests are same-lock
frozen only; treat Uncompressed as cross-version frozen or pin the compressor.

Oracles port directly: 10 pinned digests + round-trip identity + 9 structured-error courts
+ DuckDB external oracle gate (`check-columnar-oracle.sh`). F4b gap: purrdf pins digests in
Rust source, not a `.sha256` manifest — stardf must add the manifest (per 04-falsifiers).

## 4. Egress priority (P0-8)

CanonArrow (crates/arrow header + sparql-results slab encoder) and columnar (Parquet)
**share nothing** — no code, no wire bytes, no dep either direction. CanonArrow is the
query-results egress at every host boundary (C FFI `query.rs:924`, CLI, WASM; HTTP 406s by
design); columnar serves the DuckDB/DataFrame path. Priority: **CanonArrow first** for
query-result egress; columnar for the stardf_scan/SQL path.

CanonArrow gaps (P0-13): `encode_batch` hand-rolls slabs from a fresh `Vec<u8>` — no
avec-arrow Batch/ColumnBuilder, so T2 typed zero-copy ingest is unmet on this surface; the
wiring swap is the avec integration point. All 5 formats byte-frozen; `abi.rs:1039`
JSON-LD/YAML-LD masters are a disjoint surface (dataset egress).

## 5. SQL scope (P0-10, P0-12, P0-14)

- **21 exports** (17 query/UPDATE + 4 serialize; CONSTRUCT is result-kind 1, not an
  export) + full carrier-type inventory (ResultKind, QueryGovernors, GovernorEvidence +
  8-dim expression_errors, PartialCertificate, RowCursor, geo/division selectors).
  Gap is total vs the 13-export ABI — but header pre-1.0 rule makes additions append-only.
- **GO on all four features**: SPARQL 1.1 Update (transactional COW→apply→freeze→publish,
  atomicity-tested), aggregation (incl. custom/statistical/FOLD), full property paths
  (+Range/Wildcard ext), capability-gated federation (resolver-injected — preserve the
  injection point). Each with dedicated multi-hundred-line suites.
- **Governors**: 8-dimension ResourceVector, pinned precedence, sticky trip, evidence on
  the success path. Bind via existing `Admission` (bytes) + `GovernorState` adapter;
  **charge deltas not absolutes** (`claim_uncharged_growth` watermark); preserve the
  monotone-budget invariant (raising a ceiling must never lose answers).

## 6. DuckDB (P0-4, P0-15, P0-16, P0-17)

`stardf_scan()` is greenfield: a `VTab` impl (`bind/init/func`, ~6 methods) over the five
columnar tables, in-process registration. Columnar term ids are **INT64, not u32**
(premise correction); dictionary scans do not exist (PLAIN-only). Type mapping:
int64→BIGINT (or new UINTEGER mapping, PROPOSAL), BYTE_ARRAY→VARCHAR, `supports_pushdown()`
for s/p/o pruning. avec expects (design-only, `03-subsystem-mapping.md:110-118`): zero-copy
Arrow-compatible vectors, branchless TermId mask pushdown pre-materialization, 5-table OCEL
schema, 4 ASOF/window query classes. Landed zero-copy precedents to copy:
`avec-beam/ops_scan.rs` + `avec-parquet/scan.rs`. Courts: 6-court proposal (count parity
cross-checked vs stream ceil(n/64), value parity, pushdown flip, empty-input, typed error,
row-count mutant via `--mutate drop-one-row` knob). All PROPOSAL until code exists.

## 7. Dependency + license closure (P0-18)

GO — small first-party closures on the known Tier map (+`structured-zstd` for columnar).
Port `layers.toml` + `check-layers.py` alongside. License conjunctions per-file
(`rdf-core` = tri AND Unicode-3.0; lex html entities BSD-3-Clause; copy `LICENSES/` +
`PROVENANCE.md` structure). **KD-08 BLAKE3 arbitration first**: purrdf-hash native vs
stardf's blake3 crate — pulling both violates the single-impl decision; recommend
purrdf-hash (zero-dep/no_std parity) + `cargo tree` single-identity gate + digest-parity
court. `frozen-duckdb` is a machine-local absolute-path pin — re-home before portable
receipts.

## 8. Risks (P0-19) — each with falsifier

1. **TermId sentinel divergence in evaluator** (HIGH): differential match-set court +
   `from_raw(u32::MAX)` capacity court + `Option<TermId>`-in-repr(C) lint (KD-02/KD-06
   spec; courts do not exist yet).
2. **ABI freeze vs query exports** (BLOCKER-CLASS): cut the freeze-expansion ticket FIRST,
   enumerating the query export set + carrier structs; `nm` diff vs ABI-FREEZE.md is the
   enforcement.
3. **DuckDB SDK hostability** (downgraded from absence): falsifier = build with
   `$HOME/.frozen-duckdb` + `/Users/sac/frozen-duckdb` renamed away.
4. **Parquet determinism under remap** (MEDIUM): purrdf-vs-stardf digest-equality court
   pins whether the `index+1` convention crosses the write boundary.
5. **R1 license skew** (registered, gate missing): dependency-edge gate per
   04-falsifiers.md:115 before any avec boundary crossing.

## 9. Wave plan (P0-20, corrected with session receipts)

Sequencing is **forced by the layering law** (process → arrow → sql; nothing reaches past
the layer below): **columnar first, SQL second, DuckDB scan last** (as the SQL cycle's
integration slice). avec's first unmet need is the columnar seam (CONSUMER_REQUIREMENTS
T1–T4 all LANDED; zero SQL/duckdb consumer need stated anywhere).

Corrections vs P0-20's doc-only reads (session receipts supersede):
- stream_courts "contradiction" resolved: the lane-16 receipt was mid-wave; final arbiter
  ladders (F7 + C2) witnessed `cargo test --workspace` 69/0 with stream_courts 6/6.
- W1 census + `--mutate drop-type` ARE landed (C17/F7); what remains open is W1's
  mapping additions (11 anon structs) — the census correctly FAILs pre-fix.

Proposed cycle per target: W1 contract (TTL gains query/columnar export census; freeze
ticket for new exports) → W2 scaffold (vendored tier leaves + bridge query) → W3
integration slice (columnar: avec-ffi consumer court; SQL: one real purrdf golden class
ported; duckdb: one real stardf_scan VTab test) → W4 equivalence (digest-equality courts,
snapshot pins, GTS manifests) → W5 CI + receipts.

## 10. Operator decision register (OPEN)

1. Which target first — recommendation: **columnar** (grounded; only target avec needs).
2. SQL port scope — full 270K-line SPARQL stack vs subset (states a plan or it isn't one).
3. BLAKE3 single impl (KD-08): recommend purrdf-hash native.
4. ABI freeze expansion ticket: required before any query export work.
5. frozen-duckdb re-homing: required before any portable DuckDB receipt.
6. stardf_scan mechanism: in-process VTab (grounded) vs loadable dylib (unverified).
7. Columnar physical id encoding: Int64-widened (golden-stable) vs Int32 (golden regen).
