# COSMIC Color Picker Applet

A lightweight, native color picker panel applet built with Rust and `libcosmic` for the COSMIC Desktop environment.

![License](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)
![COSMIC](https://img.shields.io/badge/COSMIC-Desktop-orange.svg)

## Features

- **Native COSMIC UI:** Matches your system theme (dark/light, accent colors, and typography).
- **History Management:** Automatically keeps recent picked colors.
- **Favorites & Custom Labels:** Bookmark frequently used colors and rename them inline.
- **Instant Clipboard:** Copies color codes (`HEX`) directly to the Wayland clipboard.
- **Wayland Native:** Designed specifically for `cosmic-comp`.

## Requirements & Dependencies

Make sure you have Rust and the required runtime tools installed:

### Fedora
```bash
sudo dnf install -y rust cargo just grim slurp ImageMagick wl-clipboard
```

### Ubuntu/Pop!_OS
```bash
sudo apt update && sudo apt install -y rustc cargo just grim slurp imagemagick wl-clipboard
```

### Arch Linux
```bash
sudo pacman -S --needed rust just grim slurp imagemagick wl-clipboard
```

## Quick Installation
```bash
git clone https://github.com/fredsilveyra/cosmic-colorpicker-applet.git
cd cosmic-colorpicker-applet
```

### Option A: Install for current user (Recommended, no sudo needed)
```bash
just install-user
```

### Option B: System-wide installation
```bash
sudo just install
```

## Adding the Applet to the Panel

1. Open Settings -> Desktop -> Panel.
2. Scroll to the Applets configuration.
3. Click Add Applet and select COSMIC Color Picker.
4. Drag and position the applet wherever you prefer (e.g., next to the system clock).

**Troubleshooting:** If the applet does not immediately appear in the list, restart the panel process from your terminal:

```Bash
killall cosmic-panel
```

## Uninstall
For local user installation:
```bash
just uninstall
```

Or for system-wide:
```bash
sudo just uninstall
```

## Credits & Authors
- Fred Silveyra (@fredsilveyra)
- Gemini (Google)

## License
This project is licensed under the **GNU General Public License v3.0 or later** (GPL-3.0-or-later). See the LICENSE file for details.

## Screenshots
<img width="364" height="359" alt="image" src="https://github.com/user-attachments/assets/cc07e728-1b5b-4ab1-bc47-b6a9a2feb1f8" />

<img width="364" height="359" alt="image" src="https://github.com/user-attachments/assets/2a7fd21d-963b-4163-a7f7-a4a0367ccd81" />

<img width="364" height="412" alt="image" src="https://github.com/user-attachments/assets/939a6268-00b8-437e-9d2e-39616a47eb73" />

<img width="364" height="203" alt="image" src="https://github.com/user-attachments/assets/3826910c-e7b2-494f-9623-0c90ce53ba41" />



