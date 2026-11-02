# tests/disabled/ — parked upstream test targets (stardf-deflate)

<!-- SPDX-FileCopyrightText: 2026 Sean Chatman -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

Vendoring lane 2026-10-31: these two files are carried BYTE-VERBATIM from
purrdf rev `0da0578306983da851a9073337d57e2e2c41ab3a` `crates/deflate/tests/`
(buffers verified against `git show <rev>:crates/deflate/tests/<name>`); they
are parked, not deleted. They are in `disabled/` (a subdirectory) precisely so
cargo's auto-discovery — which scans only `tests/*.rs` top-level — does not
compile them.

| parked file | blocks it | unpark precondition |
|---|---|---|
| `deflate_conformance.rs` | dev-dep `purrdf-testkit` (`harness = false` runner; also uses `purrdf_hash`, which IS available) | `stardf-testkit` lands in the stardf/arrow workspace; move this file back to `tests/deflate_conformance.rs`; restore the `[[test]] name = "deflate_conformance" harness = false` manifest section verbatim; restore the `[dev-dependencies]` entry verbatim |
| `allocation_budget.rs` | dev-dep `purrdf-alloc-probe` | `stardf-alloc-probe` lands; move back to `tests/allocation_budget.rs`; (no manifest section — auto-discovered as a test) |

`tests/vectors/match_length_vectors.txt` stayed at `tests/vectors/` (carried
verbatim): the conformance test reads it by relative path, so unparking needs
no vector moves.

Unpark is a separate lane's work item (testkit-slice vendor), never a silent
fix inside this crate.
