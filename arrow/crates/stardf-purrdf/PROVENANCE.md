# PROVENANCE — stardf-rs (umbrella, dir crates/stardf-purrdf)

## Status and origin

Vendored PurRDF umbrella facade (purrdf rev 0da05783, package `stardf-rs`,
lib name `purrdf` pinned per the vendoring law), carried through the v26.11.1
first-class closure campaign. Vendored members of the umbrella's re-export
surface carry their own in-package `VENDOR-PROVENANCE.md`/`PROVENANCE.md`
receipts; this notice covers the umbrella package itself.

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
`crates/stardf-hash/licenses/`. This notice ships inside the package so the
published tarball is legally self-contained.

## Dependencies

All runtime deps are registry crates or intra-workspace path deps; neither
class is incorporated source of this package. No third-party notice inventory
is required in-package.

See `crates/PROVENANCE-entries.md` for the campaign-wide entry register.
