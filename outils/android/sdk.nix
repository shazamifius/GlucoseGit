# **Le SDK d'Android de ce poste** (fiche 171) : de quoi construire l'APK ici et lancer un téléphone
# virtuel — rien de cela n'existait sous NixOS, et le Java de Glucose ne se compilait que sur la CI.
#
#   nix build --impure --expr 'import ./outils/android/sdk.nix {}' -o ~/.cache/glucose-android/sdk
#
# Le SDK est celui de `construire.sh` (ANDROID_HOME=<sortie>/libexec/android-sdk) : la plateforme 36
# (`compileSdk`), le NDK de `ndkVersion`, l'émulateur et l'image d'un téléphone x86_64 de Google.
# Sa licence s'accepte ici, comme la CI l'accepte en installant le SDK.
# Le nixpkgs est celui que `flake.lock` fixe, lu dans le verrou et jamais par le flake : celui-ci,
# chemin d'un dossier, se copierait dans le magasin de Nix avec ce que Git ignore — `target/` et ses
# dizaines de Go. Le 25/11, cette copie a rempli le disque.
let
  verrou = builtins.fromJSON (builtins.readFile ../../flake.lock);
  nixpkgs = builtins.fetchTree verrou.nodes.${verrou.nodes.${verrou.root}.inputs.nixpkgs}.locked;
in
{
  pkgs ? import nixpkgs {
    system = builtins.currentSystem;
    config = {
      android_sdk.accept_license = true;
      allowUnfree = true;
    };
  },
}:
(pkgs.androidenv.composeAndroidPackages {
  platformVersions = [ "36" ];
  buildToolsVersions = [ "36.0.0" ];
  includeNDK = true;
  ndkVersions = [ "29.0.14206865" ];
  includeEmulator = true;
  includeSystemImages = true;
  systemImageTypes = [ "google_apis" ];
  abiVersions = [ "x86_64" ];
}).androidsdk
