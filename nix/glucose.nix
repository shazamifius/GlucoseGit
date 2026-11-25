# Glucose Rust, empaqueté pour Nix (fiche 44, phase 3).
#
# Tout se construit hors ligne depuis `Cargo.lock` : aucune dépendance ne vient d'un dépôt git.
# Les épreuves ne tournent pas ici — la vérification automatique les joue sur chaque système —,
# et le paquet ne fait que construire l'application.
{
  lib,
  rustPlatform,
  pkg-config,
  libxkbcommon,
  wayland,
  vulkan-loader,
  libGL,
  xorg,
  gst_all_1,
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
    # Les vidéos (VIDEO-1, fiche 169) : GStreamer et sa bibliothèque `app` (appsink, appsrc),
    # ouverts seulement quand une vidéo se lit — absents, Glucose démarre et montre l'image fixe.
    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
  ];

  # Les greffons que GStreamer charge lui-même (démultiplexeurs, décodeurs, sortie du son) :
  # sous NixOS, aucun dossier standard ne les rassemble ; le binaire porte où les chercher,
  # comme il porte le chemin de ses bibliothèques.
  greffons = [
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-good
    gst_all_1.gst-plugins-bad
    gst_all_1.gst-libav
  ];
  cheminDesGreffons = lib.makeSearchPathOutput "lib" "lib/gstreamer-1.0" greffons;
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

  nativeBuildInputs = [ pkg-config ];
  # Aucune bibliothèque à l'édition des liens : `rfd` ouvre ses dialogues par le portail du bureau
  # (`xdg-desktop-portal`, en D-Bus), et non plus par GTK 3 (fiche 100) ; le reste s'ouvre au
  # lancement, depuis le chemin que le binaire porte.
  buildInputs = ouvertes;
  env.GLUCOSE_GREFFONS_GST = cheminDesGreffons;

  # Ce que posent aussi le .deb, le .rpm et l'AppImage (`outils/paquets`), depuis les mêmes
  # fichiers : un seul raccourci de bureau, qui ouvre un .glucose d'un double-clic, et le type
  # des .glucose (fiche 90, OUVRIR-1).
  postInstall = ''
    ln -s glucose-desktop $out/bin/glucose
    install -Dm644 ${../outils/paquets/Glucose.desktop} $out/share/applications/glucose.desktop
    install -Dm644 ${../outils/paquets/glucose-mime.xml} $out/share/mime/packages/glucose.xml
    for taille in 32x32 128x128 256x256; do
      install -Dm644 ${../outils/paquets/icones}/$taille.png \
        $out/share/icons/hicolor/$taille/apps/glucose.png
    done
  '';

  postFixup = ''
    patchelf --add-rpath ${lib.makeLibraryPath ouvertes} $out/bin/glucose-desktop
  '';

  # Ce que `nix develop` reprend : une seule liste, ici.
  passthru = { inherit ouvertes cheminDesGreffons; };

  meta = {
    description = "Glucose — la toile infinie des idées et des images, en Rust natif";
    homepage = "https://github.com/shazamifius/GlucoseGit";
    license = lib.licenses.mit;
    mainProgram = "glucose";
    platforms = lib.platforms.linux;
  };
}
