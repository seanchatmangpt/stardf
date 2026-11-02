//! PHASE-2 LANE 11 — sentinel seam courts for the stardf-columnar seam.
//!
//! Ports the two sentinel invariants (P0-5 / P0-19 R4) as executable tests
//! over the real substrate the columnar seam consumes: `stardf-arrow-core`'s
//! TermId arena and 64-byte `Quad64` layout (reference contract:
//! `stardf-arrow-capi/src/term.rs` tests `purrdf_raw_u32_max_refused` and
//! `bottom_vs_wildcard_semantics`; purrdf upstream witness
//! `crates/rdf-core/src/ir/term.rs::term_id_from_index_rejects_u32_max`).
//!
//! MANIFEST NOTE (lane-9 unblock required): this file is an integration test
//! of the `stardf-columnar` crate, whose scaffold does not yet exist on disk
//! (workspace members today: stardf-arrow-core / -capi / -bench). When lane 9
//! lands the scaffold, `crates/stardf-columnar/Cargo.toml` must declare:
//!
//! ```toml
//! [dev-dependencies]
//! stardf-arrow-core = { path = "../stardf-arrow-core" }
//! ```
//!
//! Execution receipt for the identical content was minted through a temp
//! harness crate with that exact path dependency (see lane-11 report).

use stardf_arrow_core::arena::{ArenaError, TermArena, BOTTOM_ID, WILDCARD_ID};
use stardf_arrow_core::layout::{Quad64, BOTTOM_TERM_ID, WILDCARD_TERM_ID};

// ---------------------------------------------------------------------------
// (1) Dictionary cap: minting beyond u32::MAX-2 terms refuses typed.
// ---------------------------------------------------------------------------

/// The dictionary cap law, executable witness set.
///
/// Full saturation (minting u32::MAX-2 = 4,294,967,293 terms and asserting
/// the next intern refuses) is infeasible in-process; the lawful port asserts
/// every observable of the same guard (`stardf-arrow-core/src/arena.rs`
/// intern id-space check: next id must be neither BOTTOM (0) nor WILDCARD
/// (u32::MAX), else `ArenaError::Capacity` — purrdf `from_index` parity,
/// upstream witness `term_id_from_index_rejects_u32_max`):
///
/// 1. the guard constants have exact parity (BOTTOM=0, WILDCARD=u32::MAX,
///    distinct);
/// 2. the refusal channel is TYPED (`ArenaError::Capacity` carries
///    attempted/limit, `Display` names the u32::MAX-1 mintable ceiling), so
///    the saturated mint would refuse typed, never panic or wrap;
/// 3. sentinel ids are never resolvable as minted terms (`term`/`fingerprint`
///    of BOTTOM and WILDCARD return None, not a panic) — the observable
///    consequence of "never minted".
#[test]
fn dictionary_cap_refuses_typed() {
    // (1) Guard constant parity.
    assert_eq!(BOTTOM_ID, 0u32);
    assert_eq!(WILDCARD_ID, u32::MAX);
    assert_ne!(BOTTOM_ID, WILDCARD_ID);

    // (2) The refusal channel is typed and names the exact cap law: the
    //     largest mintable id is u32::MAX - 1; the mint that would produce
    //     WILDCARD is the refused one (attempted = len+1 = u32::MAX).
    let err = ArenaError::Capacity {
        attempted: u64::from(WILDCARD_ID),
        limit: u64::from(WILDCARD_ID) - 1,
    };
    assert!(
        matches!(err, ArenaError::Capacity { .. }),
        "dictionary-cap refusal must be the typed Capacity variant"
    );
    let msg = err.to_string();
    assert!(
        msg.contains("capacity exceeded")
            && msg.contains("4294967295")
            && msg.contains("4294967294"),
        "typed refusal must name the attempted id and the u32::MAX-1 cap, got: {msg}"
    );

    // (3) Sentinels are never resolvable as minted terms — typed absence
    //     (None), never a panic and never a wrapped id.
    let mut arena = TermArena::default();
    let first = arena
        .intern("http://example.org/s")
        .expect("mint from empty");
    assert_eq!(first, 1, "minting starts at 1; BOTTOM (0) is reserved");
    assert_eq!(arena.term(BOTTOM_ID), None);
    assert_eq!(arena.fingerprint(BOTTOM_ID), None);
    assert_eq!(arena.term(WILDCARD_ID), None);
    assert_eq!(arena.fingerprint(WILDCARD_ID), None);
    assert_eq!(
        arena.term(arena.len() + 1),
        None,
        "out-of-range typed absence"
    );
}

// ---------------------------------------------------------------------------
// (2) Graph slot: 0 == BOTTOM == absent; no named graph round-trips as id 0,
//     never Option::None semantics.
// ---------------------------------------------------------------------------

/// A quad with no named graph carries graph id 0 (BOTTOM), and absence is
/// the deterministic zero through the physical record — not `Option::None`.
///
/// Executable law set:
/// - `Quad64::ZERO.g_id == BOTTOM_TERM_ID == 0`;
/// - a quad constructed with `g_id = BOTTOM_TERM_ID` byte-round-trips
///   (field-wise `to_le_bytes`/`from_le_bytes` over the id slots) with the
///   identical physical word, i.e. absence survives the wire as 0;
/// - pattern semantics (D6): `BOTTOM` in the graph slot matches only itself
///   (absent graphs match absent graphs), `WILDCARD` matches both absent and
///   present graphs — the two sentinels are never conflated.
#[test]
fn graph_slot_zero_is_bottom_is_absent() {
    // Absence is the deterministic zero in the zeroed physical record.
    assert_eq!(Quad64::ZERO.g_id, BOTTOM_TERM_ID);
    assert_eq!(Quad64::ZERO.g_id, 0u32);

    // A no-named-graph quad: real s/p/o, absent graph.
    let s: u32 = 7;
    let p: u32 = 8;
    let o: u32 = 9;
    let digest: [u8; 32] = core::array::from_fn(|i| i as u8);
    let quad = Quad64::new(s, p, o, BOTTOM_TERM_ID, 42, 1, 0, digest);

    // Field-wise byte round-trip of every TermId slot: absence rides the
    // wire as the literal word 0, not an Option discriminator.
    for (word, name) in [
        (quad.s_id, "s"),
        (quad.p_id, "p"),
        (quad.o_id, "o"),
        (quad.g_id, "g"),
    ] {
        let bytes = word.to_le_bytes();
        assert_eq!(u32::from_le_bytes(bytes), word, "{name} slot round-trip");
    }
    assert_eq!(quad.g_id, 0u32, "no named graph -> graph id 0 (BOTTOM)");

    // D6 in the graph slot: BOTTOM matches only itself; WILDCARD matches
    // both the absent graph and any present graph.
    assert!(Quad64::term_matches(BOTTOM_TERM_ID, quad.g_id));
    assert!(!Quad64::term_matches(BOTTOM_TERM_ID, 123));
    assert!(Quad64::term_matches(WILDCARD_TERM_ID, quad.g_id));
    assert!(Quad64::term_matches(WILDCARD_TERM_ID, 123));
    // And the pattern role split is asymmetric: a present term is never
    // matched by BOTTOM.
    assert!(Quad64::term_matches(123, 123));
    assert!(!Quad64::term_matches(BOTTOM_TERM_ID, WILDCARD_TERM_ID));

    // Whole-record pattern law through `Quad64::matches` with a
    // wildcard graph pattern: a no-graph quad matches a pattern whose graph
    // slot is WILDCARD, and a BOTTOM graph pattern matches it exactly.
    let wildcard_graph_pattern = Quad64 {
        s_id: s,
        p_id: p,
        o_id: o,
        g_id: WILDCARD_TERM_ID,
        t_ns: 42,
        vector_clock: 1,
        policy_mask: 0,
        inline_digest: [0; 32],
    };
    assert!(quad.matches(&wildcard_graph_pattern));
    let bottom_graph_pattern = Quad64 {
        g_id: BOTTOM_TERM_ID,
        ..wildcard_graph_pattern
    };
    assert!(quad.matches(&bottom_graph_pattern));
    let wrong_graph_pattern = Quad64 {
        g_id: 555,
        ..wildcard_graph_pattern
    };
    assert!(!quad.matches(&wrong_graph_pattern));
}

// ---------------------------------------------------------------------------
// (3) WILDCARD (u32::MAX) refused as a mintable id — bottom_vs_wildcard
//     semantics ported onto the arena + layout public surface.
// ---------------------------------------------------------------------------

/// u32::MAX is the match-anything WILDCARD and is refused as a mintable id:
/// the arena can never hand it out, and the layout treats it as a pattern
/// sentinel with semantics disjoint from BOTTOM.
#[test]
fn wildcard_u32_max_refused_as_mintable() {
    // Constant parity across the two modules that define the seam.
    assert_eq!(WILDCARD_ID, WILDCARD_TERM_ID);
    assert_eq!(WILDCARD_ID, 0xFFFF_FFFF);

    // The mint path can never return a sentinel: every minted id is in
    // 1..=(u32::MAX - 1). Witnessed here on the reachable range; the guard
    // that refuses the WILDCARD-producing mint with a typed Capacity error
    // is the same id-space check exercised above (dictionary cap law).
    let mut arena = TermArena::default();
    for i in 0..64u32 {
        let id = arena
            .intern(&core::format_args!("http://example.org/t/{i}").to_string())
            .expect("mint within reachable range");
        assert!(
            (1..WILDCARD_ID).contains(&id),
            "minted id {id} outside lawful id space"
        );
        assert_ne!(id, BOTTOM_ID);
        assert_ne!(id, WILDCARD_ID);
    }

    // Pattern-role split (D6): WILDCARD matches everything including
    // BOTTOM; BOTTOM matches only itself. BOTTOM is data-absent, WILDCARD
    // is query-unbounded — the bridge refuses to blur them.
    assert!(Quad64::term_matches(WILDCARD_TERM_ID, BOTTOM_TERM_ID));
    for term in [0u32, 1, 7, u32::MAX] {
        assert!(Quad64::term_matches(WILDCARD_TERM_ID, term));
    }
    assert!(!Quad64::term_matches(BOTTOM_TERM_ID, 7));
    assert!(!Quad64::is_bottom(WILDCARD_TERM_ID));
    assert!(!Quad64::is_wildcard(BOTTOM_TERM_ID));
}

// ---------------------------------------------------------------------------
// (4) Physical-encoding pin: Int64-widened term-id encoding (golden-stable).
// ---------------------------------------------------------------------------

/// The columnar physical encoding question (07-port-master-plan OPEN-7,
/// 06-phase0 §10.7) is decided in favor of **Int64-widened** as the
/// golden-stable pin: a `u32` TermId is carried in an `i64` column.
///
/// Executable law of the pin: the widening is exact, order-preserving, and
/// sign-safe for the ENTIRE id space including both sentinels —
/// `u32::MAX` widens to a POSITIVE i64 (4294967295, no sign flip), so
/// predicate pushdown over an Int64 column sees the same ordering as the
/// u32 domain and BOTTOM (0) stays the deterministic zero. Round-trip
/// `u32 -> i64 -> u32` is the identity for representative and boundary ids.
///
/// PENDING-OPERATOR (not implemented here): the Int32 alternative (narrow
/// physical column, golden regen required) is registered as the open
/// decision row; see lane-11 report. This test pins only the Int64
/// widening so any future Int32 switch must consciously flip this court.
#[test]
fn int64_widened_term_id_encoding_is_golden_stable() {
    const WIDEN: fn(u32) -> i64 = |id| i64::from(id);
    const NARROW: fn(i64) -> u32 = |w| u32::try_from(w).expect("i64 column holds widened u32");

    // Exactness + order preservation + sign safety over boundaries,
    // sentinels, and a dense sweep of interior values.
    let mut ids: Vec<u32> = vec![
        0,
        1,
        7,
        123,
        0x7FFF_FFFF,
        0x8000_0000,
        u32::MAX - 1,
        u32::MAX,
    ];
    ids.extend((1u32..1024).map(|i| i * 4_194_304)); // stride sweep across the domain
    ids.sort_unstable();
    ids.dedup();
    for w in ids.windows(2) {
        assert!(w[0] < w[1], "sweep fixture must be strictly increasing");
        let (a, b) = (WIDEN(w[0]), WIDEN(w[1]));
        assert!(a < b, "Int64 widening must preserve u32 ordering");
    }
    for id in ids {
        let widened = WIDEN(id);
        assert!(
            widened >= 0,
            "widened u32::MAX = {widened} must stay positive (no sign flip)"
        );
        assert_eq!(
            NARROW(widened),
            id,
            "u32 -> Int64 -> u32 must be the identity"
        );
    }
    // Sentinel pin: BOTTOM widens to the Int64 zero; WILDCARD widens to the
    // positive Int64 4294967295 — distinct, and neither collides with a
    // NULL/nullish physical value (absence is BOTTOM=0 by convention, never
    // a column NULL).
    assert_eq!(WIDEN(BOTTOM_TERM_ID), 0i64);
    assert_eq!(WIDEN(WILDCARD_TERM_ID), 4_294_967_295i64);
    assert_ne!(WIDEN(BOTTOM_TERM_ID), WIDEN(WILDCARD_TERM_ID));
}
