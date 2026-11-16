# MOVE-COMPLETE — move repair receipt (lane S-6)

Date: 2026-11-01. The `stardf/` tree moved to the standalone `~/stardf`
repository (baseline commit `fe935f7`, "baseline: stardf tree as-moved from
ggen-legacy"). This is the governing-docs repair receipt.

## What was repaired (this lane, S-6)

- `AGENTS.md` — authority block rewritten for the post-move reality: the
  `../AGENTS.md`, `../tickets/GL-STARDF-001.md`, `../authority/stardf/`
  parent citations replaced; this tree is its own repo authority, parent
  `AGENTS.md` cited as historical basis only. Authority-precedence list
  renumbered 1–9 and de-parented; GL-STARDF-001 boundary section and
  See Also de-parented. Zero `../` references remain (grep-verified).
- `CLAUDE.md` — no `../` references existed; the canonical-authority chain
  (ABI-FREEZE → OPERATOR-DECISIONS → authority receipts → receipt law →
  verdict) is intact. Authority item 3 corrected to record the PARTIAL
  re-homing: only the `stardf_contract.ttl` snapshot is in-tree
  (`arrow/authority/stardf/`); the remaining register receipts
  (`stardf-contract.json`, `stardf-columnar.json`, `license-compat.json`,
  `license-inventory.toml`, `marketplace_packs.json`, `rust-pack/`) still
  live at `/Users/sac/ggen-legacy/authority/stardf/` as frozen evidence —
  the move package's P1 copy-in was NOT executed, so the R1 license gate
  (`arrow/scripts/verify_stardf_licenses.py` register inputs via
  `REPO = parents[3]`) currently dangles.
- `docs/move-archive/` — MOVE-TO-HOME-READY.md and README-MOVE-NOTE.md
  archived here (byte-copies, 6,998 B and 15,742 B). The originals remain
  at the repo root until the operator removes them — this lane ran no git.
- `tickets/README.md` — ticket index re-pointed at the in-tree recovered
  ticket (`GL-STARDF-001.md` sibling link replaces the parent-escaping
  `[t1]` link) and named as S-3's recovered copy.

## What remains (not done here, recorded, not invented)

- **Authority register re-homing (P1)** — copy
  `/Users/sac/ggen-legacy/authority/stardf/` (7 entries) to a top-level
  `authority/stardf/` in this tree, delete the stale
  `arrow/authority/stardf/` duplicate, and re-point
  `verify_stardf_licenses.py` (`parents[3]` → `parents[2]`) +
  `build_receipt.py` (`ARROW_ROOT.parent.parent` → `ARROW_ROOT.parent`).
  Owner: coordinator. Until then the R1 license gate dangles.
- **Ontology verifier copy-in (P3)** — `scripts/verify_stardf_ontology.py`
  not landed in-tree (`~/stardf/scripts/` does not exist); parent copy at
  `ggen-legacy/scripts/` is historical. Owner: coordinator.
- **Move-note removal** — root `MOVE-TO-HOME-READY.md` /
  `README-MOVE-NOTE.md` originals stay until the operator deletes them
  (git-tracked; coordinator owns git).
- **Substrate-map row** — `docs/jira/v26.10.31/03-substrate-map.md:257`
  cites `/Users/sac/ggen-legacy/scripts/verify_stardf_ontology.py`; the P6
  sed named in the move package was not witnessed as executed. Flag for
  the coordinator (outside this lane's lease).
- Root `.gitignore` may still need the lane-lease backstop entries once
  the coordinator integrates.

## Statuses

- AGENTS.md / CLAUDE.md / tickets/README.md repair: ALIVE (edits on disk,
  grep-verified zero `../` escapes in the three governing docs)
- Move completion: ALIVE (git baseline `fe935f7` exists in `~/stardf`)
- Authority register re-homing: BLOCKED (owner: coordinator; parent copy
  is the canonical register until copied)
- Move notes archived: ALIVE (byte-copies in docs/move-archive/)
