# VENDOR-PROVENANCE — stardf-n3

Vendoring receipt for `crates/stardf-n3` (copy sweep slice 8/20, repair lane
w2-l08, 2026-10-31).

## Source

- Upstream: `/Users/sac/purrdf`, pinned rev `0da0578306983da851a9073337d57e2e2c41ab3a`.
- Materialized via `git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/n3`
  — the upstream working tree was DIRTY at vendor time (Cargo.lock, Cargo.toml,
  crates/process/tests/process_courts.rs); the rev was used, never the tree.
- Destination: `stardf/arrow/crates/stardf-n3/`. NOT a member of the stardf/arrow
  workspace (compile-freeze law: membership before stardf-rdf lands would break
  every concurrent lane's cargo command; membership is the integration decision
  recorded below).

## Upstream identity receipt (git archive | shasum -a 256)

    6f01513390ed71b65808037da8612c17784d2be7ba19e16550f7fbd14af3b6e2

## Retention (proven per-file, 2026-10-31)

- Census parity: 27/27 files match the rev's tree (paths equal, sorted diff empty).
- Byte parity: all 26 non-manifest files `cmp`-identical to
  `git show REV:crates/n3/<file>` (0 mismatches, re-run twice: pre- and
  post-manifest-edit).
- Lib name `purrdf_n3` VERBATIM (upstream declared none — implicit name pinned
  explicitly in `[lib] name`, stardf-testkit precedent).
- SPDX headers + `licenses/` (LICENSE-APACHE/LICENSE-MIT/LICENSE-MULAN/NOTICE.txt/
  CC-BY-4.0.txt/inventory.json) retained byte-identical within the same 27-file
  parity set.
- Upstream observation, carried not fixed (a non-rename edit is forbidden):
  11 of 14 `src/` files have NO `SPDX-License-Identifier` header at the pinned
  rev (ast.rs, proof_check.rs, lexer.rs, error.rs, sudoku.rs, parser.rs,
  proof_check_n3.rs, printing.rs, proof_writer.rs, rdf_compat.rs, reasoner.rs).
  Upstream-at-rev state, not session-introduced; campaign W1's "SPDX headers on
  all stardf-authored files" does not reach vendored files under the
  byte-verbatim law.
- Manifest delta surface (Cargo.toml only — full diff is the receipt): package
  name purrdf-n3 -> stardf-n3; workspace-inherited scalars inlined (version
  26.9.29, edition 2024, license tri-license, repository -> ggen-legacy,
  rust-version 1.98; homepage/documentation dropped per the stardf-cdt manifest
  precedent); `[lib] name = "purrdf_n3"` pinned; first-party dep keys renamed to
  path deps (stardf-rdf, stardf-iri, stardf-lex, stardf-hash,
  stardf-branchless; dev: stardf-testkit, stardf-alloc-probe); external pin RHS
  values inlined from the upstream root (hashbrown 0.17 default-features=false,
  num-bigint 0.4, num-traits 0.2, regex 1 + upstream feature list, wasm-bindgen
  =0.2.129 optional); `[lints] workspace = true` carried as the commented
  verbatim inline (stardf-cdt posture — the arrow workspace declares no
  [workspace.lints]); standalone `[workspace]` block appended (stardf-rdf-core-slice
  precedent). LOC receipt: src/ = 20,035 lines (the task's "20K").

## Build state (commands this lane ran, 2026-10-31)

- `cargo metadata --no-deps` from crates/stardf-n3 → exit 101: `failed to load
  manifest for dependency stardf-rdf` / `failed to read
  .../crates/stardf-rdf/Cargo.toml` / No such file or directory. The manifest
  PARSES (failure is at dependency resolution, not syntax); the SOLE blocker is
  the not-yet-vendored stardf-rdf sibling (purrdf-rdf → stardf-rdf). BLOCKED on
  the stardf-rdf copy sweep lane; escalation per the vendoring law's
  missing-dependency path (n3's runtime dep closure includes purrdf-rdf, whose
  stardf twin is outside this lane's lease).
- `cargo metadata --no-deps` from stardf/arrow → exit 0, 18 members, stardf-n3
  absent — the shared compile ladder is NOT frozen by this lane.

## Integration decision (for the coordinator)

stardf-n3 joins the stardf/arrow workspace members list exactly when
`crates/stardf-rdf` lands (same rev, copy-and-rename), and not before; at that
point `cargo check -p stardf-n3` + the four custom-harness upstream test targets
(n3_parse, n3_reason, n3_proofs, n3_forgery) become the crate's gates. Until
then the crate is standalone-manifested and invisible to the workspace ladder.

## License register disclosures (authority/stardf/license-compat.json)

- Three NEW external deps introduced by this crate, observed from
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` on 2026-10-31 (all
  MIT OR Apache-2.0, gated class under R1, Apache/MIT-arm basis under the
  operator blanket sign-off 2026-10-31): hashbrown 0.17.1, num-bigint 0.4.8,
  num-traits 0.2.19 — dependency_licenses + compatibility_receipts rows added
  (consumer stardf-n3; receipt rationale records that attribution is the crate
  manifest, not a Cargo.lock edge, because no lock edge exists pre-membership).
- regex 1.13.1 + wasm-bindgen 0.2.129 reuse the EXISTING register rows
  (sparql-algebra-lane observations); no new dependency_licenses rows minted.
- Reformat disclosure: this lane's register edit re-serialized the whole JSON
  (indent-2 + literal UTF-8 instead of \uXXXX escapes); semantic preservation
  verified (all HEAD dependency names and receipts present post-edit; no entry
  removed).

## Nonclaims

- UNVERIFIED: `cargo check`/`cargo test` on stardf-n3 — cannot run until
  stardf-rdf lands; the crate's build standing is BLOCKED, not ALIVE.
- UNVERIFIED: the lock-resolved versions of hashbrown/num-bigint/num-traits
  (observed at the highest local-registry versions; the lock lands with the
  build gate).
- UNVERIFIED: whether stardf-n3 compiles even once stardf-rdf lands — no
  compile receipt exists for the vendored tree.
- Carried-not-fixed: 11/14 src files missing SPDX headers (upstream-at-rev);
  fixing is a non-rename source edit, forbidden to this lane.
- Out of lease: QuadKey u32-sentinel bridge edge (kernel lanes); not located in
  this slice — stardf-n3 touches no probe plan.
- Out of lease: workspace membership, Cargo.lock, DIVERGENCES rows.
