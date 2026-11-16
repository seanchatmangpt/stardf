# PROVENANCE — stardf-branchless

## Source

Vendored from PurRDF at rev `0da0578306983da851a9073337d57e2e2c41ab3a`
(checked out at /Users/sac/purrdf), path `crates/branchless`, per
stardf/docs/jira/v26.10.31/08-vendoring-plan.md. All bytes arrived from the
pinned rev via `git archive`, never the dirty tree.

## Upstream identity receipt (G4)

sha256 of `git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/branchless`:

```
e1757740f692377c024d463c159dbe35b643a8db5e950ca84be38589e5522e21
```

## Rename scope (declared, and only these were touched)

- package name: purrdf-branchless to stardf-branchless
- dep key: purrdf-hash to stardf-hash, path dep on ../stardf-hash
- dep key purrdf-testkit: deferred, see parked bench below

## Lib name verbatim

The lib name is kept verbatim: `[lib] name = "purrdf_branchless"`. Every
vendored source file, every doctest (`use purrdf_branchless` in module docs),
and `tests/integration.rs` (`use purrdf_branchless as b`) resolve through the
upstream lib name with zero source edits. This crate therefore REQUIRES the
stardf-hash vendor to keep its upstream lib name `purrdf_hash`; see
Divergences item 3.

## Divergences (all disclosed; none are source edits)

1. Manifest materialization (manifest-only, no source falsifier): upstream
   inherits version/edition/rust-version/license/repository/homepage and
   `[lints] workspace = true` from the purrdf workspace root; the
   stardf/arrow workspace defines none of those keys. Materialized per the
   Lane-9 vendor precedent (crates/stardf-columnar/Cargo.toml): version
   26.9.29, edition 2021, rust-version 1.88, tri-license verbatim,
   ggen-legacy repository. Upstream `homepage` dropped (Blackcat URL, not
   stardf's to claim). `[lints] workspace = true` dropped (the arrow
   workspace defines no [workspace.lints] table); the in-source
   `#![warn(missing_docs)]` (byte-preserved in src/lib.rs) carries the doc
   lint. MSRV: the 1.88 floor follows the Lane-9 precedent, not upstream
   1.98; it is precedent-carried, not independently verified against a 1.88
   toolchain (UNVERIFIED, disclosed). MSRV unification is a separate W1 work
   item (P19-B5), another lane.
2. Bench parked (disclosed-park pattern, mirrors
   crates/stardf-columnar/tests/disabled/): `benches/kernel_table.rs` moved
   to `benches/disabled/kernel_table.rs-pending-stardf-substrate`
   (byte-identical, sha256
   `ab007502519adaa3dea996c4bac7519b804892590de23a92b6d0ee50d8b55389`).
   It is the sole consumer of `purrdf_testkit::bench` (plus
   `purrdf_hash::mix::lcg64_next`), and stardf-testkit does not exist in this
   workspace yet; the `[[bench]]` section is withheld with it. Unpark
   condition: stardf-testkit landing in stardf/arrow/crates/ exposing the
   `purrdf_testkit::bench` surface, then restore `[[bench]]` and add the
   `stardf-testkit`
   dev-dep (renamed key, path ../stardf-testkit).
3. stardf-hash lib-name contract (cross-lane, load-bearing): the sources use
   `purrdf_hash::` paths byte-preserved, so the stardf-hash vendor must
   declare `[lib] name = "purrdf_hash"`. If the hash lane lands the lib as
   `stardf_hash`, `use purrdf_hash::...` fails E0432, the vendoring plan's
   falsifier fires, and this receipt escalates rather than fixes forward.
4. stardf-testkit lib-name contract (cross-lane, unpark precondition): the
   parked bench imports `purrdf_testkit::bench`, so the future
   stardf-testkit vendor must likewise keep lib name `purrdf_testkit`.
5. oracles/ + vectors/ carried verbatim and unpinned: cargo never builds
   them. They are the Python reference-oracle suite and its generated JSON
   fixtures (provenance text; `grep -rn "vectors/" src tests` = no hits, so
   nothing in the Rust test surface reads them at runtime). The Rust courts
   are self-contained deterministic xorshift-driven tests; the vectors exist
   so the Python oracles can independently re-derive the same fixtures.

## Verification receipts (this session, this lane)

- G3 byte-compare self-check: PASS. Per-file cmp of every
  `git ls-tree -r --name-only 0da05783 -- crates/branchless` entry against
  the vendored tree; only declared surfaces differ; EXTRA-file sweep over
  src/ tests/ oracles/ found none.
- G4 upstream sha256: recorded above from this session's actual output.

## Gate receipts (2026-10-31, this lane, this session)

- cargo check -p stardf-branchless: exit 0 (`Finished dev profile ... in
  0.18s`; stardf-hash dep cached from the hash lane's concurrent runs).
- cargo test -p stardf-branchless: exit 0 — 202 unit + 13 integration
  (tests/integration.rs) + 49 doctests = 264 passed, 0 failed. The doctest
  pass is itself the lib-name-verbatim proof: every `use purrdf_branchless`
  doctest resolves through lib `purrdf_branchless`, and through it
  `purrdf_hash::` resolves via the hash vendor's verbatim lib name.
- cargo metadata --no-deps: exit 0 with this crate in root members (all
  member manifests parse and the workspace resolves).
- Cargo.lock: contains the stardf-branchless entry (resolve receipt).
- stardf-hash lib name witnessed verbatim: `purrdf_hash`
  (crates/stardf-hash/Cargo.toml [lib] name, read this session).
- Bench unpark NOT executed: stardf-testkit landed mid-vendor this session
  (package still `purrdf-testkit`, workspace-inheritance keys unresolvable
  against the arrow root as read at 2026-10-31 ~15:05); the parked-bench
  conditions in benches/disabled/README.md are not yet met, so the park
  stands (loud, per divergence 2).
