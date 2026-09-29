{
  description = "Glucose — la toile infinie des idées et des images, en Rust natif";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systemes = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      pour = f: nixpkgs.lib.genAttrs systemes (system: f nixpkgs.legacyPackages.${system});
    in
    {
      # `nix run github:shazamifius/GlucoseGit` lance Glucose ; dans une configuration NixOS,
      # `inputs.glucose.packages.${system}.default` l'installe, et `nix flake update` le met à
      # jour — c'est ainsi que NixOS se met à jour (fiche 44 § 4).
      packages = pour (pkgs: rec {
        glucose = pkgs.callPackage ./nix/glucose.nix { };
        default = glucose;
      });

      # `nix develop` : de quoi construire et éprouver Glucose.
      devShells = pour (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [ self.packages.${pkgs.stdenv.hostPlatform.system}.glucose ];
          packages = with pkgs; [
            clippy
            rustfmt
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
            with pkgs;
            [
              libxkbcommon
              wayland
              vulkan-loader
              libGL
              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr
            ]
          );
        };
      });
    };
}
