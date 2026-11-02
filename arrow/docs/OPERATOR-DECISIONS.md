# Operator Decision Register — stardf/arrow (GL-STARDF-001)

Decision pack for every PENDING / DECIDED operator decision on the
stardf/arrow slice as of 2026-10-31 (completion lane 19). Nothing here flips a
decision by itself — each PENDING-OPERATOR row requires explicit sign-off.
Statuses: `DECIDED` (recorded, evidence cited), `PENDING-ON-LANE`
(owned by a concurrent lane; this register records the dependency, does not
pre-empt it), `PENDING-OPERATOR` (needs operator/coordinator sign-off before
the blocking wave dispatches).

## Register

| # | Decision | Status | Owner | Blocks |
|---|---|---|---|---|
| D1 | Commit strategy (untracked trees) | PENDING-OPERATOR | operator | W1 SHA-binding, all |
| D2 | CI placement: A (ci.yml) vs B (file) | PENDING-OPERATOR (rec: A) | operator | W5 |
| D3 | W1 census: value-structs in | DECIDED (coordinator) | coordinator | W1 |
| D4 | ggen sync vs ad hoc render | PENDING-OPERATOR (rec: sync) | operator | W2 |
| D5 | Bench crate workspace membership | PENDING-ON-LANE | C7 lane | CI bench gate |
| D6 | Avec pin: path-dep vs git-rev | PENDING-OPERATOR (rec: split) | operator | W3, W5 |
| D7 | ArrowArrayStream spec source | PENDING-OPERATOR (rec: upstream) | operator | W3 producer |
| D8 | Root `[package]` vs tests crate | PENDING-ON-LANE | C8 lane | workspace gates |

---

## D1 — Commit strategy for the untracked tree

**Status: PENDING-OPERATOR.**

**Evidence.** `git status` (session snapshot, branch `fix/v26.9.29-ci-gates`)
shows the entire substrate untracked: `stardf/` (including `stardf/arrow/`),
`authority/stardf/` (contract ttl + json + rust-pack + marketplace_packs.json),
`scripts/extract_purrdf_ontology.py`, `scripts/verify_stardf_ontology.py`,
`scripts/verify_marketplace_packs.py`, `stardf/docs/jira/v26.10.31/*`,
`tickets/GL-STARDF-001.md`. The lane-16 wave receipt
(`stardf/arrow/evidence/wave-finish-lane16-2026-10-31.json`) records
`subject.head` as the parent repo HEAD with the note: "stardf/ is untracked in
parent — tree_digest is the receipt-time whole-tree sha256 over all 45 files".
Receipts cannot bind to a commit SHA until the tree is committed
(`05-next-steps.md` §1: "commit `authority/stardf/` (receipts bind to a SHA,
not the working tree)").

**Options.**

- **Option A — 4-commit split (recommended).** Order matters: planning/authority
  first so receipts land against committed authority.
  - c1 `docs(stardf): planning pack v26.10.31 (plan, waves, substrate,
    falsifiers, next steps)` — `stardf/docs/jira/v26.10.31/`,
    `tickets/GL-STARDF-001.md`.
  - c2 `feat(stardf): authority contract + W1 verifiers (pre-census)`
    — `authority/stardf/`, `scripts/extract_purrdf_ontology.py`,
    `scripts/verify_stardf_ontology.py`, `scripts/verify_marketplace_packs.py`.
    Committing the verifiers BEFORE the W1 census lands preserves the F1
    falsifier: the committed verifier must FAIL on the committed TTL
    (`05-next-steps.md` §1 ladder).
  - c3 `feat(stardf): stardf-arrow workspace (core, capi, bench, ggen manifest, ci draft)`
    — `stardf/arrow/` (whole tree, `target/` excluded — verify `.gitignore`
    covers `target/` and `crates/*/target/` before staging; `Cargo.lock`
    included for the workspace, bench keeps its own).
  - c4 `evidence(stardf): wave-finish lane-16 receipt + evidence conventions`
    — `stardf/arrow/evidence/`, `stardf/arrow/docs/RECEIPTS.md` if not already
    in c3 scope. Note: this receipt's `subject.head` cites parent HEAD c9f8028
    with untracked stardf/ — after c1-c3 land the receipt is historical
    evidence of the pre-commit state; a fresh receipt post-commit re-binds to
    the new SHA.
- **Option B — single monolithic commit.** Simpler, but the F1 falsifier
  (verifier FAILs on pre-fix TTL) becomes unreplayable — verifier and fixed
  TTL land in the same commit, so the FAIL state exists only in the working
  tree, never on disk.
- **Option C — commit as-is without split.** Rejected: mixes authority,
  evidence, and code; breaks the per-lane atomic-commit law
  (`~/.claude/rules/same-checkout-fanout.md` — coordinator owns git
  transitions, commits per lane).

**Consequence of delay.** Every receipt stays SHA-less (tree_digest only);
W1 closure per `05-next-steps.md` §1 is blocked.

**Recommendation: Option A, coordinator-executed (lane contract forbids git
operations by this lane).**

---

## D2 — CI placement: Option A (job in ci.yml) vs Option B (standalone workflow)

**Status: PENDING-OPERATOR** (lane 15's draft already argues A; recorded here
for sign-off).

**Evidence.** `stardf/arrow/ci/stardf-arrow-reconstitution.yml` header (DRAFT,
lane 15): Option A = append the fenced job block to
`.github/workflows/ci.yml` as a new top-level job alongside
dsrust-reconstitution / docs-pack-reconstitution; `.github/expected-workflows.txt`
stays at 3 entries (after-code-reading-crown.yml, ci.yml,
v26-9-28-crown.yml) — no inventory delta. Option B (new file under
`.github/workflows/`) requires a 4th expected-workflows.txt entry and is
rejected in the draft. The staging copy exists only for yaml.safe_load
validation; moving it into `.github/workflows/` would make GitHub read it and
fail the workflow-inventory gate.

**Options.**

- **Option A (recommended)** — job block into `ci.yml`. No inventory gate
  delta; ci.yml triggers are unscoped so the diff is the job block alone.
  Con: ci.yml grows; the job rides every CI run unless path-filtered (note:
  the fenced block carries its own `on:` key which must be STRIPPED when
  splicing into ci.yml — the fenced artifact is a standalone-parseable copy,
  not a splice-ready fragment; this splice detail is part of the
  operator-approved diff).
- **Option B** — standalone `stardf-arrow-reconstitution.yml` in
  `.github/workflows/`. Cleaner isolation, real path-filtered triggers as
  drafted. Con: expected-workflows.txt must go 3→4, touching an inventory
  gate owned by the repo's CI contract.

**Dependency caveat.** The draft's own AS-BUILT NONCLAIMS: workspace
`cargo test --locked` is RED in this checkout (stream_courts court_2 per the
lane-16 receipt; court_5 panic at `stream_c.rs:68`); no GitHub Actions run has
occurred. Whichever option is chosen, the job lands red until the owning
lanes repair those courts — placement decision can precede, but wiring should
wait for green-or-expected-red sign-off.

**Recommendation: Option A**, with the splice caveat above recorded.

---

## D3 — W1 census scope: value-structs included

**Status: DECIDED (coordinator).**

Value-struct class included; opaque-only rejected. Evidence:
`05-next-steps.md` §5 row 3 — opaque-only would pass 11/11 today and be blind
to the 5 function-referenced types. Closing work remains OPEN: the census
(type census, two-directional, rename invariant, `--mutate drop-type`) does
not exist in `scripts/verify_stardf_ontology.py` yet (its `--help` exposes
only `--ttl`/`--header`), and F1 has not been demonstrated on the current TTL.

---

## D4 — ggen sync admission vs ad hoc template render

**Status: PENDING-OPERATOR** (recommendation: `ggen sync run` with replay step,
but NOT before the template-mapping prerequisite lands).

**Evidence.** `stardf/arrow/ggen.toml` (lane 14): pack route is rust-pack by
absolute path (`/Users/sac/ggen-marketplace/packs/rust-pack`) — the
W2-corrected route per `02-wave-contracts.md`; clap-noun-verb-crate-pack and
wasi-json-abi-pack rejected (wrong shape / deprecated). The same file records
the W2 prerequisite: rust-pack's gates query the `rust:` namespace
(rust:Struct/rust:Function/rust:itemName); against `ontology.ttl` — which
speaks `stardf-arrow:` — all 5 pack gates return 0 rows (rdflib one-liner
witnessed, lane-14 report) unless the W2 bridge query (CONSTRUCT
stardf:AbiFunction/TypeDefinition → rust:Function/rust:Struct) runs first.

**Options.**

- **Option A — `ggen sync run` admission (C25 self-host pattern)**, with
  `ggen.toml` [ontology] pin + documented fresh-clone replay step, per
  `05-next-steps.md` §5 row 5. Pros: deterministic regeneration, receipted,
  generator-identity pinned. **Prerequisite (blocking)**: the stardf:→rust:
  bridge query must exist and be wired into the sync pipeline, otherwise
  every pack gate passes vacuously on 0 rows — a gate that passes on an empty
  graph is exactly the vacuous-gate class the anti-vacuity law refuses.
- **Option B — ad hoc template render.** Faster to first output; no replay
  step, no generator identity, drifts O(N²) against the C25 pattern.

**Dry-run status**: dry-run renders specimen crates only; no mutating
`ggen sync run` has been executed against this tree (nothing in `evidence/`
receipts a sync run). Do not admit a mutating sync before the bridge lands.

**Recommendation: Option A**, sequenced AFTER the W2 bridge query lands and
its gate-row non-vacuity check passes (gates must see >0 rows on the bridged
graph). Until then the crate stays hand-maintained; any sync is dry-run only.

---

## D5 — Bench crate workspace membership

**Status: PENDING-ON-LANE (C7 decides concurrently).**

Recorded facts only, no pre-emption: `stardf-arrow-bench` is currently a
standalone crate with its own `Cargo.lock`/`target` — README states
`cargo build -p stardf-arrow-bench` from the root fails ("did not match any
packages"); build it from its own directory. `docs/BENCH_BASELINE.json`
numbers are self-labeled UNVERIFIED (local min-of-5, contention-noisy);
CI-qualified rerun vs the baseline is the closing court. This register defers
to C7's decision; whichever way it goes, the CI job (D2) must reference the
bench consistently with it.

---

## D6 — Avec pin strategy

**Status: PENDING-OPERATOR** (recommendation: two-phase — path-dep for the W3
one-test slice, git-rev pin at W5 CI time).

**Evidence.** `05-next-steps.md` §5 row 1 (leaning path-dep for the one-test
slice, rev pin at W5 CI time) and `stardf/arrow/README.md` §Avec Integration
Plan: `avec-ffi` consumes `stardf-arrow-capi` by path dependency — "this
workspace is never published" — while the workspace itself pins `avec-arrow`
by git rev `20248e4`, mirroring the purrdf root manifest. F4 evidence: W3's
gate is ONE real avec integration test exiting 0 on the exact subject
(cross_allocator_court, `05-next-steps.md` §2); C7 evidence: bench/substrate
falsifiers in `04-falsifiers.md`.

**Options.**

- **Option A — path-dep now, rev pin at W5 (recommended).** Path-dep matches
  the one-checkout law (no copies) and the W3 gate is a local exact-subject
  test; the rev pin lands when the edge enters CI, where reproducibility
  requires a pinned rev. Precedent: the workspace already pins `avec-arrow`
  by rev `20248e4`.
- **Option B — rev pin immediately.** Maximally reproducible now, but forces
  the avec-side test through a pushed rev before the court exists — every
  iteration of the W3 slice becomes a publish cycle. Rejected for the slice;
  correct at W5.
- **Option C — path-dep permanently.** CI never reproduces the edge
  hermetically; rejected.

**Also gating W3**: R1 license receipt (`05-next-steps.md` §3) must land
BEFORE the avec dependency edge is admitted (P28-SIL ↔ MIT/Apache/MulanPSL
skew), and D7 must be resolved (the producer struct layout is the other W3
precondition).

---

## D7 — ArrowArrayStream spec source

**Status: PENDING-OPERATOR** (recommendation: ratify the upstream Arrow C
Data Interface stream spec as authority).

**Evidence.** `05-next-steps.md` §5 row 4: `ArrowArrayStream` layout is
UNKNOWN locally — zero references in avec/purrdf/ggen-legacy; only prose pins
exist at `avec/docs/jira/v26.10.31/01-stardf-reconstitution.md:81` and
contract ttl line 182. The as-built struct in
`stardf/arrow/crates/stardf-arrow-capi/src/stream_c.rs` is spec-shaped:
`stream_courts` exercises get_schema/get_next/get_last_error/release
semantics (5/6 passing, court_5 use-after-release panic at `stream_c.rs:68`
still open). `ArrowArrayStream` "does not exist in avec, purrdf.h, or anywhere
upstream" as a purrdf symbol (README) — this is a NEW contract, not a
reconstituted one, so the predecessor-contract admission rule does not apply;
an external spec must be named as authority.

**Options.**

- **Option A — ratify upstream Arrow C Data Interface stream spec**
  (recommended). The as-built surface already matches its fn-pointer shape;
  ratifying makes the spec the checkable authority (court failures become
  spec divergences, registerable in `docs/DIVERGENCES.md` as KD rows with a
  close condition) instead of leaving layout derived from local code
  (circular — the code would be its own oracle).
- **Option B — stardf-local layout + divergence receipt.** Keeps freedom to
  deviate, but with zero local references the layout has no second source;
  every consumer (avec-ffi, foreign C hosts) must then trust a
  self-authored ABI with no spec receipt. Divergence-from-what?

**Falsifier if A**: any as-built deviation from the ratified spec must appear
as an OPEN KD row in `docs/DIVERGENCES.md` with a close condition; a spec
claim with no KD row and a failing conformance court is overclaiming.

**Recommendation: Option A.** On sign-off, W3 producer work cites the spec
section for each fn pointer, and court_5's repair is judged against the
spec's release semantics, not local intent.

---

## D8 — Root `[package]` vs tests crate

**Status: PENDING-ON-LANE (C8 decides concurrently).**

Recorded dependency only: the workspace currently has two members
(stardf-arrow-core, stardf-arrow-capi — lane-16 receipt: "cargo check
--workspace exit 0 (2 workspace members)") plus the standalone bench (D5).
C8 owns the manifest-structure question; D1's commit split (c3) should stage
the tree in whatever manifest shape C8 leaves it, so c3 lands after C8's
resolution or is amended by the coordinator.

---

## Sign-off block

| Decision | Operator verdict | Date | Signature/notes |
|---|---|---|---|
| D1 commit split (Option A, 4 commits) | | | |
| D2 CI placement (recommend A) | | | |
| D4 ggen sync (recommend A, post-bridge) | | | |
| D6 avec pin (recommend two-phase) | | | |
| D7 ArrowArrayStream spec (recommend upstream ratification) | | | |

DECIDED rows (D3) need no sign-off; PENDING-ON-LANE rows (D5, D8) are resolved
by their owning lanes and recorded here only as dependencies.

## Nonclaims

- Nothing in this file flips a decision; PENDING-OPERATOR rows await
  sign-off in the block above.
- No git operations were performed by this lane (contract: completion lane 19
  owns only this file).
- No CI run, sync run, or avec integration test has been executed; all
  evidence cited is on-disk file content and this session's receipts
  (`evidence/wave-finish-lane16-2026-10-31.json`, `docs/RECEIPTS.md`).

## See Also

- [`../jira/v26.10.31/05-next-steps.md`](../../docs/jira/v26.10.31/05-next-steps.md)
  §5 — the source decision register this pack expands
- [`../ci/stardf-arrow-reconstitution.yml`](../ci/stardf-arrow-reconstitution.yml)
  — D2 evidence (draft + Option A fenced artifact)
- [`../ggen.toml`](../ggen.toml) — D4 evidence (pack route + bridge prerequisite)
- [`DIVERGENCES.md`](DIVERGENCES.md) — KD-row ledger D7's falsifier lands in

---

# Phase-2 Closure Addendum (lane 19, 2026-10-31)

Append-only addendum: D9–D14 below are NEW decision rows for the phase-2
pending set. No existing row (D1–D8) is rewritten or flipped; D1's Option A
split below is CITED, not amended. Each row: options + evidence + recommendation,
PENDING-OPERATOR unless marked otherwise.

## Addendum register

| # | Decision | Status | Source lane/pack | Blocks |
|---|---|---|---|---|
| D9 | ABI freeze expansion — ratify Tier A/B/C | PENDING-OPERATOR (ticket is DRAFT) | lane 2: `stardf/arrow/tickets/GL-STARDF-002-freeze-expansion.md` | all SQL-cycle export work; W1 census additions (B0) |
| D10 | KD-08 BLAKE3 flip: blake3 crate vs purrdf-hash native | PENDING-SIGN-OFF | lane 4: `stardf/docs/jira/v26.10.31/11-blake3-arbitration.md` | columnar W4 digest courts (B2) |
| D11 | Columnar physical id encoding: Int64-widened vs Int32 | PENDING-OPERATOR (court-decided) | 06-phase0 §3/§10.7; 07-port-master-plan OPEN-7 | columnar W4 |
| D12 | SQL adaptation scope: Option A/B/C | PENDING-OPERATOR | lane 6: `stardf/docs/jira/v26.10.31/09-sql-adaptation-scope.md` | SQL cycle start (OPEN-2) |
| D13 | frozen-duckdb re-home: option a/b/c | PENDING-OPERATOR | lane 7 plan: `07-port-master-plan.md` §2 B3, §4 OPEN-5 | duckdb-scan W5 portable receipt (B3) |
| D14 | Commit strategy for the current on-disk tree | PENDING-OPERATOR (refines D1) | lane 19 (this lane), on `git status` of the live tree | all SHA-binding receipts (same blocks as D1) |

---

## D9 — ABI freeze expansion (GL-STARDF-002): Tier A / B / C

**Status: PENDING-OPERATOR.** Ticket is `DRAFT-PENDING-OPERATOR-RATIFICATION`;
nothing here ratifies a tier.

**Evidence.** The frozen surface (`stardf/arrow/docs/ABI-FREEZE.md`, v26.9.29)
is 13 `stardf_*` exports covering only the batch/stream surface. The SQL/query
port (06-phase0 §5) needs query/UPDATE/serialize exports whose carrier types
(ResultKind, QueryGovernors, GovernorEvidence, PartialCertificate, RowCursor)
do not exist in the CAPI. Ticket §2 enumerates the candidate set (17
query/UPDATE + 4 serialize = 21, verified by grep against
`~/purrdf/crates/rdf-capi/include/purrdf.h`; the ticket's own Nonclaims
section flags cell drift in its stardf-name column — purrdf column is the
verified enumeration). 06-phase0 §8.2 classifies this BLOCKER-CLASS: cut
FIRST, before any query export work; 07-port-master-plan §2 makes it B0,
blocking the W1 census additions (11 anon structs).

**Options.**

- **Tier A — Minimal (3 exports + 4 carriers)**: `stardf_query`,
  `stardf_query_governed`, `stardf_query_governors_init` + ResultKind,
  QueryGovernors, GovernorEvidence, RowCursor; lifecycle reuses the frozen 13.
  Smallest lawful increment that lets the W2 scaffold (bridge query) proceed.
- **Tier B — Minimal + Update + CanonArrow egress**: Tier A +
  `stardf_update_governed` + `stardf_query_canon_arrow`, dragging
  PartialCertificate (update atomicity certificate).
- **Tier C — Full 21-export parity**: all 17 query/UPDATE + 4 serialize;
  ABI-append-only per the pre-1.0 rule.

**Enforcement either way (ticket §5)**: until ratification, zero
query/UPDATE/serialize `#[no_mangle]` in capi src; `nm -gU` diff of the built
dylib vs the ratified export table must exit 1 on any extra/missing symbol;
a gate that cannot exit 1 is VACUOUS.

**Recommendation: Tier A** — smallest tier that unblocks B0 (W1 census
additions + W2 scaffold) without dragging PartialCertificate before Update
work exists. Tier boundaries are falsifiable (ticket F3: carrier drag past
the tier boundary is REFUSED).

---

## D10 — KD-08 BLAKE3 single-impl flip

**Status: PENDING-OPERATOR-SIGN-OFF** (lane 4's pack is explicit that it does
not flip; KD-08 stays OPEN in `docs/DIVERGENCES.md`).

**Evidence.** stardf binds ONE impl today — `blake3::hash`
(`arena.rs:344`), crate declared 1.8.5 / lock resolved 1.8.7 (silent drift
already witnessed), 3-crate normal closure (arrayvec, cfg-if,
constant_time_eq). purrdf predecessor: single path via `purrdf_hash::blake3`,
zero runtime deps, no_std/wasm32-clean, carries the frozen-vector
conformance suite. avec: registered defect of the two-impl shape
(`avec-blake3` + duplicate in `avec-arrow/src/blake3.rs`).

**Options.**

- **Option A — keep blake3 1.8.x crate (status quo)**. 3 registry crates of
  supply-chain surface; declared-vs-lock drift already happened; license
  audit owed per P0-18 first-party closure law.
- **Option B — vendor purrdf-hash native (recommended by lane 4)**. Zero
  runtime deps, carries its frozen-vector conformance; lawful form is
  vendoring (`blake3.rs`, `digest.rs`, `hex.rs` + reaches) with SHA-pinned
  PROVENANCE — a path dep on `~/purrdf` is rejected (portable-receipt law).
  Cost: medium migration (vendor + 1 call-site swap + courts).

**Close conditions (both options)**: (1) `cargo tree -e normal` single
blake3-identity gate wired into `ci/` + justfile, exiting 1 on >1 AND on 0
(VACUOUS when the decision in force is "bind one"); (2) digest-parity court
vs `avec-blake3` as a test-only dev-dep (empty/block-boundary/streaming/
merkle shapes) so the gate's `-e normal` scope stays clean.

**Recommendation: Option B**, per lane 4; migration steps §5 of
`11-blake3-arbitration.md`.

---

## D11 — Columnar physical id encoding: Int64-widened vs Int32

**Status: PENDING-OPERATOR (court-decided).** Per 06-phase0 §10.7 and
07-port-master-plan OPEN-7: "decided only by columnar W4 digest court."

**Evidence.** 06-phase0 §3: ids serialize as PLAIN INT64 in the frozen
byte-deterministic recipe; determinism survives u32 with carry-points
`i32::MAX` dictionary cap → `u32::MAX-2`, checked conversion, golden
regeneration with recorded reason (fixtures splitmix64-seeded, F2a-safe by
construction). §2: dictionary cap is the u32 id-space limit; D3's
offset-cancellation makes `D::Id::raw()` words legal straight through. §6:
columnar term ids are INT64, not u32 (premise correction).

**Options.**

- **Option 1 — Int64-widened (golden-stable)**: physical columns stay
  INT64; existing 10 pinned digests and golden corpus survive unchanged.
  Zero golden-regen; carries 2x width on id columns.
- **Option 2 — Int32 physical**: half-width id columns (scan-bandwidth
  win on the 5-table stardf_scan path); requires golden regeneration with
  a recorded reason and the F4b `.sha256` manifest landing in the same
  wave, else the digest court compares against stale pins.

**Recommendation: neither pre-decided — run the columnar W4
digest-equality court (purrdf-vs-stardf, same quads → byte-identical
Parquet) under Option 1 first; flip to Option 2 only if the court passes
AND a measured scan-bandwidth gain justifies the golden regen.** Per
07-port-master-plan §3a W4, the court decides OPEN-7, not this register.

---

## D12 — SQL adaptation scope: Option A / B / C

**Status: PENDING-OPERATOR.** Lane 6's pack states the plan in three
mutually exclusive options (register item OPEN-2); recommendation included,
not flipped.

**Evidence.** Volume as-of P0 receipts: algebra ~35K / eval ~215K / results
~10K / conformance ~9K (~270K total). Shared anatomy in every option: the
TermId seam is 5 bridge edges across 4 named sites (bounded, ticket-sized —
not the reason to shrink scope) and the governor binding (8-dim
ResourceVector, charge deltas not absolutes, monotone-budget invariant).

**Options (lane 6 §2).**

- **Option A — full vendor (~270K ln)**: all four SPARQL crates vendored;
  per-line adaptation cost inside the 215K eval crate is UNKNOWN (no lane
  has diffed eval internals); gate = one real purrdf eval golden class
  ported end-to-end, digest-compared, mutation-flippable.
- **Option B — algebra + results, eval external/deferred (~45K ln)**
  (lane-6 recommendation): results IS the CanonArrow egress encoder — the
  piece the host boundary actually needs; deferring eval converts the
  largest UNKNOWN into a later, better-grounded ticket. Gating probes:
  compile-standalone probe (algebra/results without eval) and eval
  public-API probe (governor watermark observability). Hybrid rename-depth
  (R1) is itself PENDING-OPERATOR.
- **Option C — facade-only, purrdf external dep**: smallest volume, but
  flagged mission skew — ggen-legacy's AGENTS.md construct stage
  manufactures, it does not wrap; and it makes the R1 license edge
  non-optional from day one.

**Recommendation: Option B**, conditioned on R3 (compile-standalone probe)
passing; if R3 fails, fall back to A with a W-lane eval-cost probe first.
Mission-skew ruling (R7) needed only if C is seriously considered.

---

## D13 — frozen-duckdb re-home (B3 / OPEN-5)

**Status: PENDING-OPERATOR.** No planning lane had authority to re-home;
lane 7's plan (`07-port-master-plan.md` §2) registers B3 as the one blocker
in the chain that is PENDING-OPERATOR, blocking only duckdb-scan W5
portable receipts. Note: lane 7 landed B3 as a blocker chain entry, not as
a dedicated a/b/c option doc — the options below are constructed from the
cited evidence (06-phase0 §1/§7/§8.3) so the operator has a menu.

**Evidence.** `purrdf-duckdb` pins `frozen-duckdb` 2.0.3 by absolute path
`/Users/sac/frozen-duckdb` (machine-local; 06 §1); the C header +
Rust VTab trait are proven in-process (06 §1, "RESOLVED, not UNKNOWN");
falsifier 06 §8.3: build with `$HOME/.frozen-duckdb` +
`/Users/sac/frozen-duckdb` renamed away — the build must still succeed.

**Options.**

- **Option a — canonical re-checkout in-portfolio**: move/copy the
  frozen-duckdb fork to a canonical checkout under the portfolio
  (e.g. `~/frozen-duckdb` at a pinned rev, one canonical checkout per repo,
  no copies), pin it in the consuming manifests by rev. Falsifier-replayable
  on any machine with the same checkout law.
- **Option b — env-var relocation pin**: honor `FROZEN_DUCKDB_DIR`
  (defaulting to the current absolute path) so the 06 §8.3 falsifier
  (rename the machine-local dir away, build via `$HOME/.frozen-duckdb`)
  becomes runnable. Smallest diff, but still machine-local by default —
  portable receipts stay weak.
- **Option c — registry/git-rev upstream pin**: publish or git-pin
  frozen-duckdb and consume by rev like `avec-arrow` @ `20248e4`. Strongest
  replay, but requires an upstream the portfolio controls (publish decision
  is outside lane authority).

**Recommendation: Option a** (canonical checkout + rev pin), because it is
the only option under which the 06 §8.3 falsifier and a portable duckdb-scan
W5 receipt are replayable off-machine; Option b is the minimal fallback if
the operator declines to give frozen-duckdb a canonical checkout.

---

## D14 — Commit strategy for everything now on disk

**Status: PENDING-OPERATOR (refines D1's Option A against the live tree).**
Lane contract: this lane performs NO git mutations; the split below is a
proposal for coordinator execution, respecting D1's falsifier-replayability
rationale (verifiers land BEFORE the artifacts they police, so the
falsifier's FAIL state exists on disk, not only in the working tree).

**Evidence.** `git status` on `fix/v26.9.29-ci-gates` @ c9f8028 (this
session): untracked `authority/stardf/` (contract ttl + json + rust-pack +
`marketplace_packs.json`), `authority/v26.9.28/graphlaw-observable-contract.json`,
`docs/jira/`, `lib_rs.tmpl`, `scripts/extract_purrdf_ontology.py`,
`scripts/extract_stardf_arrow.py`, `scripts/verify_stardf_ontology.py`,
`scripts/verify_marketplace_packs.py`, `stardf/` (arrow + columnar + docs),
`tickets/GL-STARDF-001.md`. D1's wave-era split covered the 45/50-file tree;
the tree has since grown the phase-2 planning pack
(`stardf/docs/jira/v26.10.31/09-sql-adaptation-scope.md`,
`11-blake3-arbitration.md`) and this addendum — so D14 re-derives the split
against the live tree.

**Proposed split (6 commits, coordinator-executed; parent HEAD c9f8028,
branch `fix/v26.9.29-ci-gates`)**:

- **c1** `docs(stardf): planning pack v26.10.31 (phase-0/2 packs, port master
  plan, blake3 arbitration, sql scope)` — `stardf/docs/jira/v26.10.31/`,
  `tickets/GL-STARDF-001.md`, `docs/jira/`. Lands FIRST so receipts and
  later commits cite committed authority.
- **c2** `feat(stardf): authority contract + verifiers (pre-census)` —
  `authority/stardf/`, `authority/v26.9.28/graphlaw-observable-contract.json`,
  `scripts/extract_purrdf_ontology.py`, `scripts/extract_stardf_arrow.py`,
  `scripts/verify_stardf_ontology.py`, `scripts/verify_marketplace_packs.py`,
  `lib_rs.tmpl`. Preserves D1's F1 falsifier-replayability: committed
  verifier must FAIL on the committed pre-fix TTL — verifier-before-artifact
  ordering is load-bearing.
- **c3** `feat(stardf): stardf-arrow workspace (core, capi, bench, ggen
  manifest, ci draft, ontology, queries, templates, tests, tickets)` —
  `stardf/arrow/` whole tree EXCEPT `evidence/`; verify `.gitignore` covers
  `target/` and `crates/*/target/` before staging; `Cargo.lock` included.
  After C8's manifest resolution (D8) or amended by the coordinator.
- **c4** `evidence(stardf): lane receipts + evidence conventions` —
  `stardf/arrow/evidence/`, `stardf/arrow/docs/RECEIPTS.md` if not already
  in c3 scope. Receipts cite parent HEAD c9f8028 with untracked stardf/ —
  historical pre-commit evidence; a fresh receipt post-commit re-binds to
  the new SHA.
- **c5** `docs(stardf): arrow slice docs (ABI-FREEZE, DIVERGENCES, ARD, PRD,
  OPERATOR-DECISIONS, BENCH_BASELINE)` — `stardf/arrow/docs/` (including
  this addendum) + `stardf/arrow/README.md`, `justfile`,
  `rust-toolchain.toml` if not already in c3 scope.
- **c6** `chore(stardf): phase-2 lane outputs re-checkout sweep` — any
  remaining phase-2 outputs landed after this register was drafted
  (re-run `git status` immediately before staging; the coordinator owns the
  final partition).

**Sequencing law.** c1 (planning/authority) before c2 (verifiers) before
c3 (artifacts) before c4 (receipts): each commit's contents must be able to
FAIL against the previous commit's state for the falsifier chain to replay.
c3 waits on D8 (C8 manifest shape); c4's receipt re-binding is post-commit.

**Recommendation: execute the split above, coordinator-executed, AFTER
D8/C8 resolves manifest shape; PENDING-OPERATOR.**

---

## Addendum sign-off block

| Decision | Operator verdict | Date | Signature/notes |
|---|---|---|---|
| D9 freeze expansion (recommend Tier A) | | | |
| D10 BLAKE3 (recommend purrdf-hash native, vendored) | | | |
| D11 columnar encoding (recommend W4-court-first, Int64 initial) | | | |
| D12 SQL scope (recommend B, conditioned on R3 probe) | | | |
| D13 duckdb rehome (recommend option a: canonical checkout + rev pin) | | | |
| D14 commit split (recommend 6-commit plan, post-D8) | | | |

## Addendum nonclaims

- No git mutation performed by this lane (contract); the D14 split is a
  proposal, not an execution.
- No tier ratified, no BLAKE3 flip executed, no encoding chosen, no scope
  option committed, no re-home performed — all six rows are
  PENDING-OPERATOR with recommendations only.
- Lane outputs cited as of 2026-10-31 on-disk state; counts (21 exports,
  ~270K ln, 3-crate blake3 closure) are citations from the cited packs, not
  re-measurements by this lane.

---

## OPERATOR SIGN-OFF — 2026-10-31 (completion-wave precondition)

All six pending decisions signed by the operator (in-conversation authorization,
"Sign all as recommended"; this record IS the on-disk signature per the completion
wave's disk-signature law):

- **D9: RATIFIED — GL-STARDF-002 Tier A** (stardf_query, stardf_query_governed,
  stardf_query_governors_init + carriers ResultKind / QueryGovernors /
  GovernorEvidence / RowCursor).
- **D10: DECIDED — Option A retained** (blake3 crate stays); Option B migration
  DEFERRED, remains the registered recommendation in
  DECISION-BRIEFS/D10-blake3-flip.md.
- **D11: Int64-widened** (already executed by the columnar wave).
- **D12: ADMITTED — amended Option B slice** (term model + sink + csv +
  xml_escape + blank_label + vocab + SparqlResult + dataset builder +
  branchless/deflate/events leaves; the 215K-line eval crate NOT vendored).
- **D13: DECIDED — option (a) canonical checkout + rev pin**; execution detail:
  frozen-duckdb/-sys/-builder all pinned =2.0.3 per W1-9's float datum.
- **D14: APPROVED — 6-commit split** per COMMIT-PLAN.md; c1/c2 executed by the
  coordinator pre-wave (SHAs recorded below); c3-c6 post-arbiter.
- **R1: ADMITTED — Apache-2.0-arm receipt** moved from draft_receipts_pending_
  operator into compatibility_receipts (notice carriage: LICENSE_A2 +
  LICENSE_A2LLVM).

---

## OPERATOR DECISIONS — 80/20 re-scope (2026-10-31, third operator block)

- **D15 (one-package rule):** stardf is ONE package — every purrdf crate is
  straight-copied into the single stardf/arrow workspace; internal edges
  (sql→shapes etc.) are ordinary workspace edges. No carve, no feature-gating
  by dependency.
- **D16 (80/20):** Phase 1 copy = scripted bulk pass (scripts/bulk_copy_purrdf.py
  + the prior lanes' copies — all ~53 purrdf crates now on disk as stardf-*).
  Phase 2 = mechanical rename + single root (stardf/arrow survives;
  stardf/crates/ stubs archived) + cargo check loop until green. Phase 3 = the
  four first-class targets only (arrow landed; duckdb critical path; sql whole
  + 4 goldens; n3 whole + suite + discovery).
- **D17 (honest scope):** the copied ~80% carries NO courts — compile-green at
  best; the parity claim is scoped to the four first-class targets. Copied-crate
  breakage surfacing in the ladder: feature-gate or fix forward, whichever is
  faster.
- **D18 (ABI-v2):** PENDING-OPERATOR — the DoD's revised 13-export surface is
  NOT adopted this campaign; arrow stays landed as-is. Adoption is a later
  operator call with the cross-allocator-court migration cost attached.

### H03 GATE-CLOSURE REGISTRATION (2026-10-31, lane H03) — 1 of 3 gates owner-blocked

Landing list receipt 20261101T040521Z `[G3, G5, G8]` → receipt 20261101T051252Z
`[G8]` (G3 + G5 closed fix-forward by lane H03; receipts
`evidence/h03-gate-closure-receipt-2026-10-31.json`). **G8 is registered here
as PENDING-OPERATOR — it is not manifest-reachable**: the gate counts optional
rows (crates.io 404s at upload), and every G8 dep is source-woven (ed25519
signing in `stardf-rdf`'s gts writer; sparql-eval in rdf's projections +
capture bin; shapes in `stardf-sql`'s SHACL compiler path; 18 umbrella
re-exports). The fix classes are the gate's own: the target members join the
closure (D17 re-ratification), or the deps are dropped/re-routed (source
surgery on the four crates).

**Precise rows (24):** `stardf-rdf` normal: stardf-ed25519, stardf-gts,
stardf-sparql-eval; dev: stardf-gts. `stardf-sql` normal: stardf-shapes.
`stardf-toml` dev: stardf-sparql-eval. `stardf-rs` normal: datalog,
duckdb (optional), entail, geo, glob, gts, hnsw, json, markdown, mime,
process, retrieval, shapes, shex, slice, sparql-eval, text, validate.

**Witnessed publish blast radius (bootstrap simulation,
`evidence/h03-bootstrap-sim-2026-10-31.txt`):** 20/25 closure crates package
fully in tier order (real tarballs, <10 MB, LICENSE in-tarball); 5 cannot
publish-shaped-resolve — rdf + rs directly (cargo: "no matching package named
stardf-ed25519 / stardf-datalog"), sql/toml/n3 transitively via rdf. Every
unresolvable first-party req is a version the registry will never carry
(=26.9.29 datalog/ed25519/shapes/slice; =26.11.1 gts/sparql-eval et al.).

**Decision owed:** re-ratify the D17 closure to admit the 19 non-closure
members (duckdb, shapes, gts, sparql-eval, ed25519, datalog, entail, geo,
glob, hnsw, json, markdown, mime, process, retrieval, shex, slice, text,
validate — with their own metadata/carriage costs, incl. 4 at 26.9.29 needing
a version bump), or scope the publish footprint down (drop/reroute the deps,
or take rdf/sql/toml/n3/rs out of the tier list). Until then the publish
ladder stays BLOCKED at G8 by design.

---

## D13 RESOLUTION — frozen-duckdb pins =2.0.4 x3 (2026-10-31, finishers wave, lane 15-rehome-note)

Append-only: this section does not rewrite D13's original row, the phase-2
recommendation, or the sign-off block above; it supersedes at the
execution-detail level only.

**D13: RESOLVED — operator ratified frozen-duckdb / frozen-duckdb-sys /
frozen-duckdb-builder ALL `=2.0.4`** (operator authorization carried by the
finishers-wave lane contract, 2026-10-31).

**Why (F04 falsifier fired).** The registry-only rehome court
(`scripts/verify_duckdb_rehome.sh`, pinned `=2.0.3`) **fired cache-absent**:
with `/Users/sac/frozen-duckdb` and `$HOME/.frozen-duckdb` absent, the
`=2.0.3` pin is not hostable registry-only (fail-closed court path). With the
falsifier fired on `=2.0.3`, the operator ratified `=2.0.4` x3; the
`=2.0.3` cache-absent failure is moot under the ratified pins.

**Consequences.**

- The sign-off block's execution detail ("frozen-duckdb/-sys/-builder all
  pinned =2.0.3 per W1-9's float datum") is **superseded** by `=2.0.4` x3.
- Option (c) mechanics stand: registry-only resolve, no `[patch.crates-io]`,
  no machine-local path in the one-package workspace. Option (a)'s
  canonical-checkout execution detail is superseded by exact registry pins
  (no canonical fork checkout required for the campaign footprint).
- Register effect: D13 PENDING-OPERATOR → **RESOLVED**; falsifier F04's
  close condition is satisfied by the =2.0.4 pins, not by a =2.0.3 fix.

**Nonclaims.** Docs-only lane: no cargo run, no court re-run, no manifest
edit — workspace `Cargo.toml:267` still read
`frozen-duckdb = { version = "2.0.3" }` (float pin) at this lane's read; the
`=2.0.4` x3 manifest edit belongs to the manifest-owning lane and is NOT
claimed landed. The F04 falsifier-fired receipt is cited from the
finishers-wave lane contract, not re-executed by this lane; the `=2.0.4` x3
cache-absent court re-run is NOT witnessed by this lane. No git operations.

**Refresh witness (2026-10-31, lane 15-rehome-note re-invocation; re-read
per concurrent-lane discipline).** The manifest lane HAS since landed the
pins: workspace `Cargo.toml:279-281` read
`frozen-duckdb = "=2.0.4"` / `frozen-duckdb-sys = "=2.0.4"` /
`frozen-duckdb-builder = "=2.0.4"` (grep witnessed, exit 0) at this refresh
read; the line-267 float-pin citation above is superseded on disk. The
`=2.0.3` cache-absent failure is moot under the landed pins. The court script
`scripts/verify_duckdb_rehome.sh` is on disk pinned `=2.0.4` x3 (two-arm
cache-absent design, 10528 bytes, read this refresh). Still NOT witnessed by
this lane: any cargo resolve/compile run or court execution against the
landed pins.

---

# Finish-Campaign Decision Record — D19–D21 (2026-11-01, completion lane 19 / G19)

Append-only continuation of the register after D15–D18 (80/20 re-scope) and
the D13 RESOLUTION section above. No earlier row is rewritten or flipped;
D13's `=2.0.4` resolution is CITED, not amended. All evidence is on-disk at
HEAD `724d542540b497f2928a63b5aece9b37aef4019b` (c4r) + working tree, read
2026-11-01 by this lane. No git mutations, no live publish, no registry
touch, no court execution by this lane.

## Register

| # | Decision | Status | Blocks |
|---|---|---|---|
| D19 | frozen-duckdb family exact-pinned `=2.0.4` x3 | DECIDED (operator) | live publish go |
| D20 | publish-prep first-class, ladder-judged, BLOCKED | DECIDED (campaign) | live publish go |
| D21 | edition 2024 / rust 1.98 build-driven deviation | DECIDED (registered) | avec re-run |

## D19 — frozen-duckdb `=2.0.4` x3 ratified

**Status: DECIDED.** Operator authorization carried by the finishers-wave
lane contract (2026-10-31); the D13 RESOLUTION section above records the
ratification and its refresh witness. This row gives the decision its
register number and the publish-context evidence ladder.

**Evidence.**

- `stardf/arrow/Cargo.toml:275-284` (lane 01-duckdb-pins, 2026-10-31):
  exact `=` pins for frozen-duckdb / frozen-duckdb-sys / frozen-duckdb-builder;
  comment names "registry-only the only possible resolution", no machine-local
  path override — "crates.io publish law".
- G6-frozen-duckdb-pin **PASS, detail "clean"** in all seven publish receipts
  (`evidence/publish_stardf-receipt-20261101T032659Z` … `T040521Z`, read this
  lane) — the pin gate holds in publish context, not just the rehome court.
- `scripts/verify_duckdb_rehome.sh` re-pinned `=2.0.4` x3 (two-arm
  cache-absent design) per the D13 refresh witness.
- Falsifier discipline: F04 FIRED on `=2.0.3` (cache-absent, fail-closed
  court path) BEFORE the operator ratified `=2.0.4` — the decision followed a
  fired falsifier, not a preference; the `=2.0.3` failure class is moot under
  the ratified pins.

## D20 — first-class publish closure: materialized, ladder-judged, BLOCKED

**Status: DECIDED (campaign outcome). Standing of the closure itself:
BLOCKED.** The v26.11.1 campaign treats publish-preparedness as a first-class
deliverable: a ratifiable closure carrying unified crates.io metadata,
dual-keyed dependencies at the exact publish version `=26.11.1`, and a
runnable fail-closed ladder that emits machine-readable receipts. It does NOT
mean live-published: every ladder run this campaign was dry-run; the latest
verdict is BLOCKED with a 3-row landing list; live publish sits behind the
operator gate (`STARDF_PUBLISH_CONFIRM=ratified-v26.11.1` + `--publish`,
`scripts/publish_stardf.py`) and is PENDING-OPERATOR
(`docs/V26-11-1-VERDICT.md`, PENDING-OPERATOR table).

**Evidence.**

- `stardf/arrow/Cargo.toml` — closure materialization, all read this lane:
  `version = "26.11.1"` + unified description/keywords/categories for
  workspace inheritance (`[workspace.package]`, lines 200-227, lane
  07-publish-metadata); 46 stardf-* `version+path` dual-key rows at
  `=26.11.1` in `[workspace.dependencies]` (lines 305-369, lane 08-dual-key),
  so `cargo publish` rewrites path deps to the exact publish version;
  `stardf-scan-duckdb` a first-class closure member (publish=true, lines
  170-174, 360-363).
- Fail-closed ladder: `scripts/publish_stardf.py` gates G1-G9 + R1, dry-run
  by default. Seven receipts landed:
  `evidence/publish_stardf-receipt-20261101T032659Z` … `T040521Z` — all
  verdict **BLOCKED**, landing list shrinking 4→3 (R1 flipped FAIL→PASS in
  the 21:05 window; the final receipt records the flip pair exit 0 at c2 →
  exit 1 at 20:26-20:50 → exit 0 x2 at 21:05).
- Latest receipt (T040521Z) gate standings, read this lane: PASS
  G1-closure-present, G2-tiers-strict-topo, G4-metadata-fields,
  G6-frozen-duckdb-pin, G7-patch-tables, G9-cargo-check-workspace (exit 0),
  R1-license-gate (exit 0); FAIL G3-publish-flag (4 tier-expected crates
  publish=false), G5-dep-hygiene (avec-arrow git dep in stardf-arrow-core +
  stardf-arrow-capi; residual path-only dual-key rows), G8-nonclosure-hard-deps
  (stardf-rdf→{ed25519,gts,sparql-eval}, stardf-sql→stardf-shapes,
  stardf-rs→… incl. new stardf-rs→stardf-duckdb registry-404 row);
  avec-ffi-court FAIL exit 101 (see D21).
- `evidence/v26.11.1-manufacturing-receipt.json` — final receipt (lane
  17-receipt-final pass 2): standing overall PARTIAL_ALIVE; footprint
  reality check (the contract's umbrella cell named "stardf"; the crate on
  disk joined the publish set as `stardf-rs` at 21:00); four-first-class
  targets table (arrow PARTIAL_ALIVE local; duckdb implementation landed,
  courts UNKNOWN-as-receipt; sql + n3 UNKNOWN-as-receipt).
- `docs/V26-11-1-VERDICT.md` — campaign verdict PARTIAL_ALIVE with publish
  BLOCKED; 21 member manifests declare `publish = true` (lane's grep); the
  operator must ratify one footprint before any live publish.

## D21 — edition 2024 / rust-version 1.98: build-driven deviation, registered

**Status: DECIDED (deviation registered).** The operator template for the
campaign carried edition 2021 / MSRV 1.88. Build evidence forced 2024 / 1.98;
this row registers the deviation so it is on the books, not silent drift.

**Evidence.**

- The template value: `stardf/docs/jira/v26.10.31/12-full-parity-campaign.md:32`
  — "fix 1.88/1.82/absent skew — P19-B5" (the dispatched MSRV unification
  target was 1.88).
- The build-driven correction, verbatim in
  `stardf/arrow/Cargo.toml:204-217`: upstream purrdf root
  `[workspace.package]` edition is 2024 and ALL upstream crates inherit it —
  vendored sources use let-chains (witnessed: stardf-gts E0658-class "let
  chains are only allowed in Rust 2024"), so the inherited edition must be
  2024 for the vendored closure to compile; `edition = "2024"` (line 210).
  MSRV: "Max of member-declared rust-version values (19 crates declare 1.98;
  the dispatch's '1.88' understated it — re-read from manifests at fix
  time)"; `rust-version = "1.98"` (line 217). Root package inherits via
  `rust-version.workspace = true` (line 190).
- The 1.98 MSRV is load-bearing, witnessed both directions in the latest
  publish receipt (T040521Z): G9 `cargo check --workspace` exit 0 (the
  workspace compiles under the resolving toolchain) while avec-ffi-court
  exits 101 — "stardf-arrow-core@26.11.1 requires rustc 1.98" — because
  `/Users/sac/avec` carries NO rust-toolchain pin and its rustup default
  predates 1.98 (mechanism witnessed in the final receipt update_note).
  Closure owed: a ≥1.98 toolchain pin in the avec checkout before the
  consumer court re-run.
- Toolchain posture unchanged from the operator template's dev/CI intent:
  `stardf/arrow/rust-toolchain.toml` floats nightly with clippy/rustfmt
  components + wasm32/aarch64 targets; MSRV is carried by member
  `rust-version`, not the channel pin.

**Nonclaims.** This lane re-read evidence files only; no cargo run, no court
re-run, no publish, no registry touch, no git mutation. Counts and gate
standings are citations from the named receipts/docs as of their read time,
not re-measurements. D19–D21 record decisions already executed by other
lanes; nothing here executes them.
