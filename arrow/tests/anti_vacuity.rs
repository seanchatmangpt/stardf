//! Anti-vacuity courts for the stardf-arrow substrate verifier.
//!
//! Chicago-style: real `python3 scripts/verify.py` subprocesses over real
//! fixture files under `tests/fixtures/`. No mocks. A court that cannot flip
//! (verifier accepts a corrupted input) fails the test; a court that cannot
//! run (python3/rdflib missing) skips LOUDLY with a nonzero exit.
//!
//! Fixture corpus (pristine fixtures are byte copies of stardf/arrow's OWN
//! artifacts — `ontology.ttl` and
//! `crates/stardf-arrow-capi/include/stardf_arrow.h` — asserted byte-equal in
//! `fixtures_are_byte_copies_of_own_artifacts` below):
//! - `ontology_pristine.ttl` / `header_pristine.h` — verifier must accept
//!   (exit 0, `[ALL PASS]`).
//! - `ontology_missing_export.ttl` — one export individual
//!   (`stardf-arrow:export-stream-free`) removed; the verifier's ontology
//!   census observable must shift by exactly one individual (see court 2 for
//!   the verifier-coverage note).
//! - `header_drifted.h` — one declaration (`stardf_stream_free`) renamed;
//!   header/impl parity must refuse (exit 1).

use std::path::PathBuf;
use std::process::Command;

const SCRIPT: &str = "scripts/verify.py";
const FIXTURE_NAMES: &[&str] = &[
    "ontology_pristine.ttl",
    "header_pristine.h",
    "ontology_missing_export.ttl",
    "header_drifted.h",
];

fn arrow_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture(name: &str) -> PathBuf {
    arrow_root()
        .join("tests/fixtures")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|e| panic!("fixture missing: {name}: {e}"))
}

/// Real subprocess python3 verify.py run. Returns (exit_code, combined_output).
fn run_verifier(extra_args: &[&str]) -> (Option<i32>, String) {
    let out = Command::new("python3")
        .arg(arrow_root().join(SCRIPT))
        .args(extra_args)
        .current_dir(arrow_root())
        .output()
        .expect("failed to spawn python3 — is python3 on PATH?");
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code(), text)
}

fn deps_available() -> Result<(), String> {
    let out = Command::new("python3")
        .arg("-c")
        .arg("import rdflib") // scripts/verify.py's runtime dependency
        .output()
        .map_err(|e| format!("python3 not spawnable: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Extract the individuals count from a verifier census line
/// (`[PASS] ontology parse: N triples; classes=C individuals=I`).
fn individuals_from(output: &str) -> Option<u32> {
    let line = output.lines().find(|l| l.contains("ontology parse"))?;
    let idx = line.find("individuals=")?;
    line[idx + "individuals=".len()..]
        .split(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}

#[test]
fn anti_vacuity_verifier_refuses_mutated_header_and_senses_mutated_ontology() {
    // LOUD skip: no silent pass when the verifier cannot run at all.
    if let Err(reason) = deps_available() {
        println!("[SKIP: python3 rdflib unavailable: {reason}]");
        panic!("anti-vacuity court skipped — must never pass silently");
    }
    for name in FIXTURE_NAMES {
        let p = fixture(name);
        assert!(p.is_file(), "fixture not a file: {}", p.display());
    }

    // Court 1 baseline: pristine pair is accepted (exit 0 + [ALL PASS]).
    let (code, out) = run_verifier(&[
        "--ttl",
        &fixture("ontology_pristine.ttl").to_string_lossy(),
        "--header",
        &fixture("header_pristine.h").to_string_lossy(),
    ]);
    assert_eq!(
        code,
        Some(0),
        "pristine ontology+header must verify clean, got {code:?}:\n{out}"
    );
    assert!(
        out.contains("[ALL PASS]"),
        "pristine run lost its ALL PASS banner:\n{out}"
    );

    // Court 1 flip: a renamed header declaration must be REFUSED (exit 1).
    let (code, out) = run_verifier(&[
        "--ttl",
        &fixture("ontology_pristine.ttl").to_string_lossy(),
        "--header",
        &fixture("header_drifted.h").to_string_lossy(),
    ]);
    assert_eq!(
        code,
        Some(1),
        "vacuous court: verifier ACCEPTED a drifted header:\n{out}"
    );
    assert!(
        out.contains("[FAIL]"),
        "refusal must carry a [FAIL] diagnostic:\n{out}"
    );
    assert!(
        out.contains("stardf_stream_free_drifted"),
        "refusal must name the drifted symbol:\n{out}"
    );

    // Court 2: ontology with one export individual removed. Since the
    // layer-census coverage gate landed (verify.py now cross-checks
    // ontology targetSymbols against header decls and impl exports), the
    // exit code MUST flip to 1: the header's stardf_stream_free declaration
    // becomes unclaimed by the mutated ontology. The census observable is
    // still asserted below — the flip and the census delta are independent
    // proofs that the verifier genuinely ingested the mutated fixture.
    let (code, out_pristine) = run_verifier(&[
        "--ttl",
        &fixture("ontology_pristine.ttl").to_string_lossy(),
        "--header",
        &fixture("header_pristine.h").to_string_lossy(),
    ]);
    assert_eq!(code, Some(0), "pristine baseline changed:\n{out_pristine}");
    let (code_m, out_m) = run_verifier(&[
        "--ttl",
        &fixture("ontology_missing_export.ttl").to_string_lossy(),
        "--header",
        &fixture("header_pristine.h").to_string_lossy(),
    ]);
    assert_eq!(
        code_m,
        Some(1),
        "vacuous court: verifier ACCEPTED an ontology missing an exported symbol:\n{out_m}"
    );
    assert!(
        out_m.contains("[FAIL]") && out_m.contains("stardf_stream_free"),
        "refusal must name the unclaimed symbol:\n{out_m}"
    );
    let n_pristine =
        individuals_from(&out_pristine).expect("pristine output lacks individuals census line");
    let n_mut = individuals_from(&out_m).expect("mutated output lacks individuals census line");
    assert_eq!(
        n_pristine - n_mut,
        1,
        "removing one export individual must decrement the verifier census by exactly 1 (pristine={n_pristine}, mutated={n_mut})"
    );

    // Court 3: the verifier's own in-memory anti-vacuity (drop-export) must
    // flip: parity must FAIL after dropping one impl export.
    let (code, out) = run_verifier(&[
        "--ttl",
        &fixture("ontology_pristine.ttl").to_string_lossy(),
        "--header",
        &fixture("header_pristine.h").to_string_lossy(),
        "--mutate",
        "drop-export",
    ]);
    assert_eq!(
        code,
        Some(1),
        "vacuous gate: verify.py --mutate drop-export did NOT fail the parity check:\n{out}"
    );
    assert!(
        out.contains("[MUTATE]") && out.contains("anti-vacuity"),
        "self-mutate run lost its mutation/anti-vacuity banners:\n{out}"
    );
}

#[test]
fn fixtures_exist_and_mutations_are_real_diffs() {
    // Structural non-vacuity: each mutated fixture must actually differ from
    // its pristine copy, and the drift must be exactly the named mutation.
    let pristine_ttl = std::fs::read_to_string(fixture("ontology_pristine.ttl")).unwrap();
    let mutated_ttl = std::fs::read_to_string(fixture("ontology_missing_export.ttl")).unwrap();
    assert_ne!(pristine_ttl, mutated_ttl, "ontology mutation is a no-op");
    assert!(
        !mutated_ttl.contains("export-stream-free"),
        "mutated ontology still names export-stream-free"
    );
    assert!(
        pristine_ttl.contains("export-stream-free"),
        "pristine ontology lost export-stream-free — fixture refresh needed"
    );

    let pristine_h = std::fs::read_to_string(fixture("header_pristine.h")).unwrap();
    let drifted_h = std::fs::read_to_string(fixture("header_drifted.h")).unwrap();
    assert_ne!(pristine_h, drifted_h, "header drift is a no-op");
    let repaired: String = drifted_h.replace("stardf_stream_free_drifted", "stardf_stream_free");
    assert_eq!(
        pristine_h, repaired,
        "header drift must be exactly the stardf_stream_free rename"
    );
}

#[test]
fn fixtures_are_byte_copies_of_own_artifacts() {
    // Grounding: pristine fixtures must be exact byte copies of
    // stardf/arrow's OWN artifacts (not the predecessor authority pair), so
    // the courts exercise the real substrate surface. A refresh of
    // ontology.ttl / stardf_arrow.h upstream turns this red on purpose.
    for (fixture_name, artifact_rel) in [
        ("ontology_pristine.ttl", "ontology.ttl"),
        (
            "header_pristine.h",
            "crates/stardf-arrow-capi/include/stardf_arrow.h",
        ),
    ] {
        let fx = std::fs::read(fixture(fixture_name))
            .unwrap_or_else(|e| panic!("fixture unreadable: {fixture_name}: {e}"));
        let art = std::fs::read(arrow_root().join(artifact_rel))
            .unwrap_or_else(|e| panic!("artifact unreadable: {artifact_rel}: {e}"));
        assert_eq!(
            fx, art,
            "fixture {fixture_name} drifted from its own artifact {artifact_rel} — refresh fixtures"
        );
    }
}
