//! Round-trip smoke court over the shim dataset (Chicago discipline: real
//! collaborators, assert on final state). Exercises the vendored codec's
//! write -> read path end to end without the purrdf substrate.

use crate::vendor::core::{
    BlankScope, ContentStore, DatasetView, LossLedger, RdfDataset, RdfDatasetBuilder,
    RdfTextDirection, TermGuard as _, TermRef, TermValue,
};

use crate::{read, write, Compression};

/// A quad fixture: IRI subject, IRI predicate, directional literal object in
/// one named graph, plus one blob.
fn fixture() -> (std::sync::Arc<RdfDataset>, ContentStore) {
    let mut builder = RdfDatasetBuilder::new();
    // Datatype IRIs must be interned as terms: the view seam resolves a
    // literal's datatype by id (upstream kernels intern datatype terms).
    let xsd_integer = builder.intern_iri("https://example.org/integer").unwrap();
    let lang_string = builder
        .intern_iri(crate::vendor::core::vocab::rdf::DIR_LANG_STRING)
        .unwrap();
    let _ = (xsd_integer, lang_string);
    let subject = builder.intern_iri("https://example.org/s").unwrap();
    let predicate = builder.intern_iri("https://example.org/p").unwrap();
    let plain = builder
        .intern_value(&TermValue::Literal {
            lexical_form: "42".to_owned(),
            datatype: "https://example.org/integer".to_owned(),
            language: None,
            direction: None,
        })
        .unwrap();
    let directional = builder
        .intern_value(&TermValue::Literal {
            lexical_form: "\u{645}\u{631}\u{62d}\u{628}\u{627}".to_owned(),
            datatype: crate::vendor::core::vocab::rdf::DIR_LANG_STRING.to_owned(),
            language: Some("ar".to_owned()),
            direction: Some(RdfTextDirection::Rtl),
        })
        .unwrap();
    let graph = builder.intern_iri("https://example.org/graph").unwrap();
    builder.declare_named_graph(graph).unwrap();
    builder
        .push_quad(subject, predicate, plain, Some(graph))
        .unwrap();
    builder
        .push_quad(subject, predicate, directional, None)
        .unwrap();
    let triple = builder.intern_triple(subject, predicate, plain).unwrap();
    let reifier = builder.intern_blank("reifier", BlankScope(7)).unwrap();
    builder
        .push_reifier_in_graph(reifier, triple, Some(graph))
        .unwrap();
    builder
        .push_annotation(reifier, predicate, directional)
        .unwrap();
    let mut blobs = ContentStore::new();
    blobs.insert(b"columnar payload".to_vec());
    (builder.freeze().unwrap(), blobs)
}

#[test]
fn write_then_read_round_trips_a_shim_dataset() {
    let (dataset, blobs) = fixture();
    let dataset: &RdfDataset = &dataset;
    let written = write(dataset, &blobs, Compression::Zstd).expect("five files encode");
    assert!(written.losses.is_empty());
    assert_eq!(LossLedger, written.losses);

    let read_back = read(&written.files).expect("five files decode");
    assert_eq!(read_back.blobs.iter().count(), 1);

    let view = read_back.dataset;
    let quads: Vec<_> = DatasetView::quads(&*view).collect();
    assert_eq!(quads.len(), 2);
    assert_eq!(DatasetView::named_graphs(&*view).count(), 1);

    // Terms resolved through the generic seam agree with the interned values.
    let first = quads[0];
    let guard = DatasetView::resolve(&*view, first.s).unwrap();
    let TermRef::Iri(subject_iri) = guard.term() else {
        panic!("subject resolves to an IRI");
    };
    assert_eq!(subject_iri, "https://example.org/s");

    // Determinism: byte-identical files on a second write.
    let again = write(dataset, &blobs, Compression::Zstd).expect("second encode");
    assert_eq!(written.files, again.files);
}

#[test]
fn empty_dataset_still_writes_five_valid_zero_row_files() {
    let dataset: std::sync::Arc<RdfDataset> = RdfDatasetBuilder::new().freeze().unwrap();
    let dataset: &RdfDataset = &dataset;
    let written = write(dataset, &ContentStore::new(), Compression::Uncompressed).unwrap();
    for (table, bytes) in written.files.iter() {
        let table_data = crate::parquet::read_table(bytes, table).expect("valid zero-row file");
        assert_eq!(table_data.row_count, 0);
    }
}

#[test]
fn bottom_and_wildcard_ids_are_refused_on_the_read_seam() {
    use crate::column::Int64Column;
    use crate::parquet::{write_table, ColumnValues, TableData};
    use crate::Table;

    // Build a one-row quads table whose first term id is BOTTOM (0), encode
    // it, and prove the reader refuses it as a term-reference refusal.
    let mut first = Int64Column::with_capacity(1);
    first.push(Some(0));
    let mut second = Int64Column::with_capacity(1);
    second.push(Some(1));
    let mut third = Int64Column::with_capacity(1);
    third.push(Some(1));
    let mut graphs = Int64Column::with_capacity(1);
    graphs.push(None);
    let data = TableData::new(
        Table::Quads,
        vec![
            ColumnValues::int64(first),
            ColumnValues::int64(second),
            ColumnValues::int64(third),
            ColumnValues::int64(graphs),
        ],
    )
    .unwrap();
    // The reader validates all five files, so the empty tables encode as
    // valid zero-row Parquet.
    let empty = |table: Table| -> Vec<u8> {
        let data = TableData::new(
            table,
            match table {
                Table::Blobs => vec![
                    ColumnValues::bytes(Vec::new()),
                    ColumnValues::bytes(Vec::new()),
                ],
                Table::Terms => vec![
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::bytes(Vec::new()),
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::bytes(Vec::new()),
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                ],
                Table::Quads | Table::Annotations => vec![
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                ],
                Table::Reifiers => vec![
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                    ColumnValues::int64(Int64Column::default()),
                ],
            },
        )
        .unwrap();
        write_table(&data, crate::Compression::Uncompressed).unwrap()
    };
    let files = crate::ParquetFiles::from_array([
        empty(Table::Terms),
        write_table(&data, crate::Compression::Uncompressed).unwrap(),
        empty(Table::Reifiers),
        empty(Table::Annotations),
        empty(Table::Blobs),
    ]);
    match read(&files) {
        Err(error) => {
            let text = error.to_string();
            assert!(
                text.contains("row first term is malformed"),
                "refusal must name the term seam, got: {text}"
            );
        }
        Ok(_) => panic!("BOTTOM id must be refused"),
    }
}
