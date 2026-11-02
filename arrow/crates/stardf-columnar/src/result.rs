// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The materialized SPARQL result model and the dataset builder it carries.
//!
//! Provenance (lane 16, 2026-10-31): [`SparqlResult`], [`DatasetHandle`], and
//! [`SolutionRow`] are extracted verbatim from purrdf rev
//! `0da0578306983da851a9073337d57e2e2c41ab3a` `crates/rdf-core/src/backend.rs`
//! (SolutionRow..`impl SparqlResult`, upstream lines 148-317) with only the
//! established module-path substitutions (`purrdf_hash::error` ->
//! `crate::vendor::hash`, per the vendor/hash.rs stand-in header).
//!
//! Divergence disclosure (lane 16, ESCALATED): upstream `DatasetHandle`
//! carries a second storage, `Native(purrdf_lex::allocation::SharedOwned<RdfDataset>)`,
//! with `allocation_layout`/`try_from_admitted`/`retains_admission` API. Those
//! are NOT vendored here: `SharedOwned`/`SharedAllocError` are absent from the
//! columnar `vendor/lex.rs` allocation stand-in, and minting a stand-in for an
//! exact-native-layout admission type would fake the one property it exists to
//! guarantee. Landing them is a non-rename extension of the allocation seam —
//! escalated to the coordinator per the vendoring law, not fixed silently.
//! `DatasetHandle` here is Caller-storage (`From<Arc<RdfDataset>>`) only.
//!
//! Addition disclosure (lane 16): upstream `impl StructuredError for
//! RdfDataset` lives in `ir/dataset/inspection.rs` and walks the full kernel
//! struct's private fields. The shim [`RdfDataset`] has a different (disclosed
//! by lane 9) private shape, so a verbatim impl is impossible; the impl below
//! reports the dataset variant plus row/term/graph counts through the shim's
//! public accessors only. It does not claim upstream's field-level fidelity.
//!
//! RdfDatasetBuilder moved here verbatim from `vendor/core/dataset.rs`
//! (lane 9's disclosed-divergence shim builder); the original module keeps a
//! path-parity re-export so upstream module paths still resolve.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

use crate::vendor::core::diagnostic::RdfDiagnostic;
use crate::vendor::core::model::{BlankScope, TermBox, TermValue};
use crate::vendor::core::RdfDataset;
use crate::vendor::hash::{ErrorVisitor, StructuredError};

/// One solution of a [`SparqlResult::Solutions`] sequence: the binding for each
/// projected variable, in projection order (`None` = unbound).
pub type SolutionRow = Vec<Option<TermValue>>;

/// Immutable frozen dataset ownership shared by resident and native results.
/// A native dataset retains its original storage admission through every clone;
/// borrowed access never exposes a lease-free SharedOwned pointer.
#[derive(Debug, Clone)]
pub struct DatasetHandle(DatasetHandleStorage);

#[derive(Debug, Clone)]
enum DatasetHandleStorage {
    Caller(Arc<RdfDataset>),
}

impl From<Arc<RdfDataset>> for DatasetHandle {
    fn from(value: Arc<RdfDataset>) -> Self {
        Self(DatasetHandleStorage::Caller(value))
    }
}

impl std::ops::Deref for DatasetHandle {
    type Target = RdfDataset;
    fn deref(&self) -> &Self::Target {
        match &self.0 {
            DatasetHandleStorage::Caller(value) => value,
        }
    }
}
impl AsRef<RdfDataset> for DatasetHandle {
    fn as_ref(&self) -> &RdfDataset {
        self
    }
}

/// Materialized SPARQL result model independent of any concrete query engine.
#[derive(Debug, Clone)]
pub enum SparqlResult {
    /// A SELECT solution sequence.
    Solutions {
        /// The projected variable names, in projection order.
        variables: Vec<String>,
        /// One row per solution; each cell is the binding for the variable at
        /// the same position in `variables` (`None` = unbound).
        rows: Vec<Vec<Option<TermValue>>>,
        /// Auxiliary quads invented during evaluation by value-constructing builtins
        /// (`purrdf:listSlice`/`purrdf:listConcat` mint fresh `rdf:List` cells). Empty for
        /// an ordinary SELECT. Carried so an in-process consumer can dereference a list
        /// head returned in a solution cell; W3C tabular result formats ignore it.
        aux: DatasetHandle,
    },
    /// A CONSTRUCT/DESCRIBE result, materialized as a frozen dataset.
    Graph(DatasetHandle),
    /// An ASK result.
    Boolean(bool),
}

impl StructuredError for SparqlResult {
    fn visit<'error>(&'error self, visitor: &mut dyn ErrorVisitor<'error>) {
        use crate::vendor::hash::{visit_list, ErrorValue};
        match self {
            Self::Solutions {
                variables,
                rows,
                aux,
            } => {
                visitor.variant("sparql-solutions");
                visit_list(
                    visitor,
                    Some("variables"),
                    variables.iter().map(|name| ErrorValue::Text(name)),
                );
                visitor.begin_list(Some("rows"), rows.len());
                for row in rows {
                    visit_list(
                        visitor,
                        None,
                        row.iter().map(|cell| {
                            cell.as_ref()
                                .map_or(ErrorValue::Absent, |value| ErrorValue::Term(value))
                        }),
                    );
                }
                visitor.end_list();
                visitor.value(Some("aux"), ErrorValue::Record(aux.as_ref()));
            }
            Self::Graph(graph) => {
                visitor.variant("sparql-graph");
                visitor.value(Some("graph"), ErrorValue::Record(graph.as_ref()));
            }
            Self::Boolean(value) => {
                visitor.variant("sparql-boolean");
                visitor.value(Some("value"), ErrorValue::Boolean(*value));
            }
        }
    }
}

impl SparqlResult {
    /// The query form this result answers, as the lowercase SPARQL keyword the result
    /// formats record it under: `select` for a solution sequence, `ask` for a boolean,
    /// `construct` for a graph (a DESCRIBE graph included, since it has the same shape).
    ///
    /// The one mapping from a result's shape to its query form: the SPARQL Results
    /// serializers' `queryForm` discriminator and every diagnostic that names a result's
    /// kind read it here.
    #[must_use]
    pub const fn query_form(&self) -> &'static str {
        match self {
            Self::Solutions { .. } => "select",
            Self::Boolean(_) => "ask",
            Self::Graph(_) => "construct",
        }
    }

    /// The projected variables and the rows of a solution sequence, or `None` for a
    /// graph or boolean result.
    #[must_use]
    pub fn solutions(&self) -> Option<(&[String], &[SolutionRow])> {
        match self {
            Self::Solutions {
                variables, rows, ..
            } => Some((variables, rows)),
            Self::Graph(_) | Self::Boolean(_) => None,
        }
    }

    /// The projected variables and the rows of a solution sequence, taken out of the
    /// result; any other result is handed back unchanged.
    ///
    /// # Errors
    ///
    /// Returns `self` when it is a graph or boolean result.
    pub fn into_solutions(self) -> Result<(Vec<String>, Vec<SolutionRow>), Self> {
        match self {
            Self::Solutions {
                variables, rows, ..
            } => Ok((variables, rows)),
            other @ (Self::Graph(_) | Self::Boolean(_)) => Err(other),
        }
    }
}

/// Builder for [`RdfDataset`]: interns terms, validates references.
///
/// Moved verbatim from `vendor/core/dataset.rs` (lane 9's disclosed-divergence
/// shim builder: freeze() performs reference-range and partition checks only;
/// upstream purrdf-core freeze() runs the full kernel structural validator).
#[derive(Debug, Default)]
pub struct RdfDatasetBuilder {
    terms: Vec<TermValue>,
    index: BTreeMap<TermValue, u32>,
    named_graphs: BTreeSet<u32>,
    quads: Vec<(u32, u32, u32, Option<u32>)>,
    reifiers: Vec<(u32, u32, Option<u32>)>,
    annotations: Vec<(u32, u32, u32, Option<u32>)>,
    // Carried verbatim from the lane-9 shim (upstream tracks freeze state for
    // the kernel validation the shim does not run); never read here.
    #[allow(dead_code)]
    frozen: bool,
}

impl RdfDatasetBuilder {
    /// A fresh builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn intern(&mut self, value: TermValue) -> Result<u32, RdfDiagnostic> {
        if let Some(&id) = self.index.get(&value) {
            return Ok(id);
        }
        let id = u32::try_from(self.terms.len() + 1)
            .map_err(|_| RdfDiagnostic::new("term-intern-capacity"))?;
        self.terms.push(value.clone());
        self.index.insert(value, id);
        Ok(id)
    }

    fn check(&self, id: u32) -> Result<(), RdfDiagnostic> {
        if id == 0 || usize::try_from(id).is_ok_and(|i| i > self.terms.len()) {
            return Err(RdfDiagnostic::new("term-reference-out-of-range"));
        }
        Ok(())
    }

    /// Intern a term value.
    pub fn intern_value(&mut self, value: &TermValue) -> Result<u32, RdfDiagnostic> {
        self.intern(value.clone())
    }

    /// Intern an IRI term.
    pub fn intern_iri(&mut self, iri: &str) -> Result<u32, RdfDiagnostic> {
        self.intern(TermValue::Iri(iri.to_owned()))
    }

    /// Intern a blank node.
    pub fn intern_blank(&mut self, label: &str, scope: BlankScope) -> Result<u32, RdfDiagnostic> {
        self.intern(TermValue::Blank {
            label: label.to_owned(),
            scope,
        })
    }

    /// Intern a quoted triple term.
    pub fn intern_triple(&mut self, s: u32, p: u32, o: u32) -> Result<u32, RdfDiagnostic> {
        self.check(s)?;
        self.check(p)?;
        self.check(o)?;
        let value = TermValue::Triple {
            s: TermBox::new(self.terms[usize::try_from(s).unwrap_or_default() - 1].clone()),
            p: TermBox::new(self.terms[usize::try_from(p).unwrap_or_default() - 1].clone()),
            o: TermBox::new(self.terms[usize::try_from(o).unwrap_or_default() - 1].clone()),
        };
        self.intern(value)
    }

    /// Declare a term as a named graph.
    pub fn declare_named_graph(&mut self, id: u32) -> Result<(), RdfDiagnostic> {
        self.check(id)?;
        self.named_graphs.insert(id);
        Ok(())
    }

    /// Push a base quad.
    pub fn push_quad(
        &mut self, s: u32, p: u32, o: u32, g: Option<u32>,
    ) -> Result<(), RdfDiagnostic> {
        self.check(s)?;
        self.check(p)?;
        self.check(o)?;
        if let Some(g) = g {
            self.check(g)?;
        }
        self.quads.push((s, p, o, g));
        Ok(())
    }

    /// Push a reifier binding in a graph.
    pub fn push_reifier_in_graph(
        &mut self, reifier: u32, triple: u32, g: Option<u32>,
    ) -> Result<(), RdfDiagnostic> {
        self.check(reifier)?;
        self.check(triple)?;
        if let Some(g) = g {
            self.check(g)?;
        }
        self.reifiers.push((reifier, triple, g));
        Ok(())
    }

    /// Push a statement annotation in a graph.
    pub fn push_annotation_in_graph(
        &mut self, reifier: u32, predicate: u32, value: u32, g: Option<u32>,
    ) -> Result<(), RdfDiagnostic> {
        self.check(reifier)?;
        self.check(predicate)?;
        self.check(value)?;
        if let Some(g) = g {
            self.check(g)?;
        }
        self.annotations.push((reifier, predicate, value, g));
        Ok(())
    }

    /// Push a base-graph reifier binding.
    pub fn push_reifier(&mut self, reifier: u32, triple: u32) -> Result<(), RdfDiagnostic> {
        self.push_reifier_in_graph(reifier, triple, None)
    }

    /// Push a base-graph statement annotation.
    pub fn push_annotation(
        &mut self, reifier: u32, predicate: u32, value: u32,
    ) -> Result<(), RdfDiagnostic> {
        self.push_annotation_in_graph(reifier, predicate, value, None)
    }

    /// Freeze into a validated dataset; mints `rdf:reifies` when reifier
    /// bindings exist (so `reifier_quads()` can name its fixed predicate).
    pub fn freeze(mut self) -> Result<Arc<RdfDataset>, RdfDiagnostic> {
        let reifier_predicate = if self.reifiers.is_empty() {
            None
        } else {
            Some(self.intern(TermValue::Iri(
                crate::vendor::core::vocab::rdf::REIFIES.to_owned(),
            ))?)
        };
        Ok(Arc::new(RdfDataset::from_parts(
            reifier_predicate,
            self.terms,
            self.index,
            self.named_graphs,
            self.quads,
            self.reifiers,
            self.annotations,
        )))
    }
}

impl StructuredError for RdfDataset {
    fn visit<'error>(&'error self, visitor: &mut dyn ErrorVisitor<'error>) {
        use crate::vendor::hash::ErrorValue;
        // Counts only, via the shim's public accessors: the upstream impl walks
        // the kernel struct's private field graph (see the module disclosure).
        visitor.variant("rdf-dataset");
        visitor.value(
            Some("terms"),
            ErrorValue::Unsigned(u128::try_from(self.terms().len()).unwrap_or(u128::MAX)),
        );
        let quad_count = self.quads().count();
        visitor.value(
            Some("quads"),
            ErrorValue::Unsigned(u128::try_from(quad_count).unwrap_or(u128::MAX)),
        );
        let reifier_count = self.reifier_quads().count();
        visitor.value(
            Some("reifiers"),
            ErrorValue::Unsigned(u128::try_from(reifier_count).unwrap_or(u128::MAX)),
        );
        let annotation_count = self.annotation_quads().count();
        visitor.value(
            Some("annotations"),
            ErrorValue::Unsigned(u128::try_from(annotation_count).unwrap_or(u128::MAX)),
        );
        let graph_count = self.named_graph_ids().count();
        visitor.value(
            Some("named_graphs"),
            ErrorValue::Unsigned(u128::try_from(graph_count).unwrap_or(u128::MAX)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{DatasetHandle, RdfDatasetBuilder, SolutionRow, SparqlResult};
    use crate::vendor::core::model::{BlankScope, TermValue};
    use crate::vendor::hash::{ErrorValue, ErrorVisitor, StructuredError};
    use std::sync::Arc;

    /// A real visitor: records the driven protocol as final state, which the
    /// assertions below check. Not a mock — it exercises the same trait surface
    /// a serializer's structured-error driver implements.
    #[derive(Debug, Default)]
    struct RecordingVisitor {
        variants: Vec<&'static str>,
        values: Vec<(Option<&'static str>, String)>,
        list_depth: usize,
        max_list_depth: usize,
    }

    impl ErrorVisitor<'_> for RecordingVisitor {
        fn variant(&mut self, code: &'static str) {
            self.variants.push(code);
        }
        fn value(&mut self, name: Option<&'static str>, value: ErrorValue<'_>) {
            self.values.push((name, value.to_string()));
        }
        fn begin_list(&mut self, name: Option<&'static str>, length: usize) {
            self.list_depth += 1;
            self.max_list_depth = self.max_list_depth.max(self.list_depth);
            self.values.push((name, format!("[begin len={length}]")));
        }
        fn end_list(&mut self) {
            self.list_depth = self.list_depth.saturating_sub(1);
        }
        fn begin_record(&mut self, name: Option<&'static str>, code: &'static str) {
            self.values.push((name, format!("record:{code}")));
        }
        fn end_record(&mut self) {}
    }

    fn one_quad_dataset() -> Arc<crate::vendor::core::RdfDataset> {
        let mut builder = RdfDatasetBuilder::new();
        let s = builder.intern_iri("https://example.org/s").unwrap();
        let p = builder.intern_iri("https://example.org/p").unwrap();
        let o = builder.intern_iri("https://example.org/o").unwrap();
        builder.push_quad(s, p, o, None).unwrap();
        builder.freeze().unwrap()
    }

    #[test]
    fn query_form_matches_result_shape() {
        let aux = DatasetHandle::from(one_quad_dataset());
        let solutions = SparqlResult::Solutions {
            variables: vec!["x".to_owned()],
            rows: vec![vec![Some(TermValue::Iri(
                "https://example.org/o".to_owned(),
            ))]],
            aux: aux.clone(),
        };
        let graph = SparqlResult::Graph(DatasetHandle::from(one_quad_dataset()));
        let boolean = SparqlResult::Boolean(true);
        assert_eq!(solutions.query_form(), "select");
        assert_eq!(graph.query_form(), "construct");
        assert_eq!(boolean.query_form(), "ask");
    }

    #[test]
    fn solutions_accessors_round_trip_and_refuse_other_forms() {
        let row: SolutionRow = vec![
            Some(TermValue::Iri("https://example.org/o".to_owned())),
            None,
        ];
        let result = SparqlResult::Solutions {
            variables: vec!["x".to_owned(), "y".to_owned()],
            rows: vec![row.clone()],
            aux: DatasetHandle::from(one_quad_dataset()),
        };
        let (variables, rows) = result.solutions().expect("a solution sequence");
        assert_eq!(variables, ["x", "y"]);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0][0].as_ref(),
            Some(&TermValue::Iri("https://example.org/o".to_owned()))
        );
        assert!(rows[0][1].as_ref().is_none());

        let taken = SparqlResult::Solutions {
            variables: vec!["x".to_owned()],
            rows: vec![row],
            aux: DatasetHandle::from(one_quad_dataset()),
        };
        let (variables, rows) = taken.into_solutions().expect("solutions taken out");
        assert_eq!(variables, ["x"]);
        assert_eq!(rows.len(), 1);

        let boolean = SparqlResult::Boolean(false);
        assert!(boolean.solutions().is_none());
        let refused = boolean
            .into_solutions()
            .expect_err("boolean is handed back");
        assert!(matches!(refused, SparqlResult::Boolean(false)));
    }

    #[test]
    fn structured_visit_drives_the_full_protocol() {
        let result = SparqlResult::Solutions {
            variables: vec!["x".to_owned()],
            rows: vec![vec![
                Some(TermValue::Iri("https://example.org/o".to_owned())),
                None,
            ]],
            aux: DatasetHandle::from(one_quad_dataset()),
        };
        let mut visitor = RecordingVisitor::default();
        result.visit(&mut visitor);
        assert_eq!(visitor.variants, ["sparql-solutions"]);
        assert_eq!(
            visitor.values.first().map(|(name, _)| *name),
            Some(Some("variables"))
        );
        // rows list was begun and ended; its cells carry bound then unbound.
        let rendered: Vec<_> = visitor
            .values
            .iter()
            .map(|(_, text)| text.as_str())
            .collect();
        assert!(rendered.contains(&"[begin len=1]"));
        assert!(rendered.contains(&"absent"));
        assert!(visitor.list_depth == 0 && visitor.max_list_depth >= 1);

        let graph_dataset = one_quad_dataset();
        let graph = SparqlResult::Graph(DatasetHandle::from(Arc::clone(&graph_dataset)));
        let mut visitor = RecordingVisitor::default();
        graph.visit(&mut visitor);
        // The graph variant hands the dataset to the visitor as a nested
        // record value; expanding it is the DRIVER's choice (the visitor
        // receives `ErrorValue::Record(&dyn StructuredError)`).
        assert_eq!(visitor.variants, ["sparql-graph"]);
        assert!(visitor
            .values
            .iter()
            .any(|(name, text)| *name == Some("graph") && text == "<record>"));

        // Driving the dataset's own StructuredError surfaces the real counts.
        let mut visitor = RecordingVisitor::default();
        StructuredError::visit(graph_dataset.as_ref(), &mut visitor);
        assert_eq!(visitor.variants, ["rdf-dataset"]);
        assert!(visitor
            .values
            .iter()
            .any(|(name, text)| *name == Some("quads") && text == "1"));
        assert!(visitor
            .values
            .iter()
            .any(|(name, text)| *name == Some("terms") && text == "3"));

        let boolean = SparqlResult::Boolean(true);
        let mut visitor = RecordingVisitor::default();
        boolean.visit(&mut visitor);
        assert_eq!(visitor.variants, ["sparql-boolean"]);
        assert!(visitor
            .values
            .iter()
            .any(|(name, text)| *name == Some("value") && text == "true"));
    }

    #[test]
    fn dataset_handle_derefs_to_the_frozen_dataset() {
        let handle = DatasetHandle::from(one_quad_dataset());
        assert_eq!(handle.quads().count(), 1);
        assert_eq!(handle.as_ref().quads().count(), 1);
        // s, p, o interned; rdf:reifies is minted only when reifier bindings exist.
        assert_eq!(handle.terms().len(), 3);
    }

    #[test]
    fn builder_refuses_out_of_range_references() {
        let mut builder = RdfDatasetBuilder::new();
        assert!(builder.push_quad(0, 1, 1, None).is_err());
        assert!(builder.push_quad(1, 1, 1, None).is_err()); // nothing interned yet
        let s = builder.intern_iri("https://example.org/s").unwrap();
        assert!(builder.declare_named_graph(99).is_err());
        assert!(builder.intern_blank("b1", BlankScope::DEFAULT).is_ok());
        let _ = s;
    }
}
