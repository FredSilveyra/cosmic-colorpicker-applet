# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-09-23

### Changed
- Renamed the project, binary and package to `cosmic-ext-applet-colorpicker`,
  using the `cosmic-ext-` namespace recommended by the COSMIC™ trademark policy
  for third-party applets.
- Changed the application ID to `io.github.fredsilveyra.CosmicExtAppletColorpicker`.
  The previous ID used the `com.github.` prefix, which claims a namespace not under
  our control.
- Renamed the panel entry from "COSMIC Color Picker" to "Color Picker".

### Fixed
- Documented the `libxkbcommon` development headers and `pkg-config` as build
  dependencies. Their absence caused an unclear build failure in a transitive
  crate (#1).
- Filled in project metadata placeholders that were left unexpanded by the
  project template.

### Upgrading from 1.0.0
Uninstall the previous version **before** installing this one:

    just uninstall-user     # or: sudo just uninstall

Saved history and favorites are not carried over — the application ID change moves
where configuration is stored.

## [1.0.0] - 2026-09-22

First stable release.

### Added
- Magnifier loupe while picking: a circular zoom lens follows the cursor, showing the surrounding pixels magnified, a precision crosshair on the exact pixel, and a live `HEX` preview.
- Cancel picking with `Esc` or right-click.
- Multi-monitor support: the picking overlay opens on every connected output.
- Optional `grim` fallback for screen capture when the desktop portal is unavailable.
- `X-HostWaylandDisplay=true` in the desktop entry, required for the picker to connect to `cosmic-comp`.

### Changed
- Color picking now runs in a helper process (`cosmic-ext-applet-colorpicker --pick`) that connects directly to `cosmic-comp` and draws a full-screen layer-shell overlay.
- Screen capture now goes through the COSMIC desktop portal (`xdg-desktop-portal-cosmic`) and is processed in memory.
- The applet no longer blocks while waiting for a color to be picked.
- README updated with usage, architecture, and troubleshooting sections.

### Removed
- Runtime dependencies on `slurp` and ImageMagick.

### Fixed
- Uninstall instructions for local installations now use `just uninstall-user`.

## [0.1.0] - 2026-09-20

Initial release.

### Added
- Panel applet for the COSMIC™ Desktop built with `libcosmic`.
- Screen color picking using `slurp`, `grim`, and ImageMagick.
- Automatic copy of the picked `HEX` value to the Wayland clipboard, with a desktop notification.
- Recent colors history (up to 10 entries) with a clear option.
- Favorites with inline renaming, copy, and removal.
- User and system-wide installation recipes via `just`.

[1.1.0]: https://github.com/FredSilveyra/cosmic-ext-applet-colorpicker/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/FredSilveyra/cosmic-ext-applet-colorpicker/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/FredSilveyra/cosmic-ext-applet-colorpicker/releases/tag/v0.1.0
