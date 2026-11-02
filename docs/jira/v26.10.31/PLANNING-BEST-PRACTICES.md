# Planning Best Practices — v26.10.31 (stardf reconstitution)

## 1. What's different about this milestone

v26.10.31 (stardf) is a **reconstitution, not greenfield**: the predecessor (`~/purrdf`, 126 C ABI exports, ABI 0.16.0) and the substrate (`~/avec`, ALIVE baseline, ~310 tests passing) both carry receipted ground truth. The unverified class is different from a new domain — it is **contract drift and aspirational ontology**: claims in `stardf_contract.ttl` that have no referent in either real repo (the `ArrowArrayStream` interop surface, the 5 unmapped handle types). Planning therefore front-loads *verification against the two real repos*, not external-spec discovery. Every TTL claim cited by a wave must be grounded in purrdf.h or avec code, or marked ASPIRATIONAL and scheduled for manufacture, never treated as an existing fact.

## 2. The five-wave structure

**W1 — Contract closure.** No new capability: fix the 5 missing struct mappings (StardfTermView, StardfGraphMatch, StardfQueryGovernors, StardfGovernorEvidence, StardfPartialCertificate), extend the type-census gate to cover types as well as functions, and commit `authority/stardf/` so receipts bind to a SHA instead of the working tree. Reduction before generation.

**W2 — Scaffold (ggen-projected).** `stardf-core` + `stardf-capi` projected from the TTL via the verified marketplace packs (clap-noun-verb-crate-pack, wasi-json-abi-pack). Hand-written residue lives only in `custom_handlers.rs`, generated once, never overwritten.

**W3 — First integration slice.** `fn_avec_export_stream` → `ArrowArrayStream` producer in stardf-capi, consumer court in avec-ffi. Gate: ONE real avec integration test passing. The stream is new contract manufactured here, not reconstituted from purrdf.

**W4 — Equivalence ladder.** Port `abi_signatures.snapshot` as a header-freeze court; replay purrdf's golden-master classes (`abi.rs`, 59 tests); `c_smoke`-style real-C-host test; byte-frozen GTS vectors as oracles.

**W5 — CI + receipt.** SHA-pinned determinism court modeled on the `dsrust-reconstitution` job; fix the hardcoded `/Users/sac/purrdf` path; update `expected-workflows.txt` if a workflow file is added; `evidence/stardf/` receipts + two-run byte-identical replay.

Each wave lands working code *and* its courts together — no spec-only waves.

## 3. Discipline rules carried from the parent milestone

1. **Census parity is the gate; SHACL conformance alone is not.** The verifier must count functions AND types. A verifier that only counts what it already extracts is vacuous — the 5-type drift passed it.
2. **Generation over hand-writing.** The ontology is the contract; ggen renders; hand-written code is the irreducible residue only, disclosed in the receipt.
3. **Every court joins the anti-vacuity discipline.** A court that cannot flip is FAIL. The type census ships with a deliberately-dropped-mapping negative; the cross-allocator court ships with a mismatched-allocator mutant.
4. **Real collaborators (Chicago):** real purrdf golden tests, real C compile+link, real avec integration tests. No mocks at the FFI boundary.
5. **Oracle provenance:** oracles derive from the *contract* (purrdf.h + its frozen snapshot), never from stardf source — no circularity.
6. **Operator decisions stay PENDING.** Scope cuts (e.g., dropping bcinr work — already adjudicated: registered-non-consumer per `bcinr/AGENTS.md:351`), CTQ ratifications, and family-shift sequencing are recorded for operator sign-off.
7. **Receipts from artifact one.** The 20-lane explore wave outputs are themselves receipts; every finding carries file:line or the literal marker UNKNOWN — never guessed.

## 4. Known pitfalls to plan around

- **License skew (R1):** purrdf is MIT/Apache/MulanPSL; avec is P28-SIL sovereign. A compatibility receipt gate must exist before W3 crosses the boundary.
- **Cross-allocator mismatch (R2):** stardf-owned bytes must never route through avec's release callback. The W3 court falsifies this directly.
- **Borrowed-pointer lifetimes (R3):** purrdf rowcursor/TermView borrows die at next-advance; the equivalence tests must pin this, ideally with a debug-poison guard.
- **Alignment assumptions (R4):** `Quad64` `repr(C, align(64))` asserts exist in avec — keep them pinned; stardf's arena must allocate at the same 64-byte granularity, not merely "usually aligned".
- **Untracked authority is unreceipted authority:** W1's commit is a precondition, not a formality — every later receipt's SHA chain hangs off it.
- **Verifier host-paths:** absolute `/Users/sac/purrdf` paths make the verifier CI-unhostable; fix before W5, not during.
- **Two BLAKE3 implementations already exist in avec** (`avec-blake3` and a duplicate in `avec-arrow`) — O(N²) drift is present in the substrate; stardf must bind to one, and the drift is a registered defect, not a pattern to extend.

## 5. Definition of done for the planning phase

W1 done: all 5 struct mappings present, type census implemented and witnessed failing on a deliberately dropped mapping, `authority/stardf/` committed with a SHA, verifier passes on the committed tree. Waves 2–5 planned: each wave has a plan doc with file-ownership map, court inventory (each court with its positive-control mutant), dependency/generation route, and falsifiers — and no plan cites an ASPIRATIONAL TTL claim as an existing fact.
