# stardf-sparql-algebra parked upstream tests: unpark preconditions

`ast_snapshot.rs` is PARKED here, not deleted: byte-verbatim from purrdf rev
`0da0578306983da851a9073337d57e2e2c41ab3a` (sha256
`66a35e69d421207549862e8b0e0a6096bede33a7a1036776beed17fc1ece9dd7`, verified
against `git -C /Users/sac/purrdf show 0da05783:crates/sparql-algebra/tests/ast_snapshot.rs`
before and after the move — move-only, zero source edits, no falsifier under
the vendoring law). It is inert here because cargo does not auto-discover
targets in nested directories. No `[[test]]` entry exists for it in the crate
manifest (it was an auto-discovered target), so the move alone parks it.

Parked by lane w2-l13 (wave 2, lane 13, repair; "snapshot test files" lease),
2026-10-31. Receipt commands are in the w2-l13 lane receipt; the failure it
parks was already registered by the vendoring lane in
`crates/stardf-sparql-algebra/VENDOR-PROVENANCE.md` ("5 layout-coupled algebra
targets fail on missing non-vendored corpus dirs").

## Why parked (the failure, witnessed this session)

`the_parser_output_over_every_in_repo_request_matches_the_golden` panics at
`ast_snapshot.rs:100` (`collect()` on
`/Users/sac/ggen-legacy/stardf/arrow/crates/sparql-conformance/suite`):
`No such file or directory (os error 2)`. The test's `CORPORA` table pins the
upstream repo layout — `crates/sparql-conformance/suite`,
`crates/sparql-conformance/corpus`, `crates/sparql-algebra/tests/update`,
`vectors/sparql-cdt`, `vectors/sparql-governors/cases`, `queries`,
`generated/queries` — none of which exist under the stardf/arrow workspace
root (only the crate's own `tests/update/` corpus was vendored, 13 `.ru`
files), and its `MIN_FILES >= 1500` floor cannot be met from the vendored
tree. The same root cause holds for the other four corpus-coupled targets
(`corpus`, `iterative_traits`, `serializer_roundtrip_sweep`,
`shipped_sparql_examples`) — NOT parked here; they belong to other lanes'
leases (corpus-coupling, serialize, traits slices).

## Unpark preconditions (all required)

1. Vendor the corpora from the pinned rev via git archive into the workspace
   root: `crates/sparql-conformance/{suite,corpus}`, `vectors/sparql-cdt`,
   `vectors/sparql-governors/cases`, `queries/`, `generated/queries`
   (thousands of files; `queries/` is the corpus-coupling lane's named
   surface — coordinator admission required for the root-level dirs).
2. Re-point the `crates/sparql-algebra/tests/update` entry in `CORPORA` to
   `crates/stardf-sparql-algebra/tests/update` (a NON-rename edit to a
   vendored file — falsifier under the vendoring law; needs campaign
   admission, never silent).
3. `mv` this file back to `tests/ast_snapshot.rs` (bytes stay verbatim);
   commit golden `tests/goldens/ast_snapshot.golden` then matches the vendored
   corpora by construction (same pinned rev).
4. Re-run: `CARGO_TARGET_DIR=<lane-target> cargo test -p stardf-sparql-algebra
   --test ast_snapshot` → the golden comparison must pass, and
   `regenerate_ast_snapshot` stays `#[ignore]` (writes the committed golden).

The sibling parked-sets (stardf-lex, stardf-xsd `tests/disabled/README.md`)
follow the same law; this lane's move keeps the file byte-identical so the
provenance chain to rev 0da05783 is intact.
