# StarDF Arrow C ABI Freeze — v2 (v26.11.1 campaign revision, 2026-10-31)

Status: **PARTIAL_ALIVE** — the v2 frozen surface of record below is
**PENDING-OPERATOR (D18, OPERATOR-DECISIONS.md: "not adopted this campaign;
arrow stays landed as-is")**. The operator's Definition-of-Done document
referenced a v2 surface, but adoption (dylib migration + cross-allocator-court
port) was never signed; the currently-built dylib exports and enforces the
**v1 13-symbol surface** (ABI-FREEZE v1 tables + nm audit are the enforced
contract until D18 is signed). Every claim was re-verified on disk this session; commands and
exits are in the Verification section.

## Frozen 13-symbol export surface (v2)

The v26.11.1 frozen surface of record, verbatim from the operator-issued
[Definition of Done](../../docs/jira/v26.10.31/14-definition-of-done.md)
(operator 2026-10-31, spec of record; supersedes the v26.9.29 surface).
Admission law: an export-table row may enter this doc only from the DoD table
below (or, post-ratification, from a ratified GL-STARDF-002 tier) — never
from whatever the dylib happens to export.

| # | Symbol | Params → Return | Failure | Responsibility |
|---|---|---|---|---|
| 1 | `stardf_arena_create` | void → StardfArena* | NULL | Thread-safe string interning arena |
| 2 | `stardf_arena_destroy` | StardfArena* → void | None | Reclaim arena |
| 3 | `stardf_arena_intern` | (Arena*, const char*, size_t) → uint32_t | 0=BOTTOM | Canonicalize + intern → stable TermId |
| 4 | `stardf_dataset_create` | Arena* → StardfDataset* | NULL | Empty quad dataset bound to arena |
| 5 | `stardf_dataset_destroy` | StardfDataset* → void | None | Dealloc dataset indexes |
| 6 | `stardf_dataset_append_quad` | (Dataset*, u32×4) → int32_t | non-zero | Insert (s,p,o,g) into index tables |
| 7 | `stardf_query_execute` | (Dataset*, const char*) → StardfStream* | NULL | Compile+execute SPARQL → analytical stream |
| 8 | `stardf_stream_next` | (Stream*, void**, size_t*) → int32_t | non-zero | Next CanonArrow batch payload, no allocation |
| 9 | `stardf_stream_destroy` | StardfStream* → void | None | Release cursor/scratch/locks |
| 10 | `stardf_export_arrow_stream` | (Dataset*, ArrowArrayStream*) → int32_t | non-zero | Arrow C Data Interface stream over dataset quads |
| 11 | `stardf_columnar_write_file` | (Dataset*, const char*) → int32_t | non-zero | Emit five Parquet files to a directory |
| 12 | `stardf_columnar_read_file` | (Dataset*, const char*) → int32_t | non-zero | Load+validate five Parquet files |
| 13 | `stardf_last_error_message` | void → const char* | NULL | Thread-local last-error diagnostics |

## Ratification

- **v1 → v2 ratification**: the operator-issued Definition of Done
  (`stardf/docs/jira/v26.10.31/14-definition-of-done.md`, operator
  2026-10-31, AUTHORITATIVE spec of record) IS the ratification of the v2
  frozen surface above. The DoD's "Frozen C-ABI Surface" section is copied
  verbatim; no symbol, signature, failure mode, or responsibility cell is
  locally amended. GL-STARDF-002 ticket §8 nonclaim ("this ticket admits no
  exports and amends no ABI-FREEZE.md; only operator ratification does") is
  satisfied by this operator action, not by the ticket.
- **D9 Tier A ratified** (GL-STARDF-002, OPERATOR-DECISIONS.md sign-off
  block, 2026-10-31): `stardf_query`, `stardf_query_governed`,
  `stardf_query_governors_init` + carriers ResultKind / QueryGovernors /
  GovernorEvidence / RowCursor. These are ratified as Tier A additions to
  this surface; the C exports land ONLY via the ratified ticket process
  (ticket §5/§6: pre-landing gate + nm -gU diff receipt). Until a Tier A
  export lands through that process, the query surface may exist as Rust
  surface only — no `#[no_mangle]` outside the ratified 13 above. Tier
  boundary falsifier F3 (carrier drag past Tier A is REFUSED) stands.

## v1 → v2 delta

v1 = the v26.9.29 freeze (13 exports, nm-verified on the settled tree; kept
in git history as the previous revision of this doc). Dispositions:

| v1 symbol | disposition | v2 symbol |
|---|---|---|
| `stardf_arena_intern` | RETAINED (only carry-over) | `stardf_arena_intern` |
| `stardf_arena_new` | REPLACED | `stardf_arena_create` |
| `stardf_arena_free` | REPLACED | `stardf_arena_destroy` |
| `stardf_batch_append_quad` | REPLACED (carrier: arena batch → dataset) | `stardf_dataset_append_quad` |
| `stardf_avec_export_stream` | REPLACED | `stardf_export_arrow_stream` |
| `stardf_stream_free` | REPLACED | `stardf_stream_destroy` |
| `stardf_last_error` | REPLACED | `stardf_last_error_message` |
| `stardf_arrow_version` | DROPPED (no DoD analogue) | — |
| `stardf_i32_batch` | DROPPED | — |
| `stardf_array_addr` | DROPPED | — |
| `stardf_release` | DROPPED | — |
| `stardf_pending_tags` | DROPPED | — |
| `stardf_batch_rows` | DROPPED | — |
| — | ADDED (no v1 analogue) | `stardf_dataset_create` |
| — | ADDED | `stardf_dataset_destroy` |
| — | ADDED | `stardf_query_execute` |
| — | ADDED | `stardf_stream_next` |
| — | ADDED | `stardf_columnar_write_file` |
| — | ADDED | `stardf_columnar_read_file` |

Counts: 13 = 13 (1 retained + 6 replaced + 6 dropped on the v1 side; 1
retained + 6 replaced + 6 added on the v2 side). No status-code value or
struct layout changes in this delta; only the export set.

## Current implementation surface (receipt, 2026-10-31)

The built dylib (`target/debug/libstardf_arrow_capi.dylib`, mtime 13:33
2026-10-31 on disk) still exports exactly the v1 13 `stardf_*` symbols plus
`_blake3_compress_in_place_portable` (transitive, internal to the BLAKE3
dependency; not part of either frozen surface; not declared in the header —
export-list hygiene item for a future ticket, not an ABI break). The v2
surface is not yet exported: `stardf_query_execute`, `stardf_stream_next`,
`stardf_columnar_write_file`, `stardf_columnar_read_file`, the dataset
lifecycle pair, and the renamed lifecycle calls have no `#[no_mangle]` in
`crates/stardf-arrow-capi/src/` (0 query-family matches this session).
Landing the v2 surface is v26.11.1 campaign work under GL-STARDF-001/002
Tier A governance; this doc's standing stays PARTIAL_ALIVE until the dylib
diffs clean against the v2 table.

## Enforcement law (nm -gU, unchanged from v1, extended per ticket §5.2)

Mechanical `nm -gU` diff of the built dylib against THIS doc's export table:
exit 1 on any symbol present in the dylib but absent from the table, and on
any symbol in the table but absent from the dylib, modulo the declared
`_blake3_compress_in_place_portable` hygiene item. A gate that cannot exit 1
is VACUOUS. Until the v2 surface lands, the gate fires on the delta above —
that firing is the enforcement working, not a doc defect. Additionally
(GL-STARDF-002 §5.1): no query/UPDATE/serialize `#[no_mangle]` may land in
`crates/stardf-arrow-capi/src/**` outside the ratified ticket process.

## Struct layouts (fixed by the Arrow C Data/Stream Interface specs)

All `#[repr(C)]`, 64-bit LP64 (macOS/aarch64) — these describe the current
implementation surface and carry into v2 unchanged:

| Struct | Size | Fields (in order) |
|---|---|---|
| `ArrowArray` | 80 B | length/null_count/offset/n_buffers/n_children: int64; buffers: `const void**`; children: `ArrowArray**`; dictionary: `ArrowArray*`; release: fn ptr; private_data: void* |
| `ArrowSchema` | 72 B | format/name/metadata: `const char*`; flags/n_children: int64; children: `ArrowSchema**`; dictionary: `ArrowSchema*`; release: fn ptr; private_data: void* |
| `ArrowArrayStream` | 40 B | get_schema/get_next/get_last_error/release: fn ptrs; private_data: void* |
| `Quad64` | 64 B | s_id/p_id/o_id/g_id: u32 @0x00-0x0F; t_ns: u64 @0x10; vector_clock: u32 @0x18; policy_mask: u32 @0x1C; inline_digest: [u8;32] @0x20 |

Stream batch shape is frozen on the current surface: 64 quads/batch, 4096
bytes/batch (`QUADS_PER_BATCH`/`BATCH_BYTES`, const-asserted) — consistent
with the DoD CanonArrow framing (`ROWS_PER_BATCH=64`); where the DoD adds
v2-only framing constraints (128-byte fixed header; offsets 68..100 = 32-byte
BLAKE3 domain digest over column data buffers; dedicated 192-byte ASK empty
frame), the DoD is authoritative for v2.

## Status codes

Current implementation codes (the v2 failure column above is contract-level
NULL / non-zero):

| Code | Name | Meaning |
|---|---|---|
| 0 | `STARDF_OK` | success |
| 1 | `STARDF_NULL_POINTER` | NULL out-parameter or handle |
| 3 | `STARDF_INVALID_ARGUMENT` | bad argument value (e.g. `stardf_arena_new(0)`) |
| 16 | `STARDF_ARENA_FULL` | arena byte budget exhausted |

## EOF semantics (the translation law)

Core stream (`stardf-arrow-core/src/stream.rs`) signals end-of-stream with
`STREAM_EOF = 10` (ENOENT, per the C Stream Interface spec's flag
convention). The capi bridge
(`stardf-arrow-capi/src/stream_c.rs::stream_get_next`) translates this into
the consumer-facing end-of-stream: return flag `0` with a zeroed,
release-less `ArrowArray` (`release == NULL`, all fields zero). Zero-length
yields from the core (CursorProducer on an empty slab) receive the same
translation — an empty yield carries no data on this fixed-64 surface, so
the translation never discards rows. Errors are `STREAM_ERR = 1` (any
nonzero other than 0 is an error flag). Every callback is `extern "C"` and
panic-guarded (`run_guarded`): a Rust panic becomes flag 1, never an unwind
across the FFI boundary.

## Ownership / release law

- Every release callback CAS-swaps its release slot to NULL BEFORE dropping
  private data: a double release is a typed no-op, never a double free.
- `stardf_avec_export_stream` mints an ownership tag; `stardf_stream_free`
  claims the tag before dealloc (untagged foreign free is the
  observed-defect shape, detected via `stardf_pending_tags`).
- Release consumes only `private_data`; the stream shell's disposal belongs
  solely to `stardf_stream_free`.
- These laws carry into v2 under the renamed lifecycle pair
  (`stardf_arena_create`/`stardf_arena_destroy`,
  `stardf_dataset_create`/`stardf_dataset_destroy`,
  `stardf_stream_next`/`stardf_stream_destroy`) and the DoD completion
  invariant: get_next after exhaustion returns a null array without error,
  continuously; release is idempotent.

## The freeze law

**Any change to the export surface — adding, removing, renaming, or
re-signaturing any of the 13 v2 exports (or the ratified Tier A additions
when they land), or any change to the struct layouts or status-code values
above — now requires a new ticket.** No drive-by edits to
`lib.rs`/`stream_c.rs`/`include/stardf_arrow.h` signatures. Header and Rust
stay mirrored: edit Rust, then mirror into `include/stardf_arrow.h`. This
doc's export table is admitted only from the DoD or a ratified tier —
editing a table row to match the dylib is itself a freeze violation.

## Verification (commands + exits, 2026-10-31, lane w2-l10)

```text
nm -gU target/debug/libstardf_arrow_capi.dylib
  -> exit 0; 14 exports = 13 stardf_* (v1 set, Current-implementation-surface
     section) + 1 blake3 internal
cc -std=c11 -Wall -Werror -fsyntax-only crates/stardf-arrow-capi/include/stardf_arrow.h
  -> exit 0
comm -3 <nm -gU symbol set> <DoD-13 symbol set>
  -> 24 differing lines (12 dylib-only v1 symbols, 12 DoD-only v2 symbols);
     only stardf_arena_intern common — matches the delta table above
grep -rn "no_mangle" crates/stardf-arrow-capi/src/  -> 13 sites, v1 set only;
  0 query-family matches (Tier A gate §5.1 holds: zero premature exports)
```
