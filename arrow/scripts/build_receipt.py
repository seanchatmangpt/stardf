#!/usr/bin/env python3
"""Build the stardf/arrow campaign manufacturing receipt (finisher wave).

Aggregates on-disk evidence into one receipt per the house schema
(`ggen.legacy.stardf.arrow.receipt.v1`, evidence/RECEIPT_TEMPLATE.json):

  * subject.tree_digest computed AT RECEIPT TIME via the documented house
    pipeline (prefix-relative, target dirs pruned) -- never before;
  * head/branch identity CARRIED from evidence/v26.11.1-manufacturing-
    receipt.json -- this script executes NO git commands (dispatch law,
    same posture as the H05 pass-4 receipt re-bind);
  * ladder aggregation: the newest evidence/publish_stardf-receipt-*.json
    is parsed and its gate rows + verdict are carried in honestly
    (BLOCKED rows stay BLOCKED -- never rounded up);
  * license register counts re-read from
    authority/stardf/license-compat.json (workspace-root scope, outside
    this digest scope);

Default mode is DRY: the receipt goes to stdout and nothing is written.
Pass --write to land it in evidence/ (receipt files are an admitted
transition; the coordinator owns git state).

NONCLAIMS:
  * This tool aggregates and computes; it does not run gates (no cargo,
    no R1 re-run). A green aggregation is not a green campaign.
  * replay.status is NOT_RUN by construction -- the verify.py byte-
    identical pair is witnessed separately, never claimed here.
  * Aggregated ladder standings are as-of the receipt they were parsed
    from; the tree moves between receipts, so they are stale by default.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ARROW_ROOT = Path(__file__).resolve().parents[1]
EVIDENCE_DIR = ARROW_ROOT / "evidence"
REGISTER_PATH = ARROW_ROOT.parent.parent / "authority" / "stardf" / "license-compat.json"
CARRY_RECEIPT = EVIDENCE_DIR / "v26.11.1-manufacturing-receipt.json"
LADDER_GLOB = "publish_stardf-receipt-*.json"
SCHEMA = "ggen.legacy.stardf.arrow.receipt.v1"

STANDING_UNKNOWN = "UNKNOWN"
STANDING_PARTIAL = "PARTIAL_ALIVE"


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def tree_digest(root: Path) -> dict:
    """House method, receipt-time: prefix-relative paths, target pruned.

    Runs the documented pipeline VERBATIM via /bin/sh (real collaborators;
    no Python re-implementation to drift -- a re-implementation was tried
    and FAILED the parity court: locale collation of `sort` differs from
    codepoint order). Output line: "<digest>  -".
    """
    pipeline = ("find . -path '*/target*' -prune -o -type f -print | sort | "
                "xargs shasum -a 256 | shasum -a 256")
    p = subprocess.run(["/bin/sh", "-c", pipeline],
                       cwd=root, capture_output=True, text=True)
    if p.returncode != 0:
        raise RuntimeError(f"digest pipeline failed: {p.stderr}")
    line = p.stdout.strip()
    if not line or "  -" not in line:
        raise RuntimeError(f"unexpected digest output: {line!r}")
    digest = line.split("  ")[0]
    n_out = subprocess.run(
        ["/usr/bin/find", ".", "-path", "*/target*", "-prune", "-o", "-type", "f", "-print"],
        cwd=root, capture_output=True, text=True)
    if n_out.returncode != 0:
        raise RuntimeError(f"find failed: {n_out.stderr}")
    files = n_out.stdout.count("\n")
    if files <= 0:
        raise RuntimeError("no files found under tree")
    return {"value": digest, "files": files,
            "computed": now_iso(),
            "method": pipeline + " (house method, executed verbatim)"}


def carry_identity(receipt: dict) -> dict:
    """Carry head/branch from the campaign receipt; NO git commands."""
    out = {"head": "UNKNOWN", "branch": "UNKNOWN",
           "git_state": "NO_GIT_COMMANDS_EXECUTED (dispatch law); "
                        "identity carried, not probed"}
    try:
        carried = json.loads(CARRY_RECEIPT.read_text())
        chain = carried.get("subject", {}).get("c_chain", {})
        out["head"] = chain.get("head", carried.get("subject", {}).get("head", "UNKNOWN"))
        out["branch"] = chain.get("branch", carried.get("branch", "UNKNOWN"))
        out["carried_from"] = str(CARRY_RECEIPT.relative_to(ARROW_ROOT))
    except (OSError, json.JSONDecodeError, KeyError) as exc:
        out["carry_error"] = str(exc)
    receipt["subject"].update(out)
    return receipt


def newest_ladder_receipt() -> tuple[Path | None, dict | None, str | None]:
    candidates = sorted(EVIDENCE_DIR.glob(LADDER_GLOB))
    if not candidates:
        return None, None, "no ladder receipts found in evidence/"
    path = candidates[-1]
    try:
        return path, json.loads(path.read_text()), None
    except (OSError, json.JSONDecodeError) as exc:
        return path, None, f"newest ladder receipt unparsable: {exc}"


def register_counts() -> dict:
    out = {"path": "authority/stardf/license-compat.json (workspace-root scope)",
           "dependency_licenses": None, "compatibility_receipts": None,
           "drafts_pending_operator": None, "read_error": None}
    try:
        reg = json.loads(REGISTER_PATH.read_text())
        out["dependency_licenses"] = len(reg.get("dependency_licenses", []))
        out["compatibility_receipts"] = len(reg.get("compatibility_receipts", []))
        out["drafts_pending_operator"] = len(reg.get("draft_receipts_pending_operator", []))
    except (OSError, json.JSONDecodeError) as exc:
        out["read_error"] = str(exc)
    return out


def aggregate(receipt: dict) -> dict:
    ladder_path, ladder, ladder_err = newest_ladder_receipt()
    ladder_summary: dict = {"receipt": None, "verdict": None, "gates": [], "note": None}
    checks: list[dict] = []
    errors: list[dict] = []

    checks.append({
        "id": "tree-digest-at-receipt-time",
        "passed": True,
        "count": 0,
        "comment": "digest computed by this run, not carried (see subject.tree_digest)",
    })
    if ladder_path is not None:
        ladder_summary["receipt"] = ladder_path.name
        if ladder is not None:
            ladder_summary["verdict"] = ladder.get("verdict")
            ladder_summary["gates"] = [
                {"id": g.get("id"), "standing": g.get("standing")}
                for g in ladder.get("gates", [])
            ]
            red = [g for g in ladder_summary["gates"] if g.get("standing") == "FAIL"]
            checks.append({
                "id": "newest-ladder-receipt-aggregated",
                "passed": True,
                "count": len(ladder_summary["gates"]),
                "count_basis": f"{len(ladder_summary['gates'])} gate rows parsed from {ladder_path.name}",
                "comment": f"newest ladder verdict: {ladder_summary['verdict']}; "
                           f"{len(red)} FAIL rows carried honestly (BLOCKED stays BLOCKED)",
            })
            if red:
                errors.append({
                    "code": "LADDER_BLOCKED_ROWS",
                    "message": f"{ladder_path.name}: {len(red)} FAIL gate rows: "
                               + ", ".join(g["id"] for g in red),
                })
        else:
            checks.append({
                "id": "newest-ladder-receipt-aggregated",
                "passed": False,
                "count": 0,
                "comment": ladder_err or "unparsable",
            })
            errors.append({"code": "LADDER_RECEIPT_UNPARSABLE",
                           "message": ladder_err or "unknown"})
    else:
        checks.append({
            "id": "newest-ladder-receipt-aggregated",
            "passed": False,
            "count": 0,
            "comment": ladder_err or "no ladder receipts",
        })

    reg = register_counts()
    if reg["read_error"] is None:
        closed = reg["drafts_pending_operator"] == 0
        checks.append({
            "id": "license-register-re-read",
            "passed": True,
            "count": reg["compatibility_receipts"],
            "count_basis": f"{reg['dependency_licenses']} dependency_licenses / "
                           f"{reg['compatibility_receipts']} compatibility_receipts / "
                           f"{reg['drafts_pending_operator']} drafts pending operator",
            "comment": ("register CLOSED (0 drafts)" if closed
                        else "register has drafts pending operator"),
        })
    else:
        checks.append({
            "id": "license-register-re-read",
            "passed": False,
            "count": 0,
            "comment": f"register read failed: {reg['read_error']}",
        })
        errors.append({"code": "REGISTER_READ_FAILED", "message": reg["read_error"]})

    digest = receipt["subject"]["tree_digest"]
    digest_stable = digest.get("value") != ""
    checks.append({
        "id": "receipt-schema-v1-complete",
        "passed": bool(digest_stable and receipt["subject"].get("head") != "UNKNOWN"),
        "count": 1,
        "comment": "schema fields present: digest computed + identity carried "
                   "(identity UNKNOWN -> not carried -> fails honestly)",
    })

    receipt["ladder_aggregate"] = ladder_summary
    receipt["license_register"] = reg
    receipt["checks"] = checks
    receipt["errors"] = errors
    return receipt


NONCLAIMS = [
    "aggregation is not re-execution: no cargo, no R1 re-run, no court suites, no registry touch",
    "a green aggregation is not a green campaign; aggregated ladder rows are as-of their own receipt, stale by default",
    "replay.status is NOT_RUN by construction; the verify.py byte-identical pair is witnessed separately",
    "this script executes no git commands; head/branch are carried, not probed",
    "standing vocabulary per evidence/README.md law 5; never an unqualified works",
]


def build() -> dict:
    receipt = {
        "schema": SCHEMA,
        "kind": "campaign-receipt-build",
        "generated_at": now_iso(),
        "identity": {"tool": "scripts/build_receipt.py", "mode": "build"},
        "subject": {
            "tree_digest": {},
        },
        "commands": [],
        "replay": {"status": "NOT_RUN"},
        "nonclaims": NONCLAIMS,
        "standing": STANDING_UNKNOWN,
    }
    digest = tree_digest(ARROW_ROOT)
    receipt["subject"]["tree_digest"] = digest
    receipt["commands"].append({
        "cmd": "build_receipt.py tree_digest (house pipeline, in-process over find output)",
        "exit": 0,
        "comment": f"{digest['files']} files -> {digest['value']}",
    })
    receipt = carry_identity(receipt)
    receipt = aggregate(receipt)

    red_rows = len(receipt["errors"])
    witnessed = all(c["passed"] for c in receipt["checks"])
    if red_rows == 0 and witnessed:
        receipt["standing"] = "ALIVE"
    elif witnessed or red_rows > 0:
        receipt["standing"] = STANDING_PARTIAL
    # else stays UNKNOWN

    return receipt


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Build the stardf/arrow campaign manufacturing receipt "
                    "(house schema v1; dry by default)")
    ap.add_argument("--write", action="store_true",
                    help="land the receipt in evidence/ (default: stdout only)")
    args = ap.parse_args()

    try:
        receipt = build()
    except RuntimeError as exc:
        print(f"FATAL: {exc}", file=sys.stderr)
        return 2

    payload = json.dumps(receipt, indent=2) + "\n"
    if not args.write:
        print(payload)
        print(f"== DRY: receipt not written (pass --write to land it) ==",
              file=sys.stderr)
        return 0

    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out = EVIDENCE_DIR / f"campaign-receipt-build-{stamp}.json"
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    out.write_text(payload)
    print(f"receipt: {out}")
    print(f"standing: {receipt['standing']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
