# PROVENANCE — stardf-columnar

## Source

Vendored from PurRDF `crates/columnar` at rev `0da0578306983da851a9073337d57e2e2c41ab3a`
(verified: `git rev-parse HEAD` in `/Users/sac/purrdf` = `0da05783...` on 2026-10-31),
per the lane-5 rename discipline:

- package `purrdf-columnar` -> `stardf-columnar` (lib `stardf_columnar`)
- original LICENSE files preserved in [`licenses/`](licenses/) (LICENSE-APACHE,
  LICENSE-MIT, LICENSE-MULAN, NOTICE.txt, CC-BY-4.0.txt, inventory.json)
- SPDX headers preserved in every vendored source file

## Adaptation: u32 term-id seam (P0-5)

The Dictionary seam was moved from `i64` to `u32`:

- `BOTTOM` = `0`: reserved absent-semantics sentinel, never minted.
- `WILDCARD` = `u32::MAX`: refused as mintable; the reader refuses it and any
  `0` in a term/graph reference position (`ReaderReason::TermReference`,
  `ReaderReason::MissingGraph`).
- Mintable cap `u32::MAX - 2` (`writer::MAX_MINTABLE_TERM_ID`); ids minted
  dense in `1..=MAX_MINTABLE_TERM_ID`; the reader requires `terms.id` dense
  from 1 (`id == row + 1`) and capped.

Adapted per the P0-5 file:line list:

- `src/writer.rs` — `QuadRow`/`ReifierRow` (was :19-20), `Dictionary` (was
  :315-376), `build_terms_table` id push (was :392-411), `TermColumns` push
  sites (was :488-526)
- `src/column.rs` (was :192-295) — unchanged `Int64Column` (see decision)
- `src/schema.rs` (was :48) — unchanged
- `src/parquet.rs` (was :94-162) — unchanged `ColumnValues` (see decision)
- `src/reader.rs` — `required_term_ref` / `optional_graph_ref` /
  `parse_term_records` dense-id check / `reconstruct_dataset` row derivation
  (`term_row`)

## DECISION (recorded, lane 9): keep `PhysicalType::Int64`, widen u32 at push

`Int64Column` and `ColumnValues::Int64` are UNCHANGED from upstream. u32 term
ids are widened to i64 at every column push (`i64::from(id)`), so on-wire
bytes are golden-stable against upstream. The `Int32` physical variant is
**lane 11's PENDING-OPERATOR row**, not decided here.

## Dependencies (disclosed)

Upstream deps `purrdf-hash`, `purrdf-core`, `purrdf-lex` are not available in
the stardf/arrow workspace. Minimal in-crate shims live in
[`src/vendor/`](src/vendor/):

- `vendor/hash.rs` — `StructuredError`/`ErrorValue`/`ErrorVisitor`/`visit_list`
  and the `error_visit!` macro: extracted VERBATIM from purrdf rev 0da05783
  (only `$crate` path substitutions). Divergence: upstream carries a richer
  visitor protocol (begin/end record, native upcasting); this shim keeps the
  subset the codec's impl sites call. `error_source!`/`variant_from!`
  similarly extracted verbatim from purrdf-lex.
- `vendor/core/` — minimal RDF kernel seam: `TermValue`/`TermBox`/`TermRef`/
  `DatasetView`/`RdfDataset(+Builder)`/`ContentStore`/`ContentDigest`/
  `LossLedger`/`RdfDiagnostic`/`vocab`/`langtag`/`ir::pack::bits` (the bits
  module extracted verbatim from purrdf-core). The generic `DatasetView`
  boundary is preserved; the builder validates reference ranges, NOT the full
  upstream kernel structural validator (disclosed divergence).
- Digests: **blake3** (workspace-existing dep, 1.8.5, default-features off)
  for content digests — blake3-256, same 64-hex-char shape as upstream.
  Upstream routes through purrdf-hash's allocator-metered hash; the digest
  function is blake3 in both. Disclosed divergence: no allocator metering.
- Compression: `structured-zstd = "0.0.49"` from crates.io (same codec
  upstream pins; the workspace lockfile resolves it fresh).

## Preserved upstream test modules (inert)

Every upstream `#[cfg(test)]` module is preserved behind
`#[cfg(all(test, feature = "upstream-tests"))]`. That feature is off by
default and INERT in this workspace: the gated modules require purrdf-core /
purrdf-testkit / purrdf-alloc-probe / purrdf-stack, which are not workspace
members. They are preserved as reviewable text, not dead claims.

## tests/ contents (cross-lane, disclosed)

- `tests/seam_courts.rs` (lane 11): landed in this crate per its own header
  note; honored by declaring `[dev-dependencies] stardf-arrow-core = { path =
  "../stardf-arrow-core" }` in this crate's Cargo.toml. 4/4 pass.
- `tests/golden_bytes.rs`: landed by another lane; compiles against the
  public API alone. 6/6 pass.
- `tests/disabled/structured_errors.rs-pending-stardf-substrate` (lane 9
  parked, compile-freeze SLA): requires `stardf_core`/`stardf_hash`/
  `stardf_testkit`/`stardf_alloc_probe`/`stardf_lex` crates that do not exist
  in this workspace yet. Cargo ignores `tests/disabled/`. Unpark when the
  stardf substrate lands (see tests/disabled/README.md).

## In-crate verification (this workspace)

`src/smoke.rs` (cfg(test), default features) runs a real write->read round
trip over the shim `RdfDataset` `DatasetView` impl: quad + directional literal
in a named graph + reifier binding + annotation + blob, byte-identical
second-write determinism, five zero-row files for empty input, and a BOTTOM(0)
id refusal court. `cargo test -p stardf-columnar --lib`: 3 passed.

## Vendor shim completion (lane 1, 2026-10-31)

Landed the four vendor modules the W1-11 gap list still showed missing, each a
minimal, real implementation of the consumed slice only:

- `src/vendor/testkit.rs` — `rng` (real splitmix64: `SplitMix64`,
  `splitmix64_next`, `below_usize`) and `error_capture`
  (`Capture::read`/`read_with_buffer`/`field`/`child` + `Event`), driving the
  vendored `StructuredError` visitor.
- `src/vendor/alloc_probe.rs` — real counting `GlobalAlloc` over `System` with
  per-thread measurement windows (`CountingAllocator`, `CurrentThreadWindow`,
  `Measured`).
- `src/vendor/core/backend.rs` — path-parity re-export of the
  `model::TermFactory` interning seam.
- `src/vendor/core/term_fixture.rs` — `TermShape::WellFormed`, `term_value`
  (injected splitmix64 draw fn + depth budget), `triple_chain` oracle fixture;
  plus `BlankScope::DEFAULT` added to `vendor/core/model.rs`.

Disclosed divergences:

1. `unsafe_code` lint relaxed `forbid` -> `deny` (src/lib.rs + Cargo.toml):
   `alloc_probe`'s counting `GlobalAlloc` cannot be written safely. The unsafe
   surface is scoped to that one module via `#[allow(unsafe_code)]` on its
   `pub mod` declaration in `vendor/mod.rs`; any other unsafe site still fails
   the build. Closes the W1-11 vendored-substrate gap for the preserved
   upstream test modules (the `--features upstream-tests` surface), which is
   the `tests/disabled/structured_errors.rs` unpark precondition alongside the
   future `stardf_*` substrate crates.
2. `structured-zstd` pin moved to root `[workspace.dependencies]`
   (`structured-zstd = "0.0.49"`, one line + comment); the member manifest
   consumes `structured-zstd.workspace = true`. Single-pin law, closes W1-C5.
3. `ErrorValue` in `vendor/hash.rs` gained `#[derive(Clone, Copy)]` (all
   variants hold references/primitives) so `Capture::field` can return by
   value as the preserved tests pattern-match it.

Gate receipts (this session): `cargo check -p stardf-columnar` exit 0;
`cargo check --workspace` exit 0; `cargo test -p stardf-columnar` exit 0
(10 passed: 3 lib smoke + 4 seam courts + ... see output).

## Vocab extraction (lane: vocab-consts, 2026-10-31)

The vocabulary constants and the language-datatype dispatcher now live in one
home, `src/vendor/core/vocab.rs` (values verbatim from purrdf rev
`0da05783` — `crates/iri/src/vocab.rs` and `crates/xsd/src/datatype.rs`):

- `vocab::xsd::{XSD_INTEGER, XSD_STRING}` and `vocab::its::NS` added;
  `vocab::rdf::{REIFIES, LANG_STRING, DIR_LANG_STRING}` unchanged.
- `vocab::language_datatype_iri(has_direction: bool)` — upstream
  `purrdf_iri::vocab` shape — replaces the inline IRI spellings that
  `model.rs`'s `language_datatype_iri`/`RdfLiteral::language_datatype_iri`
  carried; both now delegate via `direction.is_some()` (same shape as
  upstream `purrdf_core::model::RdfLiteral:124-127`).
- Call-site re-points, behavior-identical (same string values):
  `term_fixture.rs` xsd:integer literal -> `vocab::xsd::XSD_INTEGER`;
  `smoke.rs` dirLangString literals -> `vocab::rdf::DIR_LANG_STRING`.
- New in-file courts: `vocab::tests::vocabulary_constants_are_namespace_plus_local_name`
  and `language_datatype_iri_dispatches_on_direction`.

Gate receipts: `rustfmt --check` (4 edited files) exit 0;
`cargo check -p stardf-columnar` exit 0;
`cargo test -p stardf-columnar` exit 0 (lib 5 passed incl. both new vocab
courts; workspace-owned integration suites 4 + 7 passed, 3 loud
KD-registered ignores); `cargo clippy -p stardf-columnar --all-targets
-- -D warnings` exit 0, 0 diagnostics.

Not touched: `tests/golden_bytes.rs` inline langString/dirLangString
expectations — independent court literals are deliberate redundancy, not
drift (other lane's file).
