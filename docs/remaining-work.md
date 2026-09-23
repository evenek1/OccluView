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
