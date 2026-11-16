//! RDF-3 bench: tuples-generated, planned vs naive, across join depths
//! [2,3,4]. PARETO assertion: planned <= naive at every depth, strictly
//! less at every depth >= 2, results identical at every depth.

mod common;

use common::{chain_query, corpus};
use stardf_planner::exec::{naive_exec, planned_exec};
use stardf_planner::footer::parse_footer;
use stardf_planner::plan::plan;

#[test]
fn pareto_across_depths() {
    let corpus = corpus();
    let files: Vec<(&[u8], _)> = corpus
        .iter()
        .map(|(_, b)| (b.as_slice(), parse_footer(b).unwrap()))
        .collect();
    let footers: Vec<_> = files.iter().map(|(_, f)| f.clone()).collect();

    let mut rows = Vec::new();
    for depth in [2usize, 3, 4] {
        let q = chain_query(depth);
        let n = naive_exec(&q, &files).expect("naive depth run");
        let p = plan(&q, &footers);
        let pl = planned_exec(&q, &files, &p).expect("planned depth run");
        assert_eq!(n.solutions, pl.solutions, "depth {depth}: identical results");
        assert!(
            pl.tuples_generated < n.tuples_generated,
            "depth {depth}: planned ({}) must beat naive ({})",
            pl.tuples_generated,
            n.tuples_generated
        );
        let red = 100.0 * (1.0 - pl.tuples_generated as f64 / n.tuples_generated as f64);
        rows.push((depth, n.tuples_generated, pl.tuples_generated, red));
    }
    for (d, n, p, red) in &rows {
        println!("depth={d} naive={n} planned={p} reduction={red:.1}%");
    }
}
