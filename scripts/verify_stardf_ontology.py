#!/usr/bin/env python3
"""
scripts/verify_stardf_ontology.py
Mechanical verifier for stardf_contract.ttl:
  1. RDF syntax & graph closure validation using rdflib
  2. SHACL shape conformance validation using pyshacl
  3. 100% census match against purrdf.h
"""

import sys
import os
import re
import argparse
import rdflib
import pyshacl

PURRDF_HEADER = "/Users/sac/purrdf/crates/rdf-capi/include/purrdf.h"
DEFAULT_TTL = "authority/stardf/stardf_contract.ttl"

def parse_header_types(header_path):
    """Type census source: opaque typedefs (`typedef struct X X;`) and named
    anonymous struct typedefs (`typedef struct { ... } Name;`) from purrdf.h."""
    with open(header_path, "r", encoding="utf-8") as f:
        text = f.read()
    text = re.sub(r'/\*.*?\*/', '', text, flags=re.DOTALL)
    text = re.sub(r'//.*', '', text)
    opaque = set(re.findall(r'typedef\s+struct\s+([A-Za-z0-9_]+)\s+\1\s*;', text))
    anon = set(re.findall(r'\}\s*([A-Za-z0-9_]+)\s*;', text))
    return opaque, anon

def census_types(g, header_path):
    """Two-directional type census. Returns True on full match."""
    opaque, anon = parse_header_types(header_path)
    expected = opaque | anon
    print(f"[INFO] Type census: {len(opaque)} opaque + {len(anon)} anonymous-struct "
          f"typedefs in header ({len(expected)} total).")

    q = """
    PREFIX stardf: <https://ggen.io/ontology/stardf#>
    SELECT ?pred WHERE {
        ?t a stardf:TypeDefinition ; stardf:predecessorSymbol ?pred .
    }"""
    graph_preds = set(str(row.pred) for row in g.query(q))

    missing = expected - graph_preds
    unknown = graph_preds - expected
    ok = True
    if missing:
        print(f"[FAIL] Missing type mappings: {sorted(missing)}", file=sys.stderr)
        ok = False
    if unknown:
        print(f"[FAIL] Type mappings with no header counterpart: {sorted(unknown)}", file=sys.stderr)
        ok = False
    if ok:
        print(f"[PASS] Type census: all {len(expected)} header types mapped, 0 stale mappings.")
    return ok

def run_mutation_drop_type(g):
    """F1a positive control: drop one stardf:TypeDefinition in memory. Returns
    (mutated_graph, dropped_symbol)."""
    q = """
    PREFIX stardf: <https://ggen.io/ontology/stardf#>
    SELECT ?t ?pred WHERE {
        ?t a stardf:TypeDefinition ; stardf:predecessorSymbol ?pred .
    } LIMIT 1"""
    rows = list(g.query(q))
    if not rows:
        return None, None
    victim = rows[0].t
    pred = str(rows[0].pred)
    g2 = rdflib.Graph()
    for t in g:
        if t[0] == victim:
            continue
        g2.add(t)
    return g2, pred

def main():
    parser = argparse.ArgumentParser(description="Verify stardf ontology")
    parser.add_argument("--ttl", default=DEFAULT_TTL, help="Path to stardf_contract.ttl")
    parser.add_argument("--header", default=PURRDF_HEADER, help="Path to purrdf.h")
    parser.add_argument("--mutate", choices=["drop-type"],
                        help="F1a anti-vacuity positive control")
    args = parser.parse_args()

    print(f"=== Verifying stardf ontology: {args.ttl} ===")
    
    # 1. Parse RDF graph
    g = rdflib.Graph()
    try:
        g.parse(args.ttl, format="turtle")
        print(f"[PASS] RDF syntax valid. Graph contains {len(g)} triples.")
    except Exception as e:
        print(f"[FAIL] RDF parse error: {e}", file=sys.stderr)
        sys.exit(1)

    # 2. Query function count
    query_fns = """
    PREFIX stardf: <https://ggen.io/ontology/stardf#>
    SELECT ?fn ?target ?predecessor ?ret ?subsystem
    WHERE {
        ?fn a stardf:AbiFunction ;
            stardf:targetSymbol ?target ;
            stardf:predecessorSymbol ?predecessor ;
            stardf:returnType ?ret ;
            stardf:subsystem ?subsystem .
    }
    ORDER BY ?target
    """
    results = list(g.query(query_fns))
    print(f"[INFO] Found {len(results)} stardf:AbiFunction instances in graph.")

    # 2b. Type census (F1) + optional F1a mutation
    if args.mutate == "drop-type":
        g, dropped = run_mutation_drop_type(g)
        if g is None:
            print("[SKIP] --mutate drop-type: no TypeDefinition individuals to drop. "
                  "A skip is loud.", file=sys.stderr)
            sys.exit(1)
        print(f"[MUTATE] dropped TypeDefinition with predecessorSymbol '{dropped}'; "
              f"graph must now be REFUSED.")

    if not census_types(g, args.header):
        if args.mutate == "drop-type":
            print("[PASS] F1a: drop-type mutation REFUSED (mutant graph fails census).")
            sys.exit(0)
        sys.exit(1)
    elif args.mutate == "drop-type":
        print("[FAIL] VACUOUS GATE: drop-type mutant still passes the census.", file=sys.stderr)
        sys.exit(1)

    # 3. Verify against purrdf.h
    import re
    with open(args.header, "r", encoding="utf-8") as f:
        text = f.read()
    code_no_comments = re.sub(r'/\*.*?\*/', '', text, flags=re.DOTALL)
    code_no_comments = re.sub(r'//.*', '', code_no_comments)
    fn_pattern = re.compile(r'(?:PURRDF_EXPORT\s+)?((?:const\s+)?[A-Za-z0-9_]+[\s\*]+)\s*(purrdf_[a-zA-Z0-9_]+)\s*\(([^)]*)\)\s*;', re.MULTILINE)
    raw_functions = fn_pattern.findall(code_no_comments)
    purrdf_names = set(m[1] for m in raw_functions)
    print(f"[INFO] Found {len(purrdf_names)} exported functions in {args.header}.")

    # Non-vacuity floor: a 0-vs-0 census is not a match. An ontology with no
    # AbiFunction individuals (or a header with no purrdf_ exports) refuses.
    if len(results) == 0:
        print("[FAIL] Vacuous census: ontology declares 0 stardf:AbiFunction individuals.", file=sys.stderr)
        sys.exit(1)
    if len(purrdf_names) == 0:
        print(f"[FAIL] Vacuous census: 0 purrdf_ exports found in {args.header}.", file=sys.stderr)
        sys.exit(1)
    graph_predecessors = set(str(row.predecessor) for row in results if str(row.predecessor) != "none")
    missing = purrdf_names - graph_predecessors
    if missing:
        print(f"[FAIL] Missing purrdf functions in ontology: {missing}", file=sys.stderr)
        sys.exit(1)
    else:
        print(f"[PASS] 100% census match: All {len(purrdf_names)} purrdf.h functions are mapped.")

    # 4. SHACL Validation
    print("[INFO] Running SHACL shape validation via pyshacl...")
    conforms, v_graph, v_text = pyshacl.validate(
        data_graph=g,
        shacl_graph=g,
        inference="rdfs",
        abort_on_first=False,
        meta_shacl=False,
        advanced=True,
        debug=False
    )
    if not conforms:
        print(f"[FAIL] SHACL validation failed:\n{v_text}", file=sys.stderr)
        sys.exit(1)
    print("[PASS] SHACL shape validation conforms: 0 violations.")

    print("\n[ALL PASS] stardf_contract.ttl is fully verified and compliant.")

if __name__ == "__main__":
    main()
