//! Log-structured store over immutable pages. Deletion = page invalidation
//! + fresh delta-page sealing. All-or-nothing page transition: an operation
//! appends its records, then a CommitRecord; a reload replays the log and
//! applies a commit only when its full record set checksums — trailing or
//! partial writes (power cut) are dropped, never half-applied.

use crate::page::{facts_hash, Fact, Page, PageKind};
use std::collections::{BTreeSet, HashMap, HashSet};

pub const PAGE_CAPACITY: usize = 64;

#[derive(Clone, Debug)]
pub enum LogRecord {
    Page(Page),
    Invalidation { page_id: u64, seq: u64 },
    Commit { txn: u64, payload_hash: u64 },
}

pub fn hash_records(rs: &[LogRecord]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for r in rs {
        match r {
            LogRecord::Page(p) => {
                h ^= p.id;
                h = h.wrapping_mul(0x100000001b3);
                h ^= facts_hash(&p.facts);
            }
            LogRecord::Invalidation { page_id, seq } => {
                h ^= *page_id ^ seq.wrapping_mul(0x9e3779b97f4a7c15);
                h = h.wrapping_mul(0x100000001b3);
            }
            LogRecord::Commit { txn, .. } => {
                h ^= txn.wrapping_mul(0xff51afd7ed558ccd);
                h = h.wrapping_mul(0x100000001b3);
            }
        }
    }
    h
}

#[derive(Clone, Debug, Default)]
pub struct Store {
    /// All sealed pages ever, by id (immutable once sealed).
    pages: HashMap<u64, Page>,
    /// Page ids invalidated: their facts are logically gone unless re-added
    /// by a later delta page.
    invalidated: BTreeSet<u64>,
    /// Delta page ids in seal order.
    deltas: Vec<u64>,
    next_page_id: u64,
    next_txn: u64,
}

pub struct DeleteSummary {
    pub pages_invalidated: usize,
    pub deltas_sealed: usize,
    pub facts_deleted: u64,
    pub facts_resealed: u64,
    pub txn: u64,
    /// The exact applied transaction records (with trailing commit).
    pub records: Vec<LogRecord>,
}

impl Store {
    pub fn new() -> Self {
        Store::default()
    }

    pub fn page(&self, id: u64) -> Option<&Page> {
        self.pages.get(&id)
    }

    pub fn invalidated(&self) -> &BTreeSet<u64> {
        &self.invalidated
    }

    pub fn delta_ids(&self) -> &[u64] {
        &self.deltas
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Live pages: sealed, never invalidated. Sorted by id for determinism.
    pub fn live_pages(&self) -> Vec<&Page> {
        let mut v: Vec<&Page> = self
            .pages
            .values()
            .filter(|p| !self.invalidated.contains(&p.id))
            .collect();
        v.sort_by_key(|p| p.id);
        v
    }

    /// Logical fact set = union of live pages + delta pages.
    pub fn fact_set(&self) -> HashSet<Fact> {
        let mut s: HashSet<Fact> = HashSet::new();
        for p in self.live_pages() {
            s.extend(p.facts.iter().copied());
        }
        s
    }

    pub fn contains(&self, f: &Fact) -> bool {
        self.live_pages().iter().any(|p| p.contains(f))
    }

    pub fn seal_base_pages(&mut self, facts: &[Fact], capacity: usize) -> Vec<u64> {
        let mut ids = Vec::new();
        for chunk in facts.chunks(capacity.max(1)) {
            let id = self.next_page_id;
            self.next_page_id += 1;
            self.pages
                .insert(id, Page::new(id, PageKind::Base, chunk.to_vec()));
            ids.push(id);
        }
        ids
    }

    /// Transactional variant: returns the exact log records (with commit)
    /// that were applied — used by courts to build durable logs.
    pub fn seal_base_pages_txn(&mut self, facts: &[Fact], capacity: usize) -> Vec<LogRecord> {
        let mut records: Vec<LogRecord> = Vec::new();
        let mut ids = Vec::new();
        let cap = capacity.max(1);
        for chunk in facts.chunks(cap) {
            let id = self.next_page_id;
            self.next_page_id += 1;
            records.push(LogRecord::Page(Page::new(id, PageKind::Base, chunk.to_vec())));
            ids.push(id);
        }
        let payload_hash = hash_records(&records);
        let txn = self.next_txn;
        self.next_txn += 1;
        records.push(LogRecord::Commit { txn, payload_hash });
        assert!(self.apply_checked(&records));
        records
    }

    /// Bulk delete over live pages: invalidate each hit page, reseal
    /// survivors into fresh delta pages. All-or-nothing: records + commit
    /// applied as one unit (see `apply_checked`).
    pub fn bulk_delete(&mut self, to_delete: &[Fact], survivor_capacity: usize) -> DeleteSummary {
        let del: HashSet<Fact> = to_delete.iter().copied().collect();
        let mut touched: Vec<u64> = Vec::new();
        let mut deleted: u64 = 0;
        let mut resealed: u64 = 0;
        let mut records: Vec<LogRecord> = Vec::new();
        let mut new_delta_ids: Vec<u64> = Vec::new();
        let mut buf: Vec<Fact> = Vec::new();
        let mut next_id = self.next_page_id;
        let cap = survivor_capacity.max(1);

        let live = self.live_pages();
        for p in live {
            let hits: Vec<Fact> = p.facts.iter().copied().filter(|f| del.contains(f)).collect();
            let survivors: Vec<Fact> = p
                .facts
                .iter()
                .copied()
                .filter(|f| !del.contains(f))
                .collect();
            if hits.is_empty() {
                continue;
            }
            touched.push(p.id);
            deleted += hits.len() as u64;
            resealed += survivors.len() as u64;
            for f in survivors {
                if buf.len() == cap {
                    let id = next_id;
                    next_id += 1;
                    records.push(LogRecord::Page(Page::new(id, PageKind::Delta, buf.clone())));
                    new_delta_ids.push(id);
                    buf.clear();
                }
                buf.push(f);
            }
        }
        if !buf.is_empty() {
            let id = next_id;
            next_id += 1;
            records.push(LogRecord::Page(Page::new(id, PageKind::Delta, buf)));
            new_delta_ids.push(id);
        }
        for id in &touched {
            records.push(LogRecord::Invalidation {
                page_id: *id,
                seq: self.next_txn,
            });
        }
        let txn = self.next_txn;
        let payload_hash = hash_records(&records);
        records.push(LogRecord::Commit { txn, payload_hash });
        self.next_txn += 1;
        let deltas_sealed = new_delta_ids.len();
        let pages_invalidated = touched.len();
        self.apply_checked(&records);
        DeleteSummary {
            pages_invalidated,
            deltas_sealed,
            facts_deleted: deleted,
            facts_resealed: resealed,
            txn,
            records,
        }
    }

    /// All-or-nothing apply: only when the trailing Commit checksums over
    /// the full preceding record slice is the state transition applied.
    /// Otherwise nothing changes (torn write => previous state stands).
    pub fn apply_checked(&mut self, records: &[LogRecord]) -> bool {
        let Some(last) = records.last() else {
            return false;
        };
        let (txn, payload_hash) = match last {
            LogRecord::Commit { txn, payload_hash } => (*txn, *payload_hash),
            _ => return false,
        };
        let head = &records[..records.len() - 1];
        let _ = txn;
        if hash_records(head) != payload_hash {
            return false;
        }
        for r in head {
            match r {
                LogRecord::Page(p) => {
                    self.pages.insert(p.id, p.clone());
                    if p.kind == PageKind::Delta {
                        self.deltas.push(p.id);
                    }
                    self.next_page_id = self.next_page_id.max(p.id + 1);
                }
                LogRecord::Invalidation { page_id, .. } => {
                    self.invalidated.insert(*page_id);
                }
                LogRecord::Commit { .. } => unreachable!(),
            }
        }
        true
    }
}

// --- Durable log codec -------------------------------------------------
// Frame: [payload_len u32 LE][fnv64 u64 LE][payload]. Replay stops at the
// first truncated or checksum-mismatched frame (power-cut boundary); a
// commit is applied only when its full transaction checksums (A05-style
// all-or-nothing page transition).

fn fnv_bytes(h0: u64, bytes: &[u8]) -> u64 {
    let mut h = h0;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn put_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn get_u64(b: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(b[off..off + 8].try_into().unwrap())
}

pub fn encode_records(records: &[LogRecord]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut payload = Vec::new();
    for r in records {
        payload.clear();
        match r {
            LogRecord::Page(p) => {
                payload.push(1u8);
                put_u64(&mut payload, p.id);
                payload.push(matches!(p.kind, PageKind::Delta) as u8);
                put_u64(&mut payload, p.facts.len() as u64);
                for &Fact(a, b, c) in &p.facts {
                    put_u64(&mut payload, a);
                    put_u64(&mut payload, b);
                    put_u64(&mut payload, c);
                }
            }
            LogRecord::Invalidation { page_id, seq } => {
                payload.push(2u8);
                put_u64(&mut payload, *page_id);
                put_u64(&mut payload, *seq);
            }
            LogRecord::Commit { txn, payload_hash } => {
                payload.push(3u8);
                put_u64(&mut payload, *txn);
                put_u64(&mut payload, *payload_hash);
            }
        }
        let mut frame = Vec::with_capacity(payload.len() + 12);
        frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        frame.extend_from_slice(&fnv_bytes(0xcbf29ce484222325, &payload).to_le_bytes());
        frame.extend_from_slice(&payload);
        out.extend_from_slice(&frame);
    }
    out
}

/// Replay a possibly-truncated log. Commits apply only when complete and
/// checksum-valid; everything after the last valid commit is discarded.
pub fn replay(bytes: &[u8]) -> Store {
    let mut records: Vec<LogRecord> = Vec::new();
    let mut committed: Vec<LogRecord> = Vec::new();
    let mut off = 0usize;
    while off + 12 <= bytes.len() {
        let len = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap()) as usize;
        let crc = get_u64(bytes, off + 4);
        let end = off + 12 + len;
        if end > bytes.len() {
            break; // torn tail
        }
        let payload = &bytes[off + 12..end];
        if fnv_bytes(0xcbf29ce484222325, payload) != crc {
            break; // corrupt/torn frame
        }
        records.push(decode_payload(payload));
        off = end;
    }
    // Fold into commit-sized transactions.
    let mut txn: Vec<LogRecord> = Vec::new();
    for r in records {
        let is_commit = matches!(r, LogRecord::Commit { .. });
        txn.push(r);
        if is_commit {
            committed.extend(txn.drain(..));
        }
    }
    let mut store = Store::new();
    // Apply each complete, checksum-valid transaction in log order.
    let mut txn: Vec<LogRecord> = Vec::new();
    for r in committed {
        let is_commit = matches!(r, LogRecord::Commit { .. });
        txn.push(r);
        if is_commit {
            store.apply_checked(&txn);
            txn.clear();
        }
    }
    store
}

fn decode_payload(b: &[u8]) -> LogRecord {
    match b[0] {
        1 => {
            let id = get_u64(b, 1);
            let kind = if b[9] == 1 { PageKind::Delta } else { PageKind::Base };
            let n = get_u64(b, 10) as usize;
            let mut facts = Vec::with_capacity(n);
            let mut off = 18;
            for _ in 0..n {
                facts.push(Fact(get_u64(b, off), get_u64(b, off + 8), get_u64(b, off + 16)));
                off += 24;
            }
            LogRecord::Page(Page::new(id, kind, facts))
        }
        2 => LogRecord::Invalidation {
            page_id: get_u64(b, 1),
            seq: get_u64(b, 9),
        },
        3 => LogRecord::Commit {
            txn: get_u64(b, 1),
            payload_hash: get_u64(b, 9),
        },
        _ => panic!("unknown record tag"),
    }
}
