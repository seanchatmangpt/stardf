# benches/disabled/ — stardf-xsd

Upstream `crates/xsd/benches/` (`exact.rs`, `lexical.rs`, `numerical.rs` — all
`harness = false` [[bench]] targets) are PARKED here, not deleted: they
consume purrdf-testkit's bench harness (`Bench`, `Throughput`, `bench_group`,
`bench_main`, `black_box`).

## Unpark preconditions

1. Restore the three `[[bench]]` tables to `crates/stardf-xsd/Cargo.toml` and
   move the files back to `benches/*.rs`.
2. Verify stardf-testkit's bench surface (`testkit::bench`) — vendored
   upstream-verbatim by the testkit vendoring lane; not exercised by this
   lane's gates.
3. The benches also `use purrdf_testkit::rng::splitmix64_next` — covered by
   the same substrate.
