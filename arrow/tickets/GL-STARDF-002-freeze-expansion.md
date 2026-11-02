# GL-STARDF-002 — ABI Freeze Expansion: Query/SQL Export Surface (DRAFT)

- **Status**: `DRAFT-PENDING-OPERATOR-RATIFICATION`
- **Class**: BLOCKER-CLASS (06-phase0 §8.2 / P0-19 R2) — cut FIRST, before any query export work
- **Predecessor surface**: `~/purrdf/crates/rdf-capi/include/purrdf.h`
- **Frozen surface today**: `stardf/arrow/docs/ABI-FREEZE.md` (v26.9.29, 13 `stardf_*` exports)
- **Authority**: operator decision register item 4 (`06-phase0-columnar-sql-duckdb.md` §10.4)

## 1. Purpose

The 13-export ABI-FREEZE (v26.9.29) covers only the batch/stream surface. The SQL/query
port (06-phase0 §5) requires exposing query/UPDATE/serialize exports whose carrier types
(ResultKind, QueryGovernors, GovernorEvidence, PartialCertificate, RowCursor) do not exist
in the stardf-arrow CAPI yet. Per §8.2: *cut the freeze-expansion ticket FIRST,
enumerating the query export set + carrier structs; `nm` diff vs ABI-FREEZE.md is the
enforcement.*

This ticket enumerates the candidate option set and its dependency-drag so the operator
can ratify a tier BEFORE any `#[no_mangle]` lands. Nothing here admits an export; only
operator ratification + an amended ABI-FREEZE.md does.

## 2. Census: the 21-export option set (verified against purrdf.h this session)

Enumerated mechanically from `~/purrdf/crates/fdf.h`... to correct: from
`~/purrdf/crates/rdf-capi/include/purrdf.h` (grep of declared functions, 2026-10-31):

### 2.1 Query/UPDATE family — 17 exports

Naming law: `stardf_` prefix replacing `purrdf_`; semantics preserved. The purrdf
spelling is the verified enumeration (grep against purrdf.h, 2026-10-31); the stardf
column is a mechanical `purrdf_ → stardf_` rename, PROPOSAL until ratified.

| # | stardf name (PROPOSAL) | purrdf name (VERIFIED) | family |
|---|---|---|---|
| 1 | `stardf_query` | `purrdf_query` | query base |
| 2 | `stardf_query_with_geo` | `purrdf_query_with_geo` | query + geo selector |
| 3 | `stardf_query_xpath_regex` | `purrdf_query_xpath_regex` | query + xpath-regex dialect |
| 4 | `stardf_query_json` | `purrdf_query_json` | query, JSON egress |
| 5 | `stardf_query_json_with_geo` | `purrdf_query_json_with_geo` | query + geo, JSON egress |
| 6 | `stardf_query_json_xpath_regex` | `purrdf_query_json_xpath_regex` | query + xpath-regex, JSON egress |
| 7 | `stardf_query_governed` | `purrdf_query_governed` | governed query |
| 8 | `stardf_query_governed_with_geo` | `purrdf_query_governed_with_geo` | governed + geo |
| 9 | `stardf_query_governed_xpath_regex` | `purrdf_query_governed_xpath_regex` | governed + xpath-regex |
| 10 | `stardf_query_entailment_governed` | `purrdf_query_entailment_governed` | governed + entailment |
| 11 | `stardf_query_entailment_governed_with_geo` | `purrdf_query_entailment_governed_with_geo` | governed + entailment + geo |
| 12 | `stardf_query_entailment_governed_xpath_regex` | `purrdf_query_entailment_governed_xpath_regex` | governed + entailment + xpath-regex |
| 13 | `stardf_query_governors_init` | `purrdf_query_governors_init` | governor defaults init |
| 14 | `stardf_query_canon_arrow` | `purrdf_query_canon_arrow` | query → CanonArrow egress |
| 15 | `stardf_update_governed` | `purrdf_update_governed` | SPARQL 1.1 Update (governed) |
| 16 | `stardf_update_governed_with_geo` | `purrdf_update_governed_with_geo` | update + geo |
| 17 | `stardf_update_governed_xpath_regex` | `purrdf_update_governed_xpath_regex` | update + xpath-regex |

### 2.2 Serialize family — 4 exports

| # | stardf name (PROPOSAL) | purrdf name (VERIFIED) | family |
|---|---|---|---|
| 18 | `stardf_serialize` | `purrdf_serialize` | base serialization |
| 19 | `stardf_serialize_to_callback` | `purrdf_serialize_to_callback` | streaming egress |
| 20 | `stardf_serialize_empty_named_graphs_dropped` | `purrdf_serialize_empty_named_graphs_dropped` | graph-drop variant |
| 21 | `stardf_serialize_jsonld_configured` | `purrdf_serialize_jsonld_configured` | JSON-LD configured egress |

## 3. Carrier-type drag

Each option drags carrier structs. Enumerated from purrdf.h:

| Carrier | purrdf.h ref | Dragged by | Notes |
|---|---|include|---|
| `ResultKind` enum | :558-578 | every query/update export | result discriminant |
| `QueryGovernors` struct | :1204 | governed/update-governed/entailment families | 8-dim ResourceVector settings |
| `GovernorEvidence` | :1299 | governed families | `expression_errors: uint64_t[8]` indexed by expression discriminants; appended fields are append-only per header note :248 |
| `Evidence` union member `query` | :1334 | governed families | `GovernorEvidence query` |
| `PartialCertificate` | :1317 | governed families | partial-result certificate |
| `RowCursor` | :1071 | all result-set exports (SELECT → `**out_rows`) | opaque, freed via cursor_free |
| geo/division selectors | geo_session/geo_point_index + `dataset_division_policy` | *_with_geo + division-policy exports | selectors per 06-phase0 §5 |
| CanonArrow encoder types | `query_canon_arrow` | query_canon_arrow | shares the existing CanonArrow slab encoder (06-phase0 §4) |

## 4. Option tiers

### Tier A — Minimal (3 exports + 4 carriers)

- `stardf_query` + `stardf_query_governed` + `stardf_query_governors_init` +
  lifecycle (cursor/buffer/error/free) reuse of the frozen 13.
- Carriers: ResultKind, QueryGovernors, GovernorEvidence, RowCursor.
- Does NOT cover SPARQL 1.1 Update, CanonArrow egress, geo/xpath-regex axes.
- Smallest lawful increment that lets W2 scaffold (bridge query) proceed.

 smallest; approved by writing "RATIFIED: Tier A" into this file + new ABI-FREEZE.md revision.

### Tier B — Minimal + Update + CanonArrow egress

- Tier A + `stardf_update_governed` + `stardf_query_canon_arrow`.
- Adds carrier: PartialCertificate (update atomicity certificate chain per 06-phase0 §5 GO
  on Update atomicity-tested).

### Tier C — Full 21-export parity

- All 17 query/UPDATE + 4 serialize exports (serialize: `stardf_serialize`,
  `stardf_serialize_to_callback`, `purrdf_serialize_empty_named_graphs_dropped` →
  `stardf_serialize_empty_named_graphs_dropped`, `stfrdf_serialize_jsonld_configured` —
  serialization exports, ABI-append-only per pre-1.0 rule (06-phase0 §5).

## 5. Enforcement law (until ratified)

1. **Before ratification**: NO query/UPDATE/serialize `#[no_mangle]` may land in
   `stardf/arrow/crates/stardf-arrow-capi/src/**`. Gate:
   `nm -gU target/{debug,release}/libstardf_arrow_capi.dylib | grep -c stardf_query|stardf_update|stardf_serialize` must be 0.
2. **After ratification**: the ratified tier's export list is appended to
   13+|tier| rows in a new ABI-FREEZE.md revision; enforcement is a mechanical `nm -gU`
   diff of the built dylib against the freeze doc's export table, exit 1 on any symbol
   present in the dylib but absent from the table (and vice versa, modulo the declared
   `_blake3_compress_in_place_portable` hygiene item ABI-FREEZE.md:27-29).
3. Non-conforming gate failures are REFUSED, not skipped — a gate that cannot exit 1 is
   VACUOUS and fails the court.

## 6. Verification ladder (post-ratification, per tier)

- `nm -gU` diff vs the ratified export table (this ticket §2 + §4 selection).
- Struct-layout check: `ResultKind` enum ABI, `uint64_t[8] expression_errors` fixed size,
  opaque RowCursor/opaque handle discipline unchanged from the frozen 13's ownership law.
- Golden query court: one real purrdf golden class ported (per W3 integration slice,
  06-phase0 §9) exercised through the new exports end-to-end.

## 7. Falsifiers

- F1: a query-family export lands in the dylib without a ratified tier naming it → gate
  exit 1, work REFUSED.
- F2: ratified exports land but `nm -gU` diff is never run (gate never fires) → VACUOUS.
- F3: Tier A ratified but query work drags PartialCertificate anyway → tier boundary
  violated, REFUSED (carrier drag is part of the tier contract).

## 8. Nonclaims

- The stardf-side symbol names in §2.1/§2.2 are PROPOSAL (mechanical `purrdf_ → stardf_`
  rename); the purrdf spellings are VERIFIED by grep against
  `~/purrdf/crates/rdf-capi/include/purrdf.h` (2026-10-31).
- The 17+4 partition was re-derived this session by grep against purrdf.h (14 query + 3
  update = 17; 4 serialize) and matches 06-phase0 §5's census count; purrdf.h itself is
  the authoritative enumeration if the two ever disagree.
- No export has landed; standing of the query surface is UNKNOWN until ratification + first
  `nm` diff receipt.
- This ticket admits no exports and amends no ABI-FREEZE.md; only operator ratification
  does.