#!/bin/sh
# RDF-4 anti-vacuity flip court: sandbox copy in /tmp, mutate the metadata
# attachment path, the metadata court MUST fail; restore byte-exact.
# Usage: courts.sh --flip  |  courts.sh --measure
set -e
ARROW=/Users/sac/stardf/arrow
CRATE=$ARROW/crates/stardf-star
SB=/tmp/r4m-sandbox
export CARGO_TARGET_DIR=/tmp/r4m-target

flip() {
  rm -rf "$SB"; mkdir -p "$SB"
  cp -R "$CRATE" "$SB/stardf-star"
  # sandbox: repoint the path dep at the real vendored stardf-hash
  python3 -c "import pathlib; p=pathlib.Path('$SB/stardf-star/Cargo.toml'); p.write_text(p.read_text().replace('path = \"../stardf-hash\"', 'path = \"$ARROW/crates/stardf-hash\"'))"
  cp "$ARROW/rust-toolchain.toml" "$SB/stardf-star/rust-toolchain.toml"
  cd "$SB/stardf-star"
  echo "=== BASELINE sha256 ==="; find . -type f -name '*.rs' | sort | xargs shasum -a 256 | shasum -a 256
  echo "=== BEFORE (unmutated sandbox, must pass) ==="
  cargo test --test quoted_courts court4 > /tmp/r4m_before.log 2>&1 && echo "BEFORE: PASS (exit 0)" || { echo "BEFORE: FAIL"; exit 1; }
  echo "=== SHIM (SHIM-COURT: wrong-id metadata attachment) ==="
  python3 - <<'EOF'
import pathlib
p = pathlib.Path("tests/quoted_courts.rs")
t = p.read_text()
old = '''        store.add(Triple::new(
            q,
            "https://ex.test/score",
            Term::Literal { value: format!("{}", 90 + ni), lang: None, datatype: None },
        ));'''
if old not in t:
    print("shim target missing"); exit(1)
wrong = old.replace("q,", "match &inner { Term::Quoted(id) => Term::Quoted(*id), _ => unreachable!() },")
t = t.replace(old, wrong, 1)
p.write_text(t)
EOF
  echo "=== SHIM applied ==="
  if cargo test --test quoted_courts > /tmp/r4m_shim.log 2>&1; then
    echo "AFTER-SHIM: PASS => VACUOUS GATE (exit 1 required)"; exit 1
  else
    echo "AFTER-SHIM: FAIL (court flipped) => gate holds"
  fi
  echo "=== RESTORE ==="
  rm -rf "$SB"
  echo "restored (sandbox removed; shared tree never touched)"
}

measure() {
  cd "$CRATE"
  cargo test --test quoted_courts --release court2 > /tmp/r4m_measure.log 2>&1 || { echo "measure: courts must pass first"; exit 1; }
}

case "$1" in
  --flip) flip ;;
  --measure) measure ;;
  *) echo "usage: courts.sh --flip|--measure"; exit 2 ;;
esac
