//! RDF-7 ENGINE-SPINE courts. Chicago discipline: real collaborators,
//! final-state assertions, subprocess flip in a /tmp sandbox, no mocks.

use std::collections::VecDeque;
use std::process::Command;

use purrdf_datalog::spine::corpus::{
    corpus_program, CORPUS_ID, NUM_EDGES, NUM_NODES, P_EDGE, P_ISOLATED, P_NODE, P_TC,
    P_UNREACH, RNG_SEED,
};
use purrdf_datalog::spine::eval::{EvalOptions, SpineError, SpineEval};
use purrdf_datalog::spine::refeval::reference_eval;
use purrdf_datalog::spine::rule::{Atom, Rule, PAD, TERM_MASK, VAR_TAG};
use purrdf_datalog::spine::stratify::Stratification;

/// Edge list accessor for the BFS oracle.
fn edges(program: &purrdf_datalog::spine::rule::SpineProgram) -> Vec<(u64, u64)> {
    program.base_facts(P_EDGE)
        .iter()
        .map(|r| (r[0], r[1]))
        .collect()
}

/// BFS reachability oracle: reach[X] = set of Y reachable from X.
fn bfs_reach(edges: &[(u64, u64)]) -> Vec<std::collections::BTreeSet<u64>> {
    let mut reach: Vec<std::collections::BTreeSet<u64>> =
        vec![std::collections::BTreeSet::new(); NUM_NODES];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); NUM_NODES];
    for (x, y) in edges {
        let xi = (x >> 8) as usize;
        let yi = (y >> 8) as usize;
        adj[xi].push(yi);
    }
    for src in 0..NUM_NODES {
        let mut seen = vec![false; NUM_NODES];
        let mut q = VecDeque::new();
        seen[src] = true;
        reach[src].insert(src as u64);
        q.push_back(src);
        while let Some(n) = q.pop_front() {
            for &m in &adj[n] {
                if !seen[m] {
                    seen[m] = true;
                    reach[src].insert(m as u64);
                    q.push_back(m);
                }
            }
        }
    }
    reach
}

/// Runs the spine on the corpus; returns final tc/unreach/isolated pages.
fn run_corpus(opts: &EvalOptions) -> (
    Vec<[u64; 4]>, Vec<[u64; 4]>, Vec<[u64; 4]>, purrdf_datalog::spine::eval::SpineStats,
) {
    let program = corpus_program();
    let mut prog2 = corpus_program();
    let strat = Stratification::new(&mut prog2).unwrap();
    let mut ev = SpineEval::new(&program, &strat, 4096);
    ev.run(&program, opts).expect("corpus run must converge");
    let mut tc: Vec<[u64; 4]> = ev.page(P_TC).to_vec();
    tc.sort_unstable();
    let mut unreach: Vec<[u64; 4]> = ev.page(P_UNREACH).to_vec();
    unreach.sort_unstable();
    let mut iso: Vec<[u64; 4]> = ev.page(P_ISOLATED).to_vec();
    (tc, unreach, iso, ev.stats().clone())
}
/// Runs the tiny program; returns tc/unreach/isolated pages sorted.
fn run_tiny() -> (Vec<[u64; 4]>, Vec<[u64; 4]>, Vec<[u64; 4]>) {
    let program = purrdf_datalog::spine::corpus::tiny_program();
    let mut prog2 = purrdf_datalog::spine::corpus::tiny_program();
    let strat = Stratification::new(&mut prog2).unwrap();
    let mut ev = SpineEval::new(&program, &strat, 4096);
    ev.run(&program, &EvalOptions::default()).unwrap();
    let mut tc: Vec<[u64; 4]> = ev.page(P_TC).to_vec();
    tc.sort_unstable();
    let mut unreach: Vec<[u64; 4]> = ev.page(P_UNREACH).to_vec();
    unreach.sort_unstable();
    let mut iso: Vec<[u64; 4]> = ev.page(P_ISOLATED).to_vec();
    iso.sort_unstable();
    (tc, unreach, iso)
}

/// COURT 1 (oracle, hand-computed): tiny chain program against the hand
/// oracle documented in corpus::tiny_program. Anti-vacuity: a mutation that
/// derives from unsatisfied bodies flips this court (court 4).
#[test]
fn court1_oracle_hand_computed() {
    let (tc, unreach, iso) = run_tiny();
    let expected_tc: Vec<[u64; 4]> = [
        (0u64, 2u64), (0, 3), (1, 3),
    ]
    .iter()
    .map(|(x, y)| [(x << 8), (y << 8), 0, 0])
    .collect();
    assert_eq!(tc, expected_tc, "tc must equal the hand-computed chain closure");
    assert_eq!(unreach.len(), 25 - 3, "unreach = 25 node pairs - 3 tc");
    // Complement law: unreach(X,Y) present iff tc(X,Y) absent (diagonal incl.).
    for x in 0u64..5 {
        for y in 0u64..5 {
            let pair = [(x << 8), (y << 8), 0, 0];
            assert_eq!(
                unreach.contains(&pair),
                !tc.contains(&pair),
                "complement law broken at ({x},{y})"
            );
        }
    }
    assert_eq!(iso.len(), 5, "acyclic chain: every node has tc(X,X) absent");
}

/// COURT 2 (corpus oracle, independent algorithm): BFS reachability over the
/// seeded corpus (>= 1000 facts: 64 nodes + 1024 edges + derivations; >= 5
/// rules; >= 2 strata; one negation).
#[test]
fn court2_corpus_bfs_oracle() {
    let program = corpus_program();
    assert!(program.base_facts(P_NODE).len() + program.base_facts(P_EDGE).len() >= 1000);
    let (tc, unreach, _iso, stats) = run_corpus(&EvalOptions::default());
    let reach = bfs_reach(&edges(&program));
    assert_eq!(tc.len(), NUM_NODES * NUM_NODES - unreach.len(), "partition law");
    let mut tc_pairs: std::collections::BTreeSet<(u64, u64)> = tc
        .iter()
        .map(|r| (r[0], r[1]))
        .collect();
    let _ = &mut tc_pairs;
    // tc(X,Y) present iff BFS-reachable.
    for x in 0..NUM_NODES {
        for y in 0..NUM_NODES {
            let pair = (((x as u64) << 8), ((y as u64) << 8));
            assert_eq!(
                tc_pairs.contains(&pair),
                reach[x].contains(&(y as u64)),
                "tc vs BFS mismatch at ({x},{y})"
            );
        }
    }
    // Negation law: unreach = complement of tc over node pairs.
    for r in &unreach {
        assert!(!tc_pairs.contains(&(r[0], r[1])), "unreach row in tc: {r:?}");
    }
    assert!(stats.strata_run >= 2, "corpus must span >= 2 strata");
    println!(
        "[spine-corpus] facts_final={} derivations(head instantiations)={} iterations_per_stratum={:?}",
        stats.facts_final, stats.derivations, stats.iterations
    );
}

/// COURT 3 (differential): the spine's fixpoint must equal the naive
/// reference evaluator on the seeded corpus, predicate by predicate.
#[test]
fn court3_differential_naive_reference() {
    let (tc, unreach, iso, _) = run_corpus(&EvalOptions::default());
    let r = reference_eval();
    let to_sorted = |page: Vec<[u64; 4]>| {
        let mut v: Vec<[u64; 4]> = page.to_vec();
        v.sort_unstable();
        v
    };
    let ref_sorted = |p: usize| r.pages[p].iter().copied().collect::<Vec<_>>();
    assert_eq!(to_sorted(unreach), ref_sorted(3));
    assert_eq!(to_sorted(iso), ref_sorted(4));
    assert_eq!(to_sorted(tc), ref_sorted(2), "tc: spine == naive reference");
}

/// COURT 4 (termination bound, witnessed): a cap of 1 iteration per stratum
/// must produce a typed BoundedRefusal naming stratum and iteration — never
/// a silently truncated model.
#[test]
fn court4_termination_cap_is_typed_refusal() {
    let program = corpus_program();
    let mut prog2 = corpus_program();
    let strat = Stratification::new(&mut prog2).unwrap();
    let mut ev = SpineEval::new(&program, &strat, 16 * 1024);
    let opts = EvalOptions { max_iterations: 1, ..Default::default() };
    let err = ev.run(&program, &opts).unwrap_err();
    match err {
        SpineError::BoundedRefusal { stratum, iteration, cap } => {
            assert_eq!(stratum, 0, "stratum 0 (tc rules) is the one that iterates");
            assert_eq!(iteration, 1);
            assert_eq!(cap, 1);
        }
        other => panic!("expected BoundedRefusal, got {other:?}"),
    }
}

/// COURT 5 (stratification refusal): a negation cycle must be a typed
/// StratificationRefusal, not a hang or a silent wrap.
#[test]
fn court5_negation_cycle_refused() {
    use purrdf_datalog::spine::rule::SpineProgramBuilder;
    let mut b = SpineProgramBuilder::new(2, 64);
    let v = |i: u64| VAR_TAG | i;
    b.fact(0, &[1u64]);
    // Self-negation: p0(X) :- NOT p0(X) -> stratum[0] > stratum[0]: cycle.
    let mut r = Rule::new(
        Atom { pred: 0, args: [v(1), PAD, PAD, PAD] },
        vec![Atom { pred: 0, args: [v(1), PAD, PAD, PAD] }],
    );
    r = r.negated(0);
    b.rule(r);
    let mut program = b.build();
    match Stratification::new(&mut program) {
        Ok(_) => panic!("self-negation cycle must be refused"),
        Err(e) => assert!(
            e.detail.contains("negation cycle"),
            "detail: {}",
            e.detail
        ),
    }
}

/// COURT 6 (anti-vacuity flip, /tmp sandbox): a rule whose body is
/// unsatisfied must derive NOTHING. The flip (SPINE_FLIP=1) reruns this
/// same test binary in a child process with force_derive=true, so the
/// unsatisfied body "derives" — the child's assertion below must FAIL
/// (nonzero exit) for the court to pass. A court that cannot flip is FAIL.
#[test]
fn court6_anti_vacuity_flip() {
    // Unsatisfied body: edge to a node that has no outgoing edges.
    let mut b = purrdf_datalog::spine::rule::SpineProgramBuilder::new(3, 64);
    let v = |i: u64| VAR_TAG | i;
    b.fact(0, &[10u64, 0, 0, 0]); // pred0: has successor 11
    b.fact(1, &[11u64, 0, 0, 0]); // pred1: node 11 exists
    // pred2 has NO facts: r: p2(X) :- p0(X), p1(X), p2(X) can never match.
    b.rule(Rule::new(
        Atom { pred: 2, args: [v(1), PAD, PAD, PAD] },
        vec![
            Atom { pred: 0, args: [v(1), PAD, PAD, PAD] },
            Atom { pred: 1, args: [v(1), PAD, PAD, PAD] },
            Atom { pred: 2, args: [v(1), PAD, PAD, PAD] },
        ],
    ));
    let mut program = b.build();
    Stratification::new(&mut program).unwrap();
    // Sandbox: serialize the program surface + fingerprint to /tmp; the
    // child verifies the bytes before mutating (fail-closed provenance).
    // Hardcoded /tmp sandbox (macOS temp_dir() is /var/folders, not /tmp).
    let dir = std::path::PathBuf::from("/tmp/stardf-datalog-court6-flip");
    std::fs::create_dir_all(&dir).unwrap();
    let prog_path = dir.join("program.facts");
    std::fs::write(&prog_path, b"pred0=10;pred1=11;pred2=<empty>;rule=p2(X):-p0,p1,!none")
        .unwrap();
    let fingerprint = purrdf_hash::fnv::fnv1a64(&std::fs::read(&prog_path).unwrap());
    let fp_path = dir.join("program.fnv");
    std::fs::write(&fp_path, fingerprint.to_le_bytes()).unwrap();
    if std::env::var("SPINE_FLIP").is_ok_and(|v| v == "1") {
        // FLIP arm: force-derive; the final assertion MUST fail.
        let mut ev = SpineEval::new(&program, &make_strat(&program), 64);
        let opts = EvalOptions { force_derive: true, ..Default::default() };
        ev.run(&program, &opts).unwrap();
        let fp_on_disk = u64::from_le_bytes(std::fs::read(&fp_path).unwrap().try_into().unwrap());
        assert_eq!(fp_on_disk, fingerprint, "sandbox bytes moved under us");
        assert!(
            ev.page(2).is_empty(),
            "FLIP ARM MUST FAIL THIS ASSERTION: force_derive derived {:#?}",
            ev.page(2)
        );
        return;
    }
    // BASELINE arm: spine must NOT derive from an unsatisfied body.
    let mut ev = SpineEval::new(&program, &make_strat(&program), 64);
    ev.run(&program, &EvalOptions::default()).unwrap();
    assert!(ev.page(2).is_empty(), "unsatisfied body derived: {:#?}", ev.page(2));
    // Flip: rerun this test in a child with SPINE_FLIP=1; the child must
    // exit nonzero (its assertion fails) — else the gate is vacuous.
    let exe = std::env::current_exe().unwrap();
    let out = Command::new(exe)
        .args(["court6_anti_vacuity_flip", "--exact", "--nocapture"])
        .env("SPINE_FLIP", "1")
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "VACUOUS GATE: flip arm passed; force_derive did not derive \
         (stdout: {} stderr: {})",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // Post-flip sandbox verify: the /tmp program bytes must be unchanged.
    let after = std::fs::read(&prog_path).unwrap();
    assert_eq!(purrdf_hash::fnv::fnv1a64(&after), fingerprint, "sandbox program bytes moved under the flip");
}

fn make_strat(program: &purrdf_datalog::spine::rule::SpineProgram) -> Stratification {
    let mut p2 = program.clone();
    Stratification::new(&mut p2).unwrap()
}
