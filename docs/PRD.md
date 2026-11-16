# PRD — stardf-arrow (product requirements)

Milestone: v26.10.31 (stardf reconstitution). Companion to `docs/ARD.md`.
Architecture: `../stardf/docs/jira/v26.10.31/01-master-plan.md` via
`/Users/sac/ggen-legacy/stardf/docs/jira/v26.10.31/02-wave-contracts.md`.

**Standing discipline**: nothing is claimed ALIVE without a command receipt.
Statuses re-read 2026-10-31 (completion lane 11, truth pass #2) from gates
re-run on the final tree this session. Every court ships a positive-control
mutation; a court that cannot flip is decoration; a skip is loud, never a
faked pass
(`/Users/sac/ggen-legacy/stardf/docs/jira/v26.10.31/04-falsifiers.md`).

## 1. Product

A reconstituted C ABI (`stardf-arrow-capi`) over the avec-arrow physical
layer, admitted from `authority/stardf/stardf_contract.ttl`, proven
equivalent to predecessor purrdf by the equivalence ladder below, receipts in
`evidence/` per house style. **As-built note 2026-10-31 (lane 11)**: the tree
implements 13 `#[no_mangle]` exports / 13 header declarations in exact parity
(`scripts/verify.py` census, exit 0), not the full 126-function contract; the
126-function census remains the closure target, not an observed fact.

## 2. Acceptance criteria — courts and falsifiers

Verify commands run from `/Users/sac/ggen-legacy` unless noted; the arrow
gates run from `stardf/arrow/`.

### W1 — Contract closure

| Court | Falsifier | Verify command | Standing (2026-10-31) |
|---|---|---|---|
| Type-census court | F1 — PASS only after the extractor captures the anonymous structs and the TTL regenerates. **Observed: exit 0, ALL PASS** — F1's original "must FAIL on today's TTL" premise is OUTDATED; the extractor landed and the TTL passes | `python3 scripts/verify_stardf_ontology.py` (from `/Users/sac/ggen-legacy`) → exit 0 | PASS |
| Anti-vacuity court | F1a — `--mutate drop-type` must be refused by the verifier. **Observed: the root script does not implement `--mutate`** (`error: unrecognized arguments: --mutate drop-type`, exit 2) — the court is ABSENT, not passing | `python3 scripts/verify_stardf_ontology.py --mutate drop-type` | ABSENT (loud-missing) |

### W2 — Scaffold

| Court | Falsifier | Verify command | Standing (2026-10-31) |
|---|---|---|---|
| Projection court | F2 — bridge CONSTRUCT yields ≥1 `rust:Function` per contract subsystem; every rust-pack gate returns ≥1 row (0 rows = FAIL). **No bridge/gate run observed — no receipt** | rust-pack gate queries via rdflib against the bridged projection graph | OPEN (no receipt) |
| Determinism court (projection) | F2a — `extract → ttl` twice, byte-identical output. **Observed: two runs, exits 0/0, outputs byte-identical** | `python3 scripts/extract_purrdf_ontology.py` ×2 + `cmp` | PASS (2× exit 0, byte-identical) |

### W3 — First integration slice

| Court | Falsifier | Verify command | Standing (2026-10-31) |
|---|---|---|---|
| Stream court | F3 — consumer test ingests a stardf-capi ArrowArrayStream; flips under a mismatched-allocator mutant. **Stardf side green: 6/6 pass, exit 0** — `court_5_use_after_release` now passes via the typed dead state (`core_of` returns None on NULL private_data, `stream_c.rs:69-75`; repaired per W8/F2 receipts). **Avec-side consumer test is outside this tree — no receipt** | stardf side: `cargo test -p stardf-arrow-capi --test stream_courts` → exit 0 (6 passed); avec side: avec-ffi consumer test | PARTIAL — stardf side 6/6 PASS; avec side OPEN |
| Allocation-identity court | F3a — each released pointer proven minted by the allocator owning the release path; "0 allocs"-only assertion is vacuous. **Observed: exit 0, 2 passed** (rewritten by W10 after the earlier compile-abort; clippy clean) | `cargo test -p stardf-arrow-capi --test alloc_identity` | PASS (2 passed, exit 0, lane 11 run) |

### W4 — Equivalence ladder

| Court | Falsifier | Verify command | Standing (2026-10-31) |
|---|---|---|---|
| Header-freeze court | F4 — prototype-set equality between contract-derived snapshot and committed stardf header; flips on add/change/remove | `cargo test -p stardf-arrow-capi --test abi_signatures` | OPEN (no `abi_signatures` test target exists in this tree) |
| Golden-master court | F4a — each ported purrdf `abi.rs` test class passes on stardf AND fails under one representative mutation per class | `cargo test -p stardf-arrow-capi --test abi` | OPEN (no `abi` test target exists) |
| GTS byte-freeze court | F4b — `.sha256` manifest over copied vectors; corrupted copy must FAIL | manifest check script over `vectors/*.gts` copies | OPEN (no `vectors/` or manifest script in this tree) |

### W5 — CI + receipt

| Court | Falsifier | Verify command | Standing (2026-10-31) |
|---|---|---|---|
| Determinism court (CI) | F5 — two CI runs produce identical `receipt_sha256` + `artifacts_sha256`; any drift FAILs | two-run comparison over `ci/stardf-arrow-reconstitution.yml` | OPEN (workflow committed; no CI run observed, no two-run receipt) |
| Replay court | F5a — extract→verify twice locally: byte-identical TTL + identical exit codes. **Half observed**: extract ×2 byte-identical (F2a receipt above); the verify-side repeat + compare has no receipt | `python3 scripts/extract_purrdf_ontology.py && python3 scripts/verify_stardf_ontology.py` ×2, compare | PARTIAL (extract half PASS; verify half no receipt) |

### Standing risks (gates, from 04-falsifiers.md)

| Gate | Falsifier | Verify command | Standing (2026-10-31) |
|---|---|---|---|
| R1 license-edge gate | FAILs on a P28-SIL→MIT/Apache dependency edge without a registered compatibility receipt | dependency-edge gate over Cargo graph | OPEN (no such gate exists in this tree; no compatibility receipt in `evidence/`) |
| R3 use-after-advance court | rowcursor/TermView borrows die at next-advance; debug-poison/guard must catch | use-after-advance consumer test (debug build) | OPEN — the adjacent use-after-RELEASE hazard is closed stardf-side (court_5 passes via the typed dead state, `stream_c.rs:69-75`); the use-after-ADVANCE rowcursor court itself has no target in this tree |
| R4 quad-alignment court | compile-time asserts stay pinned; non-64-aligned quads FAIL | `cargo build -p stardf-arrow-core` + `arena::tests::arena_allocation_is_64_byte_aligned` | PASS — asserts pinned (`layout.rs:56-70`); core lib run includes `arena_allocation_is_64_byte_aligned` ok and the layout offset asserts ok |
| R5 aspirational-TTL rule | no receipt may cite an ASPIRATIONAL claim as existing fact | receipt review against TTL standing annotations | OPEN (process gate; no violation observed this session — ArrowArrayStream surfaces in this tree are real code, so the rule is currently satisfied on this subject) |

### Substrate verifier (arrow-local gate)

`python3 scripts/verify.py` (from `stardf/arrow/`) → **PASS, exit 0** (two
runs byte-identical, `cmp` 0): 13 header declarations == 13 `#[no_mangle]`
exports, set equality both directions; ontology/header cross-check all
present. The `--mutate drop-export` control flips (mutation exit 1), so the
court is non-vacuous and green. The earlier 15-vs-13 "missing `['int',
'void']`" reading was a parser artifact on multi-line prototypes, since
fixed. `cargo clippy --workspace --all-targets -- -D warnings` → exit 0
(the alloc_identity and bench doc-comment offenders are cleared).

**Currently red**: `cargo test -p stardf-arrow --test anti_vacuity` → exit 1
(1 passed / 2 failed): `ontology_pristine.ttl` drifted from `ontology.ttl`
(artifact regenerated without fixture refresh). Everything else in
`cargo test --workspace --no-fail-fast` is green.

## 3. Non-goals / non-claims

- No claim that any court above has passed beyond the receipts cited in this
  document's tables; every other standing is OPEN/PARTIAL/BUILD_BROKEN as
  marked.
- CNV pack and wasi-json-abi-pack routes are rejected (UNSUPPORTED with
  receipt), not pending.
- `stardf-kernel`: NOT FOUND at depth ≤2 under /Users/sac — UNKNOWN beyond;
  no work order.
- `~/bcinr`: registered non-consumer (zero purrdf/stardf coupling); no work
  order.
- CanonArrow is not ported (D3 divergence); canonicalization is re-derived.
- purrdf remains the oracle, not the subject; its 126 exports are the census
  target, never modified.
- No claim of avec integration: the avec-ffi consumer test has not run on
  this subject; the integration edge stays OPEN.
