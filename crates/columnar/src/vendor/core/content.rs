// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Content-addressed blob store shim: `ContentStore`, `ContentDigest`,
//! `ContentStoreError`.
//!
//! Divergence disclosure (lane 9): digests are blake3-256 hex
//! (64 lowercase hex chars), matching upstream's 64-hex shape; upstream
//! routes hashing through purrdf-hash's allocator-metered hash. The digest
//! function itself is blake3 in both, disclosed here for lane provenance.

use std::collections::BTreeMap;
use std::fmt;

use crate::vendor::hash::StructuredError;

/// A 256-bit content digest.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentDigest([u8; 32]);

impl fmt::Debug for ContentDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentDigest({})", self.to_hex())
    }
}

impl ContentDigest {
    /// Parse exactly 64 lowercase hex digits.
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        if text.len() != 64 {
            return None;
        }
        let mut bytes = [0_u8; 32];
        for (index, pair) in text.as_bytes().chunks(2).enumerate() {
            let hi = (pair[0] as char).to_digit(16)?;
            let lo = (pair[1] as char).to_digit(16)?;
            bytes[index] = ((hi << 4) | lo) as u8;
        }
        Some(Self(bytes))
    }

    /// Lowercase hex form.
    #[must_use]
    pub fn to_hex(&self) -> String {
        let mut out = String::with_capacity(64);
        for byte in self.0 {
            out.push(CHARS[usize::from(byte >> 4)] as char);
            out.push(CHARS[usize::from(byte & 0xf)] as char);
        }
        out
    }

    /// Raw bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for ContentDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

const CHARS: &[u8; 16] = b"0123456789abcdef";

/// Hash one payload with blake3-256.
#[must_use]
pub fn digest_bytes(payload: &[u8]) -> [u8; 32] {
    *blake3::hash(payload).as_bytes()
}

/// Blob digest refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentStoreError {
    /// The stored payload's digest does not match the claimed digest.
    DigestMismatch {
        /// The claimed digest.
        expected: ContentDigest,
        /// The digest of the stored payload.
        actual: ContentDigest,
    },
    /// The store is empty where a digest was required.
    Missing {
        /// The missing digest.
        digest: ContentDigest,
    },
}

impl fmt::Display for ContentStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DigestMismatch { expected, actual } => {
                write!(
                    f,
                    "blob digest mismatch: expected {expected}, actual {actual}"
                )
            }
            Self::Missing { digest } => write!(f, "blob digest missing: {digest}"),
        }
    }
}

impl std::error::Error for ContentStoreError {}

impl StructuredError for ContentStoreError {
    fn visit<'error>(&'error self, visitor: &mut dyn crate::vendor::hash::ErrorVisitor<'error>) {
        use crate::vendor::hash::ErrorValue;
        match self {
            Self::DigestMismatch { expected, actual } => {
                visitor.variant("content-digest-mismatch");
                visitor.value(Some("expected"), ErrorValue::Bytes(expected.as_bytes()));
                visitor.value(Some("actual"), ErrorValue::Bytes(actual.as_bytes()));
            }
            Self::Missing { digest } => {
                visitor.variant("content-digest-missing");
                visitor.value(Some("digest"), ErrorValue::Bytes(digest.as_bytes()));
            }
        }
    }
}

/// A content-addressed blob store.
#[derive(Debug, Clone, Default)]
pub struct ContentStore {
    blobs: BTreeMap<ContentDigest, Vec<u8>>,
}

impl ContentStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a payload under its blake3 digest.
    pub fn insert(&mut self, payload: Vec<u8>) -> ContentDigest {
        let digest = ContentDigest(digest_bytes(&payload));
        self.blobs.insert(digest, payload);
        digest
    }

    /// Insert a payload under a claimed digest, refusing a mismatch.
    pub fn insert_checked(
        &mut self, digest: ContentDigest, payload: Vec<u8>,
    ) -> Result<(), ContentStoreError> {
        let actual = ContentDigest(digest_bytes(&payload));
        if actual != digest {
            return Err(ContentStoreError::DigestMismatch {
                expected: digest,
                actual,
            });
        }
        self.blobs.insert(digest, payload);
        Ok(())
    }

    /// Verify every stored payload against its digest.
    pub fn verify_all(&self) -> Result<(), ContentStoreError> {
        for (digest, payload) in &self.blobs {
            let actual = ContentDigest(digest_bytes(payload));
            if &actual != digest {
                return Err(ContentStoreError::DigestMismatch {
                    expected: *digest,
                    actual,
                });
            }
        }
        Ok(())
    }

    /// Iterate `(digest, payload)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&ContentDigest, &Vec<u8>)> {
        self.blobs.iter()
    }

    /// Number of stored blobs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.blobs.len()
    }

    /// True when no blobs are stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blobs.is_empty()
    }
}
