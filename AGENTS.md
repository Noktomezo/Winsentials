# AGENTS.md

## Project Overview

**Winsentials** — Windows 10/11 desktop app. One-click system tuning via registry, COM, service control. Every tweak: typed frontend/backend contract (`apply`, `revert`, state, backup).

## Tech Stack

| Layer | Technology |
| --- | --- |
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
| Backend | Rust (Tauri commands) |
| Window effects | `window-vibrancy` |
| Notifications | Sonner |

## Folder Structure

- Frontend: Feature-Sliced Design (FSD)
- Backend: Vertical Slice Design

## Dependency Rules

### Frontend (src)

- **Runtime:** `bun` only. Never `npm`/`pnpm`/`node`.
- Install: `bun add <pkg>` / `bun add -d <pkg>`. Run: `bunx <tool>` / `bun run <script>`.
- Commit `bun.lock` only. Never `package-lock.json` or `pnpm-lock.yaml`.

### Backend (src-tauri)

- Add deps: `cargo add <crate>` (never edit `Cargo.toml` by hand). With features: `--features <f1>,<f2>`. Then `cargo check`.
- **rayon** mandatory-by-default for CPU-heavy or bounded independent IO/status work across many items (tweak statuses, registry scans, file parsing, metadata building).
- Keep `rayon` out of: strict-order, shared mutable state, UI-thread affinity, non-thread-safe COM/Win32, global process settings, service-control sequences, or where parallelism amplifies load/side effects.
- For Tauri commands: wrap blocking work in `tauri::async_runtime::spawn_blocking`, use `rayon` inside only when per-item work is independent.
- Prefer sequential when: tiny collection, already async, or parallelism makes error handling/rollback less predictable.

### Tauri

- Tauri v2 APIs only. Register commands in `lib.rs` via `invoke_handler(generate_handler![...])`. Use `#[tauri::command]` on all handlers.

## Core Priorities

Performance, reliability, predictability under load/failures. When trading off, choose correctness over convenience.

## Maintainability

Extract shared logic to modules. No duplicate logic across files. Change existing code; don't add local shortcuts.

## Codebase Navigation — `@colbymchenry/codegraph`

**MANDATORY:** Use codegraph for all codebase navigation, symbol discovery, relationship analysis. Generic grep discouraged unless searching raw literal strings.

```bash
rtk bunx --bun @colbymchenry/codegraph init # first time
rtk bunx --bun @colbymchenry/codegraph index # re-index after edits
rtk bunx --bun @colbymchenry/codegraph query <symbol> # find definitions/usages
rtk bunx --bun @colbymchenry/codegraph context "<task>" # structured markdown for a feature
rtk bunx --bun @colbymchenry/codegraph status # graph health
```

Workflow: explore with `query`/`context` before reading files → re-index after edits → trace deps with `query` before modifications.

## RTK — Token-Optimized Commands

**Always prefix shell commands with `rtk`** (60-90% context savings, zero behavior change, passthrough if no filter).

- Chain: `rtk git add . && rtk git commit -m "msg"`
- Debugging: raw command without `rtk`
- `rtk proxy <cmd>` — no filtering, tracks usage

## Post-Task Checks

Run after every task. Do not skip.

### Frontend (format → typecheck → dead-code → audit)

```bash
rtk bun run format # eslint --fix (eslint-stylistic replaces Prettier)
rtk bun run typecheck # tsc --noEmit, zero errors
rtk bunx fallow@latest # zero issues
rtk bunx react-doctor@latest  # UI health
```

### Backend (fmt → clippy → check)

```bash
rtk cargo fmt
rtk cargo clippy --fix --allow-dirty --allow-staged --all-targets -- -D warnings
rtk cargo check
```

<!-- gortex:communities:start -->
<!-- gortex:skills:start -->
## Community Skills

| Area | Description | Skill |
|------|-------------|-------|
| Src Tauri Src Tweaks Appearance 17 Dirs | 304 symbols | `/gortex-src-tauri-src-tweaks-appearance-17-dirs` |
| Src Pages Home Ui 5 Dirs | 287 symbols | `/gortex-src-pages-home-ui-5-dirs` |
| Src Tauri Src Tweaks Appearance 11 Dirs | 229 symbols | `/gortex-src-tauri-src-tweaks-appearance-11-dirs` |
| Src Shared Ui 8 Dirs | 172 symbols | `/gortex-src-shared-ui-8-dirs` |
| Src Tauri Src 6 Dirs | 159 symbols | `/gortex-src-tauri-src-6-dirs` |
| Src Tauri Src Startup 1 Dirs Startupentry | 142 symbols | `/gortex-src-tauri-src-startup-1-dirs-startupentry` |
| Src Tauri Src Tweaks Input Snapshot Dword | 141 symbols | `/gortex-src-tauri-src-tweaks-input-snapshot-dword` |
| Src Tauri Src Tweaks Privacy New | 92 symbols | `/gortex-src-tauri-src-tweaks-privacy-new` |
| Src Tauri Src Tweaks Context Menu Read String Or Missing | 66 symbols | `/gortex-src-tauri-src-tweaks-context-menu-read-string-or-missing` |
| Src Tauri Src Startup Entry Details | 64 symbols | `/gortex-src-tauri-src-startup-entry-details` |
| Src Features Theme Switcher Ui 9 Dirs | 63 symbols | `/gortex-src-features-theme-switcher-ui-9-dirs` |
| Src Tauri Src Tweaks Context Menu 3 Dirs | 61 symbols | `/gortex-src-tauri-src-tweaks-context-menu-3-dirs` |
| Src Tauri Src Backup 1 Dirs | 59 symbols | `/gortex-src-tauri-src-backup-1-dirs` |
| Src Tauri Src Tweaks Appearance 6 Dirs | 57 symbols | `/gortex-src-tauri-src-tweaks-appearance-6-dirs` |
| Src Tauri Src System Info 1 Dirs Gpuinfo | 56 symbols | `/gortex-src-tauri-src-system-info-1-dirs-gpuinfo` |
| Src Entities Startup Model | 50 symbols | `/gortex-src-entities-startup-model` |
| Src Shared Ui Scrollto | 45 symbols | `/gortex-src-shared-ui-scrollto` |
| Src Shared Ui 2 Dirs Sidebarprovider | 44 symbols | `/gortex-src-shared-ui-2-dirs-sidebarprovider` |
| Src Tauri Src Commands 3 Dirs | 42 symbols | `/gortex-src-tauri-src-commands-3-dirs` |
| Src Tauri Src Startup Action Info | 41 symbols | `/gortex-src-tauri-src-startup-action-info` |
<!-- gortex:skills:end -->

<!-- gortex:communities:end -->

<!-- rtk-instructions v2 -->
# RTK (Rust Token Killer) - Token-Optimized Commands

## Golden Rule

**Always prefix commands with `rtk`**. If RTK has a dedicated filter, it uses it. If not, it passes through unchanged. This means RTK is always safe to use.

**Important**: Even in command chains with `&&`, use `rtk`:
```bash
# ❌ Wrong
git add . && git commit -m "msg" && git push

# ✅ Correct
rtk git add . && rtk git commit -m "msg" && rtk git push
```

## RTK Commands by Workflow

### Build & Compile (80-90% savings)
```bash
rtk cargo build         # Cargo build output
rtk cargo check         # Cargo check output
rtk cargo clippy        # Clippy warnings grouped by file (80%)
rtk tsc                 # TypeScript errors grouped by file/code (83%)
rtk lint                # ESLint/Biome violations grouped (84%)
rtk prettier --check    # Files needing format only (70%)
rtk next build          # Next.js build with route metrics (87%)
```

### Test (60-99% savings)
```bash
rtk cargo test          # Cargo test failures only (90%)
rtk go test             # Go test failures only (90%)
rtk jest                # Jest failures only (99.5%)
rtk vitest              # Vitest failures only (99.5%)
rtk playwright test     # Playwright failures only (94%)
rtk pytest              # Python test failures only (90%)
rtk rake test           # Ruby test failures only (90%)
rtk rspec               # RSpec test failures only (60%)
rtk test <cmd>          # Generic test wrapper - failures only
```

### Git (59-80% savings)
```bash
rtk git status          # Compact status
rtk git log             # Compact log (works with all git flags)
rtk git diff            # Compact diff (80%)
rtk git show            # Compact show (80%)
rtk git add             # Ultra-compact confirmations (59%)
rtk git commit          # Ultra-compact confirmations (59%)
rtk git push            # Ultra-compact confirmations
rtk git pull            # Ultra-compact confirmations
rtk git branch          # Compact branch list
rtk git fetch           # Compact fetch
rtk git stash           # Compact stash
rtk git worktree        # Compact worktree
```

Note: Git passthrough works for ALL subcommands, even those not explicitly listed.

### GitHub (26-87% savings)
```bash
rtk gh pr view <num>    # Compact PR view (87%)
rtk gh pr checks        # Compact PR checks (79%)
rtk gh run list         # Compact workflow runs (82%)
rtk gh issue list       # Compact issue list (80%)
rtk gh api              # Compact API responses (26%)
```

### JavaScript/TypeScript Tooling (70-90% savings)
```bash
rtk bun add <pkg>        # Add dependency
rtk bun add -d <pkg>     # Add dev dependency
rtk bun run <script>     # Compact bun script output
rtk bunx <cmd>           # Compact bunx command output
rtk prisma              # Prisma without ASCII art (88%)
```

### Files & Search (60-75% savings)
```bash
rtk ls <path>           # Tree format, compact (65%)
rtk read <file>         # Code reading with filtering (60%)
rtk grep <pattern>      # Search grouped by file (75%). Format flags (-c, -l, -L, -o, -Z) run raw.
rtk find <pattern>      # Find grouped by directory (70%)
```

### Analysis & Debug (70-90% savings)
```bash
rtk err <cmd>           # Filter errors only from any command
rtk log <file>          # Deduplicated logs with counts
rtk json <file>         # JSON structure without values
rtk deps                # Dependency overview
rtk env                 # Environment variables compact
rtk summary <cmd>       # Smart summary of command output
rtk diff                # Ultra-compact diffs
```

### Infrastructure (85% savings)
```bash
rtk docker ps           # Compact container list
rtk docker images       # Compact image list
rtk docker logs <c>     # Deduplicated logs
rtk kubectl get         # Compact resource list
rtk kubectl logs        # Deduplicated pod logs
```

### Network (65-70% savings)
```bash
rtk curl <url>          # Compact HTTP responses (70%)
rtk wget <url>          # Compact download output (65%)
```

### Meta Commands
```bash
rtk gain                # View token savings statistics
rtk gain --history      # View command history with savings
rtk discover            # Analyze Claude Code sessions for missed RTK usage
rtk proxy <cmd>         # Run command without filtering (for debugging)
rtk init                # Add RTK instructions to CLAUDE.md
rtk init --global       # Add RTK to ~/.claude/CLAUDE.md
```

## Token Savings Overview

| Category | Commands | Typical Savings |
|----------|----------|-----------------|
| Tests | vitest, playwright, cargo test | 90-99% |
| Build | next, tsc, lint, prettier | 70-87% |
| Git | status, log, diff, add, commit | 59-80% |
| GitHub | gh pr, gh run, gh issue | 26-87% |
| Package Managers | pnpm, npm, npx | 70-90% |
| Files | ls, read, grep, find | 60-75% |
| Infrastructure | docker, kubectl | 85% |
| Network | curl, wget | 65-70% |

Overall average: **60-90% token reduction** on common development operations.
<!-- /rtk-instructions -->
