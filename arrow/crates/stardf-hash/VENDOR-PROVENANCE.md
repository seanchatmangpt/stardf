# VENDOR-PROVENANCE — stardf-hash

Vendoring receipt for `crates/stardf-hash`, written 2026-10-31. Companion
receipt to the upstream-verbatim [`PROVENANCE.md`](PROVENANCE.md) (Blackcat's
per-algorithm provenance record, kept byte-verbatim — it is licensed
provenance material, so the stardf receipt lives in this separate file rather
than editing it).

## Source

- Upstream: `/Users/sac/purrdf` (Blackcat Informatics PurRDF), pinned rev
  `0da0578306983da851a9073337d57e2e2c41ab3a` (verified `git rev-parse HEAD` =
  the pin, 2026-10-31).
- The upstream working tree was DIRTY at vendor time (`Cargo.lock`,
  `Cargo.toml`, `crates/process/tests/process_courts.rs`); per the vendoring
  law the crate was materialized from the rev, never the tree:

  ```
  git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/hash
  ```

- Destination: `stardf/arrow/crates/stardf-hash/` — member of the existing
  stardf/arrow cargo workspace (per the wave lane contract: "vendor into
  stardf/arrow/crates/ as new member crates"), superseding the original
  plan's separate-workspace step in
  `stardf/docs/jira/v26.10.31/08-vendoring-plan.md` step 4.

## Upstream identity receipt

`git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/hash | shasum -a 256`:

```
fb26308955e03fc648d21932d4204cefac1a12bba19866ac6b92009bd75972d8
```

## Retention (proven, not asserted)

- `diff -rq` rev-extraction vs vendored tree: the ONLY differences are the
  declared delta surface below. Every `src/**` file and
  `tests/known_answers.rs` is byte-identical to rev 0da05783.
- `licenses/` 6/6 byte-identical via `cmp` (LICENSE-APACHE, LICENSE-MIT,
  LICENSE-MULAN, NOTICE.txt, CC-BY-4.0.txt, inventory.json).
- SPDX header line 2 (`// SPDX-License-Identifier: …`) byte-compared per file:
  all 36 present files verbatim (the 2 apparent misses were the moved test
  files at their old paths; both `cmp`-verified byte-identical at
  `tests/disabled/`).
- Upstream `PROVENANCE.md` (180 lines): byte-verbatim.
- Lib name `purrdf_hash` (`[lib] name`): VERBATIM per the vendoring law.
  Consumers do `use purrdf_hash::…`. Only the PACKAGE name was renamed.

## Declared delta surface (complete list — nothing else changed)

1. `Cargo.toml` (only file with content edits):
   - `[package] name`: `purrdf-hash` -> `stardf-hash` (sanctioned rename).
   - Upstream `x.workspace = true` lines replaced by inline values (the
     stardf/arrow workspace root has different/no matching
     `[workspace.package]` keys): `version = "26.9.29"` (stardf ladder),
     `edition = "2024"` (upstream-true; purrdf workspace edition is 2024),
     `license = "MIT OR Apache-2.0 OR MulanPSL-2.0"` (verbatim upstream
     grant), `repository` (stardf), `rust-version = "1.98"` (upstream-true).
   - `homepage`/`documentation` keys dropped (upstream values point at
     purrdf's site / docs.rs — foreign/inert under `publish = false`; Lane 9
     precedent on stardf-columnar).
   - `[lints] workspace = true` replaced by the upstream
     `[workspace.lints.rust]` + `[workspace.lints.clippy]` tables inlined
     VERBATIM (the stardf/arrow root carries no `[workspace.lints]`).
   - Dev-dependency `purrdf-alloc-probe = { workspace = true }` dropped
     (disclosed; first-party crate not in this lane's vendoring scope;
     restored when `stardf-alloc-probe` is vendored — unpark note in
     `tests/disabled/README.md`).
2. `tests/structured_errors.rs` and `tests/allocation_budget.rs` MOVED to
   `tests/disabled/` (parked, not deleted; cargo ignores `tests/disabled/`).
   Both `cmp`-verified byte-identical to rev 0da05783. Reason: both set
   `CountingAllocator` as `#[global_allocator]` and import
   `purrdf_alloc_probe`, which does not exist in this workspace. Parked
   rather than edited (an edit would be a source divergence needing its own
   ticket).
3. New files: this receipt, `tests/disabled/README.md` (loud-skip register +
   unpark precondition).
4. Shared-seam touches (disclosed, coordinator-owned files):
   - root `Cargo.toml`: one member entry + disclosure comment for
     `crates/stardf-hash`. No `[workspace.dependencies]` pin added — Lane 9
     precedent: vendored members are consumed by direct path deps, and
     nothing consumes `stardf-hash` yet. First consumer adds
     `stardf-hash = { path = "crates/stardf-hash" }` to the root pin table.
   - root `Cargo.lock`: gained the `stardf-hash` member stanza (automatic
     cargo behavior on member addition; no version changes to any other
     package).
5. NO `[workspace.package]` edits (inline values instead — disjoint file
   ownership respected).

## Falsifier check (the one gate-time decision, disclosed)

First gate run FAILED: `edition = "2021"` (inlined from the in-tree Lane 9
precedent) broke on the source's 2024 let-chains (`src/blake3.rs:1221`).
Resolution: `edition = "2024"` is the UPSTREAM-TRUE value (purrdf
`[workspace.package] edition = "2024"`); restoring it is a manifest-VALUE
correction toward upstream fidelity, not a source edit and not a divergence.
Zero source edits made; the vendoring falsifier ("a non-rename SOURCE edit
needed to build = escalate") never tripped — no source file needed any edit.

## Gates (this session, real runs)

| gate | command | result |
|---|---|---|
| check | `cargo check -p stardf-hash` | exit 0 |
| all-targets | `cargo check -p stardf-hash --all-targets` | exit 0 |
| tests | `cargo test -p stardf-hash` | exit 0 — 107 passed, 0 failed (64 lib + 10 + 28 integration KATs + 5 doctests incl. 4 compile-fail doctests) |
| retention | `diff -rq` rev vs vendored | deltas = declared surface only |
| retention | `cmp` licenses 6/6 | byte-identical |
| retention | SPDX line-2 per-file compare | all verbatim |

rustc: 1.101.0-nightly (2026-10-24), satisfies `rust-version = "1.98"`.

## Unpark precondition (loud skip, not silent)

`tests/disabled/structured_errors.rs` and `tests/disabled/allocation_budget.rs`
require `purrdf_alloc_probe`. Unpark when `crates/alloc-probe` is vendored as
`stardf-alloc-probe`: restore the dev-dep key
(`stardf-alloc-probe = { workspace = true }` + root pin), move both files back
into `tests/`, `cargo test -p stardf-hash`. Until then they are absent from
the run — a loud skip registered in `tests/disabled/README.md`.

## Nonclaims

- UNVERIFIED: behavior under `--all-features` (the crate declares no features;
  `[package.metadata.docs.rs]` all-features is inert here).
- UNVERIFIED: clippy posture — `cargo clippy` not run by this lane (the
  inlined upstream clippy tables are warn-level; no clippy gate ran).
- UNVERIFIED: wasm32 target (`cargo check --target wasm32-unknown-unknown`)
  not run by this lane; upstream claims wasm-cleanliness, this receipt claims
  only the host-target gates above.
- UNKNOWN: whether `layers.toml` / `check-layers.py` gating (plan gates G1,
  G1b) applies to this single-crate lane — the plan's nine-crate ladder was
  not executed here; this lane vendored crate #1 (`stardf-hash`) only.
- Out of scope, recorded: `stardf-alloc-probe`, `stardf-hash-conformance`
  (upstream `purrdf-hash-conformance` frozen-vector suites — the deep KAT
  court), and the remaining seven tier crates.
- License register: `stardf-hash` adds ZERO new external (non-first-party)
  dependencies, so nothing was added to
  `authority/stardf/license-compat.json` (the file does not exist yet; first
  lane adding an external dep creates it).
