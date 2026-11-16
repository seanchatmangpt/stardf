// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Snapshot-delta parallel rule derivation over the stardf-datalog spine
//! (SDPF: per-stratum delta pages of fixed-width `[u64; 4]` rows).
//!
//! # DESIGN-AROUND LAW (named per lane contract, RDF-2 PARALLEL-MATERIALIZE)
//!
//! OST patent family P1's stated problem is concurrent read/write contention
//! on shared slices of one in-memory index, solved there by bespoke
//! lock-free structures. This crate names why P1 is NOT-practiced here:
//! base facts are zero-copy immutable pages. Each parallel worker reads the
//! SAME immutable base-fact pages and derives into its OWN FRESH DELTA
//! PAGES; a snapshot-merge at the stratum join point combines deltas. No
//! shared mutable index exists in the pipeline: no locks, no read/write
//! contention BY CONSTRUCTION. P1's lock-free machinery is unnecessary,
//! not approximated. The contention-honesty court (tests/parallel_courts.rs)
//! measures whether the snapshot-merge phase degenerates back toward P1's
//! lock-management problem and records the numbers either way.
//!
//! Workers own disjoint INDEPENDENT rule clusters (union-find over the
//! head/body predicate dependency graph; rules that chain share a cluster,
//! hence a worker). Each worker runs the spine's bounded stratified
//! fixpoint over the full base snapshot with only its cluster's rules,
//! diffs its final pages against the base snapshot into fresh fixed-width
//! delta pages, and the deltas merge deterministically (worker-id order).
//!
//! # Seeds are canonical
//!
//! [`big_corpus`] is splitmix64-seeded (seed pinned in the courts) and the
//! graph per family is a set of disjoint chains, so the transitive closure
//! is bounded and the anti-vacuity court's no-self-loop invariant holds by
//! construction.

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use purrdf_datalog::spine::{
    eval::{EvalOptions as SpineEvalOptions, SpineError},
    rule::{Atom, Rule, SpineProgramBuilder, ARITY, PAD, TERM_MASK, VAR_TAG},
    SpineEval, SpineProgram, Stratification,
};

/// Fixed-width row type: the spine's delta representation, verbatim.
pub type Row = [u64; ARITY];

/// Errors from parallel materialization.
#[derive(Debug)]
pub enum MaterializeError {
    /// The spine refused the program.
    Spine(SpineError),
    /// A worker failed mid-derivation.
    Worker {
        /// worker id (0-based)
        worker: usize,
        /// what failed
        cause: String,
    },
    /// A corpus-spec line was malformed.
    Spec(String),
}

impl core::fmt::Display for MaterializeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Spine(e) => write!(f, "spine refused: {e:?}"),
            Self::Worker { worker, cause } => write!(f, "worker {worker} failed: {cause}"),
            Self::Spec(msg) => write!(f, "corpus spec refused: {msg}"),
        }
    }
}

impl std::error::Error for MaterializeError {}

/// Splitmix64 (spine-corpus semantics; caller-supplied seed = own stream).
#[must_use]
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Canonical seed corpus parameters (one `key=value` line each in a spec
/// file; see [`parse_corpus_spec`]).
#[derive(Debug, Clone)]
pub struct CorpusSpec {
    /// chains per family
    pub chains_per_family: usize,
    /// edges per chain (chain length)
    pub chain_len: usize,
    /// independent rule families (disjoint predicate subgraphs)
    pub families: usize,
    /// anti-vacuity flip: when true, each chain head gains a self-loop edge
    pub self_loops: bool,
    /// splitmix64 seed (reserved for randomized tie-breaking; chains are
    /// deterministic, so the corpus is seed-stable)
    pub seed: u64,
}

/// Node id encoding for family `k`, chain `c`, position `i`.
#[must_use]
pub fn node_id(k: usize, c: usize, i: usize) -> u64 {
    (((k * 1_000_003 + c * 65_537 + i) as u64) << 8) & TERM_MASK
}

/// Predicate ids for family `k`: node=4k, edge=4k+1, tc=4k+2, iso=4k+3.
#[must_use]
pub fn pred_ids(k: usize) -> (u32, u32, u32, u32) {
    let b = 4 * k as u32;
    (b, b + 1, b + 2, b + 3)
}

/// Build the seeded chain-corpus program for `spec`.
///
/// Rules per family (clusters: tc-cluster of 3 rules, iso-cluster of 1
/// rule; independent across families and between a family's two clusters):
/// - tc_k(X,Z) :- edge_k(X,Y), edge_k(Y,Z)
/// - tc_k(X,Z) :- tc_k(X,Y), edge_k(Y,Z)
/// - tc_k(X,Z) :- edge_k(X,Y), tc_k(Y,Z)
/// - iso_k(X)  :- node_k(X), !tc_k(X,X)
///
/// Base facts: node_k over chain nodes, edge_k over chain edges; when
/// `spec.self_loops`, one extra edge_k(i, i) per chain head (the
/// anti-vacuity flip: it makes tc_k(X, X) satisfiable).
#[must_use]
pub fn big_corpus(spec: &CorpusSpec) -> SpineProgram {
    let npreds = 4 * spec.families;
    let cap = (spec.chains_per_family * (spec.chain_len + 1) + spec.chains_per_family + 16)
        .next_power_of_two()
        .max(1024);
    let mut b = SpineProgramBuilder::new(npreds, cap);
    for k in 0..spec.families {
        let (node, edge, _tc, _iso) = pred_ids(k);
        for c in 0..spec.chains_per_family {
            for i in 0..=spec.chain_len {
                b.fact(node, &[node_id(k, c, i)]);
            }
            for i in 0..spec.chain_len {
                b.fact(edge, &[node_id(k, c, i), node_id(k, c, i + 1)]);
            }
            if spec.self_loops {
                b.fact(edge, &[node_id(k, c, 0), node_id(k, c, 0)]);
            }
        }
    }
    for k in 0..spec.families {
        let (node, edge, tc, iso) = pred_ids(k);
        let v1 = VAR_TAG | 1;
        let v2 = VAR_TAG | 2;
        let v3 = VAR_TAG | 3;
        let a = |pred: u32, x: u64, y: u64| Atom { pred, args: [x, y, PAD, PAD] };
        b.rule(Rule::new(a(tc, v1, v2), vec![a(edge, v1, v2)]));
        b.rule(Rule::new(a(tc, v1, v2), vec![a(tc, v1, v3), a(edge, v3, v2)]));
        b.rule(Rule::new(a(tc, v1, v2), vec![a(edge, v1, v3), a(tc, v3, v2)]));
        let mut r = Rule::new(
            Atom { pred: iso, args: [v1, PAD, PAD, PAD] },
            vec![
                Atom { pred: node, args: [v1, PAD, PAD, PAD] },
                a(tc, v1, v1),
            ],
        );
        r = r.negated(1);
        b.rule(r);
    }
    b.build()
}

/// Union-find over rules: merge rule i and j iff rule i's head pred occurs
/// in rule j's body (negated included), or vice versa, transitively. Rules
/// within one cluster are chained; clusters are mutually independent.
#[must_use]
pub fn cluster_rules(rules: &[Rule]) -> Vec<Vec<usize>> {
    let n = rules.len();
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut Vec<usize>, mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    fn union(parent: &mut Vec<usize>, x: usize, y: usize) {
        let rx = find(parent, x);
        let ry = find(parent, y);
        if rx != ry {
            parent[rx] = ry;
        }
    }

    for i in 0..n {
        for j in i + 1..n {
            let feeds = rules[j].body.iter().any(|a| a.pred == rules[i].head.pred);
            let fed = rules[i].body.iter().any(|a| a.pred == rules[j].head.pred);
            if feeds || fed {
                union(&mut parent, i, j);
            }
        }
    }

    let mut by_root: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        by_root.entry(root).or_default().push(i);
    }
    let mut clusters: Vec<Vec<usize>> = by_root.into_values().collect();
    clusters.sort_by_key(|g| g.first().copied());
    clusters
}

/// Caller options for parallel derivation (mirrors the spine's EvalOptions).
#[derive(Debug, Clone)]
pub struct ParallelOptions {
    /// per-stratum iteration cap passed to the spine
    pub max_iterations: usize,
    /// spine page capacity (base facts + derived rows, per predicate)
    pub page_capacity: usize,
    /// anti-vacuity flip hook (court-only; see the spine's EvalOptions)
    pub force_derive: bool,
}

impl Default for ParallelOptions {
    fn default() -> Self {
        Self { max_iterations: 1024, page_capacity: 1 << 21, force_derive: false }
    }
}

/// Parallel derivation result.
#[derive(Debug)]
pub struct Materialized {
    /// merged model: base + all worker deltas, per predicate, sorted+deduped
    pub merged: Vec<Vec<Row>>,
    /// derived-only rows, per predicate, sorted (the merged deltas)
    pub derived_sorted: Vec<Vec<Row>>,
    /// per-phase timings and per-worker stats
    pub timings: Timings,
}

/// Per-phase timings, surfaced for the contention-honesty receipt.
#[derive(Debug, Clone, Default)]
pub struct Timings {
    /// wall-clock ms, parallel derive phase (spawn to last join)
    pub derive_ms: u128,
    /// wall-clock ms, snapshot-merge phase (single-threaded, worker-id order)
    pub merge_ms: u128,
    /// per-worker wall-clock ms inside the worker thread
    pub worker_ms: Vec<u128>,
    /// derived rows per worker (summed over predicates)
    pub worker_rows: Vec<usize>,
    /// workers actually used
    pub workers: usize,
}

/// Upper bound on workers.
pub const MAX_WORKERS: usize = 32;

/// Parallel derivation entry point.
///
/// Partitions `program`'s rules into `workers` independent clusters
/// (union-find; chained rules share a cluster and a worker). Each worker
/// clones the immutable base snapshot, runs the spine's bounded stratified
/// fixpoint with only its cluster's rules, diffs its final pages against
/// the base snapshot into fresh fixed-width delta pages. Deltas merge in
/// worker-id order.
///
/// # Errors
/// [`MaterializeError::Spine`] on stratification/eval refusal;
/// [`MaterializeError::Worker`] if a worker panicked.
pub fn derive_parallel(
    mut program: SpineProgram,
    workers: usize,
    opts: &ParallelOptions,
) -> Result<Materialized, MaterializeError> {
    let workers = workers.clamp(1, MAX_WORKERS);
    let clusters = cluster_rules(&program.rules);
    let njobs = workers.min(clusters.len()).max(1);
    let mut jobs: Vec<Vec<usize>> = vec![Vec::new(); njobs];
    for (ci, cluster) in clusters.iter().enumerate() {
        jobs[ci % njobs].extend(cluster.iter().copied());
    }
    let base_sorted: Arc<Vec<Vec<Row>>> = Arc::new(
        (0..program.num_preds)
            .map(|p| {
                let mut rows = program.base_facts(p as u32).to_vec();
                rows.sort_unstable();
                rows.dedup();
                rows
            })
            .collect(),
    );
    let strat = Arc::new(
        Stratification::new(&mut program)
                .map_err(SpineError::Stratification)
                .map_err(MaterializeError::Spine)?,
    );
    let shared_program = Arc::new(program);
    let mut handles = Vec::with_capacity(njobs);
    for (worker, rule_indices) in jobs.iter().enumerate() {
        let program = Arc::clone(&shared_program);
        let base = Arc::clone(&base_sorted);
        let strat = Arc::clone(&strat);
        let rule_indices = rule_indices.clone();
        let opts = opts.clone();
        handles.push(
            thread::Builder::new()
                .name(format!("stardf-materialize-{worker}"))
                .spawn(move || run_worker(worker, &program, &strat, &base, &rule_indices, &opts))
                .map_err(|e| MaterializeError::Worker {
                    worker,
                    cause: format!("spawn: {e}"),
                })?,
        );
    }
    let derive_t0 = Instant::now();
    let mut deltas: Vec<Vec<Vec<Row>>> = Vec::with_capacity(njobs);
    let mut worker_ms = Vec::with_capacity(njobs);
    let mut worker_rows = Vec::with_capacity(njobs);
    for (w, h) in handles.into_iter().enumerate() {
        let (delta, ms) = match h.join() {
            Ok(Ok(pair)) => pair,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(MaterializeError::Worker {
                    worker: w,
                    cause: "panic".to_string(),
                })
            },
        };
        worker_ms.push(ms);
        worker_rows.push(delta.iter().map(Vec::len).sum());
        deltas.push(delta);
    }
    let derive_ms = derive_t0.elapsed().as_millis();
    let (merged, derived_sorted, merge_ms) = merge_snapshots(&base_sorted, &deltas);
    Ok(Materialized {
        merged,
        derived_sorted,
        timings: Timings {
            derive_ms,
            merge_ms,
            worker_ms,
            worker_rows,
            workers: njobs,
        },
    })
}

/// One worker: clone the immutable base snapshot, evaluate ONLY this
/// worker's rules over it to fixpoint, diff final pages into delta pages.
fn run_worker(
    worker: usize,
    program: &SpineProgram,
    strat: &Stratification,
    base: &[Vec<Row>],
    rule_indices: &[usize],
    opts: &ParallelOptions,
) -> Result<(Vec<Vec<Row>>, u128), MaterializeError> {
    let t0 = Instant::now();
    let mut sub = program.clone();
    sub.rules = rule_indices.iter().map(|&i| program.rules[i].clone()).collect();
    let mut eval = SpineEval::new(&sub, strat, opts.page_capacity);
    eval.run(&sub, &SpineEvalOptions {
        max_iterations: opts.max_iterations,
        force_derive: opts.force_derive,
    })
    .map_err(MaterializeError::Spine)?;
    let delta: Vec<Vec<Row>> = (0..program.num_preds)
        .map(|p| {
            let page = eval.page(p as u32);
            let mut derived: Vec<Row> = page
                .iter()
                .filter(|row| !contains_sorted(&base[p], row))
                .copied()
                .collect();
            derived.sort_unstable();
            derived.dedup();
            derived
        })
        .collect();
    Ok((delta, t0.elapsed().as_millis()))
}

/// Binary search in a sorted, deduped slice.
fn contains_sorted(rows: &[Row], row: &Row) -> bool {
    rows.binary_search(row).is_ok()
}

/// Merge deltas in worker-id order: base + every delta, sort + dedup.
/// Deterministic by construction; the court witnesses it differentially.
fn merge_snapshots(
    base: &[Vec<Row>],
    deltas: &[Vec<Vec<Row>>],
) -> (Vec<Vec<Row>>, Vec<Vec<Row>>, u128) {
    let t0 = Instant::now();
    let npreds = base.len();
    let mut merged = Vec::with_capacity(npreds);
    let mut derived = Vec::with_capacity(npreds);
    for p in 0..npreds {
        let mut rows: Vec<Row> = base[p].clone();
        for d in deltas {
            rows.extend(d[p].iter().copied());
        }
        rows.sort_unstable();
        rows.dedup();
        let mut just_derived: Vec<Row> = rows.clone();
        just_derived.retain(|r| !contains_sorted(&base[p], r));
        merged.push(rows);
        derived.push(just_derived);
    }
    (merged, derived, t0.elapsed().as_millis())
}

/// Corpus spec file format (line-oriented, `key=value`, `#` comments):
/// `chains_per_family`, `chain_len`, `families`, `self_loops` (0/1), `seed`.
///
/// # Errors
/// [`MaterializeError::Spec`] on a malformed line; never a silent skip.
pub fn parse_corpus_spec(text: &str) -> Result<CorpusSpec, MaterializeError> {
    let mut spec = CorpusSpec {
        chains_per_family: 0,
        chain_len: 0,
        families: 0,
        self_loops: false,
        seed: 0,
    };
    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let refused = |msg: String| MaterializeError::Spec(format!("line {lineno}: {msg}"));
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| refused(format!("no '=' in {line:?}")))?;
        match key.trim() {
            "self_loops" => {
                spec.self_loops = match value.trim() {
                    "0" | "false" => false,
                    "1" | "true" => true,
                    other => {
                        return Err(refused(format!("self_loops={other:?} must be 0/1")))
                    },
                };
            },
            "seed" => {
                spec.seed = value
                    .trim()
                    .parse::<u64>()
                    .map_err(|_| refused(format!("seed={value:?} not a u64")))?;
            },
            "chains_per_family" | "chain_len" | "families" => {
                let n = value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| refused(format!("{key}={value:?} not a usize")))?;
                if n == 0 {
                    return Err(refused(format!("{key} must be >= 1")));
                }
                match key.trim() {
                    "chains_per_family" => spec.chains_per_family = n,
                    "chain_len" => spec.chain_len = n,
                    _ => spec.families = n,
                }
            },
            unknown => {
                return Err(refused(format!("unknown key {unknown:?}")));
            },
        }
    }
    if spec.families == 0 {
        return Err(MaterializeError::Spec("families missing".to_string()));
    }
    if spec.chains_per_family == 0 {
        return Err(MaterializeError::Spec(
            "chains_per_family missing".to_string(),
        ));
    }
    if spec.chain_len == 0 {
        return Err(MaterializeError::Spec("chain_len missing".to_string()));
    }
    Ok(spec)
}

/// Render a spec back to the file format (round-trip for the /tmp courts).
#[must_use]
pub fn render_corpus_spec(spec: &CorpusSpec) -> String {
    format!(
        "# stardf-materialize corpus spec\n\
         chains_per_family={}\nchain_len={}\nfamilies={}\n\
         self_loops={}\nseed={}\n",
        spec.chains_per_family,
        spec.chain_len,
        spec.families,
        u8::from(spec.self_loops),
        spec.seed,
    )
}
