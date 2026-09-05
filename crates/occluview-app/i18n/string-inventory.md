# String inventory (HEAD 056b455, `crates/occluview-app/src`)

Method: `grep -c '"'` per file + widget/call-site sampling. Test files
counted separately — they pin current English and will churn on migration.

## Scale

- ~4395 lines with quoted strings, ~4374 quoted literals total in app src.
- `format!` call sites cluster: `whole_mesh.rs` (14), `align_panel_map.rs`
  (12), `app_bootstrap.rs` (10), `app_align_results.rs` (10),
  `app_scene_export.rs` / `app_mesh_export.rs` (8 each) — first candidates
  for named-variable messages (no positional concatenation).
- egui constructors: `Button::new` ×23, `Window::new` ×11, `Label::new`
  ×11 — each needs a resource key; window/widget IDs stay untranslated.

## Hot real-UI surfaces (non-test, by literal density)

`interaction_hints.rs` (109), `repair_report.rs` (98), `align_panel.rs`
(88), `app_mesh_export.rs` (81), `align_panel_map.rs` (79),
`mesh_editor_groups.rs` (76), `layers_overlay/menu.rs` (72),
`app_align_results.rs` (67), `app_dialogs.rs` (63),
`app_settings_panel.rs` (59).

## Classification

- **UI (localize, Phase C→D):** chrome/toolbar/empty-state, settings
  language selector + About + update dialog (vertical slice first),
  dialogs/help, loading/export/update/errors, layers/repair/mesh-editor,
  align/cut/measure panels, tooltips, accessible names, transient status.
- **Errors (localize wrapper, keep payload):** user-facing explanation +
  action buttons translated; paths, hashes, diagnostic IDs, support
  copy-blocks stay raw English data.
- **CLI (stay English):** `occluview-cli` println!/eprintln!/usage text —
  machine-oriented, locale-invariant by policy.
- **Protocol (stay English):** `single_instance/protocol.rs` (69 quoted
  lines — wire format), `--version`, IPC, crash-report schema, COM/registry
  IDs (`occluview-shell`), JSON persisted schema.
- **License (stay as-is):** third-party notices shown verbatim.
- **Tests:** `primary_ui_tests/*` (~1000 quoted lines) pin English UI —
  migrate to key-aware assertions during Phase D; geometry/layout safety
  tests must not weaken.

## Exceptions approved (do-not-translate log)

File extensions/format labels (STL/PLY/OBJ/GLB/HPS), CLI flags, JSON keys,
COM/registry IDs, logs/stack traces, license text, FDI notation, numeric
geometry, technical support payloads. Each future exception needs a row
here + reviewer sign-off.
