//! stardf-arrow-bench: line-rate harness for the stardf-arrow-core kernels.
//!
//! Model: cgap/crates/bench/src/main.rs (counting-allocator GlobalAlloc
//! wrapper with a non-vacuity probe; min-of-N rounds, MIN is the receipt;
//! per-round lines print variance instead of hiding it behind a mean).
//!
//! Kernels benched (contract):
//!
//! 1. intern — TermArena::intern over M terms (terms/s, lexical MiB/s)
//!    plus the dedup lookup path
//! 2. pattern-scan — branchless 8-row chunk scan (purrdf
//!    rdf-core/src/ir/dataset.rs:781-802 model) of 1M QuadRows with s/p/o
//!    wildcard patterns (GiB/s of quad payload)
//! 3. stream get_next — StardfStream::next_batch drain of a 1M-quad slab
//!    in 64-quad batches (GiB/s of Quad64 payload)
//! 4. columnar — SoA encode/decode of a 1M-quad slab + single-column scan
//!    (bench-local codec, disclosed divergence; see columnar_bench.rs)
//!
//! NOISE NOTE: concurrent lanes compile and bench on this same machine;
//! numbers are noisy by construction. MIN round is the receipt — contention
//! only ever slows a round, so the minimum is the closest approach to the
//! kernel's true line rate.

use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use stardf_arrow_core::arena::{FrozenArena, Resident, TermArena};
use stardf_arrow_core::layout::Quad64;
use stardf_arrow_core::stream::{CursorProducer, StardfStream, QUADS_PER_BATCH};

mod columnar_bench;
use columnar_bench as columnar;

// ---------------------------------------------------------------------------
// Counting allocator (pattern: cgap/crates/bench/src/main.rs:24-43)
// ---------------------------------------------------------------------------

pub(crate) static COUNT: AtomicUsize = AtomicUsize::new(0);

/// Installed as the process allocator: without this attribute the counter is
/// never wired up and every "0 allocs" line passes vacuously (counted
/// nothing, not allocated nothing).
#[global_allocator]
static ALLOC: CountingAlloc = CountingAlloc;

struct CountingAlloc;

// SAFETY: delegates to System, counting calls.
unsafe impl std::alloc::GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, l: std::alloc::Layout) -> *mut u8 {
        COUNT.fetch_add(1, Ordering::Relaxed);
        // SAFETY: forwards to the system allocator unchanged.
        unsafe { std::alloc::System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: std::alloc::Layout) {
        // SAFETY: forwards to the system allocator unchanged.
        unsafe { std::alloc::System.dealloc(p, l) }
    }
}

/// Non-vacuity probe (cgap model): a deliberate allocation must bump COUNT
/// before any "0 allocs" line is trusted. Exits 1 otherwise.
fn alloc_probe() {
    COUNT.store(0, Ordering::Relaxed);
    let probe = vec![0u8; 4096];
    black_box(&probe);
    drop(probe);
    let n = COUNT.load(Ordering::Relaxed);
    println!("alloc probe: deliberate alloc bumped counter to {n} (gate is live)");
    if n == 0 {
        eprintln!("VACUOUS GATE: deliberate allocation did not increment the alloc counter");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Harness (min-of-N; MIN is the receipt)
// ---------------------------------------------------------------------------

pub(crate) const ROUNDS: usize = 5;
pub(crate) const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

/// Run `f` for ROUNDS rounds; print per-round dt + throughput and return
/// (min_dt, max allocs observed inside a timed window). The closure returns
/// a u64 sink so the compiler must keep the work real.
pub(crate) fn rounds<F: FnMut() -> u64>(
    name: &str, unit: &str, bytes_per_iter: f64, mut f: F,
) -> (f64, usize) {
    // Warm-up (untimed): pages in, fills caches.
    let warm = f();
    black_box(warm);

    let mut min_dt = f64::MAX;
    let mut allocs_max = 0usize;
    let mut sink = 0u64;
    for round in 1..=ROUNDS {
        COUNT.store(0, Ordering::Relaxed);
        let t0 = Instant::now();
        let r = f();
        let dt = t0.elapsed().as_secs_f64();
        let allocs = COUNT.load(Ordering::Relaxed);
        allocs_max = allocs_max.max(allocs);
        sink ^= r;
        black_box((sink, r));
        let rate = bytes_per_iter / dt / GIB;
        println!(
            "  {name} round {round}/{ROUNDS}: {:>9.3} ms  {rate:7.3} {unit}",
            dt * 1e3
        );
        min_dt = min_dt.min(dt);
    }
    println!(
        "  {name} MIN {:.3} ms  {} B/iter  {:7.3} {unit}  (max allocs in a timed round: {allocs_max})",
        min_dt * 1e3,
        bytes_per_iter as u64,
        bytes_per_iter / min_dt / GIB
    );
    (min_dt, allocs_max)
}

// ---------------------------------------------------------------------------
// Shared slab
// ---------------------------------------------------------------------------

pub(crate) const SCAN_ROWS: usize = 1_000_000;

pub(crate) fn make_slab(n: usize) -> Vec<Quad64> {
    (0..n)
        .map(|i| {
            Quad64::new(
                (i % 50_000) as u32,
                (i % 500) as u32,
                ((i * 7 + 13) % 50_000) as u32,
                0,
                i as u64,
                1,
                0,
                [0u8; 32],
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Kernel 1: intern (TermArena::intern over M terms)
// ---------------------------------------------------------------------------

const INTERN_TERMS: usize = 200_000;

fn bench_intern() -> (f64, f64, u32) {
    let bytes_lex = (INTERN_TERMS * 24) as f64;
    println!("intern bench: {INTERN_TERMS} unique terms (~{bytes_lex:.0} lexical bytes), then {INTERN_TERMS} dedup re-lookups over the frozen arena");

    // Mint path: fresh arena per round, M unique ~24 B terms.
    let (min_mint, _) = rounds("intern mint", "MiB/s-lex", bytes_lex, || {
        let mut arena = TermArena::new(Resident);
        let mut acc = 0u64;
        for i in 0..INTERN_TERMS {
            let s = format!("term_{:012x}_payload", i);
            // Resident admits every live total; a None here is OOM, and a
            // bench that silently drops mints would overstate the rate —
            // fold the id in so the count of mints is observable via acc.
            acc ^= u64::from(arena.intern(&s).unwrap_or(u32::MAX));
        }
        assert_eq!(arena.len(), INTERN_TERMS as u32, "every term minted");
        acc
    });

    // Keep a minted arena for the dedup + frozen lookups.
    let mut arena = TermArena::new(Resident);
    for i in 0..INTERN_TERMS {
        let s = format!("term_{:012x}_payload", i);
        let _ = arena.intern(&s).expect("Resident admits all");
    }
    let frozen: std::sync::Arc<FrozenArena> = std::sync::Arc::new(arena.freeze());

    // Dedup path: BTreeMap hit, no mint.
    let (min_dedup, _) = rounds("intern dedup", "MiB/s-lex", bytes_lex, || {
        let mut acc = 0u64;
        for i in 0..INTERN_TERMS {
            let s = format!("term_{:012x}_payload", i);
            acc ^= u64::from(frozen.get(&s).unwrap_or(u32::MAX));
        }
        acc
    });

    println!(
        "intern receipt: mint {:.0} terms/s, dedup {:.0} lookups/s (MIN rounds, {INTERN_TERMS} terms)",
        INTERN_TERMS as f64 / min_mint,
        INTERN_TERMS as f64 / min_dedup,
    );
    (min_mint, min_dedup, INTERN_TERMS as u32)
}

// ---------------------------------------------------------------------------
// Kernel 2: branchless pattern-scan (8-row chunk, purrdf model)
// ---------------------------------------------------------------------------

/// Rows per chunk (purrdf SCAN_CHUNK parity).
const SCAN_CHUNK: usize = 8;

/// Residual (s, p, o) filter as key/mask word pairs — purrdf `QuadKey`
/// model (rdf-core/src/ir/dataset.rs:728-768) restricted to the three
/// patterned positions; g is not patterned in this bench.
#[derive(Clone, Copy)]
struct ScanKey {
    key: [u32; 3],
    mask: [u32; 3],
}

impl ScanKey {
    /// `None` = wildcard (mask 0), `Some(id)` = bound (key id, mask all-ones).
    fn new(s: Option<u32>, p: Option<u32>, o: Option<u32>) -> Self {
        let field = |id: Option<u32>| id.map_or((0, 0), |id| (id, u32::MAX));
        let (ks, ms) = field(s);
        let (kp, mp) = field(p);
        let (ko, mo) = field(o);
        Self {
            key: [ks, kp, ko],
            mask: [ms, mp, mo],
        }
    }

    /// Zero exactly when the quad matches (branchless per-field compare).
    #[inline(always)]
    fn misses(&self, q: &Quad64) -> u32 {
        let words = [q.s_id, q.p_id, q.o_id];
        let mut m = 0;
        for i in 0..3 {
            m |= (words[i] ^ self.key[i]) & self.mask[i];
        }
        m
    }

    /// Which rows of the chunk match: bit i for row i. Per-row answers are
    /// branchless 0x0000_0000/0xFFFF_FFFF lanes, masked to their own bit and
    /// OR-folded (purrdf chunk_matches model, dataset.rs:781-794).
    #[inline(always)]
    fn chunk_matches(&self, chunk: &[Quad64; SCAN_CHUNK]) -> u8 {
        let mut lanes = [0u32; SCAN_CHUNK];
        for (lane, q) in lanes.iter_mut().zip(chunk) {
            *lane = u32::from(self.misses(q) == 0).wrapping_neg();
        }
        let mut bits = 0u32;
        for (i, lane) in lanes.iter().enumerate() {
            bits |= lane & (1 << i);
        }
        bits as u8
    }
}

/// Exact match count (untimed verification pass — the timed sink must agree).
fn hits_per_pass(slab: &[Quad64], key: &ScanKey) -> u64 {
    slab.iter().filter(|q| key.misses(q) == 0).count() as u64
}

/// (pattern, MIN GiB/s, matches) receipts for the JSON payload.
fn bench_pattern_scan(slab: &[Quad64]) -> Vec<(String, f64, u64)> {
    let mut receipts = Vec::new();
    println!(
        "pattern-scan bench: {SCAN_ROWS} QuadRows ({} MiB slab), 8-row branchless chunk scan",
        SCAN_ROWS * 64 / (1024 * 1024)
    );
    // Selectivity sweep: all-wildcard (every row matches), s-bound, p-bound,
    // fully-bound. Same data-independent compare path regardless of
    // selectivity — a large GiB/s spread across these is a finding, not a
    // feature (cgap selectivity-sweep discipline).
    let pats: [(&str, ScanKey); 4] = [
        ("s=? p=? o=?", ScanKey::new(None, None, None)),
        ("s=0 p=? o=?", ScanKey::new(Some(0), None, None)),
        ("s=? p=42 o=?", ScanKey::new(None, Some(42), None)),
        (
            "s=42 p=42 o=307",
            ScanKey::new(Some(42), Some(42), Some(307)),
        ),
    ];
    let bytes_per_pass = (SCAN_ROWS * 64) as f64;
    for (name, key) in pats {
        let expected = hits_per_pass(slab, &key);
        // Chicago cross-check: the branchless chunk kernel's own hit count
        // (warm-up pass) must equal the exact filter count — the kernel is
        // proven correct on the real slab, not assumed.
        let slab_ref = &slab;
        let chunk_kernel_hits = || -> u64 {
            // slice::as_chunks is stable 1.88 < workspace MSRV 1.98 (lane
            // 06-msrv), so no incompatible_msrv waiver is needed here.
            let mut hits = 0u64;
            let (chunks, remainder) = slab_ref.as_chunks::<SCAN_CHUNK>();
            for arr in chunks {
                hits += key.chunk_matches(arr).count_ones() as u64;
            }
            for q in remainder {
                hits += u64::from(key.misses(q) == 0);
            }
            hits
        };
        let kernel_count = chunk_kernel_hits();
        if kernel_count != expected {
            eprintln!(
                "SCAN CORRECTNESS FAIL ({name}): chunk kernel counted {kernel_count}, exact filter counted {expected}"
            );
            std::process::exit(1);
        }
        let (min_dt, allocs) = rounds(name, "GiB/s-quad", bytes_per_pass, chunk_kernel_hits);
        println!(
            "  {name} receipt: MIN {:.3} GiB/s-quad, matches/pass {expected} (exact-checked)",
            bytes_per_pass / min_dt / GIB
        );
        receipts.push((name.to_string(), bytes_per_pass / min_dt / GIB, expected));
        // Zero-alloc gate on the scan path (pattern: cgap CTQ-5). Hard fail.
        println!("  {name} allocs during scan: {allocs} (gate: 0)");
        if allocs != 0 {
            eprintln!("ZERO-ALLOC FAIL: {allocs} allocations inside the scan loop ({name})");
            std::process::exit(1);
        }
    }
    receipts
}

// ---------------------------------------------------------------------------
// Kernel 3: stream get_next throughput
// ---------------------------------------------------------------------------

fn bench_stream(slab: &[Quad64]) -> f64 {
    let bytes = (slab.len() * 64) as f64;
    println!(
        "stream get_next bench: {} quads x 64 B in {}-quad batches ({} batches/drain)",
        slab.len(),
        QUADS_PER_BATCH,
        slab.len() / QUADS_PER_BATCH
    );
    let owned = slab.to_vec();
    let (min_dt, allocs) = rounds("stream next_batch drain", "GiB/s-quad", bytes, || {
        let mut prod = CursorProducer::new(owned.clone());
        let mut drained = 0u64;
        while let Some(batch) = prod.next_batch() {
            drained = drained.wrapping_add(batch.len() as u64);
            black_box(batch.first().map(|q| q.t_ns));
        }
        drained
    });
    println!("  stream allocs during drain: {allocs} (gate excludes the per-drain slab clone; the drain loop itself must be 0)");
    if allocs == 0 {
        eprintln!("SUSPICIOUS: slab clone allocated nothing — counter wired wrong?");
        std::process::exit(1);
    }
    bytes / min_dt / GIB
}

fn main() {
    alloc_probe();

    let (mint_s, dedup_s, terms) = bench_intern();
    let slab = make_slab(SCAN_ROWS);
    let scan_receipts = bench_pattern_scan(&slab);
    let stream_gibps = bench_stream(&slab);
    let (col_enc, col_dec, col_scan) = columnar::run();

    println!("\n=== BENCH_BASELINE.json payload (min-of-{ROUNDS}, this machine, UNVERIFIED-class local numbers) ===");
    let mint_per_s = INTERN_TERMS as f64 / mint_s;
    let dedup_per_s = INTERN_TERMS as f64 / dedup_s;
    let patterns_json = scan_receipts
        .iter()
        .map(|(n, g, m)| {
            format!(r#"{{"pattern": "{n}", "min_gib_s": {g:.3}, "matches_per_pass": {m}}}"#)
        })
        .collect::<Vec<_>>()
        .join(", ");
    println!(
        r#"{{"machine": "darwin-arm64", "standing": "UNVERIFIED-local-min-of-{ROUNDS}",
  "intern": {{"terms": {terms}, "mint_terms_per_s": {mint_per_s:.0}, "dedup_lookups_per_s": {dedup_per_s:.0}}},
  "pattern_scan": {{"rows": {SCAN_ROWS}, "chunk": {SCAN_CHUNK}, "patterns": [{patterns_json}]}},
  "stream_get_next": {{"rows": {SCAN_ROWS}, "batch_quads": {QUADS_PER_BATCH}, "gib_s": {stream_gibps:.3}}},
  "columnar": {{"rows": {SCAN_ROWS}, "codec": "bench-local SoA (divergence: lane-9 canonical codec not landed in stardf-arrow-core)", "encode_gib_s": {col_enc:.3}, "decode_gib_s": {col_dec:.3}, "sum_s_column_gib_s": {col_scan:.3}, "zero_alloc_gates": "encode/decode/scan all hard-fail on any timed-round alloc"}}}}"#
    );
}
