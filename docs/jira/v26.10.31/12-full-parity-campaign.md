# Full-Parity 10-Wave Campaign — purrdf → stardf (v26.11.1+)

> **STATUS (2026-10-31): DEFERRED by the 80/20 re-scope** (OPERATOR-DECISIONS.md
> D15–D18, third operator block). D16 replaced this vendor-with-per-receipt-
> parity campaign with the scripted bulk copy (`scripts/bulk_copy_purrdf.py`;
> ~53 crates now on disk as stardf-*, copied **NO-COURTS** per D17 —
> compile-green at best). Parity is scoped to the four first-class targets
> (arrow landed; duckdb critical path; sql whole + 4 goldens; n3 whole +
> suite + discovery). The W1–W10 wave plan, launch precondition, standing
> budget, and Law below are superseded as campaign scope; do not dispatch
> without operator re-admission. ABI-v2 (the DoD 13-export surface) is
> PENDING-OPERATOR (D18) — arrow stays landed as-is. Deep reconciliation of
> per-claim flags is owned by the W7-class doc lane.
>
> **ADDENDUM (2026-10-31, one-package re-scope):** D15's one-package rule
> superseded this campaign's premises outright: every purrdf crate is now
> straight-COPIED into the single stardf/arrow workspace — compile-green at
> best, NO courts per D17 — so a future parity campaign does not resume the
> W1–W10 vendoring waves; it starts from copy-fidelity courts (per-crate
> sha256 of the copied tree vs git-archive of its purrdf origin HEAD), and
> only the surviving deltas earn per-receipt parity work. What survived: the
> 6-tier publish order (Tier0 leaves → Tier5 umbrella + capi), carried by
> scripts/publish_stardf.py (TIERS + strict topological gate) and by the
> crates.io Phase 2-5 plan (D16 phase ladder + D19–D21 publish-closure
> register in OPERATOR-DECISIONS.md; verdict docs/V26-11-1-VERDICT.md).

Designed from the 20-lane full-purrdf review (P1–P20, 2026-10-31). Companion to
01-master-plan + 07-port-master-plan. Each wave = 20 concurrent agents. LAUNCH
PRECONDITION: the v26.11.1 completion campaign (`wf_2130cfdc-9f1`) completes its
arbiter + coordinator c3–c6 — never two write-campaigns on one tree.

## Scope truth (from the reviews)

- Workspace: ~53 crates, ~1.9M src LOC total. 126 C exports; 18 crates back none.
- The 21-export ticket is NOT purrdf parity: it omits stardf_parse (!), 29
  entail/reasoner, 28 SHACL, 6 geo, 2 GTS, 3 projection exports (P14, P3).
- Already covered by the running campaign: leaves, SQL slice (algebra 46.8K +
  results + eval-subset), duckdb scan, avec gate, CI, falsifiers F1–F5a.
- This campaign: everything else, in dependency order, with per-receipt parity.

## W1 — Grant + leaves execution (20)
Workspace `[workspace.package].license` tri-license + `license.workspace = true`
(P20-A: bench declares narrower MIT; core/capi/root declare none); MSRV
unification (workspace rust-version; fix 1.88/1.82/absent skew — P19-B5);
SPDX headers on all stardf-authored files (P20-B: 0/10 today); layers.toml +
check-layers port (P20-O); vendoring execution in dependency order
(testkit-slice → rdf-events → stack?no → toml → xsd → rdf-core-slice expansion
→ hash → branchless → deflate → lex 67.6K → iri 28.7K → cdt) with per-crate
git-archive sha256 receipts (P20-F: G4 unmet); LICENSES/ copy (14 files incl.
PROVENANCE.md); license-inventory 7 PLANNED rows → executed + upstream-path
notice_files → stardf paths (P20-Q); `upstream-tests` feature: register
exemption or remove (P20-M no-features law).

## W2 — Kernel + algebra (20)
rdf-core Phase-2-narrow: probe-plan kernel (~1K LOC: Axis/QuadPermutation/
QuadProbePlan/QuadKey/chunk-scan) over the shim tables + KD-03 truth-table
court (P13); xsd_regex subtree (40.9K, Unicode-3.0 conjuncts + generators —
drags `stardf_query_xpath_regex`); sparql-algebra vendoring (46.8K ln, 640
tests) with identity.rs domain strings byte-verbatim (P4 §7); algebra corpus
coupling fix (queries/ + shipped-examples tests are repo-layout coupled);
VersionSpecifier/1.2 tokens carried; snapshot tests re-hosted (testkit shim).

## W3 — Results + parse/serialize (20)
stardf-sparql-results in dependency order (P6's 12-file list): 5 writers,
2 readers (json_read 2.6K / xml_read 1.3K), 13 goldens, provenance namespace
(NCName/injection-unconstructible), refusals-precede-emission, structured-
errors w/ alloc-probe; stardf-rdf codec layer: FORMS registry as data, 9
formats, blank-label alphabets (enveloped never refused), (graph,subject)
reifier key, chunk-parallel NT/NQ (1MiB/256K-4M thresholds); serialize_sink_
parity cross-product court; W3C rdf corpus round-trip (370 files, empty
allowlist); **stardf_parse export** — the Tier-A ingress gap (P3 §4.2) —
operator decision surfaced, Rust surface first.

## W4 — Eval minimal (20)
The 8 vendorable subsystems (P5): scratch+solution (promote-or-mint), bgp/,
binop, modifier determinism decisions, expr+vm operator slices, protocol.rs
(P0 coupling), plan_cache, parallel.rs; thin engine (~1–1.5K shape);
admit_version + basic_profile re-implementation; stripped governor refs
(287/150/85/71/67 sites); determinism courts vs purrdf's documented choices;
goernor binding per P0-12 if not landed by the running campaign.

## W5 — Governors + dataset courts (20)
GovernorState adapter (delta charging, sticky trip, precedence, monotone
invariant court); cardinality estimate; QuadPatternCursor; candidate_access
selectivity-division court (>25% → sequential); property_indexed_pattern_
matches_linear_scan oracle port; KD-04 boundary (compact rows internal,
Quad64 at writer only).

## W6 — avec + differential parity (20)
(If the running campaign's W6 lands the gate, this wave re-witnesses + extends.)
Differential census per family: ontology 128/22/126, header nm table, corpus
byte-parity, CanonArrow digests, license register vs cargo tree, KD ledger.

## W7 — GTS (20)
Reader-tier claim (tier 1–2 of 5, P9): 39 pairs + expected.json 12-field
oracle contract + stardf `.sha256` manifest (the ONLY independent receipt
anywhere — purrdf's gate does not police them and gmeow-gts is absent) +
frozen_canonical_bytes + frozen_identities (MMR proof + aggregate digest
9fef66c2…); writer-tier as stretch; corpus is frozen-not-regenerable
(generator absent) — recorded.

## W8 — Entail + datalog (20)
Vendor crates/datalog (39K) + crates/entail (100K incl. owl_dl 42K) —
shallow closure (6 leaves + hashbrown/rayon); OwlRl pinned numbers (78/78
rules, 136 clauses, contract hash c7c5b2b3…, ext-eq-diff-sym sole extension);
certificate grammars pinned as VECTORS (purrdf-reasoning-report 4 /
purrdf-dl-certificate 1 / purrdf-chase-proof 1); W3C 50-case RL corpus +
262-case DL corpus + owlrl==7.1.4 16-goldens + 8-entry divergence ledger
(XPASS discipline); regime.rs as the host-neutral layer.

## W9 — SHACL/geo Tier-C execution OR exclusion (20)
Operator-gated fork: (a) vendor shapes (99K)+validate pair → corpora floors
(129/129, 590/591, 401/401, 73/73, 115 dated) + SARIF byte goldens; or
(b) loud Tier-C exclusion registration (model.rs 844-line SARIF extract as
the only carried piece). Geo: 151K closure path-copy + refusal-law courts
OR exclusion. Either branch: every excluded surface registered loudly in
DIVERGENCES + ABI-FREEZE roadmap (Tier D: 31 entail/reasoner exports exist
beyond the 21-export ticket — P14).

## W10 — Adversarial + arbiter (20)
20 skeptics (one lens/family: bytes, census, ABI, licenses, receipts, flake,
vendoring purity, determinism, CI, duckdb, avec, governor, seed, docs,
dangling, counts, ladder, provenance, seam, blake3); repair round; final
ladder split; manufacturing receipt; parity verdict (structured).

## Standing budget
Vendor: ~450K LOC (leaves+algebra+results+codecs+eval-min+gts-reader+datalog
+entail). Rewrite-from-decisions: eval dispatch/engine (~30–35K). Exclude
loudly: wasm/link (~48K), projection/archive, jsonschema/text/markdown/mime/
hnsw/retrieval (unless Tier C+), geo (operator), SHACL (operator).

## Law
Vendoring law (08-vendoring-plan) + identity domains verbatim (P4) +
certificate grammars as vectors (P8) + per-crate sha256 receipts + one
campaign at a time on the tree. Every wave ends with a manufacturing receipt.
