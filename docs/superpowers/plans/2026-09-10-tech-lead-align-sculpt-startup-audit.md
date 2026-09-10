# OccluView Align, Sculpt, Startup, and Test Quality Plan

> **For the implementation agent:** execute this plan task-by-task, keeping each
> commit narrow and independently reviewable.

## Audit handoff

The working tree already contains the implementation from the Align/Sculpt repair
and is intentionally dirty. It must be preserved and split into commits; the
untracked `package.json` is unrelated workspace dirt and is out of scope.

### Confirmed Align contract

- A point-pair fit is coarse authority only.
- A heatmap/measurement is authorized only after a current `Refined` result has
  committed successfully.
- Returning from Manual to Automatic must not submit a measurement.
- Pose, role, geometry, orientation, influence, and mask changes revoke refined
  authority and the derived map.
- The working panel keeps the operator controls and removes persistent metrics,
  grey-count/coverage prose, and brush marked-percent text.
- The displayed magnitude ramp is absolute: exact zero is blue, the selected
  maximum defaults to `0.10 mm`, and values above it are red.

### Confirmed Sculpt contract

- The CPU mesh and GPU prepared-scene topology are separate authorities and must
  be reconciled in order.
- Sparse updates may not cross a topology rebuild boundary.
- A stroke/worker failure must become a visible terminal application error and
  stop further command consumption.
- Preparation, cancellation, queue bounds, pointer ownership, and stale worker
  generations are part of the behavior, not optional diagnostics.

### Ranked startup causes

1. `eframe` initializes the only WGPU adapter/device before the application
   creator runs. The existing offscreen fallback cannot recover from no adapter,
   surface incompatibility, or device-request failure.
2. The app requests WGPU only with a fixed 4x MSAA path and eframe's default
   `max_texture_dimension_2d = 8192`; older integrated/GL adapters can fail the
   device contract before any window reaches the app.
3. On non-Windows desktop launches, a startup error is logged to stderr only.
   A `.desktop` launch has no visible console, so a failed process looks like a
   silent no-op even when a report exists.
4. The Linux package checks `ldd` for unresolved libraries but does not exercise
   a real installed launch, adapter enumeration, or a headless/no-GPU failure
   path. Windows checks static CRT linkage but not a clean-machine launch matrix.
5. Crash reports are not collision-proof when two failures occur in one second;
   process termination by a native driver or OS signal cannot be caught by a Rust
   panic hook. These boundaries need explicit diagnostics rather than claims of
   complete crash capture.

### Test-quality policy

The repository has a large, mixed test population. Tests are classified as:

- behavior tests: retain and strengthen;
- pure math/geometry/render tests: retain when they exercise an observable
  contract;
- packaging/ABI/source-tree contracts: retain only where runtime execution is
  unavailable, but make them small and specific;
- source-string mirrors of implementation details: remove or replace with a
  callable pure seam, never delete coverage without a replacement.

The audit must report counts and names before any deletion. No blanket purge or
renaming is allowed.

## Atomic implementation sequence

### Commit 1 — `feat(align): harden matching and compact operator flow`

Stage only Align kernel, Align app/panel/brush/overlay state, the Align-specific
tests, and the requested i18n strings. Verify the refined-authority gate,
absolute range, saturated map, and robust ICP together. Do not include startup,
Sculpt, or test-cleanup files.

### Commit 2 — `feat(sculpt): make topology and worker lifecycle lossless`

Stage only mesh-edit brush code, Sculpt worker/tool/app paths, mesh-editor
handoff code, and Sculpt behavior tests. Verify ordered rebuild/completion
publication, bounded queues, cancellation, terminal failures, normals, and
stale GPU protection.

### Commit 3 — `fix(render): improve measured-map readability without changing geometry`

Stage the measured-map shader and render tests only. Verify hue/magnitude
authority, zero-to-red mapping, and the absence of CPU mesh mutation.

### Commit 4 — `fix(startup): make graphics failures diagnosable and compatible`

Test first, then implement the smallest causal startup seam:

1. Make adapter/device requirements adapter-aware rather than demanding an
   unsupported 8192 texture limit from downlevel hardware.
2. Select only surface-compatible adapters and log a redacted adapter summary
   (backend/type/name-free stable fields) before device creation.
3. Use a compatibility-safe presentation profile and keep the custom viewport's
   sample count synchronized with the actual eframe render pass.
4. Make non-Windows fatal startup visible through a desktop notification/openable
   report path when available, while always retaining a persistent report and a
   clear exit status for shell launches.
5. Make crash/startup report filenames unique and include launch/runtime context
   without paths, scan names, or patient identifiers.
6. Add a documented diagnostic command or helper that reports backend, adapter,
   driver, session/display, and required-vs-supported limits. Do not promise that
   a Rust hook can catch a native driver SIGSEGV.

If the installed Windows/Linux artifact cannot be executed in this checkout, the
commit must add package contract tests and state the remaining installed-machine
acceptance gate explicitly.

### Commit 5 — `test: remove brittle duplicates and preserve behavior coverage`

Use the inventory from the audit to remove only duplicate source-string tests or
move them into the narrowest existing contract module. Replace any removed
coverage with direct pure-helper tests. Keep source-tree and packaging tests that
protect otherwise unexercised release boundaries. Add a test-list/count check so
future test growth is visible rather than silently becoming a second product.

### Commit 6 — `docs: record release acceptance gates`

Update the existing operator/release documentation only if needed: supported OS,
GPU/driver prerequisites, startup-report locations, WGPU diagnostic invocation,
and the separate installed-app acceptance matrix. Do not claim browser,
installed-platform, or clinical acceptance from Cargo tests.

## Verification gates

After each implementation commit:

- run the smallest relevant package tests and the changed source-format check;
- inspect `git diff --cached` and `git diff --check` before committing;
- keep commit paths explicit and leave unrelated dirt untouched.

After all commits:

- run fresh `cargo fmt --all -- --check`;
- run fresh `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`;
- run the release/performance harness relevant to Sculpt and rendering;
- run Linux package syntax/checker and a launch/diagnostic smoke where the
  current display permits it;
- inspect the final diff and commit ancestry;
- perform two independent final passes: Align/interaction authority and
  Sculpt/startup/tests/release risk. If external subagents are unavailable,
  record that limitation and use separate checklists locally.

## Acceptance boundaries

Automated proof can establish contracts, determinism, and package structure. It
cannot establish a successful launch on a colleague's exact Windows driver,
Explorer/desktop behavior, or clinical registration quality. Those remain named
acceptance gates with a diagnostic report attached when they fail.
