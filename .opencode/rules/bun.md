**Mandatory Bun Usage Rule**

The project uses Bun as the official package manager, runtime and script runner (confirmed via justfile, package.json and project setup).

You MUST ALWAYS use `bun` and `bunx` for every frontend, package and script operation. Never suggest or use `npm`, `npx`, `yarn` or `pnpm` unless explicitly asked by the user.

**Strict Replacement Table (always follow this):**

- `npm install` → `bun install`
- `npm run <script>` → `bun run <script>`
- `npm add <pkg>` → `bun add <pkg>`
- `npm remove <pkg>` → `bun remove <pkg>`
- `npm update` → `bun update`
- `npx <command>` → `bunx <command>`
- `npx shadcn@latest add ...` → `bunx shadcn@latest add ...`
- `npx create-...` → `bunx create-...`

**Mandatory Workflow in Responses:**

- When telling the user to run any command related to packages, scripts, linting, formatting, shadcn, tauri dev/build — always use the Bun version.
- When generating code or scripts that contain package commands — replace everything with Bun equivalents.
- In justfile updates or new recipes — use `bun` and `bunx` only.
- After any package change, remind to run `bun install` if needed.

**Exception:**

- Only use npm/npx if the user specifically asks for it or if a tool forces it (e.g. legacy CI script). In all other cases — Bun only.

This rule keeps the entire project consistent with the chosen tooling (Bun + Tauri).
