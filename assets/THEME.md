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
| background | `#EFF3F6` | Main app background     |
| card       | `#F5F8F9` | Card / elevated surface |
| popover    | `#F5F8F9` | Dropdown / overlay      |
| secondary  | `#E0E7EB` | Subtle fill / muted bg  |
| border     | `#D4DDE2` | Borders and dividers    |
| input      | `#C3CED5` | Input field background  |

### Text

| Token            | Hex       | Role           |
| ---------------- | --------- | -------------- |
| foreground       | `#0B1014` | Primary text   |
| muted-foreground | `#647682` | Secondary text |

### Accents

| Token       | Hex       | Usage      |
| ----------- | --------- | ---------- |
| primary     | `#275ABE` | CTA, links |
| destructive | `#B82E2E` | Errors     |
| success     | `#2EB860` | Success    |
| warning     | `#C5A420` | Warnings   |

### Badges

| Token         | Hex       |
| ------------- | --------- |
| badge-blue    | `#275ABE` |
| badge-cyan    | `#29A9BC` |
| badge-yellow  | `#C5A420` |
| badge-red     | `#B82E2E` |
| badge-purple  | `#6A2EB8` |
| badge-magenta | `#BA2C8A` |

## Dark Theme — Arctic Midnight

### Surfaces

| Token      | Hex       | Role                    |
| ---------- | --------- | ----------------------- |
| background | `#0B1014` | Main app background     |
| card       | `#0F151A` | Card / elevated surface |
| popover    | `#131B20` | Dropdown / overlay      |
| secondary  | `#131B20` | Subtle fill / muted bg  |
| border     | `#182025` | Borders and dividers    |
| input      | `#131B20` | Input field background  |

### Text

| Token            | Hex       | Role           |
| ---------------- | --------- | -------------- |
| foreground       | `#E9EEF1` | Primary text   |
| muted-foreground | `#95A6B1` | Secondary text |

### Accents

| Token       | Hex       | Usage      |
| ----------- | --------- | ---------- |
| primary     | `#6A90DC` | CTA, links |
| destructive | `#D77070` | Errors     |
| success     | `#70D795` | Success    |
| warning     | `#E2C965` | Warnings   |

### Badges

| Token         | Hex       |
| ------------- | --------- |
| badge-blue    | `#6A90DC` |
| badge-cyan    | `#6CCCDA` |
| badge-yellow  | `#E2C965` |
| badge-red     | `#D77070` |
| badge-purple  | `#9C70D7` |
| badge-magenta | `#D86EB5` |

## Principles

1. **Blue undertone everywhere** — neutrals use slate/navy base, never warm gray
2. **High contrast accents** — vivid saturated colors for badges and semantic states
3. **Fluent-compatible modifiers** — works with acrylic, mica, and tabbed window modifiers
4. **No warm grays** — the entire palette stays in cool territory

Materials such as acrylic or mica are modifiers applied on top of existing light/dark themes, not standalone themes.
