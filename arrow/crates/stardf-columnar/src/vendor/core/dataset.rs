// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The `DatasetView` seam, `QuadIds`, `ViewTermId`, a minimal `RdfDataset`,
//! and `LossLedger`.
//!
//! Divergence disclosure (lane 9): the builder performs reference-range and
//! partition checks only; upstream purrdf-core freeze() runs the full kernel
//! structural validator. The generic `DatasetView` boundary is preserved.
//!
//! Path-parity note (lane 16): [`RdfDatasetBuilder`] moved to
//! `crate::result` (the SPARQL result module); the re-export below keeps the
//! upstream `backend::RdfDatasetBuilder`-style path resolving unedited at
//! every existing call site.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::vendor::core::model::{TermBox, TermGuard, TermRef, TermValue};

// Lane 16 (2026-10-31): the builder definition now lives in `crate::result`;
// this path-parity re-export preserves `vendor::core::RdfDatasetBuilder`.
pub use crate::result::RdfDatasetBuilder;

/// One quad's source ids: subject, predicate, object, optional named graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadIds<Id> {
    /// Subject id.
    pub s: Id,
    /// Predicate id.
    pub p: Id,
    /// Object id.
    pub o: Id,
    /// Named-graph id, if any.
    pub g: Option<Id>,
}

/// A copyable, ordered term id from a backend.
pub trait ViewTermId: Copy + Ord + std::fmt::Debug {}
impl<T: Copy + Ord + std::fmt::Debug> ViewTermId for T {}

/// A read-side RDF dataset view.
pub trait DatasetView {
    /// Source term id type.
    type Id: ViewTermId;
    /// Source read failure.
    type ReadError;
    /// Borrowed term guard type.
    type TermGuard<'a>: TermGuard<Self::Id> + 'a
    where
        Self: 'a;

    /// Run `read` inside a completed-read boundary: the session refusals are
    /// carried whole (upstream shape).
    fn checked_read<T>(
        &self, read: impl FnOnce(&Self) -> Result<T, crate::ColumnarWriteError<Self::ReadError>>,
    ) -> Result<T, crate::ColumnarWriteError<Self::ReadError>>;

    /// Every base quad.
    fn quads(&self) -> impl Iterator<Item = QuadIds<Self::Id>>;
    /// Every reifier binding `(reifier, rdf:reifies, triple-term)`.
    fn reifier_quads(&self) -> impl Iterator<Item = QuadIds<Self::Id>>;
    /// Every statement annotation.
    fn annotation_quads(&self) -> impl Iterator<Item = QuadIds<Self::Id>>;
    /// Every declared named graph.
    fn named_graphs(&self) -> impl Iterator<Item = Self::Id>;
    /// Resolve one term id to a borrowed guard.
    fn resolve(&self, id: Self::Id) -> Result<Self::TermGuard<'_>, Self::ReadError>;
}

/// The reconstructed, validated dataset.
#[derive(Debug, Clone, Default)]
pub struct RdfDataset {
    terms: Vec<TermValue>,
    index: BTreeMap<TermValue, u32>,
    named_graphs: BTreeSet<u32>,
    quads: Vec<(u32, u32, u32, Option<u32>)>,
    reifiers: Vec<(u32, u32, Option<u32>)>,
    annotations: Vec<(u32, u32, u32, Option<u32>)>,
    /// Interned `rdf:reifies` id, minted at freeze when reifier bindings exist.
    reifier_predicate: Option<u32>,
}

impl RdfDataset {
    /// Every base quad's ids.
    pub fn quads(&self) -> impl Iterator<Item = QuadIds<u32>> + '_ {
        self.quads
            .iter()
            .map(|&(s, p, o, g)| QuadIds { s, p, o, g })
    }

    /// Every reifier binding's ids `(reifier, rdf:reifies, triple-term)`.
    pub fn reifier_quads(&self) -> impl Iterator<Item = QuadIds<u32>> + '_ {
        let predicate = self.reifier_predicate.unwrap_or(0);
        self.reifiers
            .iter()
            .map(move |&(reifier, triple, g)| QuadIds {
                s: reifier,
                p: predicate,
                o: triple,
                g,
            })
    }

    /// Every annotation's ids.
    pub fn annotation_quads(&self) -> impl Iterator<Item = QuadIds<u32>> + '_ {
        self.annotations
            .iter()
            .map(|&(s, p, o, gr)| QuadIds { s, p, o, g: gr })
    }

    /// Interned terms in id order.
    #[must_use]
    pub fn terms(&self) -> &[TermValue] {
        &self.terms
    }

    /// Declared named-graph term ids.
    pub fn named_graph_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.named_graphs.iter().copied()
    }

    /// Lane 16 disclosed in-module seam: the builder moved to `crate::result`
    /// (a sibling module), so its `freeze()` constructs the dataset through
    /// this crate-internal constructor instead of a struct literal. Not part
    /// of the public surface.
    #[must_use]
    pub(crate) fn from_parts(
        reifier_predicate: Option<u32>, terms: Vec<TermValue>, index: BTreeMap<TermValue, u32>,
        named_graphs: BTreeSet<u32>, quads: Vec<(u32, u32, u32, Option<u32>)>,
        reifiers: Vec<(u32, u32, Option<u32>)>, annotations: Vec<(u32, u32, u32, Option<u32>)>,
    ) -> Self {
        Self {
            reifier_predicate,
            terms,
            index,
            named_graphs,
            quads,
            reifiers,
            annotations,
        }
    }
}

/// Runtime losses observed while projecting or reconstructing.
///
/// Divergence disclosure (lane 9): the codec never records a loss on a
/// successful round trip, so the shim ledger is an empty-by-construction type.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LossLedger;

impl LossLedger {
    /// Whether no loss was recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        true
    }
}

/// The shim dataset is itself a `DatasetView` over its own u32 ids, so the
/// vendored codec can round-trip through one in-crate implementation.
impl DatasetView for RdfDataset {
    type Id = u32;
    type ReadError = core::convert::Infallible;
    type TermGuard<'a>
        = ShimTermGuard<'a>
    where
        Self: 'a;

    fn checked_read<T>(
        &self, read: impl FnOnce(&Self) -> Result<T, crate::ColumnarWriteError<Self::ReadError>>,
    ) -> Result<T, crate::ColumnarWriteError<Self::ReadError>> {
        read(self)
    }

    fn quads(&self) -> impl Iterator<Item = QuadIds<u32>> {
        <Self>::quads(self)
    }

    fn reifier_quads(&self) -> impl Iterator<Item = QuadIds<u32>> {
        <Self>::reifier_quads(self)
    }

    fn annotation_quads(&self) -> impl Iterator<Item = QuadIds<u32>> {
        <Self>::annotation_quads(self)
    }

    fn named_graphs(&self) -> impl Iterator<Item = u32> {
        // Named graphs are reported as ids; resolve() maps them to terms.
        self.named_graph_ids()
    }

    fn resolve(&self, id: u32) -> Result<ShimTermGuard<'_>, core::convert::Infallible> {
        // Guard contract: only minted ids (dense 1..=N) resolve; the expect is
        // a valid-input invariant, never an input gate.
        usize::try_from(id)
            .ok()
            .and_then(|row| row.checked_sub(1))
            .filter(|&row| row < self.terms.len())
            .expect("a minted id resolves");
        Ok(ShimTermGuard { dataset: self, id })
    }
}

/// A guard borrowing the dataset, resolving `TermRef` ids on demand.
pub struct ShimTermGuard<'a> {
    dataset: &'a RdfDataset,
    id: u32,
}

impl<'a> TermGuard<u32> for ShimTermGuard<'a> {
    fn term(&self) -> TermRef<'a, u32> {
        let id = self.id;
        match &self.dataset.terms[usize::try_from(id).ok().map_or(0, |i| i - 1)] {
            TermValue::Iri(iri) => TermRef::Iri(iri),
            TermValue::Literal {
                lexical_form,
                datatype,
                language,
                direction,
            } => {
                let datatype_id = self
                    .dataset
                    .index
                    .get(&TermValue::Iri(datatype.clone()))
                    .copied()
                    .unwrap_or(0);
                TermRef::Literal {
                    lexical: lexical_form,
                    datatype: datatype_id,
                    language: language.as_deref(),
                    direction: *direction,
                }
            }
            TermValue::Blank { label, scope } => TermRef::Blank {
                label,
                scope: *scope,
            },
            TermValue::Triple { s, p, o } => {
                // The shim stores triple terms with owned values; the view
                // seam needs ids, so look the triple up by value in `index`.
                let lookup = |box_ref: &TermBox| -> u32 {
                    self.dataset
                        .index
                        .get(box_ref.value())
                        .copied()
                        .unwrap_or(0)
                };
                TermRef::Triple {
                    s: lookup(s),
                    p: lookup(p),
                    o: lookup(o),
                }
            }
        }
    }
}
