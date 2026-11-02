# stardf-arrow — ArrowArrayStream Reconstitution

Ticket: [GL-STARDF-001](tickets/GL-STARDF-001.md) · Plan docs:
[`../docs/jira/v26.10.31/`](../docs/jira/v26.10.31/) · Authority:
`authority/stardf/stardf_contract.ttl`

## What This Is

`stardf-arrow` is the Arrow seam of the stardf reconstitution (ticket
GL-STARDF-001): it reconstitutes the `fn_avec_export_stream` family as a real
`ArrowArrayStream` producer, targeting consumption by
[cgap](https://github.com/seanchatmangpt/cgap) (`cgap-ffi`). `ArrowArrayStream`
does not exist in cgap, purrdf.h, or anywhere upstream — this is a new
contract, not reconstitution of a found symbol (`01-master-plan.md`, W3).
Genre: strangler-fig — route to purrdf by default, shift per family as each is
admitted.

## One-Package State (v26.11.1)

The one-package port landed as commit c3 = `6cbb5b7` ("feat(stardf):
one-package port — purrdf -> stardf, 53 crates straight-copied") on
`fix/v26.9.29-ci-gates`, HEAD c4r `724d5425`. Per-crate git-archive sha256
receipts + rename-only attestations:
[`crates/PROVENANCE-entries.md`](crates/PROVENANCE-entries.md). Final tree:
59 crate dirs on disk (58 `stardf-*` + vendored `jsonschema`; the
`crates/PROVENANCE-entries.md` register rides beside them), 48 workspace
packages (47 members + the root `stardf-arrow` package; `cargo metadata`
exit 0 at 21:46, manufacturing-receipt pass 4). The copied ~80% carries NO
courts (D17, `docs/OPERATOR-DECISIONS.md`): compile-green at best — parity
claims are scoped to the four first-class targets below
(`../docs/jira/v26.10.31/14-definition-of-done.md`, D16 Phase 3).

- `stardf-arrow` (root) — Quad64 batch arena + C Data Interface export
  (`stardf-arrow-core`), 13-export frozen C ABI (`stardf-arrow-capi`,
  `docs/ABI-FREEZE.md`), byte-deterministic five-table columnar codec
  (`stardf-columnar`, `docs/COLUMNAR-ARD.md`).
- `stardf-scan-duckdb` — native in-process `stardf_scan()` DuckDB VTab over
  the five-table columnar profile (`docs/SCAN-COMPLETION.md`).
- `stardf-sql` — SQL-in-RDF compiler, 4 golden courts + 1 doctest
  (`tests/sql_golden.rs`).
- `stardf-n3` — N3 parse/proof/reason surface with forgery courts + the N3
  conformance corpus (`corpora/n3/`, 31 files).

## Standing

Final truth pass (doc lane 15, 2026-10-31 ~22:05 local, post-finish-campaign),
supersedes the lane-8/lane-11 tables (preserved in
[`docs/RECEIPTS.md`](docs/RECEIPTS.md)). Every gate row below was re-run by
this lane THIS session on the final tree — exits and counts quoted from this
lane's runs, logs `/tmp/lane15_doc_{antivac,golden,sql,scan,n3,oracle,r1,verify}.log`
(gate artifacts, log-standing until the coordinator re-bind lands them in
`evidence/`). The cgap gate is NOT re-run (cross-checkout, not cheap) — it
cites landed receipts. Standing vocabulary per `evidence/README.md` law 5.

| Subject | Standing | Receipt (command → exit, 2026-10-31, doc lane 15 runs unless noted) | Court that closes it |
|---|---|---|---|
| arrow (first-class) | ALIVE (local) | `cargo test -p stardf-arrow --test anti_vacuity` → exit 0, **3 passed / 0 failed** (0.95s); core 33/33 + capi all targets (unit 22, cross_allocator 2, c_smoke 1, alloc_identity 2, stream_courts 6) per LANE-11/G04 receipts; `python3 scripts/verify.py` → exit 0 (13==13==13 three-way parity) and `--mutate drop-export` → exit 1 (flip witnessed) — both this lane | anti_vacuity fixture courts + falsifier ledger (G04) + verify.py replay pair (G04/lane-16, byte-identical, sha `cb57ce4e…`) |
| duckdb (first-class) | ALIVE (local) | `cargo test -p stardf-scan-duckdb --locked` → exit 0, **12 passed / 0 failed** (6 lib + 6 courts C1–C6; C3 a loud [SKIP] by landed design, C6 mutant flip witnessed in-run) — this lane, re-witnessing `evidence/g10-scan-courts-rewitness-2026-10-31.json` (exit 0 x2); `just columnar-oracle` → exit 0, **stardf_scan() parity 1030,400,108,108,9** == read_parquet == generator projection — this lane, re-witnessing g10 | scan_courts C1–C6 + columnar-oracle tier-3 in-process registration |
| sql (first-class) | ALIVE (local, run-witnessed) | `cargo test -p stardf-sql` → exit 0, **4 passed / 0 failed (sql_golden) + 1 doctest** — this lane's run is the FIRST LANDED RUN WITNESS on any post-copy tree (prior standing: H01 coordinator-reported PASS, receipt artifact owed per `docs/V26-11-1-GAPS.md` row 16) | sql_golden 4 courts (identifier compilation, envelope pushdown, injection refusal, turtle SELECT) + compiler doctest |
| n3 (first-class) | ALIVE (local, run-witnessed) | `cargo test -p stardf-n3 --locked` → exit 0, **89 passed / 0 failed** across 5 targets (unit 28, forgery 13, parse 28, proofs 4, reason 16; doc-tests 0) — this lane, re-witnessing the H02 run (`/tmp/h02_n3_full.log` @21:44, log-standing) | forgery courts (hardened-default + flip-legacy controls) + purrdf rev 0da05783 per-file byte parity (`crates/stardf-n3/VENDOR-PROVENANCE.md`) |
| cgap integration edge | ALIVE (local) | NOT re-run (cross-checkout). Witnesses: `evidence/g04-falsifier-finalization-2026-10-31.json` — `RUSTUP_TOOLCHAIN=nightly-2026-08-12 cargo test -p cgap-ffi --test stardf_consumer_court` (cwd `/Users/sac/cgap`) → exit 0, **3/3 courts**, v26.11.1 core+capi compiled FROM THIS TREE; ladder receipt `publish_stardf-receipt-20261101T045124Z.json` (21:51) cgap-ffi-court row **PASS, default posture** (exit 0) | ONE real cgap integration test passing (W3 gate) |
| one-package workspace | check-green | c3 = `6cbb5b7` (53 crates straight-copied); G9 `cargo check --workspace` exit 0 in every ladder receipt 20:26→21:51 (newest `…T045124Z.json`); `cargo metadata --no-deps` exit 0 — 48 pkgs: 25 publish-unrestricted / 23 explicit-false ON THE CURRENT TREE (this lane's re-read; as-of H05 21:46 it was 21/27 — the four former G3 crates `stardf-core`, `stardf-rdf`, `stardf-sparql-algebra`, `stardf-sparql-results` flipped publish=true after the newest ladder receipt, a concurrent gate-closure lane's landing; fresh ladder run owed) | G9 in `scripts/publish_stardf.py` + publish ladder G1–G9 |
| publish closure | BLOCKED (operator scope) | ladder receipt `…T045124Z.json` (21:51): G1–G7 + R1 + cgap-ffi-court + G9 PASS, **G8 FAIL** (nonclosure hard deps: rdf→{ed25519,gts,sparql-eval}, sql→shapes, rs→12+ members; registry-404 class); verdict BLOCKED, registry untouched | G8 → operator D17 closure re-ratification |

Nonclaims (all standing is LOCAL): no CI Actions run ever witnessed
(`ci/stardf-arrow-reconstitution.yml` never executed; D2 unresolved); no live
publish, no registry touch (operator double-gate `STARDF_PUBLISH_CONFIRM` +
`--publish`); umbrella `stardf` crate ABSENT (footprint cell, operator);
ABI-v2 PENDING-OPERATOR (D18); the publish-set count drifted post-receipt (21 →
25 publish-unrestricted, four former G3 crates flipped by a concurrent
gate-closure lane — the newest ladder receipt predates it); the sql + n3 +
scan court runs this lane cites
are log-standing until their evidence/ receipt files land; the `=2.0.4`
cache-absent two-arm rehome-court PASS is coordinator-reported (testimony), no
on-disk receipt; full `cargo package`: 4 real tarballs (hash 145,882 B /
columnar 135,460 B / alloc-probe 34,641 B / arrow-core 40,132 B), the other 17
closure cells carry list-level carriage witnesses only; DIVERGENCES
KD-01..15: 8 OPEN / 7 PARTIAL (deferred, D17/full-parity campaign).

## Quickstart

```bash
just build      # debug build of all workspace members
just check      # cargo check across the workspace
just test       # test suite across the workspace
just clippy     # clippy, warnings denied
just fmt        # rustfmt across the workspace
just ci-all     # fmt + clippy + check + test (default verify ladder)
```

Toolchain: floating nightly (`rust-toolchain.toml`), same posture as purrdf;
workspace floor rust-version 1.98 (D21 registered deviation). Targets cover
the cross gates: `wasm32-unknown-unknown` (footprint) and
`aarch64-unknown-linux-gnu` (simd/cross).

## Avec Integration Plan

Per `../docs/jira/v26.10.31/01-master-plan.md` (W3) — court gate MET:

1. **Path-dep edge.** `cgap-ffi` (rlib) consumes `stardf-arrow-capi` by path
   dependency — this workspace is never published; the workspace pins
   `cgap-arrow` by git rev (`67c0ec1`, root `Cargo.toml:263`).
2. **Consumer court.** `cgap-ffi` `stardf_consumer_court` — 3 courts:
   export_stream lifecycle, allocation identity, dlopen + final API. Witnessed
   exit 0 3/3 natively (G04) and default-posture in-ladder (045124Z). The
   cgap-side toolchain pin (`/Users/sac/cgap/rust-toolchain.toml`, 21:42)
   landed the default-posture repair; the exit-101 regression at 21:02–21:42
   (post-version-flip rustc 1.98 floor) is superseded.
3. **Gate.** ONE real cgap integration test passing — MET, witnessed (above).

## Constraint Register

All constraints OPEN until their closing court's receipt lands.

### R1 — License Skew — CLOSED

purrdf is MIT/Apache-2.0/MulanPSL conjunct; cgap is P28-SIL sovereign. CLOSED:
register `authority/stardf/license-compat.json` at 31 dependency_licenses /
40 compatibility_receipts / 0 drafts pending operator; gate
`python3 scripts/verify_stardf_licenses.py` → exit 0 this lane (and x8
witnesses through the campaign). Conjunct notices travel in-package (66/66
canonical license texts, G08 census; NOTICE 22/22).

### R2 — Cross-Allocator Release

Memory allocated under stardf's arena and released through cgap's allocator
(or the reverse) is UB. Closed only by `cross_allocator_court` plus the
`mutant-cross-alloc` positive control — a court that cannot flip is FAIL.
Current witness: `cross_allocator_court` 2/2 (LANE-11) + cgap court_2
allocation-identity leg (G04 3/3). Default-posture court standing rests on
the ladder receipt row (045124Z).

### R3 — Borrowed-Pointer Lifetimes

Exported rowcursor views borrow arena memory; a view held past arena reset is
a dangling pointer. Poison/guard court required. G04 disk re-anchor: no
use-after-advance court exists on disk (`grep` 0 matches) — R3 UNKNOWN stands.

### R4 — Quad 64-Byte Alignment

`Quad64` is `repr(C, align(64))`; compile-time asserts exist and stay pinned
(`crates/stardf-arrow-core/src/layout.rs:58-59` + runtime test `:205`).
Any new export path must re-assert, not assume.

## See Also

- [`../docs/jira/v26.10.31/01-master-plan.md`](../docs/jira/v26.10.31/01-master-plan.md)
  — waves W1-W5, final truth pass banner
- [`../docs/jira/v26.10.31/04-falsifiers.md`](../docs/jira/v26.10.31/04-falsifiers.md)
  — falsifier ledger (finalized ALIVE-scope by G04)
- [`14-definition-of-done.md`](../docs/jira/v26.10.31/14-definition-of-done.md)
  — spec of record (amended; D16 Phase 3 four-target scope)
- [`docs/V26-11-1-GAPS.md`](docs/V26-11-1-GAPS.md) — final consolidator
  register (G20)
- [`docs/V26-11-1-VERDICT.md`](docs/V26-11-1-VERDICT.md) — campaign verdict
- [`docs/OPERATOR-DECISIONS.md`](docs/OPERATOR-DECISIONS.md) — D1-D21 + D13
  resolution + D19-D21
- [`docs/RECEIPTS.md`](docs/RECEIPTS.md) — receipt ledger (all lanes)
- [`docs/DIVERGENCES.md`](docs/DIVERGENCES.md) — KD-01..15 status
- [`evidence/README.md`](evidence/README.md) — receipt conventions
