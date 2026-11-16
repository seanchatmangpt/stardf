# ticket-001-state — GL-STARDF-001 reconstruction (lane S-3, 2026-11-01)

> Status: PARTIAL_ALIVE (RECONSTRUCTED receipt — the ticket file itself was
> recovered, so the contract below is cited, not invented; only the
> state-assessment layer is reconstructed from in-tree evidence).

## Recovery

- `tickets/GL-STARDF-001.md` was NOT in `~/stardf/tickets/` (README.md only,
  as dispatched). RECOVERED: copied read-only from
  `/Users/sac/ggen-legacy/tickets/GL-STARDF-001.md` (disclosed; source tree
  untouched). Now at `/Users/sac/stardf/tickets/GL-STARDF-001.md`.
- `GL-STARDF-002-freeze-expansion.md` was already in-tree at
  `~/stardf/arrow/tickets/GL-STARDF-002-freeze-expansion.md` (DRAFT,
  pending operator ratification). Not copied; cited only.
- `authority/stardf/stardf_contract.ttl` is NOT at the repo root — it lives
  at `~/stardf/arrow/authority/stardf/stardf_contract.ttl` (the ticket's
  path is relative to the ggen-legacy layout; divergence recorded).

## Contract (the ticket's ontology admission law)

Cited from `tickets/GL-STARDF-001.md` (recovered) + the contract TTL header
(`authority: seanchatmangpt/ggen-legacy GL-STARDF-001`):

- Model every function, data type, parameter signature, and operational
  capability from `~/purrdf` into the canonical `stardf_contract.ttl` RDF
  ontology; reconstitution proceeds from an admitted O* authority.
- Invariant bounds: zero-allocation arena model; branchless constant-time
  execution (SHACL `stardf:ExecutionInvariantsShape`: 0 conditional
  branches, 0 dynamic allocations per kernel operation); 64-byte aligned
  `TermIdArenaDefinition` binding to `ArrowArrayStream`; 100% census parity
  against `purrdf/crates/rdf-capi/include/purrdf.h`; explicit downstream
  substrate classification (`cgap`, `bcinr`, `stardf-kernel`).
- Contract binds to the legacy header by digest:
  `stardf:sourceDigestSha256 = 6ef9b4a0a9d28036b24cc702bca9113e1ab6a7e6cbb708b74ff5b11cf281ad45`
  — re-verified this session against the live
  `/Users/sac/purrdf/crates/rdf-capi/include/purrdf.h` (shasum match).
- Verification ladder (ticket): `python3 scripts/extract_purrdf_ontology.py`
  then `python3 scripts/verify_stardf_ontology.py --ttl
  authority/stardf/stardf_contract.ttl`.

## What in-tree evidence shows already-admitted

- `~/stardf/arrow/authority/stardf/stardf_contract.ttl` — 5265 lines:
  root `stardf:SubstrateContract` (MechanicalAuthority, downstream
  `cgap`), `ExecutionInvariantsShape` + `AbiFunctionShape` SHACL shapes,
  `TermIdArenaDefinition` (64B/u32/BLAKE3), 13 TypeDefinitions, and the
  full exported-C-ABI census (turnstile_admit, cgap_export_stream, plus
  the purrdf-derived family; grep counts: 130 `stardf:AbiFunction`
  occurrences ≈ 126 fn individuals + shape refs; 13 TypeDefinitions).
  Digest-bound to live purrdf.h (match confirmed this session).
- `~/stardf/arrow/ontology.ttl` — lane-14 stardf-arrow vocabulary: 13
  `#[no_mangle]` exports at cited lib.rs offsets, matched individuals for
  the 7 contract exports + 5 landed-by-W7 additions + `stardf_last_error`
  (auxiliary). Grounded in `crates/stardf-arrow-capi/src/lib.rs`.
- `~/stardf/arrow/evidence/` — 37 receipts. Ticket-001-relevant: the
  lane-16 falsifier ledger
  (`evidence/lane16-falsifier-ledger-2026-10-31.json`) — F4 header-freeze
  court WITNESSED (exit 0 check mode: "13 contract symbols == 13 header
  declarations"; shims A/B/C each flipped; sha256 fail-closed restore),
  subject head 724d5425, tree uncommitted at receipt time.
- `~/stardf/arrow/tickets/GL-STARDF-002-freeze-expansion.md` — 21-export
  expansion census enumerated against live purrdf.h (2026-10-31), DRAFT.
- `~/stardf/arrow/corpora/{n3,community}` present.

## What remains open

0. **F1 census FAILS today (witnessed 2026-11-01, this lane):**
   `python3 /Users/sac/ggen-legacy/scripts/verify_stardf_ontology.py --ttl
   arrow/authority/stardf/stardf_contract.ttl` → exit 1,
   `[FAIL] Missing type mappings:` 11 typedefs absent from the contract —
   PurrdfAbiVersion, PurrdfCapabilities, PurrdfGovernedEntailmentEvidence,
   PurrdfGovernorEvidence, PurrdfGovernorTrip, PurrdfGraphMatch,
   PurrdfPartialCertificate, PurrdfQueryGovernors, PurrdfResourceVector,
   PurrdfStr, PurrdfTermView. Graph itself parses (PASS, 4951 triples,
   128 AbiFunction instances; header census 11 opaque + 11 anonymous
   typedefs = 22). Ticket invariant 4 (100% census parity) is therefore
   NOT met by the in-tree contract as it stands.

1. **Commit gap (D1, PENDING-OPERATOR):** the whole substrate is/was
   untracked in the parent repo; receipts bind to a working-tree digest,
   not a SHA. OPERATOR-DECISIONS D1 4-commit split not yet executed in
   this tree.
2. **Verifiers missing in-tree:** `scripts/extract_purrdf_ontology.py`
   and `verify_stardf_ontology.py` are NOT under `~/stardf/` — they exist
   only in `/Users/sac/ggen-legacy/scripts/`. The ticket's verification
   ladder cannot run in-tree until copied/landed.
3. **F1 semantic type census re-run:** the DoD's F1 (22 types + 126
   functions; `--mutate drop-type` non-zero) has no witnessed in-tree run
   receipt found for the stardf_contract.ttl itself in this session.
4. **GL-STARDF-002:** DRAFT-PENDING-OPERATOR-RATIFICATION; 21-export
   expansion unratified; ABI stays 13 symbols (ABI-FREEZE.md v26.9.29).
5. **Path divergence:** contract at `arrow/authority/…`, not
   `authority/…`; the ladder commands in the ticket refer to legacy
   layout paths.

## Execution plan (admitted steps vs unstarted)

Admitted (evidence on disk this session):

- Contract TTL authored and digest-bound to live purrdf.h (sha256 match,
  `shasum -a 256` run this session, exit 0).
- F4 header-freeze court witnessed with anti-vacuity shims (lane-16
  receipt, `passed: true`, count 2).
- 13-export ABI freeze landed (ABI-FREEZE.md; lane-14 ontology.ttl names
  all 13 with lib.rs offsets).

Unstarted / open next steps (from ticket + DoD + D1):

- c1–c4 commit split (D1) so receipts bind to SHAs.
- Land `extract_purrdf_ontology.py` + `verify_stardf_ontology.py`
  in-tree; run the F1 census + `--mutate drop-type` non-vacuity against
  the in-tree contract.
- Operator ratification of GL-STARDF-002 tier; then `nm` diff vs
  amended ABI-FREEZE.md.

## Standing

PARTIAL_ALIVE — ticket recovered, contract verified digest-bound to live
purrdf.h, F4 court receipt on disk; BUT the F1 census verifier (run this
session against the in-tree contract) exits 1 with 11 missing type
mappings, and the commit split is pending operator. No admission claims invented: every
admitted item above cites an on-disk receipt or a command run this
session (see lane report).
