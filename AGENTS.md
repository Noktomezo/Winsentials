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

### Frontend (src)

- **Runtime:** `bun` only. Never use `npm`, `pnpm`, `node`.
- Install packages: `bun add <pkg>`
- Dev packages: `bun add -d <pkg>`
- Run scripts: `bunx <tool>` or `bun run <script>`
- Never commit `package-lock.json` or `pnpm-lock.yaml`. Use `bun.lock` only.

### Backend (src-tauri)

- Add dependencies: `cargo add <crate>`. Never edit `Cargo.toml` version strings by hand.
- When adding a crate with features: `cargo add <crate> --features <feat1>,<feat2>`
- After adding deps, run `cargo check`
- When adding or changing Rust code that collects independent data from many items, consider `rayon` mandatory-by-default. Use `rayon` when the work is CPU-heavy or bounded independent IO/status work, such as reading many tweak statuses, scanning many registry values, parsing many files, or building many independent metadata objects.
- Keep `rayon` out of code that depends on strict order, shared mutable state, UI-thread affinity, non-thread-safe COM/Win32 objects, global process settings, service-control sequences, or operations where parallelism can amplify system load or side effects.
- For Tauri commands, do not rely on `rayon` alone for responsiveness. Wrap blocking backend work in `tauri::async_runtime::spawn_blocking`, then use `rayon` inside that blocking task only when the per-item work is independent.
- Prefer a small, direct sequential implementation when the collection is tiny, the operation is already asynchronous, or the added parallelism would make error handling or rollback behavior less predictable.

### Tauri

- Use Tauri v2 APIs. Do not use v1 patterns (plugin system, command registration, etc.)
- Register all commands in `lib.rs` via `tauri::Builder::default().invoke_handler(tauri::generate_handler![...])`
- Use `tauri::command` macro on all public Rust handlers

## Core Priorities

1. Performance first.
2. Reliability first.
3. Keep behavior predictable under load and during failures (session restarts, reconnects, partial streams).

If a tradeoff is required, choose correctness and robustness over short-term convenience.

## Maintainability

Long-term maintainability is a core priority. If you add new functionality, first check if there is shared logic that can be extracted to a separate module. Duplicate logic across multiple files is a code smell and should be avoided. Don't be afraid to change existing code. Don't take shortcuts by just adding local logic to solve a problem.

## Codebase Navigation & Intelligence

**MANDATORY:** All codebase navigation, exploration, symbol discovery, and relationship analysis MUST be performed using `@colbymchenry/codegraph`. Generic file searches or regex greps are discouraged unless searching for raw literal strings not indexed as symbols.

### Why codegraph?
`codegraph` parses the entire codebase (both Rust & TypeScript) to build a semantic graph of functions, components, types, and files. This allows instantly finding usages, definitions, and dependencies across language boundaries without flooding the context window with raw text.

### Usage & Commands
Always bypass the unsafe Node version guard since the codebase uses modern runtimes:

```bash
# 1. Re-index codebase (run after modifying code/files)
bunx --bun @colbymchenry/codegraph index

# 2. Query specific symbol (find where functions, types, components are defined/used)
bunx --bun @colbymchenry/codegraph query <symbol_name>

# 3. Generate structured Markdown context for specific task or feature
bunx --bun @colbymchenry/codegraph context "<feature or task description>"

# 4. View indexing stats and verify graph health
bunx --bun @colbymchenry/codegraph status
```

### Workflow Rules for AI Agents:
1. **Explore First:** When starting new task, query relevant symbols or generate context markdown using `codegraph context` instead of manually reading multiple files.
2. **Post-edit Re-indexing:** After creating or modifying files, re-index using the `index` command to keep semantic graph current.
3. **Trace Dependencies:** Use `query` to understand which modules, components, or Tauri commands are impacted before making modifications.


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
