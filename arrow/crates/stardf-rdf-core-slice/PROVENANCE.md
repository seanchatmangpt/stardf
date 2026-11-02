# PROVENANCE — stardf-rdf-core-slice

## Source identity

Vendored from PurRDF `crates/rdf-core` at rev
`0da0578306983da851a9073337d57e2e2c41ab3a`
(`git -C /Users/sac/purrdf rev-parse` = same value, 2026-10-31). All extraction
ran through `git -C /Users/sac/purrdf archive <rev> <path>` — the dirty working
tree was never read.

## Deliverable: the term-model files (the lane's extraction scope)

Five files, byte-identical to the pinned rev (verified per-file with
`git show <rev>:<path> | cmp` against the landed copy; all exit 0):

| upstream `crates/rdf-core/src/...` | slice path | lines |
|---|---|---|
| `ir/term.rs` | `src/term.rs` | 2063 |
| `ir/term/canonical.rs` | `src/term/canonical.rs` | 90 |
| `ir/term_walk.rs` | `src/term_walk.rs` | 2490 |
| `model.rs` | `src/model.rs` | 566 |
| `model/traits.rs` | `src/model/traits.rs` | 654 |

The four named types land in these files: `TermValue` + `BlankScope`
(`src/term.rs`), `RdfTerm` + `RdfTermKind` (`src/model.rs`), and
`TermBox = Nested<TermValue>` (`src/term_walk.rs:53`).

## Closure modules (beyond the named file set, disclosed)

Added to make the mechanical move build as far as it closes with ZERO edits to
any vendored file: `blank_label.rs` (1382), `small.rs` (1464) +
`small_tests.rs`, `diagnostic.rs` (1035) — each byte-identical to the pinned
rev, each self-contained over the landed seams at extraction time (blank_label:
std + `purrdf_lex::allocation` + `BlankScope`; small: core only; diagnostic:
std + `purrdf_hash::error` + `resident_clone!`).

## Leaf seam crates (`vendor-crates/`, all verbatim surfaces)

Package names carry the `-seam` suffix (no collision with W1's full-vendoring
package names); lib names stay `purrdf_*` VERBATIM per the rename law, so the
vendored files' `use purrdf_*::...` paths resolve UNEDITED.

| seam | lib | files (all byte-verified) |
|---|---|---|
| stardf-hash-seam | purrdf_hash | digest.rs (50), frame.rs (295), error.rs (469), error/io.rs (173), impls.rs (SEAM-SCOPED: `debug_non_exhaustive!` lines 35-50 + `default_from_new!` lines 93-115 only) |
| stardf-lex-seam | purrdf_lex | allocation.rs (1286) + allocation/{boxed,ordered,shared,shared/buffers,text}.rs, text_out.rs (74), constructors.rs (565, `resident_clone!`/`variant_from!`/`constructors!` macros), walk.rs (1290) + walk/{scalar,sort}.rs |
| stardf-xsd-seam | purrdf_xsd | datatype.rs = SEAM-SCOPED verbatim const block only (upstream lines 13-136: XSD_NS..OWL_RATIONAL); the `XsdDatatype` enum/impl surface (exact/value/bigint closure, ~14K lines) is deliberately NOT approximated |
| stardf-iri-seam | purrdf_iri | vocab.rs (1773), vocab/ogc.rs (497), langtag.rs (3460) |
| stardf-events-seam | purrdf_events | TextDirection block verbatim (upstream rdf-events/src/lib.rs lines 214-260) |
| stardf-cdt-seam | purrdf_cdt | term.rs seam: the verbatim re-export line `pub use purrdf_events::TextDirection;` (upstream cdt/src/term.rs:33) |

Seam manifests carry upstream facts read at copy time: edition **2024**,
rust-version **1.98** (upstream workspace `Cargo.toml` lines 68-69). The
`[dependencies]` alias form `purrdf_lex = { package = "stardf-lex-seam", path
= ... }` binds the verbatim lib name to the renamed package.

## New files authored for the slice (not vendored)

- `Cargo.toml` (main crate) — skeleton; license field transcribed verbatim from
  upstream `crates/rdf-core/Cargo.toml` (`(MIT OR Apache-2.0 OR MulanPSL-2.0)
  AND Unicode-3.0`).
- `src/lib.rs` — module declarations + root re-exports + the `pub mod ir {
  pub use crate::term; }` compatibility module that preserves upstream
  `crate::ir::term::...` paths.
- `vendor-crates/*/src/lib.rs` (6 seam lib skeletons) and the two SEAM-SCOPED
  extracts called out above (hash `impls.rs`, xsd `datatype.rs`).

Everything else in this crate is byte-identical to rev 0da05783.

## Licenses

`licenses/` copied verbatim from `crates/rdf-core/licenses/`
(LICENSE-APACHE, LICENSE-MIT, LICENSE-MULAN, NOTICE.txt, CC-BY-4.0.txt,
inventory.json, third-party/unicode/0-Unicode-3.0.txt) — all byte-verified.
No new external (crates.io) dependencies were introduced: zero
license-compat.json rows to add.

## Build standing: PARTIAL — residual closure (escalation)

`cargo check` (2026-10-31, nightly-aarch64-apple-darwin, standalone workspace
root): **24 errors, all residual closure**, zero syntax/edition errors in any
vendored file. Every vendored file compiles as-is; the residual is
crate-internal modules the files reference that are OUTSIDE the slice scope:

1. `crate::cdt_blank` (term.rs:663+666 non-test, term_walk.rs:373 non-test) →
   needs cdt parser surface (CdtTerm/CdtValue/CdtContents/CdtDatatype/
   CdtError/CDT_LIST/CDT_MAP) + `purrdf_lex::terminals` (2506) +
   `purrdf_iri::terminals` (re-export; cdt_blank.rs:91-103).
2. `crate::small` RESOLVED (landed); `crate::blank_label` needs
   `purrdf_hash::hex` (809; needs hash arch/backend/dispatch — deeper hash
   closure) + `purrdf_lex::terminals` (2506; std-only top imports).
3. `crate::RdfLocation` RESOLVED by the landed diagnostic.rs, but diagnostic.rs
   itself needs `purrdf_lex::{diagnostic (491→needs lex resident.rs/variant.rs),
   json (103, std-only), hex...}` — and `purrdf_iri::{IriError (error.rs 804,
   needs iri base.rs), IriReadError (parse.rs 1387, needs idna/host/scan)}`.
4. `crate::turtle` (model.rs:281, real `Display for RdfTerm` code — needs
   turtle.rs 1128 → sink/store closure).
5. `super::dataset::TermRef` + `crate::dataset_view::{DatasetView, TermGuard}`
   + `D::Id` (term_walk.rs:41,43,532-842) → dataset.rs (5073) +
   dataset_view.rs (2987) + backend/collections/content_id/triple closure.

Every remaining sub-surface opens 2-4 more files (turtle→store, parse→idna,
hex→arch/dispatch): the build closure of the term model IS the W1 vendoring
execution (hash→lex→iri→xsd→cdt + rdf-core term-support stack). This lane drew
the line at the seam boundary and escalates the residual to the W1 queue
(campaign doc W1: "hash → branchless → deflate → lex 67.6K → iri 28.7K → cdt"
+ rdf-core-slice expansion). Joining the stardf/arrow workspace members list
and flipping to a standalone-green build are integration decisions for the
coordinator once the closure lands.

## Falsifier status

Vendoring-law falsifier ("a non-rename edit needed to build"): NOT TRIGGERED
for any vendored file — all 23 landed upstream files are byte-identical; the
five term-model files contain zero non-rename edits. The build closure is
incomplete, which is the documented, receipted escalation above — a scope
boundary, not a source edit.

## Nonclaims

- UNVERIFIED: the slice crate does not yet compile standalone (24 residual
  errors, receipted above). No `cargo test` ran; upstream `#[cfg(test)]`
  modules are present but unexercised (and several need testkit surfaces).
- UNVERIFIED: seam lib.rs skeletons and the two SEAM-SCOPED extracts are
  authored-for-slice files; their behavior is bounded by what compiles, not by
  upstream court coverage.
- UNKNOWN: whether W1 will vendor the leaves in exactly this seam shape; the
  seams are swappable placeholders whose lib names preserve the verbatim paths.
- NOT claimed: the four named types are landed verbatim, but no court/test
  witnessed their behavior in this workspace yet (Chicago: no test, no claim).
