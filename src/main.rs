// SPDX-License-Identifier: GPL-3.0-or-later

mod app;
mod config;
mod i18n;
mod picker;

fn main() -> cosmic::iced::Result {
    // Modo lupa: proceso aparte conectado directo al compositor (ver picker.rs).
    if std::env::args().any(|a| a == "--pick") {
        return picker::run();
    }

    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);
    cosmic::applet::run::<app::AppModel>(())
}
