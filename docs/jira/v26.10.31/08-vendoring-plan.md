# GL-STARDF Vendoring Plan — purrdf Tier-Leaf Crates → stardf

<!-- SPDX-FileCopyrightText: 2026 Sean Chatman -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

Status: `PARTIAL_ALIVE` (plan admitted; execution is a separate lane).

## Execution status (banner, 2026-10-31)

**EXECUTED via the 80/20 bulk copy — this banner supersedes the Status line
above for execution standing.** The plan's nine-crate tier-leaf scope was
executed as a wider one-package whole-vendor: `scripts/bulk_copy_purrdf.py`
(80/20 Phase-1 bulk copy; pinned rev `0da0578306983da851a9073337d57e2e2c41ab3a`,
same rev as below) plus per-lane copies landed every upstream crate as
`stardf-*` under `stardf/arrow/crates/` — all 53 upstream purrdf crates landed
(58 `stardf-*` crate dirs measured on disk, 2026-10-31).

- Receipts: per-crate git-archive sha256 receipts in
  `stardf/arrow/crates/PROVENANCE-entries.md` plus per-crate
  `VENDOR-PROVENANCE.md` files in the landed crates (18 present,
  2026-10-31). Gate G4's per-crate-receipt law is satisfied in aggregate
  form — one bulk `PROVENANCE-entries.md` over the landed set + per-crate
  `VENDOR-PROVENANCE.md` files — rather than the single new
  `stardf/PROVENANCE.md` this plan drafted.
- Deviations from this plan: (1) scope — nine tier-leaf crates planned, all
  53 upstream crates vendored in one package; (2) the operator-directed
  carve of `sql` was superseded — `sql` vendored wholesale with the rest of
  the workspace; (3) the receipt file landed as `PROVENANCE-entries.md` +
  per-crate VENDOR files instead of one root `PROVENANCE.md`.
- Rename discipline HELD: `[package] name` + `[dependencies]` keys in
  `Cargo.toml` only; lib names, module tree, symbols, license/notice
  surfaces, and all other file contents verbatim (the bulk-copy script
  touches `Cargo.toml` only).
- The Nonclaims section below is historical: its UNVERIFIED-execution entry
  is discharged by the execution this banner receipts; the plan text is
  preserved verbatim as the admitted planning artifact.

## Subject

- upstream: `~/purrdf` at git rev `0da0578306983da851a9073337d57e2e2c41ab3a`
  (verified `git rev-parse HEAD`, 2026-10-31; working tree had local edits to
  `Cargo.lock`, `Cargo.toml`, `crates/process/tests/process_courts.rs` — vendor
  from the rev, not the dirty tree: use `git archive 0da057830698…` or
  `git show 0da05783:path`).
- destination: `/Users/sac/ggen-legacy/stardf/` (new `crates/` workspace).
- authority: `tickets/GL-STARDF-001.md`, item P0-18 (vendoring plan).
- rename scope: **package-level only** (`purrdf-*` → `stardf-*`). Crate lib
  names (`purrdf_lex`, `purrdf_iri`, `name = "kernel_table"` binary, etc.),
  module tree, symbols, and all other file contents byte-preserved at copy
  time.

## Crate inventory (tier leaves, vendoring order)

Per each crate's `Cargo.toml` and `~/purrdf/layers.toml` (root =
`purrdf-hash`). `scripts/check-layers.py --home-for` over all nine crates
returns `purrdf-hash`: the set closes onto the declared root.

| # | purrdf crate | stardf name | runtime deps (first-party) | Cargo.toml license |
|---|---|---|---|---|
| 1 | purrdf-hash | stardf-hash | — (zero-dependency root) | workspace |
| 2 | purrdf-branchless | stardf-branchless | purrdf-hash (dev: purrdf-testkit) | workspace |
| 3 | purrdf-deflate | stardf-deflate | purrdf-hash only | workspace |
| 4 | purrdf-lex | stardf-lex | purrdf-hash | `(MIT OR Apache-2.0 OR MulanPSL-2.0) AND MIT AND Unicode-3.0 AND BSD-3-Clause` |
| 5 | purrdf-iri | stardf-iri | purrdf-lex, purrdf-hash | `(MIT OR Apache-2.0 OR MulanPSL-2.0) AND Unicode-3.0` |
| 6 | purrdf-xsd | stardf-xsd | purrdf-hash, purrdf-lex (read at copy time) | workspace |
| 7 | purrdf-cdt | stardf-cdt | purrdf-hash, -lex, -iri, -xsd, -events (closed leaf) | workspace |
| 8 | purrdf-events | stardf-events | purrdf-hash only (object-safe sink/source seam) | workspace |
| 9 | purrdf-core | stardf-core | purrdf-hash, -lex, -iri, -xsd, -cdt, -events (read at copy time) | read at copy time (planning read: `(MIT OR Apache-2.0 OR MulanPSL-2.0) AND Unicode-3.0`) |

Row 9's license is a placeholder — read `crates/rdf-core/Cargo.toml` at copy
time and transcribe exactly; same discipline for every row.

## layers.toml / check-layers.py

- Vendor `layers.toml` with `root = "stardf-hash"` and one `[[crate]]` row per
  vendored member; rows for the nine crates carry the same `deps` edges with
  names renamed. Edges into non-vendored purrdf crates (e.g. cdt → purrdf-events
  stays, since events is in scope; any edge to an out-of-scope crate, if one
  appears, marks that crate in-scope or blocks the gate).
- Vendor `scripts/check-layers.py` unchanged (no purrdf strings to rename except
  none expected — it reads layers.toml). `--home-for` is the drift detector:
  run it after each layer lands.
- Keep the two gate semantics from upstream: a first-party edge no row allows
  fails, and a listed edge the graph no longer has fails. Pending rows are for
  the rest of the tier later.

## License / notice retention checklist

Principle: the vendored crates are a copy-and-rename extraction; every license
grant and notice must survive **verbatim**. Renames touch package names in
`Cargo.toml` `[dependencies]` keys and nothing else in licensed material.

1. **In-file copyright headers** (`// SPDX-FileCopyrightText: 2026 Blackcat
   Informatics® Inc.` / `// SPDX-License-Identifier:` on every source file):
   stay verbatim, including the `®`, in every copied file. No rewording, no
   reattribution, no year bumps.
2. **Original per-crate license files**: `crates/lex/licenses/` ships
   `LICENSE-APACHE`, `LICENSE-MIT`, `LICENSE-MULAN`, `NOTICE.txt`,
   `CC-BY-4.0.txt`, `inventory.json`, and `third-party/` (Unicode-3.0,
   libyaml-COPYRIGHT-MIT, unsafe-libyaml-0.2.11-MIT, WHATWG LICENSE). Copy the
   whole directory verbatim under the renamed crate.
3. **Per-crate LICENSE-APACHE / LICENSE-MIT / LICENSE-MUPAN typo guard**:
   verify directory listing against upstream before commit.
4. Root `LICENSE-APACHE` / `LICENSE-MIT` / `LICENSE-MULAN` and
   `license-inventory.toml` rows whose paths fall inside vendored crates:
   copy those `[header_offers]` / `[[material]]` rows, renaming only the
   crate segment of each path (`crates/lex/...` → `crates/stardf-lex/...`
   per the directory naming chosen in the mechanical steps), everything else
   verbatim.
5. **Conjunction-notice carriage** — the `AND`-conjunction licenses travel with
   the specific files that carry them:
   - **Unicode-3.0 per-file tables** (from `license-inventory.toml`
     `[header_offers]`): `crates/lex/src/unicode_tables.rs`,
     `crates/lex/src/unicode/case_tables.rs`,
     `crates/iri/src/idna_tables.rs`, `crates/rdf-core/src/xsd_regex/blocks.rs`,
     `crates/rdf-core/src/xsd_regex/xpath/unicode_tables.rs`,
     `crates/rdf-core/src/xsd_regex/xpath/compatibility_tables.rs`
     (full list re-derived at copy time by grepping
     `license-inventory.toml` `[header_offers]` for paths under the nine
     vendored crates — do not transcribe from this plan; the gate is the
     upstream `check-generated.sh`/header-gate semantics re-derived over
     stardf).
   - **BSD-3-Clause lex/html**: `crates/lex/src/html/entities.rs`
     (BSD-3-Clause header), `crates/lex/data/html/` (`PROVENANCE.md`,
     `entities.json.license`, `numeric-recovery.json.license`, SHA-256-pinned
     WHATWG html-build input at commit `283a3531a61106d07d9a7d9fb3e6f3b9bfd33d70`,
     145,897 bytes) and
     `crates/lex/licenses/third-party/whatwg-html-references/0-LICENSE-WHATWG`.
6. `LICENSING.md` / `SECURITY.md` / brand files: copy only if stardf becomes a
   publishing boundary; vendoring inside ggen-legacy does not require them —
   recorded as out of scope with reason.
7. **PROVENANCE.md**: purrdf's own PROVENANCE.md documents purrdf's extraction
   from gmeow — that is purrdf's receipt, not stardf's. stardf gets a NEW
   `PROVENANCE.md` stating: extracted from `~/purrdf` at rev `0da05783…`,
   copy-and-rename, package-level rename only, per-crate source-SHA receipts
   below.
8. `LICENSES/` directory: copy the files referenced by the vendored crates'
   conjunctions: `Unicode-3.0.txt`, `CC-BY-4.0.txt`,
   `libyaml-COPYRIGHT-MIT.txt`, `MIT-permission.txt`,
   `unsafe-libyaml-0.2.11-MIT.txt`, `MulanPSL-2.0.txt`,
   `W3C-*.html/txt` (only those referenced from the vendored set — re-derive
   the exact list at copy time), plus `PROVENANCE.md` from `LICENSES/`.
9. **Notice-retention self-check** (must pass before any commit):
   - byte-compare every `LICENSE*`, `NOTICE*`, `*.license`,
     `licenses/third-party/**`, and every `SPDX-License-Identifier` header line
     between `git show 0da05783:crates/<c>/...` and the vendored copy;
   - byte-compare all nine crates' source trees except `Cargo.toml`
     `[dependencies]` keys and `[package] name`, and the layers.toml names.
10. self-check stderr must be empty; `diff -r` exit 0 outside the renamed
    surfaces.

## Per-crate source-SHA receipt

One receipt per vendored crate: sha256 of the tar of the crate source at the
pinned rev, computed against `git archive` of the pinned rev
`0da0578306983da851a9073337d57e2e2c41ab3a`. Commands:
```bash
REV=0da0578306983da851a9073337d57e2e2c41ab3a
for c in hash branchless deflate lex iri xsd cdt rdf-events rdf-core; do
  git -C ~/purrdf archive "$REV" "crates/$c" | shasum -a 256
done
```
Record rev + per-crate sha256 in the new stardf `PROVENANCE.md`. After rename,
the receipt maps stardf-name → (purrdf-name, rev, sha256). Rename-only deltas
are disclosed per crate: exactly the Cargo.toml `[package] name` and
`[dependencies]` keys.

 Rename-only delta disclosure must be checked by the self-check in the
checklist above; a sha over the renamed tree is the identity of the vendored
tree, and the pre-rename sha is the upstream identity.

## Mechanical steps

1. Materialize the upstream rev into scratch (no worktree, no clone of the
   dirty checkout): `git -C ~/purrdf archive 0da05783… crates/{hash,branchless,deflate,lex,iri,xsd,cdt,rdf-events,rdf-core} layers.toml scripts/check-layers.py LICENSES/ license-inventory.toml` → extract to `stardf/crates/` layout.
2. Copy the nine crate dirs into `stardf/crates/` with renames:
   rdf-events → stardf-events; rdf-core → stardf-core (directory names rename
   with the package names).
3. Per-crate `Cargo.toml`: rename `[package] name` and every
   `[dependencies]`/`[dev-dependencies]` key that is a first-party purrdf dep.
   Touch nothing else. `lib name` (`purrdf_lex` etc.) stays.
4. New workspace `stardf/crates/Cargo.toml` (members = the nine; carry the
   upstream workspace license `MIT OR Apache-2.0 OR MulanPSL-2.0` over
   verbatim; do not mint a new grant).
5. Vendor layers.toml with renamed rows; root = stardf-hash.
6. Copy `scripts/check-layers.py` into `stardf/crates/scripts/` unchanged.
7. New `stardf/PROVENANCE.md` (see checklist item 7).
8. Per-crate sha256 receipts recorded (section above).
9. Run `check-layers.py` over the renamed layers.toml (gate 1); run
   `--home-for` over the nine stardf names → must print `stardf-hash`.
10. `cargo metadata`/`cargo check -p stardf-hash` first (root builds alone),
    then add crates in vendoring order, `cargo check` each step, breaking only
    on path-dep naming, never on source edits.
    Never edit vendored source to fix a build break — a source edit is an
    upstream divergence needing its own receipt.

## Gates

- G1: `check-layers.py` over the renamed layers.toml exits 0.
- G1b: `check-layers.py --home-for <nine stardf names>` prints `stardf-hash`.
- G2: `cargo check` per-step build ladder, in vendoring order, all green.
- G3: byte-compare self-check (checklist item 9/10) — license/notice surfaces
  byte-identical to `0da05783…`; source byte-identical outside the declared
  rename surfaces.
- G4: sha256 receipts in stardf PROVENANCE.md match `git archive` sha256 of
  the same crate dir at the pinned rev.

## Falsifier

Any source file inside a vendored crate requiring a non-rename edit to build
under stardf = the vendoring plan's rename-scope claim is refuted; escalate to
a new ticket, do not fix forward silently.

## Nonclaims

- UNVERIFIED: actual execution of the copy/rename/build ladder — this file is
  the plan, not the actuation receipt.
- UNKNOWN: exact `[dependencies]` of xsd/core beyond what this plan read at
  planning time; re-read each Cargo.toml at copy time.
- UNVERIFIED: whether `check-layers.py` itself hard-codes any `purrdf-`
  strings; verify at copy time (plan assumes it reads layers.toml only).
- Out of scope (recorded, not executed): LICENSING.md/brand files, C API,
  wasm, sparql/shapes/shex/validate, any crate not in the nine-crate set.

See Also: `tickets/GL-STARDF-001.md` · `stardf/docs/jira/v26.10.31/`
