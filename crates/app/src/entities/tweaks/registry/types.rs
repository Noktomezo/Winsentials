#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TweakCategory {
    ContextMenu,
    Explorer,
    Interface,
    Input,
    System,
    Network,
    Privacy,
    Performance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum RestartRequirement {
    None,
    Explorer,
    Logoff,
    Reboot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SideEffectLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SideEffect {
    pub level: SideEffectLevel,
    pub description_key: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub enum TweakKind {
    Toggle {
        is_applied: fn() -> bool,
        set_applied: fn(bool) -> Result<(), String>,
    },
    Select {
        options: &'static [&'static str],
        get_current: fn() -> &'static str,
        set_current: fn(&str) -> Result<(), String>,
    },
}

impl TweakKind {
    #[must_use]
    pub const fn is_toggle(&self) -> bool {
        matches!(self, Self::Toggle { .. })
    }

    #[must_use]
    pub const fn is_select(&self) -> bool {
        matches!(self, Self::Select { .. })
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct TweakDefinition {
    pub id: &'static str,
    pub category: TweakCategory,
    pub icon: &'static str,
    pub title_key: &'static str,
    pub desc_key: &'static str,
    pub min_build: Option<u32>,
    pub max_build: Option<u32>,
    pub custom_support: Option<fn() -> bool>,
    pub restart: RestartRequirement,
    pub side_effect: Option<SideEffect>,
    pub kind: TweakKind,
}

impl TweakDefinition {
    #[must_use]
    pub const fn new(
        id: &'static str,
        category: TweakCategory,
        icon: &'static str,
        title_key: &'static str,
        desc_key: &'static str,
        is_applied: fn() -> bool,
        set_applied: fn(bool) -> Result<(), String>,
    ) -> Self {
        Self {
            id,
            category,
            icon,
            title_key,
            desc_key,
            min_build: None,
            max_build: None,
            custom_support: None,
            restart: RestartRequirement::None,
            side_effect: None,
            kind: TweakKind::Toggle {
                is_applied,
                set_applied,
            },
        }
    }

    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new_select(
        id: &'static str,
        category: TweakCategory,
        icon: &'static str,
        title_key: &'static str,
        desc_key: &'static str,
        options: &'static [&'static str],
        get_current: fn() -> &'static str,
        set_current: fn(&str) -> Result<(), String>,
    ) -> Self {
        Self {
            id,
            category,
            icon,
            title_key,
            desc_key,
            min_build: None,
            max_build: None,
            custom_support: None,
            restart: RestartRequirement::None,
            side_effect: None,
            kind: TweakKind::Select {
                options,
                get_current,
                set_current,
            },
        }
    }

    #[must_use]
    pub fn is_applied(&self) -> bool {
        match self.kind {
            TweakKind::Toggle { is_applied, .. } => is_applied(),
            TweakKind::Select { get_current, .. } => {
                let curr = get_current();
                curr != "off" && curr != "standard"
            }
        }
    }

    pub fn set_applied(&self, applied: bool) -> Result<(), String> {
        match self.kind {
            TweakKind::Toggle { set_applied, .. } => set_applied(applied),
            TweakKind::Select { .. } => Err(format!("Cannot toggle select tweak '{}'", self.id)),
        }
    }

    #[must_use]
    pub fn get_select_value(&self) -> Option<&'static str> {
        match self.kind {
            TweakKind::Select { get_current, .. } => Some(get_current()),
            TweakKind::Toggle { .. } => None,
        }
    }

    pub fn set_select_value(&self, value: &str) -> Result<(), String> {
        match self.kind {
            TweakKind::Select { set_current, .. } => set_current(value),
            TweakKind::Toggle { .. } => Err(format!(
                "Cannot set select value on toggle tweak '{}'",
                self.id
            )),
        }
    }

    #[must_use]
    pub const fn with_min_build(mut self, min: u32) -> Self {
        self.min_build = Some(min);
        self
    }

    #[must_use]
    pub const fn with_max_build(mut self, max: u32) -> Self {
        self.max_build = Some(max);
        self
    }

    #[must_use]
    pub const fn with_restart(mut self, restart: RestartRequirement) -> Self {
        self.restart = restart;
        self
    }

    #[must_use]
    pub const fn with_side_effect(
        mut self,
        level: SideEffectLevel,
        description_key: &'static str,
    ) -> Self {
        self.side_effect = Some(SideEffect {
            level,
            description_key,
        });
        self
    }

    #[must_use]
    pub const fn with_custom_support(mut self, check: fn() -> bool) -> Self {
        self.custom_support = Some(check);
        self
    }

    #[must_use]
    pub fn is_supported(&self, current_build: u32) -> bool {
        if let Some(min) = self.min_build {
            if current_build < min {
                return false;
            }
        }
        if let Some(max) = self.max_build {
            if current_build > max {
                return false;
            }
        }
        if let Some(custom) = self.custom_support {
            if !custom() {
                return false;
            }
        }
        true
    }
}
