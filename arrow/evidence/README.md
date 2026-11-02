# stardf/arrow evidence

Receipts for the stardf arrow slice. House style: `evidence/local-docs-verifier.json`
at the repo root.

## Law

1. **No slice is claimed ALIVE without the real gate output.** A check row exists
   only if the named gate actually ran this session and its observed output is in
   the receipt. Planned, inspected, or inferred gate runs are not receipts.
2. **Every wave ends with a manufacturing receipt.** It carries the commands run,
   their real exit codes, and the subject SHA (`subject.tree_digest` +
   `subject.head`) computed at receipt time — never before.
3. **Replay pair.** A receipt's replay is witnessed by running `verify.py` twice
   and confirming byte-identical output. `replay.status` says `NOT_RUN` until
   that pair has actually been observed.
4. **Honest failures stay in.** A failed check keeps `passed: false` in
   `checks[]` plus an `errors[]` row. Omitting a failing check to make a receipt
   read clean voids the receipt.
5. **Standing vocabulary** (`ALIVE` / `PARTIAL_ALIVE` / `UNKNOWN` / `BLOCKED` /
   `BUILD_BROKEN` / `UNSUPPORTED` / `REFUSED`) per the repo's canonical
   authority (`AGENTS.md`). Default `UNKNOWN`; never an unqualified "works".

## Template

`RECEIPT_TEMPLATE.json` — copy per wave, fill every field from real output.
This directory ships the template only; no receipts exist yet because nothing
has run to receipt.
