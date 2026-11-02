// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `RdfDiagnostic` shim.
//!
//! Divergence disclosure (lane 9): upstream `RdfDiagnostic` carries the full
//! kernel diagnostic algebra (attachments, ledger links). The shim keeps the
//! stable code string plus the optional storage cause the codec's Display
//! path consults.

use std::fmt;

use crate::vendor::hash::StructuredError;
use crate::vendor::lex::allocation::StorageError;

/// A kernel reconstruction diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RdfDiagnostic {
    code: String,
    storage: Option<StorageError>,
}

impl RdfDiagnostic {
    /// A diagnostic with a stable code.
    #[must_use]
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            storage: None,
        }
    }

    /// A diagnostic wrapping a storage refusal.
    #[must_use]
    pub fn from_storage(error: StorageError) -> Self {
        Self {
            code: "storage-refused".to_owned(),
            storage: Some(error),
        }
    }

    /// The stable diagnostic code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The storage cause, when the refusal originated below the kernel.
    #[must_use]
    pub fn storage_error(&self) -> Option<&StorageError> {
        self.storage.as_ref()
    }
}

impl fmt::Display for RdfDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rdf reconstruction refused: {}", self.code)
    }
}

impl std::error::Error for RdfDiagnostic {}

impl StructuredError for RdfDiagnostic {
    fn visit<'error>(&'error self, visitor: &mut dyn crate::vendor::hash::ErrorVisitor<'error>) {
        use crate::vendor::hash::ErrorValue;
        visitor.variant("rdf-diagnostic");
        visitor.value(Some("code"), ErrorValue::Text(&self.code));
        if let Some(storage) = &self.storage {
            visitor.value(Some("cause"), ErrorValue::Cause(storage));
        }
    }
}
