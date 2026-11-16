// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! PHASE-2 LANE 10 — byte-determinism courts for the vendored stardf-columnar
//! codec (reference regime: `/Users/sac/purrdf/crates/columnar/tests/golden_bytes.rs`).
//!
//! Four courts:
//!
//! 1. `seeded_files_are_the_pinned_bytes` — ten digests (5 tables x 2
//!    compression modes) over a splitmix64-seeded dataset, pinned as literals
//!    in `PINNED` and mirrored in `scripts/columnar_manifest.sha256`
//!    (F4b-class external manifest, per stardf 04-falsifiers).
//! 2. `read_back_rewrites_the_same_bytes` — round-trip identity: write →
//!    read → re-write must reproduce the exact bytes.
//! 3. `writing_twice_is_byte_identical` — write-twice determinism from the
//!    same in-memory input.
//! 4. `flipping_a_byte_moves_the_digest` — positive control: the court's
//!    digest actually moves when a byte moves; a court that cannot flip is
//!    FAIL.
//!
//! # Regenerating the pinned digests
//!
//! If (and ONLY if) an intentional, reviewed format change alters the written
//! bytes, print the new digests with:
//!
//! ```text
//! cargo test -p stardf-columnar --test golden_bytes regenerate_pinned_digests -- --ignored --nocapture
//! ```
//!
//! then paste the ten literals into `PINNED`, update
//! `../../scripts/columnar_manifest.sha256`, and record the reason with the
//! format change. Regeneration is splitmix64-seeded (state=0x6012), so a
//! regeneration is F2a-safe by construction.
//!
//! # Initial-pin reason (recorded, W1 lane 12 verification, 2026-10-31)
//!
//! The ten pins above were regenerated for THIS crate's encoder, not carried
//! from upstream purrdf (`purrdf-columnar` PINNED, e.g. Terms/Uncompressed
//! `f55714a0…`). Reason: the vendored encoder's fixture and projection differ
//! by construction — the shim kernel's plain-literal marker is the empty
//! datatype IRI (upstream `RdfLiteral::simple` canonicalizes to
//! `xsd:string`), and the vendored builder orders the canonical dictionary by
//! the shim `TermValue` order — so upstream pins cannot transfer byte-for-byte.
//! Minted via the `--ignored` regenerate court, splitmix64 state=0x6012.

use std::fs;
use std::path::PathBuf;

use stardf_columnar::vendor::core::content::digest_bytes;
use stardf_columnar::vendor::core::{
    BlankScope, ContentStore, DatasetView, RdfDatasetBuilder, RdfTextDirection, TermValue,
};
use stardf_columnar::{read, write, Compression, Table};

/// Inline splitmix64 (purrdf-testkit `rng::splitmix64_next` parity; the
/// testkit is not vendored, so the 12-line algorithm is carried here —
/// reference: purrdf `crates/testkit/src/rng.rs`).
fn mix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Every term kind in a seeded mix, so each nullable `INT64` column of the
/// terms table (datatype, direction, scope, the triple components) holds runs
/// of nulls and values of many lengths across several 64-row words, and the
/// graph columns of the quads, reifiers and annotations are sometimes null.
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

/// Encode (panicking with the typed error's Display on failure), decode,
/// re-encode; assert the re-encode is byte-identical, then hand back the
/// first encoding's per-table digests.
fn encode<D: DatasetView>(
    dataset: &D, blobs: &ContentStore, compression: Compression,
) -> stardf_columnar::ColumnarWrite
where
    <D as DatasetView>::ReadError: std::fmt::Display,
{
    match write(dataset, blobs, compression) {
        Ok(written) => written,
        Err(error) => panic!("the fixture encodes: {error}"),
    }
}

fn hex_of(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

fn digests<D: DatasetView>(
    dataset: &D, blobs: &ContentStore, compression: Compression,
) -> Vec<(Table, String)>
where
    <D as DatasetView>::ReadError: std::fmt::Display,
{
    let written = encode(dataset, blobs, compression);
    let read_back = read(&written.files).expect("the fixture decodes");
    let rewritten = encode(read_back.dataset.as_ref(), &read_back.blobs, compression);
    assert_eq!(
        rewritten.files, written.files,
        "a read-back rewrites the same bytes"
    );
    written
        .files
        .iter()
        .map(|(table, bytes)| (table, hex_of(&digest_bytes(bytes))))
        .collect()
}

const PINNED: [(Compression, Table, &str); 10] = [
    (
        Compression::Uncompressed,
        Table::Terms,
        "6e59d6c66d9e5fae087f7004176b219a80091ff30c1a20512ec1ebef60e9f118",
    ),
    (
        Compression::Uncompressed,
        Table::Quads,
        "7c08d9ef2671fdbf52d81c80d6553c79892c0a052193905342b0ead4381a636d",
    ),
    (
        Compression::Uncompressed,
        Table::Reifiers,
        "bd7c9d82c76c5878d08e747220ecc947ea0383c8aa3682bf0bf8418055b1bbca",
    ),
    (
        Compression::Uncompressed,
        Table::Annotations,
        "c3f9c85ee714ea2f4a7ff89efea74bcc40c6ae1c557e05b0a58d7fc376ef23de",
    ),
    (
        Compression::Uncompressed,
        Table::Blobs,
        "d6346d625c68920b209cb494ec39f21ee30b01a4ea81ff177c363602a0e49a45",
    ),
    (
        Compression::Zstd,
        Table::Terms,
        "fc777d0fa94ac58c50e43b29f5773df46d5c7d8257b2d42f4b4fe66dd4006245",
    ),
    (
        Compression::Zstd,
        Table::Quads,
        "af45ab2092653f6776dd95dc42f223ec47cd6c582d34d6ab99f72d342e0d5996",
    ),
    (
        Compression::Zstd,
        Table::Reifiers,
        "d681b748b10c695d501fa350edb08253c4f7af939fffa9a304219c4deca5a8af",
    ),
    (
        Compression::Zstd,
        Table::Annotations,
        "a2dd1f708678f5b2d4ec39abbf3b407f682d86da99ffd51b56fa12ec47c947ff",
    ),
    (
        Compression::Zstd,
        Table::Blobs,
        "152bf4086732a3fde2710cf6bad7c47c7cbf59aacdda57436119f7c36f4e2ef7",
    ),
];

/// The ten frozen file digests across both compression modes equal `PINNED`
/// exactly, in (compression, table) order.
#[test]
fn seeded_files_are_the_pinned_bytes() {
    assert_eq!(PINNED.len(), 10, "the pinned table carries all ten cells");
    let mut observed = Vec::new();
    for compression in [Compression::Uncompressed, Compression::Zstd] {
        let (dataset, blobs) = fixture();
        for (table, digest) in digests(dataset.as_ref(), &blobs, compression) {
            observed.push((compression, table, digest));
        }
    }
    let pinned: Vec<_> = PINNED
        .iter()
        .map(|&(compression, table, digest)| (compression, table, digest.to_owned()))
        .collect();
    assert_eq!(observed, pinned);
}

/// Round-trip identity court, both compression modes (asserted inside
/// [`digests`]); this trial names it explicitly so a failure in `digests`
/// reports as the court it is.
#[test]
fn read_back_rewrites_the_same_bytes() {
    for compression in [Compression::Uncompressed, Compression::Zstd] {
        let (dataset, blobs) = fixture();
        digests(dataset.as_ref(), &blobs, compression);
    }
}

/// Write-twice determinism: two encodings of the same in-memory input are
/// byte-identical, both modes.
#[test]
fn writing_twice_is_byte_identical() {
    for compression in [Compression::Uncompressed, Compression::Zstd] {
        let (dataset, blobs) = fixture();
        let first = encode(dataset.as_ref(), &blobs, compression);
        let second = encode(dataset.as_ref(), &blobs, compression);
        assert_eq!(
            first.files, second.files,
            "two writes of one input agree byte for byte"
        );
    }
}

/// Positive control (anti-vacuity): flipping one byte of one file moves its
/// digest. A court that cannot flip is FAIL.
#[test]
fn flipping_a_byte_moves_the_digest() {
    for compression in [Compression::Uncompressed, Compression::Zstd] {
        let (dataset, blobs) = fixture();
        let written = encode(dataset.as_ref(), &blobs, compression);
        let mut flipped = false;
        for (table, bytes) in written.files.iter() {
            let mut corrupted = bytes.to_vec();
            let last = corrupted.len() - 1;
            corrupted[last] ^= 0x01;
            assert_ne!(
                hex_of(&digest_bytes(&corrupted)),
                hex_of(&digest_bytes(bytes)),
                "one flipped byte must move the {table:?} digest"
            );
            flipped = true;
            let _ = table;
        }
        assert!(flipped, "the corpus is non-empty");
    }
}

/// The external `.sha256` manifest mirrors `PINNED` exactly (F4b-class:
/// digests pinned in Rust source AND in `scripts/columnar_manifest.sha256`,
/// one drift check over both).
#[test]
fn manifest_mirrors_the_pinned_digests() {
    let path = manifest_path();
    let body = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("the manifest at {} must exist: {e}", path.display()));
    let listed: Vec<(String, String)> = body
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (hex, name) = line
                .split_once("  ")
                .unwrap_or_else(|| panic!("manifest line must be `<sha256>  <name>`: {line:?}"));
            (hex.to_owned(), name.trim().to_owned())
        })
        .collect();
    let pinned: Vec<(String, String)> = PINNED
        .iter()
        .map(|&(compression, table, digest)| {
            (
                digest.to_owned(),
                format!("Compression::{compression:?}/Table::{table:?}"),
            )
        })
        .collect();
    assert_eq!(listed, pinned, "the manifest mirrors PINNED in order");
}

/// NOT part of the default run: prints the ten observed file digests in
/// `PINNED`-table literal form so a reviewed format change can refresh it.
/// Ignore-gated so only `-- --ignored` selects it (contract: the regeneration
/// court exists but never fires in the default suite).
#[test]
#[ignore = "regeneration court: run explicitly with -- --ignored --nocapture"]
fn regenerate_pinned_digests() {
    for compression in [Compression::Uncompressed, Compression::Zstd] {
        let (dataset, blobs) = fixture();
        for (table, digest) in digests(dataset.as_ref(), &blobs, compression) {
            println!(
                "    (\n        Compression::{compression:?},\n        Table::{table:?},\n        \"{digest}\",\n    ),"
            );
        }
    }
}

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/columnar_manifest.sha256")
}
