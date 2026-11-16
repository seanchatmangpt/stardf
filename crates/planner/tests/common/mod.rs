//! Seeded corpus shared by the RDF-3 courts: 3 sealed files, 2-hop join.
//! Seeds canonical (fixed constants, no randomness).

#![allow(dead_code)]

use stardf_planner::footer::seal_file;
use stardf_planner::plan::{Pattern, Query, Term};

pub const KNOWS: u32 = 1;
pub const NAME: u32 = 2;
pub const LIVES_IN: u32 = 3;
pub const IN_REGION: u32 = 4;
pub const BOB: u32 = 100;
pub const EU: u32 = 900;

pub fn c(v: u32) -> Term {
    Term::Const(v)
}
pub fn v(id: u32) -> Term {
    Term::Var(id)
}

/// Build the canonical 3-file corpus. Returns (bytes, name).
pub fn corpus() -> Vec<(String, Vec<u8>)> {
    // f0: 40 people know BOB + 200 NAME facts (bulk noise -> free-pred
    // scans are expensive, which is what the planner must avoid first).
    let mut f0 = Vec::new();
    for i in 0..40u32 {
        f0.push((1000 + i, KNOWS, BOB));
    }
    for i in 0..200u32 {
        f0.push((1000 + i, NAME, 5000 + i));
    }
    // f1: 40 people live in 10 regions + 300 noise facts.
    let mut f1 = Vec::new();
    for i in 0..40u32 {
        f1.push((1000 + i, LIVES_IN, 800 + (i % 10)));
    }
    for i in 0..300u32 {
        f1.push((2000 + i, LIVES_IN, 890));
    }
    // f2: 10 regions in EU + 500 noise facts.
    let mut f2 = Vec::new();
    for i in 0..10u32 {
        f2.push((800 + i, IN_REGION, EU));
    }
    for i in 0..500u32 {
        f2.push((3000 + i, IN_REGION, 999));
    }
    vec![
        ("f0-people".to_string(), seal_file(&f0)),
        ("f1-places".to_string(), seal_file(&f1)),
        ("f2-regions".to_string(), seal_file(&f2)),
    ]
}

/// Two-hop query: ?x KNOWS BOB . ?x NAME ?n . ?x LIVES_IN ?r . ?r IN_REGION EU.
/// Patterns are listed WORST-FIRST (free predicate NAME first) — this is
/// the unoptimized order the naive executor honors.
pub fn two_hop_query() -> Query {
    Query {
        patterns: vec![
            Pattern::spo(v(1), c(NAME), v(2)),   // free pred, 200 cands
            Pattern::spo(v(1), c(KNOWS), c(BOB)), // filter, 40 cands
            Pattern::spo(v(1), c(LIVES_IN), v(3)),
            Pattern::spo(v(3), c(IN_REGION), c(EU)),
        ],
    }
}

/// Depth-k chain queries over the same corpus: x0 KNOWS BOB, then k-1
/// additional hops anchored at the same subject, written worst-first.
pub fn chain_query(depth: usize) -> Query {
    let mut pats = vec![Pattern::spo(v(1), c(NAME), v(2))];
    pats.push(Pattern::spo(v(1), c(KNOWS), c(BOB)));
    for d in 1..depth {
        let r = 800 + d as u32 % 10;
        pats.push(Pattern::spo(v(1), c(LIVES_IN), c(r)));
        pats.push(Pattern::spo(c(r), c(IN_REGION), c(EU)));
    }
    Query { patterns: pats }
}
