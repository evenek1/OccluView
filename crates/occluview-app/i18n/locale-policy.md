# Locale policy (exact, implements SKILL.md)

## Preference

`enum UiLanguagePreference { Auto, Explicit(Bcp47Tag) }`. Persist ONLY
`auto` or a validated canonical BCP-47 tag. Never a localized display
name, never the detected locale. Manual always beats Auto.

## Sidecar (update-safe persistence)

- File `ui-language-preference.json`, schema `{version: 1, preference}`.
- Lives beside existing app state (`app_state_dir()`: `%APPDATA%/OccluView`
  on Windows, `$XDG_STATE_HOME or ~/.local/state/OccluView` elsewhere) —
  outside the install dir, survives installer update, ignored by old
  binaries. NEVER a new field in `settings.json` (old binary wipes
  unknown fields on rewrite).
- Atomic write (temp + rename, same pattern as settings), retry/error UX
  mirrors settings persistence. Malformed file → safe fallback Auto/en +
  diagnostic, keeps remaining Settings intact, never overwrites a good
  file with empty/corrupt value.

## Auto resolution

Runs at cold launch, first launch after update, and explicit "Apply system
language now". Never mid-operation. Steps: ordered OS list (injected
source; `sys-locale` in prod) → canonicalize BCP-47 → per item try exact
catalog → approved base mapping → next item → `en`.

Mapping: `ru*→ru, de*→de, es*→es (incl. es-419), fr*→fr, it*→it,
pt-BR→pt-BR, pt/pt-PT/*→en (never pt-BR), zh-Hans*/zh-CN/zh-SG→zh-Hans,
zh-Hant*/zh-TW/zh-HK/zh-MO→en (never zh-Hans), ja*→ja, ko*→ko,
bare zh→en, C/POSIX/malformed/private-use→en`. `[pt-PT, de-DE]` → `de`.
Linux precedence: `LANGUAGE` (colon list) > `LC_ALL` > `LC_MESSAGES` >
`LANG`. No IP/geolocation. No guessed regional equivalence.

## Runtime

Manual switch re-renders all in-app UI immediately; the native title
follows live via `ViewportCommand::Title` on every switch (one-shot per
language generation at startup). Auto changes apply after
restart/explicit apply. Secondary instances only
forward open requests, never change primary's language. `--version`, CLI,
IPC, crash schema stay neutral.

## Update/downgrade

Catalogs ship embedded in the signed binary — no runtime downloads.
Manual sidecar survives update; Auto reevaluates at next launch; manual
stays manual. New catalog activates post-update only under Auto resolving
to it. Removed/unavailable explicit catalog → safe English, tag retained
for future restore. Old update UI may stay in old language until the
installer closes it. Sidecar load diagnostics (`Malformed`,
`InvalidValue`, `Unreadable`, `UnsupportedVersion`) are log-only by
design (`tracing::warn`); only save errors surface in the settings UI,
so a broken file can never block startup with a dialog. MSI downgrade stays blocked; portable downgrade must
not crash (may not understand new catalogs). No i18n change touches
signing, manifest, version compare, package identity, or instance
protocol. Installer/MSI/desktop-metadata localization is a separate scope.
