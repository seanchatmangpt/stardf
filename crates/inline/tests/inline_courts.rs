//! Chicago courts for PP-4 inline structural folding.
//!
//! Real collaborators (stardf-star store, stardf-inline store, BLAKE3 via
//! stardf-hash) — no mocks, assert final state. Anti-vacuity flips live
//! in scripts/courts.sh (SHIM-COURT mutations on a /tmp sandbox copy with
//! sha256 fail-closed restore): mutation A = collision-trap bypass,
//! mutation B = wrong arena index. A court that cannot flip is FAIL.

use stardf_inline::{fold_word, Folded, InlineStore, Trap};
use stardf_star::store::Store;
use stardf_star::term::{canonical_quoted, Term};

/// Shared deterministic corpus: 5 subjects x 4 predicates x 3 objects =
/// 60 quoted triples, 6 nested (subject is itself a quoted triple).
fn corpus_terms() -> Vec<(Term, String, Term)> {
    let mut out = Vec::new();
    for si in 0..5 {
        for pi in 0..4 {
            for oi in 0..3 {
                let subject: Term = if si == 0 && pi == 0 && oi < 6 {
                    // nested: subject is a quoted triple term
                    Term::Quoted(stardf_star::term::quoted_id(
                        &Term::Iri("https://ex.test/deep".into()),
                        "https://ex.test/p0",
                        &Term::Literal { value: format!("d{}", oi), lang: None, datatype: None },
                    ))
                } else {
                    Term::Iri(format!("https://ex.test/s{}", si))
                };
                out.push((
                    subject,
                    format!("https://ex.test/p{}", pi),
                    Term::Literal {
                        value: format!("v{}-{}-{}", si, pi, oi),
                        lang: None,
                        datatype: None,
                    },
                ))
            }
        }
    }
    out
}

/// court1: DIFFERENTIAL vs stardf-star. The same corpus through both
/// representations must produce the same quoted-payload registry: counts
/// equal, per-payload presence equal, and each payload's folded word's
/// fingerprint must be recoverable by fold, not stored guesswork.
#[test]
fn court1_differential_vs_star() {
    let terms = corpus_terms();
    // stardf-star side
    let mut star = Store::new();
    let mut star_by_payload: std::collections::BTreeMap<Vec<u8>, u64> = Default::default();
    for (s, p, o) in &terms {
        let q = star.quote(s.clone(), p, o.clone());
        if let Term::Quoted(id) = q {
            star_by_payload.insert(canonical_quoted(s, p, o), id);
        } else {
            panic!("star quote returned a non-quoted term");
        }
    }
    // stardf-inline side
    let mut inline = InlineStore::new();
    let mut inline_by_payload: std::collections::BTreeMap<Vec<u8>, Folded> = Default::default();
    for (s, p, o) in &terms {
        let f = inline
            .fold_quote(s.clone(), p, o.clone())
            .expect("fold of fresh corpus payload must not trap");
        inline_by_payload.insert(canonical_quoted(s, p, o), f);
    }
    // registry equality
    assert_eq!(star.quoted_count(), inline.quoted_count());
    assert_eq!(star.quoted_count(), 60, "corpus is 60 quoted triples");
    assert_eq!(star_by_payload.len(), inline_by_payload.len());
    for (payload, _) in &star_by_payload {
        assert!(
            inline_by_payload.contains_key(payload),
            "payload present in star but missing in inline"
        );
    }
    // every folded word carries the discriminator and round-trips: the
    // payload found by lookup must re-fold to the same fingerprint
    for f in inline_by_payload.values() {
        assert!(f.is_folded());
        let payload = inline.lookup(*f).expect("lookup must not trap");
        assert_eq!(
            stardf_inline::fold_fingerprint(&payload.canonical()),
            f.fingerprint(),
            "fold must be recoverable from the payload"
        );
    }
    // idempotent dedupe: refolding the same payloads adds nothing
    for (s, p, o) in &terms {
        inline
            .fold_quote(s.clone(), p, o.clone())
            .expect("refold must dedupe, not trap");
    }
    assert_eq!(inline.quoted_count(), 60);
    assert!(inline.traps().is_empty());
    assert!(inline.verify_integrity().is_empty());
}

/// court2: COLLISION TRAP + integrity (anti-vacuity core). Corrupt a real
/// arena slot and a real table index; every read path must trap typed,
/// the trap log must witness it, and the store must refuse to dedupe a
/// corrupted re-fold.
#[test]
fn court2_collision_trap_and_integrity() {
    let terms = corpus_terms();
    let mut inline = InlineStore::new();
    let mut folded = Vec::new();
    for (s, p, o) in &terms {
        folded.push(inline.fold_quote(s.clone(), p, o.clone()).unwrap());
    }

    // (a) arena corruption: mutate a live payload in place
    let victim = folded[7];
    let fp7 = victim.fingerprint();
    let idx7 = inline.index_of(fp7).expect("index mapped");
    inline.arena_mut().slot_mut(idx7).unwrap().payload.p =
        "https://ex.test/PWNED".to_string();
    match inline.lookup(victim) {
        Err(Trap::ArenaCorruption) => {}
        other => panic!("arena corruption must trap, got {:?}", other.map(|_| ())),
    }
    assert_eq!(inline.verify_integrity(), vec![fp7], "integrity walk names exactly the corrupted slot");

    // (b) out-of-bounds index: a fingerprint never registered
    let bogus = fold_word(b"never-registered-payload");
    assert_eq!(inline.lookup(bogus), Err(Trap::IndexOutOfBounds));

    // (c) collision trap via the fold path: force a second payload onto
    // an occupied fingerprint by writing the table entry directly through
    // the public seam (index_of + table are the store's state; here we
    // assert the fold path traps when checksum != stored).
    // Refold after corruption: the stored slot's payload no longer equals
    // the incoming payload, so fold_quote MUST trap FoldCollision rather
    // than silently overwrite or accept.
    let (s, p, o) = &terms[7];
    match inline.fold_quote(s.clone(), p, o.clone()) {
        Err(_) => {}
        Ok(_) => panic!("refold against a corrupted slot must trap, not pass"),
    }
    assert!(
        inline.traps().iter().any(|(t, fp)| *t == Trap::FoldCollision && *fp == fp7),
        "trap log must witness the fold collision on the corrupted slot"
    );
}

/// court3: SCAN-CLASS MEASUREMENT (measured, not assumed). Folding must
/// preserve the dense-u64 scan class: folded words are indirection-free
/// u64s, so a checksum scan over folded words should run at materially
/// the same bandwidth as the same scan over raw unfolded words. We MEASURE
/// both and fence on a relative ratio (no absolute-ms fence), plus print
/// the GB/s numbers for the receipt.
#[test]
fn court3_scan_class_preserved() {
    use std::hint::black_box;
    use std::time::Instant;

    const N: usize = 16_000_000; // 128 MB of u64 words
    let raw: Vec<u64> = (0..N as u64).map(|i| i & !(1u64 << 63)).collect();
    let folded: Vec<stardf_inline::Folded> =
        raw.iter().map(|w| stardf_inline::Folded(w | (1u64 << 63))).collect();

    let bytes = (N * 8) as f64 / 1e9;
    let mut sink: u64 = 0;

    let t0 = Instant::now();
    for &w in &raw {
        sink = sink.wrapping_add(w & 1);
    }
    let t_raw = t0.elapsed().as_secs_f64();
    black_box(sink);

    let t1 = Instant::now();
    for &w in &folded {
        sink = sink.wrapping_add((w.0 >> 63) & 1);
    }
    let t_folded = t1.elapsed().as_secs_f64();
    black_box(sink);

    let gb_raw = bytes / t_raw;
    let gb_folded = bytes / t_folded;
    println!(
        "SCAN-COURT raw {:.1} GB/s | folded {:.1} GB/s | ratio {:.3} ({} words, {:.0} MB)",
        gb_raw, gb_folded, gb_folded / gb_raw, N, bytes * 1000.0
    );
    // Relative ratio fence: folding must not collapse the scan class.
    assert!(
        gb_folded / gb_raw >= 0.5,
        "folded scan must stay within 2x of raw u64 scan (ratio {:.3})",
        gb_folded / gb_raw
    );
    // Absolute floor so a pessimized build cannot pass silently.
    assert!(gb_folded >= 1.0, "folded scan {:.1} GB/s below floor", gb_folded);
}

/// court4: COLLISION-ANALYSIS WITNESS. The receipt's centerpiece math,
/// executed: birthday expectation over the 63-bit fingerprint space.
/// p_any ≈ 1 - exp(-n(n-1)/2^64) ≈ n^2/2^64 for n << 2^32. Assert the
/// formula against independently computed values and against the corpus
/// bound the receipt states (10^8 and 10^9 quoted triples).
#[test]
fn court4_collision_probability_math() {
    fn p_any(n: f64) -> f64 {
        1.0 - (-n * (n - 1.0) / 2f64.powi(64)).exp()
    }
    // n = 10^8: p ≈ 5.42e-4 ; n = 10^9: p ≈ 5.4e-2 ; n = 10^6: p ≈ 5.4e-11
    let p8 = p_any(1e8);
    let p9 = p_any(1e9);
    let p6 = p_any(1e6);
    assert!((p8 - 5.42e-4).abs() < 1e-5, "n=1e8 p={} ", p8);
    assert!((p9 - 5.42e-2).abs() < 2e-3, "n=1e9 p={} ", p9);
    assert!(p6 < 1e-7, "n=1e6 p={} ", p6);
    // Quadratic scaling witness: 10x corpus -> ~100x probability
    // (exponential correction makes it slightly under 100).
    assert!((p9 / p8 - 100.0).abs() < 5.0);
    println!(
        "COLLISION-COURT 63-bit fingerprint: p(any collision) n=1e6: {:.3e}, n=1e8: {:.4}, n=1e9: {:.3}",
        p6, p8, p9
    );
    // The bound that makes the trap MANDATORY rather than optional: the
    // expected number of colliding PAIRS is ~n^2/2^65 (pairs), which
    // crosses 1 at n ≈ 4.3e9 — past the 10^9 corpus bound the trap is a
    // live mechanism, not decoration.
    let expected_pairs = 1e10 * 1e10 / 2f64.powi(65);
    assert!(expected_pairs > 0.5, "birthday bound must bind by n=1e10");
}
