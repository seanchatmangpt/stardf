# Parity vs OST / RDFox capability frame (RDF-RECON, court-auditor)

Date: 2026-11-01. Read-only audit of /Users/sac/stardf (arrow 53-crate workspace);
single in-tree receipt file: docs/parity-vs-ost.md. Ground truth: ~/stardf/arrow.
No prior OST/RDFox deep-dive exists in cgap waves/ (grep for 'rdfox|oxford'
returned zero matches there). Frame read from first principles per the task.

## Capability gap table (OST frame)

| # | capability | in-tree evidence | verdict |
|---|---|---|---|
| P1 | parallel materialization (lock-free shared-memory Datalog) | stardf-datalog has seminaive/chase/schedule but NO lock-free shared-memory parallel engine: no rayon, no concurrent fixpoint; schedule.rs:1128-1140 pins guard stages to the CALLER thread (connected_joins_keep_guard_stages_on_the_caller_thread). grep lock-free/lockfree across crates/*/src: 0 matches. | GREENFIELD |
| P2 | SIP join planning | stardf-datalog/src/plan.rs:36,360 (binding-aware sideways-information-passing order); src/demand.rs:355 adornment closure + minimal antichains; src/lib.rs:177 names the SIP order as a certified plan frame; cache.rs:69,218,388 couple SIP order into plan identity. | PRESENT |
| P3 | SIP demand transformation (magic sets) | demand.rs:7 (subsuming adornments share one demand relation) + demand/stratified.rs; adornment lattice is demand-transformed, but magic-set rewriting is not present by name. | PARTIAL |
| P4 | stratified negation | demand.rs:223-243 routes stratified negation to demand/stratified.rs evaluate; guard.rs:47-49 unstratifiable forms refused BY NAME at compile. | PRESENT |
| P5 | RDF 1.2 quoted triples | stardf-cdt/src/term.rs:107,151,195 (<<( s p o )>> triple terms, unbounded nesting, element counts); stardf-bench/src/lib.rs:322-323,624,1158-1162 (rdf:reifies reifier rows; corpus asserts >0 triple terms each bound by exactly one reifier). | PRESENT |
| P6 | incremental deletion | stardf-datalog/src/store.rs:59,408,4927-4939 signed-weight retraction: deletion falls out of the merge, no special deletion pass (test asserts (+1)+(-1)=0 annihilates the row); maintenance.rs signed change-stream transactions with watermark contract. Not yet RDFox-class incremental MATERIALIZATION (no delta rules over the materialized closure) - retraction is input-level. | PARTIAL |
| P7 | MVCC | tree-wide grep for mvcc/multi-version/isolation-level across crates/*/src and docs: 0 source matches (only serde-serializable false positives). | GREENFIELD |

## cgap-advantage rows (present in stardf, absent from OST/RDFox)

| capability | in-tree evidence | verdict |
|---|---|---|
| footer-digested pages | stardf-columnar reader.rs:40-57,527-544 content-addressed payloads, every blob digest verified on read; golden_bytes byte-determinism courts incl. positive control flipping_a_byte_moves_the_digest (S-5 receipt docs/parity-inventory.md, exit 0). | PRESENT |
| 13-symbol frozen C-ABI | crates/stardf-arrow-capi: 13 no_mangle exports == 13 header decls == 13 ontology targetSymbols; scripts/verify.py three-way equality exit 0 this session; v2 symbol set frozen in S-4 receipt docs/abi-freeze-witness.md. | PRESENT |
| receipt economy | stardf-rdf-capi/src/error.rs:174 purrdf_error_execution_receipt_json; stardf-datalog cache.rs:214 Derivation names its producing clause; contract_hash-keyed plan cache (cache.rs:534-558). | PRESENT |
| DuckDB scan courts | stardf-scan-duckdb scan_courts: 6+6 passed this session (cargo test -p stardf-scan-duckdb, exit 0); stardf-duckdb 6 chicago court files (S-5 receipt). | PRESENT |
| verify.py anti-vacuity arm | scripts/verify.py --mutate drop-export exits 1 (S-5 receipt: the gate flips). | PRESENT |
