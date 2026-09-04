use gpui::{App, Global, Rgba, rgb, rgba};

#[allow(dead_code)]
pub mod flexoki {
    // Official 1:1 Flexoki Base Grayscale (14 steps)
    pub const BLACK: u32 = 0x0010_0F0F; // #100f0f: Darkest black
    pub const BASE_950: u32 = 0x001C_1B1A; // #1c1b1a: Surface backdrop dark
    pub const BASE_900: u32 = 0x0028_2726; // #282726: Surface elevated dark
    pub const BASE_850: u32 = 0x0034_3331; // #343331: Surface interactive dark
    pub const BASE_800: u32 = 0x0040_3E3C; // #403e3c: Borders dark
    pub const BASE_700: u32 = 0x0057_5653; // #575653
    pub const BASE_600: u32 = 0x006F_6E69; // #6f6e69: Muted text dark
    pub const BASE_500: u32 = 0x0087_8580; // #878580
    pub const BASE_300: u32 = 0x00B7_B5AC; // #b7b5ac: Muted text light
    pub const BASE_200: u32 = 0x00CE_CDC3; // #cecdc3: Borders light
    pub const BASE_150: u32 = 0x00DA_D8CE; // #dad8ce: Surface interactive light
    pub const BASE_100: u32 = 0x00E6_E4D9; // #e6e4d9: Surface elevated light
    pub const BASE_50: u32 = 0x00F2_F0E5; // #f2f0e5: Surface backdrop light
    pub const PAPER: u32 = 0x00FF_FCF0; // #fffcf0: Lightest paper white

    // Red (400, 500, 600)
    pub const RED_400: u32 = 0x00AF_3029; // #af3029: Light UI accent
    pub const RED_500: u32 = 0x00C0_3E35; // #c03e35: Mid
    pub const RED_600: u32 = 0x00D1_4D41; // #d14d41: Dark UI accent

    // Orange (400, 500, 600)
    pub const ORANGE_400: u32 = 0x00BC_5215; // #bc5215: Light UI accent
    pub const ORANGE_500: u32 = 0x00CB_6120; // #cb6120: Mid
    pub const ORANGE_600: u32 = 0x00DA_702C; // #da702c: Dark UI accent

    // Yellow (400, 500, 600)
    pub const YELLOW_400: u32 = 0x00AD_8301; // #ad8301: Light UI accent
    pub const YELLOW_500: u32 = 0x00BE_920B; // #be920b: Mid
    pub const YELLOW_600: u32 = 0x00D0_A215; // #d0a215: Dark UI accent

    // Green (400, 500, 600)
    pub const GREEN_400: u32 = 0x0066_800B; // #66800b: Light UI accent
    pub const GREEN_500: u32 = 0x0076_8D22; // #768d22: Mid
    pub const GREEN_600: u32 = 0x0087_9A39; // #879a39: Dark UI accent

    // Cyan (400, 500, 600)
    pub const CYAN_400: u32 = 0x0024_837B; // #24837b: Light UI accent
    pub const CYAN_500: u32 = 0x002F_968D; // #2f968d: Mid
    pub const CYAN_600: u32 = 0x003A_A99F; // #3aa99f: Dark UI accent

    // Blue (400, 500, 600)
    pub const BLUE_400: u32 = 0x0020_5EA6; // #205ea6: Light UI accent
    pub const BLUE_500: u32 = 0x0031_71B2; // #3171b2: Mid
    pub const BLUE_600: u32 = 0x0043_85BE; // #4385be: Dark UI accent

    // Purple (400, 500, 600)
    pub const PURPLE_400: u32 = 0x005E_409D; // #5e409d: Light UI accent
    pub const PURPLE_500: u32 = 0x0074_5EB4; // #745eb4: Mid
    pub const PURPLE_600: u32 = 0x008B_7EC8; // #8b7ec8: Dark UI accent

    // Magenta (400, 500, 600)
    pub const MAGENTA_400: u32 = 0x00A0_2F6F; // #a02f6f: Light UI accent
    pub const MAGENTA_500: u32 = 0x00B7_4683; // #b74683: Mid
    pub const MAGENTA_600: u32 = 0x00CE_5D97; // #ce5d97: Dark UI accent
}

#[allow(dead_code)]
pub mod arclate {
    // Dark base surfaces (Arctic + Slate)
    pub const BG_DARK: u32 = 0x000B_1014; // #0b1014: Window backdrop / main bg
    pub const BG2_DARK: u32 = 0x000F_151A; // #0f151a: Elevated card surface & titlebar/sidebar canvas
    pub const BG3_DARK: u32 = 0x0013_1B20; // #131b20: Interactive surface (buttons, inputs, dropdowns)
    pub const BORDER_MAIN_DARK: u32 = 0x0016_2026; // #162026: Divider lines
    pub const BORDER_CARD_DARK: u32 = 0x0019_242B; // #19242b: Card borders
    pub const BORDER_INPUT_DARK: u32 = 0x0022_303A; // #22303a: Interactive borders
    pub const TEXT_PRIMARY_DARK: u32 = 0x00E9_EEF1; // #e9eef1: Crisp white/frost text
    pub const TEXT_MUTED_DARK: u32 = 0x0074_8793; // #748793: Secondary muted slate text

    // Accent ramps: 400 for dark UI, 600 for light UI, 500 as their OKLCH midpoint
    pub const RED_DARK: u32 = 0x00D7_7070; // #d77070 (400)
    pub const RED_500: u32 = 0x00C8_5857; // #c85857 (500)
    pub const RED_LIGHT: u32 = 0x00B8_3E3E; // #b83e3e (600)

    pub const GREEN_DARK: u32 = 0x0070_D795; // #70d795 (400)
    pub const GREEN_500: u32 = 0x0050_B675; // #50b675 (500)
    pub const GREEN_LIGHT: u32 = 0x002E_9657; // #2e9657 (600)

    pub const CYAN_DARK: u32 = 0x0070_CCD7; // #70ccd7 (400)
    pub const CYAN_500: u32 = 0x004B_ACBD; // #4bacbd (500)
    pub const CYAN_LIGHT: u32 = 0x0020_8CA3; // #208ca3 (600)

    pub const BLUE_DARK: u32 = 0x0070_A2D7; // #70a2d7 (400)
    pub const BLUE_500: u32 = 0x004F_87C8; // #4f87c8 (500)
    pub const BLUE_LIGHT: u32 = 0x002E_6CB8; // #2e6cb8 (600)

    pub const PURPLE_DARK: u32 = 0x00A1_70D7; // #a170d7 (400)
    pub const PURPLE_500: u32 = 0x0086_57C8; // #8657c8 (500)
    pub const PURPLE_LIGHT: u32 = 0x006C_3EB8; // #6c3eb8 (600)

    pub const MAGENTA_DARK: u32 = 0x00D7_70BD; // #d770bd (400)
    pub const MAGENTA_500: u32 = 0x00C8_58AA; // #c858aa (500)
    pub const MAGENTA_LIGHT: u32 = 0x00B8_3E98; // #b83e98 (600)

    pub const ORANGE_DARK: u32 = 0x00D7_9770; // #d79770 (400)
    pub const ORANGE_500: u32 = 0x00C8_8057; // #c88057 (500)
    pub const ORANGE_LIGHT: u32 = 0x00B8_683E; // #b8683e (600)

    pub const YELLOW_DARK: u32 = 0x00D7_C870; // #d7c870 (400)
    pub const YELLOW_500: u32 = 0x00BD_AA4B; // #bdaa4b (500)
    pub const YELLOW_LIGHT: u32 = 0x00A3_8C20; // #a38c20 (600)

    // Light base surfaces
    pub const BG_LIGHT: u32 = 0x00F4_F7F9; // #f4f7f9: Light window backdrop
    pub const BG2_LIGHT: u32 = 0x00FF_FFFF; // #ffffff: Pure white card surface & titlebar/sidebar canvas
    pub const BG3_LIGHT: u32 = 0x00EA_F0F4; // #eaf0f4: Interactive light surface
    pub const BORDER_MAIN_LIGHT: u32 = 0x00DC_E3E8; // #dce3e8
    pub const BORDER_CARD_LIGHT: u32 = 0x00DF_E6EC; // #dfe6ec
    pub const BORDER_INPUT_LIGHT: u32 = 0x00CF_DBE3; // #cfdbe3
    pub const TEXT_PRIMARY_LIGHT: u32 = 0x000B_1014; // #0b1014
    pub const TEXT_MUTED_LIGHT: u32 = 0x005A_6F7C; // #5a6f7c
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemePalette {
    #[default]
    Arclate,
    Flexoki,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Dark,
    Light,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct Theme {
    pub palette: ThemePalette,
    pub mode: ThemeMode,
    pub transparency: bool,
    pub window_bg: Rgba,
    pub sidebar_bg: Rgba,
    pub titlebar_bg: Rgba,
    pub main_bg: Rgba,
    pub main_border: Rgba,
    pub card_bg: Rgba,
    pub card_border: Rgba,
    pub input_bg: Rgba,
    pub input_border: Rgba,
    pub text_primary: Rgba,
    pub text_muted: Rgba,
    pub selected_text: Rgba,
    pub button_hover: Rgba,
    pub button_active: Rgba,
    pub button_selected: Rgba,
    pub accent_hover_bg: Rgba,
    pub accent_active_bg: Rgba,
    pub accent_selected_bg: Rgba,
    pub accent_selected_hover_bg: Rgba,
    pub accent_red: Rgba,
    pub accent_orange: Rgba,
    pub accent_yellow: Rgba,
    pub accent_green: Rgba,
    pub accent_cyan: Rgba,
    pub accent_blue: Rgba,
    pub accent_purple: Rgba,
    pub accent_magenta: Rgba,
}

impl Global for Theme {}

#[allow(dead_code)]
impl Theme {
    #[must_use]
    pub fn build(palette: ThemePalette, mode: ThemeMode, transparency: bool) -> Self {
        let mut t = match (palette, mode) {
            (ThemePalette::Arclate, ThemeMode::Light) => Self::light(),
            (ThemePalette::Arclate, ThemeMode::Dark) => Self::dark(),
            (ThemePalette::Arclate, ThemeMode::System) => Self::system(),
            (ThemePalette::Flexoki, ThemeMode::Light) => Self::flexoki_light(),
            (ThemePalette::Flexoki, ThemeMode::Dark) => Self::flexoki_dark(),
            (ThemePalette::Flexoki, ThemeMode::System) => {
                let mut f = Self::flexoki_dark();
                f.mode = ThemeMode::System;
                f
            }
        };
        t = t.with_transparency(transparency);
        t
    }

    #[must_use]
    pub fn system() -> Self {
        let mut t = Self::dark();
        t.mode = ThemeMode::System;
        t
    }

    #[must_use]
    pub fn dark() -> Self {
        Self {
            palette: ThemePalette::Arclate,
            mode: ThemeMode::Dark,
            transparency: true,
            // Default 50% opacity backdrop of bg2 (#0f151a) for smooth acrylic/mica blur
            window_bg: rgba(0x0F15_1A80),
            sidebar_bg: rgba(0x0000_0000),
            titlebar_bg: rgba(0x0000_0000),
            main_bg: rgb(arclate::BG_DARK),
            main_border: rgb(arclate::BORDER_MAIN_DARK),
            // bg-2 & ui tokens for elevated group cards
            card_bg: rgb(arclate::BG2_DARK),
            card_border: rgb(arclate::BORDER_CARD_DARK),
            // input & dropdown tokens
            input_bg: rgb(arclate::BG3_DARK),
            input_border: rgb(arclate::BORDER_INPUT_DARK),
            // Frost white for unselected text/icons
            text_primary: rgb(arclate::TEXT_PRIMARY_DARK),
            text_muted: rgb(arclate::TEXT_MUTED_DARK),
            // High-contrast text on solid accent background
            selected_text: rgb(arclate::BG_DARK),
            button_hover: rgba(0x1924_2B99),
            button_active: rgba(0x2230_3ACC),
            button_selected: rgba(0x1924_2BFF),
            // 50% opacity of primary accent blue for hover
            accent_hover_bg: rgba(0x70A2_D780),
            accent_active_bg: rgba(0x70A2_D7A0),
            // 100% solid non-transparent primary accent blue for selected state
            accent_selected_bg: rgb(arclate::BLUE_DARK),
            accent_selected_hover_bg: rgb(arclate::BLUE_DARK),
            accent_red: rgb(arclate::RED_DARK),
            accent_orange: rgb(arclate::ORANGE_DARK),
            accent_yellow: rgb(arclate::YELLOW_DARK),
            accent_green: rgb(arclate::GREEN_DARK),
            accent_cyan: rgb(arclate::CYAN_DARK),
            accent_blue: rgb(arclate::BLUE_DARK),
            accent_purple: rgb(arclate::PURPLE_DARK),
            accent_magenta: rgb(arclate::MAGENTA_DARK),
        }
    }

    #[must_use]
    pub fn light() -> Self {
        Self {
            palette: ThemePalette::Arclate,
            mode: ThemeMode::Light,
            transparency: true,
            // Default 50% opacity backdrop of bg2 (#ffffff) for smooth acrylic/mica blur
            window_bg: rgba(0xFFFF_FF80),
            sidebar_bg: rgba(0x0000_0000),
            titlebar_bg: rgba(0x0000_0000),
            main_bg: rgb(arclate::BG_LIGHT),
            main_border: rgb(arclate::BORDER_MAIN_LIGHT),
            // bg-2 & ui tokens for elevated group cards
            card_bg: rgb(arclate::BG2_LIGHT),
            card_border: rgb(arclate::BORDER_CARD_LIGHT),
            // input & dropdown tokens
            input_bg: rgb(arclate::BG3_LIGHT),
            input_border: rgb(arclate::BORDER_INPUT_LIGHT),
            // Dark ink for unselected text/icons
            text_primary: rgb(arclate::TEXT_PRIMARY_LIGHT),
            text_muted: rgb(arclate::TEXT_MUTED_LIGHT),
            // High-contrast white text on solid accent background
            selected_text: rgb(arclate::BG2_LIGHT),
            button_hover: rgba(0xDFE6_EC99),
            button_active: rgba(0xCFDB_E3CC),
            button_selected: rgba(0xDFE6_ECFF),
            // 50% opacity of primary accent blue for hover
            accent_hover_bg: rgba(0x2E6C_B880),
            accent_active_bg: rgba(0x2E6C_B8A0),
            // 100% solid non-transparent primary accent blue for selected state
            accent_selected_bg: rgb(arclate::BLUE_LIGHT),
            accent_selected_hover_bg: rgb(arclate::BLUE_LIGHT),
            accent_red: rgb(arclate::RED_LIGHT),
            accent_orange: rgb(arclate::ORANGE_LIGHT),
            accent_yellow: rgb(arclate::YELLOW_LIGHT),
            accent_green: rgb(arclate::GREEN_LIGHT),
            accent_cyan: rgb(arclate::CYAN_LIGHT),
            accent_blue: rgb(arclate::BLUE_LIGHT),
            accent_purple: rgb(arclate::PURPLE_LIGHT),
            accent_magenta: rgb(arclate::MAGENTA_LIGHT),
        }
    }

    #[must_use]
    pub fn flexoki_dark() -> Self {
        Self {
            palette: ThemePalette::Flexoki,
            mode: ThemeMode::Dark,
            transparency: true,
            window_bg: rgba(0x1C1B_1A80),
            sidebar_bg: rgba(0x0000_0000),
            titlebar_bg: rgba(0x0000_0000),
            main_bg: rgb(flexoki::BLACK),
            main_border: rgb(flexoki::BASE_800),
            card_bg: rgb(flexoki::BASE_950),
            card_border: rgb(flexoki::BASE_850),
            input_bg: rgb(flexoki::BASE_900),
            input_border: rgb(flexoki::BASE_800),
            text_primary: rgb(flexoki::BASE_200),
            text_muted: rgb(flexoki::BASE_600),
            selected_text: rgb(flexoki::BLACK),
            button_hover: rgba(0x3433_3199),
            button_active: rgba(0x403E_3CCC),
            button_selected: rgba(0x3433_31FF),
            accent_hover_bg: rgba(0x4385_BE80),
            accent_active_bg: rgba(0x4385_BEA0),
            accent_selected_bg: rgb(flexoki::BLUE_600),
            accent_selected_hover_bg: rgb(flexoki::BLUE_600),
            accent_red: rgb(flexoki::RED_600),
            accent_orange: rgb(flexoki::ORANGE_600),
            accent_yellow: rgb(flexoki::YELLOW_600),
            accent_green: rgb(flexoki::GREEN_600),
            accent_cyan: rgb(flexoki::CYAN_600),
            accent_blue: rgb(flexoki::BLUE_600),
            accent_purple: rgb(flexoki::PURPLE_600),
            accent_magenta: rgb(flexoki::MAGENTA_600),
        }
    }

    #[must_use]
    pub fn flexoki_light() -> Self {
        Self {
            palette: ThemePalette::Flexoki,
            mode: ThemeMode::Light,
            transparency: true,
            window_bg: rgba(0xFFFC_F080),
            sidebar_bg: rgba(0x0000_0000),
            titlebar_bg: rgba(0x0000_0000),
            main_bg: rgb(flexoki::BASE_50),
            main_border: rgb(flexoki::BASE_200),
            card_bg: rgb(flexoki::PAPER),
            card_border: rgb(flexoki::BASE_150),
            input_bg: rgb(flexoki::BASE_100),
            input_border: rgb(flexoki::BASE_200),
            text_primary: rgb(flexoki::BLACK),
            text_muted: rgb(flexoki::BASE_600),
            selected_text: rgb(flexoki::PAPER),
            button_hover: rgba(0xDAD8_CE99),
            button_active: rgba(0xCECD_C3CC),
            button_selected: rgba(0xDAD8_CEFF),
            accent_hover_bg: rgba(0x205E_A680),
            accent_active_bg: rgba(0x205E_A6A0),
            accent_selected_bg: rgb(flexoki::BLUE_400),
            accent_selected_hover_bg: rgb(flexoki::BLUE_400),
            accent_red: rgb(flexoki::RED_400),
            accent_orange: rgb(flexoki::ORANGE_400),
            accent_yellow: rgb(flexoki::YELLOW_400),
            accent_green: rgb(flexoki::GREEN_400),
            accent_cyan: rgb(flexoki::CYAN_400),
            accent_blue: rgb(flexoki::BLUE_400),
            accent_purple: rgb(flexoki::PURPLE_400),
            accent_magenta: rgb(flexoki::MAGENTA_400),
        }
    }

    #[must_use]
    pub fn with_transparency(mut self, enabled: bool) -> Self {
        self.transparency = enabled;
        if enabled {
            self.window_bg = match (self.palette, self.mode) {
                (ThemePalette::Flexoki, ThemeMode::Light) => rgba(0xFFFC_F080),
                (ThemePalette::Flexoki, _) => rgba(0x1C1B_1A80),
                (ThemePalette::Arclate, ThemeMode::Light) => rgba(0xFFFF_FF80),
                (ThemePalette::Arclate, _) => rgba(0x0F15_1A80),
            };
            self.sidebar_bg = rgba(0x0000_0000);
            self.titlebar_bg = rgba(0x0000_0000);
        } else {
            let solid_bg2 = match (self.palette, self.mode) {
                (ThemePalette::Flexoki, ThemeMode::Light) => rgb(flexoki::PAPER),
                (ThemePalette::Flexoki, _) => rgb(flexoki::BASE_950),
                (ThemePalette::Arclate, ThemeMode::Light) => rgb(arclate::BG2_LIGHT),
                (ThemePalette::Arclate, _) => rgb(arclate::BG2_DARK),
            };
            self.window_bg = solid_bg2;
            self.sidebar_bg = solid_bg2;
            self.titlebar_bg = solid_bg2;
        }
        self
    }

    #[must_use]
    pub fn get(cx: &App) -> Self {
        if cx.has_global::<Theme>() {
            *cx.global::<Theme>()
        } else {
            Self::dark()
        }
    }
}
