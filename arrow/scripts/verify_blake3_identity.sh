#!/usr/bin/env bash
# verify_blake3_identity.sh — KD-08 single-identity gate (close condition, half 1)
# Design: stardf/docs/jira/v26.10.31/11-blake3-arbitration.md §3.
#
# VACUOUS-arm decision (fixed, not configurable): blake3 ABSENT -> exit 1.
# Justification: KD-08's decision-in-force is "bind exactly one"; zero bindings
# is a vacuous gate that carries no bits and must fail loudly (gate-vacuity
# law), per 11-blake3-arbitration.md §3 ("Absence is VACUOUS (exit 1)").
# NOTE: if the KD-08 flip lands (Option B, vendored native), §5.4 inverts this
# arm — absence becomes PASS and presence FAIL. This script implements the
# pre-flip (Option A) mode; rewrite or re-mode at flip time.
#
# Deviation from the doc's exact command, witnessed this session: bare
# `cargo tree -e normal -i blake3 --prefix none` from the workspace root
# selects only the ROOT package's graph (stardf-arrow does not itself depend
# on stardf-arrow-core), so it exits 101 "did not match any packages" even
# though the member crates bind blake3. The gate therefore scopes the inverse
# tree to the full workspace (`--workspace`). Same class of under-count the
# gate exists to prevent.
#
# Chicago: real `cargo tree` subprocess against the real lockfile; asserts on
# final state (resolved graph + declared-vs-locked version), no mocks.
set -euo pipefail
cd "$(dirname "$0")/.." # workspace root = stardf/arrow/

fail=0

# --- 1. Single-identity check -------------------------------------------------
# Prefer the inverse tree; if cargo reports no match at all, confirm absence
# against the forward tree before declaring VACUOUS (guards against a spurious
# mismatch masking a real second identity).
if ! inv=$(cargo tree --workspace -e normal -i blake3 --prefix none 2>&1); then
    # "did not match any packages" is the only absence signal; any other cargo
    # failure (resolution conflict, broken manifest) must NOT read as VACUOUS.
    if printf '%s' "$inv" | grep -q 'did not match any packages'; then
        fwd=$(cargo tree --workspace -e normal --charset ascii 2>/dev/null | grep -E 'blake3 v' || true)
        if [ -z "$fwd" ]; then
            echo "[VACUOUS] blake3 is ABSENT from the normal dependency graph (binding is mandatory; absence must fail loudly per gate-vacuity law)"
            exit 1
        fi
        echo "[FAIL] cargo tree -i errored but forward tree shows blake3: $fwd"
        exit 1
    fi
    echo "[ERROR] cargo tree failed (workspace resolution broken — this is not absence):"
    printf '%s\n' "$inv" | sed 's/^/  /'
    exit 1
fi

versions=$(printf '%s\n' "$inv" | grep -oE 'blake3 v[0-9.]+' | sort -u || true)
n=$(printf '%s\n' "$versions" | grep -c 'blake3 v' || true)
echo "[INFO] inverse-tree rows (normal edges, workspace):"
printf '%s\n' "$inv" | sed 's/^/  /'

if [ "$n" -eq 0 ]; then
    echo "[VACUOUS] blake3 not present in inverse closure (binding is mandatory)"
    exit 1
elif [ "$n" -eq 1 ]; then
    echo "[PASS] exactly 1 blake3 identity: $versions"
else
    echo "[FAIL] >1 blake3 identity in closure: $versions"
    fail=1
fi

# --- 2. Declared-vs-locked version skew (WARN row) ----------------------------
# Witnessed drift (lane S4 / 11-blake3-arbitration.md §1): Cargo.toml declares
# 1.8.5, lock resolved 1.8.7 with no deliberate bump. Surface as WARN, do not
# fail — the gate's failure channel is the identity count; the skew channel is
# advisory (an Option-A pin to "=1.8.7" would clear it; see §5-If-A-wins).
declared=$(grep -oE 'blake3[[:space:]]*=[[:space:]]*\{[[:space:]]*version[[:space:]]*=[[:space:]]*"[^"]+"' \
    crates/stardf-arrow-core/Cargo.toml | grep -oE '"[0-9][^"]*"' | tr -d '"' || true)
locked=$(grep -A1 '^name = "blake3"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p' || true)
if [ -z "$declared" ]; then
    echo "[WARN] no blake3 version declaration found in crates/stardf-arrow-core/Cargo.toml (post-Option-B vendoring, this is expected and can be retired)"
elif [ -z "$locked" ]; then
    echo "[WARN] blake3 not found in Cargo.lock (skew check not computable)"
elif [ "$declared" != "$locked" ]; then
    echo "[WARN] version skew: Cargo.toml declares $declared, Cargo.lock resolved $locked (silent drift class; pin '=1.8.7' or bump deliberately to clear)"
else
    echo "[PASS-INFO] declared==locked: $declared"
fi

exit "$fail"
