// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Seam: purrdf-events TextDirection surface needed by the stardf term-model slice.
//! The TextDirection block is VERBATIM from purrdf rev 0da05783
//! (crates/rdf-events/src/lib.rs lines 214-260). Pending replacement by full W1 stardf-events.

/// RDF 1.2 base direction for directional language-tagged literals.
///
/// The stack's one direction type: it lives in this protocol crate, the lowest
/// layer that carries a direction, and every layer above names it —
/// `purrdf_cdt::TextDirection` and `purrdf-core`'s `RdfTextDirection` are this
/// type — so an event's direction enters the IR, and leaves it again, as is.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TextDirection {
    /// Left-to-right.
    Ltr,
    /// Right-to-left.
    Rtl,
}

impl TextDirection {
    /// The lowercase direction token (`"ltr"` or `"rtl"`) as it appears in
    /// concrete syntaxes (the `--ltr` / `--rtl` suffix after a language tag).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }

    /// Read the RDF 1.2 base-direction token (`ltr` or `rtl`, the exact
    /// lowercase spelling RDF 1.2 Concepts §3.3 gives), or `None` for any
    /// other text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use purrdf_events::TextDirection;
    ///
    /// assert_eq!(TextDirection::from_str_token("rtl"), Some(TextDirection::Rtl));
    /// assert_eq!(TextDirection::from_str_token("RTL"), None);
    /// assert_eq!(TextDirection::Ltr.as_str(), "ltr");
    /// ```
    #[must_use]
    pub fn from_str_token(token: &str) -> Option<Self> {
        match token {
            "ltr" => Some(Self::Ltr),
            "rtl" => Some(Self::Rtl),
            _ => None,
        }
    }
}
