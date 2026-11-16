# stardf-shex parked upstream tests: unpark preconditions

Two upstream test files are PARKED, not deleted: byte-verbatim from purrdf rev
`0da0578306983da851a9073337d57e2e2c41ab3a` (diff-witnessed byte-identical
2026-10-31, lane 13). They are inert here because cargo does not auto-discover
targets in nested directories. Zero source edits were made to park them.

| file | blocked on | upstream witness at pinned rev |
| --- | --- | --- |
| exact_facets.rs | upstream mid-refactor: `ExactSchema::resolve_imports` gained a `root_iri: Option<&str>` parameter; `bounds_survive_imports` calls the one-argument form. Identical E0061 witnessed upstream (`cargo check -p purrdf-shex --test exact_facets`, 2026-10-31) | upstream fails identically |
| structured_errors.rs | same root: `parse_shexc` arity drift, E0061 3-vs-2 witnessed upstream (`cargo check -p purrdf-shex --tests`, 2026-10-31) | upstream fails identically |

Unpark when upstream (or an operator-directed port fix) reconciles the tests
with the current API.
