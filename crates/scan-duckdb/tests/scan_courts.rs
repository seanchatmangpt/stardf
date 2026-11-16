// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! LANE 03-scan-courts (2026-10-31) — the six scan courts of
//! `stardf/arrow/docs/SCAN-COMPLETION.md` §5, landed as real-collaborator
//! integration tests: a real in-process DuckDB engine (frozen-duckdb), a real
//! five-table Parquet fixture minted through `stardf_columnar::write` (the
//! receipted splitmix64 state=0x6012 fixture shape, ported verbatim from
//! `crates/stardf-columnar/tests/golden_bytes.rs`), real bytes on disk, and
//! real SQL through the registered `stardf_scan()` table function.
//!
//! Courts:
//!
//! - C1 `count_parity_across_all_five_tables` — `SELECT count(*)` through
//!   `stardf_scan()` == `read_parquet` on the same files == the stream-side
//!   fixture count, per table. The 64-row-slab arithmetic (`ceil(n/64)`)
//!   rides the pinned-count leg: slab boundaries are engine-internal chunk
//!   boundaries, not SQL-observable, so the court pins the exact stream-side
//!   row count as the arithmetic leg (any scan that merges/splits engine
//!   chunks cannot move a pinned count).
//! - C2 `value_parity_pinned_by_order_by` — `SELECT *` row-set equality
//!   through `stardf_scan()` vs `read_parquet`, rows fetched ORDER BY the
//!   full column tuple, per column, per row. Pins the INT64→BIGINT and
//!   BYTE_ARRAY→VARCHAR type map at the value level.
//! - C3 `pushdown_flip_loud_skip_until_enabled` — the s/p/o filter pushdown
//!   flip. While the VTab leaves `supports_pushdown()` = false (the W2e-last
//!   posture), the court loud-SKIPs with the reason. It re-derives the
//!   pushdown state from the crate surface at run time, so landing W2e flips
//!   this court from SKIP to the real filtered-row-set + mutant-catch court
//!   without a test edit.
//! - C4 `empty_input_yields_zero_rows_clean` — zero-quad fixture → zero rows
//!   through every path, clean exit, no error across the FFI boundary.
//! - C5 `typed_bind_error_not_panic` — a bind over a missing/invalid dataset
//!   root surfaces a typed error through `BindInfo::set_error` (surfaced as a
//!   DuckDB query error), never a panic across the C ABI. A panic in bind
//!   aborts the test process, so the court passing is itself the no-panic
//!   witness; the message is asserted to carry the typed reason text.
//! - C6 `drop_one_row_mutant_is_caught` — the anti-vacuity positive control:
//!   the `--mutate drop-one-row` knob removes one real fixture row from the
//!   materialized dataset root; the court MUST catch it (the pinned-count
//!   parity leg breaks). A court that passed under the mutation would be a
//!   vacuous gate; the mutant world is real Parquet bytes, not a stub.
//!
//! Leases: this file owns `crates/stardf-scan-duckdb/tests/` and needs NO
//! manifest edits — the fixture collaborators (`stardf_columnar`,
//! `frozen_duckdb`) are regular dependencies of the crate, so the tests
//! consume exactly the dependency set the crate itself compiles against.
//! Scratch roots come from a local drop guard (no dev-dependency needed).
//!
//! UNBLOCK DISCLOSURE (lane 02-scan-scaffold, 2026-10-31, minimal fix to this
//! lane's in-flight file under the compile-freeze SLA): the workspace ladder
//! runs `clippy --all-targets -- -D warnings`, promoting the root
//! [workspace.lints] pedantic `doc_markdown` to an error; this court file's
//! spec prose (DuckDB, Parquet, BYTE_ARRAY→VARCHAR, ...) trips it. Module-level
//! allow with the same justification as the crate root's opt-out.
#![allow(clippy::doc_markdown)]
// Court-file lint posture: the workspace pedantic/nursery set is promoted by
// the CI ladder, and these are the court-specific, reasoned opt-outs (same
// posture as the crate lib's own opt-outs).
#![allow(
    // The courts are self-contained falsifier ladders; splitting each leg
    // into helpers obscures the court text.
    clippy::too_many_lines,
    // splitmix64 `mix()` mutates through &mut by contract (testkit parity).
    clippy::missing_const_for_fn,
    // Fixture indices are drawn modulo small constants by construction.
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    // The ordered-fetch row types mirror the SQL column shapes verbatim.
    clippy::type_complexity
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;

use stardf_columnar::vendor::core::{
    BlankScope, ContentStore, RdfDataset, RdfDatasetBuilder, RdfTextDirection, TermValue,
};
use stardf_columnar::{Compression, Table, write};

// ---------------------------------------------------------------------------
// Scratch root guard: real files under the OS temp dir, removed on drop.
// ---------------------------------------------------------------------------

/// A per-court scratch directory, removed on drop.
struct ScratchDir(PathBuf);

impl ScratchDir {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("stardf-scan-courts-{}-{}", process::id(), label));
        let _ = fs::remove_dir_all(&path);
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// ---------------------------------------------------------------------------
// Fixture: the splitmix64 state=0x6012 five-table dataset, ported verbatim
// from crates/stardf-columnar/tests/golden_bytes.rs (same seed, same builder
// sequence, same 400-row loop) so the scan courts and the byte-determinism
// courts witness the same dataset by construction.
// ---------------------------------------------------------------------------

/// Inline splitmix64 (purrdf-testkit `rng::splitmix64_next` parity; reference
/// purrdf `crates/testkit/src/rng.rs`).
fn mix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The `--mutate` knob: named mutations of the fixture subject (C6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mutant {
    /// The unmutated fixture.
    None,
    /// Drop the LAST pushed quad from the stream before the write.
    DropOneRow,
}

/// Build the fixture dataset + blob store.
///
/// `mutant` applies the C6 knob to the real subject: `DropOneRow` skips the
/// final quad push, so the materialized quads table genuinely has one fewer
/// row than the pinned expectation.
fn fixture(mutant: Mutant) -> (Arc<RdfDataset>, ContentStore) {
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
        // C6 mutant knob: the final iteration's quad push is dropped, so the
        // materialized dataset genuinely carries one fewer quad row while the
        // pinned expectation (counted from the unmutated world) stays fixed.
        if mutant == Mutant::DropOneRow && row == 399 {
            // Consume the identical splitmix draws the unmutated world makes
            // so the mutant world stays otherwise identical? No: the mutant
            // stops HERE by construction — draws after this point are not
            // made, and the remaining fixture (reifiers/annotations/blobs)
            // is not minted either. The mutant world is the truncated
            // stream; the pinned expectation is from the full world.
            break;
        }
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
    let dataset = builder.freeze().expect("the fixture is valid RDF");
    (dataset, blobs)
}

/// Stream-side row count per table, from the dataset accessors themselves
/// (the C1 arithmetic leg).
fn stream_counts(dataset: &RdfDataset, blobs: &ContentStore) -> [usize; 5] {
    [
        dataset.terms().len(),
        dataset.quads().count(),
        dataset.reifier_quads().count(),
        dataset.annotation_quads().count(),
        blobs.len(),
    ]
}

/// Materialize the fixture as five real Parquet files under a fresh scratch
/// root (the shape `stardf_scan('<root>')` binds).
fn materialize(root: &Path, mutant: Mutant) -> [usize; 5] {
    let (dataset, blobs) = fixture(mutant);
    let written = write(dataset.as_ref(), &blobs, Compression::Uncompressed)
        .unwrap_or_else(|e| panic!("the fixture encodes: {e}"));
    let counts = stream_counts(&dataset, &blobs);
    fs::create_dir_all(root).expect("the scratch root creates");
    for (table, bytes) in written.files.iter() {
        fs::write(root.join(table.file_name()), bytes).expect("the table file writes");
    }
    counts
}

// ---------------------------------------------------------------------------
// The scan surface: real in-process engine + registered stardf_scan().
// ---------------------------------------------------------------------------

/// The process-shared engine: one in-memory DuckDB instance with
/// `stardf_scan` registered, held for the whole process behind a mutex.
///
/// Sharing is a court-stability decision, not a correctness claim: the scan's
/// mixed-type (VARCHAR + BIGINT) multi-column read path SIGBUSes the frozen
/// prebuilt engine on this surface (witnessed 2026-10-31, see C2's header
/// for the repro shapes); a fresh engine per court also SIGBUSed under
/// `--test-threads=1`. One serialized instance keeps the five stable courts
/// deterministic while the C2 defect is owned upstream (lane 02 / SDK).
fn engine() -> std::sync::MutexGuard<'static, frozen_duckdb::Connection> {
    use std::sync::{Mutex, OnceLock};
    static ENGINE: OnceLock<Mutex<frozen_duckdb::Connection>> = OnceLock::new();
    let mutex = ENGINE.get_or_init(|| {
        let conn = frozen_duckdb::Connection::open_in_memory().expect("the engine opens");
        stardf_scan_duckdb::register_stardf_scan(&conn).expect("stardf_scan registers");
        Mutex::new(conn)
    });
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Query one scalar (first row, first column) as i64.
fn scalar_i64(conn: &frozen_duckdb::Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |row| row.get::<_, i64>(0))
        .unwrap_or_else(|e| panic!("scalar query fails on {sql:?}: {e}"))
}

/// Fetch an ordered (s, p, o, g) triple+graph table as raw tuples.
fn fetch_quads_ordered(
    conn: &frozen_duckdb::Connection, sql: &str,
) -> Vec<(i64, i64, i64, Option<i64>)> {
    let mut stmt = conn
        .prepare(sql)
        .unwrap_or_else(|e| panic!("prepare fails on {sql:?}: {e}"));
    let mut rows = stmt
        .query([])
        .unwrap_or_else(|e| panic!("query fails on {sql:?}: {e}"));
    let mut out = Vec::new();
    while let Some(row) = rows
        .next()
        .unwrap_or_else(|e| panic!("row fetch fails: {e}"))
    {
        out.push((
            row.get::<_, i64>(0).expect("s is BIGINT"),
            row.get::<_, i64>(1).expect("p is BIGINT"),
            row.get::<_, i64>(2).expect("o is BIGINT"),
            row.get::<_, Option<i64>>(3).expect("g is nullable BIGINT"),
        ));
    }
    out
}

/// The parity predicate C1/C6 evaluate: per table,
/// `scan == parquet == pinned` with the pinned leg named.
fn parity(
    conn: &frozen_duckdb::Connection, root: &Path, pinned: &[usize; 5],
) -> Result<[i64; 5], String> {
    let mut scan = [0i64; 5];
    for (index, table) in Table::ALL.into_iter().enumerate() {
        let file = root.join(table.file_name());
        let file = file.to_str().expect("the scratch path is UTF-8");
        let dir = root.to_str().expect("the scratch root is UTF-8");
        let through_scan = scalar_i64(
            conn,
            &format!(
                "SELECT count(*) FROM stardf_scan('{}', \"table\" = '{}')",
                esc(dir),
                table.name()
            ),
        );
        let through_parquet = scalar_i64(
            conn,
            &format!("SELECT count(*) FROM read_parquet('{}')", esc(file)),
        );
        if through_scan != through_parquet {
            return Err(format!(
                "table {} scan({through_scan}) != parquet({through_parquet})",
                table.name()
            ));
        }
        if through_scan as usize != pinned[index] {
            return Err(format!(
                "table {} scan({through_scan}) != pinned stream-side {}",
                table.name(),
                pinned[index]
            ));
        }
        scan[index] = through_scan;
    }
    Ok(scan)
}

/// Single-quote SQL string escape (the fixture paths are scratch temp paths).
fn esc(path: &str) -> String {
    path.replace('\'', "''")
}

// ---------------------------------------------------------------------------
// C1 — count parity.
// ---------------------------------------------------------------------------

#[test]
fn c1_count_parity_across_all_five_tables() {
    let tmp = ScratchDir::new("c1");
    let root = tmp.path().join("root");
    let pinned = materialize(&root, Mutant::None);
    // The pinned expectation is itself non-trivial: the fixture has rows in
    // every table except annotations can be zero — assert the pinned quads
    // leg is the receipted 400-row loop, not an accidental zero.
    assert_eq!(
        pinned[1], 400,
        "the fixture's stream-side quad count is the 400-row loop"
    );
    let conn = &*engine();
    let scan = parity(conn, &root, &pinned).expect("count parity holds");
    // The ceil(n/64) slab-arithmetic witness (SCAN-COMPLETION §5 C1): the
    // 400-row quads stream spans ceil(400/64) = 7 sixty-four-row slabs
    // stream-side. The engine chunks at its own vector capacity (the crate's
    // SLAB_ROWS cap), so the SQL-observable leg is the pinned exact count
    // above; this leg witnesses the stream-side arithmetic the pinned count
    // replaces.
    assert_eq!(
        pinned[1].div_ceil(64),
        7,
        "the quad slab arithmetic ceil(n/64) holds stream-side"
    );
    let _ = scan;
}

// ---------------------------------------------------------------------------
// C2 — value parity, ORDER BY pinned.
// ---------------------------------------------------------------------------

#[test]
fn c2_value_parity_pinned_by_order_by() {
    let tmp = ScratchDir::new("c2");
    let root = tmp.path().join("root");
    let pinned = materialize(&root, Mutant::None);
    assert!(pinned[1] > 0, "the fixture has quads to compare");
    let conn = &*engine();
    let root_esc = esc(root.to_str().expect("UTF-8 root"));

    // Quads: full ordered row equality across every column.
    let through_scan = fetch_quads_ordered(
        conn,
        &format!(
            "SELECT s, p, o, g FROM stardf_scan('{root_esc}', \"table\" = 'quads') \
             ORDER BY s, p, o, g NULLS LAST"
        ),
    );
    let through_parquet = fetch_quads_ordered(
        conn,
        &format!(
            "SELECT s, p, o, g FROM read_parquet('{root_esc}/quads.parquet') \
             ORDER BY s, p, o, g NULLS LAST"
        ),
    );
    assert_eq!(
        through_scan.len(),
        through_parquet.len(),
        "quad row counts agree before the ordered compare"
    );
    assert_eq!(
        through_scan, through_parquet,
        "the ordered quad rows are value-identical through both paths"
    );

    // Terms: the closed type map pinned at the VALUE level — INT64→BIGINT
    // (id, datatype, kind), BYTE_ARRAY+UTF8→VARCHAR (lex, lang).
    //
    // Closed defect witness (2026-10-31, this lane): during lane 02's
    // landing window (lib.rs 13.6KB@20:23 -> 39.4KB@20:52) the combined
    // multi-column terms fetch was UNSAFE — mixed-type (VARCHAR+BIGINT)
    // multi-column results SIGBUSed the engine (sorted) or Utf8Errored
    // (streamed), and datatype read back NULL for every literal row. A
    // minimal 2-column VTab over the same bytes was clean (5/5), pinning the
    // defect to the scan crate, not the SDK; the parquet oracle over the
    // same files was clean throughout. The re-fetch of this shape after
    // lane 02's landing is clean (1030 rows, repeated), so the combined leg
    // below is LANDED, not skipped — the witness is retained as the court's
    // provenance.
    let fetch_terms_ordered_both = |source: &str| -> Vec<(String, String, String, String)> {
        let sql = match source {
            "scan" => format!(
                "SELECT id, lex, datatype, lang \
                 FROM stardf_scan('{root_esc}', \"table\" = 'terms') \
                 ORDER BY id, lex NULLS LAST, datatype NULLS LAST, lang NULLS LAST"
            ),
            _ => format!(
                "SELECT id, lex, datatype, lang \
                 FROM read_parquet('{root_esc}/terms.parquet') \
                 ORDER BY id, lex NULLS LAST, datatype NULLS LAST, lang NULLS LAST"
            ),
        };
        let mut stmt = conn.prepare(&sql).expect("the terms prepare");
        let mut rows = stmt.query([]).expect("the terms query");
        let mut out = Vec::new();
        while let Some(row) = rows.next().expect("row fetch") {
            let id: Option<i64> = row.get(0).expect("the id cell");
            let lex: Option<String> = row.get(1).expect("the lex cell");
            let datatype: Option<i64> = row.get(2).expect("the datatype cell");
            let lang: Option<String> = row.get(3).expect("the lang cell");
            out.push((
                id.map_or_else(|| "\u{1}NULL".to_owned(), |v| v.to_string()),
                lex.unwrap_or_else(|| "\u{1}NULL".to_owned()),
                datatype.map_or_else(|| "\u{1}NULL".to_owned(), |v| v.to_string()),
                lang.unwrap_or_else(|| "\u{1}NULL".to_owned()),
            ));
        }
        out
    };
    // Combined-tuple parity (the previously-BLOCKED leg, now clean): every
    // row value-identical across all four pinned columns.
    let terms_scan = fetch_terms_ordered_both("scan");
    let terms_parquet = fetch_terms_ordered_both("parquet");
    assert_eq!(
        terms_scan.len(),
        terms_parquet.len(),
        "term row counts agree before the ordered compare"
    );
    assert_eq!(
        terms_scan, terms_parquet,
        "the ordered term rows are value-identical through both paths"
    );

    // Per-column leg table (failure localization): each column pinned
    // row-for-row in id order against the parquet oracle.
    let fetch_terms_column = |source: &str, column: &str, kind: char| -> Vec<String> {
        let sql = match source {
            "scan" => format!(
                "SELECT {column} FROM stardf_scan('{root_esc}', \"table\" = 'terms') ORDER BY id"
            ),
            _ => {
                format!("SELECT {column} FROM read_parquet('{root_esc}/terms.parquet') ORDER BY id")
            }
        };
        let mut stmt = conn.prepare(&sql).expect("the terms prepare");
        let mut rows = stmt.query([]).expect("the terms query");
        let mut out = Vec::new();
        while let Some(row) = rows.next().expect("row fetch") {
            if kind == 'i' {
                let id: Option<i64> = row.get(0).expect("the BIGINT cell");
                out.push(id.map_or_else(|| "\u{1}NULL".to_owned(), |value| value.to_string()));
            } else {
                let cell: Option<String> = row.get(0).expect("the VARCHAR cell");
                out.push(cell.unwrap_or_else(|| "\u{1}NULL".to_owned()));
            }
        }
        out
    };
    let mut leg_results: Vec<(&str, bool)> = Vec::new();
    for (column, kind) in [
        ("id", 'i'),
        ("kind", 'i'),
        ("lex", 's'),
        ("datatype", 'i'),
        ("lang", 's'),
    ] {
        let pass =
            fetch_terms_column("scan", column, kind) == fetch_terms_column("parquet", column, kind);
        leg_results.push((column, pass));
        if !pass {
            eprintln!("[C2 FAIL DETAIL] terms.{column} diverges from the parquet oracle");
        }
    }
    let failed: Vec<&str> = leg_results
        .iter()
        .filter(|(_, pass)| !pass)
        .map(|(column, _)| *column)
        .collect();
    assert!(
        failed.is_empty(),
        "terms per-column parity failed for columns {failed:?} \
         (the scan mis-reports at least one terms column)"
    );

    // The closed type map, pinned at the SQL level: INT64 → BIGINT,
    // BYTE_ARRAY+UTF8 → VARCHAR, BYTE_ARRAY → BLOB (blobs.bytes).
    let quad_types: String = conn
        .query_row(
            &format!(
                "SELECT typeof(s) || ',' || typeof(g) \
                 FROM stardf_scan('{root_esc}', \"table\" = 'quads') LIMIT 1"
            ),
            [],
            |row| row.get::<_, String>(0),
        )
        .expect("the quads type pin queries");
    assert_eq!(
        quad_types, "BIGINT,BIGINT",
        "the id columns surface as BIGINT through the scan"
    );
    let blob_types: String = conn
        .query_row(
            &format!(
                "SELECT typeof(digest) || ',' || typeof(bytes) \
                 FROM stardf_scan('{root_esc}', \"table\" = 'blobs') LIMIT 1"
            ),
            [],
            |row| row.get::<_, String>(0),
        )
        .expect("the blobs type pin queries");
    assert_eq!(
        blob_types, "VARCHAR,BLOB",
        "UTF8 bytes surface as VARCHAR and raw bytes as BLOB through the scan"
    );
}

// ---------------------------------------------------------------------------
// C3 — pushdown flip: loud SKIP while supports_pushdown = false.
// ---------------------------------------------------------------------------

#[test]
fn c3_pushdown_flip_loud_skip_until_enabled() {
    // Re-derive the pushdown state from the crate surface at run time: when
    // lane 02 lands W2e (supports_pushdown() -> true), this court flips from
    // loud SKIP to the real filtered-row-set + mutant-catch court with no
    // test edit. The trait method is the crate's own declared state, read
    // through the VTab trait, not a hardcoded test-side constant.
    if !<stardf_scan_duckdb::StardfScanVTab as frozen_duckdb::duckdb::vtab::VTab>::supports_pushdown(
    ) {
        eprintln!(
            "[SKIP] c3 pushdown flip: supports_pushdown() = false (the landed W2e \
             divergence: the frozen-duckdb 2.0.4 SDK exposes no complex-filter \
             pushdown surface, lib.rs F05 note) — the filtered-row-set + \
             mutant-catch legs run only when this flips true"
        );
        return;
    }
    // --- pushdown-enabled legs (unreached while the SDK refuses pushdown) ---
    let tmp = ScratchDir::new("c3");
    let root = tmp.path().join("root");
    let pinned = materialize(&root, Mutant::None);
    let conn = &*engine();
    let root_esc = esc(root.to_str().expect("UTF-8 root"));
    // Pick a real subject id from the fixture (the first quad's s).
    let first_s = scalar_i64(
        conn,
        &format!("SELECT s FROM stardf_scan('{root_esc}', \"table\" = 'quads') ORDER BY s LIMIT 1"),
    );
    let filtered = scalar_i64(
        conn,
        &format!(
            "SELECT count(*) FROM stardf_scan('{root_esc}', \"table\" = 'quads') WHERE s = {first_s}"
        ),
    );
    let unfiltered = scalar_i64(
        conn,
        &format!("SELECT count(*) FROM stardf_scan('{root_esc}', \"table\" = 'quads')"),
    );
    assert!(
        filtered < unfiltered,
        "the s-equality filter actually reduces the row set (filtered={filtered}, \
         unfiltered={unfiltered})"
    );
    // Mutant-catch leg: a scan that ignores the pushed filter would return
    // the unfiltered count; the court's filtered<unfiltered assertion above
    // is the catch. Positive control: the parquet oracle agrees.
    let parquet_filtered = scalar_i64(
        conn,
        &format!(
            "SELECT count(*) FROM read_parquet('{root_esc}/quads.parquet') WHERE s = {first_s}"
        ),
    );
    assert_eq!(
        filtered, parquet_filtered,
        "the pushed filter agrees with the parquet oracle"
    );
    let _ = pinned;
}

// ---------------------------------------------------------------------------
// C4 — empty input.
// ---------------------------------------------------------------------------

#[test]
fn c4_empty_input_yields_zero_rows_clean() {
    let tmp = ScratchDir::new("c4");
    let root = tmp.path().join("root");
    // The empty world: the builder freezes with nothing interned and nothing
    // pushed, so all five tables are genuinely zero-row. The scan binds the
    // empty dataset cleanly and every table reads back zero rows with no
    // error across the FFI boundary.
    let builder = RdfDatasetBuilder::new();
    let blobs = ContentStore::new();
    let dataset = builder.freeze().expect("the empty fixture is valid RDF");
    let written = write(dataset.as_ref(), &blobs, Compression::Uncompressed)
        .expect("the empty fixture encodes");
    fs::create_dir_all(&root).expect("the scratch root creates");
    for (table, bytes) in written.files.iter() {
        fs::write(root.join(table.file_name()), bytes).expect("the table file writes");
    }
    let conn = &*engine();
    let root_esc = esc(root.to_str().expect("UTF-8 root"));
    for table in Table::ALL {
        let n = scalar_i64(
            conn,
            &format!(
                "SELECT count(*) FROM stardf_scan('{root_esc}', \"table\" = '{}')",
                table.name()
            ),
        );
        assert_eq!(n, 0, "table {} is empty through the scan", table.name());
    }
    // The typed parameter surface stays lawful on the empty world: the
    // unknown-table refusal enumerates the closed set (never a silent
    // invented surface), exercising the same parameter law on an all-empty
    // dataset.
    let refused = conn.query_row(
        &format!("SELECT count(*) FROM stardf_scan('{root_esc}', \"table\" = 'nope')"),
        [],
        |row| row.get::<_, i64>(0),
    );
    let err = refused.expect_err("an unknown table is a typed refusal");
    let text = err.to_string();
    assert!(
        text.contains("unknown table"),
        "the empty-world refusal is the typed unknown-table reason, got: {text}"
    );
}

// ---------------------------------------------------------------------------
// C5 — typed bind error, never a panic across the C ABI.
// ---------------------------------------------------------------------------

#[test]
fn c5_typed_bind_error_not_panic() {
    let tmp = ScratchDir::new("c5");
    let conn = &*engine();

    // Shape 1 — missing root dir: the DatasetIo refusal.
    let missing = tmp.path().join("does-not-exist");
    let missing_esc = esc(missing.to_str().expect("UTF-8 root"));
    let result = conn.query_row(
        &format!("SELECT count(*) FROM stardf_scan('{missing_esc}', \"table\" = 'terms')"),
        [],
        |row| row.get::<_, i64>(0),
    );
    let err = result.expect_err("a missing dataset root must surface an error, not rows");
    let text = err.to_string();
    eprintln!("[C5] dataset-io error text: {text}");
    assert!(
        text.contains("stardf_scan") && text.contains("reading terms.parquet failed"),
        "the bind error is typed (stardf_scan prefix + DatasetIo reason), got: {text}"
    );

    // Shape 2 — a root that exists but holds a corrupt profile: the reader
    // validation refusal (Profile reason), still no panic.
    let corrupt = tmp.path().join("corrupt");
    fs::create_dir_all(&corrupt).expect("the corrupt root creates");
    let fixture_root = tmp.path().join("fixture");
    materialize(&fixture_root, Mutant::None);
    for table in Table::ALL {
        let mut bytes =
            fs::read(fixture_root.join(table.file_name())).expect("the fixture table file reads");
        // Corrupt ONE byte in the middle of each file so reader validation
        // (physical or semantic) refuses the profile fail-closed.
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xff;
        fs::write(corrupt.join(table.file_name()), &bytes).expect("the corrupt table file writes");
    }
    let corrupt_esc = esc(corrupt.to_str().expect("UTF-8 root"));
    let result = conn.query_row(
        &format!("SELECT count(*) FROM stardf_scan('{corrupt_esc}', \"table\" = 'terms')"),
        [],
        |row| row.get::<_, i64>(0),
    );
    let err = result.expect_err("a corrupt profile must surface an error, not rows");
    let text = err.to_string();
    eprintln!("[C5] profile error text: {text}");
    assert!(
        text.contains("stardf_scan") && text.contains("profile validation refused"),
        "the corrupt-profile bind error is the typed Profile reason, got: {text}"
    );

    // Shape 3 — the required named parameter `table` is absent: typed
    // TableRequired refusal enumerating the closed five-table set.
    let fixture_esc = esc(fixture_root.to_str().expect("UTF-8 root"));
    let result = conn.query_row(
        &format!("SELECT count(*) FROM stardf_scan('{fixture_esc}')"),
        [],
        |row| row.get::<_, i64>(0),
    );
    let err = result.expect_err("an absent `table` parameter is a typed refusal");
    let text = err.to_string();
    eprintln!("[C5] table-required error text: {text}");
    assert!(
        text.contains("the required named parameter `table` is absent"),
        "the refusal is the typed TableRequired reason, got: {text}"
    );
}

// ---------------------------------------------------------------------------
// C6 — the drop-one-row mutant must be caught.
// ---------------------------------------------------------------------------

#[test]
fn c6_drop_one_row_mutant_is_caught() {
    let tmp = ScratchDir::new("c6");
    // Full world: the pinned expectation.
    let full_root = tmp.path().join("full");
    let pinned = materialize(&full_root, Mutant::None);
    // Mutant world: the same fixture subject with ONE quad row dropped by the
    // knob — real Parquet bytes with quads = N-1.
    let mutant_root = tmp.path().join("mutant");
    let mutant_pinned = materialize(&mutant_root, Mutant::DropOneRow);
    assert_eq!(
        mutant_pinned[1],
        pinned[1] - 1,
        "the knob genuinely dropped exactly one quad row from the materialized world"
    );
    let conn = &*engine();
    // Sub-court: the scan observes the mutant world faithfully (scan ==
    // parquet within the mutant world) — the scan is not what failed.
    let scan_mutant = scalar_i64(
        conn,
        &format!(
            "SELECT count(*) FROM stardf_scan('{}', \"table\" = 'quads')",
            esc(mutant_root.to_str().expect("UTF-8 root"))
        ),
    );
    let parquet_mutant = scalar_i64(
        conn,
        &format!(
            "SELECT count(*) FROM read_parquet('{}')",
            esc(mutant_root
                .join("quads.parquet")
                .to_str()
                .expect("UTF-8 path"))
        ),
    );
    assert_eq!(
        scan_mutant, parquet_mutant,
        "the scan tracks the mutant world faithfully (scan == parquet on the mutant root)"
    );
    // THE FLIP: the court's pinned-count leg MUST break under the mutation.
    // A court that passed here would be a vacuous gate = FAIL.
    let verdict = parity(conn, &mutant_root, &pinned);
    assert!(
        verdict.is_err(),
        "VACUOUS GATE: count parity accepted the drop-one-row mutant"
    );
    let message = verdict.unwrap_err();
    assert!(
        message.contains("pinned"),
        "the flip is the pinned-count leg (the load-bearing leg), got: {message}"
    );
}
