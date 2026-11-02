# PROVENANCE — stardf-arrow-bench

## Status and origin

First-party StarDF work (v26.11.1 first-class closure campaign, 2026-10-31).
Line-rate benches for stardf-arrow-core (intern, branchless pattern-scan,
stream get_next throughput) with a counting-allocator non-vacuity gate. No
upstream source is incorporated; no vendored third-party code or data.

## License carriage

Declared license: `MIT` (crate-declared keep, disclosed by lane
07-publish-metadata as differing from the workspace tri-license, awaiting
operator arbitration — unchanged here).

- [`licenses/LICENSE-MIT`](licenses/LICENSE-MIT)
  (sha256 `958c0e61f0ccde102a636644133533b4c76c939765acdbb790791f68546000f3`)

Byte-identical to the workspace canonical MIT text in
`crates/stardf-hash/licenses/`. This notice ships inside the package so the
published tarball is legally self-contained. If the operator arbitrates the
expression to the tri-license, LICENSE-APACHE/LICENSE-MULAN must be added
here in the same pass.

## Dependencies

Runtime dep (path dep stardf-arrow-core) travels with this workspace, not the
tarball. No third-party notice inventory is required in-package.

See `crates/PROVENANCE-entries.md` for the campaign-wide entry register.
