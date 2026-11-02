# VENDOR-PROVENANCE — stardf-datalog

Vendoring receipt for `crates/stardf-datalog`, written 2026-10-31 by
full-parity wave-2 lane w2-l04 (slice: "Copy sweep 4: datalog (64K) →
stardf-datalog"). Companion to the upstream-verbatim `README.md` +
`licenses/` carried inside this crate and to
`stardf/docs/jira/v26.10.31/08-vendoring-plan.md` (the vendoring law).

## Source

- Upstream: `/Users/sac/purrdf` (Blackcat Informatics PurRDF), pinned rev
  `0da0578306983da851a9073337d57e2e2c41ab3a`.
- Materialized from the rev, never the tree (the upstream working tree had
  local edits per the vendoring plan and the stardf-hash receipt):

  ```
  git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/datalog
  ```

- Destination: `stardf/arrow/crates/stardf-datalog/` — the landing spot every
  2026-10-31 vendoring lane used (per the wave lane contract cited in
  `crates/stardf-hash/VENDOR-PROVENANCE.md`: "vendor into
  stardf/arrow/crates/ as new member crates"), superseding the original
  plan's separate-workspace step in
  `stardf/docs/jira/v26.10.31/08-vendoring-plan.md` step 4.
- Size: 86 files; `src/` = 64,172 lines of Rust (`find src -name '*.rs' |
  xargs wc -l`) — the "64K" of the slice label; 2,564 KB on disk.

## Upstream identity receipt

`git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/datalog | shasum -a 256`
→
`cebdb4e16d42952675b4177e8bc650048e9e981dbd1f5bd2aec22fb8d78353d7` (exit 0,
2026-10-31, this lane's run).

## Copy verification (rename-scope claim)

`diff -rq /tmp/w2l04-materialize/crates/datalog stardf/arrow/crates/stardf-datalog`
→ exit 1 with EXACTLY ONE line:
`Files .../Cargo.toml and .../Cargo.toml differ`
— the sole declared surface. All 85 other files (every `src/**/*.rs`, the four
`licenses/` files, `README.md`, benches/examples/tests sources) are
byte-identical to the `git archive` extraction of the pinned rev. (exit 1 is
the expected diff exit for a non-empty diff; the load-bearing fact is the
single-line diff listing.)

## Declared manifest surface (rename + disclosed standalone-root posture)

Per the vendoring law the manifest is rename-only-plus-established-posture;
in-file notices, lib name, LICENSE files, README, and all sources are
verbatim:

- `[package] name`: `purrdf-datalog` → `stardf-datalog` (THE rename).
- `version = "26.9.29"`, `license = "MIT OR Apache-2.0 OR MulanPSL-2.0"`,
  `rust-version = "1.98"` stated explicitly (upstream workspace.package
  values, verbatim); `edition = "2024"` (upstream-equal member-level
  override); `repository` → ggen-legacy; `homepage` dropped (xsd precedent);
  `documentation`, `keywords`, `categories`, `description` verbatim.
- `[lib] name = "purrdf_datalog"` explicit (upstream implicit name, preserved
  verbatim — disclosed, same posture as stardf-testkit's pinned implicit
  name).
- Dependency KEYS renamed where the closure resolves in-tree:
  `purrdf-hash`→`stardf-hash`, `purrdf-branchless`→`stardf-branchless`,
  `purrdf-lex`→`stardf-lex` (path deps, `../stardf-*`, lib names
  `purrdf_*` verbatim). Dev-dep keys KEPT as upstream (`purrdf-testkit`,
  `purrdf-alloc-probe`) with `package =` renames to stardf-testkit /
  stardf-alloc-probe (xsd precedent: "KEY = upstream name, package = stardf-*").
- `hashbrown = { version = "0.17", default-features = false }` and
  `rayon = "1"`: upstream workspace pins restated INLINE (upstream pin text
  verbatim) because this crate is a standalone workspace root that cannot
  inherit upstream's `[workspace.dependencies]`, and the stardf/arrow
  workspace must not gain a hashbrown/rayon pin for a non-member crate
  (single-pin-source law). Registry resolution available offline:
  hashbrown 0.17.1, rayon 1.12.0 + rayon-core 1.13.0 (licenses observed and
  registered in `authority/stardf/license-compat.json`, 2026-10-31).
- `[lints]` inlined from upstream `[workspace.lints]` verbatim (rust table +
  full clippy table incl. opt-outs with their comments) — standalone-root
  posture, identical approach to stardf-xsd's inlined lints.
- Upstream `[[bench]] seminaive` + four `harness = false` `[[test]]` targets
  carried verbatim and UNPARKED: with the purrdf-core edge unresolvable the
  crate cannot compile regardless, so parking targets would be cosmetic
  (contrast the xsd/lex/deflate lanes, whose crates COMPILE and park only the
  targets that need not-yet-vendored dev-deps).

## FALSIFIER RECEIPT — stardf-core closure gap (BLOCKED build)

The vendoring law: a non-rename edit needed to build = falsifier → escalate,
never fix silently. Here the escalation fires BEFORE any source edit would be
needed: the crate's declared upstream dependency closure includes
**purrdf-core**, and **no stardf-core exists in the stardf tree** — the arrow
workspace has `stardf-rdf-core-slice` (a NON-MEMBER standalone crate, lib
name `stardf_rdf_core_slice`, term-model only) but no full-core member
(`cargo metadata` from `stardf/arrow` resolves 54 packages, `stardf-core` not
among them — witnessed this lane, exit 0).

Datalog's purrdf-core surface is WIDE, not narrow — spanning
`FastMap`/`FastSet`/`SmallVec`/`smallvec`, `evidence::{EvidenceId,
EvidenceWorkspaceError, EvidenceCombination, ReasonDescription,
SearchCompleteness, Frame, Information, InformationMap}`, `vocab::rdf::TYPE`,
`datatype::XSD_BOOLEAN`, `bytes::read_u64_le`, `ViewTermId`,
`SnapshotStatistics`, `ProbePattern`, `StatisticCount`,
`hash::hash_table_allocation_bound`, `DatasetView`, `GraphMatch`,
`RdfDatasetBuilder`, `GraphError`, `TermBox`, `TermValue` (re-read from the
vendored sources this session; 102 source-line references across the crate).
This is the full purrdf-core (`crates/rdf-core/src` = 7,420,125 bytes at the
pinned rev), not a seam-sized surface. A hand-written `purrdf_core` seam to
unblock the build would be exactly the mock-laundering the vendoring law
exists to prevent, and renaming the key to a nonexistent `stardf-core` or
dropping the edge would falsify the vendored closure.

Failure witness (this lane's run, 2026-10-31):

```
cd stardf/arrow/crates/stardf-datalog
cargo metadata --offline --format-version 1
→ exit 101:
  error: failed to parse manifest at .../stardf-datalog/Cargo.toml
  Caused by: error inheriting `purrdf-core` from workspace root manifest's
  `workspace.dependencies.purrdf-core`
  Caused by: `workspace.dependencies` was not defined
```

The `purrdf-core = { workspace = true }` edge is carried VERBATIM in
`Cargo.toml` precisely so this failure stays loud at this crate's own
boundary. The edge is the falsifier: it cannot resolve until a full
stardf-core member lands (full-parity campaign W8's "shallow closure (6
leaves…)" assumed stardf-core availability; it is not present).

### Why a standalone (non-member) crate, not an arrow member

- Joining `stardf/arrow/Cargo.toml` members with purrdf-core unresolvable
  would fail workspace-level manifest parsing for EVERY cargo command in the
  shared workspace — a compile freeze for all 20 concurrent lanes.
- Parking or dropping the purrdf-core edge would misstate the closure.
- Precedent: `crates/stardf-rdf-core-slice/Cargo.toml` carries the same
  standalone `[workspace]` root table with "Membership = integration
  decision" and "Joining it would break the shared compile ladder while this
  crate's residual closure is unresolved."
- Consequence: `cargo metadata` from `stardf/arrow` resolves 54 packages and
  stardf-datalog is NOT among them — the shared ladder is UNBROKEN (witnessed
  exit 0, 2026-10-31; arrow `Cargo.lock` mtime unchanged by this lane —
  observed 1793491181 after the runs, and no lock-mutating command was run).

## Standing table

| surface | standing | receipt |
|---|---|---|
| copy (86 files, byte-verbatim outside Cargo.toml) | ALIVE | `diff -rq` single-line listing (this file, Copy verification) + upstream sha256 `cebdb4e1…` |
| manifest reconstruction (rename-surface law) | ALIVE | the Cargo.toml diff content + this disclosure |
| build (`cargo check/test`) | BLOCKED | `cargo metadata` exit 101, `purrdf-core` inheritance failure (this file) |
| arrow-workspace membership | BLOCKED | same closure gap; joining would freeze the shared ladder (rationale above) |
| shared ladder (arrow workspace) | ALIVE | `cargo metadata` from `stardf/arrow` exit 0, 54 packages, no ladder break |
| license register (hashbrown/rayon) | ALIVE | rows added to `authority/stardf/license-compat.json` 2026-10-31 |

## Escalation path (what unblocks this crate)

1. Vendor full purrdf-core → `stardf/arrow/crates/stardf-core` (7.4 MB src at
   the pinned rev — a W2/W8-class ticket, not a lane-local unblock; W8's
   "shallow closure" line named purrdf-core a datalog leaf, so the W8 wave
   owner is the natural authority).
2. Then flip this manifest's `purrdf-core = { workspace = true }` edge to the
   renamed member path (the ONLY further edit this crate needs; sources stay
   byte-verbatim), give it a `[workspace.dependencies]` entry in its
   standalone root or join the arrow workspace with the hashbrown/rayon pins
   moved to `[workspace.dependencies]` per the single-pin law, and run the
   build ladder.
3. Membership decision (joining stardf/arrow) is the coordinator's, per the
   rdf-core-slice precedent.

## Nonclaims

- No `cargo check`/`cargo test` standing is claimed for stardf-datalog: the
  crate has never compiled anywhere in the stardf tree. Build standing is
  BLOCKED, not UNKNOWN.
- No source file was edited: the single-file diff listing above is the whole
  story. No non-rename source edit was needed to establish the BLOCKED state,
  so the falsifier fires at the manifest/dependency-closure layer, not the
  source layer — the rename-scope claim for SOURCES is UNREFUTED, not
  verified-by-build (the build never ran).
- The upstream sha256 receipt is over `git archive` output, which is
  stable for a given (rev, path) but was verified once, at landing time.
- The arrow `Cargo.lock` mtime (1793491181) is an observation, not an
  attribution: this lane ran no lock-mutating command, but concurrent lanes
  share the tree.
- Wave-1 receipts for slice 4/20 were named by the dispatch but DO NOT EXIST
  as on-disk artifacts under `stardf/` (searched; the `stardf/arrow/evidence/`
  set is the earlier arrow campaign's). This lane landed from the slice text
  + the vendoring law alone; defect candidates from a wave-1 receipt, if any
  exist, are NOT incorporated.
- Nothing was committed: per the lane contract, c3–c6 are coordinator-only;
  the landing is working-tree only.
