//! Chicago courts for page-generation invalidation. Real collaborators,
//! asserted final state; anti-vacuity mutation flips the stale-fact court.

use stardf_incremental::page::{Fact, PageKind};
use stardf_incremental::store::{encode_records, replay, LogRecord, Store, PAGE_CAPACITY};
use stardf_incremental::CountView;

fn fact(i: u64) -> Fact {
    Fact(i, i % 7, i * 31 % 997)
}

fn seeded_store(pages: usize, per_page: usize) -> (Store, Vec<Fact>) {
    let mut s = Store::new();
    let mut facts = Vec::new();
    for i in 0..(pages * per_page) as u64 {
        facts.push(fact(i));
    }
    let _ = s.seal_base_pages_txn(&facts, per_page);
    (s, facts)
}

/// COURT 1: delete-then-query correctness vs full-recompute oracle.
/// Seed 5+ pages, deletes spanning pages; store result must equal oracle.
#[test]
fn court_delete_then_query_matches_oracle() {
    let (mut s, facts) = seeded_store(6, PAGE_CAPACITY);
    let to_delete: Vec<Fact> = facts.iter().copied().filter(|f| f.0 % 5 == 0).collect();
    let last = *facts.last().unwrap();
    let mut del = to_delete.clone();
    del.push(last);

    let summary = s.bulk_delete(&del, PAGE_CAPACITY);
    assert!(summary.pages_invalidated >= 5, "must span >=5 pages");
    assert_eq!(summary.facts_deleted, del.len() as u64);
    assert_eq!(summary.facts_resealed + summary.facts_deleted, facts.len() as u64);

    let got = s.fact_set();
    let del_set: std::collections::HashSet<Fact> = del.iter().copied().collect();
    let oracle: std::collections::HashSet<Fact> =
        facts.iter().copied().filter(|f| !del_set.contains(f)).collect();
    assert_eq!(got, oracle, "store fact set must equal full-recompute oracle");
    for f in &del {
        assert!(!s.contains(f), "deleted fact still queryable: {:?}", f);
    }
    assert!(s.contains(&facts[1]));
}

/// COURT 2: footer integrity — sealed pages expose footers; delta pages
/// carry kind Delta and matching fact counts.
#[test]
fn court_footer_sequencing() {
    let (mut s, facts) = seeded_store(5, PAGE_CAPACITY);
    let summary = s.bulk_delete(&facts[..10].to_vec(), PAGE_CAPACITY);
    assert!(summary.deltas_sealed >= 1);
    for id in s.delta_ids() {
        let p = s.page(*id).expect("delta page present");
        assert_eq!(p.kind, PageKind::Delta);
        let ft = p.footer();
        assert_eq!(ft.fact_count as usize, p.facts.len());
    }
}

/// COURT 3: power-cut interleave — kill at every byte boundary of the
/// durable log (seal txn + delete txn); reloaded state is never torn:
/// any committed invalidation implies its delta reseal committed too, and
/// no invalidated-page fact is visible without its reseal.
#[test]
fn court_power_cut_never_torn() {
    let mut s = Store::new();
    let facts: Vec<Fact> = (0..(5 * PAGE_CAPACITY) as u64).map(fact).collect();
    let to_delete: Vec<Fact> = facts.iter().copied().filter(|f| f.0 % 7 == 0).collect();

    let seal_txn = s.seal_base_pages_txn(&facts, PAGE_CAPACITY);
    let del_txn = s.bulk_delete(&to_delete, PAGE_CAPACITY).records;
    let log = encode_records(&seal_txn);
    let log2 = encode_records(&del_txn);
    let mut full = log.clone();
    full.extend_from_slice(&log2);

    for cut in 0..=full.len() {
        let st = replay(&full[..cut]);
        let n_delta = st.delta_ids().len();
        for id in st.invalidated() {
            let p = st.page(*id).expect("invalidated page known");
            let live = st.fact_set();
            for f in &p.facts {
                let resealed = st
                    .delta_ids()
                    .iter()
                    .any(|d| st.page(*d).map(|dp| dp.contains(f)) == Some(true));
                assert!(
                    !live.contains(f) || resealed,
                    "torn at byte {}: fact {:?} visible from invalidated page {} without reseal",
                    cut, f, id
                );
            }
        }
        let _ = n_delta;
    }
}

/// COURT 4: materialized view recomputed per delta page matches full
/// recompute oracle.
#[test]
fn court_view_per_delta_page_matches_oracle() {
    let (mut s, facts) = seeded_store(6, PAGE_CAPACITY);
    let mut view = CountView::new();
    view.full_recompute(&s);

    let to_delete: Vec<Fact> = facts.iter().copied().filter(|f| f.0 % 3 == 0).collect();
    s.bulk_delete(&to_delete, PAGE_CAPACITY);
    let folded = view.maintain(&s);
    assert!(folded >= 1, "view must recompute per delta page");

    let mut oracle = CountView::new();
    oracle.full_recompute(&s);
    assert_eq!(view, oracle, "maintained view must equal full recompute");
}

/// COURT 5 (anti-vacuity flip): skip invalidation -> deleted fact still
/// queryable -> stale-fact court fires. The court must flip; a court that
/// cannot flip is FAIL.
#[test]
fn court_stale_fact_flip() {
    let (mut s, facts) = seeded_store(5, PAGE_CAPACITY);
    let target = facts[3];
    let mut del = vec![target, facts[40]];
    del.push(facts[100]);
    s.bulk_delete(&del, PAGE_CAPACITY);
    assert!(!s.contains(&target), "post-delete the fact is gone");
    let n_live = s.fact_set().len();

    // MUTATION: skip invalidation — re-materialize invalidated pages into
    // the live read path (what a never-consulted invalidation marker does).
    let mut mutated = s.clone();
    let records: Vec<LogRecord> = mutated
        .invalidated()
        .iter()
        .copied()
        .collect::<Vec<_>>()
        .iter()
        .filter_map(|id| mutated.page(*id).cloned())
        .map(|mut p| {
            p.id = 10_000 + p.id;
            p.kind = PageKind::Delta;
            LogRecord::Page(p)
        })
        .collect();
    let hash = stardf_incremental::store::hash_records(&records);
    let mut txn = records;
    txn.push(LogRecord::Commit { txn: 999, payload_hash: hash });
    assert!(mutated.apply_checked(&txn), "mutation applied");
    assert!(
        mutated.contains(&target),
        "FLIP WITNESS: with invalidation skipped the deleted fact IS queryable"
    );
    assert!(mutated.fact_set().len() > n_live);
}

/// COURT 6: log round-trip — a full replay reproduces the in-memory state
/// exactly (receipt-checkable durability).
#[test]
fn court_replay_roundtrip() {
    let (mut s, facts) = seeded_store(5, PAGE_CAPACITY);
    let seal_txn = s.seal_base_pages_txn(&facts, PAGE_CAPACITY);
    let del: Vec<Fact> = facts.iter().copied().filter(|f| f.0 % 4 == 0).collect();
    let del_txn = s.bulk_delete(&del, PAGE_CAPACITY).records;
    let mut log = encode_records(&seal_txn);
    log.extend_from_slice(&encode_records(&del_txn));
    let st = replay(&log);
    assert_eq!(st.fact_set(), s.fact_set());
    assert_eq!(st.invalidated(), s.invalidated());
    assert_eq!(st.delta_ids(), s.delta_ids());
}
