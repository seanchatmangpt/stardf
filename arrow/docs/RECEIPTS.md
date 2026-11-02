# RECEIPTS

Manufacturing receipts for this branch. Law: no slice claimed ALIVE without the
real gate output (`stardf/arrow/evidence/README.md`); every wave ends with a
receipt carrying commands, exits, and SHA.

Updated 2026-10-31 ~22:05 local (doc lane 15, final ledger rebuild): the
construction-era standing table is retained below; the finish-campaign ledger
(lanes 01-20 + G/H waves) follows — one row per landed receipt file in
`evidence/`, plus wave-report citations for lanes whose receipt FILES never
landed. Every cited file was opened and re-read by this lane before citation.

## Construction-era standing table (pre-campaign, retained)

| id | subject | standing | receipt | commands / exits | SHA | replay |
|----|---------|----------|---------|------------------|-----|--------|
| LANE-9 | `stardf/arrow` verify.py substrate gate | PARTIAL_ALIVE (local gate only) | `schema: ggen.legacy.stardf.arrow.receipt.v1`, `receipt_sha256: 1b29abcdedfcb4a330a0a0c0845c48fc4dfced26fb246414c9a6365d7ad7d5f9` | `python3 scripts/verify.py` → 0; `python3 scripts/verify.py --mutate drop-export` → 1 (flip witnessed) | receipt file sha256 `b0ad32811cac3f1eb93ea496efa099838110a550fefb9efc738cab1a4800e154` (both runs) | WITNESSED: two `--emit-receipt` runs byte-identical (`cmp` exit 0) |
| LANE-16 | wave-finish full ladder, receipt-time tree | PARTIAL_ALIVE | `evidence/wave-finish-lane16-2026-10-31.json` (schema `ggen.legacy.stardf.arrow.receipt.v1`, tree_digest `332392fa…`) | `cargo test --workspace --no-fail-fast` → 101 (stream_courts court_2 5/6); clippy → 101 (alloc_identity); fmt → 1; verify.py ×2 → 0/0 byte-identical; `--mutate drop-export` → 1; c_smoke 0; cross_allocator 0; core 0 | parent HEAD `c9f8028…`, tree_digest `332392fa9da7…` | WITNESSED (manual cmp of two stdout captures; `--emit-receipt` flag had not landed at receipt time) |
| LANE-11 | completion truth pass #2 on the final tree | PARTIAL_ALIVE | this row + README standing table (2026-10-31) | `cargo test -p stardf-arrow-core` → 0 (33 passed, SIGTRAP gone); `cargo test -p stardf-arrow-capi` → 0 (22+2+1+2+6, all targets green); `cargo test --workspace --no-fail-fast` → 1 (anti_vacuity 1/3, fixture drift — only red); `cargo fmt --all -- --check` → 0; `cargo clippy --workspace --all-targets -- -D warnings` → 0 (final run; an earlier same-session run exited 101 on bench doc-comment overindent before a concurrent lane's fix landed); `python3 scripts/verify.py` ×2 → 0/0, `cmp` → 0, 13==13 parity; `--mutate drop-export` → 1 (flip witnessed); bench `cargo run --release` → 0, min-of-5 40.141 GiB/s-quad | parent HEAD `c9f8028…`; `stream_c.rs` sha256 `6a84c661…`, `ontology.ttl` sha256 `cfdd8a75…` at receipt time | WITNESSED for verify.py pair (`/tmp/l11_v1.out` vs `/tmp/l11_v2.out`, byte-identical) |
| LANE-8 | columnar truth pass #3: contract gates re-run on the landed columnar tree; docs reconciled to as-built | PARTIAL_ALIVE | this row + README standing table + `docs/DIVERGENCES.md` status table + `docs/COLUMNAR-ARD.md` header (2026-10-31) | `cargo test -p stardf-columnar --test golden_bytes` → 0 (6 passed / 0 failed); `cargo test -p stardf-arrow-capi --test columnar_integration` → 0 (1 passed); `cargo test -p stardf-columnar` → 0 (3 lib + 6 golden + 4 seam); `cargo test --workspace --no-fail-fast` → 0 (82 passed / 0 failed / 1 ignored across 12 targets — anti_vacuity 3/3 witnessed green, prior fixture drift repaired by its owning lane); `bash scripts/verify_blake3_identity.sh` → 0 (1 blake3 identity v1.8.7; WARN Cargo.toml 1.8.5 vs lock 1.8.7); `bash scripts/check-columnar-oracle.sh` → 0 (DuckDB reads fixture, counts=6,3,0,0,2) | parent HEAD `c9f8028…` (unchanged; stardf/ untracked in parent); no git operations by lane 8 | WITNESSED for all listed runs (outputs quoted in the README table and DIVERGENCES status rows this lane wrote) |

Construction-era nonclaims (retained; superseded where the finish-campaign
ledger differs): no external production standing; no CI-gate standing (local
runs only); the W4 equivalence ladder and W5 two-run CI replay were OPEN at
these rows' write time (W4's header-freeze court is since WITNESSED — lane-16
ledger + G04; W5 CI-in-Actions stays N/A REGISTERED). The anti_vacuity fixture
repair was its owning lane's landing (lane 8 only witnesses the green run).

## Finish-campaign ledger (lanes 01-20 + G/H waves, 2026-10-31)

Subject: HEAD `724d542540b497f2928a63b5aece9b37aef4019b` (c4r), branch
`fix/v26.9.29-ci-gates`, working tree uncommitted for the finish wave
(coordinator owns git; no receipt lane ran git mutations). One row per landed
receipt file in `evidence/` (§A); lanes whose receipt FILES never landed cite
the on-disk wave-report record (§B). Publish ladder receipts are outputs of
the `scripts/publish_stardf.py` harness (gates G1-G9 + R1 + avec-ffi-court).

### §A — one row per landed receipt file

| # | receipt file | lane | standing (as written) | what it witnesses (opened + re-read by this lane) |
|---|---|---|---|---|
| 1 | `evidence/lane11-license-carriage-2026-10-31.json` (20:12) | 11-license-carriage r1 | PARTIAL_ALIVE | license texts copied to 22 crates lacking carriage; 20/20 closure list-gate; conjunct notice dirs; first real tarball (stardf-hash); `cargo package --list` dirty-tree refusal witnessed; testkit full-package FAIL row (honest) |
| 2 | `evidence/lane07-publish-metadata-2026-10-31.json` (20:14) | 07-publish-metadata | (checks only) | metadata parse+inherit 47 pkgs; closure metadata 19/19; publish-list gate 19/19; 3/19 full package exit 0; 16 registry/dual-key blockers (honest false rows); umbrella-stardf cell false |
| 3 | `evidence/publish_stardf-receipt-20261101T032659Z.json` (20:26) | publish ladder run 1 | BLOCKED | first ladder run: G2/G3/G5/G7/G8/R1 red; avec-ffi court PASS (first green witness 20:26) |
| 4 | `evidence/publish_stardf-receipt-20261101T032924Z.json` (20:29) | publish ladder run 2 | BLOCKED | G9 transient FAIL on concurrent edits (compile-freeze window disclosed) |
| 5 | `evidence/publish_stardf-receipt-20261101T033004Z.json` (20:30) | publish ladder run 3 | BLOCKED | landing list G3/G5/G8/R1; avec court PASS (20:30) |
| 6 | `evidence/publish_stardf-receipt-20261101T035046Z.json` (20:50) | publish ladder run 4 | BLOCKED | same 3-row landing list; G9 PASS on the scan-member tree; avec court PASS (4th green 20:50) |
| 7 | `evidence/publish_stardf-receipt-20261101T040225Z.json` (21:02) | publish ladder run 5 | BLOCKED | avec-ffi-court FAIL first appears (post-version-flip rustc 1.98 floor, exit 101) |
| 8 | `evidence/publish_stardf-receipt-20261101T040456Z.json` (21:04) | publish ladder run 6 | BLOCKED | avec FAIL + G3/G5/G8; R1 PASS (21:04) |
| 9 | `evidence/publish_stardf-receipt-20261101T040521Z.json` (21:05) | publish ladder run 7 | BLOCKED | G9 `cargo check --workspace` exit 0 (21:05, current tree); R1 PASS; avec FAIL exit 101; 3-row landing list |
| 10 | `evidence/lane16-falsifier-ledger-2026-10-31.json` (21:10) | 16-falsifier-ledger | ALIVE (per-row scope) | F4 header-freeze check + `--court` FIRST witness; F5 `--emit-receipt` pair byte-identical on the current tree; F1 census 22, F1a drop-type refusal, F2 bridge gates 5/5 re-witnessed; avec-gate re-open witness (honest false row) |
| 11 | `evidence/lane11-license-carriage-r2-2026-10-31.json` (21:14) | 11-license-carriage r2 | PARTIAL_ALIVE | R1 baseline exit 1 → register edit → exit 0 flip; 66/66 canonical license census; 22/22 list gate with byte-identical replay pair; 3 real leaf tarballs; PROVENANCE.md added to scan-duckdb; 19 non-leaf refusals classified |
| 12 | `evidence/g04-falsifier-finalization-2026-10-31.json` (21:39) | G04-falsifier-finalization | ALIVE (ledger finalization scope) | EVERY F-row re-witnessed on the current tree: F1 census 22/128/126/SHACL-0; F1a drop-type REFUSED (4th witness); F2 5/5; F4 check + 3-shim court (A/B/C each flipped, restore byte-identical); F4b golden_bytes 5/0/1i + byte-flip control; F5/F5a verify.py pair byte-identical (sha `cb57ce4e…`); F3/F3a avec court closed-out natively — `RUSTUP_TOOLCHAIN=nightly-2026-08-12 cargo test -p avec-ffi --test stardf_consumer_court` → exit 0 **3/3** from `/Users/sac/avec` with v26.11.1 core+capi compiled FROM THIS TREE; R1 exit 0; R3/R4/KD-14 disk re-anchors |
| 13 | `evidence/publish_stardf-receipt-20261101T043905Z.json` (21:39) | publish ladder run 8 | BLOCKED | G3/G5/G8 + avec default-posture FAIL; R1 PASS (21:39) |
| 14 | `evidence/publish_stardf-receipt-20261101T044033Z.json` (21:40) | publish ladder run 9 | BLOCKED | same gate state (21:40) |
| 15 | `evidence/publish_stardf-receipt-20261101T044209Z.json` (21:42) | publish ladder run 10 | BLOCKED | newest PRE-decouple ladder receipt (11 gate rows); R1 PASS; G9 PASS; avec default-posture FAIL (ordering ambiguity vs the 21:42 pin mtime) |
| 16 | `evidence/g08-license-carriage-verify-2026-10-31.json` (21:50) | G08 license-carriage verify | ALIVE | R1 exit 0; 66/66 census with mutation NON-VACUITY control; NOTICE 22/22; conjunct dirs 3; provenance rows 22/22; list gate 22/22; 4 real tarballs (hash 145,882 / columnar 135,460 / alloc-probe 34,641 / arrow-core 40,132 B NEW post-decouple); 18 non-leaf refusals classified; census non-vacuity control witnessed |
| 17 | `evidence/g10-scan-courts-rewitness-2026-10-31.json` (21:48) | G10 scan-courts re-witness | ALIVE | duckdb target FIRST LANDED RUN RECEIPT: `cargo test -p stardf-scan-duckdb --locked` → exit 0 (6 lib + 6 courts C1-C6; C3 loud [SKIP] by landed design; C6 mutant flip WITNESSED IN-RUN); 2nd independent scan_courts run exit 0; `just columnar-oracle` → exit 0, tier-3 in-process registration, counts **1030,400,108,108,9**, scan parity stardf_scan() == read_parquet == generator projection; per-file sha256 digests before/after byte-identical |
| 18 | `evidence/g07-tarball-audit-2026-10-31.json` (re-landed 21:57) | G07 tarball audit | PARTIAL_ALIVE | 22/22 list gate + in-tarball license shas canonical (9) + tarball census/replay; exclude tables audited (6); class-1 eliminated mid-lane by lane-08 (4 rows); 19 publish blockers re-witnessed + 2 drift rows + receipt-write corruption disclosed (honest false rows). GAPS row 11 called the earlier 424B landing a truncated stub — the re-landed receipt at 21:57 supersedes that reading |
| 19 | `evidence/v26.11.1-manufacturing-receipt.json` (pass 4, ~21:56) | lane 17 pass 1-2 → G12 pass 3 → H05 pass 4 | PARTIAL_ALIVE | final consolidation receipt: c-chain c1..c4r identity; tree digest pass 4 `d330d234…` over 12,264 files (21:55:23); R1 gate x8 witnesses + register closure 31/40/0; D13/D19 `=2.0.4` x3 pins `Cargo.toml:282-284`; 21-crate publish-set metadata (48 pkgs: 21/27); scan courts first-run log-standing + n3 first-run log-standing (pass 4); sql unrun at pass-4 probe time (superseded by doc lane 15's run, README table); verify.py replay WITNESSED on the current tree; honest error rows (AVEC_DEFAULT_POSTURE_STILL_RED, PUBLISH_LANDING_LIST_OPEN, UMBRELLA_STARDF_ABSENT, REHOME_TWO_ARM_COURT_TESTIMONY_STANDING) |
| 20 | `evidence/publish_stardf-receipt-20261101T045124Z.json` (21:51) | publish ladder run 11 | BLOCKED | FINAL ladder receipt: G1-G7 + R1 + avec-ffi-court (default posture, exit 0) + G9 all PASS; **G8 the only FAIL** (nonclosure hard deps, registry-404 class); verdict BLOCKED; registry untouched |
| 21 | `evidence/lane8-blake3-align-2026-10-31.json` (~22:57) | publish lane 8 — blake3 version-pin alignment | PARTIAL_ALIVE (gate surface ALIVE) | blake3 declared 1.8.5 → 1.8.7 in 2 member manifests (arrow-core:32, columnar:47) matching the lock's already-resolved 1.8.7 — WARN cleared, alignment not upgrade; gate `verify_blake3_identity.sh` exit 0 ×3, zero WARN, `[PASS] exactly 1 blake3 identity: blake3 v1.8.7` + `[PASS-INFO] declared==locked: 1.8.7`, replay pair cmp exit 0; declaring crates compile `--locked` exit 0 (10m 13s); CROSS-LANE UNBLOCK disclosed: dangling `cgap-arrow.workspace = true` removed from stardf-process (decouple residue had broken whole-workspace resolution ~62 min) — honest-failure row `cargo check -p stardf-process` exit 101, 30 cgap_arrow errors, attributed Lane 09; root-pin alternative falsified (avec@67c0ec1 lacks cgap-arrow; crates.io 404) and probe reverted byte-exact |

### §B — no-receipt lanes (wave-report citations)

Lanes whose receipt FILES never landed in `evidence/` (work recorded from
on-disk artifacts, landed receipts, coordinator testimony, and the parent
session's wave reports — the campaign law "run receipts are in flight, never
read as landed" applies):

| lane | subject | standing | citation (opened by this lane) |
|---|---|---|---|
| 01-duckdb-pins | `frozen-duckdb/-sys/-builder = "=2.0.4"` x3 workspace pins | LANDED (pins on disk) | `docs/OPERATOR-DECISIONS.md:648` D13 RESOLUTION + manufacturing-receipt check `d13-duckdb-pins-ratified` (pins re-read `Cargo.toml:282-284`); G6 PASS in every ladder receipt |
| 02-scan-scaffold | `crates/stardf-scan-duckdb` crate (lib.rs 1,012 lines + scan_courts.rs 841 lines, six courts c1-c6) | LANDED; courts first-ran green (g10 receipt + G15 log) | manufacturing-receipt check `duckdb-scan-scaffold` + `docs/V26-11-1-GAPS.md` DONE row 6 |
| 05-avec-gate | avec consumer-court repair (path-dep + toolchain wrap) | REPAIRED-BY-WRAP; default posture green in-ladder | receipt file NEVER LANDED across four passes (mfg-receipt pass-4 disclosure); work witnessed by g04 check `f3-f3a-avec-gate-native-posture-rerun` + ladder receipts; avec-side pin `/Users/sac/avec/rust-toolchain.toml` (749 B, 21:42) |
| 06-msrv | root package `rust-version.workspace = true` | LANDED (manifest comment) | root `Cargo.toml` "Lane 06-msrv" comment |
| 08-dual-key | 46→25 stardf-* dual-key workspace dep rows at `=26.11.1` | LANDED (workspace table on disk) | manufacturing-receipt pass-3/4 notes + GAPS DONE row 13 (G06 coordinator-reported 25/25); `Cargo.toml` [workspace.dependencies] |
| 09-avec-decouple | avec-arrow removed from stardf-arrow-core + stardf-arrow-capi manifests (G5 avec-git-dep sub-row) | LANDED postdates newest ladder receipt | manufacturing-receipt pass-4 check `publish-prep` (grep-witnessed core `:21-36` / capi `:28-35` comments) + g07 (arrow-core now packages, real tarball) |
| 15-rehome-note / arbiter | D13 resolution block + root `autoexamples = false` | LANDED | `docs/OPERATOR-DECISIONS.md:648` D13 RESOLUTION; root `Cargo.toml` "Lane 15 arbiter fix" comment |
| 17 / 19 / 20 | manufacturing-receipt passes; lane-19 draft consolidation; lane-20 verdict | superseded chain | passes recorded in `evidence/v26.11.1-manufacturing-receipt.json` (`lane17_pass*`/`g12_pass3` digest records); draft superseded per its own note; `docs/V26-11-1-VERDICT.md` (revised 21:34) — stale where GAPS rows differ (avec, G3/G5, duckdb courts, replay), per GAPS as-of disclosure |
| G01/G03/G05/G06/G11 | metadata probes, check-loop settles, dual-key gate | work witnessed via landed receipts | manufacturing-receipt commands[] (cargo metadata exit 0, 48 pkgs) + GAPS DONE row 13; no separate receipt files landed |
| G12 | manufacturing-receipt pass 3 | WITNESSED (inside the receipt) | `evidence/v26.11.1-manufacturing-receipt.json` pass-3 update note + commands[] |
| G15 | scan courts + anti_vacuity + oracle first runs | LOG-STANDING | `/tmp/g15-scan-courts.log` @21:42 (6/0), `/tmp/g15-anti-vacuity.log` @21:42 (3/0), `/tmp/g15-oracle-shim.log`; superseded as receipt-standing by g10 + doc lane 15's re-runs |
| G19 | commit-series post-mortem | LANDED (doc) | `docs/COMMIT-PLAN.md` post-mortem (G19 finish lane): chain c1..c4r verified in history; flag 2 (bench member lock) + flag 6 (docs/jira c0) open; e06c257 violation REGISTERED |
| G20 | final consolidator register | LANDED (doc, ALIVE) | `docs/V26-11-1-GAPS.md` (written ~21:53): arrow + duckdb ALIVE-LOCAL; sql testimony; n3 in flight; publish BLOCKED at G8 only |
| H01 | sql target courts (sql_golden) | PASS-BY-TESTIMONY → RUN-WITNESSED | coordinator-reported PASS, receipt artifact not landed (GAPS row 16); NOW witnessed by doc lane 15's own run: `cargo test -p stardf-sql` → exit 0, 4 goldens + 1 doctest (`/tmp/lane15_doc_sql.log`) — sql target court-green; evidence/ receipt file still owed |
| H02 | n3 target suite (forgery/parse/proofs/reason) | RUN-WITNESSED (log → re-run) | `/tmp/h02_n3_full.log` @21:44 (89/0 exit 0, log-standing) + doc lane 15's re-run: `cargo test -p stardf-n3 --locked` → exit 0, 89/0 across 5 targets (`/tmp/lane15_doc_n3.log`); evidence/ receipt file still owed |
| H03 | gate-closure lane | IN FLIGHT at this lane's read | /tmp artifacts (h03-cargo.lock.bak 21:59, h03-ch 22:02); no receipt; do not cite as landed. OBSERVED EFFECT of its in-flight work (doc lane 15 re-read, cargo metadata exit 0): the four former G3 crates (stardf-core, stardf-rdf, stardf-sparql-algebra, stardf-sparql-results) are now publish=true — publish-set 25 unrestricted / 23 explicit-false on the current tree vs 21/27 as-of H05 21:46; fresh ladder run owed to re-judge |
| H04 | c5r/c6 staging classification | LANDED (doc) | `docs/C5R-C6-STAGING.md` (task H04, lane lane19): 1,448-path snapshot, per-bucket add commands + gates |
| H05 | manufacturing-receipt pass 4 (post-settlement re-bind) | WITNESSED (inside the receipt) | `evidence/v26.11.1-manufacturing-receipt.json` pass-4 update note |
| DOC-15 | this ledger + README final rebuild | ALIVE (local) | this file + README standing table; this lane's own gate runs: anti_vacuity 3/0 exit 0, golden_bytes 5/0/1i exit 0, sql 4+1dt exit 0, scan 12/0 exit 0, n3 89/0 exit 0, `just columnar-oracle` exit 0 (counts 1030,400,108,108,9), R1 exit 0, verify.py exit 0 + `--mutate drop-export` exit 1 — logs `/tmp/lane15_doc_*.log` |

## Nonclaims (campaign-final)

- All standing is LOCAL: no CI Actions run ever witnessed; no external
  production standing; no live publish, no registry touch (operator
  double-gate); no git mutations by any receipt lane.
- The sql + n3 + scan court runs cited from `/tmp` logs are LOG-STANDING until
  their `evidence/` receipt files land (doc lane 15's runs included — its runs
  are cited as this lane's commands, the receipt file for the DOC-15 pass is
  this file + the README table).
- Publish closure BLOCKED at ONE gate (G8 nonclosure hard deps, operator D17
  scope); 4 real tarballs exist; the other 17 closure cells carry list-level
  carriage witnesses only; umbrella `stardf` crate ABSENT (footprint cell,
  operator); ABI-v2 PENDING-OPERATOR (D18).
- The `=2.0.4` cache-absent two-arm rehome-court PASS is coordinator-reported
  testimony — no on-disk run receipt; the witnessed falsifier event is F04
  firing cache-absent on `=2.0.3`.
- R1 exit 0 is witnessed on the UNCOMMITTED working tree only.
- Copied ~80% carries NO courts (D17): compile-green at best; parity scoped to
  the four first-class targets. DIVERGENCES KD-01..15: 8 OPEN / 7 PARTIAL,
  deferred (full-parity campaign
  `../../docs/jira/v26.10.31/12-full-parity-campaign.md`).

## See Also

- [`../README.md`](../README.md) — standing table (four first-class targets)
- [`../evidence/README.md`](../evidence/README.md) — receipt conventions
- [`V26-11-1-GAPS.md`](V26-11-1-GAPS.md) — final consolidator register
- [`V26-11-1-VERDICT.md`](V26-11-1-VERDICT.md) — campaign verdict
- [`C5R-C6-STAGING.md`](C5R-C6-STAGING.md) — c5r/c6 staging classification
- [`COMMIT-PLAN.md`](COMMIT-PLAN.md) — commit series + post-mortem
