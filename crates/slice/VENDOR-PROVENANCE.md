# VENDOR-PROVENANCE — stardf-slice

Vendoring receipt for `crates/stardf-slice` (copy-sweep lane w2-l06, 2026-10-31).

## Source

- Upstream: `/Users/sac/purrdf`, pinned rev `0da0578306983da851a9073337d57e2e2c41ab3a`.
- Materialized via `git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a crates/slice | tar -x`
  — per the vendoring law, the pinned rev was used, never the (dirty) working tree.
- Destination: `stardf/arrow/crates/stardf-slice/`, located with the live vendored
  crates but **NOT yet a workspace member** of `stardf/arrow/Cargo.toml` — its
  first-party closure (stardf-rdf, stardf-core, stardf-gts) has not landed (sibling
  copy-sweep lanes' leases), and registering a member whose path deps do not exist
  would freeze the shared `cargo metadata`/compile seam for every concurrent lane.
  Member registration is deferred to integration, gated on the closure.

## Upstream identity receipt (git archive | shasum -a 256)

    9b824d65d3f27c2b9401071265db609712447f1c869bd95ea9742b2489ccff81

## Retention (proven 2026-10-31)

- 39/39 non-manifest files byte-identical to the pinned rev (per-file sha256 loop
  vs `git show 0da05783:crates/slice/<path>`, exit 0; re-run after the manifest
  edit, exit 0). `diff -rq` vs the rev extraction: the ONLY delta is `Cargo.toml`
  (the declared rename/materialization surface).
- Lib name `purrdf_slice` VERBATIM per the vendoring law (upstream declared none;
  implicit name pinned explicitly — stardf-testkit posture, disclosed). Only the
  PACKAGE name renamed (purrdf-slice -> stardf-slice).
- SPDX headers, `licenses/` (LICENSE-APACHE/LICENSE-MIT/LICENSE-MULAN/NOTICE.txt/
  CC-BY-4.0.txt/inventory.json), README.md, src/, tests/ (+fixtures), benches/
  retained byte-verbatim.
- Manifest delta surface (Cargo.toml only): package name; workspace-inherited
  scalars materialized (version 26.9.29 / edition 2024 upstream-equal / license
  string / repository -> ggen-legacy / rust-version 1.98 / homepage + documentation
  carried at upstream values); first-party dep keys renamed purrdf-X -> stardf-X
  as path deps; external pins materialized upstream-equal (`sha2 = "0.11"` default
  features ON = the upstream use-site intent over the purrdf workspace pin
  `sha2 = { version = "0.11", default-features = false }`; `regex` feature set
  verbatim from the upstream workspace entry; `rayon = "1"` verbatim); dev-dep
  `purrdf-testkit = { package = "stardf-testkit", path = "../stardf-testkit" }`
  (upstream KEY kept verbatim so test/bench sources stay byte-verbatim);
  `[lints] workspace = true` inlined verbatim from the purrdf root
  `[workspace.lints]` at the pin (stardf-xsd posture, not the sparql-algebra
  comment-only posture — see divergences).

## Gates (commands this lane ran, 2026-10-31)

- Per-file verbatim verification (39 files): exit 0 (twice: post-extract and
  post-manifest-edit).
- `cargo metadata --no-deps` (shared-seam health): exit 0; 18 packages resolve
  (was 17 pre-sweep — a concurrent lane landed stardf-ed25519 mid-lane);
  stardf-slice correctly NOT auto-included (inert unregistered copy).
- `cargo check -p stardf-slice`: **exit 101 — "package ID specification
  `stardf-slice` did not match any packages"**. [SKIP: loud — the compile gate is
  not runnable until (a) sibling copy-sweep lanes land stardf-rdf, stardf-core,
  stardf-gts (their leases) and (b) integration registers stardf-slice as a
  workspace member. Not a silent pass; a nonzero, reasoned skip.]
- Manifest TOML parse: exit 0 (10 deps, 1 dev-dep, lib name purrdf_slice).

## Known divergences / registered items (not silent)

1. **Missing first-party closure** — the manifest's `../stardf-rdf`,
   `../stardf-core`, `../stardf-gts` path deps do not exist yet (sibling lane
   leases). Until they land + this crate is registered, the crate is inert
   (cargo ignores non-member directories: metadata exit 0 proves it). Do NOT
   register stardf-slice before the closure lands: it will break `cargo metadata`
   workspace-wide (path-dep resolution error) — that would be a compile freeze.
2. **Homepage/documentation retention vs precedent divergence.** The landed
   manifest KEEPS `homepage` and `documentation` at their upstream values
   (non-rename fields carried verbatim per the vendoring law's "everything else
   verbatim"); the stardf-xsd and stardf-sparql-algebra precedents DROPPED both
   fields. Law-strict retention chosen; coordinator may arbitrate toward the
   precedent (dropping them is a one-line deletion, reversible).
3. **[lints] posture**: inlined (stardf-xsd posture), not comment-only
   (stardf-sparql-algebra posture). The two live crates diverge on this today;
   stardf-slice follows xsd. Coordinator may unify.
4. **sha2 default-features materialization**: upstream expresses
   "workspace pin default-features=false + use-site default-features=true";
   cargo's workspace-inheritance semantics for re-enabling defaults are
   version-dependent, so the use-site intent (default features ON) is
   materialized directly. If the coordinator reads the upstream intent as
   "inherit the workspace pin's default-features=false", the one-line flip is
   `sha2 = { version = "0.11", default-features = false }` — disclosed for
   arbitration, not silently chosen either way beyond the use-site text.
5. **rayon (new external dep) registered** in `authority/stardf/license-compat.json`:
   dependency_licenses 17 -> 18 entries, compatibility_receipts 19 -> 20 (consumer
   stardf-slice -> rayon 1.12.0, MIT OR Apache-2.0, observed from
   `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rayon-1.12.0/Cargo.toml`
   + LICENSE files verified present). rayon-core 1.13.0 (transitive) observed
   MIT OR Apache-2.0 in the note but NOT separately registered — per the
   register's own law, transitive attribution comes from Cargo.lock edges, and
   rayon has no lock edge until the workspace resolves it.

## Nonclaims

- NOT claimed: stardf-slice compiles. The compile gate has never run green for
  this crate anywhere in the stardf tree; the only honest receipt is the exit-101
  skip with the reason above.
- NOT claimed: tests/bench pass. Same reason — no dependency closure, not a
  member.
- NOT claimed: the transitive external closure of rayon is registered. Only the
  direct declared dep (rayon) carries entries; the full closure lands with the
  Cargo.lock edge (register law, disclosed above).
- NOT claimed: the workspace member line for stardf-slice. That edit belongs to
  integration (compile-freeze law); a `crates/stardf-slice` member line cannot
  land until stardf-rdf / stardf-core / stardf-gts exist.
- NOT claimed: QuadKey u32-sentinel bridge edge. It does not live in this lease
  (QuadKey lives in `stardf/arrow/crates/stardf-rdf-core/src/ir/dataset.rs:715`
  — the kernel lanes' lease, w2-l04 per the campaign lane table).
