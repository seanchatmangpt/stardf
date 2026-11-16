#!/usr/bin/env python3
"""
stardf/arrow/scripts/verify_header_freeze.py
F4 header-freeze court — docs/jira/v26.10.31/04-falsifiers.md (W4):
  "stardf snapshot regenerated from contract targetSymbols; test asserts full
   prototype-set equality with the committed stardf header. Flips when a
   prototype is added/changed/removed (GONE/NEW diff report, ported from
   abi_signatures.rs:239-286)."

Three surfaces, one symbol set each:
  S_ontology  CabiExport targetSymbols (rdflib query, ontology.ttl) — the contract
  S_snapshot  committed snapshots/prototypes.snapshot — the FROZEN state
  S_header    function declarations in
              crates/stardf-arrow-capi/include/stardf_arrow.h

Check mode (default):
  1. regenerate the snapshot deterministically from S_ontology and diff vs the
     committed snapshot: GONE = frozen but no longer regenerated (the contract
     dropped/renamed a symbol), NEW = regenerated but never frozen (undeclared
     addition). Set + '# count:' arity + byte equality; any drift -> exit 1.
     This is the direction the live three-way parity cannot see: a
     consistently-edited ontology+header+impl triple still passes verify.py,
     but flips here against the frozen state.
  2. diff S_ontology vs S_header: GONE = contract symbol absent from the
     header; NEW = header decl the contract never froze. Exit 1 both ways.

--write-snapshot: (re)pin snapshots/prototypes.snapshot from the live
  ontology (explicit regeneration step; the check never re-pins silently).

--court (non-vacuity, shim-court shape): baseline (pristine check must exit 0)
  -> shim (mutate exactly one thing in a TEMP copy each time, real files are
  never written):
     A. rename ONE declaration in a temp header copy       -> must FAIL
     B. rename ONE targetSymbol in a temp ontology copy    -> must FAIL
     C. drop ONE symbol line from a temp snapshot copy     -> must FAIL
  each flip must carry the EXPECTED GONE/NEW membership (not merely a nonzero
  exit) -> restore proof (sha256 of the three real files byte-identical
  before/after). A court that cannot flip is a VACUOUS GATE -> exit 1.

Depends on nothing outside this directory; reuses scripts/verify.py parsing
(header_declarations, ontology_target_symbols, render/snapshot_diff) — one
implementation of the extraction calculus, no N-copies drift.
"""

import argparse
import hashlib
import os
import shutil
import sys
import tempfile

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
if SCRIPT_DIR not in sys.path:
    sys.path.insert(0, SCRIPT_DIR)

import verify  # scripts/verify.py — parsing + snapshot helpers (single implementation)

ARROW_DIR = verify.ARROW_DIR
DEFAULT_TTL = verify.DEFAULT_TTL
DEFAULT_HEADER = verify.DEFAULT_HEADER
DEFAULT_SNAPSHOT = verify.DEFAULT_SNAPSHOT

# The frozen symbol sacrificed in the court's mutations (present in ontology
# label+targetSymbol literals and in one header declaration; no other symbol
# contains it as a substring).
PROBE_SYMBOL = "stardf_arena_intern"
PROBE_RENAMED = "sd_f4_probe_renamed"


def sha256_file(path: str) -> str:
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def header_diff(contract: set, header_path: str) -> dict:
    """S_ontology vs header declarations: GONE = contract symbol missing from
    the header, NEW = header decl never in the contract. Silent core."""
    if not os.path.isfile(header_path):
        return {
            "ok": False, "gone": [], "new": [], "decls": [],
            "errors": [("HEADER_MISSING", f"header not found: {header_path}")],
        }
    decls = verify.header_declarations(header_path)
    if not decls:
        return {
            "ok": False, "gone": [], "new": [], "decls": [],
            "errors": [(
                "HEADER_EMPTY",
                f"no declarations parsed from {header_path} (empty census is not a pass)",
            )],
        }
    errors = []
    gone = sorted(contract - decls)
    new = sorted(decls - contract)
    if gone:
        errors.append((
            "HEADER_GONE",
            f"contract targetSymbols absent from header declarations: {gone}",
        ))
    if new:
        errors.append((
            "HEADER_NEW",
            f"header declarations with no contract targetSymbol: {new}",
        ))
    return {"ok": not errors, "gone": gone, "new": new,
            "decls": sorted(decls), "errors": errors}


def run_check(ttl: str, header: str, snapshot: str, quiet: bool = False) -> dict:
    """The freeze court on explicit paths. Returns structured results; prints
    the GONE/NEW report unless quiet."""
    result = {"ok": True, "onto": set(), "snap": None, "hdr": None}

    if not os.path.isfile(ttl):
        result["ok"] = False
        result["errors"] = [("ONTOLOGY_MISSING", f"ontology not found: {ttl}")]
        if not quiet:
            for code, msg in result["errors"]:
                print(f"[FAIL] freeze [{code}]: {msg}", file=sys.stderr)
        return result
    try:
        onto = verify.ontology_target_symbols(ttl)
    except Exception as e:
        result["ok"] = False
        result["errors"] = [("ONTOLOGY_PARSE", str(e))]
        if not quiet:
            for code, msg in result["errors"]:
                print(f"[FAIL] freeze [{code}]: {msg}", file=sys.stderr)
        return result
    if not onto:
        result["ok"] = False
        result["errors"] = [(
            "ONTOLOGY_EMPTY_EXPORTS",
            "no CabiExport targetSymbols parsed from ontology (empty census is not a pass)",
        )]
        if not quiet:
            for code, msg in result["errors"]:
                print(f"[FAIL] freeze [{code}]: {msg}", file=sys.stderr)
        return result
    result["onto"] = onto

    snap = verify.snapshot_diff(onto, snapshot)
    hdr = header_diff(onto, header)
    result["snap"], result["hdr"] = snap, hdr
    result["ok"] = snap["ok"] and hdr["ok"]
    result["errors"] = snap["errors"] + hdr["errors"]

    if not quiet:
        print(
            f"[INFO] contract: {len(onto)} CabiExport targetSymbols; "
            f"header: {len(hdr['decls'])} declarations; snapshot: {snapshot}"
        )
        verify.print_snapshot_errors("snapshot-freeze", snap["errors"])
        for code, msg in hdr["errors"]:
            print(f"[FAIL] header-freeze [{code}]: {msg}", file=sys.stderr)
        if result["ok"]:
            print(
                f"[PASS] header-freeze: {len(onto)} contract symbols == "
                f"{len(hdr['decls'])} header declarations, and committed snapshot "
                "is set+byte+count identical to the regeneration (no GONE, no NEW)."
            )
    return result


def mutate_text(src_path: str, dst_path: str, old: str, new: str, what: str) -> int:
    """Write a temp copy of src with every `old` replaced by `new`; returns
    replacement count (0 = the mutation missed its target -> court must fail)."""
    with open(src_path, "r", encoding="utf-8") as f:
        text = f.read()
    count = text.count(old)
    with open(dst_path, "w", encoding="utf-8") as f:
        f.write(text.replace(old, new))
    print(f"[SHIM-COURT] {what}: replaced {count} occurrence(s) of '{old}' in temp copy")
    return count


def run_court(args) -> int:
    real = {"ttl": args.ttl, "header": args.header, "snapshot": args.snapshot}
    failures = 0

    for label, path in real.items():
        if not os.path.isfile(path):
            print(f"[FAIL] court: real {label} not found: {path}", file=sys.stderr)
            return 1
    before = {k: sha256_file(p) for k, p in real.items()}

    # ---- baseline: pristine must PASS -----------------------------------
    print("=== F4 header-freeze court — baseline (pristine) ===")
    base = run_check(real["ttl"], real["header"], real["snapshot"])
    if base["ok"]:
        print("[PASS] court baseline: pristine check exits 0.")
    else:
        print("[FAIL] court baseline: pristine check already fails; the court cannot start from a red ladder.", file=sys.stderr)
        failures += 1

    # ---- shims: one mutation each, in TEMP copies ------------------------
    tmpdir = tempfile.mkdtemp(prefix="f4-court-")
    flips = []
    try:
        print("=== F4 court — shim A: rename ONE header declaration (temp copy) ===")
        t_header = os.path.join(tmpdir, "stardf_arrow.h")
        n = mutate_text(real["header"], t_header, PROBE_SYMBOL, PROBE_RENAMED, "shim A (header decl rename)")
        ra = run_check(real["ttl"], t_header, real["snapshot"], quiet=True)
        ok_a = (
            n > 0
            and not ra["ok"]
            and PROBE_SYMBOL in ra["hdr"]["gone"]
            and PROBE_RENAMED in ra["hdr"]["new"]
            and ra["snap"] is not None and ra["snap"]["ok"]  # isolated flip
        )
        flips.append(("A header-decl rename", ok_a,
                      f"header GONE={ra['hdr']['gone']} NEW={ra['hdr']['new']} (snapshot diff stayed clean)"))

        print("=== F4 court — shim B: rename ONE targetSymbol (temp ontology copy) ===")
        t_ttl = os.path.join(tmpdir, "mutant.ttl")
        n = mutate_text(real["ttl"], t_ttl, PROBE_SYMBOL, PROBE_RENAMED, "shim B (targetSymbol rename)")
        rb = run_check(t_ttl, real["header"], real["snapshot"], quiet=True)
        # Snapshot diff (baseline=committed, current=regeneration): the probe
        # is GONE from the regeneration, the rename is NEW. Header diff
        # (baseline=contract regeneration, current=header): the header still
        # carries the probe -> probe is NEW, rename is GONE.
        ok_b = (
            n > 0
            and not rb["ok"]
            and PROBE_SYMBOL in rb["snap"]["gone"]
            and PROBE_RENAMED in rb["snap"]["new"]
            and PROBE_RENAMED in rb["hdr"]["gone"]
            and PROBE_SYMBOL in rb["hdr"]["new"]
        )
        flips.append(("B ontology-targetSymbol rename", ok_b,
                      f"snapshot GONE={rb['snap']['gone']} NEW={rb['snap']['new']}; header GONE={rb['hdr']['gone']} NEW={rb['hdr']['new']}"))

        print("=== F4 court — shim C: drop ONE line from snapshot (temp copy) ===")
        t_snap = os.path.join(tmpdir, "mutant.snapshot")
        with open(real["snapshot"], "r", encoding="utf-8") as f:
            lines = f.readlines()
        probe_line = PROBE_SYMBOL + "\n"
        dropped = lines.count(probe_line)
        with open(t_snap, "w", encoding="utf-8") as f:
            f.writelines(ln for ln in lines if ln != probe_line)
        print(f"[SHIM-COURT] shim C (snapshot line drop): dropped {dropped} line(s)")
        rc = run_check(real["ttl"], real["header"], t_snap, quiet=True)
        ok_c = (
            dropped > 0
            and not rc["ok"]
            and PROBE_SYMBOL in rc["snap"]["new"]
            and rc["hdr"] is not None and rc["hdr"]["ok"]  # isolated flip
        )
        flips.append(("C snapshot-line drop", ok_c,
                      f"snapshot NEW={rc['snap']['new']} (header diff stayed clean)"))
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)

    for name, ok, detail in flips:
        if ok:
            print(f"[PASS] court flip {name}: check correctly FAILed ({detail}).")
        else:
            print(f"[FAIL] court flip {name}: expected flip with expected GONE/NEW membership did not happen ({detail}).", file=sys.stderr)
            failures += 1

    # ---- restore proof: real files byte-identical -------------------------
    after = {k: sha256_file(p) for k, p in real.items()}
    if after == before:
        print("[PASS] court restore: real ontology/header/snapshot byte-identical (sha256) before/after the court.")
    else:
        print(f"[FAIL] court restore: real files mutated by the court: before={before} after={after}", file=sys.stderr)
        failures += 1

    if failures:
        print(f"\n[F4 COURT FAIL] {failures} court leg(s) failed — VACUOUS or regressed gate.", file=sys.stderr)
        return 1
    print("\n[F4 COURT PASS] baseline PASS; shims A/B/C each flipped with expected GONE/NEW; restore byte-exact.")
    return 0


def main() -> None:
    parser = argparse.ArgumentParser(description="F4 header-freeze court (stardf-arrow)")
    parser.add_argument("--ttl", default=DEFAULT_TTL, help="Path to ontology.ttl")
    parser.add_argument("--header", default=DEFAULT_HEADER, help="Path to stardf_arrow.h")
    parser.add_argument("--snapshot", default=DEFAULT_SNAPSHOT, help="Path to prototypes.snapshot")
    parser.add_argument(
        "--write-snapshot",
        action="store_true",
        help="(Re)pin the snapshot from the live ontology targetSymbols, exit 0.",
    )
    parser.add_argument(
        "--court",
        action="store_true",
        help="Non-vacuity witness: pristine PASS + three one-mutation flips "
        "(temp copies) + sha256 restore proof; exit 1 if any leg fails.",
    )
    args = parser.parse_args()

    if args.court:
        sys.exit(run_court(args))

    if args.write_snapshot:
        os.makedirs(os.path.dirname(args.snapshot), exist_ok=True)
        onto = verify.ontology_target_symbols(args.ttl)
        if not onto:
            print("[FAIL] no CabiExport targetSymbols parsed; refusing to pin an empty snapshot.", file=sys.stderr)
            sys.exit(1)
        with open(args.snapshot, "w", encoding="utf-8") as f:
            f.write(verify.render_snapshot(onto))
        print(f"[PASS] snapshot write: pinned {len(onto)} symbols -> {args.snapshot}")
        sys.exit(0)

    result = run_check(args.ttl, args.header, args.snapshot)
    sys.exit(0 if result["ok"] else 1)


if __name__ == "__main__":
    main()
