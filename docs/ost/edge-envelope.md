# Edge-Deployment Envelope (RDF-9)

Lane: RDF-9-edge (gate-runner), 2026-11-01, repo /Users/sac/stardf (canonical checkout).

## 1. Capability receipts — poll result (bounded poll, all four LOUDLY ABSENT)

At poll deadline, `arrow/docs/ost/` contains ONLY `sameas.md`. Named loudly:

| Receipt | Path | Status |
|---|---|---|
| parallel-materialize | docs/ost/parallel-materialize.md | ABSENT (find over /Users/sac/stardf: zero matches) |
| quoted-triples | docs/ost/quoted-triples.md | ABSENT (zero matches) |
| incremental | docs/ost/incremental.md | ABSENT (zero matches) |
| datalog-spine | docs/ost/datalog-spine.md | ABSENT (zero matches) |

The capability lanes have not landed receipts in this tree. What exists instead:
crates `stardf-datalog` (full implementation, vendored w2-l04) and
`stardf-incremental` (Cargo.toml-only shell, no `src/`).
`stardf-materialize` and `stardf-star` crates: absent entirely.

## 2. Per-crate edge build table

| Crate | Target | Builds | Size | Budget | Verdict |
|---|---|---|---|---|---|
| stardf-datalog 26.9.29 | wasm32-wasip1 (debug) | YES (exit 0, 21.3s) | rlib 20.9 MB (debug, incl. debug info) | n/a | BUILD-OK |
| stardf-datalog | wasm32-wasip1 (release) | YES (exit 0, 2m04s incl. test profile) | rlib 2,504,338 B | n/a | BUILD-OK |
| stardf-datalog | wasm32-wasip1 release, `--test rules_runtime` linked image | YES (exit 0) | raw 5,099,663 B; wasm-strip 4,502,346 B; CODE 4,373,087 + DATA 123,549 (llvm-size, measured) | ESTIMATE (upper bound — includes testkit + all rules_runtime tests, not a minimal lib link) | BUILD-OK, EXECUTION NOT WITNESSED |
| stardf-datalog | riscv32imc-unknown-none-elf | NO — exit 101 | n/a | n/a | FAIL: std-gated via dep stardf-hash (392 errors: `Vec`/`Box`/`std` E0433/E0463 in stardf-hash/src/dispatch.rs:292, error/io.rs:53 — no `extern crate alloc` / no_std cfg) |
| stardf-incremental | any | NO — no subject | n/a | n/a | SUBJECT MISSING (Cargo.toml only, no src/) |
| stardf-materialize | any | n/a | n/a | n/a | CRATE ABSENT |
| stardf-star | any | n/a | n/a | n/a | CRATE ABSENT |

## 3. Std-gate census

- stardf-datalog: **std-gated (host/WASI only)**, NOT no_std-capable today. It compiles
  on wasm32-wasip1 because wasip1 ships std; the riscv32imc (bare-metal) build fails at
  dependency `stardf-hash` (std types without an alloc facade). rayon (std) also gates it.
  The crate's own "wasm-clean" claim (wasm32-unknown-unknown via Node runner) is intact
  for the wasi/unknown-unknown class, but that is a std-carrying wasm, not bare-metal.
- stardf-incremental: empty shell — std-gate posture UNVERIFIED (no subject).
- stardf-materialize, stardf-star: absent.
- Zero of the four capability crates are bare-metal (riscv32imc) capable as of this run.

## 4. Honest comparison vs OST's Samsung on-device deployment

WITNESSED (this session, commands + exits above):
- stardf-datalog builds for wasm32-wasip1 release, exit 0, llvm-size measured on a real
  linked image: CODE 4.37 MB / DATA 123 KB (test-harness-inclusive upper bound, ESTIMATE).
- riscv32imc build fails exit 101 at stardf-hash — the bare-metal story is a gap, not a
  differentiator, until the alloc facade lands.

PITCH (unwitnessed in this tree):
- "Zero-copy footer integrity at the edge": no receipt (docs/ost/incremental.md absent;
  stardf-incremental has no src/), no build, nothing to measure. PITCH only.
- "Receipt-checkable derivations AT the edge": stardf-datalog's derivation-ID machinery
  (resolve_fol derivation_id, SHA-1 Merkle fold over stardf-hash) compiles to wasi-wasm
  (witnessed at build level only; execution under wasmtime traps in stardf-testkit's
  wasm-bindgen host, not in the evaluator) — but no on-device run, no SRAM budget table
  against a Samsung-class device. PITCH with a partial build-level witness.
- OST Samsung on-device: nothing in this tree witnesses it either; the comparison
  remains a pitch on BOTH sides until a receipt lands.

## 5. Commands (this session)

- `cargo build -p stardf-datalog --target riscv32imc-unknown-none-elf` → exit 101
- `cargo build -p stardf-datalog --target wasm32-wasip1` → exit 0
- `cargo build -p stardf-datalog --target wasm32-wasip1 --release` → exit 0
- `cargo build -p stardf-datalog --target wasm32-wasip1 --release --test rules_runtime` → exit 0
- `wasmtime run /tmp/rr.wasm` → trap (stardf-testkit wasm-bindgen host panic) — execution not witnessed
- `wasm-strip` + `llvm-size -A` on the linked image → CODE/DATA as tabled above
