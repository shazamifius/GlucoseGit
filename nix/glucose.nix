# Glucose Rust, empaqueté pour Nix (fiche 44, phase 3).
#
# Tout se construit hors ligne depuis `Cargo.lock` : aucune dépendance ne vient d'un dépôt git.
# Les épreuves ne tournent pas ici — la vérification automatique les joue sur chaque système —,
# et le paquet ne fait que construire l'application.
{
  lib,
  rustPlatform,
  pkg-config,
  copyDesktopItems,
  makeDesktopItem,
  gtk3,
  libxkbcommon,
  wayland,
  vulkan-loader,
  libGL,
  xorg,
}:

let
  cargo = lib.importTOML ../crates/glucose-desktop/Cargo.toml;

  # Ce que winit et wgpu ouvrent pendant l'exécution, sans le déclarer à l'édition des liens :
  # le clavier, Wayland, X11, Vulkan, OpenGL. Sous NixOS, rien de cela n'est dans un chemin
  # standard ; le binaire porte donc lui-même où les trouver.
  ouvertes = [
    libxkbcommon
    wayland
    vulkan-loader
    libGL
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];
in
rustPlatform.buildRustPackage {
  pname = "glucose";
  inherit (cargo.package) version;

  # Le code et ce qu'il faut pour le construire — ni la documentation, ni les fiches : les
  # changer ne reconstruit rien.
  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../crates
    ];
  };

  cargoLock.lockFile = ../Cargo.lock;
  cargoBuildFlags = [
    "-p"
    "glucose-desktop"
  ];
  doCheck = false;

  nativeBuildInputs = [
    pkg-config
    copyDesktopItems
  ];
  # `rfd` ouvre les dialogues de fichiers par GTK 3.
  buildInputs = [ gtk3 ] ++ ouvertes;

  postInstall = ''
    ln -s glucose-desktop $out/bin/glucose
  '';

  postFixup = ''
    patchelf --add-rpath ${lib.makeLibraryPath ouvertes} $out/bin/glucose-desktop
  '';

  desktopItems = [
    (makeDesktopItem {
      name = "glucose";
      desktopName = "Glucose";
      comment = "La toile infinie des idées et des images";
      exec = "glucose";
      categories = [ "Graphics" ];
    })
  ];

  meta = {
    description = "Glucose — la toile infinie des idées et des images, en Rust natif";
    homepage = "https://github.com/shazamifius/GlucoseGit";
    license = lib.licenses.mit;
    mainProgram = "glucose";
    platforms = lib.platforms.linux;
  };
}
