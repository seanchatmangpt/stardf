/*
 * stardf v26.11.1 full-parity campaign — ultracode Workflow script.
 *
 * Shape: 10 waves x 20 concurrent lanes per wave (200 lane agents + 1 arbiter).
 * Source of truth for wave content: stardf/docs/jira/v26.10.31/12-full-parity-campaign.md
 * ("Each wave = 20 concurrent agents"; W10 names its 20 skeptic lenses verbatim).
 * Lane slicing below is that doc's per-wave work items cut into disjoint file-ownership
 * lanes; agents ground file-level specifics on disk before editing.
 *
 * LAUNCH PRECONDITION (12-full-parity-campaign.md): the v26.11.1 completion campaign
 * (wf_2130cfdc-9f1) completes its arbiter + coordinator c3-c6 first — never two
 * write-campaigns on one tree. Enforced in-script as a gate on the W1-L20 lane result:
 * if the gate lane returns standing BLOCKED, waves 2-10 are NOT dispatched and the run
 * returns a BLOCKED receipt (loud, disclosed). args.force = true skips the gate
 * (coordinator's explicit call).
 *
 * CONCURRENCY DISCLOSURE (no overclaiming): every wave submits all 20 lanes as one
 * parallel() barrier — 20 in flight per wave by construction. The Workflow runtime
 * caps physically-running agents per workflow at min(16, available CPUs - 2); on the
 * 16-CPU operator machine that is 14 running + 6 queued per wave. No script can raise
 * that cap; waves are sequential barriers, lanes concurrent.
 *
 * Launch (main session, Workflow tool):
 *   Workflow({ scriptPath: "/Users/sac/ggen-legacy/stardf/workflows/v26.11.1-full-parity.workflow.js",
 *              args: { run: "<label>", operatorFork: "vendor" | "exclude" | undefined,
 *                      force: false } })
 * Resume: relaunch with the same {scriptPath, args} + resumeFromRunId — the longest
 * unchanged prefix of agent() calls replays from journal.jsonl.
 *
 * args:
 *   run           — run label stamped into every receipt (no Date.now() in scripts).
 *   operatorFork  — W9 branch: "vendor" (SHACL shapes+validate) | "exclude" (loud
 *                   Tier-C exclusion registration). Unset => W9 fork lanes loud-skip.
 *   force         — skip the W1 launch-precondition gate (default false).
 */

export const meta = {
  name: 'stardf-v26111-full-parity',
  description: 'stardf full-parity: 10 waves x 20 concurrent lanes, D9-D14 contract, W10 adversarial arbiter',
  phases: [
    { title: 'W1 - Grant + leaves execution', detail: 'tri-license, MSRV, SPDX, layers, vendoring in dependency order, license receipts, launch gate' },
    { title: 'W2 - Kernel + algebra', detail: 'probe-plan kernel + KD-03 court, xsd_regex, sparql-algebra vendoring + identity domains' },
    { title: 'W3 - Results + parse/serialize', detail: 'sparql-results writers/readers/goldens, rdf codec FORMS, W3C round-trip, stardf_parse Tier-A' },
    { title: 'W4 - Eval minimal', detail: '8 vendorable eval subsystems, thin engine, governor-ref strips, determinism courts' },
    { title: 'W5 - Governors + dataset courts', detail: 'GovernorState adapter, cardinality, cursors, selectivity/oracle courts, bridge-edge seam' },
    { title: 'W6 - avec + differential parity', detail: 're-witness avec gate + differential census per family vs purrdf' },
    { title: 'W7 - GTS', detail: 'reader-tier 1-2, 39-pair oracle contract, sha256 manifest, frozen canonical bytes/identities' },
    { title: 'W8 - Entail + datalog', detail: 'datalog+entail vendoring, OwlRl pinned numbers, certificate vectors, W3C corpora' },
    { title: 'W9 - SHACL/geo Tier-C fork', detail: 'operator-gated: vendor shapes+validate w/ corpora floors OR loud Tier-C exclusion' },
    { title: 'W10 - Adversarial + arbiter', detail: '20 skeptics (one lens each) + arbiter verdict + repair work orders' },
  ],
}

// ---------------------------------------------------------------- constants

const REPO = '/Users/sac/ggen-legacy'
const RUN = args && args.run ? String(args.run) : 'unlabeled-run'
const FORK = args && args.operatorFork ? String(args.operatorFork) : null
const FORCE = !!(args && args.force)
const CAMPAIGN_DOC = REPO + '/stardf/docs/jira/v26.10.31/12-full-parity-campaign.md'
const DOC_DIR = REPO + '/stardf/docs/jira/v26.10.31/'
const ARROW_DOCS = REPO + '/stardf/arrow/docs/'

const CONTRACT = [
  'LANE CONTRACT (stardf v26.11.1 completion+validation; ' + REPO + '/):',
  '- Operator signed D9-D14 (' + ARROW_DOCS + 'OPERATOR-DECISIONS.md, 2026-10-31 block). R1 ADMITTED+GREEN (license gate exit 0). Commits: c1=26d0140, c2=98afd71 on fix/v26.9.29-ci-gates. c3-c6 = coordinator-only, post-campaign. NO git state-changing commands from any lane (read-only status/log/rev-parse ok).',
  '- One canonical checkout; disjoint file ownership per lane label; shared-seam edits = disclosed compile-freeze unblocks; re-read before editing (other lanes run concurrently).',
  '- Chicago: real subprocesses/tests, no mocks; a court that cannot flip is FAIL; skips loud ([SKIP: reason] + nonzero). Receipts only from commands YOU ran; cite ' + DOC_DIR + '*.md and ' + ARROW_DOCS + '* for everything else.',
  '- Vendoring law: source = purrdf rev 0da05783 via "git -C /Users/sac/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a <path>" (never the dirty tree); rename packages purrdf-* -> stardf-* ONLY; lib names, LICENSE files, in-file notices verbatim; a NON-rename edit needed to build = falsifier - escalate, never fix silently. New external deps: observe license from ~/.cargo/registry, add dependency_licenses + compatibility_receipts entries in authority/stardf/license-compat.json (Apache/MIT-arm basis, operator blanket sign-off 2026-10-31), disclose.',
  '- ABI: 13 exports frozen + D9 Tier A RATIFIED (stardf_query, stardf_query_governed, stardf_query_governors_init + carriers may be implemented as Rust surface; C exports land only via the ratified ticket process). Header: stardf/crates/stardf-arrow-capi/include/stardf_arrow.h.',
  '- Bridge edge (W4<->W5 seam): SolutionTerm::join_key + ScratchId bit-32 disjointness. Both sides implement to this seam; neither lane edits the other side\'s files.',
].join('\n')

const RETURN_SPEC =
  'Return ONLY the structured object your schema forces: lane id, standing ' +
  '(ALIVE | PARTIAL_ALIVE | UNKNOWN | BLOCKED | REFUSED | UNSUPPORTED), one-line summary, ' +
  'files you touched or produced, commands you ran with real exit codes, nonclaims ' +
  '(everything you did NOT verify), blockers. Your final message is data for a script, not prose.'

const SCHEMA = {
  type: 'object',
  properties: {
    lane: { type: 'string', description: 'lane id, e.g. w1-l05' },
    standing: { type: 'string', enum: ['ALIVE', 'PARTIAL_ALIVE', 'UNKNOWN', 'BLOCKED', 'REFUSED', 'UNSUPPORTED'] },
    summary: { type: 'string' },
    files: { type: 'array', items: { type: 'string' } },
    commands: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          cmd: { type: 'string' },
          exit: { type: 'number' },
          note: { type: 'string' },
        },
      },
    },
    verdict: { type: 'string', enum: ['CONFIRMED', 'REFUTED', 'SKIP'], description: 'skeptic lanes only' },
    findings: { type: 'array', items: { type: 'string' } },
    nonclaims: { type: 'array', items: { type: 'string' } },
    blockers: { type: 'array', items: { type: 'string' } },
  },
  required: ['lane', 'standing', 'summary', 'files', 'commands', 'nonclaims'],
}

// ---------------------------------------------------------------- wave specs
// Each lane: id, lens (short name), own (disjoint file-ownership statement),
// task (grounded in 12-full-parity-campaign.md section of the same wave).

const WAVES = [
  {
    title: 'W1 - Grant + leaves execution',
    lanes: [
      { id: 'w1-l01', lens: 'tri-license', own: 'workspace Cargo.toml [workspace.package].license + crates/*/Cargo.toml license keys', task: 'Workspace tri-license: set [workspace.package].license + license.workspace = true; fix bench (declares narrower MIT) and core/capi (declare none) per P20-A.' },
      { id: 'w1-l02', lens: 'MSRV', own: 'rust-version keys across workspace Cargo.toml files', task: 'MSRV unification: workspace-level rust-version; fix the 1.88 / 1.82 / absent skew (P19-B5).' },
      { id: 'w1-l03', lens: 'SPDX headers', own: 'license-header lines of existing stardf-authored .rs files (not vendored crates)', task: 'SPDX headers on all stardf-authored files (P20-B: 0/10 today). Do not touch vendored crate bodies.' },
      { id: 'w1-l04', lens: 'layers gate', own: 'layers.toml + check-layers script port', task: 'layers.toml + check-layers port (P20-O) over the crate inventory.' },
      { id: 'w1-l05', lens: 'vendor testkit-slice', own: 'crates/stardf-testkit', task: 'Vendor testkit slice per vendoring law + 08-vendoring-plan.md; per-crate git-archive sha256 receipt.' },
      { id: 'w1-l06', lens: 'vendor rdf-events', own: 'crates/stardf-events', task: 'Vendor rdf-events (rdf crate-events) per vendoring law.' },
      { id: 'w1-l07', lens: 'vendor toml', own: 'toml leaf crate', task: 'Vendor toml leaf per vendoring law.' },
      { id: 'w1-l08', lens: 'vendor xsd', own: 'crates/stardf-xsd', task: 'Vendor xsd leaf per vendoring law.' },
      { id: 'w1-l09', lens: 'vendor rdf-core-slice', own: 'rdf-core slice expansion crate', task: 'Expand rdf-core slice vendoring per 08-vendoring-plan.md.' },
      { id: 'w1-l10', lens: 'vendor hash', own: 'crates/stardf-hash', task: 'Vendor hash leaf per vendoring law.' },
      { id: 'w1-l11', lens: 'vendor branchless', own: 'branchless leaf crate', task: 'Vendor branchless leaf per vendoring law.' },
      { id: 'w1-l12', lens: 'vendor deflate', own: 'deflate leaf crate', task: 'Vendor deflate leaf per vendoring law.' },
      { id: 'w1-l13', lens: 'vendor lex', own: 'crates/stardf-lex', task: 'Vendor lex (67.6K) per vendoring law.' },
      { id: 'w1-l14', lens: 'vendor iri', own: 'crates/stardf-iri', task: 'Vendor iri (28.7K) per vendoring law.' },
      { id: 'w1-l15', lens: 'vendor cdt', own: 'cdt leaf crate', task: 'Vendor cdt leaf per vendoring law.' },
      { id: 'w1-l16', lens: 'LICENSES copy', own: 'LICENSES/ directory', task: 'LICENSES/ copy: 14 files incl. PROVENANCE.md, verbatim.' },
      { id: 'w1-l17', lens: 'license-inventory', own: 'authority/stardf/license-inventory.toml + license-compat.json rows', task: 'license-inventory 7 PLANNED rows -> executed; notice_files upstream paths -> stardf paths (P20-Q).' },
      { id: 'w1-l18', lens: 'upstream-tests feature', own: 'upstream-tests feature declaration + exemption register', task: 'upstream-tests feature: register exemption or remove (P20-M no-features law).' },
      { id: 'w1-l19', lens: 'receipts', own: 'stardf/arrow/evidence/ wave-receipt files for W1', task: 'Assemble the W1 wave receipt from the other 19 lanes\' structured returns; every claim receipted to a command you ran or a doc path.' },
      { id: 'w1-l20', lens: 'LAUNCH GATE', own: 'no file edits - read-only census', task: 'Launch-precondition gate (read-only): confirm no second write-campaign is in flight on this tree; confirm c1/c2 SHAs on fix/v26.9.29-ci-gates via git rev-parse; confirm R1 license gate state on disk. If the v26.11.1 completion campaign c3-c6 are still pending and lanes would write, return standing BLOCKED with blockers[] naming each. READ-ONLY: you edit nothing.' },
    ],
  },
  {
    title: 'W2 - Kernel + algebra',
    lanes: [
      { id: 'w2-l01', lens: 'kernel Axis', own: 'kernel Axis module', task: 'rdf-core Phase-2-narrow probe-plan kernel: Axis (~1K LOC kernel slice) over the shim tables.' },
      { id: 'w2-l02', lens: 'kernel QuadPermutation', own: 'kernel QuadPermutation module', task: 'QuadPermutation slice of the probe-plan kernel.' },
      { id: 'w2-l03', lens: 'kernel QuadProbePlan', own: 'kernel QuadProbePlan module', task: 'QuadProbePlan slice of the probe-plan kernel.' },
      { id: 'w2-l04', lens: 'kernel QuadKey', own: 'kernel QuadKey module', task: 'QuadKey slice of the probe-plan kernel.' },
      { id: 'w2-l05', lens: 'kernel chunk-scan', own: 'kernel chunk-scan module', task: 'chunk-scan slice of the probe-plan kernel.' },
      { id: 'w2-l06', lens: 'KD-03 court', own: 'KD-03 truth-table court file(s)', task: 'KD-03 truth-table court over the kernel (P13). Court must be able to flip (anti-vacuity).' },
      { id: 'w2-l07', lens: 'xsd_regex conjuncts', own: 'xsd_regex Unicode-3.0 conjunct subtree', task: 'xsd_regex subtree (40.9K): Unicode-3.0 conjuncts + generators; drags stardf_query_xpath_regex.' },
      { id: 'w2-l08', lens: 'xsd_regex generators', own: 'xsd_regex generator/test subtree', task: 'xsd_regex generators + property tests, re-hosted to stardf naming.' },
      { id: 'w2-l09', lens: 'xpath_regex linkage', own: 'stardf_query_xpath_regex crate linkage', task: 'Wire stardf_query_xpath_regex to the vendored xsd_regex subtree.' },
      { id: 'w2-l10', lens: 'algebra identity domains', own: 'sparql-algebra identity.rs', task: 'Vendor sparql-algebra (46.8K ln, 640 tests); identity.rs domain strings byte-verbatim (P4 section 7).' },
      { id: 'w2-l11', lens: 'algebra corpus coupling', own: 'sparql-algebra queries/ + shipped-examples tests', task: 'Fix algebra corpus coupling: queries/ + shipped-examples tests are repo-layout coupled.' },
      { id: 'w2-l12', lens: 'VersionSpecifier 1.2', own: 'VersionSpecifier + 1.2 token modules', task: 'VersionSpecifier / 1.2 tokens carried through the algebra vendoring.' },
      { id: 'w2-l13', lens: 'algebra snapshots', own: 'snapshot test files', task: 'Snapshot tests re-hosted (testkit shim).' },
      { id: 'w2-l14', lens: 'algebra parser/ast', own: 'sparql-algebra parser/ast slice', task: 'sparql-algebra parser/ast slice vendoring.' },
      { id: 'w2-l15', lens: 'algebra serialize', own: 'sparql-algebra serialize slice', task: 'sparql-algebra serialize slice vendoring.' },
      { id: 'w2-l16', lens: '640-test court', own: 'algebra test-court harness', task: '640-test algebra court: real cargo test over the vendored crate; record pass/fail counts.' },
      { id: 'w2-l17', lens: 'algebra determinism', own: 'algebra determinism court files', task: 'Determinism court vs purrdf documented algebra choices; two-run byte-identical.' },
      { id: 'w2-l18', lens: 'KD-04 boundary', own: 'KD-04 boundary registration + enforcement notes', task: 'KD-04 boundary: compact rows internal, Quad64 at writer only - register + first enforcement court.' },
      { id: 'w2-l19', lens: 'receipts', own: 'W2 wave receipt in stardf/arrow/evidence/', task: 'Assemble the W2 wave receipt from lanes w2-l01..l18.' },
      { id: 'w2-l20', lens: 'skeptic', own: 'read-only audit of W2 lanes\' diffs', task: 'Skeptic over W2: audit the other 19 lanes\' diffs and claims. Try to refute; verdict CONFIRMED/REFUTED per finding. READ-ONLY.' },
    ],
  },
  {
    title: 'W3 - Results + parse/serialize',
    lanes: [
      { id: 'w3-l01', lens: 'results writers s-expr/json', own: 'stardf-sparql-results s-expr + json writers', task: 'stardf-sparql-results writers (s-expr, json) per P6 12-file list.' },
      { id: 'w3-l02', lens: 'results writers xml/csv', own: 'stardf-sparql-results xml + csv/tsv writers', task: 'stardf-sparql-results writers (xml, csv/tsv).' },
      { id: 'w3-l03', lens: 'results json_read', own: 'json_read module (2.6K)', task: 'json_read reader vendoring.' },
      { id: 'w3-l04', lens: 'results xml_read', own: 'xml_read module (1.3K)', task: 'xml_read reader vendoring.' },
      { id: 'w3-l05', lens: 'results goldens', own: '13 goldens + .sha256 manifest', task: '13 goldens + provenance manifest for the results crate.' },
      { id: 'w3-l06', lens: 'provenance ns + errors', own: 'provenance namespace + structured-errors modules', task: 'Provenance namespace (NCName/injection-unconstructible), refusals-precede-emission, structured errors w/ alloc-probe.' },
      { id: 'w3-l07', lens: 'FORMS registry', own: 'rdf codec FORMS registry', task: 'stardf-rdf codec layer: FORMS registry as data.' },
      { id: 'w3-l08', lens: '9 formats', own: 'codec format dispatch', task: 'Codec dispatch over 9 formats.' },
      { id: 'w3-l09', lens: 'blank-label alphabets', own: 'blank-label alphabet module', task: 'Blank-label alphabets: enveloped never refused.' },
      { id: 'w3-l10', lens: 'reifier key', own: '(graph,subject) reifier key module', task: '(graph,subject) reifier key.' },
      { id: 'w3-l11', lens: 'chunk-parallel NT/NQ', own: 'NT/NQ chunk-parallel writers', task: 'Chunk-parallel NT/NQ (1MiB/256K-4M thresholds).' },
      { id: 'w3-l12', lens: 'serialize_sink court', own: 'serialize_sink parity court', task: 'serialize_sink_ parity cross-product court; must flip.' },
      { id: 'w3-l13', lens: 'W3C round-trip', own: 'W3C rdf corpus round-trip harness', task: 'W3C rdf corpus round-trip (370 files, empty allowlist).' },
      { id: 'w3-l14', lens: 'stardf_parse surface', own: 'stardf_parse export surface', task: 'stardf_parse export - the Tier-A ingress gap (P3 4.2). Operator decision surfaced, Rust surface first (D9).' },
      { id: 'w3-l14b', lens: 'stardf_parse court', own: 'stardf_parse court files', task: 'Court for stardf_parse: real parse courts, flipable.' },
      { id: 'w3-l15', lens: 'alloc-probe courts', own: 'alloc-probe court files for results/codec', task: 'alloc-probe courts over results + codec paths.' },
      { id: 'w3-l16', lens: 'seed court', own: 'seed/determinism court files', task: 'Seed court: any nondeterminism in results/codec surfaced via two-run byte compare.' },
      { id: 'w3-l17', lens: 'census', own: 'read-only census of W3 surface', task: 'Census lane: count writers/readers/formats/goldens actually landed vs P6 list. READ-ONLY.' },
      { id: 'w3-l18', lens: 'docs', own: 'stardf/arrow/docs/ W3-slice docs + DIVERGENCES rows', task: 'Docs: DIVERGENCES + ABI-FREEZE roadmap rows for anything W3 carried differently.' },
      { id: 'w3-l19', lens: 'receipts', own: 'W3 wave receipt', task: 'Assemble the W3 wave receipt from lanes w3-l01..l18.' },
      { id: 'w3-l20', lens: 'skeptic', own: 'read-only audit of W3 diffs', task: 'Skeptic over W3: refute what you can. READ-ONLY.' },
    ],
  },
  {
    title: 'W4 - Eval minimal',
    lanes: [
      { id: 'w4-l01', lens: 'scratch+solution', own: 'scratch + solution modules (SolutionTerm)', task: 'Vendor scratch+solution (promote-or-mint). OWNS the bridge edge SolutionTerm::join_key half; ScratchId bit-32 disjointness is the seam contract with W5.' },
      { id: 'w4-l02', lens: 'bgp', own: 'bgp/ eval module', task: 'Vendor bgp/ eval slice.' },
      { id: 'w4-l03', lens: 'binop', own: 'binop eval module', task: 'Vendor binop slice.' },
      { id: 'w4-l04', lens: 'modifier determinism', own: 'modifier modules', task: 'Modifier determinism decisions documented + courted.' },
      { id: 'w4-l05', lens: 'expr', own: 'expr operator slice', task: 'expr operator slice vendoring.' },
      { id: 'w4-l06', lens: 'vm', own: 'vm operator slice', task: 'vm operator slice vendoring.' },
      { id: 'w4-l07', lens: 'protocol.rs', own: 'protocol.rs', task: 'protocol.rs vendoring (P0 coupling surface).' },
      { id: 'w4-l08', lens: 'plan_cache', own: 'plan_cache module', task: 'plan_cache vendoring.' },
      { id: 'w4-l09', lens: 'parallel.rs', own: 'parallel.rs', task: 'parallel.rs vendoring.' },
      { id: 'w4-l10', lens: 'thin engine', own: 'engine dispatch module', task: 'Thin engine (~1-1.5K shape) binding the vendored slices.' },
      { id: 'w4-l11', lens: 'admit_version', own: 'admit_version module', task: 'admit_version re-implementation per documented decisions.' },
      { id: 'w4-l12', lens: 'basic_profile', own: 'basic_profile module', task: 'basic_profile re-implementation.' },
      { id: 'w4-l13', lens: 'governor-ref strip 287', own: 'strip sites bucket 1', task: 'Stripped governor refs, bucket 1 of the 287/150/85/71/67 site counts.' },
      { id: 'w4-l14', lens: 'governor-ref strip 150/85', own: 'strip sites bucket 2', task: 'Stripped governor refs, buckets 2+3.' },
      { id: 'w4-l15', lens: 'governor-ref strip 71/67', own: 'strip sites bucket 3', task: 'Stripped governor refs, buckets 4+5.' },
      { id: 'w4-l16', lens: 'determinism courts', own: 'eval determinism court files', task: 'Determinism courts vs purrdf documented choices across the eval slice.' },
      { id: 'w4-l17', lens: 'ScratchId seam', own: 'ScratchId module (bit-32 disjointness)', task: 'ScratchId bit-32 disjointness half of the bridge edge; seam contract with w4-l01 (SolutionTerm::join_key). Neither side edits the other\'s file.' },
      { id: 'w4-l18', lens: 'governor binding P0-12', own: 'governor binding module', task: 'Governor binding per P0-12 if not landed by the running campaign - check first, loud-skip if already landed.' },
      { id: 'w4-l19', lens: 'receipts', own: 'W4 wave receipt', task: 'Assemble the W4 wave receipt from lanes w4-l01..l18.' },
      { id: 'w4-l20', lens: 'skeptic', own: 'read-only audit of W4 diffs', task: 'Skeptic over W4, incl. the bridge-edge seam: join_key + ScratchId disjointness holds across both modules. READ-ONLY.' },
    ],
  },
  {
    title: 'W5 - Governors + dataset courts',
    lanes: [
      { id: 'w5-l01', lens: 'GovernorState delta', own: 'GovernorState delta-charging module', task: 'GovernorState adapter: delta charging.' },
      { id: 'w5-l02', lens: 'sticky trip', own: 'sticky-trip module', task: 'GovernorState adapter: sticky trip.' },
      { id: 'w5-l03', lens: 'precedence', own: 'precedence module', task: 'GovernorState adapter: precedence.' },
      { id: 'w5-l04', lens: 'monotone court', own: 'monotone invariant court', task: 'Monotone invariant court over the governor adapter; must flip.' },
      { id: 'w5-l05', lens: 'cardinality', own: 'cardinality estimate module', task: 'Cardinality estimate vendoring.' },
      { id: 'w5-l06', lens: 'QuadPatternCursor', own: 'QuadPatternCursor module', task: 'QuadPatternCursor vendoring.' },
      { id: 'w5-l07', lens: 'selectivity court', own: 'candidate_access court', task: 'candidate_access selectivity-division court (>25% -> sequential); must flip.' },
      { id: 'w5-l08', lens: 'oracle port', own: 'property_indexed oracle module', task: 'property_indexed_pattern_matches_linear_scan oracle port.' },
      { id: 'w5-l09', lens: 'KD-04 enforcement', own: 'KD-04 boundary court files', task: 'KD-04 boundary enforcement courts: compact rows internal, Quad64 at writer only.' },
      { id: 'w5-l10', lens: 'ScratchId seam half', own: 'ScratchId consumer side', task: 'Bridge edge from the dataset side: ScratchId bit-32 disjointness consumed per seam contract; do not edit w4-l01 files.' },
      { id: 'w5-l11', lens: 'governor ABI carriers', own: 'governor ABI carrier files', task: 'D9 Tier A: governor ABI carriers as Rust surface; no new C exports outside the ratified ticket process.' },
      { id: 'w5-l12', lens: 'dataset court 1', own: 'dataset court file group 1', task: 'Dataset court group 1 per W5 spec.' },
      { id: 'w5-l13', lens: 'dataset court 2', own: 'dataset court file group 2', task: 'Dataset court group 2.' },
      { id: 'w5-l14', lens: 'dataset court 3', own: 'dataset court file group 3', task: 'Dataset court group 3.' },
      { id: 'w5-l15', lens: 'dataset court 4', own: 'dataset court file group 4', task: 'Dataset court group 4.' },
      { id: 'w5-l16', lens: 'dataset court 5', own: 'dataset court file group 5', task: 'Dataset court group 5.' },
      { id: 'w5-l17', lens: 'differential census', own: 'read-only census vs purrdf', task: 'Differential census vs purrdf over governor/dataset surface. READ-ONLY.' },
      { id: 'w5-l18', lens: 'CI hooks', own: 'CI job block for governor/dataset gates', task: 'Wire W5 gates into the CI job block per OPERATOR-DECISIONS D2 Option A.' },
      { id: 'w5-l19', lens: 'receipts', own: 'W5 wave receipt', task: 'Assemble the W5 wave receipt from lanes w5-l01..l18.' },
      { id: 'w5-l20', lens: 'skeptic', own: 'read-only audit of W5 diffs', task: 'Skeptic over W5 + the W4<->W5 bridge edge. READ-ONLY.' },
    ],
  },
  {
    title: 'W6 - avec + differential parity',
    lanes: [
      { id: 'w6-l01', lens: 'census ontology', own: 'ontology census receipt', task: 'Differential census: ontology 128/22/126 vs purrdf.' },
      { id: 'w6-l02', lens: 'census header nm', own: 'header nm-table census receipt', task: 'Differential census: header nm table vs purrdf.' },
      { id: 'w6-l03', lens: 'census corpus bytes', own: 'corpus byte-parity receipt', task: 'Differential census: corpus byte-parity.' },
      { id: 'w6-l04', lens: 'census CanonArrow', own: 'CanonArrow digest census receipt', task: 'Differential census: CanonArrow digests.' },
      { id: 'w6-l05', lens: 'census licenses', own: 'license-register census receipt', task: 'Differential census: license register vs cargo tree.' },
      { id: 'w6-l06', lens: 'census KD ledger', own: 'KD-ledger census receipt', task: 'Differential census: KD ledger.' },
      { id: 'w6-l07', lens: 'stream_courts re-witness', own: 'stream courts', task: 'Re-witness stream_courts 6/6; extend if the running campaign landed the gate.' },
      { id: 'w6-l08', lens: 'cross_allocator', own: 'cross-allocator court', task: 'Re-witness cross_allocator flip.' },
      { id: 'w6-l09', lens: 'c_smoke', own: 'c_smoke test files', task: 'Re-witness c_smoke port (nested cargo -> cc -> link -> run).' },
      { id: 'w6-l10', lens: 'abi_signatures', own: 'abi_signatures canonicalization', task: 'abi_signatures port: verbatim canonicalization, parameterized purrdf_ prefix, stardf snapshot regenerated from contract targetSymbols.' },
      { id: 'w6-l11', lens: 'GTS-vector port', own: 'GTS-vector port files', task: 'GTS-vector port + stardf-side .sha256 manifest (purrdf gate does not police them).' },
      { id: 'w6-l12', lens: 'crate-family parity 1', own: 'parity court family 1', task: 'Differential parity court, crate family 1.' },
      { id: 'w6-l13', lens: 'crate-family parity 2', own: 'parity court family 2', task: 'Differential parity court, crate family 2.' },
      { id: 'w6-l14', lens: 'crate-family parity 3', own: 'parity court family 3', task: 'Differential parity court, crate family 3.' },
      { id: 'w6-l15', lens: 'crate-family parity 4', own: 'parity court family 4', task: 'Differential parity court, crate family 4.' },
      { id: 'w6-l16', lens: 'crate-family parity 5', own: 'parity court family 5', task: 'Differential parity court, crate family 5.' },
      { id: 'w6-l17', lens: 'CI witness', own: 'CI run receipts', task: 'CI witness: the CI draft has never run in GitHub Actions (UNVERIFIED beyond local) - get a real Actions run receipt or record BLOCKED.' },
      { id: 'w6-l18', lens: 'docs', own: 'parity docs + errata rows', task: 'Docs: release errata rows (header path, delta-claim provenance) into DIVERGENCES.' },
      { id: 'w6-l19', lens: 'receipts', own: 'W6 wave receipt', task: 'Assemble the W6 wave receipt.' },
      { id: 'w6-l20', lens: 'skeptic', own: 'read-only audit of W6', task: 'Skeptic over W6 censuses + avec courts. READ-ONLY.' },
    ],
  },
  {
    title: 'W7 - GTS',
    lanes: [
      { id: 'w7-l01', lens: 'oracle contract', own: '12-field oracle contract spec', task: 'Reader-tier claim: 12-field expected.json oracle contract spec (tier 1-2 of 5, P9).' },
      { id: 'w7-l02', lens: 'sha256 manifest', own: 'stardf gts .sha256 manifest', task: 'stardf .sha256 manifest over the 39 pairs - the ONLY independent receipt anywhere for this corpus.' },
      { id: 'w7-l03', lens: 'frozen canonical bytes', own: 'frozen_canonical_bytes port', task: 'frozen_canonical_bytes test port (crates/gts/tests precedent, :14-38).' },
      { id: 'w7-l04', lens: 'frozen identities', own: 'frozen_identities port', task: 'frozen_identities: MMR proof + aggregate digest 9fef66c2... port.' },
      { id: 'w7-l05', lens: 'reader tier 1', own: 'gts reader tier-1 code', task: 'Reader tier 1 implementation.' },
      { id: 'w7-l06', lens: 'reader tier 2', own: 'gts reader tier-2 code', task: 'Reader tier 2 implementation.' },
      { id: 'w7-l07', lens: 'pair parity 1-10', own: 'pair-parity court batch 1', task: 'Byte-parity courts over pairs 1-10.' },
      { id: 'w7-l08', lens: 'pair parity 11-20', own: 'pair-parity court batch 2', task: 'Byte-parity courts over pairs 11-20.' },
      { id: 'w7-l09', lens: 'pair parity 21-30', own: 'pair-parity court batch 3', task: 'Byte-parity courts over pairs 21-30.' },
      { id: 'w7-l10', lens: 'pair parity 31-39', own: 'pair-parity court batch 4', task: 'Byte-parity courts over pairs 31-39.' },
      { id: 'w7-l11', lens: 'writer-tier stretch', own: 'gts writer code (stretch)', task: 'Writer-tier as stretch; loud-skip if reader tiers are not green.' },
      { id: 'w7-l12', lens: 'corpus provenance', own: 'corpus frozen-not-regenerable record', task: 'Record: corpus is frozen-not-regenerable (generator absent).' },
      { id: 'w7-l13', lens: 'alloc courts', own: 'gts alloc court files', task: 'alloc-probe courts over the gts reader.' },
      { id: 'w7-l14', lens: 'API surface', own: 'gts cross-crate API surface', task: 'Cross-crate gts API surface (purrdf-gts reader precedent).' },
      { id: 'w7-l15', lens: 'purrdf-side census', own: 'read-only purrdf census citation', task: 'purrdf-side census citations via git archive rev 0da05783. READ-ONLY.' },
      { id: 'w7-l16', lens: 'ABI delta', own: 'header delta check', task: 'Header/ABI delta check for gts surface vs 13-export freeze.' },
      { id: 'w7-l17', lens: 'docs', own: 'gts docs + DIVERGENCES rows', task: 'Docs for the gts slice.' },
      { id: 'w7-l18', lens: 'roots manifest', own: 'roots.toml equivalent', task: 'stardf roots manifest equivalent (purrdf check-corpus-frozen.py does NOT police the gts vectors).' },
      { id: 'w7-l19', lens: 'receipts', own: 'W7 wave receipt', task: 'Assemble the W7 wave receipt.' },
      { id: 'w7-l20', lens: 'skeptic', own: 'read-only audit of W7', task: 'Skeptic over W7. READ-ONLY.' },
    ],
  },
  {
    title: 'W8 - Entail + datalog',
    lanes: [
      { id: 'w8-l01', lens: 'datalog vendor', own: 'crates/stardf-datalog', task: 'Vendor crates/datalog (39K); shallow closure (6 leaves + hashbrown/rayon).' },
      { id: 'w8-l02', lens: 'entail base', own: 'entail base slice', task: 'Vendor entail base slice (excl. owl_dl).' },
      { id: 'w8-l03', lens: 'owl_dl', own: 'owl_dl module (42K)', task: 'Vendor owl_dl 42K.' },
      { id: 'w8-l04', lens: 'entail leaves', own: 'entail closure leaves', task: 'Entail closure leaves (6 leaves + hashbrown/rayon).' },
      { id: 'w8-l05', lens: 'OwlRl numbers', own: 'OwlRl pinned-numbers module', task: 'OwlRl pinned numbers: 78/78 rules, 136 clauses, contract hash c7c5b2b3..., ext-eq-diff-sym sole extension.' },
      { id: 'w8-l06', lens: 'extension court', own: 'ext-eq-diff-sym court', task: 'Court: ext-eq-diff-sym is the sole extension; must flip.' },
      { id: 'w8-l07', lens: 'vector reasoning-report', own: 'purrdf-reasoning-report vectors', task: 'Certificate grammar pinned as VECTORS: purrdf-reasoning-report (4 vectors).' },
      { id: 'w8-l08', lens: 'vector dl-certificate', own: 'purrdf-dl-certificate vectors', task: 'Certificate vectors: purrdf-dl-certificate (1).' },
      { id: 'w8-l09', lens: 'vector chase-proof', own: 'purrdf-chase-proof vectors', task: 'Certificate vectors: purrdf-chase-proof (1).' },
      { id: 'w8-l10', lens: 'W3C RL corpus', own: 'W3C 50-case RL corpus harness', task: 'W3C 50-case RL corpus court.' },
      { id: 'w8-l11', lens: 'DL corpus 1', own: 'DL corpus batch 1', task: '262-case DL corpus, batch 1.' },
      { id: 'w8-l12', lens: 'DL corpus 2', own: 'DL corpus batch 2', task: '262-case DL corpus, batch 2.' },
      { id: 'w8-l13', lens: 'DL corpus 3', own: 'DL corpus batch 3', task: '262-case DL corpus, batch 3.' },
      { id: 'w8-l14', lens: 'DL corpus 4', own: 'DL corpus batch 4', task: '262-case DL corpus, batch 4.' },
      { id: 'w8-l15', lens: 'owlrl goldens', own: 'owlrl 7.1.4 16-goldens', task: 'owlrl==7.1.4 16-goldens court.' },
      { id: 'w8-l16', lens: 'divergence ledger', own: '8-entry divergence ledger', task: '8-entry divergence ledger, XPASS discipline.' },
      { id: 'w8-l17', lens: 'regime.rs', own: 'regime.rs', task: 'regime.rs as the host-neutral layer.' },
      { id: 'w8-l18', lens: 'alloc/determinism courts', own: 'entail/datalog court files', task: 'alloc + determinism courts over entail/datalog.' },
      { id: 'w8-l19', lens: 'receipts', own: 'W8 wave receipt', task: 'Assemble the W8 wave receipt.' },
      { id: 'w8-l20', lens: 'skeptic', own: 'read-only audit of W8', task: 'Skeptic over W8. READ-ONLY.' },
    ],
  },
  {
    title: 'W9 - SHACL/geo Tier-C fork',
    lanes: [
      { id: 'w9-l01', lens: 'shapes vendor 1', own: 'shapes crate slice 1', task: FORK_LANE_TASK_A('shapes', 1) },
      { id: 'w9-l02', lens: 'shapes vendor 2', own: 'shapes crate slice 2', task: FORK_LANE_TASK_A('shapes', 2) },
      { id: 'w9-l03', lens: 'shapes vendor 3', own: 'shapes crate slice 3', task: FORK_LANE_TASK_A('shapes', 3) },
      { id: 'w9-l04', lens: 'shapes vendor 4', own: 'shapes crate slice 4', task: FORK_LANE_TASK_A('shapes', 4) },
      { id: 'w9-l05', lens: 'validate vendor 1', own: 'validate crate slice 1', task: FORK_LANE_TASK_A('validate', 1) },
      { id: 'w9-l06', lens: 'validate vendor 2', own: 'validate crate slice 2', task: FORK_LANE_TASK_A('validate', 2) },
      { id: 'w9-l07', lens: 'validate vendor 3', own: 'validate crate slice 3', task: FORK_LANE_TASK_A('validate', 3) },
      { id: 'w9-l08', lens: 'validate vendor 4', own: 'validate crate slice 4', task: FORK_LANE_TASK_A('validate', 4) },
      { id: 'w9-l09', lens: 'floor 129', own: 'corpus floor court 129/129', task: FORK_LANE_TASK_A('floor-129', 0) },
      { id: 'w9-l10', lens: 'floor 590', own: 'corpus floor court 590/591', task: FORK_LANE_TASK_A('floor-590', 0) },
      { id: 'w9-l11', lens: 'floor 401', own: 'corpus floor court 401/401', task: FORK_LANE_TASK_A('floor-401', 0) },
      { id: 'w9-l12', lens: 'floor 73', own: 'corpus floor court 73/73', task: FORK_LANE_TASK_A('floor-73', 0) },
      { id: 'w9-l13', lens: 'floor 115', own: 'corpus floor court 115 dated', task: FORK_LANE_TASK_A('floor-115', 0) },
      { id: 'w9-l14', lens: 'SARIF goldens', own: 'model.rs SARIF extract + byte goldens', task: FORK_LANE_TASK_A('sarif', 0) },
      { id: 'w9-l15', lens: 'geo', own: 'geo crate path', task: 'Geo: 151K closure path-copy + refusal-law courts OR loud exclusion. If operatorFork is unset: [SKIP: operator fork unset] with standing REFUSED.' },
      { id: 'w9-l16', lens: 'Tier-C exclusions', own: 'DIVERGENCES + ABI-FREEZE roadmap rows', task: 'Loud Tier-C exclusion registration in DIVERGENCES + ABI-FREEZE roadmap (Tier D: 31 entail/reasoner exports beyond the 21-export ticket, P14). Registration is required on BOTH fork branches.' },
      { id: 'w9-l17', lens: 'operator receipt', own: 'operator-decision receipt for the fork', task: 'Record which fork (args.operatorFork=' + '"${FORK_PLACEHOLDER}"' + ') the coordinator passed; if unset, register the pending decision loudly.' },
      { id: 'w9-l18', lens: 'census', own: 'read-only SHACL/geo census', task: 'Census of SHACL/geo surface vs purrdf. READ-ONLY.' },
      { id: 'w9-l19', lens: 'receipts', own: 'W9 wave receipt', task: 'Assemble the W9 wave receipt.' },
      { id: 'w9-l20', lens: 'skeptic', own: 'read-only audit of W9', task: 'Skeptic over W9. READ-ONLY.' },
    ],
  },
  {
    title: 'W10 - Adversarial + arbiter',
    lanes: [
      { id: 'w10-l01', lens: 'bytes', own: 'read-only, lens: bytes', task: 'Skeptic lens: bytes (byte-parity claims across waves).' },
      { id: 'w10-l02', lens: 'census', own: 'read-only, lens: census', task: 'Skeptic lens: census.' },
      { id: 'w10-l03', lens: 'ABI', own: 'read-only, lens: ABI', task: 'Skeptic lens: ABI (13-export freeze, D9 Tier A).' },
      { id: 'w10-l04', lens: 'licenses', own: 'read-only, lens: licenses', task: 'Skeptic lens: licenses.' },
      { id: 'w10-l05', lens: 'receipts', own: 'read-only, lens: receipts', task: 'Skeptic lens: receipts (every receipt traces to a real command).' },
      { id: 'w10-l06', lens: 'flake', own: 'read-only, lens: flake', task: 'Skeptic lens: flake.' },
      { id: 'w10-l07', lens: 'vendoring purity', own: 'read-only, lens: vendoring purity', task: 'Skeptic lens: vendoring purity (renames only; NON-rename edits = falsifiers).' },
      { id: 'w10-l08', lens: 'determinism', own: 'read-only, lens: determinism', task: 'Skeptic lens: determinism.' },
      { id: 'w10-l09', lens: 'CI', own: 'read-only, lens: CI', task: 'Skeptic lens: CI.' },
      { id: 'w10-l10', lens: 'duckdb', own: 'read-only, lens: duckdb', task: 'Skeptic lens: duckdb.' },
      { id: 'w10-l11', lens: 'avec', own: 'read-only, lens: avec', task: 'Skeptic lens: avec.' },
      { id: 'w10-l12', lens: 'governor', own: 'read-only, lens: governor', task: 'Skeptic lens: governor.' },
      { id: 'w10-l13', lens: 'seed', own: 'read-only, lens: seed', task: 'Skeptic lens: seed.' },
      { id: 'w10-l14', lens: 'docs', own: 'read-only, lens: docs', task: 'Skeptic lens: docs.' },
      { id: 'w10-l15', lens: 'dangling', own: 'read-only, lens: dangling', task: 'Skeptic lens: dangling.' },
      { id: 'w10-l16', lens: 'counts', own: 'read-only, lens: counts', task: 'Skeptic lens: counts.' },
      { id: 'w10-l17', lens: 'ladder', own: 'read-only, lens: ladder', task: 'Skeptic lens: ladder.' },
      { id: 'w10-l18', lens: 'provenance', own: 'read-only, lens: provenance', task: 'Skeptic lens: provenance.' },
      { id: 'w10-l19', lens: 'seam', own: 'read-only, lens: seam', task: 'Skeptic lens: seam (bridge edge: SolutionTerm::join_key + ScratchId bit-32 disjointness).' },
      { id: 'w10-l20', lens: 'blake3', own: 'read-only, lens: blake3', task: 'Skeptic lens: blake3.' },
    ],
  },
]

// W9 fork-dependent lane prompt text (branch a = vendor per doc W9(a); else loud-skip).
function FORK_LANE_TASK_A(subject, slice) {
  if (FORK === 'vendor') {
    if (subject.startsWith('floor-')) return 'Corpora floor court: ' + subject.slice(6) + ' (vendor branch per operator fork). Must flip.'
    if (subject === 'sarif') return 'SARIF byte goldens: model.rs 844-line SARIF extract as the only carried piece if excluding, full goldens if vendoring (vendor branch active).'
    return 'Vendor ' + subject + ' crate slice ' + slice + ' per vendoring law (operator fork = vendor).'
  }
  if (FORK === 'exclude') {
    return '[SKIP: operator fork = exclude] This vendor-branch lane does not run. Return standing REFUSED with verdict SKIP and the reason; do not edit files. Verify instead that your surface is registered loudly as a Tier-C exclusion (check, do not write).'
  }
  return '[SKIP: W9 operator fork unset] Return standing REFUSED with verdict SKIP and reason "operator fork unset"; edit nothing.'
}

// ---------------------------------------------------------------- helpers

function lanePrompt(waveIdx, lane) {
  const w = waveIdx + 1
  return [
    'You are lane ' + lane.id + ' (lens: ' + lane.lens + ') of wave W' + w + ' in the stardf v26.11.1 full-parity campaign.',
    'Run label: ' + RUN + '. Wave W' + w + ' of 10; 20 lanes run concurrently in this wave; your file ownership is disjoint from theirs.',
    '',
    CONTRACT,
    '',
    'FILE OWNERSHIP (yours alone this wave): ' + lane.own,
    'TASK: ' + lane.task,
    '',
    'Authoritative on-disk docs (ground every specific before acting; the campaign section for your wave is "W' + w + '"):',
    '- ' + CAMPAIGN_DOC,
    '- ' + DOC_DIR + '08-vendoring-plan.md (vendoring law details)',
    '- ' + ARROW_DOCS + 'OPERATOR-DECISIONS.md (D9-D14)',
    '- ' + ARROW_DOCS + 'ABI-FREEZE.md',
    '- ' + ARROW_DOCS + 'DIVERGENCES.md',
    '',
    'Chicago discipline: real subprocesses/tests; assert on final state; no mocks. A court that cannot flip is FAIL. Skips are loud. Receipts only from commands YOU ran.',
    RETURN_SPEC,
  ].join('\n')
}

function nullReceipt(laneId) {
  return {
    lane: laneId,
    standing: 'UNKNOWN',
    summary: 'agent() returned null (skipped by user or terminal API error)',
    files: [],
    commands: [],
    nonclaims: ['lane did not run; no claims'],
    blockers: ['null agent result'],
  }
}

// ---------------------------------------------------------------- main

log('stardf v26.11.1 full-parity: 10 waves x 20 lanes = 200 lane agents + 1 arbiter. Run: ' + RUN)
log('Concurrency: all 20 lanes of a wave are submitted simultaneously; runtime caps physically-running agents at min(16, CPUs-2) per workflow (14 on this 16-CPU host) - the rest queue.')
if (!FORK) log('W9 operatorFork unset: W9 fork lanes will loud-skip (REFUSED/SKIP) unless args.operatorFork is set.')
if (FORCE) log('args.force=true: W1 launch-precondition gate will be SKIPPED (coordinator override).')

const waveReceipts = []
let abortedAt = null

for (let i = 0; i < WAVES.length; i++) {
  const wave = WAVES[i]
  const w = i + 1
  phase(wave.title)
  log('Wave W' + w + ' dispatching ' + wave.lanes.length + ' lanes: ' + wave.title)

  const results = await parallel(
    wave.lanes.map((lane) => () =>
      agent(lanePrompt(i, lane), {
        label: lane.id + ' (' + lane.lens + ')',
        phase: wave.title,
        schema: SCHEMA,
      })
    )
  )

  const receipts = results.map((r, j) => r || nullReceipt(wave.lanes[j].id))
  waveReceipts.push({
    wave: 'W' + w,
    title: wave.title,
    lanes: receipts,
    counts: {
      total: receipts.length,
      alive: receipts.filter((r) => r.standing === 'ALIVE').length,
      partial: receipts.filter((r) => r.standing === 'PARTIAL_ALIVE').length,
      blocked: receipts.filter((r) => r.standing === 'BLOCKED').length,
      refused: receipts.filter((r) => r.standing === 'REFUSED').length,
      null: results.filter((r) => !r).length,
    },
  })

  const summary = waveReceipts[waveReceipts.length - 1].counts
  log(
    'Wave W' + w + ' complete: ' + summary.alive + ' ALIVE / ' + summary.partial + ' PARTIAL_ALIVE / ' +
    summary.blocked + ' BLOCKED / ' + summary.refused + ' REFUSED / ' + summary.null + ' null'
  )

  // Launch gate after W1 (12-full-parity-campaign.md precondition).
  if (w === 1 && !FORCE) {
    const gate = receipts.find((r) => r.lane === 'w1-l20')
    const gateBlocked = gate && (gate.standing === 'BLOCKED' || (gate.blockers && gate.blockers.length > 0))
    if (gateBlocked) {
      abortedAt = 'W1 launch gate: precondition unmet - ' + (gate.summary || 'see w1-l20 blockers')
      log('ABORT: ' + abortedAt + ' - waves 2-10 NOT dispatched.')
      break
    }
  }
}

// Arbiter after W10's 20 skeptics (12-full-parity-campaign.md W10: verdict, structured).
let arbiter = null
if (!abortedAt) {
  const skepticReceipts = waveReceipts[9] ? waveReceipts[9].lanes : []
  const skepticEvidence = skepticReceipts.map((r) => ({
    lane: r.lane,
    standing: r.standing,
    verdict: r.verdict || null,
    findings: r.findings || [],
    summary: r.summary,
  }))
  phase('W10 - Adversarial + arbiter')
  log('Arbiter: synthesizing 20 skeptic verdicts into the parity verdict + repair work orders.')
  arbiter = await agent(
    [
      'You are the W10 arbiter of the stardf v26.11.1 full-parity campaign (' + REPO + '/).',
      '',
      CONTRACT,
      '',
      'Below are the structured receipts of the 20 adversarial skeptic lanes (one lens each: bytes, census, ABI, licenses, receipts, flake, vendoring purity, determinism, CI, duckdb, avec, governor, seed, docs, dangling, counts, ladder, provenance, seam, blake3).',
      'Ground-truth check anything suspicious on disk yourself (read-only + the same lane-contract rules bind you; you may run real commands but change no files).',
      '',
      'Skeptic receipts (JSON):',
      JSON.stringify(skepticEvidence),
      '',
      'Produce the structured verdict per 13-v26111-target-ledger.md law: parity=true AND finished=true ONLY with zero unresolved blockers AND the gap rows either landed or amended out. Also emit repair work orders for every confirmed finding (coordinator dispatches repairs; you do not edit files).',
      RETURN_SPEC +
        ' Additionally: verdict fields parity (boolean), finished (boolean), and repairs (array of {finding, owner_wave, suggested_lane}).',
    ].join('\n'),
    {
      label: 'W10-ARB (arbiter verdict)',
      phase: 'W10 - Adversarial + arbiter',
      schema: {
        type: 'object',
        properties: {
          lane: { type: 'string' },
          standing: { type: 'string', enum: ['ALIVE', 'PARTIAL_ALIVE', 'UNKNOWN', 'BLOCKED', 'REFUSED', 'UNSUPPORTED'] },
          summary: { type: 'string' },
          parity: { type: 'boolean' },
          finished: { type: 'boolean' },
          repairs: {
            type: 'array',
            items: {
              type: 'object',
              properties: {
                finding: { type: 'string' },
                owner_wave: { type: 'string' },
                suggested_lane: { type: 'string' },
              },
            },
          },
          files: { type: 'array', items: { type: 'string' } },
          commands: {
            type: 'array',
            items: {
              type: 'object',
              properties: { cmd: { type: 'string' }, exit: { type: 'number' }, note: { type: 'string' } },
            },
          },
          nonclaims: { type: 'array', items: { type: 'string' } },
          blockers: { type: 'array', items: { type: 'string' } },
        },
        required: ['lane', 'standing', 'summary', 'parity', 'finished', 'repairs', 'nonclaims'],
      },
    }
  )
}

return {
  run: RUN,
  script: 'stardf/workflows/v26.11.1-full-parity.workflow.js',
  fleet: { waves: WAVES.length, lanes_per_wave: WAVES.map((w) => w.lanes.length), total_lanes: 200, arbiter: 1 },
  aborted_at: abortedAt,
  waves: waveReceipts,
  arbiter: arbiter,
}
