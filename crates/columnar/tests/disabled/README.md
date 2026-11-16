# tests/disabled/

Parked integration test awaiting the stardf substrate crates
(`stardf_core`, `stardf_hash`, `stardf_testkit`, `stardf_alloc_probe`,
`stardf_lex`) that do not exist in this workspace yet. Cargo ignores this
directory (no top-level `tests/*.rs`).

Lane 9 parked this file (compile-freeze SLA unblock, disclosed); it belongs
to whichever lane owns the `structured_errors` court. Unpark by moving it to
`tests/structured_errors.rs` once the substrate crates land.
