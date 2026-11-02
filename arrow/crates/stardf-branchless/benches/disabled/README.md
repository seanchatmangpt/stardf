# benches/disabled — parked bench, loud not silent

`kernel_table.rs-pending-stardf-substrate` is the upstream
`benches/kernel_table.rs` moved byte-identical (sha256
ab007502519adaa3dea996c4bac7519b804892590de23a92b6d0ee50d8b55389) from the
stardf-branchless vendor of PurRDF rev
0da0578306983da851a9073337d57e2e2c41ab3a. It is parked, not deleted, because
its only unavailable collaborator is stardf-testkit: the bench imports
`purrdf_testkit::bench::{Bench, Throughput, bench_group, bench_main, black_box}`
(lib `purrdf_testkit`, upstream name kept verbatim per the vendoring law), and
no stardf-testkit crate exists in this workspace yet.

Cargo ignores the `disabled/` subdirectory; without the park, the bench would
either fail to compile under --all-targets or force a dev-dep declaration on a
crate that does not exist (manifest-level failure — it would break even plain
`cargo check -p stardf-branchless`).

Unpark when BOTH hold:

1. `stardf/arrow/crates/stardf-testkit/` exists, its lib name is
   `purrdf_testkit`, and it exposes
   `purrdf_testkit::bench::{Bench, Throughput, bench_group, bench_main, black_box}`.
2. Someone owns the unpark transition: move the file back to
   `benches/kernel_table.rs`, restore the `[[bench]]` section
   (`name = "kernel_table"`, `harness = false`) and the renamed dev-dep key
   (`stardf-testkit = { path = "../stardf-testkit" }`).

Until then the bench is a loud registered divergence (this README +
PROVENANCE.md divergence 2), never a silent skip.
