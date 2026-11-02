# receipt-law

## Statement

No slice claims standing without a receipt. Every receipt uses schema
`ggen.legacy.stardf.arrow.receipt.v1` and carries the commands run, their
real exit codes, and the subject identity (`subject.tree_digest` /
`subject.head`) computed at receipt time — never before. Planned,
inspected, or inferred gate runs are not receipts.

1. **Replay pair byte-identical.** A receipt's replay is witnessed only by
   running `verify.py` twice and confirming byte-identical output
   (`cmp` exit 0). `replay.status` says `NOT_RUN` until that pair has
   actually been observed. The anti-vacuity flip is part of the same law:
   `verify.py --mutate drop-export` must exit 1 — a gate that cannot flip
   is not a gate.
2. **Honest failures stay in.** A failed check keeps `passed: false` in
   `checks[]` plus an `errors[]` row. Omitting a failing check to make a
   receipt read clean voids the receipt.
3. **Nonclaims mandatory.** Every receipt, ledger, and decision record
   carries an explicit nonclaims section: what was NOT run, NOT witnessed,
   NOT claimed (no external production standing, no CI standing, no live
   publish, no git mutations by receipt lanes, log-standing vs landed
   receipts). Omitting nonclaims voids the receipt.
4. **Standing vocabulary.** ALIVE / PARTIAL_ALIVE / UNKNOWN / BLOCKED /
   BUILD_BROKEN / UNSUPPORTED / REFUSED. Default UNKNOWN; never an
   unqualified "works".

## Why

- `stardf/arrow/evidence/README.md` — the receipt law (5 numbered laws:
  real gate output, receipt per wave, replay pair, honest failures, standing
  vocabulary) and `RECEIPT_TEMPLATE.json` (copy per wave, fill from real
  output).
- `stardf/arrow/docs/RECEIPTS.md` — the ledger of record: schema name
  `ggen.legacy.stardf.arrow.receipt.v1` on every landed receipt row;
  witnessed replay pairs (file sha256 + `cmp` exit 0); honest false rows
  retained (testkit full-package FAIL, 16 registry blockers, receipt-write
  corruption disclosed); campaign-final nonclaims block.
- `stardf/docs/jira/v26.10.31/04-falsifiers.md` — F5/F5a determinism rows:
  the replay-pair law is falsified if two runs differ; three receipt
  generations witnessed byte-identical pairs.

## How to apply

From `stardf/arrow`:

```sh
python3 scripts/verify.py --emit-receipt        # run 1
python3 scripts/verify.py --emit-receipt        # run 2, must be byte-identical
cmp <run1-output> <run2-output>                 # exit 0 = replay witnessed
python3 scripts/verify.py --mutate drop-export  # must exit 1 (anti-vacuity)
```

Fill `RECEIPT_TEMPLATE.json` from the real outputs; keep failed rows with
`passed: false`; write the nonclaims block; cite the file in
`docs/RECEIPTS.md`. A receipt written before the gate ran is not a receipt.
