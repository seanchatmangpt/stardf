# ARD — stardf-arrow (architecture decision record)

Milestone: v26.10.31 (stardf reconstitution). Status: PARTIAL_ALIVE as of
2026-10-31 (completion lane 11, truth pass #2) — core 33/33 and all capi
targets green (22 unit + 6 stream_courts + 2 alloc_identity + 1 c_smoke +
2 cross_allocator, exit 0), bench emits min-of-5, verify.py 13==13 parity
exit 0 ×2 byte-identical; the one red gate is anti_vacuity fixture drift
(`ontology_pristine.ttl` vs `ontology.ttl`, 1/3); the W3 avec edge, W4
equivalence ladder and W5 CI replay remain OPEN. Receipts in the README
standing table and `docs/RECEIPTS.md`.

Canonical planning documents (read in this order):

- `/Users/sac/ggen-legacy/stardf/docs/jira/v26.10.31/01-master-plan.md`
- `/Users/sac/ggen-legacy/stardf/docs/jira/v26.10.31/02-wave-contracts.md`
- `/Users/sac/ggen-legacy/stardf/docs/jira/v26.10.31/03-substrate-map.md`
- `/Users/sac/ggen-avec/docs/jira/v26.10.31/01-stardf-reconstitution.md` (external pin `ArrowArrayStream`, :81)
- `/Users/sac/ggen-legacy/stardf/docs/jira/v26.10.31/04-falsifiers.md`

## 1. Mission

Reconstitute purrdf's 126-function C ABI over avec-arrow's physical layer (TermId u32 +
`Quad64`), manufacturing the ArrowArrayStream that no local substrate has, and admit every
capability through the house pipeline: contract TTL → rust-pack projection → C ABI →
equivalence ladder vs purrdf → CI determinism court → receipt. Predecessor purrdf stays
ALIVE (126 exports, ABI 0.16.0) as the oracle, not the subject.

## 2. Components

### 2.1 stardf-arrow-core

Crate: `crates/stardf-arrow-core` (exists on disk: `Cargo.toml`, `src/lib.rs`,
`src/layout.rs`; workspace manifest `stardf/arrow/Cargo.toml`).

Physical layer: `TermId = u32` with sentinels `BOTTOM=0x0`, `WILDCARD=0xFFFF_FFFF`
(cited substrate idiom: `~/avec/crates/avec-arrow/src/layout.rs:24-31`). Arena for TermId
interning modeled on the `MmuBuf` 16 KiB page arena (`~/avec/crates/avec-arrow/src/mmu_arena.rs:1,18`;
receipted 1,303 GiB/s, `PHASE_CHANGE_RECEIPTS.md:84`). 64-byte buffer allocation law
(`Layout::from_size_align(bytes, 64)`, `buffer.rs:3-12`). Quad storage is `Quad64`
`repr(C, align(64))` with compile-time size/align asserts pinned
(`layout.rs:34-69`, asserts at `:72-75`).

Divergence from purrdf (registered, deliberate):
- purrdf uses `TermId = NonZeroU32` with `QuadRow` 16 bytes (`crates/rdf-core/src/ir/term.rs:63`,
  `dataset.rs:143-154`); stardf adopts the avec u32-with-zero + 64-byte `Quad64` shape.
  The contract TTL already chose the avec shape — registered divergence, not a defect.
- purrdf's canonical `CanonArrow` is NOT reused. Canonicalization is re-derived as a
  pure function over stardf's physical layer; the port of purrdf's snapshot-canonicalization
  is parameterized on prefix (W4, `abi_signatures.rs:137` hardcodes `"purrdf_"`).

### 2.2 stardf-arrow-capi

Crate: `crates/stardf-arrow-capi` (**on disk and green as of 2026-10-31,
lane-11 runs**: `src/lib.rs`, `src/term.rs`, `src/stream_c.rs`,
`src/ownership.rs`, `include/stardf_arrow.h`, tests `c_smoke` (PASS, 1),
`cross_allocator_court` (PASS, 2), `stream_courts` (PASS, 6/6 — court_5
passes via the typed dead state, `stream_c.rs:69-75`), `alloc_identity`
(PASS, 2 — rewritten after the earlier compile-abort), unit 22 PASS; whole
target exit 0). Projection route via `rust-pack`
(`~/ggen-marketplace/packs/rust-pack/`, lib_rs/c_abi `.tmpl` templates with inlined SPARQL
front-matter, 5 gates, pass/fail witnesses) from
`authority/stardf/stardf_contract.ttl` (4951 triples, 126 fn + 2 kernel individuals,
13 TypeDefinitions, source digest `6ef9b4a0…`). Contract binding via the bridge query
`authority/stardf/rust-pack/queries/rust/010_contract_to_rust_items.rq` (W2 prerequisite;
without it every pack gate returns 0 rows).

Conventions mirrored from purrdf (contract source `include/purrdf.h`):
- Status enum append-only with frozen discriminants (`purrdf.h:344-440`, pinned by
  `crates/rdf-capi/src/abi.rs:215,2965`).
- Handle model: Box::into_raw with Send+Sync proofs (`crates/rdf-capi/src/handles.rs:22-97`).
- House release discipline reused from avec: CAS-swap release before private-data drop
  (`~/avec/crates/avec-arrow/src/c_abi.rs:70-77`), `OwnedPrimitive` boxing
  (`c_abi.rs:84-104`), parent→child release cascade (`c_abi_nested.rs:63-91`).
- W3 manufactures ArrowArrayStream (4 fn pointers: get_schema/get_next/get_last_error/release).
  Layout is UNKNOWN locally — zero references in avec/purrdf/ggen-legacy code; only pins are
  `avec/docs/jira/v26.10.31/01-stardf-reconstitution.md:81` and contract ttl:182. The Arrow C
  Data Interface stream spec must be sourced externally before implementation.

### 2.3 stardf-arrow-bench

Crate exists (`crates/stardf-arrow-bench`, standalone — not a workspace
member); runs and emits min-of-5 payloads (README standing table). Perf
fences as data modeled on
`~/avec/docs/BENCH_FLOORS.json`; arena/interning paths measured against the receipted
1,303 GiB/s MmuBuf floor. Branchless CC=1 gate idiom from
`~/avec/scripts/audit_branchless.sh` + `branchless_manifest.toml`.

## 3. avec-pattern adoptions (with citations from 03-substrate-map.md)

All citations from `/Users/sac/ggen-legacy/stardf/docs/jira/v26.10.31/03-substrate-map.md`:
1. `TermId = u32`, `BOTTOM=0x0`, `WILDCARD=0xFFFF_FFFF` — `crates/avec-arrow/src/layout.rs:24-31`.
2. `Quad64` `repr(C, align(64))` + compile asserts — `layout.rs:34-69` (`:72-75`).
3. Named-column batch `Batch::try_new` (T1) — `batch.rs:170`.
4. Zero-copy foreign ingest (T2) with `Misaligned`/`BufferTooShort` typed refusals — `foreign.rs:102-145`.
5. CAS-swap release discipline — `c_abi.rs:70-77, 104-154`.
6. Parent→child release cascade — `c_abi_nested.rs:14-24, 63-91`.
7. `MmuBuf` 16 KiB page arena (1,303 GiB/s receipted) — `mmu_arena.rs:1,18`; `PHASE_CHANGE_RECEIPTS.md:84`.
8. 64-byte alloc law — `buffer.rs:3-12`.
9. Zero-alloc CountingAlloc scoped tracking (dealloc NOT counted, :65-67) — `crates/audit/src/alloc_counter.rs:39-153` (W3 hazard, drives F3a).
10. `SLOTS: Mutex<()>` court serialization idiom — `avec-ffi/tests/alloc_courts.rs:30-133`.
11. Branchless CC=1 gate + manifest — `scripts/audit_branchless.sh`, `scripts/branchless_manifest.toml`.
12. Bench floors as data — `docs/BENCH_FLOORS.json`.
13. ArrowArrayStream: **absent** in avec (0 hits) — W3 manufactures it, never imports it.
14. Substrate-side 6-fn C ABI membrane — `avec-ffi/src/lib.rs:21-103`.
15. T1/T2 consumer gates — `~/avec/docs/CONSUMER_REQUIREMENTS.md` (purrdf pin `c0059bf5`).

Registered substrate defect (not a pattern to adopt): two BLAKE3 implementations
(`avec-blake3` crate + duplicate in `avec-arrow/src/blake3.rs`); stardf binds to exactly one.

## 4. Registered divergences from purrdf

| # | Divergence | purrdf | stardf | Authority |
|---|---|---|---|---|
| D1 | TermId representation | `NonZeroU32` (`term.rs:63`) | u32 + `BOTTOM=0x0`/`WILDCARD=0xFFFF_FFFF` sentinels | contract TTL chose avec shape |
| D2 | quad row | `QuadRow` 16B (`dataset.rs:143-154`) | `Quad64` 64B, align 64 | R4 law |
| D3 | CanonArrow | canonical `CanonArrow` type | not reused; canonicalization re-derived as pure fn over stardf layer, prefix-parameterized | W4 plan |
| D4 | license | MIT OR Apache-2.0 OR MulanPSL-2.0 (`Cargo.toml:70`) | inherits avec P28-SIL posture | R1 falsifier F-R1 |
| D5 | prefix | hardcoded `"purrdf_"` (`abi_signatures.rs:137`) | prefix parameterized | W4 plan |

## 5. Standing vocabulary table

| Term | Meaning in this repo |
|---|---|
| ALIVE | Exact admitted subject executed this session; command + exit cited. |
| PARTIAL_ALIVE | Some runs witnessed, some surfaces missing. |
| UNKNOWN | Not observed; includes ArrowArrayStream layout, `stardf-kernel` (NOT FOUND depth ≤2 under /Users/sac). |
| BLOCKED | A named blocking hop cited file:line. |
| BUILD_BROKEN | Compile/verify gate exits nonzero. |
| UNSUPPORTED | Route rejected with receipt (CNV pack CLI-shaped; wasi-json-abi-pack DEPRECATED). |
| REFUSED | Typed refusal (topology guard, unauthorized path). |
| OPEN | Planned, never executed — no receipt exists. Default for all stardf-arrow courts. |

License-skew compatibility receipt (R1) must exist before W3 crosses the avec boundary
(`04-falsifiers.md`, standing risks section).