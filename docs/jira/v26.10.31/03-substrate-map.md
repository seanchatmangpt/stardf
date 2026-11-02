# Substrate Map — v26.10.31 (stardf reconstitution)

File:line surfaces from the 20-lane explore (`wf_2cd4ca49-806`) + 10-lane wave-2 explore
(`wf_91d14711-714`), both 2026-10-31. All citations re-read against disk 2026-10-31 after the
one-package landing (commit `6cbb5b77` "one-package port — purrdf -> stardf, 53 crates
straight-copied (c3)", branch `fix/v26.9.29-ci-gates`); superseded rows fixed inline.

## ~/avec (substrate, ALIVE — cargo test --workspace 0 failures, ~310 tests)

| Surface | Citation | Role in reconstitution |
|---|---|---|
| `TermId = u32`, `BOTTOM=0x0`, `WILDCARD=0xFFFF_FFFF` | `crates/avec-arrow/src/layout.rs:24-31` | Physical layer stardf targets |
| `Quad64` `repr(C, align(64))`, size/align compile asserts | `layout.rs:34-69` (`QUAD_SIZE/QUAD_ALIGN` :72-75) | 64-byte invariant R4 |
| `Batch::try_new` named-column batch (T1) | `batch.rs:170` | Named batch/table layer |
| Zero-copy foreign ingest (T2), `Misaligned`/`BufferTooShort` refusals | `foreign.rs:102-145` | Typed foreign-buffer views |
| ArrowArray/ArrowSchema structs, CAS-swap release | `c_abi.rs:25-53, 70-77, 104-154` | House release discipline to reuse |
| Nested parent→child release cascade | `c_abi_nested.rs:14-24, 63-91` | Multi-column export precedent |
| `MmuBuf` 16 KiB page arena, receipted 1,303 GiB/s | `mmu_arena.rs:1,18`; `PHASE_CHANGE_RECEIPTS.md:84` | Arena target for TermId interning |
| 64-byte buffer alloc (`Layout::from_size_align(bytes,64)`) | `buffer.rs:3-12` | Alignment law |
| Zero-alloc CountingAlloc + scoped tracking | `crates/audit/src/alloc_counter.rs:39-153` (dealloc NOT counted :65-67) | Court substrate; W3 hazard |
| alloc courts / zero-copy tests / membranes | `avec-ffi/tests/alloc_courts.rs:30-133`, `zero_copy.rs:153-198`, `integration/tests/membranes.rs` | Court idiom (SLOTS Mutex serialization) |
| Branchless CC=1 gate + manifest | `scripts/audit_branchless.sh`, `scripts/branchless_manifest.toml` | Kernel-ABI gate |
| Bench floors as data | `docs/BENCH_FLOORS.json` | Perf fences |
| avec C ABI (6 fns) | `avec-ffi/src/lib.rs:21-103` | Substrate-side membrane |
| T1/T2 consumer gates for purrdf adoption | `docs/CONSUMER_REQUIREMENTS.md` (purrdf pin `c0059bf5`) | Adoption acceptance |
| **ArrowArrayStream** | **LANDED 2026-10-31** — `stardf/arrow/crates/stardf-arrow-core/src/stream.rs` (core vtable) + `stardf-arrow-capi/src/stream_c.rs` (extern "C" adapter vtable, `stream_c.rs:50-56` get_schema/get_next/get_last_error/release) + `stardf_avec_export_stream`/`stardf_stream_free` (`stardf-arrow-capi/src/lib.rs:190/:207`, fn decls) | W3 landed; stream is part of the 13-export frozen ABI |

Known substrate defect (registered, not a pattern): two BLAKE3 implementations
(`avec-blake3` crate + duplicate in `avec-arrow/src/blake3.rs`) — stardf binds to one.

## ~/purrdf (predecessor, ALIVE — 126 exports, ABI 0.16.0)

| Surface | Citation | Role |
|---|---|---|
| Generated header (cbindgen via cargo-c 0.10.23 --locked) | `include/purrdf.h:3`; `Makefile:1012-1018` (`make capi-header`/`capi-check`); CI drift gate `ci.yaml:438-460` | Contract source; regeneration law |
| 126 exports by family | `purrdf.h:1413-4799`; impl map: shacl.rs 28, entail.rs 29, query.rs 18, … (handles minted only in `handles.rs:66-97`) | Census target |
| Status enum (append-only, frozen discriminants) | `purrdf.h:344-440`; pinned by `abi.rs:215,2965` | Error convention to mirror |
| Handle model (Box::into_raw, Send+Sync proofs) | `crates/rdf-capi/src/handles.rs:22-97` | Handle-first families |
| TermId = NonZeroU32, QuadRow == 16B assert | `crates/rdf-core/src/ir/term.rs:63`; `dataset.rs:143-154` | Divergence vs stardf u32-with-zero: contract already chose u32 sentinels (avec shape) — registered divergence |
| FrozenTerms arena (Box<[u8]> + HashTable + Blake3 content ids) | `crates/rdf-core/src/ir/frozen_terms.rs:17-93` | Interning design input |
| Golden-master ABI suite (59 tests) | `crates/rdf-capi/tests/abi.rs` (census test :2737; byte-frozen JSON-LD :1039) | W4 oracle |
| Prototype snapshot (canonicalization, hardcoded prefix :137) | `tests/abi_signatures.rs:44-364` + `.snapshot` (abi-version 0.16.0) | W4 header-freeze court source |
| Real-C-host smoke (driver) | `tests/c_smoke.rs:97-158` | W4 port template |
| GTS vectors (39 pairs, CBOR-seq, NOT freeze-policed `check-corpus-frozen.py:52-55`) | `vectors/*.gts`; consumers `crates/gts/tests/frozen_*.rs` | W4 byte-frozen oracles + own-manifest requirement |
| License MIT OR Apache-2.0 OR MulanPSL-2.0 | `Cargo.toml:70` | R1 skew vs avec P28-SIL |

## ~/ggen-legacy (authority)

| Surface | Citation | Standing |
|---|---|---|
| Contract TTL (4951 triples, 126 fn + 2 kernel individuals, 13 TypeDefinitions) | `authority/stardf/stardf_contract.ttl` (source digest `6ef9b4a0…`) | PARTIAL_ALIVE, untracked |
| Verifier (fn census + SHACL + two-directional type census + `--mutate drop-type` F1a control; `--header` flag :21; hardcoded default :16) | `scripts/verify_stardf_ontology.py` | exit 0 2026-10-31; type census LANDED (see as-built section) |
| Extractor (tagged-struct regex only, :83) | `scripts/extract_purrdf_ontology.py` | 11/11 opaque mapped; 11 anon structs missed |
| Marketplace manifest (15 packs, all exist) | `authority/stardf/marketplace_packs.json`; verifier exit 0 | ALIVE |
| rust-pack ARD/PRD (projectional pipeline spec; compiler receipt missing) | `authority/stardf/rust-pack/{ARD,PRD}.md` | PARTIAL_ALIVE |
| CI model jobs | `.github/workflows/ci.yml:111-198` (dsrust), `:200-312` (docs-pack); workflow-count gate `:27-38`, `expected-workflows.txt` 3 entries | Template |
| Receipt exemplar | `evidence/local-docs-verifier.json` (schema/subject/checks/standing/nonclaims) | House style |

## stardf/arrow (as-built, landed 2026-10-31)

All paths relative to `/Users/sac/ggen-legacy/stardf/arrow/`. Counts re-read from files this
session (2026-10-31); standing for the whole section: PARTIAL_ALIVE — tree TRACKED on branch
`fix/v26.9.29-ci-gates` (commit `6cbb5b77` names the one-package port), gates not re-run by
this doc lane this session.

### One-package crate inventory (one workspace, one root)

Every purrdf crate now lives under the single workspace root `crates/` under a `stardf-*` name
(or its registered equivalent). Disk census re-read 2026-10-31:

- purrdf (rev `0da0578306983da851a9073337d57e2e2c41ab3a`) ships 53 crates (re-counted:
  `ls /Users/sac/purrdf/crates | wc -l` = 53). Re-homed: 52 as full `stardf-*` crate dirs; the
  53rd, `jsonschema`, is carried as an EXCLUDED-LOUDLY fixture tree only — `crates/jsonschema/`
  holds `tests/metaschemas/` (17 verbatim JSON Schema meta-schemas) with NO `Cargo.toml`; sole
  consumer `stardf-testkit/src/jsonschema_metaschemas.rs:19` (`include_str!`); disposition
  receipted in `crates/jsonschema/DISPOSITION.md`.
- `crates/` holds 58 `stardf-*` crate dirs (each with `Cargo.toml`) + the `jsonschema` fixture
  dir + `PROVENANCE-entries.md` (per-crate git-archive sha256 receipts, 16 rows; its own header
  marks the snapshot IN-FLIGHT — entries carry as-of times and MUST be re-run).
- 47 unique workspace members in `stardf/arrow/Cargo.toml` (+ implicit root package
  `stardf-arrow`; `stardf-cdt`/`stardf-ed25519`/`stardf-sparql-algebra` each appear twice in the
  members list — comment block + alphabetical block, harmless).
- 11 manifest-bearing dirs are NOT members (excluded, each registered) — table below.
- First-party stardf crates (no purrdf origin): `stardf-arrow-core`, `stardf-arrow-capi`,
  `stardf-arrow-bench`, `stardf-scan-duckdb` (members) + `stardf-rdf-core-slice` (excluded).
- **Slice coexistence**: member `stardf-slice` (26.9.29 literal, publish=false) coexists with
  the FULL `stardf-rdf-core` copy (package `stardf-core`, publish=true, `=26.11.1`, publish
  tier T4); the earlier stardf-authored partial extraction `stardf-rdf-core-slice` is excluded
  (registered PARTIAL build standing, residual source closure escalated,
  `crates/stardf-rdf-core-slice/PROVENANCE.md:78`).

Full member list (47; origin = purrdf crate at rev `0da05783` unless noted):

| Member (`crates/…`) | purrdf origin | Notes |
|---|---|---|
| stardf-alloc-probe | alloc-probe | zero-dep leaf (T0) |
| stardf-arrow | arrow | package renamed `stardf-arrow-rs`; workspace key `purrdf-arrow` |
| stardf-arrow-bench | — (first-party) | promoted into members so the workspace ladder exercises it |
| stardf-arrow-capi | — (first-party) | frozen 13-export C ABI (below) |
| stardf-arrow-core | — (first-party) | arena/batch/layout/release/stream; avec-free layout mirror |
| stardf-branchless | branchless | T1 |
| stardf-cdt | cdt | T3 |
| stardf-columnar | columnar | vendored rev receipted; dual-key (`stardf-columnar` + `purrdf-columnar`) |
| stardf-conformance-kit | conformance-kit | path-only workspace key |
| stardf-datalog | datalog | 26.9.29 literal; outside the D17 publish closure (H03) |
| stardf-deflate | deflate | T1 |
| stardf-duckdb | duckdb | publish=false; umbrella's optional dep, NOT in closure (G8) |
| stardf-ed25519 | ed25519 | 26.9.29 literal; outside closure (H03) |
| stardf-entail | entail | |
| stardf-events | rdf-events | receipted vendor (sparql-algebra lane); `stardf-rdf-events` dir = excluded byte-identical dup |
| stardf-geo | geo | |
| stardf-geo-kernel | geo-kernel | path-only key |
| stardf-glob | glob | |
| stardf-gts | gts | |
| stardf-hash | hash | zero-dep vendored root (T0) |
| stardf-hnsw | hnsw | |
| stardf-iri | iri | T2 |
| stardf-json | json | |
| stardf-lex | lex | T1 |
| stardf-markdown | markdown | |
| stardf-mime | mime | |
| stardf-n3 | n3 | closure member (T7) |
| stardf-process | process | sole avec-arrow consumer (decouple below) |
| stardf-purrdf | purrdf | THE UMBRELLA: package `stardf-rs`, T8 apex; key `purrdf` dual-key |
| stardf-rdf | rdf | T6 |
| stardf-rdf-core | rdf-core | FULL COPY; package `stardf-core`, publish=true (T4); coexists with `stardf-slice` |
| stardf-retrieval | retrieval | |
| stardf-scan-duckdb | — (first-party, greenfield) | `stardf_scan()` DuckDB table fn; closure member (T2); frozen-duckdb `=2.0.4` feature-gated `vtab` |
| stardf-shapes | shapes | 26.9.29 literal; outside closure (H03) |
| stardf-shex | shex | |
| stardf-slice | slice | 26.9.29 literal, publish=false; coexists with the full `stardf-rdf-core` copy |
| stardf-sparql-algebra | sparql-algebra | T4 |
| stardf-sparql-conformance | sparql-conformance | path-only key |
| stardf-sparql-eval | sparql-eval | |
| stardf-sparql-results | sparql-results | T5 |
| stardf-sql | sql | closure member (T7) |
| stardf-stack | stack | T2 |
| stardf-testkit | testkit | lib name `purrdf_testkit` pinned |
| stardf-text | text | |
| stardf-toml | toml | T7 |
| stardf-validate | validate | |
| stardf-xsd | xsd | lib unit tests LIVE (failing set identical to upstream at the pinned rev) |

Excluded, manifest-bearing, NOT members (11 dirs; each exclusion registered):

| Excluded dir (`crates/…`) | Origin | Why excluded |
|---|---|---|
| stardf-bench | bench | tooling/binary crate — 80/20 bulk-copy exclusion set (root `Cargo.toml` comment) |
| stardf-cli | cli | same set |
| stardf-envelope-probe | envelope-probe | same set |
| stardf-hash-conformance | hash-conformance | same set |
| stardf-helper-census | helper-census | same set |
| stardf-hooks | hooks | upstream-broken at pinned rev — same 9 errors from `cargo check -p purrdf-hooks` at `0da05783` in `/Users/sac/purrdf`; sources byte-identical (diff exit 0); escalated, excluded loudly |
| stardf-rdf-capi | rdf-capi | bindings/tooling crate — same bulk-copy set |
| stardf-rdf-events | rdf-events | byte-identical duplicate of member `stardf-events` (src/ diff exit 0; zero dep keys reference it) |
| stardf-rdf-core-slice | — (first-party) | PARTIAL build standing — 24-error residual source closure (turtle.rs, dataset.rs, dataset_view.rs, cdt_blank, lex diagnostics), escalated per `PROVENANCE.md:78` |
| stardf-rdf-wasm | rdf-wasm | tooling crate — same bulk-copy set |
| stardf-wasm-link | wasm-link | tooling crate — same bulk-copy set |

### avec-arrow decouple (Lane 09, 2026-10-31)

The publish closure is avec-free by construction. Crates.io rejects git dependencies in any
published manifest — even optional, feature-gated ones — so:

- `stardf-arrow-core`: avec-arrow REMOVED from the published surface; `batch.rs` consumes the
  in-tree spec-fixed layout mirror `src/layout.rs` (size/alignment/field-offset const-asserted,
  mirrored from `avec/crates/avec-arrow/src/layout.rs`). Manifest record:
  `crates/stardf-arrow-core/Cargo.toml:21-28`.
- `stardf-arrow-capi`: avec-arrow REMOVED entirely — `cargo package -p stardf-arrow-capi`
  rejects even an OPTIONAL git dep ("dependency `avec-arrow` does not specify a version"); the
  release-tagged exports mint their i32 batch with stardf-owned primitives over the spec-fixed
  `stardf-arrow-core` struct types. `crates/stardf-arrow-capi/Cargo.toml:28-37`.
- `stardf-scan-duckdb`: avec-arrow NOT consumed (manifest note, `Cargo.toml:11-12`).
- The single workspace pin `avec-arrow = { git = "https://github.com/seanchatmangpt/avec",
  rev = "67c0ec1" }` (root `Cargo.toml:263`) is consumed by exactly ONE member: `stardf-process`
  (`avec-arrow.workspace = true`, `crates/stardf-process/Cargo.toml:20`; publish=false,
  internal).

### Publish closure (25 packages, T0–T8)

`scripts/publish_stardf.py:87` defines strict topological tiers over the 25-package closure
(re-verified against live cargo metadata at run time — normal+dev+build edges, intra-closure
only):

- T0 `stardf-hash` `stardf-alloc-probe` `stardf-arrow-core` · T1 `stardf-lex` `stardf-deflate`
  `stardf-branchless` `stardf-testkit` `stardf-events` `stardf-arrow-bench` `stardf-columnar` ·
  T2 `stardf-stack` `stardf-iri` `stardf-arrow-rs` `stardf-xsd` `stardf-arrow-capi`
  `stardf-scan-duckdb` · T3 `stardf-cdt` · T4 `stardf-core` (`crates/stardf-rdf-core`)
  `stardf-sparql-algebra` · T5 `stardf-sparql-results` · T6 `stardf-rdf` · T7 `stardf-sql`
  `stardf-n3` `stardf-toml` · T8 `stardf-rs` (`crates/stardf-purrdf`, umbrella).
- Dual-keying: 27 workspace keys → 25 packages at `=26.11.1` (columnar and the umbrella carry
  two keys each — upstream's own `purrdf`/`purrdf-rs` dual-key alias pattern).
- Non-closure members stay out: `stardf-shapes`/`stardf-datalog`/`stardf-slice`/
  `stardf-ed25519` at 26.9.29 literals (H03); `stardf-conformance-kit`/`stardf-geo-kernel`/
  `stardf-sparql-conformance` path-only.

### Frozen C ABI — 13 exports

`crates/stardf-arrow-capi/include/stardf_arrow.h` (frozen header) matched 1:1 by 13
`#[unsafe(no_mangle)]` `pub extern "C"` exports (edition-2024 attribute spelling) in
`crates/stardf-arrow-capi/src/lib.rs`:

| Export | lib.rs line (fn decl) |
|---|---|
| `stardf_last_error` | :56 |
| `stardf_arrow_version` | :68 |
| `stardf_arena_new` / `stardf_arena_intern` / `stardf_arena_free` | :78/:96/:123 |
| `stardf_batch_append_quad` | :159 |
| `stardf_avec_export_stream` | :190 |
| `stardf_stream_free` | :207 |
| `stardf_i32_batch` | :268 |
| `stardf_array_addr` / `stardf_batch_rows` / `stardf_pending_tags` | :385/:391/:397 |
| `stardf_release` (returns `ReleaseOutcome`) | :411 |

Three-way parity is enforced, not asserted: `scripts/verify.py` (1) rdflib census of
`ontology.ttl`, (2) header↔impl `#[no_mangle]` parity both directions, (3)
ontology↔header `stardf-arrow:targetSymbol` cross-check both directions.

### stardf-arrow-core modules

`crates/stardf-arrow-core/src/{arena,batch,layout,release,stream}.rs`
(`lib.rs:6-10`): arena (TermId interning), named batch, Quad64/layout law, CAS-swap
release discipline, ArrowArrayStream core vtable. `stardf-arrow-capi/src/{stream_c,term,ownership}.rs`
adapt the core vtable to `extern "C"` (`stream_c.rs:93-177`) and carry house
term/ownership membranes.

### stardf-columnar (vendored)

Vendored from purrdf `crates/columnar` at rev `0da0578306983da851a9073337d57e2e2c41ab3a`
(verified against `/Users/sac/purrdf` HEAD 2026-10-31) — `crates/stardf-columnar/PROVENANCE.md:5-6`.
The u32 seam is `src/vendor/core/dataset.rs:39-40` (`trait ViewTermId` + blanket impl) with the
projection binding `type Id = u32` at `src/vendor/core/dataset.rs:163` — matching avec/stardf
u32-with-zero, vs purrdf's NonZeroU32 (the registered divergence at :37 above stands). Codec:
fixed **five-table** schema projection (`src/lib.rs:7`, `src/files.rs:12,25,45`) over
`crate::vendor::core::ContentStore`.

### Courts inventory (as-built)

| Court file | `#[test]` count | Covers |
|---|---|---|
| `crates/stardf-arrow-capi/tests/stream_courts.rs` | 6 | stream lifecycle / honest release tags |
| `crates/stardf-arrow-capi/tests/cross_allocator_court.rs` | 3 | untagged foreign release; `--features mutant-cross-alloc` flip (:20) proves the court detects, not just passes |
| `crates/stardf-arrow-capi/tests/alloc_identity.rs` | 2 | alloc identity across the membrane |
| `crates/stardf-arrow-capi/tests/c_smoke.rs` | 1 | real C-host smoke |
| `crates/stardf-arrow-capi/tests/columnar_integration.rs` | 1 | capi→columnar integration |
| `crates/stardf-columnar/tests/golden_bytes.rs` | 6 | 5 tables × 2 pinned digests + read-back/twice/flip (:278-323) |
| `crates/stardf-columnar/tests/seam_courts.rs` | 4 | vendored seam behavior |
| `tests/anti_vacuity.rs` | 3 | header/ontology mutation fixtures (`tests/fixtures/header_{drifted,pristine}.h`, `ontology_{missing_export,pristine}.ttl`) — the gate must refuse |

### Verifier scripts (as-built)

| Script | What it gates |
|---|---|
| `stardf/arrow/scripts/verify.py` | three-way parity: ontology census, header↔impl, ontology↔header (docstring :5-12; header default :37) |
| `stardf/arrow/scripts/verify_header_freeze.py` | F4 header-freeze court: three-surface symbol-set equality — ontology `CabiExport` targetSymbols vs committed `snapshots/prototypes.snapshot` vs header fn decls; drift → GONE/NEW diff + exit 1 (docstring :4-16) |
| `stardf/arrow/scripts/verify_stardf_licenses.py` | P28-SIL→MIT/Apache/MulanPSL dependency-edge gate backed by `authority/stardf/license-compat.json`; blake3-red per 04-falsifiers R1 — IN-TREE at `stardf/arrow/scripts/` (the repo-root staging row is superseded) |
| `/Users/sac/ggen-legacy/scripts/verify_stardf_ontology.py` | type census (opaque + anon structs, two-directional, :31-55) + `--mutate drop-type` F1a positive control (:58-60, :123-125) — supersedes the "type census MISSING" row below |
| `stardf/arrow/scripts/verify_blake3_identity.sh` | KD-08 single-identity gate; blake3 ABSENT → exit 1 (vacuity law), per `stardf/docs/jira/v26.10.31/11-blake3-arbitration.md` §3 |
| `stardf/arrow/scripts/verify_duckdb_rehome.sh` | fail-closed frozen-duckdb 2.0.3 crates.io rehome court (W1 lane 9), /tmp sandbox, D13 acceptance |

Note: `verify_stardf_ontology.py` is staged at repo root `scripts/` (alongside
`extract_stardf_arrow.py`, `extract_stardf_columnar.py`), not at `stardf/arrow/scripts/` —
path divergence registered. `publish_stardf.py` (crates.io publish ladder, 25-package
closure) and `lane11_tarball_gate.sh` are also in-tree at `stardf/arrow/scripts/`.

## ~/ggen-marketplace (generation capital)

`rust-pack` (templates lib_rs/c_abi `.tmpl` with inlined SPARQL front-matter, 5 gates,
witnesses) = the W2 route. CNV pack rejected (CLI-shaped), wasi-json-abi-pack DEPRECATED.

## ~/bcinr — registered-non-consumer

Zero purrdf/stardf coupling; RDF out of scope (`bcinr/AGENTS.md:351`). No work order.
`stardf-kernel`: NOT FOUND at depth ≤2 under /Users/sac (UNKNOWN beyond).
