# Live Publish Runbook — stardf v26.11.1 (Phase 5, Lane 9)

> **STATUS: NOT EXECUTED — operator's script, prose only.** This document is
> a runbook, not a receipt and not an authorization. It performs nothing. The
> only executor of record is the operator, at the console, and the only
> machine actor is `arrow/scripts/publish_stardf.py` under the operator's own
> hand. Lane 9 owns this file; it ran no publish, no gate, and no git command.
>
> Publish lane standing as of authoring: **BLOCKED** — the latest witnessed
> dry-run receipt `arrow/evidence/publish_stardf-receipt-20261101T051252Z.json`
> (git HEAD `724d5425`, mode dry-run) reads verdict BLOCKED with G8 the sole
> red gate; every other gate PASSes, including R1-license-gate (exit 0) and
> avec-ffi-court (exit 0). D20 records live publish as PENDING-OPERATOR.

## Subject and authority

- Actuator of record: `arrow/scripts/publish_stardf.py`
  (workspace root `stardf/arrow`; the script docstring is the lane contract:
  fail-closed gates G1-G9 + R1 + cgap court, dry-run by default, live behind
  `STARDF_PUBLISH_CONFIRM`).
- Decision register: `arrow/docs/OPERATOR-DECISIONS.md` — D17 (first-class
  closure only), D20 (publish closure first-class, ladder-judged, BLOCKED;
  live publish PENDING-OPERATOR), and the 80/20 re-scope "Decision owed" block
  (~line 677) that names the two G8 exits.
- Tier table of record: the `TIERS` list in the script itself (25-crate
  closure, runtime-re-derived from live `cargo metadata`; the script refuses
  to run the ladder on any backwards/sideways intra-closure edge — gate G2).
- Precedent for failure handling: `~/purrdf/docs/RELEASE.md` (Cargo
  publication section) and `~/purrdf/scripts/release-crates.sh` (ordering
  contract, dev-edge tier constraint, bootstrap ledger law).
- Per the repo receipt law, nothing here claims standing the receipts do not
  already carry.

## Prerequisites (all must hold before Step 1; fail-closed means all or nothing)

**P1 — Confirmation env.** `STARDF_PUBLISH_CONFIRM=ratified-v26.11.1` set in
the invoking shell. The script REFUSES live mode without it (exit 2).
`--dry-run` and `--publish` are mutually exclusive; `--skip-check` is refused
in live mode.

**P2 — crates.io credential, witnessed.** A crates.io account with publish
rights to the `stardf` namespace and a logged-in token:

```sh
cargo login   # token entry lands in $CARGO_HOME/credentials.toml
cargo whoami  # prints the logged-in owner — the witness that P2 holds
```

`cargo whoami` output is the P2 witness; without it, the lane has no
credential and must not start.

**P3 — G8 resolved or footprint scope-down ratified (currently OPEN).** The
one red gate. Witnessed G8 rows (receipt `20261101T051252Z`): hard normal deps
on non-closure workspace members — `stardf-rdf` -> {`stardf-ed25519`,
`stardf-gts`, `stardf-sparql-eval`} plus hard dev dep `stardf-gts`;
`stardf-sql` -> `stardf-shapes`; `stardf-toml` dev -> `stardf-sparql-eval`;
`stardf-rs` -> 16 members incl. optional `stardf-duckdb`. Every one of these
is a registry 404 at upload: the registry will never carry `=26.9.29`
datalog/ed25519/shapes/slice or `=26.11.1` gts/sparql-eval et al. The two
operator exits (OPERATOR-DECISIONS.md, 80/20 re-scope "Decision owed"):

1. **Re-ratify the D17 closure** — admit the 19 non-closure members (duckdb,
   shapes, gts, sparql-eval, ed25519, datalog, entail, geo, glob, hnsw, json,
   markdown, mime, process, retrieval, shex, slice, text, validate) with
   their own metadata and carriage costs (datalog/ed25519/shapes/slice sit at
   `26.9.29` and need a version bump to `26.11.1`), or
2. **Scope the publish footprint down** — drop/reroute the offending deps, or
   take `stardf-rdf`, `stardf-sql`, `stardf-toml`, `stardf-n3`,
   `stardf-rs` out of the tier list (each removed crate moves to
   `publish = false`).

Either exit is operator-owned; the ladder stays BLOCKED at G8 until one is
ratified. Do not publish with G8 red.

**P4 — All closure crates package-clean (dry-run verdict GREEN).** The
pre-publish gate run is the script in its default mode:

```sh
cd /Users/sac/ggen-legacy/stardf/arrow
python3 scripts/publish_stardf.py            # default = dry-run
```

Verdict must read **GREEN** (exit 0); the receipt lands at
`evidence/publish_stardf-receipt-<UTCSTAMP>.json`. The latest witnessed
receipt (above) is BLOCKED at G8 — P4 cannot hold until P3 clears. A
GREEN-WITH-SKIPS verdict is NOT publish clearance: live mode refuses on any
SKIP gate row unless `--accept-skips` is passed explicitly (a ratified
deviation that lands in the receipt); clear the skip instead where possible
(e.g. a missing cgap court file is fixed by restoring the `/Users/sac/cgap`
checkout, not by skip-accepting).

**P5 — Clean git tree (live mode never passes `--allow-dirty`).** `cargo
publish` refuses on a dirty worktree; the ladder's live path never passes
`--allow-dirty` (script nonclaim: commits/tags belong to the coordinator).
The coordinator commits/tags the v26.11.1 tree BEFORE the live ladder — lane
law bans the publish lane from git state changes (see Nonclaims).

**P6 — Registry propagation budget.** After each publish, the next tier's
`cargo publish --locked` re-resolves intra-closure deps from the registry;
freshly uploaded versions need a propagation window before they resolve. The
script paces publishes 25-30 s apart; if a propagation stall fires anyway
(cargo: "no matching package named `stardf-<x>` with version `=26.11.1`"),
treat it as a FAIL row, stop, and use the bounded propagation check in the
Failure handling section — never a blind immediate retry.

## Execution

Single invocation, from `arrow/`:

```sh
cd /Users/sac/ggen-legacy/stardf/arrow
STARDF_PUBLISH_CONFIRM=ratified-v26.11.1 python3 scripts/publish_stardf.py --publish
```

What one invocation does, in order:

1. **Gate suite in full** (G1-G9, R1, cgap court — every run, live or not).
   Any hard FAIL -> verdict BLOCKED, ladder not attempted, exit 1. Any SKIP
   in live mode without `--accept-skips` -> BLOCKED, exit 1.
2. **Live ladder T0 -> T8**, one crate at a time, in TIERS order:
   `cargo publish --locked -p <crate>` from `arrow/`. Pacing: uniform random
   25-30 s between successive publishes (none before the first — script
   `SPACING_MIN_S`/`SPACING_MAX_S`). Fail-closed: the ladder stops at the
   first failing publish.
3. **Receipt**: `evidence/publish_stardf-receipt-<UTCSTAMP>.json` with mode
   `live`, per-crate rows (tier, crate, cmd, exit, error), verdict
   GREEN / GREEN-WITH-SKIPS / BLOCKED. Exit 0 only on GREEN.

Tier order (TIERS verbatim; 25 crates — the script's `TIERS` list is the
source of table truth; if this table and the script disagree, the script
wins and this runbook is stale):

- **T0** — stardf-hash, stardf-alloc-probe, stardf-arrow-core
- **T1** — stardf-lex, stardf-deflate, stardf-branchless, stardf-testkit,
  stardf-events, stardf-arrow-bench, stardf-columnar
- **T2** — stardf-stack, stardf-iri, stardf-arrow-rs, stardf-xsd,
  stardf-arrow-capi, stardf-scan-duckdb
- **T3** — stardf-cdt
- **T4** — stardf-core, stardf-sparql-algebra
- **T5** — stardf-sparql-results
- **T6** — stardf-rdf
- **T7** — stardf-sql, stardf-n3, stardf-toml
- **T8** — stardf-rs

Note (T8): the umbrella is `stardf-rs` (on disk at `crates/stardf-purrdf`);
its optional `stardf-duckdb` dep is NOT in the closure (`publish = false`) —
the G8 rows cover the registry-404 that would follow if published as-is. Its
optional `stardf-sql` dep makes sql (T7) a hard publish-order prerequisite.

Segment mode (operator's alternative — verifies per tier): run the ladder in
9 segments, `--start-from` the first crate of each tier:

```sh
# per segment, in order; verify the tier between segments (see next section)
STARDF_PUBLISH_CONFIRM=ratified-v26.11.1 \
  python3 scripts/publish_stardf.py --publish --start-from stardf-hash
# then: stardf-lex (T1), stardf-stack (T2), stardf-cdt (T3), stardf-core (T4),
# stardf-sparql-results (T5), stardf-rdf (T6), stardf-sql (T7), stardf-rs (T8)
```

Every segment re-runs the full gate suite (cost: one `cargo check
--workspace` + cgap court per segment). `--start-from` outside the closure is
REFUSED (exit 2). The script paces across segment boundaries only within one
invocation; between segments the operator owns the 25-30 s spacing.

## Per-tier verification (after each tier — or after the single invocation)

Every published crate must resolve from crates.io. Two checks, first
authoritative, second corroborating:

1. **Sparse-index check (authoritative).** The index shard for a `stardf-*`
   crate is `https://index.crates.io/st/ar/<crate>` (sharding: names >4
   chars -> `first2/next2/name`). Each line is one version; expect a line
   with `"vers":"26.11.1"`:

```sh
for c in stardf-hash stardf-alloc-probe stardf-arrow-core; do
  printf '%s: ' "$c"
  curl -sf "https://index.crates.io/st/ar/$c" | grep -c '"vers":"26.11.1"'
done   # each prints >= 1; 0 or curl failure = propagation NOT witnessed
```

2. **`cargo search` (corroborating; the API may lag the index).** `cargo
   search stardf-hash --limit 3` shows `stardf-hash = "26.11.1"`.

Propagation law (purrdf RELEASE.md precedent): API visibility alone cannot
prove a dependent can resolve the new version; the check is satisfied only
when both the crates.io API and the sparse index serve the exact **unyanked**
version with matching checksums. Run the check for every crate before the
next tier's publish (or after the full ladder), and for versions an earlier
attempt already uploaded — resumed publication cannot skip the evidence its
dependents need.

## Failure handling — partial-publish recovery (purrdf release-cargo precedent)

The crates.io lane is immutable-and-resumable, per `~/purrdf/docs/RELEASE.md`:

- **Immutability.** A published version is permanent. Never yank a version to
  retry, never republish a `<crate> = 26.11.1` that already uploaded (crates.io
  refuses already-present versions; a forced re-upload is not a recovery
  step). Never delete/recreate/move anything to work around immutability —
  purrdf's publisher never deletes a release, recreates it, or moves a tag.
- **Diagnose before retry.** The ladder stops at the first failing publish;
  the receipt's `ladder[]` rows name the failing crate and cargo's error
  tail. Distinguish registry rejection (fix the root cause, then resume) from
  transport/auth failure (stop the lane — purrdf: only a literal 404 permits
  record creation; authentication and transport failures stop the lane, never
  guessed-through).
- **Already-uploaded vs not.** Before resuming, determine which crates the
  registry actually has: for each published-or-attempted crate run the
  sparse-index check above. A crate whose `"vers":"26.11.1"` line is present
  is DONE — resume AFTER it, never re-run it. A crate with no such line is
  unpublished and safe to attempt again once the root cause is fixed. (The
  script cannot skip an already-present version by itself — a repeat `cargo
  publish` of an existing `<crate>-26.11.1` fails loudly; treat that as a
  stop-and-diagnose, not a retry loop.)
- **Resume** with the same live command plus `--start-from <crate>` where
  `<crate>` is the first UNpublished crate in TIERS order. Gates re-run in
  full on every resume — a red gate blocks the resume too. Keep 25-30 s
  spacing between the manual resume and the last real publish.
- **Bounded propagation check before resuming dependents.** Poll (bounded,
  deadline-checked — no fixed sleeps as a methodology) until both the
  crates.io API and the sparse index serve each already-uploaded crate's
  exact unyanked `26.11.1` with matching checksums; only then `--start-from`
  the dependent tier. An index that has not propagated times out visibly —
  that timeout is the loud outcome, not a signal to retry faster.

## Post-publish verification (after verdict GREEN)

1. **Fresh-checkout resolution from registry alone.** In a scratch dir with a
   scratch CARGO_HOME (no machine-local overrides, no login needed):

```sh
SCRATCH=$(mktemp -d)
export CARGO_HOME="$SCRATCH/home"
mkdir -p "$SCRATCH/consumer" && cd "$SCRATCH/consumer"
cat > Cargo.toml <<'EOF'
[package]
name = "stardf-consumer-probe"
version = "0.1.0"
edition = "2021"

[dependencies]
stardf-rs = "=26.11.1"
EOF
cargo generate-lockfile    # network on; resolves from the sparse index alone
cargo tree                 # every stardf-* resolves; umbrella pulls the closure
```

Expect: resolution succeeds; every `stardf-*` entry in `cargo tree` /
`cargo metadata` output carries version `26.11.1` and source
`registry+https://github.com/rust-lang/crates.io-index`. Then repeat the
dependency block with `stardf-<crate> = "=26.11.1"` for each of the 25 crates
(or loop the sparse-index check of the Per-tier section over all 25) — the
umbrella probe proves the closure end to end; the per-crate probe proves
each cell.

2. **Zero path deps in the published graph.** Two direct looks:

```sh
# (a) the uploaded manifests: download a representative .crate tarball
curl -sf https://crates.io/api/v1/crates/stardf-rs/26.11.1/download -o "$SCRATCH/rs.crate"
tar -xzf "$SCRATCH/rs.crate" -C "$SCRATCH"
grep -c 'path[[:space:]]*=' "$SCRATCH/stardf-rs-26.11.1/Cargo.toml"   # expect 0
# (b) the consumer graph: no stardf-* node resolves from a path source
cargo metadata --format-version 1 \
  | python3 -c 'import json,sys; ms=json.load(sys.stdin)["packages"]; \
print([p["name"] for p in ms if p["name"].startswith("stardf-") and not p.get("source")])'
# expect: []
```

Dual-keyed manifests (G5) are what make (a) come out 0: every intra-closure
dep carries `version` alongside `path` in the workspace, and cargo strips the
path key at upload. If (a) or (b) is nonzero, the publish has a real defect —
register it, do not paper it.

3. **Receipt the verification.** Per receipt law: commands + real exit codes,
subject identity at receipt time, honest failure rows, a nonclaims block
(mandatory), written to `arrow/evidence/`, ledger row in
`arrow/docs/RECEIPTS.md`.

## Coordinator-owned steps (outside this lane's authority)

- Commit and tag the v26.11.1 tree before the live ladder (P5; script
  nonclaim: the script never mutates git state; commits/tags belong to the
  coordinator).
- After GREEN plus a post-publish verification receipt: tag `stardf-v26.11.1`
  if the operator wants a tag mirror of the registry state. Registry state is
  the record; the tag is a pointer, not the publication.

## Nonclaims

- This runbook does NOT execute. No command in it was run by lane 9; the
  file is the operator's script. Publishing, resuming, and verifying are
  operator acts.
- No live publish has been performed or authorized by this document. D20
  holds: live publish is PENDING-OPERATOR, and the ladder is BLOCKED at G8
  (receipt `20261101T051252Z`, HEAD `724d5425`) until P3 clears.
- No git command was run by lane 9; the coordinator owns commits/tags.
- The tier table above is a projection of the script's `TIERS` list; on any
  disagreement the script wins (runtime cargo-metadata re-derivation), and
  this runbook is stale.
- Dry-run receipts prior to the live run are not live evidence; a GREEN
  dry-run proves packageability, not registry propagation.

## See Also

- `arrow/scripts/publish_stardf.py` — the ladder: gates G1-G9 + R1 + cgap
  court, TIERS, spacing, confirmation gate, receipts
- `arrow/docs/OPERATOR-DECISIONS.md` — D17/D20 and the 80/20 re-scope
  "Decision owed" block (the two G8 exits)
- `~/purrdf/docs/RELEASE.md` — Cargo publication section: immutability,
  resumable partial releases, bounded propagation check
- `~/purrdf/scripts/release-crates.sh` — publish-order contract, dev-edge
  tier constraint, bootstrap ledger law
- `stardf/docs/jira/v26.10.31/14-definition-of-done.md` — campaign DoD
- `arrow/docs/RECEIPTS.md` — receipt ledger of record
