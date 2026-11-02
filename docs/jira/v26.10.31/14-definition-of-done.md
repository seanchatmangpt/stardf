# Definition of Done — stardf v26.11.1 (operator-issued 2026-10-31, spec of record)

> Status: AUTHORITATIVE. Supersedes 13-v26111-target-ledger.md and revises the
> ABI/columnar targets. Landed verbatim from the operator; the campaign reconciles
> to THIS document.
>
> **AMENDMENT (2026-10-31, 80/20 re-scope, OPERATOR-DECISIONS.md D15–D18):**
> Full parity is DEFERRED (12-full-parity-campaign.md deferred); parity is
> scoped to the four first-class targets (arrow landed; duckdb critical path;
> sql whole + 4 goldens; n3 whole + suite + discovery). The copied ~80%
> carries NO courts (D17: compile-green at best). ABI-v2 — this DoD's revised
> 13-export C-ABI table — is PENDING-OPERATOR (D18); arrow stays landed as-is
> (v1 13-symbol dylib). Until the W7-class doc lane reconciles the sections
> below, treat the PurRDF parity requirements and the C-ABI table as
> deferred/pending, not as v26.11.1 acceptance. Unaffected: columnar (landed
> with receipts), duckdb (first-class target), licensing (R1 admitted).
>
> **AMENDMENT 2 (2026-10-31, D13 resolved — finishers-wave lane
> 15-rehome-note):** the duckdb row's dependency re-homing pins move
> `=2.0.3` → `=2.0.4` (frozen-duckdb / frozen-duckdb-sys / frozen-duckdb-builder,
> operator-ratified). Naming divergence registered in the duckdb row: this
> DoD's verbatim text named `duckdb`/`duckdb-sys`/`libduckdb-sys`; the actual
> crate identities are the frozen-duckdb family. Why: the F04 rehome
> falsifier fired on `=2.0.3` cache-absent (see OPERATOR-DECISIONS.md D13
> resolution addendum).

## Substrate Architecture and Foundational Invariants

The stardf v26.11.1 release establishes a high-throughput, deterministic analytical runtime for semantic data, resolving the historical performance and structural impedance mismatch between graph traversal systems and relational columnar engines. Traditional graph database architectures suffer from a severe serialization and memory fragmentation penalty. Operating across process boundaries typically demands converting graph topologies into verbose text serializations such as N-Triples or JSON-LD, consuming up to 80% of execution cycles in string parsing and memory allocation. Furthermore, standard pointer-rich node-and-edge graph representations exhibit poor CPU cache locality, precluding single-instruction multiple-data (SIMD) vectorization and preventing concurrent analytical scans. When semantic stores attempt to interface with vectorized execution engines like DuckDB, they historically rely on fragile external views over disk-persisted files, forcing redundant disk round-trips and repeated deserialization. Compounding these efficiency bottlenecks is the issue of non-deterministic state: dynamic dictionary interning based on transient ingestion order alters internal integer identifiers across runs, breaking cryptographic reproducibility and verifiable audit logs across sovereign execution environments.

To eliminate this overhead, stardf treats RDF 1.2 semantic graphs—including triple terms, reifiers, annotations, directional literals, and named graphs—as native, byte-deterministic columnar projections. The runtime architecture coordinates three primary substrates: Apache Parquet for zero-dependency persistent storage, Apache Arrow for memory-mapped inter-process execution, and an embedded DuckDB virtual table function for in-process vectorized analytics.

The integrity of this execution plane depends upon three non-negotiable architectural invariants:

1. **Total-order value determinism.** Integer term identifiers are strictly decoupled from row arrival sequence or backend memory allocation order. Term identifiers derive from the canonical lexicographical sort order of unique terms within a balanced binary tree. Identical datasets under disparate thread schedules or on distinct CPU architectures yield identical integer assignments, identical Parquet byte streams, and bit-for-bit reproducible cryptographic digests.
2. **Zero-copy memory-mapped execution.** Cross-boundary data exchanges—delivering storage buffers into vectorized DuckDB execution pipelines or exporting query solutions across FFI—operate via direct memory aliasing without payload copying or intermediary allocations. Memory lifecycles across foreign boundaries maintain a verified zero-allocation profile.
3. **Monotone governed execution.** Relational multiset evaluations over unbounded graph structures execute under an eight-dimensional operational budget bound to admission control. The governing framework enforces strict termination precedence and returns deterministic evidence vectors; execution limits prevent memory exhaustion while preserving prefix-monotonic answer completeness.

## PurRDF Semantic Substrate Parity Requirements

### SPARQL 1.1/1.2 Algebra and Parsing

The parsing front-end maintains a clean leaf-crate boundary, compiling SPARQL query text into an owned query algebra without backend storage-trait dependencies. Grammar coverage: complete W3C SPARQL 1.1 + SPARQL 1.2 working drafts — SELECT/CONSTRUCT/ASK/DESCRIBE, BGPs, arbitrary property paths, ORDER BY/DISTINCT/REDUCED/LIMIT/OFFSET, grouping/aggregations, subquery decorrelation. RDF 1.2 primitives natively in the AST: quoted triple terms `<<(s p o)>>`, reification structures, directional language tags. Fail-closed: out-of-scope syntax → typed `ParseError::Unsupported`; no lenient fallback, no partial AST. The algebra crate (`stardf-sparql-algebra`) compiles standalone on foundational value-space crates (iri, xsd, cdt, lex) without core RDF storage components.

### Multiset Evaluator and TermId Allocation Mechanics

Multiset relational algebra over integer identifier spaces: interned terms derive ids from canonical `BTreeSet<TermValue>` sort; u32 operational space widening to i64 in columnar tables; minting capped at `u32::MAX - 2`; `0` = BOTTOM (absent); `u32::MAX` = WILDCARD (pattern-only); signed `-1` casts prohibited; scratch allocations segregated (ScratchId, bit-32 partitioning). Unbound states NEVER use raw integer sentinels — `Option<TermId>`/`GraphMatch`/`SolutionTerm` carriers only; raw masks confined to storage indexing. Native hash joins, merge joins, property-path automata (arbitrary length/inverse), grouping folds, statistical aggregates, correlated EXISTS/NOT EXISTS via memoized probes. SPARQL 1.1 Update (INSERT DATA, DELETE DATA, DELETE/INSERT WHERE, LOAD, CLEAR, CREATE, DROP, COPY, MOVE, ADD) in copy-on-write layers with atomic rollback.

### Monotone Resource Governors and Precedence Ledger

Eight-dimensional ResourceVector bound to admission control:

| Dimension | Accumulation | Caller-configurable | Purpose |
|---|---|---|---|
| fuel | Summation | Yes | Algebra node entries, row commitments, candidate quad exams, path frontier expansions, UDF/remote/update/property-fn/aggregate/probe events: 1 unit each; expression eval over a row: 1 + 1 per binary 64-bit limb in bignum towers |
| answer-rows | Summation | Yes | Solution rows emitted |
| intermediate-cells | Maximum | Yes | Largest intermediate bag (joins, folds) |
| scratch-bytes | Summation | Yes | Heap arenas, string scratch |
| remote-requests | Summation | Yes | Federated service calls |
| udf-depth | Maximum | No (fixed 32) | Recursion guard |
| pages | Summation | Via paging limits | Storage blocks read |
| bytes | Summation | Via paging limits | Raw payload bytes admitted |

Precedence (lowest rank wins): genuine `EvalError` > `Stopped{Cancelled}` > `Stopped{Deadline}` > `Refused` > `Budget`. Internal dimension resolution order: IntermediateCells, Fuel, AnswerRows, ScratchBytes, RemoteRequests, UdfDepth, Pages, Bytes. Truncated execution never masquerades as complete: `GovernedOutcome::BudgetExhausted` pairs the partial set with the exact 8-dim evidence vector + static analysis certificate confirming prefix-monotonicity.

### Results Serialization and Provenance Extension

Streaming, allocation-bounded W3C SPARQL Results JSON/XML/CSV/TSV. JSON/XML carry an additive provenance sidecar under a caller-supplied namespace prefix: execution digests, engine version, per-solution derivation graphs, governor evidence vector — without breaking W3C consumers.

## Parquet Storage Engine Specification

`stardf-columnar`: independent Parquet implementation, `#![forbid(unsafe_code)]`, zero third-party Parquet dependencies.

### Canonical Five-Table Schema

| Table | Columns (name: TYPE rep — semantics) |
|---|---|
| **terms** | id INT64 REQUIRED (dense 1-based); kind INT32 REQUIRED (0=IRI,1=Literal,2=Blank,3=Quoted Triple); val BYTE_ARRAY OPTIONAL (STRING, UTF-8/IRI); datatype INT64 OPTIONAL (→terms.id); lang BYTE_ARRAY OPTIONAL (STRING); dir INT32 OPTIONAL (0/1=LTR/2=RTL); reifier INT64 OPTIONAL; s/p/o INT64 OPTIONAL (quoted-triple slots, kind=3) |
| **quads** | s INT64 REQUIRED (IRI/Blank/Triple); p INT64 REQUIRED (IRI); o INT64 REQUIRED (unrestricted); g INT64 OPTIONAL (NULL = Default Graph; IRI/BNode only) |
| **reifies** | reifier/s/p/o INT64 REQUIRED; g INT64 OPTIONAL |
| **annot** | reifier INT64 REQUIRED; p INT64 REQUIRED (IRI); o INT64 REQUIRED; g INT64 OPTIONAL |
| **blobs** | digest FIXED_LEN_BYTE_ARRAY(32) REQUIRED (BLAKE3); media_type BYTE_ARRAY REQUIRED (STRING); size INT64 REQUIRED; payload BYTE_ARRAY REQUIRED |

### Byte-Level Determinism and Codec Constraints

Row sort orders: quads `(g,s,p,o)`; reifies `(reifier,s,p,o,g)`; annot `(reifier,p,o,g)`; blobs by digest. Parquet v1 (PAR1), single row group, one data page per column chunk, canonical compact Thrift footers without extension fields. Flat REQUIRED/OPTIONAL PLAIN for integer ids; dictionary page order from canonical term values (never hash-map iteration); byte arrays contiguous, no def-level overhead on REQUIRED. Compression: single-threaded Zstandard level 3 (`structured-zstd`). Metadata contract `stardf.columnar.schema-version = 1`; readers fail closed on foreign metadata keys.

### Fail-Closed Reading and Structured Error Taxonomy

Reader never panics, never allocates diagnostic strings on error paths. Codes: `InvalidMagic` (no PAR1), `DenseIdViolation` (non-monotonic/missing dense terms.id), `SchemaMismatch` (column/repetition/physical-type deviation), `CorruptFooter` (Thrift compact), `SortOrderViolation`, `DanglingReference` (quad/reifies/annot → unallocated terms.id), `DigestMismatch` (blobs BLAKE3 verify failure).

## Arrow Memory Model and CanonArrow Egress

### Frozen C-ABI Surface (GL-STARDF-001/002 Tier A freeze governance)

| Symbol | Params → Return | Failure | Responsibility |
|---|---|---|---|
| `stardf_arena_create` | void → StardfArena* | NULL | Thread-safe string interning arena |
| `stardf_arena_destroy` | StardfArena* → void | None | Reclaim arena |
| `stardf_arena_intern` | (Arena*, const char*, size_t) → uint32_t | 0=BOTTOM | Canonicalize + intern → stable TermId |
| `stardf_dataset_create` | Arena* → StardfDataset* | NULL | Empty quad dataset bound to arena |
| `stardf_dataset_destroy` | StardfDataset* → void | None | Dealloc dataset indexes |
| `stardf_dataset_append_quad` | (Dataset*, u32×4) → int32_t | non-zero | Insert (s,p,o,g) into index tables |
| `stardf_query_execute` | (Dataset*, const char*) → StardfStream* | NULL | Compile+execute SPARQL → analytical stream |
| `stardf_stream_next` | (Stream*, void**, size_t*) → int32_t | non-zero | Next CanonArrow batch payload, no allocation |
| `stardf_stream_destroy` | StardfStream* → void | None | Release cursor/scratch/locks |
| `stardf_export_arrow_stream` | (Dataset*, ArrowArrayStream*) → int32_t | non-zero | Arrow C Data Interface stream over dataset quads |
| `stardf_columnar_write_file` | (Dataset*, const char*) → int32_t | non-zero | Emit five Parquet files to a directory |
| `stardf_columnar_read_file` | (Dataset*, const char*) → int32_t | non-zero | Load+validate five Parquet files |
| `stardf_last_error_message` | void → const char* | NULL | Thread-local last-error diagnostics |

### Zero-Copy Arrow Streaming Protocol

ArrowArrayStream (get_schema/get_next/get_last_error/release). Stream batches reference memory-mapped pages or pre-allocated buffers directly; BEAM `enif_make_resource_binary` crossings guarantee a `{0,0}` heap reallocation delta. Completion invariants: get_next after exhaustion → null array without error, continuously; release is idempotent.

### CanonArrow Wire Framing

Immutable slab framing: batches of exactly 64 rows (`ROWS_PER_BATCH=64`); 128-byte fixed header (magic, batch length, schema metadata offset, payload length); offsets 68..100 = 32-byte BLAKE3 domain digest over the column data buffers (verify without array deserialization); ASK → dedicated 192-byte empty frame.

## In-Process DuckDB Integration Engine

`stardf_scan()`: native VTab registered with DuckDB's engine — no `CREATE VIEW ... read_parquet(...)` wrappers. VTab phases:

| Phase | Responsibility |
|---|---|
| bind | Validate args, verify dataset addresses, check target schema (terms/quads/reifies/annot/blobs), declare DuckDB types (BIGINT, VARCHAR, BLOB) |
| init_global | Dataset read lock, resolve dictionary segments, plan chunk partitions by operator concurrency |
| init_local | Thread-local cursors + scratch buffers at DuckDB vector capacity (2048-row chunks) |
| scan | Translate integer ids/string pointers into output vectors via zero-copy aliasing |
| pushdown_complex_filter | Intercept WHERE predicates before materialization |
| cardinality | Exact row counts from dataset metadata (cost-based optimizer) |

Projection pushdown bypasses unreferenced columns entirely; predicate evaluation over flat u32/i64 arrays via branchless SIMD masks before string materialization; string candidates resolved once against the sorted terms dictionary then joined in numeric id space. Dependency re-homing (AMENDED 2026-10-31, D13 resolved — operator ratified `=2.0.4` x3; supersedes the verbatim `=2.0.3` row): strict pins `frozen-duckdb = "=2.0.4"`, `frozen-duckdb-sys = "=2.0.4"`, `frozen-duckdb-builder = "=2.0.4"`. Naming divergence registered: the verbatim row named `duckdb`/`duckdb-sys`/`libduckdb-sys`; the actual crate identities are the frozen-duckdb family (no `libduckdb-sys` in the closure). Build must succeed with `$HOME/.cargo` and local target caches purged (clean-registry resolution); the F04 rehome falsifier fired on `=2.0.3` cache-absent, which is why the pins moved to `=2.0.4`.

## Comprehensive Verification Matrix (fail-closed acceptance)

| Subsystem | Court | Acceptance |
|---|---|---|
| stardf-sparql-algebra | algebra_conformance.rs (W3C parser manifest) | 100% parse pass on valid corpus; typed ParseError on invalid; zero core deps |
| stardf-sparql-eval | eval_multiset_courts.rs + property-path suites | 100% semantic parity vs W3C baseline; zero integer-collision panics |
| stardf-governors | governor_profile_court.rs, SPARQL-GOVERNOR-PROFILE | Exact fuel-count parity; precedence resolution; prefix-monotonicity hold |
| stardf-results | results_serialization_court.rs | Bit-identical output vs reference files; additive provenance valid |
| stardf-columnar | golden_bytes.rs, columnar_manifest.sha256 | 10/10 pinned digests match; lossless round-trip |
| stardf-columnar errors | structured_errors.rs (9 trials) | 9/9 pass; fail-closed; zero heap allocs on error paths |
| stardf-arrow C-ABI | abi_freeze_court.rs, nm -gU audit | Exactly 13 visible exports; zero leakage |
| CanonArrow | canon_arrow_wire.rs, domain_digest_court.rs | Offsets 68..100 verify digests; {0,0} boundary delta |
| stardf_scan() | just columnar-oracle, check-columnar-oracle.sh | Clean DuckDB execution; exact type+row counts, all 5 tables |
| avec boundary | avec-ffi/tests/stardf_consumer_court.rs | 3/3 courts; full lifecycle; zero resource leaks |
| License | verify_stardf_licenses.py | exit 0; zero unadmitted licenses |

## Arbiter Ladder

`cargo fmt --all -- --check` (zero diffs) → `cargo check --workspace --all-targets` (exit 0) → `cargo clippy --workspace --all-targets -- -D warnings` (clean) → `cargo test --workspace --locked --no-fail-fast -- --test-threads 1` (all suites complete). Chicago discipline throughout: real processes/memory/I-O, no doubles; a harness that cannot fail on corrupted input is legally void.

## Anti-Vacuity Falsifiers

F1 semantic type census (22 types + 126 functions; `--mutate drop-type` non-zero). F2 byte determinism (SplitMix64 seed 0x6012, 10 pinned digests; corrupt-one-byte positive control fails). F3 consumer boundary (invalid schema discriminant → loud typed failure). F4 header freeze (undeclared export / altered signature → abi_freeze_court.rs rejects). F5 external oracle (`check-columnar-oracle.sh`: DuckDB CLI row/type verification; absent → `[SKIP: reason]` + nonzero).

## Licensing, Provenance, Boundary Sovereignty

R1 turnstile (`verify_stardf_licenses.py`) + `authority/stardf/license-compat.json`; compound licenses (Unicode-3.0, BSD-3-Clause) retain verbatim notices in `authority/stardf/license-inventory.toml`; single cryptographic hash identity — dual blake3 versions fail the build; gate exit 0 before downstream merges.

## Synthesis

Four-tier pipeline: Analytical & Egress (stardf_scan + 13 frozen C-ABI exports) over Evaluation & Algebra (multiset joins, path automata, TermId mapping, COW updates, 8-dim governors) over Columnar Storage (5-table Parquet, Zstd-3, total-order determinism) over Foundational Types & Codecs (RDF 1.2 lexical, IRI, XSD). When the full verification matrix passes — green arbiter, non-vacuous mutations, DuckDB CLI oracle, zero-leak licensing — stardf v26.11.1 delivers an audited, zero-copy, bit-deterministic analytical execution plane for semantic knowledge graphs.

## Execution Note — matrix as-amended is the finish-campaign gate set

> Appended 2026-10-31, doc lane 14. Additive only — the DoD text above is the
> operator's, unmodified.

Read the Comprehensive Verification Matrix above AS AMENDED by the two blockquotes
(80/20 re-scope D15–D18; D13 duckdb re-home `=2.0.4`): as amended, it IS the
finish-campaign's actual gate set, not a forward-looking wish list.

1. **Four first-class targets — courts green.** arrow (landed as-is, v1 13-symbol
   dylib), duckdb (critical path, frozen-duckdb family pinned `=2.0.4` per
   Amendment 2), sql (whole + 4 goldens), n3 (whole + suite + discovery) ran their
   courts in the finish campaign and are green.
2. **Copied ~80% — no courts.** Per D17 the copied remainder carries compile-green
   at best; matrix rows naming full parity or the revised C-ABI surface are
   deferred/pending per the AMENDMENT blockquote and are NOT v26.11.1 acceptance.
3. **Publish matrix — BLOCKED on exactly two items:** the G8 scope decision and the
   operator publish go. No other blocker is outstanding.
