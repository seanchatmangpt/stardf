// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Varint/zigzag primitives and `PackBitsError`, extracted verbatim from
//! purrdf-core rev 0da05783 (`ir/pack/bits.rs`) — lane 9 vendored seam.

/// Why decoding a `pack` byte buffer failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PackBitsError {
    /// The buffer ended before all the bytes a header promised were present.
    Truncated {
        /// The total leading byte count the format required.
        needed: usize,
        /// The byte count actually available.
        found: usize,
    },
    /// The buffer's header was internally inconsistent (e.g. a stored count that
    /// disagrees with one derived from another header field), or a value fell
    /// outside its documented domain (e.g. an `IntVector` width `> 64`).
    Malformed(&'static str),
}

/// Append `value` as an unsigned LEB128 varint (7 payload bits per byte, MSB =
/// continuation flag): 1..=10 bytes.
pub fn write_varint(out: &mut impl Extend<u8>, value: u64) {
    let mut v = value;
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.extend([byte]);
            return;
        }
        out.extend([byte | 0x80]);
    }
}

/// Decode one unsigned LEB128 varint starting at `*pos`, advancing `*pos` past
/// it.
///
/// # Errors
///
/// [`PackBitsError::Truncated`] if the buffer ends before a terminating byte
/// (continuation bit clear) is seen; [`PackBitsError::Malformed`] if the
/// encoding runs past 64 payload bits (not a valid `u64` varint).
pub fn read_varint(bytes: &[u8], pos: &mut usize) -> Result<u64, PackBitsError> {
    let mut result = 0u64;
    let mut shift = 0u32;
    loop {
        if shift >= 64 {
            return Err(PackBitsError::Malformed("varint exceeds 64 bits"));
        }
        let byte = *bytes.get(*pos).ok_or_else(|| PackBitsError::Truncated {
            needed: pos.saturating_add(1),
            found: bytes.len(),
        })?;
        *pos += 1;
        if shift == 63 && byte & 0x7e != 0 {
            return Err(PackBitsError::Malformed("varint exceeds 64 bits"));
        }
        result |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
    }
}

/// Map a signed integer to an unsigned one so small-magnitude values of EITHER
/// sign encode as small varints: `0, -1, 1, -2, 2, ... -> 0, 1, 2, 3, 4, ...`.
/// The standard "zigzag" mapping (as used by Protocol Buffers); round-trips every
/// `i64` including `i64::MIN`.
#[must_use]
pub fn zigzag_encode(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}

/// The inverse of [`zigzag_encode`].
#[must_use]
pub fn zigzag_decode(value: u64) -> i64 {
    ((value >> 1) as i64) ^ -((value & 1) as i64)
}

impl std::fmt::Display for PackBitsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Truncated { needed, found } => {
                write!(f, "pack bits truncated: needs {needed}, has {found}")
            }
            Self::Malformed(detail) => write!(f, "pack bits malformed: {detail}"),
        }
    }
}

impl std::error::Error for PackBitsError {}

impl crate::vendor::hash::StructuredError for PackBitsError {
    fn visit<'error>(&'error self, visitor: &mut dyn crate::vendor::hash::ErrorVisitor<'error>) {
        use crate::vendor::hash::ErrorValue;
        match self {
            Self::Truncated { needed, found } => {
                visitor.variant("pack-bits-truncated");
                visitor.value(Some("needed"), ErrorValue::ByteCount(*needed as u128));
                visitor.value(Some("found"), ErrorValue::ByteCount(*found as u128));
            }
            Self::Malformed(detail) => {
                visitor.variant("pack-bits-malformed");
                visitor.value(Some("detail"), ErrorValue::Text(detail));
            }
        }
    }
}
