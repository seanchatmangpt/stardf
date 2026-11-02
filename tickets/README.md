# stardf tickets

Index of the `GL-*` tickets governing the stardf reconstitution of `purrdf`.
This file is an index only: it admits nothing, ratifies nothing, and amends
neither ticket. Statuses are as-of disk reads dated below. Lanes owning
tickets edit the ticket files; this index is updated to match, never the
reverse authority.

- Read: 2026-10-31/11-01 (lane E, docs-only)
- No git operations performed by this lane

## Index

| Ticket | File | Status |
|---|---|---|
| GL-STARDF-001 | [`GL-STARDF-001.md`][t1] | scope closed |
| GL-STARDF-002 | [`GL-STARDF-002-freeze-expansion.md`][t2] | Tier A (D9); ABI-v2 pending (D18) |

[t1]: ../../tickets/GL-STARDF-001.md
[t2]: ../arrow/tickets/GL-STARDF-002-freeze-expansion.md

## GL-STARDF-001 — scope closed

Author the exhaustive stardf ontology matching `purrdf`
(`stardf_contract.ttl`), then reconstitute against it. Scope is closed as of
the v26.11.1 campaign (2026-10-31).

Landed, in the one-package workspace `stardf/arrow` (D15 one-package law,
D16 phase-3 first-class targets):

- arrow + columnar + sql + n3 + duckdb-scan implementations on disk
- census + ABI contract + verifiers in c2 = `98afd71`
  ("feat(stardf): authority contract + verifiers, census parity 22/22");
  commit chain c1 `26d0140` - c4r `724d5425` per
  [`../arrow/docs/COMMIT-PLAN.md`](../arrow/docs/COMMIT-PLAN.md) and the
  final receipt's `c_chain`

Standing boundaries per the final receipt
(`../arrow/evidence/v26.11.1-manufacturing-receipt.json`, standing
PARTIAL_ALIVE) — landed is not the same as courts-run:

- arrow: PARTIAL_ALIVE (courts green locally; no CI standing)
- columnar: courts witnessed (golden_bytes, integration, full suite)
- duckdb-scan: courts first-ran green, receipted in
  `g10-scan-courts-rewitness-2026-10-31.json` (12/0, exit 0)
- n3: first post-copy suite run green (89/0, exit 0) — log-standing; the
  evidence receipt file is still owed
- sql: UNKNOWN-as-receipt (no run, no log, no binary on any post-copy tree)

## GL-STARDF-002 — Tier A ratified per D9; ABI-v2 PENDING-OPERATOR per D18

ABI freeze expansion: the query/SQL export surface beyond the frozen
13-export v1 ABI (`../arrow/docs/ABI-FREEZE.md`, v26.9.29).

- Tier A RATIFIED by operator sign-off 2026-10-31 (D9,
  `../arrow/docs/OPERATOR-DECISIONS.md` sign-off block):
  `stardf_query` + `stardf_query_governed` + `stardf_query_governors_init`,
  carriers ResultKind / QueryGovernors / GovernorEvidence / RowCursor.
- Enforcement law (ticket §5) holds either way: `nm -gU` diff of the built
  dylib against the ratified export table must exit 1 on any extra or
  missing symbol; a gate that cannot exit 1 is VACUOUS.
- ABI-v2 (the DoD's revised 13-export surface): PENDING-OPERATOR adoption
  per D18 — NOT adopted this campaign; arrow stays landed as v1
  13-symbol. Adoption is a later operator call with the
  cross-allocator-court migration cost attached.

## Admission rule for new tickets

Ticket-gated admission, per [`../../AGENTS.md`](../../AGENTS.md):
executable source is admitted only by a deterministic ticket naming
authority, exact dependencies, observable behavior, witnesses, falsifiers,
verification, receipts, replay, and exclusions.

- every capability change names its `GL-*` ticket
- check the `AGENTS.md` header before starting executable work — it names
  the active and concurrent executable tickets for the session
- nothing executable is admitted by this index; a new capability without a
  ticket is refused, and a ticket ratifies nothing by itself (ratification
  runs through the operator decision register or its named authority)

## Scope-closure evidence

All receipts verified to exist on disk under
[`../arrow/evidence/`](../arrow/evidence/) at the read date:

- `v26.11.1-manufacturing-receipt.json` — final finish-campaign receipt;
  standing PARTIAL_ALIVE; commit chain c1-c4r; four-target standings;
  publish BLOCKED landing list
- `w1-closure-2026-10-31.json` — W1 closure: per-lane standing census of
  the wave-era tree
- `g04-falsifier-finalization-2026-10-31.json` — avec consumer court
  3/3 natively (wrap posture) plus F1/F1a/F2/F4/F4b/F5 re-witnesses
- `g10-scan-courts-rewitness-2026-10-31.json` — scan courts receipted:
  `cargo test -p stardf-scan-duckdb --locked` exit 0, 12/0
- `h03-gate-closure-receipt-2026-10-31.json` — G3+G5 closed
  fix-forward; G8 registered owner-blocked (D17 closure membership)
- `g07-tarball-audit-2026-10-31.json` — 4 real tarballs, license shas
  canonical, replay byte-identical
- `g08-license-carriage-verify-2026-10-31.json` — R1 exit 0, 66/66
  license census with a mutation non-vacuity control
- `lane16-falsifier-ledger-2026-10-31.json` — header-freeze court 4/4
  shims flip; verify.py gate+flip+replay pair byte-identical
- `publish_stardf-receipt-20261101T051252Z.json` — newest publish-ladder
  receipt: BLOCKED at G8 only (G1-G7, R1, avec-ffi court, G9 all PASS)
- further wave-era receipts: `manufacturing-receipt-2026-10-31.json`,
  `wave-consolidated-2026-10-31.json`, `columnar-wave-2026-10-31.json`,
  `finish-wave-2026-10-31.json`, `wave-finish-lane16-2026-10-31.json`,
  `replay-2026-10-31.json`, `lane07-publish-metadata-2026-10-31.json`,
  `lane11-license-carriage-2026-10-31.json`,
  `lane11-license-carriage-r2-2026-10-31.json`, and the
  `publish_stardf-receipt-20261101T*.json` ladder series

## Nonclaims

- This index admits no capability and amends neither ticket file; the
  ticket files and `OPERATOR-DECISIONS.md` are the authority this file
  cites.
- The c2 SHA is cited from `COMMIT-PLAN.md` and the receipt's `c_chain`,
  not re-derived by git (this lane ran no git commands).
- The n3 evidence receipt file and the sql court runs remain owed; do not
  read "scope closed" as "every target receipted" — see the standing
  boundaries above.
- Publish standing is BLOCKED (G8) in the newest receipt; that receipt
  postdates the final manufacturing receipt, so it is the current verdict
  of record for the ladder.

## See Also

- [`GL-STARDF-001.md`][t1]
- [`GL-STARDF-002-freeze-expansion.md`][t2]
- [`../arrow/docs/OPERATOR-DECISIONS.md`](../arrow/docs/OPERATOR-DECISIONS.md) — D9/D18 sign-off
- [`../arrow/docs/COMMIT-PLAN.md`](../arrow/docs/COMMIT-PLAN.md) — commit chain incl. c2
- [`../arrow/docs/ABI-FREEZE.md`](../arrow/docs/ABI-FREEZE.md) — frozen v1 13-export surface
- [`../../AGENTS.md`](../../AGENTS.md) — ticket-gated admission authority
