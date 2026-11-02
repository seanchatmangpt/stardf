# AGENTS.md — stardf executable reconstitution

## Authority and exact subject

This file governs `stardf/` inside `seanchatmangpt/ggen-legacy`. Every path in
this file is relative to `stardf/` (this file's directory). Parent-repo
artifacts carry a `../` prefix. The repo-level `../AGENTS.md` is the standing
authority; this file scopes it to stardf and never relaxes it.

- governing repo authority: `../AGENTS.md`
- active executable ticket: `../tickets/GL-STARDF-001.md` (ontology contract)
- concurrent executable ticket: `arrow/tickets/GL-STARDF-002-freeze-expansion.md`
- operator decision register: `arrow/docs/OPERATOR-DECISIONS.md` (D1–D21)
- operator spec of record: `docs/jira/v26.10.31/14-definition-of-done.md`
- authority contract: `../authority/stardf/stardf_contract.ttl`
- upstream vendored from: `~/purrdf` at rev
  `0da0578306983da851a9073337d57e2e2c41ab3a` (cited "0da05783")
- publication boundary: draft pull request unless merge is explicitly
  authorized; live crates.io publish behind the operator double gate (below)

The stardf tree began as an admitted vendoring + reconstitution plan. Nothing
executable is admitted except by a deterministic ticket naming authority,
exact subject, dependencies, observable behavior, witnesses, falsifiers,
verification, receipt, replay, and exclusions.

## Mission

Reconstitute the purrdf semantic substrate as the stardf one-package
workspace, prove parity only where a court exists, and preserve evidence
sufficient for replay.

```text
observe → align → admit/refuse → construct
→ execute → verify → receipt → replay
```

Per-target instantiation:

- arrow: vendored core/capi → frozen C ABI → anti-vacuity + stream courts →
  `nm -gU` export diff → receipt → replay
- duckdb: five-table columnar profile → `stardf_scan()` VTab → scan courts
  C1–C6 + columnar oracle → receipt → replay
- sql: SQL-in-RDF compiler → 4 golden courts + compiler doctest → receipt →
  replay
- n3: parse/proof/reason surface → forgery courts + conformance corpus →
  receipt → replay

## Foundational order

1. Preserve the vendored upstream identity (rev, bytes, licenses).
2. Fence unadmitted export-surface and provenance transitions.
3. Define objects, morphisms, authority, actuation, receipt, replay.
4. State exclusions (D17 scope, D18 pending).
5. Name falsifiers against the exact subject.
6. Preserve reversible lawful extension points.
7. Bind implementation to executable acceptance.

## Absolute invariants

### Zero unreceipted actuation

No ambient shell, Git, package-manager, network, deployment, or durable
filesystem-write authority beyond what an admitted ticket grants. A lane
performs no git operations — the coordinator owns git transitions and commits
per lane (`~/.claude/rules/same-checkout-fanout.md`). Publication boundary is
a draft pull request unless merge is explicitly authorized.

### Observation is not admission

```text
O  = partial or stale observation
O* = admitted, aligned, grounded, bounded observation
A  = μ(O*)
R  = receipt(A)
```

Track observed, admitted, executed, changed, verified, inferred, refused,
blocked, and unsupported separately. Source inspection is not execution;
`--help` output is not a run.

### Vendoring law

Upstream is `~/purrdf` at rev `0da05783`, materialized by `git archive <rev>`
— never the dirty working tree. The extraction is copy-and-rename:
`[package] name` and `[dependencies]` keys in `Cargo.toml` only. Lib names,
module tree, symbols, file contents, and every license/notice surface stay
byte-verbatim (SPDX headers included, per-crate `licenses/` dirs, and the
AND-conjunction notices that travel with the files carrying them). Per-crate
`git archive` sha256 receipts live in `arrow/crates/PROVENANCE-entries.md`
plus per-crate `VENDOR-PROVENANCE.md` files.

**Falsifier:** any source file inside a vendored crate requiring a non-rename
edit refutes the rename-scope claim. Escalate to a new ticket — never fix
forward silently. (D17 ladder repairs below are disclosed, not silent.)

### ABI freeze law

The frozen C ABI is enforced, not descriptive. The built dylib's enforced
contract is the **v1 13-symbol surface** (13 `stardf_*` exports; the
transitive `_blake3_compress_in_place_portable` is a declared hygiene item).
The v26.11.1 v2 table in `arrow/docs/ABI-FREEZE.md` is the operator spec of
record but is **PENDING-OPERATOR (D18)** — not adopted this campaign; arrow
stays landed as-is.

- Enforcement is a mechanical `nm -gU` diff of the built dylib against the
  freeze doc's export table: exit 1 on any symbol present but unratified and
  on any ratified symbol absent. A gate that cannot exit 1 is VACUOUS.
- D9 Tier A is ratified (`stardf_query`, `stardf_query_governed`,
  `stardf_query_governors_init` + carriers ResultKind / QueryGovernors /
  GovernorEvidence / RowCursor). These land as C exports ONLY through the
  GL-STARDF-002 §5/§6 process: pre-landing gate + `nm -gU` diff receipt.
  Until one lands through that process, no query/UPDATE/serialize
  `#[no_mangle]` may appear in `arrow/crates/stardf-arrow-capi/src/**`.
- **The freeze law:** any change to the export surface — adding, removing,
  renaming, or re-signaturing any enforced export, or any change to struct
  layouts or status-code values — requires a new ticket. No drive-by edits to
  `lib.rs`, `stream_c.rs`, or `include/stardf_arrow.h`. Header mirrors Rust:
  edit Rust, then mirror into the header. Editing a freeze-doc table row to
  match the dylib is itself a freeze violation.

### Provenance is append-only

`arrow/crates/PROVENANCE-entries.md`, per-crate `VENDOR-PROVENANCE.md`
files, and everything under `arrow/evidence/` are receipts of record. Append;
never rewrite to make a claim read clean.

### Generated projections are not hand-edited

`stardf-arrow`'s ggen route (`arrow/ggen.toml`) is admitted planning only —
no mutating `ggen sync run` has been executed against this tree (D4: dry-run
only until the `stardf:` → `rust:` bridge query lands and its gate non-vacuously
sees rows). Until then the workspace is hand-maintained; do not hand-promote
generated output into semantic source.

## Typed states

Use `UNKNOWN`, `PARTIAL_ALIVE`, `ALIVE`, `BLOCKED`, `BUILD_BROKEN`,
`UNSUPPORTED`, and `REFUSED:<CODE>` precisely. No toolchain means
`BLOCKED:TOOLCHAIN_UNAVAILABLE`, not `BUILD_BROKEN`. A compile abort is not a
failure count — it is a census of zero executed tests plus the broken file.
Statuses cite command + exit + date; standing recalled from an earlier read
is stale by default.

## Authority precedence

1. `../AGENTS.md` (repo authority)
2. this file
3. admitted tickets (`../tickets/GL-STARDF-001.md`,
   `arrow/tickets/GL-STARDF-002-freeze-expansion.md`)
4. operator decisions and spec of record
   (`arrow/docs/OPERATOR-DECISIONS.md`,
   `docs/jira/v26.10.31/14-definition-of-done.md`)
5. authority contract (`../authority/stardf/stardf_contract.ttl`) and
   verifiers (`../scripts/verify_stardf_ontology.py`)
6. the ABI freeze doc (`arrow/docs/ABI-FREEZE.md`) as the enforced C-ABI
   contract
7. executable source and fixtures
8. PRD/ARD (`arrow/docs/PRD.md`, `arrow/docs/ARD.md`) and the planning pack
   (`docs/jira/v26.10.31/`)
9. explanatory documentation and generated reports
10. assumptions

Contradiction returns `BLOCKED:AUTHORITY_CONTRADICTION`; resolve by reading
both sides, never by destructive heuristics.

## Testing discipline — Chicago, no mocks, no sleep

Real collaborators, real subprocesses, real bytes; assert on final state.
Mocking an owned/locally-runnable collaborator is banned by default
(`unittest.mock`/`Mock()`/`patch`/`monkeypatch`, Rust `mockall`, Jest
`jest.mock()`). The named exception (infeasible real collaborator) stays a
simple fake, stated in the receipt — never a mock.

Time is not a proxy for state. `Process.sleep` is banned as a test
coordination step and unannotated in `lib/`; use bounded polls on real
observables or structural sync. Production sleeps are scheduling only.

A loud `[SKIP]` with its reason is lawful; a faked pass is not. A gate that
cannot exit 1 is VACUOUS and fails the court. Mutation flips prove
non-vacuity: `python3 arrow/scripts/verify.py --mutate drop-export` must exit
1 (witnessed), and the scan court's mutant flip is part of its pass.

Standing courts on this tree (paths verified):

- `arrow/crates/stardf-arrow/tests/anti_vacuity.rs` — fixture courts
- `arrow/crates/stardf-arrow-capi/tests/stream_courts.rs` — C stream ABI
- `arrow/crates/stardf-scan-duckdb` — scan courts C1–C6 (C3 loud SKIP by
  design)
- `arrow/crates/stardf-sql/tests/sql_golden.rs` — 4 golden courts + doctest
- `arrow/crates/stardf-n3` — forgery courts + N3 conformance corpus
  (`arrow/corpora/n3/`)
- `arrow/crates/stardf-columnar/tests/golden_bytes.rs` — pinned digests
- `arrow/scripts/verify.py` (+ `--mutate`), `arrow/scripts/verify_header_freeze.py`,
  `arrow/scripts/verify_stardf_licenses.py`,
  `arrow/scripts/verify_blake3_identity.sh`,
  `arrow/scripts/verify_duckdb_rehome.sh` (two-arm cache-absent),
  `arrow/scripts/check-columnar-oracle.sh` (loud skip, exit 3, when duckdb
  is absent)

## Verification ladder

```text
just ci-all                        (fmt → clippy → check → test, workspace)
→ python3 arrow/scripts/verify.py  (13==13==13 export parity; --mutate flips)
→ python3 arrow/scripts/verify_header_freeze.py
→ python3 arrow/scripts/verify_stardf_licenses.py
→ bash arrow/scripts/verify_blake3_identity.sh
→ per-target courts (anti_vacuity, stream_courts, scan_courts,
   sql_golden, n3 forgery, columnar golden_bytes)
→ publish ladder dry-run (below)
```

Hosted CI is not a substitute for local execution.
`arrow/ci/stardf-arrow-reconstitution.yml` is a DRAFT that has never been
executed (D2 placement PENDING-OPERATOR); a workflow file is not a run.

## Publish law

Footprint is **first-class closure only** (D17): everything outside the
closure stays internal path-only. The ladder is `arrow/scripts/publish_stardf.py`
(gates G1–G9 + R1), dry-run by default (`cargo package --list`).

- Live publish requires BOTH `--publish` AND
  `STARDF_PUBLISH_CONFIRM=ratified-v26.11.1` — the operator double gate.
  Neither alone actuates.
- Fail-closed on any non-green gate. A SKIP gate row is not a PASS; the
  ladder refuses on SKIP rows unless `--accept-skips` is explicit.
- crates.io law for every published crate: no machine-local path overrides;
  `version`+`path` dual-key dependency rows at the exact publish version;
  unified metadata; exclude filters keep corpora/testdata under the 10 MB
  tarball ceiling; license files physically in the tarball; frozen-duckdb /
  frozen-duckdb-sys / frozen-duckdb-builder exact-pinned `=2.0.4` (D19).
- Current verdict: **BLOCKED** (G8 nonclosure hard deps) pending the operator
  D17 closure re-ratification (PENDING-OPERATOR,
  `arrow/docs/OPERATOR-DECISIONS.md`, H03 registration). No live publish has
  occurred; the registry is untouched.

## Four first-class targets — the only parity scope (D17)

The copied ~80% of the workspace carries **NO courts** (D17, honest scope):
compile-green at best. Parity claims are scoped to exactly:

| Target | Scope | Standing (as recorded in `arrow/README.md`, 2026-10-31 truth pass) |
|---|---|---|
| arrow | Quad64 arena, C Data Interface export, frozen ABI, columnar codec | ALIVE (local) |
| duckdb | in-process `stardf_scan()` VTab over the columnar profile | ALIVE (local) |
| sql | SQL-in-RDF compiler + 4 golden courts + 1 doctest | ALIVE (local, run-witnessed) |
| n3 | N3 parse/proof/reason + forgery courts + conformance corpus | ALIVE (local, run-witnessed) |

All standing is LOCAL. Copied-crate breakage surfacing in the ladder:
feature-gate or fix forward, whichever is faster, with the divergence
disclosed in a receipt (never silent — see the vendoring falsifier).
Full parity is DEFERRED to a later campaign
(`docs/jira/v26.10.31/12-full-parity-campaign.md`,
`workflows/v26.11.1-full-parity.workflow.js`).

## Authored boundaries

### GL-STARDF-001 (ontology contract; `../tickets/GL-STARDF-001.md`)

Authority contract + verifiers at parent level: `../authority/stardf/`,
`../scripts/extract_purrdf_ontology.py`,
`../scripts/verify_stardf_ontology.py`,
`../scripts/verify_marketplace_packs.py`, plus the planning pack
`docs/jira/v26.10.31/` and the vendoring plan
`docs/jira/v26.10.31/08-vendoring-plan.md`.

### GL-STARDF-002 (ABI freeze expansion; `arrow/tickets/GL-STARDF-002-freeze-expansion.md`)

Enumerates the 21-export query/UPDATE/serialize option set and its carrier
drag; ratification lives in `arrow/docs/OPERATOR-DECISIONS.md` (D9 Tier A)
and `arrow/docs/ABI-FREEZE.md` — the ticket admits no exports by itself.

## Repair law

Preserve the exact failure, classify the failed transition, repair the
earliest lawful cause, add a permanent guard, rerun that boundary, expand
only after success. Never rerun an unchanged failure without a new
hypothesis.

## Publication safety

Resolve exact base and head before writing. Lanes never run git commands;
the coordinator owns transitions and per-lane atomic commits. Preserve
unrelated work. Non-force updates only. Do not merge unless explicitly
requested.

## Required receipt

Every consequential action lands a receipt against schema
`ggen.legacy.stardf.arrow.receipt.v1` (template:
`arrow/evidence/RECEIPT_TEMPLATE.json`; conventions:
`arrow/evidence/README.md`). Report: subject `tree_digest`/`head` computed
at receipt time (not before), every command with its real exit code, checks
with counts (a failing check stays in `checks[]` with `passed=false` and
also gets an `errors[]` row — never omitted to read clean), replay status
(`NOT_RUN` until a byte-identical rerun is witnessed), standing from the
typed vocabulary, and nonclaims. Report file paths changed and scoped
standing; never claim planned or inspected work as executed.

## See Also

- `../AGENTS.md` — repo-level executable-reconstruction authority
- `arrow/README.md` — standing table, quickstart, one-package state
- `arrow/docs/ABI-FREEZE.md` — frozen C ABI + enforcement law
- `arrow/docs/OPERATOR-DECISIONS.md` — D1–D21 register
- `arrow/docs/DIVERGENCES.md` — KD-row ledger
- `docs/jira/v26.10.31/` — planning pack (01–14)
- `arrow/docs/RECEIPTS.md`, `arrow/evidence/` — receipts of record
