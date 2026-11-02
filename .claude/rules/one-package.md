# one-package

## Statement

stardf is ONE package (D15). Every purrdf crate is straight-copied into the
single `stardf/arrow` workspace; internal edges (sql -> shapes, etc.) are
ordinary workspace edges. No carve, no feature-gating by dependency.

One root: `stardf/arrow` survives; `stardf/crates/` stubs are archived
(D16 phase 2). Phase 1 = the scripted bulk copy
(`scripts/bulk_copy_purrdf.py`); phase 2 = mechanical rename + single root +
`cargo check` loop until green; phase 3 = the four first-class targets only:
arrow (landed), duckdb critical path, sql whole + 4 goldens, n3 whole +
suite + discovery.

The bar for non-first-class crates is **compile-green only**. The copied
~80% carries NO courts (D17): compile-green at best; the parity claim is
scoped to the four first-class targets. Copied-crate breakage surfacing in
the ladder: feature-gate or fix forward, whichever is faster.

## Why

- `stardf/arrow/docs/OPERATOR-DECISIONS.md` — the 80/20 re-scope block:
  D15 (one-package rule), D16 (80/20 phase ladder), D17 (honest scope:
  copied ~80% no courts, compile-green at best, parity scoped to four
  first-class targets). Also D14/D19-D21 for the publish-context
  consequences (closure crates, dual-key pins).
- `stardf/docs/jira/v26.10.31/12-full-parity-campaign.md` — STATUS +
  ADDENDUM banners: the W1-W10 per-receipt parity campaign is DEFERRED/
  superseded by D15/D16; do not dispatch it without operator re-admission;
  a future parity campaign starts from copy-fidelity courts, and only
  surviving deltas earn parity work.
- `stardf/arrow/docs/RECEIPTS.md` — campaign-final nonclaims: copied ~80%
  carries NO courts (D17); DIVERGENCES KD-01..15 (8 OPEN / 7 PARTIAL)
  deferred to the full-parity campaign.

## How to apply

Compile-green is the gate; from `stardf/arrow`:

```sh
cargo check --workspace          # the bar: exit 0 (G9)
```

Do not add courts to copied crates — that scope belongs to a future
operator-admitted parity campaign, not to drive-by ambition. A new upstream
crate arrives by straight copy per `stardf/.claude/rules/vendoring-law.md`,
not by carve. Adding a workspace edge is an ordinary manifest edit; carving
a crate out, or gating one behind a feature to dodge compile-green, is a
D15 violation.
