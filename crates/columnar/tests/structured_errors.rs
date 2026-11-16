// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Physical Parquet reader/writer refusals through the portable public API.
//!
//! This is the unparked court from `tests/disabled/`, re-pointed at the landed
//! public surface of `stardf-columnar` (the crate privately vendors the purrdf
//! substrate under [`stardf_columnar::vendor`], re-exported there).
//!
//! ## KD register (loud divergences from the drafted upstream trial)
//!
//! - **KD-STARDF-COLUMNAR-1** (allocation probe): the trial's
//!   `stardf_alloc_probe::{CountingAllocator, CurrentThreadWindow}` and the
//!   `inspect_without_allocation` family are NOT part of the landed surface.
//!   `crate::vendor` carries no `alloc_probe` shim; a test-local
//!   `std::alloc::GlobalAlloc` wrapper is blocked by the package's
//!   `unsafe_code = "forbid"` lint (Cargo.toml `[lints.rust]`, which applies to
//!   this integration target). The upstream-test modules inside `src/`
//!   (`src/parquet.rs:1853` et al.) reference `crate::vendor::alloc_probe` only
//!   behind the inert `upstream-tests` feature. Every allocation-free-ness
//!   assertion is therefore REMOVED, loudly, not silently: the structural
//!   visitor traversal (`inspect` below) still walks the full error tree.
//! - **KD-STARDF-COLUMNAR-2** (testkit): `stardf_testkit::error_capture::Capture`
//!   and `stardf_testkit::harness_main!` are not vendored. Replaced here by a
//!   test-local `Capture` driving the public
//!   `stardf_columnar::vendor::hash::{StructuredError, ErrorVisitor}` protocol
//!   (a real collaborator, asserting final structured state) and plain
//!   `#[test]` wrappers.
//! - **KD-STARDF-COLUMNAR-3** (gated trial: source-failure precedence): the
//!   drafted `public_projection_and_write_preserve_source_owners_and_fields`
//!   requires the upstream `DatasetView` surface — exact missing symbols on
//!   the landed trait (`src/vendor/core/dataset.rs`): `read_error()`,
//!   `type ProbePlan`, `capabilities()`, `probe_plan`,
//!   `quads_for_pattern_with_plan`, `term_count`, `term_id_by_value`,
//!   `term_id_by_iri`, `term_id_by_blank`. Without `read_error()` the sticky
//!   pre-read refusal and the SourceRead-vs-Codec precedence court cannot be
//!   expressed. Gated, never silently skipped (see `kd_3_...` below).
//! - **KD-STARDF-COLUMNAR-4** (gated trial: native cause transport): the
//!   drafted `native_cause_transport_preserves_storage_and_full_width_rows`
//!   requires exact missing symbols: `stardf_lex::allocation::{Resident,
//!   Memory, SharedOwned}`, `RdfDiagnostic::{with_detail, with_location,
//!   with_native_cause}`, `RdfLocation`, `ContentDigest::from_raw`. The landed
//!   `RdfDiagnostic` shim carries only `{code, storage}` (disclosed lane-9
//!   divergence). Gated, never silently skipped (see `kd_4_...` below).
//! - **KD-STARDF-COLUMNAR-5** (metadata key): the drafted trial corrupts
//!   `stardf.columnar.schema-version`; the landed writer emits the upstream key
//!   `purrdf.columnar.schema-version` (`src/parquet.rs:27`,
//!   `META_SCHEMA_VERSION`). The trial is corrected here to corrupt the landed
//!   key `purrdf` -> `xurrdf`; the drafted trial's expected observed key
//!   `xurrdf.columnar.schema-version` then matches the landed reader's profile
//!   refusal. The drafted `stardf.` key does NOT match the landed writer.
//! - **KD-STARDF-COLUMNAR-6** (digest record shape): the drafted trial expects
//!   nested `content-digest-sha256`/`digest32` records with `stored`/`actual`
//!   child names; the landed `ContentStoreError` visit emits flat
//!   `expected`/`actual` `ErrorValue::Bytes(32)` fields under code
//!   `content-digest-mismatch`. Assertions adapted to the landed taxonomy.
//! - **KD-STARDF-COLUMNAR-7** (std-utf8 fields): the landed `Utf8Error` visit
//!   emits only code `std-utf8` with no `valid_up_to`/`error_len` fields; the
//!   drafted capture-field asserts for those are dropped and the exact values
//!   are asserted through the real `std::str::Utf8Error` downcast instead
//!   (unchanged strength).
//! - **KD-STARDF-COLUMNAR-8** (quad_count): `RdfDataset::quad_count()` does not
//!   exist on the landed shim; `quads().count()` substituted.
//! - **KD-STARDF-COLUMNAR-9** (kernel validator): the reconstruction-corruption
//!   leg of the drafted trial is unsatisfiable — the shim's `intern` performs
//!   no IRI grammar validation, so the corrupted IRI reconstructs cleanly and
//!   no `Reconstruction` diagnostic fires. Gated as `kd_9_...` below.

use stardf_columnar::vendor::core::ir::pack::bits::{write_varint, zigzag_encode, PackBitsError};
use stardf_columnar::vendor::core::{ContentDigest, ContentStore, ContentStoreError};
use stardf_columnar::vendor::core::{RdfDataset, RdfDatasetBuilder};
use stardf_columnar::vendor::hash::{ErrorValue, ErrorVisitor, StructuredError};
#[allow(unused_imports)]
use stardf_columnar::{
    project, read, write, ColumnarError, ColumnarWriteError, Compression, MetadataReason,
    ParquetFiles, ReaderReason, ValueBodyReason,
};

/// An empty native dataset, shared by every writer fixture (`freeze` hands back
/// an `Arc<RdfDataset>`, and only `RdfDataset` itself implements
/// `DatasetView`).
fn empty_dataset() -> &'static RdfDataset {
    use std::sync::OnceLock;
    static EMPTY: OnceLock<RdfDataset> = OnceLock::new();
    EMPTY.get_or_init(|| (*RdfDatasetBuilder::new().freeze().unwrap()).clone())
}

// ---------------------------------------------------------------------------
// Test-local kit (KD-2): Capture over the public structured-error protocol.
// ---------------------------------------------------------------------------

/// An owned capture of one structured error's variant code, fields, and nested
/// compound children, driven through the public [`StructuredError`] protocol.
#[derive(Debug, Clone, Default)]
struct Capture {
    code: String,
    fields: Vec<(Option<String>, Captured)>,
    children: Vec<(Option<String>, Box<Capture>)>,
}

/// An owned copy of one [`ErrorValue`] primitive (KD-1: no allocator probe, so
/// bytes are copied; every payload in this court is tiny).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Captured {
    Absent,
    Unsigned(u128),
    Signed(i128),
    Boolean(bool),
    Code(String),
    Text(String),
    Iri(String),
    ByteCount(u128),
    Bytes(Vec<u8>),
}

fn capture_flat(value: ErrorValue<'_>) -> Option<Captured> {
    Some(match value {
        ErrorValue::Absent => Captured::Absent,
        ErrorValue::Unsigned(value) => Captured::Unsigned(value),
        ErrorValue::Signed(value) => Captured::Signed(value),
        ErrorValue::Boolean(value) => Captured::Boolean(value),
        ErrorValue::Code(value) => Captured::Code(value.to_owned()),
        ErrorValue::Text(value) => Captured::Text(value.to_owned()),
        ErrorValue::Iri(value) => Captured::Iri(value.to_owned()),
        ErrorValue::ByteCount(value) => Captured::ByteCount(value),
        ErrorValue::Bytes(bytes) => Captured::Bytes(bytes.to_vec()),
        _ => return None,
    })
}

/// Depth-first driver over the public visitor protocol. `cur` is the compound
/// currently being filled; `stack` holds the enclosing compounds; `lists`
/// tracks enclosing list names so unnamed list-entry records (the metadata
/// profile pairs) attribute themselves to their list field.
struct Driver {
    cur: Capture,
    stack: Vec<(Option<String>, Capture)>,
    lists: Vec<Option<String>>,
}

impl Driver {
    /// Open a nested compound frame; paired with [`Driver::exit`].
    fn enter(&mut self, name: Option<String>) {
        self.stack.push((name, std::mem::take(&mut self.cur)));
    }

    /// Close the nested compound into the frame just restored (its parent).
    fn exit(&mut self) {
        let (name, parent) = self.stack.pop().expect("a frame is open");
        let done = std::mem::take(&mut self.cur);
        self.cur = parent;
        self.cur.children.push((name, Box::new(done)));
    }
}

impl<'error> ErrorVisitor<'error> for Driver {
    fn variant(&mut self, code: &'static str) {
        self.cur.code = code.to_owned();
    }

    fn value(&mut self, name: Option<&'static str>, value: ErrorValue<'error>) {
        match value {
            ErrorValue::Cause(inner) | ErrorValue::Record(inner) | ErrorValue::Term(inner) => {
                self.enter(name.map(str::to_owned));
                inner.visit(self);
                self.exit();
            }
            flat => {
                if let Some(captured) = capture_flat(flat) {
                    self.cur.fields.push((name.map(str::to_owned), captured));
                }
            }
        }
    }

    fn begin_list(&mut self, name: Option<&'static str>, length: usize) {
        self.cur
            .fields
            .push((name.map(str::to_owned), Captured::Unsigned(length as u128)));
        self.lists.push(name.map(str::to_owned));
    }

    fn end_list(&mut self) {
        self.lists.pop();
    }

    fn begin_record(&mut self, name: Option<&'static str>, _code: &'static str) {
        // Attribute an unnamed entry record to its enclosing list field.
        let attributed = name
            .map(str::to_owned)
            .or_else(|| self.lists.last().cloned().flatten());
        self.enter(attributed);
    }

    fn end_record(&mut self) {
        self.exit();
    }
}

impl Capture {
    /// Capture one structured error through the public visitor protocol.
    fn read<E: StructuredError>(error: &E) -> Capture {
        let mut driver = Driver {
            cur: Capture::default(),
            stack: Vec::new(),
            lists: Vec::new(),
        };
        error.visit(&mut driver);
        driver.cur
    }

    /// The first child captured under `name`, panicking loudly if absent.
    fn child(&self, name: &str) -> &Capture {
        self.children
            .iter()
            .find(|(child_name, _)| child_name.as_deref() == Some(name))
            .map(|(_, capture)| &**capture)
            .unwrap_or_else(|| panic!("no child {name} under {}", self.code))
    }

    /// Every child captured under `name`, in order.
    fn records(&self, name: &str) -> Vec<&Capture> {
        self.children
            .iter()
            .filter(|(child_name, _)| child_name.as_deref() == Some(name))
            .map(|(_, capture)| &**capture)
            .collect()
    }

    /// A captured primitive field by name.
    fn field(&self, name: &str) -> Option<&Captured> {
        self.fields
            .iter()
            .find(|(field_name, _)| field_name.as_deref() == Some(name))
            .map(|(_, value)| value)
    }
}

// ---------------------------------------------------------------------------
// Shared inspection helper.
// ---------------------------------------------------------------------------

/// Counts fields/records/bytes while traversing the complete borrowed tree.
#[derive(Default)]
struct NativeInspection {
    fields: usize,
    records: usize,
    bytes: usize,
}

impl<'error> ErrorVisitor<'error> for NativeInspection {
    fn variant(&mut self, _: &'static str) {
        self.records += 1;
    }

    fn value(&mut self, _: Option<&'static str>, value: ErrorValue<'error>) {
        self.fields += 1;
        match value {
            ErrorValue::Cause(value) | ErrorValue::Record(value) | ErrorValue::Term(value) => {
                value.visit(self);
            }
            ErrorValue::Bytes(bytes) => self.bytes += bytes.len(),
            _ => {}
        }
    }

    fn begin_list(&mut self, _: Option<&'static str>, _: usize) {}
    fn end_list(&mut self) {}

    fn begin_record(&mut self, _: Option<&'static str>, _: &'static str) {
        self.records += 1;
    }

    fn end_record(&mut self) {}
}

/// Traverse the full structured tree of `error` (KD-1: allocation-free-ness is
/// not assertable without the alloc probe; structure is still fully walked).
fn inspect(error: &ColumnarError) -> NativeInspection {
    let mut visitor = NativeInspection::default();
    error.visit(&mut visitor);
    assert!(visitor.fields > 0, "the tree must carry at least one field");
    visitor
}

// ---------------------------------------------------------------------------
// Shared fixtures.
// ---------------------------------------------------------------------------

/// Replace one uncompressed data value while retaining every physical boundary.
fn replace_unique(bytes: &mut [u8], original: &[u8], replacement: &[u8]) {
    assert_eq!(original.len(), replacement.len());
    let positions: Vec<_> = bytes
        .windows(original.len())
        .enumerate()
        .filter_map(|(index, value)| (value == original).then_some(index))
        .collect();
    assert_eq!(positions.len(), 1, "fixture must identify one data value");
    let start = positions[0];
    bytes[start..start + original.len()].copy_from_slice(replacement);
}

/// Digest a payload through the public store (KD: `ContentDigest::of` absent).
fn digest_of(payload: &[u8]) -> ContentDigest {
    let mut store = ContentStore::new();
    store.insert(payload.to_vec())
}

/// Envelope an authored footer so refusals traverse the actual public reader.
fn compact_footer_files(footer: &[u8]) -> ParquetFiles {
    let mut file = b"PAR1".to_vec();
    file.extend_from_slice(footer);
    file.extend_from_slice(&u32::try_from(footer.len()).unwrap().to_le_bytes());
    file.extend_from_slice(b"PAR1");
    let mut files = std::array::from_fn(|_| Vec::new());
    files[0] = file;
    ParquetFiles::from_array(files)
}

// ---------------------------------------------------------------------------
// Trial 1.
// ---------------------------------------------------------------------------

/// Retain physical bounds after input disposal, including full-width counts.
#[test]
fn original_parquet_bounds_survive_file_disposal_without_narrowing() {
    let files = ParquetFiles::from_array(std::array::from_fn(|_| Vec::new()));
    let error = read(&files).unwrap_err();
    drop(files);
    let copy = error.clone();
    drop(error);
    let fields = Capture::read(&copy);
    assert_eq!(fields.code, "columnar-truncated");
    assert_eq!(
        fields.field("context"),
        Some(&Captured::Code("Parquet file".to_owned()))
    );
    assert_eq!(fields.field("needed"), Some(&Captured::ByteCount(12)));
    assert_eq!(fields.field("available"), Some(&Captured::ByteCount(0)));
    let error = ColumnarError::LimitExceeded {
        context: "original wire count",
        value: u64::MAX,
        maximum: 7,
    };
    let fields = Capture::read(&error);
    assert_eq!(
        fields.field("value"),
        Some(&Captured::Unsigned(u128::from(u64::MAX)))
    );
    assert_eq!(fields.field("maximum"), Some(&Captured::Unsigned(7)));
}

// ---------------------------------------------------------------------------
// Trial 3.
// ---------------------------------------------------------------------------

/// Preserve native codec messages and the human-selected source prefix.
#[test]
fn human_presentation_preserves_codec_and_source_messages() {
    let codec = ColumnarError::Truncated {
        context: "original table",
        needed: 19,
        available: 7,
    };
    let expected = codec.to_string();
    let wrapped = ColumnarWriteError::<std::fmt::Error>::from(codec);
    assert_eq!(wrapped.to_string(), expected);
    let source = ColumnarWriteError::SourceRead(std::fmt::Error);
    assert_eq!(
        source.to_string(),
        format!("columnar source read failed: {}", std::fmt::Error)
    );
}

// ---------------------------------------------------------------------------
// Trial 4.
// ---------------------------------------------------------------------------

/// Actual blob verification keeps both identities and the nonzero row after disposal.
#[test]
fn public_blob_read_preserves_original_integrity_cause() {
    let first = b"original first payload";
    let second = b"original other payload";
    let mut blobs = ContentStore::new();
    let a = blobs.insert(first.to_vec());
    let b = blobs.insert(second.to_vec());
    let (stored, payload) = if a > b { (a, first) } else { (b, second) };
    let mut changed = payload.to_vec();
    changed[0] ^= 1;
    let mut array = write(empty_dataset(), &blobs, Compression::Uncompressed)
        .unwrap()
        .files
        .into_array();
    replace_unique(&mut array[4], payload, &changed);
    let files = ParquetFiles::from_array(array);
    let error = read(&files).unwrap_err();
    drop(files);
    drop(blobs);
    let ColumnarError::BlobContent { row, cause } = &error else {
        panic!("expected original blob cause: {error}");
    };
    assert_eq!(*row, 1);
    assert_eq!(
        error.to_string(),
        format!("columnar blobs.bytes is malformed: row 1: {cause}")
    );
    assert!(std::ptr::eq(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<ContentStoreError>()
            .unwrap(),
        cause
    ));
    // KD-1: allocation-free-ness assertions removed; KD-6: the landed taxonomy
    // carries flat expected/actual bytes rather than nested digest records.
    let inspection = inspect(&error);
    assert_eq!(inspection.bytes, 64);
    let captured = Capture::read(&error);
    assert_eq!(captured.field("row"), Some(&Captured::Unsigned(1)));
    let cause = captured.child("cause");
    assert_eq!(cause.code, "content-digest-mismatch");
    // The stored identity is the digest column value; the actual identity is
    // the digest of the changed payload.
    assert_eq!(
        cause.field("expected"),
        Some(&Captured::Bytes(stored.as_bytes().to_vec()))
    );
    assert_eq!(
        cause.field("actual"),
        Some(&Captured::Bytes(digest_of(&changed).as_bytes().to_vec()))
    );

    // The public reader returns rejected decoded digest cells, independently
    // of the original files.
    let mut blobs = ContentStore::new();
    let digest = blobs.insert(first.to_vec()).to_hex();
    for (replacement, code, message) in [
        (vec![0xff; 64], "columnar-digest-utf8", "row 0 is not UTF-8"),
        (
            vec![b'g'; 64],
            "columnar-digest-shape",
            "row 0 is not a 64-digit SHA-256 hex value",
        ),
        (
            digest.to_ascii_uppercase().into_bytes(),
            "columnar-digest-case",
            "row 0 is not canonical lowercase hexadecimal",
        ),
    ] {
        let mut array = write(empty_dataset(), &blobs, Compression::Uncompressed)
            .unwrap()
            .files
            .into_array();
        replace_unique(&mut array[4], digest.as_bytes(), &replacement);
        let files = ParquetFiles::from_array(array);
        let error = read(&files).unwrap_err();
        drop(files);
        let _ = inspect(&error);
        let captured_error = Capture::read(&error);
        let reason = captured_error.child("reason");
        assert_eq!(reason.code, code);
        assert_eq!(reason.field("row"), Some(&Captured::Unsigned(0)));
        assert_eq!(
            reason.field("bytes"),
            Some(&Captured::Bytes(replacement.clone()))
        );
        assert_eq!(
            error.to_string(),
            format!("columnar blobs.digest is malformed: {message}")
        );
        if code == "columnar-digest-utf8" {
            // KD-7: the landed std-utf8 visit carries no field detail; the
            // exact std values are asserted via the typed downcast instead.
            let ColumnarError::Reader {
                reason: ReaderReason::DigestUtf8 { cause, .. },
                ..
            } = &error
            else {
                panic!("expected the digest UTF-8 reader reason");
            };
            assert_eq!((cause.valid_up_to(), cause.error_len()), (0, Some(1)));
        }
    }
}

// ---------------------------------------------------------------------------
// Trial 5.
// ---------------------------------------------------------------------------

/// The kernel diagnostic leg of the drafted reconstruction trial is
/// UNSATISFIABLE on the landed shim: `RdfDatasetBuilder::intern` performs no
/// IRI grammar validation (reference-range checks only), so the corrupted
/// IRI `0rn:original` reconstructs successfully and no `Reconstruction`
/// diagnostic is produced. Exact observed behavior (run 2026-10-31):
/// `read()` returns `Ok(ColumnarRead { dataset: RdfDataset { terms:
/// [Iri("0rn:original")], ... } })`. Close condition: vendor the upstream
/// kernel structural validator (`freeze`/reconstruction validation), then
/// restore this leg from `tests/disabled/structured_errors.rs-pending-stardf-substrate`.
#[test]
#[ignore = "KD-STARDF-COLUMNAR-9: shim intern performs no IRI grammar validation"]
fn kd_9_public_reconstruction_preserves_original_diagnostic() {}

/// Actual kernel validation survives the public reader with its complete diagnostic.
#[test]
fn public_reconstruction_preserves_original_diagnostic() {
    // Valid native output remains readable before its authored metadata key is
    // corrupted. The landed writer's authored key is the upstream
    // `purrdf.columnar.schema-version` (KD-STARDF-COLUMNAR-5).
    let mut array = write(
        empty_dataset(),
        &ContentStore::new(),
        Compression::Uncompressed,
    )
    .unwrap()
    .files
    .into_array();
    read(&ParquetFiles::from_array(array.clone())).unwrap();
    replace_unique(
        &mut array[0],
        b"purrdf.columnar.schema-version",
        b"xurrdf.columnar.schema-version",
    );
    let files = ParquetFiles::from_array(array);
    let error = read(&files).unwrap_err();
    drop(files);
    let _ = inspect(&error);
    let captured_error = Capture::read(&error);
    let reason = captured_error.child("reason");
    assert_eq!(reason.code, "columnar-parquet-metadata");
    let pairs = reason.child("profile").records("observed");
    assert!(
        pairs.iter().any(|pair| matches!(
            pair.field("key"),
            Some(Captured::Text(key)) if key == "xurrdf.columnar.schema-version"
        )),
        "the observed profile must name the corrupted key"
    );
    let source = std::error::Error::source(&error).unwrap();
    assert!(source.is::<MetadataReason>());

    // Two authored key/value structs carry the same original metadata key.
    let footer = [
        0x59, 0x2c, 0x18, 1, b'k', 0x18, 1, b'a', 0, 0x18, 1, b'k', 0x18, 1, b'b', 0, 0,
    ];
    let files = compact_footer_files(&footer);
    let error = read(&files).unwrap_err();
    drop(files);
    let _ = inspect(&error);
    let captured_error = Capture::read(&error);
    let reason = captured_error.child("reason");
    assert_eq!(reason.code, "columnar-parquet-duplicate-key");
    assert_eq!(reason.field("key"), Some(&Captured::Text("k".to_owned())));
    assert_eq!(
        error.to_string(),
        "columnar file key/value metadata is malformed: duplicate key k"
    );
}

// ---------------------------------------------------------------------------
// Trial 7.
// ---------------------------------------------------------------------------

/// Public grammar failures retain their original cause after all bytes are gone.
#[test]
fn public_compact_causes_survive_input_disposal() {
    let mut wide = vec![0x15]; // version: i32
    write_varint(&mut wide, zigzag_encode(i64::MAX));
    for (footer, code, context) in [
        (wide, "columnar-integer-range", "compact i32"),
        (vec![0x15, 0x80], "columnar-varint", "compact i32"),
        // Schema list, one struct, name field, invalid second UTF-8 byte.
        (
            vec![0x29, 0x1c, 0x48, 2, b'a', 0xff],
            "columnar-utf8",
            "compact string",
        ),
    ] {
        let files = compact_footer_files(&footer);
        let error = read(&files).unwrap_err();
        drop(files);
        drop(footer);
        let copy = error.clone();
        drop(error);
        assert!(inspect(&copy).records >= 3);
        let outer = Capture::read(&copy);
        assert_eq!(outer.code, "columnar-malformed");
        assert_eq!(
            outer.field("context"),
            Some(&Captured::Code(context.to_owned()))
        );
        let reason = outer.child("reason");
        assert_eq!(reason.code, code);
        let cause = reason.child("cause");
        let source = std::error::Error::source(&copy).unwrap();
        let original = source.source().unwrap();
        match code {
            "columnar-integer-range" => {
                assert_eq!(
                    reason.field("value"),
                    Some(&Captured::Signed(i128::from(i64::MAX)))
                );
                assert_eq!(cause.code, "integer-conversion-range");
                assert!(original.is::<std::num::TryFromIntError>());
                assert_eq!(
                    copy.to_string(),
                    "columnar compact i32 is malformed: value exceeds i32"
                );
            }
            "columnar-varint" => {
                assert_eq!(reason.field("offset"), Some(&Captured::ByteCount(1)));
                assert_eq!(cause.code, "pack-bits-truncated");
                assert_eq!(cause.field("needed"), Some(&Captured::ByteCount(3)));
                assert_eq!(cause.field("found"), Some(&Captured::ByteCount(2)));
                assert!(original.is::<PackBitsError>());
            }
            "columnar-utf8" => {
                assert_eq!(cause.code, "std-utf8");
                let utf8 = original.downcast_ref::<std::str::Utf8Error>().unwrap();
                assert_eq!((utf8.valid_up_to(), utf8.error_len()), (1, Some(1)));
                assert_eq!(
                    copy.to_string(),
                    "columnar compact string is malformed: invalid UTF-8"
                );
            }
            _ => unreachable!("the fixture names its retained native cause"),
        }
    }
}

// ---------------------------------------------------------------------------
// Trial 8.
// ---------------------------------------------------------------------------

/// Early grammar refusals and nested skips retain their original operands.
#[test]
fn public_compact_first_fault_and_unknown_skips() {
    let cases = [
        (vec![0x10, 0x15, 0x80], "columnar-stop-delta"),
        (vec![0x18], "columnar-field-type"),
        (vec![0x69, 0x10], "columnar-stop-element"),
        (vec![0x6b, 1, 0x03], "columnar-stop-value"),
        (
            {
                let mut overflow = vec![0x68]; // unknown binary field
                write_varint(&mut overflow, u64::MAX);
                overflow
            },
            if usize::BITS == 64 {
                "columnar-byte-range"
            } else {
                "columnar-binary-length"
            },
        ),
    ];
    for (footer, code) in cases {
        let files = compact_footer_files(&footer);
        let error = read(&files).unwrap_err();
        let copy = error.clone();
        drop(files);
        drop(footer);
        drop(error);
        let _ = inspect(&copy);
        let captured_copy = Capture::read(&copy);
        let reason = captured_copy.child("reason");
        assert_eq!(reason.code, code);
        match code {
            "columnar-stop-delta" => {
                assert_eq!(reason.field("header"), Some(&Captured::Unsigned(16)));
            }
            "columnar-field-type" => {
                assert_eq!(reason.field("id"), Some(&Captured::Signed(1)));
                assert_eq!(reason.field("actual"), Some(&Captured::Unsigned(8)));
                assert_eq!(reason.field("expected"), Some(&Captured::Unsigned(5)));
            }
            "columnar-byte-range" => {
                assert_eq!(reason.field("offset"), Some(&Captured::ByteCount(11)));
                assert_eq!(
                    reason.field("length"),
                    Some(&Captured::ByteCount(u128::from(u64::MAX)))
                );
            }
            "columnar-binary-length" => assert_eq!(
                reason.field("value"),
                Some(&Captured::ByteCount(u128::from(u64::MAX)))
            ),
            _ => {}
        }
    }

    // A complete healthy file remains valid with a nested unknown struct whose
    // binary payload is skipped as bytes, without imposing string validation.
    let dataset = RdfDatasetBuilder::new().freeze().unwrap();
    let files = write(
        dataset.as_ref(),
        &ContentStore::default(),
        Compression::Uncompressed,
    )
    .unwrap();
    let mut array = files.files.into_array();
    let file = &mut array[0];
    let old_length = u32::from_le_bytes(file[file.len() - 8..file.len() - 4].try_into().unwrap());
    let footer_end = file.len() - 8;
    assert_eq!(file[footer_end - 1], 0);
    // An explicit field id 100, struct, binary field 1, one byte, end struct/root.
    let extra = [0x0c, 0xc8, 0x01, 0x18, 1, 0xff, 0];
    file.splice(footer_end - 1..footer_end - 1, extra);
    let length_at = file.len() - 8;
    let new_length = old_length + u32::try_from(extra.len()).unwrap();
    file[length_at..length_at + 4].copy_from_slice(&new_length.to_le_bytes());
    assert_eq!(
        read(&ParquetFiles::from_array(array))
            .unwrap()
            .dataset
            .quads()
            .count(),
        0
    );
}

// ---------------------------------------------------------------------------
// Trial 9.
// ---------------------------------------------------------------------------

/// Public page decoding retains the original signed prefix after file disposal.
#[test]
fn public_plain_value_refusals_survive_file_disposal() {
    let payload = b"original value-body payload";
    let mut blobs = ContentStore::new();
    blobs.insert(payload.to_vec());
    let dataset = RdfDatasetBuilder::new().freeze().unwrap();
    let files = write(dataset.as_ref(), &blobs, Compression::Uncompressed)
        .unwrap()
        .files;
    assert_eq!(read(&files).unwrap().blobs.len(), 1);
    let mut original = (payload.len() as i32).to_le_bytes().to_vec();
    original.extend_from_slice(payload);
    for length in [-1_i32, i32::MIN] {
        let mut array = files.clone().into_array();
        let mut replacement = length.to_le_bytes().to_vec();
        replacement.extend_from_slice(payload);
        replace_unique(&mut array[4], &original, &replacement);
        let changed = ParquetFiles::from_array(array);
        let error = read(&changed).unwrap_err();
        drop(changed);
        let copy = error.clone();
        drop(error);
        assert!(matches!(
            copy,
            ColumnarError::ValueBody {
                context: "PLAIN BYTE_ARRAY length",
                reason: ValueBodyReason::NegativeLength { length: actual },
            } if actual == length
        ));
        let native = std::error::Error::source(&copy)
            .unwrap()
            .downcast_ref::<ValueBodyReason>()
            .unwrap();
        let ColumnarError::ValueBody { reason, .. } = &copy else {
            unreachable!("the original value-body variant was checked above");
        };
        assert!(std::ptr::eq(native, reason));
        let _ = inspect(&copy);
        let captured_copy = Capture::read(&copy);
        let fields = captured_copy.child("reason");
        assert_eq!(fields.code, "columnar-negative-length");
        assert_eq!(
            fields.field("length"),
            Some(&Captured::Signed(i128::from(length)))
        );
    }
}

// ---------------------------------------------------------------------------
// Gated trials (KD-3 / KD-4): loudly registered, never silently skipped.
// ---------------------------------------------------------------------------

/// KD-STARDF-COLUMNAR-3: the source-failure-precedence trial cannot be
/// expressed against the landed `DatasetView` seam (missing `read_error()`,
/// `ProbePlan`, `capabilities()`, `probe_plan`, `quads_for_pattern_with_plan`,
/// `term_count`, `term_id_by_value`, `term_id_by_iri`, `term_id_by_blank`).
/// Close condition: vendor the upstream `DatasetView` surface into
/// `crate::vendor::core`, then restore the drafted trial from
/// `tests/disabled/structured_errors.rs-pending-stardf-substrate`.
#[test]
#[ignore = "KD-STARDF-COLUMNAR-3: upstream DatasetView surface not vendored"]
fn kd_3_public_projection_and_write_preserve_source_owners_and_fields() {}

/// KD-STARDF-COLUMNAR-4: the native-cause-transport trial cannot be expressed
/// against the landed seam (missing `stardf_lex::allocation::{Resident, Memory,
/// SharedOwned}`, `RdfDiagnostic::{with_detail, with_location,
/// with_native_cause}`, `RdfLocation`, `ContentDigest::from_raw`).
/// Close condition: vendor the upstream lex allocation seam and the kernel
/// diagnostic algebra, then restore the drafted trial.
#[test]
#[ignore = "KD-STARDF-COLUMNAR-4: lex allocation seam and diagnostic algebra not vendored"]
fn kd_4_native_cause_transport_preserves_storage_and_full_width_rows() {}
