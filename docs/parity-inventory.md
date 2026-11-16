# Parity evidence inventory (S-5, court-auditor)

Date: 2026-11-01. Read-only audit of /Users/sac/stardf (arrow workspace);
one in-tree receipt file: docs/parity-inventory.md. All cargo runs were made
in a /tmp sandbox copy (/tmp/s5-arrow, deleted after use) because the shared
tree is manifest-broken: crates/stardf-arrow-capi/Cargo.toml:60-61 point
cgap-audit / cgap-ffi at "../../../../../cgap/..." which resolves to
/Users/cgap/... (one path level short of /Users/sac/cgap). Every in-tree
cargo command exited 101 at audit start. Note (20:16 local, mid-audit): a
concurrent actor changed the dep paths to "../../../../cgap/..." (resolves to
/Users/sac/cgap - correct); in-tree cargo metadata now exits 0. The cargo
courts below were still witnessed in the sandbox copy, not the shared tree.

## Bucket 1 - RUNS-GREEN (witnessed this session, in a sandbox copy whose
## only delta is the corrected cgap path dep; otherwise byte-identical tree)

| surface | command | result |
|---|---|---|
| stardf-columnar golden_bytes (byte-determinism courts incl. positive control flipping_a_byte_moves_the_digest) | cargo test -p stardf-columnar | exit 0, 5 passed / 0 failed |
| stardf-columnar seam_courts (TermId/Quad64 sentinel invariants) | cargo test -p stardf-columnar | exit 0, 4 passed |
| stardf-columnar lib + structured_errors | cargo test -p stardf-columnar | exit 0 |
| stardf-hash known_answers (RFC 1321 / NIST FIPS 180-2 / FIPS 202 / CRC-32) | cargo test -p stardf-hash | exit 0 (10 / 64 / 28 / 5 across targets) |
| stardf-lex | cargo test -p stardf-lex | exit 0 (198 / 12 / 90) |
| stardf-deflate | cargo test -p stardf-deflate | exit 0 (7 passed; upstream suite parked in tests/disabled/) |
| stardf-rs umbrella (vendored purrdf surface: base_surface, closure_relations, governed_entailment, aggregate_sort_witnesses, dataset_state_digest, etc.) | cargo test -p stardf-rs | exit 0, all targets 0 failed |
| stardf-rdf serialize_sink_parity | cargo test -p stardf-rdf --test serialize_sink_parity | exit 0, 3 passed |
| stardf-rdf serialize_pack_parity | cargo test -p stardf-rdf --test serialize_pack_parity | exit 0, 2 passed |
| stardf-entail pack_view_parity | cargo test -p stardf-entail --test pack_view_parity | exit 0, 3 passed |
| stardf-sparql-eval relation_witness | --test relation_witness | exit 0, 20 passed |
| stardf-sparql-eval function_body_relation_witness | --test function_body_relation_witness | exit 0, 7 passed |
| stardf-sparql-eval path_witness_property_function | --test path_witness_property_function | exit 0, 18 passed |
| stardf-validate dl_nominal_witnesses | --test dl_nominal_witnesses | exit 0, 2 passed |
| stardf-scan-duckdb scan_courts | cargo test -p stardf-scan-duckdb | exit 0, 6+6 passed |
| stardf-duckdb chicago courts (6 chicago_*_court.rs files) | cargo test -p stardf-duckdb | exit 0, all targets 0 failed |
| stardf-arrow-capi cgap_consumer_court (incl. court_3 dlopen of the real cdylib) | cargo build -p stardf-arrow-capi, then cargo test with the default target dir (court hardcodes <ws>/target/debug dylib path) | exit 0, 3 passed |
| header/ontology parity gate | python3 scripts/verify.py | exit 0: 13 decls == 13 exports, three-way ontology equality |
| verify.py anti-vacuity arm | python3 scripts/verify.py --mutate drop-export | exit 1 (expected): mutation correctly FAILs the gate - court flips |
| single-identity gate | bash scripts/verify_blake3_identity.sh | exit 0, "exactly 1 blake3 identity: blake3 v1.8.7" |
| columnar oracle | bash scripts/check-columnar-oracle.sh | exit 0, scan parity 1030,400,108,108,9 |

## Bucket 2 - RUNS-RED (coordinates)

| surface | command | result |
|---|---|---|
| stardf-sparql-eval unit tests (--lib target) | cargo test -p stardf-sparql-eval | SIGABRT stack overflow in the lib target; witness integration targets pass |
| every cargo command in the SHARED tree | any cargo invocation | exit 101: broken cgap path dep (see header note) |

## Bucket 3 - DECLARED-NOT-RUN (exists, not executed this session)

| surface | status |
|---|---|
| stardf-process tests/process_courts.rs | UNRUNNABLE as-is: crate is neither a workspace member nor excluded; cargo refuses with workspace-ambiguity (exit 101) |
| stardf-columnar tests/disabled/structured_errors.rs-pending-stardf-substrate | parked, named pending |
| stardf-deflate tests/disabled/{allocation_budget,deflate_conformance}.rs | parked upstream suite; VENDOR-PROVENANCE admits the upstream suite is NOT executed |
| stardf-arrow-capi stream_courts / cross_allocator_court | present; not re-run this session (prior receipt standing only) |
| justfile rehome-court / header-freeze court | receipt standing only (evidence/duckdb-rehome-two-arm-court-receipt-2026-10-31.json); not re-run this session (arms rename $HOME paths, 60-min bound) |
| ci/stardf-arrow-reconstitution.yml | itself declares cargo test --workspace --locked is still RED; court-by-receipt only |
| workflows/v26.11.1-full-parity.workflow.js | node --check exits 1 (Illegal return statement, line 541) - the full-parity workflow is syntactically invalid |
| docs/12-full-parity-campaign.md | banner: DEFERRED by the 80/20 re-scope |

## Bucket 4 - MISSING (parity claim, no court at all)

| claim | location | court status |
|---|---|---|
| digest-parity vs upstream purrdf-hash | docs/DIVERGENCES.md KD-12; docs/V26-11-1-VERDICT.md:149 | none exists - grep for a purrdf_hash differential test finds none |
| purrdf digest-ordering parity (columnar bytes vs upstream purrdf/columnar) | docs/DIVERGENCES.md KD-14 | none. Witnessed: upstream /Users/sac/purrdf/crates/columnar/tests/golden_bytes.rs pins and stardf-columnar PINNED share ZERO digests (10 vs 10, disjoint) - golden_bytes proves self-determinism, not upstream parity |
| 100% census parity (TermIdArenaDefinition binding) | docs/ticket-001-state.md:31,77 | no named court file; closest executed court is seam_courts (green) - claim over-reaches its court |
| full-parity campaign | workflows/v26.11.1-full-parity.workflow.js | the nearest executable check (node --check) FAILS, exit 1 |

## Honest parity statement

PROVEN (witnessed 2026-11-01): the vendored crates run green on their own
suites (stardf-hash/lex/deflate/columnar + the stardf-rs umbrella), the
columnar byte court proves write determinism and a flipping positive
control, the header/ontology three-way parity gate runs and its anti-vacuity
mutation flips, blake3 single-identity and the columnar oracle pass, and the
serialize/pack-view/witness parity tests across stardf-rdf, stardf-entail,
stardf-sparql-eval and stardf-validate pass.

CLAIMED-NOT-PROVEN: parity WITH UPSTREAM purrdf. Every upstream-parity claim
(digest parity vs purrdf-hash, columnar digest-ordering parity, full-parity
campaign) is either registered as open in docs/DIVERGENCES.md with no court,
or is backed by a syntactically invalid workflow. The vendored crates are
byte-verbatim to purrdf rev 0da05783 on the declared surfaces per
VENDOR-PROVENANCE receipts (2026-10-31, cmp/diff evidence), but no court
re-proves it today against the live upstream tree, and the columnar byte
pins deliberately diverge from upstream pins.

Standing: PARTIAL_ALIVE - parity is proven against self-referential and
standard-vector courts; upstream-parity remains CLAIMED, courtless, or
broken at the manifest level (cgap path dep) in the shared tree.
