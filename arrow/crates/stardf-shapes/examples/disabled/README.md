# stardf-shapes parked oracle-fixture examples: unpark preconditions

Four upstream example targets are PARKED, not deleted: byte-verbatim from purrdf
rev `0da0578306983da851a9073337d57e2e2c41ab3a`-pinned files (diff-witnessed
2026-10-31, lane 13). They are inert here because cargo does not auto-discover
examples in
nested directories. Zero source edits were made to park them.

## Blocked on: `purrdf-jsonschema` exclusion (crates/jsonschema/DISPOSITION.md)

| file | blocked on |
| --- | --- |
| graphql_oracle_fixture.rs | direct `purrdf_jsonschema` use (5 sites) |
| linkml_oracle_fixture.rs | `support::metaschemas` + `support::shacl_value_shapes` (both consume `purrdf_jsonschema`) |
| pydantic_oracle_fixture.rs | same support seam |
| typescript_oracle_fixture.rs | direct `purrdf_jsonschema` use (5 sites) |

Unpark when (a) an operator Tier-C decision vendors purrdf-jsonschema as
stardf-jsonschema, or (b) a stardf-native draft-2020-12 validator exists and
the fixtures are re-seamed onto it.
