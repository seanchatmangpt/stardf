# GL-STARDF-001 — Author Exhaustive stardf Ontology Matching purrdf

## Subject

- repository: `seanchatmangpt/ggen-legacy`
- target namespace: `stardf`
- predecessor: `purrdf`
- authority contract: `authority/stardf/stardf_contract.ttl`
- downstream substrates: `seanchatmangpt/avec`, `seanchatmangpt/bcinr`
- status: `PARTIAL_ALIVE`

## Mission

Model every function, data type, parameter signature, and operational capability from `~/purrdf` into the canonical `stardf_contract.ttl` RDF ontology so that reconstitution proceeds from an admitted $\mathcal{O}^*$ authority without starting from scratch.

## Invariant Bounds

1. Zero-allocation memory model (`stardf:ZeroAllocationArena`).
2. Constant-time branchless execution model (`stardf:BranchlessConstantTime`) adhering to SHACL shape `stardf:ExecutionInvariantsShape`.
3. 64-byte aligned arena memory layout (`stardf:TermIdArenaDefinition`) binding directly to `avec` `ArrowArrayStream`.
4. 100% census parity against legacy exported C ABI functions in `purrdf/crates/rdf-capi/include/purrdf.h`.
5. Explicit classification of downstream substrate bindings (`avec`, `bcinr`, `stardf-kernel`).

## Verification Ladder

```text
python3 scripts/extract_purrdf_ontology.py
→ python3 scripts/verify_stardf_ontology.py --ttl authority/stardf/stardf_contract.ttl
```
