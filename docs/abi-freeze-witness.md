# ABI freeze witness — S-4 (court-auditor), 2026-11-01

Subject: /Users/sac/stardf @ fe935f7f18f6a910955aa5228ca46f837abae899 (working tree).

## Commands + exits (this lane's own runs)

- `cargo build --release -p stardf-arrow-capi` (in-tree, CARGO_TARGET_DIR=/tmp/s4-target):
  first run EXIT=101 (`/Users/cgap/crates/audit` not found — committed manifest
  has 5-up `../../../../../cgap/...` paths; the shared-tree working copy has
  since carried a 4-up fix as an uncommitted modification); re-run after the
  fix EXIT=0. Fresh build, clean target: EXIT=0 (/tmp/s4-target-b, cleaned).
- `nm -gU` on the built dylib: EXIT=0, 14 exports.
- `cc -std=c11 -Wall -Werror -fsyntax-only include/stardf_arrow.h`: EXIT=0.
- grep v2 symbol names in capi src: 0 matches (13 `#[no_mangle]` sites, v1 set).

## Symbol table (dylib = fresh in-tree build, sha256 84b17b09...)

| symbol | in v1 | in v2 | in dylib |
|---|---|---|---|
| stardf_arena_intern | Y | Y | Y |
| stardf_arena_new | Y | - | Y |
| stardf_arena_free | Y | - | Y |
| stardf_batch_append_quad | Y | - | Y |
| stardf_avec_export_stream | Y | - | Y |
| stardf_stream_free | Y | - | Y |
| stardf_last_error | Y | - | Y |
| stardf_arrow_version | Y | - | Y |
| stardf_i32_batch | Y | - | Y |
| stardf_array_addr | Y | - | Y |
| stardf_release | Y | - | Y |
| stardf_pending_tags | Y | - | Y |
| stardf_batch_rows | Y | - | Y |
| stardf_arena_create | - | Y | N |
| stardf_arena_destroy | - | Y | N |
| stardf_dataset_create | - | Y | N |
| stardf_dataset_destroy | - | Y | N |
| stardf_dataset_append_quad | - | Y | N |
| stardf_query_execute | - | Y | N |
| stardf_stream_next | - | Y | N |
| stardf_stream_destroy | - | Y | N |
| stardf_export_arrow_stream | - | Y | N |
| stardf_columnar_write_file | - | Y | N |
| stardf_columnar_read_file | - | Y | N |
| _blake3_compress_in_place_portable | - | - | Y (transitive, declared hygiene item) |

## D18 state (PENDING-OPERATOR, witnessed as-of 2026-11-01)

OPERATOR-DECISIONS.md:642-644: D18 (ABI-v2) PENDING-OPERATOR — "not adopted
this campaign; arrow stays landed as-is."

## Verdict: v1-ACTIVE — expected state, no freeze violation

The dylib matches the v1 13-symbol table exactly (13/13 present, 0 extra
stardf_* beyond the declared blake3 hygiene export). The v2 table is 1/13
present (arena_intern only). Header (include/stardf_arrow.h) declares the v1
surface and syntax-checks clean. 0 v2 names in capi src. This is exactly the
D18 PENDING-OPERATOR state ABI-FREEZE.md records; v1-file + v1-dylib is
consistent, not the v2-active/v1-file violation shape.
