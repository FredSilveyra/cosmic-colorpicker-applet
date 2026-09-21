prefix := "/usr"
bindir := prefix + "/bin"
datadir := prefix + "/share"
app_id := "com.github.fredsilveyra.cosmic-colorpicker-applet"
destdir := env_var_or_default("DESTDIR", "")

user_prefix := env_var("HOME") + "/.local"
user_bindir := user_prefix + "/bin"
user_datadir := user_prefix + "/share"

# Build optimized release binary
build:
    cargo build --release

# Local user installation (no sudo required)
install-user: build
    install -Dm0755 "target/release/cosmic-colorpicker-applet" "{{user_bindir}}/cosmic-colorpicker-applet"
    install -Dm0644 "resources/{{app_id}}.desktop" "{{user_datadir}}/applications/{{app_id}}.desktop"
    -update-desktop-database "{{user_datadir}}/applications"

# System-wide installation (requires sudo)
install: build
    install -Dm0755 "target/release/cosmic-colorpicker-applet" "{{destdir}}{{bindir}}/cosmic-colorpicker-applet"
    install -Dm0644 "resources/{{app_id}}.desktop" "{{destdir}}{{datadir}}/applications/{{app_id}}.desktop"
    -if [ -z "{{destdir}}" ]; then update-desktop-database "{{datadir}}/applications"; fi

# Uninstall from local user directory
uninstall-user:
    rm -f "{{user_bindir}}/cosmic-colorpicker-applet"
    rm -f "{{user_datadir}}/applications/{{app_id}}.desktop"
    -update-desktop-database "{{user_datadir}}/applications"

# System-wide uninstallation
uninstall:
    rm -f "{{destdir}}{{bindir}}/cosmic-colorpicker-applet"
    rm -f "{{destdir}}{{datadir}}/applications/{{app_id}}.desktop"
    -if [ -z "{{destdir}}" ]; then update-desktop-database "{{datadir}}/applications"; fi
