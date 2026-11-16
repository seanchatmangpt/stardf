# SIP-ALT: Metadata-Driven Access Paths from Sealed Footers

Lane RDF-3 SIP-ALT receipt. Branch `main`, one canonical checkout at
`/Users/sac/stardf`. All commands run 2026-11-01 under
`CARGO_TARGET_DIR=/tmp/r3m-target` (lease deleted via python3
`shutil.rmtree` at close; verified absent).

## Differentiator (design-around law)

OST P2/P3/P4 claim RUNTIME sideways information passing — dynamic
constraint propagation DURING join execution. This lane's mechanism is
PRECOMPUTED STRUCTURAL PLANNING: triple files are sealed at write time
with a per-file per-predicate count table; the planner selects join
order + filter placement FROM SEALED METADATA before execution. Cardinality
bounds are known statically per file, so runtime SIP is unnecessary.
Cited as NOT-practiced: US 11,216,456 / US 11,409,698 claim classes —
runtime constraint propagation between join operators during query
execution; nothing here propagates constraints at runtime between
joins (ordering fixed before the first triple is read, from sealed bytes).

## Landed surface

- `arrow/crates/stardf-planner/` (NEW crate, workspace member):
  - `src/footer.rs` — `PLNF` sealed footer: per-file per-predicate counts
    + row count, FNV-1a-64 seal (A02 seal class, cross-referenced
    read-only from cgap-parquet `crates/cgap-parquet/src/footer.rs`).
    Runtime divergence channel: `FooterFault::CountDivergence` fires when
    the ACTUAL per-predicate count observed during a scan differs from the
    sealed count, and `SealMismatch` on any seal-span byte flip.
  - `src/plan.rs` — s-p-o patterns over `Term::Const/Var(u32)` term ids;
    cardinality bounds from the footer (bound predicate -> sealed
    per-predicate count; free predicate -> file row count); greedy
    smallest-bound-first join order (filter placement: bound-predicate
    patterns land first). The static analog of SIP.
  - `src/exec.rs` — naive (written order) vs planned executors, both
    instrumented with an intermediate-tuple counter (every
    (triple x partial-binding) candidacy counts once).
- `arrow/Cargo.toml` — one membership line (outside the strict lease;
  required by "workspace member"; disclosed here).

## Courts (all ran 2026-11-01, exit 0)

- `differential_court` — 3-file seeded corpus, 2-hop join (4 patterns,
  joins on ?x and ?r). IDENTICAL solutions naive vs planned
  (structural equality asserted); planner reorders asserted (non-vacuous
  selection); intermediate tuples naive=306290 planned=131890,
  reduction 56.9% (asserted >25%).
- `poison_court` — anti-vacuity flip in /tmp sandbox: KNOWS count in
  f0 poisoned 40->41, seal RE-COMPUTED so parse passes (metadata-only
  poison); `scan_verified` must fire `CountDivergence` typed.
  Witnessed: `flip witnessed: CountDivergence pred=1 sealed=41 actual=40`.
  Baseline control: unpoisoned corpus runs clean end-to-end.
- `bench_depths` (PARETO) — depths [2,3,4], debug profile, seeded corpus:
  naive/planned tuples 267050/92650 (65.3%), 271410/92650 (65.9%),
  271410/92650 (65.9%); planned strictly < naive at every depth,
  identical solutions at every depth. Debug-profile numbers are TARGETs
  for a release bench; method: instrumented candidacy counter in
  `tests/bench_depths.rs`.

## Honest boundary

Static planning cannot adapt to data skew WITHIN a file: the footer
carries per-file per-predicate counts only, so a file whose matching
rows cluster adversarially can still make the planned order mispriced —
adapting mid-execution is exactly the OST-dynamic (runtime SIP) remaining
advantage, deliberately not practiced here. It is a boundary, not a
defect: the sealed footer admits no finer bound.

## Standing

ALIVE (all three courts ran + passed this session, exits cited above).
Plan is zero-dep; `cargo check -p stardf-planner` exit 0 as workspace
member. No git writes; coordinator owns transitions.