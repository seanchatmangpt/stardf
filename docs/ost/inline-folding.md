# inline-folding: bit-63 structural folding of quoted triples

Lane: PP-4 BUILD-INLINE (2026-11-01). Crate: `crates/stardf-inline`
(new, workspace member). Receipt per dispatch: this file.

## 1. Design-around (stardf-star assessed, not extended)

stardf-star (landed, RDF-4) represents a quoted triple ⟨⟨s p o⟩⟩ as a
content-addressed `truncate64(BLAKE3(...))` u64 — full 64-bit id, no
discriminator bit, no arena. That is a different design-around from the
folding scheme the dispatch names, so the NEW crate route was taken (the
dispatch's alternative — adding `inline.rs` to stardf-star — would have
mixed two incompatible word layouts in one crate).

| surface | stardf-star | stardf-inline |
|---|---|---|
| word layout | 64-bit content id | bit 63 discriminator + 63-bit fingerprint |
| storage | BTreeMap registry | fingerprint table + dense arena |
| collision | assumed absent | typed trap, fail-closed |

## 2. The fold (layout law)

Bit 63 is the STRUCTURAL-FOLD discriminator: set iff the word carries a
folded quoted triple. Bits 0..62 carry `truncate64(BLAKE3(FOLD_DOMAIN ||
canonical(s,p,o)))` with bit 63 masked off. Folded and plain words
occupy disjoint halves of the u64 space; one shift tells them apart.
`FOLD_DOMAIN = b"stardf-inline:fold:v1"` is distinct from stardf-star's
`QUOTED_DOMAIN`, so an inline fold and a star id never agree by
accident.

## 3. Collision analysis (centerpiece)

Fingerprint space: 2^63. Birthday bound for n quoted triples:
`p(any collision) = 1 - exp(-n(n-1)/2^64) ≈ n^2/2^64`.

| corpus n | p(any collision) | expected colliding pairs ≈ n^2/2^65 |
|---|---|---|
| 10^6 | 5.42e-8 | 2.7e-8 |
| 10^8 | 5.42e-4 | 2.7e-4 |
| 10^9 | 5.28e-2 | 2.7e-2 |
| 4.3e9 | ~0.5 | ~0.5 |

Witness: `court4_collision_probability_math` (executed, not asserted
from prose; the exact birthday values are computed in the court and
fenced against closed-form anchors — note the court initially fenced the
wrong hand-arithmetic (5.4e-3 at n=1e8) and was corrected by the
computation: 5.42e-4).

Trap discipline: a collision is a TYPED TRAP, not corruption. At fold
time a fingerprint hit with a differing payload checksum refuses the
store (`FoldError::Trapped`, logged to `store.traps()`). At read time
every arena read is checksum-verified. Because a collision can only
cost a loud refusal — never silent misidentification — the scheme is
sound at any corpus size; the analysis above quantifies refusal
frequency, which stays < 6% at the 10^9 bound.

## 4. Differential vs stardf-star (court1)

The shared 60-triple corpus (6 nested) through both representations
produces equal quoted registries (count 60 = 60, payload presence
bijective, refold dedupes idempotently, integrity walk empty).

## 5. Scan class (court3, MEASURED)

Folded words are indirection-free u64s; scan bandwidth measured over
128 MB (16M words), checksum-kernel:

- release profile: raw 68.6 GB/s, folded 57.7 GB/s, ratio 0.842
- debug profile: raw 2.0 GB/s, folded 1.5 GB/s, ratio 0.747

Fence: ratio >= 0.5 and folded >= 1.0 GB/s floor. The fold PRESERVES
the dense-u64 scan class; the indirection (table lookup) is only on
fold/lookup paths, never in the scan stream.

## 6. Anti-vacuity courts (scripts/courts.sh, /tmp sandbox)

Both flips ran this session, BEFORE pass -> SHIM fail -> byte-exact
sha256 restore:

- `--flip-trap-bypass`: mutation = accept any payload on a fingerprint
  hit. court2 fails (exit 101). Confirmed.
- `--flip-wrong-index`: mutation = `table.insert(fp, idx + 1)`. court1
  fails (exit 101). Confirmed.

## 7. Commands (this session)

- `CARGO_TARGET_DIR=/tmp/pp4-target cargo test -p stardf-inline` — 4
  passed, 0 failed (after two arithmetic corrections in court4 fences).
- release scan court run: `cargo test --release ... court3` — numbers
  above.
- `courts.sh --flip-trap-bypass` / `--flip-wrong-index` — both flips
  confirmed, restores byte-exact.
