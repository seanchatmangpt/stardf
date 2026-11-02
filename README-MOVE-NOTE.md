# README-MOVE-NOTE — stardf relocation package (Lane H)

Drafted 2026-10-31 ~22:50 by Lane H (authority lane, `/Users/sac/ggen-legacy/stardf`).
Read-only sweep plus this one file. This lane ran no git; every git step below is
coordinator-executed. Numbers are as-of the disk read at drafting time — concurrent
lanes were landing files during the sweep (`stardf/.claude/`, `stardf/tickets/`,
`stardf/AGENTS.md` appeared at 22:38–22:39), so the T0 re-verify gate (§6) re-runs the
sweep at move time.

## Verdict

**SELF-CONTAINED WITH NAMED FIXES.** The `stardf/` tree moves clean as one directory
after the pre-move fixes P1–P8 (§2). Swept 12,297 text files; 16 raw `../` escapes,
collapsing to 10 live file surfaces (§1a) + 1 gitignored artifact (`arrow/rust_out/`) +
1 synthetic bench string. The one LIVE code dependency on the parent is the license gate
(`verify_stardf_licenses.py` + `build_receipt.py` read the parent
`authority/stardf/` register) — closed by P1+P5. `~/stardf` is free, and `/Users/sac`,
`/Users/sac/ggen-legacy`, `/Users/sac/ggen-legacy/stardf` all sit on device 16777231
(same APFS volume) — the 75 GB / ~263k-file move is an instant rename(2), not a copy.

## 1. Self-containment sweep — method and fix list

Method: a python resolver walked every text file under `stardf/` (excluding `target*`,
`node_modules`, `.git`, `.venv`, `*.lock`, binaries, files >2 MB), resolved every
`(../)+` chain against its file's directory, and tested for escape. Plus literal greps
for `../CLAUDE.md`, `../AGENTS.md`, `../tickets/`, `ggen-legacy`,
`/Users/sac/ggen-legacy`, and `path = "../../../"` in Cargo manifests (zero hits).

### 1a. Escapes needing fixes (live surfaces)

| # | File:line | Reference | Post-move failure mode | Fix |
|---|---|---|---|---|
| 1 | `arrow/docs/avec-rehearsal.md:3` | `../../../tickets/GL-STARDF-001.md` | link escapes to parent `tickets/` | P2 |
| 1b | `arrow/docs/avec-rehearsal.md` | `../../../ggen-legacy/stardf/arrow/crates/stardf-arrow-capi` | already malformed today (resolves to `ggen-legacy/ggen-legacy/...`) | P2 |
| 2 | `tickets/README.md` | `[t1]: ../../tickets/GL-STARDF-001.md`, `../../AGENTS.md` (x2 each, 4 refs) | link + authority citations escape | P2 |
| 2b | `AGENTS.md` (top-level, landed 22:39) | `../AGENTS.md`, `../tickets/GL-STARDF-001.md`, `../authority/stardf/stardf_contract.ttl` | authority chain escapes | P1+P2+P7 |
| 2c | `arrow/scripts/h03_bootstrap_sim.sh:9` | `cd /Users/sac/ggen-legacy/stardf/arrow` | live script cd's into a dead path | P4 |
| 2d | `workflows/v26.11.1-full-parity.workflow.js:24,56` | `REPO = '/Users/sac/ggen-legacy'` | campaign workflow pins dead repo root | P4 |
| 3 | `arrow/scripts/verify_stardf_licenses.py:20-21,88` | `REPO = parents[3]` reads parent `authority/stardf/license-compat.json` + `license-inventory.toml` | R1 license gate input vanishes (parents[3] becomes `/Users/sac`) | P1+P5 |
| 3b | `arrow/scripts/build_receipt.py:43` | `ARROW_ROOT.parent.parent / authority/stardf/license-compat.json` | same register dependency | P1+P5 |
| 4 | `arrow/docs/PRD.md:5,12,26,33`; `arrow/docs/ARD.md:14-18,90` | absolute `/Users/sac/ggen-legacy/stardf/...` citations | live docs of record point at dead absolute paths | P6 |
| 4b | `docs/jira/v26.10.31/03-substrate-map.md:257` | `/Users/sac/ggen-legacy/scripts/verify_stardf_ontology.py` | live substrate-map row cites parent script | P3+P6 |

### 1b. Explicit leave-as-is (evidence/provenance — rewriting corrupts receipts)

- `arrow/evidence/*.json` — absolute paths are as-of-run records (incl.
  `replay-2026-10-31.json` `"repo": "/Users/sac/ggen-legacy"`)
- `VENDOR-PROVENANCE.md` (~15 crates) and `PROVENANCE.md` files — historical provenance
- `arrow/docs/C5R-C6-STAGING.md`, `arrow/docs/COMMIT-PLAN.md:258` — as-of-date campaign
  receipts
- planning docs' as-of-date path statements (`docs/jira/v26.10.31/*.md`, except the one
  live row at 03-substrate-map.md:257)
- `arrow/crates/stardf-sparql-algebra/tests/disabled/README.md:22` — disabled-test citation
- `arrow/rust_out/` — gitignored build artifact (parent `.gitignore` carries `**/rust_out`);
  moves with the filesystem mv; its `../backtrace/...` refs are artifact-internal, untracked
- `arrow/crates/stardf-iri/benches/parse.rs` — synthetic `/root`-escaping string in a
  path-normalization bench; not a file reference; no fix
- `Cargo.toml` `repository = "https://github.com/seanchatmangpt/ggen-legacy"` (~15 crates)
  — remote metadata, not a file include; zero build impact; optional follow-up commit in
  the new repo, not the move commit

### 1c. Looks like an escape, is not (targets verified existing in-tree)

- `arrow/README.md` → `../docs/jira/v26.10.31/*` (stardf/docs — exists, 15 files)
- `arrow/docs/V26-11-1-VERDICT.md:212`, `V26-11-1-GAPS.md:242` →
  `../scripts/publish_stardf.py` (exists)
- `arrow/crates/stardf-rdf/tests/corpus/w3c/{trig,turtle}/iri/manifest.ttl:12` →
  `../../README.md` (exists)
- `arrow/crates/stardf-text` refs to `../iri/unicode/*`, `../../lexicons/artifacts/*`
  (exist)
- `arrow/crates/stardf-columnar/tests/golden_bytes.rs:400` →
  `../../scripts/columnar_manifest.sha256` (exists)
- arrow workspace `path = "../stardf-*"` sibling deps; no `../../../` path deps (grep exit 1)
- `arrow/ggen.toml` absolute paths → `/Users/sac/ggen-marketplace/...` (external repo,
  unaffected by this move)
- `AGENTS.md` cites `~/purrdf` rev `0da05783` — external upstream, unaffected

## 2. Move mechanics (coordinator-executed)

### 2a. Pre-move fixes P1–P8 (applied inside stardf/ before the mv)

- **P1** Copy parent `authority/stardf/` (7 entries: `stardf_contract.ttl` — CANONICAL,
  174,481 B, 66 diff lines vs the in-tree copy; in-tree
  `arrow/authority/stardf/stardf_contract.ttl` at 172,248 B is an older snapshot lacking
  the `StardfAbiVersion`/`StardfCapabilities`/... mappings — plus `license-compat.json`,
  `license-inventory.toml`, `stardf-contract.json`, `stardf-columnar.json`,
  `marketplace_packs.json`, `rust-pack/`) into `stardf/authority/` (new top-level dir in
  the moved tree). Parent copy then stands as frozen evidence; the in-tree copy becomes
  canonical for the moved tree. Sub-flag: `arrow/authority/stardf/` then holds a stale
  duplicate — delete it in the same pass (ERRC: delete outranks keep-both).
- **P2** Copy parent `tickets/GL-STARDF-001.md` → `stardf/tickets/`; re-point the three
  citing surfaces: `arrow/docs/avec-rehearsal.md:3` → `../../tickets/GL-STARDF-001.md`
  (valid once the ticket is in-tree at `stardf/tickets/`); `tickets/README.md` [t1] →
  `GL-STARDF-001.md`; `AGENTS.md` `../tickets/...` → `tickets/...`. Also fix the
  already-malformed 1b link to `../../arrow/crates/stardf-arrow-capi`.
- **P3** Copy parent `scripts/verify_stardf_ontology.py` → `stardf/scripts/`; re-point its
  `DEFAULT_TTL` to the in-tree contract (`authority/stardf/stardf_contract.ttl` relative
  to the moved repo root). `PURRDF_HEADER = /Users/sac/purrdf/...` stays — external
  upstream, unaffected. Parent keeps its original as historical.
- **P4** `sed -i '' 's|/Users/sac/ggen-legacy/stardf|/Users/sac/stardf|g'`
  `arrow/scripts/h03_bootstrap_sim.sh` and
  `s|/Users/sac/ggen-legacy'|/Users/sac/stardf'|` on
  `workflows/v26.11.1-full-parity.workflow.js` (REPO constant, line 56; scriptPath
  comment, line 24).
- **P5** Re-point the two license-gate constants after P1:
  `verify_stardf_licenses.py:20` `parents[3]` → `parents[2]` (repo root is the moved
  tree root now, where P1 lands `authority/`); `build_receipt.py:43`
  `ARROW_ROOT.parent.parent` → `ARROW_ROOT.parent`.
- **P6** sed the live docs of record:
  `s|/Users/sac/ggen-legacy/stardf|/Users/sac/stardf|g` on `arrow/docs/PRD.md` and
  `arrow/docs/ARD.md`; on `docs/jira/v26.10.31/03-substrate-map.md:257`:

```bash
sed -i '' \
  's|/Users/sac/ggen-legacy/scripts/verify_stardf_ontology.py|'\
'/Users/sac/stardf/scripts/verify_stardf_ontology.py|' \
  docs/jira/v26.10.31/03-substrate-map.md
```
- **P7** `AGENTS.md` (top-level, landed 22:39 by another lane) cites three parent paths —
  `../AGENTS.md` (standing authority), `../tickets/GL-STARDF-001.md`,
  `../authority/stardf/stardf_contract.ttl`. Post-P1/P2 these become in-tree
  (`tickets/GL-STARDF-001.md`, `authority/stardf/...`). The `../AGENTS.md` standing-authority
  pointer needs an explicit amendment naming the post-move authority chain —
  coordinator/lane decision; minimum mechanical fix: replace the pointer with a
  historical-basis citation (the ggen-legacy AGENTS.md governed stardf through the move
  commit) so no dangling `../` remains.
- **P8** Nothing references parent `CLAUDE.md` from inside the tree (grep: zero hits for
  `../CLAUDE.md`) — no fix needed; noted because the parent CLAUDE.md names ggen-legacy
  verify commands that will not cover stardf post-move.

### 2b. The move itself

```bash
# inside /Users/sac/ggen-legacy, after P1-P8 and a clean tree for stardf/*
mv /Users/sac/ggen-legacy/stardf /Users/sac/stardf   # rename(2), instant, same volume
cd /Users/sac/ggen-legacy
git add -A stardf/            # stages the mass deletion of the tracked tree
git add README-stardf.md      # pointer stub (§3), created in the same commit
```

Notes: `git mv` cannot move a tracked dir out of its repo — filesystem `mv` first, then
stage deletions. The mv carries untracked build artifacts (`target/`, `target-lane07/`,
`rust_out/`) and uncommitted tracked modifications (the session's stardf diff rides
along; commit in the new location or before the mv, coordinator's call — declare which).
History stays queryable via `git log --follow` from the deletion commit.

### 2c. Move commit message (coordinator: write to a file, `git commit -F <file>`)

```text
chore(stardf): relocate stardf/ to standalone ~/stardf (Lane H package)

The stardf reconstitution tree (arrow workspace, 53 crates, docs, tickets,
workflows) moves out of ggen-legacy to /Users/sac/stardf as a standalone
tree. Whole-dir move: nothing of stardf/ remains in ggen-legacy.

Self-containment sweep (Lane H, 2026-10-31, 12,297 files): 16 raw ../
escapes, all closed pre-move — license-gate register copied in-tree
(authority/stardf/), GL-STARDF-001 ticket copied in-tree (tickets/),
verify_stardf_ontology.py copied in-tree (scripts/), license-gate root
constants re-pointed (parents[3] -> parents[2], ARROW_ROOT.parent),
parent-absolute paths re-pointed in live docs and one script, ticket and
authority citations re-pointed in AGENTS.md / tickets/README.md /
avec-rehearsal.md. Historical evidence and VENDOR-PROVENANCE files left
as-written (as-of-date records; see stardf/README-MOVE-NOTE.md §1b).

Parent side: README-stardf.md pointer stub added; parent CLAUDE.md,
AGENTS.md, README.md, justfile, Cargo.toml, .gitignore, .github carry
zero stardf references (grep-verified) — no parent edits required.
Parent authority/stardf/ and scripts/verify_stardf_ontology.py kept as
frozen evidence (in-tree copies are canonical post-move).

Standing: SELF-CONTAINED WITH NAMED FIXES (all fixes applied pre-move).
Package of record: stardf/README-MOVE-NOTE.md (moves with the tree).
```

## 3. Pointer stub for ggen-legacy (coordinator creates `README-stardf.md` at repo root)

```markdown
# README-stardf — pointer stub

The `stardf/` reconstitution tree moved out of this repository to the standalone
tree at `~/stardf` (filesystem move 2026-10-31; git history via
`git log --follow` from this commit).

- Tree: `/Users/sac/stardf` (arrow workspace: `~/stardf/arrow`, 53 crates)
- Authority: see `stardf/CLAUDE.md` there (and `stardf/AGENTS.md` /
  `stardf/tickets/README.md` for the ticket-gated admission chain)
- Package of record for the move: `~/stardf/README-MOVE-NOTE.md`
```

Dependency flag: `stardf/CLAUDE.md` did NOT exist as of 22:50 (checked on disk;
`stardf/AGENTS.md` does). Either a lane lands `stardf/CLAUDE.md` pre-move or the stub
cites `stardf/AGENTS.md` instead — coordinator picks at T0.

## 4. Post-move verification (run from `/Users/sac/stardf`)

```bash
cd /Users/sac/stardf
# 1. escape sweep re-run (the Lane H resolver) — must print 0 real escapes
#    (rust_out artifact + the synthetic bench string are the only tolerable hits)
# 2. license gate on the in-tree register
python3 arrow/scripts/verify_stardf_licenses.py --root arrow
#    -> exit 0, R1 PASS (register now read from ~/stardf/authority/stardf/)
# 3. workspace resolves standalone, no parent involvement
cd arrow && cargo metadata --no-deps --format-version 1 > /dev/null && echo WORKSPACE-OK
# 4. no live references to the old repo root remain (live surfaces only)
grep -rn '/Users/sac/ggen-legacy' --include='*.md' --include='*.py' --include='*.sh' \
  --include='*.js' --include='*.toml' . | \
  grep -vE 'evidence/|PROVENANCE|COMMIT-PLAN|C5R-C6|disabled/' || echo LIVE-SWEEP-CLEAN
# 5. ontology verifier on the in-tree contract
python3 scripts/verify_stardf_ontology.py   # after P3 re-point; needs ~/purrdf present
# 6. ticket and authority citations resolve in-tree
ls tickets/GL-STARDF-001.md authority/stardf/stardf_contract.ttl
```

Expected: R1 exit 0, `WORKSPACE-OK`, `LIVE-SWEEP-CLEAN`, both paths exist. Plus the
existing local gates if a full census is wanted: `cargo test` courts per the standing
table in `AGENTS.md` / `arrow/evidence/v26.11.1-manufacturing-receipt.json`.

## 5. Parent-side residue and decision flags (coordinator)

- `authority/stardf/` (7 entries) — P1 copies it in-tree; parent copy stays as frozen
  evidence. DECISION: keep as evidence (recommended) vs delete post-copy.
- `scripts/verify_stardf_ontology.py` — P3 copies it in-tree; parent copy historical.
- `tickets/GL-STARDF-001.md` — P2 copies it in-tree; parent keeps the original.
- `docs/DECISION-BRIEFS/D9-freeze-ratification.md`, `scripts/bulk_copy_purrdf.py`,
  `scripts/extract_stardf_*.py`, `scripts/extract_purrdf_ontology.py`,
  `scripts/verify_marketplace_packs.py`, `scripts/workflows/stardf-10x20.workflow.mjs` —
  parent-side historical campaign artifacts; stay (grep-verified: no live parent surface
  needs them post-move).
- Parent `CLAUDE.md`, `AGENTS.md`, `README.md`, `RELEASE_CONTROL.md`, `justfile`,
  root `Cargo.toml`, `.gitignore`, `.github/` — zero stardf references (grep-verified),
  no edits required. Parent CI has no stardf job (the arrow CI file under `stardf/arrow/ci/`
  is an explicitly staged draft, not wired; its own header documents that
  `.github/expected-workflows.txt` has 3 entries, no stardf job).
- Post-move, parent-side verify commands in parent `CLAUDE.md` no longer cover stardf —
  expected, that is the point of the move.

## 6. T0 re-verify gate (concurrent-lane caveat)

Lanes were landing files during this sweep (`stardf/.claude/`, `stardf/tickets/README.md`,
`stardf/AGENTS.md` at 22:38–22:39). At move time, re-run before the mv:

1. the python escape resolver (§1 method) over `stardf/` — expect 0 new real escapes;
2. `grep -rIn '/Users/sac/ggen-legacy' stardf/ --exclude-dir='target*' |`
   `grep -vE 'evidence/|PROVENANCE|COMMIT-PLAN|C5R-C6'`
   — expect only the enumerated §1a rows;
3. re-check `stardf/CLAUDE.md` existence (§3 stub dependency);
4. confirm no lane holds in-flight edits in stardf/ (compile-freeze discipline).

If the re-run matches this package, the move is mechanical. If new escapes appear,
amend this file (it moves with the tree) before the mv.

## See Also

- `AGENTS.md` — stardf executable-reconstitution authority (in-tree, scoped)
- `arrow/docs/OPERATOR-DECISIONS.md` — D1–D21 operator register
- `tickets/README.md` — GL-STARDF ticket index (lane E)
- `arrow/evidence/v26.11.1-manufacturing-receipt.json` — v26.11.1 campaign standing
