// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! In-crate stand-in for the pieces of `purrdf-hash` structured-error surface
//! the columnar codec consumes: `StructuredError`, `ErrorValue`, `ErrorVisitor`,
//! `visit_list`, and the `error_visit!` impl generator (extracted verbatim from
//! purrdf rev 0da05783 with only `$crate` path substitutions).
//!
//! Divergence disclosure (lane 9): upstream carries a richer visitor protocol
//! (begin/end record, native upcasting, I/O category names). The codec only
//! IMPLEMENTS the trait via `error_visit!` and never drives a visitor, so this
//! minimal protocol preserves the vendored code's call shape.

use std::fmt;

impl std::fmt::Debug for ErrorValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // A debug view must not render a retained cause structurally.
        f.write_str("ErrorValue")
    }
}

/// An error, or a structured compound value carried by an error.
pub trait StructuredError {
    /// Report this value's variant and fields without rendering it to text.
    fn visit<'error>(&'error self, visitor: &mut dyn ErrorVisitor<'error>);
}

/// Receives one structured error and its fields.
pub trait ErrorVisitor<'error> {
    /// Stable identity of the current error or compound payload variant.
    fn variant(&mut self, code: &'static str);
    /// A named field or an unnamed list entry, preserving its primitive kind.
    fn value(&mut self, name: Option<&'static str>, value: ErrorValue<'error>);
    /// Begin a named list field or an unnamed nested list entry.
    fn begin_list(&mut self, name: Option<&'static str>, length: usize);
    /// End the most recently begun list.
    fn end_list(&mut self);
    /// Begin a compound view whose temporary adapter need not survive.
    fn begin_record(&mut self, name: Option<&'static str>, code: &'static str);
    /// End the most recently begun compound view.
    fn end_record(&mut self);
}

/// One structured field value, borrowed from its original owner.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum ErrorValue<'error> {
    /// An optional field is present but has no value.
    Absent,
    /// An exact unsigned integer.
    Unsigned(u128),
    /// An exact signed integer.
    Signed(i128),
    /// A Boolean condition.
    Boolean(bool),
    /// A machine-readable code or discriminant, distinct from prose.
    Code(&'error str),
    /// Text that is itself data.
    Text(&'error str),
    /// An IRI-valued field.
    Iri(&'error str),
    /// Original bytes, without an encoding or allocation.
    Bytes(&'error [u8]),
    /// A byte quantity, distinct from an unqualified integer.
    ByteCount(u128),
    /// A nested error retaining its own stable variant and fields.
    Cause(&'error dyn StructuredError),
    /// A compound non-error payload retaining its variant and fields.
    Record(&'error dyn StructuredError),
    /// A structured RDF term payload.
    Term(&'error dyn StructuredError),
}

/// Emit a named list field: streams the mapped values in order.
pub fn visit_list<'error, I>(
    visitor: &mut dyn ErrorVisitor<'error>, name: Option<&'static str>, values: I,
) where
    I: IntoIterator<Item = ErrorValue<'error>>,
{
    let values: Vec<_> = values.into_iter().collect();
    visitor.begin_list(name, values.len());
    for value in values {
        visitor.value(name, value);
    }
    visitor.end_list();
}

impl fmt::Display for ErrorValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absent => f.write_str("absent"),
            Self::Unsigned(value) => write!(f, "{value}"),
            Self::Signed(value) => write!(f, "{value}"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Code(value) | Self::Text(value) | Self::Iri(value) => f.write_str(value),
            Self::Bytes(_) => f.write_str("<bytes>"),
            Self::ByteCount(value) => write!(f, "{value}"),
            Self::Cause(_) | Self::Record(_) | Self::Term(_) => f.write_str("<record>"),
        }
    }
}

#[macro_export]
macro_rules! error_visit {
    (
        @tuple $this:ident ($($member:ident),* $(,)?) => $code:literal {
            $($field:literal => $kind:ident $(($value:expr))?),* $(,)?
        }
    ) => {
        fn visit<'error>(
            &'error $this,
            visitor: &mut dyn $crate::vendor::hash::ErrorVisitor<'error>,
        ) {
            // Bind the complete shape, as for named records and enum variants.
            // (Not every invocation's body reads every member by name — tuple
            // records address fields positionally — so bindings may go unused.)
            #[allow(unused_variables)]
            let Self($($member),*) = $this;
            visitor.variant($code);
            $($crate::error_visit!(@field visitor, $field, $kind $(($value))?);)*
        }
    };
    (
        @record $this:ident { $($member:ident),* $(,)? } => $code:literal {
            $($field:literal => $kind:ident $(($value:expr))?),* $(,)?
        }
    ) => {
        fn visit<'error>(
            &'error $this,
            visitor: &mut dyn $crate::vendor::hash::ErrorVisitor<'error>,
        ) {
            // Check the complete physical shape, while semantic accessors can
            // decode private representations without duplicating their logic.
            let Self { $($member: _),* } = $this;
            visitor.variant($code);
            $($crate::error_visit!(@field visitor, $field, $kind $(($value))?);)*
        }
    };
    (
        $(
            $variant:ident
            $({ $($named:ident),* $(,)? })?
            $(($($tuple:ident),* $(,)?))?
            => $code:literal {
                $($field:literal => $kind:ident $(($value:expr))?),* $(,)?
            }
        ),* $(,)?
    ) => {
        fn visit<'error>(
            &'error self,
            visitor: &mut dyn $crate::vendor::hash::ErrorVisitor<'error>,
        ) {
            match self {
                $(Self::$variant $({ $($named),* })? $(($($tuple),*))? => {
                    visitor.variant($code);
                    $($crate::error_visit!(@field visitor, $field, $kind $(($value))?);)*
                }),*
            }
        }
    };
    (@field $visitor:ident, $field:literal, List($value:expr)) => {
        $crate::vendor::hash::visit_list($visitor, Some($field), $value)
    };
    (@field $visitor:ident, $field:literal, Value($value:expr)) => {
        $visitor.value(Some($field), $value)
    };
    (@field $visitor:ident, $field:literal, $kind:ident $(($value:expr))?) => {
        $visitor.value(Some($field), $crate::vendor::hash::ErrorValue::$kind $(($value))?)
    };
}

impl StructuredError for std::num::TryFromIntError {
    fn visit<'error>(&'error self, visitor: &mut dyn ErrorVisitor<'error>) {
        visitor.variant("integer-conversion-range");
    }
}

impl StructuredError for std::str::Utf8Error {
    fn visit<'error>(&'error self, visitor: &mut dyn ErrorVisitor<'error>) {
        visitor.variant("std-utf8");
    }
}

impl StructuredError for core::convert::Infallible {
    fn visit<'error>(&'error self, _visitor: &mut dyn ErrorVisitor<'error>) {
        // Generic seams can name Infallible without ever carrying one.
    }
}
