// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The SDPF driver: per-stratum bounded fixpoint over delta pages.

use super::rule::{Atom, Rule, SpineProgram, MAX_BODY, ARITY, PAD, VAR_TAG, WILD};

/// Sentinel for "variable not yet bound" (term 0 is a legitimate value).
const UNBOUND: u64 = u64::MAX;
use super::stratify::Stratification;

#[derive(Debug, Clone)]
pub enum SpineError {
    /// Per-stratum iteration cap hit: a typed refusal naming the stratum and
    /// the iteration at which the cap was reached. Never a silent truncation.
    BoundedRefusal {
        stratum: u32,
        iteration: usize,
        cap: usize,
    },
    /// A pre-reserved page capacity was exceeded: refuse, don't reallocate.
    CapacityExceeded {
        pred: u32,
        cap: usize,
    },
    Stratification(crate::spine::stratify::StratificationRefusal),
}

/// Evaluation options.
pub struct EvalOptions {
    /// Per-stratum iteration cap (termination bound, witnessed).
    pub max_iterations: usize,
    /// Force-derive every rule head (anti-vacuity flip hook; court-only).
    pub force_derive: bool,
}

impl Default for EvalOptions {
    fn default() -> Self {
        EvalOptions { max_iterations: 1024, force_derive: false }
    }
}

/// A page: fixed-width rows, sorted prefix + unsorted tail.
struct Page {
    rows: Vec<[u64; ARITY]>,
    /// Length of the sorted, deduped prefix.
    sorted_len: usize,
    cap: usize,
}

impl Page {
    fn new(cap: usize) -> Page {
        Page { rows: Vec::with_capacity(cap), sorted_len: 0, cap }
    }

    /// Appends a row; refuses past capacity (never reallocates past the cap).
    fn push(&mut self, row: [u64; ARITY]) -> Result<(), u32> {
        if self.rows.len() >= self.cap {
            return Err(0);
        }
        self.rows.push(row);
        Ok(())
    }

    /// Membership: binary search sorted prefix, linear scan of tail.
    fn contains(&self, row: &[u64; ARITY]) -> bool {
        let (sorted, tail) = self.rows.split_at(self.sorted_len);
        sorted.binary_search(row).is_ok()
            || tail.iter().any(|r| r == row)
    }

    /// Merges the tail into the sorted prefix, in place (alloc-free).
    fn consolidate(&mut self) {
        self.rows.sort_unstable();
        self.rows.dedup();
        self.sorted_len = self.rows.len();
    }
}

/// The evaluator: owns pre-reserved pages; single writer per stratum.
pub struct SpineEval {
    pages: Vec<Page>,
    /// Per-iteration delta buffers (one per predicate, reused each iteration).
    delta: Vec<Vec<[u64; ARITY]>>,
    delta_cap: usize,
    stats: SpineStats,
}

#[derive(Debug, Default, Clone)]
pub struct SpineStats {
    pub iterations: Vec<usize>,
    pub derivations: u64,
    pub facts_final: u64,
    pub strata_run: u32,
}

impl SpineEval {
    /// Builds the evaluator from the program: pre-reserves every page and
    /// delta buffer; the evaluation path itself never allocates (the only
    /// Vec growth is the pre-reserved page push; past-cap is a refusal).
    pub fn new(program: &SpineProgram, strat: &Stratification, cap: usize) -> SpineEval {
        let pages = (0..program.num_preds)
            .map(|p| {
                let mut pg = Page::new(cap);
                for row in &program.facts[p] {
                    pg.rows.push(*row);
                }
                pg.consolidate();
                pg
            })
            .collect();
        SpineEval {
            delta: (0..program.num_preds).map(|_| Vec::with_capacity(cap)).collect(),
            delta_cap: cap,
            pages,
            stats: SpineStats::default(),
        }
    }

    pub fn stats(&self) -> &SpineStats {
        &self.stats
    }

    pub fn page(&self, pred: u32) -> &[[u64; ARITY]] {
        &self.pages[pred as usize].rows
    }

    /// Runs the bounded stratified fixpoint.
    pub fn run(
        &mut self,
        program: &SpineProgram,
        opts: &EvalOptions,
    ) -> Result<(), SpineError> {
        let max_stratum = program
            .rules
            .iter()
            .map(|r| r.stratum)
            .max()
            .unwrap_or(0);
        self.stats.strata_run = max_stratum + 1;
        for s in 0..=max_stratum {
            self.run_stratum(program, s, opts)?;
        }
        let facts_final: u64 = self.pages.iter().map(|p| p.rows.len() as u64).sum();
        self.stats.facts_final = facts_final;
        Ok(())
    }

    fn run_stratum(
        &mut self,
        program: &SpineProgram,
        s: u32,
        opts: &EvalOptions,
    ) -> Result<(), SpineError> {
        let rules: Vec<usize> = (0..program.rules.len())
            .filter(|i| program.rules[*i].stratum == s)
            .collect();
        if rules.is_empty() {
            return Ok(());
        }
        let mut iteration = 0usize;
        loop {
            // Delta pages are cleared at the top of each iteration: the
            // stratum's single writer owns them for the iteration's duration.
            for d in self.delta.iter_mut() {
                d.clear();
            }
            let mut derived: u64 = 0;
            for ri in &rules {
                derived += self.fire_rule(&program.rules[*ri], opts)?;
            }
            self.stats.derivations += derived;
            // Consolidate: sort+dedup merges tails into sorted prefixes.
            let mut new_facts = 0usize;
            for (p, d) in self.delta.iter().enumerate() {
                let page = &mut self.pages[p];
                for row in d {
                    if !page.contains(row) {
                        page.push(*row).map_err(|_| {
                            SpineError::CapacityExceeded { pred: p as u32, cap: page.cap }
                        })?;
                        new_facts += 1;
                    }
                }
            }
            for page in self.pages.iter_mut() {
                if page.sorted_len < page.rows.len() {
                    page.consolidate();
                }
            }
            iteration += 1;
            #[cfg(feature = "spine_trace")]
            eprintln!("[spine] stratum {s} it{iteration} new={new_facts} tcpage={:?}", self.pages[2].rows);
            self.stats.iterations.push(iteration);
            if new_facts == 0 {
                return Ok(());
            }
            if iteration >= opts.max_iterations {
                return Err(SpineError::BoundedRefusal {
                    stratum: s,
                    iteration,
                    cap: opts.max_iterations,
                });
            }
        }
    }


    /// Fires one rule against the current pages; returns derivation count.
    ///
    /// `force_derive` is the anti-vacuity flip hook: when set, every rule
    /// derives exactly one head row with all head variables grounded to PAD
    /// (term 0), constants verbatim, body match bypassed — so an unsatisfied
    /// body "derives". A court asserting no-derive-without-satisfied-body
    /// must fail under this mutation; that is the non-vacuity witness.
    fn fire_rule(
        &mut self,
        rule: &Rule,
        opts: &EvalOptions,
    ) -> Result<u64, SpineError> {
        if opts.force_derive {
            let mut row = [PAD; ARITY];
            for (i, slot) in rule.head.args.iter().enumerate() {
                if *slot & VAR_TAG == 0 {
                    row[i] = *slot;
                }
            }
            self.delta[rule.head.pred as usize].push(row);
            return Ok(1);
        }
        let mut bindings = [UNBOUND; 32];
        let mut matches = [0usize; MAX_BODY];
        self.eval_body(rule, 0, &mut bindings, &mut matches, 0)
    }

    /// Recursive body evaluation over the fixed body length.
    ///
    /// `bindings` maps var index -> value (wildcard never binds). `matches`
    /// records the row index matched at each positive atom (unused beyond
    /// recursion plumbing; kept fixed-width by construction).
    fn eval_body(
        &mut self,
        rule: &Rule,
        atom_idx: usize,
        bindings: &mut [u64; 32],
        matches: &mut [usize; MAX_BODY],
        mut count: u64,
    ) -> Result<u64, SpineError> {
        if atom_idx == rule.neg {
            // Negated atom: ground it from bindings; absence check.
            let a = &rule.body[atom_idx];
            let row = self.ground(a, bindings);
            if self.pages[a.pred as usize].contains(&row) {
                return Ok(count);
            }
            return self.eval_body(rule, atom_idx + 1, bindings, matches, count);
        }
        if atom_idx == rule.body.len() {
            // All positives matched (negation passed): project the head.
            let row = self.ground(&rule.head, bindings);
            self.delta[rule.head.pred as usize].push(row);
            return Ok(count + 1);
        }
        // Positive atom: scan its page, extend bindings, recurse.
        let a = rule.body[atom_idx].clone();
        let page_len = self.pages[a.pred as usize].rows.len();
        for row_idx in 0..page_len {
            let row = self.pages[a.pred as usize].rows[row_idx];
            if let Some(ext) = match_row(&a, &row, bindings) {
                let saved = *bindings;
                *bindings = ext;
                count = self.eval_body(rule, atom_idx + 1, bindings, matches, count)?;
                *bindings = saved;
            }
        }
        Ok(count)
    }

    fn ground(&self, atom: &Atom, bindings: &[u64; 32]) -> [u64; ARITY] {
        let mut row = [PAD; ARITY];
        for (i, slot) in atom.args.iter().enumerate() {
            row[i] = if *slot & VAR_TAG == VAR_TAG {
                if *slot == WILD { PAD } else {
                    let b = bindings[(*slot & !VAR_TAG) as usize];
                    if b == UNBOUND { PAD } else { b }
                }
            } else {
                *slot
            };
        }
        row
    }
}

/// Matches `atom` against a page row under `bindings`.
/// Returns the extended binding set, or None if no match.
fn match_row(
    atom: &Atom,
    row: &[u64; ARITY],
    bindings: &[u64; 32],
) -> Option<[u64; 32]> {
    let mut ext = *bindings;
    for (i, slot) in atom.args.iter().enumerate() {
        if *slot & VAR_TAG == VAR_TAG {
            if *slot == WILD {
                continue; // anonymous: matches anything, binds nothing
            }
            let v = (*slot & !VAR_TAG) as usize;
            if ext[v] == UNBOUND {
                ext[v] = row[i];
            } else if ext[v] != row[i] {
                return None;
            }
        } else if *slot != row[i] {
            return None;
        }
    }
    Some(ext)
}
