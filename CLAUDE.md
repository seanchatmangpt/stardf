# stardf

## What this is

`stardf` is a deterministic, zero-copy columnar RDF 1.2 execution substrate: RDF 1.2 terms and
quads carried as Arrow/Parquet columnar data, queryable through DuckDB. It is a one-package
port of purrdf rev `0da05783` — 53 crates straight-copied as `stardf-*` members of the single
`arrow` workspace (decision D15). The tree is self-contained: no machine-local path overrides,
no `[patch.crates-io]`; frozen-duckdb is pinned registry-only (D19).

This is not generic columnar tooling — every export, decision, and dependency edge is
governed: exports by the frozen C-ABI surface, decisions by the operator register, receipts
by the receipt law below.

## Canonical authority (defer in this order)

1. `arrow/docs/ABI-FREEZE.md` — frozen C-ABI export surface. The v2 13-symbol table is the
   surface of record (PENDING-OPERATOR per D18); the built dylib still exports the v1
   13-symbol surface. Any export/layout/status-code change requires a new ticket; editing a
   table row to match the dylib is itself a freeze violation.
2. `arrow/docs/OPERATOR-DECISIONS.md` — operator decision register D1–D21, append-only: a
   later section supersedes execution detail only and never rewrites an earlier row.
3. `authority/stardf/*` receipts — `stardf-contract.json`, `stardf-columnar.json`,
   `stardf_contract.ttl`, `license-compat.json`, `license-inventory.toml`,
   `marketplace_packs.json`.
4. Receipt law `ggen.legacy.stardf.arrow.receipt.v1` — emitted by
   `arrow/scripts/verify.py --emit-receipt`; ledger at `arrow/docs/RECEIPTS.md`.
5. `arrow/docs/V26-11-1-VERDICT.md` and `arrow/docs/COMMIT-PLAN.md` — campaign verdict and
   commit ladder as executed.

## Standing vocabulary

Report subsystem state as one of: `ALIVE` / `PARTIAL_ALIVE` / `UNKNOWN` / `BLOCKED` /
`REFUSED`. Never claim an unqualified "works" or "done". A gate that cannot exit 1 is
VACUOUS; a skip is loud (`[SKIP] <reason>`), never a faked pass.

## Ticket-gated surface changes

The C-ABI surface is admission-gated: no `#[no_mangle]` outside the ratified surface (the 13
exports plus ratified Tier A additions) may land in `arrow/crates/stardf-arrow-capi/src/` —
surface work goes through the ratified ticket process with a pre-landing gate and an
`nm -gU` diff receipt (`arrow/tickets/GL-STARDF-002-freeze-expansion.md`).

## Verify

Run from `arrow/`:

```bash
cargo check --workspace --locked
cargo test --workspace --locked
python3 scripts/verify.py                  # substrate gate: ontology census, header/impl
                                           # parity, --mutate anti-vacuity, --snapshot freeze
python3 scripts/verify_stardf_licenses.py  # R1 license gate over authority/stardf/
bash scripts/verify_blake3_identity.sh     # D10 BLAKE3 identity court
python3 scripts/verify_header_freeze.py    # F4 header freeze court (standalone)
bash scripts/verify_duckdb_rehome.sh       # registry-only rehome court, =2.0.4 x3
just columnar-oracle                       # external DuckDB oracle (loud exit-3 skip)
```

Run the relevant subset before claiming any subsystem standing. Absent inputs are loud
`[SKIP]` + nonzero exit, never silent passes; a passing anti-vacuity mutation is a VACUOUS
GATE.

## Code discovery

Prefer `mcp__lumen__semantic_search` over `find`/`ls -R`/`grep -r`/broad reads for locating
code, definitions, or usages; fall back to exact-string grep only where the target path or
string is already known. A stale index is `UNKNOWN`, not evidence of absence — reindex if
results look incomplete.

## Decision register D15-D21 (summary)

Full text and evidence: `arrow/docs/OPERATOR-DECISIONS.md`.

- **D15 — one-package rule (DECIDED):** stardf is ONE package — every purrdf crate is
  straight-copied into the single `arrow` workspace; internal edges are ordinary workspace
  edges. No carve, no feature-gating by dependency.
- **D16 — 80/20 (DECIDED):** phase 1 scripted bulk copy (~53 crates); phase 2 mechanical
  rename + single root; phase 3 covers the four first-class targets only — arrow, duckdb,
  sql, n3.
- **D17 — honest scope (DECIDED):** the copied ~80% carries NO courts — compile-green at
  best; the parity claim is scoped to the four first-class targets.
- **D18 — ABI-v2 (PENDING-OPERATOR):** the revised 13-export surface is NOT adopted this
  campaign; arrow stays landed as-is on the v1 surface.
- **D19 — frozen-duckdb pins (DECIDED):** frozen-duckdb / -sys / -builder exact-pinned
  `=2.0.4` x3, registry-only; ratified after falsifier F04 fired cache-absent on `=2.0.3`.
- **D20 — publish closure (DECIDED; closure BLOCKED):** publish-preparedness is a
  first-class deliverable — unified crates.io metadata, dual-keyed deps at `=26.11.1`, and
  the fail-closed ladder `arrow/scripts/publish_stardf.py` (G1–G9 + R1, dry-run by
  default). Live publish sits behind the operator gate and is PENDING-OPERATOR.
- **D21 — edition 2024 / rust-version 1.98 (DECIDED):** build-driven deviation from the
  operator template (2021/1.88), registered on the books — vendored sources use let-chains
  and 19 member crates declare 1.98.

## Versioning

Workspace version `26.11.1` (`arrow/Cargo.toml`); campaign planning pack lives at
`docs/jira/v26.10.31/`.

## See Also

- `docs/jira/v26.10.31/` — campaign planning pack (`01-master-plan.md`,
  `14-definition-of-done.md`, `04-falsifiers.md`)
- `arrow/docs/` — as-built docs (`ABI-FREEZE.md`, `OPERATOR-DECISIONS.md`, `RECEIPTS.md`,
  `V26-11-1-VERDICT.md`)
- `workflows/v26.11.1-full-parity.workflow.js` — full-parity workflow definition
