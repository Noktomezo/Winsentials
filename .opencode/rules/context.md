**Context Gathering Priority Rule (MCP Tools)**

You have access to specialized MCP servers for retrieving context and external/project information.
You MUST follow this strict hierarchical order every time you need any additional context, documentation, code references, technical details, examples, or research.

**Mandatory Workflow:**

1. **First Priority: DocFork MCP**
   - ALWAYS start with DocFork MCP.
   - Use its tools (e.g. search_repo, read_file, get_docs, etc.) to fetch information directly from the current project codebase, documentation, or linked repositories.
   - Purpose: project-specific, up-to-date, internal context (features, architecture, existing code, READMEs, etc.).

2. **Second Priority: Context7 MCP** (only if DocFork gave no relevant results or information is insufficient)
   - Switch to Context7 MCP only after confirming DocFork returned empty/irrelevant output.
   - Use its tools for deeper or specialized context loading (long documents, version history, extended project knowledge).
   - Purpose: when DocFork is not enough but the information is still project-related or semi-internal.

3. **Third Priority: Exa Search MCP** (only as last resort)
   - Use Exa Search MCP ONLY if both DocFork and Context7 returned insufficient or no useful information.
   - Craft high-precision search queries.
   - Purpose: general web knowledge, external library docs, best practices, latest updates, or anything outside the project scope.

**Additional Rules:**

- Never skip steps or jump directly to Exa Search.
- After each tool call, evaluate the results: if they fully answer the need → stop and use the data. If not → proceed to the next tool.
- Always combine and synthesize information from the tools used.
- Store all valuable findings (features, decisions, code patterns, references) immediately into the Knowledge Graph using the Memory MCP tools.
- In your final response clearly indicate which MCP sources were used (e.g. “Context gathered from DocFork → Context7”).

This rule guarantees the most relevant, project-first context while avoiding unnecessary web noise.
