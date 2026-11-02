# D12–D13 Dispatch Brief (v26.10.31, Lane 12)

Status: PENDING-OPERATOR (both decisions). Lane 12, 2026-10-31.
Scope: operator dispatch brief only — analysis of landed W-lane evidence; no code
landed, no git mutations. Owns only this file.

## Sources (cited, not re-derived)

- `stardf/docs/jira/v26.10.31/06-phase0-columnar-sql-duckdb.md` — §1 sizes/tests, §2 seam, §4 CanonArrow, §10 register
- `stardf/docs/jira/v26.10.31/09-sql-adaptation-scope.md` — Options A/B/C, gate law §3, recommendation §4
- `stardf/docs/jira/v26.10.31/10-duckdb-rehome-plan.md` — ground truth §0, court §1, option (c) §4
- W1-7 / W1-9 probe verdicts: **operator-brief-delivered, not on disk.** No W1-7/W1-9
  receipt file was found under `stardf/arrow/evidence/` or `stardf/docs/jira/v26.10.31/`
  at write time (grep `W1-7|W1-9` across both trees: zero matches). Verdicts below are
  as-delivered by the coordinator brief; this lane did not independently re-witness them.

## D12 — SQL dispatch (SPARQL stack admission scope)

### W1-7 probe verdict (as-delivered, not re-witnessed)

- **sparql-algebra**: compiles standalone TODAY, zero adaptation, by-design closure.
  This resolves the 09 §2 Option B UNKNOWN ("whether algebra and results compile
  standalone against stardf types") for the algebra half only.
- **sparql-results**: deeply coupled — ~25 core symbols across 13/13 files. The
  two-crate Option B pair as written in 09 §2 is not viable as-is; it survives as a
  slice (below), not a pair.

### Option B as a slice (amended scope)

The slice = term model + sink + csv + xml_escape + blank_label + vocab +
`SparqlResult` + dataset builder, plus 3 extra leaf crates (branchless/deflate/events).
~45K estimate — **UNVERIFIED at slice granularity** (the ~45K figure descends from the
§1 whole-crate counts, not a per-slice count; no lane has counted the slice boundary).
Governor binding: partial — compile-time limits via the `Admission`/`GovernorState`
adapter only; runtime delta-charging remains UNKNOWN without eval (unchanged from
09 §2).

### What Option B admission unblocks

1. **CanonArrow egress surface** — results is the CanonArrow encoder (06 §4); the
   slice carries `SparqlResult` + sink so the egress court has a real subject.
   Without admission, the egress court is blocked on D12.
2. **QuadKey / ViewTermId::encode seam edges** (edges 1 + 4 of the 5-edge seam, 09 §1a) —
   these land with algebra's DatasetView consumption; the other 3 edges defer with eval.
2. **Rename-depth decision becomes concrete** (09 §2 Option B hazard, PENDING-OPERATOR):
   with results' ~25 symbols across 13/13 files now measured, the full-rename vs
   type-alias-bridge choice has a real symbol inventory to price. It stays
   PENDING-OPERATOR.
4. **Governor compile-limit adapter** gets a real vendored subject to bind against.
5. Does NOT unblock: sparql-eval, sparql-conformance (defer with eval); runtime
   delta-charging; conformance courts.

### Recommendation (included, not flipped — register item 2 stays OPEN)

Amended Option B slice, per the W1-7 probe. The 09 §4 lanewise recommendation
(Option B) stands with the slice amendment; the two-crate form is refuted by the
results-coupling datum.

## D13 — frozen-duckdb rehome

### W1-9 datum (as-delivered, not re-witnessed)

- **2.0.3 resolves from registry WITHOUT local paths, checksum witnessed** — i.e. a
  registry-only build is real, resolving the §0 version-skew question in favor of
  registry 2.0.3.
- **sys/builder float to 2.0.4 under a `=2.0.3` pin** — pin ALL THREE crates with `=`
  (10 §4 option (c) said "or =2.0.4 if the operator accepts the bump"); the float
  datum upgrades this from caution to required: `=2.0.3` on frozen-duckdb,
  frozen-duckdb-sys, frozen-duckdb builder — all three, exact-pin `=`.

### What D13 admission unblocks

1. **P0-19 R3 rename-court** (10 §1): with `[patch.crates-io]` gone, the rename-both-roots
   court becomes non-vacuous; until then it is vacuous by construction (the patch IS the
   machine-local dependency being falsified).
2. **purrdf-duckdb port** (477 ln / 38 tests, 06 §1) — first port governed by this plan.
3. The 1.0 GB `~/.frozen-duckdb` cache can be re-scoped after the court passes.

### Remaining gap (unchanged, still open)

- **Cache-absent compile still unwitnessed** — no lane has built frozen-duckdb with
  `~/.frozen-duckdb` absent. The registry checksum witnesses resolution, not the
  source-compile path in `frozen-duckdb-sys/build.rs`. Rename-court is the named
  follow-up; until it passes, D13 stands at PENDING-OPERATOR with this gap disclosed.

## Standing summary

| Decision | Standing | Unblocks |
|---|---|---|
| D12 | PENDING-OPERATOR (amended Option B slice recommended) | CanonArrow egress court, seam edges 1+4, rename-depth pricing, governor compile-limit adapter |
| D13 | PENDING-OPERATOR | rename-court non-vacuity, purrdf-duckdb port, cache re-scope |

Nonclaims: this lane landed no code, ran no builds, ran no courts, re-witnessed none of
the W1-7/W1-9 probes; the ~45K slice estimate is UNVERIFIED; runtime delta-charging
UNKNOWN; cache-absent compile UNWITNESSED; the probe verdicts are as-delivered by the
coordinator, receipts absent from disk at write time.