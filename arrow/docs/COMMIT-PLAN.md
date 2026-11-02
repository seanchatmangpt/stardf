# Commit Plan — GL-STARDF-001 (6-commit split, Lane-14 final issue)

Lane 14 final issue (2026-10-31 17:54 PDT). Supersedes the Lane 13 issue.
NO git mutations performed by this lane — staged plan for coordinator
execution. Parent HEAD `e06c257`, branch `fix/v26.9.29-ci-gates`.

SERIES STATE CORRECTION vs the Lane 13 issue: c1 and c2 of the original
six-commit split have ALREADY LANDED — `26d0140` (c1 planning pack) and
`98afd71` (c2 authority contract + verifiers). The remaining series is
c1r/c2r (residue), c3, c4, c5, c6 below. All gate claims against c2's
content were re-probed live against the COMMITTED state, not the working
tree.

Process violation registered this wave: `e06c257` is a gitignore-only
commit that landed c3's planned `.gitignore` delta without a ticket. It
stays in history; consequence: c3 no longer carries a gitignore-delta step
(the rules are committed at root and ship inside `stardf/arrow/.gitignore`
with the c3 tree).

Census method: `git status --porcelain=v1 --untracked-files=all` (read-only),
snapshot 2026-10-31 17:54 PDT. The tree was still growing while read
(arrow grew 12,098 -> 12,133 files during this session); c6 absorbs drift.

Gate status (probed live this session):

| gate | probe | result |
|---|---|---|
| c2 F1 falsifier | `python3 scripts/verify_stardf_ontology.py --ttl authority/stardf/stardf_contract.ttl --header lib_rs.tmpl` | exit 1 — HOLDS. 22 Purrdf* types unmapped, 128 AbiFunction instances, 4995 triples. Expected pre-census FAIL rung; exit 0 would mean the falsifier is dead. |
| c2 packs verifier | `python3 scripts/verify_marketplace_packs.py` | exit 0 — HOLDS. All 15 marketplace packs verified. |
| c3 check gate | `cd stardf/arrow && cargo check --workspace --locked --offline` | **RED — exit 101** (re-witnessed 2026-10-31, 58-member workspace). `stardf-rdf-core-slice` fails to compile: 17 errors, all E0432/E0433 — unresolved `purrdf_lex::{terminals, diagnostic, json}` (the vendored stardf-lex never shipped those modules) and missing own modules `crate::{dataset, dataset_view, turtle}`. Lane 13's exit-0 witness is confirmed STALE (3-member tree). |
| c3 test gate | `cd stardf/arrow && cargo test --workspace --locked --offline` | **NOT RUN** — chained after check; check failed so test never started. |

## c0 (recommended pre-split) — historical docs/jira packs

`docs/jira/` (9 files: `v26.9.17/cleanup-merge-plan.md`,
`v26.9.19/000-survey.md` … `007-dirty-tree.md`) pre-date the stardf wave
and are not phase-2 output. Land first as
`docs(jira): historical survey/PR packs` so c1 stays wave-scoped.

## c1r — planning pack residue (c1 already landed as 26d0140)

Commit `26d0140` ("docs(stardf): planning pack v26.10.31 + GL-STARDF-001
(c1)") is IN HISTORY: the 12 original planning packs and the ticket are
tracked. Only the post-c1 additions remain untracked:

- **git add** (3 files): `stardf/docs/jira/v26.10.31/` —
  12-full-parity-campaign.md, 13-v26111-target-ledger.md,
  14-definition-of-done.md.
- **Message file text**:
  ```
  docs(stardf): planning pack additions (full-parity campaign, v26.11.1 target ledger, definition of done)
  ```
- **GATE (exit 0 required)**: `python3 scripts/verify_docs.py --strict`
  (closest standing repo docs gate; no planning-markdown verifier exists).

## c2r — authority + tooling residue (c2 already landed as 98afd71)

Commit `98afd71` ("feat(stardf): authority contract + verifiers, census
parity 22/22 (c2)") is IN HISTORY: all 8 authority/stardf files, the
graphlaw contract, the extract/verify scripts, and lib_rs.tmpl are
tracked. Remaining residue:

- **git add** (5 files):
  - `authority/stardf/license-compat.json` (TRACKED, MODIFIED)
  - `authority/stardf/license-inventory.toml` (TRACKED, MODIFIED)
  - `scripts/bulk_copy_purrdf.py` (NEW — slice copy tooling)
  - `scripts/workflows/stardf-10x20.workflow.mjs` (NEW — wave dispatch)
  - `stardf/workflows/v26.11.1-full-parity.workflow.js` (NEW — wave dispatch)
- **Message file text**:
  ```
  chore(stardf): authority license refresh + slice copy/wave dispatch tooling

  License compat/inventory updates from the vendoring sweep, plus the
  bulk copy script and the 10x20 / full-parity wave dispatch workflows.
  F1 falsifier still exits 1 against the committed contract (re-witnessed
  2026-10-31) — the census FAIL rung stays replayable.
  ```
- **GATE (exit 0 required)**: `python3 scripts/verify_marketplace_packs.py`
  PLUS the F1 falsifier rung — the series stays lawful only in the state
  where
  `python3 scripts/verify_stardf_ontology.py --ttl authority/stardf/stardf_contract.ttl --header lib_rs.tmpl`
  exits 1. If it exits 0, the falsifier is dead (D1 Option B rejection).
  Both re-witnessed 2026-10-31 against the committed c2 state.

## c3 — stardf-arrow workspace (one-package port)

- **Scope**: full `stardf/arrow/` tree MINUS `evidence/` (c4) MINUS `docs/`
  (c5). Census 2026-10-31 17:54 PDT: **12,108 files**.
- **Composition**:
  - `crates/` — 58 crate dirs: 57 `stardf-*` + vendored `jsonschema`
    (incl. tests/metaschemas tree — the bulk of the 12,019+ crate files).
    Workspace root `Cargo.toml` declares all members (core, capi, bench
    promoted lane 7, columnar lane 9, then the vendoring-lane slices:
    hash, lex, deflate, branchless, iri, … through sparql/shex/n3/geo/
    duckdb families). `Cargo.lock` at workspace root.
  - `corpora/n3/` — 31 files, N3 conformance corpus (NEW since Lane 13).
  - `tests/` — 5: anti_vacuity.rs + 4 fixtures (header_drifted.h,
    header_pristine.h, ontology_missing_export.ttl, ontology_pristine.ttl).
  - `scripts/` — 8 (verify.py, verify_blake3_identity.sh,
    verify_duckdb_rehome.sh, verify_stardf_licenses.py,
    check-columnar-oracle.sh, branchless_manifest.toml,
    columnar_manifest.sha256, n3_corpus_manifest.sha256).
  - `queries/rust/` — 2 (010_contract_to_rust_items.rq, run_bridge.py).
  - 1 each: `tickets/GL-STARDF-002-freeze-expansion.md`,
    `templates/.gitkeep`, `examples/columnar_oracle_fixture.rs`,
    `ci/stardf-arrow-reconstitution.yml`,
    `authority/stardf/stardf_contract.ttl` (vendored contract copy).
  - root 9: .gitignore, Cargo.toml, Cargo.lock, README.md, ggen.toml,
    golden_bytes.rs, justfile, ontology.ttl, rust-toolchain.toml.
    (README/justfile/rust-toolchain moved HERE from c5 — c3 is now the
    full subtree minus evidence/docs, per coordinator directive.)
- **Preconditions before staging (coordinator, file operations not git)**:
  1. DELETE the 6 lane build-root leases `stardf/arrow/target-w2l{01,07,10,13,14,19}/`
     (12 untracked cache files). Same-checkout-fanout cleanup law: lane
     build roots are leases, deleted at integration, never committed.
  2. Bench lock decision: `crates/stardf-arrow-bench/Cargo.lock` still
     exists while the workspace Cargo.toml comment declares "One workspace,
     one lock". Recommend deleting the stale member lock. D1's older
     "bench keeps its own" is superseded by the promotion comment.
  3. RESOLVED (was a Lane 13 MUST-FIX): `stardf/arrow/stardf/` and
     `stardf/stardf/` no longer exist on disk.
  4. RESOLVED (was a Lane 13 MUST-FIX): ignore rules cover
     `crates/*/target/` (stardf/arrow/.gitignore), `.ggen/`, `.ggen-v2/`,
     `rust_out`, `examples/columnar_oracle_fixture`. `git check-ignore`
     witnessed exit 0 for `.ggen/keys/signing.key`, `.ggen-v2/receipt.json`,
     `crates/stardf-arrow-bench/target` — the signing key canNOT be swept
     in by `git add stardf/arrow`. Root `.gitignore` (`e06c257`) double-
     covers with `**/target/`, `**/rust_out`, `**/.ggen/`, `**/.ggen-v2/`.
     No staging-time ignore work remains.
- **Message file text**:
  ```
  feat(stardf): stardf-arrow one-package workspace (58 crates, corpora, ci draft, ontology, queries, tests)

  Full port: 57 stardf-* crates + vendored jsonschema under one workspace
  (core, capi, promoted bench, columnar, and the vendoring-lane slices).
  N3 conformance corpora, anti-vacuity tests + fixtures, ggen manifest,
  vendored authority contract copy, ci draft. Lands after the c2 verifiers
  so the census polices an already-committed header surface.
  ```
- **GATE (exit 0 required, both)**:
  ```
  cd stardf/arrow && cargo check --workspace --locked --offline
  cd stardf/arrow && cargo test --workspace --locked --offline
  ```
  Witness status: **RED** — check re-witnessed exit 101 this session
  (2026-10-31, 58-member workspace, `--locked --offline`):
  `crates/stardf-rdf-core-slice` has 17 compile errors (E0432/E0433) —
  it imports `purrdf_lex::terminals`, `purrdf_lex::diagnostic`,
  `purrdf_lex::json`, none of which exist in the vendored stardf-lex, and
  own modules `dataset`, `dataset_view`, `turtle` are absent from the
  crate root. This is an incomplete vendoring (consumer landed, modules
  not), not a gate artifact. c3 is **BLOCKED** until the owning lane
  repairs stardf-rdf-core-slice (ship the missing lex modules / drop the
  crate from the workspace) and the full check+test ladder goes green.
  Lane 13's 3-member exit-0 witness is confirmed stale. Test gate NOT RUN
  (skipped when check failed). No check-only landing.

## c4 — evidence receipts

- **git add** (9 files): `stardf/arrow/evidence/` — README.md,
  RECEIPT_TEMPLATE.json, columnar-wave-2026-10-31.json,
  finish-wave-2026-10-31.json (NEW since Lane 13),
  manufacturing-receipt-2026-10-31.json, replay-2026-10-31.json,
  w1-closure-2026-10-31.json, wave-consolidated-2026-10-31.json,
  wave-finish-lane16-2026-10-31.json.
- **Message file text**:
  ```
  evidence(stardf): lane receipts + evidence conventions

  Historical pre-commit receipts: subject.head cites parent c9f8028 with
  stardf/ untracked (tree_digest only, SHA-less by construction — written
  before 26d0140/98afd71/e06c257 landed). After c3 lands, a fresh receipt
  re-binds to the new SHA per 05-next-steps §1 and is owed before the wave
  closes.
  ```
- **GATE (exit 0 required, ad hoc — flagged)**:
  ```
  python3 -c "import json,glob,sys; [json.load(open(p)) for p in glob.glob('stardf/arrow/evidence/*.json')]; print('ok')"
  ```
  (All receipts parse as valid JSON. No standing evidence verifier exists;
  this well-formedness court is a substitute — flagged as such.)

## c5 — arrow slice docs + decision briefs

- **git add** (17 files):
  - `stardf/arrow/docs/` — 16: ABI-FREEZE.md, ARD.md, BENCH_BASELINE.json,
    COLUMNAR-ARD.md, DIVERGENCES.md, OPERATOR-DECISIONS.md,
    PRD.md, RECEIPTS.md, COMMIT-PLAN.md (this file), SCAN-COMPLETION.md
    (NEW), avec-rehearsal.md (NEW), DECISION-BRIEFS/{D10-blake3-flip.md,
    D12-D13-dispatch.md, R1-blake3-receipt-admission.md} (NEW),
    archive/c3-dual-site/slice11-umbrella/{Cargo.toml,lib.rs} (NEW).
  - `docs/DECISION-BRIEFS/D9-freeze-ratification.md` (repo root, NEW).
  - README.md / justfile / rust-toolchain.toml NO LONGER HERE — moved to c3
    (c3 is now the full subtree minus evidence/docs).
- **Message file text**:
  ```
  docs(stardf): arrow slice docs + decision briefs (ABI-FREEZE, DIVERGENCES, ARD/PRD, OPERATOR-DECISIONS, RECEIPTS, COMMIT-PLAN, D9-D13 briefs)

  Includes the commit plan this series executes, the D9-D13 decision
  briefs, and the archived c3-dual-site slice-11 umbrella alternative.
  BENCH_BASELINE.json is self-labeled UNVERIFIED; the CI-qualified rerun
  is the closing court.
  ```
- **GATE (exit 0 required)**: `python3 scripts/verify_docs.py --strict`

## c6 — vendored slice-11 port crates + sweep

- **Explicit content (not residue — enumerated)**: `stardf/crates/` —
  `stardf/` (3 files: umbrella facade) + `stardf-rdf/` (1,476 files) =
  1,479 files, vendored from PurRDF rev 0da05783 (slice 11 of the
  v26.11.1 copy sweep). These are OUTSIDE the arrow workspace.
- **GATE: BLOCKED (flag 3 below).** Both manifests use
  `version.workspace = true` (workspace-inherited keys) with NO ancestor
  `[workspace]` manifest (`stardf/Cargo.toml` does not exist) — `cargo
  check` cannot resolve them standalone. The commit gate for these crates
  is undecided until the coordinator picks: (a) add a `stardf/Cargo.toml`
  workspace root, or (b) inline the inherited keys. Until then this commit
  is UNLAWFUL on any compile gate; land it only with an explicit
  documented waiver or after the workspace-root decision.
- **Final sweep**: re-run `git status --porcelain` immediately before
  staging; stage anything left as `chore(stardf): phase-2 sweep`. Expected
  residue after c0-c5 + the enumerated crates commit: empty.
- **GATE (series-complete court)**: `test -z "$(git status --porcelain)"` —
  the tree must be clean after c6 for the series to be complete.

## Flags

1. **c3 is BLOCKED on `stardf-rdf-core-slice`** — check gate re-witnessed
   RED (exit 101, 17 compile errors, E0432/E0433): the crate imports
   `purrdf_lex::{terminals, diagnostic, json}` and
   `crate::{dataset, dataset_view, turtle}`, none shipped. Coordinator
   requirement stands: check + test both green, no check-only landing;
   the Lane 13 red-court registration option is withdrawn. Until repair,
   c3 cannot land at all (it fails even the check gate it previously
   hoped to land on).
2. **Bench member lock contradiction** —
   `crates/stardf-arrow-bench/Cargo.lock` exists; workspace Cargo.toml
   says "One workspace, one lock". Recommend deleting the stale lock in
   the c3 preconditions.
3. **`stardf/crates/` orphan-workspace manifests** — `version.workspace =
   true` with no ancestor `[workspace]`; compile gate BLOCKED pending
   workspace-root decision (or key inlining). 1,479 files cannot land
   lawfully on a compile gate until resolved.
4. **c4 receipt re-binding** — all 9 evidence receipts are SHA-less
   (tree_digest over the pre-commit tree); a fresh post-commit receipt
   re-binding to the new HEAD is owed after the series lands
   (05-next-steps §1).
5. **c4 gate is ad hoc** — JSON parse court substitutes for a standing
   evidence verifier; flagged, not hidden.
6. **docs/jira provenance** — 9 historical files; recommended c0 split.
7. **Live-tree drift** — census is a 17:54 PDT snapshot; the arrow tree
   grew during this lane's read (12,098 -> 12,133). All counts are
   as-of; the c6 sweep plus a stage-time `git status --porcelain` re-run
   absorb drift.

## Receipt

- Subject: `/Users/sac/ggen-legacy` @ e06c257 + working tree, branch
  `fix/v26.9.29-ci-gates`.
- Diff: rewrite of `stardf/arrow/docs/COMMIT-PLAN.md` (this file) only.
- Commands/exits (this session): F1 falsifier exit 1 (expected pre-census
  FAIL rung, 22/128/4995); verify_marketplace_packs exit 0 (15 packs);
  `git check-ignore` exit 0 (signing.key/.ggen-v2/bench-target all
  ignored); `cargo check --workspace --locked --offline` **exit 101** —
  stardf-rdf-core-slice 17× E0432/E0433 (missing purrdf_lex modules +
  missing own modules); `cargo test` NOT RUN (skipped after check fail).
- No git mutations performed (read-only `git status --porcelain` only;
  commit e06c257 was made by another lane and is only READ here).
- Standing: **c3 BLOCKED** (check RED on stardf-rdf-core-slice, flag 1);
  c1r/c2r/c4/c5 gates witnessed; `stardf/crates/` gate BLOCKED (flag 3);
  c4 receipt re-bind OPEN; docs/jira c0 split RECOMMENDED. Overall plan
  standing: PARTIAL_ALIVE — the c3 hop cannot execute until the owning
  lane repairs the crate.

---

# Post-Mortem — v26.11.1 commit series (G19 finish lane, 2026-11-01)

Append-only post-mortem written after the series executed. This lane ran
read-only git only (`git log`/`git status`/`git show`); NO git mutations,
staging, or commits. HEAD at read time: `724d542540b497f2928a63b5aece9b37aef4019b`
(c4r), branch `fix/v26.9.29-ci-gates`.

## Landed chain (verified by this lane's `git log --oneline -14`, exit 0)

| plan ref | landed SHA | subject (abbrev.; full text in git log) |
|---|---|---|
| c1 | `26d0140` | docs(stardf): planning pack v26.10.31 + GL-STARDF-001 (c1) |
| c2 | `98afd71` | feat(stardf): authority contract + verifiers, census parity 22/22 (c2) |
| — | `e06c257` | chore: update .gitignore — **VIOLATION, see below** |
| c3 | `6cbb5b7` | feat(stardf): one-package port — purrdf -> stardf, 53 crates (c3) |
| c4 | `de5ca40` | chore(stardf): evidence receipts through the v26.11.1 campaign (c4) |
| c5 | `b87ea6f` | docs(stardf): as-built doc pack + decision briefs (c5) |
| c2r | `4d354af` | chore(stardf): license register closure + tooling (c2r) |
| c4r | `724d5425` | chore(stardf): final manufacturing receipt (c4r) |

## Plan-vs-landed deltas

- **c0 never landed.** The 9 historical `docs/jira/` files
  (`v26.9.17/cleanup-merge-plan.md`, `v26.9.19/000-survey.md` …
  `007-dirty-tree.md`) are still untracked today (read this lane).
- **c1r absorbed into c5.** The three planning-pack additions (12-full-parity,
  13-v26111-target-ledger, 14-definition-of-done) show in `b87ea6f`'s stat;
  c1r never landed as a distinct commit.
- **c5 landed wider than drafted.** The draft scoped c5 to 17 files; the
  landed c5 (`b87ea6f`) carried the as-built doc pack + decision briefs +
  planning-pack updates (incl. COMMIT-PLAN.md and OPERATOR-DECISIONS.md
  themselves).
- **c6's enumerated content is MOOT.** `stardf/crates/` is ABSENT on disk
  (verified this lane) — the one-package port (D15/D16) archived the stubs,
  so the 1,479-file commit cannot be staged. Its blocking question
  (orphan `version.workspace = true` manifests) died with the stubs.

## Resolved flags

- **Flag 1 (c3 BLOCKED on stardf-rdf-core-slice): RESOLVED.** c3 landed
  (`6cbb5b7`) and the latest publish receipt (T040521Z) witnesses
  G9 `cargo check --workspace` exit 0 at 21:05 — the compile gate c3's
  landing was conditioned on.
- **Flag 3 (stardf/crates orphan-workspace manifests): RESOLVED by
  obsolescence** (stubs archived, dir absent — verified this lane).
- **Flag 4 (receipt re-bind): RESOLVED.**
  `evidence/v26.11.1-manufacturing-receipt.json` binds `subject.head` to
  `724d5425...` (c4r itself), with full c-chain identity — the SHA-binding
  debt from the SHA-less pre-commit receipts is closed.
- **Flag 5 (ad hoc c4 JSON court): stands** — the ad-hoc parse court was
  used as flagged; a standing evidence verifier still does not exist.
- **Flag 6 (docs/jira c0 split): still OPEN** (9 files untracked).
- **Flag 7 (live-tree drift): absorbed** — c4r's receipt carries the
  receipt-time tree digest; drift continues into the residue below.
- **Flag 2 (bench member lock): still OPEN** —
  `crates/stardf-arrow-bench/Cargo.lock` still exists on disk (verified
  this lane); workspace Cargo.toml still says "One workspace, one lock".

## Remaining — c5r/c6 finish-campaign residue (uncommitted, as of this read)

The finishers-wave work is on disk, NOT in history. Snapshot this lane
(`git status --porcelain=v1 --untracked-files=all`, 1,447 dirty paths):
**139 untracked / 1,285 modified / 23 deleted.**

- **c5r (docs residue)**: 9 historical docs/jira files (c0, above); 4 new
  docs (`docs/SUPERSEDED-CLAIMS.md`, `docs/V26-11-1-VERDICT.md`,
  `docs/design/purrdf-change-path-allocations.md`,
  untracked `docs/design/purrdf-retrieval-ladder.md`); plus the modified
  `docs/OPERATOR-DECISIONS.md` / `COMMIT-PLAN.md` appends THIS lane is
  writing.
- **c6 (sweep)**: 1,285 modified files — the campaign source sweep across
  crates (src/tests/benches: edition-2024/MSRV/metadata unification, dual-key
  manifest work, scan-crate growth, court repairs); 130 untracked under
  `stardf/`: 11 new evidence receipts (incl. the 7 publish receipts), ~60
  license-carriage files (licenses/ x5 per crate, NOTICE.txt), 7 PROVENANCE.md,
  corpora/community (2), vectors/ shacl12+shacl test vectors (~40), 1
  workflows file, 2 disabled-tests READMEs; 23 deletions — 6 lane
  build-root leases (`target-w2l{01,07,10,13,14,19}/` + `target-l3sql/`
  cache files, per the same-checkout-fanout cleanup law), the
  `v26.11.1-manufacturing-receipt-DRAFT.json`, `golden_bytes.rs`, and 7
  removed tests/examples (stardf-rdf gts_composite_compaction, stardf-shapes
  4 oracle fixtures, stardf-shex 2 tests).
- The split above is descriptive, not prescriptive: the coordinator owns the
  final partition at staging time (re-run `git status --porcelain` then).

## The registered e06c257 process violation

`e06c257` (gitignore-only commit landing c3's planned .gitignore delta
without a ticket) **stays in history**; the header registration above stands.
Consequence recorded at registration time still holds: c3 carried no
gitignore-delta step (the rules ship inside `stardf/arrow/.gitignore` with
the c3 tree). Closure is unavailable — history is immutable; the standing
guard is the ticket-gated admission law (nothing executable admitted without
a deterministic ticket). Status: **REGISTERED, not remediated.**

## Post-mortem standing

**PARTIAL_ALIVE.** The chain landed through c4r (8 commits verified in
history this session); G9 `cargo check --workspace` exit 0 witnessed
(21:05, T040521Z receipt); the finish-campaign residue (c5r/c6) is
uncommitted on the live tree; flag 2 + flag 6 open; the e06c257 violation is
REGISTERED, not remediated. The series is not "complete" until the residue
lands and a sweep commit reads a clean tree
(`test -z "$(git status --porcelain)"` — the original c6 gate).
