// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fixed-width rules and atoms over caller-interned u64 term ids.
//!
//! Encoding: a slot is a u64. A slot with `VAR_TAG` (0xC000_0000_0000_0000)
//! set is a variable whose index is in the low 31 bits; index 0 is the
//! anonymous wildcard (binds nothing, matches anything). Every other u64 is a
//! literal term id. Term ids MUST have the top two bits 00 (the corpus masks
//! with `TERM_MASK`); this is asserted at program build.

pub const ARITY: usize = 4;
/// Padding slot value for arities < 4.
pub const PAD: u64 = 0;
/// Variable tag: top two bits set.
pub const VAR_TAG: u64 = 0xC000_0000_0000_0000;
/// Wildcard var index (never bound, always matches).
pub const WILD: u64 = VAR_TAG; // var index 0
/// Term ids are masked to keep the top two bits clear.
pub const TERM_MASK: u64 = 0x3FFF_FFFF_FFFF_FFFF;
pub const MAX_BODY: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Atom {
    pub pred: u32,
    pub args: [u64; ARITY],
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub head: Atom,
    pub body: Vec<Atom>,
    /// Index into body of the (last) negated atom, or usize::MAX.
    pub neg: usize,
    /// Assigned by [stratify::Stratification::new]; must be set before eval.
    pub stratum: u32,
}

impl Rule {
    pub fn new(head: Atom, body: Vec<Atom>) -> Rule {
        Rule { head, body, neg: usize::MAX, stratum: 0 }
    }
    pub fn negated(mut self, idx: usize) -> Rule {
        self.neg = idx; self
    }
}

impl SpineProgram {
    /// Read-only view of the base facts for one predicate.
    pub fn base_facts(&self, pred: u32) -> &[[u64; ARITY]] {
        &self.facts[pred as usize]
    }
}

#[derive(Clone, Debug)]
pub struct SpineProgram {
    pub rules: Vec<Rule>,
    pub num_preds: usize,
    /// Per-predicate max arity seen in facts (for padding at fact build).
    pub arities: Vec<usize>,
    /// Upper bound on facts per predicate (used to pre-reserve pages).
    pub fact_cap: usize,
    /// Base facts per predicate (fixed-width rows, padded to ARITY).
    pub(crate) facts: Vec<Vec<[u64; ARITY]>>,
}

/// Builder: registers predicates, facts, and rules; pre-reserves pages.
pub struct SpineProgramBuilder {
    facts: Vec<Vec<[u64; ARITY]>>,
    rules: Vec<Rule>,
    arities: Vec<usize>,
    fact_cap: usize,
}

impl SpineProgramBuilder {
    pub fn new(num_preds: usize, fact_cap: usize) -> SpineProgramBuilder {
        SpineProgramBuilder {
            facts: vec![Vec::new(); num_preds],
            rules: Vec::new(),
            arities: vec![0; num_preds],
            fact_cap,
        }
    }

    pub fn fact(&mut self, pred: u32, args: &[u64]) -> &mut Self {
        assert!(args.len() <= ARITY);
        let mut row = [PAD; ARITY];
        row[..args.len()].copy_from_slice(args);
        let n = args.len();
        self.arities[pred as usize] = self.arities[pred as usize].max(n);
        self.facts[pred as usize].push(row);
        self
    }

    pub fn rule(&mut self, rule: Rule) -> &mut Self {
        for a in &rule.body {
            assert!((a.pred as usize) < self.facts.len());
        }
        self.rules.push(rule);
        self
    }

    /// Consumes the builder: asserts term tags, returns the program.
    pub fn build(self) -> SpineProgram {
        for page in &self.facts {
            for row in page {
                for slot in row {
                    assert!(slot & VAR_TAG != VAR_TAG, "term id collides with VAR_TAG");
                }
            }
        }
        SpineProgram {
            fact_cap: self.fact_cap,
            rules: self.rules,
            num_preds: self.facts.len(),
            arities: self.arities,
            facts: self.facts,
        }
    }
}
