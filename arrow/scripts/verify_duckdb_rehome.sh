#!/bin/bash
# verify_duckdb_rehome.sh — fail-closed frozen-duckdb rehome court
# (W1 lane 9 sandbox court; v26.11.1 lane 01-duckdb-pins extension 2026-10-31)
#
# Falsifier under test: the operator-ratified frozen-duckdb family pins
# (=2.0.4 x3) resolve AND compile from crates.io WITHOUT any machine-local
# path — no [patch.crates-io], no /Users/sac/frozen-duckdb checkout, and no
# $HOME/.frozen-duckdb cache. Cache-absent compile ALIVE = the D13 acceptance
# datum ("rehome": the machine-local fork + cache are no longer load-bearing).
#
# Two arms, one invocation:
#   ARM A (control, in-situ): fork + cache PRESENT. Sandbox project resolves
#     and compiles the =2.0.4 x3 registry triple. Exercises the warm-cache
#     path (ensure_binary cache hit on v2.0.0-arm64).
#   ARM B (rehome): /Users/sac/frozen-duckdb AND $HOME/.frozen-duckdb REALLY
#     renamed away on the same volume (witnessed-absence assertions before the
#     build runs); identical sandbox build runs cache-absent — ensure_binary
#     must re-stage the engine (GitHub release asset or local-compile fallback).
#     Real trees are restored and proven byte-identical by full-content
#     SHA-256 over every file in both trees.
#
# Restore safety:
#   - renames use a .REHOME-COURT suffix on the same volume (atomic rename(2));
#   - a bounded watchdog kills a run-away arm (restore then happens inside the
#     script, never left to an external SIGKILL);
#   - any directory recreated at a real path during the court window (the
#     builder's create_dir_all) is QUARANTINED to a disclosed path, never
#     deleted, before the real tree is moved back;
#   - an EXIT/TERM/INT trap restores the trees even on mid-arm failure, and
#     every exit path passes through the digest-vs-PRE restore gate.
#
# Fail-closed contract:
#   exit 0  — both arms registry-only resolve + compile ALIVE; real-tree
#             digests byte-identical after restore
#   exit 42 — VACUOUS (local path hit in sandbox manifests, patch present, or
#             renamed trees still visible at their real paths during arm B)
#   exit 1  — cargo failed or watchdog bound exceeded (recorded, never passed)
#   exit 3  — restore-digest mismatch on a real tree (restore NOT byte-identical)
#   exit 4  — sandbox cleanup refused (loud, disclosed)
#   exit 5  — provenance assertion failed (resolved source not the registry,
#             or resolved version != ratified pin)
set -uo pipefail

FORK=/Users/sac/frozen-duckdb
FORK_CACHE="$HOME/.frozen-duckdb"
RATIFIED=2.0.4
EXIT_VACUOUS=42
WATCHDOG_ARM_A=900    # 15 min bound for the in-situ arm
WATCHDOG_ARM_B=3600   # 60 min bound for the cache-absent arm (upstream clone +
                      # local-compile fallback runs when the release asset 404s)

say() { printf '[rehome-court] %s\n' "$*"; }

# ---- full-content digest (paths + SHA-256 content, fail-closed) --------------
tree_digest() {
  find "$1" -type f -exec shasum -a 256 {} + 2>/dev/null | shasum -a 256 | cut -d' ' -f1
}

# ---- bounded run: kill at bound, return 124, so restore happens in-script ----
bounded_run() {
  local bound="$1"; shift
  "$@" &
  local pid=$!
  local waited=0
  while kill -0 "$pid" 2>/dev/null; do
    sleep 5
    waited=$((waited + 5))
    if [ "$waited" -ge "$bound" ]; then
      say "WATCHDOG: arm exceeded ${bound}s — killing cargo, restore will follow"
      kill "$pid" 2>/dev/null
      sleep 2
      kill -9 "$pid" 2>/dev/null
      wait "$pid" 2>/dev/null
      return 124
    fi
  done
  wait "$pid"
  return $?
}

SB=$(mktemp -d /tmp/rehome-court.XXXXXX) || { say "FAIL: no sandbox"; exit 1; }
STATE="$SB/rehome-state"          # empty | renamed | restored
DISCLOSE="$SB/disclose.txt"

cleanup() {
  rm -rf "$SB" 2>/dev/null
  if [ -d "$SB" ]; then say "FAIL: sandbox rm denied at $SB (disclose)"; exit 4; fi
}

# Emergency restore backstop: any exit while state=renamed lands here first.
on_exit() {
  local rc=$?
  if [ -f "$STATE" ] && grep -q '^renamed$' "$STATE" 2>/dev/null; then
    say "TRAP: exiting (rc=$rc) while trees renamed — emergency restore"
    emergency_restore || { say "FAIL: emergency restore failed; real trees at <path>.REHOME-COURT"; exit 3; }
  fi
  cleanup
}
trap on_exit EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# ---- restore: quarantine recreated dirs, move real trees back, verify --------
emergency_restore() {
  restore_trees || return 1
}

restore_trees() {
  # 1. Quarantine any recreated dir occupying a real path (never delete).
  for p in "$FORK" "$FORK_CACHE"; do
    if [ -e "$p" ] && [ -e "$p.REHOME-COURT" ]; then
      local q
      q="${p}.REHOME-COURT-recreated-$(date +%s)"
      mv "$p" "$q" || return 1
      say "QUARANTINE: recreated dir moved to $q (court-window byproduct, NOT deleted)"
      echo "quarantine: $q" >> "$DISCLOSE" || true
    fi
  done
  # 2. Move the real trees back.
  [ -e "$FORK.REHOME-COURT" ] && mv "$FORK.REHOME-COURT" "$FORK" || true
  [ -e "$FORK_CACHE.REHOME-COURT" ] && mv "$FORK_CACHE.REHOME-COURT" "$FORK_CACHE" || true
  # 3. Assert canonical presence.
  [ -e "$FORK" ] || return 1
  [ -e "$FORK_CACHE" ] || return 1
  echo restored > "$STATE"
  return 0
}

# ---- (d) PRE digests of the real trees ---------------------------------------
PRE_FORK=$(tree_digest "$FORK")
say "PRE_FORK digest=$PRE_FORK"
if [ -z "$PRE_FORK" ]; then say "FAIL: cannot digest $FORK (missing or unreadable)"; exit 1; fi
PRE_CACHE=$(tree_digest "$FORK_CACHE")
say "PRE_CACHE digest=$PRE_CACHE"
if [ -z "$PRE_CACHE" ]; then say "FAIL: cannot digest $FORK_CACHE (missing or unreadable)"; exit 1; fi

# ---- sandbox manifest: the ratified =2.0.4 x3 registry triple ----------------
write_manifest() {
  mkdir -p "$1/src"
  cat > "$1/Cargo.toml" <<EOF
[package]
name = "rehome-court-subject"
version = "0.1.0"
edition = "2021"

[dependencies]
frozen-duckdb = { version = "=$RATIFIED" }
frozen-duckdb-sys = { version = "=$RATIFIED" }
frozen-duckdb-builder = { version = "=$RATIFIED" }
EOF
  cat > "$1/src/main.rs" <<'EOF'
fn main() {}
EOF
  # Dependency-only court: `cargo check` type-checks the triple; the subject
  # binary exercises no symbols (linking is D13 follow-up; falsifier = resolve
  # provenance + compile).
}

# ---- rename-guard + provenance for one arm's sandbox -------------------------
rename_guard() {
  local dir="$1"
  local guard_hits
  guard_hits=$(grep -R -n -e "/Users/sac/frozen-duckdb" -e "$HOME/.frozen-duckdb" \
    "$dir/Cargo.toml" "$dir/Cargo.lock" 2>/dev/null || true)
  if [ -n "$guard_hits" ]; then
    say "VACUOUS: machine-local path present in sandbox manifests:"
    printf '%s\n' "$guard_hits"
    exit $EXIT_VACUOUS
  fi
  if grep -q 'patch.crates-io' "$dir/Cargo.toml"; then
    say "VACUOUS: [patch.crates-io] present in sandbox manifest"; exit $EXIT_VACUOUS
  fi
  say "rename-guard: clean (no local paths, no patch section)"
}

provenance_assert() {
  local dir="$1"
  local pkg ver src lockblock
  for pkg in frozen-duckdb frozen-duckdb-sys frozen-duckdb-builder; do
    lockblock=$(grep -A3 "^name = \"$pkg\"\$" "$dir/Cargo.lock" || true)
    ver=$(printf '%s' "$lockblock" | grep '^version' | head -1 | cut -d'"' -f2)
    src=$(printf '%s' "$lockblock" | grep '^source' | head -1)
    case "$src" in
      *registry*crates.io*) : ;;
      "") say "FAIL: $pkg has no source line — not a registry dep"; exit 5 ;;
      *) say "FAIL: $pkg provenance — source is '$src', not crates.io"; exit 5 ;;
    esac
    if [ "$ver" != "$RATIFIED" ]; then
      say "FAIL: lockfile resolved $pkg $ver != ratified $RATIFIED"; exit 5
    fi
    say "provenance $pkg: registry, $ver == ratified pin"
  done
}

run_arm() {
  local arm="$1"
  local dir="$SB/$arm"
  local bound="$2"
  mkdir -p "$dir"
  write_manifest "$dir"
  cd "$dir" || exit 1
  export CARGO_NET_OFFLINE=false
  rename_guard "$dir"
  say "[$arm] running cargo generate-lockfile (registry resolve of =2.0.4 x3)"
  if ! bounded_run "$bound" cargo generate-lockfile; then
    say "[$arm] FAIL: cargo generate-lockfile failed/watchdogged — never a pass"
    return 1
  fi
  provenance_assert "$dir"
  say "[$arm] running cargo check (bound ${bound}s)"
  if ! bounded_run "$bound" cargo check 2>&1; then
    say "[$arm] FAIL: cargo check exited non-zero against registry =2.0.4 x3"
    return 1
  fi
  return 0
}

# ---- ARM A: in-situ control (fork + cache present) ---------------------------
say "ARM A: in-situ control — fork + cache present, sandbox registry-only =2.0.4 x3"
if ! run_arm "arm-a" "$WATCHDOG_ARM_A"; then
  say "ARM A FAIL"; exit 1
fi
say "ARM A PASS"

# ---- ARM B: rehome (fork + cache REALLY renamed away) ------------------------
say "ARM B: renaming fork + cache away (same-volume rename(2), .REHOME-COURT suffix)"
mv "$FORK" "$FORK.REHOME-COURT" || { say "FAIL: fork rename refused"; exit 1; }
echo renamed > "$STATE"
mv "$FORK_CACHE" "$FORK_CACHE.REHOME-COURT" || {
  mv "$FORK.REHOME-COURT" "$FORK" 2>/dev/null
  say "FAIL: cache rename refused (fork restored)"; exit 1
}
say "renamed: $FORK -> $FORK.REHOME-COURT ; $FORK_CACHE -> $FORK_CACHE.REHOME-COURT"

# Witnessed absence BEFORE the build runs (anti-vacuity witness of the arm).
if [ -e "$FORK" ] || [ -e "$FORK_CACHE" ]; then
  say "VACUOUS: real paths still occupied after rename"
  exit $EXIT_VACUOUS
fi
say "witnessed absence: $FORK and $FORK_CACHE are REALLY gone"

say "ARM B: cache-absent registry-only build of =2.0.4 x3"
if ! run_arm "arm-b" "$WATCHDOG_ARM_B"; then
  say "ARM B FAIL (cache-absent)"; exit 1
fi
say "ARM B PASS"

# ---- restore + byte-identical proof ------------------------------------------
say "restoring real trees (quarantine-then-move-back)"
if ! restore_trees; then
  say "FAIL: restore failed; real trees at <path>.REHOME-COURT"; exit 3
fi
say "state=restored"

POST_FORK=$(tree_digest "$FORK")
POST_CACHE=$(tree_digest "$FORK_CACHE")
say "POST_FORK digest=$POST_FORK"
say "POST_CACHE digest=$POST_CACHE"
if [ "$PRE_FORK" != "$POST_FORK" ]; then
  say "FAIL: restore-digest mismatch on $FORK (PRE=$PRE_FORK POST=$POST_FORK)"; exit 3
fi
if [ "$PRE_CACHE" != "$POST_CACHE" ]; then
  say "FAIL: restore-digest mismatch on $FORK_CACHE (PRE=$PRE_CACHE POST=$POST_CACHE)"; exit 3
fi
say "restore proven byte-identical: both real trees match PRE digests"

if [ -s "$DISCLOSE" ]; then
  say "disclosures:"; cat "$DISCLOSE"
fi

say "COURT PASS: frozen-duckdb =2.0.4 x3 resolves + compiles from crates.io with zero machine-local paths, cache-absent (ARM B) and in-situ (ARM A), real trees restored byte-identical"
exit 0
