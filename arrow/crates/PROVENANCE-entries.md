# PROVENANCE-entries — stardf/arrow/crates

Per-crate git-archive sha256 receipts + rename-only attestation for the crates
landed in the stardf v26.11.1 vendoring wave. Written by the provenance lane,
2026-10-31 15:05–15:14 PDT, on `fix/v26.9.29-ci-gates`.

## Method

- Source of truth: `git -C /Users/sac/purrdf archive
  0da0578306983da851a9073337d57e2e2c41ab3a <path>` — the pinned rev, never the
  dirty tree (dirty-tree disclosure at snapshot time: `Cargo.lock`,
  `Cargo.toml`, `crates/process/tests/process_courts.rs` — none of these paths
  are receipted here).
- Per-crate archive receipt = `git -C /Users/sac/purrdf archive <rev> <path> |
  shasum -a 256`. Deterministic for (git version, rev, path); cross-validated:
  this lane's independent recomputation of `crates/hash` reproduces the exact
  digest already recorded in `crates/stardf-hash/VENDOR-PROVENANCE.md`
  (`fb263089…72d8`), and `crates/branchless` reproduces the G4 receipt in
  `crates/stardf-branchless/PROVENANCE.md` (`e1757740…2e21`).
- Attestation method: rev content extracted to a fresh `mktemp -d` per pass,
  `diff -rq` vs the vendored tree, every delta classified against the crate's
  own declared surface (its VENDOR-PROVENANCE.md / PROVENANCE.md), parked and
  moved files `cmp`-verified byte-identical.
- Temp extraction roots were deleted after each pass (nothing left under
  /tmp/stardf-prov*).

## Snapshot status (read this first)

This is a concurrent wave: another lane re-vendored the 2026-10-29 crate batch
while this file was being computed (observed 15:07:20–15:12:22 PDT: `Cargo.toml`
deleted from five crates mid-scan, `stardf-rdf-core-slice/` gained `Cargo.toml`
+ `vendor-crates/` + `model/` during the scan). Every entry carries its as-of
time. The IN-FLIGHT entries carry no attestation and MUST be re-run after that
lane lands.

## Upstream archive receipts (rev `0da0578306983da851a9073337d57e2e2c41ab3a`)

| purrdf path | stardf crate | git-archive sha256 |
|---|---|---|
| crates/alloc-probe | stardf-alloc-probe | `85d17d47abc5bd391c302f84f1ee92c9c8e673356c8b1acc1888b77706a0877a` |
| crates/branchless | stardf-branchless | `e1757740f692377c024d463c159dbe35b643a8db5e950ca84be38589e5522e21` |
| crates/cdt | stardf-cdt | `1d9d856ea92d50ae1b979b4301d026fed0a742cc58306efde723aa6da2dd47be` |
| crates/columnar | stardf-columnar | `2bd3c97daa6c2fc54a42664bb8f3e7c98cb6c303d4e01133c33bfa82db743385` |
| crates/deflate | stardf-deflate | `de89f76fecb6165c7c91a306f9e6a7168fd711571af5b951b80e880a72da0b9e` |
| crates/hash | stardf-hash | `fb26308955e03fc648d21932d4204cefac1a12bba19866ac6b92009bd75972d8` |
| crates/iri | stardf-iri | `dea25f9644baa0dbb8bf752701bbce3cdd6ca3b810850c15d3f7800a55d9fd58` |
| crates/lex | stardf-lex | `f31e9f6bb3e64e6efacdbc5179968081722576bcc0bbeaaebab29cb0508b44d9` |
| crates/rdf-events | stardf-events | `1026a375cb9031be1ecacf4a93bbca75914213728c1a70648b1bc3bd7617b915` |
| crates/slice | stardf-rdf-core-slice | `9b824d65d3f27c2b9401071265db609712447f1c869bd95ea9742b2489ccff81` |
| crates/sparql-algebra | stardf-sparql-algebra | `54c4fc286d5bf69c73b418adca07de344ab406d74620533ea8390e068ba1a11e` |
| crates/stack | stardf-stack | `3f3e9eea4ec4df42705df39233203c354b70c2c5e3f27b54e721304ea5a51967` |
| crates/testkit | stardf-testkit | `1f720713fd609e478928184bb14e70be044ab734b3ca38b80cf3f8d4ad781bdf` |
| crates/xsd | stardf-xsd | `4a58fda637a95200233fac03a746cf24a21925b5a4db50286987048478e9e015` |
| crates/arrow | (no stardf vendor) | `4ec711b1f7932c2dd302eb0ad3d7b71ad66dbadd6e316323e0b6899637a9e782` |
| crates/rdf-capi | (no stardf vendor) | `e3f925e61b47443e2ba2feb89c9a137b223fa6b7e17eb91285cb54ff53708b72` |

## Rename-only attestation

Legend: RENAME-ONLY = every delta vs rev is (a) the sanctioned
`purrdf-* -> stardf-*` package rename, (b) declared in the crate's own
receipt, or (c) a byte-identical park/move. Lib names were checked per crate
against the upstream `[lib] name` (or the upstream default lib target when no
`[lib]` section exists).

| stardf crate | as-of (PDT) | verdict | key evidence |
|---|---|---|---|
| stardf-hash | 15:12:49 | RENAME-ONLY (attested) | src deltas 0; licenses/ `diff -r` exit 0; parked tests byte-identical (`cmp` exit 0 x2); lib `purrdf_hash` verbatim |
| stardf-deflate | 15:12:49 | RENAME-ONLY (attested) | src deltas 0; licenses/ `diff -r` exit 0; parked tests+bench byte-identical (`cmp` exit 0 x3); lib `purrdf_deflate` verbatim |
| stardf-branchless | 15:12:49 | RENAME-ONLY (attested) | src deltas 0; upstream carries no licenses/ dir (N/A, not a divergence); parked bench byte-identical under declared name `benches/disabled/kernel_table.rs-pending-stardf-substrate` (`cmp` exit 0); lib `purrdf_branchless` = upstream default lib target (upstream has no `[lib]` section) |
| stardf-lex | 15:13:32 | RENAMES + DECLARED cfg-gating | src deltas = 11 single-line `#[cfg(test)]` -> `#[cfg(all(test, feature = "upstream-tests"))]` substitutions across 10 files (json_escape.rs has 2 sites, the other 9 files 1 each); zero non-gate edits; README renames-only (declared in VENDOR-PROVENANCE.md "Note A"); licenses/ identical; parked tests/bench/example byte-identical (`cmp` exit 0 x8); lib `purrdf_lex` verbatim |
| stardf-columnar | 15:12:49 | DIVERGENT-BY-DECLARATION | src deltas = declared u32 term-id seam (P0-5) + declared `src/vendor/` shims + feature-gated upstream tests; lib name renamed to `stardf_columnar` (declared in its PROVENANCE.md header) — NOT rename-only; adjudicated in `crates/stardf-columnar/PROVENANCE.md`, not re-attested here |
| stardf-arrow-core, stardf-arrow-capi, stardf-arrow-bench | 15:12:49 | FIRST-PARTY (no vendor receipt applicable) | no purrdf source path; see "Upstream identity of the three first-party crates" below |
| stardf-alloc-probe, stardf-cdt, stardf-events, stardf-iri, stardf-sparql-algebra, stardf-stack, stardf-testkit, stardf-xsd | 15:12:33 | IN-FLIGHT (no attestation) | dir mtimes bumped 15:07:20–15:08:32 by a concurrent re-vendoring lane; `Cargo.toml` deleted from cdt/events/sparql-algebra/testkit/xsd mid-scan (verified absent at 15:08); re-run attestation post-landing |
| stardf-rdf-core-slice | 15:12:22 | IN-FLIGHT (no attestation) | gained `Cargo.toml` 15:07:31, `vendor-crates/` 15:06:52, `model/` mid-scan; target shape not yet fixed; re-run post-landing |

## Wave crates — full classification detail

### stardf-hash (source crates/hash)

- Archive sha256 `fb263089…72d8` (cross-validates the crate's own receipt).
- Full-tree `diff -rq`: only Cargo.toml differs (declared manifest
  materialization), `tests/disabled/{allocation_budget,structured_errors}.rs`
  parked byte-identical, VENDOR-PROVENANCE.md new (receipt file).
- licenses/ (6 files): `diff -r` exit 0.
- Lib `purrdf_hash` verbatim; `use purrdf_hash::…` sites unmodified.

### stardf-deflate (source crates/deflate)

- Archive sha256 `de89f76f…b9e`.
- Full-tree diff: Cargo.toml (declared materialization), VENDOR-PROVENANCE.md
  new, benches/tests moved to `disabled/` byte-identical (`cmp` exit 0 x3).
- licenses/ byte-identical. Lib `purrdf_deflate` verbatim.

### stardf-branchless (source crates/branchless)

- Archive sha256 `e1757740…2e21` (matches its PROVENANCE.md G4 receipt).
- Full-tree diff: Cargo.toml (declared materialization), PROVENANCE.md new,
  bench parked as `benches/disabled/kernel_table.rs-pending-stardf-substrate`
  (`cmp` vs rev `benches/kernel_table.rs` exit 0).
- Upstream crate carries no licenses/ dir — nothing to compare (N/A).
- Lib `purrdf_branchless` = upstream default lib target name — verbatim by
  construction; the crate requires `stardf-hash` to keep lib name
  `purrdf_hash` (its PROVENANCE.md divergence register item 3).

### stardf-lex (source crates/lex)

- Archive sha256 `f31e9f6b…4d9`.
- At the 15:05 sweep src/ was byte-identical (0 deltas). At 15:10:49 the lex
  lane applied the columnar-precedent upstream-tests cfg-gating to 10 src
  files; as of 15:13:32 every src delta is the single-line `#[cfg(test)]` ->
  `#[cfg(all(test, feature = "upstream-tests"))]` substitution (11 sites: 2 in
  json_escape.rs, 1 in each of cbor.rs, crockford.rs, iri_escape.rs, json.rs,
  json_escape.rs, literal_escape.rs, scan.rs, unicode/casing.rs, walk.rs,
  yaml.rs), zero non-gate edits.
- README.md: renames-only (`purrdf-` -> `stardf-` tokens), declared in
  VENDOR-PROVENANCE.md "Note A — README renames".
- licenses/ byte-identical. Parked tests/bench/example byte-identical
  (`cmp` exit 0 x8). Lib `purrdf_lex` verbatim.

### stardf-columnar (source crates/columnar) — NOT rename-only

- Archive sha256 `2bd3c97d…385`. src/ carries the DECLARED u32 term-id seam
  (P0-5) adaptation, the declared `src/vendor/` shims, lane-11
  `tests/seam_courts.rs` (cross-lane, disclosed), `tests/disabled/…
  -pending-stardf-substrate` parking, and the renamed lib `stardf_columnar`.
  Full adjudication lives in `crates/stardf-columnar/PROVENANCE.md`; this file
  records the attestation verdict only.

## Upstream identity of the three first-party crates

`git -C /Users/sac/purrdf show 0da0578306983da851a9073337d57e2e2c41ab3a:crates/arrow/Cargo.toml`
-> package `purrdf-arrow`, description "PurRDF CanonArrow columnar egress
framing (proprietary 128-byte header; not Arrow IPC)".

`git -C /Users/sac/purrdf show 0da0578306983da851a9073337d57e2e2c41ab3a:crates/rdf-capi/Cargo.toml`
-> package `purrdf-capi`, description "PurRDF purrdf C-ABI: libpurrdf
cdylib/staticlib over the purrdf semantic kernel".

Neither matches stardf-arrow-core ("StarDF Arrow core: Quad64 batch -> C Data
Interface export") or stardf-arrow-capi (D9 Tier A C ABI surface)
semantically. stardf-arrow-bench is a workspace-root bench promotion (root
Cargo.toml comment, Lane 7). UNVERIFIED beyond manifests: no byte-level
non-derivation claim is made for the first-party trio.

## Nonclaims

- IN-FLIGHT crates (alloc-probe, cdt, events, iri, sparql-algebra, stack,
  testkit, xsd, rdf-core-slice): UNKNOWN attestation standing; no rename-only
  claim; re-run required post-landing.
- stardf-lex src/ after 15:10:49: renames + DECLARED cfg-gating only — this
  lane has NOT run the lex crate's test gates post-gating; test standing is
  the lex lane's receipt, not this file's.
- stardf-columnar: this file adds no attestation beyond pointing at its
  PROVENANCE.md; the u32-seam adaptation is declared there.
- First-party trio: grounded in manifest identity only; byte-level
  non-derivation is UNVERIFIED.
- All attestations are as-of their table timestamps; any lane landing edits
  after those times invalidates the row until re-run. `stardf-lex` dirmtime
  15:11:20 and `stardf-branchless` 15:07:28 both postdate parts of this
  computation; rows are pinned to explicit diff timestamps, not dir mtimes.
- License-compat register: this lane added no external dependencies, so no
  `dependency_licenses` / `compatibility_receipts` entries were created in
  `authority/stardf/license-compat.json`.
