# cgap-rehearsal — Lane 14 Receipt (2026-10-31)

Ticket [GL-STARDF-001](../../../tickets/GL-STARDF-001.md), W3 gate rehearsal.
Consumes the FINAL `stardf-arrow-capi` as a foreign consumer exactly as the
W3 `cgap-ffi` court will — without writing to `/Users/sac/cgap` (no cgap
writes; the ready-made diff below is deposited here, not applied).

## Standing

- W3 consumer court, final capi — **ALIVE (rehearsed)**.
  `cd /tmp/stardf-cgap-rehearsal && cargo test --test avec_consumer_court`
  → exit 0: **3 passed / 0 failed**.
- Final cdylib foreign-load — ALIVE. Court 3 `dlopen`s
  `target/debug/libstardf_arrow_capi.dylib`, `dlsym`s + calls
  `stardf_arrow_version` → `"26.9.29"`, mints and releases through
  `stardf_i32_batch` / `stardf_release`; ownership-tag delta closes.
- Header/exports three-way check — ALIVE (pre-existing gate).
  `python3 stardf/arrow/scripts/verify.py` → exit 0: 13 ontology
  targetSymbols == 13 header decls == 13 impl exports.
- cgap integration edge (real cgap-ffi test) — OPEN. The rehearsal consumes
  the capi from a /tmp crate; the cgap-ffi dev-dep edge itself remains
  unwired, gated by R1.
- R1 license gate — BLOCKED (by design).
  `python3 stardf/arrow/scripts/verify_stardf_licenses.py` → **exit 1**:
  `edge stardf-arrow-core -> blake3: P28-SIL -> gated license
  'CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception' without
  registered compatibility receipt (R1)` — stays exit 1 until the operator
  moves the drafted receipt in `authority/stardf/license-compat.json`
  (`draft_receipts_pending_operator`) into `compatibility_receipts`.

Environment receipt: cgap local checkout HEAD =
`67c0ec1bad0bf4a81b2512e60f554b137ae596d9` (the exact pinned rev; the scratch
crate consumed `cgap-arrow` by the same git rev so a single cgap-arrow
identity links into the court binary, and `cgap-audit` by local path
`/Users/sac/cgap/crates/audit` — its `CountingAlloc` was the court binary's
`#[global_allocator]`, with `with_alloc_tracking` witnessing real allocations
on the cgap substrate's counting allocator).

## Consumer court (3/3)

Scratch crate `/tmp/stardf-cgap-rehearsal` (deleted after this receipt;
scratch-only per lane contract), one integration target
`tests/avec_consumer_court.rs`:

1. `court_1_export_stream_consume_get_schema_get_next_release` — full
   lifecycle through the C struct function pointers only (no Rust shims):
   `stardf_arena_new` → `stardf_arena_intern` ×4 → `stardf_batch_append_quad`
   ×130 → `stardf_cgap_export_stream` (STATUS_OK) → `get_schema` (flag 0,
   armed release, non-empty format) → `get_next` yields exactly
   ceil(130/64) = 3 non-empty batches then a zeroed release-less EOS array →
   array release callbacks → stream release → `stardf_stream_free` →
   `stardf_pending_tags()` back to baseline (every tag claimed).
2. `court_2_allocation_identity_boxed_array_tags` — `stardf_i32_batch(64, 7)`
   → returned pointer IS `stardf_array_addr()` (zero-copy slot identity),
   `stardf_batch_rows() == 64`, pending tags +1; `stardf_release` returns
   `Released`, live slot cleared, tag claimed; fresh mint inside
   `avec_audit::with_alloc_tracking` shows non-zero allocation count on the
   cgap counting allocator; release returns tags to baseline.
3. `court_3_dylib_loads_and_final_api_serves` — the final dylib loads via
   `dlopen(RTLD_NOW|RTLD_LOCAL)`, serves version `26.9.29`, and the
   release-tagged surface (`stardf_pending_tags`, `stardf_i32_batch`,
   `stardf_release`) is live through `dlsym`: mint → tag +1 → release → tag
   baseline.

## API drift since C16 (header re-read before the run)

- The header (`crates/stardf-arrow-capi/include/stardf_arrow.h`) now carries
  the **release-tagged allocation-identity surface** absent from the original
  W3 plan: `stardf_i32_batch`, `stardf_array_addr`, `stardf_batch_rows`,
  `stardf_pending_tags`, `stardf_release` (returning `stardf_ReleaseOutcome`)
  — 13 declarations total (three-way check green).
- The cgap pin moved: the README's integration plan still names rev
  `20248e4`, but the workspace pin is `67c0ec1` (`20248e4` is broken upstream
  — missing `chase_lev.rs`; see `Cargo.toml` pin comment). The rehearsal ran
  on `67c0ec1`.
- Stream semantics pinned by `stream_c.rs`: end-of-stream is flag 0 with a
  zeroed release-less array; released streams are a typed dead state
  (`private_data == NULL` → error flag), never a crash.
- `Cargo.toml` grew a `stardf-columnar` **dev-dependency** only (lane-14
  unblock note) — invisible to foreign consumers of the rlib.

## Ready-made cgap-ffi diff (NOT applied — R1 gate)

Once R1 admits the blake3 compatibility receipt, `cgap/crates/cgap-ffi`:

```toml
# crates/cgap-ffi/Cargo.toml [dev-dependencies]
stardf-arrow-capi = { path = "../../../ggen-legacy/stardf/arrow/crates/stardf-arrow-capi" }
cgap-audit = { path = "../audit" }
```

plus the court file dropped in as
`crates/cgap-ffi/tests/stardf_consumer_court.rs` — courts 1 and 2 verbatim
from the rehearsal (they use only the capi's public surface and cgap-audit);
court 3's `dlopen` path resolves to the stardf workspace's
`target/debug/libstardf_arrow_capi.dylib` and must be re-pointed at wherever
the built dylib lives from cgap's tree. Modeled on
`cgap/crates/cgap-ffi/tests/alloc_courts.rs` per the W3 plan.

## Verdict

The W3 gate ("ONE real cgap integration test passing") is rehearsed 3/3
ALIVE on the final capi + final dylib; the gate itself remains OPEN until the
court lives inside cgap-ffi, which is blocked on R1 (operator admission of
the blake3 compatibility receipt).
