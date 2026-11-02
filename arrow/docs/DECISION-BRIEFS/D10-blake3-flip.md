# D10 Blake3 Flip — Operator Decision Brief

**Status: PENDING-OPERATOR-SIGN-OFF.** This brief does NOT flip KD-08;
KD-08 remains OPEN in `stardf/arrow/docs/DIVERGENCES.md` (KD-08/KD-12 rows).
Lane 10, 2026-10-31. Composes: lane 4's arbitration pack
(`stardf/docs/jira/v26.10.31/11-blake3-arbitration.md`), the W1-8 gate
receipt evidence, and lane 11's R1 brief
(`DECISION-BRIEFS/R1-blake3-receipt-admission.md`).

## 1. Decision on the table (D10 / KD-08)

Which single BLAKE3 implementation `stardf/arrow` binds. Current state
(observed on disk, cited from lane 4's pack + re-checked this session):

- One impl: `blake3::hash` at
  `crates/stardf-arrow-core/src/arena.rs:344` (test sites `arena.rs:628,701`);
  declared `blake3 = { version = "1.8.5", default-features = false }`
  (`crates/stardf-arrow-core/Cargo.toml:15`); lock resolves **1.8.7**
  (`Cargo.lock:17-20`, checksum pinned).
- Normal closure of blake3 1.8.7: `arrayvec 0.7.8`, `cfg-if 1.0.5`,
  `constant_time_eq 0.4.2` — 3 registry crates of supply-chain surface.
- purrdf predecessor binds its own zero-dep native impl
  (`purrdf_hash::blake3`), rlib-only, no_std/wasm32-clean, and carries the
  frozen-vector conformance suite (`purrdf/crates/hash-conformance/`).
- avec: registered two-impl defect (`avec-blake3` + duplicate
  `avec-arrow/src/blake3.rs`; `03-substrate-map.md:26-27`).

**Recommendation (lane 4, carried forward): Option B — vendored purrdf-hash
native** into `crates/stardf-arrow-core/src/hash/`, provenance-headered with
upstream SHA pin; a path dependency on `~/purrdf` is NOT lawful
(portable-receipt / one-canonical-checkout discipline; the frozen-duckdb
absolute-path pin in P0-18 §7 is the cautionary precedent).

## 2. W1-8 gate receipt — what it establishes

The KD-08 half-1 gate exists on disk:
`stardf/arrow/scripts/verify_blake3_identity.sh`, designed in
`11-blake3-arbitration.md` §3, and its receipt evidence records:

1. **Single identity: exactly `blake3 v1.8.7`** across the workspace normal
   graph. Re-run by this lane (read-only) 2026-10-31:

   ```
   [PASS] exactly 1 blake3 identity: blake3 v1.8.7
   [WARN] version skew: Cargo.toml declares 1.8.5, Cargo.lock resolved 1.8.7
   ```

   exit 0. Inverse tree rows: blake3 v1.8.7 pulled by stardf-arrow-core,
   which is pulled by bench/capi/columnar.
2. **Workspace-scoped command fix (witnessed deviation from the doc's
   command).** The doc's exact command
   (`cargo tree -e normal -i blake3 --prefix none` from the workspace root)
   exits 101 "did not match any packages" — bare inverse-tree at the root
   selects only the ROOT package's graph, and `stardf-arrow` does not itself
   depend on `stardf-arrow-core`. The gate therefore runs
   `cargo tree --workspace -e normal -i blake3 --prefix none`. This is the
   same under-count class the gate exists to prevent, witnessed on the real
   tree.
3. **Absence-arm inversion note (pre-registered in the script header).** The
   script implements the pre-flip mode: blake3 ABSENT → exit 1 VACUOUS
   (binding is mandatory; zero bindings carry no bits). Its header states
   the flip-time consequence: under Option B the arm inverts — absence
   becomes PASS, presence FAIL.
4. **Skew WARN is live, advisory-only.** Declared 1.8.5 vs locked 1.8.7
   surfaces as `[WARN]`, not FAIL: the gate's failure channel is the
   identity count; the skew channel is advisory (an Option-A pin to
   `=1.8.7` clears it; post-Option-B the declaration is gone and the WARN
   row is retired).

Gate wiring status (unchanged from the arbitration pack's KD-07 warning):
the script lives in `scripts/` but is **not wired into
`ci/stardf-arrow-reconstitution.yml` nor the `justfile`** — per KD-07's
failure mode, unwired = PARTIALLY CLOSED. Wiring is a flip-time obligation
under both options.

## 3. What the flip changes

If the operator signs off on Option B, three things change at once:

1. **Gate mode inversion.** `verify_blake3_identity.sh` must be rewritten
   or re-moded at flip time: identity-to-count becomes the vendored module;
   the check becomes "no `blake3 v` registry node exists at all" — absence
   PASS, presence FAIL. The current script's VACUOUS arm (absence → exit 1)
   is the Option-A mode and would actively FALSE-FAIL the post-flip state.
   The script header pre-registers this: "rewrite or re-mode at flip time."
2. **Conformance-vector carriage arrives with the impl.** Option B carries
   purrdf-hash's frozen-vector suites (official BLAKE3 vectors, streaming
   boundaries, base16) via the purrdf-hash-conformance port — the digest
   court cost drops from "build a corpus" to "port a corpus." Under
   Option A, no vectors are bundled; the §4 court corpus must be built.
3. **Second-copy drift risk created and courted.** After the flip, BLAKE3
   exists in purrdf's tree AND stardf's vendored copy. KD-08's law is
   per-dependency-graph, so this is compliant; drift is policed by the
   parity court case 6 (upstream purrdf differential — an upstream change
   must flip the court or be registered as a KD-08 addendum row) plus the
   SHA-pinned provenance header.

KD-08's law is per-dependency-graph, not global — the gate is defined over
the stardf workspace, so upstream purrdf's own blake3 binding is out of
scope.

## 4. Migration step lists (verbatim from arbitration §5)

### If B wins (recommended)

1. Vendor `purrdf/crates/hash/src/{blake3.rs,digest.rs,hex.rs}` plus every
   module `blake3.rs` reaches (determine by compiling, not guessing) into
   `crates/stardf-arrow-core/src/hash/`, with provenance (upstream path +
   commit SHA + copy date) and license headers preserved verbatim.
2. Drop `blake3 = "1.8.5"` from
   `crates/stardf-arrow-core/Cargo.toml:15`; swap `arena.rs:344` (and test
   sites 628, 701) to the vendored `hash::blake3::hash` equivalent.
3. Land both gates: `verify_blake3_identity.sh` wired into
   `ci/stardf-arrow-reconstitution.yml` + `justfile`, and
   `tests/blake3_parity_court.rs` (dev-dep `avec-blake3`, path-pinned to
   the avec checkout SHA recorded in `03-substrate-map.md`).
4. Invert the gate's absence arm (§3.1 above).
5. Update `DIVERGENCES.md` KD-08/KD-12 rows and `06-phase0` §7 only after
   operator sign-off, with gate+court receipts attached.

### If A wins

1. Pin exactly: `blake3 = { version = "=1.8.7", default-features = false }`
   (or re-pin to 1.8.5 and regenerate the lock deliberately) so silent lock
   drift cannot recur — this also clears the live skew WARN.
2. Land the §3 gate unchanged (count == 1 registry identity) and the §4
   court minus case 6.
3. Accept the 3-crate third-party closure into the P0-18 Tier map with
   per-file license audit (Apache-2.0 for
   blake3/arrayvec/cfg-if/constant_time_eq — verify at audit time).

## 5. License interplay with the R1 receipt (lane 11's brief)

The R1 gate (`verify_stardf_licenses.py`) currently FAILs the edge
`stardf-arrow-core -> blake3` (gated license expression
`CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception`, no registered
compatibility receipt; gate exit 1, also blocked by the missing
`license-inventory.toml` conjunct census). Interplay per option:

- **Under Option A**, the R1 decision (lane 11's two arms) must land first
  or alongside: Arm A registers the drafted compatibility receipt
  (Apache-arm selection; carriage obligation: ship `LICENSE_A2` +
  `LICENSE_A2LLVM` + notices), Arm B reclassifies the register row to
  `CC0-1.0` sovereign-class (zero carriage, register-fidelity cost). Until
  one lands, W3's avec-gate work stays BLOCKED per OPERATOR-DECISIONS D6.
- **Under Option B**, the `stardf-arrow-core -> blake3` registry edge
  **ceases to exist** — the R1 FAIL row disappears mechanically, and no R1
  receipt (Arm A) or reclassification (Arm B) is needed for blake3. The
  license surface moves to the **vendored copy**: the vendored modules carry
  purrdf-hash's licensing (MIT OR Apache-2.0 OR MulanPSL-2.0 per its
  Cargo.toml, cited from the arbitration pack; first-party tree, so the R1
  gate does not classify vendored first-party code the way it classifies
  registry edges — UNVERIFIED: the R1 verifier's treatment of vendored
  first-party modules is asserted, not tested).
- Either way, the R1 gate's independent exit-1 cause (the missing
  `license-inventory.toml` conjunct census) remains open and is NOT
  resolved by the D10 flip. And either way, D6's sequencing law holds: the
  R1 landing precedes avec-edge admission, independent of D10.

Sequencing note: choosing Option B **removes** the blake3 R1 fork entirely,
which simplifies lane 11's decision to (in effect) an Option-B-only
consequence. If Option A wins, lane 11's decision becomes load-bearing for
W3 unblocking.

## 6. Nonclaims

- **Not flipped**: KD-08 remains OPEN; this brief flips nothing. No code
  vendored, no call sites swapped, no gate landed or re-moded, no
  DIVERGENCES/OPERATOR-DECISIONS rows edited.
- **No git mutations** by this lane (lane contract); parent HEAD
  `c9f8028…` untouched.
- Gate re-run was the lane's own **local read-only witness** (cargo tree
  over the real lockfile), not a new receipt artifact; CI
  (`ci/stardf-arrow-reconstitution.yml`) has never been witnessed executing
  (w1-closure nonclaim, still standing).
- The W1-8 gate receipt items (single identity 1.8.7, workspace-scoped
  command fix, absence-arm inversion note, skew WARN) are cited from the
  gate script's on-disk header + this lane's re-run; the original W1-8
  receipt artifact itself did not resolve to an on-disk file this session
  (per-lane standing for wave-1 lanes recorded UNKNOWN in
  `evidence/w1-closure-2026-10-31.json`).
- UNVERIFIED: whether `purrdf/crates/hash/src/blake3.rs` compiles
  standalone once extracted (module reach = "determine by compiling").
- UNVERIFIED: exact blake3 1.8.7 license text (cited from W1-5's
  verification of the 1.8.5 checkout; not re-read for 1.8.7).
- UNVERIFIED: R1 verifier behavior toward vendored first-party modules
  under Option B (asserted, not tested).
- Option B's residual risk (second copy across trees) is mitigated by court
  case 6 + provenance headers, not eliminated.
- End state of this file: **PENDING-OPERATOR-SIGN-OFF.**
