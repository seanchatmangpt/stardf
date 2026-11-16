// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! SEAM-SCOPED extract: the debug_non_exhaustive! macro only (upstream
//! crates/hash/src/impls.rs lines 35-50, rev 0da05783). Pending replacement
//! by full W1 stardf-hash.

#[macro_export]
macro_rules! debug_non_exhaustive {
    (
        $(#[$meta:meta])*
        $([$($generics:tt)*])? $name:ident $(<$($arg:tt),+>)? { $($field:ident),* $(,)? }
    ) => {
        impl $(<$($generics)*>)? ::core::fmt::Debug for $name $(<$($arg),+>)? {
            $(#[$meta])*
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(::core::stringify!($name))
                    $(.field(::core::stringify!($field), &self.$field))*
                    .finish_non_exhaustive()
            }
        }
    };
}

#[macro_export]
macro_rules! default_from_new {
    (
        $(#[$meta:meta])*
        $([$($generics:tt)*])? $name:ident $(<$($arg:tt),+>)? => $constructor:ident
    ) => {
        impl $(<$($generics)*>)? ::core::default::Default for $name $(<$($arg),+>)? {
            $(#[$meta])*
            fn default() -> Self {
                Self::$constructor()
            }
        }
    };
    (
        $(#[$meta:meta])*
        $([$($generics:tt)*])? $name:ident $(<$($arg:tt),+>)?
    ) => {
        $crate::default_from_new!(
            $(#[$meta])*
            $([$($generics)*])? $name $(<$($arg),+>)? => new
        );
    };
}
