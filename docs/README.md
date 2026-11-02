# stardf Docs Index — Lane F, 2026-10-31

One-line status per file across both stardf doc trees. Statuses come from each file's
landed header banner (CURRENT / SUPERSEDED / DEFERRED / RESOLVED); files without a
banner carry their own stated standing plus a note. Read-only lane: this file is the
only file this lane owns; no git commands were run.

## docs/jira/v26.10.31/ (planning pack)

- `01-master-plan.md` — CURRENT — FINAL TRUTH PASS banner; plan of record, chain
  c1..c4r landed, HEAD `724d5425`, four first-class targets court-green.
- `02-wave-contracts.md` — SUPERSEDED — banner: W1–W5 wave shape replaced by the
  80/20 re-scope (D15–D18, `OPERATOR-DECISIONS.md`); kept as historical receipt.
- `03-substrate-map.md` — CURRENT — avec/purrdf surfaces re-read after one-package
  landing (`6cbb5b77`); superseded rows fixed inline.
- `04-falsifiers.md` — CURRENT — F-rows re-anchored to on-disk receipts; G04
  finalization banner re-witnesses every F-row on subject `724d5425`.
- `05-next-steps.md` — CURRENT — sixth refresh at campaign terminal state; decision
  rows live in `OPERATOR-DECISIONS.md`.
- `06-phase0-columnar-sql-duckdb.md` — RESOLVED — phase-0 inputs consumed and landed
  (columnar vendored, D12 probe, D13 precondition, freeze ticket, R1 landed red).
- `07-port-master-plan.md` — CURRENT — §0 truth-pass separates WITNESSED vs OPEN;
  remainder is original plan text.
- `08-vendoring-plan.md` — RESOLVED — banner: EXECUTED via the 80/20 bulk copy; all
  53 crates landed as `stardf-*`.
- `09-sql-adaptation-scope.md` — SUPERSEDED — banner: superseded by the one-package
  rule (D15); Option A/B/C decision moot; analysis kept as method record.
- `10-duckdb-rehome-plan.md` — RESOLVED — banner: Status RESOLVED — D13 decided, all
  three frozen-duckdb crates `=2.0.4` (D19).
- `11-blake3-arbitration.md` — RESOLVED — banner: OUTCOME D10 DECIDED, Option A
  retained (blake3 stays bound, purrdf-hash NOT vendored); Option B deferred.
- `12-full-parity-campaign.md` — DEFERRED — banner: DEFERRED by the 80/20 re-scope
  (D15–D18); not dispatchable without operator re-admission.
- `13-v26111-target-ledger.md` — SUPERSEDED — banner: partially superseded by the
  re-scope; `14-definition-of-done.md` is the spec of record.
- `14-definition-of-done.md` — CURRENT — AUTHORITATIVE spec of record; Amendment 1
  (80/20 re-scope) + Amendment 2 (D13 `=2.0.4` pins, naming divergence).
- `PLANNING-BEST-PRACTICES.md` — CURRENT — method record, no banner; discipline
  rules standing; §2 W1–W5 structure historical (superseded in practice per `02`).

`C5R-C6-STAGING.md` is NOT in this tree — it lives at
`stardf/arrow/docs/C5R-C6-STAGING.md`.

## arrow/docs/ (as-built + campaign record)

- `V26-11-1-VERDICT.md` — CURRENT — final verdict of record; reconciliation pass 2
  (lane H06, ~22:29, newest file), supersedes G11 pass 1; rows re-read on disk.
- `V26-11-1-GAPS.md` — CURRENT — G20 consolidator (21:53, as-of): PARTIAL_ALIVE
  campaign, publish BLOCKED at G8 only; rows reconciled by VERDICT pass 2.
- `ARD.md` — CURRENT — PARTIAL_ALIVE truth pass #2; no later banner; W3/W4/W5 rows
  overtaken by finish-campaign receipts but not banner-updated in-file.
- `PRD.md` — CURRENT — truth pass #2; as-built note: 13 exports == 13 header decls.
- `ABI-FREEZE.md` — CURRENT — v1 13-symbol surface enforced; v2 surface of record is
  PENDING-OPERATOR (D18), adoption unsigned.
- `COLUMNAR-ARD.md` — CURRENT — lane 8 truth pass #3; columnar landed with
  golden/seam/integration receipts.
- `DIVERGENCES.md` — CURRENT — KD-nn registered-divergence ledger; per-row status
  re-read (lane 11 truth pass #2).
- `RECEIPTS.md` — CURRENT — final ledger rebuild (doc lane 15, ~22:05): construction
  table + finish-campaign ledger; every cited receipt file re-read.
- `OPERATOR-DECISIONS.md` — CURRENT — decision register D1–D21: D15–D18 re-scope,
  D19–D21 finish record; sign-off table.
- `COMMIT-PLAN.md` — CURRENT — Lane 14 six-commit split staged for coordinator;
  c3 `6cbb5b77`, c4/c4r `724d5425` landed; c5r/c6 pending per staging doc.
- `SCAN-COMPLETION.md` — RESOLVED — design assembly consumed: scan courts C1–C6
  green twice (G10 receipt), C6 mutant flip; `=` pin resolved by D19.
- `avec-rehearsal.md` — RESOLVED — W3 rehearsal ALIVE 3/3; its one OPEN row (real
  avec-ffi edge) closed by the G04 finalization receipt (21:39).
- `C5R-C6-STAGING.md` — CURRENT — coordinator staging input (1,448 paths, ~21:45);
  c5r/c6 owned by coordinator, not yet landed at `724d5425`.
- `BENCH_BASELINE.json` — CURRENT — self-declared UNVERIFIED: local min-of-5, one
  machine, not CI-qualified; MIN round is the receipt.
- `SUPERSEDED-CLAIMS.md` — CURRENT — repo's superseded-claims ledger (reversal
  record, not a changelog).

### arrow/docs/DECISION-BRIEFS/

- `D10-blake3-flip.md` — RESOLVED — D10 DECIDED (Option A) via the
  `11-blake3-arbitration.md` outcome banner; brief's PENDING status superseded.
- `D12-D13-dispatch.md` — RESOLVED — D12/D13 dispatched; D13 RESOLVED per D19
  (`=2.0.4` x3 operator-ratified); brief's PENDING status superseded.
- `R1-blake3-receipt-admission.md` — RESOLVED — R1 license gate resolved; blake3
  edge admitted (DoD licensing note; register closure landed in c2r).

### arrow/docs/design/ (predecessor design references)

- `purrdf-change-path-allocations.md` — CURRENT — purrdf SHACL change-path
  allocation closed form.
- `purrdf-retrieval-ladder.md` — CURRENT — purrdf retrieval composition
  (plan/compile/execute/fuse).

### arrow/docs/archive/

`archive/c3-dual-site/slice11-umbrella/` — `Cargo.toml` + `lib.rs` of the slice-11
umbrella facade (`stardf` package, lib name `purrdf`), vendored from purrdf
`crates/purrdf` at pinned rev `0da05783`, dual-site copy-sweep era. ARCHIVED:
superseded by the one-package workspace layout (D15); the in-tree umbrella lives in
`crates/stardf/`. `lib.rs` carries the STARDF RE-DERIVATION BANNER.

## Cross-references

- Spec of record: `stardf/docs/jira/v26.10.31/14-definition-of-done.md`
- Final verdict: `stardf/arrow/docs/V26-11-1-VERDICT.md` (newest, ~22:29)
- Decision authority: `stardf/arrow/docs/OPERATOR-DECISIONS.md` (D1–D21)
- Receipt ledger: `stardf/arrow/docs/RECEIPTS.md`
- Evidence receipts: `stardf/arrow/evidence/` (cited throughout the trees)
