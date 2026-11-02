#!/usr/bin/env python3
"""
stardf/arrow/scripts/verify.py
Self-contained deterministic verifier for the stardf-arrow substrate:
  1. rdflib parse of ontology.ttl (classes / individuals census)
  2. header/impl parity: include/stardf_arrow.h declarations vs
     #[no_mangle] exports in crates/stardf-arrow-capi/src (both directions)
  3. ontology<->header cross-check: every stardf-arrow:targetSymbol exported
     by a CabiExport individual in ontology.ttl must appear in BOTH the
     header declaration set and the #[no_mangle] impl set (both directions:
     an ontology symbol absent from header or impl FAILs; a header decl or
     impl export absent from the ontology is caught by the parity check's
     own census-relation below).
  4. --mutate drop-export anti-vacuity: removing one export in memory must
     FAIL BOTH the parity check AND the ontology cross-check (one mutation
     drops one ontology individual AND one impl export, so the anti-vacuity
     covers both directions); a passing mutation is a VACUOUS GATE.
  5. --snapshot check|write (F4 header freeze, docs/jira/v26.10.31/
     04-falsifiers.md W4): `write` (re)pins snapshots/prototypes.snapshot
     from the CabiExport targetSymbols; `check` diffs the committed snapshot
     against a fresh deterministic regeneration (GONE = frozen but no longer
     regenerated, NEW = regenerated but never frozen; set + byte equality,
     exit 1 both directions). The freeze catches a consistently-edited
     ontology+header+impl triple that the live three-way parity above still
     accepts. scripts/verify_header_freeze.py is the standalone F4 court
     built on the same helpers.
Style follows scripts/verify_stardf_ontology.py but depends on nothing
outside this directory (no ~/purrdf, no git).
"""

import argparse
import os
import re
import sys

try:
    import rdflib
except ImportError:
    print("[FAIL] rdflib is required (pip install rdflib)", file=sys.stderr)
    sys.exit(1)

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ARROW_DIR = os.path.dirname(SCRIPT_DIR)

DEFAULT_TTL = os.path.join(ARROW_DIR, "ontology.ttl")
DEFAULT_HEADER = os.path.join(ARROW_DIR, "crates", "stardf-arrow-capi", "include", "stardf_arrow.h")
DEFAULT_CAPI_SRC = os.path.join(ARROW_DIR, "crates", "stardf-arrow-capi", "src")
DEFAULT_SNAPSHOT = os.path.join(ARROW_DIR, "snapshots", "prototypes.snapshot")

ONTOLOGY_NS = "https://ggen.io/ontology/stardf-arrow#"
TSYM = rdflib.URIRef(ONTOLOGY_NS + "targetSymbol")
CAB = rdflib.URIRef(ONTOLOGY_NS + "CabiExport")

RDFAPI_RE = re.compile(
    r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\([^;{}]*\)\s*(?:;|\s*\{)"
)

# Run state collected by main() and consumed by emit_receipt(); populated
# fresh on every run, never read before main sets it.
STATE = {
    "triples": 0,
    "census": {"classes": 0, "individuals": 0},
    "header": [],
    "impl": [],
    "parity_passed": False,
    "cross_passed": False,
    "ontology_symbols": [],
    "snapshot_mode": None,
    "snapshot_passed": False,
    "snapshot_symbols": [],
    "errors": [],
}


def strip_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.DOTALL)
    text = re.sub(r"//[^\n]*", "", text)
    return text


def header_declarations(header_path: str) -> set:
    """Function names declared in the C header (comments stripped)."""
    with open(header_path, "r", encoding="utf-8") as f:
        text = strip_comments(f.read())
    # declaration-ish: identifier followed by (...) then ';' or opening brace
    names = set(m.group(1) for m in RDFAPI_RE.finditer(text))
    # drop C keywords / macro noise / primitive types (the naive regex
    # captures the type keyword of `int (*fp)(...)`-style fn-pointer fields)
    return {
        n
        for n in names
        if n
        not in {
            "if", "for", "while", "switch", "return",
            "int", "void", "char", "float", "double", "long", "short",
            "unsigned", "signed", "const", "struct", "enum", "union",
            "size_t", "uintptr_t", "int32_t", "int64_t", "uint32_t",
            "uint64_t", "uint8_t", "int8_t",
        }
    }


def capi_exports(capi_src_dir: str) -> set:
    """Symbol names behind #[no_mangle] pub extern in the capi crate."""
    exports = set()
    for root, _dirs, files in os.walk(capi_src_dir):
        for fname in sorted(files):
            if not fname.endswith(".rs"):
                continue
            with open(os.path.join(root, fname), "r", encoding="utf-8") as f:
                text = f.read()
            # #[no_mangle] / #[unsafe(no_mangle)] pub extern "C" fn <name>
            # (edition 2024 requires the unsafe-attribute-wrapper form)
            for m in re.finditer(
                r"#\[unsafe\(?no_mangle\)?\]\s*(?:pub\s+)?(?:unsafe\s+)?extern\s+\"C\"\s+fn\s+(\w+)",
                text,
            ):
                exports.add(m.group(1))
    return exports


def ontology_target_symbols(ttl_path: str) -> set:
    """CabiExport targetSymbols via rdflib (single implementation shared by
    the three-way cross-check and the F4 freeze court). Raises on parse
    errors; an empty result is a loud failure at the call sites."""
    g = rdflib.Graph()
    g.parse(ttl_path, format="turtle")
    return set(
        str(o)
        for s, p, o in g.triples((None, TSYM, None))
        if (s, rdflib.RDF.type, CAB) in g
    )


SNAPSHOT_HEADER_LINES = [
    "# stardf-arrow F4 header-freeze snapshot — GENERATED; single source is ontology.ttl",
    "# (CabiExport targetSymbols). Hand edits FAIL the byte-freeze; regenerate instead:",
    "#   python3 scripts/verify.py --snapshot write",
    "# Court: python3 scripts/verify_header_freeze.py --court",
    "# Format: one symbol per line (sorted); '#' lines are comments; '# count:' pins arity.",
]


def render_snapshot(symbols: set) -> str:
    """Deterministic snapshot text (sorted, count-pinned, no timestamps)."""
    lines = list(SNAPSHOT_HEADER_LINES)
    lines.append(f"# count: {len(symbols)}")
    lines.extend(sorted(symbols))
    return "\n".join(lines) + "\n"


def parse_snapshot(path: str):
    """Return (symbols:set, pinned_count:int|None) from a snapshot file."""
    symbols = set()
    pinned = None
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            t = line.strip()
            if not t or t.startswith("#"):
                m = re.match(r"#\s*count:\s*(\d+)$", t)
                if m:
                    pinned = int(m.group(1))
                continue
            symbols.add(t)
    return symbols, pinned


def snapshot_diff(regenerated: set, snapshot_path: str) -> dict:
    """Silent core of the F4 freeze diff: committed snapshot vs fresh
    regeneration. GONE = frozen but no longer regenerated; NEW = regenerated
    but never frozen. Also pins the '# count:' arity line and byte equality
    (the file is GENERATED; hand edits are a loud failure, not a silent
    re-pin). Returns {ok, gone, new, errors:[(code,msg)]}; printing is the
    caller's job."""
    if not os.path.isfile(snapshot_path):
        return {
            "ok": False, "gone": [], "new": [],
            "errors": [(
                "SNAPSHOT_MISSING",
                f"snapshot missing: {snapshot_path} "
                "(regenerate: python3 scripts/verify.py --snapshot write)",
            )],
        }
    with open(snapshot_path, "rb") as f:
        raw = f.read()
    try:
        committed, pinned = parse_snapshot(snapshot_path)
    except Exception as e:
        return {"ok": False, "gone": [], "new": [],
                "errors": [("SNAPSHOT_UNREADABLE", str(e))]}
    errors = []
    gone = sorted(committed - regenerated)
    new = sorted(regenerated - committed)
    if gone:
        errors.append((
            "SNAPSHOT_GONE",
            "frozen in the snapshot but no longer regenerated from the "
            f"ontology: {gone}",
        ))
    if new:
        errors.append((
            "SNAPSHOT_NEW",
            "regenerated from the ontology but absent from the frozen "
            f"snapshot: {new}",
        ))
    if pinned is None:
        errors.append(("SNAPSHOT_COUNT_UNPINNED", "snapshot lacks a '# count:' pin"))
    elif pinned != len(regenerated):
        errors.append((
            "SNAPSHOT_COUNT_DRIFT",
            f"pinned '# count: {pinned}' != regenerated count {len(regenerated)}",
        ))
    if raw != render_snapshot(regenerated).encode("utf-8"):
        errors.append((
            "SNAPSHOT_BYTES_DRIFT",
            "committed snapshot is not byte-identical to the deterministic "
            "regeneration (GENERATED file: regenerate, do not hand-edit)",
        ))
    return {"ok": not errors, "gone": gone, "new": new, "errors": errors}


def print_snapshot_errors(where: str, errors: list) -> None:
    for code, msg in errors:
        print(f"[FAIL] {where} [{code}]: {msg}", file=sys.stderr)


def parity_check(header: set, impl: set, label: str) -> bool:
    """Bidirectional set equality; prints findings, returns True on pass."""
    ok = True
    missing = header - impl  # declared in header, absent from impl
    phantom = impl - header  # exported from impl, absent from header
    if missing:
        msg = f"declared in header but missing from impl: {sorted(missing)}"
        print(f"[FAIL] {label}: {msg}", file=sys.stderr)
        STATE["errors"].append(("HEADER_IMPL_MISSING", msg))
        ok = False
    if phantom:
        msg = f"exported from impl but not declared in header (phantom): {sorted(phantom)}"
        print(f"[FAIL] {label}: {msg}", file=sys.stderr)
        STATE["errors"].append(("HEADER_IMPL_PHANTOM", msg))
        ok = False
    if ok:
        print(f"[PASS] {label}: {len(header)} declarations == {len(impl)} exports, set equality both directions.")
    return ok


def main() -> None:
    parser = argparse.ArgumentParser(description="Verify stardf-arrow substrate")
    parser.add_argument("--ttl", default=DEFAULT_TTL, help="Path to ontology.ttl")
    parser.add_argument("--header", default=DEFAULT_HEADER, help="Path to stardf_arrow.h")
    parser.add_argument("--capi-src", default=DEFAULT_CAPI_SRC, help="stardf-arrow-capi src dir")
    parser.add_argument(
        "--mutate",
        choices=["drop-export"],
        default=None,
        help="Anti-vacuity: apply an in-memory mutation; the check must then FAIL.",
    )
    parser.add_argument(
        "--snapshot",
        choices=["check", "write"],
        default=None,
        help="F4 header freeze: 'write' (re)pins snapshots/prototypes.snapshot "
        "from ontology targetSymbols then runs the full ladder; 'check' adds "
        "the freeze diff (committed snapshot vs fresh regeneration, GONE/NEW, "
        "set+byte+count equality, exit 1 both directions).",
    )
    parser.add_argument(
        "--emit-receipt",
        metavar="PATH",
        default=None,
        help="Write a deterministic JSON receipt (sorted keys, no timestamps, "
        "no run-varying paths) to PATH; includes receipt_sha256 over the "
        "canonical content (schema family: evidence/RECEIPT_TEMPLATE.json).",
    )
    args = parser.parse_args()

    failures = 0

    # ---- Check 1: ontology parse + census -------------------------------
    print(f"=== Verifying stardf-arrow: {args.ttl} ===")
    if not os.path.isfile(args.ttl):
        print(f"[FAIL] ontology not found: {args.ttl}", file=sys.stderr)
        failures += 1
        census = {"classes": 0, "individuals": 0}
        g = None
    else:
        g = rdflib.Graph()
        try:
            g.parse(args.ttl, format="turtle")
            classes = set(g.subjects(rdflib.RDF.type, rdflib.OWL.Class))
            individuals = set(
                s
                for s in set(g.subjects(rdflib.RDF.type, None))
                if (s, rdflib.RDF.type, rdflib.OWL.Class) not in g
                and not isinstance(s, rdflib.BNode)
            )
            census = {"classes": len(classes), "individuals": len(individuals)}
            STATE["triples"] = len(g)
            STATE["census"] = census
            print(f"[PASS] ontology parse: {len(g)} triples; classes={census['classes']} individuals={census['individuals']}")
            # Extract CabiExport targetSymbols for the header cross-check
            # (and the F4 snapshot freeze); shared implementation.
            ontology_symbols = ontology_target_symbols(args.ttl)
            if not ontology_symbols:
                msg = "no CabiExport targetSymbols parsed from ontology (empty census is not a pass)"
                print(f"[FAIL] {msg}", file=sys.stderr)
                STATE["errors"].append(("ONTOLOGY_EMPTY_EXPORTS", msg))
                failures += 1
            STATE["ontology_symbols"] = sorted(ontology_symbols)
        except Exception as e:
            print(f"[FAIL] RDF parse error: {e}", file=sys.stderr)
            failures += 1
            g = None

    # ---- Check 2: header/impl parity ------------------------------------
    if not os.path.isfile(args.header):
        msg = f"header not found: {args.header}"
        print(f"[FAIL] {msg}", file=sys.stderr)
        STATE["errors"].append(("HEADER_NOT_FOUND", msg))
        failures += 1
        header = set()
    else:
        header = header_declarations(args.header)
        if not header:
            msg = f"no declarations parsed from {args.header} (empty census is not a pass)"
            print(f"[FAIL] {msg}", file=sys.stderr)
            STATE["errors"].append(("HEADER_EMPTY_CENSUS", msg))
            failures += 1
        print(f"[INFO] {len(header)} declarations in {args.header}")

    if not os.path.isdir(args.capi_src):
        msg = f"capi src dir not found: {args.capi_src}"
        print(f"[FAIL] {msg}", file=sys.stderr)
        STATE["errors"].append(("CAPI_SRC_NOT_FOUND", msg))
        failures += 1
        impl = set()
    else:
        impl = capi_exports(args.capi_src)
        if not impl:
            msg = f"no #[no_mangle] exports found under {args.capi_src} (empty census is not a pass)"
            print(f"[FAIL] {msg}", file=sys.stderr)
            STATE["errors"].append(("CAPI_EMPTY_EXPORTS", msg))
            failures += 1
        print(f"[INFO] {len(impl)} #[no_mangle] exports under {args.capi_src}")

    mutated_impl = impl
    mutated_ontology = set(STATE["ontology_symbols"])
    if args.mutate == "drop-export":
        # One mutation, both directions: drop one impl export AND the matching
        # ontology individual (so the anti-vacuity covers both the parity check
        # and the ontology<->header cross-check). If the sets diverge (that is
        # itself a failure the cross-check catches), fall back to whichever
        # symbol exists so the mutation still flips.
        pool = sorted(impl) or sorted(mutated_ontology)
        if pool:
            drop = pool[0]
            mutated_impl = impl - {drop}
            mutated_ontology.discard(drop)
            print(f"[MUTATE] drop-export: removing impl export '{drop}' AND matching ontology individual (expect FAIL)")

    if not parity_check(header, mutated_impl, "header/impl parity"):
        failures += 1
    else:
        STATE["parity_passed"] = True
    STATE["header"] = sorted(header)
    STATE["impl"] = sorted(impl)

    # ---- Check 3: ontology<->header cross-check -------------------------
    if STATE["ontology_symbols"]:
        # Full three-way set equality (bidirectional): every ontology
        # targetSymbol must appear in BOTH header and impl, AND every header
        # decl and impl export must be claimed by an ontology individual —
        # so dropping an ontology export individual flips exit 1 instead of
        # merely decrementing the census.
        cross_ok = True
        onto_header = mutated_ontology - header
        onto_impl = mutated_ontology - mutated_impl
        header_onto = header - mutated_ontology
        impl_onto = mutated_impl - mutated_ontology
        if onto_header:
            msg = f"ontology targetSymbols absent from header decls: {sorted(onto_header)}"
            print(f"[FAIL] ontology/header cross-check: {msg}", file=sys.stderr)
            STATE["errors"].append(("ONTOLOGY_HEADER_MISSING", msg))
            cross_ok = False
        if onto_impl:
            msg = f"ontology targetSymbols absent from impl exports: {sorted(onto_impl)}"
            print(f"[FAIL] ontology/header cross-check: {msg}", file=sys.stderr)
            STATE["errors"].append(("ONTOLOGY_IMPL_MISSING", msg))
            cross_ok = False
        if header_onto:
            msg = f"header decls with no ontology targetSymbol (unclaimed): {sorted(header_onto)}"
            print(f"[FAIL] ontology/header cross-check: {msg}", file=sys.stderr)
            STATE["errors"].append(("HEADER_ONTOLOGY_UNCLAIMED", msg))
            cross_ok = False
        if impl_onto:
            msg = f"impl exports with no ontology targetSymbol (unclaimed): {sorted(impl_onto)}"
            print(f"[FAIL] ontology/header cross-check: {msg}", file=sys.stderr)
            STATE["errors"].append(("IMPL_ONTOLOGY_UNCLAIMED", msg))
            cross_ok = False
        if cross_ok:
            print(f"[PASS] ontology/header cross-check: {len(mutated_ontology)} ontology targetSymbols == {len(header)} header decls == {len(mutated_impl)} impl exports (three-way equality).")
        else:
            failures += 1
        STATE["cross_passed"] = cross_ok

    # ---- Check 4: F4 snapshot freeze (--snapshot check only) ------------
    # A consistently-edited ontology+header+impl triple still passes the live
    # three-way parity above; the committed freeze snapshot is what flips.
    STATE["snapshot_mode"] = args.snapshot
    if args.snapshot == "write":
        os.makedirs(os.path.dirname(DEFAULT_SNAPSHOT), exist_ok=True)
        with open(DEFAULT_SNAPSHOT, "w", encoding="utf-8") as f:
            f.write(render_snapshot(set(STATE["ontology_symbols"])))
        print(
            f"[PASS] snapshot write: pinned {len(STATE['ontology_symbols'])} "
            f"symbols -> {os.path.relpath(DEFAULT_SNAPSHOT, ARROW_DIR)} "
            "(snapshot check itself runs with --snapshot check)"
        )
    elif args.snapshot == "check":
        if not STATE["ontology_symbols"]:
            msg = "ontology targetSymbols empty/unparsed; snapshot freeze not checkable (loud-missing, not a silent skip)"
            print(f"[FAIL] snapshot-freeze: {msg}", file=sys.stderr)
            STATE["errors"].append(("SNAPSHOT_SKIPPED_NO_ONTOLOGY", msg))
            failures += 1
        else:
            snap = snapshot_diff(set(STATE["ontology_symbols"]), DEFAULT_SNAPSHOT)
            STATE["snapshot_passed"] = snap["ok"]
            STATE["snapshot_symbols"] = sorted(set(STATE["ontology_symbols"]))
            print_snapshot_errors("snapshot-freeze", snap["errors"])
            if snap["ok"]:
                print(
                    f"[PASS] snapshot-freeze: committed snapshot == deterministic "
                    f"regeneration ({len(STATE['snapshot_symbols'])} symbols, "
                    "set+byte+count equality)."
                )
            else:
                failures += 1

    # ---- Check 5: anti-vacuity ------------------------------------------
    if args.mutate == "drop-export":
        if failures == 0:
            print("[FAIL] VACUOUS GATE: drop-export mutation passed; parity+cross checks detect nothing.", file=sys.stderr)
            failures += 1
        else:
            print("[PASS] anti-vacuity: drop-export mutation correctly FAILed the checks (both directions covered).")

    if args.emit_receipt:
        emit_receipt(args, failures, bool(args.mutate))

    if failures:
        print(f"\n[ALL FAIL] {failures} check(s) failed.", file=sys.stderr)
        sys.exit(1)
    print("\n[ALL PASS] stardf-arrow substrate verified.")


def emit_receipt(args, failures: int, mutated: bool) -> None:
    """Deterministic JSON receipt: sorted keys, no timestamps, no
    run-varying paths; receipt_sha256 covers the canonical content with
    the digest field itself removed (fixed point of self-hashing)."""
    import hashlib
    import json

    census = STATE["census"]
    receipt = {
        "schema": "ggen.legacy.stardf.arrow.receipt.v1",
        "subject": {
            "ontology_census": {"classes": census["classes"], "individuals": census["individuals"]},
            "header_declarations": len(STATE["header"]),
            "capi_exports": len(STATE["impl"]),
            "ontology_export_symbols": len(STATE["ontology_symbols"]),
        },
        "checks": [
            {
                "id": "ontology-parse",
                "passed": STATE["triples"] > 0,
                "count": STATE["triples"],
                "comment": "rdflib parse of ontology.ttl; count = triples",
            },
            {
                "id": "header/impl-parity",
                "passed": STATE["parity_passed"],
                "count": len(STATE["header"]),
                "comment": "bidirectional set equality header decls vs #[no_mangle] exports",
            },
            {
                "id": "ontology-header-cross-check",
                "passed": STATE["cross_passed"],
                "count": len(STATE["ontology_symbols"]),
                "comment": "every CabiExport targetSymbol present in header decls AND impl exports",
            },
        ] + (
            [
                {
                    "id": "snapshot-freeze",
                    "passed": STATE["snapshot_passed"],
                    "count": len(STATE["snapshot_symbols"]),
                    "comment": "F4: committed prototypes.snapshot set+byte+count "
                    "identical to regeneration from ontology targetSymbols "
                    "(--snapshot check; row present only when the mode ran)",
                }
            ]
            if args.snapshot == "check"
            else []
        ),
        "errors": [list(e) for e in STATE["errors"]],
        "commands": [
            {
                "cmd": "python3 scripts/verify.py --emit-receipt <path>",
                "exit": 1 if failures else 0,
                "comment": "this verify.py run (mutated=%s)" % mutated,
            }
        ],
        "replay": {
            "status": "UNKNOWN",
            "comment": "replay pair = two verify.py --emit-receipt runs byte-identical; "
            "witnessed externally (docs/RECEIPTS.md), never asserted by this script",
        },
        "standing": "UNKNOWN",
        "nonclaims": [
            "external production standing",
            "CI-gate standing (local run only)",
        ],
    }
    canonical = json.dumps(
        {k: v for k, v in receipt.items() if k != "receipt_sha256"},
        sort_keys=True,
        indent=2,
    )
    digest = hashlib.sha256(canonical.encode("utf-8")).hexdigest()
    receipt["receipt_sha256"] = digest
    out = json.dumps(receipt, sort_keys=True, indent=2)
    with open(args.emit_receipt, "w", encoding="utf-8") as f:
        f.write(out + "\n")


if __name__ == "__main__":
    main()
