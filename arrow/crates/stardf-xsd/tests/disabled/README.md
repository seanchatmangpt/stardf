# tests/disabled/ — stardf-xsd

Upstream `crates/xsd/tests/*.rs` (14 integration test files) are PARKED here,
not deleted: they consume the upstream `[[test]]` targets (`any_uri`,
`certified_numerical`, `exact_governance`, `exact_wasm_determinism` — all
`harness = false`), the testkit bench/exact/vectors surfaces, and the frozen
vector corpora under `tests/vectors/` (kept in place).

## Unpark preconditions

1. Restore the upstream `[[test]]` tables (`any_uri`, `certified_numerical`,
   `exact_governance`, `exact_wasm_determinism`, plus `frozen_vectors` if
   rehosted) to `crates/stardf-xsd/Cargo.toml`, and move the files back to
   `tests/*.rs` (`git -C /Users/sac/purrdf show 0da05783:crates/xsd/tests/<f>`
   for byte-verbatim restore; this directory keeps the upstream text).
2. The testkit substrate is vendored (stardf-testkit, workspace member) — but
   `exact_wasm_determinism` also expects the upstream wasm test-runner script
   (upstream `scripts/wasm-test-runner.sh`), not carried by this workspace.
3. `data_range.rs` and `range_storage.rs` need only hash/lex/testkit surfaces
   (no harness=false table); they may unpark individually after (1) for those
   targets or as plain auto-discovered tests.

## Upstream-defect falsifiers

None registered. The 12 failing LIB unit tests (allocation-boundary asserts)
are NOT parked here: they fail identically in upstream purrdf-xsd at rev
0da05783 on this machine (macOS 26.2 arm64, floating nightly) — see
VENDOR-PROVENANCE.md, "Unit-test parity court" for the court receipt. They
are upstream-environment failures, registered for the coordinator's
divergence ledger, not for parking.
