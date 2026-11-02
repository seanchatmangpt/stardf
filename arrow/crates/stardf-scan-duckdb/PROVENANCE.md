# PROVENANCE — stardf-scan-duckdb

## Status and origin

First-party StarDF work (v26.11.1 first-class closure campaign, 2026-10-31):
greenfield `stardf_scan()` DuckDB table function over the closed five-table
columnar profile (docs/SCAN-COMPLETION.md §4 W2a–W2d), authored by lane
02-scan-scaffold. No upstream source is incorporated; this crate carries no
vendored third-party code or data. `avec-arrow` is NOT consumed anywhere in
this crate (crates.io publish law: feature-gated or decoupled — decoupled
here).

This notice was added by lane 11-license-carriage (2026-10-31 re-run) so the
published tarball is legally self-contained.

## License carriage

Declared license: `MIT OR Apache-2.0 OR MulanPSL-2.0`
(`license.workspace = true`).

- [`licenses/LICENSE-APACHE`](licenses/LICENSE-APACHE)
  (sha256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`)
- [`licenses/LICENSE-MIT`](licenses/LICENSE-MIT)
  (sha256 `958c0e61f0ccde102a636644133533b4c76c939765acdbb790791f68546000f3`)
- [`licenses/LICENSE-MULAN`](licenses/LICENSE-MULAN)
  (sha256 `eb7a1d713eb919b146787629e22e4c975cb701f529a65d4d7e0fcd417558bf1c`)
- [`licenses/NOTICE.txt`](licenses/NOTICE.txt)
  (sha256 `77430cb214e12d4a98902eb8875b795212a4a2cadc80a2f0e97bbf13b74ea30f`)
- [`licenses/inventory.json`](licenses/inventory.json) (schema 1, empty
  components; per-file sha256 census of the carriage files themselves,
  sha256 `cafc5f9382c9508a1aacacea6ae7b81f7abd7026439950ce9bde1889c8013b8c`)

Texts are byte-identical to the workspace canonical copies in
`crates/stardf-hash/licenses/`.

## Dependencies

Runtime deps `frozen-duckdb` (workspace pin `=2.0.4`, MIT — registered with a
compatibility receipt in `authority/stardf/license-compat.json`) and
`stardf-columnar` (intra-workspace, tri-licensed) are consumed as packages,
not incorporated source. Optional `stardf-arrow-core` behind feature
`arrow-core` is intra-workspace, tri-licensed. No third-party notice inventory
is required in-package.

See `crates/PROVENANCE-entries.md` for the campaign-wide entry register.
