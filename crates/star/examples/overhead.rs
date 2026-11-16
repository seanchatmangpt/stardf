//! Storage-overhead measurement: quoted-slot storage vs flat reified
//! storage over the same 66-quoted-triple corpus (60 flat + 6 nested, 2
//! metadata edges each on the flat ones, 1 on the nested ones).
use stardf_star::store::Store;
use stardf_star::term::{quote, Term, Triple};

fn main() {
    let mut store = Store::new();
    for si in 0..5 {
        let s = Term::Iri(format!("https://ex.test/s{}", si));
        for pi in 0..4 {
            let p = format!("https://ex.test/p{}", pi);
            for oi in 0..3 {
                let q = store.quote(s.clone(), &p, Term::Literal { value: format!("v{}-{}-{}", si, pi, oi), lang: None, datatype: None });
                store.add(Triple::new(q.clone(), "https://ex.test/score", Term::Literal { value: format!("{}", (si * 12 + pi * 3 + oi) % 100), lang: None, datatype: None }));
                store.add(Triple::new(q, "https://ex.test/source", Term::Iri("https://ex.test/feedA".into())));
            }
        }
    }
    for ni in 0..6 {
        let inner = quote(Term::Iri("https://ex.test/s0".into()), "https://ex.test/p0",
            Term::Literal { value: format!("v0-0-{}", ni), lang: None, datatype: None });
        let q = store.quote(inner, "https://ex.test/derivedFrom",
            Term::Literal { value: format!("n{}", ni), lang: None, datatype: None });
        store.add(Triple::new(q, "https://ex.test/score", Term::Literal { value: format!("{}", 90 + ni), lang: None, datatype: None }));
    }
    let n_stored = store.triples().len();
    let n_quoted = store.quoted_count();
    let flat_rows = store.flat_projection().len();
    // classical RDF reification would need, per quoted triple, 4 rows
    // (rdf:subject/predicate/object + rdf:type rdf:Statement) and metadata
    // re-pointed at the statement node; measure what our store holds and
    // what the flat projection needs for the SAME fact set.
    // metadata rows are ALL of n_stored here; classical reification needs
    // 4 rows per quoted triple plus the same metadata re-pointed at the
    // statement node.
    let classic = n_quoted * 4 + n_stored;
    println!("corpus: quoted_triples={} stored_rows={} flat_rows={} classic_reified_rows={}", n_quoted, n_stored, flat_rows, classic);
    println!("overhead_vs_ground_facts: stored={} ground=132 ratio={:.2}x", n_stored, n_stored as f64 / 132.0);
    println!("vs_ost_framing: quoted-slot={} classic-reified={} (OST claims ~400% for naive reification; ours is {} rows over the same fact set, i.e. {:.1}% of classic)", n_stored, classic, n_stored, 100.0 * n_stored as f64 / classic as f64);
    assert_eq!(store.verify_integrity(), 0);
}
