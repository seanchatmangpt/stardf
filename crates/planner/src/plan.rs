//! Conjunctive query representation + static join-order selection.
//!
//! The static analog of SIP: cardinality bounds come from the sealed
//! footer BEFORE execution; the plan (a total order on patterns) is fixed
//! before the first triple is read. No runtime constraint propagation.

use crate::footer::{sealed_count, SealedFooter};
use std::vec::Vec;

/// Query term: a bound constant or a named variable (u32 var id, shared
/// across patterns to express joins).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Term {
    Const(u32),
    Var(u32),
}

/// One s-p-o pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pattern {
    pub s: Term,
    pub p: Term,
    pub o: Term,
}

impl Pattern {
    pub const fn spo(s: Term, p: Term, o: Term) -> Self {
        Self { s, p, o }
    }
    /// Constant filter = bound predicate (selective placement target).
    pub const fn is_const_filter(&self) -> bool {
        matches!(self.p, Term::Const(_))
    }
}

/// A conjunctive query over term ids.
#[derive(Debug, Clone)]
pub struct Query {
    pub patterns: Vec<Pattern>,
}

/// Static per-pattern cardinality bound in one file: the sealed
/// per-predicate count when the predicate is bound, else the file row
/// count (the footer admits no finer bound for a free predicate).
pub fn pattern_bound(pat: &Pattern, f: &SealedFooter) -> u64 {
    match pat.p {
        Term::Const(p) => sealed_count(f, p).unwrap_or(0),
        Term::Var(_) => f.row_count,
    }
}

/// Metadata-driven plan: pattern indices in execution order.
#[derive(Debug, Clone)]
pub struct Plan {
    pub order: Vec<usize>,
}

/// Select the join order from sealed footers: repeatedly pick the
/// pattern with the smallest cardinality bound summed over files
/// (filter placement = bound-predicate patterns land first, because they
/// carry the tightest bounds). This is the static analog of SIP.
pub fn plan(query: &Query, footers: &[SealedFooter]) -> Plan {
    let mut remaining: Vec<usize> = (0..query.patterns.len()).collect();
    let mut order = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let best = *remaining
            .iter()
            .min_by_key(|&&i| {
                let pat = &query.patterns[i];
                footers.iter().map(|f| pattern_bound(pat, f)).sum::<u64>()
            })
            .expect("non-empty remaining");
        order.push(best);
        remaining.retain(|&i| i != best);
    }
    Plan { order }
}
