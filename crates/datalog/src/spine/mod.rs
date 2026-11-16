// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! # The spine: a fixed-width, zero-alloc forward-chaining evaluator
//!
//! DIFFERENTIATING MECHANISM — "Stratified Delta-Page Fixpoint" (SDPF).
//! Bounded iteration to fixpoint with PER-STRATUM DELTA PAGES: append-only,
//! fixed-width ([u64; 4]) fact buffers per derivation stratum. Evaluation is
//! single-writer-per-stratum by construction (the fixpoint driver owns the
//! stratum's pages outright for the duration of that stratum's iteration
//! loop; no other stratum touches them until its own turn). There are no
//! locks, and there are no in-memory iterators over shared indices.
//!
//! CITED NON-PRACTICE (what this spine does NOT practice):
//! - No lock-free shared-memory parallel materialization (OST P1's mechanism).
//! - No runtime SIP-style join planning. The IN-TREE INCUMBENT is the vendored
//!   planner: `src/plan.rs` (adornment antichains, plan.rs:36) and
//!   `src/demand.rs` (demand propagation, demand.rs:355). This spine does
//!   static, fixed body-order evaluation; plan.rs is the read-only reference
//!   incumbent and is never modified by the spine.
//! - No string allocation on the evaluation path: terms are caller-interned
//!   u64 ids compatible with stardf's fixed-width term pages.
//!
//! Termination: each stratum iterates under a caller-visible cap; hitting the
//! cap is a typed [`SpineError::BoundedRefusal`] naming the stratum and the
//! iteration at which it was hit — limits REFUSE, they never truncate.
//! Capacity: every page is pre-reserved at program build; exceeding it is a
//! typed [`SpineError::CapacityExceeded`].
//!
//! Structure: [`rule`] (fixed-width atoms/rules), [`stratify`] (stratum graph),
//! [`eval`] (the SDPF driver), [`corpus`] (seeded court corpus), [`refeval`]
//! (naive reference evaluator for the differential court).

pub mod corpus;
pub mod eval;
pub mod refeval;
pub mod rule;
pub mod stratify;

pub use eval::{EvalOptions, SpineEval};
pub use rule::{Atom, Rule, SpineProgram, SpineProgramBuilder, ARITY, PAD, VAR_TAG, WILD};
pub use stratify::{Stratification, StratificationRefusal, MAX_STRATA};
pub use eval::SpineError;
