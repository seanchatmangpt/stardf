# VENDOR-PROVENANCE — stardf-rdf-core

Lane w2-l01 (wave 2, repair lane), 2026-10-31. Copy-sweep task: full straight
copy of purrdf `crates/rdf-core` (the `purrdf-core` kernel, 190K src LOC) to
`stardf-rdf-core`, alongside the landed `stardf-rdf-core-slice`; both exist, no
conflation. No git state changed; no shared-seam edit was required.

## Source identity

- Upstream: PurRDF `crates/rdf-core` at rev
  `0da0578306983da851a9073337d57e2e2c41ab3a`.
- Transport: `git -C /Users/sac/purrdf archive <rev> crates/rdf-core` — the
  dirty working tree was never read.
- Upstream per-crate tar sha256 (vendoring-plan G4 form):
  `d00032fc429b56882098749cad1218cd8bf5de6b164a38dbbc45da2e3481ccd8`.
- Extracted 315 files / 10,750,128 bytes; per-file `cmp` against a second
  independent `git archive` extraction: **315/315 byte-identical pre-rename**.

## Landed layout (declared, disclosed)

- `stardf/arrow/crates/stardf-rdf-core/` — sibling of
  `stardf-rdf-core-slice` (slice intact, untouched). Not the vendoring plan's
  `stardf/crates/` location: that tree has no workspace root manifest and
  lacks stardf-branchless / stardf-deflate, so the task's cargo-check-green
  gate is unreachable there this lane; the arrow workspace carries the full
  dep closure and is today's one working stardf substrate workspace. The
  plan-named `stardf/crates/stardf-core/` directory exists with a single
  in-flight file (`src/blank_label.rs`, mtime ~100 min before this lane) — not
  touched, not conflated.
- Standalone workspace root (empty `[workspace]` table) — same posture as
  `stardf-rdf-core-slice`. NOT an arrow-workspace member: membership drags 13
  `[[test]]` + 15 `[[bench]]` targets into the shared test ladder and is a
  coordinator integration decision (slice PROVENANCE.md precedent).
- Lane build root lease: `stardf/arrow/target-w2l01` (coordinator deletes at
  integration, same as `target-w2l07`).

## Rename-scope delta (the only edits)

`Cargo.toml` is the single non-verbatim file (G3 sweep: 315 checked, 1 diff,
`./Cargo.toml`; 1 authored extra: `Cargo.lock` from the standalone resolve).
Manifest delta, each disclosed:

1. `[package] name = "stardf-rdf-core"` (task-named target; NOT the plan's
   `stardf-core` mechanical mapping — disclosed for arbitration).
2. Lib name `purrdf_core` VERBATIM (vendoring law).
3. First-party dep keys renamed to `stardf-*` with `{ path = "../stardf-*" }`
   RHS (branchless, lex, iri, cdt, xsd, events, hash, deflate; dev: stack,
   alloc-probe, testkit). All 11 resolve to arrow-workspace siblings.
4. Workspace-inherited scalars materialized: version `26.9.29` (arrow
   workspace posture, Lane-9 precedent), edition `2024`, rust-version `1.98`
   (upstream-equal), repository -> ggen-legacy; upstream `homepage` omitted;
   upstream `documentation = "https://docs.rs/purrdf-core"` kept verbatim.
5. External dep RHS materialized verbatim from the upstream root at the rev:
   hashbrown `0.17` default-features=false; sha2 `0.11`
   default-features=true; regex `1` (upstream feature list verbatim);
   regex-syntax `0.8.11`.
6. `[lints] workspace = true` -> upstream root `[workspace.lints]` inlined
   verbatim (stardf-lex/stardf-xsd precedent).
7. Authored root tables: empty `[workspace]`; `[profile.dev]` opt-level 3,
   debug 1, debug-assertions/overflow-checks true; `[profile.test]` debug 0;
   `[profile.dev.package."*"]` debug false; `[profile.dev.build-override]`
   opt-level 3 — upstream gate profile semantics from the upstream root at
   the rev, byte-equal in values. Without the gate profile the standalone
   test build compiles first-party code at dev opt-0, which is NOT the
   upstream test environment (see probe receipt below).

Zero non-rename edits to any vendored source file. **The vendoring-law
falsifier ("a non-rename edit needed to build") is NOT TRIGGERED.**

## Gates (commands this lane ran, real exits)

All with `CARGO_TARGET_DIR=/Users/sac/ggen-legacy/stardf/arrow/target-w2l01`
from the crate dir; toolchain rustc/cargo 1.98.0-nightly (91fe22da8 /
a595d0da2), arrow `rust-toolchain.toml` nightly floating.

- G2 cargo check: **exit 0**, zero warning/error lines. Cold run Finished in
  12.53s building the full leaf closure (stardf-hash/lex/iri/xsd/cdt/events/
  branchless/deflate checked green under this root too); re-run on final
  manifest (with gate profile) exit 0, `Finished dev [optimized + debuginfo]`.
- Extended `cargo check --all-targets`: **exit 0**, zero errors/warnings —
  all 13 [[test]] + 15 [[bench]] + examples targets compile incl. dev-deps
  (stardf-stack, stardf-testkit, stardf-alloc-probe).
- Bonus census `cargo test --lib`: **1414 passed / 4 failed** (45.33s), all
  four classified with receipts, none a stardf divergence:
  1. `distance::shape::tests::this_build_records_its_own_identity`
     (src/distance/shape.rs:873) — panics printing an upstream-CI compiler
     identity `rustc=1.101.0-nightly; commit=12433c30...; llvm=23.1.4` vs
     this host's 1.98.0-nightly: compiler-golden coupling, not a rename or
     copy divergence.
  2. `loss::tests::generated_artifact_has_not_drifted` (src/loss.rs:2632) and
  3. `loss::tests::transcode_matrix_has_not_drifted` (src/loss.rs:2650) —
     read `<arrow-root>/generated/rdf-loss-matrix.json` and
     `.../transcode-loss-matrix.json`: upstream workspace-root `generated/`
     artifacts outside the nine-crate sweep scope (plan step 1 archives
     crates/ only). Repo-layout coupling, campaign-known class.
  4. `sssom::tests::corpus_accessibility_parses_and_validates_clean`
     (src/sssom.rs:2246) — reads
     `crates/stardf-rdf-core/../../generated/mappings/gmeow-accessibility.sssom.tsv`:
     same workspace-root `generated/` coupling.
- Probe receipt (for every other copy-sweep lane): at dev opt-0 (no gate
  profile in the standalone root) the lib suite aborts with a **stack
  overflow SIGABRT after ~496-497 ok lines** (deterministic across two runs;
  NOT moved by `RUST_MIN_STACK=33554432`); with the upstream gate profile the
  abort disappears and the full 1418-test census runs. Attribution:
  opt-level codegen (frame/inlining shape), not stack size, not source.

## Falsifier status

NOT TRIGGERED. cargo check goes green on a byte-verbatim copy with
rename-only manifest deltas; no source file required any edit.

## Escalations (coordinator, not this lane)

1. The 3 `generated/`-coupled test failures need an arbitration decision
   (sweep the upstream workspace-root `generated/` artifacts, or adopt the
   parked-tests posture) — same class as the campaign's "algebra corpus
   coupling fix" line.
2. Workspace membership for stardf-rdf-core (and the slice) — integration
   decision; the crate is standalone-green today.
3. `stardf/crates/stardf-core/` (plan-name directory, one in-flight file,
   no manifest) and the missing stardf/crates root manifest + branchless/
   deflate are the remaining stardf/crates-tree gaps for whichever lane owns
   that tree.
4. QuadKey bridge edge: NOT-IN-LEASE. Upstream `QuadKey
   { key: [u32;4], mask: [u32;4] }` lands byte-verbatim here
   (crates/rdf-core/src/ir/dataset.rs:705-731 at the rev); the u32-sentinel
   adaptation belongs to the stardf-side pattern/probe-plan layer per
   `stardf/arrow/docs/DIVERGENCES.md` KD-03 (purrdf framing vs stardf
   BOTTOM=0/WILDCARD=u32::MAX inverse framing, "probe-plan kernel is a later
   wave").

## Nonclaims

- UNVERIFIED: upstream differential for the 4 lib-test failures (would need
  the upstream workspace test run at the pinned rev on this host; the
  stardf-xsd lane's method). The classification above is failure-mode + path
  evidence, not an upstream-parity witness.
- UNVERIFIED: integration-test targets (13 [[test]]) and benches were
  compile-checked only (`--all-targets`), not executed.
- UNKNOWN: the identity of the opt-0 stack-overflowing test (the harness
  died before naming it; not investigated further — the gate profile is the
  upstream posture, not a workaround).
- NOT claimed: behavioral equivalence beyond the census above; no mock, no
  golden asserted that this lane did not run.
