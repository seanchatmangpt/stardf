# VENDOR-PROVENANCE — stardf-testkit

Vendoring receipt for `crates/stardf-testkit` (sparql-algebra vendoring lane, 2026-10-31).

## Source

- Upstream: `/Users/sac/purrdf`, pinned rev `0da0578306983da851a9073337d57e2e2c41ab3a`.
- Materialized via `git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/testkit` — the upstream
  working tree was DIRTY at vendor time; the rev was used, never the tree.
- Destination: `stardf/arrow/crates/stardf-testkit/`, member of the stardf/arrow workspace.

## Upstream identity receipt (git archive | shasum -a 256)

    1f720713fd609e478928184bb14e70be044ab734b3ca38b80cf3f8d4ad781bdf

## Retention (proven by diff -rq vs the rev, 2026-10-31)

- Source trees byte-identical outside the declared delta surface (`diff -rq` vs
  the rev extraction: ZERO non-Cargo.toml deltas for this crate, this session).
- Lib name `purrdf_testkit` VERBATIM per the vendoring law; only the PACKAGE name renamed
  (purrdf-testkit -> stardf-testkit).
- SPDX headers + licenses/ retained verbatim (byte-identical in the same diff).
- Manifest delta surface (Cargo.toml only): package name; workspace-inherited
  scalars (version 26.9.29 / edition 2024 / upstream license string / repository
  -> ggen-legacy / rust-version 1.98); first-party dep keys -> path deps; external
  pins carried upstream-equal. Manifest extras: Explicit [lib] name = purrdf_testkit added (same implicit-name posture as stardf-alloc-probe). FALSIFIER-RECEIPT.md added (escalation record).

## Gates (commands this lane ran, 2026-10-31)

cargo check -p stardf-testkit exit 0; lib tests 31 passed/1 failed (paths.rs:108 layout-pin divergence, registered); integration targets not individually run (lib abort gates the default --tests flow)

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
