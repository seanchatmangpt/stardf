#!/usr/bin/bash
# Lane 11 license-carriage: per-crate tarball carriage gate (dry-run only).
# Packages each closure crate locally (--no-verify --allow-dirty --offline),
# witnesses the carriage rows INSIDE the real .crate tarball, and records the
# tarball size against the 10MB ceiling. No registry touch, no git commands.
set -u
cd "$(dirname "$0")/.."

OUT="${1:?usage: lane11_tarball_gate.sh <output-file>}"
: > "$OUT"

# 20 first-class closure packages + the task-named conjunct witness.
CRATES="stardf-arrow-core stardf-arrow-capi stardf-arrow-bench \
stardf-columnar stardf-sql stardf-n3 stardf-scan-duckdb stardf-duckdb \
stardf-hash stardf-branchless stardf-lex stardf-iri stardf-xsd stardf-cdt \
stardf-events stardf-deflate stardf-stack stardf-testkit stardf-alloc-probe \
stardf-toml stardf-rs stardf-core"

for pkg in $CRATES; do
  echo "== $pkg ==" >> "$OUT"
  cargo package --no-verify --allow-dirty --offline -p "$pkg" > "/tmp/lane11_$pkg.log" 2>&1
  ec=$?
  echo "cargo_package_exit=$ec" >> "$OUT"
  if [ $ec -ne 0 ]; then
    tail -5 "/tmp/lane11_$pkg.log" >> "$OUT"
    continue
  fi
  tarball="target/package/${pkg}-26.11.1.crate"
  if [ ! -f "$tarball" ]; then
    echo "TARBALL_MISSING: $tarball" >> "$OUT"
    continue
  fi
  size=$(stat -f%z "$tarball")
  echo "size_bytes=$size" >> "$OUT"
  lic=$(tar -tzf "$tarball" | grep -cE 'licenses/LICENSE-(MIT|APACHE|MULAN)$')
  prov=$(tar -tzf "$tarball" | grep -cE '(PROVENANCE|VENDOR-PROVENANCE|FALSIFIER-RECEIPT)\.md$')
  tp=$(tar -tzf "$tarball" | grep -cE 'licenses/third-party/')
  notice=$(tar -tzf "$tarball" | grep -cE 'licenses/NOTICE.txt$')
  echo "tar_rows lic=$lic prov=$prov tp=$tp notice=$notice" >> "$OUT"
  if [ "$size" -ge 10485760 ]; then
    echo "OVER_10MB_CEILING" >> "$OUT"
  fi
done
