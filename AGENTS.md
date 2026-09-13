# Working in OccluView

`CONTRIBUTING.md` holds the commit, changelog, and pre-PR rules; read it first.
This file covers only what the code does not say for itself.

## Before changing anything

- One writer per worktree. Check `git status --short --branch` and
  `git worktree list` before starting; another session may hold this checkout.
- Verify a suspected defect before fixing it. Write the failing scenario
  first, run it against the current code, and change runtime code only when it
  is red. If the scenario is green, the finding was wrong: say so and stop.
- `cargo test -p <crate> --locked <filter>` is the fast loop;
  `cargo clippy --workspace --all-targets --locked -- -D warnings` and
  `cargo fmt --all --check` are the gates before a PR.

## What owns what

- A document transition has one owner. Scene content changes go through
  `commit_structural_scene`/`set_scene`, unsaved tracking through
  `mark_mesh_edits_unsaved`, and both feed `content_revision`, which is what
  the Replace load guard compares against. Do not add a second guard beside
  them.
- An in-place scene edit requires the single `Arc<Scene>` handle; a second one
  is a reader that will not see the edit. Do not relax that assertion to make a
  call site pass.
- Worker ordering, Sculpt topology/partial updates, and the Replace/Append
  distinction are load-bearing. Preserve them.

## Tests and comments

- Prefer behaviour over source text. A test that reads `include_str!` of a
  sibling file is acceptable only when the runtime path cannot run in the
  harness, and it must fail closed if its anchor moves.
- Fixtures must fail loudly. A fixture returning `Option` that tests unwrap
  with `else { return; }` turns a broken fixture into a green test.
- Comments state an invariant, a contract, or a measured number. Measured
  numbers include the scene and machine they came from; a performance claim
  that no longer holds is a bug in the comment.
- When a finding turns out to be already handled, record that instead of
  making a cosmetic change to look busy.

## Not covered by CI

Software rendering and the test harness do not prove the installed application
works. GUI checks, packaging, and any dental case remain separate acceptance
steps, and a green CI run is not one of them.
