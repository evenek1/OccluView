# Align and Contact Controls Recovery Implementation Plan

> **For agentic workers:** Use a bounded implementation cycle per task, with a failing behavioral check before changing product behavior. Keep the preserved Pi export untracked.

**Goal:** Restore a usable contact strip and verify that Best Fit Matching behaves as the owner requested, then ship a traceable Debian package.

**Architecture:** The contact UI uses the existing viewport overlay and Layers geometry; the Align tool keeps one route from panel action through worker to accepted pose and map. Document-loading safety and copy review are separate bounded changes.

**Tech Stack:** Rust, egui, wgpu, Debian packaging.

**Spec:** `docs/superpowers/specs/2026-09-12-align-contact-recovery-design.md`

## Global constraints

- Preserve untracked Pi export and unrelated shared work. Stage explicit paths.
- A passed unit test, software render or `.deb` check does not prove installed GUI or operator acceptance.
- Do not claim a fit between two different jaws is a unique rigid registration without case-specific evidence.

---

### Task 1: Contact strip geometry and close action

**Files:** `crates/occluview-app/src/app/app_contact_bar.rs`, `crates/occluview-app/src/app/app_cut_measure.rs`.

- [x] Reproduce viewport overflow and Layers overlap with a test at 600, 1024 and 1600 points.
- [x] Derive the strip rectangle from `layers_overlay::layer_overlay_rect`, keeping it inside the viewport.
- [x] Paint `AppIcon::Close` instead of a font glyph and reserve the real Details width.
- [x] Inspect a live viewport with contacts open, including Details and a narrowed window; follow-up Align resize inspection exposed and repaired a saved-position overlap.
- [x] Run focused layout tests and formatting after the last UI edit.

### Task 2: Align behavior and speed

**Files:** `crates/occluview-align/tests/real_scans.rs`, `crates/occluview-align/src/icp.rs`, `crates/occluview-app/src/align_worker.rs`, `crates/occluview-app/src/app/app_align_results.rs` only if a measured failure identifies an owner.

- [x] Run real STL placement and refusal sweeps; seven real-scan tests passed, including 0.5–25 mm same-arch starts and 0–40 mm different-arch refusals.
- [x] Find a distinct public prep/original pair for the same jaw; near and 25 mm plus rotation starts converged to the same trusted result in about 3.1 seconds each.
- [x] Reproduce a changed-anatomy, partial-overlap case with known ground truth; the old path reported trust at 3.897 mm error at one probe, and the feature-seeded path returned at most 0.006 mm error across three probes. A harder 12%-width patch case remains unresolved and must not authorize a false map.
- [x] Clear a stale map when a new fit starts; existing worker trust gating prevents a refused fit from authorizing a fresh map.
- [x] Inspect the Align panel in the live app at normal and narrow sizes; resize reanchoring was added after a persisted position overlapped Layers.

### Task 3: Remaining requests from the original prompt

**Files:** `crates/occluview-app/src/app/app_contact.rs`, `app_mesh_editor.rs`, `app_sculpt.rs`, Settings and i18n files only for confirmed gaps.

- [x] Confirm the existing contact shading route and inspect live contact PNGs; the owner still needs to judge blue and gloss on their own scans.
- [x] Check the multi-layer Edit Mesh implementation and its behavioral tests, and the prior first-hover Sculpt fix. Live brush feel remains for operator acceptance.
- [x] Confirm Settings contains shortcut help and no recent-count control; review the changed Align hints across seven locales. A full application-wide copy rewrite is outside this Align-focused release.

### Task 4: Separate document-safety findings

**Files:** `crates/occluview-app/src/app/app_loading.rs`, `state_document.rs`, `crates/occluview-formats/src/dispatch.rs`, and behavioral tests for the failing transitions.

- [x] Guard a Replace result when edits or an in-flight edit appeared after authorization, and keep busy sessions parked.
- [x] Bound active decoding to one and let the latest Replace supersede prior queued requests without changing Append order.
- [x] Replace the unsafe mapping read with owned bytes and verify a truncated source does not invalidate loaded bytes.

### Task 5: Release and independent check

**Files:** `install/linux/build-deb.sh`, `install/linux/check-deb.sh` only if packaging evidence reveals a defect.

- [x] Review UI placement, solver refusal/heatmap, and document safety through independent counterexamples and live resize; avoid an unbounded audit fan-out.
- [ ] Run appropriate Rust tests, strict Clippy, a real-case render, and `install/linux/check-deb.sh`.
- [ ] Commit owned files, rebuild `target/deb/occluview_1.1.1_amd64.deb` from that commit, verify provenance and SHA-256, upload a uniquely named package to the already authorized WebDAV share, download it and compare bytes.
- [ ] Record installed-app/operator acceptance as a separate final gate.
