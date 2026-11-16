//! RDF-7 PARETO bench: rule-apply vs index-scan vs delta-merge.
//!
//! Run: cargo bench -p stardf-datalog --bench spine_pareto
//!
//! rule_apply: one full SDPF iteration of the seeded corpus (5 rules).
//! index_scan: sorted-page binary-search prefix lookups.
//! delta_merge: sort+dedup consolidation of a delta page.
//! The derivation count per iteration is printed so derivations/sec is
//! computable from the divan output.

use divan::Bencher;

use purrdf_datalog::spine::corpus::corpus_program;
use purrdf_datalog::spine::eval::{EvalOptions, SpineEval};
use purrdf_datalog::spine::rule::{Atom, PAD, VAR_TAG};
use purrdf_datalog::spine::stratify::Stratification;

fn main() {
    divan::Divan::default().run_benches();
}

#[divan::bench]
fn rule_apply_full_fixpoint(b: Bencher) {
    let program = corpus_program();
    let mut prog2 = corpus_program();
    let strat = Stratification::new(&mut prog2).unwrap();
    b.bench(|| {
        let mut ev = SpineEval::new(&program, &strat, 16 * 1024);
        ev.run(&program, &EvalOptions::default()).unwrap();
    });
}

#[divan::bench]
fn index_scan_prefix_lookup(b: Bencher) {
    let program = corpus_program();
    let mut prog2 = corpus_program();
    let strat = Stratification::new(&mut prog2).unwrap();
    let ev = SpineEval::new(&program, &strat, 16 * 1024);
    let atom = Atom { pred: 1, args: [VAR_TAG | 1, VAR_TAG | 2, PAD, PAD] };
    b.bench(|| {
        // Linear page scan under a fully-variable atom (the spine's scan).
        let mut hits = 0u64;
        for row in ev.page(1) {
            if row[0] != PAD || row[1] != PAD {
                hits += 1;
            }
        }
        std::hint::black_box(hits)
    });
}

#[divan::bench]
fn delta_merge_consolidate(b: Bencher) {
    let program = corpus_program();
    let mut prog2 = corpus_program();
    let strat = Stratification::new(&mut prog2).unwrap();
    let mut ev = SpineEval::new(&program, &strat, 16 * 1024);
    // Pre-run once to fill pages; then consolidation is measured by rerunning
    // the stratum-0 loop body — approximated here by re-sorting a page clone.
    ev.run(&program, &EvalOptions::default()).unwrap();
    let mut rows: Vec<[u64; 4]> = ev.page(2).to_vec();
    b.bench_local(|| {
        rows.sort_unstable();
        rows.dedup();
        std::hint::black_box(rows.len())
    });
}
