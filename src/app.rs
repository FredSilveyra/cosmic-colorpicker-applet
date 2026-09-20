use crate::config::Config;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{window::Id, Color, Limits, Subscription, Task};
use cosmic::prelude::*;
use cosmic::widget::{self, container};
use std::process::Command as StdCommand;

pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    config: Config,
    history: Vec<String>,
    favorites: Vec<String>,
}

impl Default for AppModel {
    fn default() -> Self {
        Self {
            core: cosmic::Core::default(),
            popup: None,
            config: Config::default(),
            history: vec![
                "#FFFFFF".into(),
                "#1E1E2E".into(),
                "#89B4FA".into(),
                "#A6E3A1".into(),
            ],
            favorites: vec!["#F38BA8".into()],
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
    ClearHistory,
}

/// Convierte una cadena "#RRGGBB" a un cosmic::iced::Color
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

    const APP_ID: &'static str = "com.github.fredsilveyra.cosmic-colorpicker-applet";

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
        let app = AppModel {
            core,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                 Err((_errors, config)) => config,
            })
            .unwrap_or_default(),
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // Icono en la barra del panel
        self.core
        .applet
        .icon_button("color-select-symbolic")
        .on_press(Message::TogglePopup)
        .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let mut list = widget::list_column().add(
            widget::button::standard("Tomar color de pantalla")
            .leading_icon(widget::icon::from_name("color-select-symbolic"))
            .on_press(Message::PickColor),
        );

        // Sección: Favoritos
        if !self.favorites.is_empty() {
            list = list.add(widget::text::title4("Favoritos"));
            for color_hex in &self.favorites {
                list = list.add(self.render_color_row(color_hex, true));
            }
        }

        // Sección: Historial
        list = list.add(widget::divider::horizontal::default());
        list = list.add(
            widget::row(vec![
                widget::text::title4("Historial").into(),
                        widget::Space::new().width(cosmic::iced::Length::Fill).into(),
                        widget::button::text("Limpiar")
                        .on_press(Message::ClearHistory)
                        .into(),
            ])
            .align_y(cosmic::iced::Alignment::Center),
        );

        if self.history.is_empty() {
            list = list.add(widget::text::caption("Sin colores en el historial"));
        } else {
            for color_hex in &self.history {
                let is_fav = self.favorites.contains(color_hex);
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
                    .min_width(320.0)
                    .min_height(100.0)
                    .max_height(600.0);
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
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
                        tokio::task::spawn_blocking(|| {
                            let slurp_out = StdCommand::new("slurp")
                            .arg("-p")
                            .output()
                            .ok()?;
                            if !slurp_out.status.success() {
                                return None;
                            }
                            let coords = String::from_utf8(slurp_out.stdout).ok()?;
                            let coords = coords.trim();
                            if coords.is_empty() {
                                return None;
                            }

                            let grim = StdCommand::new("grim")
                            .args(["-g", coords, "-t", "ppm", "-"])
                            .stdout(std::process::Stdio::piped())
                            .spawn()
                            .ok()?;

                            let magick = StdCommand::new("magick")
                            .args(["-", "-format", "%[hex:p{0,0}]", "info:"])
                            .stdin(grim.stdout?)
                            .output()
                            .ok()?;

                            if !magick.status.success() {
                                return None;
                            }

                            let raw_hex = String::from_utf8(magick.stdout).ok()?;
                            let hex = raw_hex.trim();
                            if hex.len() >= 6 {
                                let final_hex = format!("#{}", &hex[..6].to_uppercase());

                                let _ = StdCommand::new("wl-copy").arg(&final_hex).status();
                                let _ = StdCommand::new("notify-send")
                                .args([
                                    "Color Picker",
                                    &format!("Copiado: {}", final_hex),
                                      "-i",
                                      "color-select",
                                ])
                                .status();

                                Some(final_hex)
                            } else {
                                None
                            }
                        })
                        .await
                        .ok()?
                    },
                    |res| cosmic::Action::App(Message::ColorPicked(res)),
                );

                return Task::batch(vec![close_task, pick_task]);
            }
            Message::ColorPicked(Some(hex)) => {
                self.history.retain(|h| h != &hex);
                self.history.insert(0, hex);
                if self.history.len() > 10 {
                    self.history.truncate(10);
                }
            }
            Message::ColorPicked(None) => {}
            Message::CopyColor(hex) => {
                let _ = StdCommand::new("wl-copy").arg(&hex).status();
                let _ = StdCommand::new("notify-send")
                .args([
                    "Color Picker",
                    &format!("Copiado al portapapeles: {}", hex),
                      "-i",
                      "color-select",
                ])
                .status();
            }
            Message::ToggleFavorite(hex) => {
                if let Some(pos) = self.favorites.iter().position(|h| h == &hex) {
                    self.favorites.remove(pos);
                } else {
                    self.favorites.push(hex);
                }
            }
            Message::ClearHistory => {
                self.history.clear();
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
    /// Renderiza una fila con el cuadrito de color, el código HEX, botón favorito y botón copiar
    fn render_color_row<'a>(&self, hex: &'a str, is_fav: bool) -> Element<'a, Message> {
        let color = parse_hex_color(hex);
        let hex_copy = hex.to_string();
        let hex_fav = hex.to_string();

        // Cuadrito con el color real (24x24 px)
        let color_box = container(widget::Space::new())
        .width(cosmic::iced::Length::Fixed(24.0))
        .height(cosmic::iced::Length::Fixed(24.0))
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
                    widget::Space::new().width(cosmic::iced::Length::Fill).into(),
                    widget::button::icon(widget::icon::from_name(star_icon))
                    .on_press(Message::ToggleFavorite(hex_fav))
                    .into(),
                    widget::button::icon(widget::icon::from_name("edit-copy-symbolic"))
                    .on_press(Message::CopyColor(hex_copy))
                    .into(),
        ])
        .spacing(8)
        .align_y(cosmic::iced::Alignment::Center)
        .into()
    }
}
