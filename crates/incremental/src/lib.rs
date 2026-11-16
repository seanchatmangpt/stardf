//! Page-generation invalidation incremental maintenance over immutable,
//! footer-sequenced pages.
//!
//! DESIGN-AROUND LAW (named): OST's incremental maintenance is per-fact
//! Backward/Forward chaining through in-memory indices. stardf pages are
//! IMMUTABLE and footer-sequenced — deletion here = page invalidation +
//! fresh delta-page sealing (all-or-nothing page transition, zero torn
//! state, receipt-checkable); materialized views recompute PER-DELTA-PAGE,
//! not per-fact.
//!
//! TRADE: bulk page granularity vs OST per-fact granularity. Page
//! granularity wins when churn is bulk (deleting a fraction of a page's
//! facts amortizes the invalidate+reseal over many facts); per-fact
//! granularity wins for surgical single-fact edits, where page
//! granularity forces rewriting an entire page per edit. Boundary:
//! crossover where (facts deleted per page) x (pages touched) makes
//! reseal cost per edit exceed per-fact index cost — measured in
//! `examples/bench_bulk.rs`.

pub mod page;
pub mod store;
pub mod view;

pub use page::{Fact, Page, PageFooter, PageKind};
pub use store::{LogRecord, Store};
pub use view::CountView;
