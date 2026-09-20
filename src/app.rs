use crate::config::Config;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{window::Id, Limits, Subscription, Task};
use cosmic::prelude::*;
use cosmic::widget;
use std::process::Command as StdCommand;

pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    config: Config,
    history: Vec<String>,
}

impl Default for AppModel {
    fn default() -> Self {
        Self {
            core: cosmic::Core::default(),
            popup: None,
            config: Config::default(),
            history: vec!["#FFFFFF".into(), "#000000".into(), "#1E1E2E".into()],
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
    ClearHistory,
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
        self.core
        .applet
        .icon_button("color-select-symbolic")
        .on_press(Message::TogglePopup)
        .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let mut list = widget::list_column()
        .add(
            widget::button::text("Tomar color de pantalla")
            .on_press(Message::PickColor),
        );

        if self.history.is_empty() {
            list = list.add(widget::text::caption("Sin colores en el historial"));
        } else {
            list = list.add(
                widget::button::text("Limpiar historial")
                .on_press(Message::ClearHistory),
            );

            for color_hex in &self.history {
                let hex_copy = color_hex.clone();
                let item = widget::button::text(color_hex.as_str())
                .on_press(Message::CopyColor(hex_copy));
                list = list.add(item);
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
                    .max_width(372.0)
                    .min_width(260.0)
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
                // Cerrar popup para que no obstruya la selección en pantalla
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
                    &format!("Copiado: {}", hex),
                      "-i",
                      "color-select",
                ])
                .status();
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
