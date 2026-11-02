# benches/disabled/ — parked upstream bench target (stardf-deflate)

<!-- SPDX-FileCopyrightText: 2026 Sean Chatman -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

Vendoring lane 2026-10-31: `gzip.rs` is carried BYTE-VERBATIM from purrdf rev
`0da0578306983da851a9073337d57e2e2c41ab3a` `crates/deflate/benches/gzip.rs`;
it is parked, not deleted. It is in `disabled/` (a subdirectory) precisely so
cargo's auto-discovery — which scans only `benches/*.rs` top-level — does not
compile it.

| parked file | blocks it | unpark precondition |
|---|---|---|
| `gzip.rs` | dev-dep `purrdf-testkit` (`harness = false` bench runner; `use purrdf_hash::Backend as _` resolves fine) | `stardf-testkit` lands in the stardf/arrow workspace; move this file back to `benches/gzip.rs`; restore the `[[bench]] name = "gzip" harness = false` manifest section verbatim; restore the `[dev-dependencies]` entries verbatim |

Unpark is a separate lane's work item (testkit-slice vendor), never a silent
fix inside this crate.
