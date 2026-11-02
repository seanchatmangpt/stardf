# stardf-lex parked upstream tests: unpark preconditions

These ten upstream test files are PARKED, not deleted: byte-verbatim from purrdf
rev `0da0578306983da851a9073337d57e2e2c41ab3a`. They are inert here because
cargo does not auto-discover targets in nested directories. Zero source edits
were made to park them.

Unpark one when its preconditions hold (and see VENDOR-PROVENANCE.md for the
two upstream-defect falsifiers).

## Missing dev-dep substrate (`stardf-testkit` / `stardf-stack` not vendored)

| file | blocked on | corpora also needed at unpark |
| --- | --- | --- |
| frozen_vectors.rs | purrdf-testkit (vectors) | none (tests/vectors/ already in-tree) |
| json_test_suite.rs | purrdf-testkit (paths) | JSON Test Suite corpus |
| json_value_equality.rs | purrdf-stack (on_stack, line 177) | none |
| structured_errors.rs | purrdf-testkit (error_capture) | none |
| unicode_normalization.rs | purrdf-testkit (ucd) | UnicodeData.txt, normalization differentials |
| xmlconf.rs | purrdf-testkit (paths) | XMLCONF corpus |

## Upstream-defect falsifiers (escalated, per the vendoring law)

| file | observed | receipt |
| --- | --- | --- |
| yaml_scalar_keys.rs | does not typecheck: mixes `&str` and `Err(ErrorKind::...)` elements in one array (E0308 on every edition); minimal repro `/tmp/mini.rs` E0308 both editions, rustc 1.101.0-nightly | purrdf commit `2f40f979` added it; no later purrdf commit touches it |
| yaml_reader_edges.rs | 14/16 pass; 2 fail identically in the pristine upstream tree (scratch workspace `/tmp/lex-repro2` with pristine purrdf lex+hash at the pinned rev, nightly 1.101.0-nightly 2026-10-24): `a_tab_indented_line_cannot_continue_a_plain_multi_line_scalar` (line-count 3 vs 2), `an_anchor_or_alias_name_stops_at_a_colon_only_when_it_reads_as_a_value_indicator` (parsed `{"a":"v","v":"w"}` vs expected `{"a":"v","k":"w"}`) | scratch repro receipt in VENDOR-PROVENANCE.md |

## Missing external corpora (no testkit needed)

| file | observed | unpark needs |
| --- | --- | --- |
| unicode_aligned.rs | compile-time `include_str!` of upstream-sibling corpus `crates/iri/unicode/17.0.0/NormalizationTest.txt` (not vendored) | corpus at that relative path |
| yaml_test_suite.rs | panics at line 83 ("a readable suite directory" NotFound): walks a suite dir for `in.yaml` | YAML Test Suite corpus dir |

ACTIVE surface: lib unit tests (198), `html.rs` (12), `gen_html_entities.rs`
example, 90 doc tests — all green, `cargo test -p stardf-lex` exit 0.
