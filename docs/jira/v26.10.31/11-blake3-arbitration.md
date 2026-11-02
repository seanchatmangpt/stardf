# 11-blake3-arbitration

## OUTCOME — D10 DECIDED (supersession banner, 2026-10-31)

This banner records the D10 outcome. The pack below it is retained verbatim;
where it reads PENDING-OPERATOR-SIGN-OFF or recommends Option B (§6), that
status is superseded by this banner.

**D10 DECIDED — Option A retained**: the blake3 crate stays bound;
purrdf-hash is NOT vendored.

- Gate landed + witnessed ×3: `stardf/arrow/scripts/verify_blake3_identity.sh`
  → exit 0, "exactly 1 blake3 identity: blake3 v1.8.7" — witnessed in
  `stardf/arrow/docs/RECEIPTS.md` (LANE-8 row),
  `stardf/arrow/docs/DECISION-BRIEFS/D10-blake3-flip.md` (lane re-run), and
  `stardf/arrow/README.md` (standing table).
- WARN live, registered: declared 1.8.5 vs locked 1.8.7 skew surfaces as the
  gate's advisory WARN and is registered in all three rows above. Clearing
  step (pin `=1.8.7`, §5 If-A-wins step 1) not yet executed — WARN still live
  as of this banner.
- Option B DEFERRED, not rejected: §5 "If B wins" migration steps stand as
  the standing recommendation for any future revisit. Nothing vendored, no
  call sites swapped, §4 digest-parity court not landed.
- Open residual at decision time: the gate is local-run only — NOT wired into
  `ci/stardf-arrow-reconstitution.yml` or the `justfile` (§3's
  wire-at-landing instruction did not land); `DIVERGENCES.md` KD-08/KD-12
  rows still read OPEN / PARTIALLY CLOSED as of this banner (owned outside
  this lane).

---

Evidence + recommendation pack for KD-08: which single BLAKE3 implementation
`stardf/arrow` binds to. Status: **PENDING-OPERATOR-SIGN-OFF** — this file
does NOT flip the KD-08 decision; KD-08 remains OPEN in
`stardf/arrow/docs/DIVERGENCES.md:28`.

Lane 4, 2026-10-31. Evidence re-read this session on the real trees.

## 1. Current state (observed, this session)

- stardf/arrow binds ONE impl: `blake3::hash` at
  `stardf/arrow/crates/stardf-arrow-core/src/arena.rs:344` (fingerprint push;
  test sites at `arena.rs:628,701`); declared
  `blake3 = { version = "1.8.5", default-features = false }` in
  `crates/stardf-arrow-core/Cargo.toml:15`; lock resolves **blake3 1.8.7**
  (`Cargo.lock:17-20`, checksum pinned).
- blake3 1.8.7 normal closure (this session, `cargo tree -e normal -p blake3`
  in `stardf/arrow/`): `arrayvec 0.7.8`, `cfg-if 1.0.5`,
  `constant_time_eq 0.4.2` — 3 registry crates.
- purrdf: single BLAKE3 path via `purrdf_hash::blake3`
  (`purrdf/crates/rdf-core/src/ir/frozen_terms.rs:13-19`; `content_id.rs` —
  `Blake3ContentId` never hashes itself, callers hash via
  `purrdf_hash::blake3`; hex via shared `purrdf_hash::hex` so the two digest
  domains never drift).
- `purrdf-hash` crate: `publish = false`, rlib-only, **zero runtime
  dependencies** (`purrdf/crates/hash/Cargo.toml` `[lib]` comment: root of
  `layers.toml`, wasm32-clean). BLAKE3 module
  `purrdf/crates/hash/src/blake3.rs`; frozen-vector suites live in the
  unpublished `purrdf-hash-conformance` (`purrdf/crates/hash-conformance/` —
  digest differentials, BLAKE3 streaming boundaries, base16, per its
  Cargo.toml comments).
- avec: registered defect — TWO impls (`avec-blake3` zero-dep no_std crate +
  a 932-line duplicate in `avec-arrow/src/blake3.rs`; per
  `stardf/docs/jira/v26.10.31/03-substrate-map.md:26-27`). `avec-blake3` is
  consumed by avec-beam/bench/cli/wasm-footprint/integration (Cargo.toml
  reads, this session).

## 2. What each choice drags in

### Option A — blake3 1.8.5 crate (status quo)

- Drags 3 registry crates (arrayvec, cfg-if, constant_time_eq), each a
  supply-chain surface. Version drift already witnessed in this tree:
  declared 1.8.5, lock resolved 1.8.7 with no deliberate bump — a first
  concrete instance of the drift the single-identity gate exists to catch.
- Per P0-18 (`06-phase0-columnar-sql-duckdb.md` §7): small first-party
  closure on the known Tier map. blake3 is a third-party Tier surface that
  then needs per-file license audit like the rest of the map.
- `default-features = false` already set, so the C/SIMD optimizer paths are
  excluded — the 3-crate closure is the *best case* for this option;
  enabling default features grows it.
- Keeps stardf structurally like avec's good half (one crate, one impl) but
  diverges from purrdf's impl identity (zero-dep native).

### Option B — purrdf-hash native (recommended)

- Zero runtime deps, rlib-only, no_std/wasm32-clean — matches the P0-18
  first-party closure law exactly, and the purrdf predecessor identity.
- **Carries the hash-conformance frozen vectors** (via the
  purrdf-hash-conformance port): official BLAKE3 vectors, streaming
  boundaries, base16 — the digest court arrives with the impl, not built
  after.
- Cost: purrdf-hash is `publish = false` in a foreign tree. Adoption means
  vendoring the needed modules (at minimum `blake3.rs`, `digest.rs`,
  `hex.rs`, plus whatever `blake3.rs` reaches) into
  `stardf/arrow/crates/stardf-arrow-core/src/hash/` — a first-party copy with
  provenance header + upstream SHA pin. A path dependency on `~/purrdf` is
  NOT lawful here (violates the portable-receipt / one-canonical-checkout
  discipline; the frozen-duckdb absolute-path pin in P0-18 §7 is the
  cautionary precedent being cleaned up, not a pattern).
- After Option B, BLAKE3 exists in purrdf's tree AND stardf's vendored copy.
  The KD-08 law is per-dependency-graph, not global — the cargo-tree gate
  (§3) is defined over the stardf workspace, so this is compliant; the
  parity court case 6 (§4) is what proves the vendored copy never drifts
  from upstream purrdf.

### Decision inputs

| Axis | A: blake3 crate | B: purrdf-hash native |
|---|---|---|
| Normal deps added | 3 registry crates | 0 (vendored first-party) |
| no_std / wasm | no (std-oriented) | yes |
| Frozen vectors | none bundled | carried (conformance port) |
| Supply-chain surface | registry (silent lock drift 1.8.5→1.8.7 witnessed) | vendored copy, SHA-pinned provenance |
| Predecessor parity (purrdf) | divergent impl identity | identical impl identity |
| Substrate (avec) parity court | trivial (same crate) | required (§4) |
| Migration size | none | medium (vendor + swap 1 call site + courts) |

## 3. Cargo-tree single-identity gate (KD-08 close condition, half 1)

Primary form (exact, from `stardf/arrow/`):

```bash
cargo tree -e normal -i blake3 --prefix none | wc -l
```

Under a single identity this prints exactly one line (blake3 itself).
Pass condition: output == `1`. Exit-nonzero form:

```bash
# scripts/verify_blake3_identity.sh  (run from stardf/arrow/)
set -euo pipefail
tree=$(cargo tree -e normal --charset ascii)
versions=$(printf '%s\n' "$tree" | grep -oE 'blake3 v[0-9.]+' | sort -u)
n=$(printf '%s\n' "$versions" | grep -c 'blake3 v' || true)
[ "$n" -eq 0 ] && { echo "VACUOUS: blake3 absent (binding is mandatory)"; exit 1; }
[ "$n" -eq 1 ] || { echo "FAIL: >1 blake3 identity: $versions"; exit 1; }
echo "PASS: single blake3 identity: $versions"
```

Properties:

- `-e normal` scopes to the normal dependency graph — a test-only
  `avec-blake3` dev-dependency (needed by the §4 court) does not violate the
  gate. The gate must always be run with normal edges; running it with
  `--edges dev` would flag the court's own dev-dep (expected, and the reason
  the gate pins `-e normal`).
- Absence is VACUOUS (exit 1), not a pass — KD-08's decision-in-force is
  "bind exactly one", so zero bindings fail loudly per the gate-vacuity law.
- Version-uniqueness is checked, not just node count: it catches a second
  blake3 *version* introduced anywhere in the closure even if inversion
  output formatting changes.
- Wire into `ci/stardf-arrow-reconstitution.yml` + the local `justfile`.
  KD-07 is the precedent for the failure mode to avoid: a gate living in
  `scripts/` but not wired into `ci/` reads PARTIALLY CLOSED (DIVERGENCES.md
  KD-07 row). Wire it at landing, not later.

## 4. Digest-parity court vs avec-blake3 (KD-08 close condition, half 2)

Scope note: the avec defect is that avec has TWO impls. The court's job is
narrower — prove whatever stardf binds is byte-compatible with
`avec-blake3` (the substrate's sovereign impl) on identical inputs, so
stardf arena fingerprints are substrate-interoperable.

Design (Chicago: real hashing, assert on final digests, no mocks):

```rust
// stardf/arrow/crates/stardf-arrow-core/tests/blake3_parity_court.rs
// KD-08 digest-parity court: the bound impl must be byte-identical to
// avec-blake3 on empty input, block boundaries, streaming split
// permutations, and merkle-tree shapes.
```

`avec-blake3` enters as a **dev-dependency only** (never normal), so the
§3 gate stays green.

Court cases (assert on 32-byte digests as raw bytes):

1. Empty input.
2. Block-boundary sizes: 0, 64, 128, 192, 1024, 4096 bytes.
3. Streaming split permutations: same bytes fed as 1/2/7/64/1024-byte
   update chunks vs one-shot — all must agree (this is the
   purrdf-hash-conformance streaming-boundary class).
4. Merkle-tree shapes: inputs of 2^10, 2^11, and sizes crossing
   parent-merge thresholds (1024-chunk tree behavior).
5. Cross-impl vs `avec-blake3`: identical digests on all of the above;
   also the official BLAKE3 vector corpus (avec carries
   `test_official_blake3_vectors_multi_block_and_merkle_tree` at
   `avec/crates/avec-blake3/src/lib.rs:1052`; the same corpus is what
   purrdf-hash-conformance runs).
6. purrdf differential: same bytes through the vendored `purrdf-hash`
   `blake3.rs` — identical digests, and hex rendering via the vendored
   `hex.rs` matches `purrdf_hash::hex`. This case is what makes the
   vendored copy drift-proof against upstream purrdf: an upstream change to
   `blake3.rs` must flip this court (case 6 failing = drift detected) or be
   registered as a KD-08 addendum row.

## 5. Migration steps

### If B wins (recommended): purrdf-hash native

1. Vendor: copy `purrdf/crates/hash/src/{blake3.rs,digest.rs,hex.rs}` plus
   every module `blake3.rs` reaches (`block.rs`, `dispatch.rs`, etc. —
   determine by compiling, not guessing) into
   `stardf/arrow/crates/stardf-arrow-core/src/hash/`. Add provenance:
   upstream path + commit SHA + copy date in a module-level comment, license
   headers preserved verbatim (purrdf-hash is MIT OR Apache-2.0 OR
   MulanPSL-2.0 per its Cargo.toml).
2. Drop `blake3 = "1.8.5"` from `crates/stardf-arrow-core/Cargo.toml:15`;
   swap `arena.rs:344` (and test sites 628, 701) to the vendored
   `hash::blake3::hash` equivalent.
3. Land both gates from §3/§4: `scripts/verify_blake3_identity.sh` wired
   into `ci/stardf-arrow-reconstitution.yml` + `justfile`, and
   `tests/blake3_parity_court.rs` (dev-dep `avec-blake3`, path-pinned to the
   avec checkout SHA recorded in `03-substrate-map.md`).
4. Run `cargo tree -e normal -i blake3 --prefix none` → expect exit 1
   VACUOUS *only if* the decision changes to "no external blake3 allowed";
   under Option B the gate script needs its target updated: the identity to
   count becomes `stardf-arrow-core`'s vendored module, and the check is
   "no `blake3 v` registry node exists at all" (absence now = PASS; presence
   = FAIL). I.e. Option B inverts the VACUOUS arm of the gate — the script
   must take the mode as an argument or be rewritten at flip time.
5. Update `stardf/arrow/docs/DIVERGENCES.md` KD-08 row and
   `docs/jira/v26.10.31/06-phase0-columnar-sql-duckdb.md` §7 only after
   operator sign-off, with the gate+court receipts attached.

### If A wins: blake3 crate stays

1. Pin exactly: change `Cargo.toml` to `blake3 = { version = "=1.8.7",
   default-features = false }` (or re-pin to 1.8.5 and regenerate the lock
   deliberately) so lock drift can't recur silently.
2. Land the §3 gate unchanged (registry identity expected, count == 1) and
   the §4 court minus case 6.
3. Accept the 3-crate third-party closure into the P0-18 Tier map with
   per-file license audit (Apache-2.0 for blake3/arrayvec/cfg-if/
   constant_time_eq — verify at audit time, not asserted here).

## 6. Recommendation

**Option B — purrdf-hash native (vendored)**, status
PENDING-OPERATOR-SIGN-OFF.

Reasons, in weight order:

1. P0-18's law is a small first-party closure; Option A drags a registry
   surface whose silent drift (1.8.5→1.8.7 in this very lock file) the
   project would otherwise have to police forever.
2. The conformance frozen vectors travel with the impl — the KD-08 court
   cost drops from "build a corpus" to "port a corpus".
3. Predecessor identity: purrdf's single-impl discipline is preserved
   verbatim in the successor tree; Option A makes stardf's BLAKE3 identity
   differ from purrdf's, a permanent registered divergence with no
   compensating gain (fingerprints are internal, 32 bytes either way).
4. The avec-defect shape is avoided *by construction* in the stardf
   workspace (one vendored module family, gated), not merely avoided by
   currently-happening-to-bind-one (Option A's state, which is exactly how
   avec ended up with two — nobody added a second; the tree just grew one).

Residual risks if B wins: vendoring re-creates a second copy of the
algorithm across purrdf/stardf trees (mitigated by court case 6 + SHA-pinned
provenance); the migration touches `arena.rs` call sites (3 sites, all
mechanical).

## 7. Nonclaims

- NOT executed: no code vendored, no call sites swapped, no gates landed,
  no KD-08 flip. This file is evidence + design only.
- UNVERIFIED: whether `purrdf/crates/hash/src/blake3.rs` compiles
  standalone once extracted (its exact internal module reach is asserted as
  "determine by compiling" in §5, not verified here).
- UNVERIFIED: the official BLAKE3 vector corpus files' presence/location in
  purrdf-hash-conformance is cited from Cargo.toml comments, not opened.
- UNVERIFIED: license of blake3 1.8.7 crate (stated as Apache-2.0 from
  general knowledge, not read from the vendored source this session).
- The recommendation is PENDING-OPERATOR-SIGN-OFF; KD-08 remains OPEN.
