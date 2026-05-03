{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    rustup
    nodejs
  ];

  buildInputs = with pkgs; [
    openssl
    glib
    gtk3
    webkitgtk_4_1
    librsvg
    cairo
    pango
    atk
    gdk-pixbuf
    libsoup_3
    dbus
    harfbuzz
    freetype
    fontconfig
    xdotool
    # extra webkit deps
    libxkbcommon
    wayland
  ];

  shellHook = ''
    export PATH="$HOME/.cargo/bin:$PATH"
    export WEBKIT_DISABLE_COMPOSITING_MODE=1
    rustup default stable 2>/dev/null || true
    echo "Atelier dev environment ready — $(rustc --version 2>/dev/null || echo 'rustc not yet installed')"
  '';
}
