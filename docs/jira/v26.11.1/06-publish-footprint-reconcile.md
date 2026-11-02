# Publish Footprint Reconcile — stardf v26.11.1 (Publish Lane 6)

> STATUS: RECONCILE EXECUTED — DECISION-READY, NOT RATIFIED. Three sources read
> from disk and diffed; census re-run by this lane, as-of 2026-10-31 22:56 PDT.
> Live publish stays BLOCKED behind the operator gates (G8 + double-gate); this
> document performs no publish, no manifest edit, no git action. Owner of the
> decision: operator. File owner: lane 6 only.

## Sources (all re-read this lane)

1. **Contract 21-cell closure (D20):** 21 cells incl. umbrella cell `stardf`;
   scan cell `stardf-duckdb-scan`.
2. **`scripts/publish_stardf.py` TIERS:** 25 crates, T0-T8 (comment-clean read
   of lines 87-117); count witnessed 25.
3. **On-disk publish=true manifests:** 25 packages unrestricted; 23 publish=[]
   explicit-false; root `stardf-arrow` publish=[].

Citations for source 1: `arrow/docs/V26-11-1-VERDICT.md:71` ("Contract closure
names 21 cells incl. umbrella `stardf`") + `arrow/evidence/v26.11.1-manufacturing-receipt.json`
pending_operator row "footprint reconcile" ("stardf-duckdb-scan contracted vs
stardf-scan-duckdb landed"). Citations for source 3: `cargo metadata
--format-version 1 --no-deps` in `arrow/` (this lane, 22:56): 48 packages, 25
publish=null; 25 `crates/*/Cargo.toml` carry explicit `publish = true`; latest
ladder receipt `publish_stardf-receipt-20261101T051252Z.json` G3 PASS clean.

Note on source 1: the contract's literal 21-name text is the operator's dispatch
and is not on disk in this tree. The 21 cells are recovered here by arithmetic
forced by three receipted citations: 21 cells incl. umbrella `stardf`
(VERDICT:71) = TIERS-25 minus the exact 4 crates H03 flipped publish=false→true
to close G3 (`arrow/evidence/h03-gate-closure-receipt-2026-10-31.json`:
stardf-sparql-algebra, stardf-core [crates/stardf-rdf-core],
stardf-sparql-results, stardf-rdf), with the two contracted spellings
(`stardf`, `stardf-duckdb-scan`). Consistency witnesses: VERDICT "was 21
pre-H03" matches 25-4; the manufacturing receipt contrasts contract-21 vs
on-disk-21 naming ONLY the umbrella + scan renames — no third delta.

## Union / intersection (as spellings vs as packages)

- Union (spellings): 27 = the 25 TIERS names + `stardf` (contract umbrella
  spelling; no crate) + `stardf-duckdb-scan` (contract scan spelling; no crate).
- Intersection (as packages): 25 — every contract cell resolves to exactly one
  on-disk publishable package; TIERS == census set exactly (set-equal,
  witnessed this lane).

Package-identity resolution of the two contracted spellings: `stardf` →
`stardf-rs` (the real umbrella, `crates/stardf-purrdf`, T8); `stardf-duckdb-scan`
→ `stardf-scan-duckdb` (T2).

## Deltas (each names its fix class)

1. **Umbrella cell named `stardf` — no crate named `stardf` exists.** Census:
   no package named `stardf` in cargo metadata; no `crates/stardf` dir
   (re-witnessed this lane). The umbrella ON DISK is package `stardf-rs`
   (`crates/stardf-purrdf`), already publish=true at T8. Never on disk either
   (lane07 receipt check `umbrella-stardf-cell`: "no live crates/stardf
   manifest exists in this workspace").
   - Fix classes: (a) AMEND the contract cell to `stardf-rs` — zero manifest
     work, footprint unchanged; (b) MATERIALIZE a new crate named `stardf` —
     new manifest + name-availability check (crates.io availability
     UNVERIFIED), and it duplicates `stardf-rs`'s re-export role. Per the
     dispatch naming note ("umbrella package is stardf-rs not stardf"),
     **amend is the reconciled choice**; materialize stays the operator's
     alternative.
2. **Scan cell naming: contract `stardf-duckdb-scan` vs landed
   `stardf-scan-duckdb`.** The landed package is publish=true, T2, first-class
   closure member (D20 evidence, `arrow/Cargo.toml:170-174,360-363`). Fix
   class: amend the contract cell spelling to `stardf-scan-duckdb`.
3. **Contract stale by 4 members (membership drift, superseded event).**
   Contract predates H03: stardf-core, stardf-sparql-algebra,
   stardf-sparql-results, stardf-rdf were publish=false at contract time and
   joined via H03's G3 fix-forward flips (h03 receipt, count 4). They are IN
   TIERS and IN census today. Fix class: amend the contract's member count to
   25 (no on-disk change).
4. **G8 nonclosure hard deps — the open membership fork (operator-owned,
   D17).** 24 rows (h03 receipt + VERDICT): stardf-rdf → {stardf-ed25519,
   stardf-sparql-eval, stardf-gts} + dev stardf-gts; stardf-sql →
   stardf-shapes; stardf-toml dev → stardf-sparql-eval; stardf-rs → 18
   (datalog, duckdb optional, entail, geo, glob, gts, hnsw, json, markdown,
   mime, process, retrieval, shapes, shex, slice, sparql-eval, text,
   validate). G8 FAIL at the newest ladder receipt (20261101T051252Z). This
   fork does not change the 25-name footprint today; it decides whether the
   closure stays 25 (deps re-routed/feature-gated) or moves (join-closure →
   44 members, or scope-down → rdf/sql/toml/n3/rs flip publish=false).
   Operator ratifies.

## Reconciled canonical publish list (25 packages, tier assignment = TIERS verbatim)

All at `=26.11.1` (census: the 25 publishable packages carry version 26.11.1
uniformly).

- T0: stardf-hash, stardf-alloc-probe, stardf-arrow-core
- T1: stardf-lex, stardf-deflate, stardf-branchless, stardf-testkit,
  stardf-events, stardf-arrow-bench, stardf-columnar
- T2: stardf-stack, stardf-iri, stardf-arrow-rs, stardf-xsd, stardf-arrow-capi,
  stardf-scan-duckdb
- T3: stardf-cdt
- T4: stardf-core, stardf-sparql-algebra
- T5: stardf-sparql-results
- T6: stardf-rdf
- T7: stardf-sql, stardf-n3, stardf-toml
- T8: stardf-rs (the umbrella; `crates/stardf-purrdf`)

Notes: the umbrella package is `stardf-rs`, not `stardf` (delta 1); the scan
crate is `stardf-scan-duckdb`, not `stardf-duckdb-scan` (delta 2). Root package
`stardf-arrow` is publish=[] (internal, not in the closure). Non-closure members
at 26.9.29 literals (datalog/ed25519/shapes/slice) stay publish=[] (H03).

## Decision rows (operator ratifies; ladder refuses until then)

1. **Footprint ratification** — amend the contract to this 25-package list
   (2 renames + 4 joined members), or re-cut it. Ladder effect: G1/G3 already
   read it from disk; consistent.
2. **Umbrella cell** — amend `stardf` → `stardf-rs` (recommended), or
   materialize a crate named `stardf`. Ladder effect: name availability on
   crates.io UNVERIFIED if materialize.
3. **G8 fork (D17)** — join-closure (+19, with version bumps) / feature-gate
   or re-route (source surgery) / scope-down (rdf/sql/toml/n3/rs out). Ladder
   effect: G8 stays FAIL until one exit is ratified and landed.
4. **Live publish go** — double gate `STARDF_PUBLISH_CONFIRM=ratified-v26.11.1`
   + `--publish`. Ladder effect: registry untouched until both present.

## Nonclaims

- No git command, no manifest edit, no ladder run, no registry touch by this
  lane. Census = read-only `cargo metadata --no-deps` + manifest greps at
  22:56 PDT 2026-10-31.
- The contract 21-name list is a RECONSTRUCTION (arithmetic + receipts), not
  the literal operator text; the literal text lives with the operator's
  dispatch. If the literal list contradicts the reconstruction, the literal
  list wins and this file is stale.
- G8 standing cited as-of the newest receipt (20261101T051252Z, 22:12:50);
  parallel lanes are landing edits on this tree (root `Cargo.toml` mtime
  22:53:57 during this lane's read) — a fresh ladder run is owed before any
  publish claim.
- crates.io name availability for `stardf` (if the materialize path is taken):
  UNVERIFIED — registry-time check.
- This file carries no live-publish authorization; D20's double gate stands.

## See Also

- `arrow/scripts/publish_stardf.py` — TIERS of record (25, T0-T8); runtime
  re-derivation from live cargo metadata
- `arrow/docs/OPERATOR-DECISIONS.md` — D17 (closure scope), D20 (publish
  closure first-class, BLOCKED), H03 registration (G3 flips + G8 rows)
- `arrow/docs/V26-11-1-VERDICT.md` — "Publish footprint vs contract" section
- `arrow/evidence/h03-gate-closure-receipt-2026-10-31.json` — the 4 G3 flips,
  24 G8 rows, bootstrap blast radius
- `docs/jira/v26.11.1/09-publish-runbook.md` — the operator's live-publish
  runbook (Lane 9; TIERS projection)
- `docs/jira/v26.10.31/03-substrate-map.md` — package↔dir map (stardf-rs =
  crates/stardf-purrdf; stardf-core = crates/stardf-rdf-core; stardf-arrow-rs
  = crates/stardf-arrow)
