// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! In-crate stand-in for the pieces of `purrdf-lex` the columnar codec
//! consumes: the `error_source!` / `variant_from!` macros (extracted verbatim
//! from purrdf rev 0da05783, only `$crate` paths substituted) and an
//! `allocation::StorageError` shim.
//!
//! Divergence disclosure (lane 9): upstream `StorageError::Capacity` carries a
//! `CapacityError` payload from the allocator seam with u128 fields; this shim
//! keeps the same variant shape with a simplified payload sufficient for the
//! codec's LimitExceeded-class refusals.

/// Minimal allocation-error types (lane 9 shim for purrdf-lex allocation).
pub mod allocation {
    use crate::vendor::hash::StructuredError;
    use std::fmt;

    /// A logical capacity or physical storage refusal.
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum StorageError {
        /// A logical address space refused the requested count before mutation.
        Capacity(CapacityError),
        /// A concrete allocation layout cannot be represented.
        SizeOverflow,
        /// A fallible allocation refused its already admitted destination.
        AllocationFailed,
        /// The caller's admission callback refused; it retains its typed error.
        AdmissionFailed,
        /// A native Display implementation failed without a storage refusal.
        FormattingFailed,
    }

    /// Observed value exceeds a logical capacity bound.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CapacityError {
        /// Observed value.
        pub value: u128,
        /// Maximum accepted value.
        pub maximum: u128,
    }

    impl fmt::Display for CapacityError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "capacity refused: {} exceeds {}",
                self.value, self.maximum
            )
        }
    }

    impl fmt::Display for StorageError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Capacity(error) => error.fmt(f),
                Self::SizeOverflow => f.write_str("diverge: layout overflow"),
                Self::AllocationFailed => f.write_str("native destination allocation failed"),
                Self::AdmissionFailed => f.write_str("native workspace admission refused"),
                Self::FormattingFailed => f.write_str("native diagnostic formatting failed"),
            }
        }
    }

    impl std::error::Error for StorageError {}

    impl StructuredError for CapacityError {
        fn visit<'error>(
            &'error self, visitor: &mut dyn crate::vendor::hash::ErrorVisitor<'error>,
        ) {
            use crate::vendor::hash::ErrorValue;
            visitor.variant("storage-capacity-error");
            visitor.value(Some("value"), ErrorValue::Unsigned(self.value));
            visitor.value(Some("maximum"), ErrorValue::Unsigned(self.maximum));
        }
    }

    impl crate::vendor::hash::StructuredError for StorageError {
        crate::error_visit! {
            Capacity(error) => "storage-capacity" { "cause" => Cause(error) },
            SizeOverflow => "storage-size-overflow" {},
            AllocationFailed => "storage-allocation-failed" {},
            AdmissionFailed => "storage-admission-failed" {},
            FormattingFailed => "storage-formatting-failed" {},
        }
    }
}

#[macro_export]
macro_rules! error_source {
    ($type:ty, $field:ident) => {
        impl ::core::error::Error for $type {
            fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)> {
                Some(&self.$field)
            }
        }
    };
    ($type:ty { $($variant:ident),+ $(,)? }) => {
        $crate::error_source!($type => { $(Self::$variant(error) => error),+ });
    };
    ($type:ty => { $($pattern:pat => $error:expr),+ $(,)? }) => {
        impl ::core::error::Error for $type {
            fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)> {
                match self {
                    $($pattern => Some($error),)+
                    _ => None,
                }
            }
        }
    };
}

#[macro_export]
macro_rules! variant_from {
    (impl [$($parameters:tt)*] $type:ty where [$($bounds:tt)*] { $($items:tt)* }) => {
        $crate::variant_from!(@variants [<$($parameters)*>] [where $($bounds)*] $type { $($items)* });
    };
    (impl <$($parameter:ident),+ $(,)?> $type:ty { $($items:tt)* }) => {
        $crate::variant_from!(@variants [<$($parameter),+>] [] $type { $($items)* });
    };
    ($type:ty { $($items:tt)* }) => {
        $crate::variant_from!(@variants [] [] $type { $($items)* });
    };
    (@variants $parameters:tt $bounds:tt $type:ty {
        $variant:ident($($source:ty),+ $(,)?) via $convert:path
    }) => {
        $(
            $crate::variant_from!(@owned $parameters $bounds $type {
                $variant($source) via $convert
            });
        )*
    };
    (@variants $parameters:tt $bounds:tt $type:ty {
        $variant:ident($($source:ty),+ $(,)?) as $convert:path
    }) => {
        $(
            $crate::variant_from!(@render $parameters $bounds $type {
                $variant($source) as $convert
            });
        )+
    };
    (@variants $parameters:tt $bounds:tt $type:ty {
        $($variant:ident($source:ty)),+ $(,)?
    }) => {
        $(
            $crate::variant_from!(@wrap $parameters $bounds $type { $variant($source) });
        )+
    };
    (@owned [$($parameters:tt)*] [$($bounds:tt)*] $type:ty {
        $variant:ident($source:ty) via $convert:path
    }) => {
        impl $($parameters)* ::core::convert::From<$source> for $type $($bounds)* {
            fn from(value: $source) -> Self {
                Self::$variant($convert(value))
            }
        }
    };
    (@render [$($parameters:tt)*] [$($bounds:tt)*] $type:ty {
        $variant:ident($source:ty) as $convert:path
    }) => {
        impl $($parameters)* ::core::convert::From<$source> for $type $($bounds)* {
            fn from(value: $source) -> Self {
                Self::$variant($convert(&value))
            }
        }
    };
    (@wrap [$($parameters:tt)*] [$($bounds:tt)*] $type:ty { $variant:ident($source:ty) }) => {
        impl $($parameters)* ::core::convert::From<$source> for $type $($bounds)* {
            fn from(value: $source) -> Self {
                Self::$variant(value)
            }
        }
    };
}
