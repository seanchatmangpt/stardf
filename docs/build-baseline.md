# Build Baseline Board (S-2, 2026-11-01)

Receipt for the 53-crate stardf-arrow workspace reconstitution baseline.
All commands run from `/Users/sac/stardf/arrow` this session; exits cited
per command. Status: PARTIAL_ALIVE.

## Justfile ladder (the intended verify ladder)

| target | command |
|---|---|
| build | cargo build --workspace |
| check | cargo check --workspace |
| test | cargo test --workspace |
| clippy | cargo clippy --workspace --all-targets -- -D warnings |
| fmt | cargo fmt --all |
| ci-all | fmt + clippy + check + test |
| bench | cargo bench -p stardf-arrow-bench |
| columnar-oracle | bash scripts/check-columnar-oracle.sh (skip loud, exit 3 if no duckdb) |
| blake3-gate | bash scripts/verify_blake3_identity.sh |
| rehome-court | bash scripts/verify_duckdb_rehome.sh |
| header-freeze | python3 scripts/verify_header_freeze.py |

## The board

Build: `cargo build --workspace` — EXIT=0 (first run EXIT=101 on the capi
manifest defect below; post-fix EXIT=0). 310 crates compiled (workspace
members + deps), 28 warnings (mostly unused-workspace-dep + missing [lints]
inheritance).

Tests: `cargo test --workspace --no-fail-fast` — EXIT=101. Recorded:
2510 passed / 33 failed across 112 ok suites + 4 FAILED suites. Of the 33
recorded failures, 29 are doctest "extern location does not exist"
(stardf-hash doc suite) — environment-clobber artifacts, not genuine.
Genuine failures: 5 tests / 3 suites. 554 additional targets "never
executed" (binary missing at spawn) — clobber artifacts (see divergences).

### Genuine failures (the finishing critical path)

| suite | count | coordinate | panic |
|---|---|---|---|
| stardf-rdf-core lib (as stardf-core -p alias) mutable_declared_graph_drain | 1 | crates/stardf-rdf-core/tests/mutable_declared_graph_drain.rs:319 | assert left==right (true vs false) |
| stardf-rdf-core native_regex_storage | 1 | crates/stardf-rdf-core/tests/native_regex_storage.rs:1598 | assert left==right |
| stardf-duckdb chicago_agent_hacker_ergonomics_court | 2 | crates/stardf-duckdb/tests/chicago_agentHACKER_court.rs:47 | perf fences: boot 101.07ms, tool validation 6.92ms exceed fences |
| stardf-hash --doc | 29 | doctests, "extern location for purrdf_hash does not exist" | clobber artifact, re-run to clear |

Note: the duckdb court fences are absolute-ms fences — per the no-sleep /
ratio-fence law these are flake-prone by construction; classify as
fence-defect, not product regression, until re-run on a warm cache.

## Divergences

1. arrow/CLAUDE.md referenced in the lane contract does not exist on disk
   (checked, exit 1). ABI-FREEZE.md + OPERATOR-DECISIONS.md read instead.
2. Pre-existing manifest defect (fixed this session, minimal unblock, disclosed):
   `crates/stardf-arrow-capi/Cargo.toml:60-61` had 5-up relative paths
   (`../../../../../cgap/...`) resolving to nonexistent /Users/cgap/...;
   corrected to 4-up (`../../../../cgap/...`). Original defect is pre-existing;
   the 2-line fix is session-introduced and disclosed here. This single
   manifest defect blocked workspace resolution for ALL 53 crates — it is
   the critical path root.
3. Mid-run target-dir clobber: /Users/sac/stardf/arrow/target/ was deleted
   by an external process at ~20:16 PST while cargo test was running
   (554 targets "never executed", 29 doctest extern-missing failures).
   A concurrent fleet process (cleanup law misfire: the in-tree target/ is
   the workspace build, not lane debris) removed the workspace build dir.
   Test numbers above are from the interrupted run; a clean re-run requires
   a full rebuild. Do not treat the 554 as test failures.
4. stardf/ is untracked in the parent repo (D1, OPERATOR-DECISIONS.md) —
   receipts bind to tree state, not SHA.

## Critical path

1. Keep the manifest path fix (capi, 2 lines) — unblocks resolution.
2. Fix the two stardf-rdf-core test failures (assertion regressions).
3. Ratio-fence the duckdb hacker-ergonomics court (absolute-ms fences).
4. Re-run the full test ladder on an intact target/ for a clean baseline.
