prefix := "/usr/local"
bindir := prefix + "/bin"
datadir := prefix + "/share"
app_id := "com.github.fredsilveyra.cosmic-colorpicker-applet"

# Build binary in release mode
build:
    cargo build --release

# Install locally for the current user (no sudo required)
install-user: build
    @echo "Stopping previous instances..."
    -killall -q cosmic-colorpicker-applet || true
    @echo "Installing binary to ~/.local/bin..."
    install -Dm0755 "target/release/cosmic-colorpicker-applet" "$$HOME/.local/bin/cosmic-colorpicker-applet"
    @echo "Restarting COSMIC Panel..."
    -killall -q -9 cosmic-panel || true
    @echo "Installation completed successfully!"

# System-wide installation (requires sudo)
install: build
    @echo "Stopping previous instances..."
    -killall -q cosmic-colorpicker-applet || true
    @echo "Installing binary to {{bindir}}..."
    install -Dm0755 "target/release/cosmic-colorpicker-applet" "{{DESTDIR}}{{bindir}}/cosmic-colorpicker-applet"
    @echo "Installing desktop entry..."
    install -Dm0644 "resources/{{app_id}}.desktop" "{{DESTDIR}}{{datadir}}/applications/{{app_id}}.desktop"
    @echo "Restarting COSMIC Panel..."
    -killall -q -9 cosmic-panel || true
    @echo "System-wide installation completed successfully!"

# Uninstall from system and user directories
uninstall:
    rm -f "{{DESTDIR}}{{bindir}}/cosmic-colorpicker-applet"
    rm -f "{{DESTDIR}}{{datadir}}/applications/{{app_id}}.desktop"
    rm -f "$$HOME/.local/bin/cosmic-colorpicker-applet"
    -killall -q -9 cosmic-panel || true
    @echo "Applet removed successfully."
