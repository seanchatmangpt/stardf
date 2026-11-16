# DISPOSITION — `crates/jsonschema` (excluded-surface fixture tree)

Registered disposition for the excluded-surface fixture tree at
`stardf/arrow/crates/jsonschema/` (receipted 2026-10-31, full-parity campaign
wave-2 lane w2-l15, task: "jsonschema disposition receipted").

## Disposition

- **The crate is EXCLUDED, loudly.** `purrdf-jsonschema` (upstream full crate:
  src/benches/licenses, 383 files at the pinned rev) is in the campaign's
  exclude-loudly bucket — `stardf/docs/jira/v26.10.31/12-full-parity-campaign.md`
  standing budget: "Exclude loudly: ... jsonschema/..." (line 105).
- **What is carried is fixture data, not the crate.** The only subtree present
  is `tests/metaschemas/`: 17 verbatim JSON Schema meta-schema documents
  (draft-07 x1, draft2019-09 x7, draft2020-12 x9) plus their provenance files.
  No `Cargo.toml`, no `src/`, not a workspace member (`stardf/arrow/Cargo.toml`
  members list names no `crates/jsonschema` entry) — the tree cannot compile
  and is consumed only via `include_str!`.
- **Sole consumer**: `crates/stardf-testkit/src/jsonschema_metaschemas.rs:19`
  — `include_str!(concat!("../../jsonschema/tests/metaschemas/", $path))`.
  The macro reaches outside the vendored testkit crate into this tree; the
  17 include_str! call sites (DRAFT_2020_12 = 9, DRAFT_2019_09 = 7,
  DRAFT_07 = 1) resolve 17/17 against the carried files.

## Provenance

- **Source**: purrdf rev `0da0578306983da851a9073337d57e2e2c41ab3a`, via
  `git -C /Users/sac/purrdf archive <rev> -- crates/jsonschema/tests/metaschemas/`
  (never the dirty tree), per 08-vendoring-plan.md.
- **Upstream origin**: json-schema-org/json-schema-spec — draft2020-12 branch
  commit `601a66c8b0f25246bf0e1fb488c5b5f030a79b72` (schema.json + 8 vocabulary
  meta-schemas); draft2019-09 and draft-07 branches (upstream commits not
  recorded upstream; content pinned by the frozen manifest below). Cited from
  upstream `crates/jsonschema/PROVENANCE.md` §`tests/metaschemas/` at the
  pinned rev.
- **License**: BSD-3-Clause, "Copyright (c) 2022 JSON Schema Specification
  Authors" — taken under BSD-3-Clause per upstream REUSE.toml;
  `LICENSES/BSD-3-Clause.txt` is the upstream BSD section verbatim.
- **Files carried** (20): the 17 JSON documents,
  `REUSE.toml`, `LICENSES/BSD-3-Clause.txt` (both byte-verbatim from the
  pinned rev), and `jsonschema-metaschemas.sha256` (byte-verbatim copy of the
  upstream frozen manifest `scripts/conformance-frozen/jsonschema-metaschemas.sha256`,
  relocated into the directory it indexes so `shasum -c` runs in place).

## Standing receipts (this lane, 2026-10-31)

1. Byte-parity: `cmp` of each of the 17 JSON files against the `git archive`
   extraction of the pinned rev — 17/17 BYTE-EQUAL, 0 diffs.
2. Frozen manifest: `shasum -a 256 -c jsonschema-metaschemas.sha256` in
   `tests/metaschemas/` — 17/17 OK, exit 0.
3. Anti-vacuity positive control (F4b pattern): one-byte mutation of
   `draft-07/schema.json` in a temp copy makes the same check FAIL — exit 1,
   `draft-07/schema.json: FAILED`. The court can flip; a court that cannot
   flip is FAIL.
4. Compile gate: `cargo check -p stardf-testkit` exit 0 — the include_str!
   seam is green against the receipted tree (see receipt below).

## Flip conditions

- The exclusion flips to VENDORED only by an operator Tier-C decision
  (12-full-parity-campaign.md W9: "jsonschema/text/markdown/mime/hnsw/
  retrieval (unless Tier C+)"); this receipt does not admit the crate.
- Any byte change under `tests/metaschemas/` must flip the frozen-manifest
  check; a change that does not flip it means the manifest is not indexing
  the tree (vacuous) and this receipt is void.

## Nonclaims

- The metaschema files were placed on disk by a concurrent lane (observed,
  FALSIFIER-RECEIPT.md §Resolution records the same observation); THIS lane's
  receipts are the byte-parity + manifest + positive-control runs above, not
  the original placement.
- The crate itself (src/benches, 383 upstream files) is not vendored, not
  compiled, and not claimed.
- The upstream crate's other test trees (tests/suite, differential vectors)
  and src/ are NOT carried and are NOT receipted here.
- The REUSE.toml coverage paths are relative (`draft2020-12/**/*.json`, ...);
  they resolve against this directory's layout unchanged — no edit was made
  or needed (a non-rename edit needed to build would be a falsifier; none
  was required).
- `authority/stardf/license-compat.json` was observed NOT parseable (strict
  JSON error, control character at line 514) on this lane's first R1 run
  (exit 1), with an mtime seconds old — a concurrent lane's in-flight edit;
  this lane did not modify it. The lane's second R1 run, minutes later,
  PASSED (exit 0) — the in-flight edit settled between runs. Timeline
  receipted in full; no gap filled by assumption.

## See Also

- `../stardf-testkit/FALSIFIER-RECEIPT.md` — the cross-crate include_str!
  escalation this tree resolves
- `../../docs/jira/v26.10.31/12-full-parity-campaign.md` — the exclusion line
- Upstream `crates/jsonschema/PROVENANCE.md` at purrdf rev 0da05783 —
  metaschema origin + license basis
- `authority/stardf/license-inventory.toml` — the BSD-3 carriage census row
