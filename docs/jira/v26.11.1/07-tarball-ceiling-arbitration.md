# Tarball Ceiling Arbitration — stardf-core v26.11.1 (Publish Lane 7)

> STATUS: DECISION-READY, NOTHING EXECUTED. The S1 10.25 MB blocker is already
> closed on disk by the landed `exclude`
> (`arrow/crates/stardf-rdf-core/Cargo.toml:35`, landed 22:11): both ceilings
> measure green as-of 2026-10-31 23:02 PDT. This document arbitrates
> options (a)/(b)/(c) and executes none of them — no manifest edit, no publish,
> no git action. Remaining operator row: 01-register S1 (ladder measurement
> basis). File owner: lane 7 only.

## Subject

Blocker S1 (`01-publish-blockers-register.md` §S1): stardf-core 10.25 MB raw
package sum vs the 10 MB ceiling; sole driver
`tests/csv_differential_vectors.txt` (1,504,576 B, `include_str!` at
`crates/stardf-rdf-core/tests/csv_differential.rs:75`).
Package `stardf-core` lives at `arrow/crates/stardf-rdf-core/` (dir name is a
copy-time artifact; lib name `purrdf_core` verbatim per the vendoring law).

Two different ceilings are in play; conflating them is the S1 confusion:

1. **The repo's own fail-closed gate** (`publish_stardf.py:76,510-512`):
   `MAX_PACKAGE_BYTES = 10 * 1024 * 1024` = 10,485,760 B, enforced against the
   **uncompressed package-file sum** (`cargo package --list` sizes).
2. **The crates.io ceiling**: 10 MiB against the **compressed `.crate`**
   artifact that upload accepts.

## Measurements (all run this lane, 2026-10-31 23:02 PDT, from `arrow/`)

- **R1 — gate-side uncompressed sum.** `cargo package --offline --allow-dirty
  --list -p stardf-core` (exit 0): 318 files, sum **9,245,324 B** — under
  10,485,760 by **1,240,436 B**; `csv_differential_vectors` NOT in list;
  5 license rows.
- **R2 — real compressed `.crate`.** `CARGO_HOME=/tmp/h03-ch
  CARGO_TARGET_DIR=target-lane07 cargo package --offline --allow-dirty
  --no-verify -p stardf-core` (exit 0): `target-lane07/package/
  stardf-core-26.11.1.crate` = **2,088,792 B** (1.99 MiB), sha256
  `7e09c6476c59a0087b820d379be280f2f48d87192feaae21901eec11835aa860`;
  tar: 318 entries, vector rows 0, license rows 5; cargo report
  "Packaged 318 files, 8.8MiB (2.0MiB compressed)".
- **R3 — option-(c) compression bound.** `gzip -9` of the vector file:
  241,626 B standalone; realistic `2,088,792 + 241,626 = 2,330,418 B`;
  zero-compression worst case `2,088,792 + 1,504,576 = 3,593,368 B`
  (3.43 MiB) — still 34% of ceiling.
- **X — cross-check witness.** Prior H03-sim artifact
  `target/package/stardf-core-26.11.1.crate` (22:12:14): 2,088,586 B,
  sha256 `23af3e115a00202006ad663ea35390c2b67ea272699ceb6aa197c28d06738fe0`;
  206 B smaller than R2 (src edits between 22:12 and 23:02) — mechanism
  independently reproduces.

Mechanism note (R2): the upload-prep resolution in `cargo package` requires
every dual-keyed (`version` + `path`) sibling dep to resolve as-if-published;
none are on crates.io yet. The H03 harness
(`scripts/h03_bootstrap_sim.sh`) resolves this with a directory source
(`/tmp/stardf-vendor`, registered by `/tmp/h03-ch/config.toml`) seeded with
each sibling `.crate` in publish-tier order. R2 reuses the 22:12 sim's seeds
verbatim (sibling seed staleness does not enter the stardf-core `.crate`
content; seeds satisfy resolution only). No registry was touched.

Receipt-number evolution (same subject, three times; all pre-R2 receipts
include the vector, R1/R2 do not):

| Receipt | Time | Raw sum (B) | Files |
|---|---|---|---|
| H03 bootstrap (manifest comment) | 22:09 | 10,760,224 | — |
| G07 tarball audit (01-register §S1) | pre-22:11 | 10,748,822 | 319 |
| R1/R2 (this lane) | 23:02 | 9,245,324 | 318 |

## Options with mechanics

### (a) dev-feature-gate the consuming test — NOT a ceiling remedy

Mechanics: `[[test]] name = "csv_differential" required-features =
[...]` (or `#[cfg(feature)]`) plus a `[features]` table entry. Packaging is
governed by include/exclude rules, not target gating: **the 1.5 MB vector
still ships** (required-features gates building the test target, not the
package file set). As a ceiling remedy (a) is a no-op. Its real use is
residual hardening (below). Cost: `[features]` table = a disclosed manifest
edit; vendoring-law scope note required.

### (b) relocate the vector to stardf-conformance-kit — REJECTED

Mechanics: move `tests/csv_differential_vectors.txt` to
`stardf-conformance-kit`, repoint `include_str!` at a `../../../` sibling
path. Same package-size outcome as the landed exclude (file leaves the
stardf-core dir entirely), but: it fires the vendoring-law falsifier (a
non-rename edit to vendored source `csv_differential.rs` — new ticket
required); it couples `cargo test -p stardf-core` to a sibling crate's
presence (breaks standalone-crate test runs); zero gain over the landed
exclude. Cost > benefit; rejected.

### (c) ship as-is without the exclude — passes crates.io, re-opens the repo's own gate

Mechanics: drop `exclude = ["tests/csv_differential_vectors.txt"]`. Compressed
outcome measured/bounded in R3: ~2.33 MB realistic, 3.59 MB absolute worst
case — passes crates.io's compressed 10 MiB ceiling with >= 3x margin, and
even keeps the published crate's `cargo test` compiling (include_str! target
present). But the gate-side raw sum becomes `9,245,324 + 1,504,576 =
10,749,900 B` > 10,485,760 → `ladder_dry_run` FAILs the row fail-closed and
stops the ladder. Adopting (c) requires amending the gate first (01-register
S1 option (i) — measure the real `.crate` — which this doc's R2 method
supplies), an operator call. Rejected as a lane-level move.

## Verdict

1. **The violation is already closed.** The landed `exclude`
   (`Cargo.toml:35`) holds both ceilings green on real measurement: raw sum
   9,245,324 < 10,485,760 (margin 1,240,436 B) and compressed 2,088,792 B =
   19.9% of the crates.io ceiling. No option needs executing for the
   publish ladder's ceiling gate to pass today.
2. **Keep the landed exclude.** It is the only option state that passes BOTH
   ceilings with zero further edits.
3. **(c) is available to the operator only as a package deal**: drop the
   exclude AND amend `ladder_dry_run` to measure the real `.crate`
   (01-register S1 (i), R2 method). Trades: published-crate test compilation
   restored + gate semantics matched to crates.io, at the cost of a gate
   amendment. Not recommended for this campaign (no functional consumer of
   the vector outside `cargo test`).
4. **(a) deferred** as optional hardening, (b) rejected.

## Residual risk (named, accepted, non-blocking)

With the exclude landed, an unpacked `.crate` run of `cargo test` fails to
compile `tests/csv_differential.rs` (`include_str!` target absent). This
affects no publish surface: `cargo package` verify builds the lib only,
crates.io runs no tests, docs.rs builds the lib only. Optional hardening:
option (a) through a GL ticket. 01-register §S1 line 241 already names this
residual.

## Nonclaims

- No option was executed: no manifest edit, no file move, no gate amendment,
  no publish, no registry touch, no git action by this lane.
- R2 reused the H03 sim's sibling seeds (22:12 state); the sibling seeds are
  stale relative to 23:02 sibling sources — irrelevant to stardf-core's
  `.crate` content, disclosed anyway.
- `stardf-process`/`cgap-arrow` manifest-parse breakage (~22:11-22:2x, another
  lane's in-flight landing) blocked the default `CARGO_HOME` run briefly; it
  resolved upstream. The R1/R2 commands are re-runnable verbatim now.
- Standing of the S1 operator row is unchanged (01-register owns it); this
  doc arms, does not close, that row.

## See Also

- `01-publish-blockers-register.md` §S1 — the blocker row this doc arbitrates
  (operator decision owed on ladder measurement basis)
- `arrow/crates/stardf-rdf-core/Cargo.toml:30-35` — the landed exclude + its
  disclosure comment
- `arrow/scripts/h03_bootstrap_sim.sh` — the tier-order local-dir-source
  packaging mechanism R2 reuses
- `arrow/scripts/publish_stardf.py:76,510-512` — the gate law (uncompressed
  sum ceiling)
