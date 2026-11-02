# abi-freeze

## Statement

The `stardf-arrow-capi` C export surface is frozen. The **enforced** surface
is the v1 13-symbol `stardf_*` set (the dylib on disk exports exactly those
13 plus the declared `_blake3_compress_in_place_portable` hygiene item — a
transitive blake3 internal, not an ABI break). The **v2 13-symbol surface of
record** (from the operator's Definition of Done) is PENDING-OPERATOR (D18):
"not adopted this campaign; arrow stays landed as-is." Adoption is a later
operator call carrying the cross-allocator-court migration cost.

The freeze is ticket-gated: any change to the export surface — adding,
removing, renaming, or re-signaturing any export, or any change to struct
layouts or status-code values — requires a new GL-STARDF ticket. No drive-by
edits to `lib.rs`, `stream_c.rs`, or `include/stardf_arrow.h` signatures.
Header and Rust stay mirrored: edit Rust, then mirror into the header.

The export table in `docs/ABI-FREEZE.md` is admitted only from the DoD or a
ratified GL-STARDF-002 tier — never from whatever the dylib happens to
export. Editing a table row to match the dylib is itself a freeze violation.

D9 Tier A is ratified (stardf_query, stardf_query_governed,
stardf_query_governors_init + carriers ResultKind / QueryGovernors /
GovernorEvidence / RowCursor) but those exports land ONLY through the
GL-STARDF-002 ticket process: no query/UPDATE/serialize `#[no_mangle]` may
land in `crates/stardf-arrow-capi/src/**` outside that process (§5.1); tier
falsifier F3 — carrier drag past the ratified tier boundary is REFUSED.

## Why

- `stardf/arrow/docs/ABI-FREEZE.md` — frozen surface, freeze law, enforcement
  law; its Verification section records the 2026-10-31 witnessed state: 14
  dylib exports = 13 v1 `stardf_*` + 1 blake3 internal; 24 differing lines vs
  the DoD-13 table, only `stardf_arena_intern` common.
- `stardf/arrow/docs/OPERATOR-DECISIONS.md` — D18 (v2 not adopted this
  campaign), D9 sign-off (Tier A ratified, ticket-process only), D7
  (ArrowArrayStream spec authority).
- `stardf/docs/jira/v26.10.31/14-definition-of-done.md` — the v2 surface of
  record, copied verbatim into ABI-FREEZE.md.
- `stardf/arrow/tickets/GL-STARDF-002-freeze-expansion.md` — the ticket
  process any surface change must go through (§5 pre-landing gate, §6 nm
  diff receipt, §5.1 premature-export ban).

## How to apply

`scripts/verify_abi_freeze.sh` does not exist on disk (checked 2026-10-31),
so the gate is the raw `nm -gU` diff. From `stardf/arrow`:

```sh
nm -gU target/debug/libstardf_arrow_capi.dylib
# diff the result against the ENFORCED table in docs/ABI-FREEZE.md;
# exit 1 on any extra/missing symbol, modulo the declared blake3 item.
grep -rn "no_mangle" crates/stardf-arrow-capi/src/   # 13 sites, v1 set only
```

A gate that cannot exit 1 is VACUOUS — the v1-vs-v2 delta currently firing
IS the enforcement working, not a doc defect. On any firing: stop, open a
new ticket; do not edit the doc table to match the dylib, do not land the
symbol.
