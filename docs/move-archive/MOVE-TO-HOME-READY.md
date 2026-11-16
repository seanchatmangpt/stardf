# MOVE-TO-HOME-READY

Move-readiness checklist for relocating `stardf/` from
`/Users/sac/ggen-legacy/stardf` to `~/stardf`. Census run 2026-10-31 against the
tree as it stands (lane G; NO git, read-only except this file).

## Census method and totals

```bash
grep -rn "/Users/sac" /Users/sac/ggen-legacy/stardf \
  --exclude-dir=target --exclude-dir=target-lane07 --exclude-dir=.git \
  --exclude-dir=__pycache__ --exclude-dir=rust_out
# 192 hits / 78 files (pinned; re-run before the move — the tree is live).
```

Decomposition (sums exactly): 170 hits in `*.md`/`*.json` (docs, planning,
evidence receipts — no runtime effect) + 22 hits in code/config/scripts, split:

| bucket | hits | verdict |
|---|---|---|
| breaks on move (runtime) | 2 | must fix pre-move |
| machinery prose (workflow.js comment/prompt text, test doc-comments) | 4 | fix optional |
| sibling targets outside stardf (do not move; unaffected) | 16 | no action for the move |

## 1. Breaks on move — must fix before/at move time

- `arrow/scripts/h03_bootstrap_sim.sh:9` — current:
  `cd /Users/sac/ggen-legacy/stardf/arrow`.
  Fix: `cd "$(dirname "$0")/.."` (script lives in `arrow/scripts/`).
- `workflows/v26.11.1-full-parity.workflow.js:56` — current:
  `const REPO = '/Users/sac/ggen-legacy'`.
  Fix: derive from `import.meta.url` (`dirname` of the script + `..`), or mark
  the script campaign-historical. Breaks only if re-run post-move.

`workflows/...workflow.js:56` breaks only if the workflow is re-run post-move;
`h03_bootstrap_sim.sh` breaks on every run. Both are one-line fixes.

## 2. Absolute but not move-breaking (targets live outside stardf/)

These point at machine-local siblings that stay put when stardf/ moves. Not
broken by the move; listed for completeness.

- `arrow/ggen.toml:34` + `:117-121` — `/Users/sac/ggen-marketplace/packs/rust-pack`
  (pack path + 5 gate .rq paths). Sibling repo, stays put; unaffected. Optional
  later hardening: `~/`-relative; NOT required for the move.
- `arrow/scripts/publish_stardf.py:75` — `DEFAULT_AVEC_REPO = Path("/Users/sac/avec")`
  — sibling; unaffected. Optional hardening: `Path.home() / "avec"`.
- `arrow/scripts/verify_duckdb_rehome.sh:44` — `FORK=/Users/sac/frozen-duckdb` —
  machine-local fork outside stardf/; unaffected by the move (this is the
  frozen-duckdb caveat, resolved below).
- `arrow/authority/stardf/stardf_contract.ttl:20` — `file:///Users/sac/purrdf/...`
  provenance IRI; never resolved at runtime.
- `arrow/Cargo.toml:134,260,272` and
  `arrow/crates/stardf-rdf-core/Cargo.toml:5` — comments only.

## 3. Prose/evidence (no runtime effect)

README.md, docs/ (incl. `docs/jira/v26.10.31/*` planning prose), all
`VENDOR-PROVENANCE.md`, `PROVENANCE-entries.md`, `DISPOSITION.md`, disabled-test
READMEs, doc-comments in `stream_courts.rs:6` / `golden_bytes.rs:5`, and the
`evidence/*.json` receipts. Receipts record the old subject path as-of receipt
time; receipts are immutable — do NOT rewrite them for the move. New receipts
post-move will name `~/stardf` naturally.

## 4. Frozen-duckdb caveat — rehome court is location-independent (verified)

`arrow/scripts/verify_duckdb_rehome.sh` — verified by reading in full:

- Sandbox is `mktemp -d /tmp/rehome-court.XXXXXX` (line 81); subject manifests
  are generated in-sandbox; the script `cd`s into the sandbox. Nothing resolves
  through stardf's own location.
- The machine-local trees it renames (`/Users/sac/frozen-duckdb` line 44,
  `$HOME/.frozen-duckdb` line 45) live OUTSIDE stardf/ and do not move. ARM B's
  rename-away of the fork + cache is unaffected by stardf's relocation.
- The `/Users/sac/frozen-duckdb` literals at lines 7/15/163 are comments and the
  anti-vacuity guard grep (court REFUSES if that path appears in sandbox
  manifests). Guard semantics are location-independent.
- Post-move precondition unchanged: the court requires both machine-local trees
  to exist (PRE-digest step exits 1 if missing).

Verdict: the court still passes from `~/stardf` by design. Optional post-move
witness: `bash arrow/scripts/verify_duckdb_rehome.sh` from the new root (15-60
min, renames real trees mid-run — do not run concurrently with anything using
frozen-duckdb).

## 5. avec-arrow git pin — location-independent (verified)

- `arrow/Cargo.toml:263`:
  `avec-arrow = { git = "https://github.com/seanchatmangpt/avec", rev = "67c0ec1" }`
- `arrow/Cargo.lock:317`:
  `source = "git+https://github.com/seanchatmangpt/avec?rev=67c0ec1`
  `#67c0ec1bad0bf4a81b2512e60f554b137ae596d9"` (one line in the lockfile)

URL + rev, no path component. Unaffected by the move. Note: the git checkout
lives under `~/.cargo/git/checkouts/` — machine-global, not tree-relative.

## 6. Workspace manifests — relative (verified)

- Root `arrow/Cargo.toml` `[workspace.dependencies]` (line 252 ff.): every `path`
  is relative (`crates/...`); no absolute path dep exists anywhere in the
  workspace (per-crate grep for `path = "/` and `git =` returned zero hits
  outside the root's avec pin).
- `rust-toolchain.toml`: channel/components/targets only.
- `justfile`: no absolute paths.
- No `.cargo/config.toml` exists anywhere under stardf/ (checked).
- 117 `env!("CARGO_MANIFEST_DIR")` occurrences across `*.rs` — compile-time,
  resolved against the tree at build time; correct at any location.

## 7. Post-move gate (run from `~/stardf/arrow`)

```bash
cargo check --workspace            # expect exit 0
python3 scripts/verify.py          # run 1
python3 scripts/verify.py          # run 2; diff run1 vs run2 receipts -> byte-identical
```

- verify.py is location-independent by construction (`SCRIPT_DIR` from
  `__file__`; all inputs under `ARROW_DIR`), and its receipt is deterministic
  ("sorted keys, no timestamps, no run-varying paths" — its own docstring), so
  the x2 byte-identical gate is sound.
- First `cargo` invocation from the new root re-uses `Cargo.lock` as-is; the
  build tree regenerates (do NOT move `target/`/`target-lane07/` — leave build
  roots behind; they are leases, not assets).
- Heavy optional witness: the duckdb rehome court (section 4).

## 8. Cargo.lock path-independence

`grep -n '/Users/\|source = "git+\|source = "path' arrow/Cargo.lock` returns
exactly one non-registry line: the avec git URL (section 5). Workspace path
deps appear with no `source` line; no absolute path anywhere in the lockfile.
Content-based fingerprinting means cargo re-uses the lock unchanged post-move.

## 9. Ordering

1. Fix the two section-1 lines (one-line each).
2. Move the tree (coordinator/git-owner transition — this lane ran NO git).
3. Run section 7 from the new root; compare receipts byte-identical.
4. Optional heavy witness: rehome court from new root (section 4).

## See Also

- `arrow/scripts/verify_duckdb_rehome.sh` — rehome court (location-independent
  by design)
- `arrow/scripts/verify.py` — deterministic post-move gate
- `arrow/scripts/publish_stardf.py` — publish ladder (CONFIRM_ENV-gated)
