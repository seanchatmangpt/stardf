# examples/disabled/ — stardf-xsd

Upstream `crates/xsd/examples/gen_exact_numeric_vectors.rs` is PARKED here,
not deleted: it consumes purrdf-testkit's exact-rational ORACLE
(`purrdf_testkit::exact::{Rational, Direction}`), the rng streams and the
vector recorder (`purrdf_testkit::vectors::Recorder`) to regenerate
`tests/vectors/exact_numeric_vectors.txt`. The upstream manifest comment
carries the law these vectors are generated under; it is preserved in
`tests/disabled/upstream-Cargo.toml.txt`.

## Unpark preconditions

1. Move the file back to `examples/` (auto-discovered) — no manifest table
   needed.
2. Verify stardf-testkit's `exact` + `vectors` surfaces (vendored upstream-
   verbatim by the testkit lane; not exercised by this lane's gates).
3. Regeneration is a REVIEWED transition upstream ("never edit by hand" on
   the generated vector files); unparking restores capability, it does not
   regenerate anything by itself.
