// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! RDF 1.2 vocabulary constants (values verbatim from purrdf-iri vocab and
//! purrdf-xsd `datatype`).

/// The RDF namespace vocabulary.
pub mod rdf {
    /// `rdf:reifies`.
    pub const REIFIES: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies";
    /// `rdf:langString`.
    pub const LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";
    /// `rdf:dirLangString`.
    pub const DIR_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString";
}

/// The XML Schema datatype IRIs the codec names (values verbatim from
/// purrdf-xsd `datatype`, which keeps them beside the value space they name).
pub mod xsd {
    /// `xsd:integer`.
    pub const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
    /// `xsd:string`.
    pub const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
}

/// Internationalization Tag Set (ITS) 2.0 namespace
/// (`http://www.w3.org/2005/11/its`); `its:dir` carries an RDF 1.2 base
/// direction in RDF/XML and in the SPARQL XML results format.
///
/// Specification: <https://www.w3.org/TR/its20/>.
pub mod its {
    /// The namespace IRI: every term of this vocabulary starts with it.
    pub const NS: &str = "http://www.w3.org/2005/11/its";
}

/// The datatype RDF 1.2 gives a language-tagged string: `rdf:dirLangString`
/// when it carries a base direction, otherwise `rdf:langString`.
///
/// Upstream home: `purrdf_iri::vocab::language_datatype_iri`; the columnar
/// shim reaches it here instead of spelling either IRI a second time.
#[must_use]
pub const fn language_datatype_iri(has_direction: bool) -> &'static str {
    if has_direction {
        rdf::DIR_LANG_STRING
    } else {
        rdf::LANG_STRING
    }
}

#[cfg(test)]
mod tests {
    use super::{its, rdf, xsd};

    /// Namespace-plus-local-name parity: each term const equals its module's
    /// namespace concatenated with the local name (upstream vocab law).
    #[test]
    fn vocabulary_constants_are_namespace_plus_local_name() {
        assert_eq!(
            rdf::REIFIES,
            format!("{}reifies", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")
        );
        assert_eq!(
            rdf::LANG_STRING,
            format!(
                "{}langString",
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
            )
        );
        assert_eq!(
            rdf::DIR_LANG_STRING,
            format!(
                "{}dirLangString",
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
            )
        );
        assert_eq!(
            xsd::XSD_INTEGER,
            format!("{}integer", "http://www.w3.org/2001/XMLSchema#")
        );
        assert_eq!(
            xsd::XSD_STRING,
            format!("{}string", "http://www.w3.org/2001/XMLSchema#")
        );
        assert_eq!(its::NS, "http://www.w3.org/2005/11/its");
    }

    /// Direction dispatch matches RDF 1.2 Concepts §3.3 exactly.
    #[test]
    fn language_datatype_iri_dispatches_on_direction() {
        assert_eq!(super::language_datatype_iri(false), rdf::LANG_STRING);
        assert_eq!(super::language_datatype_iri(true), rdf::DIR_LANG_STRING);
    }
}
