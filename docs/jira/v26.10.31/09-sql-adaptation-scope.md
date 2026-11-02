# Phase-2 Lane 6 — Vendored-Crate Adaptation Scope, SPARQL Stack (v26.10.31)

> **RESOLUTION — SUPERSEDED by the one-package rule (D15).** The §2 Option A/B/C decision
> is moot: stardf-sql was vendored WHOLE, with the shapes edge intact (ordinary workspace
> edge). Post-settlement: 4 goldens + doctest green (H01 receipt). The ViewTermId seam
> (§1a) and governor binding inputs (§1b) remain valid as method record. The analysis
> below is retained as written; it no longer states the live scope.

Makes the operator's no-rewrite decision concrete: three adaptation-scope options for
bringing the purrdf SPARQL stack into stardf. Companion to `06-phase0-columnar-sql-duckdb.md`
(§1 sizes, §2 seam, §5 governors, §10 operator register). PLANNING LANE — analysis only,
no code landed, no git mutations. All size/test counts are re-read from
`06-phase0-columnar-sql-duckdb.md` §1 (P0-1/P0-4/P0-15 receipts); this lane did not
independently re-count the crates — counts are as-of the P0 receipts, marked as such.

## 0. Inputs (cited, not re-derived)

| Crate | Size | Tests | Receipt |
|---|---|---|---|
| sparql-algebra | ~35K ln | 640 | P0-1 |
| sparql-eval | ~215K ln | 2,678 | P0-1 |
| sparql-results | ~10K ln | 223 | P0-1 |
| sparql-conformance | ~9K ln | 184 | P0-1 |
| purrdf-columnar (context) | 8,697 ln / 37 | — | P0-15 |

Operator decision register item 2 (`06-...md` §10.2): "SQL port scope — full 270K-line
SPARQL stack vs subset (states a plan or it isn't one)". This doc is that stated plan,
in three mutually exclusive options.

## 1. Shared anatomy (applies to every option)

Two work items every option must absorb, independent of scope:

### 1a. TermId seam (P0-11)

Per `06-...md` §2: the evaluator never uses integer sentinels (unbound is
`Option<TermId>`/`GraphMatch`/`SolutionTerm` enums); the seam is `ViewTermId` impl plus
**5 named bridge edges**:

1. `QuadKey::new` / `QuadKey::misses` (`dataset.rs`)
2. `ProbePattern` construction (`bgp.rs:647/716`)
3. `SolutionTerm::join_key` (`scratch.rs:112`)
4. `ViewTermId::encode` (`dataset_view.rs:415`)

Counted as 5 bridge edges across 4 named sites (QuadKey counts twice: new+misses).
Any option that includes sparql-eval pays this exactly; any option that does not
include eval defers it. This is a bounded, ticket-sized item — the seam is not the
reason to shrink scope.

The seam is not the reason to shrink scope: it is 5 edges, not 5 subsystems.

### 1b. Governor binding (P0-12)

Per `06-...` §5: 8-dim ResourceVector, pinned precedence, sticky trip, evidence on the
success path; bind via existing `Admission` (bytes) + `GovernorState` adapter;
**charge deltas not absolutes** (`claim_uncharged_growth` watermark); preserve the
monotone-budget invariant (raising a ceiling must never lose answers). Options A and B
pay this inside the vendored tree; Option C pays it as an adapter crate over the external
dep — same adapter, different tree it points at.

## 2. Options

### Option A — Full vendor (algebra + eval + results [+ conformance])

- **Trees**: all four SPARQL crates vendored into stardf.
- **Size estimate**: ~270K lines admitted at once (35K + 215K + 10K + 9K); plus the
  TermId seam (5 edges) and governor adapter. Estimate class: **PARTIAL_ALIVE — grounded
  in P0-1 counts, the per-crate adaptation cost within the 215K-line eval crate is
  UNKNOWN** (no lane has diffed eval internals against stardf naming/layering).
  UNKNOWN marked explicitly: adaptation-cost-per-line in eval is not grounded.
- **What breaks (per P0-11/P0-12)**: the 5 bridge edges must all land before any eval
  test compiles against stardf dictionary ids; governor binding must land before any
  governed query path is callable; monotone-budget invariant must hold pre-freeze.
- **Gate** ("ONE real integration test" analog): **one real purrdf eval golden class
  ported end-to-end** — a query compiled by vendored algebra, executed by vendored eval
  over a stardf-backed `DatasetView`, results serialized by vendored results, digest-
  compared against the purrdf golden. Exit 0 + mutation court (`--mutate` class) must
  flip the digest court to FAIL; a gate that cannot flip is `VACUOUS GATE`, exit 1
  (house anti-vacuity law, `02-wave-contracts.md` W1.3).
- **License/rename obligations**: full closure — per-file tri-license conjunctions
  (MIT OR Apache-2.0 OR MulanPSL-2.0; `rdf-core` = tri AND Unicode-3.0), copy
  `LICENSES/` + `PROVENANCE.md` structure wholesale, `layers.toml` + `check-layers.py`
  ported alongside, R1 skew row (vs avec P28-SIL) registered before any avec boundary
  crossing (registered in `01-master-plan.md:37`; gate itself is per `04-falsifiers.md:115`,
  status: **PENDING-OPERATOR** on whether the gate lands before any avec crossing).
  Rename `Purrdf→Stardf` across all 4 crates (W1 rename invariant:
  `target == predecessor.replace("Purrdf","Stardf",1)`).

### Option B — Select subsystems: algebra + results WITHOUT eval

- **Trees**: algebra + results vendored; eval NOT vendored — the eval role is served by
  purrdf as an external dependency, or eval is deferred entirely.
- **Size estimate**: ~45K lines vendored (35K + 10K) + seam edges 1–4 minus
  eval-side edges (bridge edges touching `bgp.rs`/`scratch.rs` deferred). The two named
  edge sites that are pure-eval (`bgp.rs:647/716` ProbePattern, `scratch.rs:112`
  join_key) defer with eval; `QuadKey` and `ViewTermId::encode` land with algebra's
  DatasetView consumption. Governor binding: partial — `Admission`/`GovernorState`
  adapter for compile-time limits only; **runtime delta-charging UNKNOWN** without eval's
  success-path hooks (marked UNKNOWN, not guessed).
- **What breaks**: the two crates must agree on a single `TermId`/`ViewTermId` contract
  with the external or absent eval — purrdf's algebra↔eval interface is the load-bearing
  boundary; **whether algebra and results compile standalone against stardf types is
  UNKNOWN** (no lane has compiled algebra or results without eval present — marked
  UNKNOWN, requires a W-lane compile probe before commitment).
  Conformance crate inapplicable (it drives eval); deferred with eval.
- **Gate**: one real algebra golden (a query compiled to vendored algebra output,
  digest-compared) + one real results golden (CanonArrow slab digest per `06-...md` §4).
  Same anti-vacuity: mutation must flip the digest court to FAIL.
- **License/rename**: subset closure — same per-file conjunction rules applied only to
  vendored crates; `LICENSES/` + `PROVENANCE.md` for vendored subset only; rename on
  vendored subset only. **Hybrid renaming hazard (PENDING-OPERATOR)**: keeping purrdf
  as external dep alongside vendored `stardf-*` algebra/results means two names for
  shared types (`PurrdfTerm` vs `StardfTerm`) in one binary — rename-depth decision
  (full rename vs type alias bridge) PENDING-OPERATOR.
- **Key property**: lowest-cost path that keeps the CanonArrow egress surface alive
  (results is the CanonArrow encoder, `06-...md` §4) without admitting 215K eval lines
  on ungrounded per-line estimates.

### Option C — Facade-only: purrdf as external dep

- **Trees**: nothing vendored; stardf ships a facade crate depending on purrdf crates
  (path or registry), exposing the 21-export query surface (`06-...md` §5).
- **Size estimate**: smallest — facade + `ViewTermId` impl + governor adapter, order
  1–5K lines; the 270K remains external, upstream-tracked. Estimate class: PARTIAL_ALIVE
  (grounded in export count; facade line estimate is a guess band, marked as such).
- **What breaks**: nothing breaks in the vendored sense — the break moves to the
  dependency edge: every purrdf version bump is an ABI-relevant change; the seam
  (5 edges) is paid once but re-verified per upstream bump; governor delta-charging
  must be implemented in the adapter without access to eval internals (watermark
  semantics must be observable from the dep's public surface — **UNKNOWN whether it
  is; requires an eval public-API probe**, marked UNKNOWN).
  **Re-constitution mission skew**: stardf's purpose (per ggen-legacy AGENTS.md) is
  verified reconstitution, not wrapping — Option C arguably does not perform
  reconstitution at all. Flagged, decision is not this lane's to make.
  Machine-local `frozen-duckdb` absolute-path pin (06-...md §7) applies to the
  facade's DuckDB edge too until re-homed (operator register item 5).
  **Largest open risk: R1 license compatibility at the dependency edge without vendoring**
  (01-master-plan.md:37) — under C the entire query surface sits behind an external-dep
  license edge, making the R1 gate non-optional from day one (PENDING-OPERATOR on
  gate-before-any-avec-crossing).
  Highest upstream coupling; contradicts the one-canonical-checkout/no-rewrite
  doctrine least on code volume but most on the authority boundary —
  a registry/path dep is a standing edge the portfolio does not control; contradicts the
  reconstitution mission (AGENTS.md pipeline "construct" stage manufactures, it does not
  wrap. — see §4 Recommendation note.
- **Gate**: one real integration test — a stardf-capi query export exercising the
  facade end-to-end over a real purrdf-built graph, results digest-compared against
  the purrdf golden. Same anti-vacuity law (mutation must flip it).
- **License/rename**: no rename obligations (names stay Purrdf*); license obligation
  is dependency-edge-only (R1 row + per-file conjunction check of the purrdf closure as
  consumed). `check-layers.py` gains a dep-edge rule instead of a vendored-crate rule.

## 3. Gate law (common to all options)

The gate is the "ONE real integration test" analog, stated per option in §2. Common
shape: **one real golden class ported end-to-end through every vendored (or facaded)
layer, digest-compared against the purrdf golden, mutation court must flip it to FAIL
— else `VACUOUS GATE`, exit 1.** A scope option that cannot name its one integration
test is not a plan (per operator register §10.2: states a plan or it isn't one).

## 4. Recommendation (included, not flipped — decision register §10.2 remains OPEN)

Lanewise recommendation: **Option B (algebra + results, eval external/deferred)**,
because:

- results is the CanonArrow egress encoder (06-...md §4: "CanonArrow first for
  query-result egress") — it is the piece avec's host boundary actually needs.
- eval is 80% of the volume (215K of 270K) and the only crate whose per-line adaptation
  cost is ungrounded; deferring it converts the largest UNKNOWN into a later, better-
  grounded ticket.
- the TermId seam and governor adapter are bounded in all options; they do not
  differentiate the options.

Status: **recommendation only — register item 2 stays PENDING-OPERATOR**; this doc does
not flip it. Two UNKNOWNs gate any commitment: (a) compile-standalone probe for
algebra/results without eval; (b) eval public-API probe for governor watermark
observability (decides whether Option C is even feasible). Both are one-lane probes.

## 5. PENDING-OPERATOR rows (explicit register)

| # | Item | Options | Blocking |
|---|---|---|---|
| R1 | Rename-depth for hybrid trees (Purrdf* ext + Stardf* vendored) | B | blocks W-lane start |
| R2 | eval public-API probe: watermark observable? | B, C | C feasibility; B charging |
| R3 | Compile-standalone probe: algebra/results sans eval | B | blocks B commitment |
| R4 | R1 license skew gate placement | A, B, C | per 04-falsifiers.md:115 |
| R5 | eval per-line adaptation cost ungrounded | A | blocks A commitment |
| R6 | frozen-duckdb re-homing (register item 5) | A, B, C | before portable receipts |
| R7 | Mission-skew ruling: is facade-only reconstitution? | C | decides if C is lawful |

Detail:

- **R1** — rename-depth for hybrid trees: `Purrdf*` external + `Stardf*` vendored in one
  binary (Option B). Full rename vs type-alias bridge.
- **R2** — eval public-API probe: is the governor watermark observable from purrdf's
  public surface? Decides Option C feasibility; Option B's runtime-charging path.
- **R3** — compile-standalone probe: do algebra/results build without eval?
- **R4** — R1 license skew gate placement (before any avec crossing?) — per
  `04-falsifiers.md:115`.
- **R5** — full-vendor per-line adaptation cost of eval is ungrounded (no lane has
  diffed eval internals against stardf naming/layering).
- **R6** — frozen-duckdb re-homing (operator register item 5, `06-...md` §10) applies
  to the SQL/DuckDB edge in all options.
- **R7** — mission-skew ruling: does facade-only (C) count as reconstitution under
  ggen-legacy `AGENTS.md`? Decides whether C is a lawful option at all.

## 6. Nonclaims

- Size/test counts are as-of P0-1/P0-4/P0-15 receipts (2026-10-31); this lane did not
  re-count crates on disk. (Note: this doc lives in ggen-legacy's stardf/ tree; the P0
  receipts were grounded in the purrdf/frozen-duckdb trees — the counts are citations,
  not re-measurements.)
- No code landed; no gates exist yet; §2 gates are specifications, not executions.
- No git mutations performed.
- UNKNOWN-marked items (eval per-line cost, compile-standalone, watermark observability)
  are UNKNOWN, not negative findings.
- Recommendation not flipped; register item 2 (06-...md §10.2) remains open.
