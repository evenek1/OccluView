# Implementation plan (status: wave-1 Latin scripts embedded)

- **Phase B — runtime foundation:** DONE.
- **Phase C — vertical slice:** DONE.
- **Phase D — full migration:** DONE (all in-app surfaces).
- **Phase E — catalogs:** DONE for `en, ru, de, es, fr, it, pt-BR`
  (612 keys each, contract-validated, all DRAFT). `zh-Hans, ja, ko`
  stay out until the font spike (proven missing by
  `i18n::fonts` tests: subset, licensing, Han unification, package
  size). `pl, tr, zh-Hant` wait for evidence of demand. RTL out.

- **Phase B — runtime foundation:** `i18n/` module in `occluview-app`:
  sidecar store, resolver + injected OS sources (+ `sys-locale` prod
  adapter), startup snapshot loaded once (no double `Settings::load`),
  embedded `en` (+ `qps-ploc`) resource manager, per-message `en`
  fallback, catalog validation test, native/egui title integration.
- **Phase C — vertical slice:** Settings language selector, About, one
  dynamic error, one update surface, title behavior, persistence/restart
  tests, pseudo + real-language layout tests.
- **Phase D — full migration:** chrome/toolbar/empty-state, dialogs/help,
  loading/export/update/errors, layers/repair/mesh-editor,
  align/cut/measure panels, a11y labels, transient status. Migrate
  `primary_ui_tests` to key-aware assertions; geometry/layout safety tests
  stay green and unweakened.
- **Phase E — catalogs:** DONE (see status above). Every catalog stays
  DRAFT until human terminology review + visual review → APPROVED.
- **Phase F — artifacts:** installed-package behavior (app catalogs work
  when installed), update/downgrade matrix, single-instance handoff
  invariance. No installer-localization claims.

## Acceptance matrix (STATUS.md tracks PASS/FAIL/NOT RUN/BLOCKED)

source/build/tests · catalog validation · visual UI · Windows package ·
Linux package · update/downgrade · fonts/IME · accessibility ·
terminology · clinical acceptance · privacy/scope.
Manual-only gates (screen-reader, IME, physical packages) are NOT RUN
with reason until executed — never inferred from green unit tests.
