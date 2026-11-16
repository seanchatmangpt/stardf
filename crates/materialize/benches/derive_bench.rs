// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! divan bench: derived facts/second at worker counts [1, 2, 4, 8] over a
//! >=100k-base-fact seeded chain corpus. PARETO split (derivation vs merge)
//! printed per iteration from Timings; receipt in
//! arrow/docs/ost/parallel-materialize.md.

use divan::Bencher;

#[divan::bench(args = [1, 2, 4, 8])]
fn derive_parallel_workers(bench: Bencher, workers: usize) {
    let program = stardf_materialize::big_corpus(&spec());
    bench.bench_local(|| {
        let out = stardf_materialize::derive_parallel(
            program.clone(),
            workers,
            &stardf_materialize::ParallelOptions::default(),
        )
        .expect("parallel derivation");
        let derived: usize = out.derived_sorted.iter().map(Vec::len).sum();
        eprintln!(
            "PARETO workers={workers} derive_ms={} merge_ms={} derived={derived}",
            out.timings.derive_ms, out.timings.merge_ms
        );
        assert!(derived > 0);
    });
}

/// Bench spec: 2 families x 800 chains x 32 edges = 104k base facts.
fn spec() -> stardf_materialize::CorpusSpec {
    stardf_materialize::CorpusSpec {
        chains_per_family: 800,
        chain_len: 32,
        families: 2,
        self_loops: false,
        seed: 42,
    }
}

fn main() {
    divan::main();
}
