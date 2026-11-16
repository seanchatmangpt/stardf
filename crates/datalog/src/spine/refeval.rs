// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Naive stratified reference evaluator — the independent oracle for the
//! differential court. Deliberately simple (full re-derivation each round
//! over a BTreeSet of rows); NOT the zero-alloc subject under test.

use std::collections::BTreeSet;

use super::corpus::{corpus_program, NUM_PREDS};
use super::rule::{Atom, Rule, PAD, VAR_TAG, WILD};

const UNBOUND: u64 = u64::MAX;

pub struct RefResult {
    /// Sorted, deduped rows per predicate (final least model).
    pub pages: Vec<BTreeSet<[u64; 4]>>,
}

pub fn reference_eval() -> RefResult {
    let program = corpus_program();
    let mut pages: Vec<BTreeSet<[u64; 4]>> = vec![BTreeSet::new(); NUM_PREDS];
    for (p, rows) in program.facts.iter().enumerate() {
        pages[p].extend(rows.iter().copied());
    }
    let max_stratum = program.rules.iter().map(|r| r.stratum).max().unwrap_or(0);
    for s in 0..=max_stratum {
        let rules: Vec<&Rule> = program
            .rules
            .iter()
            .filter(|r| r.stratum == s)
            .collect();
        loop {
            let before: usize = pages.iter().map(|p| p.len()).sum();
            let mut derived: Vec<(usize, [u64; 4])> = Vec::new();
            for rule in &rules {
                fire(&program, rule, &pages, &mut derived);
            }
            for (p, row) in &derived {
                pages[*p].insert(*row);
            }
            let after: usize = pages.iter().map(|p| p.len()).sum();
            if after == before {
                break; // fixpoint: the round derived nothing new
            }
        }
    }
    RefResult { pages }
}

fn fire(program: &super::rule::SpineProgram, rule: &Rule, pages: &[BTreeSet<[u64; 4]>],
        out: &mut Vec<(usize, [u64; 4])>) {
    let mut bindings = [UNBOUND; 8];
    walk(rule, 0, &mut bindings, pages, program, out);
}

fn walk(rule: &Rule, idx: usize, bindings: &mut [u64; 8],
        pages: &[BTreeSet<[u64; 4]>], program: &super::rule::SpineProgram,
        out: &mut Vec<(usize, [u64; 4])>) {
    if idx == rule.neg {
        let a = &rule.body[idx];
        let row = ground(a, bindings);
        if pages[a.pred as usize].contains(&row) {
            return;
        }
        walk(rule, idx + 1, bindings, pages, program, out);
        return;
    }
    if idx == rule.body.len() {
        out.push((rule.head.pred as usize, ground(&rule.head, bindings)));
        return;
    }
    let a = &rule.body[idx];
    for row in pages[a.pred as usize].iter().copied() {
        if let Some(ext) = match_row(a, &row, bindings) {
            let saved = *bindings;
            *bindings = ext;
            walk(rule, idx + 1, bindings, pages, program, out);
            *bindings = saved;
        }
    }
}

fn ground(atom: &Atom, bindings: &[u64; 8]) -> [u64; 4] {
    let mut row = [PAD; 4];
    for (i, slot) in atom.args.iter().enumerate() {
        row[i] = if *slot & VAR_TAG == VAR_TAG {
            if *slot == WILD {
                PAD
            } else {
                let b = bindings[(*slot & !VAR_TAG) as usize];
                if b == UNBOUND { PAD } else { b }
            }
        } else {
            *slot
        };
    }
    row
}

fn match_row(atom: &Atom, row: &[u64; 4], bindings: &[u64; 8]) -> Option<[u64; 8]> {
    let mut ext = *bindings;
    for (i, slot) in atom.args.iter().enumerate() {
        if *slot & VAR_TAG == VAR_TAG {
            if *slot == WILD {
                continue;
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
