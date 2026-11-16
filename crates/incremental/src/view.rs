//! Materialized count view: per-predicate fact counts. Recomputed
//! PER-DELTA-PAGE (one incremental update per sealed delta page), never
//! per-fact — the page-granularity analog of OST's per-fact view deltas.

use crate::page::{Fact, Page};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct CountView {
    counts: HashMap<u64, u64>,
    deltas_folded: usize,
    removed_pages: std::collections::BTreeSet<u64>,
}

impl PartialEq for CountView {
    fn eq(&self, other: &Self) -> bool {
        self.counts == other.counts
    }
}

impl Eq for CountView {}

impl CountView {
    pub fn new() -> Self {
        CountView {
            counts: HashMap::new(),
            deltas_folded: 0,
            removed_pages: Default::default(),
        }
    }

    pub fn get(&self, predicate: u64) -> u64 {
        self.counts.get(&predicate).copied().unwrap_or(0)
    }

    pub fn total(&self) -> u64 {
        self.counts.values().sum()
    }

    fn add_page(&mut self, page: &Page) {
        for &Fact(_, p, _) in &page.facts {
            *self.counts.entry(p).or_insert(0) += 1;
        }
    }

    fn add_facts(&mut self, facts: &[Fact]) {
        for &Fact(_, p, _) in facts {
            *self.counts.entry(p).or_insert(0) += 1;
        }
    }

    fn remove_facts(&mut self, facts: &[Fact]) {
        for &Fact(_, p, _) in facts {
            let e = self.counts.entry(p).or_insert(0);
            *e = e.saturating_sub(1);
        }
    }

    /// Full recompute over the store (the oracle path).
    pub fn full_recompute(&mut self, store: &crate::store::Store) {
        self.counts.clear();
        self.deltas_folded = 0;
        self.removed_pages.clear();
        for p in store.live_pages() {
            self.add_page(p);
        }
    }

    /// PER-DELTA-PAGE maintenance: fold each newly sealed delta page in
    /// once (cursor over delta ids) and drop each newly invalidated page's
    /// contribution once. Returns the number of delta pages folded.
    pub fn maintain(&mut self, store: &crate::store::Store) -> usize {
        for id in store.invalidated() {
            if self.removed_pages.insert(*id) {
                if let Some(p) = store.page(*id) {
                    self.remove_facts(&p.facts);
                }
            }
        }
        let mut folded = 0;
        let deltas = store.delta_ids();
        while self.deltas_folded < deltas.len() {
            let p = store
                .page(deltas[self.deltas_folded])
                .expect("sealed delta page present");
            self.add_page(p);
            self.deltas_folded += 1;
            folded += 1;
        }
        folded
    }
}
