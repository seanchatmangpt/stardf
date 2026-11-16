//! Bench: bulk-delete throughput (facts/sec invalidated+resealed) and the
//! granularity boundary: page-granularity delete vs per-fact oracle-style
//! set removal. Per-fact wins below the crossover edit size; page
//! granularity wins above it.

use stardf_incremental::page::Fact;
use stardf_incremental::store::{Store, PAGE_CAPACITY};
use std::collections::HashSet;
use std::time::Instant;

fn fact(i: u64) -> Fact {
    Fact(i, i % 7, i * 31 % 997)
}

fn main() {
    const PAGES: usize = 256;
    const N: usize = PAGES * PAGE_CAPACITY;

    // Bulk-delete throughput across delete fractions.
    println!("bulk_delete throughput (facts/sec invalidated+resealed):");
    for frac in [0.01, 0.1, 0.5, 0.9] {
        let facts: Vec<Fact> = (0..N as u64).map(fact).collect();
        let mut s = Store::new();
        s.seal_base_pages(&facts, PAGE_CAPACITY);
        let ndel = (N as f64 * frac) as usize;
        let to_delete: Vec<Fact> = facts.iter().step_by((N / ndel.max(1)).max(1)).copied().take(ndel).collect();
        let t0 = Instant::now();
        let summary = s.bulk_delete(&to_delete, PAGE_CAPACITY);
        let dt = t0.elapsed().as_secs_f64();
        let rate = summary.facts_deleted as f64 / dt;
        println!(
            "  frac={:>4} deleted={} pages_inv={} deltas={} -> {:.0} facts/sec",
            frac, summary.facts_deleted, summary.pages_invalidated, summary.deltas_sealed, rate
        );
    }

    // Boundary: per-edit cost, page-granularity vs per-fact set removal.
    println!("boundary (per-edit cost, us): page_granularity vs per_fact_set");
    for k in [1usize, 4, 16, 64, 256] {
        let facts: Vec<Fact> = (0..N as u64).map(fact).collect();
        // Page granularity: k surgical edits, each invalidating + resealing.
        let mut s = Store::new();
        s.seal_base_pages(&facts, PAGE_CAPACITY);
        let t0 = Instant::now();
        for i in 0..k {
            s.bulk_delete(&[fact(i as u64 * 13)], PAGE_CAPACITY);
        }
        let page_us = t0.elapsed().as_secs_f64() * 1e6 / k as f64;

        // Per-fact: hash-set removal (OST-style surgical index edit).
        let mut set: HashSet<Fact> = facts.iter().copied().collect();
        let t1 = Instant::now();
        for i in 0..k {
            set.remove(&fact(i as u64 * 13));
        }
        let perfact_us = t1.elapsed().as_secs_f64() * 1e6 / k as f64;
        println!("  k={:>4}  page={:9.3}  per_fact={:9.3}", k, page_us, perfact_us);
    }
    println!(
        "boundary note: page granularity reseals ~PAGE_CAPACITY={} survivors per edit; \
         per-fact wins when edits touch <1 fact per page on average and pages are large",
        PAGE_CAPACITY
    );
}
