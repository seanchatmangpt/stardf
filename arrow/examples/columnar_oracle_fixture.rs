//! Lane 04-scan-oracle (2026-10-31): deterministic five-table Parquet fixture
//! for the external DuckDB columnar oracle
//! (`stardf/arrow/scripts/check-columnar-oracle.sh`).
//!
//! Retarget (lane contract v26.11.1 finishers): the fixture is generated
//! through the real first-party codec — `stardf_columnar::write` — replacing
//! the earlier hand-rolled PLAIN/Data-Page-V2 writer that went through
//! `stardf-arrow-core`'s `TermArena`/`StardfBatch`. The dataset is the
//! standing splitmix64 fixture (state `0x6012`), the same seeding the
//! stardf-columnar `golden_bytes` byte-determinism court uses, so the oracle
//! observes exactly the bytes the golden court pins.
//!
//! Chicago discipline: real terms through the vendored `RdfDatasetBuilder`,
//! real content-addressed blobs, real codec bytes on disk; an external
//! `duckdb` process observes the physical files, never our in-memory claim.
//! The binary prints a machine-readable `ORACLE-COUNTS` line (its own
//! projection row counts) so the shell gate can assert generator-side
//! arithmetic against DuckDB's observation, not just a hardcoded literal.

use std::env;
use std::fs;
use std::path::PathBuf;

use stardf_columnar::vendor::core::{
    BlankScope, ContentStore, RdfDatasetBuilder, RdfTextDirection, TermValue,
};
use stardf_columnar::{project, write, Compression, Table};

/// Inline splitmix64 (purrdf-testkit `rng::splitmix64_next` parity; the
/// testkit is not vendored, so the 12-line algorithm is carried here —
/// reference: purrdf `crates/testkit/src/rng.rs`, identical to the
/// stardf-columnar `golden_bytes` court's inline copy).
fn mix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The standing golden fixture: 400 seeded quads over 5 predicates / 3 named
/// graphs, every term kind in the mix, RDF 1.2 reifiers + annotations on the
/// rows where the stream draws a reify step, and 9 content-addressed blobs.
/// Byte-for-byte the `golden_bytes` court's `fixture()`; keep the two in
/// step (a deliberate divergence here is a format court matter, not a
/// fixture edit).
fn fixture() -> (
    std::sync::Arc<stardf_columnar::vendor::core::RdfDataset>,
    ContentStore,
) {
    let mut state = 0x6012_u64;
    let mut builder = RdfDatasetBuilder::new();
    let predicates: Vec<_> = (0..5)
        .map(|i| {
            builder
                .intern_iri(&format!("https://example.org/p{i}"))
                .unwrap()
        })
        .collect();
    let graphs: Vec<_> = (0..3)
        .map(|i| {
            builder
                .intern_iri(&format!("https://example.org/g{i}"))
                .unwrap()
        })
        .collect();
    let empty = builder.intern_iri("https://example.org/empty").unwrap();
    builder.declare_named_graph(empty).unwrap();
    // The shim resolves a literal's datatype through the term index; every
    // datatype IRI (including the plain-literal marker) must be interned
    // first or the resolver maps it to id 0 and the lookup refuses.
    for datatype in [
        "",
        "https://example.org/integer",
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString",
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString",
    ] {
        builder.intern_iri(datatype).unwrap();
    }
    for row in 0..400_u32 {
        let subject = match mix(&mut state) % 3 {
            0 => builder
                .intern_blank(&format!("b{row}"), BlankScope(row % 5))
                .unwrap(),
            _ => builder
                .intern_iri(&format!("https://example.org/s{row}"))
                .unwrap(),
        };
        let object = match mix(&mut state) % 6 {
            0 => builder
                .intern_value(&TermValue::Literal {
                    lexical_form: format!("plain {row}"),
                    datatype: String::new(),
                    language: None,
                    direction: None,
                })
                .unwrap(),
            1 => builder
                .intern_value(&TermValue::Literal {
                    lexical_form: row.to_string(),
                    datatype: "https://example.org/integer".to_owned(),
                    language: None,
                    direction: None,
                })
                .unwrap(),
            2 => builder
                .intern_value(&TermValue::Literal {
                    lexical_form: format!("mot {row}"),
                    datatype: "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString".to_owned(),
                    language: Some("fr".to_owned()),
                    direction: None,
                })
                .unwrap(),
            3 => builder
                .intern_value(&TermValue::Literal {
                    lexical_form: format!("كلمة {row}"),
                    datatype: "http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString".to_owned(),
                    language: Some("ar".to_owned()),
                    direction: Some(if row % 2 == 0 {
                        RdfTextDirection::Rtl
                    } else {
                        RdfTextDirection::Ltr
                    }),
                })
                .unwrap(),
            4 => builder
                .intern_blank(&format!("o{row}"), BlankScope(row % 3))
                .unwrap(),
            _ => builder
                .intern_iri(&format!("https://example.org/o{row}"))
                .unwrap(),
        };
        let predicate = predicates[(mix(&mut state) % 5) as usize];
        let graph = match mix(&mut state) % 4 {
            0 => None,
            k => Some(graphs[(k - 1) as usize]),
        };
        builder
            .push_quad(subject, predicate, object, graph)
            .unwrap();
        if mix(&mut state) % 4 == 0 {
            let triple = builder.intern_triple(subject, predicate, object).unwrap();
            let reifier = builder
                .intern_blank(&format!("r{row}"), BlankScope(row % 7))
                .unwrap();
            let graph = (mix(&mut state) % 2 == 0).then_some(graphs[0]);
            builder
                .push_reifier_in_graph(reifier, triple, graph)
                .unwrap();
            let graph = (mix(&mut state) % 3 == 0).then_some(graphs[1]);
            builder
                .push_annotation_in_graph(reifier, predicates[0], object, graph)
                .unwrap();
        }
    }
    let mut blobs = ContentStore::new();
    for row in 0..9 {
        blobs.insert(format!("payload {row}").into_bytes());
    }
    (builder.freeze().expect("the fixture is valid RDF"), blobs)
}

fn main() {
    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("usage: columnar_oracle_fixture OUTPUT_DIRECTORY"));
    fs::create_dir_all(&output).expect("output directory");

    let (dataset, blobs) = fixture();
    let dataset = dataset.as_ref();

    // Real codec pass 1: the canonical projection, read for its row counts
    // (generator-side arithmetic the shell gate cross-checks against DuckDB).
    let projection = project(dataset, &blobs).expect("the fixture projects");
    let counts: Vec<String> = Table::ALL
        .map(|table| projection.row_count(table).to_string())
        .to_vec();
    println!("ORACLE-COUNTS {}", counts.join(","));

    // Real codec pass 2: the bytes on disk. One write() over the same input;
    // the golden court pins these bytes' digests.
    let written = write(dataset, &blobs, Compression::Uncompressed)
        .unwrap_or_else(|error| panic!("the fixture encodes: {error}"));
    for (table, bytes) in written.files.iter() {
        fs::write(output.join(table.file_name()), bytes)
            .expect("write table to the fixture directory");
    }
}
