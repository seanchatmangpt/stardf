# stardf migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- Path: /Users/sac/stardf
- Remote: https://github.com/seanchatmangpt/stardf.git
- Branch: main@de7c9f5 (tracks origin/main, in sync)
- Last commit: 2026-11-01 — "chore: add root .gitignore"
- Commits: 2; size: 442 MB
- Languages/build: Rust workspace (Cargo.toml, Cargo.lock, rust-toolchain.toml, ~60 crates
  under crates/), justfile, ggen.toml, ontology.ttl, authority/, corpora/, evidence/
- Nested repos: none

## Observed state
- Dirty: 12306 tracked — 12301 are `D arrow/...` (the entire tracked top-level `arrow/` tree is
  deleted from the working tree); plus .claude/settings.json, AGENTS.md, CLAUDE.md, docs/ (29),
  tickets/ (3), workflows, vectors, tests, templates, snapshots, scripts, rust-toolchain.toml.
- Untracked: 12316 — count matches the deletion; `crates/arrow` exists. This is an
  UNCOMMITTED MOVE of `arrow/` into `crates/arrow/` plus the reorganisation docs.
- Ahead 0 / behind 0. Stash: 0. Unpushed branches: 0.
- Build dirs: no `target/` present. 442 MB is mostly corpora/ + the arrow tree.
- Submodules: none.

## What it is
stardf — the RDF/SPARQL/Arrow data stack reconstituted from ggen-legacy: ~60 Rust crates
(rdf-core, sparql-eval, shapes, duckdb-bridge, hnsw, ...) with `stardf-arrow` providing an
ArrowArrayStream seam for cgap. Ticket GL-STARDF-001.

## Classification
ACTIVE (census agrees: Nov 2026, live refactor in flight).

## Migration risk
- Lost if not copied: the entire workspace reorganisation (arrow -> crates/arrow move, 29 doc
  edits, 3 ticket edits, toolchain pin, snapshots/vectors). Only 2 commits exist on origin —
  the committed state is the pre-refactor baseline.
- No secrets or .env observed.
- Rating: HIGH.

## Recommended action
COMMIT-PUSH

1. `cd /Users/sac/stardf && git status --porcelain | grep -v '^ D arrow/' | grep -v '^?? crates/arrow' | head -60`
   — confirm the non-move changes are the 29 docs + tickets + configs.
2. `git add -A` (git will detect the renames) then
   `git status --porcelain | grep -c '^R'` — expect ~12k renames, not add+delete.
3. `git commit -m "refactor: move arrow/ into crates/arrow, workspace reorg (v26.11.16 census)"`.
4. `git push origin main`.
5. DECISION: `corpora/`, `evidence/`, `snapshots/` — confirm they are meant to be in git (size).
   If not, add to .gitignore before step 2 and RSYNC-COPY them separately.
6. `git status --porcelain | wc -l` -> 0; new machine:
   `git clone https://github.com/seanchatmangpt/stardf.git`.

## First commands on the new machine
- `cargo check --workspace` then `just --list` and the justfile's gate (e.g. `just verify`)

## See Also
- ../v26.11.1/
- ../v26.10.31/
- /Users/sac/docs/jira/v26.11.16/README.md
