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
    pub is_applied: fn() -> bool,
    pub set_applied: fn(bool) -> Result<(), String>,
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
            is_applied,
            set_applied,
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
