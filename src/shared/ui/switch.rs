use std::sync::Arc;

use gpui::{
    AnimationExt, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SpringAnimation, SpringConfig, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::shared::theme::Theme;
use crate::widgets::sidebar::lerp_rgba;

pub type SwitchToggleHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Switch {
    id: &'static str,
    checked: bool,
    on_toggle: Option<SwitchToggleHandler>,
}

impl Switch {
    #[must_use]
    pub fn new(id: &'static str, checked: bool) -> Self {
        Self {
            id,
            checked,
            on_toggle: None,
        }
    }

    #[must_use]
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let id_str = self.id;
        let checked = self.checked;
        let on_toggle = self.on_toggle;

        // Switch dimensions:
        // Height: 20px (compact, proportional toggle)
        // Width: 36px
        // Radius: full pill (9999px)
        // Border: 1px -> inner content: 34px width x 18px height
        // Thumb: 14px x 14px, rounded(9999px)
        // Positioning: top: 2px, left travel: 2px -> 18px (travel delta = 16px)
        let target_state = if checked { 1.0 } else { 0.0 };

        let state_spring = SpringAnimation::new(SpringConfig::new(380.0, 30.0, 1.0))
            .to(target_state)
            .with_epsilon(0.005);

        // Smooth transition Red (#d77070) <-> Green (#70d795)
        let red_color = theme.accent_red;
        let green_color = theme.accent_green;
        let thumb_bg = theme.card_bg;

        let toggle_action = move |window: &mut Window, cx: &mut App| {
            if let Some(ref h) = on_toggle {
                h(!checked, window, cx);
            }
        };

        div()
            .id(ElementId::Name(id_str.into()))
            .relative()
            .w(px(36.0))
            .h(px(20.0))
            .rounded(px(6.0))
            .border_1()
            .cursor_pointer()
            .on_click(move |_, window, cx| {
                toggle_action(window, cx);
            })
            .with_spring(
                ElementId::Name(format!("{id_str}_track_spring").into()),
                state_spring,
                move |track, val| {
                    let progress = val.clamp(0.0, 1.0);
                    let color = lerp_rgba(red_color, green_color, progress);
                    let slide_x = progress * 16.0;

                    let thumb_el = div()
                        .absolute()
                        .top(px(2.0))
                        .left(px(2.0 + slide_x))
                        .size(px(14.0))
                        .rounded(px(4.0))
                        .bg(thumb_bg);

                    track.bg(color).border_color(color).child(thumb_el)
                },
            )
    }
}
