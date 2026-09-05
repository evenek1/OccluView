# Message contract (Fluent, `occluview-app/i18n/*.ftl`)

- Semantic keys: `settings-language-auto`, `update-available-title`,
  `help-hint-navigation-orbit-the-camera`. Never English sentences as
  keys, and never dots inside keys: `.` is the `key.attribute` lookup
  separator, so a dotted key could never resolve (enforced by the
  `keys_never_contain_dots` test).
- Full messages + named variables: `open-error = Невозможно открыть {$path}`.
  No concatenated translated fragments, no positional `format!`.
- Every user-visible static string gets a key; dynamic data
  (path/file/layer names) stays untranslated interpolation data.
- Every non-`en` catalog MUST have exact key/attribute/variable parity
  with `en`. Parse, UTF-8, duplicate-key, placeholder, plural/select,
  markup errors fail `cargo test` validation. Unused keys fail too.
- Missing `en` key = build/CI failure. Missing non-`en`
  message/attribute/variable at runtime = that message falls back to `en`
  + non-sensitive diagnostic; no panic, no blank label, no raw key.
- One frame = one catalog generation: the active catalog never changes
  mid-frame, so two non-English languages can never mix. Individual
  messages may substitute their English text (with a diagnostic) — that
  per-message substitution is by design, not a mixed frame.
- Select-shape parity: a message that selects in `en` must select in
  every catalog (same number of select expressions; extra plural
  categories like ru `few` are welcome). Enforced by test.
- Widget/ComboBox/Window/grid IDs and persistent UI state are stable,
  independent of translated labels.
- Worker-thread statuses (align refusals) use positional `$a`/`$b`
  variables with per-key FTL comments: the worker has no locale, so
  details arrive pre-formatted. Everywhere else variables are named.
- Interpolated values carry Fluent bidi isolation marks (U+2068/U+2069).
  A select that IS the whole pattern isolates once (inner variable only);
  an embedded select isolates twice (whole placeable + inner variable).
  Tests pin the exact marks — never strip them to make strings "pretty".
- List joining (`", "` over already-localized parts, e.g. repair
  toast segments, grey-cause lists, digit pair lists, axis names) is an
  accepted documented pattern, not fragment concatenation: each part is
  a whole message, joined with a locale-neutral separator. ICU
  ListFormat stays a future option.
- Sentence-fragment interpolation (`$dropped`, `$settled`, `$weak`,
  `$surface`, `$blind` inside align status lines) is load-bearing
  technical debt, not a pattern to copy: every new sentence must be a
  whole message with per-variant predicates (see `align-status-measured`,
  where each plural variant carries its own predicate). Native review
  must check fragment word order in every locale before APPROVED.
- Pseudo-locale `qps-ploc` is generated from `en` (expansion + brackets)
  and must pass the same contract.
- Catalogs are DRAFT until native dental/CAD terminology review + visual
  UI review pass; only then APPROVED. Source edit invalidates approvals
  for touched messages.
