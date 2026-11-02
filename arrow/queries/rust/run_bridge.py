#!/usr/bin/env python3
"""
queries/rust/run_bridge.py — Lane 10 (v26.10.31 stardf finishing wave)

Applies queries/rust/010_contract_to_rust_items.rq (CONSTRUCT) in memory
over stardf/arrow/ontology.ttl with rdflib, then runs the 5 rust-pack gates
(ggen-marketplace/packs/rust-pack/gates/*.rq) against the bridged graph.

Receipt semantics (GateVerify law: gates>=1 row, unbound=0 rows):
  - coverage rows per gate  >= 1 on the pristine bridge
  - violation rows per gate = 0 on the pristine bridge (bridge is gate-clean)
  - --drop-mapping buffer-slot: bridge without the BufferSlot mapping ->
    gate 030 coverage = 0 rows, LOUD (exit 1). A court that cannot flip is FAIL.

No git, no network, no writes outside queries/rust/ (this file, the .rq).
Depends only on rdflib.
"""

import argparse
import os
import sys

try:
    import rdflib
except ImportError:
    print("[FAIL] rdflib is required (pip install rdflib)", file=sys.stderr)
    sys.exit(1)

QUERY_DIR = os.path.dirname(os.path.abspath(__file__))
ARROW_DIR = os.path.dirname(os.path.dirname(QUERY_DIR))  # stardf/arrow/
GATES_DIR = os.path.expanduser(
    "~/ggen-marketplace/packs/rust-pack/gates"
)

DEFAULT_TTL = os.path.join(ARROW_DIR, "ontology.ttl")
BRIDGE_RQ = os.path.join(QUERY_DIR, "010_contract_to_rust_items.rq")

RUST = rdflib.Namespace("urn:ggen:rust:#")

# Antecedent-census (coverage) query per gate: the population each gate
# guards, i.e. the gate's triple patterns minus its violation filter.
CENSUS = {
    "010_api_naming.rq": """
        PREFIX rust: <urn:ggen:rust:#>
        SELECT DISTINCT ?item WHERE {
          ?item a ?c .
          FILTER(?c IN (rust:Struct, rust:Enum, rust:Function, rust:Module))
        }
    """,
    "020_unsafe_safety_proof.rq": """
        PREFIX rust: <urn:ggen:rust:#>
        SELECT ?item WHERE { ?item rust:isUnsafe true . }
    """,
    "030_alignment_invariant.rq": """
        PREFIX rust: <urn:ggen:rust:#>
        SELECT ?slot WHERE {
          ?slot a rust:BufferSlot ; rust:simdProfile ?p .
          FILTER(?p != "None")
        }
    """,
    "040_ffi_unwind_guard.rq": """
        PREFIX rust: <urn:ggen:rust:#>
        SELECT ?export WHERE { ?export a rust:FfiExport . }
    """,
    "050_workspace_inheritance.rq": """
        PREFIX rust: <urn:ggen:rust:#>
        SELECT ?crate ?dep WHERE {
          ?crate rust:dependsOn ?dep .
          ?dep a rust:Dependency ;
               rust:versionLiteral ?version .
          ?workspace a rust:Workspace ;
                     rust:workspaceDependency ?dep .
        }
    """,
}


def coverage_census(gate: str):
    """Variant of the 050 census that matches the pristine bridge: the
    workspace-inheritance population is a dependsOn edge to a workspace
    member WITHOUT a versionLiteral (the gate fires only when one is
    present). Return the census query actually used."""
    if gate == "050_workspace_inheritance.rq":
        return """
            PREFIX rust: <urn:ggen:rust:#>
            SELECT ?crate ?dep WHERE {
              ?crate rust:dependsOn ?doc .
              ?dep a rust:Dependency .
              FILTER NOT EXISTS { ?dep rust:versionLiteral ?v }
              ?workspace a rust:Workspace ;
                         rust:workspaceDependency ?dep .
            }
        """
    return CENSUS[gate]


def run_gate(g, gate):
    path = os.path.join(GATES_DIR, gate)
    q = open(path).read()
    rows = list(g.query(q))
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--drop-mapping", choices=["buffer-slot"],
                    help="negative control: bridge without the BufferSlot "
                         "mapping; gate 030 coverage must flip to 0 rows "
                         "loudly (exit 1)")
    args = ap.parse_args()

    g = rdflib.Graph()
    g.parse(DEFAULT_TTL, format="turtle")
    base = len(g)

    bridge = open(BRIDGE_RQ).read()
    bridged = rdflib.Graph()
    for t in g.query(bridge):
        bridged.add(t)

    if args.drop_mapping:
        # Drop the BufferSlot/simdProfile/alignment mapping (in memory).
        for t in list(bridged.triples((rdflib.URIRef(
                "https://ggen.io/ontology/stardf-arrow#arrow-array-stream"),
                None, None))):
            if (t[1] == rdflib.RDF.type and t[2] == RUST.BufferSlot) or \
               t[1] in (RUST.simdProfile, RUST.byteAlignment):
                bridged.remove(t)
        print(f"[NEGATIVE] BufferSlot mapping dropped: "
              f"{base} base + N bridge triples -> {len(bridged)} bridged")

    print(f"base ontology triples: {base}; bridged triples: {len(bridged)}")

    gates = sorted(os.listdir(GATES_DIR))
    gates = [x for x in gates if x.endswith(".rq")]
    ok = True
    print(f"\n{'gate':38} {'coverage':>8} {'violations':>10}")
    print("-" * 60)
    for gate in gates:
        cov_q = coverage_census(gate)
        cov = len(list(bridged.query(cov_q)))
        viol = len(run_gate(bridged, gate))
        flag = ""
        expected_cov_zero = args.drop_mapping and gate == "030_alignment_invariant.rq"
        if args.drop_mapping:
            if expected_cov_zero and cov != 0:
                print(f"[FAIL] {gate}: expected coverage 0 after mapping drop, "
                      f"got {cov} — COURT DOES NOT FLIP (vacuous)", file=sys.stderr)
                ok = False
                flag = "  <-- VACUOUS"
            elif not expected_cov_zero and cov == 0:
                print(f"[FAIL] {gate}: collateral coverage loss to 0 "
                      f"(over-broad drop)", file=sys.stderr)
                ok = False
        else:
            if cov < 1:
                print(f"[FAIL] {gate}: coverage 0 — gate pattern binds nothing "
                      f"(unbound, loud)", file=sys.stderr)
                ok = False
                flag = "  <-- UNBOUND"
            if viol != 0:
                print(f"[FAIL] {gate}: {viol} violation row(s) on pristine bridge",
                      file=sys.stderr)
                ok = False
                flag = "  <-- REFUSAL"
        print(f"{gate:38} {cov:>8} {viol:>10}{flag}")

    print("-" * 60)
    if args.drop_mapping:
        if ok:
            print("[PASS] negative control: gate 030 flipped to 0 coverage rows "
                  "loudly; a court that cannot flip is FAIL — this one flips.")
            return 0
        print("[FAIL] negative control failed to flip (vacuous court)")
        return 1

    if ok:
        print("[PASS] all 5 gates: coverage >= 1 row, violations = 0 "
              "(bridge is gate-clean)")
        return 0
    print("[FAIL] gate court failed — see rows above", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
