# stardf Reconstitution Plan — v26.10.31

Ticket: GL-STARDF-001 (PARTIAL_ALIVE) · Authority: authority/stardf/stardf_contract.ttl
Predecessor: ~/purrdf (126 C ABI exports, ABI 0.16.0, cbindgen) · Substrate: ~/avec (ALIVE, ~310 tests pass)
bcinr: registered-non-consumer (AGENTS.md:351 declares RDF out of scope) — no work order.

## Standing summary (as-built truth pass, 2026-10-31; plan-update lane 1)

> **FINAL TRUTH PASS (doc lane 1, 2026-10-31, post-finish-campaign): rows below the
> banner predate the finish campaign and the c3→c4r commit chain. FINAL state —
> coordinator dispatch + on-disk receipts:** commit chain LANDED c1..c4r, HEAD
> `724d5425` (re-probed exit 0 this lane; stardf/ tracked, 12,159 files;
> `authority/stardf/` tracked) with finish residue staged pre-c5r
> (`stardf/arrow/docs/C5R-C6-STAGING.md`, 1,448 paths; c5r/c6 coordinator-owned).
> Four first-class targets court-green: arrow (RECEIPTS LANE-11 + G04), duckdb-scan
> (`evidence/g10-scan-courts-rewitness-2026-10-31.json` exit 0 x2, ALIVE), n3
> (89/0 across 5 targets, `/tmp/h02_n3_full.log` @21:44 — log-standing), sql
> (coordinator-final; receipt in flight per `docs/V26-11-1-GAPS.md`). avec gate
> 3/3 WITNESSED (`evidence/g04-falsifier-finalization-2026-10-31.json` exit 0
> 3/3 + ladder receipt `045124Z` avec-ffi-court PASS, default posture). `=2.0.4`
> x3 RATIFIED (D13 RESOLVED / D19 DECIDED; pins `Cargo.toml:282-284`; G6 PASS in
> every ladder receipt). One-package port c3=`6cbb5b7` (53 crates straight-copied,
> 47-member workspace) subsumes the vendoring-era rows below.

Grounded in on-disk receipts only — `stardf/arrow/evidence/*.json`
(wave-consolidated / columnar-wave / manufacturing / w1-closure, 2026-10-31),
`stardf/arrow/docs/RECEIPTS.md` (LANE-9/11/16 rows),
`stardf/arrow/docs/ABI-FREEZE.md`.
No gate re-run by this lane; citations, not re-derivations.
- Frozen ABI: 13 `stardf_*` exports, ALIVE per `stardf/arrow/docs/ABI-FREEZE.md`
  (nm + header syntax + c_smoke receipts in-file). verify.py three-way parity
  13==13==13 witnessed exit 0 with byte-identical `--emit-receipt` replay pair
  (sha `cb57ce4e…`) across trees 50→58→93 files
  (`evidence/wave-consolidated-2026-10-31.json`, `columnar-wave-…json`,
  `w1-closure-…json`).
- Workspace ladder: NOT green on the latest receipts. capi courts green
  (stream 6/6, cross-alloc 2/2, c_smoke 1, alloc_identity 2; core 33 pass;
  clippy/fmt exit 0 — `docs/RECEIPTS.md` LANE-11 row), but `cargo test
  --workspace` witnessed exit 101: capi lib cfg(test) E0308 x2 compile-abort
  under nightly 1.101 (manufacturing receipt, `commands` block), and
  anti_vacuity 1/3 (fixture-drift class) is the latest consistent witness.
  [FINAL: predates — LANE-8 witnessed workspace 82/0/1i exit 0 with anti_vacuity 3/3
  green (fixture drift repaired by its owning lane); post-c3, G9 `cargo check
  --workspace` exit 0 in every ladder receipt 20:26→21:51 (`docs/V26-11-1-GAPS.md`).]
- Columnar: LANDED as a vendored workspace member — `crates/stardf-columnar`
  with own `Cargo.toml`, vendored from purrdf `0da05783` with the u32 term-id
  seam, licenses preserved (`crates/stardf-columnar/PROVENANCE.md`; member
  list in `stardf/arrow/Cargo.toml`, lane-9 comment). No gate receipt for the
  vendored crate yet — standing PARTIAL_ALIVE (code), gate UNVERIFIED.
  [FINAL: predates c3 — the one-package port (`6cbb5b7`, 53 crates) landed the full
  47-member workspace; columnar courts green post-port (G04 golden_bytes 5/0/1i on the
  current tree; G10 columnar-oracle tier-3 exit 0, counts 1030,400,108,108,9).]
- W1 contract closure: gate machinery LANDED (type census + `--mutate
  drop-type` in `scripts/verify_stardf_ontology.py`; the 5 struct mappings
  present in `authority/stardf/stardf_contract.ttl`). The F1 ladder (FAIL →
  extract → PASS → mutate-refused) is recorded RUN-UNWITNESSED — no on-disk
  receipt of the full ladder (`evidence/w1-closure-2026-10-31.json` nonclaims
  + K3). `authority/stardf/` remains untracked; receipts bind to tree digests,
  not SHAs.
  [FINAL: predates — G04 (21:39) witnessed the ladder on the CURRENT tree: F1 census
  22/128/126 SHACL-0 exit 0, F1a drop-type REFUSED, F2 5/5, F4 header-freeze + 3-shim
  anti-vacuity, F5/F5a byte-identical replay (sha `cb57ce4e…`); `authority/stardf/` is
  tracked (this lane's `git ls-files` witness; contract landed c2).]
- Integration edge: avec gate NOT RUN — avec-arrow now consumed as a git-rev
  dep (`rev 67c0ec1`, `stardf/arrow/Cargo.toml` [workspace.dependencies]) but
  no avec integration test witnessed; R1 license gate witnessed exit 1,
  PENDING-OPERATOR (`docs/DECISION-BRIEFS/R1-blake3-receipt-admission.md`).
  W3 stays BLOCKED-on-R1. BLAKE3 KD-08 PENDING-SIGN-OFF
  (`docs/DECISION-BRIEFS/D10-blake3-flip.md`). No commit landed; parent HEAD
  still `c9f8028`, stardf/ untracked.
  [FINAL: every clause predates — avec gate 3/3 WITNESSED (G04 exit 0 3/3 + ladder
  `045124Z` avec-ffi-court PASS default-posture); R1 CLOSED (register 31 deps /
  40 receipts / 0 drafts, exit-0 re-runs), W3's BLOCKED basis gone; chain c1..c4r
  landed, HEAD `724d5425`, stardf/ tracked.]
- SQL/duckdb: PLANNED-NOT-BUILT per
  `stardf/docs/jira/v26.10.31/07-port-master-plan.md` (B0–B3 blocker chain;
  no SQL or duckdb artifact or gate witnessed).
  [FINAL: predates — four first-class targets court-green per coordinator-final:
  arrow/sql/n3/duckdb-scan. duckdb-scan: G10 exit 0 x2 (6 courts + 6 lib, ALIVE);
  n3: 89/0 across 5 targets (`/tmp/h02_n3_full.log` @21:44); sql: receipt in flight
  (`docs/V26-11-1-GAPS.md`); arrow: LANE-11 + G04.]
- Standing vocabulary preserved: every ALIVE above is a LOCAL gate witness,
  never external production standing; no CI Actions run witnessed.

## Waves (as-built statuses, 2026-10-31 truth pass)
W1 (contract closure): PARTIAL. Gate machinery landed — type census (opaque +
  anonymous, two-directional, rename invariant) and `--mutate drop-type` in
  `scripts/verify_stardf_ontology.py`; the 5 struct mappings
  (StardfTermView, StardfGraphMatch, StardfQueryGovernors,
  StardfGovernorEvidence, StardfPartialCertificate) present in
  `authority/stardf/stardf_contract.ttl`. NOT witnessed: the full F1 ladder
  (FAIL → extract → PASS → drop-type refused) has no on-disk receipt
  (`evidence/w1-closure-2026-10-31.json`, nonclaims + K3); the closing git
  commit of `authority/stardf/` is not executed — receipts still bind to tree
  digests, not SHAs.
  [FINAL: predates — the F1/F1a/F2/F4/F5 ladder WAS witnessed on the current tree
  (G04, 21:39: census 22/128/126 SHACL-0 exit 0; drop-type REFUSED exit 0; F2 5/5;
  F4 header-freeze + 3-shim anti-vacuity; F5/F5a replay pair sha `cb57ce4e…`);
  `authority/stardf/` is tracked.]

W2 (scaffold): LANDED as PARTIAL_ALIVE (local). 13-export frozen ABI per
  `stardf/arrow/docs/ABI-FREEZE.md` (ALIVE; nm/header/c_smoke receipts
  in-file). Gate receipts: verify.py exit 0 with 13==13==13 three-way parity
  + byte-identical `--emit-receipt` pair sha `cb57ce4e…` (wave-consolidated /
  columnar-wave / w1-closure receipts); core 33 pass, stream 6/6, cross-alloc
  2/2, c_smoke 1, alloc_identity 2 (docs/RECEIPTS.md LANE-11 + manufacturing
  receipt). Residual reds on the same receipts: capi lib cfg(test) E0308 x2
  compile-abort (nightly 1.98→1.101 drift), anti_vacuity 1/3 fixture drift.
  [FINAL: predates — both reds repaired: capi all-targets green (LANE-11) and
  anti_vacuity 3/3 green inside workspace 82/0/1i exit 0 (LANE-8); post-c3 compile
  witness: G9 `cargo check --workspace` exit 0 through the 21:51 ladder receipt.]

W3 (first integration slice): BLOCKED-on-R1. avec-arrow pinned as git-rev dep
  (rev 67c0ec1) but no avec integration test witnessed anywhere in the
  receipts; R1 license gate witnessed exit 1, PENDING-OPERATOR
  (`docs/DECISION-BRIEFS/R1-blake3-receipt-admission.md`). Gate remains: ONE
  real avec integration test passing.
  [FINAL: predates — the avec consumer court is GREEN: G04 exit 0, 3/3 courts from
  `/Users/sac/avec` against this tree (v26.11.1 core+capi compiled from it); ladder
  `045124Z` rows avec-ffi-court PASS (default posture). The gate is MET.]

W4 (equivalence ladder): OPEN. `abi_signatures`/`abi`/`vectors/` targets do
  not exist (`docs/RECEIPTS.md` nonclaims) — no header-freeze court, no
  golden-master replay, no GTS vector run witnessed.
  [FINAL: partially predates — the header-freeze court IS witnessed (G04: F4 check +
  3-shim anti-vacuity, each flipped, exit 0); golden-master replay + GTS vectors remain
  deferred (D17).]

W5 (CI + receipt): OPEN. `ci/stardf-arrow-reconstitution.yml` never witnessed
  executing in Actions; all gate standing is local runs only (all four
  evidence receipts' nonclaims). Two-run replay IS witnessed locally
  (`--emit-receipt` pairs, sha `cb57ce4e…`); SHA-pinned CI determinism court
  and the `authority/stardf/` commit remain unlanded.
  [FINAL: partially predates — two-run replay re-witnessed on the CURRENT tree (G04
  F5/F5a, sha `cb57ce4e…`); CI-in-Actions still never executed (no CI standing);
  c5r/c6 residue staged, coordinator-owned (`docs/C5R-C6-STAGING.md`).]

Columnar wave (added per 07-port-master-plan): LANDED — vendored workspace
  member (`crates/stardf-columnar/`, purrdf `0da05783`, u32 term-id seam,
  licenses preserved per `PROVENANCE.md`). Gate receipt for the vendored
  crate: NONE on disk — G_COL_EXIT evidence not yet witnessed.
  [FINAL: predates c3 (one-package port) and the gate gap is closed: columnar courts
  green post-port — G04 golden_bytes 5/0/1i on the current tree; G10 columnar-oracle
  tier-3 exit 0 (counts 1030,400,108,108,9).]

SQL wave / duckdb-scan wave: PLANNED-NOT-BUILT (07-port-master-plan §2–3;
  gated behind G_COL_EXIT / G_SQL_EXIT; B3 PENDING-OPERATOR).
  [FINAL: predates — sql + duckdb-scan court-green per coordinator-final (G10 exit 0
  x2 for duckdb-scan; sql receipt in flight per `docs/V26-11-1-GAPS.md`); `=2.0.4` x3
  RATIFIED (D13 RESOLVED / D19 DECIDED), superseding the B3 PENDING-OPERATOR basis.]

W2 (scaffold): ggen-project stardf-core + stardf-capi from the TTL via clap-noun-verb-crate-pack
  /wasi-json-abi-pack; handle-first families: buffer, dataset, cursor, error (what avec needs:
  interning, pattern match, term view). Hand-written residue in custom_handlers.rs only.

W3 (first integration slice): fn_avec_export_stream → ArrowArrayStream producer in stardf-capi;
  consumer court in avec-ffi modeled on alloc_courts.rs with a cross-allocator release test
  (R2 falsifier). Gate: ONE real avec integration test passing.

W4 (equivalence ladder): port abi_signatures.snapshot as header-freeze court; replay purrdf
  golden-master classes; c_smoke-style real-C-host test; GTS vectors as byte-frozen oracles.

W5 (CI + receipt): SHA-pinned determinism court modeled on dsrust-reconstitution job; fix
  hardcoded /Users/sac/purrdf path in verifier; update .github/expected-workflows.txt if a
  workflow file is added; evidence/stardf/ receipts + two-run byte-identical replay.

## Risks (lane 19)
R1 license skew: purrdf MIT/Apache/MulanPSL vs avec P28-SIL sovereign — needs compatibility
   receipt gate. R2 cross-allocator release mismatch at Arrow boundary (W3 court). R3
   rowcursor borrowed-pointer lifetime (poison/guard court). R4 quad 64-byte alignment
   assumptions (compile-time asserts exist; keep pinned).

## Law
Every wave ends with a manufacturing receipt (commands, exits, SHA). No slice claimed ALIVE
without the real gate output. Genre: strangler-fig — route to purrdf by default, shift per
family as each is admitted.
