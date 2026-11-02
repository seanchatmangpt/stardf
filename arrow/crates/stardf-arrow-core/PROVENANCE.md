# PROVENANCE — stardf-arrow-core

## Status and origin

First-party StarDF work (v26.11.1 first-class closure campaign, 2026-10-31).
Authored in this workspace for the Quad64 arrow layer: a `repr(C, align(64))`
Quad64 batch with BLAKE3 term fingerprints exported through the C Data
Interface. No upstream source is incorporated; this crate carries no vendored
third-party code or data.

## License carriage

Declared license: `MIT OR Apache-2.0 OR MulanPSL-2.0`.

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

Runtime deps (`avec-arrow` git pin, `blake3`) are external crates consumed via
registry/git, not incorporated source; their terms travel with their own
published packages. No third-party notice inventory is required in-package.

See `crates/PROVENANCE-entries.md` for the campaign-wide entry register.
