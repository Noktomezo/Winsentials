# Arclate

Arclate is a cool, restrained color system for native interfaces, dashboards, and data-dense tools. It pairs deep slate surfaces with frost-toned text and eight accents that remain crisp and recognizable across light and dark themes.

The palette is structured into:
- **Base Neutrals**: A 14-step monochromatic ladder for structural surfaces, interactive layers, borders, and text, centered around the Arctic Slate hue angle ($h \approx 235^\circ \dots 243^\circ$).
- **Main Accents**: Eight chromatic families with primary anchors at `400` (dark UI) and `600` (light UI), with `500` serving as the perceptual OKLCH midpoint.
- **Extended Palette**: Complete 13-tone gradations (`50`–`950`) for each family, enabling data visualizations, badges, charts, and fine-grained UI states.

---

## Principles

- **Elevation Ladder**: Surfaces, layers, and borders form an unambiguous elevation hierarchy without abrupt hue shifts.
- **Perceptual Balance**: Accents and neutrals are calibrated in the **Oklab / OKLCH** color space to maintain consistent contrast and saturation across both themes.
- **Anchor Consistency**: The `400` and `600` values act as fixed perceptual anchors; the `500` tone is the exact OKLCH midpoint.
- **Analog Restraint**: Inspired by the philosophy of [Flexoki](https://stephango.com/flexoki), Arclate uses subdued, natural pigments rather than hyper-saturated neon primaries.

---

## Base Colors

The Base color ladder follows the Arctic Slate trajectory from pure White (`#FFFFFF`) down to deep Canvas Black (`#0B1014`):

| Token | Hex | Light Theme Role | Dark Theme Role | Intended Use |
| :--- | :--- | :--- | :--- | :--- |
| `white` | `#FFFFFF` | `bg-2` (Elevated) | `tx` (Primary text) | Pure white card surface / frost text |
| `base-50` | `#F4F7F9` | `bg` (Canvas) | — | Light window canvas |
| `base-100` | `#EAF0F4` | `ui` (Interactive) | — | Light inputs, dropdown wells |
| `base-150` | `#DFE6EC` | `border-card` | — | Card container borders |
| `base-200` | `#DCE3E8` | `border-main` | — | Structural dividers and rules |
| `base-300` | `#CFDBE3` | `border-input` | — | Interactive input borders |
| `base-400` | `#A0B0BA` | `tx-3` (Faint) | — | Subtle indicators, faint text |
| `base-500` | `#748793` | — | `tx-2` (Muted) | Secondary muted text in dark mode |
| `base-600` | `#5A6F7C` | `tx-2` (Muted) | — | Secondary muted text in light mode |
| `base-700` | `#384852` | — | `tx-3` (Faint) | Dark comments, faint indicators |
| `base-800` | `#19242B` | — | `border-card` | Dark card container borders |
| `base-850` | `#162026` | — | `border-main` | Structural divider rules in dark mode |
| `base-900` | `#131B20` | — | `ui` (Interactive) | Dark inputs, dropdown wells, buttons |
| `base-950` | `#0F151A` | — | `bg-2` (Elevated) | Elevated cards, sidebar panels, titlebar |
| `black` | `#0B1014` | `tx` (Primary text) | `bg` (Canvas) | Darkest slate canvas / dark ink |

---

## Accent Colors

The main 16 accent tones used for interactive elements, status indications, charts, and syntax:

| Family | `400` · Dark UI | `500` · Midpoint | `600` · Light UI | Suggested Role |
| :--- | :--- | :--- | :--- | :--- |
| **Red** | `#D77070` | `#C85857` | `#B83E3E` | Errors, destructive actions, critical state |
| **Orange** | `#D79770` | `#C88057` | `#B8683E` | Attention, warnings, active processes |
| **Yellow** | `#D7C870` | `#BDAA4B` | `#A38C20` | Caution, moderate loads, alerts |
| **Green** | `#70D795` | `#50B675` | `#2E9657` | Success, health, online state |
| **Cyan** | `#70CCD7` | `#4BACBD` | `#208CA3` | Links, networking, live telemetry |
| **Blue** | `#70A2D7` | `#4F87C8` | `#2E6CB8` | Primary accent, selection, focus rings |
| **Purple** | `#A170D7` | `#8657C8` | `#6C3EB8` | Memory, configuration, hardware |
| **Magenta** | `#D770BD` | `#C858AA` | `#B83E98` | GPU, graphics, expressive highlights |

---

## Extended Palette

For complex interfaces, charts, heatmaps, badges, and multi-tier states, the extended palette provides complete `50` to `950` gradations for each family:

| Family | `50` | `100` | `150` | `200` | `300` | `400` | `500` | `600` | `700` | `800` | `850` | `900` | `950` |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Base** | `#F4F7F9` | `#EAF0F4` | `#DFE6EC` | `#DCE3E8` | `#CFDBE3` | `#A0B0BA` | `#748793` | `#5A6F7C` | `#384852` | `#19242B` | `#162026` | `#131B20` | `#0F151A` |
| **Red** | `#FDE4E2` | `#FBCECC` | `#F5B9B7` | `#EEA5A3` | `#E38988` | `#D77070` | `#C85857` | `#B83E3E` | `#8A3332` | `#612927` | `#4D2321` | `#3B1D1C` | `#231514` |
| **Orange** | `#FBE6DA` | `#F7D7C5` | `#F1C9B2` | `#EABB9F` | `#E0A886` | `#D79770` | `#C88057` | `#B8683E` | `#894F31` | `#603925` | `#4C2E1F` | `#3A2419` | `#221611` |
| **Yellow** | `#EFEBD6` | `#EBE5C2` | `#E7DFB0` | `#E3D99E` | `#DDD086` | `#D7C870` | `#BDAA4B` | `#A38C20` | `#79681E` | `#54491A` | `#423A17` | `#312C13` | `#1C190E` |
| **Green** | `#DDF0E2` | `#C8EDD3` | `#B6E9C5` | `#A3E4B7` | `#89DEA5` | `#70D795` | `#50B675` | `#2E9657` | `#287043` | `#214F31` | `#1D3F28` | `#18301F` | `#121C15` |
| **Cyan** | `#D6F0F4` | `#C3EBF0` | `#B1E4EB` | `#9FDEE5` | `#87D5DE` | `#70CCD7` | `#4BACBD` | `#208CA3` | `#1D697A` | `#194A56` | `#163C45` | `#122E35` | `#0D1C20` |
| **Blue** | `#DDECFD` | `#C8DFF9` | `#B4D2F3` | `#A1C5EC` | `#87B3E1` | `#70A2D7` | `#4F87C8` | `#2E6CB8` | `#27538A` | `#203D61` | `#1C324E` | `#18273B` | `#131A24` |
| **Purple** | `#EEE6FA` | `#E1D0F8` | `#D3BBF3` | `#C5A6EC` | `#B28AE2` | `#A170D7` | `#8657C8` | `#6C3EB8` | `#523489` | `#3C2A61` | `#31254D` | `#271F3A` | `#1A1723` |
| **Magenta** | `#F8E4F1` | `#F6CEEA` | `#F1B9E1` | `#EBA5D7` | `#E189CA` | `#D770BD` | `#C858AA` | `#B83E98` | `#893471` | `#5F2950` | `#4C2440` | `#391E30` | `#20151D` |

---

## Semantic Mappings

| Role | Light Theme | Dark Theme |
| :--- | :--- | :--- |
| **Canvas Background** | `bg` (`#F4F7F9`) | `bg` (`#0B1014`) |
| **Elevated Surface (Cards)** | `bg-2` (`#FFFFFF`) | `bg-2` (`#0F151A`) |
| **Interactive Surface (Inputs/Wells)** | `bg-3` (`#EAF0F4`) | `bg-3` (`#131B20`) |
| **Divider Borders** | `border-main` (`#DCE3E8`) | `border-main` (`#162026`) |
| **Card Borders** | `border-card` (`#DFE6EC`) | `border-card` (`#19242B`) |
| **Input / Focus Borders** | `border-input` (`#CFDBE3`) | `border-input` (`#22303A`) |
| **Primary Text** | `text-primary` (`#0B1014`) | `text-primary` (`#E9EEF1`) |
| **Muted Text** | `text-muted` (`#5A6F7C`) | `text-muted` (`#748793`) |
| **Primary Accent** | `blue-600` (`#2E6CB8`) | `blue-400` (`#70A2D7`) |
| **Accent Hover / Mid** | `blue-500` (`#4F87C8`) | `blue-500` (`#4F87C8`) |
| **Text on Solid Accent** | `#FFFFFF` | `#0B1014` |

---

## Code Implementations

### CSS Variables

```css
:root {
  color-scheme: light;
  --arclate-bg: #f4f7f9;
  --arclate-bg-2: #ffffff;
  --arclate-bg-3: #eaf0f4;
  --arclate-border-main: #dce3e8;
  --arclate-border-card: #dfe6ec;
  --arclate-border-input: #cfdbe3;
  --arclate-text-primary: #0b1014;
  --arclate-text-muted: #5a6f7c;
  --arclate-accent-blue: #2e6cb8;
  --arclate-accent-green: #2e9657;
  --arclate-accent-yellow: #a38c20;
  --arclate-accent-red: #b83e3e;
}

@media (prefers-color-scheme: dark) {
  :root {
    color-scheme: dark;
    --arclate-bg: #0b1014;
    --arclate-bg-2: #0f151a;
    --arclate-bg-3: #131b20;
    --arclate-border-main: #162026;
    --arclate-border-card: #19242b;
    --arclate-border-input: #22303a;
    --arclate-text-primary: #e9eef1;
    --arclate-text-muted: #748793;
    --arclate-accent-blue: #70a2d7;
    --arclate-accent-green: #70d795;
    --arclate-accent-yellow: #d7c870;
    --arclate-accent-red: #d77070;
  }
}
```

### Rust GPUI Constants

```rust
pub mod arclate {
    // Dark base surfaces (Arctic + Slate)
    pub const BG_DARK: u32 = 0x000B_1014;
    pub const BG2_DARK: u32 = 0x000F_151A;
    pub const BG3_DARK: u32 = 0x0013_1B20;
    pub const BORDER_MAIN_DARK: u32 = 0x0016_2026;
    pub const BORDER_CARD_DARK: u32 = 0x0019_242B;
    pub const BORDER_INPUT_DARK: u32 = 0x0022_303A;
    pub const TEXT_PRIMARY_DARK: u32 = 0x00E9_EEF1;
    pub const TEXT_MUTED_DARK: u32 = 0x0074_8793;

    // Light base surfaces
    pub const BG_LIGHT: u32 = 0x00F4_F7F9;
    pub const BG2_LIGHT: u32 = 0x00FF_FFFF;
    pub const BG3_LIGHT: u32 = 0x00EA_F0F4;
    pub const BORDER_MAIN_LIGHT: u32 = 0x00DC_E3E8;
    pub const BORDER_CARD_LIGHT: u32 = 0x00DF_E6EC;
    pub const BORDER_INPUT_LIGHT: u32 = 0x00CF_DBE3;
    pub const TEXT_PRIMARY_LIGHT: u32 = 0x000B_1014;
    pub const TEXT_MUTED_LIGHT: u32 = 0x005A_6F7C;

    // Accent Ramps (400, 500, 600)
    pub const BLUE_DARK: u32 = 0x0070_A2D7;  // 400
    pub const BLUE_500: u32 = 0x004F_87C8;   // 500
    pub const BLUE_LIGHT: u32 = 0x002E_6CB8; // 600

    pub const GREEN_DARK: u32 = 0x0070_D795;
    pub const GREEN_500: u32 = 0x0050_B675;
    pub const GREEN_LIGHT: u32 = 0x002E_9657;

    pub const YELLOW_DARK: u32 = 0x00D7_C870;
    pub const YELLOW_500: u32 = 0x00BD_AA4B;
    pub const YELLOW_LIGHT: u32 = 0x00A3_8C20;

    pub const RED_DARK: u32 = 0x00D7_7070;
    pub const RED_500: u32 = 0x00C8_5857;
    pub const RED_LIGHT: u32 = 0x00B8_3E3E;
}
```

---

## Mathematical Model & Verification

Arclate follows the anchor convention established by [Flexoki 2.0](https://github.com/kepano/flexoki): accent tones and neutrals are calibrated in the perceptually uniform **OKLCH** color space ($L$: Lightness, $C$: Chroma, $h$: Hue angle).

For each family, `500` is computed directly from `400` and `600`:

$$L_{500} = \frac{L_{400} + L_{600}}{2}, \quad C_{500} = \frac{C_{400} + C_{600}}{2}, \quad h_{500} = \text{shortest\_angle}(h_{400}, h_{600}, 0.5)$$

### Verification Coordinates

| Family | `400` OKLCH $(L, C, h)$ | `500` OKLCH $(L, C, h)$ | `600` OKLCH $(L, C, h)$ | Monotonic $L$ |
| :--- | :--- | :--- | :--- | :---: |
| **Base** | `(0.748, 0.023, 235.1°)` | `(0.612, 0.029, 235.7°)` | `(0.530, 0.032, 235.4°)` | Yes |
| **Red** | `(0.663, 0.129, 21.2°)` | `(0.601, 0.143, 22.6°)` | `(0.538, 0.158, 24.1°)` | Yes |
| **Orange** | `(0.730, 0.094, 52.1°)` | `(0.666, 0.106, 49.6°)` | `(0.600, 0.117, 47.2°)` | Yes |
| **Yellow** | `(0.827, 0.111, 99.8°)` | `(0.736, 0.116, 98.0°)` | `(0.643, 0.122, 96.2°)` | Yes |
| **Green** | `(0.800, 0.135, 153.9°)` | `(0.699, 0.135, 153.3°)` | `(0.598, 0.134, 152.7°)` | Yes |
| **Cyan** | `(0.792, 0.089, 205.6°)` | `(0.693, 0.093, 210.9°)` | `(0.593, 0.097, 216.2°)` | Yes |
| **Blue** | `(0.698, 0.095, 250.7°)` | `(0.614, 0.115, 253.0°)` | `(0.530, 0.135, 255.3°)` | Yes |
| **Purple** | `(0.638, 0.156, 304.3°)` | `(0.562, 0.170, 300.2°)` | `(0.486, 0.183, 296.2°)` | Yes |
| **Magenta** | `(0.688, 0.158, 337.6°)` | `(0.627, 0.172, 338.7°)` | `(0.566, 0.185, 339.8°)` | Yes |

All ramps are strictly monotonic in perceptual lightness, preventing perceptual inversion when transitioning across interface states.

---

## References

- [Flexoki Color Scheme by Steph Ango (kepano)](https://stephango.com/flexoki)
- [Flexoki 2.0 Repository & Source](https://github.com/kepano/flexoki)
- [Oklab Color Space (Björn Ottosson)](https://bottosson.github.io/posts/oklab/)
