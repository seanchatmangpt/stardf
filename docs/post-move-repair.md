# Post-move repair receipt (lane S-1-post-move, 2026-11-01)

Repo moved /Users/sac/ggen-legacy/stardf -> /Users/sac/stardf. HEAD at receipt
write: de7c9f5 (baseline fe935f7 + .gitignore). MOVE-doc census: 192 hits / 78
files pre-move. This lane's census this session: 251 hits / 85 files (excludes
.git, target*, rust_out) at lane start; 304 hits at lane end (concurrent lanes
kept landing; counts are as-of-cited).

## Runtime-affecting fixes (this lane)

| # | file | before | after | verify |
|---|---|---|---|---|
| 1 | arrow/scripts/h03_bootstrap_sim.sh:9 | `cd /Users/sac/ggen-legacy/stardf/arrow` | `cd "$(dirname "$0")/.."` | bash -n exit 0; real run exit 0 (below) |
| 2 | workflows/v26.11.1-full-parity.workflow.js:24,56,60-62 | REPO='/Users/sac/ggen-legacy'; REPO+'/stardf/...' joins; scriptPath comment | REPO='/Users/sac/stardf'; joins drop '/stardf'; scriptPath updated | grep clean of ggen-legacy in code lines; node --check result unchanged vs HEAD (below) |
| 3 | .claude/settings.json `_scope` annotation | AUTHORITY LANE C (/Users/sac/ggen-legacy/stardf) ... stardf/arrow/scripts/ | AUTHORITY LANE C (/Users/sac/stardf) ... arrow/scripts/ | python3 json.load exit 0 |

## Runtime verification (commands run this session)

- `bash -n arrow/scripts/h03_bootstrap_sim.sh` -> exit 0.
- `bash scripts/h03_bootstrap_sim.sh` (cwd /Users/sac/stardf/arrow,
  Cargo.lock sha256-snapshotted, byte-restored after: cmp clean) -> exit 0.
  The run itself proves fix #1: failure/output lines now cite
  /Users/sac/stardf/arrow (pre-fix the script silently cd'd into the OLD
  /Users/sac/ggen-legacy tree, which still exists on disk).
  Result: 4 OK / 21 FAIL / SIM DONE.
- `node --check workflows/v26.11.1-full-parity.workflow.js` -> exit 1
  "Illegal return statement" at line 541 -- IDENTICAL failure on the HEAD
  version (`git show HEAD:...` piped to node --check, exit 1). Pre-existing
  condition of the file (top-level return expects the Workflow runtime's
  wrapper), NOT introduced by this lane; the edited lines are string/const
  paths only.
- `python3 json.load` on .claude/settings.json -> parse clean.

## Pre-existing (NOT move damage; disclosed, unmodified)

- h03 sim 21 FAILs: workspace.dependencies are PATH-ONLY (arrow/Cargo.toml:313
  `stardf-hash = { path = "crates/stardf-hash" }`), so `cargo package` refuses
  ("dependency `stardf-hash` does not specify a version"). Manifest
  dual-key condition; independent of the move; owner = publish-closure lane.
- arrow/crates/stardf-arrow-capi/Cargo.toml:60-61 path deps
  `../../../../cgap/crates/{audit,cgap-ffi}`: broke under the OLD location,
  RESOLVE correctly from /Users/sac/stardf (up 4 = $HOME). Confirmed
  `cargo metadata --no-deps` exit 0. (A concurrent lane modified this file
  mid-lane; final state verified resolving.)

## Remaining /Users/sac hits after this lane (census classification)

- *.md / *.mdx prose: 217 lines (S-6's scope, named per dispatch).
- arrow/evidence/*.json historical receipts: 22 files -- NEVER rewritten
  (receipt identity); their /Users/sac/ggen-legacy paths are as-of-run
  records of commands executed pre-move.
- Runtime files KEEPING absolute paths deliberately (all targets EXIST on
  disk this session: ls exit 0):
  - arrow/ggen.toml -> /Users/sac/ggen-marketplace (pack pin)
  - arrow/scripts/publish_stardf.py:75 -> /Users/sac/cgap (DEFAULT_AVEC_REPO)
  - arrow/scripts/verify_duckdb_rehome.sh -> /Users/sac/frozen-duckdb (court
    subject; its guard greps for that literal by design)
  - arrow/authority/stardf/stardf_contract.ttl ->
    file:///Users/sac/purrdf provenance IRI (external identity anchor)
  - workflows/*.js line 69 CONTRACT string -> /Users/sac/purrdf archive
    command (vendoring law; external)
  - comments in arrow/Cargo.toml, stardf-rdf-core/Cargo.toml,
    stream_courts.rs, golden_bytes.rs -> historical-basis prose (S-6-class)
- Remaining `ggen-legacy` strings in runtime files are GitHub repository =
  URLs and CI checkout step NAMES (7 crate manifests, ci yml, contract.ttl
  comment) -- remote identities, not filesystem paths; not move damage.

## Concurrency incident (disclosed)

All three fixes landed, then were found REVERTED (tree rewritten; HEAD
advanced fe935f7 -> de7c9f5 mid-lane). All three re-applied and re-verified
on disk at receipt time; the h03 run above is on the RE-APPLIED script.

## Standing

ALIVE for the three named repairs (each verified on disk this session with
the commands above). The h03 bootstrap sim remains 21-FAIL as a pre-existing
manifest condition, not a move regression.
