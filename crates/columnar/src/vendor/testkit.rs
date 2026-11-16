// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Path-parity shim for the consumed slice of `purrdf-testkit`: the
//! splitmix64 RNG and the structured error-capture surface, consumed by the
//! preserved upstream test modules (behind `--features upstream-tests`).
//!
//! Divergence disclosure (lane 1, 2026-10-31): upstream testkit is a full
//! test harness (its `harness_main!` runs suites over real storage); this
//! shim implements only the consumed slice.

/// The splitmix64 RNG.
pub mod rng {
    /// A `SplitMix64` generator (golden gamma 0x9E3779B97F4A7C15).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SplitMix64 {
        state: u64,
    }

    impl SplitMix64 {
        /// Seed from any u64.
        #[must_use]
        pub const fn new(seed: u64) -> Self {
            Self { state: seed }
        }

        /// The next raw 64-bit draw.
        #[must_use]
        pub fn next_u64(&mut self) -> u64 {
            splitmix64_next(&mut self.state)
        }

        /// A scaled draw in `0..bound` (the preserved tests use it only for
        /// shape selection, not bit-exact replay).
        #[must_use]
        pub fn below_usize(&mut self, bound: usize) -> usize {
            if bound == 0 {
                return 0;
            }
            (self.next_u64() % bound as u64) as usize
        }
    }

    /// One splitmix64 step over a bare state word (the free-function form the
    /// fixture generator injects).
    #[must_use]
    pub fn splitmix64_next(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// Structured error capture over the visitor in `vendor::hash`.
pub mod error_capture {
    use crate::vendor::hash::{ErrorValue, ErrorVisitor, StructuredError};

    /// One recorded visitor event.
    pub enum Event<'e> {
        /// The variant code of the current record.
        Variant(&'static str),
        /// A named or unnamed field value.
        Value(Option<&'static str>, ErrorValue<'e>),
    }

    /// The captured view of one structured error: its variant code plus the
    /// flattened field list of its own `visit` report. Nested causes are
    /// reached through [`Capture::child`], which re-captures the nested
    /// error in place.
    pub struct Capture<'e> {
        /// The variant code of the captured report.
        pub code: &'e str,
        fields: Vec<(Option<&'static str>, ErrorValue<'e>)>,
    }

    impl<'e> Capture<'e> {
        /// Capture a structured error's report with a fresh event buffer.
        pub fn read(error: &dyn StructuredError) -> Capture<'_> {
            Self::read_dyn(error)
        }

        /// Capture a structured error's report, reusing a caller-provided
        /// event buffer as storage (upstream signature parity).
        pub fn read_with_buffer<'a>(
            error: &'a dyn StructuredError, buffer: Vec<Event<'a>>,
        ) -> Capture<'a> {
            let _ = buffer;
            Self::read_dyn(error)
        }

        fn read_dyn(error: &'_ dyn StructuredError) -> Capture<'_> {
            let mut code: &'_ str = "";
            let mut fields: Vec<(Option<&'static str>, ErrorValue<'_>)> = Vec::new();
            struct Recorder<'c, 'f, 'e> {
                code: &'c mut &'e str,
                fields: &'f mut Vec<(Option<&'static str>, ErrorValue<'e>)>,
                seen: bool,
            }
            impl<'e> ErrorVisitor<'e> for Recorder<'_, '_, 'e> {
                fn variant(&mut self, code: &'static str) {
                    if !self.seen {
                        *self.code = code;
                        self.seen = true;
                    }
                }
                fn value(&mut self, name: Option<&'static str>, value: ErrorValue<'e>) {
                    self.fields.push((name, value));
                }
                fn begin_list(&mut self, name: Option<&'static str>, length: usize) {
                    self.fields
                        .push((name, ErrorValue::Unsigned(length as u128)));
                }
                fn end_list(&mut self) {}
                fn begin_record(&mut self, _name: Option<&'static str>, code: &'static str) {
                    if !self.seen {
                        *self.code = code;
                        self.seen = true;
                    }
                }
                fn end_record(&mut self) {}
            }
            let mut recorder = Recorder {
                code: &mut code,
                fields: &mut fields,
                seen: false,
            };
            error.visit(&mut recorder);
            Capture { code, fields }
        }

        /// The last recorded value under `name` (or [`ErrorValue::Absent`]).
        #[must_use]
        pub fn field(&self, name: &str) -> ErrorValue<'e> {
            for (field_name, value) in self.fields.iter().rev() {
                if let Some(field_name) = field_name {
                    if *field_name == name {
                        return *value;
                    }
                }
            }
            ErrorValue::Absent
        }

        /// Re-capture the nested cause or record recorded under `name`.
        #[must_use]
        pub fn child(&self, name: &str) -> Capture<'e> {
            for (field_name, value) in self.fields.iter().rev() {
                if let Some(field_name) = field_name {
                    if *field_name == name {
                        match value {
                            ErrorValue::Cause(nested) | ErrorValue::Record(nested) => {
                                return Self::read_dyn(*nested);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Capture {
                code: "",
                fields: Vec::new(),
            }
        }
    }

    impl std::fmt::Display for Capture<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.code)
        }
    }
}
