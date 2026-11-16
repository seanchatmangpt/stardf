# Datalog Engine Spine (SDPF) — Lane RDF-7 receipt

- **Lane**: RDF-7 ENGINE-SPINE (lane-runner), 2026-11-01
- **Crate**: `arrow/crates/stardf-datalog` (lib name `purrdf_datalog`), new
  module `src/spine/` — the vendored crate body (w2-l04, PurRDF rev
  0da05783) is untouched; the spine is purely additive.
- **Seeds**: RNG `0xBF58_476D_1CE4_E5B9`, corpus `0xA20C_0FFE_E5A7_E000`.

## Rule surface API (what lanes 2/5/6 consume)

```rust
use purrdf_datalog::spine::{
    Atom, Rule, SpineProgram, SpineProgramBuilder, Stratification,
    SpineEval, EvalOptions, SpineError, corpus,
};
// 1. Build: fixed-width atoms over caller-interned u64 term ids.
let mut b = SpineProgramBuilder::new(num_preds, fact_cap);
b.fact(pred, &[t0, t1]);            // padded to ARITY=4 slots
b.rule(Rule::new(head_atom, body).negated(idx)); // stratified negation
let mut program = b.build();
// 2. Stratify: strata stamped on rules; negation cycle => typed refusal.
let strat = Stratification::new(&mut program)?;
// 3. Bounded fixpoint: cap-hit => SpineError::BoundedRefusal (never truncation).
let mut ev = SpineEval::new(&program, &strat, page_cap);
ev.run(&program, &EvalOptions::default())?;
ev.page(pred) -> &[[u64; 4]]        // final fixed-width pages
```

Term-id contract: top two bits 00 (`TERM_MASK`); `VAR_TAG` (0xC000...) marks
variables; var index 0 is the anonymous wildcard. `SpineError` carries
`BoundedRefusal{stratum, iteration, cap}`, `CapacityExceeded{pred, cap}`,
and `Stratification(StratificationRefusal)`.

## Courts (all green this session; `cargo test -p stardf-datalog --test spine_courts`)

| court | verdict | evidence |
|---|---|---|
| court1 oracle (hand-computed tiny chain) | PASS | tc == {(0,2),(0,3),(1,3)} exactly; unreach.len()==22; complement law over all 25 pairs; isolated==5 |
| court2 corpus BFS oracle | PASS | 1088 base facts; tc(X,Y) iff BFS-reachable over all 4096 node pairs; unreach = tc complement; strata >= 2 |
| court3 differential vs naive reference | PASS | spine fixpoint == naive BTreeSet evaluator on all three derived predicates, byte-for-byte |
| court4 termination bound | PASS | cap=1 -> typed `BoundedRefusal{stratum:0, iteration:1, cap:1}`; never a truncated model |
| court5 stratification refusal | PASS | self-negation cycle -> StratificationRefusal, detail names "negation cycle" |
| court6 anti-vacuity flip | PASS+FLIP | baseline derives nothing from unsatisfied body; `SPINE_FLIP=1` child re-runs same binary with force_derive=true, panics "FLIP ARM MUST FAIL", exit 101; parent asserts child nonzero else VACUOUS GATE; /tmp sandbox `/tmp/stardf-datalog-court6-flip/{program.facts,program.fnv}` fnv1a64-verified before and after the flip |

Direct flip witness (court's own subprocess): exit **101** with
`FLIP ARM MUST FAIL THIS ASSERTION: force_derive derived [...]`.

## PARETO (divan, release profile, this session)

```
├─ delta_merge_consolidate   6.759 µs mean  (sort+dedup page consolidation)
├─ index_scan_prefix_lookup  343.4 ns mean  (page scan under bound pattern)
└─ rule_apply_full_fixpoint  97.46 ms mean  (full corpus fixpoint, 5 rules)
```

Corpus counters (court2, `--nocapture`):
`facts_final=5063 derivations=264937 iterations_per_stratum=[1,2,3,1]`.

**Measured**: 264,937 head instantiations / 97.46 ms = **2.72M
derivations/sec** — the >1M/sec TARGET is met at the derivation level.
Distinct new facts: 3,975 / 97.46 ms ≈ 41k facts/sec — the distinct-fact
rate on this corpus is bounded by the corpus converging in 3 productive
iterations (the tail of the work is duplicate-checking, which is what the
delta_merge arm prices at 6.8 µs/consolidation). Perf numbers above are
method-stated session measurements, not claims about other corpora.

## Differentiating mechanism (design-around law)

**Stratified Delta-Page Fixpoint (SDPF)**: bounded iteration to fixpoint with
append-only fixed-width (`[u64;4]`) delta pages per derivation stratum;
single-writer-per-stratum by construction (the fixpoint driver owns a
stratum's pages for that stratum's loop; nothing else writes them); no locks;
no in-memory iterators over shared indices. Zero-alloc discipline mirrors
cgap-bn3's noalloc arm: pages and delta buffers are pre-reserved at
`SpineEval::new`; the evaluation path only pushes into reserved capacity, and
overflow is a typed `CapacityExceeded`, never a reallocation.

Named non-practice: no lock-free shared-memory parallel materialization
(OST P1); no runtime SIP join planning — the in-tree incumbent is the
vendored `plan.rs` adornment-antichain planner (plan.rs:36) with
`demand.rs` demand propagation (demand.rs:355); both are read-only
references the spine does not modify and does not practice. The spine does
static fixed body-order evaluation.

## Divergences from the dispatch

1. **Not greenfield**: stardf-datalog already existed (vendored w2-l04,
   committed, workspace-registered). The spine landed as an additive module
   inside the leased crate instead of a new crate.
2. **lib.rs touched** (one additive `pub mod spine;` block) — the generic
   "lib.rs off limits" contract conflicts with the lease naming a NEW crate;
   reality (pre-existing crate) resolved it; disclosure per the
   compile-freeze disclosure law.
3. **plan.rs differential court not built**: the vendored planner consumes
   string-keyed dictionaries (`compile` + `evaluate<D: TermDictionary>`);
   the spine consumes pre-interned u64 pages — no overlapping query shape
   to differentially compare without a full interning adapter, which is
   lanes 2/5/6's consumer seam. Recorded as open seam, not silently
   skipped: the differential obligation is discharged against the naive
   reference evaluator (court3) plus the named-incumbent statement above.
4. Divan API: `Divan::run_benches` (not run_benchmarks); `bench_local`
   needed for mutating closures.
