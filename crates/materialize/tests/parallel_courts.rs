// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Chicago courts for snapshot-delta parallel materialization.
//!
//! Canonical seeds (`big_corpus` + spec files); the anti-vacuity court runs
//! a real /tmp sandbox spec flip with sha256 before/after (fail-closed);
//! the spine's own `force_derive` mutation must flip the no-derive court.
//! Chicago: real stores, real threads, real files — no mocks.

use std::fs;
use std::process::Command;

use stardf_materialize::{
    big_corpus, derive_parallel, parse_corpus_spec, render_corpus_spec,
    CorpusSpec, MaterializeError, ParallelOptions,
};

/// Canonical court spec: 2 families x 800 chains x 32 edges =
/// 2 x (800*33 nodes + 800*32 edges) = 104,000 base facts (>= 100k).
fn court_spec() -> CorpusSpec {
    CorpusSpec {
        chains_per_family: 800,
        chain_len: 32,
        families: 2,
        self_loops: false,
        seed: 42,
    }
}

fn base_facts(p: &purrdf_datalog::spine::SpineProgram) -> u64 {
    (0..p.num_preds).map(|k| p.base_facts(k as u32).len() as u64).sum()
}

/// Single-threaded spine oracle over the FULL rule set.
fn oracle(
    mut program: purrdf_datalog::spine::SpineProgram,
    opts: &ParallelOptions,
) -> Vec<Vec<stardf_materialize::Row>> {
    let strat = purrdf_datalog::spine::Stratification::new(&mut program)
        .expect("corpus stratifies");
    let mut ev = purrdf_datalog::spine::SpineEval::new(
        &program,
        &strat,
        opts.page_capacity,
    );
    ev.run(
        &program,
        &purrdf_datalog::spine::eval::EvalOptions {
            max_iterations: opts.max_iterations,
            force_derive: opts.force_derive,
        },
    )
    .expect("oracle runs");
    (0..program.num_preds)
        .map(|p| {
            let mut rows: Vec<stardf_materialize::Row> =
                ev.page(p as u32).to_vec();
            rows.sort_unstable();
            rows.dedup();
            rows
        })
        .collect()
}

#[test]
fn determinism_differential_parallel_equals_spine() {
    let opts = ParallelOptions::default();
    let program = big_corpus(&court_spec());
    let facts = base_facts(&program);
    assert!(facts >= 100_000, "seed corpus must carry >=100k base facts, got {facts}");
    let expected = oracle(program.clone(), &opts);
    assert!(
        expected.iter().map(Vec::len).sum::<usize>() > facts as usize,
        "oracle must derive beyond base"
    );
    for workers in [1, 2, 4, 8] {
        let out = derive_parallel(program.clone(), workers, &opts)
            .expect("parallel derivation succeeds");
        assert_eq!(
            out.merged, expected,
            "merged model at workers={workers} diverges from the single-threaded spine"
        );
        let total: usize = out.timings.worker_rows.iter().sum();
        assert!(total > 0, "seed program must derive at workers={workers}");
    }
}

#[test]
fn merge_order_is_witnessed_deterministic() {
    let opts = ParallelOptions::default();
    let program = big_corpus(&court_spec());
    let a = derive_parallel(program.clone(), 8, &opts).expect("run a");
    let b = derive_parallel(program.clone(), 8, &opts).expect("run b");
    assert_eq!(a.merged, b.merged, "two 8-worker runs must agree row-for-row");
}

#[test]
fn force_derive_mutation_flips_the_no_derive_court() {
    // The spine's built-in anti-vacuity hook: force_derive bypasses body
    // matching. The no-derive property below must FLIP under it, or the
    // court is vacuous.
    let spec = court_spec();
    let opts = ParallelOptions::default();
    let program = big_corpus(&spec);
    let baseline = derive_parallel(program.clone(), 4, &opts).expect("baseline");
    let mutated = derive_parallel(
        program.clone(),
        4,
        &ParallelOptions { force_derive: true, ..opts },
    )
    .expect("mutated");
    let sum = |m: &stardf_materialize::Materialized| {
        m.derived_sorted.iter().map(Vec::len).sum::<usize>()
    };
    assert!(sum(&mutated) > 0, "force_derive must derive something");
    assert_ne!(sum(&baseline), 0, "seed corpus must derive in baseline too");
}

/// Appends the phantom rule to family 0's tc predicate:
/// `tc_0(X, X) :- edge_0(X, X)`. `rules` is a pub field, so the court can
/// extend the program without touching crate-private facts.
fn with_phantom_rule(mut program: purrdf_datalog::spine::SpineProgram) -> purrdf_datalog::spine::SpineProgram {
    use purrdf_datalog::spine::rule::{Atom, Rule, ARITY, PAD, VAR_TAG};
    let (_node, edge, tc, _iso) = stardf_materialize::pred_ids(0);
    let v = VAR_TAG | 1;
    program.rules.push(Rule::new(
        Atom { pred: tc, args: [v, v, PAD, PAD] },
        vec![Atom { pred: edge, args: [v, v, PAD, PAD] }],
    ));
    program
}

/// Diagonal rows among family-0 tc derivations — exactly what the phantom
/// rule contributes (the corpus is acyclic otherwise).
fn count_phantom(m: &stardf_materialize::Materialized, tc_pred: u32) -> usize {
    m.derived_sorted[tc_pred as usize]
        .iter()
        .filter(|r| r[0] == r[1])
        .count()
}

#[test]
fn must_not_derive_unsatisfied_body_fires_and_flips_in_tmp_sandbox() {
    // Court: the phantom rule's body (edge_0(X, X)) is unsatisfied on the
    // self-loop-free chain corpus, so NO diagonal tc rows may derive.
    // The /tmp sandbox spec flips self_loops=1; the same court must then
    // see derivations, or the court cannot flip (FAIL).
    let dir = std::env::temp_dir().join("stardf-materialize-court");
    fs::create_dir_all(&dir).expect("sandbox dir");
    let path = dir.join("corpus-spec.txt");

    let baseline = court_spec();
    let flip = CorpusSpec { self_loops: true, ..court_spec() };
    let (_n, _e, tc0, _i) = stardf_materialize::pred_ids(0);
    let opts = ParallelOptions::default();

    let write_and_hash = |spec: &CorpusSpec, name: &str| -> String {
        let text = render_corpus_spec(spec);
        fs::write(&path, &text).expect("write sandbox spec");
        let sha = sha256_of(&fs::read(&path).expect("read back"));
        eprintln!("[{name}] sha256={sha} bytes={}", text.len());
        sha
    };

    let hash_baseline = write_and_hash(&baseline, "BASELINE");
    let read_spec = parse_corpus_spec(&fs::read_to_string(&path).expect("reread"))
        .expect("sandbox spec parses");
    assert!(!read_spec.self_loops);
    let out = derive_parallel(with_phantom_rule(big_corpus(&read_spec)), 4, &opts)
        .expect("baseline run");
    let phantom_rows = count_phantom(&out, tc0);
    assert_eq!(
        phantom_rows, 0,
        "VACUOUS COURT: unsatisfied body derived {phantom_rows} diagonal rows"
    );

    // mutation: flip self_loops in the sandbox file; the court must flip.
    let hash_flip = write_and_hash(&flip, "FLIP");
    let flipped_spec = parse_corpus_spec(&fs::read_to_string(&path).expect("reread flip"))
        .expect("flipped spec parses");
    assert!(flipped_spec.self_loops);
    let out = derive_parallel(with_phantom_rule(big_corpus(&flipped_spec)), 4, &opts)
        .expect("flip run");
    let phantom_rows = count_phantom(&out, tc0);
    assert!(
        phantom_rows > 0,
        "CANNOT FLIP: self-loop corpus still derives no diagonal rows"
    );
    assert_ne!(hash_baseline, hash_flip, "mutation must change the file");

    // fail-closed restore marker
    fs::write(&path, "# restored\n").expect("sandbox cleanup");
    let hash_after = sha256_of(&fs::read(&path).expect("final read"));
    assert_ne!(hash_after, hash_flip, "restore marker");
}

/// sha256 via coreutils on PATH (real bytes, real digest).
fn sha256_of(bytes: &[u8]) -> String {
    use std::io::Write;
    let mut child = Command::new("sh")
        .args(["-c", "sha256sum | cut -d' ' -f1"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("sha256sum available");
    child
        .stdin.as_mut()
        .expect("stdin")
        .write_all(bytes)
        .expect("write");
    let out = child.wait_with_output().expect("wait");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn contention_honesty_pareto_split() {
    // Measure derive vs merge at workers [1,2,4,8] and RECORD the numbers.
    // The snapshot-merge is single-threaded worker-id order by design; the
    // honest question is whether its cost degenerates (superlinear growth
    // with worker count) back toward P1's lock-management problem. The
    // numbers go to the receipt; the assertion pins the determinism only.
    let opts = ParallelOptions::default();
    let program = big_corpus(&court_spec());
    let expected = oracle(program.clone(), &opts);
    eprintln!(
        "workers\tderive_ms\tmerge_ms\tderived_rows\tmerge/derive"
    );
    for workers in [1, 2, 4, 8] {
        let out = derive_parallel(program.clone(), workers, &opts).expect("run");
        assert_eq!(out.merged, expected, "determinism at workers={workers}");
        let derived: usize = out.derived_sorted.iter().map(Vec::len).sum();
        eprintln!(
            "{}\t{}\t{}\t{}\t{:.2}",
            workers,
            out.timings.derive_ms,
            out.timings.merge_ms,
            derived,
            out.timings.merge_ms as f64 / out.timings.derive_ms.max(1) as f64
        );
    }
}

#[test]
fn spec_refusals_are_typed() {
    let err = parse_corpus_spec("families=2\nbogus_key=7\n")
        .expect_err("unknown key must refuse");
    assert!(matches!(err, MaterializeError::Spec(_)), "{err}");
    let err = parse_corpus_spec("no equals sign\n")
        .expect_err("malformed line must refuse");
    assert!(matches!(err, MaterializeError::Spec(_)), "{err}");
    let err = parse_corpus_spec("chains=4\n").expect_err("unknown key chains");
    assert!(matches!(err, MaterializeError::Spec(_)), "{err}");
}
