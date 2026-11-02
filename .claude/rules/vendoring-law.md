# vendoring-law

## Statement

All stardf code is vendored from `~/purrdf` at pinned rev
`0da0578306983da851a9073337d57e2e2c41ab3a`. Vendor from the rev, never the
dirty upstream working tree: `git archive 0da05783…` or
`git show 0da05783:<path>`.

The rename scope is **package-level only**: `purrdf-*` -> `stardf-*`, and it
touches ONLY the `[package] name` and `[dependencies]` keys of `Cargo.toml`.
Crate lib names, module tree, symbols, license/notice surfaces, and all
other file contents stay verbatim (byte-preserved at copy time). Licenses
are carried verbatim: the license/notice surfaces must remain byte-identical
to the upstream rev; read each crate's license at copy time and transcribe
exactly.

The falsifier fires on ANY non-rename edit: a source file inside a vendored
crate that requires a non-rename edit to build under stardf refutes the
rename-scope claim. Escalate to a new ticket — do not fix forward silently.

## Why

- `stardf/docs/jira/v26.10.31/08-vendoring-plan.md` — subject (pinned rev,
  rev-not-dirty-tree law), rename scope (§Subject), Gates G3 (byte-compare:
  license/notice surfaces byte-identical to `0da05783…`; source
  byte-identical outside the declared rename surfaces) and G4 (sha256
  receipts match `git archive` sha256 of the crate at the pinned rev),
  §Falsifier (non-rename edit = refuted claim, new ticket, not silent
  fix-forward).
- `stardf/arrow/crates/PROVENANCE-entries.md` + per-crate
  `VENDOR-PROVENANCE.md` files — the landed per-crate git-archive sha256
  receipts (G4 satisfied in aggregate form per the plan's execution banner).
- `stardf/docs/jira/v26.10.31/12-full-parity-campaign.md` — addendum banner:
  a future parity campaign starts from copy-fidelity courts (per-crate sha256
  of the copied tree vs git-archive of its purrdf origin HEAD); only
  surviving deltas earn parity work.

## How to apply

Before touching a vendored file, establish the copy-fidelity baseline
(runnable on any machine with the same checkout law):

```sh
git -C ~/purrdf archive 0da0578306983da851a9073337d57e2e2c41ab3a \
  crates/<name> | tar -x -C <scratch>
# or per file: git -C ~/purrdf show 0da05783:<path> > <path>
diff -r <scratch>/crates/<name> \
  /Users/sac/ggen-legacy/stardf/arrow/crates/stardf-<name>
```

Any delta outside the declared rename surfaces (Cargo.toml `name` /
`[dependencies]` keys) is a non-rename edit: open a new ticket instead of
editing the vendored source. New vendoring follows the same law — copy from
the pinned rev via `git archive`, rename package names only, carry licenses
verbatim, land the sha256 row in `PROVENANCE-entries.md`.
