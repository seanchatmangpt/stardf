# VENDOR-PROVENANCE — stardf-iri

## Source and law

Vendored from PurRDF `crates/iri` at rev `0da0578306983da851a9073337d57e2e2c41ab3a`
(extraction: `git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/iri`
— never the dirty tree), per the vendoring law: package rename `purrdf-iri` →
`stardf-iri` ONLY; lib name `purrdf_iri` stays VERBATIM; LICENSE files,
in-file notices, SPDX headers, source, tests, benches, `unicode/` data and
`licenses/` travel byte-verbatim. Edition pinned `2024` inline
(upstream-equal; root workspace carries 2021). Upstream
`homepage.workspace = true` and `documentation = "https://docs.rs/purrdf-iri"`
dropped (disclosed; same posture as stardf-lex). No new EXTERNAL (crates.io)
dependencies — `[dependencies]` are first-party `stardf-lex` + `stardf-hash`
(intra-workspace paths); `[dev-dependencies]` is first-party `stardf-testkit`.
No `authority/stardf/license-compat.json` entries required by this lane.

## Byte-identity receipts (all commands this lane, 2026-10-31)

- Inventory: 69 upstream files ↔ 69 vendored files (1:1; `find | wc -l` = 69 both sides).
- 61 files byte-identical in place (`diff -r` upstream-vs-vendored classified
  per file; `cmp` per file): all of `src/` (13 files), 7 active integration
  tests, `licenses/` (7 files), `unicode/` (24 files), `README.md`,
  `tests/PROVENANCE.md`.
- Rolling SHA256 over the same 61-file relpath set, both sides:
  `ddfc5151e9f9a5820f078882b2fa877d58233e52f93d12876d3f4f915edf1a22`.
- 7 upstream files traveled byte-verbatim via park→unpark (per-file `cmp` 7/7
  VERBATIM at park time; `mv` preserves bytes): `host_differential.rs`,
  `host_differential_vectors.txt`, `langtag_differential.rs`,
  `langtag_differential_vectors.txt`, `property.rs`, `structured_errors.rs`,
  `benches/parse.rs`.
- 1 file adapted: `Cargo.toml` (the disclosed adaptation surface; source files
  carry ZERO edits).

## Gate receipts (all commands this lane, 2026-10-31)

- `cargo check -p stardf-iri` → exit 0.
- `cargo check -p stardf-iri --all-targets` → exit 0 (bench `parse` + all 12
  test targets compile).
- `cargo test -p stardf-iri --no-fail-fast` → 243 passed / 1 failed:
  lib 70/70; authority 5/5; base_scope 17/18; host_differential 3/3;
  idna 16/16; iri_suite 13/13; langtag_corpus 17/17; langtag_differential 4/4;
  property 10/10; resolution 3/3; structured_errors 2/2; w3c_iri 3/3;
  bench `parse` 80/80 (harness=false, run as test).
- The single failure
  `base_scope::empty_reference_is_the_same_document_reference`
  (tests/base_scope.rs:127, expects the empty-reference serialization
  `""`; crate relativizes to fragment-only `"#frag"`) is UPSTREAM-INHERITED,
  NOT vendor-induced: witnessed failing IDENTICALLY under purrdf's own
  workspace at the same rev — scratch extraction
  (`git -C /Users/sac/purrdf archive 0da057830698… | tar -x` to /tmp, isolated
  `CARGO_TARGET_DIR`), `cargo test -p purrdf-iri --test base_scope` → exit 101,
  same left/right assertion text. Carried verbatim per the vendoring law; a
  fix would be a non-rename source edit (falsifier class) — escalated to the
  campaign integrator, not fixed here.

## Sequence log (same-tree concurrent lanes; all disclosed)

1. This lane arrived to find the `stardf-iri` directory already extracted
   (byte-identical to the rev, `diff -r` exit 0, manifest still `purrdf-iri`
   unrenamed) — placed by the sparql-algebra vendoring lane as part of its
   dep-closure copy (their NOTE in the root manifest; "disclosed in the
   sparql-algebra lane receipt").
2. This lane applied the rename+adaptation manifest, registered the member
   entry, parked the 4 testkit-dependent tests + 2 vector files + bench under
   `tests/disabled/` + `benches/disabled/` with unpark-precondition READMEs
   (testkit was not yet a resolvable dep at that moment).
3. sparql-algebra lane rebuilt this crate's manifest twice during a
   manifest-write collision (their root-manifest NOTE): first restoring
   dev-dep + bench row, then re-alling to the parked posture ("PARKED-DEVDEP" /
   "PARKED-TARGET" comments), which also commented out this lane's inlined
   lints tables. The second rewrite left the handoff instruction: "Restore the
   line when that lane unparks them."
4. testkit lane landed `crates/stardf-testkit` (manifest, src, tests; lib name
   `purrdf_testkit` pinned ~200s after this lane's bounded poll began — the
   owner lane fixed their own missing-[lib]-name defect).
5. This lane unparked all 7 files back to canonical paths, deleted the
   `disabled/` dirs + stale READMEs, and executed the manifest handoff:
   restored the ACTIVE `stardf-testkit` dev-dep, the `[[bench]] parse` row,
   and the inlined lints tables (manifest rewritten clean, single Write).
6. stardf-testkit's lib failed to compile on first `cargo test`:
   17 `include_str!` metaschema JSONs missing at the literal path
   `crates/jsonschema/tests/metaschemas/**` (upstream-relative layout). Owner
   window elapsed (240s bounded poll, no landing); this lane applied the
   sanctioned minimal compile-freeze unblock: copied the 17 files byte-verbatim
   from the rev extraction into
   `stardf/arrow/crates/jsonschema/tests/metaschemas/**` (per-file `cmp`:
   ALL-17-VERBATIM). Data only — no manifest, no src, nothing of the future
   stardf-jsonschema vendor touched. CROSS-LANE UNBLOCK, disclosed here and in
   this lane's campaign receipt.

## Standing

`ALIVE` for the vendored crate: check gate green, all-targets green, 243/244
tests passing with the 1 failure upstream-inherited (witnessed at source rev).
