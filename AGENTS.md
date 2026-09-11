# AGENTS.md

Guidance for AI coding agents working in this repository. `CLAUDE.md` carries
the same rules in more depth; the guides below are normative.

## What this repo is

GPUI Kit — a Rust desktop UI framework built on GPUI (https://gpui-kit.com).

- `crates/base` (`gpui-base`) — unstyled behavior and infrastructure: focus, overlays, dock layout, semantic tokens.
- `crates/component` (`gpui-component`) — the styled component library (60+ desktop components).
- `crates/shell` (`gpui-shell`) — JavaScript scripting runtime for a Rust host; git-only deps, excluded from workspace tests.
- `crates/kit` (`gpui-kit`) — umbrella crate apps depend on; re-exports GPUI at its root and pins a `gpui-pre-*` snapshot. Apps never depend on GPUI directly.
- `crates/story` (`gpui-component-story`) — gallery app for showcasing/testing components; workspace default member.
- `examples/` — standalone example apps; `crates/story-web` — WASM story build; `website/` — Astro docs site (Bun).

GPUI comes from versioned `gpui-pre*` snapshot crates pinned in the workspace
`Cargo.toml`, not from the Zed git repo.

## Required reading before editing

- `website/docs/design-guides.md` and `website/docs/coding-guides.md` — normative design/coding guides, not optional inspiration.
- `docs/ARCHITECTURE.md` and `docs/STYLING-AND-MOTION.md` — internal architecture specs (not published to the site).
- Do not infer the design system from one existing screen or add a control merely because the feature exists.

## Commands

```bash
cargo run                                                            # story gallery (default member)
cargo clippy -p gpui-component -p gpui-component-story -p gpui-kit-assets -- --deny warnings
cargo fmt --check                                                    # edition 2024
typos                                                                # spell check (CI gate)
cargo machete                                                        # unused dependencies
cargo check -p gpui-component --no-default-features                  # tree-sitter-off build must compile
cargo test --workspace --exclude gpui-shell --features gpui-component-story/test-support
cargo test -p gpui-component --doc
```

Tests are not required for pure visual/sizing tweaks; add them when the change
affects behavior, interaction, data flow, or prevents a meaningful regression.
For manual UI verification, run the story app and use accessibility-driven
interaction (`docs/ACCESSIBILITY-UI-TESTING.md`).

## Layer rules

- Never modify `crates/base` unless the user explicitly requests a Base-layer change. Implement component behavior and styling in `crates/component` or the application layer.
- Base is no-style: no colors, sizing, padding, radius, borders, or animation in base controls; presentation belongs above the seam. (Sanctioned exception: the base Input frame's semantic 1px border and radius baseline.)
- Dock: layout truth lives in `crates/base/src/dock` (`LayoutTree`, `DockArea`); `crates/component/src/dock` is the `DockSkin` presentation layer. A panel implements both the base `Panel` (behavior) and the component `Panel` (presentation).
- Theme APIs expose semantic tokens (color, spacing, radius, typography, shadows), never per-component style fields. Access via `ActiveTheme` (`cx.theme()`).
- Public data types crossing the seam keep fields private, are built with builders, and are read through methods — adding a `pub` field is a breaking change. See "Public Data Types Across the Seam" in `docs/ARCHITECTURE.md`.

## App wiring gotchas

- Call `gpui_component::init(cx)` at the application entry point before using any component.
- The first-level view of every window must be a `Root` (owns sheets, dialogs, notifications, keyboard navigation).

## Code style

- Prefer stateless `RenderOnce` components. Sizes via `Sizable` (`xs`/`sm`/`md`/`lg`), default `md`.
- Keep GPUI builder chains fluent: use `when`/`when_some`/`when_none`/`map` instead of mutable temporaries and imperative reassignment.
- Buttons use the `default` mouse cursor, not `pointer` (desktop convention), unless it is a link button.
- No new enum type names ending in `Kind`; name the type after what its variants are (`NodeKind` is grandfathered).
- Spell `Context` out in type names (`ComboboxTriggerContext`, not `…Ctx`); `cx` is reserved for GPUI's `App`/`Context<T>`/`AsyncApp`.
- Workspace clippy denies `dbg_macro` and `todo`; style lints are relaxed (see `[workspace.lints]` in `Cargo.toml`).

## PRs and commits

- One PR does one thing; keep the diff minimal, no drive-by refactors or formatting.
- Match existing title style: `<area>: lowercase summary` (e.g. `input: preserve CRLF cursor boundaries`) or `chore:`/`docs:`/`ci:` — do not default to `feat:`/`fix:`.
- Mark AI-generated portions. If a PR changes the public API of `crates/component`, add a `## Breaking Changes` section with `diff` blocks showing old and new usage.

## Docs, i18n, icons

- Website docs are bilingual: edit `website/docs/` (en) and `website/zh-CN/docs/` together.
- `skills/gpui-kit/references/coding-guides.md` and `skills/gpui-kit-design-guides/references/design-guides.md` are vendored copies of the English guides. Edit `website/docs/`, then copy across (`cp website/docs/<guide>.md skills/<skill>/references/<guide>.md`) — CI fails if they drift. Never edit the copies directly.
- Locales: `crates/component/locales/` via rust-i18n; only add `en`, `zh-CN`, `zh-HK`.
- `Icon` does not bundle SVGs by default. The default icon set lives in `crates/assets/assets/icons/`; the `IconName` enum is generated from those files by the `icon_named!` macro (one PascalCase variant per SVG name) — add a Lucide-style SVG there to add an icon.

## Platforms

macOS (aarch64, x86_64), Linux (x86_64), Windows (x86_64) — CI tests all three, so keep changes cross-platform. Website dev: `bun run --cwd website dev`; WASM story: `make dev-web`.
