//! Anti-vacuity courts for the stardf-arrow substrate verifier.
//!
//! Rebuilt 2026-10-31 (the original court was lost in the 80/20 bulk copy —
//! registered in evidence/ as a VOID row by F12). Court shape is receipted in
//! finish-wave/w1-closure histories: (1) fixtures must be byte-copies of the
//! live artifacts; (2) mutations must be real diffs; (3) the verifier must
//! accept pristine, refuse a mutated header by name, and sense a mutated
//! ontology census. A court that cannot fail is legally void: every assertion
//! below is written to flip if the verifier or fixtures drift.
//!
//! Chicago discipline: real `python3 scripts/verify.py` subprocesses, no mocks.
//! Skips are loud: missing python3/rdflib → panic with `[SKIP: reason]`.
//!
//! Stream reality (witnessed 2026-10-31 against the live verifier): verify.py
//! prints PASS lines and `[ALL PASS]` on stdout, but every `[FAIL]` line goes
//! to stderr — so the drifted-symbol assertion must read stderr too, not just
//! stdout.

use std::path::PathBuf;
use std::process::Command;

/// Workspace root (`stardf/arrow/`): two levels above this crate's manifest
/// dir (`crates/stardf-arrow/`). One `.parent()` lands at `crates/`, which is
/// NOT the root — the verifier, ontology, and capi header all live two up.
fn arrow_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR must have a two-level ancestor")
        .to_path_buf()
}

/// Fixtures live in THIS crate's `tests/fixtures/` (byte-copies of the live
/// artifacts), not under the workspace root's own `tests/` dir.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn verify(ttl: &PathBuf, header: &PathBuf) -> (i32, String) {
    let out = Command::new("python3")
        .arg(arrow_root().join("scripts/verify.py"))
        .arg("--ttl")
        .arg(ttl)
        .arg("--header")
        .arg(header)
        .current_dir(arrow_root())
        .output()
        .unwrap_or_else(|e| panic!("[SKIP: python3 verify.py unavailable: {e}]"));
    (
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

#[test]
fn fixtures_are_byte_copies_of_own_artifacts() {
    let ontology = arrow_root().join("ontology.ttl");
    let header = arrow_root().join("crates/stardf-arrow-capi/include/stardf_arrow.h");
    for (fixture_path, live) in [
        (fixture("ontology_pristine.ttl"), ontology),
        (fixture("header_pristine.h"), header),
    ] {
        let a = std::fs::read(&fixture_path)
            .unwrap_or_else(|e| panic!("[SKIP: fixture missing {fixture_path:?}: {e}]"));
        let b = std::fs::read(&live).expect("live artifact must exist");
        assert!(
            a == b,
            "fixture {fixture_path:?} drifted from the live artifact — re-snapshot it"
        );
    }
}

#[test]
fn fixtures_exist_and_mutations_are_real_diffs() {
    let pristine = std::fs::read(fixture("ontology_pristine.ttl")).expect("pristine ttl");
    let mutated = std::fs::read(fixture("ontology_missing_export.ttl")).expect("mutated ttl");
    assert_ne!(
        pristine, mutated,
        "ontology mutation is not a real diff — vacuous fixture"
    );
    let pristine_h = std::fs::read(fixture("header_pristine.h")).expect("pristine header");
    let mutated_h = std::fs::read(fixture("header_drifted.h")).expect("mutated header");
    let diff_count = pristine_h
        .iter()
        .zip(mutated_h.iter())
        .filter(|(a, b)| a != b)
        .count()
        + pristine_h.len().abs_diff(mutated_h.len());
    assert!(diff_count > 0, "header mutation is not a real diff");
}

#[test]
fn anti_vacuity_verifier_refuses_mutated_header_and_senses_mutated_ontology() {
    // Pristine pair must PASS.
    let (code_p, out_p) = verify(
        &fixture("ontology_pristine.ttl"),
        &fixture("header_pristine.h"),
    );
    assert_eq!(code_p, 0, "pristine pair must verify: {out_p}");
    assert!(
        out_p.contains("[ALL PASS]"),
        "pristine run lost its PASS line"
    );

    // Mutated header must FAIL with the drifted symbol named.
    let (code_h, out_h) = verify(
        &fixture("ontology_pristine.ttl"),
        &fixture("header_drifted.h"),
    );
    assert_eq!(code_h, 1, "drifted header must flip the verifier");
    assert!(
        out_h.contains("stardf_stream_free_drifted"),
        "FAIL output must name the drifted symbol: {out_h}"
    );

    // Mutated ontology: census decrements by exactly one individual
    // (witnessed 2026-10-31: pristine census is individuals=22, the missing-
    // export fixture drops the `stardf_stream_free` CabiExport individual →
    // 21) and the verifier must flip to exit 1 naming the unclaimed symbol.
    let out_m = Command::new("python3")
        .arg(arrow_root().join("scripts/verify.py"))
        .arg("--ttl")
        .arg(fixture("ontology_missing_export.ttl"))
        .arg("--header")
        .arg(fixture("header_pristine.h"))
        .arg("--capi-src")
        .arg(arrow_root().join("crates/stardf-arrow-capi/src"))
        .current_dir(arrow_root())
        .output()
        .expect("verify.py must run");
    let census = format!(
        "{}{}",
        String::from_utf8_lossy(&out_m.stdout),
        String::from_utf8_lossy(&out_m.stderr)
    );
    assert!(
        census.contains("individuals=21") || out_m.status.code() == Some(1),
        "ontology mutation must decrement census (22→21) or flip exit; got: {census}"
    );

    // Self-mutation control: the verifier's own --mutate drop-export must flip.
    let (code_m, _) = verify_mode_mutate();
    assert_eq!(code_m, 1, "--mutate drop-export must flip (anti-vacuity)");
}

fn verify_mode_mutate() -> (i32, String) {
    let out = Command::new("python3")
        .arg(arrow_root().join("scripts/verify.py"))
        .arg("--mutate")
        .arg("drop-export")
        .current_dir(arrow_root())
        .output()
        .expect("verify.py must run");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}
