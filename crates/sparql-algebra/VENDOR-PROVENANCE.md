# VENDOR-PROVENANCE — stardf-sparql-algebra

Vendoring receipt for `crates/stardf-sparql-algebra` (sparql-algebra vendoring lane, 2026-10-31).

## Source

- Upstream: `/Users/sac/purrdf`, pinned rev `0da0578306983da851a9073337d57e2e2c41ab3a`.
- Materialized via `git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/sparql-algebra` — the upstream
  working tree was DIRTY at vendor time; the rev was used, never the tree.
- Destination: `stardf/arrow/crates/stardf-sparql-algebra/`, member of the stardf/arrow workspace.

## Upstream identity receipt (git archive | shasum -a 256)

    54c4fc286d5bf69c73b418adca07de344ab406d74620533ea8390e068ba1a11e

## Retention (proven by diff -rq vs the rev, 2026-10-31)

- Source trees byte-identical outside the declared delta surface (`diff -rq` vs
  the rev extraction: ZERO non-Cargo.toml deltas for this crate, this session).
- Lib name `purrdf_sparql_algebra` VERBATIM per the vendoring law; only the PACKAGE name renamed
  (purrdf-sparql-algebra -> stardf-sparql-algebra).
- SPDX headers + licenses/ retained verbatim (byte-identical in the same diff).
- Manifest delta surface (Cargo.toml only): package name; workspace-inherited
  scalars (version 26.9.29 / edition 2024 / upstream license string / repository
  -> ggen-legacy / rust-version 1.98); first-party dep keys -> path deps; external
  pins carried upstream-equal. Manifest extras: None.

## Gates (commands this lane ran, 2026-10-31)

cargo check -p stardf-sparql-algebra exit 0 (THE LANE GATE); unit tests 480/481 (1 pre-existing upstream-at-rev failure, reproduced at pristine rev); integration 18/23 targets green at upstream-true opt-level 3, 5 targets fail on missing non-vendored corpus dirs (registered)

## Known divergences / failures (registered, not silent)

- stardf-sparql-algebra unit test `same_document_references_follow_rfc_3986_under
  _a_sparql_base` fails AND reproduces at the pristine upstream rev (scratch purrdf
  extraction: `cargo test -p purrdf-iri --test base_scope` exit 101, same test, 17/1)
  — pre-existing upstream-at-rev, NOT session-introduced.
- 5 layout-coupled algebra targets fail on missing non-vendored corpus dirs
  (crates/sparql-conformance/suite, crates/rdf-wasm, queries/) — the campaign's
  own algebra-corpus-coupling item.
- Testkit lib test paths.rs:108 pins upstream layout (crates/testkit/) — registered
  divergence; a source edit is forbidden by the vendoring law.
- Upstream [profile.dev] opt-level=3 is load-bearing for deep-recursion integration
  tests (128 KiB-stack tests overflow at the stardf default opt-level 0); receipt
  runs used CARGO_PROFILE_DEV_OPT_LEVEL=3 (env var, no manifest change).
