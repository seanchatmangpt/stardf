# 01-publish-blockers-register

Authoritative publish-blockers register for the stardf v26.11.1 publish
campaign. Subject: `stardf/arrow` workspace at HEAD `724d542540b497f2928a63b5
aece9b37aef4019b` (c4r), branch `fix/v26.9.29-ci-gates`. Register of truth
for the gate standings: `arrow/evidence/publish_stardf-receipt-20261101T051252Z.json`
(verdict BLOCKED; G8 the sole hard FAIL, `landing_list =
["G8-nonclosure-hard-deps"]`), cross-checked against
`arrow/docs/V26-11-1-GAPS.md` REMAINING #1/#4/#5. As-of: written 2026-10-31
(UTC receipts 2026-11-01T05:12:52Z); every count re-read from disk this
session, per the receipt-cited-counts law.

Standing up front: **publish BLOCKED at exactly ONE hard gate (G8), plus
three advisory soft gates.** The DoD execution note states the same:
"Publish matrix — BLOCKED on exactly two items: the G8 scope decision and the
operator publish go. No other blocker is outstanding"
(`stardf/docs/jira/v26.10.31/14-definition-of-done.md:183-184`).

Owner column: **OPERATOR** on every row — these are scope/footprint
decisions, not lane-executable fixes. The in-repo G8 authority is the H03
registration in `arrow/docs/OPERATOR-DECISIONS.md` ("G8 is registered here as
PENDING-OPERATOR — it is not manifest-reachable").

## 1. G8-nonclosure-hard-deps — the sole hard gate

**Receipt:** `arrow/evidence/publish_stardf-receipt-20261101T051252Z.json`,
gates[] row `G8-nonclosure-hard-deps` (class hard, standing FAIL), `fails`
array = **24 rows**, verdict BLOCKED. The console detail is capped at 12 rows
("+12 more"); the JSON `fails` array carries the full enumeration (G17 fix,
`publish_stardf.py:143-149`). Gate law (`publish_stardf.py:329-334`): a
closure crate's dependency on a workspace member outside the publish closure
is publish-blocking; **optional rows count** because `cargo publish` rewrites
a dual-keyed optional path dep to version-only and crates.io 404s at upload
when that version was never published. Dev rows count too — cargo publish
resolves dev-deps at package time and crates.io validates them at upload
(`publish_stardf.py:42-45`).

### 1.1 Why not manifest-fixable (source-woven)

Per the H03 registration (`OPERATOR-DECISIONS.md`,
"G8-nonclosure-hard-deps-registered-with-owner", count 24): the gate is NOT
closeable by manifest-only fixes — every G8 dep is source-woven, and every
G8 dep names a member the registry will never carry at its current pin.
Witnessed with real greps this session:

| consumer | site | usage |
|---|---|---|
| stardf-rdf -> stardf-ed25519 | `src/gts_compose.rs`, `src/gts_dict_vectors.rs` | snapshot signing path; the same impl stardf-gts signs/verifies with |
| stardf-rdf -> stardf-sparql-eval | `src/capture_support.rs`, `src/bin/capture_sparql_goldens.rs`, `src/projections/dataset_description.rs` | the SOLE SPARQL authority: projections + capture bin |
| stardf-rdf -> stardf-gts (dev) | `tests/pinned_dict_compaction.rs`, `tests/rdf12_canon_profile.rs`, `tests/gts_multi_binding_reifier.rs` | GTS vector/canon test oracles |
| stardf-sql -> stardf-shapes | `src/compiler.rs` | SHACL compiler path |
| stardf-rs (umbrella) | `src/lib.rs` — 41 `pub use` lines | 17 hard + 1 optional re-export surface |

Table notes: all sites under `crates/stardf-rdf/`, `crates/stardf-sql/`,
`crates/stardf-purrdf/` respectively. The rdf dev row survives rdf's
`exclude = ["tests/**"]` — `exclude` strips files from the package, never
dev-deps from the manifest. The umbrella's purpose is its own manifest's
consumer contract: "a consumer depends on purrdf alone" (SigningKey from 32
raw Ed25519 bytes, per the stardf-rdf manifest comment on the ed25519 edge).

### 1.2 The 24 rows (receipt fails[], transcribed; pins re-read from root Cargo.toml)

| # | consumer | kind | target member | root pin |
|---|---|---|---|---|
| 1 | stardf-rdf | hard normal | stardf-ed25519 | `=26.9.29` (`Cargo.toml:345`) |
| 2 | stardf-rdf | hard normal | stardf-gts | `=26.11.1` |
| 3 | stardf-rdf | hard normal | stardf-sparql-eval | `=26.11.1` |
| 4 | stardf-rdf | hard dev | stardf-gts | `=26.11.1` |
| 5 | stardf-sql | hard normal | stardf-shapes | `=26.9.29` (`Cargo.toml:365`) |
| 6 | stardf-rs | hard normal | stardf-datalog | `=26.9.29` (`Cargo.toml:337`) |
| 7 | stardf-rs | optional normal | stardf-duckdb | `=26.11.1` |
| 8 | stardf-rs | hard normal | stardf-entail | `=26.11.1` |
| 9 | stardf-rs | hard normal | stardf-geo | `=26.11.1` |
| 10 | stardf-rs | hard normal | stardf-glob | `=26.11.1` |
| 11 | stardf-rs | hard normal | stardf-gts | `=26.11.1` |
| 12 | stardf-rs | hard normal | stardf-hnsw | `=26.11.1` |
| 13 | stardf-rs | hard normal | stardf-json | `=26.11.1` |
| 14 | stardf-rs | hard normal | stardf-markdown | `=26.11.1` |
| 15 | stardf-rs | hard normal | stardf-mime | `=26.11.1` |
| 16 | stardf-rs | hard normal | stardf-process | `=26.11.1` |
| 17 | stardf-rs | hard normal | stardf-retrieval | `=26.11.1` |
| 18 | stardf-rs | hard normal | stardf-shapes | `=26.9.29` (`Cargo.toml:365`) |
| 19 | stardf-rs | hard normal | stardf-shex | `=26.11.1` |
| 20 | stardf-rs | hard normal | stardf-slice | `=26.9.29` (`Cargo.toml:367`) |
| 21 | stardf-rs | hard normal | stardf-sparql-eval | `=26.11.1` |
| 22 | stardf-rs | hard normal | stardf-text | `=26.11.1` |
| 23 | stardf-rs | hard normal | stardf-validate | `=26.11.1` |
| 24 | stardf-toml | hard dev | stardf-sparql-eval | `=26.11.1` |

Row arithmetic: stardf-rdf 4 (3 normal + 1 dev), stardf-sql 1, stardf-rs 18
(17 hard + 1 optional), stardf-toml 1 (dev) = 24. Distinct target members:
19. Four members sit at `=26.9.29` root pins (datalog, ed25519, shapes,
slice); the rest at `=26.11.1`.

### 1.3 Witnessed publish blast radius (bootstrap simulation)

`arrow/evidence/h03-bootstrap-sim-2026-10-31.txt` (generator
`scripts/h03_bootstrap_sim.sh`): real `cargo package --offline --allow-dirty
--no-verify` per closure crate in TIERS order, publish-shaped resolution
against a local directory source seeded with already-packaged siblings.
**20/25 closure crates produced real .crate tarballs, every one under 10 MB
with LICENSE rows inside.** The 5 FAIL rows are the G8 blast radius with
cargo's own errors: stardf-rdf (no package named `stardf-ed25519`), stardf-rs
(no package named `stardf-datalog`), stardf-sql + stardf-toml + stardf-n3
(blocked transitively via stardf-rdf). Naive per-crate packaging fails the
whole non-leaf class on the same wall; the sim is the strongest pre-publish
witness available without a registry touch.

### 1.4 The three operator options

#### Option A — join the closure (D17 re-ratification) — RECOMMENDED

Re-ratify the D17 publish closure to admit the **20 members** = the 19 G8
targets + **stardf-geo-kernel** (recursive joiner witnessed this session:
stardf-geo, stardf-sparql-eval and stardf-validate each declare
`stardf-geo-kernel` dep keys; root key `Cargo.toml:359` is path-only, no
version). Option A is the only option where the published surface equals the
workspace surface, needs zero source surgery, and flips G8 by construction —
the gate re-derives from live `cargo metadata` at run time.

**Exact manifest mechanics (manifest lane, then ladder re-run):**

1. Operator appends the D17 re-ratification row to
   `arrow/docs/OPERATOR-DECISIONS.md` (append-only register) admitting the 20
   members.
2. `publish = true` on 20 manifests — all 20 currently `publish = false`
   (re-read on disk this session: ed25519, gts, sparql-eval, shapes, datalog,
   duckdb, entail, geo, glob, hnsw, json, markdown, mime, process, retrieval,
   shex, slice, text, validate, geo-kernel).
3. Four members at literal `26.9.29` flip to `version.workspace = true` and
   their root keys re-pin: `Cargo.toml:337` (datalog), `:345` (ed25519),
   `:365` (shapes), `:367` (slice), `=26.9.29` -> `=26.11.1`.
4. `stardf-glob` + `stardf-process`: add `keywords` (1..5) and `categories`
   (1..10) — the only two of the 20 whose metadata misses the G4 bounds
   (re-read on disk; the other 18 carry description/keywords/categories).
5. `stardf-geo-kernel` root key dual-keyed: `Cargo.toml:359`
   `stardf-geo-kernel = { path = ... }` -> version+path at `=26.11.1` (G5
   dual-key law; the member itself is metadata-complete,
   `version.workspace = true`).
6. `stardf-process` decouples `cgap-arrow` (lane-09 class: a git-source dep
   cannot publish; lane 09 removed it from stardf-arrow-core + capi the same
   way — `crates/stardf-arrow-core/Cargo.toml:21-36`,
   `crates/stardf-arrow-capi/Cargo.toml:28-30`). stardf-process is the last
   cgap-arrow consumer (`crates/stardf-process/Cargo.toml:20`).
7. `scripts/publish_stardf.py` `TIERS` extended with the 20 members in
   strict topo order (G2 re-derives every intra-closure edge from live
   metadata and refuses on any backwards/sideways edge — placement is
   checked, not trusted).
8. Carriage census + package-sim extended to the 45-crate set; ladder re-run:
   G8 re-derives PASS; verdict path to GREEN.

**Consequences:** crates.io footprint 25 -> 45 crates (45 name claims); every
joiner inherits the dual-key `=26.11.1` graph; joiners carry the D17
copied-crate bar (compile-green only, no courts); frozen-duckdb `=2.0.4` x3
rides in stardf-duckdb (G6 law extends to it); bootstrap order = extended
tier order; consumers reach the full parity surface from crates.io.

#### Option B — drop + feature-gate (scope the footprint down)

Remove the 24 manifest rows and surgically remove/gate the source sites.
**Load-bearing mechanic (from the gate's own law): feature-gating alone does
NOT clear a G8 row** — optional deps still 404 at upload — so the rows must
be REMOVED, and the capability work is source surgery:

- stardf-rs: 41 re-export lines removed -> the umbrella narrows to a shell
  and loses its stated purpose ("a consumer depends on purrdf alone");
  geo/text/hnsw/retrieval/json/mime/markdown/glob/process/slice/datalog-type
  reachability lost.
- stardf-rdf: GTS compose/dict-vector signing path + SPARQL golden-capture
  bin removed or stubbed; dev rows (gts) moved to `tests/disabled/` (tree
  precedent) or rewritten against published siblings.
- stardf-sql: SHACL compiler path removed from compiler.rs.
- stardf-toml: roundtrip test's SPARQL-eval oracle moved/disabled.

**Exact manifest mechanics:** delete the 24 dep rows (4 manifests), delete or
`tests/disabled/` the orphaned source/test sites listed in 1.1, re-run the
ladder; the bootstrap sim must be re-run (all 5 blocked crates resolve after
surgery, so the sim's FAIL set empties).

**Consequences:** published surface != workspace surface — a standing D15
one-package divergence (D15: "No carve, no feature-gating by dependency")
requiring an operator amendment registering the publish-realm exception;
permanent surgery reapplied at every re-vendor; consumers lose SHACL, GTS,
SPARQL capture, GeoSPARQL, HNSW, full-text, N3 reachability from the
umbrella; the umbrella loses its stated consumer contract.

#### Option C — re-route to registry crates

Replace the in-tree target with a published registry equivalent. **Witnessed
empty as a standalone resolution:** the closest candidate, stardf-ed25519, is
a NATIVE RFC 8032 implementation ("deterministic signing over constant-time
radix-2^51 field and Montgomery scalar arithmetic, strict cofactorless
verification" — `crates/stardf-ed25519/Cargo.toml` description), not a thin
wrapper; re-routing stardf-rdf/stardf-gts to `ed25519-dalek` would be a
crypto-identity swap needing a signing-parity court and a co-swap in
stardf-gts (which signs/verifies with the same implementation). No registry
equivalents exist for first-party crates (gts, shapes, sparql-eval, datalog,
entail, geo(+kernel), hnsw, text, retrieval, shex, validate, json, mime,
markdown, glob, slice, process, duckdb). Option C survives only as a per-edge
tactic inside Option B (a registry crate replaces a dropped first-party dep,
each with its own parity court). Recommendation: do not select.

### 1.5 Options comparison

| | A join (recommended) | B drop+gate | C re-route |
|---|---|---|---|
| G8 rows cleared | 24/24 by construction | 24/24 after surgery | none standalone |
| Source surgery | none | 4 crates, witnessed sites | 4 crates + parity courts |
| D15 one-package | consistent | divergence, needs amendment | divergence |
| Footprint | 25 -> 45 | 25 or fewer | n/a |
| New version bumps | 4 (`=26.9.29` -> `=26.11.1`) | none | per-edge |
| Re-vendor cost | none | surgery reapplied per rev | courts reapplied |
| Ladder flip | G8 PASS by re-derivation | G8 PASS after surgery | no flip |

### 1.6 Recommendation

**Select Option A.** Rationale: only D15-consistent option; zero source
surgery; fully mechanical manifest diff (publish flags x20, 4 version bumps,
2 metadata rows, 1 dual-key, 1 decouple, TIERS extension); G8 re-derives from
live metadata so the gate flips by construction once the manifests land; the
bootstrap sim already witnessed the ladder mechanics on 20/25 real tarballs
in tier order. Falsifier for this recommendation: if the operator declines a
45-crate crates.io footprint, Option B's source surgery is the fallback, with
the D15 publish-realm amendment registered first. Option C: empty standalone,
do not select.

**Landing condition (G8 -> GREEN):** operator re-ratification row + manifest
diff landed + `python3 scripts/publish_stardf.py` re-run reading G8 PASS with
`landing_list = []`.

## 2. Soft gates (advisory; none block the ladder flip)

### S1 — stardf-core tarball ceiling (10.25 MB raw vs 10 MB)

- `MAX_PACKAGE_BYTES = 10 * 1024 * 1024` = 10,485,760
  (`publish_stardf.py:76`). The G07 tarball audit
  (`evidence/g07-tarball-audit-2026-10-31.json`, stardf-core row) lists 319
  files / **10,748,822 B raw listed sum** — over the ceiling arithmetic by
  263,062 B. Driver: the disclosed KEPT exception
  `tests/csv_differential_vectors.txt` (1,504,576 B, `include_str!` at
  `tests/csv_differential.rs:75`; excluding it would break published-crate
  test compilation).
- **The gate arithmetic measures pre-compression file sum, while crates.io
  caps the .crate artifact**: the h03 sim witnessed the real stardf-core
  tarball at **2,088,586 B** (318 entries) — comfortably under. G07's own
  nonclaim ("10.25MB listed raw is pre-compression, unwitnessable as tarball
  until bootstrap") is superseded on this cell by the sim.
- **Decision owed (owner OPERATOR):** the dry-run ladder
  (`ladder_dry_run`) sums listed file sizes and FAILs a row over
  `MAX_PACKAGE_BYTES` — post-G8, the stardf-core ladder row will false-fail
  on 10,748,822 > 10,485,760. Either (i) amend `ladder_dry_run` to measure
  the real .crate artifact (matches crates.io semantics, sim-witnessed
  method), or (ii) accept the disclosed strictness and special-case the row.
  Recommended: (i), with the sim's stat-on-tarball method.
- Standing: PARTIAL (arithmetic exposure witnessed; real artifact under
  ceiling, sim-witnessed; ladder amendment pending operator call).

### S2 — blake3 pin (caret req, skew-WARN channel armed)

- Declarations on disk (re-read): `crates/stardf-arrow-core/Cargo.toml:32`
  and `crates/stardf-columnar/Cargo.toml:47`, both
  `blake3 = { version = "1.8.7", default-features = false }` — a caret req,
  not an exact pin; `Cargo.lock` resolves 1.8.7. Declared == locked today,
  so the gate's skew WARN is clear on values — but the caret keeps the
  silent-drift class armed (a future lock regeneration may resolve 1.8.8
  under a `1.8.7` caret; the witnessed 1.8.5->1.8.7 drift is exactly this
  class, per the D10 brief).
- Gate: `scripts/verify_blake3_identity.sh` — identity-count is the FAIL
  channel; declared-vs-locked skew is the advisory WARN channel ("pin
  '=1.8.7' or bump deliberately to clear", script lines 66-75); the D10
  brief's "If A wins" step 1 is the same pin.
- **Decision owed (owner OPERATOR):** pin `=1.8.7` at both declaration sites
  (or bump deliberately + regenerate the lock) to disarm the drift class.
  Note: stardf-columnar's blake3 declaration is outside the gate's
  declared-read (the script reads only the arrow-core manifest) — the pin
  should cover both sites, and the gate optionally extended to read both.
- As-of disclosure: this lane attempted a live gate re-witness and got
  `[ERROR] cargo tree failed (workspace resolution broken — this is not
  absence)`: `crates/stardf-process/Cargo.toml` (edited minutes before the
  read) still declares `cgap-arrow.workspace = true` while the root
  `[workspace.dependencies]` carries no `cgap-arrow` key — a concurrent lane
  mid-edit at write time (transient; the cited receipts bind to the
  05:12:52Z tree where G9 `cargo check --workspace` was exit 0). The gate
  itself is non-vacuous: a broken-resolution error correctly refuses to read
  as VACUOUS absence.
- Standing: PARTIAL (exposure named, fix is a 2-line manifest diff; live
  re-witness pending the concurrent lane's landing).

### S3 — umbrella `stardf` absent (footprint-name cell)

- Witnessed: no crate named `stardf` exists (`grep '^name = "stardf"'` over
  all member manifests: zero hits). The on-disk umbrella is **stardf-rs** at
  `crates/stardf-purrdf` (lib `purrdf`), publish=true, in TIERS T8.
- The contract cell names "stardf": the v26.11.1 manufacturing receipt's
  footprint reality check ("the contract's umbrella cell named `stardf`; the
  crate on disk joined the publish set as `stardf-rs` at 21:00" —
  `evidence/v26.11.1-manufacturing-receipt.json`), and GAPS REMAINING #4
  ("footprint cell unsatisfiable as dispatched").
- **Options (owner OPERATOR):**
  1. **Materialize**: a 1-crate facade named `stardf` re-exporting
     stardf-rs, joining TIERS above T8. Cost: a new crates.io name claim +
     one more tier row; benefit: the contract cell is satisfied verbatim,
     `cargo add stardf` works for consumers.
  2. **Amend** (recommended): amend the contract/footprint row to
     `stardf-rs` (the umbrella already joined the publish set under that
     name at 21:00, receipted). Cost: zero manifest work; the DoD's own
     execution note already reads the finish state as "the G8 decision + the
     publish go", not the umbrella name.
- Standing: PARTIAL (cell unsatisfiable as dispatched; both paths
  mechanical; amendment recommended as zero-cost).

## 3. Footprint reconcile (couples G8-A to the publish go)

- DoD execution note: publish matrix BLOCKED on exactly two items — the G8
  scope decision and the operator publish go.
- Live re-read this session: **25 member manifests declare
  `publish = true`**, and the set matches the 25-crate TIERS closure exactly
  (GAPS recorded 21 at 21:52; the 4 H03 flips at 21:51 + the umbrella's
  earlier join bring it to 25). The "21+" count in older receipts is stale;
  this count is current.
- Under Option A the ratified footprint becomes 45 crates; under Option B it
  shrinks. **The operator ratifies ONE footprint before any live publish**
  (GAPS REMAINING #5); the double-gated live publish then runs bottom-up tier
  order behind `STARDF_PUBLISH_CONFIRM=ratified-v26.11.1` + `--publish`
  (`publish_stardf.py:601-615`), 25-30 s spacing.

## 4. Decision summary (owner: OPERATOR on every row)

| id | blocker | owner | options | recommendation |
|---|---|---|---|---|
| G8 | 24 nonclosure hard-dep rows; source-woven; 5/25 blast | OPERATOR | A join (20 members) / B drop+gate / C re-route | **A**; B fallback; C no |
| S1 | stardf-core 10.25 MB raw sum vs 10 MB ceiling (real tarball 2.0 MB) | OPERATOR | amend ladder to .crate measure / accept strictness | amend ladder |
| S2 | blake3 caret req, skew-WARN armed (declared==locked today) | OPERATOR | pin `=1.8.7` x2 / bump deliberately | pin `=1.8.7` x2 |
| S3 | contract umbrella cell `stardf`; crate is `stardf-rs` | OPERATOR | materialize facade / amend row | amend row |

## 5. Nonclaims

- This register is a written synthesis of receipts + on-disk manifest reads;
  it executes nothing. No ladder re-run, no manifest edits, no registry
  touch, no git mutations by this lane ("NO git" lane contract).
- All G8 row content is cited from the receipt's `fails` array (the register
  of truth for the 24 rows); pins were re-read from `Cargo.toml` root
  `[workspace.dependencies]` this session.
- Live counts (25 publish=true; 20 publish=false joiners; glob/process
  metadata gaps; geo-kernel recursion; ed25519 native-impl description) were
  re-read on disk this session; they may drift while the finish wave is still
  landing — re-read before acting.
- The blake3 gate was NOT successfully re-witnessed live (workspace
  resolution transiently broken by a concurrent lane's stardf-process
  mid-edit); the S2 standing rests on manifest reads + the script's header
  law, not a fresh gate run.
- The h03 bootstrap sim proves publish-shaped manifest resolution and real
  tarball assembly against a local directory source; it does NOT prove
  registry behavior (index, upload validation, rate limits) — that proof
  only exists post-bootstrap on the real registry.
- stardf-core's 2,088,586 B tarball size is sim-witnessed at the sim's run
  time; the G07 audit's 10,748,822 B raw sum was measured on a different
  file set (319 vs 318 entries) — re-measure at ladder time.
- sql/n3 standing (receipt artifacts in flight, H01/H02) is outside this
  register's scope: this register covers the publish blockers, not the
  first-class target court receipts (see GAPS REMAINING #2/#3).

## 6. Standing

- Register itself: **ALIVE** (written and re-read on disk;
  `stardf/docs/jira/v26.11.1/01-publish-blockers-register.md`).
- Publish readiness: **BLOCKED at G8** (sole hard gate, owner OPERATOR,
  three options registered, Option A recommended). Soft gates S1-S3:
  advisory, PARTIAL, none block the ladder flip.
- Citations: `arrow/evidence/publish_stardf-receipt-20261101T051252Z.json`
  (G8 sole FAIL, 24 rows, verdict BLOCKED); `arrow/docs/V26-11-1-GAPS.md`
  (REMAINING #1/#4/#5); `arrow/docs/OPERATOR-DECISIONS.md` H03 registration;
  `arrow/evidence/h03-bootstrap-sim-2026-10-31.txt` + h03 receipt (blast
  radius); `arrow/evidence/g07-tarball-audit-2026-10-31.json` (S1);
  `arrow/docs/DECISION-BRIEFS/D10-blake3-flip.md` (S2);
  `evidence/v26.11.1-manufacturing-receipt.json` (S3 footprint reality
  check); `publish_stardf.py` (gate law, tiers, ceiling).

## See Also

- [`V26-11-1-GAPS.md`](../../arrow/docs/V26-11-1-GAPS.md) — campaign
  consolidator; REMAINING #1/#4/#5 are the rows this register details
- [`OPERATOR-DECISIONS.md`](../../arrow/docs/OPERATOR-DECISIONS.md) —
  D15-D21 + H03 G8 registration (the in-repo G8 authority)
- [`publish receipt 051252Z`](../../arrow/evidence/publish_stardf-receipt-20261101T051252Z.json)
  — G8 sole FAIL, 24-row fails array, landing_list
- [`publish_stardf.py`](../../arrow/scripts/publish_stardf.py) — tier law,
  G1-G9 + R1, 10 MB ceiling, live-publish double gate
- [`h03-bootstrap-sim-2026-10-31.txt`](../../arrow/evidence/h03-bootstrap-sim-2026-10-31.txt)
  — 20/25 blast-radius witness
- [`D10-blake3-flip.md`](../../arrow/docs/DECISION-BRIEFS/D10-blake3-flip.md)
  — S2 pin mechanics
- [`14-definition-of-done.md`](../v26.10.31/14-definition-of-done.md) — spec
  of record; execution note (publish matrix: two items)
