//! stardf-planner — metadata-driven access paths from sealed footers.
//!
//! Lane RDF-3 SIP-ALT. Design-around law: OST P2/P3/P4 practice runtime
//! sideways information passing (dynamic constraint propagation DURING join
//! execution). This crate does PRECOMPUTED STRUCTURAL PLANNING instead:
//! triple files carry a sealed footer with per-file per-predicate counts
//! sealed at write time; the planner selects join order and filter placement
//! FROM SEALED METADATA before execution. Runtime SIP is unnecessary because
//! cardinality bounds are known statically per file.
//!
//! Not-practiced claim classes (US 11,216,456 / US 11,409,698): both claim
//! runtime constraint propagation between operators while a query runs.
//! Nothing here propagates constraints at runtime between joins — ordering
//! is fixed before the first triple is touched, from sealed bytes.

pub mod exec;
pub mod footer;
pub mod plan;

/// Little-endian `u32` read at `off` (None if out of bounds).
pub(crate) fn rd_u32(b: &[u8], off: usize) -> Option<u32> {
    b.get(off..off + 4).map(|s| u32::from_le_bytes(s.try_into().expect("4 bytes")))
}

/// Little-endian `u64` read at `off` (None if out of bounds).
pub(crate) fn rd_u64(b: &[u8], off: usize) -> Option<u64> {
    b.get(off..off + 8).map(|s| u64::from_le_bytes(s.try_into().expect("8 bytes")))
}
