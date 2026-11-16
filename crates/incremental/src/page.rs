//! Immutable pages with footer-sequenced metadata. Zero-dep: FNV-1a 64 as
//! the footer checksum (named honestly: a checksum, not a crypto hash).

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Fact(pub u64, pub u64, pub u64);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PageKind {
    Base,
    Delta,
}

#[derive(Clone, Debug)]
pub struct Page {
    pub id: u64,
    pub kind: PageKind,
    pub facts: Vec<Fact>,
}

#[derive(Clone, Debug)]
pub struct PageFooter {
    pub page_id: u64,
    pub kind: PageKind,
    pub fact_count: u64,
    pub facts_hash: u64,
}

/// FNV-1a 64 over fact bytes (big-endian per field).
pub fn facts_hash(facts: &[Fact]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut feed = |b: u8, h: &mut u64| {
        *h ^= b as u64;
        *h = h.wrapping_mul(0x100000001b3);
    };
    for &Fact(a, b, c) in facts {
        for v in [a, b, c] {
            for i in 0..8 {
                feed(((v >> (56 - 8 * i)) & 0xff) as u8, &mut h);
            }
        }
    }
    h
}

impl Page {
    pub fn new(id: u64, kind: PageKind, facts: Vec<Fact>) -> Self {
        Page { id, kind, facts }
    }

    pub fn footer(&self) -> PageFooter {
        PageFooter {
            page_id: self.id,
            kind: self.kind,
            fact_count: self.facts.len() as u64,
            facts_hash: facts_hash(&self.facts),
        }
    }

    pub fn contains(&self, f: &Fact) -> bool {
        self.facts.iter().any(|x| x == f)
    }

    /// Binary-ish linear containment; pages are small vectors.
    pub fn contains_all(&self, fs: &[Fact]) -> bool {
        fs.iter().all(|f| self.contains(f))
    }
}

impl fmt::Display for PageFooter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PageFooter({} {:?} n={} h={:016x})",
            self.page_id, self.kind, self.fact_count, self.facts_hash
        )
    }
}
