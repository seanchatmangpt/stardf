//! Executors: naive (pattern order as written, nested loop) vs planned
//! (footer-ordered), both instrumented with a tuples-generated counter.
//!
//! Both propagate bindings within the pipeline (a nested-loop join does
//! that in any engine); the DIFFERENTIATOR is join order chosen from
//! sealed metadata before execution — no runtime constraint propagation.

use crate::footer::{scan_verified, FooterFault, SealedFooter};
use crate::plan::{Plan, Pattern, Query, Term};
use std::collections::BTreeMap;

/// Variable binding: var id -> term id.
pub type Binding = BTreeMap<u32, u32>;

/// Instrumented run: solution bindings + tuples generated (every triple
/// candidacy a pattern evaluation examines counts once).
#[derive(Debug, PartialEq, Eq)]
pub struct RunStats {
    pub solutions: Vec<Binding>,
    pub tuples_generated: u64,
}

/// Naive executor: patterns in the ORDER WRITTEN.
pub fn naive_exec(
    query: &Query,
    files: &[(&[u8], SealedFooter)],
) -> Result<RunStats, FooterFault> {
    let order: Vec<usize> = (0..query.patterns.len()).collect();
    exec_order(query, files, &order)
}

/// Planned executor: patterns in the SEALED-METADATA order.
pub fn planned_exec(
    query: &Query,
    files: &[(&[u8], SealedFooter)],
    plan: &Plan,
) -> Result<RunStats, FooterFault> {
    exec_order(query, files, &plan.order)
}

fn term_matches(t: &Term, val: u32, b: &Binding) -> bool {
    match t {
        Term::Const(c) => *c == val,
        Term::Var(v) => b.get(v).map_or(true, |prev| *prev == val),
    }
}

/// Bind `t` to `val` in `out`; conflicting prior binding fails the match.
fn bind(t: &Term, val: u32, out: &mut Binding) -> bool {
    if let Term::Var(v) = t {
        let prev = out.insert(*v, val);
        if let Some(p) = prev {
            if p != val {
                return false;
            }
        }
    }
    true
}

fn exec_order(
    query: &Query,
    files: &[(&[u8], SealedFooter)],
    order: &[usize],
) -> Result<RunStats, FooterFault> {
    let mut tuples: u64 = 0;
    let mut partial: Vec<Binding> = vec![BTreeMap::new()];
    for &pi in order {
        let pat: Pattern = query.patterns[pi];
        let mut next: Vec<Binding> = Vec::new();
        for (data, footer) in files {
            let triples = scan_verified(data, footer)?;
            for &(s, p, o) in &triples {
                // Intermediate-tuple generation: every (triple x partial
                // binding) candidacy the pattern evaluation constructs.
                tuples += partial.len() as u64;
                for b in &partial {
                    if term_matches(&pat.s, s, b)
                        && term_matches(&pat.p, p, b)
                        && term_matches(&pat.o, o, b)
                    {
                        let mut m = b.clone();
                        if bind(&pat.s, s, &mut m)
                            && bind(&pat.p, p, &mut m)
                            && bind(&pat.o, o, &mut m)
                        {
                            next.push(m);
                        }
                    }
                }
            }
        }
        partial = next;
        if partial.is_empty() {
            break;
        }
    }
    Ok(RunStats { solutions: partial, tuples_generated: tuples })
}
