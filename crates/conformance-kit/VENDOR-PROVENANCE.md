# VENDOR-PROVENANCE — stardf-conformance-kit

Vendored from PurRDF `crates/conformance-kit` at rev
`0da0578306983da851a9073337d57e2e2c41ab3a` (extraction via
`git -C /Users/sac/purrdf archive <sha> crates/conformance-kit | tar -x`),
2026-10-31, lane w2-l07 (copy sweep 7), per the vendoring law: package rename
`purrdf-conformance-kit` -> `stardf-conformance-kit` ONLY; lib name
`purrdf_conformance_kit` verbatim; no LICENSE directory upstream (package
`license` scalar carries the grant; SPDX headers on every source file carry
the in-file notices) — copied verbatim. Source is the pinned git rev, never
the dirty tree.

## Landing posture (precedent: stardf-rdf-core-slice)

Byte-verbatim tree + reconstructed manifest, NOT a member of the
stardf/arrow workspace (empty `[workspace]` detach table). Membership is an
integration decision gated on the residual closure below; joining early
would break the shared compile ladder with dangling path deps.

## Delta surface (complete, disclosed)

1. `Cargo.toml` — rewritten for the host workspace (package name per rename
   law; concrete scalars; dep keys to `stardf-*` paths; upstream
   `[lints] workspace = true` inlined verbatim from the purrdf root
   `[workspace.lints]` at the vendored rev; `[[test]] grading harness=false`
   carried verbatim; empty `[workspace]` detach table added — host plumbing,
   rdf-core-slice posture). Disclosed divergences:
   - `version = "26.9.29"` (host landed-version convention; upstream
     inherited 26.10.29 from the purrdf root).
   - `[dependencies]` keys renamed per law: purrdf-core→stardf-core,
     purrdf-iri→stardf-iri, purrdf-lex→stardf-lex, purrdf-hash→stardf-hash,
     purrdf-sparql-results→stardf-sparql-results,
     purrdf-cdt→stardf-cdt, purrdf-testkit→stardf-testkit,
     purrdf-rdf→stardf-rdf (all `path` deps to the named sibling crates,
     same layout as stardf-cdt's manifest).
2. Everything else byte-verbatim (15 files: 11 src + 4 tests incl. 2 goldens),
   sha256 receipts below.

## Gates (run 2026-10-31, lane w2-l07)

| Gate | Command | Result |
|---|---|---|
| G1 byte-parity | per-file `git show REV:path \| shasum -a 256` vs landed shasum, 15 files | 15/15 OK, exit 0 |
| G2 manifest delta | this file, "Delta surface" item 1 | disclosed in full |
| G3 build witness | `cargo check` in the crate dir (CARGO_TARGET_DIR=target-w2l07) | exit 101 — expected-fail witness: "failed to load manifest for dependency `stardf-core` / failed to read .../stardf-core/Cargo.toml, No such file or directory". [SKIP: closure not vendored] |
| G4 no-freeze | `cargo metadata --no-deps` at stardf/arrow root | exit 0 — workspace graph resolves with this dir present and NOT a member |

## Residual closure (the falsifier, escalated)

Not buildable until three first-party purrdf deps land as stardf crates:

- `purrdf-core` — 54 use sites across src/ (RdfDataset/TermId/TermValue/
  try_canonicalize/blank_label/cdt_blank). stardf-core exists today only as
  the src-only shim at `/Users/sac/ggen-legacy/stardf/crates/stardf-core`
  (no manifest; W2 kernel lanes l01–l05 land the probe-plan kernel over it).
- `purrdf-sparql-results` — 1 use site (src/result_set.rs:10,
  `ParsedSolutions`). W3 results lanes.
- `purrdf-rdf` — tests only (tests/grading.rs, tests/n3_corpus.rs). W3 codec
  lanes.

Unblock order: vendor the closure (W3/W4), then the coordinator adds
`crates/stardf-conformance-kit` to the arrow workspace members and G3 flips
from expected-fail witness to a real build+test court
(`cargo test -p stardf-conformance-kit`, harness=false grading binary
included).

## sha256 receipts (landed tree, 2026-10-31)

See the per-file shasum run recorded in the lane report; the landed files
match `git show 0da0578306983da851a9073337d57e2e2c41ab3a:crates/conformance-kit/<path>`
byte-exact (G1: 15/15, exit 0, witnessed 2026-10-31).

## See Also

- `/Users/sac/ggen-legacy/stardf/docs/jira/v26.10.31/08-vendoring-plan.md`
- `/Users/sac/ggen-legacy/stardf/docs/jira/v26.10.31/12-full-parity-campaign.md` (W2)
