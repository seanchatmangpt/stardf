//! Columnar (struct-of-arrays) encode/decode benches for Quad64 slabs.
//!
//! DIVERGENCE DISCLOSED: the contract pointed at lane 9's canonical columnar
//! codec; no columnar module exists in `stardf-arrow-core` (grep over src/:
//! zero "columnar" hits; docs/RECEIPTS.md LANE-9 row is the verify.py gate,
//! not a codec). Per the blocker-branches-the-graph law this lane ships a
//! bench-local codec HERE, in the file this lane owns, so the baseline rows,
//! manifest rows, and zero-alloc gates exist and are honestly labeled.
//! When lane 9's canonical codec lands in core, this module's codec body is
//! deleted and the benches re-point — the receipt rows keep their shape.
//!
//! LANE-7-FINAL RECONCILIATION (2026-10-31): lane 6 landed the canonical
//! codec as `crates/stardf-columnar` (PurRDF five-table Parquet bridge,
//! `stardf_columnar::{write, read}` over `DatasetView` + `ContentStore`).
//! Re-point verdict, measured on this session's tree:
//! 1. BUILD_BROKEN — `cargo check -p stardf-columnar` fails with 6 errors in
//!    its unconditional `vendor` modules (`vendor/alloc_probe.rs:82` realloc
//!    must return `*mut u8`; `vendor/testkit.rs:81,91,135` Sized/move-out;
//!    `vendor/core/term_fixture.rs:37,38` moved `rng`), so the capi
//!    integration gate cannot link either.
//! 2. Even built, the API cannot feed this bench: `write()` takes an RDF
//!    `DatasetView` + `ContentStore` (no Quad64-slab entry point) and
//!    allocates by construction (eager `Vec` projection in
//!    `stardf-columnar/src/writer.rs` `project_read`: `view.quads().collect()`,
//!    dictionary/table builds) — a semantic serialization codec, structurally
//!    incapable of the zero-alloc timed-loop gate this file enforces.
//!
//! Verdict: the bench-local SoA codec is NOT superseded — the two measure
//! different things (memory-layout transpose vs semantic RDF→Parquet
//! serialization). It stays as the honest columnar-transpose measurand; the
//! landed crate's own benches belong to the stardf-columnar crate's owners.
//!
//! Design (zero-alloc promise made testable by construction):
//! - `encode_into` / `decode_into` write into CALLER-PREALLOCATED buffers,
//!   promise: zero allocations inside the timed loop (hard gate, cgap CTQ-5
//!   pattern) — a codec that silently grows a Vec fails the gate, not the
//!   baseline.
//! - Round-trip correctness is exact-checked on the real 1M-quad slab
//!   (Chicago: real collaborators, assert final state) before any timing.
//!
//! Benches (min-of-5, shared harness):
//! 1. columnar encode — AoS Quad64 slab -> SoA columns (GiB/s of payload)
//! 2. columnar decode — SoA columns -> AoS Quad64 slab (GiB/s of payload)
//! 3. columnar sum s_id — columnar scan of one u32 column (GiB/s of that
//!    column) vs the AoS row path (cache-line economics made visible)

use std::hint::black_box;

use stardf_arrow_core::layout::Quad64;

use crate::{make_slab, rounds, GIB, SCAN_ROWS};

/// Struct-of-arrays projection of a Quad64 slab.
///
/// Byte width per quad: 4*4 + 8 + 2*4 + 32 = 64 — identical to the AoS
/// Quad64 layout, so GiB/s numbers are apples-to-apples with the AoS benches.
#[derive(Default)]
pub struct QuadColumns {
    /// Subject column.
    pub s: Vec<u32>,
    /// Predicate column.
    pub p: Vec<u32>,
    /// Object column.
    pub o: Vec<u32>,
    /// Graph column.
    pub g: Vec<u32>,
    /// Timestamp column.
    pub t_ns: Vec<u64>,
    /// Causal-register column.
    pub vector_clock: Vec<u32>,
    /// Policy-mask column.
    pub policy_mask: Vec<u32>,
    /// Flat digest column (32 bytes per quad, quad-major).
    pub inline_digest: Vec<u8>,
}

impl QuadColumns {
    /// Allocate zeroed columns for `n` quads.
    pub fn with_capacity(n: usize) -> Self {
        Self {
            s: vec![0; n],
            p: vec![0; n],
            o: vec![0; n],
            g: vec![0; n],
            t_ns: vec![0; n],
            vector_clock: vec![0; n],
            policy_mask: vec![0; n],
            inline_digest: vec![0; n * 32],
        }
    }

    /// Number of quads the columns hold.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.s.len()
    }
}

/// Encode AoS slab -> SoA columns, in place, no allocation.
///
/// # Panics
/// Panics if any column length != `slab.len()` (caller-preallocated contract).
pub fn encode_into(cols: &mut QuadColumns, slab: &[Quad64]) {
    let n = slab.len();
    assert_eq!(cols.s.len(), n, "caller must pre-size columns");
    assert_eq!(cols.p.len(), n);
    assert_eq!(cols.o.len(), n);
    assert_eq!(cols.g.len(), n);
    assert_eq!(cols.t_ns.len(), n);
    assert_eq!(cols.vector_clock.len(), n);
    assert_eq!(cols.policy_mask.len(), n);
    assert_eq!(cols.inline_digest.len(), n * 32);
    for (i, q) in slab.iter().enumerate() {
        cols.s[i] = q.s_id;
        cols.p[i] = q.p_id;
        cols.o[i] = q.o_id;
        cols.g[i] = q.g_id;
        cols.t_ns[i] = q.t_ns;
        cols.vector_clock[i] = q.vector_clock;
        cols.policy_mask[i] = q.policy_mask;
        let d = &mut cols.inline_digest[i * 32..(i + 1) * 32];
        d.copy_from_slice(&q.inline_digest);
    }
}

/// Decode SoA columns -> AoS slab, in place, no allocation.
///
/// # Panics
/// Panics on any column length mismatch.
pub fn decode_into(slab: &mut [Quad64], cols: &QuadColumns) {
    let n = slab.len();
    assert_eq!(cols.s.len(), n);
    assert_eq!(cols.p.len(), n, "caller must pre-size slab");
    assert_eq!(cols.o.len(), n);
    assert_eq!(cols.g.len(), n);
    assert_eq!(cols.t_ns.len(), n);
    assert_eq!(cols.vector_clock.len(), n);
    assert_eq!(cols.policy_mask.len(), n);
    assert_eq!(cols.inline_digest.len(), n * 32);
    for (i, q) in slab.iter_mut().enumerate() {
        q.s_id = cols.s[i];
        q.p_id = cols.p[i];
        q.o_id = cols.o[i];
        q.g_id = cols.g[i];
        q.t_ns = cols.t_ns[i];
        q.vector_clock = cols.vector_clock[i];
        q.policy_mask = cols.policy_mask[i];
        q.inline_digest
            .copy_from_slice(&cols.inline_digest[i * 32..(i + 1) * 32]);
    }
}

/// A slab of `n` zeroed quads (Quad64 carries no Default; BOTTOM_TERM_ID /
/// zero timestamp / zero digest is the canonical zero record).
fn zero_slab(n: usize) -> Vec<Quad64> {
    vec![Quad64::new(0, 0, 0, 0, 0, 0, 0, [0u8; 32]); n]
}

/// Run the three columnar benches over a fresh 1M-quad slab.
/// Returns (encode GiB/s, decode GiB/s, scan-column GiB/s) at MIN round.
pub fn run() -> (f64, f64, f64) {
    let slab = make_slab(SCAN_ROWS);
    let bytes = (SCAN_ROWS * 64) as f64;
    println!(
        "columnar bench: {SCAN_ROWS} QuadRows ({} MiB payload), SoA split/merge + single-column scan",
        SCAN_ROWS * 64 / (1024 * 1024)
    );

    // Correctness first (Chicago): exact round-trip on the real slab.
    let mut cols = QuadColumns::with_capacity(SCAN_ROWS);
    encode_into(&mut cols, &slab);
    let mut back = zero_slab(SCAN_ROWS);
    decode_into(&mut back, &cols);
    // g was preserved column-wise; assert full field equality.
    for (a, b) in slab.iter().zip(back.iter()) {
        assert_eq!(a, b, "columnar round-trip must be exact");
    }
    println!(
        "columnar round-trip: exact on {} quads (AoS == SoA-decoded)",
        SCAN_ROWS
    );

    // Preallocate everything the timed loops touch so the zero-alloc gate
    // measures the codec, not the allocator bootstrap.
    let mut enc_dst = QuadColumns::with_capacity(SCAN_ROWS);
    let mut dec_dst: Vec<Quad64> = zero_slab(SCAN_ROWS);
    let cols_ref = &cols;

    // Kernel 1: encode (AoS -> SoA).
    let (min_enc, enc_allocs) = rounds("columnar encode", "GiB/s-quad", bytes, || {
        encode_into(&mut enc_dst, &slab);
        black_box(u64::from(enc_dst.s[SCAN_ROWS - 1]))
    });
    println!("  columnar encode allocs in timed round: {enc_allocs} (gate: 0)");
    if enc_allocs != 0 {
        eprintln!("ZERO-ALLOC FAIL: columnar encode made {enc_allocs} allocations");
        std::process::exit(1);
    }

    // Kernel 2: decode (SoA -> AoS).
    let (min_dec, dec_allocs) = rounds("columnar decode", "GiB/s-quad", bytes, {
        let dec_dst = &mut dec_dst;
        move || {
            decode_into(dec_dst, cols_ref);
            black_box(dec_dst[SCAN_ROWS - 1].s_id) as u64
        }
    });
    println!("  columnar decode allocs in timed round: {dec_allocs} (gate: 0)");
    if dec_allocs != 0 {
        eprintln!("ZERO-ALLOC FAIL: columnar decode made {dec_allocs} allocations");
        std::process::exit(1);
    }

    // Kernel 3: single-column scan (s) vs the AoS row path — the columnar
    // thesis made falsifiable: 4 B/row should move ~16x the rows a 64 B/row
    // AoS scan does at equal bandwidth.
    let (min_scan, scan_allocs) = rounds(
        "columnar sum s_id",
        "GiB/s-col",
        4.0 * SCAN_ROWS as f64,
        || {
            let acc: u64 = cols_ref.s.iter().map(|&v| v as u64).sum();
            black_box(acc)
        },
    );
    let aos_scan_gibps = {
        let (dt, _) = rounds(
            "AoS sum s_id (row path)",
            "GiB/s-col",
            4.0 * SCAN_ROWS as f64,
            || {
                let acc: u64 = slab.iter().map(|q| q.s_id as u64).sum();
                black_box(acc)
            },
        );
        (4.0 * SCAN_ROWS as f64) / dt / GIB
    };
    println!("  columnar sum allocs in timed round: {scan_allocs} (gate: 0)");
    if scan_allocs != 0 {
        eprintln!("ZERO-ALLOC FAIL: columnar scan made {scan_allocs} allocations");
        std::process::exit(1);
    }

    println!(
        "columnar receipt: encode {:.3} GiB/s, decode {:.3} GiB/s, sum(s) col {:.3} GiB/s-col (4 B/row) vs AoS row path {:.3} GiB/s-col",
        bytes / min_enc / GIB,
        bytes / min_dec / GIB,
        (4.0 * SCAN_ROWS as f64) / min_scan / GIB,
        aos_scan_gibps
    );

    (
        bytes / min_enc / GIB,
        bytes / min_dec / GIB,
        (4.0 * SCAN_ROWS as f64) / min_scan / GIB,
    )
}
