# stardf-arrow — one-command DX recipes (modeled on cgap justfile conventions)
#
#   just build    - debug build of all workspace members
#   just check    - cargo check across the workspace
#   just test     - test suite across the workspace
#   just clippy   - clippy with warnings denied
#   just fmt      - rustfmt across the workspace
#   just ci-all   - fmt + clippy + check + test (the default verify ladder)
#
# Member manifests are owned by their creating lanes; recipes here operate on
# the workspace as a whole via -p selectors only, never per-file.

default:
    @just --list

build:
    cargo build --workspace

check:
    cargo check --workspace

test:
    cargo test --workspace

# Criterion benches live in the stardf-arrow-bench member (Lane 7: promoted
# from standalone root to member); this drives the actual bench runs, which
# the plain test/check ladder deliberately does not (bench is slow by design).
bench:
    cargo bench --workspace -p stardf-arrow-bench

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

fmt:
    cargo fmt --all

ci-all: fmt clippy check test

# Lane 14: external DuckDB oracle (skips loudly, exit 3, when duckdb is absent).
columnar-oracle:
    bash scripts/check-columnar-oracle.sh

# Blake3 single-identity gate (KD-07/D10)
blake3-gate:
    bash scripts/verify_blake3_identity.sh

# DuckDB rehome court (D13, two-arm =2.0.4)
rehome-court:
    bash scripts/verify_duckdb_rehome.sh

# Header freeze court (F4)
header-freeze:
    python3 scripts/verify_header_freeze.py
