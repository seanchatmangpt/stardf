// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Stratification by monotone stratum propagation over the rule graph.
//!
//! Constraint: stratum(head) >= stratum(positive body pred), and
//! stratum(head) > stratum(negated body pred). Strata are monotonically
//! non-decreasing under the sweep, so propagation converges; a stratum value
//! that would exceed MAX_STRATA witnesses a negation cycle and is a typed
//! [`StratificationRefusal`] (never a silent wrap).

pub const MAX_STRATA: u32 = 4096;

#[derive(Debug, Clone)]
pub struct StratificationRefusal {
    pub rule: usize,
    pub head_pred: u32,
    pub neg_pred: u32,
    pub detail: String,
}

pub struct Stratification {
    pub strata: Vec<u32>,
    pub num_strata: u32,
}

impl Stratification {
    /// Computes strata and stamps them on each rule's `stratum` field.
    pub fn new(
        program: &mut crate::spine::rule::SpineProgram,
    ) -> Result<Stratification, StratificationRefusal> {
        let n = program.num_preds;
        let mut strata = vec![0u32; n];
        loop {
            let mut changed = false;
            for (ri, rule) in program.rules.iter().enumerate() {
                let h = rule.head.pred as usize;
                for (ai, a) in rule.body.iter().enumerate() {
                    let p = a.pred as usize;
                    let need = if ai == rule.neg { strata[p] + 1 } else { strata[p] };
                    if need > strata[h] {
                        strata[h] = need;
                        changed = true;
                        if strata[h] > MAX_STRATA {
                            return Err(StratificationRefusal {
                                rule: ri,
                                head_pred: rule.head.pred,
                                neg_pred: a.pred,
                                detail: format!(
                                    "negation cycle through pred {} (rule {})",
                                    a.pred, ri
                                ),
                            });
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        for r in program.rules.iter_mut() {
            r.stratum = strata[r.head.pred as usize];
        }
        Ok(Stratification { num_strata: strata.iter().copied().max().unwrap_or(0), strata })
    }
}
