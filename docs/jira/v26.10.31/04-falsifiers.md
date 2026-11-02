# Falsifiers — v26.10.31 (stardf reconstitution)

One falsifier per wave + standing risks. A falsifier that cannot flip is decoration.
Anti-vacuity law: each court ships with its positive-control mutation; a skip is loud.

Ledger note (2026-10-31, finishers-wave lane 16-falsifier-ledger): every row below is
re-anchored to an on-disk receipt; this lane's own court runs are receipted in
`arrow/evidence/lane16-falsifier-ledger-2026-10-31.json` (sha256
`bd7ef9c5d8c5198be9ae6f0b716e90fc2659a802260d776c3267a66563a4024d`). Subject:
`724d5425` (branch `fix/v26.9.29-ci-gates`), working tree uncommitted (coordinator
owns git). Receipt citations of the form `publish_stardf-receipt-<TS>` live in
`arrow/evidence/`.

Finalization note (2026-10-31 21:37 PDT, G04-finalization lane): every F-row re-witnessed
on the current tree (same subject `724d5425`, uncommitted); the avec-gate native-posture
re-run that closes F3/F3a, plus F1/F1a/F2/F4/F4b/F5/F5a/R1 same-tree reruns and the
R3/R4/KD-14 disk re-anchors, are receipted in
`arrow/evidence/g04-falsifier-finalization-2026-10-31.json` (standing ALIVE for the
ledger finalization; bounds per row below). KD-14 (F4b) re-anchored against
`arrow/docs/DIVERGENCES.md` — register text unchanged, row and register agree.

## W1 — Contract closure

**F1 (type census)**: `python3 scripts/verify_stardf_ontology.py` must FAIL (exit 1,
"Missing type mappings") on today's TTL, and PASS only after the extractor captures the 11
anonymous structs and the TTL is regenerated. A census that passes the pre-fix TTL is
vacuous.

STATUS (2026-10-31, lane 16; re-witnessed by G04-finalization): **WITNESSED (full ladder,
re-run on current tree).** Pre-fix FAIL receipt stands: exit 1, `[FAIL] Missing type
mappings` naming the 11 anonymous-struct typedefs. Post-fix PASS re-witnessed by lane 16
and again by the G04-finalization receipt (`f1-ontology-census-rerun`): exit 0,
`[ALL PASS] stardf_contract.ttl is fully verified and compliant` (type census 11 opaque +
11 anonymous-struct typedefs = 22 total; 126/126 purrdf.h functions mapped; SHACL 0
violations); earlier W1-2/W1-3 census receipts (22/22) stand. Both directions of the flip
are on record; the census is non-vacuous.

**F1a (anti-vacuity)**: `verify_stardf_ontology.py --mutate drop-type` — the in-memory
mutation must be refused (mutation step exits 1); if the mutated graph still passes →
`[FAIL] VACUOUS GATE`, exit 1.

STATUS (2026-10-31, lane 16; re-witnessed by G04-finalization): **REFUSED (witnessed x4).**
Lane 16 re-ran `python3 scripts/verify_stardf_ontology.py --mutate drop-type` on the
current tree: exit 0 (court passed), `[PASS] F1a: drop-type mutation REFUSED (mutant graph
fails census)`; G04-finalization re-ran it again, same result (fourth witnessing run). The
earlier refusal witnesses post-date the F1 fix, so the refusal holds against the passing
census too. Positive control in `scripts/verify_stardf_ontology.py`
(`run_mutation_drop_type`; mutant-still-passes → `[FAIL] VACUOUS GATE` exit 1; zero
victims → loud `[SKIP]` exit 1).

## W2 — Scaffold

**F2 (projection non-empty)**: the bridge query CONSTRUCT must yield ≥1 `rust:Function` per
contract subsystem and every rust-pack gate must return ≥1 row against the bridged graph
(witnessed 0 rows pre-bridge). A gate returning 0 rows must FAIL, not skip.

STATUS (2026-10-31, lane 16; re-witnessed by G04-finalization): **WITNESSED (re-run on
current tree).** As-built path is `stardf/arrow/queries/rust/run_bridge.py` (earlier
`scripts/run_bridge.py` citation corrected): exit 0, all 5 rust-pack gates coverage ≥1 row
(010_api_naming 15, 020_unsafe_safety_proof 13, 030_alignment_invariant 1,
040_ffi_unwind_guard 13, 050_workspace_inheritance 1), violations 0; bridged graph
non-empty (104 triples over 109 base) — identical output in the G04-finalization rerun
(`f2-bridge-gates-rerun`). Positive control stands: the drop-mapping mutation flips gate
`030` to fail, so the gates are not vacuously green. Pre-bridge 0-rows witness (F13 lane)
stands.

**F2a (regeneration determinism)**: `extract → ttl` twice; byte-identical output, else
FAIL (nondeterministic serializer = forbidden class).

STATUS (2026-10-31, lane 16): **WITNESSED (on-disk receipt).**
`arrow/evidence/replay-2026-10-31.json` pair 2 (`purrdf-contract-extract`): two in-place
extract→ttl runs, sha256 pre/post identical `8a9df387…c1ffb` (5264 lines), exit 0/0,
followed by the census verifier exit 0. Earlier multi-lane determinism witnesses stand.
Note (disclosed, not this lane's lease): the repo-root `scripts/extract_stardf_arrow.py`
currently exits 1 `[FAIL] Vacuous census: 0 #[no_mangle] stardf_* exports found in capi
src` against the post-one-package-port tree — the root extractor's paths are stale after
the port; owner = substrate-map lane.

## W3 — First integration slice

**F3 (stream court)**: avec-ffi consumer test ingests a stardf-capi-produced
ArrowArrayStream; passes on correct wiring; **flips to fail** under the mismatched-allocator
mutant (stardf bytes routed through avec's release callback). Mutant must be caught at
court time, not UB.

STATUS (2026-10-31, G04-finalization): **WITNESSED (both halves, current tree; regression
and repair both witnessed).** Stardf-side: mismatched-allocator mutant-cross-alloc flip
witnessed (`cross_allocator_court` 2/2, `stream_courts` 6/6; `arrow/docs/RECEIPTS.md`
LANE-11/LANE-16 rows). Avec-side witness chain, all on disk: green x4 native-posture runs
(20:26-20:50 local, `publish_stardf-receipt-20261101T032659Z`/`-T032924Z`/`-T033004Z`/
`-T035046Z.json`, subjects stardf `724d5425` + avec `4ede671`; capi edition-2021-pinned,
stardf-arrow-capi Cargo.toml:8); then a witnessed regression — exit 101 at 21:02-21:05 in
receipts `-T040225Z`/`-T040456Z`/`-T040521Z` + manufacturing receipt `avec-w3-gate`
(`arrow/evidence/v26.11.1-manufacturing-receipt.json`): the uncommitted workspace version
flip to v26.11.1 invalidated avec's cached build; `/Users/sac/avec` has NO rust-toolchain
pin, so its default resolves nightly-2026-06-22 (< rustc 1.98 floor). Then the
native-posture re-run closed it: `RUSTUP_TOOLCHAIN=nightly-2026-08-12 cargo test -p
avec-ffi --test stardf_consumer_court` in /Users/sac/avec → **exit 0, 3/3**
(court_1_export_stream_consume_get_schema_get_next_release,
court_2_allocation_identity_boxed_array_tags, court_3_dylib_loads_and_final_api_serves),
compiling stardf-arrow-core/stardf-arrow-capi v26.11.1 from THIS tree (the avec lane's
path dep into ggen-legacy), avec HEAD `4ede671718c9…`. Receipt:
`arrow/evidence/g04-falsifier-finalization-2026-10-31.json`
`f3-f3a-avec-gate-native-posture-rerun`. Bounds, disclosed: the 3/3 standing is the
pinned-toolchain **wrap posture** — a default-posture run in /Users/sac/avec still hits
the rustc 1.98 floor (witnessed, receipts above); pinning ≥1.98 there remains the
avec-side landing item, owner = avec-gate lane. Lane 05's own receipt file still never
landed.

**F3a (allocation identity)**: because CountingAlloc does not count deallocs
(`alloc_counter.rs:65-67`), the court asserts each released pointer was minted by the
allocator that owns the release path. A court asserting only "0 allocs" is vacuous here.

STATUS (2026-10-31, G04-finalization): **WITNESSED (both halves, same subject bounds as
F3).** Mutant-cross-alloc flip witnessed stardf-side (release of a pointer minted by the
other allocator detected by the ownership hook, not "0 allocs"; `cross_allocator_court`
2/2). Avec-side allocation-identity release-path half witnessed in the green native-posture
runs (court_2 `allocation_identity_boxed_array_tags` PASS, exit 0) and re-witnessed in the
G04-finalization wrap re-run (court_2 PASS, same 3/3 receipt as F3); the 21:02-21:05
regression and its wrap-posture close-out apply identically.

## W4 — Equivalence ladder

**F4 (header freeze)**: stardf snapshot regenerated from contract targetSymbols; test
asserts full prototype-set equality with the committed stardf header. Flips when a
prototype is added/changed/removed (GONE/NEW diff report, ported from
`abi_signatures.rs:239-286`).

STATUS (2026-10-31, lane 16 first witness; re-witnessed by G04-finalization): **WITNESSED
(both arms twice).** The court is `stardf/arrow/scripts/verify_header_freeze.py` +
`stardf/arrow/snapshots/prototypes.snapshot` (landed 17:55/17:56; no run receipt before
21:10). Lane 16 executed both arms on the current tree, exit 0 both:
(1) check mode (`--snapshot` freeze arm) — `[PASS] header-freeze: 13 contract symbols ==
13 header declarations, and committed snapshot is set+byte+count identical to the
regeneration (no GONE, no NEW)`;
(2) `--court` non-vacuity — baseline pristine PASS, then shims A (header-decl rename), B
(ontology targetSymbol rename), C (snapshot line drop) each flipped with the EXPECTED
GONE/NEW membership (probe `stardf_arena_intern`), restore proof sha256 byte-identical for
all three real files; mutations in temp copies only. Receipt:
`arrow/evidence/lane16-falsifier-ledger-2026-10-31.json` `f4-header-freeze-court`.
G04-finalization re-ran both arms on the same tree, identical results (check 13==13;
court 3/3 flips + byte-exact restore) — receipt
`arrow/evidence/g04-falsifier-finalization-2026-10-31.json` `f4-header-freeze-rerun`.
The frozen state now polices the consistently-edited ontology+header+impl triple that
`verify.py` alone cannot see.

**F4a (golden masters)**: each ported purrdf `abi.rs` test class must pass on stardf AND
fail when its subject behavior is deliberately mutated (one representative mutation per
class; untested mutation = unproven oracle).

STATUS (2026-10-31, lane 16): **N/A-YET (loud-missing).** No purrdf `abi.rs` test class
has been ported to the stardf arrow surface (no `abi`/`abi_signatures` test target in
stardf-arrow-capi; `arrow/docs/PRD.md` rows OPEN); no mutation-per-class court is possible
yet.

**F4b (GTS byte-freeze)**: stardf-side `.sha256` manifest over the copied vectors (purrdf's
gate does not police them, `check-corpus-frozen.py:52-55`); corrupted copy must FAIL the
manifest check.

STATUS (2026-10-31, lane 8 first witness; re-witnessed by G04-finalization): **WITNESSED
(columnar surface).** On-disk receipts: `cargo test -p stardf-columnar --test golden_bytes`
→ 0 — lane-8 run 6 passed / 0 failed; G04-finalization same-tree rerun exit 0, 5 passed /
0 failed / 1 ignored, where the ignored court is `regenerate_pinned_digests` (explicit-run
regeneration court, `-- --ignored --nocapture` — a loud by-design skip, not a silent one)
and the byte-flip positive control `flipping_a_byte_moves_the_digest` PASSED in that same
run (`arrow/docs/RECEIPTS.md` LANE-8 row; G04 receipt `f4b-golden-bytes-rerun`); manifest
`scripts/columnar_manifest.sha256`; KD-14 PARTIALLY CLOSED
(`arrow/docs/DIVERGENCES.md`: corrupted-copy CI gate + purrdf digest-ordering parity not
established; re-anchored — register text unchanged, row and register agree). The full
GTS-vector-set manifest parity with purrdf's gate (`check-corpus-frozen.py:52-55`) beyond
the columnar surface is not yet claimed.

## W5 — CI + receipt

**F5 (determinism court)**: two CI runs produce identical `receipt_sha256` +
`artifacts_sha256` (dsrust model, `ci.yml:167-196`); any drift FAILs.

STATUS (2026-10-31, lane 16; re-witnessed by G04-finalization): **witnessed-local;
CI-in-Actions loud N/A.** Local pairs on the current tree: `python3 scripts/verify.py
--emit-receipt` run twice by lane 16 — exit 0/0, receipt files byte-identical (`cmp` exit
0, file sha256 `cb57ce4e…62cef` both runs, `receipt_sha256` `ceaf5a25…514c9`); the
G04-finalization pair reproduced it on the same tree after intervening lane edits — exit
0/0, `cmp` exit 0, identical stdout, the SAME file digest `cb57ce4e…62cef` and
`receipt_sha256` `ceaf5a25…514c9` (receipt `f5-f5a-determinism-pair-rerun`), so the
digest is now stable across three receipt generations and two tree revisions. Disclosure:
this verifier emits `receipt_sha256` only — no `artifacts_sha256` key exists (re-checked
in the G04 receipt); the dsrust two-key shape is partially instantiated locally.
**CI-in-Actions N/A (loud):** `arrow/ci/stardf-arrow-reconstitution.yml` has never
executed in GitHub Actions (D2 unresolved, `arrow/docs/OPERATOR-DECISIONS.md`); no CI-run
standing is claimed anywhere.

**F5a (replay)**: extract→verify twice locally: byte-identical TTL + identical exit codes;
second run differing = nondeterminism FAIL.

STATUS (2026-10-31, lane 16; re-witnessed by G04-finalization): **WITNESSED (three receipt
generations).** `arrow/evidence/replay-2026-10-31.json` pair 1: two `verify.py
--emit-receipt` runs byte-identical (file sha256 `b0ad3281…0e154`, cmp exit 0, 13==13
parity) on its tree; lane 16's fresh pair on the current tree: byte-identical at file
sha256 `cb57ce4e…62cef` (the same digest the corpus-growth witness cites) — exit 0/0,
`cmp` exit 0, identical stdout. G04-finalization pair: byte-identical at the SAME
`cb57ce4e…62cef` — exit 0/0, `cmp` exit 0, identical stdout. Receipts:
`f5-emit-receipt-determinism-pair` (lane16 ledger) +
`f5-f5a-determinism-pair-rerun` (G04-finalization receipt).

## Standing risks (from 20-lane wave, lane risk-register)

- **R1 license skew** (purrdf MIT/Apache/MulanPSL vs avec P28-SIL sovereign): falsifier =
  dependency-edge gate that FAILs on a P28-SIL→MIT/Apache edge without a registered
  compatibility receipt. Gate on disk: `arrow/scripts/verify_stardf_licenses.py`. Flip
  cycles witnessed: exit 0 at c2 (98afd71d); **FAIL** at 20:44 (exit 1, 2 failures —
  `v26.11.1-manufacturing-receipt.json` `r1-license-gate` row and the 20:30 publish
  receipt); **PASS** in the 21:04/21:05 publish receipts; **FAIL again** at lane11-r2 lane
  start (exit 1: frozen-duckdb-sys/-builder "not in license register - loudly-missing"
  after the duckdb-pins lane added workspace keys), **PASS after registration** in
  `authority/stardf/license-compat.json`, re-witnessed after all lane edits
  (`arrow/evidence/lane11-license-carriage-r2-2026-10-31.json`
  `r1-gate-restored-exit-0`, passed=true, 21:14). G04-finalization re-ran the gate on the
  current tree: exit 0, "license-compat gate: PASS" (receipt `r1-license-gate-rerun`).
  Current standing: **PASS on the current tree**, witnessed three independent ways
  post-21:05 (21:04/21:05 publish receipts, lane11-r2 receipt, G04 rerun).
- **R2 cross-allocator release**: covered by F3/F3a (above; avec-side standing is the
  pinned-toolchain wrap posture, bounds per the F3 row — default-posture run still hits
  the rustc 1.98 floor until the avec-side toolchain pin lands).
- **R3 borrowed-pointer lifetime** (rowcursor/TermView borrows die at next-advance,
  `purrdf.h:1066-1078`): falsifier = use-after-advance court; debug-poison/guard, else
  mark UNKNOWN whether purrdf-class debug builds detect it. **Still OPEN** (re-verified by
  G04-finalization): no use-after-advance court exists on disk (grep over `crates/*/tests`
  + `crates/*/src`: zero matches for `use_after_advance`/`after_advance`; no debug-poison
  guard in stardf-arrow-core src). UNKNOWN stands.
- **R4 quad alignment**: compile-time assert witnessed on disk —
  `crates/stardf-arrow-core/src/layout.rs:59`
  `const _: () = assert!(core::mem::align_of::<Quad64>() == 64)` (plus the
  `size_and_alignment_laws` runtime test at :205; both halves re-read on disk by
  G04-finalization); stardf arena allocates at 64-byte granularity — an arena emitting
  non-64-aligned quads fails the build/assert court.
- **R5 aspirational-TTL claims**: no plan or receipt may cite `interopSurface
  "ArrowArrayStream"` or any ASPIRATIONAL claim as an existing fact until its wave lands
  real code; violation = overclaimed standing, receipt invalid. Discipline law; no
  receipt class (applies to this ledger too — every W3 avec claim above carries its
  subject bounds).
