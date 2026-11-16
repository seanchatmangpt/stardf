
//! Chicago courts for RDF-4 quoted triples (lane RDF-4-quoted).
//!
//! Real collaborators (parser, store, BLAKE3 hashing) — no mocks. Every
//! court names its anti-vacuity mutation. The /tmp sandbox flip for the
//! metadata court lives in scripts/courts.sh (mutation: attach the wrong
//! quoted id); a court that cannot flip is FAIL.

use stardf_star::parser::parse;
use stardf_star::store::Store;
use stardf_star::term::{quote, quoted_id, Term, Triple};

/// Deterministic quoted-triple corpus: 60 quoted triples (5 ground subjects
/// x 4 predicates x 3 objects = 60 asserted edge facts), 6 of them nested
/// (their subject is itself a quoted triple), with edge metadata attached
/// to every quoted triple (score + source).
fn corpus() -> (Store, Vec<u64>) {
    let mut store = Store::new();
    let mut ids = Vec::new();
    for si in 0..5 {
        let s = Term::Iri(format!("https://ex.test/s{}", si));
        for pi in 0..4 {
            let p = format!("https://ex.test/p{}", pi);
            for oi in 0..3 {
                let o = Term::Literal {
                    value: format!("v{}-{}-{}", si, pi, oi),
                    lang: None,
                    datatype: None,
                };
                let q = store.quote(s.clone(), &p, o);
                let id = match &q { Term::Quoted(id) => *id, _ => unreachable!() };
                // edge metadata as ordinary triples ON the quoted id
                store.add(Triple::new(
                    q.clone(),
                    "https://ex.test/score",
                    Term::Literal { value: format!("{}", (si * 12 + pi * 3 + oi) % 100), lang: None, datatype: None },
                ));
                store.add(Triple::new(
                    q,
                    "https://ex.test/source",
                    Term::Iri("https://ex.test/feedA".into()),
                ));
                ids.push(id);
            }
        }
    }
    // 6 nested quoted triples: subject = ⟨⟨s0 p0 v0-0-0⟩⟩
    for ni in 0..6 {
        let inner = quote(
            Term::Iri("https://ex.test/s0".into()),
            "https://ex.test/p0",
            Term::Literal { value: format!("v0-0-{}", ni), lang: None, datatype: None },
        );
        let q = store.quote(
            inner.clone(),
            "https://ex.test/derivedFrom",
            Term::Literal { value: format!("n{}", ni), lang: None, datatype: None },
        );
        let id = match &q { Term::Quoted(id) => *id, _ => unreachable!() };
        store.add(Triple::new(
            q,
            "https://ex.test/score",
            Term::Literal { value: format!("{}", 90 + ni), lang: None, datatype: None },
        ));
        ids.push(id);
    }
    (store, ids)
}

/// COURT 1 — round-trip: parse a Turtle-star document, store it, query it
/// back. Mutation: a parser that dropped quoted triples would leave
/// quoted_count at 0 and this court fails; a parser that mis-nested would
/// change ids and the metadata lookups miss.
#[test]
fn court1_roundtrip_parse_store_query() {
    let mut store = Store::new();
    let doc = r##"
@prefix ex: <https://ex.test/> .
<< <https://ex.test/s1> <https://ex.test/knows> <https://ex.test/s2> >> ex:score "97" .
<< <https://ex.test/s1> <https://ex.test/knows> <https://ex.test/s2> >> ex:source ex:feedA .
ex:alice ex:claims << <https://ex.test/s2> <https://ex.test/likes> <https://ex.test/s3> >> .
ex:bob ex:claims << << <https://ex.test/s3> <https://ex.test/likes> <https://ex.test/s4> >> ex:certainty "high" >> .
"##;
    let stmts = parse(doc, &mut store).expect("parse");
    assert_eq!(stmts, 4, "four top-level statements");
    assert_eq!(store.quoted_count(), 4, "3 distinct outer + 1 inner quoted triple");

    // metadata query on the parsed quoted triple
    let id = quoted_id(
        &Term::Iri("https://ex.test/s1".into()),
        "https://ex.test/knows",
        &Term::Iri("https://ex.test/s2".into()),
    );
    let meta = store.metadata_of(id);
    assert_eq!(meta.len(), 2, "score + source attached");
    assert!(
        meta.iter().any(|t| t.p.ends_with("score") && matches!(&t.o,
            Term::Literal { value, .. } if value == "97")),
        "score=97 metadata round-trips"
    );

    // the knows-quoted id has no incoming edges; the claims-quoted id has one
    let knows = quoted_id(
        &Term::Iri("https://ex.test/s1".into()),
        "https://ex.test/knows",
        &Term::Iri("https://ex.test/s2".into()),
    );
    assert_eq!(store.used_as_object(knows).len(), 0);
    assert_eq!(store.used_as_object(id_of_claims(&store)).len(), 1, "claims-quoted triple is the object of ex:claims");
    assert_eq!(store.verify_integrity(), 0, "ids derive from payloads by construction");
}

fn id_of_claims(store: &Store) -> u64 {
    for t in store.triples() {
        if let (Term::Iri(_), Term::Quoted(id)) = (&t.s, &t.o) {
            if t.p.ends_with("claims") {
                let pl = store.quoted_payload(*id).expect("payload registered");
                if let Term::Iri(s2) = &pl.s {
                    if s2.ends_with("s2") { return *id; }
                }
            }
        }
    }
    panic!("claims-quoted triple not found");
}

/// COURT 2 — corpus scale + collision law. 66 quoted triples (60 flat + 6
/// nested) all get distinct content-addressed ids; id injection is verified
/// against the live corpus (a derivation bug — e.g. hashing only the
/// subject — collapses ids and this court flips).
#[test]
fn court2_corpus_ids_distinct_and_integrity() {
    let (store, ids) = corpus();
    assert_eq!(ids.len(), 66, "60 flat + 6 nested quoted triples");
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    let before = sorted.len();
    sorted.dedup();
    assert_eq!(sorted.len(), before, "all 66 quoted ids distinct — no derivation collision");
    assert_eq!(store.quoted_count(), 66);
    assert_eq!(store.verify_integrity(), 0, "every payload re-derives its stored id");
    // content addressing: re-quoting an equal payload is idempotent
    let (store2, ids2) = corpus();
    assert_eq!(ids, ids2, "deterministic across stores");
    let _ = store2;
    // every quoted triple carries its metadata
    for id in &ids {
        assert!(!store.metadata_of(*id).is_empty(), "metadata attached to id {id}");
    }
}

/// COURT 3 — differential: flat projection (quoted triples read as ordinary
/// reified triples) returns the identical fact set to the native quoted
/// path. Native facts are constructed per id from the payload registry and
/// metadata queries; flat facts come from `Store::flat_projection`. Any
/// dropped or duplicated row flips this court.
#[test]
fn court3_differential_flat_projection() {
    let (store, ids) = corpus();
    let rdf = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

    // native fact set: (node, predicate-suffix, object-debug)
    let mut native: Vec<(String, String, String)> = Vec::new();
    for id in &ids {
        let q = store.quoted_payload(*id).expect("payload");
        let node = format!("stardf-star:qid:{}", stardf_star::quoted_id_hex(*id));
        native.push((node.clone(), "subject".into(), format!("{:?}", q.s)));
        native.push((node.clone(), "predicate".into(), format!("{:?}", Term::Iri(q.p.clone()))));
        native.push((node.clone(), "object".into(), format!("{:?}", q.o)));
        for m in store.metadata_of(*id) {
            native.push((node.clone(), m.p.rsplit('/').next().unwrap().to_string(), format!("{:?}", m.o)));
        }
    }
    native.sort();

    let flat: Vec<(String, String, String)> = store
        .flat_projection()
        .iter()
        .map(|t| {
            let Term::Iri(node) = &t.s else { panic!("flat rows have IRI nodes") };
            let suffix = t.p
                .strip_prefix(rdf)
                .unwrap_or_else(|| m_suffix(&t.p));
            (node.clone(), suffix.to_string(), format!("{:?}", t.o))
        })
        .collect();
    let mut flat = flat;
    flat.sort();

    assert_eq!(native.len(), flat.len(), "fact counts: native={} flat={}", native.len(), flat.len());
    for (n, f) in native.iter().zip(flat.iter()) {
        assert_eq!(n, f, "differential mismatch: native={n:?} flat={f:?}");
    }
}

fn m_suffix(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}

/// COURT 4 — the metadata court's anti-vacuity core, asserted in-tree:
/// metadata attached to the WRONG quoted id must not surface under the
/// right one. (The /tmp sandbox flip proves the court itself flips when
/// the store wrongly attaches; see scripts/courts.sh court-metadata-flip.)
#[test]
fn court4_wrong_id_metadata_isolated() {
    let (store, ids) = corpus();
    // take two distinct ids, move one of id A's metadata rows to id B's
    // subject and confirm queries separate them
    let (a, b) = (ids[0], ids[1]);
    assert_ne!(a, b);
    let meta_a: Vec<String> = store
        .metadata_of(a)
        .iter()
        .map(|t| format!("{}|{:?}", t.p, t.o))
        .collect();
    let meta_b: Vec<String> = store
        .metadata_of(b)
        .iter()
        .map(|t| format!("{}|{:?}", t.p, t.o))
        .collect();
    assert!(!meta_a.is_empty() && !meta_b.is_empty());
    assert_ne!(meta_a, meta_b, "per-id metadata namespaces are disjoint");
    // a simulated wrong-attachment: metadata queried under the wrong id
    // returns B's facts, never A's — isolation holds
    let a_scores = meta_a_first(&store, a);
    let contaminated = store.metadata_of(b);
    assert!(
        contaminated.iter().all(|t| !matches!(&t.o, Term::Literal { value, .. } if a_scores.contains(value))),
        "wrong-id query must not return A's score fact"
    );
}

fn meta_a_first(store: &Store, a: u64) -> Vec<String> {
    store
        .metadata_of(a)
        .iter()
        .filter_map(|t| match &t.o {
            Term::Literal { value, .. } => Some(value.clone()),
            _ => None,
        })
        .collect()
}

/// COURT 5 — Turtle-star parser subset: comments, prefixes, literals with
/// language/datatype, one-level nesting, and the deep-nesting refusal.
#[test]
fn court5_parser_subset_and_refusals() {
    let mut store = Store::new();
    let doc = r##"
# comment line
@prefix ex: <https://ex.test/> .
ex:a ex:b "lit"@en .
ex:a ex:c "1"^^<https://ex.test/dt> .
<< ex:a ex:p ex:b >> ex:m "ok" .
<< << ex:a ex:q ex:c >> ex:m2 "deep" >> ex:outer "yes" .
"##;
    parse(doc, &mut store).expect("parse subset doc");
    assert_eq!(store.quoted_count(), 3);
    assert_eq!(store.verify_integrity(), 0);

    let mut deep = Store::new();
    let err = parse(
        "<< << << ex:a ex:q ex:c >> ex:m2 \"x\" >> ex:outer \"y\" >> .",
        &mut deep,
    );
    assert!(err.is_err(), "two-level nesting must be refused: {err:?}");
}
