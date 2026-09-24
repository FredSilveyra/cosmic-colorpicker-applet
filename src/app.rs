// ============================================================================
// COSMIC Colorpicker Applet
//
// Author: Fred Silveyra (@fredsilveyra)
// Co-author & Technical Assistance: Claude (Anthropic), Gemini (Google)
// License: GPL-3.0-or-later
// Repository: https://github.com/FredSilveyra/cosmic-ext-applet-colorpicker
// ============================================================================

// SPDX-License-Identifier: GPL-3.0-or-later

use crate::config::{Config, FavoriteColor};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{window::Id, Alignment, Color, Length, Limits, Subscription, Task};
use cosmic::prelude::*;
use cosmic::widget::{self, container};
use std::process::Command as StdCommand;

pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    config_handler: Option<cosmic_config::Config>,
    config: Config,
    editing_index: Option<usize>,
    edit_input_value: String,
}

impl Default for AppModel {
    fn default() -> Self {
        Self {
            core: cosmic::Core::default(),
            popup: None,
            config_handler: None,
            config: Config::default(),
            editing_index: None,
            edit_input_value: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    UpdateConfig(Config),
    PickColor,
    ColorPicked(Option<String>),
    CopyColor(String),
    ToggleFavorite(String),
    StartRename(usize, String),
    FavoriteNameInput(String),
    SaveRename(usize),
    CancelRename,
    RemoveFavorite(usize),
    ClearHistory,
}

fn parse_hex_color(hex: &str) -> Color {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(255) as f32 / 255.0;
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(255) as f32 / 255.0;
        Color::from_rgb(r, g, b)
    } else {
        Color::WHITE
    }
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "io.github.fredsilveyra.CosmicExtAppletColorpicker";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let config_handler = cosmic_config::Config::new(Self::APP_ID, Config::VERSION).ok();
        let config = if let Some(ref handler) = config_handler {
            match Config::get_entry(handler) {
                Ok(c) => c,
                Err((_errors, c)) => c,
            }
        } else {
            Config::default()
        };

        let app = AppModel {
            core,
            config_handler,
            config,
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("color-select-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let mut list = widget::list_column().add(
            widget::button::standard("Pick Screen Color")
                .leading_icon(widget::icon::from_name("color-select-symbolic"))
                .on_press(Message::PickColor),
        );

        // Favorites Section
        if !self.config.favorites.is_empty() {
            list = list.add(widget::text::title4("Favorites"));
            for (idx, fav) in self.config.favorites.iter().enumerate() {
                let is_editing = self.editing_index == Some(idx);
                list = list.add(self.render_favorite_row(idx, fav, is_editing));
            }
        }

        // History Section
        list = list.add(widget::divider::horizontal::default());
        list = list.add(
            widget::row(vec![
                widget::text::title4("Recent Colors").into(),
                        widget::Space::new().width(Length::Fill).into(),
                        widget::button::text("Clear")
                            .on_press(Message::ClearHistory)
                            .into(),
            ])
                .align_y(Alignment::Center),
        );

        if self.config.history.is_empty() {
            list = list.add(widget::text::caption("No recent colors"));
        } else {
            for color_hex in &self.config.history {
                let is_fav = self.config.favorites.iter().any(|f| &f.hex == color_hex);
                list = list.add(self.render_color_row(color_hex, is_fav));
            }
        }

        self.core.applet.popup_container(list).into()
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                            new_id,
                            None,
                            None,
                            None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(380.0)
                        .min_width(360.0)
                        .min_height(100.0)
                        .max_height(600.0);
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                    self.editing_index = None;
                    self.edit_input_value.clear();
                }
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::PickColor => {
                let close_task = if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    Task::none()
                };

                let pick_task = Task::perform(
                    async move {
                        // La lupa corre en un proceso aparte: el applet vive dentro del
                        // compositor anidado de cosmic-panel y no puede crear superficies
                        // layer-shell. El hijo se conecta directo a cosmic-comp.
                        let exe = std::env::current_exe().ok()?;
                        let out = tokio::process::Command::new(exe)
                            .arg("--pick")
                            .env_remove("WAYLAND_SOCKET")
                            .env_remove("X_PRIVILEGED_WAYLAND_SOCKET")
                            .stdin(std::process::Stdio::null())
                            .output()
                            .await
                            .ok()?;
                        if !out.status.success() {
                            return None;
                        }
                        let stdout = String::from_utf8(out.stdout).ok()?;
                        let hex = stdout.lines().last()?.trim().to_string();
                        if hex.len() != 7 || !hex.starts_with('#') {
                            return None;
                        }

                        let _ = StdCommand::new("wl-copy").arg(&hex).status();
                        let _ = StdCommand::new("notify-send")
                        .args([
                            "Color Picker",
                            &format!("Copied to clipboard: {}", hex),
                              "-i",
                              "color-select",
                        ])
                            .status();
                        Some(hex)
                    },
                    |res| cosmic::Action::App(Message::ColorPicked(res)),
                );

                return Task::batch(vec![close_task, pick_task]);
            }
            Message::ColorPicked(Some(hex)) => {
                self.config.history.retain(|h| h != &hex);
                self.config.history.insert(0, hex);
                if self.config.history.len() > 10 {
                    self.config.history.truncate(10);
                }
                self.save_config();
            }
            Message::ColorPicked(None) => {}
            Message::CopyColor(hex) => {
                let _ = StdCommand::new("wl-copy").arg(&hex).status();
                let _ = StdCommand::new("notify-send")
                    .args([
                        "Color Picker",
                        &format!("Copied to clipboard: {}", hex),
                        "-i",
                        "color-select",
                    ])
                    .status();
            }
            Message::ToggleFavorite(hex) => {
                if let Some(pos) = self.config.favorites.iter().position(|f| f.hex == hex) {
                    self.config.favorites.remove(pos);
                } else {
                    self.config.favorites.push(FavoriteColor {
                        name: "Custom Color".into(),
                            hex,
                    });
                }
                self.save_config();
            }
            Message::StartRename(idx, current_name) => {
                self.editing_index = Some(idx);
                self.edit_input_value = current_name;
            }
            Message::FavoriteNameInput(val) => {
                self.edit_input_value = val;
            }
            Message::SaveRename(idx) => {
                if let Some(fav) = self.config.favorites.get_mut(idx) {
                    let trimmed = self.edit_input_value.trim();
                    if !trimmed.is_empty() {
                        fav.name = trimmed.to_string();
                        self.save_config();
                    }
                }
                self.editing_index = None;
                self.edit_input_value.clear();
            }
            Message::CancelRename => {
                self.editing_index = None;
                self.edit_input_value.clear();
            }
            Message::RemoveFavorite(idx) => {
                if idx < self.config.favorites.len() {
                    self.config.favorites.remove(idx);
                    self.save_config();
                }
            }
            Message::ClearHistory => {
                self.config.history.clear();
                self.save_config();
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.core()
            .watch_config::<Config>(Self::APP_ID)
            .map(|update| Message::UpdateConfig(update.config))
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

impl AppModel {
    fn save_config(&self) {
        if let Some(ref handler) = self.config_handler {
            let _ = self.config.write_entry(handler);
        }
    }

    fn render_favorite_row<'a>(&'a self, idx: usize, fav: &'a FavoriteColor, is_editing: bool) -> Element<'a, Message> {
        let color = parse_hex_color(&fav.hex);
        let hex_copy = fav.hex.clone();

        let color_box = container(widget::Space::new())
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .style(move |_theme| container::Style {
            background: Some(color.into()),
               border: cosmic::iced::border::rounded(4.0),
               ..Default::default()
        });

        if is_editing {
            widget::row(vec![
                color_box.into(),
                widget::text_input("Color name...", &self.edit_input_value)
                    .on_input(Message::FavoriteNameInput)
                    .on_submit(move |_| Message::SaveRename(idx))
                    .width(Length::Fill)
                    .into(),
                widget::button::standard("Save")
                    .on_press(Message::SaveRename(idx))
                    .into(),
                widget::button::icon(widget::icon::from_name("window-close-symbolic"))
                    .on_press(Message::CancelRename)
                    .into(),
            ])
            .spacing(6)
            .align_y(Alignment::Center)
            .into()
        } else {
            // Columna central con Nombre arriba y HEX abajo
            let text_info = widget::column::with_children(vec![
                widget::text::body(&fav.name)
                    .wrapping(cosmic::iced::widget::text::Wrapping::None)
                    .into(),
                    widget::text::caption(&fav.hex)
                        .into(),
            ])
            .spacing(2)
            .width(Length::Fill);

            // Fila de acciones a la derecha
            let actions = widget::row::with_children(vec![
                widget::button::icon(widget::icon::from_name("accessories-text-editor-symbolic"))
                    .on_press(Message::StartRename(idx, fav.name.clone()))
                    .into(),
                widget::button::icon(widget::icon::from_name("edit-copy-symbolic"))
                    .on_press(Message::CopyColor(hex_copy))
                    .into(),
                widget::button::icon(widget::icon::from_name("user-trash-symbolic"))
                    .on_press(Message::RemoveFavorite(idx))
                    .into(),
            ])
            .spacing(4)
            .align_y(Alignment::Center);

            widget::row::with_children(vec![
                color_box.into(),
                text_info.into(),
                actions.into(),
            ])
            .spacing(8)
            .align_y(Alignment::Center)
            .into()
        }
    }

    fn render_color_row<'a>(&'a self, hex: &'a str, is_fav: bool) -> Element<'a, Message> {
        let color = parse_hex_color(hex);
        let hex_copy = hex.to_string();
        let hex_fav = hex.to_string();

        let color_box = container(widget::Space::new())
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .style(move |_theme| container::Style {
                background: Some(color.into()),
                border: cosmic::iced::border::rounded(4.0),
                ..Default::default()
            });

        let star_icon = if is_fav {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        };

        widget::row(vec![
            color_box.into(),
                    widget::text::body(hex).into(),
                    widget::Space::new().width(Length::Fill).into(),
                    widget::button::icon(widget::icon::from_name(star_icon))
                        .on_press(Message::ToggleFavorite(hex_fav))
                        .into(),
                    widget::button::icon(widget::icon::from_name("edit-copy-symbolic"))
                        .on_press(Message::CopyColor(hex_copy))
                        .into(),
        ])
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    }
}
