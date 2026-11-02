# frozen-duckdb Re-Homing Plan (v26.10.31 doc 10)

Status: RESOLVED (2026-10-31, finishers-wave lane 15-rehome-note) — D13 decided:
operator ratified frozen-duckdb / frozen-duckdb-sys / frozen-duckdb-builder ALL
`=2.0.4`; see §6 resolution register and `OPERATOR-DECISIONS.md` (D13 resolution
addendum; register number **D19**, finish-campaign record). Lane 7 draft,
2026-10-31; the PENDING-OPERATOR option selection below is the superseded
historical analysis.
Scope: re-home the machine-local `/Users/sac/frozen-duckdb` path patch
(`purrdf/Cargo.toml:383-390 [patch.crates-io]`, three path patches) and the
`~/.frozen-duckdb` cache (1.0 GB, `cache/v2.0.4-2-g87f1daa` prebuilt mega-libs)
so a build succeeds with both renamed away. Companion to 06-phase0 §8.3 (P0-19
R3) and §10.5.

## 0. Ground truth (re-read this session)

- `/Users/sac/purrdf/Cargo.toml:130`: `frozen-duckdb = { version = "2.0.3" }` — registry req
  is `2.0.3`, but the `[patch.crates-io]` path patch overlays local fork HEAD =
  **2.0.4-2-g87f1daa** (v2.0.4 published to crates.io 2026-10-30, un-yanked; 2.0.3 published
  2026-10-29, un-yanked). **Version skew: patched builds compile 2.0.4 sources; a
  registry-only build resolves 2.0.3.**
- Fork is a real git checkout (HEAD `87f1daa`, working tree clean at read time).
- Fork working tree 2.2 GB; cache 1.0 GB under `~/.frozen-duckdb/cache/` with per-target
  prebuilt dirs `v1.4.0` through `v2.0.4-2-g87f1daa`.
- `frozen-duckdb-sys/build.rs` resolves the DuckDB library at build time via
  `frozen_duckdb_builder::link_kind_for_target` + `-L`/`-rpath` emitted to a lib dir; the
  docs.rs branch emits no link directives. Library source: prebuilt cache or source compile
  per builder resolution. No absolute path lives anywhere except the three `[patch]` lines.
- stardf has **zero** frozen-duckdb references in any Cargo.toml — nothing to migrate yet;
  this plan governed the first port (purrdf-duckdb, 477 ln / 38 tests, per 06-phase0 §1).
  [2026-10-31 CORRECTION, lane 15-rehome-note: this bullet is STALE — stardf now consumes
  frozen-duckdb via the stardf-duckdb crate (`crates/stardf-duckdb/Cargo.toml:24`
  `frozen-duckdb = { workspace = true }`); workspace ground truth at this lane's read:
  `Cargo.toml:267` `frozen-duckdb = { version = "2.0.3" }` (float pin, pre-ratification;
  the =2.0.4 x3 manifest edit is the manifest lane's lease, not this doc lane's).
  2026-10-31 REFRESH, lane 15-rehome-note re-invocation: the manifest lane HAS since
  landed the pins — `Cargo.toml:279-281` now carry
  `frozen-duckdb`/`frozen-duckdb-sys`/`frozen-duckdb-builder` all `"=2.0.4"`
  (grep witnessed, exit 0); the line-267 float-pin citation is superseded on disk. The
  F04 court (`scripts/verify_duckdb_rehome.sh`, on disk, =2.0.4 x3, two-arm
  cache-absent) is present but NOT executed by this lane. The =2.0.4 two-arm
  court PASS (both arms) is coordinator-reported lane-01 dispatch testimony —
  no on-disk run receipt; standing registered at §6 item 5.]

## 1. Shared acceptance gate (P0-19 R3 falsifier)

One court for all options: rename both machine-local roots away, build, restore byte-exact.

```bash
#!/bin/bash
# scripts/court_frozen_duckdb_rehome.sh — run against a scratch copy of the consumer workspace
set -euo pipefail

PRE_F=$(shasum -a 256 /Users/sac/frozen-duckdb/.git/HEAD | cut -d' ' -f1)
PRE_C=$(shasum -a 256 "$HOME/.frozen-duckdb/cache" 2>/dev/null | cut -d' ' -f1 || true)

mv "$HOME/.frozen-duckdb" "$HOME/.frozen-duckdb.court-hide"
mv /Users/sac/frozen-duckdb /Users/sac/frozen-duckdb.court-hide
trap 'mv /Users/sac/frozen-duckdb.court-hide /Users/sac/frozen-duckdb; \
      mv "$HOME/.frozen-duckdb.court-hide" "$HOME/.frozen-duckdb"' EXIT

# Precondition: no [patch.crates-io] for frozen-* remains in the subject manifest.
grep -n "patch.crates-io" Cargo.toml && \
  { echo "VACUOUS: [patch.crates-io] still present"; exit 42; }

cargo build -p purrdf-duckdb          # exit 0 = hostability proven
# when sources are ported: cargo test -p purrdf-duckdb   (38 tests, must be 38/0)
```

Fail-closed: any non-zero cargo exit, the `VACUOUS` exit-42, or a restore-digest mismatch
(PRE_F/PRE_C vs post-restore) = court FAIL. A skip must be loud, never a faked pass.

Preconditions per option:
- **(a) and (c):** the `[patch.crates-io]` section must be gone from the subject manifest —
  a court run with the patch present is vacuous (the patch IS the machine-local dependency
  being falsified).
- **(c) additionally:** assert registry provenance — `cargo tree -i frozen-duckdb-sys` must
  show the crates.io source, and the lockfile-resolved version must equal the manifest req
  (2.0.3, or 2.0.4 if the operator accepts the bump; skew registered, not silent).

## 2. Option (a) — vendor the fork into stardf

**Steps**
1. Vendor the three crates from `/Users/sac/frozen-duckdb/crates/` (`frozen-duckdb`,
   `frozen-duckdb-sys`, `frozen-duckdb-builder`) into stardf under `vendor/frozen-duckdb/`
   (or `crates/vendor/`), recording fork SHA `87f1daa` in a `VENDOR-PROVENANCE.md`.
2. Replace the three `[patch.crates-io]` path lines with path deps into `vendor/`.
3. Carry `frozen-duckdb-sys/build.rs` unchanged; it already degrades (docs.rs branch) and
   resolves the lib via the builder.
4. Vendor the cache the same way: copy the applicable `~/.frozen-duckdb/cache/v*` dirs into
   `vendor/frozen-duckdb/prebuilt/` (subsetting to v2.0.x saves ~600 MB; full copy is 1 GB)
   — otherwise `build.rs` re-resolves and may source-compile DuckDB (~30 min) per clean
   machine.
5. Gate: `vendor/` additions must pass the existing vendor-license gate (04-falsifiers R1
   per-file license conjunction) + `cargo tree` single-identity court.

**Size**: vendor tree ~40–80 MB (crate sources only) + 600 MB–1 GB if prebuilt dylibs are
carried. Fork 2.2 GB total, but only 3 crates are needed.

**What breaks**: nothing at the API surface (same sources, same SHA); breaks are
operational — stardf repo grows ~1 GB; every `frozen-duckdb` upstream fix now requires a
re-vendor + provenance receipt; license conjunction (fork LICENSE + DuckDB MIT) must be
carried per-file. If prebuilts are NOT vendored, clean-machine builds pay a full DuckDB
source compile and `build.rs` behavior diverges from the cache-pinned baseline.

**Portability receipt implications**: receipt can claim `SUBJECT_ALIVE` on machines that
have the vendor dir (self-contained). But the vendored prebuilt dylibs are
machine/arch-pinned (arm64/x86_64 split already visible in cache dir names); a cross-arch
receipt needs per-target prebuilts or source-compile mode. Receipt must register
`VENDORED-FORK@87f1daa`, cache subset sha256 manifest, and license conjunction.

## 3. Option (b) — prebuilt-artifact pin (dylib + header + sha256 manifest)

**Steps**
1. From fork `prebuilt/` + cache, select one per-target artifact set:
   `libduckdb.dylib` (+ symlink ids), `duckdb.h`/`duckdb.hpp`, plus `libduckdb.a` if static
   mode is wanted; write `artifacts.sha256` manifest over all files.
2. Publish artifacts to a durable, machine-independent location: git LFS in the consumer
   repo, or an OCI/release-asset store keyed by `v<duckdb>-<target>` (cache dir names
   already use this key shape, e.g. `v2.0.0-arm64`).
3. Point the consumer at the artifacts without the path patch: frozen-duckdb-builder
   supports lib-dir resolution — set `DUCKDB_LIB_DIR`/equivalent builder env (builder
   `link_kind_for_target` branch) in `.cargo/config.toml` `[env]` so no absolute `/Users/sac`
   path enters the repo; the two candidate locations (`/Users/sac/frozen-duckdb/prebuilt/`,
   `~/.frozen-duckdb/cache/v2.0.4-2-g87f1daa`) stay as local bootstrap cache, not authority.
4. Court verifies sha256 manifest before link and refuses on mismatch (fail-closed).

**Size**: ~10–40 MB per target (dylib + header + optional static archive); 2 targets =
~20–80 MB. Smallest footprint of the three.

**What breaks**: the link is against a *prebuilt binary of DuckDB itself*, while crate
sources come from crates.io — ABI between `duckdb.h` (2.0.0 C API per 06-phase0) and the
crates.io crate sources must be version-matched, else link/registration failures appear at
runtime (`register_table_function` misbehaving). Also breaks if the artifact host is
unreachable (network-dependency at build time unless LFS-local); fork-side fixes to
`frozen-duckdb-sys/build.rs` no longer apply (you pinned the binary, not the sys crate
logic — the sys crate still comes from the registry, so its build.rs must accept the
env-provided lib dir).

**Portability receipt implications**: receipts are strongest per-target (artifact digest
named in receipt), weakest on rebuild-from-nothing (a fresh machine needs the artifact
store). `SUBJECT_ALIVE` possible per-target; whole-fabric portability gated on artifact
availability. Must register: artifact digests, builder env contract, and that the crate
sources are registry-not-fork (mixed provenance — registered divergence).

**P0-15 tie-in**: this option does not change the VTab mechanism question; it only fixes
the library supply. Same decision as §5.

## 4. Option (c) — upstream registry 2.0.x-only build (recommended)

**Steps**
1. Delete the three `[patch.crates-io]` path lines (purrdf `Cargo.toml:388-390`).
2. Pin exactly: `frozen-duckdb = { version = "=2.0.3", ... }` today, or `=2.0.4` if the
   operator accepts the golden-regen cost (see skew item). Prefer `=` pin over range to
   keep goldens version-matched.
3. Keep using the local checkout/cache only as an optional developer acceleration via a
   documented `[patch.crates-io]` in a **non-committed** `Cargo.toml` override or a
   developer-local `~/.cargo/config.toml` — machine-local convenience must never be repo
   authority. (Cargo supports local path overrides via config; if that proves unusable,
   drop the acceleration and go pure registry.)
4. Run the §1 court with both roots renamed away; assert `cargo tree -i frozen-duckdb-sys`
   shows registry source and lockfile version == pin.
5. Bump lane: when the fork publishes a new crates.io release, bump the pin + regen
   goldens with recorded reason (frozen-vector reason string per 06-phase0 §3).

**Size**: near-zero repo delta (3 deleted lines + pin edit); no vendor tree, no artifacts.

**What breaks**: version skew between fork HEAD (2.0.4) and pin (2.0.3) — the patched dev
build and the registry build compile different sources today; resolving it means either
bumping to 2.0.4 (golden regen, recorded) or pinning =2.0.3 (fork HEAD drift becomes
irrelevant because it is no longer authority). The dev-cache acceleration (if kept) is
best-effort and must not be load-bearing — the court renames it away, so CI must never
depend on it.

**Portability receipt implications**: cleanest of the three — provenance is a single
crates.io version; receipts replay from `Cargo.lock` + registry, machine-independent.
`SUBJECT_ALIVE` on any machine with network/registry access (vendoring crates.io tarballs
or `cargo vendor` closes even that). Register: pin version, skew disposition, and the
`cargo tree` provenance assertion in the receipt.

## 5. Mechanism decision: in-process VTab vs loadable dylib

**Decision: in-process VTab.** Grounded per 06-phase0 §1: `VTab` trait +
`Connection::register_table_function` with live examples in the fork; the loadable dylib
entry macro has zero local code and no `LOAD`-path test exists. Dylib route = UNSUPPORTED
until (i) a loadable-extension entry macro exists in the local fork, and (ii) a real
`LOAD`-based court registers `stardf_scan()` through the C API plugin path and passes.
Revisit only with that falsifier receipt. This also keeps option (b)'s single dylib
supplying the *engine*, while `stardf_scan()` stays compiled into the consumer binary —
two different dylib questions; do not conflate them.

## 6. Recommendation and decision register

Lane recommendation: **(c) registry-only with `=` pin**, with option (b) as the fallback if
registry build proves non-viable on a target (e.g. docs.rs/WASI). Grounds: smallest diff,
single provenance, cleanest receipts, no repo growth. Option (a) only if operator wants
fork-head authority locally without publishing lag.

RESOLVED (2026-10-31, finishers-wave lane 15-rehome-note — replaces the
PENDING-OPERATOR list that stood here):

1. Option: **(c) registry-only with exact-version pins** (mechanics per §4). The
   option (a) canonical-checkout execution detail from OPERATOR-DECISIONS.md's
   sign-off block ("option (a) … all pinned =2.0.3 per W1-9's float datum") is
   superseded at the execution-detail level: the one-package workspace consumes
   frozen-duckdb as a registry dependency, no `[patch.crates-io]`, no
   machine-local path. No canonical fork checkout is required for the campaign
   footprint.
2. Pins: **frozen-duckdb / frozen-duckdb-sys / frozen-duckdb-builder ALL
   `=2.0.4`** (operator-ratified 2026-10-31; register number **D19**,
   `OPERATOR-DECISIONS.md` finish-campaign record — the D13 RESOLUTION
   section carries the ratification, D19 gives it the register row). Why: the
   rehome falsifier (**F04**, `scripts/verify_duckdb_rehome.sh` — registry-only
   resolve+compile court) **fired on `=2.0.3` cache-absent**: with
   `/Users/sac/frozen-duckdb` and `$HOME/.frozen-duckdb` absent, the `=2.0.3`
   pin is not hostable registry-only (fail-closed court path). The =2.0.3
   cache-absent failure is therefore moot — the pin moved to =2.0.4.
   Firing mechanism (contract-stated; this lane did not re-execute): the
   cache-absent refusal executed through the fork's **KD-01 export fence** —
   `fence_exports_for` in `frozen-duckdb-builder` applies a 1,177-symbol
   `-Wl,-exported_symbols_list` allowlist before cache persist (fork register
   `/Users/sac/frozen-duckdb/KNOWN_DIVERGENCES.md` KD-01, CLOSED) — failing
   against an **unpinned duckdb-master clone**; typed confounder disclosed:
   the refusal is attributed to the unpinned source identity, not
   independently to the `=2.0.3` pin version. (This KD-01 is the fork's
   dylib-symbol register row — unrelated to stardf `docs/DIVERGENCES.md`
   KD-01, a TermId offset divergence.)
3. Cache acceleration: none — no `[patch.crates-io]` anywhere in the
   one-package workspace (crates.io law: no machine-local path overrides in
   published crates).
4. Mechanism: in-process VTab (unchanged; §5 grounding stands).
5. **=2.0.4 court witness (additive 2026-10-31, D10 doc lane).** Registry
   resolve + compile PASS at `=2.0.4`, **both arms** of the two-arm court, is
   coordinator-reported **lane-01 dispatch testimony**; no on-disk run
   receipt for the two-arm run exists in `evidence/` (standing class
   `REHOME_TWO_ARM_COURT_TESTIMONY_STANDING`,
   `evidence/v26.11.1-manufacturing-receipt.json`). This doc lane did not
   execute the court; the run receipt for the two-arm PASS remains OWED to
   lane 01.

Naming-divergence registration: the DoD (14-definition-of-done.md) verbatim
row named the pins `duckdb` / `duckdb-sys` / `libduckdb-sys`; the actual crate
identities are `frozen-duckdb` / `frozen-duckdb-sys` / `frozen-duckdb-builder`
(no `libduckdb-sys` exists in the closure). Registered here and in the DoD row
amendment.

Nonclaims (this lane): docs-only lane — no cargo run, no court re-run, no
manifest edit. The F04 falsifier-fired receipt is cited from the finishers-wave
lane contract, not re-executed here; the =2.0.4 x3 cache-absent court re-run is
NOT witnessed by this lane — the lane-01 two-arm PASS (both arms) is
coordinator-reported testimony with no on-disk run receipt (standing per §6
item 5).
