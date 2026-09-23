## COSMIC Color Picker Applet

A lightweight, native color picker panel applet built with Rust and `libcosmic` for the COSMIC Desktop environment.

![License](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)
![COSMIC](https://img.shields.io/badge/COSMIC-Desktop-orange.svg)

<p align="center">
  <img width="400" alt="Opening the applet" src="https://github.com/user-attachments/assets/317a12ab-f96f-460a-be7c-98d600a16e91" />
  <img width="400" alt="Picking a color with the loupe" src="https://github.com/user-attachments/assets/7edf3963-7a42-4a18-a147-3ca8d2ff291a" />
</p>
<p align="center">
  <img width="400" alt="Managing favorites" src="https://github.com/user-attachments/assets/d5a2d1bb-8a32-4c9a-b762-3030572b9a43" />
  <img width="400" alt="Recent colors history" src="https://github.com/user-attachments/assets/d578b7d2-27e2-4c97-ba59-7fba7fed6034" />
</p>


## Features

- **Magnifier Loupe:** A circular zoom lens follows your cursor while picking, showing the surrounding pixels magnified, a precision crosshair on the exact pixel, and a live `HEX` preview.
- **Native COSMIC UI:** Matches your system theme (dark/light, accent colors, and typography).
- **History Management:** Automatically keeps recent picked colors.
- **Favorites & Custom Labels:** Bookmark frequently used colors and rename them inline.
- **Instant Clipboard:** Copies color codes (`HEX`) directly to the Wayland clipboard.
- **Wayland Native:** Designed specifically for `cosmic-comp`. Screen capture goes through the COSMIC desktop portal, with no third-party picking tools required.

## Requirements & Dependencies

Make sure you have Rust and the required runtime tools installed. The screen capture uses `xdg-desktop-portal-cosmic`, which ships with COSMIC by default.

### Fedora
```bash
sudo dnf install -y rust cargo just wl-clipboard
```

### Ubuntu/Pop!_OS
```bash
sudo apt update && sudo apt install -y rustc cargo just wl-clipboard
```

### Arch Linux
```bash
sudo pacman -S --needed rust just wl-clipboard
```

**Optional:** if the desktop portal is unavailable, the applet falls back to `grim` for screen capture when it is installed.

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

After installing or updating, restart the panel so it picks up the applet's desktop entry:

```bash
killall cosmic-panel
```

## Adding the Applet to the Panel

1. Open Settings -> Desktop -> Panel.
2. Scroll to the Applets configuration.
3. Click Add Applet and select COSMIC Color Picker.
4. Drag and position the applet wherever you prefer (e.g., next to the system clock).

## Usage

1. Click the applet icon in the panel and press **Pick Screen Color**.
2. Move the cursor over the screen. The loupe shows the magnified pixels and the `HEX` value under the crosshair.
3. **Left-click** to pick the color. It is copied to the clipboard and added to your recent colors.
4. Press **Esc** or **right-click** to cancel.

## How It Works

COSMIC panel applets run inside the panel's own nested compositor, which only forwards popups, so an applet cannot draw over the whole screen by itself. When you start picking, the applet launches a small helper process (`cosmic-colorpicker-applet --pick`) that connects directly to `cosmic-comp`. The helper:

1. Captures the screen into memory through the COSMIC desktop portal.
2. Opens a transparent full-screen overlay using the Wayland layer-shell protocol.
3. Draws the loupe on a canvas that follows the cursor and returns the picked `HEX` value to the applet.

## Troubleshooting

**The applet does not appear in the list:** restart the panel process with `killall cosmic-panel`.

**The loupe does not open:** make sure the installed desktop entry includes `X-HostWaylandDisplay=true` (it is included by default) and restart the panel. You can also run the helper directly from a terminal to see any error messages:

```bash
cosmic-colorpicker-applet --pick
```

**The loupe feels slow or leaves trails:** check which renderer is being used by forcing each one and comparing:

```bash
ICED_BACKEND=wgpu cosmic-colorpicker-applet --pick
ICED_BACKEND=tiny-skia cosmic-colorpicker-applet --pick
```

## Uninstall
For local user installation:
```bash
just uninstall-user
```

Or for system-wide:
```bash
sudo just uninstall
```

## Credits & Authors
- Fred Silveyra (@fredsilveyra)
- Claude (Anthropic)
- Gemini (Google)

## License
This project is licensed under the **GNU General Public License v3.0 or later** (GPL-3.0-or-later). See the LICENSE file for details.

## Changelog
See [CHANGELOG.md](CHANGELOG.md) for release notes.


## Screenshots

<!-- Add a screenshot or GIF of the magnifier loupe here -->

<img width="364" height="359" alt="image" src="https://github.com/user-attachments/assets/cc07e728-1b5b-4ab1-bc47-b6a9a2feb1f8" />

<img width="364" height="359" alt="image" src="https://github.com/user-attachments/assets/2a7fd21d-963b-4163-a7f7-a4a0367ccd81" />

<img width="364" height="412" alt="image" src="https://github.com/user-attachments/assets/939a6268-00b8-437e-9d2e-39616a47eb73" />

<img width="364" height="203" alt="image" src="https://github.com/user-attachments/assets/3826910c-e7b2-494f-9623-0c90ce53ba41" />
