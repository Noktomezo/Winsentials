use std::sync::Arc;

use gpui::{App, Window};

pub type SearchChangeHandler = Arc<dyn Fn(String, &mut Window, &mut App) + 'static>;
pub type SearchHoverHandler = Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SearchFocusHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;
pub type SearchSelectionHandler =
    Arc<dyn Fn(Option<(usize, usize)>, &mut Window, &mut App) + 'static>;
