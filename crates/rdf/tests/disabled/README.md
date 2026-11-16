# stardf-rdf parked upstream tests: unpark preconditions

`gts_composite_compaction.rs` is PARKED, not deleted: byte-verbatim from purrdf
rev `0da0578306983da851a9073337d57e2e2c41ab3a` (diff-witnessed byte-identical
2026-10-31, lane 13). It is inert here because cargo does not auto-discover
targets in nested directories. Zero source edits were made to park it.

| file | blocked on | upstream witness at pinned rev |
| --- | --- | --- |
| gts_composite_compaction.rs | upstream mid-refactor: `Writer::add_terms`/`add_quads`/`sign_with`/`add_index`/`into_bytes` resolve to `M::FrameResult = Result<Vec<u8>, WriterError>` under the streamable layout, but the test uses the bare builder style; identical E0599 witnessed upstream (`cargo check -p purrdf-rdf --tests`, 2026-10-31) | upstream purrdf fails identically at the pinned rev |

Unpark when upstream (or an operator-directed port fix) reconciles the test
with the `FrameResult` API.
