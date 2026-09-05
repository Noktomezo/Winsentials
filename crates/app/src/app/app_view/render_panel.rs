use std::rc::Rc;

use gpui::{Context, Div, ParentElement, SharedString, Styled, div, px};

use crate::entities::cleanup::CleanupCategory;
use crate::features::navigation::AppRoute;
use crate::pages::{CleanupPage, render_route};
use crate::shared::theme::Theme;

use super::AppView;

impl AppView {
    #[allow(clippy::too_many_lines)]
    pub(super) fn render_main_panel(&mut self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let current_route = self.current_route;
        let current_locale = self.current_locale;
        let open_dropdown = self.open_dropdown;
        let opening_dropdown = self.opening_dropdown;
        let closing_dropdown = self.closing_dropdown;
        let hovered_dropdown = self.hovered_dropdown;
        let hovered_option = self.hovered_option;
        let pending_selection = self.pending_selection;
        let hovered_telemetry_card = self.hovered_telemetry_card.clone();
        let windows_build = self.windows_build;
        let telemetry = self.telemetry.clone();

        let on_hover_telemetry_card = cx.listener(
            |this, &(ref card_id, is_hovered): &(SharedString, bool), window, cx| {
                this.set_hovered_telemetry_card(card_id.clone(), is_hovered, window, cx);
            },
        );

        let on_toggle_tweak = cx.listener(
            |this, &(tweak_id, enabled): &(&'static str, bool), _window, cx| {
                this.toggle_tweak(tweak_id, enabled, cx);
            },
        );

        let on_change_keyboard_repeat = cx.listener(|this, preset: &str, window, cx| {
            this.select_option("keyboard_repeat", preset, window, cx);
        });

        let on_change_ctf_optimization = cx.listener(|this, preset: &str, window, cx| {
            this.select_option("ctf_optimization", preset, window, cx);
        });

        let on_change_snapkey = cx.listener(|this, preset: &str, window, cx| {
            this.select_option("snapkey", preset, window, cx);
        });

        let on_change_pal = cx.listener(|this, palette: &str, window, cx| {
            this.select_option("palette", palette, window, cx);
        });

        let on_change_lang = cx.listener(|this, lang: &str, window, cx| {
            this.select_option("language", lang, window, cx);
        });

        let on_change_th = cx.listener(|this, mode: &str, window, cx| {
            this.select_option("theme", mode, window, cx);
        });

        let on_change_trans = cx.listener(|this, enabled: &bool, _window, cx| {
            this.set_transparency(*enabled, cx);
        });

        let on_toggle_min_tray = cx.listener(|this, enabled: &bool, _window, cx| {
            this.toggle_minimize_to_tray(*enabled, cx);
        });

        let on_toggle_autostart = cx.listener(|this, enabled: &bool, _window, cx| {
            this.toggle_autostart(*enabled, cx);
        });

        let on_toggle_autostart_tray = cx.listener(|this, enabled: &bool, _window, cx| {
            this.toggle_autostart_to_tray(*enabled, cx);
        });

        let on_change_disc = cx.listener(|this, act: &str, window, cx| {
            this.change_discord_rpc(act, window, cx);
        });

        let on_select_gpu_engine = cx.listener(
            |this, &(gpu_id, slot_idx, engine): &(usize, usize, &'static str), _window, cx| {
                this.set_gpu_engine_slot(gpu_id, slot_idx, engine, cx);
            },
        );

        let on_reset_gpu_slots = cx.listener(|this, &gpu_id: &usize, _window, cx| {
            this.reset_gpu_engine_slots(gpu_id, cx);
        });

        let on_toggle_drop = cx.listener(|this, &name: &&'static str, window, cx| {
            this.toggle_dropdown(name, window, cx);
        });

        let on_hover_drop = cx.listener(
            |this, &(name, is_hovered): &(&'static str, bool), window, cx| {
                this.set_hovered_dropdown(name, is_hovered, window, cx);
            },
        );

        let on_hover_opt = cx.listener(
            |this,
             &(dropdown, opt, is_hovered): &(&'static str, &'static str, bool),
             window,
             cx| {
                this.set_hovered_option(dropdown, opt, is_hovered, window, cx);
            },
        );

        let on_close_drop = cx.listener(|this, _event: &(), window, cx| {
            this.close_dropdowns(window, cx);
        });

        let on_navigate_page = cx.listener(|this, route: &AppRoute, window, cx| {
            this.navigate_to(*route, window, cx);
        });

        let on_toggle_startup = cx.listener(
            |this, entry: &crate::entities::startup::StartupEntry, _window, cx| {
                this.toggle_startup(entry, cx);
            },
        );

        let on_delete_startup = cx.listener(
            |this, entry: &crate::entities::startup::StartupEntry, _window, cx| {
                this.delete_startup(entry, cx);
            },
        );

        let on_open_startup_folder = cx.listener(
            |_this, entry: &crate::entities::startup::StartupEntry, _window, _cx| {
                crate::entities::startup::open_startup_file_location(entry);
            },
        );

        let on_open_startup_source = cx.listener(
            |_this, entry: &crate::entities::startup::StartupEntry, _window, _cx| {
                crate::entities::startup::open_startup_source_manager(entry);
            },
        );

        let on_copy_startup_path = cx.listener(
            |_this, entry: &crate::entities::startup::StartupEntry, _window, cx| {
                let path_to_copy = entry
                    .target_path
                    .as_deref()
                    .or(entry.command.as_deref())
                    .unwrap_or(&entry.raw_id);
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(path_to_copy.to_string()));
            },
        );

        let on_toggle_startup_menu = cx.listener(|this, menu_id: &Option<String>, _window, cx| {
            this.set_startup_menu(menu_id.clone(), cx);
        });

        let on_select_startup_filter = cx.listener(
            |this, filter: &Option<crate::entities::startup::StartupSource>, _window, cx| {
                this.set_startup_filter(*filter, cx);
            },
        );

        let on_change_startup_search = cx.listener(|this, query: &String, _window, cx| {
            this.set_startup_search_query(query.clone(), cx);
        });

        let on_hover_startup_search = cx.listener(|this, &hovered: &bool, _window, cx| {
            this.set_startup_search_hovered(hovered, cx);
        });

        let on_focus_startup_search = cx.listener(|this, &focused: &bool, _window, cx| {
            this.set_startup_search_focused(focused, cx);
        });

        let on_selection_startup_search =
            cx.listener(|this, selection: &Option<(usize, usize)>, _window, cx| {
                this.set_startup_search_selection(*selection, cx);
            });

        let on_hover_startup_card = cx.listener(|this, card_id: &Option<String>, _window, cx| {
            this.set_hovered_startup_card(card_id.clone(), cx);
        });

        let on_cleanup_toggle_target = cx.listener(|this, id: &String, _window, cx| {
            this.cleanup.toggle_target(id);
            cx.notify();
        });
        let on_cleanup_toggle_category =
            cx.listener(|this, category: &CleanupCategory, _window, cx| {
                this.cleanup.toggle_category(*category);
                cx.notify();
            });
        let on_cleanup_toggle_expanded =
            cx.listener(|this, category: &CleanupCategory, _window, cx| {
                this.cleanup.expanded =
                    (this.cleanup.expanded != Some(*category)).then_some(*category);
                cx.notify();
            });
        let on_cleanup_toggle_all = cx.listener(|this, _event: &(), _window, cx| {
            this.cleanup.toggle_all();
            cx.notify();
        });
        let on_cleanup_refresh = cx.listener(|this, _event: &(), _window, cx| {
            this.refresh_cleanup(cx);
        });
        let on_cleanup_clean =
            cx.listener(|this, category: &Option<CleanupCategory>, _window, cx| {
                this.clean_cleanup(*category, cx);
            });
        let on_toggle_check_updates = cx.listener(|this, enabled: &bool, _window, cx| {
            this.toggle_check_updates(*enabled, cx);
        });
        let on_check_update = cx.listener(|this, _event: &(), _window, cx| {
            this.check_for_updates(true, cx);
        });
        let on_download_and_install_update = cx.listener(|this, _event: &(), _window, cx| {
            this.download_and_install_update(cx);
        });
        let page_tooltip_listener =
            cx.listener(|this, tooltip: &Option<crate::shared::ui::TooltipState>, _window, cx| {
                this.set_active_tooltip(tooltip.clone(), cx);
            });

        let minimize_to_tray = self.config.minimize_to_tray;
        let autostart = self.config.autostart;
        let autostart_to_tray = self.config.autostart_to_tray;
        let discord_rpc = self.config.discord_rpc;
        let check_updates = self.config.check_updates;
        let update_state = &self.update_state;
        let startup_filter = self.startup_filter;
        let startup_open_menu_id = self.startup_open_menu_id.as_deref();
        let hovered_startup_card = self.hovered_startup_card.clone();
        let startup_search_focus = self
            .startup_search_focus
            .get_or_insert_with(|| cx.focus_handle())
            .clone();
        let cleanup_page = CleanupPage::new(
            self.cleanup.clone(),
            Rc::new(move |id, window, cx| {
                on_cleanup_toggle_target(&id, window, cx);
            }),
            Rc::new(move |category, window, cx| {
                on_cleanup_toggle_category(&category, window, cx);
            }),
            Rc::new(move |category, window, cx| {
                on_cleanup_toggle_expanded(&category, window, cx);
            }),
            Rc::new(move |window, cx| {
                on_cleanup_toggle_all(&(), window, cx);
            }),
            Rc::new(move |window, cx| {
                on_cleanup_refresh(&(), window, cx);
            }),
            Rc::new(move |category, window, cx| {
                on_cleanup_clean(&category, window, cx);
            }),
        );

        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(theme.main_bg)
            .border_t_1()
            .border_l_1()
            .border_color(theme.main_border)
            .rounded_tl(px(8.0))
            .overflow_hidden()
            .child(render_route(
                current_route,
                telemetry,
                windows_build,
                hovered_telemetry_card,
                current_locale,
                open_dropdown,
                self.open_dropdown_upward,
                opening_dropdown,
                closing_dropdown,
                hovered_dropdown,
                hovered_option,
                pending_selection,
                &self.gpu_engine_slots,
                minimize_to_tray,
                autostart,
                autostart_to_tray,
                discord_rpc,
                check_updates,
                update_state,
                &self.startup_entries,
                startup_filter,
                &self.startup_search_query,
                self.startup_search_focused,
                self.startup_search_hovered,
                self.startup_search_selection,
                &startup_search_focus,
                startup_open_menu_id,
                hovered_startup_card,
                cleanup_page,
                move |target_route, window, cx| {
                    on_navigate_page(&target_route, window, cx);
                },
                move |card_id, is_hovered, window, cx| {
                    on_hover_telemetry_card(&(card_id, is_hovered), window, cx);
                },
                move |tweak_id, enabled, window, cx| {
                    on_toggle_tweak(&(tweak_id, enabled), window, cx);
                },
                move |preset, window, cx| {
                    on_change_keyboard_repeat(preset, window, cx);
                },
                move |preset, window, cx| {
                    on_change_ctf_optimization(preset, window, cx);
                },
                move |preset, window, cx| {
                    on_change_snapkey(preset, window, cx);
                },
                move |pal, window, cx| {
                    on_change_pal(pal, window, cx);
                },
                move |lang, window, cx| {
                    on_change_lang(lang, window, cx);
                },
                move |mode, window, cx| {
                    on_change_th(mode, window, cx);
                },
                move |enabled, window, cx| {
                    on_change_trans(&enabled, window, cx);
                },
                move |enabled, window, cx| {
                    on_toggle_min_tray(&enabled, window, cx);
                },
                move |enabled, window, cx| {
                    on_toggle_autostart(&enabled, window, cx);
                },
                move |enabled, window, cx| {
                    on_toggle_autostart_tray(&enabled, window, cx);
                },
                move |act, window, cx| {
                    on_change_disc(act, window, cx);
                },
                move |enabled, window, cx| {
                    on_toggle_check_updates(&enabled, window, cx);
                },
                move |window, cx| {
                    on_check_update(&(), window, cx);
                },
                move |window, cx| {
                    on_download_and_install_update(&(), window, cx);
                },
                move |gpu_id, slot_idx, engine, window, cx| {
                    on_select_gpu_engine(&(gpu_id, slot_idx, engine), window, cx);
                },
                move |gpu_id, window, cx| {
                    on_reset_gpu_slots(&gpu_id, window, cx);
                },
                move |name, window, cx| {
                    on_toggle_drop(&name, window, cx);
                },
                move |name, &is_hovered, window, cx| {
                    on_hover_drop(&(name, is_hovered), window, cx);
                },
                move |dropdown, opt, &is_hovered, window, cx| {
                    on_hover_opt(&(dropdown, opt, is_hovered), window, cx);
                },
                move |window, cx| {
                    on_close_drop(&(), window, cx);
                },
                move |tt, window, cx| {
                    page_tooltip_listener(&tt, window, cx);
                },
                move |entry, window, cx| {
                    on_toggle_startup(entry, window, cx);
                },
                move |entry, window, cx| {
                    on_delete_startup(entry, window, cx);
                },
                move |entry, window, cx| {
                    on_open_startup_folder(entry, window, cx);
                },
                move |entry, window, cx| {
                    on_open_startup_source(entry, window, cx);
                },
                move |entry, window, cx| {
                    on_copy_startup_path(entry, window, cx);
                },
                move |menu_id, window, cx| {
                    on_toggle_startup_menu(&menu_id, window, cx);
                },
                move |filter, window, cx| {
                    on_select_startup_filter(&filter, window, cx);
                },
                move |q, window, cx| {
                    on_change_startup_search(&q, window, cx);
                },
                move |hov, window, cx| {
                    on_hover_startup_search(hov, window, cx);
                },
                move |foc, window, cx| {
                    on_focus_startup_search(&foc, window, cx);
                },
                move |sel, window, cx| {
                    on_selection_startup_search(&sel, window, cx);
                },
                move |card_id, window, cx| {
                    on_hover_startup_card(&card_id, window, cx);
                },
            ))
    }
}