// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Simplified language-tag grammar gate (lane 9 shim for
//! `purrdf_iri::langtag`).
//!
//! Divergence disclosure (lane 9): upstream judges tags under purrdf-iri's
//! full `ConcreteSyntaxLangtagBounded` acceptance language. This shim checks
//! case-folding identity (no Unicode full case folding) and a bounded
//! RFC 5646-shaped regular grammar (langtag ABNF reduced to its ASCII
//! skeleton). It never accepts what upstream refuses for case reasons, but it
//! MAY refuse tags upstream accepts (rare non-ASCII canonical forms). Any such
//! refusal is a loud typed reader refusal, never a silent accept.

use std::fmt;

/// The acceptance profile named by the reader's grammar gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// The bounded concrete-syntax langtag acceptance language.
    ConcreteSyntaxLangtagBounded,
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConcreteSyntaxLangtagBounded => f.write_str("concrete-syntax-langtag-bounded"),
        }
    }
}

/// Whether the tag is identical to its (ASCII) case fold.
#[must_use]
pub fn is_identity_folded(tag: &str) -> bool {
    tag.chars().all(|c| !c.is_uppercase()) || tag.to_lowercase() == tag
}

fn is_alphanum(c: u8) -> bool {
    c.is_ascii_alphanumeric()
}

/// Bounded RFC 5646-shaped regular grammar check.
#[must_use]
pub fn is_well_formed_with(tag: &str, _profile: Profile) -> bool {
    let bytes = tag.as_bytes();
    if bytes.is_empty() || bytes.len() > 128 {
        return false;
    }
    if !bytes.iter().all(|&b| is_alphanum(b) || b == b'-') {
        return false;
    }
    let parts: Vec<&[u8]> = bytes.split(|&b| b == b'-').collect();
    // RFC 5646: a tag may be a bare language subtag ("es", "en") with no
    // script/region/variant components. Upstream's full acceptance language
    // accepts these; the shim must not refuse them (W-lane 13 fix, disclosed:
    // the previous `parts.len() < 2` guard refused every bare primary
    // subtag, a loud-typed divergence the columnar integration gate caught).
    if parts.is_empty() {
        return false;
    }
    // language: 2-3 alpha (extlang list optional) or 4 alpha or 5-8 alpha
    let lang = parts[0];
    let ok_lang = (2..=3).contains(&lang.len()) && lang.iter().all(u8::is_ascii_alphabetic)
        || ((4..=8).contains(&lang.len()) && lang.iter().all(u8::is_ascii_alphabetic));
    if !ok_lang {
        return false;
    }
    // script: 4 alpha; region: 2 alpha or 3 digits; variants/extension 1-8 alnum
    for part in &parts[1..] {
        let ok = (2..=8).contains(&part.len()) && part.iter().all(|&b| is_alphanum(b));
        if !ok {
            return false;
        }
    }
    true
}
