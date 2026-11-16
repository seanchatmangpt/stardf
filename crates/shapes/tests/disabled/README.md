<!-- SPDX-FileCopyrightText: 2026 Sean Chatman -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Parked test target — stardf-shapes

Parked by G03 offline-ladder lane (2026-10-31), stardf-ed25519/lex/deflate
parked-targets posture (D17: exclusion over deep-fix for copied-80% failures).

## ontology_schema_surface.rs

- Parked target: `tests/ontology_schema_surface.rs` (upstream auto-discovered
  `[[test]]`, byte-verbatim at `tests/disabled/ontology_schema_surface.rs`,
  sha256 `0e75fa1689e83645cc30e96a3265009f5ec58d644359e4fc347a45ead9e7757c`).
- Reason: repo-layout coupled to the excluded `purrdf-jsonschema` crate —
  same coupling as owl_value_semantics.rs below. The helper
  `accepts_with_namespaces` (line 675) builds a `purrdf_jsonschema::Registry`
  and ~15 `accepts()` call sites across 29 tests weave through it; surgical
  cfg(any()) parking would be deep surgery on an upstream file, so the whole
  target is parked instead (D17 exclusion over deep-fix). 29 tests parked,
  loudly registered.
- [SKIP: repo-layout coupled — dev-dep target excluded from this tree per
  crates/jsonschema/DISPOSITION.md; unpark = vendor purrdf-jsonschema
  (upstream full crate, 383 files at rev 0da05783), then restore
  `tests/ontology_schema_surface.rs` from `tests/disabled/` byte-verbatim]

## owl_value_semantics.rs

- Parked target: `tests/owl_value_semantics.rs` (upstream auto-discovered
  `[[test]]`, byte-verbatim at `tests/disabled/owl_value_semantics.rs`, sha256
  `1a1979bd5c33288aec9a3346e2b07bf04bce75f52d40d2c2deac75bad893fc48`).
- Reason: repo-layout coupled to the excluded `purrdf-jsonschema` crate. Four
  unresolved-crate errors (E0433) at lines 81, 92, 512, 522 —
  `purrdf_jsonschema::Metaschemas::new` / `purrdf_jsonschema::Registry::with_metaschemas`.
  The upstream `purrdf-jsonschema` crate is in the campaign's exclude-loudly
  bucket (`crates/jsonschema/DISPOSITION.md`; carried tree is fixture data
  only, 17 metaschema JSONs, no src/, not a member). Without a dev-dep on a
  crate that does not exist in this tree, the target cannot link.
- Contradiction registered: the repair-lane manifest comment
  (`crates/stardf-shapes/Cargo.toml:123-125`, "no source in this crate imports
  stardf_jsonschema (grep-verified), and the upstream purrdf-jsonschema crate
  is excluded loudly") is FALSE for the test target — the grep scope missed
  `tests/`. Premise rot, same class as the stardf-arrow promotion
  ("dead tooling premise was false"). The dev-dep removal comment stands for
  `src/` only.
- [SKIP: repo-layout coupled — dev-dep target excluded from this tree per
  crates/jsonschema/DISPOSITION.md; unpark = vendor purrdf-jsonschema
  (upstream full crate, 383 files at rev 0da05783) or reroute the 4 call
  sites to an in-tree metaschema harness, then restore
  `tests/owl_value_semantics.rs` from `tests/disabled/` byte-verbatim]
