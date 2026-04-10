# AGENTS.md

## Project Overview

**Winsentials** is a desktop application for Windows 10/11 that allows users to tune system settings with just one click.

The app exposes a clean, friendly UI on top of low-level OS operations (registry edits, COM commands, service control, etc.). Every tweak has a typed contract on both frontend and backend (apply, revert, get current state, backup mechanism, etc.)

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri v2 |
| Frontend runtime | Bun |
| Build tool | Vite |
| UI framework | React 19 |
| Language | TypeScript (strict) |
| Styling | TailwindCSS v4 |
| Component library | shadcn/ui |
| Routing | TanStack Router |
| State management | Zustand |
| i18n | i18next + react-i18next |
| Backend language | Rust (Tauri commands) |
| Window effects | `window-vibrancy` crate |
| Notifications | Sonner (toast) |

## Folder Structure

- Frontend — Feature-Sliced Design (FSD)
- Backend — Vertical Slice Design

## Dependency & Runtime Rules

### Frontend

- **Runtime:** `bun` only. Never use `npm`, `pnpm`, `node` directly.
- Install packages: `bun add <pkg>`
- Dev packages: `bun add -d <pkg>`
- Run scripts: `bunx <tool>` or `bun run <script>`
- Do **not** commit `package-lock.json` or `pnpm-lock.yaml` — only `bun.lockb`
- Treat window materials as modifiers, not standalone themes. `acrylic`, `mica`, and `tabbed` must stay modeled via `data-webview-material` on the root element together with the existing `light`/`dark` theme values in `data-theme`; never replace the theme key with a material value.
- When changing theme or appearance code, preserve the invariant that theme selection (`light`/`dark`/`system`) and material selection (`none`/`acrylic`/`mica`/`tabbed`) are combined, not merged into one field.

### Backend (Rust)

- Add dependencies: `cargo add <crate>` — never manually edit `Cargo.toml` version strings
- When adding a crate with features: `cargo add <crate> --features <feat1>,<feat2>`
- After adding deps, always run `cargo check` to verify the build compiles

### Tauri

- Use Tauri v2 APIs — do not use v1 patterns (different plugin system, command registration, etc.)
- Register all commands in `lib.rs` via `tauri::Builder::default().invoke_handler(tauri::generate_handler![...])`
- Use `tauri::command` macro on all public Rust handlers

## Post-Task Checks

Run these after **every** completed task before considering it done. Do not skip even for "small" changes.

### Frontend

Order matters — format first so typecheck sees clean code:

```bash
# 1. Fix formatting and lint errors
bun run format
# fallback if script not available:
bunx eslint --fix .

# 2. Type check — must pass with zero errors
bun run typecheck
# fallback:
bunx tsc --noEmit
```

> `eslint-stylistic` is used for formatting — it replaces Prettier. `bun run format` runs `eslint --fix`, not a separate formatter.

### Backend

Order matters — fmt before clippy so clippy sees formatted code; check after clippy fix to confirm the build is clean:

```bash
# 1. Format
cargo fmt

# 2. Lint + auto-fix what's fixable
cargo clippy --fix --allow-dirty --allow-staged

# 3. Verify the build compiles cleanly
cargo check
```
