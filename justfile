prefix := "/usr"
bindir := prefix + "/bin"
datadir := prefix + "/share"
app_id := "io.github.fredsilveyra.CosmicExtAppletColorpicker"
destdir := env_var_or_default("DESTDIR", "")

user_prefix := env_var("HOME") + "/.local"
user_bindir := user_prefix + "/bin"
user_datadir := user_prefix + "/share"

# Verify build dependencies are available
_check-deps:
    @pkg-config --exists xkbcommon || { \
        echo "Missing build dependency: xkbcommon development headers."; \
        echo ""; \
        echo "  Fedora / RHEL:     sudo dnf install libxkbcommon-devel pkgconf-pkg-config"; \
        echo "  Debian / Ubuntu:   sudo apt install libxkbcommon-dev pkg-config"; \
        echo "  Arch / Manjaro:    sudo pacman -S libxkbcommon pkgconf"; \
        echo "  openSUSE:          sudo zypper install libxkbcommon-devel pkg-config"; \
        echo ""; \
        echo "Verify with: pkg-config --modversion xkbcommon"; \
        exit 1; \
    }

# Build optimized release binary
build: _check-deps
    cargo build --release

# Local user installation (no sudo required)
install-user: build
    install -Dm0755 "target/release/cosmic-ext-applet-colorpicker" "{{user_bindir}}/cosmic-ext-applet-colorpicker"
    install -Dm0644 "resources/{{app_id}}.desktop" "{{user_datadir}}/applications/{{app_id}}.desktop"
    -update-desktop-database "{{user_datadir}}/applications"

# System-wide installation (requires sudo)
install: build
    install -Dm0755 "target/release/cosmic-ext-applet-colorpicker" "{{destdir}}{{bindir}}/cosmic-ext-applet-colorpicker"
    install -Dm0644 "resources/{{app_id}}.desktop" "{{destdir}}{{datadir}}/applications/{{app_id}}.desktop"
    -if [ -z "{{destdir}}" ]; then update-desktop-database "{{datadir}}/applications"; fi

# Uninstall from local user directory
uninstall-user:
    rm -f "{{user_bindir}}/cosmic-ext-applet-colorpicker"
    rm -f "{{user_datadir}}/applications/{{app_id}}.desktop"
    -update-desktop-database "{{user_datadir}}/applications"

# System-wide uninstallation
uninstall:
    rm -f "{{destdir}}{{bindir}}/cosmic-ext-applet-colorpicker"
    rm -f "{{destdir}}{{datadir}}/applications/{{app_id}}.desktop"
    -if [ -z "{{destdir}}" ]; then update-desktop-database "{{datadir}}/applications"; fi
