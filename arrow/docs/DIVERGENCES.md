# DIVERGENCES.md

# Divergences — stardf/arrow vs purrdf (predecessor) and avec (substrate)

Registered-divergence ledger for `stardf/arrow` against predecessor `~/purrdf`
and substrate `~/avec`. KD-nn row format: surface, both sides' behavior with
file:line citations, and a close condition — the court/test/gate that would
flip the row to pass. Per the SDK-parity register law, every surface is
pass / registered-divergence / loudly-MISSING, never a silent skip.

Sources: `stardf/docs/jira/v26.10.31/03-substrate-map.md` (20-lane explore
`wf_2cd4ca49-806` + wave-2 `wf_91d14711-714`, 2026-10-31); purrdf/avec/stardf
file:line citations re-read this session (LANE 20, 2026-10-31).

Standing: rows are registered divergences, not defects. PLANNED marks
surfaces not yet in `stardf/arrow/crates/`. Per-row status re-read 2026-10-31
(completion lane 11, truth pass #2; gates re-run on the final tree):

| Row | Status 2026-10-31 (lane 11) | Basis |
|---|---|---|
| KD-01 | OPEN (divergence registered, code exists, no differential court yet) | `layout.rs:23-33`/`arena.rs:40-53` match the row's description (lane-11 spot-check); no purrdf-vs-stardf interner court exists in the tree |
| KD-02 | PARTIALLY CLOSED (capacity + truth-table halves green) — `arena::tests::bottom_reserved_and_wildcard_refused` ok in the lane-11 `cargo test -p stardf-arrow-core` run (33 ok, exit 0); capacity refusal typed; **the compile-break note below is STALE — `arena.rs` compiles** | same test run, exit 0 |
| KD-03 | OPEN (pattern layer implemented, tests ok; no differential truth-table court vs purrdf `QuadKey`) | core lib tests incl. `quad_level_pattern_match` ok (33 ok, exit 0); no purrdf differential |
| KD-04 | OPEN (stardf-side layout court exists and passes: `field_offsets_match_spec_table`, `four_quads_are_exactly_256_bytes`; purrdf round-trip differential court does not exist) | core lib test run, exit 0 |
| KD-05 | OPEN (FFI 4-fn-pointer stream exists and is green: `stream_courts` 6/6, exit 0, lane-11 run; court_5 passes via the typed dead state in `stream_c.rs`; the foreign-C-host byte-compare vs purrdf export court does not exist) | lane-11 stream_courts run, exit 0 |
| KD-06 | OPEN (sentinel convention implemented; no GraphMatch-trio court, no `Option<TermId>`-in-repr(C) CI gate) | no such gate in `ci/` or `scripts/` |
| KD-07 | PARTIALLY CLOSED — `scripts/branchless_manifest.toml` exists; no `audit_branchless.sh`-equivalent gate is wired into `ci/` | `scripts/` + `ci/` contents |
| KD-08 | OPEN (single blake3 binding exists at `arena.rs:318`; no cargo-tree single-identity gate, no digest-parity court) | no gate in tree |
| KD-09 | OPEN (`ArenaError::BudgetExceeded` exists; no CountingAlloc-driven alloc court in this tree) | core lib tests cover budget refusal, not the alloc court |
| KD-10 | PARTIALLY CLOSED (Int64-widened implemented `writer.rs:402-405,606`; local golden-bytes court 6/6 exit 0; W4 purrdf-vs-stardf differential digest-equality court NOT run — differential half OPEN) | lane-8 golden_bytes run, exit 0 |
| KD-11 | PARTIALLY CLOSED (columnar crate + sentinel seam courts green: `seam_courts` 4/4 exit 0 incl `dictionary_cap_refuses_typed`; KD-02 truth-table reuse at the columnar boundary + purrdf differential not run) | lane-8 seam_courts run, exit 0 |
| KD-12 | OPEN (recommendation NOT implemented — columnar Cargo.toml binds blake3 1.8.5; single-identity gate EXISTS and passes: `verify_blake3_identity.sh` → 0, "exactly 1 blake3 identity: blake3 v1.8.7", WARN version skew 1.8.5 vs lock 1.8.7; digest-parity court vs purrdf-hash not run) | lane-8 gate run, exit 0 |
| KD-13 | PARTIALLY CLOSED (taxonomy ported: `src/error.rs` + `src/error/{reader,projection,construction,metadata,compact,value_body,page_body}.rs` + `vendor/hash.rs` StructuredError stand-in; structured-error court corpus port PENDING — `tests/disabled/structured_errors.rs-pending-stardf-substrate`) | crate tree on disk |
| KD-14 | PARTIALLY CLOSED (`.sha256` manifest EXISTS: `scripts/columnar_manifest.sha256`, mirrored by `manifest_mirrors_the_pinned_digests` + byte-flip positive control `flipping_a_byte_moves_the_digest`, 6/6 exit 0; F4b corrupted-copy CI gate + purrdf digest-ordering parity not established) | lane-8 golden_bytes run, exit 0 |
| KD-15 | PARTIALLY CLOSED (`#![forbid(unsafe_code)]` carried at `lib.rs:21` + `[lints.rust]`; determinism court `writing_twice_is_byte_identical` green locally; F5 two-run CI replay NOT run; branchless gate: `scripts/branchless_manifest.toml:44` — `audit_branchless.sh` TO BE LANDED) | lane-8 runs + manifest :44 |

## KD-01 — TermId offset convention: +1 / 0=none vs purrdf NonZeroU32 niche

- **Surface**: `TermId` numeric convention.
- **purrdf**: `TermId(NonZeroU32)` — dense index stored **+1** internally;
  `NonZeroU32` buys the `Option<TermId>` niche. Hand-written `Hash` hashes
  `get() - 1` so hash-iteration order is unchanged.
  `purrdf/crates/rdf-core/src/ir/term.rs:63,68-72`.
- **stardf/arrow**: `TermId = u32` with `BOTTOM_TERM_ID = 0` as the
  deterministic zero/unbound; the interner stores `id + 1` internally and only
  mints ids >= 1. `stardf/arrow/crates/stardf-arrow-core/src/layout.rs:23-29`;
  `arena.rs:40-53`.
- **avec**: same convention — `TermId = u32`, `BOTTOM = 0x0`,
  `WILDCARD = 0xFFFF_FFFF`. `avec/crates/avec-arrow/src/layout.rs:24-31`.
- **Close condition**: differential court running the same insert/lookup
  sequence through purrdf's interner and stardf's `TermArena`, asserting
  identical minted-id sequences and identical hash-iteration order (stardf
  must hash the dense index with the offset stripped, mirroring
  `term.rs:68-72`).

## KD-02 — u32::MAX: typed refusal vs WILDCARD sentinel

- **Surface**: meaning of the top of the id space (`0xFFFF_FFFF`).
- **purrdf**: `u32::MAX` unrepresentable (`NonZeroU32`); overflow is a typed
  capacity-error path, no sentinel. stardf's doc names this "purrdf parity"
  (`arena.rs:55-57`).
- **avec**: `0xFFFF_FFFF` is the `WILDCARD` match-anything sentinel;
  `BOTTOM` matches only itself, `WILDCARD` matches any term including
  `BOTTOM` (pattern contract, spec §1.1). `avec layout.rs:24-31` + doc
  `stardf layout.rs:20-22`.
- **stardf/arrow**: both, by layer — capacity refusal on `from_raw(u32::MAX)`
  (`arena.rs:59-70`, typed `ArenaError::Capacity`) **and** `WILDCARD_TERM_ID`
  as the pattern-side sentinel (`layout.rs:29`).
- **Close condition**: (a) truth-table court over
  {BOTTOM/BOTTOM, BOTTOM/WILDCARD, term/WILDCARD, term/BOTTOM} matching;
  (b) capacity court asserting `from_raw(u32::MAX)` →
  `Err(ArenaError::Capacity)`.

  **Status update 2026-10-31 (lane 13)**: the previously-reported compile
  break (missing paren on `Err(ArenaError::Capacity { … })`) is FIXED —
  `arena.rs` compiles and `cargo test -p stardf-arrow-core --lib` runs
  `arena::tests::bottom_reserved_and_wildcard_refused` ok (33 ok, one
  unrelated harness SIGTRAP). Row remains open on the differential-vs-purrdf
  courts, which do not exist.

## KD-03 — QuadProbePlan 0=unbound inverse semantics

- **Surface**: pattern-plan word/mask encoding.
- **purrdf**: `QuadKey { key: [u32;4], mask: [u32;4] }` — a bound field keys
  its id under an all-ones mask; a wildcard keys **0 under a zero mask**
  (0 = unbound). Match predicate `(word ^ key) & mask == 0`, branchless.
  `purrdf/crates/rdf-core/src/ir/dataset.rs:705-731` (struct :716-718,
  `QuadKey::new` :720-731).
- **stardf/avec**: inverse framing — with `BOTTOM = 0` a literal unbound slot
  in a data row is 0, and `WILDCARD` (u32::MAX) is the pattern-side wildcard;
  key=0/mask=0 (unbound) vs key=BOTTOM/mask=all-ones (bound to BOTTOM) must
  be distinguished. PLANNED for the pattern layer (stream spec at
  `stardf/docs/jira/v26.10.31/02-wave-contracts.md:50`; probe-plan kernel is
  a later wave).
- **Close condition**: branchless-kernel court (avec idiom:
  `avec/scripts/audit_branchless.sh` + `branchless_manifest.toml` coverage)
  running the purrdf `QuadKey` truth table against stardf's plan encoder —
  every purrdf `(key, mask)` pair must yield the identical match set, and the
  encoder fails closed (typed refusal) on `WILDCARD` in a data row.

## KD-04 — QuadRow 16B AoS vs Quad64 64B extended row

- **Surface**: quad row physical layout.
- **purrdf**: `QuadRow` = 16 bytes AoS, packed via the `Option<TermId>` niche
  (3×4B ids + 4B niche-packed optional graph); compile assert
  `size_of::<QuadRow>() == 16`. `purrdf/crates/rdf-core/src/ir/dataset.rs:143-154`
  (assert :154; P3a comment names the ~20% table win).
- **stardf/avec**: `Quad64` = 64B `repr(C, align(64))` with extension fields
  `t_ns: u64` (0x10), `vector_clock: u32` (0x18), `policy_mask: u32` (0x1C),
  `inline_digest: [u8;32]` (0x20). avec:
  `avec/crates/avec-arrow/src/layout.rs:34-75` (QUAD_SIZE/QUAD_ALIGN asserts
  :72-75); stardf/arrow:
  `stardf/arrow/crates/stardf-arrow-core/src/layout.rs:35-64` (size/align
  asserts :58-59), offset asserts `stream.rs:72-81`.
- **Close condition**: layout court asserting `size_of::<Quad64>() == 64`,
  `align_of == 64`, and every `offset_of!` (`layout.rs:60-64`,
  `stream.rs:74-81`); plus a differential court that a purrdf `QuadRow` and a
  stardf `Quad64` round-trip the same logical quad set (16B→64B lift,
  64B→16B projection, extension fields dropped deterministically).

## KD-05 — CanonArrow 128B header not reused; Arrow C Data Interface instead

- **Surface**: stream/export container format.
- **purrdf**: CanonArrow (128B canonical header) is the predecessor container
  concept; not found in purrdf sources re-read this session, so the
  divergence is registered at the contract level (no 128B-header surface in
  the purrdf citation set of `03-substrate-map.md`).
- **stardf/arrow**: exports over the Arrow C Data Interface stream spec (4 fn
  pointers: get_schema/get_next/get_last_error/release) per
  `stardf/docs/jira/v26.10.31/02-wave-contracts.md:50`. The batch trait
  surface exists (`stream.rs:97` `next_batch -> Option<&[Quad64]>`) and the
  FFI stream is REAL and green — `stream_c.rs` implements the 4-fn-pointer
  membrane, `stream_courts` 6/6 exit 0 (lane-11 run), use-after-release
  handled as the typed dead state (`core_of` returns None on NULL
  private_data, `stream_c.rs:69-75`). ArrowArrayStream remains ABSENT in
  avec (0 hits in crates/include/bindings, `03-substrate-map.md:24`).
- **Close condition**: stream court: a real foreign C host consumes stardf's
  stream via the 4 fn pointers and byte-compares against a purrdf CanonArrow
  export of the same quad set; plus a gate asserting the 4 fn pointers are
  non-null and release discipline matches avec's CAS-swap release
  (`avec/crates/avec-arrow/src/c_abi.rs:70-77,104-154`; nested cascade
  `c_abi_nested.rs:14-24,63-91`).

## KD-06 — Option<TermId> niche lost in u32 repr — graph slot encoding

- **Surface**: graph-slot nullability encoding.
- **purrdf**: nullability is a type-level fact — `g: Option<TermId>` packed
  into the same 4 bytes by the `NonZeroU32` niche
  (`dataset.rs:143-154`); the compiler proves exhaustiveness.
- **stardf/arrow**: nullability is a sentinel convention — graph slot = 0
  (BOTTOM) encodes the default graph (`layout.rs:19-29`). Compiler no longer
  proves None/Some; payoff is C-ABI-clean u32 words (KD-05's C Data Interface
  stream cannot express a niche-packed Option).
- **Close condition**: pattern-court over the graph slot re-expressing the
  purrdf `GraphMatch` trio (`dataset.rs:727-731`: Any → 0/zero-mask,
  Default → BOTTOM keyed under all-ones mask, Named(id) → id) over
  sentinels, asserting identical match sets; plus a lint/CI gate refusing any
  `Option<TermId>` in `#[repr(C)]` stardf layouts (niche must not silently
  re-enter the ABI).

## KD-07 — purrdf branchless-by-comment kernels vs avec CC=1 gate

- **Surface**: kernel branch discipline (CC=1 branchless ABI audit).
- **purrdf**: branchless is a doc comment, not a gated ABI — the `QuadKey`
  match predicate `(word ^ key) & mask == 0` is documented branchless
  (`dataset.rs:705-714`) but no purrdf CI gate polices CC on kernels
  (`ci.yaml:438-460` is the header-drift gate only).
- **avec**: CC=1 branchless kernel-ABI gate with a manifest:
  `avec/scripts/audit_branchless.sh` + `scripts/branchless_manifest.toml`
  (spec `avec/docs/jira/v26.10.31/27-branchless-kernel-abi.md` §5).
- **stardf/arrow**: PLANNED — adopt the avec gate for stardf kernels;
  the waiver list (kernels exempted from CC=1) is carried in
  `branchless_manifest.toml` and must be re-justified per entry at adoption.
- **Close condition**: `audit_branchless.sh`-equivalent gate wired into the
  stardf CI with a stardf-scoped manifest; gate exits 1 on any non-waived
  kernel with CC > 1, and every waiver row names its justifying receipt.

## KD-08 — BLAKE3 single-impl decision (purrdf-hash vs avec dual-impl defect)

- **Surface**: BLAKE3 implementation count in the dependency graph.
- **purrdf**: single BLAKE3 path — Blake3 content ids in the FrozenTerms
  arena (`purrdf/crates/rdf-core/src/ir/frozen_terms.rs:17-93`, per
  `03-substrate-map.md:38`). No duplicate impl in the purrdf tree re-read
  this session.
- **avec (registered defect, not a pattern)**: two BLAKE3 implementations —
  the `avec-blake3` crate plus a duplicate in `avec-arrow/src/blake3.rs`
  (`03-substrate-map.md:26-27`).
- **stardf/arrow**: binds to ONE — `blake3::hash` in
  `stardf/arrow/crates/stardf-arrow-core/src/arena.rs:318`; fingerprints
  stored per minted id (`arena.rs:241,318`).
- **Close condition**: dependency court: `cargo tree -e normal` over the
  stardf workspace resolves exactly one blake3 identity (gate exits 1 on
  >1); plus a digest-parity court hashing the same bytes through stardf's
  bound impl and the `avec-blake3` crate asserting identical digests (the
  single bound impl proven byte-compatible with the substrate's).

  **Status update 2026-10-31 (lane 8)**: half of close condition (a) now
  EXISTS — `stardf/arrow/scripts/verify_blake3_identity.sh` runs
  `cargo tree -e normal` and exits 0 with "exactly 1 blake3 identity:
  blake3 v1.8.7" (WARN: Cargo.toml declares 1.8.5, lock resolves 1.8.7 —
  silent-drift class). The purrdf-hash recommendation remains NOT implemented:
  the columnar crate binds the blake3 crate too
  (`crates/stardf-columnar/Cargo.toml [dependencies] blake3 = "1.8.5"`).
  Digest-parity court (b) and the golden court vs purrdf digests (c) have not
  run. Row stays OPEN.

## KD-09 — allocation: purrdf `Admission` trait adopted, avec fixed-budget court impl

- **Surface**: allocation admission contract.
- **purrdf**: `Admission` trait —
  `purrdf/crates/lex/src/allocation.rs:272-276`
  (`fn resize(&mut self, live_bytes) -> Result<(), StorageError>`;
  "admit the next exact total before allocation; shrink only after
  destruction").
- **avec**: fixed-budget court substrate — `CountingAlloc`
  (`avec/crates/audit/src/alloc_counter.rs:24-67`); dealloc NOT counted
  (:65-67) — the W3 hazard.
- **stardf/arrow**: ADOPTS purrdf's `Admission` trait as the allocation
  contract; avec's `CountingAlloc` is the court instrument, not the
  contract.
- **Close condition**: alloc court (avec idiom:
  `avec/crates/avec-ffi/tests/alloc_courts.rs:30-133`, SLOTS Mutex
  serialization) driving a stardf `Admission` impl — resize() to exact
  totals, shrink-after-destroy asserted via CountingAlloc live-byte deltas;
  the court fails on any allocation outside the admitted envelope, and
  asserts on live-byte deltas never cumulative alloc counts (dealloc
  uncounted, `alloc_counter.rs:65-67`).

## KD-10 — columnar physical id width: Int64-widened u32 vs purrdf native i64

- **Surface**: physical Parquet id-column encoding for the five-table schema.
- **purrdf-columnar**: ids are native i64 dictionary indices in Int64 columns
  (`purrdf/crates/columnar/src/schema.rs:122-161` — every s/p/o/g/reifier/
  predicate/value column is `ColumnSchema::int64`; `writer.rs:315-376` re-mints
  `D::Id` to i64 via the `DatasetView` seam).
- **stardf/arrow**: u32 `TermId` space (`layout.rs:23-29`) widened at write to
  PLAIN INT64, checked-conversion narrowed at read — **Int64-widened is the
  admitted decision** (06 §10 item 7; `docs/COLUMNAR-ARD.md` §4). The Int32
  alternative (golden regen) is PENDING-OPERATOR (`07-port-master-plan.md`
  OPEN-7: decided by the W4 digest-equality court).
- **Close condition**: W4 purrdf-vs-stardf digest-equality court — same
  splitmix64-seeded fixture (`state=0x6012`, 06 §3) written through both must be
  byte-identical; any later Int32 admission only after golden regen with
  recorded reason.

## KD-11 — u32 dictionary adaptation: BOTTOM/WILDCARD sentinels + `u32::MAX-2` cap

- **Surface**: dictionary id space and sentinel semantics vs purrdf's i64
  no-sentinel dictionary.
- **purrdf-columnar**: `Dictionary { terms: Vec, ids: BTreeMap<TermValue,i64>,
  named_graphs: BTreeSet }` (`writer.rs:323-326`) — no integer sentinels, no
  reserved top-of-space values; optional graph slot is an `Option<i64>` column.
- **stardf/arrow**: `BOTTOM_TERM_ID = 0` (unbound/default graph),
  `WILDCARD_TERM_ID = 0xFFFF_FFFF` (pattern layer only), dictionary cap
  `u32::MAX - 2` per P0-5/06 §2; `Option<i64>` graph slot collapses to u32 with
  BOTTOM = absent (D3); `from_raw(u32::MAX)` is a typed capacity refusal
  (`arena.rs:59-70`). Specified in `docs/COLUMNAR-ARD.md` §3.

  **Status update 2026-10-31 (lane 8)**: the columnar crate EXISTS
  (`crates/stardf-columnar/`) and the sentinel/cap seam is courted —
  `tests/seam_courts.rs` 4/4 pass, exit 0, including
  `dictionary_cap_refuses_typed`. Open remainder: the purrdf differential
  truth-table reuse at the columnar boundary.
- **Close condition**: (a) KD-02 truth-table court reused at the columnar
  boundary; (b) over-cap `Dictionary::build` court asserting typed fail-closed
  refusal (never wrap); (c) court asserting a `WILDCARD` word in a data row is
  a typed refusal.

## KD-12 — digest identity: purrdf-hash recommended vs stardf's blake3 crate

- **Surface**: content-address digest implementation behind the columnar
  blob/digest surface (Blobs table, `schema.rs:158-161`; reader verifies every
  blob digest, `reader.rs:40-44`).
- **purrdf-columnar**: single purrdf-hash native path; Blake3 content ids in
  FrozenTerms (`purrdf/crates/rdf-core/src/ir/frozen_terms.rs:17-93`, per
  `03-substrate-map.md:38`; KD-08 row).
- **stardf/arrow**: recommendation registered — **purrdf-hash native**
  (zero-dep/no_std parity), since pulling both blake3-crate and purrdf-hash
  violates the single-impl decision (06 §7, P0-18). stardf/arrow currently
  binds one blake3 crate at `arena.rs:318`; the recommendation is NOT yet
  implemented — no gate, no digest-parity court. Digest divergence is a
  byte-freeze divergence for columnar, so this row gates KD-10/KD-15 close
  conditions.
- **Close condition**: (a) `cargo tree -e normal` single-blake3-identity gate
  (exit 1 on >1); (b) digest-parity court: same bytes through the stardf-chosen
  impl and purrdf-hash produce identical digests; (c) columnar golden-bytes
  court vs purrdf-columnar golden digests at rev 0da05783.

## KD-13 — error taxonomy: per-surface reason enums carried, stardf refusals added

- **Surface**: typed failure vocabulary of the columnar codec.
- **purrdf-columnar**: `ColumnarError` (`#[non_exhaustive]`, fail-closed,
  `error.rs:25-126`) with per-surface reason enums (`error.rs:5-22`:
  Reader/Projection/Construction/Metadata/Compact/ValueBody/PageBody) and
  `StructuredError` derives (`error.rs:126,303`); 9 structured-error courts
  upstream (`tests/structured_errors.rs`, 41 test fns).
- **stardf/arrow**: taxonomy shape carried by decision (ARD §2), extended with
  the stardf `TermId`-typed refusals (KD-02 `ArenaError::Capacity`).

  **Status update 2026-10-31 (lane 8)**: the taxonomy is PORTED —
  `crates/stardf-columnar/src/error.rs` + `src/error/` carry the seven
  per-surface reason enums, and `src/vendor/hash.rs` carries the vendored
  `StructuredError` surface. Open remainder: the structured-error court corpus
  port (`tests/disabled/structured_errors.rs-pending-stardf-substrate`).
- **Close condition**: structured-error court corpus ported to stardf — every
  `ReaderReason`-class malformed input from
  `purrdf/crates/columnar/tests/structured_errors.rs` must produce the
  corresponding typed stardf refusal (court exits 1 on any untyped/panic path).

## KD-14 — golden digests: purrdf pins in Rust source vs stardf `.sha256` manifest

- **Surface**: storage medium for byte-freeze golden digests.
- **purrdf-columnar**: digests pinned in Rust source (06 §3: "purrdf pins
  digests in Rust source, not a `.sha256` manifest").
- **stardf/arrow**: must add the `.sha256` manifest over the copied vectors —
  law F4b (`stardf/docs/jira/v26.10.31/04-falsifiers.md:89-94`: the
  corrupted-copy FAIL is the falsifier; the manifest does not exist yet).

  **Status update 2026-10-31 (lane 8)**: the manifest EXISTS at
  `stardf/arrow/scripts/columnar_manifest.sha256` over the ten golden digests
  (5 tables x 2 compression modes), mirrored by the
  `manifest_mirrors_the_pinned_digests` court with the byte-flip positive
  control `flipping_a_byte_moves_the_digest` (6/6, exit 0). Open remainder:
  the corrupted-copy mutation as a wired CI gate, and purrdf digest-ordering
  parity.
- **Close condition**: manifest gate — corrupted-copy mutation FAILs (F4b's own
  falsifier executed as a court, not a skip), plus digest-ordering parity with
  purrdf golden output.

## KD-15 — determinism/branch/unsafe gates: carried as ARD invariants, none wired

- **Surface**: CI gates carried from purrdf-columnar/avec onto the columnar
  surface (KD-07's branchless-gate class, F5 determinism-receipt court,
  `#![forbid(unsafe_code)]` enforcement).
- **purrdf-columnar**: `#![forbid(unsafe_code)]` (`lib.rs:21`), determinism
  witness test `writer.rs:698`
  (`writer_projects_all_rdf_layers_and_is_deterministic`), byte-deterministic
  recipe (PLAIN/RLE/DataPageV2, pinned Zstd level-3 `structured-zstd 0.0.49`;
  06 §3).
- **stardf/arrow**: all three carried as ARD invariants (ARD §2).

  **Status update 2026-10-31 (lane 8)**: the columnar crate EXISTS and carries
  `#![forbid(unsafe_code)]` (`crates/stardf-columnar/src/lib.rs:21` +
  `[lints.rust] unsafe_code = "forbid"` in its Cargo.toml); the write-twice
  determinism court is green locally (`writing_twice_is_byte_identical`,
  6/6 exit 0). Still NOT wired: the F5 two-run CI determinism replay, the
  branchless `audit_branchless.sh` gate (`scripts/branchless_manifest.toml:44`
  — the script is still TO BE LANDED, manifest is inventory of record), and
  the F5 receipt fields (`receipt_sha256`/`artifacts_sha256`).
- **Close condition**: (a) stardf columnar crate carries
  `#![forbid(unsafe_code)]` with a CI assert; (b) determinism court: two CI
  runs produce identical artifact digests (F5); (c) KD-07 branchless-gate
  manifest extended with the columnar kernel list.

## Wave-1 conflict rows W1-C1..C5 (reconciled to as-built, lane 8, 2026-10-31)

Source: `evidence/columnar-wave-2026-10-31.json` conflict_log (phase-2
closure lane 18). Each row: open/closed + the receipt that closes it.

| Row | Observation (superseded side) | Resolution | Status | Closing receipt |
|---|---|---|---|---|
| W1-C1 | F16: stream_courts 5/6 (court_2) | 6/6, exit 0 per consolidated + LANE-11 | CLOSED | `evidence/columnar-wave-2026-10-31.json` C1; lane-8 workspace run: stream_courts 6 passed, exit 0 |
| W1-C2 | F16 nonclaim: `--emit-receipt` flag NOT landed | flag on disk at `scripts/verify.py:136`; replay pair byte-identical | CLOSED | C2 receipt: two `--emit-receipt` runs exit 0, `cmp` 0, sha `cb57ce4e…` |
| W1-C3 | W14 wave report: 8 exports | 13 header decls == 13 exports == 13 ontology targetSymbols | CLOSED | C3 receipt: lane-own verify.py run, three-way parity |
| W1-C4 | consolidated: anti_vacuity 1/3, capi E0308 compile-abort | capi repaired by owning lane; anti_vacuity fixture drift | CLOSED | lane-8 `cargo test --workspace --no-fail-fast` → exit 0, anti_vacuity 3 passed / 0 failed — the final witness; earlier 1/3 witnesses superseded |
| W1-C5 | dispatch named S8–S16 per-lane receipts | no S8–S16 artifact on disk; per-lane UNKNOWN, not guessed | CLOSED (as recorded UNKNOWN) | C5 receipt consolidation note; naming did not survive to disk |

KD-candidate provenance: KD-10..KD-15 were registered as candidate rows by the
wave-1 planning lanes (06/07 docs) and confirmed as the row set by completion
lane 11's status table; as-built reconciliation of each is the status table
above (2026-10-31, lane 8 truth pass #3). No additional KD-candidate rows from
the W1 lanes resolve to artifacts on disk.
