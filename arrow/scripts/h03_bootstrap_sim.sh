#!/usr/bin/bash
# H03 gate-closure lane: tier-order publish-bootstrap simulation.
# Real `cargo package --allow-dirty --no-verify` per closure crate in tier
# order, publish-shaped resolution against a local directory source seeded
# with already-packaged siblings (crates.io bottom-up bootstrap law; no
# registry touch, no git). Cargo.lock snapshotted before + byte-restored
# after by the caller (cmp-gated).
set -u
cd /Users/sac/ggen-legacy/stardf/arrow

OUT=/tmp/h03-sim-results.txt
: > "$OUT"
# Stale/poisoned seeds from interrupted runs break every directory-source
# query (checksum load fails) — start each run from externals-only.
rm -rf /tmp/stardf-vendor/stardf-* /tmp/stardf-vendor/purrdf-* 2>/dev/null

# publish_stardf.py TIERS, flattened in strict topo order (tier = line).
ORDER="stardf-hash stardf-alloc-probe stardf-arrow-core \
stardf-lex stardf-deflate stardf-branchless stardf-testkit stardf-events stardf-arrow-bench stardf-columnar \
stardf-stack stardf-iri stardf-arrow-rs stardf-xsd stardf-arrow-capi stardf-scan-duckdb \
stardf-cdt \
stardf-core stardf-sparql-algebra \
stardf-sparql-results \
stardf-rdf \
stardf-sql stardf-n3 stardf-toml \
stardf-rs"

declare -A PKGNAME=(
)

for crate in $ORDER; do
  pkg="$crate"
  tb="target/package/${pkg}-26.11.1.crate"
  rm -f "$tb"
  CARGO_HOME=/tmp/h03-ch cargo package --offline --allow-dirty --no-verify -p "$crate" > /tmp/h03-pkg.log 2>&1
  ec=$?
  if [ $ec -eq 0 ] && [ -f "$tb" ]; then
    entries=$(tar tzf "$tb" | wc -l | tr -d ' ')
    bytes=$(stat -f%z "$tb")
    lic=$(tar tzf "$tb" | grep -cE '(^|/)(LICENSE|NOTICE|UNLICENSE)' | tr -d ' ')
    seed=/tmp/stardf-vendor/${pkg}-26.11.1
    rm -rf "$seed" 2>/dev/null
    tar xzf "$tb" -C /tmp/stardf-vendor/
    python3 - "$seed" "$tb" <<'PYEOF'
import hashlib, json, os, sys
seed, tb = sys.argv[1], sys.argv[2]
files = {}
for root, _, names in os.walk(seed):
    for n in names:
        p = os.path.join(root, n)
        rel = os.path.relpath(p, seed)
        if rel == ".cargo-checksum.json":
            continue
        files[rel] = hashlib.sha256(open(p, "rb").read()).hexdigest()
pkg = hashlib.sha256(open(tb, "rb").read()).hexdigest()
open(os.path.join(seed, ".cargo-checksum.json"), "w").write(
    json.dumps({"files": files, "package": pkg}))
PYEOF
    echo "OK $crate pkg=$pkg entries=$entries bytes=$bytes license_rows=$lic" >> "$OUT"
  else
    err=$(grep -m1 "no matching package named" /tmp/h03-pkg.log | head -1)
    [ -z "$err" ] && err=$(grep -m1 "^error" /tmp/h03-pkg.log | head -1)
    echo "FAIL $crate pkg=$pkg exit=$ec :: $err" >> "$OUT"
  fi
  tail -1 "$OUT" >&2
done
echo "SIM DONE" >> "$OUT"
