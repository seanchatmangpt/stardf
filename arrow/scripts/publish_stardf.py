#!/usr/bin/env python3
"""stardf publish ladder (lane 14-publish-script, v26.11.1 finisher wave).

Law (lane contract, 2026-10-31):
  * publish footprint = FIRST-CLASS CLOSURE ONLY (D17): everything outside
    the closure stays internal path-only.
  * crates.io law for every PUBLISHED crate: no machine-local path
    overrides; cgap-arrow consumption decoupled (git-source deps cannot be
    published at all); every dependency (normal, dev, build) that resolves
    to a workspace member carries the version+path dual key -- at the
    consuming member AND at the root [workspace.dependencies] entry;
    description / license / keywords / categories metadata present;
    exclude filters keep corpora/testdata out (10 MB tarball ceiling);
    license files physically IN the tarball.
  * frozen-duckdb workspace pin exactly "=2.0.4" (D13 ratification; the
    =2.0.3 cache-absent failure class is moot only at =2.0.4).
  * cgap gate (F06) and R1 license gate exit 0 are prerequisites:
    fail-closed on any non-green gate. No live publish without explicit
    operator authorization; default mode is dry-run (`cargo package
    --list` per crate instead of `cargo publish`).
  * spacing: 25-30 s between successive live publishes (crates.io pacing).

Tier table provenance: the dispatch sketched 6 tiers (Tier0 leaves,
Tier1 hash/lex/iri/arrow-rs, Tier2 xsd/cdt/core/arrow-core, Tier3
rdf/columnar/algebra, Tier4 results/n3/sql, Tier5 umbrella stardf + capi).
The witnessed dependency graph (cargo metadata, re-derived 2026-10-31 by
lane 14 against the live tree) contradicts that sketch; the table below is
the minimal graph-true adjustment, and the runtime topological check
re-derives every intra-closure edge from live cargo metadata and refuses
to run the ladder on any backwards/sideways edge:
  1. stardf-iri depends on stardf-lex, and stardf-cdt depends on
     stardf-xsd/iri/events: both dispatch pairs shared a tier. lex moved
     ahead of iri; cdt got its own tier.
  2. stardf-core depends on branchless/deflate/events as well as the cdt
     chain, so those leaves get an explicit tier.
  3. stardf-arrow-core has ZERO intra-workspace deps (a leaf); both
     stardf-arrow-capi (normal arrow-core, dev columnar) and
     stardf-scan-duckdb (hard columnar, optional arrow-core) level at 2,
     not the later slots a first reading suggests; stardf-arrow-bench
     (contract footprint "arrow core/capi/bench") depends only on
     stardf-arrow-core and levels at 1.
  4. stardf-rdf dev-depends on stardf-sparql-results, and stardf-toml
     dev-depends on stardf-rdf: dev-dep edges are tier constraints too
     (cargo publish resolves dev-deps at package time and crates.io
     validates them at upload), so results precedes rdf precedes toml.

NONCLAIMS (standing vocabulary; do not round up):
  * This script never executes `cargo publish` unless --publish AND the
    confirmation env var are both present. The authoring lane executed
    dry-run only.
  * `cargo package --list` gathers files only; it does NOT resolve
    registry dependencies. The structural gates (G5/G6) are what catch
    unpublishable manifests -- a green listing ladder alone proves
    nothing.
  * A SKIP gate row is not a PASS. Live publish refuses on any SKIP row
    unless --accept-skips is explicit.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import re
import subprocess
import sys
import time
import tomllib
from datetime import datetime, timezone
from pathlib import Path

ARROW_ROOT = Path(__file__).resolve().parents[1]
RECEIPTS_DIR = ARROW_ROOT / "evidence"
DEFAULT_AVEC_REPO = Path("/Users/sac/cgap")
MAX_PACKAGE_BYTES = 10 * 1024 * 1024  # 10 MB crates.io tarball ceiling
SPACING_MIN_S = 25.0
SPACING_MAX_S = 30.0
FROZEN_DUCKDB_PIN = "=2.0.4"  # operator ratification D13
CONFIRM_ENV = "STARDF_PUBLISH_CONFIRM"
CONFIRM_VALUE = "ratified-v26.11.1"
LICENSE_FILE_RE = re.compile(r"(license|notice|unlicense)", re.IGNORECASE)

# Strict topological tiers over the 25-crate closure. Every dependency of a
# tier-k crate must live in a tier < k; re-verified against live cargo
# metadata at run time (normal + dev + build edges, intra-closure only).
TIERS: list[list[str]] = [
    # T0 leaves: zero intra-closure deps (hash is a zero-dep vendored root;
    # arrow-core's only deps are external: cgap-arrow + blake3).
    ["stardf-hash", "stardf-alloc-probe", "stardf-arrow-core"],
    # T1 hash dependents + arrow-core-only dependents (bench, columnar).
    ["stardf-lex", "stardf-deflate", "stardf-branchless", "stardf-testkit",
     "stardf-events", "stardf-arrow-bench", "stardf-columnar"],
    # T2 lex/testkit dependents + crates whose only intra deps level below
    # (capi: arrow-core normal + columnar dev; scan-duckdb: columnar hard +
    # arrow-core optional). scan-duckdb = the contract's "duckdb-scan";
    # frozen-duckdb (external, =2.0.4) is a registry dep, tier-free.
    ["stardf-stack", "stardf-iri", "stardf-arrow-rs", "stardf-xsd",
     "stardf-arrow-capi", "stardf-scan-duckdb"],
    # T3 cdt needs iri + xsd (T2) + events (T1).
    ["stardf-cdt"],
    # T4 core needs cdt (T3) + branchless/deflate/events (T1);
    # algebra needs cdt (T3).
    ["stardf-core", "stardf-sparql-algebra"],
    # T5 results needs core (T4) + arrow-rs (T2).
    ["stardf-sparql-results"],
    # T6 rdf needs core/algebra (T4) + dev-deps results (T5).
    ["stardf-rdf"],
    # T7 sql/n3 need rdf (T6); toml needs core (T4) but dev-deps rdf (T6).
    ["stardf-sql", "stardf-n3", "stardf-toml"],
    # T8 umbrella stardf-rs (crates/stardf-purrdf) needs everything; its
    # optional stardf-sql dep (feature "sql") makes sql (T7) a hard
    # publish-order prerequisite too. NOTE: its other optional dep,
    # stardf-duckdb, is NOT in the closure (publish = false) -- G8 rows
    # cover the registry-404 that would follow.
    ["stardf-rs"],
]
CLOSURE = [c for tier in TIERS for c in tier]
CLOSURE_SET = set(CLOSURE)


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def run(cmd: list[str], cwd: Path) -> tuple[int, str, str]:
    p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    return p.returncode, p.stdout or "", p.stderr or ""


class Gate:
    def __init__(self, gid: str, kind: str = "hard"):
        self.id = gid
        self.kind = kind  # hard | soft
        self.standing = "UNKNOWN"
        self.detail = ""
        self.fails: list[str] = []

    def row(self) -> dict:
        # dict[str, object]: "fails" carries a list[str] (full enumeration in
        # the JSON receipt), the rest are str -- annotated so the mixed-value
        # assignment below type-checks.
        row: dict[str, object] = {"id": self.id, "class": self.kind, "standing": self.standing, "detail": self.detail}
        # G17 fix (disclosed, 2026-10-31): the console detail is capped at 12
        # rows ("+43 more") but the receipt is the landing register -- full
        # enumeration ships in the JSON, never truncated.
        if self.fails:
            row["fails"] = self.fails
        return row


def load_metadata(arrow_root: Path) -> dict:
    rc, out, err = run(["cargo", "metadata", "--offline", "--no-deps", "--format-version", "1"],
                       arrow_root)
    if rc != 0:
        sys.stderr.write("FATAL: cargo metadata failed\n" + err + "\n")
        sys.exit(2)
    return json.loads(out)


def member_map(meta: dict) -> dict:
    return {p["name"]: p for p in meta["packages"]}


def root_workspace_deps(arrow_root: Path) -> dict:
    data = tomllib.loads((arrow_root / "Cargo.toml").read_text())
    return data.get("workspace", {}).get("dependencies", {})


def load_manifests(meta: dict) -> dict:
    """{package_name: parsed manifest dict} for closure crates.

    A manifest that fails to parse stores {"_parse_error": msg} instead;
    G5 reports it as a FAIL row (loudly, never a silent pass).
    """
    out = {}
    for pkg in meta["packages"]:
        if pkg["name"] in CLOSURE_SET:
            try:
                out[pkg["name"]] = tomllib.loads(Path(pkg["manifest_path"]).read_text())
            except tomllib.TOMLDecodeError as exc:
                out[pkg["name"]] = {"_parse_error": str(exc)}
    return out


def manifest_dir(meta: dict, name: str) -> Path:
    for pkg in meta["packages"]:
        if pkg["name"] == name:
            return Path(pkg["manifest_path"]).parent
    raise KeyError(name)


def dep_entries(manifest: dict) -> list:
    """Yield (dep_key, kind, value) for all dependency-table entries.

    Kinds: dependencies | dev-dependencies | build-dependencies, including
    target-specific tables. Any other manifest tables are ignored.
    """
    out = []
    for section in ("dependencies", "dev-dependencies", "build-dependencies"):
        val = manifest.get(section)
        if isinstance(val, dict):
            for dk, dv in val.items():
                out.append((dk, section, dv))
    for key, val in manifest.items():
        if isinstance(val, dict) and key.startswith("target."):
            for section in ("dependencies", "dev-dependencies", "build-dependencies"):
                sub = val.get(section)
                if isinstance(sub, dict):
                    for dk, dv in sub.items():
                        out.append((dk, f"{key}/{section}", dv))
    return out


def resolve_dep(value, key: str, root_deps: dict) -> dict:
    """Normalize one dep declaration.

    Returns {source: workspace-invalid|version|path|git, ...}.
    workspace=true resolves against root [workspace.dependencies].
    """
    if isinstance(value, str):
        return {"source": "version", "version": value, "optional": False}
    if isinstance(value, dict):
        if value.get("workspace") is True:
            if key not in root_deps:
                return {"source": "invalid", "note": "workspace=true but key absent from root [workspace.dependencies]"}
            r = resolve_dep(root_deps[key], key, root_deps)
            if r["source"] == "path":
                r["base"] = "root"  # root entry paths are workspace-root-relative
            return r
        if "git" in value:
            return {"source": "git", "optional": bool(value.get("optional"))}
        if "path" in value:
            return {"source": "path", "rel_path": value["path"], "base": "member",
                    "dual_key": "version" in value,
                    "optional": bool(value.get("optional"))}
        if "version" in value:
            return {"source": "version", "version": value["version"],
                    "optional": bool(value.get("optional"))}
        return {"source": "invalid"}
    return {"source": "invalid"}


def tier_of(name: str) -> int:
    for i, tier in enumerate(TIERS):
        if name in tier:
            return i
    return -1


def check_closure_present(meta: dict) -> list:
    names = member_map(meta)
    missing = [c for c in CLOSURE if c not in names]
    return [f"closure crate absent from workspace: {c}" for c in missing]


def check_tiers(meta: dict) -> list:
    """Strict topo check: dependent tier > dependency tier (intra-closure,
    normal+dev+build)."""
    problems = []
    for pkg in meta["packages"]:
        name = pkg["name"]
        if name not in CLOSURE_SET:
            continue
        mt = tier_of(name)
        for dep in pkg["dependencies"]:
            dn = dep["name"]
            if dn in CLOSURE_SET and tier_of(dn) >= mt:
                problems.append(f"{name}(T{mt}) depends on {dn}(T{tier_of(dn)})")
    return problems


def check_publish_flags(meta: dict) -> list:
    fails = []
    for pkg in meta["packages"]:
        if pkg["name"] in CLOSURE_SET and pkg.get("publish") == []:
            fails.append(f"{pkg['name']}: publish = false (metadata publish==[])")
    return fails


def check_metadata_fields(meta: dict) -> list:
    fails = []
    for pkg in meta["packages"]:
        name = pkg["name"]
        if name not in CLOSURE_SET:
            continue
        missing = [f for f in ("description", "license") if not pkg.get(f)]
        kw = pkg.get("keywords") or []
        cat = pkg.get("categories") or []
        if not (1 <= len(kw) <= 5):
            missing.append(f"keywords count {len(kw)} outside 1..5")
        if not (1 <= len(cat) <= 10):
            missing.append(f"categories count {len(cat)} outside 1..10")
        if missing:
            fails.append(f"{name}: {', '.join(missing)}")
    return fails


def check_dep_hygiene(meta: dict, root_deps: dict, manifests: dict) -> list:
    """Dual-key law, no-local-path, no-git, cgap law -- per closure crate."""
    fails = []
    dirs = {name: manifest_dir(meta, name) for name in manifests}
    for name in CLOSURE:
        if name not in manifests:
            continue
        if "_parse_error" in manifests[name]:
            fails.append(f"{name}: manifest not TOML-parsable: {manifests[name]['_parse_error']}")
            continue
        for dep_key, section, value in dep_entries(manifests[name]):
            r = resolve_dep(value, dep_key, root_deps)
            where = f"{name} [{section}] {dep_key}"
            src = r["source"]
            if src == "invalid":
                fails.append(f"{where}: unparsable/empty dep declaration ({r.get('note', 'no version/path/git')})")
            elif src == "git":
                note = (" (cgap-arrow: git-rev dep cannot be published -- decouple or consume a "
                        "published versioned substrate)") if dep_key == "cgap-arrow" else ""
                fails.append(f"{where}: git-source dependency is publish-blocking{note}")
            elif src == "path" and not r["dual_key"]:
                fails.append(f"{where}: path-only dependency -- dual key (version+path) required")
            elif src == "path" and r["dual_key"]:
                base = ARROW_ROOT if r["base"] == "root" else dirs[name]
                target = (base / r["rel_path"]).resolve()
                if ARROW_ROOT not in target.parents and target != ARROW_ROOT:
                    fails.append(f"{where}: machine-local path override (target outside workspace): {target}")
    return fails


def check_hard_nonclosure_deps(meta: dict) -> list:
    """Deps on workspace members outside the closure are publish-blocking
    (D17: everything else stays internal path-only). Optional deps count:
    `cargo publish` rewrites a dual-keyed optional path dep to version-only
    and crates.io 404s at upload when that version was never published."""
    fails = []
    for pkg in meta["packages"]:
        name = pkg["name"]
        if name not in CLOSURE_SET:
            continue
        for dep in pkg["dependencies"]:
            if dep["name"] in CLOSURE_SET:
                continue
            dep_pkg = member_map(meta).get(dep["name"])
            if dep_pkg is not None:  # a workspace member, but outside closure
                label = "optional" if dep.get("optional") else "hard"
                kind = dep.get("kind") or "normal"
                fails.append(
                    f"{name}: {label} {kind} dep on non-closure workspace member "
                    f"{dep['name']} -- target crate must join the closure or the "
                    f"dep must be dropped/re-routed (D17); registry 404 at upload")
    return fails


def check_frozen_duckdb_pin(root_deps: dict, meta: dict) -> list:
    consumed = any(
        dep["name"] == "frozen-duckdb"
        for pkg in meta["packages"]
        if pkg["name"] in CLOSURE_SET
        for dep in pkg["dependencies"]
    )
    if not consumed:
        return []
    val = root_deps.get("frozen-duckdb")
    req = val.get("version") if isinstance(val, dict) else val
    if req != FROZEN_DUCKDB_PIN:
        return [f"frozen-duckdb workspace pin is {req!r}; operator-ratified D13 pin is {FROZEN_DUCKDB_PIN!r}"]
    return []


def check_patch_sections(meta: dict) -> list:
    """No [patch] tables in the root or in any closure manifest. Paths are
    taken from cargo metadata (package names != directory names)."""
    fails = []
    files = [ARROW_ROOT / "Cargo.toml"] + [
        Path(p["manifest_path"]) for p in meta["packages"] if p["name"] in CLOSURE_SET
    ]
    for f in files:
        if not f.exists():
            fails.append(f"manifest missing: {f}")
            continue
        if re.search(r"^\[(patch|replace)\b", f.read_text(), re.M):
            fails.append(f"{f}: [patch]/[replace] table present "
                         f"(machine-local overrides are publish-blocking)")
    return fails


def run_gate_cmd(g: Gate, cmd: list, cwd: Path, timeout_s: int = 1800) -> None:
    print(f"  [RUN ] {g.id}: {' '.join(cmd)} (cwd={cwd})", flush=True)
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout_s)
    except subprocess.TimeoutExpired:
        g.standing = "FAIL"
        g.detail = f"{' '.join(cmd)} -> TIMEOUT after {timeout_s}s"
        print(f"  [{g.standing}] {g.id}: {g.detail}", flush=True)
        return
    err, out = p.stderr or "", p.stdout or ""
    tail = (err.strip().splitlines() or [""])[-1] if err.strip() else (out.strip().splitlines() or [""])[-1]
    if p.returncode == 0:
        g.standing = "PASS"
        g.detail = f"{' '.join(cmd)} -> exit 0"
    else:
        g.standing = "FAIL"
        g.detail = f"{' '.join(cmd)} -> exit {p.returncode}: {tail}"
    print(f"  [{g.standing}] {g.id}: {g.detail[:200]}", flush=True)


def check_license_gate() -> Gate:
    g = Gate("R1-license-gate")
    script = ARROW_ROOT / "scripts" / "verify_stardf_licenses.py"
    if not script.exists():
        g.standing = "SKIP"
        g.detail = f"gate script missing: {script}"
        return g
    run_gate_cmd(g, ["python3", str(script)], ARROW_ROOT)
    return g


def check_avec_court(avec_repo: Path) -> Gate:
    # G17 fix (disclosed, 2026-10-31): was kind="soft", contradicting the
    # header law "cgap gate (F06) ... fail-closed on any non-green gate" --
    # a soft FAIL would not have blocked live publish.
    g = Gate("cgap-ffi-court")
    court = avec_repo / "crates" / "cgap-ffi" / "tests" / "stardf_consumer_court.rs"
    if not court.exists():
        g.standing = "SKIP"
        g.detail = f"court file absent: {court} (F06 cites 3/3 on it)"
        return g
    run_gate_cmd(g, ["cargo", "test", "-p", "cgap-ffi", "--test", "stardf_consumer_court"], avec_repo)
    return g


def gate_suite(args) -> tuple:
    meta = load_metadata(ARROW_ROOT)
    root_deps = root_workspace_deps(ARROW_ROOT)
    manifests = load_manifests(meta)

    gates = []

    def hard(gid, fails):
        g = Gate(gid)
        g.fails = list(fails)
        if fails:
            g.standing = "FAIL"
            g.detail = "; ".join(fails[:12]) + (f" (+{len(fails) - 12} more)" if len(fails) > 12 else "")
        else:
            g.standing = "PASS"
            g.detail = "clean"
        gates.append(g)

    hard("G1-closure-present", check_closure_present(meta))
    hard("G2-tiers-strict-topo", check_tiers(meta))
    hard("G3-publish-flag", check_publish_flags(meta))
    hard("G4-metadata-fields", check_metadata_fields(meta))
    hard("G5-dep-hygiene", check_dep_hygiene(meta, root_deps, manifests))
    hard("G6-frozen-duckdb-pin", check_frozen_duckdb_pin(root_deps, meta))
    hard("G7-patch-tables", check_patch_sections(meta))
    hard("G8-nonclosure-hard-deps", check_hard_nonclosure_deps(meta))
    gates.append(check_license_gate())
    gates.append(check_avec_court(args.avec_repo))
    g = Gate("G9-cargo-check-workspace")
    if args.skip_check:
        g.standing = "SKIP"
        g.detail = "--skip-check explicit (dry-run only; live publish refuses)"
    else:
        run_gate_cmd(g, ["cargo", "check", "--workspace"], ARROW_ROOT)
    gates.append(g)
    return gates, meta


def ladder_dry_run(arrow_root: Path, meta: dict, start_from, receipt: dict) -> int:
    fails = 0
    dirs = {p["name"]: Path(p["manifest_path"]).parent for p in meta["packages"]}
    started = start_from is None
    for tier_i, tier in enumerate(TIERS):
        for crate in tier:
            if not started:
                if crate == start_from:
                    started = True
                else:
                    continue
            p = subprocess.run(
                ["cargo", "package", "--offline", "--allow-dirty", "--list", "-p", crate],
                cwd=arrow_root, capture_output=True, text=True)
            files = [l for l in p.stdout.splitlines() if l.strip()]
            row = {
                "tier": tier_i,
                "crate": crate,
                "cmd": "cargo package --offline --allow-dirty --list",
                "exit": p.returncode,
            }
            if p.returncode != 0:
                errs = [l for l in p.stderr.strip().splitlines() if l.strip()]
                row["error"] = errs[-1] if errs else "unknown"
                row["standing"] = "FAIL"
                fails += 1
            else:
                row["file_count"] = len(files)
                row["license_files"] = sorted(
                    {os.path.basename(l) for l in files if LICENSE_FILE_RE.search(os.path.basename(l))})
                cdir = dirs[crate]
                row["size_bytes"] = sum(
                    (cdir / l).stat().st_size for l in files if (cdir / l).exists())
                if not files:
                    row["standing"] = "FAIL"
                    row["error"] = "empty package file list"
                    fails += 1
                elif not row["license_files"]:
                    row["standing"] = "FAIL"
                    row["error"] = "no LICENSE/UNLICENSE/NOTICE file in the package list (license files must ship IN the tarball)"
                    fails += 1
                elif row["size_bytes"] > MAX_PACKAGE_BYTES:
                    row["standing"] = "FAIL"
                    row["error"] = f"uncompressed file sum {row['size_bytes']} exceeds 10MB ceiling"
                    fails += 1
                else:
                    row["standing"] = "PASS"
            receipt["ladder"].append(row)
            print(f"  [{row['standing']}] T{tier_i} {crate}: exit={row['exit']} "
                  f"files={row.get('file_count', '-')} licenses={row.get('license_files', '-')} "
                  f"bytes={row.get('size_bytes', '-')} {row.get('error', '')}", flush=True)
            if row["standing"] == "FAIL":
                print("  fail-closed: stopping ladder at first failing row", flush=True)
                return fails
    return fails


def ladder_live(arrow_root: Path, start_from, receipt: dict) -> int:
    fails = 0
    started = start_from is None
    first = True
    for tier_i, tier in enumerate(TIERS):
        for crate in tier:
            if not started:
                if crate == start_from:
                    started = True
                else:
                    continue
            if not first:
                # SLEEP-CLASS: SCHEDULING -- crates.io publish pacing
                # (contract: 25-30 s between publishes); the ladder has
                # nothing else to do during the interval.
                pause = random.uniform(SPACING_MIN_S, SPACING_MAX_S)
                print(f"  spacing {pause:.1f}s ...", flush=True)
                time.sleep(pause)
            first = False
            print(f"  publishing {crate} (T{tier_i}) ...", flush=True)
            p = subprocess.run(["cargo", "publish", "--locked", "-p", crate],
                               cwd=arrow_root, capture_output=True, text=True)
            row = {"tier": tier_i, "crate": crate, "cmd": "cargo publish --locked",
                   "exit": p.returncode}
            if p.returncode != 0:
                errs = [l for l in p.stderr.strip().splitlines() if l.strip()]
                row["error"] = errs[-1] if errs else "unknown"
                row["standing"] = "FAIL"
                fails += 1
            else:
                row["standing"] = "PASS"
            receipt["ladder"].append(row)
            print(f"  [{row['standing']}] T{tier_i} {crate}: exit={p.returncode} {row.get('error', '')}", flush=True)
            if row["standing"] == "FAIL":
                print("  fail-closed: stopping live ladder at first failing publish", flush=True)
                return fails
    return fails


def write_receipt(receipt: dict) -> Path:
    RECEIPTS_DIR.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    path = RECEIPTS_DIR / f"publish_stardf-receipt-{stamp}.json"
    path.write_text(json.dumps(receipt, indent=2) + "\n")
    return path


NONCLAIMS = [
    "cargo package --list does not resolve registry dependencies; the structural gates (G5/G6/G8) carry publishability, not the listing ladder",
    "this receipt was produced without any live cargo publish unless mode == live",
    "a SKIP gate row is not a PASS; live publish refuses on SKIP rows without --accept-skips",
    "per-crate readiness FAIL rows are landing-list items for the manifest lanes, not evidence the ladder script itself is broken",
    "the script never mutates git state; commits/tags belong to the coordinator",
    "dry-run listings pass --allow-dirty (concurrent finisher lanes own uncommitted edits; coordinator owns git); live publish never passes --allow-dirty",
    "G5 enforces the version+path dual key on normal AND dev AND build deps (cargo's packaging version-requirement check witnessed on stardf-iri: 'all dependencies must have a version requirement specified when packaging'); if a witnessed cargo package receipt later proves dev-deps exempt, relax G5 with that receipt, not by assumption",
]


def main() -> int:
    ap = argparse.ArgumentParser(description="stardf closure publish ladder (dry-run by default)")
    ap.add_argument("--publish", action="store_true",
                    help="LIVE publish (also requires env " + CONFIRM_ENV + "=" + CONFIRM_VALUE + ")")
    ap.add_argument("--dry-run", action="store_true", default=False,
                    help="dry-run ladder via cargo package --list (default mode; "
                         "cannot combine with --publish)")
    ap.add_argument("--skip-check", action="store_true",
                    help="skip cargo check --workspace (dry-run only; marks gate SKIP, never green)")
    ap.add_argument("--accept-skips", action="store_true",
                    help="live mode only: allow SKIP gate rows (still fail on FAIL)")
    ap.add_argument("--start-from", metavar="CRATE",
                    help="start the ladder at this crate (gates always run in full)")
    ap.add_argument("--cgap-repo", type=Path, default=DEFAULT_AVEC_REPO,
                    help="path to the cgap checkout for the cgap-ffi court gate")
    args = ap.parse_args()

    live = args.publish
    if args.publish and args.dry_run:
        print("REFUSED: --publish and --dry-run are mutually exclusive", file=sys.stderr)
        return 2
    if args.start_from is not None and args.start_from not in CLOSURE_SET:
        print(f"REFUSED: --start-from {args.start_from!r} is not a closure crate; "
              f"the ladder would silently run zero rows", file=sys.stderr)
        return 2
    if live:
        if os.environ.get(CONFIRM_ENV) != CONFIRM_VALUE:
            print(f"REFUSED: live publish requires {CONFIRM_ENV}={CONFIRM_VALUE}", file=sys.stderr)
            return 2
        if args.skip_check:
            print("REFUSED: --skip-check cannot combine with --publish", file=sys.stderr)
            return 2

    receipt = {
        "identity": {
            "tool": "scripts/publish_stardf.py",
            "workspace_root": str(ARROW_ROOT),
            "generated_at": now_iso(),
            "mode": "live" if live else "dry-run",
        },
        "gates": [],
        "ladder": [],
        "landing_list": [],
        "verdict": "UNKNOWN",
        "nonclaims": NONCLAIMS,
    }
    rc, head_out, _ = run(["git", "rev-parse", "HEAD"], ARROW_ROOT)
    if rc == 0:
        receipt["identity"]["git_head"] = head_out.strip()

    gates, meta = gate_suite(args)
    receipt["gates"] = [g.row() for g in gates]

    hard_fails = [g for g in gates if g.kind == "hard" and g.standing == "FAIL"]
    skips = [g for g in gates if g.standing == "SKIP"]
    print("== gates ==")
    for g in gates:
        print(f"  [{g.standing}] {g.id}: {g.detail[:200]}")

    if hard_fails:
        receipt["landing_list"] = sorted({g.id for g in hard_fails})
        receipt["verdict"] = "BLOCKED"
        print(f"== VERDICT: BLOCKED -- {len(hard_fails)} hard gate(s) red; ladder not attempted ==")
        for g in hard_fails:
            print(f"  BLOCKER {g.id}: {g.detail[:300]}")
        path = write_receipt(receipt)
        print(f"receipt: {path}")
        return 1
    if skips and live and not args.accept_skips:
        receipt["verdict"] = "BLOCKED"
        print("== VERDICT: BLOCKED -- SKIP gate rows present in live mode (use --accept-skips explicitly) ==")
        path = write_receipt(receipt)
        print(f"receipt: {path}")
        return 1

    print("== ladder ==")
    ladder_fails = ladder_live(ARROW_ROOT, args.start_from, receipt) if live \
        else ladder_dry_run(ARROW_ROOT, meta, args.start_from, receipt)

    if ladder_fails == 0 and not skips:
        receipt["verdict"] = "GREEN"
    elif ladder_fails == 0:
        receipt["verdict"] = "GREEN-WITH-SKIPS"
    else:
        receipt["verdict"] = "BLOCKED"

    path = write_receipt(receipt)
    print(f"== VERDICT: {receipt['verdict']} ==")
    print(f"receipt: {path}")
    return 0 if receipt["verdict"] == "GREEN" else 1


if __name__ == "__main__":
    sys.exit(main())
