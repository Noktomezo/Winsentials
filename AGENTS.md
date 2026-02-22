# Winsentials

## DESCRIPTION

A desktop app for Windows 10/11 optimization, being done by "tweaks".

## KNOWLEDGE GRAPH MEMORY MCP (Model Context Protocol)

The project uses **Knowledge Graph Memory** via MCP as the single source of persistent, long-term memory.
This memory survives across sessions, tool restarts, model changes, and different machines.

### Core Principles

1. **Always query the graph first**
   Search the graph before any architectural decision, code change or recommendation.

2. **Store only high-value information**
   - Architectural decisions + rationale
   - Domain rules and constraints
   - Component relationships and dependencies
   - Known issues + current status
   - Key naming conventions

3. **Structure**
   - **Entity**: CamelCase (unique and clear)
   - **Relation**: UPPER_SNAKE_CASE
   - **Observation**: concise factual sentence in present tense

### Workflow

- Search first → decide → update graph
- After every meaningful change or decision — immediately add/update observations/relations

**Memory Priority**

1. Knowledge Graph (highest truth)
2. Current codebase
3. Current conversation
4. General knowledge

Never bypass or ignore the Knowledge Graph.

## TECH STACK

- Tauri
- React (with TypeScript)
- TailwindCSS
- ShadcnUI
- Zustand (w/ persist middleware)
- TanStack Router
- Zod
- `i18next` and `react-i18next`

## RULES

1. CONTEXT RETRIEVAL
   - Always use `Context7 MCP` first for library/API docs, code generation and configuration.

2. CONTEXT RETRIEVAL FALLBACK
   - If Context7 didn’t provide the needed information, use `Exa MCP`.

3. JS/TS RUNTIME & PACKAGE MANAGER
   - Always use `bun` and `bunx`. Never use `npm` or `npx`.

4. STRICTNESS
   - After each task:
     - Frontend: `bun run typecheck` + `bun run format`
     - Rust: `cargo clippy` + `cargo check` + `cargo fmt`
     - Final check: `opengrep scan`

5. FRONTEND FOLDER STRUCTURE (./src/)
   - Follow Feature-Sliced Design (FSD).

6. BACKEND FOLDER STRUCTURE (./src-tauri)
   - Keep clean and logically categorized.

7. OPERATIONAL DIRECTIVES
   - Execute the request immediately and precisely.
   - Zero fluff. Stay concise.
   - Prioritize code and visual solutions.

8. DESIGN PHILOSOPHY: INTENTIONAL MINIMALISM
   - Reject generic templates.
   - Create unique, purposeful, asymmetrical interfaces.
   - Every element must have a clear reason to exist.

9. FRONTEND CODING STANDARDS
   - **Critical**: Always use ShadcnUI / Radix primitives. Do not build low-level components from scratch.

10. RESPONSE FORMAT
    1. **Rationale:** (one short sentence)
    2. **The Code.**

## RUSSIAN PLURALIZATION (FOR `i18next`)

Russian uses 3 main plural categories (CLDR: one / few / many + other as fallback):

- one: numbers ending in 1 but not 11 (1, 21, 31, 101, ...)
- few: numbers ending in 2,3,4 but not 12,13,14 (2–4, 22–24, 32–34, ...)
- many: everything else (0, 5–20, 25–30, 100, 111, ...)

Use this structure in JSON:

```json
{
  "key_one": "...",
  "key_few": "...",
  "key_many": "...",
  "key_other": "..." // usually same as many
}
```

Example for "disk":

```json
{
  "disk_one": "disk",
  "disk_few": "disks (few form)",
  "disk_many": "disks (many form)",
  "disk_other": "disks"
}
```
