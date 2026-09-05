use std::sync::Arc;
use std::time::Duration;

use gpui::{App, SharedString, Window};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastPosition {
    #[default]
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
    BottomCenter,
    TopCenter,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastVariant {
    #[default]
    Default,
    Success,
    Warning,
    Error,
    Info,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Destructive,
}

pub type ToastActionHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type ToastDismissHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type ToastButtonHoverHandler = Arc<dyn Fn(usize, &bool, &mut Window, &mut App) + 'static>;

#[derive(Clone)]
pub struct ToastButton {
    pub label: SharedString,
    pub variant: ToastButtonVariant,
    pub icon: Option<SharedString>,
    pub on_click: Option<ToastActionHandler>,
    pub full_width: bool,
}

#[allow(dead_code)]
impl ToastButton {
    #[must_use]
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: ToastButtonVariant::Primary,
            icon: None,
            on_click: None,
            full_width: false,
        }
    }

    #[must_use]
    pub const fn variant(mut self, variant: ToastButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub const fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    #[must_use]
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[must_use]
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastProgress {
    pub value: f32, // 0.0 .. 1.0
    pub label: Option<SharedString>,
}

#[derive(Clone)]
pub struct ToastData {
    pub id: SharedString,
    pub variant: ToastVariant,
    pub position: ToastPosition,
    pub icon: Option<SharedString>,
    pub title: SharedString,
    pub description: Option<SharedString>,
    pub buttons: Vec<ToastButton>,
    pub progress: Option<ToastProgress>,
    pub duration: Option<Duration>,
    pub count: usize,
}

#[allow(dead_code)]
impl ToastData {
    #[must_use]
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            variant: ToastVariant::Default,
            position: ToastPosition::BottomRight,
            icon: None,
            title: title.into(),
            description: None,
            buttons: Vec::new(),
            progress: None,
            duration: Some(Duration::from_secs(5)),
            count: 1,
        }
    }

    #[must_use]
    pub const fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub const fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    #[must_use]
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[must_use]
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    pub fn button(mut self, button: ToastButton) -> Self {
        self.buttons.push(button);
        self
    }

    #[must_use]
    pub fn buttons(mut self, buttons: Vec<ToastButton>) -> Self {
        self.buttons = buttons;
        self
    }

    #[must_use]
    pub fn progress(mut self, progress: Option<ToastProgress>) -> Self {
        self.progress = progress;
        self
    }

    #[must_use]
    pub const fn duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }

    #[must_use]
    pub const fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }
}

pub type ToastDismissIdHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type ToastHoverBtnIdHandler = Arc<dyn Fn(&str, usize, &bool, &mut Window, &mut App) + 'static>;
pub type ToastStackHoverHandler = Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

