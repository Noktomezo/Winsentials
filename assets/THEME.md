# Arctic — Winsentials Color Palette

A cold, modern palette inspired by snow and midnight skies. Designed for Fluent/Material-style desktop apps with intentional blue tints throughout both themes.

## Design Direction

- **Light theme (Arctic Snow):** Cool blue-white surfaces like fresh snow under a clear sky. Blue-gray neutrals instead of warm grays.
- **Dark theme (Arctic Midnight):** Deep navy-black backgrounds reminiscent of a polar night. All neutrals carry a blue undertone.
- **Accents:** Vivid, saturated colors that pop against both cold surfaces. No muted earth tones.

## Light Theme — Arctic Snow

### Surfaces

| Token      | Hex       | Role                    |
| ---------- | --------- | ----------------------- |
| background | `#EEF2F7` | Main app background     |
| card       | `#F7F9FC` | Card / elevated surface |
| popover    | `#F7F9FC` | Dropdown / overlay      |
| secondary  | `#DFE5EE` | Subtle fill / muted bg  |
| border     | `#CDD5E0` | Borders and dividers    |
| input      | `#C4CDD9` | Input field background  |

### Text

| Token            | Hex       | Role           |
| ---------------- | --------- | -------------- |
| foreground       | `#0C1425` | Primary text   |
| muted-foreground | `#64748B` | Secondary text |

### Accents

| Token       | Hex       | Usage      |
| ----------- | --------- | ---------- |
| primary     | `#2563EB` | CTA, links |
| destructive | `#DC2626` | Errors     |
| success     | `#16A34A` | Success    |
| warning     | `#D97706` | Warnings   |

### Badges

| Token         | Hex       |
| ------------- | --------- |
| badge-blue    | `#2563EB` |
| badge-cyan    | `#0891B2` |
| badge-yellow  | `#D97706` |
| badge-red     | `#DC2626` |
| badge-purple  | `#7C3AED` |
| badge-magenta | `#DB2777` |

## Dark Theme — Arctic Midnight

### Surfaces

| Token      | Hex       | Role                    |
| ---------- | --------- | ----------------------- |
| background | `#0B1120` | Main app background     |
| card       | `#111827` | Card / elevated surface |
| popover    | `#1E293B` | Dropdown / overlay      |
| secondary  | `#1E293B` | Subtle fill / muted bg  |
| border     | `#293548` | Borders and dividers    |
| input      | `#1E293B` | Input field background  |

### Text

| Token            | Hex       | Role           |
| ---------------- | --------- | -------------- |
| foreground       | `#CBD5E1` | Primary text   |
| muted-foreground | `#94A3B8` | Secondary text |

### Accents

| Token       | Hex       | Usage      |
| ----------- | --------- | ---------- |
| primary     | `#3B82F6` | CTA, links |
| destructive | `#EF4444` | Errors     |
| success     | `#22C55E` | Success    |
| warning     | `#F59E0B` | Warnings   |

### Badges

| Token         | Hex       |
| ------------- | --------- |
| badge-blue    | `#3B82F6` |
| badge-cyan    | `#06B6D4` |
| badge-yellow  | `#F59E0B` |
| badge-red     | `#EF4444` |
| badge-purple  | `#8B5CF6` |
| badge-magenta | `#EC4899` |

## Principles

1. **Blue undertone everywhere** — neutrals use slate/navy base, never warm gray
2. **High contrast accents** — vivid saturated colors for badges and semantic states
3. **Fluent-compatible modifiers** — works with acrylic, mica, and tabbed window modifiers
4. **No warm grays** — the entire palette stays in cool territory

Materials such as acrylic or mica are modifiers applied on top of existing light/dark themes, not standalone themes.
