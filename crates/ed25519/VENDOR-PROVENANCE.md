# VENDOR-PROVENANCE — stardf-ed25519

## Source and law

Vendored from PurRDF `crates/ed25519` at rev
`0da0578306983da851a9073337d57e2e2c41ab3a` (extraction: `git -C /Users/sac/purrdf
archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/ed25519` — never the
dirty tree), per the vendoring law: package rename `purrdf-ed25519` →
`stardf-ed25519` ONLY; lib name `purrdf_ed25519` stays VERBATIM; LICENSE files,
in-file notices, SPDX headers, source, tests, bench, vectors and `licenses/`
travel byte-verbatim. Edition pinned `2024` inline (upstream-equal; root
workspace carries 2021). Upstream `homepage.workspace = true` and
`documentation = "https://docs.rs/purrdf-ed25519"` dropped (disclosed; same
posture as stardf-iri/stardf-lex).

## Copy-sweep disposition (lane w2-l10, 2026-10-31 fix-forward disclosure)

This crate is the one member of the seven-crate copy sweep
(json/glob/markdown/mime/hnsw/retrieval/ed25519) with a COMPLETE in-tree
first-party closure (stardf-hash + external sha2 0.11). The other six are
dep-closure-BLOCKED, not skipped-for-kind: none is wasm-only or binary-only
(all seven CHECKed this lane: rlib-only, no `[[bin]]`, wasm32-clean comments
only) — they wait on the not-yet-vendored purrdf-core (full crate; tree has
only stardf-rdf-core-slice, lib `stardf_rdf_core_slice`, a different lib) /
purrdf-sparql-eval / purrdf-text surfaces. Per-crate blockers:

| crate | runtime deps | missing in tree |
|---|---|---|
| purrdf-json | core, hash, lex | core |
| purrdf-glob | core, hash, iri, lex | core |
| purrdf-markdown | core, hash | core |
| purrdf-mime | core, hash, lex, xsd | core |
| purrdf-hnsw | branchless, core, sparql-eval, xsd, rayon, hash, lex | core, sparql-eval, rayon (external, not in license-compat.json) |
| purrdf-retrieval | core, lex, sparql-eval, sparql-algebra, text, xsd, hash | core, sparql-eval, text |
| purrdf-ed25519 | hash, sha2 | none — VENDORED |

## Byte-identity receipts (all commands this lane, 2026-10-31)

- 25 upstream files (`git archive --list` tar count, 25 non-dir entries) ↔
  25 vendored files + 2 lane-authored files (tests/disabled/README.md,
  VENDOR-PROVENANCE.md).
- Whole-tree byte self-check (per-file sha256 vs
  `git show REV:crates/ed25519/<rel>`): ALL non-rename surfaces
  byte-identical to `0da05783` (self-check loop, exit 0, zero BYTE-DIFF
  lines). Declared adaptation surface = `Cargo.toml` ONLY.
- `tests/disabled/wycheproof.rs` = upstream `tests/wycheproof.rs`
  byte-verbatim, sha256
  `330efcd534f308b25a60d29c882b651a2fef97ef0d3c1adfe398703ed8a98c5b`
  (both sides, `shasum -a 256` after `mv`).
- Upstream crate archive receipt:
  `git -C ~/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/ed25519 | shasum -a 256`
  → `c7310c1388db6f980615ddc039ceb05e4bfb9df4f4da26de01498b1c3b067756`.
- `licenses/` (`LICENSE-APACHE`, `LICENSE-MIT`, `LICENSE-MULAN`,
  `NOTICE.txt`, `CC-BY-4.0.txt`, `inventory.json`) byte-verbatim (covered by
  the whole-tree self-check). All 7 `src/*.rs` carry the Blackcat
  SPDX-FileCopyrightText headers verbatim (grep count 7/7).
- `Cargo.lock` (workspace root, shared seam): mechanical member resolution —
  added `stardf-ed25519 v26.9.29` → `sha2` edge; `sha2 v0.11.1` already
  locked from the registry with checksum.

## Parked target (loud skip, unpark condition recorded)

`tests/wycheproof.rs` (upstream auto-discovered test) is parked byte-verbatim
under `tests/disabled/wycheproof.rs`: it resolves
`purrdf_testkit::paths::workspace_root().join("vectors/wycheproof/ed25519_test.json")`
— the upstream WORKSPACE-ROOT corpus
(`purrdf@0da05783 vectors/wycheproof/`: LICENSE + PROVENANCE.md +
ed25519_test.json, 153,600 tar bytes), outside the crate and outside this
lane's crate-slice lease (`stardf/arrow/vectors/` is a workspace-root seam).
[SKIP: repo-layout coupled — corpus not in lane lease] Unpark = coordinator
copies the 3-file corpus to `stardf/arrow/vectors/wycheproof/` and restores
the test byte-verbatim; register the Apache-2.0 corpus in
`authority/stardf/license-inventory.toml` per the notice law.

## Gate receipts (all commands this lane, 2026-10-31)

- `cargo check -p stardf-ed25519` → exit 0 (sha2 0.11.1 from registry;
  stardf-hash path dep resolves; lib `purrdf_ed25519` compiles, zero edits
  to any `src/` file).
- `cargo test -p stardf-ed25519` → 44 passed / 0 failed: lib
  unittests 23/23 (`strictness_tests` included); edge_cases 13/13;
  reference_vectors 4/4 (2,000 seeded keys/messages vs frozen vectors);
  rfc8032 3/3 (every §7.1 vector reproduces + verifies); doc-test 1/1.
- `cargo check --workspace` → exit 0 (post-landing; lane build root
  `target-w2l10`; baseline exit 0 pre-landing, same root).
- `python3 stardf/arrow/scripts/verify_stardf_licenses.py` → exit 0
  ("license-compat gate: PASS (all dependency edges registered and
  ungated)").

## External deps

`sha2 = { version = "0.11", default-features = false }` — upstream pin
inlined verbatim from purrdf root `[workspace.dependencies]` at `0da05783`;
already registered in `authority/stardf/license-compat.json`
`dependency_licenses` (operator blanket sign-off 2026-10-31). No new
external-dep register entries required by this lane.
