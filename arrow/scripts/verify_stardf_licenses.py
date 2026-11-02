#!/usr/bin/env python3
"""StarDF license-compatibility gate (P0-18 / P0-19 R1).

Falsifier (04-falsifiers.md R1): a dependency-edge gate that FAILs (exit 1)
on a P28-SIL -> MIT/Apache/MulanPSL edge without a registered compatibility
receipt in authority/stardf/license-compat.json.

Chicago-school: real reads of real manifests; absent inputs are loud [SKIP]
+ nonzero, never silent passes. No fabrication: the compat register is the
only license registry, and an unregistered dependency is a FAIL row
(loudly-missing), not a guessed classification.
"""

import argparse
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
DEFAULT_COMPAT = REPO / "authority" / "stardf" / "license-compat.json"
ARROW_ROOT = REPO / "stardf" / "arrow"

# Gated permissive class per 04-falsifiers.md R1: MIT / Apache / MulanPSL.
# A P28-SIL crate consuming any of these without a compatibility receipt
# is a FAIL. Match is by license-token containment (handles SPDX OR/AND
# conjunctions: any arm landing in the gated class gates the edge).
GATED_LICENSE_TOKENS = ("MIT", "Apache-2.0", "MulanPSL-2.0")

MANIFESTS = [
    "Cargo.toml",
    "crates/stardf-arrow-core/Cargo.toml",
    "crates/stardf-arrow-capi/Cargo.toml",
    "crates/stardf-arrow-bench/Cargo.toml",
]

SECTION_RE = re.compile(r"^\s*\[([^\]]+)\]\s*$")
DEP_LINE_RE = re.compile(r"^([A-Za-z0-9_-]+)\s*=\s*(.+)$")


def parse_manifest(path: Path):
    """Return (own_license_or_None, {dep_name: dep_spec_string}).

    Minimal flat-table parser sufficient for these manifests: tracks the
    current [section], captures `license = "..."` under [package] and
    `name = ...` lines under dependency tables. Intra-workspace path deps
    are marked and exempt from the gate.
    """
    own_license = None
    deps = {}
    section = None
    for line in path.read_text().splitlines():
        sec = SECTION_RE.match(line)
        if sec:
            section = sec.group(1).strip()
            continue
        m = DEP_LINE_RE.match(line)
        if not m or section is None:
            continue
        key, val = m.group(1), m.group(2).strip()
        if section == "package" and key == "license":
            own_license = val.strip('"')
        elif section in ("dependencies", "workspace.dependencies") and key not in deps:
            deps[key] = val
    return own_license, deps


def is_workspace_path_dep(val: str) -> bool:
    return "path =" in val or val.startswith('"../') or "workspace = true" in val


def classify_license(lic):
    """Return 'gated' | 'sovereign' | 'unknown' for a license string."""
    if lic is None:
        return "unknown"
    for token in GATED_LICENSE_TOKENS:
        if token in lic:
            return "gated"
    if lic.strip() in ("CC0-1.0", "Unlicense", "MIT-0"):
        return "sovereign-class"
    return "unknown"


def main() -> int:
    ap = argparse.ArgumentParser(description="StarDF license-compat gate (R1)")
    ap.add_argument("--compat", default=str(DEFAULT_COMPAT),
                    help="license-compat.json path")
    ap.add_argument("--inventory", default=str(REPO / "authority" / "stardf" / "license-inventory.toml"),
                    help="license-inventory.toml path (conjunct-notice tracking)")
    ap.add_argument("--root", default=str(ARROW_ROOT))
    args = ap.parse_args()

    compat_path = Path(args.compat)
    inventory_path = Path(args.inventory)
    root = Path(args.root)
    failures = []
    skips = []
    rows = []

    # ---- input checks: loud [SKIP] + nonzero when an input is missing ----
    if not compat_path.is_file():
        print(f"[SKIP] compat register missing: {compat_path}", file=sys.stderr)
        print("license-compat gate: SKIP (input missing), exit 1", file=sys.stderr)
        return 1
    if not inventory_path.is_file():
        skips.append(f"license-inventory.toml not found: {inventory_path} "
                     f"(conjunct notices untracked: Unicode-3.0/BSD-3-Clause census NOT performed)")

    try:
        compat = json.loads(compat_path.read_text())
    except json.JSONDecodeError as exc:
        print(f"[FAIL] compat register is not valid JSON: {compat_path}: {exc}",
              file=sys.stderr)
        return 1

    receipts = {(r.get("consumer"), r.get("dependency"))
                for r in compat.get("compatibility_receipts", [])}
    registry = {d.get("name"): d for d in compat.get("dependency_licenses", [])}

    # ---- dependency-edge gate over real manifests ----
    for rel in MANIFESTS:
        manifest = root / rel
        if not manifest.is_file():
            failures.append(f"declared workspace manifest missing: {manifest}")
            continue
        own_license, deps = parse_manifest(manifest)
        crate = own_name(manifest) or manifest.parent.name
        rows.append(f"crate={crate} license={own_license or 'UNSPECIFIED'} "
                    f"deps={len(deps)}")
        for name, val in sorted(deps.items()):
            if is_workspace_path_dep(val):
                rows.append(f"  edge {crate} -> {name}: intra-workspace path dep, exempt")
                continue
            entry = registry.get(name)
            if entry is None:
                failures.append(
                    f"edge {crate} -> {name}: dependency not in license register "
                    f"({compat_path.name} dependency_licenses) — loudly-missing")
                continue
            lic = entry.get("license")
            cls = classify_license(lic)
            if cls == "gated":
                if (crate, name) in receipts:
                    rows.append(f"  edge {crate} -> {name}: license={lic!r} GATED, "
                                f"compatibility receipt REGISTERED — pass")
                else:
                    failures.append(
                        f"edge {crate} -> {name}: P28-SIL -> gated license {lic!r} "
                        f"without registered compatibility receipt (R1)")
            elif cls == "sovereign-class":
                rows.append(f"  edge {crate} -> {name}: license={lic!r} sovereign-class — pass")
            else:
                if lic is None:
                    rows.append(f"  edge {crate} -> {name}: no license field "
                                f"(sovereign-fork-posture per register) — pass")
                else:
                    failures.append(
                        f"edge {crate} -> {name}: license {lic!r} not classifiable "
                        f"against gated set {GATED_LICENSE_TOKENS} — loudly-missing")

    # ---- conjunct-notice tracking (Unicode-3.0 / BSD-3-Clause) ----
    if not skips:
        for line in inventory_path.read_text().splitlines():
            if "Unicode-3.0" in line or "BSD-3-Clause" in line:
                print(f"[NOTICE] conjunct license: {line.strip()}")

    # ---- verdict ----
    for s in skips:
        print(f"[SKIP] {s}", file=sys.stderr)
    for r in rows:
        print(f"[ROW] {r}")
    for f in failures:
        print(f"[FAIL] {f}")
    if skips:
        print(f"license-compat gate: SKIP ({len(skips)} input(s) missing), exit 1 — "
              f"a skipped input is never a pass", file=sys.stderr)
        return 1
    if failures:
        print(f"license-compat gate: FAIL ({len(failures)} failure(s))", file=sys.stderr)
        return 1
    print("license-compat gate: PASS (all dependency edges registered and ungated)")
    return 0


def own_name(manifest: Path):
    m = re.search(r'^name\s*=\s*"([^"]+)"', manifest.read_text(), re.M)
    return m.group(1) if m else None


if __name__ == "__main__":
    sys.exit(main())
