// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Path-parity shim for `purrdf_testkit`'s term fixture helpers, consumed by
//! the preserved upstream test modules (behind `--features upstream-tests`).
//!
//! Divergence disclosure (lane 1, 2026-10-31): upstream `term_fixture`
//! generates the full term grammar over the kernel fixture algebra; this shim
//! implements the consumed slice only — recursive triple-chain generation
//! driven by an injected splitmix64 draw function and a depth budget.

use super::model::{BlankScope, TermBox, TermValue};

/// Shapes the fixture generator may produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TermShape {
    /// Terms whose literal datatypes resolve to IRI terms.
    WellFormed,
}

/// Generate one term value from a splitmix64 state stream.
///
/// `rng` is the injected draw function (upstream signature parity: the caller
/// passes the stream so tests can substitute a fixed one); `budget` bounds
/// triple nesting depth.
pub fn term_value<R>(state: &mut u64, rng: &R, budget: &mut usize, shape: TermShape) -> TermValue
where
    R: Fn(&mut u64) -> u64,
{
    let _ = shape;
    let draw = rng(state);
    if *budget > 0 && draw % 4 == 3 {
        *budget -= 1;
        let mut child_budget = 0;
        let s = term_value(state, rng, &mut child_budget, shape);
        let p = term_value(state, rng, &mut child_budget, shape);
        let o = term_value(state, rng, &mut child_budget, shape);
        return TermValue::Triple {
            s: TermBox::new(s),
            p: TermBox::new(p),
            o: TermBox::new(o),
        };
    }
    match draw % 3 {
        0 => TermValue::Iri(format!("http://example.org/gen/{draw}")),
        1 => TermValue::Literal {
            lexical_form: format!("{draw}"),
            datatype: crate::vendor::core::vocab::xsd::XSD_INTEGER.to_owned(),
            language: None,
            direction: None,
        },
        _ => TermValue::Blank {
            label: format!("gen{draw}"),
            scope: BlankScope::DEFAULT,
        },
    }
}

/// The recursive oracle fixture: a `Triple` chain of exactly `levels` nested
/// quoted triples whose subject/predicate are fixed IRIs and whose object
/// spine recurses, matching the preserved reader test's hand-built record
/// table (`s: 0, p: 1, o: level + 2` with leaves the s/p/o IRIs).
#[must_use]
pub fn triple_chain(levels: usize) -> TermValue {
    if levels == 0 {
        return TermValue::Iri("http://example.org/o".to_owned());
    }
    TermValue::Triple {
        s: TermBox::new(TermValue::Iri("http://example.org/s".to_owned())),
        p: TermBox::new(TermValue::Iri("http://example.org/p".to_owned())),
        o: TermBox::new(triple_chain(levels - 1)),
    }
}
