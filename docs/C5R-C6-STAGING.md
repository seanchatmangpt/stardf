# C5R-C6-STAGING

> Staging classification for the v26.11.1 finish-campaign residue (task H04,
> lane lane19). Written read-only: NO `git add`, NO commits — execution belongs
> to the coordinator. Snapshot below is as-of its timestamp; the coordinator
> re-runs `git status --porcelain` at staging time (COMMIT-PLAN.md post-mortem:
> "the split above is descriptive, not prescriptive").

## Snapshot receipt

- Repo: `/Users/sac/ggen-legacy` @ branch `fix/v26.9.29-ci-gates`, HEAD
  `724d5425` (c4r). Scope: `stardf/arrow` + `authority/stardf` + `stardf/docs`.
- Command: `git status --porcelain=v1 --untracked-files=all -- stardf/arrow
  authority/stardf stardf/docs` — **1,448 paths** (1,288 `M` / 135 untracked /
  24 `D`), read 2026-10-31 ~21:45 local.
- Live-tree drift witnessed while classifying: publish receipt
  `evidence/publish_stardf-receipt-20261101T044209Z.json` landed mid-read, and
  lane lease `target-lane07/CACHEDIR.TAG` (21:39) appeared mid-read. The add
  commands below are path-globs, so further drift of the same classes absorbs.
- Reconciles with the COMMIT-PLAN.md post-mortem snapshot (1,447 paths:
  1,285 M / 139 untracked / 23 D); delta = live drift, not disagreement.

## c5r — docs commit (gate: `python3 scripts/verify_docs.py --strict`)

| path | state | carries |
|---|---|---|
| `stardf/arrow/docs/OPERATOR-DECISIONS.md` | M | D19-D21 appends (detail below) |
| `stardf/arrow/docs/COMMIT-PLAN.md` | M | final post-mortem + c5r/c6 split |
| `stardf/arrow/docs/SUPERSEDED-CLAIMS.md` | ?? | supersession register |
| `stardf/arrow/docs/V26-11-1-VERDICT.md` | ?? | verdict + explicit nonclaims |
| `stardf/arrow/docs/V26-11-1-GAPS.md` | ?? | gap register (landed 21:37) |
| `stardf/arrow/docs/design/` (2 files) | ?? | purrdf change-path + retrieval-ladder docs |
| `stardf/docs/jira/v26.10.31/04-falsifiers.md` | M | falsifier ledger update |
| `stardf/docs/jira/v26.10.31/10-duckdb-rehome-plan.md` | M | rehome plan RESOLVED (D13) |
| `stardf/docs/jira/v26.10.31/14-definition-of-done.md` | M | DoD Amendment 2 |
| `stardf/arrow/docs/C5R-C6-STAGING.md` | ?? | this file; lands with c5r |

Detail: D19 ratifies frozen-duckdb `-sys`/`-builder` `=2.0.4` x3; D20 makes
publish closure first-class and ladder-judged, standing BLOCKED; D21 registers
the edition 2024 / rust-version 1.98 build-driven deviation. The rehome-plan
diff flips PENDING-OPERATOR to RESOLVED (option c: registry-only, exact pins);
the DoD diff records the `duckdb`/`libduckdb-sys` naming divergence against
the actual frozen-duckdb family identities.

Classification decisions (registered, with authority):

1. `SUPERSEDED-CLAIMS.md` and `docs/design/` are NOT named in the H04 brief's
   c5r file list, but the COMMIT-PLAN.md post-mortem's own c5r definition
   names both — in-repo authority governs, so both are c5r.
2. `SCAN-COMPLETION.md`: the H04 brief lists it for c5r, but it is already
   tracked and clean — landed in `b87ea6fb` (c5). Nothing to stage.
3. `docs/jira/` at repo root (9 historical files, c0 residue): OUTSIDE the
   H04 scope (`stardf/arrow` + `authority/stardf` + `stardf/docs`) but the
   post-mortem assigns it to c5r. Coordinator folds it in at staging time or
   defers; do not silently skip.

### c5r stage + gate commands (repo root `/Users/sac/ggen-legacy`)

```bash
git add stardf/arrow/docs/OPERATOR-DECISIONS.md \
        stardf/arrow/docs/COMMIT-PLAN.md \
        stardf/arrow/docs/SUPERSEDED-CLAIMS.md \
        stardf/arrow/docs/V26-11-1-VERDICT.md \
        stardf/arrow/docs/V26-11-1-GAPS.md \
        stardf/arrow/docs/design/ \
        stardf/arrow/docs/C5R-C6-STAGING.md \
        stardf/docs/jira/v26.10.31/04-falsifiers.md \
        stardf/docs/jira/v26.10.31/10-duckdb-rehome-plan.md \
        stardf/docs/jira/v26.10.31/14-definition-of-done.md
# optional, coordinator decision (out of H04 scope; post-mortem c5r):
git add docs/jira/
# gate (exit 0 required)
python3 scripts/verify_docs.py --strict
```

## c6 — sweep commit (gate: `cargo check --workspace` exit 0 + four-target courts green)

Everything in scope not staged by c5r. Counts as of the snapshot: **1,283
modified**, **130 untracked**, **24 deletions** (1,288-5, 135-5, 24).

| bucket | paths | carries |
|---|---|---|
| license register | `authority/stardf/license-compat.json` (M) | register closure |
| manifests | `stardf/arrow/Cargo.toml`, `Cargo.lock` (M) | `=2.0.4` x3 pins |
| gitignore | `stardf/arrow/.gitignore` (M) | `.ggen*`, `signing.key` |
| CI workflow | `stardf/arrow/ci/stardf-arrow-reconstitution.yml` (M) | workflow delta |
| crates sweep | `stardf/arrow/crates/` (detail below) | source sweep |
| evidence | `stardf/arrow/evidence/` (14 new + 1 deletion) | receipts |
| corpora/vectors | `stardf/arrow/corpora/` (2), `vectors/` (17) | test corpora |
| court scripts | `stardf/arrow/scripts/` (2 new) | publish tooling |
| deletions | 24 `D` (detail below) | sweep removals |
| NOT staged | `stardf/arrow/target-lane07/CACHEDIR.TAG` | lane lease — never commit |

Bucket detail:

- crates sweep, modified (1,274 files at snapshot): edition-2024/MSRV/metadata
  unification, dual-key manifest work, scan-crate growth, court repairs
  across all 53 crates (src/tests/benches, incl. 26 Cargo.toml).
- crates sweep, untracked (~130 files): `stardf-scan-duckdb/` (10: manifest,
  PROVENANCE, 6 license files, src, scan courts); `licenses/` dirs on 11
  crates (56 files: LICENSE-APACHE/MIT/MULAN, CC-BY-4.0, NOTICE,
  inventory.json); `PROVENANCE.md` x7; `stardf-testkit/metaschemas/` (20);
  `tests|examples/disabled/` on 4 crates.
- evidence: 8 publish receipts (incl. T044209Z), lane07/11/16 receipts,
  avec-court FAIL receipts 21:02-21:05 (exit 101) — the receipt notes for
  the avec repair owed by the avec-gate lane; DRAFT manufacturing receipt
  deleted.
- deletions: `golden_bytes.rs`, `evidence/v26.11.1-manufacturing-receipt-
  DRAFT.json`, 7 removed tests/examples (stardf-rdf x1, stardf-shapes x5,
  stardf-shex x2), 6 lane build-root leases (`target-l3sql/`,
  `target-w2l{01,07,10,13,14,19}/` cache files) — cleanup-law removals.
- target-lane07: freshly minted lease (21:39). Delete at integration
  (cleanup law) or extend `.gitignore` with `/target-*/`; do NOT commit.

### c6 stage commands (repo root)

```bash
git add authority/stardf/license-compat.json
git add stardf/arrow/Cargo.toml stardf/arrow/Cargo.lock
git add stardf/arrow/.gitignore
git add stardf/arrow/ci/stardf-arrow-reconstitution.yml
git add stardf/arrow/crates/
git add stardf/arrow/evidence/
git add stardf/arrow/corpora/ stardf/arrow/vectors/
git add stardf/arrow/scripts/
git add stardf/arrow/golden_bytes.rs
git add -A stardf/arrow/target-l3sql \
          stardf/arrow/target-w2l01 \
          stardf/arrow/target-w2l07 \
          stardf/arrow/target-w2l10 \
          stardf/arrow/target-w2l13 \
          stardf/arrow/target-w2l14 \
          stardf/arrow/target-w2l19
```

`git add <dir>` stages deletions under the pathspec (git 2.x), so the 24
deletions stage through the same commands. After c6 staging, verify nothing
was missed:

```bash
git status --porcelain -- stardf/arrow authority/stardf stardf/docs
# expected residue: only target-lane07/CACHEDIR.TAG (untracked lease — remove
# on disk, never committed) + any drift that landed after this snapshot
```

### c6 gate

Run from `stardf/arrow`. Compile gate witnessed exit 0 at 21:05 via publish
receipt T040521Z (G9) on this tree's content; re-run required at staging time.

```bash
cargo check --workspace && echo C6-COMPILE-GATE-PASS
```

Four-target courts green (arrow, duckdb, sql, n3 per
`stardf/docs/jira/v26.10.31/13-v26111-target-ledger.md`). Standing warning,
from `V26-11-1-VERDICT.md` nonclaims (21:29 probe): **only arrow holds court
receipts** — `scan_courts.rs` (6 courts), `sql_golden.rs`, and the n3 suites
have no run receipts on any post-copy tree. The court-green component of this
gate is therefore UNWITNESSED as of this snapshot; the gate run at staging
time must produce those receipts, not cite them. Court commands implied by
crate layout (paths witnessed on disk):

```bash
cargo test -p stardf-scan-duckdb --test scan_courts
```

sql goldens + n3 suites run through the workspace test ladder
(`cargo test --workspace`) with per-target receipt capture. Adjacent on-disk
courts (additive, recommended before the sweep lands): license gate
`python3 stardf/arrow/scripts/verify_stardf_licenses.py` (exit 0 — validates
the c6 `license-compat.json` update), header freeze
`python3 stardf/arrow/scripts/verify_header_freeze.py`, and the two-arm
cache-absent rehome court `stardf/arrow/scripts/verify_duckdb_rehome.sh`
(heavy; the F04 cache-absent re-run is separately UNWITNESSED per the
verdict). All three exist on disk and were read, not executed, by this lane.

## Open items this staging does NOT close

- avec-court repair + re-run owed by the avec-gate lane (FAIL exit 101,
  21:02-21:05 receipts land in c6; no green claim until re-witnessed).
- The frozen-duckdb-sys/-builder register-extension ratification
  (lane-11 r2, verdict row 122) — register update lands in c6 regardless.
- Flag 2 (stardf-arrow-bench member lock) and Flag 6 (docs/jira c0 split)
  remain open per the post-mortem.
- Lane lease `target-lane07/` — disk action, not a commit.
