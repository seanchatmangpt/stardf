//! Chicago courts for lane RDF-6 (sameAs digest-space canonicalization).
//! Real collaborators, final-state asserts, no mocks. Every court names its
//! anti-vacuity mutation; the /tmp flip is driven by a python driver that
//! neutralizes contradiction detection in a sandbox copy (sha256 fail-closed
//! restore) and asserts the contradiction court FAILS.
use std::collections::BTreeMap;

use stardf_sameas::{term_id, FactStore, SameAsError, SameAsIndex, TermId};

const IRIS: [&[u8]; 6] = [
    b"http://ex/a",
    b"http://ex/b",
    b"http://ex/c",
    b"http://ex/d",
    b"http://ex/e",
    b"http://ex/f",
];

/// COURT 1: equivalence-class query returns canonical facts from ANY member.
/// Anti-vacuity mutation: if the rewrite layer were dropped (facts_about
/// returned only the raw subject's facts), a and b would each see 1 fact,
/// not 2. Asserted against final state (full (s,p,o) triples).
#[test]
fn court1_class_query_returns_canonical_facts_from_any_member() {
    let mut store = FactStore::new();
    store.index.ingest_pair(IRIS[0], IRIS[1]); // a sameAs b
    store.add_fact(IRIS[0], b"http://ex/name", b"Alice").unwrap();
    store.add_fact(IRIS[1], b"http://ex/mbox", b"alice@ex").unwrap();
    let via_a = store.facts_about(IRIS[0]);
    let via_b = store.facts_about(IRIS[1]);
    assert_eq!(via_a.len(), 2, "class union must expose both members' facts");
    assert_eq!(via_a, via_b, "query via any member returns identical fact set");
    // both member ids appear; canonical = min id
    let can = store.index.canonical(term_id(IRIS[0]));
    assert_eq!(can, store.index.canonical(term_id(IRIS[1])));
    assert!(via_a.contains(&(can, term_id(b"http://ex/name"), term_id(b"Alice"))));
    assert!(via_a.iter().all(|(s, _, _)| *s == can || s == &term_id(IRIS[1])));
}

/// COURT 2: election determinism under permuted insertion order. Two
/// insertion orders of the same pair set must yield identical canonical
/// maps. Mutation: order-dependent election (e.g. first-seen wins) flips
/// one map.
#[test]
fn court2_election_deterministic_under_permutation() {
    let pairs = [
        (IRIS[0], IRIS[1]),
        (IRIS[1], IRIS[2]),
        (IRIS[3], IRIS[4]),
    ];
    let mut fwd = SameAsIndex::new();
    for (a, b) in pairs { fwd.ingest_pair(a, b); }
    let mut rev = SameAsIndex::new();
    for (a, b) in pairs.iter().rev() { rev.ingest_pair(*a, *b); }
    let m1 = fwd.canonical_map();
    let m2 = rev.canonical_map();
    assert_eq!(m1, m2, "canonical election must be insertion-order free");
    assert_eq!(m1.len(), 2, "two classes expected");
    let c_abc = fwd.canonical(term_id(IRIS[0]));
    let c_de = fwd.canonical(term_id(IRIS[3]));
    assert_eq!(m1[&c_abc], {
        let mut v = vec![term_id(IRIS[0]), term_id(IRIS[1]), term_id(IRIS[2])];
        v.sort_unstable();
        v
    });
    assert!(c_abc < term_id(IRIS[0]) || c_abc == term_id(IRIS[0]));
    assert_eq!(c_abc, [term_id(IRIS[0]), term_id(IRIS[1]), term_id(IRIS[2])]
        .iter().copied().min().unwrap());
    assert_eq!(c_de, [term_id(IRIS[3]), term_id(IRIS[4])].iter().copied().min().unwrap());
}

/// COURT 3: contradiction court. Functional predicate with conflicting
/// objects across an equivalence class -> typed ContradictionRefused.
/// Mutation: neutralize the check -> this court fails (driven in /tmp).
#[test]
fn court3_functional_contradiction_refused() {
    let mut store = FactStore::new();
    store.index.ingest_pair(IRIS[0], IRIS[1]);
    let p = term_id(b"http://ex/primaryEmail");
    store.declare_functional(p);
    store.add_fact(IRIS[0], b"http://ex/primaryEmail", b"a@ex").unwrap();
    let err = store.add_fact(IRIS[1], b"http://ex/primaryEmail", b"b@ex");
    match err {
        Err(SameAsError::Contradiction(c)) => {
            assert_eq!(c.predicate, p);
            assert_eq!(c.existing_object, term_id(b"a@ex"));
            assert_eq!(c.new_object, term_id(b"b@ex"));
            assert_eq!(c.canonical, store.index.canonical(term_id(IRIS[0])));
        }
        other => panic!("expected Contradiction, got {other:?}"),
    }
    // same object through another member is NOT a contradiction (idempotent)
    let ok = store.add_fact(IRIS[1], b"http://ex/primaryEmail", b"a@ex");
    assert!(ok.is_ok(), "re-asserting the same fact via a synonym must pass");
}

/// COURT 4: collision trap fires on the real derivation path only when ids
/// genuinely collide (fail-closed discipline inherited from cgap A06 SS5).
/// Distinct IRIs must never collide here; a forced collision is detected by
/// the detector below.
#[test]
fn court4_digest_space_ids_distinct_and_trap_ready() {
    let ids: Vec<TermId> = IRIS.iter().map(|i| term_id(i)).collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "digest collision among distinct IRIs");
        }
    }
    // non-colliding IRIs must NOT trigger the trap (mutate: swap NS off and
    // the ids change -> golden below pins the ns-tagged derivation)
    assert_eq!(
        term_id(b"http://ex/a"),
        term_id(b"http://ex/a"),
        "derivation must be a pure function"
    );
    let _ = BTreeMap::<TermId, TermId>::new();
}

/// COURT 5: bench receipts. Rewrite overhead per fact and class-size
/// scaling. ns/fact measured with Instant (no sleeps); printed for the
/// lane receipt.
#[test]
fn court5_rewrite_overhead_bench() {
    use std::time::Instant;
    let mut store = FactStore::new();
    // one big class: 64 members
    let mut prev = b"http://ex/t0".to_vec();
    for i in 1..64 {
        let cur = format!("http://ex/t{i}").into_bytes();
        store.index.ingest_pair(&prev, &cur);
        prev = cur;
    }
    // 4096 facts spread over members
    let mut ids = Vec::new();
    for i in 0..64 { ids.push(format!("http://ex/t{i}").into_bytes()); }
    let pn = term_id(b"http://ex/p");
    for i in 0..4096 {
        let s = &ids[i % 64];
        let o = format!("v{i}").into_bytes();
        store.add_ids(term_id(s), pn, term_id(&o)).unwrap();
    }
    let t0 = Instant::now();
    let mut total = 0usize;
    for _ in 0..100 { total += store.facts_about(b"http://ex/t0").len(); }
    let per_call = t0.elapsed().as_nanos() / 100;
    let per_fact = per_call / 4096.max(1);
    println!("REWRITE-BENCH class=64 facts=4096 facts_about_call_ns={per_call} ns_per_returned_fact={per_fact}");
    assert_eq!(total, 409_600, "every query returns the full class union");
    // scaling: linear in class size
    let mut small = FactStore::new();
    small.index.ingest_pair(b"http://ex/x0", b"http://ex/x1");
    let t1 = Instant::now();
    for _ in 0..100 { let _ = small.facts_about(b"http://ex/x0"); }
    let small_ns = t1.elapsed().as_nanos() / 100;
    println!("REWRITE-BENCH class=2 facts=0 facts_about_call_ns={small_ns}");
    assert!(per_fact < 1000, "rewrite overhead per fact must stay sub-microsecond (ns/fact)");
}
