# tests/disabled/ — parked upstream tests (cargo ignores this directory)

Upstream purrdf rev `0da0578306983da851a9073337d57e2e2c41ab3a` carried three
integration tests for this crate. Two of them (`structured_errors.rs`,
`allocation_budget.rs`) set `CountingAllocator` as the `#[global_allocator]`
and import `purrdf_alloc_probe` — the first-party alloc-metering crate, which
is NOT in this lane's vendoring scope and does not exist in the stardf/arrow
workspace. Cargo ignores `tests/disabled/`; they are parked, not deleted.

- `structured_errors.rs` — parked (requires `purrdf_alloc_probe`)
- `allocation_budget.rs` — parked (requires `purrdf_alloc_probe`)
- `known_answers.rs` — ACTIVE in `tests/` (imports `purrdf_hash` only; runs
  against the vendored lib with zero additional deps).

Unpark precondition: vendor `crates/alloc-probe` as `stardf-alloc-probe`
(same copy-and-rename discipline, purrdf rev pin recorded in
VENDOR-PROVENANCE.md), restore the dropped
`purrdf-alloc-probe = { workspace = true }` dev-dependency key as
`stardf-alloc-probe`, move both files back into `tests/`, and re-run
`cargo test -p stardf-hash`. Until then these two suites are a loud skip,
not a pass: they are absent from the run, and this README is the register.
