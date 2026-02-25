**Mandatory Post-Task Strictening Rule**

You MUST enforce strict project hygiene after completing **every single task** (new feature, bug fix, refactor, improvement, chore, documentation change, etc.). This is non-negotiable.

You have access to the project’s justfile with pre-defined recipes for formatting and linting.

**Mandatory Workflow (exact order):**

1. **Formatting** (always first)
   - Run: `just format`
   - This executes `format-back` (cargo fmt + clippy --fix) and `format-front` (bun run format).

2. **Linting & Static Analysis** (always second)
   - Run: `just lint`
   - This executes:
     • `lint-back` → cargo check + cargo clippy (backend)
     • `lint-front` → bun run lint + react-doctor (frontend)
     • `opengrep scan` (project-wide)

3. **Final Verification & Fixes**
   - If any issues are found, fix them immediately.
   - Re-run `just lint` until it passes cleanly (zero errors, zero warnings where possible).
   - For any backend changes (recommended): `cargo check --manifest-path=./src-tauri/Cargo.toml`

**Completion Criteria:**

- A task is considered 100% complete **only** after `just format` and `just lint` both finish successfully with a clean output.
- Never mark a task as “Done”, “Completed” or “Ready for review” until the user confirms the commands passed cleanly.

**Response Requirements:**

- At the end of every implementation response, add a dedicated section exactly like this:

**Project Strictening**
After applying the changes, please run in order:

1. just format
2. just lint
   Confirm the output is clean before committing.

- If you generate code, always remind the user to run the strictening commands.

This rule guarantees the project stays clean, consistent and maintainable at all times (Rust backend + TypeScript frontend).
