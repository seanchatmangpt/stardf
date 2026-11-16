# RDF-4: quoted triples as content-addressed nested slots — receipt

Lane: RDF-4-quoted (2026-11-01). Crate: `arrow/crates/stardf-star`
(workspace member; root `Cargo.toml` members list edited to register it).

## The design-around (named law)

OST eliminated RDF-star's ~400% reification overhead by embedding compound
terms inside their dictionary encoding. stardf eliminates it DIFFERENTLY:
a quoted triple ⟨⟨s p o⟩⟩ gets its own CONTENT-ADDRESSED id —

    id = truncate64(BLAKE3("stardf-star:quoted-triple:v1" || canonical(s,p,o)))

The id IS the digest: first-class, integrity-checkable BY CONSTRUCTION.
Storage is ordinary triples whose subject or object position holds the
quoted id; edge metadata attaches as ordinary triples ON the quoted id. No
dictionary machinery, no reification inflation.

## Discipline citation (not wholesale import)

Truncation law mirrors stardf's own `derive_seed`
(`arrow/crates/stardf-gts/src/dict.rs:158-168`): BLAKE3 output, first 8
bytes LE as u64. stardf-lex carries no collision/trap handling for
truncated digests (grep of stardf-lex/src: no collision/truncate-digest
surface), so the 64-bit truncation gets the cgap A06 trap treatment
adapted: domain-separated preimage (`QUOTED_DOMAIN` in `term.rs`) and
injectivity asserted on the live corpus (court 2: 66 ids, all distinct;
integrity re-derivation on every payload). Birthday bound for 64 bits is
~5e9 pairs, so a corpus-scale injectivity failure indicates a derivation
bug, not a birthday hit.

## Surfaces

- `src/term.rs` — Term model (Iri/Blank/Literal/Quoted), canonical bytes,
  `quoted_id`, `quote`, hex display.
- `src/store.rs` — triple store; `quote` registers payload under id
  (idempotent — content addressing dedupes equal payloads);
  `metadata_of` / `used_as_object`; `verify_integrity` (every payload
  re-derives its id); `flat_projection` (reified differential baseline).
- `src/parser.rs` — Turtle-star subset parser (`<<s p o>>`, @prefix, one
  nesting level, deep-nesting refusal).
- `tests/quoted_courts.rs` — 5 courts.
- `scripts/courts.sh --flip` — anti-vacuity sandbox flip (SHIM-COURT:
  metadata attached to the WRONG quoted id; the court set must fail).
- `examples/overhead.rs` — storage-overhead measurement.

## Court receipts (all run this session)

`cargo test -p stardf-star` exit 0, 5/5 pass:

1. court1 round-trip parse/store/query (4 statements, 4 quoted triples,
   nested case included, metadata round-trips).
2. court2 corpus: 66 quoted triples (60 flat + 6 nested), all ids
   distinct, `verify_integrity()==0`, metadata present per id,
   deterministic across stores.
3. court3 differential: flat projection (324 reified rows) fact-set-equal
   to the native quoted path, per row.
4. court4 wrong-id isolation: metadata namespaces of two quoted ids are
   disjoint; wrong-id query returns none of the other id's facts.
5. court5 parser subset: comments/prefix/lang/datatype/nesting + two-level
   nesting refused.

### Anti-vacuity flip (sandbox)

`scripts/courts.sh --flip` exit 0:

- BASELINE sha256 of the sandbox .rs corpus: `13fa3bd354b4...f66655e0`
- BEFORE: unmutated sandbox court4 run PASSES (exit 0)
- SHIM (SHIM-COURT): metadata attached to the WRONG quoted id →
  `test result: FAILED. 4 passed; 1 failed` → court flipped, gate holds
- RESTORE: sandbox removed; shared tree sha256 re-measured identical
  (`13fa3bd3...`), never touched.

## Storage overhead (honest number, measured `cargo run --example overhead`)

Same fact set (66 quoted triples + 126 metadata edges):

- quoted-slot store: **126 rows**
- flat reified projection (our own baseline): 324 rows
- classical RDF reification (4 rows per statement + re-pointed metadata):
  **390 rows**

Quoted-slot storage is 32.3% of classical reification row count over the
identical fact set — i.e. a ~3.1x reduction against the naive scheme the
~400% figure describes, achieved WITHOUT a dictionary encoding: identity
is a digest, not a dictionary lookup. Edge metadata costs exactly what
it costs for ground triples (one row per edge), which is why total row
count (126) sits at parity with the metadata edges themselves plus the
registry, not at reification multiples.

## Standing

ALIVE (courts + flip + measurement all executed this session). Cargo
target `/tmp/r4m-target` deleted at integration per lane contract.
