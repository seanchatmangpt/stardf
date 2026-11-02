# VENDOR-PROVENANCE — stardf-xsd

## Source

Vendored from PurRDF `crates/xsd` at rev
`0da0578306983da851a9073337d57e2e2c41ab3a` (verified: `git rev-parse HEAD` in
`/Users/sac/purrdf` = the pinned rev, and `git diff 0da05783 --stat --
crates/xsd/src` is empty, so tree = rev for this crate), extracted by
`git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a
crates/xsd` (archive stream sha256
`4a58fda637a95200233fac03a746cf24a21925b5a4db50286987048478e9e015`), never
the dirty tree. 65 files carried (`src/` + `licenses/`), tree digest
`f148ce256c07232c92b80a4bf6e82a0108e3ecb0639aebd2794e42cf1c4cccf1`
(sha256-of-sorted-sha256-manifest, per-file loop receipt below).

## Posture

- Package `purrdf-xsd` -> `stardf-xsd` (package rename ONLY). Lib name stays
  `purrdf_xsd` (verbatim, per the vendoring law: lib names, LICENSE files,
  in-file notices unchanged) — same posture as stardf-hash's `purrdf_hash`
  and stardf-lex's `purrdf_lex`.
- Sources byte-verbatim: every file under `src/` diffed against
  `git show 0da05783:crates/xsd/src/<path>` — zero drift (per-file diff
  loop receipt, this lane 2026-10-31: `ALL_SRC_FILES_VERBATIM_OK`).
- LICENSE files carried verbatim in `licenses/` (LICENSE-APACHE, LICENSE-MIT,
  LICENSE-MULAN, NOTICE.txt, CC-BY-4.0.txt, inventory.json; NOTICE.txt diffed
  byte-equal).
- `README.md` upstream-verbatim.

## Manifest (reconstructed, disclosed)

- edition `2024` + rust-version `1.98` — upstream-equal (workspace default is
  2021; member-level override disclosed). Upstream `[package]` metadata
  carried (keywords/categories verbatim); upstream `documentation`/`homepage`
  keys point at upstream docs.rs/website paths and are OMITTED (they would be
  false here); `[package.metadata.docs.rs]` omitted (nothing is feature-gated
  here).
- Runtime deps: dep keys `stardf-hash`/`stardf-lex` (path deps, workspace
  members) — code-level paths resolve through those crates' verbatim lib
  names `purrdf_hash`/`purrdf_lex`.
- Dev-deps use the inverse form (dep KEY = upstream name,
  `package = "stardf-testkit"` / `"stardf-alloc-probe"`): those two substrate
  crates keep their default (package-name) lib names, so the KEY rename is
  what makes the vendored sources' verbatim `purrdf_testkit::` /
  `purrdf_alloc_probe::` paths resolve. Kept DECLARED because the lib unit
  tests (live) consume them; the parked integration tests are not the only
  consumer.
- Upstream `[[test]]`/`[[bench]]` tables omitted: the 14 integration tests,
  3 benches and 1 example are PARKED (not deleted) under
  `tests|benches|examples/disabled/`, unpark preconditions in the READMEs
  there; `tests/disabled/upstream-Cargo.toml.txt` is the archive receipt of
  the upstream manifest.
- `[lints]` inlined from upstream `[workspace.lints]` verbatim (this
  workspace declares no `[workspace.lints]`; without the inlining, clippy
  would default-lint vendored code it was never written against).

## Lane-collision history (disclosed, coordinator-relevant)

This directory was vendored CONCURRENTLY by two lanes of the same wave:

1. The sparql-algebra lane landed an xsd posture first (workspace manifest
   comment: "purrdf xsd -> stardf-xsd (lib purrdf_xsd verbatim). Runtime
   deps: stardf-hash/stardf-lex paths. MANIFEST COLLISION DISCLOSED ...").
2. This lane (the xsd vendor lane) independently built an in-crate shim
   posture (src/vendor/ shims + crate-path substitution of every
   `purrdf_hash`/`purrdf_lex` path in the vendored sources). The postures
   collided on the shared directory; the substituted sources and this lane's
   manifest were what ended up on disk.
3. ARBITRATION (this lane, fix-forward): the concurrent lane's architecture
   is the vendoring-law-faithful one — with stardf-hash/stardf-lex vendored
   as real workspace crates carrying verbatim lib names, xsd's sources stay
   byte-verbatim with ZERO source edits. This lane's shim posture (approx.
   14K lines of duplicated substrate under src/vendor/) was WITHDRAWN: every
   shim file was moved out of the tree to
   `/tmp/stardf-xsd-vendor-shims-rejected-20261031/` (retained for
   coordinator inspection, not deleted — they are receipts of the collision);
   sources restored byte-verbatim from the pinned rev; manifest rebuilt in
   the concurrent lane's declared posture plus the dev-dep correction below.
4. Cross-lane manifest edits by the concurrent lane during the collision
   window (commenting the two dev-deps as "PARKED-DEVDEP ... Restore the line
   when that lane unparks them") were REVERTED with reason: the dev-deps
   serve the live lib unit tests, not only the parked integration tests.
   Escalated for coordinator awareness; no further cross-lane manifest writes
   by either lane without arbitration.

## Unit-test parity court

`cargo test -p stardf-xsd --lib` → 413 passed / 12 failed (the initial 42
compile errors were fixed by the dev-dep key renames above; runtime 43s,
4 threads).

Court against upstream: `cd /Users/sac/purrdf &&
CARGO_TARGET_DIR=/tmp/purrdf-xsd-court-target cargo test -p purrdf-xsd --lib`
(same machine, same toolchain, isolated target dir; purrdf working tree
untouched — Cargo.lock diff-clean) → 413 passed / 12 failed, failing set
BYTE-IDENTICAL (receipt: 12-line failing-set diff `FAILING_SETS_BYTE_IDENTICAL`,
this lane 2026-10-31): bigint::arithmetic_tests x2, bigint::ntt x1,
bigint::scratch x1, bigint::tests x1, exact::rational::certified x2,
ieee::ratio x3, math::tests x1.

Verdict: the vendored tree reproduces upstream OUTCOME-FOR-OUTCOME —
including the 12 failures. Each failure is an allocation-boundary assert
observing exactly one extra allocation inside a `CurrentThreadWindow`
(`left: 1, right: 0`); they are UPSTREAM-ENVIRONMENT failures on this host
(macOS 26.2 arm64, floating nightly) — they pre-date vendoring, they are not
a stardf-alloc-probe carve defect, and they are not stardf divergences.
Registered for the coordinator's divergence ledger; fixing them is an
upstream transition, not a stardf edit.

## Divergences (complete list)

1. Package rename purrdf-xsd -> stardf-xsd (the ONLY source-tree edit class;
   the sources themselves are byte-verbatim).
2. Manifest reconstruction: edition 2024 + rust-version 1.98 (upstream-equal,
   member override disclosed), upstream lints inlined, path-dep keys,
   upstream-key dev-dep renames, [[test]]/[[bench]] tables omitted with the
   targets parked, documentation/homepage keys omitted.
3. Parked upstream targets (14 tests, 3 benches, 1 example) under
   {tests,benches,examples}/disabled/ with READMEs + unpark preconditions.
4. The 12 upstream-environment lib-unit-test failures are carried AS-IS
   (outcome-parity with upstream), registered for the coordinator ledger.

## See Also

- `tests/disabled/upstream-Cargo.toml.txt` — the upstream manifest archive
  receipt
- `crates/stardf-hash/VENDOR-PROVENANCE.md`,
  `crates/stardf-lex/VENDOR-PROVENANCE.md` — substrate crates (concurrent
  lane)
- `stardf/docs/jira/v26.10.31/12-full-parity-campaign.md` — W1 ladder
