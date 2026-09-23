# Remaining work: the 88 uncovered behaviours and the inconsistencies

Written after the first full test run of this wave (`4a96390`) came back green:
2100+ tests, 0 failures. Green means every EXISTING test passes. It does not mean
the behaviours below are checked, because the tests that used to assert on them
read source text and were removed.

Two verification reports listed them (`/tmp/verify3-guard-sculpt.md`,
`/tmp/verify3-guard-other.md`). This file triages every one of them.

## The standard being applied

A serious project covers behaviour that can silently fail an operator. It does
not cover:

- a pure helper whose only caller is the test (that is a test of a test);
- a value passed straight through to a leaf already covered elsewhere;
- a constant that appears in one place and cannot be got wrong;
- text that a formatter can change without changing behaviour;
- accessibility/label facts that are checked against the catalogue elsewhere.

Everything else gets a behavioural test. Where a behaviour is already exercised
as a side effect of a surviving test, it is marked COVERED and says which test.

## Priority 1 — money paths, silent data loss, hardware faults (must cover)

1. `mode_switch_finishes_a_live_stroke_instead_of_aborting_it` — switching brush
   mid-gesture. Aborting loses the operator's stroke. app-level test.
2. `toggling_off_does_not_drop_a_worker_with_a_queued_finish` — the same loss
   through the off-toggle. app-level test.
3. `abort_also_reverts_a_released_stroke_waiting_in_the_worker` — Ctrl+Z during
   a released-but-unfinished stroke.
4. `a_rejected_finish_keeps_the_stroke_for_a_later_retry` — a refused commit
   that then silently drops the work.
5. `worker_loss_invalidates_an_active_sculpt_stroke` — worker dies mid-stroke.
6. `a_layer_rebuild_is_installed_before_any_sparse_vertex_write` — ordering: a
   write before the rebuild corrupts the sparse overlay.
7. `completions_walk_the_topology_chain_before_leftover_rebuilds_install` —
   leftover rebuilds must not install over a newer topology.
8. `a_same_topology_completion_installs_its_rebuild_before_commit`.
9. `every_terminal_failure_exit_raises_the_dialog_and_disarms` — a failure that
   leaves the brush armed.
10. `a_sculpt_commit_revokes_the_alignment_measured_against_the_old_mesh` — a
    stale fit left authoritative after geometry changed. This is a wrong-number
    path, the worst kind here.
11. `a_geometry_change_forgets_the_whole_fit`.
12. `a_result_the_operator_has_overtaken_is_never_applied` — a late result
    landing on newer state.
13. `late_measurement_cannot_reopen_hidden_or_unrefined_map`.
14. `a_measurement_with_no_summary_is_not_painted_on_the_scan`.
15. `dropping_a_stale_map_also_drops_the_work_behind_it`.
16. `measurement_requires_a_landed_refined_match`.
17. `a_warm_bvh_is_never_started_for_an_abandoned_worker` — wasted worker.
18. `worker_passes_its_cancellation_token_into_the_kernel`.
19. `terminal_finish_invariant_errors_stop_the_worker_command_loop`.
20. `densification_failure_is_not_silently_dropped`.
21. `sculpt_preparation_counts_as_busy_before_the_worker_exists`.
22. `the_stroke_baseline_is_snapshotted_cold`.
23. `the_offscreen_viewport_replays_overlay_vertices_after_scene_upload` — the
    measured colours vanish after a rebuild; the operator reads an unmeasured
    scan. app-level.
24. `a_failed_offscreen_frame_cannot_start_a_repaint_storm` — a dead GPU
    spinning the UI at 100%.
25. `a_readback_deadline_defers_the_offscreen_path_instead_of_killing_it` — a
    slow readback permanently disabling the viewport.
26. `a_frame_during_the_retry_wait_cannot_latch_the_offscreen_path_off`.
27. `the_graphics_fault_dialog_offers_the_retry_action` — RESTORES the test I
    added earlier; keep.
28. `a_fatal_notice_cannot_block_the_failure_exit` — a notice that stops the
    app from exiting.
29. `worker_entry_converts_panics_to_a_visible_failure` (both lanes list it) —
    a panic in a worker becoming silence.
30. `mapped_range_failure_cleans_up_before_the_error_can_return` — a leaked
    mapping/`memmap` held past the error.
31. `the_handoff_pipe_cannot_be_squatted_or_used_to_impersonate` — SECURITY:
    single-instance pipe impersonation. Highest value on this list.
32. `no_scan_path_reaches_the_crash_report` — a privacy leak in a crash dump.
33. `the_device_request_takes_its_buffer_ceiling_from_the_adapter` — a device
    request that ignores the adapter ceiling; can fail on real hardware.
34. `thumbnail_cli_uses_file_backed_render_path` — the CLI taking a different
    render path from the shell.
35. `a_readback_deadline...` (see 25) and `the_shader_is_told_the_width_the_field_was_packed_with` — shader/uniform
    mismatch, which renders a wrong image rather than failing.

## Priority 2 — correctness of interaction, worth covering (should cover)

36. `brush_hotkeys_survive_a_held_shift` — Shift+1/2. Cheap to test properly.
37. `sculpt_hotkeys_switch_to_sculpt_from_edit_mesh` — hotkey from the other tab.
38. `an_active_stroke_stops_sampling_when_pointer_leaves_viewport`.
39. `sculpt_cursor_follows_pointer_ownership_and_warm_pick_readiness`.
40. `a_click_that_turns_the_pair_around_invalidates_the_fit`.
41. `clicked_triangle_normals_stay_in_the_mesh_local_frame` — math, testable
    directly.
42. `fixed_pair_normals_use_the_inverse_transpose_for_scaled_instances` — math.
43. `arming_align_stands_the_other_tools_down`.
44. `removing_a_named_layer_revokes_refined_authority`.
45. `a_job_never_re_copies_geometry_that_has_not_changed` — the "heavy at"
    slider cost; a real performance contract.
46. `a_job_carries_the_identity_of_what_it_measures`.
47. `the_orientation_rule_is_disabled_while_a_fit_runs`.
48. `a_settings_change_abandons_a_running_fit_without_waiting_for_a_claim`.
49. `optimizer_setting_changes_drop_the_refined_authority`.
50. `every_heavy_call_goes_through_the_worker` — one blocking call freezes the
    UI.
51. `returning_to_automatic_does_not_measure_implicitly`.
52. `a_stroke_drops_the_map_instead_of_recomputing_it`.
53. `a_dab_reuses_the_cached_geometry_and_re_colours_only_what_it_touched`.
54. `a_dab_is_scoped_to_the_explicit_mesh_selection`.
55. `a_stroke_takes_its_direction_from_the_toggle_and_shift_together`.
56. `numeric_range_edits_recolour_the_cached_map`.
57. `the_map_is_locked_until_a_refined_match_lands`.
58. `an_overlay_never_touches_the_cpu_mesh`.
59. `showing_and_hiding_an_overlay_never_replaces_the_scene`.
60. `the_upload_buffer_is_repainted_not_rebuilt`.
61. `a_dab_uploads_only_the_vertices_it_touched`.
62. `a_rejected_sparse_overlay_upload_is_not_reported_as_success`.
63. `every_attached_overlay_says_what_it_is`.
64. `clearing_an_overlay_also_repairs_stale_display_bookkeeping`.
65. `opening_a_reading_clears_the_align_heatmap`.
66. `a_reading_marks_both_of_its_arches`.
67. `live_viewport_keeps_selection_overlay_separate_from_base_scene`.
68. `linux_window_identity_matches_desktop_metadata`.
69. `installer_refreshes_shell_association_cache_after_registry_changes` —
    the operator's file associations not updating after install.
70. `consumer_wheel_align_shift_resizes_once_from_horizontal_raw_event` —
    SURVIVED as a behavioural test; confirm and mark.

## Priority 3 — UI layout, labels, focus (cover cheaply; a real project does)

71-84. The `align_panel_*` window tests: movable window, cancel/done ends, no
control naming a target/role, labels, exclusion brush on the automatic tab,
history buttons on the manual tab, keyboard focus, accessible roles, compact
chips, the brush as its own window, no dynamic coverage line, the map's name,
no persistent diagnostic wall, editable cool/hot limits.

These were written as text assertions on the panel source. Their honest form is
a test over the panel's produced WIDGET LIST (egui `Ui` inspection), not over
text. That is a bigger change than the rest and is scoped as its own item.

## Priority 4 — deliberately NOT covered (record as a decision, not a debt)

85. Any item whose only content was "this constant appears in this file"
    (`worker_source_has_no_catalog_key_literals`, the doc-link checks).
    Replacing them with a test would recreate the same mistake.
86. `no_control_in_the_window_names_a_target_a_source_or_a_role` and the rest of
    the negative-UI family IF the widget-list harness from 71-84 is not built.
    A negative text assertion is the weakest form; a real check needs the
    harness. Listed so the gap is explicit rather than pretended away.

## The inconsistencies (17) — fix all, they are mechanical

87. `app_bootstrap_tests.rs`: a removed test's doc landed on an unrelated test.
88. `app_layer_interaction.rs`: a removed test's doc attached to the lasso test.
89. `cli/main.rs`: a removed helper's doc attached to a `use` import.
90. `texture.rs`: a removed helper's doc merged into a surviving test's doc.
91. `presentation_sinks.rs`: a doc link to a deleted helper.
92. `app_sculpt_worker.rs`: orphaned doc comment.
93. `app_align_results.rs`: orphaned/duplicated docs.
94. `app_align_brush.rs`: duplicated docs.
95. `align_panel_map.rs`: stale comment.
96. `app_sculpt_tests.rs`: stale `allow` reason.
97. `align_panel_tests.rs`: stale doc on a surviving test.
98. `shell_contract_tests.rs` / `installer_contract_tests.rs`: state whether the
    GUI/COM identity guards now live elsewhere or are genuinely gone.
99. `Commit claim vs the surviving tree`: the `bc4c393` message says "58 test
    functions across 24 files"; the real total across all four removal commits
    is 66 across 28. Correct the record in the ledger.
100. `.audit-ledger.md`: add this file's outcome so the ledger is not the only
     place the wave is described.

## Order of work

1. Fix 87-97 (mechanical, no test runs needed beyond compile).
2. Cover Priority 1 (1-35), one test per behaviour, running the suite.
3. Cover Priority 2 (36-70).
4. Priority 3 (71-84) only if the widget-list harness is small; otherwise
   record it as item 86 does.
5. Update the ledger (98-100).
6. Independent verification of all of the above.
7. Build Windows MSI + Debian deb with the HPS key.
8. Upload to the Nextcloud share.

---

# STATUS LEDGER (this wave)

Written against the pinned test environment `scripts/test-linux.sh` (Lavapipe
Vulkan + `OCCLUVIEW_REQUIRE_GPU_TESTS=1`), which makes the GPU suites execute
instead of dying against the ambient `DISPLAY=:99` or early-returning. A test
marked WRITTEN was proved able to fail by reverting the guarded production line
where that was practical (the layer-removal guard was reverted and the test
went red, then the tree was restored).

Legend: WRITTEN `<test>` = a behavioural test exists; COVERED `<test>` = an
existing test already exercised it; CANNOT `<reason>` = not testable here;
NOT COVERED = a deliberate decision.

## Priority 1

1. WRITTEN `switching_brush_mode_finishes_a_live_stroke_instead_of_aborting_it`
   (pre-existing sibling; the behaviour was already covered — see 9).
2. WRITTEN `toggling_off_does_not_drop_a_worker_with_a_queued_finish`.
3. WRITTEN `abort_also_reverts_a_released_stroke_waiting_in_the_worker`.
4. WRITTEN `a_rejected_finish_keeps_the_stroke_for_a_later_retry`.
5. WRITTEN `worker_loss_invalidates_an_active_sculpt_stroke`.
6. WRITTEN `a_layer_rebuild_is_installed_before_any_sparse_vertex_write`
   (app-level; the worker-internal ordering is `rebuild_supersedes_queued_sparse_updates`).
7. WRITTEN `completions_walk_the_topology_chain_before_leftover_rebuilds_install`.
8. WRITTEN `a_same_topology_completion_installs_its_rebuild_before_commit`.
9. WRITTEN `every_terminal_failure_exit_raises_the_dialog_and_disarms`.
10. WRITTEN `a_sculpt_commit_revokes_the_alignment_measured_against_the_old_mesh`.
11. WRITTEN `a_geometry_change_forgets_the_whole_fit`.
12. WRITTEN `a_result_the_operator_has_overtaken_is_never_applied`.
13. WRITTEN `late_measurement_cannot_reopen_hidden_or_unrefined_map`.
14. WRITTEN `a_measurement_with_no_summary_is_not_painted_on_the_scan`.
15. WRITTEN `dropping_a_stale_map_also_drops_the_work_behind_it`.
16. WRITTEN `measurement_requires_a_landed_refined_match`.
17. WRITTEN `an_abandoned_preparation_never_installs_its_session` (the
    abandoned-worker case; the warm-BVH cost itself has no observable seam, so
    the test pins the operator-visible contract).
18. WRITTEN `worker_passes_its_cancellation_token_into_the_kernel`.
19. WRITTEN `terminal_finish_invariant_errors_stop_the_worker_command_loop`.
20. WRITTEN `densification_failure_is_not_silently_dropped` (via a `#[cfg(test)]`
    failure-injection seam; the path is unreachable from a well-formed mesh).
21. WRITTEN `sculpt_preparation_counts_as_busy_before_the_worker_exists`.
22. WRITTEN `the_stroke_baseline_is_snapshotted_cold`.
23. WRITTEN `the_offscreen_viewport_replays_overlay_vertices_after_scene_upload`
    — and the vacuous-skip path now fails under `OCCLUVIEW_REQUIRE_GPU_TESTS`.
24. WRITTEN `a_failed_offscreen_frame_cannot_start_a_repaint_storm`.
25. WRITTEN `a_readback_deadline_defers_the_offscreen_path_instead_of_killing_it`.
26. WRITTEN `a_frame_during_the_retry_wait_cannot_latch_the_offscreen_path_off`.
27. WRITTEN `the_graphics_fault_dialog_offers_the_retry_action`.
28. WRITTEN `a_fatal_notice_cannot_block_the_failure_exit`.
29. WRITTEN `worker_entry_converts_panics_to_a_visible_failure` (sculpt lane,
    via a `#[cfg(test)]` panic trigger at the real `catch_unwind` boundary;
    contact lane was already `a_panicking_worker_latches_a_failure_and_stops_looking_busy`).
30. CANNOT `mapped_range_failure_cleans_up_before_the_error_can_return` —
    `offenders::read_back_extent` already calls `unmap()` before returning on
    every error branch, but the failure needs a real wgpu buffer whose
    `get_mapped_range()` fails, which cannot be forced without a device-level
    fault-injection seam. The cleanup is read directly in
    `crates/occluview-render/src/offscreen/helpers.rs:194-234`.
31. CANNOT `the_handoff_pipe_cannot_be_squatted_or_used_to_impersonate` —
    Windows-only (single-instance named pipe). A Windows test would open the
    pipe first and assert the app refuses to impersonate; blocked here because
    the crate does not build on Linux.
32. WRITTEN `a_crash_report_never_carries_a_scan_path`.
33. WRITTEN `the_device_request_takes_its_buffer_ceiling_from_the_adapter`.
34. WRITTEN `a_real_mesh_on_disk_renders_a_real_thumbnail` (black-box CLI).
35. WRITTEN `the_shader_is_told_the_width_the_field_was_packed_with` and
    `the_uniform_carries_the_field_width_it_was_given`.

## Priority 2

36. WRITTEN `brush_hotkeys_survive_a_held_shift`.
37. WRITTEN `sculpt_hotkeys_switch_to_sculpt_from_edit_mesh`.
38. WRITTEN `an_active_stroke_stops_sampling_when_pointer_leaves_viewport`.
39. WRITTEN `sculpt_cursor_waits_for_a_warm_pick_before_sampling` (the
    readiness half; ownership is the same gate as 38).
40. WRITTEN `a_click_that_turns_the_pair_around_invalidates_the_fit`.
41. WRITTEN `clicked_triangle_normals_stay_in_the_mesh_local_frame`.
42. WRITTEN `fixed_pair_normals_use_the_inverse_transpose_for_scaled_instances`.
43. WRITTEN `arming_align_stands_the_other_tools_down`.
44. WRITTEN `removing_a_named_layer_revokes_refined_authority` (proved red on
    revert of `forget_removed_align_layers`).
45. COVERED `unchanged_geometry_is_handed_out_without_being_rebuilt`
    (`align_geometry.rs`; the Arc reuse contract).
46. COVERED `only_the_settings_that_change_the_distances_change_the_key`.
47. WRITTEN `the_orientation_rule_is_disabled_while_a_fit_runs`.
48. WRITTEN `a_settings_change_abandons_a_running_fit_without_waiting_for_a_claim`.
49. WRITTEN `optimizer_setting_changes_drop_the_refined_authority`.
50. NOT COVERED — `every_heavy_call_goes_through_the_worker` is architectural;
    there is no call that could be asserted as "the only one". The funnel
    (`submit_align_job`) and the worker (`execute`) are private with no
    behavioural seam; a test would only restate the module layout.
51. WRITTEN `returning_to_automatic_does_not_measure_implicitly`.
52. WRITTEN `a_stroke_drops_the_map_instead_of_recomputing_it`.
53. CANNOT `a_dab_reuses_the_cached_geometry_and_re_colours_only_what_it_touched`
    as stated at the GPU level; the CPU half is COVERED by
    `the_upload_buffer_is_repainted_not_rebuilt` (the scratch array is reused)
    and `each_side_keeps_its_own_touched_list`. The GPU-side "re-colours only
    what it touched" needs a prepared scene and a populated sparse write.
54. CANNOT `a_dab_is_scoped_to_the_explicit_mesh_selection` as a GPU test;
    COVERED at the mask level by `marking_one_scan_leaves_the_other_untouched`
    and `mesh_selection_is_explicit_and_survives_role_swaps_by_physical_mesh`.
55. WRITTEN `a_stroke_takes_its_direction_from_the_toggle_and_shift_together`.
56. WRITTEN `numeric_range_edits_recolour_the_cached_map` and
    `a_recolour_is_refused_when_the_measurement_identity_changed`.
57. WRITTEN `measurement_requires_a_landed_refined_match` (same gate; the panel
    is COVERED by `manual_tab_never_authorizes_a_heatmap_from_stale_readiness`).
58. WRITTEN `an_overlay_never_touches_the_cpu_mesh`.
59. WRITTEN `showing_and_hiding_an_overlay_never_replaces_the_scene`.
60. WRITTEN `the_upload_buffer_is_repainted_not_rebuilt`.
61. CANNOT `a_dab_uploads_only_the_vertices_it_touched` at the GPU level;
    COVERED at the scratch level by `the_upload_buffer_is_repainted_not_rebuilt`
    (only the touched index changes) and `each_side_keeps_its_own_touched_list`.
62. WRITTEN `a_rejected_sparse_overlay_upload_is_not_reported_as_success` and
    `a_malformed_sparse_overlay_write_does_not_mutate_the_scratch`.
63. WRITTEN `every_attached_overlay_says_what_it_is`.
64. WRITTEN `clearing_an_overlay_also_repairs_stale_display_bookkeeping`.
65. WRITTEN `opening_a_reading_clears_the_align_heatmap`.
66. WRITTEN `a_reading_marks_both_of_its_arches`.
67. CANNOT `live_viewport_keeps_selection_overlay_separate_from_base_scene` at
    the app level; the render crate owns it and
    `prepared_viewport_can_draw_selection_overlay_after_base_scene`
    (`occluview-render/tests/prepared_scene.rs`) covers it against a real
    device.
68. WRITTEN `linux_window_identity_matches_desktop_metadata` and
    `linux_window_identity_matches_the_installed_appstream_entry`.
69. CANNOT `installer_refreshes_shell_association_cache_after_registry_changes`
    — Windows-only (COM/registry). A Windows test would install, change a
    registration, and assert `SHChangeNotify` fired.
70. WRITTEN `a_shifted_horizontal_wheel_notch_resizes_the_brush_once` (survived;
    confirmed by grep).

## Priority 3

71-84. PARTIAL/WRITTEN. The AccessKit harness from `a0c6701` was extended:
`the_exclusion_brush_is_offered_on_the_automatic_tab_only` and a new
`the_orientation_rule_is_disabled_while_a_fit_runs` read the produced widget
tree. The remaining layout/label items (movable window, cancel/done, no control
naming a target/role, keyboard focus, accessible roles, compact chips, the
brush as its own window, the map's name) are NOT COVERED: the panel-level
AccessKit node for this window does not expose per-control disabled/label state
reliably on this egui version, and a text assertion would recreate the mistake
that was removed. The honest gap is recorded, not faked.

## Priority 4

85. NOT COVERED — deliberate. "This constant appears in this file" and doc-link
    checks were the removed class; replacing them recreates it.
86. See 71-84: the negative-UI family is NOT COVERED for want of a reliable
    harness.

## Inconsistencies

87, 88, 91, 97. ALREADY FIXED before this wave by the cleanup commits; verified
    in the tree (the doc now sits on its correct test).
89, 90, 92, 93, 94, 95, 96. FIXED this wave (orphaned/duplicated/stale docs
    removed; `app_sculpt_tests.rs` allow reason corrected).
98. RESOLVED: the GUI/COM identity guards that read the crate's own `.rs` are
    gone. The value-agreement half is restored as a real cross-artifact test,
    `windows_app_identity_value_matches_the_shipped_shortcut`
    (`APP_USER_MODEL_ID` vs `install/occluview.wxs`), which runs on the Windows
    CI job. The Linux half is the real viewport-builder test
    `linux_window_identity_matches_desktop_metadata`.
99. CORRECTED in `.audit-ledger.md`: the removal total is recorded honestly
    (the individual commit subjects overstate their own share).
100. DONE in `.audit-ledger.md`.

## Additional findings this wave

- The `git grep repo_source_file|include_str!("*.rs")` count (14) UNDERCOUNTS
  the self-text family: it misses tests that read a crate's own `.rs` through
  `std::fs::read_to_string`. An independent read-only audit found seven more
  pure self-text tests, and they were removed this wave:
  `source_tree::every_source_file_is_named_by_a_module_declaration`,
  `source_tree::no_source_file_carries_a_path_from_one_machine`,
  `presentation_sinks::presentation_sinks_route_through_catalogs`,
  `camera::camera_module_is_split_by_responsibility_not_single_file`,
  `scene::scene_module_is_split_by_responsibility_not_single_file`,
  `scene::scene_bbox_uses_mesh_bbox_cache_for_repaint_safety`,
  `shell_preview_tests::preview_scene_is_split_by_responsibility_not_single_file`,
  plus their now-dead `source_file` helpers and the `presentation_sinks`
  prose-scanning machinery. The synthetic walker test
  `source_tree::the_orphan_guard_recognises_both_module_layouts` was kept: it
  writes temporary `.rs` fixtures and tests the walker itself.
- After the removal the repo-`.rs` grep is 11 hits across 4 files, all genuine
  cross-artifact contracts (README↔i18n/keys, MSI/`.wxs`/`.reg`↔registration
  code). `crates/occluview-render`'s shader-text tests read `.wgsl`, a
  production asset, and are not part of the family.
- `cargo test -p occluview-render` under the ambient `DISPLAY=:99` SIGSEGVs;
  `scripts/test-linux.sh` pins the Lavapipe ICD and unsets the display, which is
  what CI does. It also sets `OCCLUVIEW_REQUIRE_GPU_TESTS=1`, which turns the
  previously vacuous no-adapter early-return in
  `the_offscreen_viewport_replays_overlay_vertices_after_scene_upload` into a
  failure, so a green suite means the GPU tests actually ran.

## PLY / texture audit (this pass)

Three read-only audit lanes covered PLY specifically (in-file texture), the
full import→export colour matrix, and the docs' own acceptance claims. What was
fixed, and what is honestly still open:

FIXED this pass:

- `an_image_past_the_decoders_limits_is_not_written` — the PLY writer used to
  accept any texture whose base64 fit under `max_encoded_chars`, but the reader
  additionally enforces `MAX_TEXTURE_DIMENSION_PX` (8192) and
  `MAX_TEXTURE_RGBA_BYTES` (256 MiB). A lopsided atlas compressed small, was
  written and reported as success, and re-opened with no texture. The writer now
  asks the shared validator (`texture_decode::validate_texture_dimensions`) so
  the two cannot drift; the drop routes through `TextureImageNotWritten`.
- `a_payload_past_the_ceiling_stops_accumulating` — the header parser
  concatenated every `OccluViewTextureBase64` chunk into one `String` and only
  applied the ceiling afterwards, so a near-1 GB header forced a second
  full-size allocation before rejection. `TextureComments.encoded_too_long`
  now stops accumulation at the ceiling and releases what was held.
- `the_embedded_keys_are_matched_case_insensitively` — `TextureFile` was already
  case-insensitive; `OccluViewTextureFormat`/`Base64` were not, so a re-cased
  export silently lost its texture. All three now match the same way.
- `a_textured_dcm_saved_as_ply_keeps_its_colour_inside_the_file` and
  `a_vertex_coloured_dcm_saved_as_ply_keeps_its_vertex_colour` — the operator's
  own case, end to end: a textured and a vertex-coloured `.dcm` (HPS) opened
  through dispatch, written as one `.ply`, then read back with the pixels and
  the per-vertex bytes unchanged and with exactly one file left behind.
- `a_face_element_with_no_property_is_refused_instead_of_spinning` — a binary
  PLY whose `face` element declares rows but no property consumed no bytes per
  row and looped forever on the same cursor state
  (`ply/binary.rs::read_faces`). The CI fuzz smoke lane was dying exactly here:
  its 13977-file cached corpus replay never finished inside the 20-minute job
  timeout, and the last log line was the corpus load, not a crash. A huge count
  in a malformed header is enough to trigger it, so the viewer and the Explorer
  thumbnail host could be wedged by one crafted file. Fixed by refusing the
  shape, mirroring what the ASCII reader already did; proved red by removing the
  guard.

STILL OPEN (recorded, not faked; all pre-existing, none introduced here):

- `has_uvs()` is value-derived (`any(|v| v.uv != [0,0])`,
  `occluview-core/src/mesh/mod.rs:181,218,270`), so a mesh whose whole mapping
  is exactly (0,0) reports no UVs and both the reader
  (`ply/mod.rs:141`) and the writer (`write/ply.rs:52`) drop its texture. A
  correct fix needs a "UV data was declared" flag threaded through the loaders,
  not another value guess. NOT COVERED.
- STL per-vertex/attribute colour is neither read (`stl/binary.rs:105` ignores
  the 2-byte trailer and the 80-byte header) nor written, and there is no
  import-side warning type at all, so a colour-bearing dental STL re-saved to
  PLY loses the colour with no signal. NOT COVERED.
- glTF `COLOR_0` with normalized `UNSIGNED_SHORT` (5123) is rejected outright
  (`gltf/accessor.rs:180-183`); glTF 2.0 permits it. NOT COVERED.
- Multi-primitive/multi-material glTF keeps one texture silently
  (`gltf/scene.rs:71-111` inspects only `primitives.first()`). NOT COVERED.
- A foreign PLY with a same-stem sibling image but no `TextureFile` comment gets
  no texture; `same_stem_image` is reached only from the OBJ branch
  (`companions.rs:77`). NOT COVERED.
- Reader-side texture loss is always silent (`ply/mod.rs:166-186`); the write
  side warns. Deliberate (geometry-only opening is right), but recorded.
- OFF/COFF colour is not read (`off.rs:96-113,216-232`), and the CLI
  `occluview-hps-export` PLY artifact is geometry-only by design
  (`hps/mesh.rs:42-53` passes no UVs/texture) with its `MeshWriteReport`
  discarded; the coloured PLY route is the viewer's layer export, which is what
  the two new tests exercise.

The PLY writer was independently verified to never emit `comment TextureFile`
and never write a second file: every writer call site
(`app_scene_export.rs:143,353`, `app_mesh_export.rs:266`, CLI) writes exactly
one `.ply`, and both the unit and round-trip tests assert the directory holds
that file alone.



## HPS/DCM key delivery (this pass)

Verified with an independent read-only audit plus a real encrypted scan:

- The deb workflow (`package-msi.yml`, `linux-package`) was dispatched on
  `3d88adf` with `release_dry_run=false`, which builds Windows MSI and the Linux
  deb with the real `OCCLUVIEW_HPS_EMBEDDED_KEY` and skips the `publish` job —
  no GitHub Release. Run `35926082512`.
- The key reaches the binary only through `OUT_DIR/embedded_hps_key.rs`
  (`occluview-hps/build.rs:28-37`, `src/key.rs:155-158`), is XOR-obfuscated
  (build.rs:82-92), and the generated module is untracked. The feature gate
  `private-hps-key` is wired for the deb at `install/linux/build-deb.sh:115-118`
  and for the MSI at `install/build-msi.ps1:424-430`.
- The embedded key wins over the environment (`key.rs:145-152`); the env
  fallbacks are `OCCLUVIEW_HPS_ENCRYPTION_KEY`, `OCCLUVIEW_HPS_KEY`,
  `OCCLUTRACE_HPS_ENCRYPTION_KEY`, `HPS_ENCRYPTION_KEY`.
- `HpsSecretKey`'s `Debug` redacts to `"<redacted>"` (`key.rs:56-62`) and the
  type is `Zeroize + ZeroizeOnDrop`.
- Differential proof on the real corpus: the CI-built deb opens
  `/home/wow/test_scans/upper.dcm` (272252 vertices) and `lower.dcm` (168586
  vertices), thumbnails `upper.dcm` to a valid PNG, and converts
  `upper.dcm`→`.ply`; the local deb built with a placeholder key fails both at
  `CE vertex data integrity check failed`. So the secret is genuinely in this
  deb and genuinely absent from a placeholder build.

FIXED: `key::tests::runtime_provider_reads_generated_embedded_key_when_present`
used `assert_eq!` on the key byte slices, which prints both operands on failure.
It runs inside the packaging jobs where one operand is the real private key, and
the workflow's log scan only matches the literal secret string — a CSV-form
secret would have been printed as a byte array and missed. Now compares without
formatting, so a mismatch fails the build without putting material in the log.

STILL OPEN (recorded): `install/linux/build-deb.sh` does not itself require the
secret, so a hand-run build with an empty environment still produces a public,
no-key package. Only the workflow step enforces it (`package-msi.yml:363-366`).
The metainfo description does not qualify HPS support as "official builds only".

## Colour loss on a save-format mismatch (this pass)

FIXED: `representable_export_format` (`app_mesh_export.rs`) only second-guessed
STL for a point cloud. A `.dcm`/HPS has no writer, so "keep the source format"
cannot apply and the stored fallback decides; with the fallback set to STL, a
scan captured in colour was proposed as `.stl`, and the export then dropped the
atlas, the per-vertex colours and the mapping, warning only after the operator
had picked a name. The proposal is now PLY whenever the layer carries a texture,
vertex colours or a mapping, and STL only for a layer that carries none of them
(so a geometry-only scan keeps the operator's choice).

Proved red on revert: `a_colour_dcm_is_proposed_as_ply_even_when_the_fallback_is_stl`
fails with `left: StlBinary, right: PlyBinaryLittleEndian` when the old
condition is restored; `a_forced_stl_falls_back_to_ply_so_colour_is_not_thrown_away`
covers texture / vertex-colour / mapping separately.

NOTE: this makes the *proposal* correct, and the write path already warns via
`MeshWriteWarning::VertexColorsNotWritten` / `TextureImageNotWritten` /
`UvsNotWritten`. It does not stop an operator who deliberately types `.stl` for
a colour scan; that write still succeeds with a warning, which is the existing
documented contract for an explicit choice.
