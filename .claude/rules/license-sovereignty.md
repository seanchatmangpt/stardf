# license-sovereignty

## Statement

stardf-side code is permissive (MIT / Apache-2.0 / MulanPSL-2.0); the avec
consumer side is P28-SIL sovereign. That skew is policed by the **R1 gate**:
`scripts/verify_stardf_licenses.py` must exit 0 before any merge. The gate
FAILs (exit 1) on any P28-SIL -> MIT/Apache/MulanPSL dependency edge without
a registered compatibility receipt in
`/Users/sac/ggen-legacy/authority/stardf/license-compat.json` — an
unregistered dependency is a loudly-missing FAIL row, never a guessed
classification. Matching is by license-token containment: any arm of an
SPDX OR/AND expression landing in the gated class gates the edge.

Conjunct notice carriage is mandatory: the AND-conjuncts
(Unicode-3.0 / BSD-3-Clause) carry attribution/notice obligations, and
selecting the Apache-2.0 arm of a disjunction carries Apache-2.0 §4
carriage — retain the license text and all attribution notices (blake3:
`LICENSE_A2` + `LICENSE_A2LLVM`; `LICENSE_CC0` optional under the Apache
arm). Carriage is witnessed as NOTICE carriage 22/22 plus conjunct notice
dirs (`evidence/g08-license-carriage-verify-2026-10-31.json`); license
texts ride verbatim per `vendoring-law.md`.

New dependency edge: register the compatibility receipt in
`license-compat.json` FIRST, then re-run the gate. The register is the only
license registry; it lives outside this rules lane — edit it under its own
lane's authority, not by drive-by.

## Why

- `stardf/docs/jira/v26.10.31/04-falsifiers.md` — R1 risk row: purrdf
  MIT/Apache/MulanPSL vs avec P28-SIL sovereign; falsifier = dependency-edge
  gate FAILing without a registered compatibility receipt; flip history
  witnessed (exit 0 -> FAIL -> PASS cycles, loudly-missing class).
- `stardf/arrow/scripts/verify_stardf_licenses.py` — the gate itself
  (docstring: falsifier P0-18/P0-19 R1; Chicago-school real manifest reads;
  absent inputs are loud [SKIP] + nonzero, never silent passes).
- `/Users/sac/ggen-legacy/authority/stardf/license-compat.json` — the only
  license registry; entries carry permissive-classification rationale,
  notice_carriage_obligations (Apache-arm selection, LICENSE_A2 +
  LICENSE_A2LLVM carriage), carriage disclosures.
- `stardf/arrow/evidence/g08-license-carriage-verify-2026-10-31.json` —
  R1 exit 0; 66/66 census with non-vacuity control; NOTICE 22/22; conjunct
  dirs 3; provenance rows 22/22; 4 real tarballs.

## How to apply

```sh
python3 /Users/sac/ggen-legacy/stardf/arrow/scripts/verify_stardf_licenses.py
# exit 0 required before merge; exit 1 = unregistered gated edge
```

On exit 1: add the compatibility receipt row to
`/Users/sac/ggen-legacy/authority/stardf/license-compat.json` (with
classification rationale + notice-carriage obligations), then re-run to
exit 0. Do not relax the gate; do not guess a classification the register
does not carry.
