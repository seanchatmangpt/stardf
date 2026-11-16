// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Seeded court corpus (splitmix64, seeds pinned by the RDF-7 dispatch):
//! RNG seed 0xBF58_476D_1CE4_E5B9, corpus id 0xA20C_0FFE_E5A7_E000.
//!
//! Predicates: 0=node/1, 1=edge/2, 2=tc/2, 3=unreach/2, 4=isolated/1.
//! Rules (5):
//!   r0 tc(X,Z)      :- edge(X,Y), edge(Y,Z).          [stratum 1]
//!   r1 tc(X,Z)      :- tc(X,Y), edge(Y,Z).            [stratum 1]
//!   r2 tc(X,Z)      :- edge(X,Y), tc(Y,Z).            [stratum 1]
//!   r3 unreach(X,Y) :- node(X), node(Y), !tc(X,Y).    [stratum 2]
//!   r4 isolated(X)  :- node(X), !tc(X,X).             [stratum 2]

use super::rule::{Atom, Rule, SpineProgram, SpineProgramBuilder, PAD, TERM_MASK, VAR_TAG};
use super::stratify::Stratification;

pub const RNG_SEED: u64 = 0xBF58_476D_1CE4_E5B9;
pub const CORPUS_ID: u64 = 0xA20C_0FFE_E5A7_E000;
pub const P_NODE: u32 = 0;
pub const P_EDGE: u32 = 1;
pub const P_TC: u32 = 2;
pub const P_UNREACH: u32 = 3;
pub const P_ISOLATED: u32 = 4;
pub const NUM_PREDS: usize = 5;
pub const NUM_NODES: usize = 64;
pub const NUM_EDGES: usize = 1024;

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn var(i: u64) -> u64 {
    VAR_TAG | i
}

/// Builds the corpus program. Facts: node/1 over NUM_NODES ids, edge/2 over
/// NUM_EDGES seeded pairs. Rules r0..r4 as documented above.
pub fn corpus_program() -> SpineProgram {
    let mut state = RNG_SEED ^ CORPUS_ID;
    let mut b = SpineProgramBuilder::new(NUM_PREDS, 16 * 1024);
    for n in 0..NUM_NODES {
        let id = ((n as u64) << 8) & TERM_MASK;
        b.fact(P_NODE, &[id]);
    }
    for _ in 0..NUM_EDGES {
        let a = (splitmix64(&mut state) as usize) % NUM_NODES;
        let c = (splitmix64(&mut state) as usize) % NUM_NODES;
        let x = ((a as u64) << 8) & TERM_MASK;
        let y = ((c as u64) << 8) & TERM_MASK;
        b.fact(P_EDGE, &[x, y]);
    }
    // r0: tc(X,Z) :- edge(X,Y), edge(Y,Z)
    b.rule(Rule::new(
        Atom { pred: P_TC, args: [var(1), var(2), PAD, PAD] },
        vec![
            Atom { pred: P_EDGE, args: [var(1), var(3), PAD, PAD] },
            Atom { pred: P_EDGE, args: [var(3), var(2), PAD, PAD] },
        ],
    ));
    // r1: tc(X,Z) :- tc(X,Y), edge(Y,Z)
    b.rule(Rule::new(
        Atom { pred: P_TC, args: [var(1), var(2), PAD, PAD] },
        vec![
            Atom { pred: P_TC, args: [var(1), var(3), PAD, PAD] },
            Atom { pred: P_EDGE, args: [var(3), var(2), PAD, PAD] },
        ],
    ));
    // r2: tc(X,Z) :- edge(X,Y), tc(Y,Z)
    b.rule(Rule::new(
        Atom { pred: P_TC, args: [var(1), var(2), PAD, PAD] },
        vec![
            Atom { pred: P_EDGE, args: [var(1), var(3), PAD, PAD] },
            Atom { pred: P_TC, args: [var(3), var(2), PAD, PAD] },
        ],
    ));
    // r3: unreach(X,Y) :- node(X), node(Y), !tc(X,Y)
    let mut r3 = Rule::new(
        Atom { pred: P_UNREACH, args: [var(1), var(2), PAD, PAD] },
        vec![
            Atom { pred: P_NODE, args: [var(1), PAD, PAD, PAD] },
            Atom { pred: P_NODE, args: [var(2), PAD, PAD, PAD] },
            Atom { pred: P_TC, args: [var(1), var(2), PAD, PAD] },
        ],
    );
    r3 = r3.negated(2);
    b.rule(r3);
    // r4: isolated(X) :- node(X), !tc(X,X)
    let mut r4 = Rule::new(
        Atom { pred: P_ISOLATED, args: [var(1), PAD, PAD, PAD] },
        vec![
            Atom { pred: P_NODE, args: [var(1), PAD, PAD, PAD] },
            Atom { pred: P_TC, args: [var(1), var(1), PAD, PAD] },
        ],
    );
    r4 = r4.negated(1);
    b.rule(r4);
    let mut program = b.build();
    let _ = Stratification::new(&mut program).expect("corpus must stratify");
    program
}

/// Tiny hand-computed program for the oracle court: chain 0->1->2->3 over
/// 5 nodes. Hand oracle:
///   tc (len>=2 paths) = {(0,2),(0,3),(1,3)} (3 pairs; r0 is 2-hop and
///   r1/r2 recurse, so 1-hop pairs are edges, never tc)
///   unreach = 25 - 3 = 22 pairs
///   isolated = {0,1,2,3,4}: the chain is acyclic, tc(X,X) absent for all
///   five nodes, so every node is "isolated" per r4.
pub fn tiny_program() -> SpineProgram {
    let mut b = SpineProgramBuilder::new(NUM_PREDS, 256);
    for n in 0u64..5 {
        b.fact(P_NODE, &[n << 8]);
    }
    for (x, y) in [(0u64, 1u64), (1, 2), (2, 3)] {
        b.fact(P_EDGE, &[x << 8, y << 8]);
    }
    let a = |pred: u32, x: u64, y: u64| Atom { pred, args: [x, y, PAD, PAD] };
    b.rule(Rule::new(a(P_TC, var(1), var(2)), vec![
        a(P_EDGE, var(1), var(3)),
        a(P_EDGE, var(3), var(2)),
    ]));
    b.rule(Rule::new(a(P_TC, var(1), var(2)), vec![
        a(P_TC, var(1), var(3)),
        a(P_EDGE, var(3), var(2)),
    ]));
    b.rule(Rule::new(a(P_TC, var(1), var(2)), vec![
        a(P_EDGE, var(1), var(3)),
        a(P_TC, var(3), var(2)),
    ]));
    let mut r3 = Rule::new(a(P_UNREACH, var(1), var(2)), vec![
        Atom { pred: P_NODE, args: [var(1), PAD, PAD, PAD] },
        Atom { pred: P_NODE, args: [var(2), PAD, PAD, PAD] },
        a(P_TC, var(1), var(2)),
    ]);
    r3 = r3.negated(2);
    b.rule(r3);
    let mut r4 = Rule::new(
        Atom { pred: P_ISOLATED, args: [var(1), PAD, PAD, PAD] },
        vec![
            Atom { pred: P_NODE, args: [var(1), PAD, PAD, PAD] },
            a(P_TC, var(1), var(1)),
        ],
    );
    r4 = r4.negated(1);
    b.rule(r4);
    let mut program = b.build();
    Stratification::new(&mut program).expect("tiny program must stratify");
    program
}
