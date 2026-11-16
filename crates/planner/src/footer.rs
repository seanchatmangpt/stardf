//! Sealed footer: per-file per-predicate counts, FNV-1a-64 sealed.
//!
//! Seal class cross-referenced read-only from cgap-parquet
//! `crates/cgap-parquet/src/footer.rs` (A02: FNV-1a-64 with fixed
//! basis/prime, magic + footer-length trailer). The planner reads
//! stardf-side files natively: this IS the stardf-side footer surface.

use std::vec::Vec;
use core::fmt;
use crate::{rd_u32, rd_u64};

/// FNV-1a-64, fixed basis/prime, no dependency (A02 seal class).
pub fn fnv1a64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

pub const MAGIC: [u8; 4] = *b"PLNF";

/// Typed footer faults. Divergence detection at runtime: a poisoned
/// (re-sealed but wrong) count is `CountDivergence`, not a parse error.
#[derive(Debug, PartialEq, Eq)]
pub enum FooterFault {
    /// Trailer magic is not `PLNF`.
    BadMagic,
    /// Footer region shorter than the trailer claims.
    Truncated,
    /// FNV seal over the counts table does not match the stored seal.
    SealMismatch { sealed: u64, stored: u64 },
    /// Actual per-predicate count observed while scanning differs from the
    /// sealed count. This is the anti-vacuity flip channel.
    CountDivergence { pred: u32, sealed: u64, actual: u64 },
}

impl fmt::Display for FooterFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMagic => write!(f, "bad footer magic"),
            Self::Truncated => write!(f, "footer truncated"),
            Self::SealMismatch { sealed, stored } => {
                write!(f, "seal mismatch sealed={sealed:#x} stored={stored:#x}")
            }
            Self::CountDivergence { pred, sealed, actual } => {
                write!(f, "count divergence pred={pred} sealed={sealed} actual={actual}")
            }
        }
    }
}

/// Parsed footer: sorted `(pred, count)` entries + total row count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedFooter {
    pub entries: Vec<(u32, u64)>,
    pub row_count: u64,
}

/// Seal span = entries + row_count + n_entries, u64 LE.
fn seal_span(entries: &[(u32, u64)], row_count: u64, n: u32) -> Vec<u8> {
    let mut b = Vec::with_capacity(entries.len() * 12 + 16);
    for &(p, c) in entries {
        b.extend_from_slice(&p.to_le_bytes());
        b.extend_from_slice(&c.to_le_bytes());
        let _ = n;
    }
    b.extend_from_slice(&row_count.to_le_bytes());
    b.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    b
}

/// Seal a body of `u32` triples (s,p,o) into a complete file: body +
/// counts table + FNV seal + magic + footer length.
pub fn seal_file(triples: &[(u32, u32, u32)]) -> Vec<u8> {
    let mut body = Vec::with_capacity(triples.len() * 12);
    let mut counts: Vec<(u32, u64)> = Vec::new();
    for &(s, p, o) in triples {
        body.extend_from_slice(&s.to_le_bytes());
        body.extend_from_slice(&p.to_le_bytes());
        build_counts(&mut counts, p);
        body.extend_from_slice(&o.to_le_bytes());
        let _ = s;
    }
    counts.sort_unstable_by_key(|e| e.0);
    let row_count = triples.len() as u64;
    let span = seal_span(&counts, row_count, counts.len() as u32);
    let mut out = body;
    out.extend_from_slice(&span);
    let seal = fnv1a64(&span);
    out.extend_from_slice(&seal.to_le_bytes());
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&((span.len() + 16) as u32).to_le_bytes());
    out
}

/// Trailer is 12 bytes: seal (8) + magic (4); then footer_len u32 (4).
const TRAILER: usize = 12 + 4;

/// Parse + verify the seal on a sealed file. `SealMismatch` fires on any
/// byte flipped inside the counts table or row_count.
pub fn parse_footer(bytes: &[u8]) -> Result<SealedFooter, FooterFault> {
    if bytes.len() < TRAILER || bytes[bytes.len() - 8..bytes.len() - 4] != MAGIC {
        return Err(FooterFault::BadMagic);
    }
    let flen = rd_u32(bytes, bytes.len() - 4).ok_or(FooterFault::Truncated)? as usize;
    if flen < 16 || flen > bytes.len() {
        return Err(FooterFault::Truncated);
    }
    let footer_start = bytes.len() - flen;
    let seal_span_len = flen - TRAILER;
    let span = &bytes[footer_start..footer_start + seal_span_len];
    let stored = rd_u64(bytes, footer_start + seal_span_len).ok_or(FooterFault::Truncated)?;
    let sealed = fnv1a64(span);
    if sealed != stored {
        return Err(FooterFault::SealMismatch { sealed, stored });
    }
    // Span tail layout: ... entries | row_count (8) | n_entries (4).
    let n = rd_u32(span, span.len() - 4).ok_or(FooterFault::Truncated)? as usize;
    let mut entries = Vec::with_capacity(n);
    let mut off = 0;
    for _ in 0..n {
        let p = rd_u32(span, off).ok_or(FooterFault::Truncated)?;
        let c = rd_u64(span, off + 4).ok_or(FooterFault::Truncated)?;
        entries.push((p, c));
        off += 12;
    }
    let row_count =
        rd_u64(span, span.len() - 12).ok_or(FooterFault::Truncated)?;
    Ok(SealedFooter { entries, row_count })
}

/// Sealed count for `pred`, if the footer carries it.
pub fn sealed_count(f: &SealedFooter, pred: u32) -> Option<u64> {
    f.entries.iter().find(|e| e.0 == pred).map(|e| e.1)
}

/// Scan a file's body collecting the ACTUAL per-predicate counts, then
/// compare against the seal. Returns the triples on success; a typed
/// [`FooterFault::CountDivergence`] on the first mismatch (refuse: the
/// caller may re-plan excluding the divergent file).
pub fn scan_verified(
    bytes: &[u8],
    footer: &SealedFooter,
) -> Result<Vec<(u32, u32, u32)>, FooterFault> {
    let body_len = bytes.len() - rd_u32(bytes, bytes.len() - 4).unwrap_or(0) as usize;
    let n = body_len / 12;
    let mut counts: Vec<(u32, u64)> = Vec::new();
    let mut triples = Vec::with_capacity(n);
    for i in 0..n {
        let s = rd_u32(bytes, i * 12).ok_or(FooterFault::Truncated)?;
        let p = rd_u32(bytes, i * 12 + 4).ok_or(FooterFault::Truncated)?;
        let o = rd_u32(bytes, i * 12 + 8).ok_or(FooterFault::Truncated)?;
        build_counts(&mut counts, p);
        triples.push((s, p, o));
    }
    counts.sort_unstable_by_key(|e| e.0);
    for (p, actual) in &counts {
        match sealed_count(footer, *p) {
            Some(sealed) if sealed != *actual => {
                return Err(FooterFault::CountDivergence {
                    pred: *p,
                    sealed,
                    actual: *actual,
                })
            }
            _ => {}
        }
    }
    for (p, sealed) in &footer.entries {
        if counts.iter().all(|(ap, _)| ap != p) && *sealed != 0 {
            return Err(FooterFault::CountDivergence {
                pred: *p,
                sealed: *sealed,
                actual: 0,
            });
        }
    }
    Ok(triples)
}

fn build_counts(counts: &mut Vec<(u32, u64)>, p: u32) {
    for e in counts.iter_mut() {
        if e.0 == p {
            e.1 += 1;
        } else {
            continue;
        }
        return;
    }
    counts.push((p, 1));
}
