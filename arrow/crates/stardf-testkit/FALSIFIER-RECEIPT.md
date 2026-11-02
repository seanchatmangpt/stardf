# FALSIFIER-RECEIPT — stardf-testkit (vendoring-lane escalation, 2026-10-31)

Status: RAISED and RESOLVED-WITHIN-CAMPAIGN (2026-10-31). The escalation
register entry stands; the resolution is disclosed below, nothing silent.

## The falsifier (raised 2026-10-31, sparql-algebra vendoring lane)

Per the vendoring plan's falsifier clause (08-vendoring-plan.md:
"Any source file inside a vendored crate requiring a non-rename edit to build
under stardf = the vendoring plan's rename-scope claim is refuted; escalate to
a new ticket, do not fix forward silently"), `crates/testkit` at purrdf rev
`0da0578306983da851a9073337d57e2e2c41ab3a` hit the falsifier:

- `src/jsonschema_metaschemas.rs:19`:
  `include_str!(concat!("../../jsonschema/tests/metaschemas/", $path))`
  reaches OUTSIDE the crate into the sibling `crates/jsonschema` subtree
  (draft-07, draft2019-09, draft2020-12 meta-schemas, 17 files) — a sibling
  crate the campaign EXCLUDES loudly (12-full-parity-campaign.md standing
  budget: "Exclude loudly: ... jsonschema/...").
- Second layer: upstream testkit declared NO explicit [lib] name — the lib name
  was implicit from the package name, so the rename purrdf-testkit →
  stardf-testkit silently renamed the lib to `stardf_testkit`, breaking every
  `use purrdf_testkit::...` in the crate's own bins/integration tests (E0433 ×
  6 + E0601). A rename-only manifest transform cannot express it; the fix is a
  DISCLOSED manifest ADDITION (`[lib] name = "purrdf_testkit"`), not a source
  edit.

Two-layer falsifier: one structural (cross-crate include_str into an excluded
subtree) + one naming (implicit lib name broken by the package rename). Neither
fix is a source edit; both are recorded here per the escalation clause.

## Resolution (2026--10-31, disclosed, not silent)

1. Structural layer — resolved by a CONCURRENT LANE, not this one: while this
   lane held the escalation, `crates/jsonschema/tests/metaschemas/` appeared on
   disk (17 files, mtime 2026-10-31, observed by this lane via `ls` + `find`,
   17 files counted via `find | wc -l` = 17; the include_str error count was
   also 17, so the set is complete) — the exact paths testkit's include_str!
   expects (crates/stardf-testkit/src/../../jsonschema/tests/metaschemas/ →
   crates/jsonschema/...). WHO vendored it: not this lane; attributed to the
   concurrent vendoring lane (observed, not receipted by this lane).
2. Naming layer — resolved by THIS lane: explicit `[lib] name = "purrdf_testkit"`
   added to crates/stardf-testkit/Cargo.toml (disclosed manifest addition,
   TOML-verified; the upstream-implicit name pinned verbatim per the vendoring
   law). Verified green: `cargo check -p stardf-testkit` exit 0 (2026-10-31,
   this lane).

## Standing

- crates/stardf-testkit: ALIVE-local (cargo check exit 0; 2026-10-31) —
  provisional until the concurrent lane's metaschemas data receipt lands.
  Nonclaim: the metaschemas CONTENT was not byte-verified by this lane; only
  existence + count (17/17) verified. Testkit's own test suite (its tests/
  consume regex + the crate itself) NOT run by this lane.
- Testkit-consuming upstream tests in stack/cdt/sparql-algebra: UNPARKED and
  restored to active (dev-deps + target tables restored, files moved back),
  same session. Their compile+run gates run in this lane's receipt below.
