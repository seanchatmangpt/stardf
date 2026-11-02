# Wave Contracts — v26.10.31 (stardf reconstitution)

Companion to `01-master-plan.md`. Each wave: file-ownership, dispatch contract, exact edit
plan source (10-lane explore run `wf_91d14711-714`, 2026-10-31), verification ladder.

> **FINAL STATUS (2026-10-31, doc lane 2) — SUPERSEDED; historical receipt.** The W1-W5
> wave-contract structure in this document was superseded by the 80/20 re-scope
> (`arrow/docs/OPERATOR-DECISIONS.md`, third operator block, D15-D18: one-package rule,
> scripted bulk copy, honest court scope, ABI-v2 pending) and by the one-package execution
> that re-scope authorized (D15/D16: all ~53 purrdf crates straight-copied into the single
> `stardf/arrow` workspace). W1-W5 as designed were replaced by the finish-campaign shape —
> the landed c-chain from c3, the one-package port (`6cbb5b7`, `arrow/docs/COMMIT-PLAN.md`),
> through the final manufacturing receipt (`arrow/evidence/v26.11.1-manufacturing-receipt.json`,
> FINISH-CAMPAIGN re-bind; see also `arrow/evidence/finish-wave-2026-10-31.json` and
> `arrow/evidence/wave-consolidated-2026-10-31.json`, construction W1-W20 / finishing F1-F17).
> The per-wave standing block and wave sections below are preserved as the plan-of-record as
> it stood 2026-10-31, not the executed shape. **"Corrections learned" remains a valid method
> record**: its lessons (pack selection is an EXPLORE-lane output; census before scoping;
> reconstitute-vs-specify-first) were inherited by the campaign, not invalidated by it.
> Acceptance now reconciles to `14-definition-of-done.md` as amended by D15-D18.

**Status update (2026-10-31, plan-update lane 2)** — standing per wave, grounded in on-disk
receipts under `arrow/evidence/` (paths relative to `stardf/`):

- **W1 — ALIVE-local.** Extractor anon-struct branch landed
  (`scripts/extract_purrdf_ontology.py`); TTL regenerated deterministically
  (`authority/stardf/stardf_contract.ttl`, sha256 `1cc48a66d3f8...29e5`). F1 triply
  witnessed PASS: `arrow/evidence/replay-2026-10-31.json` pair 1 — both runs exit 0,
  pre/post sha256 identical `8a9df387...c1ffb` (11 structs / 16 enums / 126 functions),
  verifier exit 0 (4951 triples, 126-function census). F1a `--mutate drop-type` refused.
- **W2 — ALIVE-local, route CORRECTED mid-course.** Delivered via `rust-pack` through the
  bridge query `arrow/queries/rust/010_contract_to_rust_items.rq` + `run_bridge.py` (not
  clap-noun-verb / wasi-json-abi, both rejected — see Corrections learned).
- **W3 — PARTIAL_ALIVE.** stardf-side producer GREEN: `court.stream_courts` ALIVE-local 6/6,
  `court.cross_allocator` ALIVE-local 2/2, real-C `c_smoke` pass
  (`arrow/evidence/wave-consolidated-2026-10-31.json` standing_table + checks). **avec-side
  gate NOT RUN — BLOCKED-on-R1**
  (`arrow/docs/DECISION-BRIEFS/R1-blake3-receipt-admission.md`; R1 receipt gates W3 per
  `docs/OPERATOR-DECISIONS.md` §D6).
- **W4 — PARTIAL_ALIVE.** `c_smoke` ported
  (`arrow/crates/stardf-arrow-capi/tests/c_smoke.rs`); `abi_signatures` and GTS-vector
  ports OPEN.
- **W5 — PARTIAL_ALIVE.** CI draft landed
  (`arrow/ci/stardf-arrow-reconstitution.yml`); never run in GitHub Actions — UNVERIFIED
  beyond local.

One caveat carried by the receipts themselves: construction-wave (W1-W20) receipts exist only
in parent-session wave reports; the consolidated receipt quotes only on-disk sources
(`arrow/evidence/wave-consolidated-2026-10-31.json` `consolidation.source_limits`).

## W1 — Contract closure

**Owners**: `scripts/extract_purrdf_ontology.py`, `scripts/verify_stardf_ontology.py`,
`authority/stardf/stardf_contract.ttl` (regenerated, never hand-edited).

1. Extend extractor regex (`extract_purrdf_ontology.py:83`) to capture anonymous
   `typedef struct { ... } Name;` bodies — 11 exist in purrdf.h (PurrdfStr L1090,
   PurrdfTermView L1105, PurrdfGraphMatch L1146, PurrdfQueryGovernors L1167,
   PurrdfResourceVector L1209/1242, PurrdfGovernorTrip L1272, PurrdfGovernorEvidence L1277,
   PurrdfPartialCertificate L1304, PurrdfGovernedEntailmentEvidence L1339,
   PurrdfAbiVersion L1375, PurrdfCapabilities L1410). Regenerate TTL.
2. Extend verifier: type census (opaque structs AND value structs), two-directional
   (missing-in-TTL FAIL + phantom-mapping FAIL) + rename invariant
   (`target == predecessor.replace("Purrdf","Stardf",1)`).
3. Anti-vacuity: `--mutate drop-type` — mutation must FAIL; if it passes → `VACUOUS GATE`, exit 1.
4. Commit `authority/stardf/` (receipts bind to SHA).

**Scope decision (coordinator)**: value-struct class included (recommended) — exposes the
real drift; opaque-only would pass 11/11 today and stay blind to the 5 function-referenced
types until first used.

**Ladder**: extract → verify (exit 0, function + type census PASS) → `--mutate drop-type`
(mutation refused) → re-verify identical → commit.

## W2 — Scaffold (CORRECTED route)

Master plan named clap-noun-verb-crate-pack / wasi-json-abi-pack — **both rejected by lane
receipts**: CNV pack manufactures CLI binaries (wrong shape); wasi-json-abi-pack is
DEPRECATED. Route: **`rust-pack`** (`~/ggen-marketplace/packs/rust-pack/` — lib_rs/c_abi
templates, 5 gates, pass/fail witnesses).

**Prerequisite (new)**: bridge query `authority/stardf/rust-pack/queries/rust/010_contract_to_rust_items.rq`
— CONSTRUCT `stardf:AbiFunction`/`TypeDefinition` → `rust:Function`/`rust:Struct` projection
graph. Without it, every pack gate returns 0 rows (verified: gates parse and run via rdflib,
return empty against the 4951-triple contract).

**Note**: ARD.md names `queries/rust/*.rq` + `templates/rust/*.tera`; disk truth is queries
inlined in `.tmpl` front-matter. Follow disk, fix ARD text in the same diff.

## W3 — First integration slice

**Design inputs** (lane w3-stream-design): ArrowArrayStream struct layout **UNKNOWN locally**
— zero references in avec/purrdf/ggen-legacy code; only pins are
`avec/docs/jira/v26.10.31/01-stardf-reconstitution.md:81` and contract ttl:182. Requires the
Arrow C Data Interface stream spec (4 fn pointers: get_schema/get_next/get_last_error/release)
from an external source before implementation. House release discipline to reuse:
CAS-swap release before private-data drop (`avec-arrow/src/c_abi.rs:70-77`),
`OwnedPrimitive` boxing (`c_abi.rs:84-104`), parent-child cascade (`c_abi_nested.rs:63-91`).
Batch shape: 64-quad batch = 4096 bytes of `Quad64` (align 64, `layout.rs:34-75`).

**Court** (lane w3-court): avec-ffi path-dep on stardf-capi rlib; serialize with `SLOTS:
Mutex<()>` idiom (`alloc_courts.rs:30`); **CountingAlloc does not count deallocs**
(`alloc_counter.rs:65-67`) — court must assert allocation-side identity (which allocator
minted the released pointer), not absence of allocation. Positive-control mutant:
mismatched-allocator release must be caught.

## W4 — Equivalence ladder

- Port `abi_signatures.rs` canonicalization verbatim (pure fns over `&str`); parameterize
  the hardcoded `"purrdf_"` prefix (`abi_signatures.rs:137`). Regenerate a stardf snapshot
  from the contract's targetSymbols — the purrdf snapshot's bytes are NOT consumable
  (purrdf symbols/types throughout).
- `c_smoke.rs` is a driver (nested cargo build → locate cdylib from Cargo JSON → `cc
  -std=c11` → link → DYLD_LIBRARY_PATH) — parameterize crate name/header.
- GTS vectors: 39 top-level `*.gts`+`*.expected.json` pairs, binary CBOR-seq, consumed via
  `purrdf-gts` reader (`crates/gts/tests/frozen_canonical_bytes.rs:14-38`). **purrdf's
  `check-corpus-frozen.py` deliberately does NOT police them** (`:52-55`) — stardf copies
  need their own `.sha256` manifest + roots.toml equivalent.

## W5 — CI + receipt

- Model: `ci.yml:111-198` dsrust-reconstitution (pinned specimen checkout + SHA assert +
  cleanliness assert + two-run determinism court comparing receipt_sha256/artifacts_sha256).
- purrdf pins at SHA `0da0578306983da851a9073337d57e2e2c41ab3a`; verifier takes `--header`
  already (`verify_stardf_ontology.py:21`) — CI passes the flag, no script path edit needed
  for CI (fix the default anyway for local ergonomics).
- Runner deps: `pip install rdflib pyshacl` (no existing pip step in ci.yml).
- Workflow-count gate: adding a new workflow file requires updating
  `.github/expected-workflows.txt` (currently 3 entries) — or add the job to `ci.yml`
  instead (no gate delta). Prefer same-file job.
- Receipts: `evidence/stardf/` per house style (`local-docs-verifier.json` exemplar: schema,
  subject tree_digest, typed checks[] with counts, honestly-failed checks, standing,
  nonclaims[]). Replay pair: extract → verify → extract → verify, byte-identical TTL +
  identical exit codes.

## Corrections learned (mid-course, 2026-10-31)

Three plan corrections landed during execution; recorded so the next planning cycle inherits
them:

1. **W2 pack route was wrong in the master plan.** clap-noun-verb-crate-pack and
   wasi-json-abi-pack were both rejected by lane receipts (CNV manufactures CLI binaries —
   wrong shape; wasi-json-abi is DEPRECATED). Corrected route: `rust-pack` via the bridge
   query `arrow/queries/rust/010_contract_to_rust_items.rq`. Lesson: pack selection is an
   EXPLORE-lane output, not a master-plan output — lane receipts outrank plan text.
2. **The anon-struct count was 11, not 5.** W1 was scoped against 5 known opaque types; the
   extractor branch found 11 anonymous `typedef struct { ... } Name;` bodies in purrdf.h
   (PurrdfStr L1090 through PurrdfCapabilities L1410). Lesson: census before scoping —
   the value-struct class decision (coordinator, above) is what exposed the real drift;
   opaque-only would have passed 11/11 blind.
3. **ArrowArrayStream is new contract, not reconstitution.** The stream struct has zero
   references in avec/purrdf/ggen-legacy code — it cannot be "reconstituted", only
   specified fresh (4 fn pointers: get_schema/get_next/get_last_error/release). The W3 lane
   spent its design pass discovering this boundary. Lesson: classify each W3 target as
   reconstitute vs. specify-first before dispatch; the two have different verification
   ladders (spec-derived courts vs. parity courts).
