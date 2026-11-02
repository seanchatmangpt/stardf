# VENDOR-PROVENANCE.md — stardf-deflate

<!-- SPDX-FileCopyrightText: 2026 Sean Chatman -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

## Identity

- Upstream: `/Users/sac/purrdf` `crates/deflate` at git rev
  `0da0578306983da851a9073337d57e2e2c41ab3a`.
- Materialized via `git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/deflate`
  (never the dirty purrdf working tree).
- Upstream archive sha256 (the pre-rename upstream identity, per
  08-vendoring-plan "Per-crate source-SHA receipt"):
  `de89f76fecb6165c7c91a306f9e6a7168fd711571af5b951b80e880a72da0b9e`
- Destination: `/Users/sac/ggen-legacy/stardf/arrow/crates/stardf-deflate` —
  directory renamed with the package (crates/deflate → crates/stardf-deflate).
- Copy-and-rename extraction; package-level rename scope only.

## Rename surfaces (the ONLY byte deltas to upstream content)

1. `Cargo.toml` `[package] name`: `purrdf-deflate` → `stardf-deflate`.
2. `Cargo.toml` `[dependencies]` key: `purdff-hash = { workspace = true }` →
   `stardf-hash = { path = "../stardf-hash" }` (first-party dep key rename;
   path form per the stardf-columnar first-party dep precedent; stardf-hash is
   vendored from the same purrdf rev, package renamed, lib name `purrdf_hash`
   verbatim).
3. Directory rename `crates/deflate/` → `crates/stardf-deflate/`.

NOTHING else in the carried files differs from the rev. In particular:

- lib name `purrdf_deflate` stays verbatim (law); consumers
  `use purrdf_deflate::...` in the stardf tree. (The two doc-comment
  mentions of the package spelling `purrdf-deflate` inside src/ doc comments
  stay verbatim.)
- Module tree, symbols, in-file SPDX headers (including `®`), README.md,
  crate-level PROVENANCE.md (upstream's own first-party provenance receipt,
  retained verbatim), `licenses/` (all 6 files), `tests/vectors/` — verbatim.
- Workspace-inherited manifest fields are INLINED (not renamed content):
  version (26.9.29 = stardf/arrow workspace version), edition (2024,
  upstream-true), repository (stardf posture), license (verbatim),
  rust-version (1.98, upstream-true), lints tables (upstream
  `[workspace.lints]` inlined verbatim; the stardf/arrow root has no
  `[workspace.lints]`). Upstream `homepage` and `documentation` fields are
  omitted (stardf/arrow is not a publishing boundary; same posture as the
  stardf-hash landing).

## Parked targets (layout, not bytes)

Three upstream targets require first-party purrdf substrate crates that are
not in this lane's vendoring scope and do not exist in the stardf/arrow
workspace. They are parked (never deleted), byte-verbatim, in subdirectories
cargo auto-discovery does not scan:

| upstream path | parked at | blocker |
|---|---|---|
| `tests/deflate_conformance.rs` | `tests/disabled/deflate_conformance.rs` | dev-dep `purrdf-testkit` (`harness = false` runner) |
| `tests/allocation_budget.rs` | `tests/disabled/allocation_budget.rs` | dev-dep `purrdf-alloc-probe` |
| `benches/gzip.rs` | `benches/disabled/gzip.rs` | dev-dep `purrdf-testkit` (harness=false bench runner) |

Unpark preconditions per `tests/disabled/README.md` /
`benches/disabled/README.md`. Unpark is the testkit-slice lane's work item.

## Gates (this lane's receipts)

| gate | command | result |
|---|---|---|
| G2-ladder | `cargo check -p stardf-hash` (root builds alone) | exit 0 |
| G2 | `cargo check -p stardf-deflate` | exit 0 |
| G2-tests | `cargo test -p stardf-deflate` | (recorded below) |
| G3 | byte-compare self-check vs `git show 0da05783…:crates/deflate/<path>` | pass — all carried files byte-identical outside the 3 declared rename surfaces |
| G4 | upstream archive sha256 | `de89f76fecb6165c7c91a306f9e6a7168fd711571af5b951b80e880a72da0b9e` (matches receipt above) |

## Nonclaims

- The upstream suite is NOT executed: the round-trip/refusal/differential
  conformance cases run on purrdf-testkit's `harness = false` runner, which is
  not vendored here; they are parked loud. `cargo test -p stardf-deflate`
  therefore runs the crate's own inline `#[cfg(test)]` modules only.
- `layers.toml` / `check-layers.py` are a plan-level (08-vendoring-plan)
  workspace gate not yet landed in stardf/arrow; this lane adds no layers
  state (UNKNOWN at workspace level, not silently assumed).
- Registration in the root members list is a disclosed shared-seam edit, not
  this crate's file.
