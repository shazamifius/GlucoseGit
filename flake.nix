{
  description = "Glucose — surface cognitive infinie (Tauri + React)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Toolchain Rust stable + components nécessaires à Tauri
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
          targets = [ ];
        };

        # Dépendances système requises par Tauri sur Linux
        # (WebKitGTK pour le webview, GTK, librsvg pour les icônes, etc.)
        tauriBuildInputs = with pkgs; [
          glib
          gtk3
          libsoup_3
          webkitgtk_4_1
          librsvg
          openssl
          dbus
        ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
          libayatana-appindicator
        ];

        tauriNativeBuildInputs = with pkgs; [
          pkg-config
          wrapGAppsHook
        ];

        # ── Build du frontend (React/Vite) ──
        frontend = pkgs.buildNpmPackage {
          pname = "glucose-frontend";
          version = "0.3.0";
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              let
                relPath = pkgs.lib.removePrefix (toString ./. + "/") (toString path);
              in
                ! (pkgs.lib.hasPrefix "src-tauri/" relPath
                  || pkgs.lib.hasPrefix "node_modules/" relPath
                  || pkgs.lib.hasPrefix "dist/" relPath);
          };
          # NOTE : à régénérer avec `prefetch-npm-deps package-lock.json` après chaque
          # modification de package.json. Voir docs nixpkgs buildNpmPackage.
          npmDepsHash = pkgs.lib.fakeHash;
          npmBuildScript = "build";
          installPhase = ''
            mkdir -p $out
            cp -r dist/* $out/
          '';
          dontFixup = true;
        };

        # ── Glucose complet (frontend + backend Rust) ──
        glucose = pkgs.rustPlatform.buildRustPackage {
          pname = "glucose";
          version = "0.3.0";
          src = ./.;
          sourceRoot = "source/src-tauri";

          cargoLock = {
            lockFile = ./src-tauri/Cargo.lock;
          };

          # Hook pour utiliser le frontend pré-buildé
          preBuild = ''
            mkdir -p ../dist
            cp -r ${frontend}/* ../dist/
          '';

          nativeBuildInputs = tauriNativeBuildInputs;
          buildInputs = tauriBuildInputs;

          # Tauri-cli si présent dans Cargo.toml — sinon on build directement
          # le binaire Rust avec cargo build et le frontend est inline via
          # tauri::generate_context!() qui regarde dans ../dist
          cargoBuildFlags = [ "--release" ];

          meta = with pkgs.lib; {
            description = "Glucose — surface cognitive infinie (canvas avec annotations, miroirs, domaines sémantiques)";
            homepage = "https://github.com/shaza/glucose";
            license = licenses.mit;
            platforms = platforms.linux ++ platforms.darwin;
            mainProgram = "glucose";
          };
        };
      in {
        # `nix run .` ou `nix run github:shaza/glucose`
        packages.default = glucose;
        packages.glucose = glucose;

        # `nix develop` — environnement de développement avec toutes les deps
        devShells.default = pkgs.mkShell {
          name = "glucose-dev";
          packages = with pkgs; [
            rustToolchain
            bun
            nodejs_20
            cargo-tauri
            yt-dlp
          ] ++ tauriNativeBuildInputs ++ tauriBuildInputs;

          shellHook = ''
            echo "──────────────────────────────────────────"
            echo "  Glucose — environnement de développement"
            echo "──────────────────────────────────────────"
            echo "  bun run tauri dev    # lancer en mode dev"
            echo "  bun run tauri build  # build release"
            echo "  bun run test         # tests Vitest"
            echo "──────────────────────────────────────────"

            # Variables Tauri / WebKit qui peuvent être nécessaires sur certains
            # systèmes (NixOS pur en particulier)
            export WEBKIT_DISABLE_COMPOSITING_MODE=1
          '';
        };

        # `nix flake check`
        checks.glucose = glucose;

        # `nix run .#format` etc. — placeholder pour formatters
        formatter = pkgs.alejandra;
      });
}
