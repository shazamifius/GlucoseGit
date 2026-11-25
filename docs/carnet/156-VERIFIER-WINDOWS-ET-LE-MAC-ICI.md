# 156 — Vérifier Windows et le Mac ici (25/11 vers 2 h 40, le relais, compte a)

## 1. D'où il vient

Le plan du relais, point 10. Une faute dans ce qui ne se compile que pour Windows (`#[cfg(windows)]` :
le glisser par COM, la voie WinHTTP) ou pour le Mac n'apparaissait qu'à la CI d'une publication.
L'ombre des nœuds sous Windows (fiche 155 § 3) attendait : du COM jamais compilé avant la CI serait un
pari.

## 2. Ce qui est fait

**`outils/verifier_cibles.sh [windows|mac]`** : le `clippy` de la CI (`--workspace --all-targets -D
warnings`) pour `x86_64-pc-windows-msvc` et `aarch64-apple-darwin`. Le 25/11, à froid : **Windows 66
s, le Mac 35 s** ; puis une seconde quand rien n'a changé. MÉTHODE § 5 dit quand s'en servir.

**Sans un second Rust.** Le `rustc` de nixpkgs n'apporte que la bibliothèque standard de Linux.
`-Zbuild-std=std,panic_abort` la compile pour la cible, depuis ses sources (`rustPlatform.rustLibSrc`,
du même nixpkgs verrouillé que `nix develop`) ; `RUSTC_BOOTSTRAP=1` ouvre les `-Z` à un compilateur
stable ; `__CARGO_TESTS_ONLY_SRC_ROOT` montre ces sources à Cargo, qui les cherche dans le dossier de
`rustc`, où nixpkgs ne les met pas.

* **Écarté : fenix ou rust-overlay** (la voie que donnent le wiki de NixOS et le README de fenix) :
  une chaîne qui porte la bibliothèque toute faite des cibles — mais un second Rust, d'une autre
  version que celui du dépôt, une entrée de plus au flake, et des centaines de mégaoctets. Il reste la
  sortie si Cargo retire son nom interne : l'outil tomberait alors en réclamant « rust-src », jamais
  vert en silence.
* **Écarté : `cargo-xwin`** (le wiki de NixOS) : il télécharge les en-têtes de Microsoft pour lier ;
  une vérification n'a rien à lier.
* **`-msvc`, pas `-gnu`** : la cible de la CI ; `-gnu` demande le `dlltool` de MinGW (essayé, le 25/11).
* **Le Mac** : `ring` (sous `rustls`) compile du C dans son script de construction. `gcc` ne connaît pas
  `-arch` ; le `clang` enveloppé de nixpkgs ajoute les options de Linux : le `clang` nu
  (`llvmPackages.clang-unwrapped`). Il lui faut ensuite `TargetConditionals.h` : le SDK d'Apple de
  nixpkgs, **sa source seule** (`apple-sdk.src`, 424 Mo ; le paquet ne se construit que sur un Mac), en
  `SDKROOT`.
* **Sa propre cible, `target/autres`** (2,9 Go) : `RUSTC_BOOTSTRAP` change les empreintes de Cargo ;
  dans `target/`, chaque passage recompilerait tout le dépôt.

## 3. Ce qui reste

* **Android** : `ring` y demande les en-têtes du NDK, absent ici. La même voie (`clang` nu, un
  `--sysroot` du NDK de nixpkgs, non libre et lourd) est possible ; pas tentée.
* **Le Mac Intel** (`x86_64-apple-darwin`) : rien dans le code ne distingue les deux Mac aujourd'hui.
* Rien ne s'y lie ni ne s'y lance : leurs épreuves ne tournent que sur la CI.
* **L'ombre sous Windows** (fiche 155 § 3) : la suite, vérifiée par cet outil.

## 4. La preuve

* Vert sur `d51d33b` pour les deux cibles (`clippy -D warnings`, épreuves comprises).
* **3 sabotages sur 3** (`outils/sabotages/cibles-156.json`), le témoin vert d'abord : une faute de
  type dans le code du bureau de Windows (`RedrawWindow` sans `Some`), un avertissement dans une
  épreuve de Windows (ce que `--all-targets` et `-D warnings` gardent), une faute de type dans le
  presse-papiers du Mac. Aucun des trois ne se voit au `clippy` de Linux : leurs fichiers ne s'y
  compilent pas.

## 5. Piège payé

* Un `rustc-ice-….txt` à la racine du dépôt, du 25/11 à 1 h 19 : « Failed to recover key for
  associated_item » — le cache incrémental de `rustc`, abîmé (un `rustc` tué par sa borne, le plus
  souvent). Pas une faute du code : `cargo clean -p <crate>`, recompiler ; le fichier se retire.

Sources : [wiki de NixOS, Rust](https://wiki.nixos.org/wiki/Rust) ;
[fenix](https://github.com/rcerc/fenix) ;
[build-std sans rust-src, NixOS Discourse](https://discourse.nixos.org/t/build-rust-app-using-cargos-build-std-feature-with-naersk-fails-because-rust-src-is-missing/13161) ;
[croiser vers Windows, NixOS Discourse](https://discourse.nixos.org/t/cross-compiling-rust-project-for-windows/81316).
