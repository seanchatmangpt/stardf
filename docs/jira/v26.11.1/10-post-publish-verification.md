# Post-Publish Verification Plan (v26.11.1 doc 10)

Status: PLAN AUTHORED (2026-10-31, publish lane 10). Every step below is
UNVERIFIED until executed post-publish; per-step standing is stamped at run
time into `stardf/arrow/evidence/` per the lane contract. This file is the
verification contract, not a receipt.
Scope: post-publish verification and reversal for the v26.11.1 first-class
publish closure (25 packages) of `stardf/arrow` (workspace root
`stardf/arrow/Cargo.toml`).
Authority chain: `stardf/arrow/scripts/publish_stardf.py` (TIERS T0-T8, gates
G1-G9, live-publish confirmation law), D13/D19 (frozen-duckdb `=2.0.4`
ratification), D17 (first-class-closure-only publish footprint),
`docs/jira/v26.10.31/10-duckdb-rehome-plan.md` (frozen-duckdb re-home record),
lane 07/08/09/14 manifest receipts.

## 0. Ground truth (re-read this session, 2026-10-31)

- Closure size: 25 packages, `publish = true`. Publish order = TIERS from
  `publish_stardf.py:87-120` (re-derived from member manifests this session;
  matches lane 14's runtime topological check):
  - T0: stardf-hash, stardf-alloc-probe, stardf-arrow-core
  - T1: stardf-lex, stardf-deflate, stardf-branchless, stardf-testkit,
    stardf-events, stardf-arrow-bench, stardf-columnar
  - T2: stardf-stack, stardf-iri, stardf-arrow-rs, stardf-xsd,
    stardf-arrow-capi, stardf-scan-duckdb
  - T3: stardf-cdt
  - T4: stardf-core (dir crates/stardf-rdf-core), stardf-sparql-algebra
  - T5: stardf-sparql-results
  - T6: stardf-rdf
  - T7: stardf-sql, stardf-n3, stardf-toml
  - T8: stardf-rs (dir crates/stardf-purrdf; lib name `purrdf`)
- Every published member inherits the workspace scalars: version 26.11.1,
  edition 2024 (stardf-arrow-capi and stardf-arrow-core pinned edition 2021
  for authored `#[no_mangle]` syntax), rust-version 1.98, license
  `MIT OR Apache-2.0 OR MulanPSL-2.0`, repository
  `https://github.com/seanchatmangpt/ggen-legacy`, homepage
  `https://blackcatinformatics.ca/purrdf/`, description
  "Deterministic zero-copy columnar RDF 1.2, Arrow, Parquet, and DuckDB
  execution substrate", keywords rdf/arrow/parquet/duckdb/sparql.
- Name-mapping hazards for tooling (both witnessed this session):
  - `crates/stardf-purrdf` = package **stardf-rs**; `crates/stardf-arrow` =
    package **stardf-arrow-rs**; `crates/stardf-rdf-core` = package
    **stardf-core**. Dep keys `purrdf`, `purrdf-arrow`, `purrdf-columnar`,
    `purrdf-testkit`, `purrdf-alloc-probe` are rename keys
    (`package = "stardf-*"`) resolving to closure packages.
  - Duplicate package-name trap: `crates/stardf-events` (member,
    publish=true) and `crates/stardf-rdf-events` (NOT a member, publish=false)
    BOTH declare `[package] name = "stardf-events"`. A naive
    `crates/*/Cargo.toml` sweep reads the non-member duplicate last and
    miscounts the closure as 24 (witnessed twice this session). Enumerate via
    `cargo metadata` members, never by directory glob.
- Witnessed blocker (as-of-now, pre-existing, outside lane 10's file
  ownership): `cargo metadata --no-deps` at the workspace root exits 101 —
  `crates/stardf-process/Cargo.toml` inherits `cgap-arrow.workspace = true`
  but root `[workspace.dependencies]` carries no `cgap-arrow` key (lane 09
  decouple removed the key; stardf-process kept the row). stardf-process is
  publish=false and outside the closure, but until fixed this breaks every
  workspace-level re-derivation step, including lane 14's own ladder (the
  script re-derives edges from live `cargo metadata` and refuses). Re-check at
  execution time.
- Toolchain witnessed: cargo/rustc 1.98.0-nightly (2026-06-20/21); the
  `cargo yank pkg@version` spelling is supported.

## 1. Fresh-checkout smoke (registry-only resolution)

Two branches by law. 1a is unconditional (the 20-package G8-free subgraph);
1b is conditional on the G8 resolution (D17 registration). Encode both; never
round a 1b failure up to a 1a pass.

### 1a. Must-pass core-chain smoke (G8-free subgraph)

```console
SMOKE=$(mktemp -d)/stardf-smoke; mkdir -p "$SMOKE"; cd "$SMOKE"
cargo init --name stardf_smoke --lib
cargo add stardf-columnar@=26.11.1 stardf-arrow-core@=26.11.1 \
  stardf-arrow-rs@=26.11.1 stardf-cdt@=26.11.1 stardf-core@=26.11.1 \
  stardf-sparql-results@=26.11.1 stardf-scan-duckdb@=26.11.1
cargo metadata --format-version 1 > meta.json
cargo build 2>&1 | tee build.log   # first build compiles frozen-duckdb-sys
```

Mechanical assertions (exit non-zero on any violation):

```console
# A1: zero path-source packages beyond the smoke root itself
test "$(jq '[.packages[] | select(.source == null)] | length' meta.json)" = 1

# A2: zero git-source packages
test "$(jq '[.packages[] | select(.source != null and
  (.source | startswith("git+")))] | length' meta.json)" = 0

# A3: every stardf-* package is exactly 26.11.1 and registry-sourced
test "$(jq '[.packages[] | select(.name | startswith("stardf")) |
  select(.version == "26.11.1" and .source != null)] | length' meta.json)" \
  = "$(jq '[.packages[] | select(.name | startswith("stardf"))] | length' \
       meta.json)"

# A4: the published chain actually compiles and links
cargo build
```

Receipt: `cargo tree -e normal,build --charset ascii | tee tree.txt` (human
receipt of the published graph; the jq assertions above carry the truth —
`cargo package --list`-class gathering proves nothing, per the lane-14
nonclaims).

Offline proof (closure reachable from the registry cache alone):

```console
rm -rf target Cargo.lock
CARGO_NET_OFFLINE=true cargo build
```

### 1b. Umbrella smoke (conditional on G8 resolution)

```console
SMOKE=$(mktemp -d)/stardf-umbrella-smoke; mkdir -p "$SMOKE"; cd "$SMOKE"
cargo init --name stardf_umbrella_smoke --lib
cargo add stardf-rs@=26.11.1 stardf-rdf@=26.11.1 stardf-sql@=26.11.1 \
  stardf-toml@=26.11.1
cargo metadata --format-version 1 > meta.json   # branch point
```

- PASS branch: resolution succeeds, then run the §1a assertions A1-A3 plus
  `cargo build`.
- REGISTERED-404 branch: resolution fails with `failed to find a package ...`
  naming any of the never-published members: stardf-ed25519, stardf-gts,
  stardf-sparql-eval (consumed by stardf-rdf); stardf-shapes (stardf-sql);
  stardf-sparql-eval (stardf-toml); stardf-datalog, stardf-entail, stardf-geo,
  stardf-glob, stardf-gts, stardf-hnsw, stardf-json, stardf-markdown,
  stardf-mime, stardf-process, stardf-retrieval, stardf-shapes, stardf-shex,
  stardf-slice, stardf-sparql-eval, stardf-text, stardf-validate
  (stardf-rs hard deps); stardf-duckdb (stardf-rs optional, feature `sql`,
  also pulled by all-features docs.rs builds, §3). All are publish=false
  members — the D17/G8 registered divergence. Record the failure signature
  verbatim as a registered-divergence row; do NOT retry via `[patch]` or path
  overrides (that falsifies the registry-only claim under test).
- Optional-dep law: a published optional dep must also resolve from the
  registry once its feature is enabled; stardf-rs `sql` pulls stardf-duckdb
  (publish=false) → `--all-features`-class resolution fails under the same G8
  registration.

## 2. Dependent re-resolution (stardf-columnar consumers)

Closure consumers of stardf-columnar (re-derived from member manifests this
session; `purrdf-columnar` is a rename key resolving to package
stardf-columnar):

- stardf-arrow-capi (dev-dependency)
- stardf-scan-duckdb (normal, hard; frozen-duckdb is also a hard dep,
  `=2.0.4`)
- stardf-rs (normal, via rename key `purrdf-columnar`)

Confirm the published manifests carry the rewritten rows (path stripped by
`cargo publish`):

```console
UA="stardf-v26.11.1-publish-verify (info@chatmangpt.com)"
for spec in stardf-arrow-capi:26.11.1 stardf-scan-duckdb:26.11.1 \
            stardf-rs:26.11.1; do
  c="${spec%%:*}"; v="${spec##*:}"
  curl -sL -A "$UA" \
    "https://crates.io/api/v1/crates/$c/$v/download" -o "/tmp/$c.crate"
  tar -xzOf "/tmp/$c.crate" "$c-$v/Cargo.toml" \
    | grep -E '^(stardf-columnar|purrdf-columnar)[[:space:]]*=' \
    || echo "MISSING columnar row: $c"
done
```

Expected: every matching row is `{ version = "=26.11.1" }` with NO `path =`
and NO `git =`. Any `path`/`git` surviving in a published Cargo.toml is a hard
FAIL (cargo publish must have refused to upload it).

Fresh-consumer re-resolution of the scan-duckdb consumer:

```console
CONS=$(mktemp -d)/stardf-colcons; mkdir -p "$CONS"; cd "$CONS"
cargo init --name stardf_colcons --lib
cargo add stardf-scan-duckdb@=26.11.1
cargo metadata --format-version 1 | jq -r '
  .packages[] | select(.name == "stardf-columnar")
  | "\(.name) \(.version) \(.source)"'
# expected: stardf-columnar 26.11.1 registry+https://github.com/rust-lang/crates.io-index
cargo build   # compiles frozen-duckdb 2.0.4 first; slow
```

## 3. docs.rs build posture

- Toolchain: docs.rs builds on latest stable; every published member declares
  rust-version 1.98 and edition 2024 (stardf-arrow-capi and stardf-arrow-core
  edition 2021 — both stable-since-1.85-compatible). Optional local MSRV probe
  (only if the toolchain exists locally):
  `rustup toolchain list | grep -q '^1\.98' && cargo +1.98 check -p stardf-columnar`.
- build.rs census (registry access must be zero): the ONLY build.rs in the
  closure is `crates/stardf-rdf-core/build.rs` — it reads cargo-set env
  (RUSTC/TARGET/OPT_LEVEL/rustflags) and runs `rustc -vV` and
  `--print target-cpus`; it reads no file, no network, no clock (witnessed
  source read this session). Execution-time re-verification:

```console
cd /Users/sac/ggen-legacy/stardf/arrow
CLOSURE="stardf-hash stardf-alloc-probe stardf-arrow-core stardf-lex \
stardf-deflate stardf-branchless stardf-testkit stardf-events \
stardf-arrow-bench stardf-columnar stardf-stack stardf-iri stardf-arrow-rs \
stardf-xsd stardf-arrow-capi stardf-scan-duckdb stardf-cdt stardf-core \
stardf-sparql-algebra stardf-sparql-results stardf-rdf stardf-sql \
stardf-n3 stardf-toml stardf-rs"
for c in $CLOSURE; do
  if test -f "crates/$c/build.rs"; then
    grep -nEi 'curl|reqwest|ureq|https?://|download|git ' \
      "crates/$c/build.rs" && echo "FAIL: $c build.rs network surface" || \
      echo "OK(clean): $c"
  else
    echo "OK(no-build.rs): $c"
  fi
done
```

- frozen-duckdb family NOT in the publish closure — CONFIRMED this session:
  the closure is exactly the 25 named packages in §0; none is named
  `frozen-duckdb*`. frozen-duckdb / frozen-duckdb-sys / frozen-duckdb-builder
  are external crates.io crates (operator fork; D13/D19 ratified `=2.0.4`;
  2.0.4 published to crates.io 2026-10-30 per doc 10 of v26.10.31) consumed
  only through the workspace pin. stardf-duckdb — the workspace's own engine
  crate — is publish=false, outside the closure. frozen-duckdb-sys's docs.rs
  branch emits no link directives (same doc), so docs.rs documents without
  linking DuckDB. Registry-side re-check:

```console
curl -s -A "$UA" "https://crates.io/api/v1/crates/frozen-duckdb/2.0.4" \
  | jq -r '.version.num, .version.yanked'
# expected: 2.0.4 then false
```

- docs.rs build status for the closure (fallback: each crate page on
  docs.rs; status.json field shape is UNKNOWN — assert on `.build_status`
  being present, never on a guessed sibling field):

```console
for c in $CLOSURE; do
  curl -s -A "$UA" "https://docs.rs/crate/$c/26.11.1/status.json" \
    | jq -r '.build_status // "ENDPOINT-SHAPE-UNKNOWN -> use the crate page"'
done
```

- Expected docs.rs outcomes: the 20-package G8-free subgraph (§1a set) builds
  green. The 4 G8-affected crates (stardf-rdf, stardf-sql, stardf-toml,
  stardf-rs) carry `[package.metadata.docs.rs] all-features = true`; enabling
  features cannot cross a registry-404 wall, so their docs.rs builds are
  REGISTERED-FAIL under the G8 registration until that resolution lands —
  record the row verbatim, do not re-run hoping for drift.

## 4. crates.io page checks (25 crates)

```console
UA="stardf-v26.11.1-publish-verify (info@chatmangpt.com)"
for c in $CLOSURE; do
  curl -s -A "$UA" "https://crates.io/api/v1/crates/$c" \
    | jq -r '"\(.crate.name) \(.crate.max_version) lic=\(.versions[0].license)"'
  curl -s -A "$UA" "https://crates.io/api/v1/crates/$c" \
    | jq -r '.crate | "desc=\(.description)\nrepo=\(.repository)\nhome=\(.homepage)"'
done
```

Expected for every row (workspace-inherited values): max_version 26.11.1;
license `MIT OR Apache-2.0 OR MulanPSL-2.0`; description
"Deterministic zero-copy columnar RDF 1.2, Arrow, Parquet, and DuckDB
execution substrate"; repository
`https://github.com/seanchatmangpt/ggen-legacy`; homepage
`https://blackcatinformatics.ca/purrdf/`. `documentation` renders only on
stardf-rs and reads `https://docs.rs/purrdf` — registered divergence (lane 07
upstream-verbatim), noted so a checker does not file it as new drift.
Alternative without jq quoting risk:
`cargo info stardf-columnar@26.11.1`.

## 5. Reversal procedure (yank law)

Law: yank in the EXACT reverse of the publish order (flatten TIERS, reverse);
unyank in publish order. Deepest tier first, so no fresh resolver can name a
parent whose children are already unresolvable; a parent-first yank leaves
resolvers pinning parents whose children vanish mid-sequence.

Yank list (reverse topological; within-tier order reversed):

```console
YANK_LIST="stardf-rs stardf-toml stardf-n3 stardf-sql stardf-rdf \
stardf-sparql-results stardf-sparql-algebra stardf-core stardf-cdt \
stardf-scan-duckdb stardf-arrow-capi stardf-xsd stardf-arrow-rs stardf-iri \
stardf-stack stardf-columnar stardf-arrow-bench stardf-events stardf-testkit \
stardf-branchless stardf-deflate stardf-lex stardf-arrow-core \
stardf-alloc-probe stardf-hash"
for c in $YANK_LIST; do cargo yank "$c@26.11.1"; done
```

(`cargo yank --version 26.11.1 "$c"` is the equivalent older spelling.)

Post-yank confirmation and unyank (publish order = §0 TIERS flattened):

```console
for c in $YANK_LIST; do
  curl -s -A "$UA" "https://crates.io/api/v1/crates/$c" \
    | jq -r '.versions[] | select(.num == "26.11.1")
      | "\(.crate) \(.num) yanked=\(.yanked)"'
done
UNYANK_LIST=$(printf '%s\n' $YANK_LIST | tac)
for c in $UNYANK_LIST; do cargo unyank "$c@26.11.1"; done
```

Bounds: yank is not deletion (tarballs stay downloadable; existing Cargo.lock
files keep resolving); fresh `cargo update` and new resolutions skip yanked
versions. Requires `cargo login` with a token carrying yank permission.
Takedown beyond yank is a crates.io-team action, out of scope.

## 6. Standing ladder + receipt contract

| Step | Gates | Standing if green | Standing if red |
|---|---|---|---|
| 1a | A1-A3 + build exit 0 | ALIVE for the 20-crate G8-free chain | BUILD_BROKEN / REFUSED |
| 1b | resolution + A1-A3 | ALIVE (full 25) | REGISTERED-404 (G8) — record, not retry |
| 2 | published-manifest rows + consumer resolution | ALIVE | REFUSED (bad publish rewrite) |
| 3 | build.rs census + docs.rs status | ALIVE | BLOCKED (docs.rs) |
| 4 | page fields == workspace values | ALIVE | REFUSED (metadata drift) |
| 5 | 25 yanked=true rows | REVERSAL COMPLETE | BLOCKED (token/scope) |

Receipt: append commands + exits per executed step to `stardf/arrow/evidence/`
per the lane contract; this file is amended only with as-of-dated addenda.

## See Also

- `stardf/arrow/scripts/publish_stardf.py` — TIERS, gates G1-G9, `--publish`
  + `STARDF_PUBLISH_CONFIRM=ratified-v26.11.1` live law, 25-30 s spacing
- `stardf/docs/jira/v26.10.31/10-duckdb-rehome-plan.md` — frozen-duckdb
  D13/D19 re-home record; docs.rs no-link posture
- `stardf/arrow/docs/OPERATOR-DECISIONS.md` — D13/D17/D19 register
