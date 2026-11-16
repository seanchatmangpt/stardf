# RDF-5 Incremental Receipt — stardf-incremental (page-generation invalidation)

Date: 2026-11-01. Lane: RDF-5-incremental. Repo: /Users/sac/stardf (arrow workspace).

## Design-around law (named)

OST's incremental maintenance is per-fact Backward/Forward chaining through
in-memory indices. stardf pages are IMMUTABLE and footer-sequenced — deletion
here = page invalidation + fresh delta-page sealing (A05-style all-or-nothing
page transition: records + commit record checksummed; a commit applies only
when its full record set checksums; torn tails are dropped on replay, never
half-applied). Materialized views recompute PER-DELTA-PAGE, not per-fact.

## Trade table (page granularity vs OST per-fact granularity)

| dimension | page-generation invalidation (this lane) | OST per-fact chaining |
|---|---|---|
| delete unit | whole page invalidated; survivors resealed into delta pages | single fact |
| bulk churn | wins: 16.4M facts/sec at 90% delete fraction (measured) | per-fact overhead dominates |
| surgical edit | loses: ~455 us per single-fact edit (page reseal dominates) | ~0.25 us (measured) |
| durability | log + commit checksum; power-cut-safe (byte-boundary sweep passed) | in-memory indices; no page log |
| torn state | zero (byte-cut sweep, 6 courts) | n/a |
| workload boundary | crossover ~1,800 facts per delete operation (455us / 0.25us) | surgical edits below crossover |
| view maintenance | per delta page (one fold per sealed delta page) | per fact |

Boundary: per-fact incremental wins while each delete operation touches fewer
than ~1,800 facts; bulk page-granularity wins above it. Both numbers measured
this session (commands below).

## Files written

- arrow/crates/stardf-incremental/Cargo.toml
- arrow/crates/stardf-incremental/src/{lib,page,store,view}.rs
- arrow/crates/stardf-incremental/tests/incremental_courts.rs
- arrow/crates/stardf-incremental/examples/bench_bulk.rs
- arrow/docs/ost/incremental.md (this receipt)
- arrow/Cargo.toml — ONE line: workspace member registration (required by the
  lease's "workspace member" clause; coordinator-visible divergence: the lease
  text listed only the crate dir + receipt, but membership requires the
  manifest line).

## Commands (this session)

- `CARGO_TARGET_DIR=/tmp/r5m-target cargo build -p stardf-incremental` — exit 0
- `CARGO_TARGET_DIR=/tmp/r5m-target cargo test -p stardf-incremental` — exit 0,
  6 passed / 0 failed
- `CARGO_TARGET_DIR=/tmp/r5m-target cargo run --release -p stardf-incremental
  --example bench_bulk` — exit 0, output above in report

## Courts (all passing; names -> evidence)

1. court_delete_then_query_matches_oracle — 6 pages, deletes spanning >=5
   pages; store fact set == full-recompute oracle set equality.
2. court_footer_sequencing — delta pages expose footers (kind, count).
3. court_power_cut_never_torn — kill at EVERY byte boundary of the durable
   log; no committed invalidation without its reseal; no invalidated-page
   fact visible un-resealed.
4. court_view_per_delta_page_matches_oracle — maintained CountView ==
   full recompute after per-delta-page folds.
5. court_stale_fact_flip (ANTI-VACUITY): mutation = re-materialize
   invalidated pages into the live read path (invalidation never consulted).
   Flip witnessed: deleted fact becomes queryable under the mutation; the
   base court asserts it absent. A court that cannot flip is FAIL.
6. court_replay_roundtrip — replay of the encoded log reproduces fact set,
   invalidated set, delta ids exactly.

## Standing

ALIVE: built, courts green, bench measured this session on the exact subject
(/Users/sac/stardf/arrow/crates/stardf-incremental). Two court-found defects
were repaired this session (PartialEq over bookkeeping fields; court seeded
the store twice so the durable log lacked invalidated pages) — evidence the
courts are not vacuous.

## Divergences

- arrow/docs/ost/ did not exist; created it (lease names the path).
- arrow/Cargo.toml members line added beyond the literal lease file list
  (required for workspace membership).
- Bench crossover is measured only on the synthetic corpus in bench_bulk.rs
  (32,768 facts / 256 pages); no OST implementation was benched side-by-side —
  the per-fact comparison is a hash-set removal baseline, stated as such.
