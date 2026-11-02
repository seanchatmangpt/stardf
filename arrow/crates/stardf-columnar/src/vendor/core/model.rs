// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Minimal term model: `TermValue`, `TermBox`, `TermRef`, `BlankScope`,
//! `RdfTextDirection`, and `RdfLiteral` helpers.

use crate::vendor::hash::StructuredError;
use std::fmt;

/// Blank-node scope discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlankScope(pub u32);

/// RDF 1.2 base direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RdfTextDirection {
    /// Left-to-right.
    Ltr,
    /// Right-to-left.
    Rtl,
}

impl BlankScope {
    /// The columnar encoding id for the scope.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        self.0 as usize
    }
}

impl BlankScope {
    /// The default blank-node scope (columnar scope id 0).
    pub const DEFAULT: Self = Self(0);
}

impl RdfTextDirection {
    /// Columnar encoding id.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        match self {
            Self::Ltr => 0,
            Self::Rtl => 1,
        }
    }
}

/// An owned RDF 1.2 term value, canonical declaration order
/// (Iri < Literal < Blank < Triple).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TermValue {
    /// An IRI.
    Iri(String),
    /// A literal.
    Literal {
        /// Lexical form.
        lexical_form: String,
        /// Datatype IRI.
        datatype: String,
        /// Optional language tag.
        language: Option<String>,
        /// Optional base direction.
        direction: Option<RdfTextDirection>,
    },
    /// A blank node.
    Blank {
        /// Blank label.
        label: String,
        /// Blank scope.
        scope: BlankScope,
    },
    /// A quoted triple.
    Triple {
        /// Subject.
        s: TermBox,
        /// Predicate.
        p: TermBox,
        /// Object.
        o: TermBox,
    },
}

impl StructuredError for TermValue {
    fn visit<'error>(&'error self, visitor: &mut dyn crate::vendor::hash::ErrorVisitor<'error>) {
        use crate::vendor::hash::ErrorValue;
        match self {
            Self::Iri(iri) => {
                visitor.variant("term-iri");
                visitor.value(Some("value"), ErrorValue::Text(iri));
            }
            Self::Literal { lexical_form, .. } => {
                visitor.variant("term-literal");
                visitor.value(Some("lexical"), ErrorValue::Text(lexical_form));
            }
            Self::Blank { label, .. } => {
                visitor.variant("term-blank");
                visitor.value(Some("label"), ErrorValue::Text(label));
            }
            Self::Triple { .. } => visitor.variant("term-triple"),
        }
    }
}

impl fmt::Display for TermValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Iri(iri) => f.write_str(iri),
            Self::Literal { lexical_form, .. } => f.write_str(lexical_form),
            Self::Blank { label, .. } => write!(f, "_:{label}"),
            Self::Triple { .. } => f.write_str("<<...>>"),
        }
    }
}

/// A shared owned term (shim: clone-backed box; upstream is a refcounted
/// interned box from the kernel backend).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermBox(Box<TermValue>);

impl TermBox {
    /// Wrap an owned value.
    #[must_use]
    pub fn new(value: TermValue) -> Self {
        Self(Box::new(value))
    }

    /// Borrow the wrapped value.
    #[must_use]
    pub const fn value(&self) -> &TermValue {
        &self.0
    }
}

/// Borrowed view of an interned term, carrying source ids for recursion.
pub enum TermRef<'a, Id> {
    /// An IRI.
    Iri(&'a str),
    /// A literal; `datatype` is its datatype term's source id.
    Literal {
        /// Lexical form.
        lexical: &'a str,
        /// Datatype term id.
        datatype: Id,
        /// Optional language tag.
        language: Option<&'a str>,
        /// Optional base direction.
        direction: Option<RdfTextDirection>,
    },
    /// A blank node.
    Blank {
        /// Blank label.
        label: &'a str,
        /// Blank scope.
        scope: BlankScope,
    },
    /// A quoted triple of source ids.
    Triple {
        /// Subject id.
        s: Id,
        /// Predicate id.
        p: Id,
        /// Object id.
        o: Id,
    },
}

/// Guard granting borrowed term access.
pub trait TermGuard<Id> {
    /// The borrowed term.
    fn term(&self) -> TermRef<'_, Id>;
}

/// The literal datatype IRI determined by an optional base direction.
///
/// Delegates to the vocabulary home ([`super::vocab::language_datatype_iri`]);
/// upstream `purrdf_core::model::RdfLiteral::language_datatype_iri` has the
/// same shape over `purrdf_iri::vocab`.
#[must_use]
pub fn language_datatype_iri(direction: Option<RdfTextDirection>) -> &'static str {
    super::vocab::language_datatype_iri(direction.is_some())
}

/// Namespace alias kept for source parity with upstream `RdfLiteral`.
pub struct RdfLiteral;

impl RdfLiteral {
    /// The literal datatype IRI for a language literal with/without direction.
    #[must_use]
    pub fn language_datatype_iri(direction: Option<RdfTextDirection>) -> &'static str {
        super::vocab::language_datatype_iri(direction.is_some())
    }
}

/// Interning factory seam (upstream `purrdf_core::backend::TermFactory`).
///
/// Divergence disclosure (lane 9): upstream carries the full backend
/// interning algebra; the shim names the trait the reader imports and lets
/// implementors intern owned values.
pub trait TermFactory {
    /// Intern an owned term value.
    fn intern_value(
        &mut self, value: &TermValue,
    ) -> Result<u32, crate::vendor::core::RdfDiagnostic>;
}
