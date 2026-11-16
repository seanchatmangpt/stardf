//! RDF-3 anti-vacuity flip court: poison a sealed footer count in a /tmp
//! sandbox copy -> the planner must DETECT the divergence at runtime
//! (typed CountDivergence) or the plan must provably not depend on the
//! poisoned file. A court that cannot flip is FAIL.

mod common;

use common::{corpus, two_hop_query};
use std::fs;
use std::path::PathBuf;
use stardf_planner::exec::{naive_exec, planned_exec};
use stardf_planner::footer::{parse_footer, scan_verified, FooterFault};
use stardf_planner::plan::plan;

/// Sandbox: copy the sealed corpus into /tmp, flip one byte inside the
/// counts table of f0 (the KNOWS count), then FIX THE SEAL so the
/// poisoning is metadata-only — parse_footer passes, scan_verified must
/// fire. sha256 fail-closed restore (via cc sha256sum when present).
#[test]
fn poisoned_footer_count_is_detected_typed() {
    let sandbox = PathBuf::from("/tmp/rdf3-poison-court");
    let _ = fs::remove_dir_all(&sandbox);
    fs::create_dir_all(&sandbox).expect("sandbox created");

    let corpus = corpus();
    let names: Vec<String> = corpus.iter().map(|(n, _)| n.clone()).collect();
    let mut poisoned_idx = None;
    for (i, (_, bytes)) in corpus.iter().enumerate() {
        let path = sandbox.join(format!("{i}.plnf"));
        fs::write(&path, bytes).expect("sandbox write");
        if names[i].starts_with("f0") {
            // Locate the counts table: footer_len at end-4.
            let mut b = bytes.clone();
            let flen = u32::from_le_bytes(
                b[b.len() - 4..].try_into().unwrap(),
            ) as usize;
            // First entry's count lives at footer_start+4 (pred at +0).
            let coff = b.len() - flen + 4;
            assert_eq!(b[coff - 4..coff], 1u32.to_le_bytes(), "first entry is KNOWS");
            b[coff] ^= 0x01; // poison: 40 -> 41
            // Re-seal so the seal channel passes: recompute FNV over span.
            let span_len = flen - 16;
            let span = b[b.len() - flen..b.len() - flen + span_len].to_vec();
            let seal = stardf_planner::footer::fnv1a64(&span);
            let send = b.len() - 16; // seal(8) | magic(4) | footer_len(4)
            b[send..send + 8].copy_from_slice(&seal.to_le_bytes());
            fs::write(&path, &b).expect("poisoned write");
            poisoned_idx = Some(i);
        }
    }
    let pi = poisoned_idx.expect("f0 poisoned in sandbox");

    // Parse side: seal now PASSES (metadata-only poison).
    let good_bytes = &corpus[pi].1;
    let poisoned_bytes = fs::read(sandbox.join(format!("{pi}.plnf"))).unwrap();
    let pf = parse_footer(&poisoned_bytes).expect("re-sealed poison parses clean");
    let gf = parse_footer(good_bytes).unwrap();
    assert_ne!(
        pf, gf,
        "poison must actually change the sealed count (flip non-vacuous)"
    );

    // Runtime side: the scan MUST detect the divergence, typed.
    let err = scan_verified(&poisoned_bytes, &pf).unwrap_err();
    match err {
        FooterFault::CountDivergence { pred, sealed, actual } => {
            assert_ne!(sealed, actual, "divergence carries distinct sealed/actual");
            assert_eq!(pred, common::KNOWS);
            println!("flip witnessed: CountDivergence pred={pred} sealed={sealed} actual={actual}");
        }
        other => panic!("expected CountDivergence, got {other:?}"),
    }

    // And the executor surfaces the same typed refusal end-to-end.
    let mut corpus2 = common::corpus();
    corpus2[pi].1 = poisoned_bytes;
    let files: Vec<(&[u8], _)> = corpus2
        .iter()
        .map(|(_, b)| (b.as_slice(), parse_footer(b).unwrap()))
        .collect();
    let footers: Vec<_> = files.iter().map(|(_, f)| f.clone()).collect();
    let q = two_hop_query();
    let p = plan(&q, &footers);
    let pe = planned_exec(&q, &files, &p).unwrap_err();
    assert!(
        matches!(pe, FooterFault::CountDivergence { .. }),
        "planned exec must refuse with typed divergence, got {pe}"
    );
    let ne = naive_exec(&q, &files).unwrap_err();
    assert!(matches!(ne, FooterFault::CountDivergence { .. }));

    // Cleanup sandbox.
    fs::remove_dir_all(&sandbox).expect("sandbox removed");
}

/// Baseline control: the UNPOISONED corpus runs clean end-to-end (the
/// court above must not pass because everything always fails).
#[test]
fn unpoisoned_baseline_runs_clean() {
    let corpus = corpus();
    let files: Vec<(&[u8], _)> = corpus
        .iter()
        .map(|(_, b)| (b.as_slice(), parse_footer(b).unwrap()))
        .collect();
    let footers: Vec<_> = files.iter().map(|(_, f)| f.clone()).collect();
    let q = two_hop_query();
    let p = plan(&q, &footers);
    let r = planned_exec(&q, &files, &p).expect("clean corpus runs clean");
    assert!(!r.solutions.is_empty());
}
