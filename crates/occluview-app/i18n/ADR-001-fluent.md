# ADR-001: Fluent для runtime-каталогов OccluView

Status: accepted (pilot ru/de; CJK-волна после font-spike).
Date: 2026-09-05. Scope: `occluview-app` only.

## Candidates

- **Fluent** (`fluent-bundle 0.16` + `fluent-syntax 0.12` + `unic-langid 0.9`,
  `sys-locale 0.3` for OS detection): pure Rust, named variables,
  per-locale plural/select (русские one/few/many — из коробки),
  build-time parseable `.ftl`, `cargo info` MSRV 1.64–1.67 (< project
  MSRV 1.98), license MIT/Apache-2.0 ∈ `deny.toml` allow-list.
- **gettext** (Crate `gettext 0.4` / `tr 0.1` + PO): требует либо linkage
  к C-библиотеке libgettext (кросс-компиляция Windows MSI, static CRT,
  toolchain-риск — reject), либо PO-макросы без именованных переменных и
  со слабым select; review-workflow через msgmerge вне Git-привычек;
  plural для русского — ручные формы без контрактной проверки атрибутов.
- **Hand-rolled JSON + ICU**: `icu_*` уже транзитивно в дереве, но тянет
  Unicode-данные и собственный формат плюралов; никакой выгоды против
  Fluent при равном объёме своего кода. Rejected.

## Decision

Fluent. Каталоги `.ftl` в репозитории (`crates/occluview-app/i18n/`),
вкомпилированы через `include_str!`, валидация паритета с `en` — build
test (parse + key/attribute/variable contract). Fallback — per-message на
`en`, never mixed-language frame. Pseudo-locale `qps-ploc` генерируется
из `en` на этапе валидации. TMS/Weblate/SaaS — deferred; драфты только
машинный драфт из allowlisted static source + glossary.

## Consequences

- operational mode: versioned catalogs in repo, compiled into binary,
  no runtime downloads (update-installer story unchanged);
- `sys-locale` только как один из injected OS sources (тесты инжектят
  fake sources);
- CJK-шрифты и сабсеттинг — отдельный spike перед волной 2
  (default_fonts покрывает пилот, доказать тестом в Phase B).
