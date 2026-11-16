#!/bin/sh
# PP-4 anti-vacuity flip courts: /tmp sandbox copy, SHIM-COURT mutation,
# the courts MUST fail; sha256 fail-closed restore.
# Usage: courts.sh --flip-trap-bypass | --flip-wrong-index | --measure
set -e
ARROW=/Users/sac/stardf/arrow
CRATE=$ARROW/crates/stardf-inline
SB=/tmp/pp4-sandbox
export CARGO_TARGET_DIR=/tmp/pp4-target

flip() {
  rm -rf "$SB"; mkdir -p "$SB"
  cp -R "$CRATE" "$SB/stardf-inline"
  python3 - "$ARROW" <<'EOF'
import pathlib, sys
arrow = sys.argv[1]
cargo = pathlib.Path('/tmp/pp4-sandbox/stardf-inline/Cargo.toml')
t = cargo.read_text()
cargo.write_text(t.replace('../stardf-hash', arrow + '/crates/stardf-hash')
                      .replace('../stardf-star', arrow + '/crates/stardf-star'))
EOF
  cp "$ARROW/rust-toolchain.toml" "$SB/stardf-inline/rust-toolchain.toml"
  cd "$SB/stardf-inline"
  find . -type f -name '*.rs' | sort | xargs shasum -a 256 | shasum -a 256 > /tmp/pp4_sha_before
  rm -rf "$SB/pristine"
  cp -R "$SB/stardf-inline" "$SB/pristine"
  echo "=== BEFORE (unmutated sandbox, must pass) ==="
  cargo test --test inline_courts > /tmp/pp4_before.log 2>&1 && echo "BEFORE: PASS (exit 0)" || { echo "BEFORE: FAIL"; exit 1; }
}

restore_check() {
  rm -rf "$SB/stardf-inline"
  cp -R "$SB/pristine" "$SB/stardf-inline"
  cd "$SB/stardf-inline"
  find . -type f -name '*.rs' | sort | xargs shasum -a 256 | shasum -a 256 > /tmp/pp4_sha_after
  if cmp -s /tmp/pp4_sha_before /tmp/pp4_sha_after; then
    echo "RESTORE: byte-exact (sha256 match)"
  else
    echo "RESTORE: FAILED"; exit 1
  fi
}

case "$1" in
  --flip-trap-bypass)
    flip
    echo "=== SHIM (SHIM-COURT: collision-trap bypass) ==="
    python3 - <<'EOF'
import pathlib
p = pathlib.Path("src/store.rs")
t = p.read_text()
old = '''            (slot.payload == payload).then_some(()).ok_or(FoldError::Trapped)'''
if old not in t:
    print("shim target missing"); exit(1)
# bypass: accept ANY payload that lands on an occupied fingerprint
p.write_text(t.replace(old, '''            (true).then_some(()).ok_or(FoldError::Trapped)'''))
EOF
    echo "=== SHIM applied ==="
    if cargo test --test inline_courts court2 > /tmp/pp4_shim.log 2>&1; then
      echo "AFTER-SHIM: PASS => VACUOUS GATE"; exit 1
    else
      echo "AFTER-SHIM: court2 FAILS (exit $?) => FLIP CONFIRMED"
    fi
    restore_check
    ;;
  --flip-wrong-index)
    flip
    echo "=== SHIM (SHIM-COURT: wrong arena index) ==="
    python3 - <<'EOF'
import pathlib
p = pathlib.Path("src/store.rs")
t = p.read_text()
old = '''        self.table.insert(fp, idx);'''
if old not in t:
    print("shim target missing"); exit(1)
p.write_text(t.replace(old, '''        self.table.insert(fp, idx + 1);'''))
EOF
    echo "=== SHIM applied ==="
    if cargo test --test inline_courts court1 > /tmp/pp4_shim.log 2>&1; then
      echo "AFTER-SHIM: PASS => VACUOUS GATE"; exit 1
    else
      echo "AFTER-SHIM: court1 FAILS (exit $?) => FLIP CONFIRMED"
    fi
    restore_check
    ;;
  --measure)
    cd "$CRATE"
    cargo test --test inline_courts -- --nocapture --test-threads=1
    ;;
esac
