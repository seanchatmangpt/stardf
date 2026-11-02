# R1 Blake3 Receipt Admission — Operator Decision Brief

**Status: PENDING-OPERATOR** (lane 11, 2026-10-31; drafted from W1-5's
`draft_receipts_pending_operator` entry in `authority/stardf/license-compat.json`)
**Decides**: how the `stardf-arrow-core -> blake3` edge clears the R1
license-compatibility gate.

## Current state (witnessed this session)

```
$ python3 stardf/arrow/scripts/verify_stardf_licenses.py
...
[FAIL] edge stardf-arrow-core -> blake3: P28-SIL -> gated license
'CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception' without registered
compatibility receipt (R1)
license-compat gate: SKIP (1 input(s) missing), exit 1 — a skipped input is
never a pass
```

Exit 1, from `stardf/arrow/scripts/verify_stardf_licenses.py`. Two
**independent** exit-1 causes are present in the same run:

1. The FAIL row above (no registered `(stardf-arrow-core, blake3)` receipt in
   `compatibility_receipts`).
2. `[SKIP] license-inventory.toml not found` — the conjunct-notice census
   (Unicode-3.0 / BSD-3-Clause) has never been performed, and per the gate's
   own law "a skipped input is never a pass."

Consequence both arms must respect: **admitting the receipt alone does not
make the gate exit 0** until `license-inventory.toml` also exists. The gate
law requires both.

The draft itself (`draft_receipts_pending_operator[0]`) is NOT a registered
receipt and does not satisfy the gate — the verifier reads only
`compatibility_receipts`. Admission = the operator moving it (post-decision)
into `compatibility_receipts`; this lane has not done so and is not
authorized to.

## The fork

blake3 1.8.5 declares
`CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception` (source:
`~/.cargo/registry/src/index.crates.io-*/blake3-1.8.5/Cargo.toml`, verified
by W1-5 2026-10-31). The gate classifies by SPDX-token containment
(`GATED_LICENSE_TOKENS = MIT / Apache-2.0 / MulanPSL-2.0`): the presence of
the `Apache-2.0` token makes the whole edge **gated**, hence the FAIL.

### Arm A — Apache-2.0-arm admission (drafted path)

**Mechanics**: register the drafted receipt in `compatibility_receipts`
(consumer `stardf-arrow-core`, dependency `blake3` 1.8.5). Under
license-token containment, an OR-expression with an Apache-2.0 arm is
consumable on permissive terms by selecting that arm; the verifier's
`classify_license` stays "gated" and the edge passes via the
`compatibility receipt REGISTERED — pass` row.

**What changes in `verify_stardf_licenses.py` output**: the blake3 FAIL row
becomes `[ROW] edge stardf-arrow-core -> blake3: license=... GATED,
compatibility receipt REGISTERED — pass`. Exit code is then determined
solely by the inventory SKIP (see above) — green on the blake3 edge, but the
gate still exits 1 until the conjunct census lands.

**Carriage obligation** (the cost of selecting the Apache arm — Apache-2.0
§4): the binary/source distribution must ship
`LICENSE_A2` (Apache-2.0 full text) **and** `LICENSE_A2LLVM` (Apache-2.0 with
the "LLVM Exceptions to the Apache 2.0 License" addendum, verified present at
line 204 of the blake3-1.8.5 checkout), plus blake3's copyright/patent/
trademark notices. `LICENSE_CC0` becomes optional under this arm. Cargo
vendoring carries these automatically; a shipped binary does not.

**Legal posture**: standard permissive consumption. Attribution and notice
text flow downstream; obligations are real but routine.

### Arm B — CC0-arm sovereign-class reclassification

**Mechanics**: instead of registering a receipt, the operator decides the
CC0-1.0 arm is the selected term and changes the `blake3` row's `license`
field in `dependency_licenses` (currently the full OR-expression) to
`CC0-1.0`. `classify_license` then returns `sovereign-class` and the edge
passes with no receipt at all: `[ROW] edge ... license='CC0-1.0'
sovereign-class — pass`.

**What changes in `verify_stardf_licenses.py` output**: the FAIL row is
replaced by a sovereign-class pass row; no entry is added to
`compatibility_receipts`; the gate's receipt machinery is never exercised on
this edge.

**Carriage obligation**: none. CC0 imposes zero attribution/notice
obligations — no license files must ship.

**Legal posture**: different in kind, not degree. CC0 is a public-domain
dedication; consuming under it means the downstream work takes nothing from
the Apache grant and owes nothing back. The draft's own note flags this as a
"legal-strategy decision above lane authority." Cost of this arm: the
register's `license` field would no longer quote the upstream
`Cargo.toml` expression verbatim — the fidelity of the register to observed
upstream truth is traded for the simpler posture, and the receipt/register
divergence should be recorded in the register itself if chosen.

## What does NOT change under either arm

- The `license-inventory.toml` SKIP keeps the gate at exit 1 until the
  conjunct-notice census (Unicode-3.0 / BSD-3-Clause) is performed — a
  separate, already-open work item.
- The avec-arrow edges stay exempt/pass (path-dep / sovereign-fork-posture
  rows, unchanged).
- **W3's avec gate stays BLOCKED** either way until the decision is executed:
  `docs/OPERATOR-DECISIONS.md` §D6 ("Also gating W3": R1 receipt must land
  BEFORE the avec dependency edge is admitted) and
  `stardf/docs/jira/v26.10.31/05-next-steps.md` §2 preconditions both pin
  this. Arm A executes by receipt registration; Arm B by register
  reclassification — but until one lands, W3 cannot admit the edge.
- Separately witnessed open item (not this decision):
  `05-next-steps.md` §3 records the gate's **flip** (exit 0 → in-memory
  receipt removal → exit 1) as not yet demonstrated this session; whichever
  arm lands, the flip witness should be captured on the same run.

## Recommendation

None stated — both arms are lawful; the choice is a legal-strategy posture
call (notice carriage vs. public-domain dedication) that is the operator's
by the draft's own terms.

## Nonclaims

- This lane did NOT move the draft into `compatibility_receipts`, did not
  modify `license-compat.json`, and did not modify the verifier script.
- The gate was observed at exit 1 exactly as quoted; no other invocation
  (e.g. with `--inventory` pointing at a real census file) was attempted, so
  no claim is made about gate behavior once the census exists.
- blake3's `Cargo.toml` license expression and the presence of
  `LICENSE_A2`/`LICENSE_A2LLVM` are cited from W1-5's verification receipt,
  not independently re-verified by this lane.
- No claim that Arm B's register edit (quoting `CC0-1.0` rather than the
  full OR-expression) is the only mechanical form it could take — any
  mechanism producing a sovereign-class classification for the edge has the
  same effect on the verifier output.
- W3's BLOCKED posture is cited from `OPERATOR-DECISIONS.md` D6 and
  `05-next-steps.md`; this lane did not re-verify the avec-side test state.
