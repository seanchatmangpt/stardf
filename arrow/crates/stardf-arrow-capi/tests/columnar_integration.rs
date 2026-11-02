//! PHASE-2 LANE 14 — THE WAVE GATE: the one real W-columnar integration test.
//!
//! Round-trip through the real collaborators, no mocks:
//!
//! 1. intern terms into a real `StardfArena` via the C-ABI functions
//!    `stardf_arena_new` / `stardf_arena_intern` — the actual membrane;
//! 2. append quads via `stardf_batch_append_quad` (the P0-16 cross-check
//!    pattern: the same fixture drives both the columnar codec and the
//!    batch-append path);
//! 3. implement a minimal `stardf_columnar` vendored `DatasetView` over the
//!    arena + batch (arena-interned term ids -> `QuadIds<u32>`);
//! 4. run `stardf_columnar::write` (five-table Parquet codec) and
//!    `stardf_columnar::read` back;
//! 5. assert the round-trip is lossless (terms, quads, named graphs) and the
//!    row counts cross-check against `stardf_batch_append_quad`.
//!
//! LANE-14 UNBLOCKS (disclosed, see both target files for the markers):
//! 1. `stardf-arrow-capi/Cargo.toml` gained `[dev-dependencies]
//!    stardf-columnar = { path = ".." }` — without it the gate cannot compile;
//! 2. `stardf-columnar/src/lib.rs` `mod vendor` -> `pub mod vendor` — the
//!    vendored `DatasetView` trait and its associated types must be nameable
//!    by an external implementor.

use std::collections::BTreeSet;
use std::convert::Infallible;

use stardf_arrow_capi::{
    stardf_arena_free, stardf_arena_intern, stardf_arena_new, stardf_batch_append_quad,
    StardfArena, StardfBatch,
};
use stardf_columnar::vendor::core::{
    BlankScope, ContentStore, DatasetView, QuadIds, TermGuard, TermRef, TermValue,
};
use stardf_columnar::{read, write, Compression, Table};

// ---------------------------------------------------------------------------
// Fixture: the term table + quad list both the arena path and the view share.
// ---------------------------------------------------------------------------

/// A fixture term as written: kind plus the fields the arena cannot carry
/// (the arena is a pure lexical interner; the view resolves ids against this
/// table).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FixtureTerm {
    /// IRI.
    Iri(&'static str),
    /// Typed / language-tagged literal. `datatype` names the fixture IRI
    /// whose arena id is the datatype id (the vendored `TermRef::Literal`
    /// carries the datatype as a term id, so the datatype IRI must itself be
    /// interned).
    Literal {
        lexical: &'static str,
        datatype: &'static str,
        language: Option<&'static str>,
    },
    /// Blank node with a dataset-local scope id.
    Blank { label: &'static str, scope: u32 },
}

impl FixtureTerm {
    /// `TermValue` projection (datatype as IRI string, for read-back compare).
    fn value(&self) -> TermValue {
        match self {
            FixtureTerm::Iri(iri) => TermValue::Iri((*iri).to_string()),
            FixtureTerm::Literal {
                lexical,
                datatype,
                language,
            } => TermValue::Literal {
                lexical_form: (*lexical).to_string(),
                datatype: (*datatype).to_string(),
                language: (*language).map(|l| l.to_string()),
                direction: None,
            },
            FixtureTerm::Blank { label, scope } => TermValue::Blank {
                label: (*label).to_string(),
                scope: BlankScope(*scope),
            },
        }
    }

    /// Canonical lexical key interned into the arena (kind-prefixed so an IRI
    /// and a literal lexical can never collide onto one id).
    fn arena_key(&self) -> Vec<u8> {
        match self {
            FixtureTerm::Iri(iri) => {
                let mut k = b"I|".to_vec();
                k.extend_from_slice(iri.as_bytes());
                k
            }
            FixtureTerm::Literal {
                lexical,
                datatype,
                language,
            } => {
                let mut k = b"L|".to_vec();
                k.extend_from_slice(lexical.as_bytes());
                k.extend_from_slice(b"|");
                k.extend_from_slice(datatype.as_bytes());
                k.extend_from_slice(b"|");
                k.extend_from_slice(language.unwrap_or("").as_bytes());
                k
            }
            FixtureTerm::Blank { label, scope } => {
                let mut k = b"B|".to_vec();
                k.extend_from_slice(label.as_bytes());
                k.extend_from_slice(b"|");
                k.extend_from_slice(scope.to_string().as_bytes());
                k
            }
        }
    }
}

/// The fixture: six terms interned in declaration order (arena ids 1..=6),
/// then the two datatype IRIs (ids 7..=8). Distinct per-row values so a lossy
/// round-trip cannot pass by aliasing.
const TERMS: &[FixtureTerm] = &[
    FixtureTerm::Iri("http://example.org/s"),
    FixtureTerm::Iri("http://example.org/p"),
    FixtureTerm::Iri("http://example.org/g"),
    FixtureTerm::Literal {
        lexical: "42",
        datatype: "http://www.w3.org/2001/XMLSchema#integer",
        language: None,
    },
    FixtureTerm::Literal {
        lexical: "hola",
        datatype: "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString",
        language: Some("es"),
    },
    FixtureTerm::Blank {
        label: "b0",
        scope: 7,
    },
];

/// Datatype IRIs: interned after TERMS, so their arena ids are
/// `TERMS.len() + position`.
const DATATYPES: &[&str] = &[
    "http://www.w3.org/2001/XMLSchema#integer",
    "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString",
];

/// The five quads as 1-based fixture ids `(s, p, o, g)`; `g == 0` means the
/// default graph (BOTTOM).
const QUADS: &[(u32, u32, u32, u32)] = &[
    (1, 2, 4, 3), // named graph (fixture id 3)
    (1, 2, 5, 3), // named graph
    (6, 2, 4, 3), // named graph, blank subject
    (1, 2, 4, 0), // default graph (BOTTOM)
    (6, 2, 5, 0), // default graph
];

/// Built state shared by the two real collaborators: the live C-ABI arena +
/// its batch, plus the id -> term table the view resolves against.
struct Fixture {
    /// Live arena handle (C-ABI allocation).
    arena: *mut StardfArena,
    /// The arena-owned batch, read back through `stardf_batch_append_quad`.
    batch: *mut StardfBatch,
    /// Arena id (index + 1) -> fixture term. Datatype IRIs occupy the tail.
    term_of_id: Vec<FixtureTerm>,
    /// The five quad rows in batch order, graph as Some(id)/None (BOTTOM).
    quad_rows: Vec<(u32, u32, u32, Option<u32>)>,
}

impl Fixture {
    /// Intern TERMS + DATATYPES, append QUADS, return the built fixture.
    fn build() -> Self {
        let arena = unsafe { stardf_arena_new(1 << 16) };
        assert!(!arena.is_null(), "arena allocation refused");

        // Intern every fixture term through the real C-ABI membrane; ids land
        // in declaration order (TERMS first, datatype IRIs after).
        let mut term_of_id: Vec<FixtureTerm> = Vec::new();
        let all_terms = TERMS
            .iter()
            .cloned()
            .chain(DATATYPES.iter().map(|d| FixtureTerm::Iri(d)));
        for term in all_terms {
            let key = term.arena_key();
            let id = unsafe { stardf_arena_intern(arena, key.as_ptr(), key.len()) };
            assert_ne!(id, 0, "intern refused for {term:?}");
            assert_eq!(
                id as usize,
                term_of_id.len() + 1,
                "arena ids must land in declaration order"
            );
            term_of_id.push(term.clone());
        }

        // Append the quads. Named-graph rows go through the real C-ABI
        // append; default-graph rows go through the rlib batch directly
        // (stardf_batch_append_quad refuses g = 0, the null-term guard) —
        // still the same real batch the C-ABI path appends into.
        let mut batch: *mut StardfBatch = core::ptr::null_mut();
        let mut quad_rows: Vec<(u32, u32, u32, Option<u32>)> = Vec::new();
        for &(s, p, o, g) in QUADS {
            let id_of = |fixture_id: u32| fixture_id; // fixture ids ARE arena ids (declaration order)
            let (sid, pid, oid) = (id_of(s), id_of(p), id_of(o));
            let appended = if g == 0 {
                let b = unsafe { (*arena).batch_mut() };
                b.quads.push([sid, pid, oid, 0]);
                b as *mut StardfBatch
            } else {
                unsafe { stardf_batch_append_quad(arena, sid, pid, oid, g) }
            };
            assert!(
                !appended.is_null(),
                "batch append refused for ({sid},{pid},{oid},{g})"
            );
            batch = appended;
            quad_rows.push((sid, pid, oid, if g == 0 { None } else { Some(g) }));
        }

        Fixture {
            arena,
            batch,
            term_of_id,
            quad_rows,
        }
    }

    /// Arena id of an interned datatype IRI (the tail of `term_of_id`).
    fn arena_id_of_datatype(&self, datatype: &str) -> Option<u32> {
        self.term_of_id
            .iter()
            .position(|t| matches!(t, FixtureTerm::Iri(name) if *name == datatype))
            .map(|i| i as u32 + 1)
    }

    /// Append count observable through the batch: the P0-16 cross-check
    /// anchor — columnar quad row counts must equal this.
    fn batch_row_count(&self) -> usize {
        unsafe { (*self.batch).quads.len() }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        unsafe { stardf_arena_free(self.arena) }
    }
}

// ---------------------------------------------------------------------------
// Minimal DatasetView over StardfArena/StardfBatch.
// ---------------------------------------------------------------------------

/// Guard resolving one arena id through the fixture table.
struct ArenaGuard<'a> {
    fixture: &'a Fixture,
    id: u32,
}

impl TermGuard<u32> for ArenaGuard<'_> {
    fn term(&self) -> TermRef<'_, u32> {
        let term = &self.fixture.term_of_id[(self.id - 1) as usize];
        match term {
            FixtureTerm::Iri(iri) => TermRef::Iri(iri),
            FixtureTerm::Literal {
                lexical,
                datatype,
                language,
            } => {
                let dt_id = self
                    .fixture
                    .arena_id_of_datatype(datatype)
                    .unwrap_or_else(|| panic!("datatype IRI {datatype} not interned"));
                TermRef::Literal {
                    lexical,
                    datatype: dt_id,
                    language: *language,
                    direction: None,
                }
            }
            FixtureTerm::Blank { label, scope } => TermRef::Blank {
                label,
                scope: BlankScope(*scope),
            },
        }
    }
}

/// The view: borrows the live arena fixture.
struct ArenaDatasetView<'a> {
    fixture: &'a Fixture,
}

impl<'view> DatasetView for ArenaDatasetView<'view> {
    type Id = u32;
    type ReadError = Infallible;
    type TermGuard<'guard>
        = ArenaGuard<'guard>
    where
        Self: 'guard;

    fn checked_read<T>(
        &self,
        read: impl FnOnce(&Self) -> Result<T, stardf_columnar::ColumnarWriteError<Infallible>>,
    ) -> Result<T, stardf_columnar::ColumnarWriteError<Infallible>> {
        // Pass-through: the completed-read boundary carries the (never
        // inhabited, for this fixture) session failure whole.
        read(self)
    }

    fn quads(&self) -> impl Iterator<Item = QuadIds<u32>> {
        self.fixture
            .quad_rows
            .iter()
            .map(|&(s, p, o, g)| QuadIds { s, p, o, g })
    }

    fn reifier_quads(&self) -> impl Iterator<Item = QuadIds<u32>> {
        std::iter::empty()
    }

    fn annotation_quads(&self) -> impl Iterator<Item = QuadIds<u32>> {
        std::iter::empty()
    }

    fn named_graphs(&self) -> impl Iterator<Item = u32> {
        // The single named graph the fixture appends (fixture/arena id 3).
        std::iter::once(3u32)
    }

    fn resolve(&self, id: u32) -> Result<ArenaGuard<'_>, Infallible> {
        assert!(
            id >= 1 && (id as usize) <= self.fixture.term_of_id.len(),
            "resolve out of range: {id}"
        );
        Ok(ArenaGuard {
            fixture: self.fixture,
            id,
        })
    }
}

// ---------------------------------------------------------------------------
// The court.
// ---------------------------------------------------------------------------

#[test]
fn w_columnar_arena_roundtrip_is_lossless_and_row_counts_cross_check() {
    let fixture = Fixture::build();

    // Cross-check anchor (P0-16 pattern): the batch the append path built
    // holds exactly the five fixture rows.
    assert_eq!(fixture.batch_row_count(), 5, "batch row count anchor");
    assert_eq!(fixture.quad_rows.len(), 5);

    // Write through the real five-table codec.
    let view = ArenaDatasetView { fixture: &fixture };
    let written = write(&view, &ContentStore::new(), Compression::Uncompressed)
        .expect("columnar write of the arena fixture");
    assert!(
        written.losses.is_empty(),
        "lossless fixture must produce an empty loss ledger, got: {:?}",
        written.losses
    );

    // All five tables are present as valid (non-empty) Parquet files.
    for table in [
        Table::Terms,
        Table::Quads,
        Table::Reifiers,
        Table::Annotations,
        Table::Blobs,
    ] {
        let bytes = written.files.get(table);
        assert!(
            !bytes.is_empty(),
            "{table:?} must be a valid (non-empty) Parquet file"
        );
    }

    // Read back.
    let readback = read(&written.files).expect("columnar read of the written files");
    assert!(readback.losses.is_empty(), "read losses must be empty");

    let ds = &readback.dataset;

    // Term set is preserved. The codec reassigns ids canonically, so the
    // assertion is on the TermValue sets, not the id words.
    let before: BTreeSet<TermValue> = fixture.term_of_id.iter().map(|t| t.value()).collect();
    let after: BTreeSet<TermValue> = ds.terms().iter().cloned().collect();
    assert_eq!(before, after, "term set must survive the round-trip");

    // Quad set is preserved as TermValue tuples.
    let quad_before: BTreeSet<(TermValue, TermValue, TermValue, Option<TermValue>)> = fixture
        .quad_rows
        .iter()
        .map(|&(s, p, o, g)| {
            let tv = |id: u32| fixture.term_of_id[(id - 1) as usize].value();
            (tv(s), tv(p), tv(o), g.map(tv))
        })
        .collect();
    let quad_after: BTreeSet<_> = ds
        .quads()
        .map(|QuadIds { s, p, o, g }| {
            let tv = |id: u32| ds.terms()[(id - 1) as usize].clone();
            (tv(s), tv(p), tv(o), g.map(tv))
        })
        .collect();
    assert_eq!(
        quad_before, quad_after,
        "quad set must survive the round-trip"
    );

    // Named graph survived. Ids are reassigned canonically by the codec, so
    // the expected id is the canonical row of the graph term, not the arena
    // id (3).
    let graph_term = TermValue::Iri("http://example.org/g".to_string());
    let expected_graph_id = ds
        .terms()
        .iter()
        .position(|t| *t == graph_term)
        .expect("graph term present in the reconstructed dictionary")
        + 1;
    let graphs_after: BTreeSet<u32> = ds.named_graph_ids().collect();
    assert_eq!(graphs_after, BTreeSet::from([expected_graph_id as u32]));

    // P0-16 cross-check: quads row count == batch append count.
    assert_eq!(quad_after.len(), fixture.batch_row_count());
}
