# COLUMNAR-ARD.md

# Columnar ARD — stardf/arrow columnar surface (lane 16, phase 2)

Architecture decision record for the stardf/arrow columnar surface, reconstituted
from predecessor `purrdf-columnar`. Provenance: source purrdf rev
`0da0578306983da851a9073337d57e2e2c41ab3a` (witnessed this session:
`git rev-parse HEAD` in `/Users/sac/purrdf` matches the pin in
`stardf/arrow/evidence/manufacturing-receipt-2026-10-31.json:28` and
`stardf/docs/jira/v26.10.31/02-wave-contracts.md:79`).

Standing: **PARTIAL_ALIVE** (updated 2026-10-31, lane 8 truth pass #3). The
`stardf-columnar` crate now exists on disk (`crates/stardf-columnar/`, vendored
from purrdf rev 0da05783 per `crates/stardf-columnar/PROVENANCE` refs in its
Cargo.toml comment) with the five-table schema, `#![forbid(unsafe_code)]`
(`src/lib.rs:21` + `[lints.rust] unsafe_code = "forbid"` in Cargo.toml), typed
per-surface error taxonomy (`src/error.rs` + `src/error/`), and the byte-freeze
courts. Command receipts (this lane, this tree):

- `cargo test -p stardf-columnar --test golden_bytes` → exit 0, 6 passed /
  0 failed (5 counted + regenerate, ignored in the ladder run) — byte-freeze,
  round-trip identity, write-twice determinism, byte-flip positive control,
  manifest mirror.
- `cargo test -p stardf-arrow-capi --test columnar_integration` → exit 0,
  1 passed (`w_columnar_arena_roundtrip_is_lossless_and_row_counts_cross_check`).
- `cargo test -p stardf-columnar` → exit 0 across all targets (3 lib + 6
  golden_bytes + 4 seam_courts).
- `bash scripts/verify_blake3_identity.sh` → exit 0: "exactly 1 blake3
  identity: blake3 v1.8.7" (WARN: Cargo.toml declares 1.8.5, lock resolves
  1.8.7 — silent-drift class, unpinned).
- `bash scripts/check-columnar-oracle.sh` → exit 0: DuckDB reads the stardf
  columnar fixture (counts=6,3,0,0,2).
- `cargo test --workspace --no-fail-fast` → exit 0, fully green including
  anti_vacuity 3/3 (prior fixture drift repaired by its owning lane).

The full workspace is green; the W4 purrdf-vs-stardf differential digest-equality
court and the structured-error court corpus port remain OPEN (KD-10/KD-13).
Design-decision sections below keep their purrdf file:line citations; as-built
status now cites this tree.

## 1. The closed five-table schema (carried verbatim)

Source of truth: `purrdf/crates/columnar/src/schema.rs` (rev 0da05783).
`Table::ALL` (`schema.rs:92-98`) fixes the closed set in canonical dependency
order — Terms, Quads, Reifiers, Annotations, Blobs — each with a stable logical
name, deterministic Parquet filename (`terms.parquet`, `quads.parquet`,
`reifiers.parquet`, `annotations.parquet`, `blobs.parquet`; `schema.rs:163-191`),
and a `&'static [ColumnSchema]` column list in schema order (`schema.rs:66-73`).
Column surface is the closed physical-type set {Int64, ByteArray} ×
{Required, Optional} with a UTF8 annotation on lex/lang/digest BYTE_ARRAY columns
(`schema.rs:8-13,22-29,122-161`). The 11-column Terms dictionary (id, kind, lex,
datatype, lang, direction, scope, triple_s/p/o, named_graph), 4-column Quads
(s/p/o required, g optional), Reifiers, Annotations and Blobs schemas are carried
column-for-column, order included (`schema.rs:122-161`).

Close condition (schema parity): a schema-parity court asserting the stardf
`TableSchema` constants are column-for-column identical to
`purrdf/crates/columnar/src/schema.rs:122-161` at rev 0da05783 — names, physical
types, repetition, utf8 flags — plus the two upstream tests
(`schema.rs:214-247`: unique names/files, closed physical-type set) ported as
stardf tests.

## 2. Invariants carried

- **Byte-determinism law** (`06-phase0-columnar-sql-duckdb.md` §3): total-order
  dictionary assignment from `BTreeSet<TermValue>` (never ingest order) — witness
  `writer.rs:330-336` (`Dictionary::build` drains a `BTreeSet`); tuple-set row
  ordering; digest-ordered blobs; the exact PLAIN/RLE/DataPageV2 recipe; Zstd
  level-3 single-frame pinned `structured-zstd 0.0.49`. Determinism survives the
  u32 adaptation (§3 below). The determinism witness test exists upstream:
  `writer.rs:698` `writer_projects_all_rdf_layers_and_is_deterministic`.
- **`#![forbid(unsafe_code)]`** — the purrdf-columnar crate-level attribute
  (`purrdf/crates/columnar/src/lib.rs:21`). Carried as a hard gate on the stardf
  columnar crate; the FFI boundary stays in the existing capi crate, which is the
  only place unsafe may ever appear.
- **Fail-closed reader** — `reader.rs` is "Strict five-table columnar-to-RDF
  reconstruction" (`reader.rs:4`): it validates the physical file profile,
  enforces canonical dictionary and row order, rejects dangling/contradictory
  references, verifies every blob digest, and runs reconstruction through the
  kernel structural validator (`reader.rs:37-44`); the read entry returns typed
  `ColumnarError` for each class (`reader.rs:47-51`).
- **Error taxonomy** — typed `ColumnarError` (`#[non_exhaustive]`, fail-closed per
  its own doc `error.rs:25-26`) with per-surface reason enums:
  `ReaderReason`/`ProjectionReason`/`ConstructionReason`/`MetadataReason`/
  `CompactReason`/`ValueBodyReason`/`PageBodyReason` (`error.rs:5-22`),
  `StructuredError` derives (`error.rs:126,303`), and 9 structured-error courts
  (upstream `tests/structured_errors.rs`, 41 test fns; 06 §3 "9 structured-error
  courts"). Carried: the stardf port keeps the reason-enum-per-surface taxonomy
  shape; the stardf `TermId`-typed refusals (KD-02) extend it.

## 2.1 Invariants explicitly NOT carried unchanged (registered divergences)

- **Digest identity** — see §5 / KD-12.
- **Physical id width** — see §4 / KD-10.
- **Golden-digest storage medium** — purrdf pins digests in Rust source, not a
  `.sha256` manifest; stardf must add the manifest (F4b, `04-falsifiers.md:89-94`;
  KD-14).

## 3. u32 dictionary adaptation (per P0-5 / 06 §2)

Columnar consumes quads only via the generic `DatasetView`/`ViewTermId` trait;
term identity is re-minted as local dictionary indices (`writer.rs:315-376`;
06 §2). The stardf adaptation:

- **Sentinels by layer** (KD-02 convention): `BOTTOM_TERM_ID = 0` is the
  deterministic zero/unbound, `WILDCARD_TERM_ID = 0xFFFF_FFFF` is the
  pattern-side sentinel only (`layout.rs:23-29`); the interner mints ids >= 1 and
  stores `id + 1` internally (`arena.rs:40-53`), and `from_raw(u32::MAX)` is a
  typed capacity refusal (`arena.rs:59-70`).
- **Dictionary cap = `u32::MAX - 2`** (P0-5/06 §2: "dictionary cap becomes
  `u32::MAX - 2`"), replacing purrdf's i64 space; conversion from source ids is
  checked, fail-closed.
- **Evaluator never uses integer sentinels** — unbound is `Option<TermId>`/
  `GraphMatch`/`SolutionTerm` enums; raw words appear only inside `QuadKey` masks
  (06 §2, citing purrdf `dataset.rs:722,745`). The wildcard only exists on the
  pattern layer; a `WILDCARD` in a data row is a typed refusal (KD-02 close
  condition (b)).
- **Option\<i64\> graph slot collapses to u32 with BOTTOM = absent** (06 §2, D3);
  nullability becomes a sentinel convention, not a niche (KD-06).

Close conditions: the KD-02 truth-table court plus a capacity court asserting the
`u32::MAX - 2` cap is enforced with a typed refusal (a `Dictionary::build`
over-cap ingest must fail closed, never wrap).

## 4. Int64-widened physical encoding — decision + PENDING-OPERATOR Int32 row

Decision (06 §10 item 7, first half): **Int64-widened** — stardf u32 dictionary
ids serialize as PLAIN INT64 columns, exactly as purrdf serializes i64; the u32
words are widened at write, narrowed with a checked conversion at read
(fail-closed on out-of-range words). Rationale (06 §3): "Determinism survives
u32 — ids serialize as PLAIN INT64"; golden digests stay stable, no golden
regeneration.

Registered operator row (06 §10 item 7 second half; `07-port-master-plan.md`
OPEN-7): **Int32 encoding with golden regen — PENDING-OPERATOR**, to be decided
by the W4 digest-equality court (07 OPEN-7: "PENDING-W4 — decided by the W4
digest-equality court"). Until that court runs, Int64-widened is the only
admitted encoding and Int32 is an unadmitted candidate, not an alternative in
force.

Close condition: the W4 digest-equality court (`04-falsifiers.md` F-court class;
06 §8.4 "purrdf-vs-stardf digest-equality court") — a purrdf-columnar golden
write and a stardf Int64-widened write of the same splitmix64-seeded fixture
(`state=0x6012`, 06 §3) must be byte-identical; the Int32 candidate is admitted
only if it survives the same court after golden regeneration with a recorded
reason (06 §3).

## 5. Digest identity: BLAKE3 crate vs purrdf-hash (KD-12)

Registered arbitration (06 §7, P0-18): "**KD-08 BLAKE3 arbitration first**:
purrdf-hash native vs stardf's blake3 crate — pulling both violates the
single-impl decision; **recommend purrdf-hash** (zero-dep/no_std parity) +
`cargo tree` full single-identity gate + digest-parity court." stardf/arrow
currently binds one blake3 crate (`arena.rs:318`, per KD-08's row). The
columnar surface follows the recommendation: digest identity is **purrdf-hash
native**, because the columnar byte-freeze is golden-digest-anchored — columnar
blobs are content-addressed and the reader verifies every blob digest
(reader.rs:40-44), so a digest-identity divergence is a byte-freeze divergence.
The divergence from purrdf-columnar is registered, not resolved: until the
digest-parity court and the single-identity gate exist, the columnar digest
surface is loudly-MISSING, and the arbiter row stays PENDING-OPERATOR.

Close condition: (a) `cargo tree -e normal` single-blake3-identity gate over the
stardf workspace (exit 1 on >1 blake3 identity); (b) digest-parity court hashing
the same bytes through the stardf-chosen impl and purrdf-hash asserting
identical digests; (c) a columnar golden-bytes court pinning whole-tree Parquet
byte output against purrdf-columnar golden digests at rev 0da05783.

## 6. Provenance

- Source: `/Users/sac/purrdf` at rev `0da0578306983da851a9073337d57e2e2c41ab3a`
  (HEAD witness, 2026-10-31, this session).
- Planning receipts: `stardf/docs/jira/v26.10.31/06-phase0-columnar-sql-duckdb.md`
  (P0-2/5/6/9/18, §2/§3/§7/§10), `07-port-master-plan.md` (OPEN-7), `04-falsifiers.md` (F4b/F5).
- stardf/arrow in-tree anchors: `layout.rs:23-29`, `arena.rs:40-70,318`,
  DIVERGENCES.md KD-02/KD-06/KD-08 rows.
- Receipt discipline: per `docs/RECEIPTS.md` LANE-9 row format;
  `evidence/RECEIPT_TEMPLATE.json`.
