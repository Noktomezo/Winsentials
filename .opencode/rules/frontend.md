**Mandatory Shadcn/UI Component Rule**

You are using shadcn/ui as the official design system and component library.

For ANY low-level / primitive UI component (button, input, label, card, dialog, dropdown-menu, table, tooltip, separator, badge, etc.) you MUST follow this strict hierarchy. Never build a low-level component from scratch without checking first.

**Mandatory Workflow (exact order):**

1. **First Priority — Shadcn MCP**
   - Always start here.
   - Use shadcn MCP tools (search_component, get_component, list_components, etc.) to check if the exact component (or very close variant) already exists.

2. **Second Priority — Local Project Folder**
   - If nothing found in Shadcn MCP, immediately check the local codebase with DocFork MCP or direct file search.
   - Look specifically in: `src/shared/ui/` (the official shadcn installation folder).

3. **Create New Component ONLY as Last Resort**
   - A brand-new low-level UI component may be created **only** if it is absent from both Shadcn MCP and the `src/shared/ui/` folder.
   - When creating:
     - Run `npx shadcn@latest add <component-name>` whenever possible (preferred method).
     - If manual creation is required, strictly follow shadcn/ui conventions:
       • Use `cn()` utility for class merging
       • `React.forwardRef` + full TypeScript support
       • Built on Radix UI primitives
       • Full accessibility (aria-\*, role, etc.)
       • Consistent styling with existing design tokens
     - Place the new file directly into `src/shared/ui/`

**Additional Rules:**

- Reusing an existing component (even with small modifications via props or wrapper) is always preferred over creating a duplicate.
- After adding any new component, immediately save it to Knowledge Graph (entityType: `ui_component`).
- In every response where UI code is generated, explicitly state which step of the workflow was used.

This rule guarantees zero duplication, perfect consistency with shadcn/ui, and maximum reuse across the entire project.
