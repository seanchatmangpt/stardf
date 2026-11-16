# PROVENANCE — stardf-duckdb

## Status and origin

First-party StarDF work (v26.11.1 first-class closure campaign, 2026-10-31).
Embedded DuckDB execution engine, virtual Parquet views and relational-to-RDF
row projection over `frozen-duckdb`. No upstream source is incorporated; no
vendored third-party code or data in-package.

## License carriage

Declared license: workspace tri-license
`MIT OR Apache-2.0 OR MulanPSL-2.0` (`license.workspace = true`).

- [`licenses/LICENSE-APACHE`](licenses/LICENSE-APACHE)
  (sha256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`)
- [`licenses/LICENSE-MIT`](licenses/LICENSE-MIT)
  (sha256 `958c0e61f0ccde102a636644133533b4c76c939765acdbb790791f68546000f3`)
- [`licenses/LICENSE-MULAN`](licenses/LICENSE-MULAN)
  (sha256 `eb7a1d713eb919b146787629e22e4c975cb701f529a65d4d7e0fcd417558bf1c`)

Texts are byte-identical to the workspace canonical copies in
`crates/stardf-hash/licenses/`. This notice ships inside the package so every
published tarball is legally self-contained.

## Dependencies

`frozen-duckdb` is an external registry crate consumed as a dependency, not
incorporated source; its terms travel with its own published package. No
third-party notice inventory is required in-package.

See `crates/PROVENANCE-entries.md` for the campaign-wide entry register.
