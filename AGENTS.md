# AGENTS.md

## Project Overview

**Winsentials** desktop app for Windows 10/11. Tune system settings with one click.

App exposes clean UI over low-level OS ops (registry edits, COM commands, service control, etc.). Every tweak has typed contract on frontend + backend (`apply`, `revert`, current state, backup, etc.)

## Tech Stack

| Layer             | Technology              |
| ----------------- | ----------------------- |
| Desktop shell     | Tauri v2                |
| Frontend runtime  | Bun                     |
| Build tool        | Vite                    |
| UI framework      | React 19                |
| Language          | TypeScript (strict)     |
| Styling           | TailwindCSS v4          |
| Component library | shadcn/ui               |
| Routing           | TanStack Router         |
| State management  | Zustand                 |
| i18n              | i18next + react-i18next |
| Backend language  | Rust (Tauri commands)   |
| Window effects    | `window-vibrancy` crate |
| Notifications     | Sonner (toast)          |

## Folder Structure

- Frontend: Feature-Sliced Design (FSD)
- Backend: Vertical Slice Design

## Dependency & Runtime Rules

### Frontend

- **Runtime:** `bun` only. Never use `npm`, `pnpm`, `node`.
- Install packages: `bun add <pkg>`
- Dev packages: `bun add -d <pkg>`
- Run scripts: `bunx <tool>` or `bun run <script>`
- Never commit `package-lock.json` or `pnpm-lock.yaml`. Use `bun.lock` only.

### Backend (Rust)

- Add dependencies: `cargo add <crate>`. Never edit `Cargo.toml` version strings by hand.
- When adding a crate with features: `cargo add <crate> --features <feat1>,<feat2>`
- After adding deps, run `cargo check`

### Backend Performance

- When adding or changing Rust code that collects independent data from many items, consider `rayon` mandatory-by-default. Use `rayon` when the work is CPU-heavy or bounded independent IO/status work, such as reading many tweak statuses, scanning many registry values, parsing many files, or building many independent metadata objects.
- Keep `rayon` out of code that depends on strict order, shared mutable state, UI-thread affinity, non-thread-safe COM/Win32 objects, global process settings, service-control sequences, or operations where parallelism can amplify system load or side effects.
- For Tauri commands, do not rely on `rayon` alone for responsiveness. Wrap blocking backend work in `tauri::async_runtime::spawn_blocking`, then use `rayon` inside that blocking task only when the per-item work is independent.
- Prefer a small, direct sequential implementation when the collection is tiny, the operation is already asynchronous, or the added parallelism would make error handling or rollback behavior less predictable.

### Tauri

- Use Tauri v2 APIs. Do not use v1 patterns (plugin system, command registration, etc.)
- Register all commands in `lib.rs` via `tauri::Builder::default().invoke_handler(tauri::generate_handler![...])`
- Use `tauri::command` macro on all public Rust handlers

## Post-Task Checks

Run after every task. Do not skip, even for small changes.

### Frontend

Order matters: format first so typecheck sees clean code:

```bash
# 1. Fix formatting and lint errors
bun run format
# fallback if script not available:
bunx eslint --fix .

# 2. Type check — must pass with zero errors
bun run typecheck
# fallback:
bunx tsc --noEmit

# 3. Dead-code check (fallow) — must pass with zero issues
bunx fallow --only dead-code

# 4. React Doctor audit (ensure UI health)
bunx react-doctor --full --json-compact
# or:
bunx react-doctor --full --compact-json
```

> `eslint-stylistic` handles formatting. It replaces Prettier. `bun run format` runs `eslint --fix`, not separate formatter.

### Backend

Order matters: `fmt` before `clippy`; run `check` after `clippy` fix to confirm clean build:

```bash
# 1. Format
cargo fmt

# 2. Lint + auto-fix what's fixable
cargo clippy --fix --allow-dirty --allow-staged

# 3. Verify the build compiles cleanly
cargo check
```
