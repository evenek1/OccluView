# HANDOFF — OccluView audit/coverage wave

You are taking over a job that is **60% done**. Read this whole file before touching
anything. It contains: what the repo is, what was asked, what is finished, exactly
what remains, and how to prove each step.

---

## 0. TL;DR for the impatient

- **Repo**: `/home/wow/orca/workspaces/occluview-public/1.21`
- **Branch**: `zer0ltrnce/1.21` — pushed, working tree CLEAN. The handoff you
  are reading is itself a commit on top; take the real HEAD from `git log -1`, and
  note that the commit which added this file is `a60ccf1`.
- **Upstream**: `https://github.com/occlutrace/OccluView.git` (gh authed as `zer0ltrnce`, has `workflow` scope)
- **Version**: `1.2.1` (the user calls it "1.21"; the branch is named after it)
- **Language**: Rust workspace, dental CAD viewer (egui + wgpu)
- **Done**: full test suite run and green (**909 app-lib tests, 0 failures**), ~50
  source-text tests deleted, 40 behavioural tests added, **1 real product bug found
  and fixed**, CI green once.
- **Remaining**: 32 uncovered behaviours, independent verification, rebuild
  artifacts (current ones are 18 commits stale), upload to Nextcloud.

---

## 1. THE STANDING USER MANDATE (read this twice)

The user is the owner of OccluView. Over many turns he demanded, in escalating
frustration, these things — **all still active**:

1. **Delete ALL source-text tests, completely.** Tests that read a crate's own
   `.rs` file (`include_str!("*.rs")`, a `production_source()` helper, or
   `.contains("fn foo")` on source) are worthless: they pass on their own text,
   break on reformatting, and gave false confidence. He asked **four separate
   times**. This is not negotiable and is *mostly done* — see §5.
2. **Run the tests.** He explicitly authorised the full suite ("я тебе разрешаю
   полностью гонять тесты и вообще все что угодно"). Do not be timid about running
   them.
3. **Bring the codebase to an ideal state.** Not "good enough".
4. **Cover the behaviours the deleted tests were *supposed* to guard.** He accepts
   that some cannot be covered, but wants to know which and why.
5. **Write the remaining work as a numbered list**, then **implement every item**.
6. **Then send independent subagents to verify everything was really done.**
7. **Then build Windows MSI + Debian deb with the HPS key** and **upload to his
   Nextcloud share** (details in §8).

**His tone expectations**: he wants honesty about limits, not optimism. If you
cannot do something, say so plainly and say what would settle it. Do not claim
"done" for anything you did not verify. He notices when a tracking file loses a
line.

---

## 2. THE MACHINE RULES (violating these breaks the shared box)

This box is 12 cores / 15 GB shared by **many** agent sessions.

- **Every heavy command goes through the lock wrapper:**
  ```
  /home/wow/occlutraceio/scripts/heavy.sh cargo test -p <crate> <filter>
  ```
  - `exit 75` = box busy, NOT an error. Wait and retry. For a long wait:
    `HEAVY_WAIT_SECONDS=1500 /home/wow/occlutraceio/scripts/heavy.sh ...`
  - The lock is also held by other projects (`next build`, `tsc`, deploys). If it
    stays busy, check `ps -ef | grep -E 'cargo|next|tsc'` — the holder is often a
    legitimate 15-minute build. Do not bypass the lock.
- **NEVER run `cargo test --workspace`** while other sessions may be working — too
  heavy. Run per-package.
- **One writer per worktree.** If you spawn subagents that edit files, run them
  **sequentially**, or give each its own `worktree: true`. Two agents editing the
  same files WILL destroy work — this happened twice in this job (see §6).
- **`AGENTS.md` at `/home/wow/AGENTS.md`** is binding. Read it.

**Git rules**: never force-push, never rewrite history, never touch `main`.
Pushing to `zer0ltrnce/1.21` is implied-authorised.

---

## 3. THE REPO, IN ONE PARAGRAPH

A Rust workspace. Crates that matter:

| Crate | What it does |
|---|---|
| `occluview-app` | the GUI app (egui + wgpu, eframe). ~900 tests. This is where most work is. |
| `occluview-formats` | STL / PLY / OBJ / OFF / glTF / 3MF / HPS readers + writers |
| `occluview-hps` | the encrypted `.hps`/`.dcm` format (needs a private key in CI) |
| `occluview-render` | wgpu pipelines, offscreen rendering, contact field shaders |
| `occluview-shell` | Windows Explorer integration (COM preview + thumbnail handlers). **Windows-only, does not compile on this Linux box.** |
| `occluview-cli` | `occluview-cli` with `thumbnail`, `convert`, `close-holes` subcommands |
| `occluview-thumbnail` | the shared thumbnail renderer |
| `occlu-mesh-edit`, `occluview-align`, `occluview-contact`, `occluview-core` | geometry kernels |
| `occluview-update` | in-app updater (minisign-verified) |

**Key domain facts you must not break**:

- The **PLY texture must stay embedded** in one file. There is no sidecar PNG, and
  `comment TextureFile` must never be written.
- The **HPS key lives only in CI secrets** (`OCCLUVIEW_HPS_EMBEDDED_KEY`). A local
  build cannot open encrypted `.hps`/`.dcm` — that is by design, not a bug.
- The switchable-shell contract: the MSI ships a **pinned** Explorer shell revision
  listed in `install/shell-pin.json` (currently `659725632dff` / v1.1.0). The 110+
  commit gap to the current tree **is the contract**, not a defect.
- The **.dcm ProgID is canonicalised to `MeshFile.HPS`**; `.dcm` is offered to the
  user but never claimed from medical DICOM.

---

## 4. WHAT WAS ALREADY DONE (do not redo)

### 4.1 The test suite was run for the first time in this wave

It found **4 real failures**, three of them regressions introduced by earlier
audit fixes. All four are fixed in commit `4a96390`:

| Failure | Cause | Fix |
|---|---|---|
| append lost a held drag pose | `set_scene` called `discard_align_drag` (drop) — but `set_scene` is also how an **append** installs its combined scene, and an append keeps the layer, so the pose survived into the new scene with nothing recording it | call `abandon_align_drag` (the commit form) |
| a superseded decode's successor never started | `start_next_queued_load` refused **every** queued Replace, including the newest one | removed the blanket refusal; obsolete ones are already dropped by `supersede_queued_replaces` |
| BOM-prefixed binary STL, by extension | the raw size formula reads the count 3 bytes late on a BOM-prefixed file | ask the formula again of the stripped bytes |
| Bridge Split cancel | `cancel()` replaced the worker with a default one, throwing away a compute injected by a test | `BridgeSplitWorker::abandon()` restarts the thread over the **same** compute |
| i18n pin | left at 691 after the wave added 6 keys and removed 4 | 693 |

**Result after fixes**: `cargo test --workspace --all-targets` → **~2100 tests, 0 failures**.

### 4.2 A REAL product bug was found and fixed (`6bb7987`)

**Opening a contact reading from the layer menu panicked a debug build.**

Path: Layers (or viewport right-click) → **Contacts**, with a Best-fit heatmap up.
`begin_contacts_from_layer` clears the align overlay, and clearing it edits the
document's **live** scene in place. `live_scene_mut` asserts in debug that it holds
the only `Arc<Scene>` — but the caller had passed a second handle. In release,
`live_scene_mut` falls back to **copying** the scene, so the caller's handle went
stale for the rest of the action while the operator saw a reading measured on the copy.

**Fix**: contact actions get their own branch in `apply_layer_overlay_changes`
before the structural draft is built, so the caller's handle is dropped first.

**Why it survived**: a source-text guard
(`nothing_holds_a_second_scene_handle_across_an_in_place_edit`) parsed the source
for `scene.clone()` alive across an edit and asserted there were none. It could not
see this one. That test is **deleted**, and replaced by a behavioural test
`opening_contacts_from_the_menu_does_not_edit_the_scene_under_a_second_handle`.

**Proven both ways**: with the fix reverted the test panics at
`state_document.rs:146`; with the fix it passes. **Always do this** — a new test
that cannot fail is worthless.

### 4.3 Source-text tests: 66+5 deleted

Deleted across commits `bc4c393`, `8209fc2`, `2415880`, `0470435`, `6bb7987`,
`97b8c50`, plus helpers (`production_source`, `method_body`, `function_source`,
`appears_before`, `registration_source`, `combined_com_source`, `mod_source`, and
per-file slicers).

**Deliberately kept** — these read a `.rs` **and** another artifact, so they compare
things that can disagree. **17 remain, and the split is a judgment call — verify it
rather than trusting this list:**

- `crates/occluview-shell/src/shell_contract_tests.rs` (3) and
  `installer_contract_tests.rs` (2): read the shell registration **sources** and the
  shipped `install/occluview.wxs` / `install/occluview-shell-registration.reg`.
  A genuine cross-artifact contract (the DLL and the MSI must agree).
- `primary_ui_tests/platform.rs` (10): packaging/release contracts. They read
  `install/build-msi.ps1`, `install/linux/build-deb.sh`, `.github/workflows/*`,
  `Cargo.toml`, `CHANGELOG.md`. Most compare two shipped artifacts.
  **Two of them are mostly self-text and are candidates for deletion or conversion:**
  `windows_app_reports_startup_and_panic_failures` and
  `linux_build_uses_real_gui_instead_of_failure_stub` (they assert
  `source.contains("install_panic_hook();")` and similar).
- `primary_ui_tests/chrome.rs::readme_lists_every_embedded_interface_language` (README vs catalogue)
- `primary_ui_tests/documents.rs::the_readme_documents_the_shortcuts_the_build_implements` (README + source)

**One genuinely useless test is still there — delete it:**
`primary_ui_tests/platform.rs::platform_identity_values_are_pinned_unconditionally`
(reads `lib_source()` and asserts constants appear in the text).

**Exact current count**:
```
grep -rn 'repo_source_file\|include_str!("[^"]*\.rs"' crates/    # 14 hits, 5 of them helpers
```
18 test functions in that family; 1 is clearly useless, 2 are borderline, 15 compare
real cross-artifact contracts. `repo_source_file` also lives in
`primary_ui_tests/mod.rs` along with `main_source` / `lib_source` /
`app_bootstrap_source`; before deleting the helper, confirm the remaining callers.

### 4.4 ~50 behavioural tests added across 21 commits

By area: sculpt stroke lifecycle (7), sculpt commit + align authority (8), offscreen
failure ladder (5), CLI/crash-privacy (2), normal-frame math (2), overlay/map
bookkeeping (12), window identity (2), upload buffer, align wheel, AccessKit panel.
All are in `/tmp/cover-*.md` and `/tmp/cover2-*.md`.

### 4.5 One source-text check converted into a REAL security test

`the_updater_speaks_only_https_outside_its_own_tests` used to grep `src/lib.rs` for
`.https_only(!cfg!(test))`. Replaced by
`crates/occluview-update/tests/plain_http_refused.rs` — an **integration** test
(crucial: `cfg!(test)` is false there, so it drives the same `https_only(true)`
agent a shipped build uses). It opens a loopback HTTP listener, calls `check_with`,
and asserts no connection is ever opened. Proven: with `.https_only(false)` it fails,
with the policy held it passes.

---

## 5. REMAINING WORK

### 5.1 The plan file — read it first

**`docs/remaining-work.md`** (in-tree, committed) is the authoritative plan. It
lists **88 items** from two verification reports (`/tmp/verify3-guard-sculpt.md`,
`/tmp/verify3-guard-other.md`), triaged into:
- **P1 (35)** — money paths, silent data loss, hardware faults, security. **Must cover.**
- **P2 (35)** — interaction correctness, maths. **Should cover.**
- **P3 (14)** — panel layout/labels/focus. These were text assertions; the honest
  form is a test over egui's produced **widget list**, a bigger change.
- **P4 (4)** — "this constant appears in this file". **Deliberately NOT covered** —
  covering them would recreate the mistake.

### 5.2 Exactly what is still uncovered: **32 items**

Verified by name (accounting for renames) with:
```
grep -rn "fn <item>" crates/ --include=*.rs
```

**Testable on this machine (do these):**
```
brush_hotkeys_survive_a_held_shift
sculpt_hotkeys_switch_to_sculpt_from_edit_mesh
an_active_stroke_stops_sampling_when_pointer_leaves_viewport
sculpt_cursor_follows_pointer_ownership_and_warm_pick_readiness
a_click_that_turns_the_pair_around_invalidates_the_fit
arming_align_stands_the_other_tools_down
removing_a_named_layer_revokes_refined_authority
the_orientation_rule_is_disabled_while_a_fit_runs
a_settings_change_abandons_a_running_fit_without_waiting_for_a_claim
optimizer_setting_changes_drop_the_refined_authority
returning_to_automatic_does_not_measure_implicitly
a_job_never_re_copies_geometry_that_has_not_changed
a_job_carries_the_identity_of_what_it_measures
a_warm_bvh_is_never_started_for_an_abandoned_worker
densification_failure_is_not_silently_dropped
every_heavy_call_goes_through_the_worker
numeric_range_edits_recolour_the_cached_map
the_map_is_locked_until_a_refined_match_lands
a_stroke_takes_its_direction_from_the_toggle_and_shift_together
consumer_wheel_align_shift_resizes_once_from_horizontal_raw_event
no_control_in_the_window_names_a_target_a_source_or_a_role
```
(Names may differ from the ones you write — that is fine. Cover the BEHAVIOUR.)

**Needs a GPU (lavapipe may provide one — check):**
```
a_dab_is_scoped_to_the_explicit_mesh_selection
a_dab_reuses_the_cached_geometry_and_re_colours_only_what_it_touched
a_dab_uploads_only_the_vertices_it_touched
a_rejected_sparse_overlay_upload_is_not_reported_as_success
a_layer_rebuild_is_installed_before_any_sparse_vertex_write
mapped_range_failure_cleans_up_before_the_error_can_return
live_viewport_keeps_selection_overlay_separate_from_base_scene
the_shader_is_told_the_width_the_field_was_packed_with
```
**Important**: several offscreen tests exist already but **early-return when no
adapter is present**. Check whether they assert anything on that path — a test that
passes vacuously on a GPU-less box is false coverage. Say so if you find it.

**Windows-only (say CANNOT and describe the Windows test that would settle it):**
```
the_handoff_pipe_cannot_be_squatted_or_used_to_impersonate   <- SECURITY, highest value
installer_refreshes_shell_association_cache_after_registry_changes
```

**Deliberately not covered (record as a decision):**
```
worker_source_has_no_catalog_key_literals   (P4)
```

**Lost report** — a lane was writing these and died before reporting; the tests may
or may not exist. Verify by grep, and write if missing:
```
a_result_the_operator_has_overtaken_is_never_applied   (may exist: app_align_authority_tests.rs)
late_measurement_cannot_reopen_hidden_or_unrefined_map  (may exist)
a_measurement_with_no_summary_is_not_painted_on_the_scan (may exist)
dropping_a_stale_map_also_drops_the_work_behind_it       (may exist)
measurement_requires_a_landed_refined_match             (may exist)
a_geometry_change_forgets_the_whole_fit                (may exist)
a_sculpt_commit_revokes_the_alignment_measured...      (may exist)
every_terminal_failure_exit_raises_the_dialog_and_disarms (may exist)
clicked_triangle_normals_stay_in_the_mesh_local_frame  (EXISTS, aba42c5)
fixed_pair_normals_use_the_inverse_transpose...        (EXISTS, aba42c5)
```

### 5.3 P3 decision

Judge for yourself whether an egui **widget-list / AccessKit** harness is cheap.
One such test already exists (`a0c6701 test(align): check the window's controls
through AccessKit`) — read it and decide whether to extend the pattern. If the
harness is expensive, record the gap explicitly rather than faking coverage.

---

## 6. HOW TO RUN SUBAGENTS WITHOUT DESTROYING WORK

This bit you twice in this job. **Read carefully.**

- Use `subagent` with `workflowScript` (write the script to a **file** — inline
  scripts with backticks/escapes fail to parse — then use `workflowScriptPath`).
- **Do NOT fan out parallel writers into one worktree.** Two lanes died mid-edit
  and left **production code mutated**:
  - one replaced `apply_dab_cancellable(.., &state.stopping)` with `apply_dab(..)`
    in `sculpt_worker_loop.rs`, silently removing dab cancellation;
  - another left `app_align.rs` and test files in a half-edited state.
  Run writer lanes **sequentially** (`for (const lane of lanes) await runs.run(...)`)
  or set `worktree: true` per lane (costs a cold build each).
- **Give every writing lane a budget-discipline paragraph**: "commit after the first
  2 passing tests; `git diff` before every commit must show ONLY test files and
  `#[cfg(test)]` seams; never leave a mutation in production code."
- **Instruct read-only lanes to stay read-only.** Do not ask a read-only lane to
  prove a test fails by reverting production code — that is how the tree got broken.
  Ask it to reason about vacuously-passing assertions instead.
- After **every** lane: `git status --porcelain` and `git diff` before doing anything
  else. Revert production damage with `git checkout -- <file>`, keep good tests.
- A lane that times out may still have written useful tests — salvage them, then
  compile and fix them yourself (they often have wrong API names).

Helper scripts already exist in `/tmp`: `verify4-workflow.js` (independent
verification, 3 lanes, written and validated but **never run**),
`coverage2-workflow.js`, `coverage3-workflow.js`, `rustlex.py` (a Python Rust lexer
used for safe deletion).

---

## 7. VERIFICATION (step 6 of the mandate)

`/tmp/verify4-workflow.js` is written and passes `action: "validate"`. It has three
lanes: sculpt coverage, render/crash coverage, and suite-honesty (runs the real
suites per package and checks every claimed-closed item has a passing test).

Run it with `workflowScriptPath`. Keep the lanes read-only. Expect it to find
weaknesses — that is its job. Fix what it finds, then re-verify.

---

## 8. BUILD ARTIFACTS AND UPLOAD (steps 7-8)

### 8.1 Windows MSI + Debian deb, with the HPS key

Both jobs live in **`.github/workflows/package-msi.yml`** and both use
`secrets.OCCLUVIEW_HPS_EMBEDDED_KEY`.

```
gh workflow run package-msi.yml --ref zer0ltrnce/1.21 \
  -f release_dry_run=false -f windows_configuration=release
```
Then watch it:
```
gh run list --workflow=package-msi.yml --limit 3
gh run watch <id>
```
Artifacts uploaded by the workflow: `occluview-windows-package` (the `dist/` dir
with the MSI + portable zip + `.sha256`) and `occluview-linux-package` (`target/deb/*.deb`).

**The current CI artifacts are STALE** — the successful run `35828545129` was built
from `941339e`, which is behind the current HEAD. **Rebuild after the tree is
final.** This matters to the user: stale artifacts are exactly what he complained
about before.

**You can also build the deb locally** (proven to work):
```
OCCLUVIEW_HPS_EMBEDDED_KEY="any-placeholder" \
  /home/wow/occlutraceio/scripts/heavy.sh bash -c "install/linux/build-deb.sh"
/home/wow/occlutraceio/scripts/heavy.sh bash -c \
  "install/linux/check-deb.sh target/deb/occluview_1.2.1_amd64.deb"
```
The MSI cannot be built locally (needs Windows + WiX) — that is what the CI job is for.

### 8.2 Upload to the user's Nextcloud

- **Share page**: `https://cloud.mortals.run/index.php/s/PoYHbp3nwLfJZZP`
- **WebDAV (upload here)**: `https://cloud.mortals.run/public.php/dav/files/PoYHbp3nwLfJZZP/`
- **Auth**: `-u 'PoYHbp3nwLfJZZP:'` (share token as username, empty password)
- **Verified**: PROPFIND returns **207**; the folder currently holds old files
  (`occluview_1.1.1_amd64.deb`, `FLASH4.1.zip`, etc.). **Delete the stale
  OccluView artifacts** so the user does not download the wrong thing.
- **Download URL pattern**: `.../download?path=%2F&files=<name>`

Upload with, for example:
```
curl -T dist/OccluView-1.2.1-x64.msi -u 'PoYHbp3nwLfJZZP:' \
  https://cloud.mortals.run/public.php/dav/files/PoYHbp3nwLfJZZP/
```
Upload the MSI, the portable zip, the `.deb`, their `.sha256` files, and a short
"what changed" note in Russian if you can write one. **Verify byte-for-byte** after
upload (re-download and compare hashes).

---

## 9. THE TRACKING FILES (the user checks these)

- **`.audit-ledger.md`** (in-tree, committed) — the running record. It has been
  updated for this wave. **The user noticed once when it lost a line — keep it accurate.**
- **`docs/remaining-work.md`** (in-tree, committed) — the triaged plan. **Update it
  as you close items**, and do not overstate: mark CANNOT honestly.

---

## 10. HOW TO TELL YOU ARE DONE (acceptance criteria)

Do not report success until **all** of these hold:

1. `timeout 1700 /home/wow/occlutraceio/scripts/heavy.sh bash -c "cargo fmt --all --check; cargo clippy --workspace --all-targets --locked -- -D warnings"`
   → **0 errors, 0 warnings**.
2. Per-package test runs all green (`occluview-app`, `-formats`, `-cli`,
   `occluview-update`, `occlu-mesh-edit`, `-render`, `-thumbnail`, `-align`, `-contact`, `-core`).
3. Source-text tests: only the cross-artifact families in §4.3 should remain.
   Verify the split yourself: 18 test functions read a `.rs`; 1 is clearly useless
   (`platform_identity_values_are_pinned_unconditionally`), 2 are borderline
   (`windows_app_reports_startup_and_panic_failures`,
   `linux_build_uses_real_gui_instead_of_failure_stub`), and 15 compare a `.rs`
   against a shipped artifact (MSI/`.reg`/CI/README) — a legitimate property.
4. Every item in `docs/remaining-work.md` is marked **WRITTEN / COVERED / CANNOT
   (with reason)** — none left ambiguous.
5. Independent verification lanes (§7) have run and their findings are fixed or
   explicitly recorded.
6. `git status --porcelain` is **empty**; HEAD is pushed.
7. Fresh MSI + deb built from the **final** commit, downloaded, and uploaded to the
   Nextcloud share; stale OccluView files removed; hashes verified.
8. The final report to the user states: the outcome, the evidence (real command
   output), the remaining limits (the CANNOT list and the P3/P4 decisions), and the
   next acceptance gate. **Plain language, no emoji, no canned phrasing.**

---

## 11. KNOWN TRAPS (each of these already bit this job once)

1. **`git stash pop` applied an unrelated old stash** and polluted the tree. Do not
   use `git stash` casually in this repo. Verify with `git status` after any git
   plumbing.
2. **A 14 GB corrupted `target/debug/incremental`** caused linker errors
   (`undefined hidden symbol`) that looked like real compile failures. Fix:
   `rm -rf target/debug/incremental`. The disk is fine (593 GB free); RAM is the
   constraint.
3. **`cargo fmt --all` reformats files mid-lane** — if you see unexplained diffs in
   files you did not touch, that is what happened. Commit them as `style:`.
4. **egui API drift**: in this version it is `ctx.run_ui(raw, |ui| ...)`, the closure
   takes `ui` not `ctx`, and you must call `.drop_without_applying_deltas()` or the
   test panics with "Dropped TexturesDelta with 1 unapplied deltas".
5. **`git add -A` races a subagent working in the same worktree.** Be explicit about
   which paths you stage.
6. **Reading `.rs` files in a test is the thing being eliminated.** If you catch
   yourself writing `source.contains("...")`, stop.
7. **A test that early-returns when a GPU/adapter is missing asserts nothing.** Say
   so rather than counting it.

---

## 12. FIRST FIVE COMMANDS TO RUN

```bash
cd /home/wow/orca/workspaces/occluview-public/1.21
git log --oneline -1 && git status --porcelain          # expect empty tree, and a60ccf1 somewhere below HEAD
sed -n '1,80p' docs/remaining-work.md                    # the plan
grep -rn "repo_source_file\|include_str!(\"[^\"]*\.rs\"" crates/ | wc -l   # expect 14 (5 of them are helpers, not tests)
timeout 1700 /home/wow/occlutraceio/scripts/heavy.sh bash -c \
  "cargo test -p occluview-app --lib --locked 2>&1 | tail -3"   # expect 909 passed, 0 failed
```

Then work §5, verify §7, build §8, and report against §10.
