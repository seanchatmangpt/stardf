# VENDOR-PROVENANCE — stardf-lex

Vendored from PurRDF `crates/lex` at rev
`0da0578306983da851a9073337d57e2e2c41ab3a` (verified: `git rev-parse` of the
pinned SHA resolves in `/Users/sac/purrdf`; extraction via
`git -C /Users/sac/purrdf archive <sha> crates/lex | tar -x`), 2026-10-31,
per the vendoring law: package rename `purrdf-lex` -> `stardf-lex` ONLY; lib
name `purrdf_lex` verbatim; LICENSE files and in-file notices verbatim.
Source is the pinned git rev, never the dirty tree.

## Delta surface (complete, disclosed)

1. `Cargo.toml` — rewritten for the host workspace (concrete keys, no
   `[workspace.package]` inheritance; the root's `[workspace.package]` carries
   edition 2021 which this crate's source cannot use). Disclosed divergences:
   - `edition = "2024"` (upstream-equal): lex src uses let-chains
     (`src/cbor/decode.rs:548`). The stardf/arrow root workspace edition is
     2021 but that key is not inherited by this manifest — the crate's own
     edition key is authoritative.
   - `[dependencies] stardf-hash = { path = "../stardf-hash" }` — upstream dep
     `purrdf-hash` renamed per the vendoring law; resolves to the vendored
     stardf-hash member (lib `purrdf_hash` verbatim).
   - Upstream `[dev-dependencies]` `purrdf-testkit`/`purrdf-stack` omitted (not
     vendored, not in this lane's scope); see item 3.
   - Upstream `[[bench]] scan` / `[[test]] frozen_vectors` tables dropped
     together with the parked targets (item 3).
   - Upstream `[lints] workspace = true` inlined verbatim from purrdf root
     `[workspace.lints]` at the vendored rev (hash-lane posture).
2. `README.md` — the only content-edited vendored doc: the sanctioned
   package-token renames applied (see note A).
3. Parking (no source edits): the ten upstream `tests/*.rs` files listed in
   `tests/disabled/README.md` moved to `tests/disabled/` (not auto-discovered
   by cargo); upstream `benches/scan.rs` -> `benches/disabled/scan.rs`;
   `examples/gen_unicode_tables.rs` -> `examples/disabled/`. Unpark
   preconditions per file: `tests/disabled/README.md`. Two of the ten are
   UPSTREAM-DEFECT falsifiers (yaml_scalar_keys.rs does not typecheck;
   yaml_reader_edges.rs 2/16 fails — both receipts below).
4. In-src test-module gating (disclosed non-rename edit, escalated via this
   file + the lane report): ten `#[cfg(test)]` declarations (11 sites) across
   `src/crockford.rs`, `src/iri_escape.rs`, `src/json_escape.rs` (2 sites),
   `src/literal_escape.rs`, `src/walk.rs`, `src/scan.rs`, `src/cbor.rs`,
   `src/json.rs`, `src/yaml.rs`, `src/unicode/casing.rs` changed
   `#[cfg(test)]` -> `#[cfg(all(test, feature = "upstream-tests"))]` — same
   posture as stardf-columnar's reviewed `upstream-tests` feature. Reason:
   these upstream unit-test modules import purrdf-testkit/purrdf-stack (not
   vendored) or include upstream-sibling corpus files, and cargo test builds
   them unconditionally. Post-gating the lib-only `cargo check -p stardf-lex`
   surface is UNCHANGED; `cargo test -p stardf-lex` runs the 198 remaining
   unit tests green.
5. Everything else byte-verbatim: `src/` (all 47 files), `data/` (HTML
   entities + numeric-recovery data with their `.license` files and
   `PROVENANCE.md`), `licenses/` (all 9 files, incl.
   `third-party/unicode/0-Unicode-3.0.txt` and
   `third-party/whatwg-html-references/0-LICENSE-WHATWG` — the Unicode-3.0 +
   BSD-3-Clause conjunct files), `tests/vectors/` (18 vector files),
   `PROVENANCE.md` (upstream data-provenance chain, verbatim — it is part of
   the legal chain for `data/html/` and the Unicode tables).

## Note A — README renames

Applied `purrdf-lex` -> `stardf-lex` and `purrdf-hash` -> `stardf-hash` to
README.md. Remaining `purrdf` tokens in README.md are upstream-ecosystem prose
and URLs (crates.io/repo badges, sibling-crate references, logo URLs) kept
verbatim: the upstream ecosystem facts (crates.io `purrdf-lex` artifact,
Blackcat-Informatics/purrdf repo) remain true of the SOURCE of this vendored
tree.

## Note B — sibling-name doc prose

src/ doc comments name upstream sibling packages (`purrdf-core`, `purrdf-iri`,
`purrdf-text`, `purrdf-xsd`, `purrdf-sparql-algebra`, `purrdf-shex`,
`purrdf-jsonschema`) in architecture prose. Kept verbatim: they describe the
upstream layering truthfully; renaming them would name crates that do not exist
here. Same posture as the hash lane's manifest (its lints comment keeps
`purrdf_core::` verbatim).

## Note C — licenses/inventory.json

Carried verbatim (LICENSE files verbatim per law). It retains upstream fields
(`"profile": "purrdf-lex"`, upstream `crates/lex/...` paths in `components[].paths`)
— upstream inventory facts about the vendored files, kept verbatim by the
LICENSE-files-verbatim law; a stardf-local inventory rewrite is future work,
not silently performed here.

## Verification receipts (commands the lane ran)

- `git -C /Users/sac/purrdf rev-parse 0da05783...` -> resolves (SHA exists).
- `git -C /Users/sac/purrdf ls-tree -r --name-only <sha> -- crates/lex` ->
  112 files listed upstream.
- Extraction: `git archive <sha> crates/lex | tar -x` into
  `stardf/arrow/crates/stardf-lex` (112 files).
- Verbatim court (post-vendor): `diff -r` of vendored tree vs a pristine
  `git archive` extract of `<sha> crates/lex` — the complete delta is the
  disclosed surface above (see lane report for the diff receipt).
- `cargo check -p stardf-lex` — see lane report for the exit receipt.
- License-compat gate `python3 scripts/verify_stardf_licenses.py` — see lane
  report for the exit receipt.

## Upstream-defect falsifiers (escalated per the vendoring law)

1. `tests/yaml_scalar_keys.rs` (pinned rev) does not typecheck: one `for`
   array mixes `&str` elements with `Err(ErrorKind::...)` elements (E0308,
   every edition). Minimal standalone repro `/tmp/mini.rs` fails E0308 under
   `--edition=2024` AND `--edition=2021`, rustc 1.101.0-nightly (12433c307
   2026-10-24). Upstream citation: added by purrdf commit `2f40f979`
   ("wave-22 inline repairs across 7 crates + doc-drift batch"); no later
   purrdf commit touches the file. Parked, not repaired.
2. `tests/yaml_reader_edges.rs` (pinned rev): 14/16 pass, 2 fail — and the
   same 2 fail IDENTICALLY in the pristine upstream tree (scratch workspace
   `/tmp/lex-repro2`: pristine purrdf `crates/lex` + `crates/hash` at the
   pinned rev, upstream-equal workspace config, `cargo +nightly test -p
   purrdf-lex --test yaml_reader_edges` exit 101, same assertion messages).
   Not a vendoring artifact, not a stardf-hash divergence. Parked, not
   repaired.
3. consequence: the vendoring law's falsifier clause fired twice ("a
   NON-rename edit needed to build = falsifier — escalate, never fix
   silently"); both escalated as parking + this register + the lane report,
   zero silent repairs.
