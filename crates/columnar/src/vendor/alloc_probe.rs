// SPDX-FileCopyrightText: 2026 Blackcat Informatics(R) Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Path-parity shim for the consumed slice of `purrdf-alloc-probe`: a
//! counting global allocator with per-thread measurement windows, consumed by
//! the preserved upstream test modules.
//!
//! Divergence disclosure (lane 1, 2026-10-31): the shim counts real
//! allocations through a real `GlobalAlloc` over `System` (Chicago-school: no
//! fake counter); it does not implement upstream's cross-thread identity or
//! historical-window queries.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

/// Per-thread window state, kept allocation-free (`Cell` with a const init).
#[derive(Clone, Copy)]
struct WindowState {
    open: bool,
    allocations: u64,
    requested: u64,
    freed: u64,
    peak: u64,
}

thread_local! {
    static WINDOW: Cell<WindowState> = const { Cell::new(WindowState {
        open: false, allocations: 0, requested: 0, freed: 0, peak: 0,
    }) };
}

/// The counts a closed window reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measured {
    /// Number of allocations inside the window.
    pub allocations: u64,
    /// Total bytes requested inside the window.
    pub requested_bytes: u64,
    /// Bytes still logically held (requested minus freed) at close.
    pub retained_bytes: u64,
    /// Peak net live bytes (requested minus freed) inside the window.
    pub peak_working_bytes: u64,
}

// The GlobalAlloc impl below is the crate's single sanctioned unsafe surface;
// it cannot be written safely. Disclosure lives in src/lib.rs / PROVENANCE.md.
/// A counting global allocator. Install with `#[global_allocator]` in test
/// binaries that assert zero-allocation error paths.
pub struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            WINDOW.with(|w| {
                let mut s = w.get();
                if s.open {
                    s.allocations += 1;
                    s.requested += layout.size() as u64;
                    let live = s.requested - s.freed;
                    if live > s.peak {
                        s.peak = live;
                    }
                }
                w.set(s);
            });
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        WINDOW.with(|w| {
            let mut s = w.get();
            if s.open {
                s.freed += layout.size() as u64;
            }
            w.set(s);
        });
        System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            WINDOW.with(|w| {
                let mut s = w.get();
                if s.open {
                    s.freed += layout.size() as u64;
                    s.allocations += 1;
                    s.requested += new_size as u64;
                    let live = s.requested - s.freed;
                    if live > s.peak {
                        s.peak = live;
                    }
                }
                w.set(s);
            });
        }
        new_ptr
    }
}

/// A per-thread measurement window: allocations on this thread between
/// [`open`](Self::open) and [`close`](Self::close) are counted.
#[must_use = "a window that is never closed measures nothing"]
pub struct CurrentThreadWindow {
    _not_send: std::marker::PhantomData<*const ()>,
}

impl Default for CurrentThreadWindow {
    fn default() -> Self {
        Self::open()
    }
}

impl CurrentThreadWindow {
    /// Open a measurement window on the calling thread.
    pub fn open() -> Self {
        WINDOW.with(|w| {
            let mut s = w.get();
            s.open = true;
            s.allocations = 0;
            s.requested = 0;
            s.freed = 0;
            s.peak = 0;
            w.set(s);
        });
        Self {
            _not_send: std::marker::PhantomData,
        }
    }

    /// Close the window and report what it measured on this thread.
    #[must_use]
    pub fn close(self) -> Measured {
        WINDOW.with(|w| {
            let mut s = w.get();
            s.open = false;
            let m = Measured {
                allocations: s.allocations,
                requested_bytes: s.requested,
                retained_bytes: s.requested.saturating_sub(s.freed),
                peak_working_bytes: s.peak,
            };
            s.requested = 0;
            s.freed = 0;
            s.peak = 0;
            w.set(s);
            m
        })
    }
}
