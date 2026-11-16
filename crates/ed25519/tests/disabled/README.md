<!-- SPDX-FileCopyrightText: 2026 Sean Chatman -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Parked test targets — stardf-ed25519

Parked by vendoring lane w2-l10 (2026-10-31), stardf-lex/xsd/deflate posture.

## wycheproof.rs

- Parked target: `tests/wycheproof.rs` (upstream auto-discovered `[[test]]`,
  byte-verbatim at `tests/disabled/wycheproof.rs`, sha256
  `330efcd534f308b25a60d29c882b651a2fef97ef0d3c1adfe398703ed8a98c5b`,
  byte-identical to
  `git show 0da0578306983da851a9073337d57e2e2c41ab3a:crates/ed25519/tests/wycheproof.rs`).
- Reason: repo-layout coupled. It resolves
  `purrdf_testkit::paths::workspace_root().join("vectors/wycheproof/ed25519_test.json")`
  — the upstream purrdf WORKSPACE-ROOT corpus
  (`purrdf@0da05783 vectors/wycheproof/`: `LICENSE`, `PROVENANCE.md`,
  `ed25519_test.json`, 153,600 tar bytes total), which lives outside the
  vendored crate and outside lane w2-l10's crate-slice lease (the
  `stardf/arrow/vectors/` workspace-root seam is not in this lane's slice).
- [SKIP: repo-layout coupled — corpus at upstream workspace root not in lane
  lease; unpark = copy purrdf rev 0da05783 `vectors/wycheproof/` (3 files)
  to `stardf/arrow/vectors/wycheproof/`, then restore
  `tests/wycheproof.rs` from `tests/disabled/` byte-verbatim]
- Unpark precondition, restated: the moved-in corpus is Apache-2.0
  (Wycheproof Project, verbatim LICENSE travels with it); register the
  corpus in `authority/stardf/license-inventory.toml` per the compound-license
  notice law before the workspace ladder claims it.
