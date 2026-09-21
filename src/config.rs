// ============================================================================
// COSMIC Colorpicker Applet
//
// Author: Fred Silveyra (@fredsilveyra)
// Co-author & Technical Assistance: Gemini (Google)
// License: GPL-3.0-or-later
// Repository: https://github.com/fredsilveyra/cosmic-colorpicker-applet
// ============================================================================

// SPDX-License-Identifier: GPL-3.0-or-later

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavoriteColor {
    pub name: String,
    pub hex: String,
}

#[derive(Clone, Debug, PartialEq, Eq, CosmicConfigEntry, Serialize, Deserialize)]
#[version = 1]
pub struct Config {
    pub history: Vec<String>,
    pub favorites: Vec<FavoriteColor>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            history: Vec::new(), // Starts empty for new users
            favorites: vec![
                FavoriteColor {
                    name: "Nightshade".into(),
                    hex: "#332A57".into(),
                },
            ],
        }
    }
}
