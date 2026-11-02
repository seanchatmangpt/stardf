# VENDOR-PROVENANCE — stardf-shapes (copy-sweep lane w2-l03, 2026-10-31)

- Subject: purrdf rev `0da0578306983da851a9073337d57e2e2c41ab3a`
  `crates/shapes` -> `stardf/arrow/crates/stardf-shapes`, via `git -C /Users/sac/purrdf
  archive 0da05783... crates/shapes` (never the dirty tree). Rename scope:
  package-level only; lib name `purrdf_shapes` pinned VERBATIM in [lib].
- Pre-rename archive identity: `git -C /Users/sac/purrdf archive 0da05783... crates/shapes | shasum -a 256`
  = `21bd268a39633529b8602d5dcbf47bbb5ca7add8a4d216f674167dcd4cfb07ec`.
- Scale: 497 files; src/ = 142 files / 155,167 lines; corpus/ = 75 SHACL
  conformance cases (data.nt + shapes.ttl + expected-report.nt each);
  spec/ = 6 W3C SHACL spec TTLs; licenses/ byte-identical to upstream.
- Byte parity: `diff -rq` vs a fresh `git archive` extraction of the same rev
  reports ONLY `Cargo.toml` differs; 497/497 file count parity; `licenses/` and
  `spec/` byte-identical (LICENSE-APACHE/LICENSE-MIT/LICENSE-MULAN/NOTICE.txt/
  inventory.json/CC-BY-4.0.txt/third-party/ carried verbatim).
- Manifest rename-delta (the ONLY file that differs from upstream):
  [package] name; workspace-inherited scalars inlined (version 26.9.29 stardf
  workspace convention, edition 2024, license verbatim, repository ggen-legacy,
  rust-version 1.98); homepage + documentation dropped (stardf-sparql-algebra
  precedent); 17 first-party dep keys purrdf-* -> stardf-* as path deps;
  regex/rayon/syn/proc-macro2 RHS inlined from the purrdf root workspace pin;
  [lib] name pinned; [lints] workspace = true carried as verbatim comment block
  (copied from the green stardf-sparql-algebra manifest).
- Falsifier status: NO non-rename source edit was required. The rename-scope
  claim of the vendoring plan (08-vendoring-plan.md) HOLDS for this crate at
  this rev. (Disclosed process note: two earlier freehand generations of this
  manifest corrupted in transit and were deleted unreferenced; the landed
  manifest was produced by a scripted transform of the pristine upstream file,
  diff-verified hunk by hunk.)
- Standing: PARKED. The crate is intentionally NOT a stardf/arrow workspace
  member yet: 8 of 17 declared first-party path deps do not exist in
  crates/ yet, and adding the member first would break workspace-wide resolve
  (shared-seam compile freeze for every lane).
