//! RDF-3 differential court: metadata-planned vs naive nested-loop over
//! the seeded 3-file 2-hop corpus. IDENTICAL solutions required; the
//! tuples-generated counter must show a measurable reduction.
//!
//! Anti-vacuity mutation: the naive/planned executors share scan_verified;
//! this court's flip lives in poison_court.rs (footer count poisoning).
//! Additional in-court flip: assert the planned order actually differs
//! from the written order (a planner that returns the written order makes
//! the reduction assertion vacuous-by-construction).

mod common;

use common::{corpus, two_hop_query};
use stardf_planner::exec::{naive_exec, planned_exec};
use stardf_planner::footer::parse_footer;
use stardf_planner::plan::plan;

#[test]
fn differential_identical_results_and_reduction() {
    let corpus = corpus();
    let files: Vec<(&[u8], _)> = corpus
        .iter()
        .map(|(_, bytes)| {
            let f = parse_footer(bytes).expect("footer parses + seal verifies");
            (bytes.as_slice(), f)
        })
        .collect();
    let footers: Vec<_> = files.iter().map(|(_, f)| f.clone()).collect();

    let q = two_hop_query();
    let p = plan(&q, &footers);

    // Flip guard: the planner MUST reorder (written order has the 200-row
    // NAME pattern first; footers say KNOWS-BOB has 40). A plan equal to
    // the written order cannot claim metadata-driven selection.
    assert_ne!(p.order, vec![0, 1, 2, 3], "planner must reorder: plan is metadata-driven");
    assert_eq!(p.order[0], 1, "KNOWS-BOB filter (40 rows) must be placed first");

    let naive = naive_exec(&q, &files).expect("naive run verifies footers");
    let planned = planned_exec(&q, &files, &p).expect("planned run verifies footers");

    assert_eq!(
        naive.solutions, planned.solutions,
        "differential: planned results must be IDENTICAL to naive"
    );
    assert!(
        naive.solutions.len() >= 40,
        "corpus must produce the 40 known people, got {}",
        naive.solutions.len()
    );
    let reduction = 100.0 * (1.0 - planned.tuples_generated as f64 / naive.tuples_generated as f64);
    assert!(
        reduction > 25.0,
        "intermediate-tuple reduction must be measurable (>25%), got {reduction:.1}% \
         (naive={}, planned={})",
        naive.tuples_generated,
        planned.tuples_generated
    );
    println!(
        "RDF-3 differential: naive={} planned={} tuples generated, reduction={reduction:.1}%",
        naive.tuples_generated, planned.tuples_generated
    );
}
