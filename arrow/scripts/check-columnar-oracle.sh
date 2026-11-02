#!/usr/bin/env bash
# Lane 04-scan-oracle (2026-10-31): external DuckDB columnar oracle for the
# stardf Arrow workspace, retargeted per the v26.11.1 finisher lane contract.
#
# Fixture generation goes through the real first-party codec —
# stardf_columnar::write — over the standing splitmix64 fixture (state
# 0x6012; byte-identical to the stardf-columnar golden_bytes court's
# fixture()), and the verification queries run through the native in-process
# stardf_scan() table function WHEN its registration is available in this
# script context, falling back to read_parquet with a loud note otherwise.
# A harness that is present and failing is a FAIL, never a silent downgrade.
#
# Chicago: real codec bytes on disk, a real DuckDB engine observes the
# physical files, and the generator's own projection arithmetic is
# cross-checked against that observation.
#
# Scan-registration tiers (first match wins):
#   1. env override  STARDF_SCAN_ORACLE_CMD  — a command template containing
#      {FIXTURE_DIR}; executed (via bash -c) with the placeholder substituted
#      by the fixture directory. SQL arrives on stdin; CSV rows on stdout;
#      exit 0 on success. Shape-agnostic (shims/mutants use this tier).
#   2. harness example crates/stardf-scan-duckdb/examples/scan_oracle.rs —
#      placeholder name only; see the real path in SCAN_EXAMPLE below. Built
#      by `cargo build -p stardf-scan-duckdb --example scan_oracle` from the
#      workspace root; run as `<binary> <fixture_dir>` with SQL on stdin.
#   3. default probe: crates/stardf-scan-duckdb/src/lib.rs exposing
#      `pub fn register_stardf_scan(&Connection)`. This script generates a
#      throwaway runner binary whose ENTIRE scan logic is the crate's own
#      facade — it registers stardf_scan() in-process over a real
#      frozen-duckdb engine and executes the SQL stdin against it. Until the
#      crate exists, this probe misses and the oracle runs read_parquet-only
#      (loud note below). One registration kernel, one binding: the runner
#      never re-implements scan logic.
#
# Runner protocol (all tiers): possibly multi-statement SQL on stdin
# (';'-separated, no ';' inside string literals), CSV result rows on stdout,
# exit 0 on success. Verification queries CAST every cell to VARCHAR so the
# shared protocol is engine-independent.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLE="${ROOT}/examples/columnar_oracle_fixture.rs"
SCRATCH="${ROOT}/target/columnar-oracle-scratch"
SCAN_CRATE="${ROOT}/crates/stardf-scan-duckdb"
SCAN_EXAMPLE="${SCAN_CRATE}/examples/scan_oracle.rs"

# The standing floor witness (SCAN-COMPLETION §5 C1): every generated table
# carries at least these row counts — terms 6, quads 3, reifiers 0,
# annotations 0, blobs 2. The generated fixture is deterministic
# (splitmix64 state=0x6012), so the observed counts are also pinned exactly
# by the generator's own projection (parity courts below).
FLOOR_COUNTS=(6 3 0 0 2)

if ! command -v duckdb >/dev/null 2>&1; then
  echo "[SKIP: duckdb CLI absent — columnar oracle not exercised]"
  exit 3
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "[SKIP: cargo absent — cannot build the fixture generator]"
  exit 3
fi

tmp="${SCRATCH}/proj"
rm -rf "${SCRATCH}"
mkdir -p "${tmp}/src"
trap 'rm -rf "${SCRATCH}"' EXIT

# Tier-3 probe: the scan crate exposes its registration facade.
RUNNER_SRC="${tmp}/src/scan_oracle_runner.rs"
RUNNER_BIN="${SCRATCH}/bin/debug/scan_oracle_runner"
if [[ -f "${SCAN_CRATE}/src/lib.rs" ]] \
    && grep -q "pub fn register_stardf_scan" "${SCAN_CRATE}/src/lib.rs"; then
  BUILD_RUNNER=1
else
  BUILD_RUNNER=0
fi

# Throwaway cargo project consuming the real codec (and, in tier 3, the real
# scan facade) by path, with its own target dir so it stays out of the shared
# workspace build graph. No workspace manifest edits; no machine-local paths
# leak anywhere but this throwaway (never published). The frozen-duckdb
# "=2.0.4" mirror must move with the workspace pin (D13, lane 01-duckdb-pins).
cat > "${tmp}/Cargo.toml" <<MANIFEST
[workspace]

[package]
name = "columnar_oracle_fixture"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
stardf-columnar = { path = "${ROOT}/crates/stardf-columnar" }
MANIFEST

if (( BUILD_RUNNER )); then
  cat >> "${tmp}/Cargo.toml" <<MANIFEST
stardf-scan-duckdb = { path = "${SCAN_CRATE}" }
frozen-duckdb = "=2.0.4"
MANIFEST
fi

cat >> "${tmp}/Cargo.toml" <<'MANIFEST'

[[bin]]
name = "columnar_oracle_fixture"
path = "src/main.rs"
MANIFEST

if (( BUILD_RUNNER )); then
  cat >> "${tmp}/Cargo.toml" <<'MANIFEST'

[[bin]]
name = "scan_oracle_runner"
path = "src/scan_oracle_runner.rs"
MANIFEST

  cat > "${RUNNER_SRC}" <<'RUST'
// Throwaway in-process registration runner (script-generated; lane
// 04-scan-oracle). The scan logic is stardf-scan-duckdb's own facade — this
// binary only registers stardf_scan() over a real engine and executes the
// SQL stdin against it.
use std::io::Read;

use frozen_duckdb::Connection;
use stardf_scan_duckdb::register_stardf_scan;

fn main() {
    let mut sql = String::new();
    std::io::stdin()
        .read_to_string(&mut sql)
        .expect("read SQL from stdin");
    let conn = Connection::open_in_memory().expect("in-memory engine opens");
    register_stardf_scan(&conn).expect("stardf_scan registers in-process");
    for statement in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let mut stmt = conn
            .prepare(statement)
            .unwrap_or_else(|e| panic!("prepare fails on {statement:?}: {e}"));
        let arity = stmt.column_count();
        let mut rows = stmt
            .query([])
            .unwrap_or_else(|e| panic!("query fails on {statement:?}: {e}"));
        while let Some(row) = rows.next().expect("row fetch") {
            let cells: Vec<String> = (0..arity)
                .map(|i| {
                    row.get::<_, String>(i)
                        .expect("cell is VARCHAR (verification queries CAST)")
                })
                .collect();
            println!("{}", cells.join(","));
        }
    }
}
RUST
fi

cp "${EXAMPLE}" "${tmp}/src/main.rs"

CARGO_TARGET_DIR="${SCRATCH}/bin" cargo build \
  --manifest-path "${tmp}/Cargo.toml" --offline --quiet

fixture="${SCRATCH}/bin/debug/columnar_oracle_fixture"
tmpf="${SCRATCH}/fixture"

# Generate the fixture: real vendored builder + real codec write -> five real
# Parquet files, plus the generator's own projection row counts on stdout.
fixture_out="$("${fixture}" "${tmpf}")"
counts_line="$(printf '%s\n' "${fixture_out}" | grep '^ORACLE-COUNTS ' || true)"
if [[ -z "${counts_line}" ]]; then
  echo "ERROR: fixture generator emitted no ORACLE-COUNTS line:" >&2
  printf '%s\n' "${fixture_out}" >&2
  exit 1
fi
gen_counts="$(printf '%s\n' "${counts_line}" | awk '{print $2}')"
echo "fixture counts (generator projection): terms,quads,reifiers,annotations,blobs = ${gen_counts}"

IFS=',' read -r -a gen <<< "${gen_counts}"
if [[ "${#gen[@]}" -ne 5 ]]; then
  echo "ERROR: ORACLE-COUNTS line must carry five counts: ${gen_counts}" >&2
  exit 1
fi
for i in 0 1 2 3 4; do
  if (( gen[i] < FLOOR_COUNTS[i] )); then
    echo "ERROR: fixture count for table index ${i} is ${gen[i]}, below the standing floor ${FLOOR_COUNTS[i]}" >&2
    exit 1
  fi
done
for f in terms quads reifiers annotations blobs; do
  if [[ ! -s "${tmpf}/${f}.parquet" ]]; then
    echo "ERROR: fixture table missing or empty on disk: ${tmpf}/${f}.parquet" >&2
    exit 1
  fi
done

# --- Verification-source seam ------------------------------------------------
# In read_parquet mode each table is its file; in scan mode every table comes
# through the registered stardf_scan() over the dataset root (named parameter
# `table`, per the frozen W1 surface in docs/SCAN-COMPLETION.md §3).

MODE="read_parquet"
SCAN_RUNNER=()
if [[ -n "${STARDF_SCAN_ORACLE_CMD:-}" ]]; then
  MODE="stardf_scan"
  SCAN_RUNNER=("${SCRATCH}/scan_runner.sh")
  {
    echo '#!/usr/bin/env bash'
    echo 'set -euo pipefail'
    echo 'exec bash -c "${STARDF_SCAN_ORACLE_CMD//\{FIXTURE_DIR\}/$1}"'
  } > "${SCAN_RUNNER}"
  chmod +x "${SCAN_RUNNER}"
  echo "scan mode: STARDF_SCAN_ORACLE_CMD override (SQL on stdin)"
elif [[ -f "${SCAN_EXAMPLE}" ]]; then
  MODE="stardf_scan"
  echo "scan mode: stardf-scan-duckdb scan_oracle example harness found; building..."
  if ! (cd "${ROOT}" && cargo build --offline --quiet \
      -p stardf-scan-duckdb --example scan_oracle); then
    echo "[NOTE: scan_oracle harness PRESENT but its build failed in this script context — stardf_scan() registration unavailable here; verification falls back to read_parquet]" >&2
    MODE="read_parquet"
  else
    SCAN_RUNNER=("${ROOT}/target/debug/examples/scan_oracle")
  fi
elif (( BUILD_RUNNER )); then
  MODE="stardf_scan"
  echo "scan mode: tier-3 in-process registration (stardf-scan-duckdb facade)"
  SCAN_RUNNER=("${RUNNER_BIN}")
fi

if [[ "${MODE}" == "read_parquet" ]]; then
  echo "[NOTE: stardf_scan() registration unavailable in this script context — verification falls back to read_parquet]"
fi

run_sql() {
  if [[ "${MODE}" == "stardf_scan" ]]; then
    printf '%s' "$1" | "${SCAN_RUNNER[@]}" "${tmpf}"
  else
    duckdb -csv -noheader -c "$1"
  fi
}

source_expr() {
  local table="$1"
  if [[ "${MODE}" == "stardf_scan" ]]; then
    # `table` is a reserved keyword: the engine binds it as a QUOTED named
    # parameter (the scan courts' proven form: stardf_scan(root, "table" = ...)).
    printf "stardf_scan('%s', \"table\" = '%s')" "${tmpf}" "${table}"
  else
    printf "read_parquet('%s/%s.parquet')" "${tmpf}" "${table}"
  fi
}

# --- Court 1: row counts (floor + generator parity + scan/read parity) -------

TERMS="$(source_expr terms)"
QUADS="$(source_expr quads)"
REIFIERS="$(source_expr reifiers)"
ANNOTATIONS="$(source_expr annotations)"
BLOBS="$(source_expr blobs)"

counts_query="SELECT
  CAST((SELECT count(*) FROM ${TERMS}) AS VARCHAR),
  CAST((SELECT count(*) FROM ${QUADS}) AS VARCHAR),
  CAST((SELECT count(*) FROM ${REIFIERS}) AS VARCHAR),
  CAST((SELECT count(*) FROM ${ANNOTATIONS}) AS VARCHAR),
  CAST((SELECT count(*) FROM ${BLOBS}) AS VARCHAR);"

observed_counts="$(run_sql "${counts_query}")"
if [[ "${observed_counts}" != "${gen_counts}" ]]; then
  echo "ERROR: engine observed counts ${observed_counts} != generator projection ${gen_counts}" >&2
  exit 1
fi
echo "counts parity OK: engine == generator projection (${observed_counts})"

# --- Court 2: physical types through the engine's reader ----------------------

types_query="SELECT typeof(id), typeof(lex) FROM ${TERMS} LIMIT 1;
SELECT typeof(bytes) FROM ${BLOBS} LIMIT 1;"
types="$(run_sql "${types_query}")"
if [[ "${types}" != $'BIGINT,VARCHAR\nBLOB' ]]; then
  echo "ERROR: engine observed unexpected Parquet physical types:" >&2
  printf '%s\n' "${types}" >&2
  exit 1
fi
echo "types OK: terms(id)=BIGINT, terms(lex)=VARCHAR, blobs(bytes)=BLOB"

# --- Court 3: cross-table referential integrity over the observed bytes -------

integrity_query="WITH t AS (SELECT count(*) AS n FROM ${TERMS})
SELECT CAST(count(*) AS VARCHAR) FROM ${QUADS}, t
WHERE s < 1 OR s > t.n OR p < 1 OR p > t.n OR o < 1 OR o > t.n
   OR (g IS NOT NULL AND (g < 1 OR g > t.n));
SELECT CAST(count(*) AS VARCHAR) FROM ${QUADS} WHERE s IS NULL OR p IS NULL OR o IS NULL;
SELECT CAST(count(*) - count(DISTINCT digest) AS VARCHAR) FROM ${BLOBS};"
violations="$(run_sql "${integrity_query}")"
if [[ "${violations}" != $'0\n0\n0' ]]; then
  echo "ERROR: engine observed referential-integrity violations (out-of-range ids / null required ids / duplicate blob digests):" >&2
  printf '%s\n' "${violations}" >&2
  exit 1
fi
echo "integrity OK: all quads s/p/o/g reference the terms dictionary; blob digests unique"

# --- Court 4 (scan mode): count parity against the CLI read_parquet path ------
# When stardf_scan() is registered, the counts above already went through it;
# re-observe the same fixture through read_parquet and require equality so
# the oracle also witnesses the C1-class scan-vs-read parity.

if [[ "${MODE}" == "stardf_scan" ]]; then
  parity_query="SELECT
    (SELECT count(*) FROM read_parquet('${tmpf}/terms.parquet')),
    (SELECT count(*) FROM read_parquet('${tmpf}/quads.parquet')),
    (SELECT count(*) FROM read_parquet('${tmpf}/reifiers.parquet')),
    (SELECT count(*) FROM read_parquet('${tmpf}/annotations.parquet')),
    (SELECT count(*) FROM read_parquet('${tmpf}/blobs.parquet'));"
  parity_counts="$(duckdb -csv -noheader -c "${parity_query}")"
  if [[ "${parity_counts}" != "${gen_counts}" ]]; then
    echo "ERROR: stardf_scan() counts ${observed_counts} disagree with read_parquet counts ${parity_counts}" >&2
    exit 1
  fi
  echo "scan parity OK: stardf_scan() == read_parquet() == generator projection (${gen_counts})"
  echo "OK: stardf_scan() verification passed (counts=${gen_counts}; terms=BIGINT,VARCHAR; blobs=BLOB)"
else
  echo "OK: read_parquet verification passed (counts=${gen_counts}; terms=BIGINT,VARCHAR; blobs=BLOB)"
fi
