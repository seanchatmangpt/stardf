# stardf-sameas receipt (lane RDF-6) — owl:sameAs via SLOT-ID CANONICALIZATION

Date: 2026-11-01. Crate: `arrow/crates/stardf-sameas` (workspace member).
Status: ALIVE (commands + exits this session, listed below).

## Differentiator (DESIGN-AROUND LAW)

OST elects canonical representatives and rewrites dictionary references inside
C++ index structures. This crate never rewrites stored references.
Canonicalization happens in the DIGEST SPACE: every term id derives as
`truncate64(BLAKE3(SAMEAS_NS_V1 ++ iri))` with `SAMEAS_NS_V1 = b"stardf-sameas\x01"`
— the namespace-tag + collision-trap discipline ADAPTED from cgap A06's
`b"bont\x01"` + `slot_id` law (`docs/jira/v26.11.1/lanes/A06-bont-scalars.md`
SS 4-5; mirrored in cgap-bont `crates/cgap-bont/src/primitives.rs`). Facts are
stored under raw subject ids; queries rewrite only the LOOKUP KEY and union the
class members' facts. Collision trapping and pin layout survive unchanged
because ids are never mutated after derivation.

## Surfaces

- `term_id` / `truncate64` / `SAMEAS_NS_V1` — digest-space id derivation,
  A06-adapted; collision trap type `CollisionRefused` (fail-closed, cgap A06
  SS5 producer obligation).
- `SameAsIndex` — union-find over term ids; union elects the MINIMUM id as
  root, so canonical = `find` and the election is insertion-order free (min of
  a set is order-free).
- `FactStore` — rewrite layer: `facts_about(any_member)` unions all class
  members' facts; `declare_functional` + typed `ContradictionRefused` when
  conflicting objects exist across the class for a functional predicate.
  Re-asserting the SAME object via a synonym passes (idempotent).

## Courts (all run green this session)

1. `court1_class_query_returns_canonical_facts_from_any_member` — class union
   of 2 facts visible identically from both members. Mutation: dropping the
   rewrite layer yields 1 fact per member (fails).
2. `court2_election_deterministic_under_permutation` — forward vs reversed
   insertion order produce byte-identical canonical maps; canonical == min id
   of class. Mutation: first-seen-wins election flips the map.
3. `court3_functional_contradiction_refused` — conflicting functional facts
   across a class -> typed `ContradictionRefused` with canonical/predicate/
   both objects; same-object re-assertion passes. ANTI-VACUITY FLIP (below).
4. `court4_digest_space_ids_distinct_and_trap_ready` — distinct IRIs never
   collide on the real derivation path; derivation is a pure function.
5. `court5_rewrite_overhead_bench` — no-sleep Instant measurement.

## Anti-vacuity flip (SHIM-COURT, /tmp sandbox, shared tree untouched)

Shim: injected `if true { return Ok(()); }` (marked `// SHIM-COURT`) into the
contradiction check in a `/tmp/r6m-sbx` copy. Result: court3 FAILED
("expected Contradiction, got Ok(())"), the other 4 courts still passed —
exit 101. Shared-tree bytes hash `a16b6a6b3f8243cd` (sha256 prefix) confirmed
identical before/after (fail-closed restore). Restored sandbox re-ran 5/5
green (exit 0). Sandbox + `/tmp/r6m-target` deleted via python
`shutil.rmtree` at lane end (verified absent).

## Bench (court5, Instant, no sleeps)

- class=64, facts=4096: `facts_about` call 100,771 ns; **24 ns/returned fact**
  (fence: sub-microsecond per fact — passes).
- class=2, facts=0: 2,900 ns per call (baseline).

Numbers are TARGET-class until reproduced by an external rerun; method stated
above (Instant wall-clock, 100 iterations, mean).

## Commands + exits

- `cargo test -p stardf-sameas` (workspace, CARGO_TARGET_DIR=/tmp/r6m-target):
  5/5 pass, exit 0.
- flip run (shimmed /tmp sandbox): court3 fail, exit 101.
- restored run: 5/5 pass, exit 0.

## Divergences / limitations

- Lease edit outside `crates/stardf-sameas/**`: one 6-line comment block in
  `arrow/Cargo.toml` workspace members (authorized by the lease's "workspace
  member" clause).
- `FactStore.add_fact` requires owl:sameAs pairs to be ingested before fact
  ingestion for functional checks to see the full class; late sameAs arrival
  is correct for queries (facts_about unions live) but functional contradiction
  detection only checks facts present at insert time.
- The CGAP_MIRROR law: stardf-hash's lib name is `purrdf_hash` (vendoring law
  keeps upstream lib names), so the dep is `stardf-hash` but the import is
  `use purrdf_hash::blake3`.
- Sandbox flip needed the workspace `rust-toolchain.toml` copied (stardf-hash
  MSRV 1.98) and the stardf-hash dep re-pointed to its absolute shared-tree
  path inside the sandbox manifest (read-only consumption).
