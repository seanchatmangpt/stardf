# WORK-BOARD — stardf fold (lane S-7, 2026-11-01)

Fold of sibling lanes S-1/S-3/S-4/S-5/S-6 receipts (docs/). S-2
(build-baseline.md) was ABSENT at poll deadline (45-min bounded poll,
20:15-21:00Z; killed at limit, 0/6 then; landed-later check: still absent
at ~21:02) — named loudly; build rows below cite S-5's sandbox court runs
plus this lane's own in-tree commands.

## (a) Post-move state

- Move ALIVE: baseline fe935f7 (S-6; git log this lane: fe935f7 + de7c9f5).
- Runtime repairs landed (S-1, ALIVE, re-verified on disk this lane:
  grep for ggen-legacy in h03_bootstrap_sim.sh, full-parity workflow.js,
  .claude/settings.json -> 0 hits): h03 script (real run exit 0, 4 OK /
  21 FAIL), workflow.js REPO re-point, settings.json _scope.
- Remaining pinned-path census (this lane, grep census excl.
  target*, .git, rust_out, __pycache__, docs/move-archive): 99 hits /
  38 files. Biggest: README-MOVE-NOTE.md 17, ARD.md 5, MOVE-TO-HOME-READY.md 4, PRD.md 4,
  license-sovereignty.md 4 (stale: register now in-tree), substrate-map:257 1
  (P6 sed NOT witnessed).
- P5 unapplied (witnessed: sed of verify_stardf_licenses.py lines 19-22
  shows `REPO = parents[3]`; gate run -> [SKIP] missing /Users/sac/authority/...,
  exit 1) even though P1 landed (root authority/stardf/ 6 entries present,
  license-compat.json exists, ls exit 0). One-line repair; R1 gate dangles until then.
- build_receipt.py:43 `ARROW_ROOT.parent.parent` still 2-up (same dangling
  resolution) — companion one-line repair.
- Stale duplicate arrow/authority/stardf/ still present (P1 sub-flag:
  delete pending, ERRC).
- P3 landed: scripts/verify_stardf_ontology.py in-tree (this lane ran it,
  below). P6 (PRD/ARD/substrate-map seds) NOT executed (4+5+1 hits live).

## (b) Build/test board + critical path

In-tree `cargo check --workspace` exit 0 (this lane; the concurrent 4-up
cgap path-dep fix held). Per-crate court counts cite S-5 (sandbox) unless noted.

| crate | state | counts |
|---|---|---|
  128 AbiFunction instances, 22 header typedefs vs 11 MISSING type
  mappings (S-3 witnessed the same exit 1). Invariant 4 (census parity) NOT met.
| stardf-columnar | GREEN | golden_bytes 5/0 (flip control flips), seam_courts 4/0 |
| stardf-hash | GREEN | RFC1321 10/0, FIPS180-2 64/0, FIPS202 28/0, CRC32 5/0 |
| stardf-lex | GREEN | 198/12/90, 0 fail |
| stardf-deflate | GREEN (partial) | 7/0; upstream suite parked tests/disabled/ |
| stardf-rs umbrella | GREEN | all targets 0 failed |
| stardf-rdf | GREEN | serialize_sink 3/0, serialize_pack 2/0 |
| stardf-entail | GREEN | pack_view_parity 3/0 |
| stardf-sparql-eval | RED lib target | SIGABRT stack overflow (--lib); witness targets pass |
| stardf-validate | GREEN | dl_nominal_witnesses 2/0 |
| stardf-scan-duckdb | GREEN | scan_courts 6+6/0; c3 loud-SKIP by design (pushdown) |
| stardf-duckdb | GREEN | chicago courts all targets 0 failed |
| stardf-process | UNRUNNABLE | not a member nor excluded; cargo exit 101 workspace-ambiguity |
| n3 | GREEN (log-standing) | 89/0 (tickets/README + manufacturing receipt; no evidence file yet) |
| sql | UNKNOWN-as-receipt | no run, no log, no binary (tickets/README) |

Critical path: (1) R1 license gate (P5 two one-liners) -> (2) stardf-sparql-eval
lib SIGABRT -> (3) stardf-process workspace membership decision -> (4) n3/sql
receipts -> (5) D1 commit split.

## (c) Ticket-001 execution state (S-3 receipt + this lane re-run)

- Ticket file RECOVERED to tickets/GL-STARDF-001.md (S-3).
- Contract digest-bound to live purrdf.h (S-3, shasum exit 0).
- F4 header-freeze court WITNESSED (lane-16 ledger; shims flip; 13==13).
- F1 census FAILS today: this lane re-ran scripts/verify_stardf_ontology.py
  --ttl arrow/authority/stardf/stardf_contract.ttl -> exit 1, 4951 triples,
  128 AbiFunction instances, 22 header typedefs vs 11 MISSING type mappings
  (S-3 witnessed the same exit 1). Invariant 4 (100% census parity) NOT met.
- Open: extract_purrdf_ontology.py not in-tree; D1 commit split pending
  operator; GL-STARDF-002 DRAFT pending ratification; ladder paths name the
  legacy layout (contract at arrow/authority/, not authority/).

## (d) ABI freeze witness verdict (S-4)

**v1-ACTIVE, no freeze violation.** Fresh in-tree release build exit 0
(after the 4-up cgap dep fix); nm -gU 14 exports = 13 v1 symbols + 1
declared blake3 hygiene export; header syntax-checks clean (-Werror);
0 v2 names in capi src; v2 table 1/13 present. D18 = PENDING-OPERATOR
(OPERATOR-DECISIONS.md:642-644). v1-file + v1-dylib consistent.

## (e) Parity statement (S-5)

PROVEN (witnessed 2026-11-01, sandbox): vendored crates green on their own
suites; columnar byte court with flipping positive control; header/ontology
three-way parity gate + anti-vacuity mutation flips (verify.py --mutate
drop-export -> exit 1); blake3 single-identity; columnar oracle parity
1030,400,108,108,9; serialize/pack/witness parity tests green.

CLAIMED-NOT-PROVEN: parity WITH UPSTREAM purrdf (KD-12 digest parity, KD-14
columnar digest-ordering — zero shared pins, 10 vs 10 disjoint; full-parity
campaign workflow node --check exit 1; the census-parity claim over-reaches
its court). Standing PARTIAL_ALIVE.

## (f) Ranked finishing backlog (dependency-ordered)

1. P5 license-gate re-point — 2 one-liners (verify_stardf_licenses.py
   parents[3]->[2]; build_receipt.py ARROW_ROOT.parent.parent -> .parent);
   register already in-tree (P1 landed). Unblocks R1.
2. Delete stale arrow/authority/stardf/ duplicate (P1 sub-flag; ERRC delete
   outranks; depends on nothing).
3. stardf-sparql-eval lib-target SIGABRT — only RED crate on the board.
4. stardf-process workspace membership — member-or-exclude decision;
   currently cargo exit 101 ambiguity.
5. P6 prose seds — PRD.md(4)/ARD.md(5)/substrate-map:257 — cheap.
6. Land extract_purrdf_ontology.py in-tree; close F1 11 missing type
   mappings; --mutate drop-type non-vacuity run. Depends on (5)/(6) home.
7. n3 evidence receipt + first sql courts — retire log-standing /
   UNKNOWN-as-receipt rows.
8. D1 4-commit split (operator) — binds receipts to SHAs; after 1-4 so
   commits land green.
9. Operator ratification GL-STARDF-002 + D18 (ABI-v2) — last; migration
   cost attached.
10. Operator delete root move-note originals (docs/move-archive/ holds
    byte-copies; S-6).

## Command receipts (this lane)

- cargo check --workspace (arrow/) -> exit 0.
- python3 arrow/scripts/verify_stardf_licenses.py --root arrow -> [SKIP]
  exit 1 (P5 unapplied, register in-tree).
- python3 scripts/verify_stardf_ontology.py --ttl
  arrow/authority/stardf/stardf_contract.ttl -> exit 1 (11 missing type
  mappings; F1 open).
- grep census -> 99 hits / 38 files (excl. target*, .git, rust_out,
  __pycache__, move-archive).
- Receipts read: S-1, S-3, S-4, S-5, S-6 landed and cited; S-2
  build-baseline.md ABSENT at deadline — named loudly above.

## v26.11.2 forward plan (lane V2-9, 2026-11-01)

Plan-of-record lives in /Users/sac/cgap/docs/jira/v26.11.2/ and is landing
concurrently; this section points, it does not duplicate. Observed at this
lane's read (ls, 2026-11-01): `00-ard-prd.md` and `05-wave-plan.md` present,
plus `waves/` (v2-01-charter.md, v2-05-wave-plan.md). Landing (named, not
yet observed): `01-release-gates.md`, `02-pareto-levers.md`,
`03-ost-design-around-register.md`, `04-stardf-closure.md`. Re-read the
directory at use time; counts above are as-of-read.

### stardf-side open items restated (with coordinates)

1. F1 census: 11 missing type mappings — PurrdfAbiVersion,
   PurrdfCapabilities, PurrdfGovernedEntailmentEvidence,
   PurrdfGovernorEvidence, PurrdfGovernorTrip, PurrdfGraphMatch,
   PurrdfPartialCertificate, PurrdfQueryGovernors, PurrdfResourceVector,
   PurrdfStr, PurrdfTermView (docs/ticket-001-state.md "What remains
   open" item 0; board section (c): verify_stardf_ontology.py exit 1,
   4951 triples, 128 AbiFunction, 22 header typedefs vs 11 missing).
2. verify_stardf_licenses.py authoring: witnessed never-written under
   stardf as a register-consuming gate; the in-tree file today is
   arrow/scripts/verify_stardf_licenses.py (still `parents[3]`,
   P5 unapplied — board section (a)).
3. std-strip: 392 errors at arrow/crates/stardf-hash/src/dispatch.rs:292
   (claimed from the std-strip ticket; not re-run this lane).
4. KD-12 / KD-14 upstream-parity courts: CLAIMED-NOT-PROVEN — 0/10
   shared digests vs purrdf (board section (e)).
5. workflow syntax: workflows/v26.11.1-full-parity.workflow.js line 541
   — the `return {` at line 540 closes with `},` then line 542 opens a
   second `return {`; node --check exit 1 per board section (e).
6. stardf-process workspace ambiguity: cargo exit 101, member-or-exclude
   decision pending (board section (b)).
7. Authority-register re-homing: landed coordinator-side — verified this
   lane, ls /Users/sac/stardf/authority/stardf/ -> 6 entries
   (license-compat.json, license-inventory.toml, marketplace_packs.json,
   stardf_contract.ttl, stardf-columnar.json, stardf-contract.json).
   Companion repair still open: build_receipt.py 2-up `parent.parent`
   resolution, and stale arrow/authority/stardf/ duplicate.

### Concurrent-lane-collision disclosure

S-1's reverted fixes were re-applied post-revert (witnessed this lane:
grep for ggen-legacy in h03_bootstrap_sim.sh, workflow line refs,
.claude/settings.json -> 0 hits). HEAD moved fe935f7 -> de7c9f5
("chore: add root .gitignore", git log this lane). During the wave,
arrow/crates/stardf-arrow-capi/Cargo.toml took a cgap path-dep fix, and
target/ lease debris was deleted (coordinator cleanup); the workspace
check below re-ran green on the post-collision tree.

### Receipt

- ls /Users/sac/cgap/docs/jira/v26.11.2/ -> exit 0 (2 files + waves/).
- ls /Users/sac/stardf/authority/stardf/ -> exit 0 (6 entries).
- git -C /Users/sac/stardf log --oneline -5 -> exit 0 (de7c9f5, fe935f7).
- awk 'NR==541' workflows/v26.11.1-full-parity.workflow.js -> exit 0.
