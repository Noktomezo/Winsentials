use std::sync::Arc;

use gpui::{App, Window};

use crate::entities::startup::{StartupEntry, StartupSource};
use crate::shared::ui::TooltipState;

pub type StartupToggleHandler = Arc<dyn Fn(&StartupEntry, &mut Window, &mut App) + 'static>;
pub type StartupDeleteHandler = Arc<dyn Fn(&StartupEntry, &mut Window, &mut App) + 'static>;
pub type StartupActionHandler = Arc<dyn Fn(&StartupEntry, &mut Window, &mut App) + 'static>;
pub type TooltipHoverHandler = Arc<dyn Fn(Option<TooltipState>, &mut Window, &mut App) + 'static>;
pub type MenuToggleHandler = Arc<dyn Fn(Option<String>, &mut Window, &mut App) + 'static>;
pub type FilterSelectHandler = Arc<dyn Fn(Option<StartupSource>, &mut Window, &mut App) + 'static>;
pub type SearchHoverHandler = Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SearchFocusHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;
pub type SearchSelectionHandler =
    Arc<dyn Fn(Option<(usize, usize)>, &mut Window, &mut App) + 'static>;
pub type StartupHoverCardHandler = Arc<dyn Fn(Option<String>, &mut Window, &mut App) + 'static>;