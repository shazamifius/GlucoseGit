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

* ✅ **Android** (§ 7).
* **Le Mac Intel** (`x86_64-apple-darwin`) : rien dans le code ne distingue les deux Mac aujourd'hui.
* Rien ne s'y lie ni ne s'y lance : leurs épreuves ne tournent que sur la CI.
* **L'ombre sous Windows** (fiche 155 § 3) : faite, § 6.

## 4. La preuve

* Vert sur `d51d33b` pour les deux cibles (`clippy -D warnings`, épreuves comprises).
* **3 sabotages sur 3** (`outils/sabotages/cibles-156.json`), le témoin vert d'abord : une faute de
  type dans le code du bureau de Windows (`RedrawWindow` sans `Some`), un avertissement dans une
  épreuve de Windows (ce que `--all-targets` et `-D warnings` gardent), une faute de type dans le
  presse-papiers du Mac. Aucun des trois ne se voit au `clippy` de Linux : leurs fichiers ne s'y
  compilent pas.

## 5. Piège payé

* **Le cache incrémental de `rustc` s'abîme** : un `rustc-ice-….txt` à la racine du dépôt, le 25/11 à
  1 h 19 (« Failed to recover key for associated_item »), puis deux fois vers 3 h, juste après des
  sabotages (« on_disk_cache.rs:658 », dans les cliquets puis dans le témoin d'un sabotage). Pas une faute
  du code : vider `target/debug/incremental/glucose_desktop-*`, recompiler ; le fichier se retire. Les
  sabotages se jouent désormais avec `CARGO_INCREMENTAL=0` (MÉTHODE § 5) : rejoué ainsi, le témoin a
  tenu.

## 6. L'ombre sous Windows (OMBRE-2, `a1c37f0`)

La suite de la fiche 155 § 3, vérifiée par l'outil ci-dessus.

* **L'image** (`glisser_windows.rs`, `poser_l_ombre`) : un DIB de 32 bits, de haut en bas (une hauteur
  négative, comme Chromium), confié à `IDragSourceHelper::InitializeFromBitmap` avec le point de la main en
  `ptOffset` ; `crColorKey` à `CLR_NONE`, l'alpha dit la transparence. Le gestionnaire ne prend l'image
  qu'en réussissant : sinon `DeleteObject`. Une ombre qui ne se pose pas n'empêche pas de glisser : une
  ligne au journal, le curseur seul.
* **Des couleurs non prémultipliées** (`ombre_bgra.rs`) : « because InitializeFromBitmap always performs
  the RGB multiplication step in calculating the alpha value, you should always pass a bitmap without
  premultiplied alpha blending » (la documentation de Microsoft) ; Chromium les rend droites pour la même
  raison. Écrit hors de Windows, éprouvé partout.
* **`SetData` enfin** (`selection_windows.rs`) : le gestionnaire range l'image dans l'objet (« DragImageBits »,
  « DragWindow »…) et chaque fenêtre survolée l'y relit ; notre objet répondait `E_NOTIMPL`, et l'ombre ne
  se serait jamais posée (le piège que décrit Adam Root pour WPF et WinForms). Les octets sont copiés, le
  support rendu si on nous le confie ; nos propres formats passent avant ; ce qui est rangé s'énumère.
  De même, la description que l'explorateur y range (« Copier vers Bureau ») devrait se voir sous le
  curseur — pas vu.
* **L'image au-dessus de Glucose** (`depot_windows/image.rs`) : Windows ne la montre que sur une fenêtre
  dont la cible appelle `IDropTargetHelper`. Sans lui, l'ombre disparaissait sur Glucose — sur la fenêtre
  même d'où le geste part —, et la vignette d'un fichier venu de l'explorateur aussi. La cible lui passe
  chaque entrée, survol, sortie et dépôt, avec l'effet qu'elle vient de répondre (`ecrire_effet` le rend).
* **L'échelle exacte** (`plateforme::densite_de_l_ombre`) : Wayland ne prend qu'une échelle entière (2
  pour 150 %, le compositeur réduit) ; Windows montre l'image pixel pour pixel — à 150 %, l'ombre prise
  à 2 aurait été d'un tiers trop grande.

**La preuve, et sa limite.** `verifier_cibles.sh windows` vert (clippy strict, épreuves comprises) ;
**4 sabotages sur 4** (`outils/sabotages/ombre-156.json` : les couleurs laissées prémultipliées, rouge et
bleu inversés, l'échelle exacte sous Wayland, la densité entière oubliée). L'épreuve de `SetData`
(`selection_windows/tests.rs`) ne tourne qu'à la CI de Windows : son sabotage l'y attend. **Jamais
lancée sous Windows** : que l'ombre se voie, à sa taille, sous la main, se regarde à la première
version publiée sur une machine Windows.

Sources : [InitializeFromBitmap](https://learn.microsoft.com/en-us/windows/desktop/api/shobjidl_core/nf-shobjidl_core-idragsourcehelper-initializefrombitmap) ;
[SHDRAGIMAGE](https://learn.microsoft.com/windows/win32/api/shobjidl_core/ns-shobjidl_core-shdragimage) ;
[Chromium, `drag_utils_win.cc`](https://sources.debian.org/src/chromium-browser/41.0.2272.118-1/ui/base/dragdrop/drag_utils_win.cc) ;
[Adam Root, Shell Style Drag and Drop in .NET](https://learn.microsoft.com/en-us/archive/blogs/adamroot/shell-style-drag-and-drop-in-net-wpf-and-winforms).

Sources : [wiki de NixOS, Rust](https://wiki.nixos.org/wiki/Rust) ;
[fenix](https://github.com/rcerc/fenix) ;
[build-std sans rust-src, NixOS Discourse](https://discourse.nixos.org/t/build-rust-app-using-cargos-build-std-feature-with-naersk-fails-because-rust-src-is-missing/13161) ;
[croiser vers Windows, NixOS Discourse](https://discourse.nixos.org/t/cross-compiling-rust-project-for-windows/81316).

## 7. Android aussi (le 25/11 vers 3 h)

`bash outils/verifier_cibles.sh android` : `aarch64-linux-android`, l'API 21 de `cargo ndk -P 21`. Le
`clang` nu ne suffit pas : `android-activity` compile du C++ contre `<android/api-level.h>`. **Le NDK 29
de nixpkgs** (`androidenv.composeAndroidPackages`, le même `29.0.14206865` que la CI, `decisions/08`) :
non libre, sa licence acceptée comme la CI l'accepte déjà (`echo "y" | sdkmanager`), 2,4 Go une fois ;
ses `aarch64-linux-android21-clang`, `clang++` et `llvm-ar`. 45 s à froid.

**Ce qu'il a trouvé au premier passage**, que rien ne compilait jamais :
* **Deux épreuves de la vraie porte** (`agents/porte/tests.rs` : `agents_2_…`, `tuyau_1_…`) appelaient
  `connecter` et `Porte::nom`, qui n'existent pas sur le téléphone (« Le téléphone n'a pas de porte »).
  La CI construit l'APK sans ses épreuves : personne ne le voyait. Réservées aux machines qui ont une
  porte, leurs `use` avec elles.
* **Une fausse alerte de `clippy`** : `missing_const_for_thread_local` sur trois `thread_local!` déjà
  écrits `const { … }` (`perf.rs`, `ui/couleurs.rs`). Sous Android, la bibliothèque standard prend sa
  variante `os_local`, qui produit toujours un `__init()` ; la règle s'y trompe (rust-clippy #12186, qui
  devait le corriger). Écartée pour Android seul, dans l'outil.

**La suite entière, 3 697 épreuves vertes** sur Linux après OMBRE-2 et CIBLES-2. **2 sabotages de plus, 2 sur 2** (`cibles-156.json`) : une faute de type dans la porte du téléphone ;
une épreuve de la vraie porte rendue au téléphone (ce que `--all-targets` garde).

Sources : [rust-clippy #12186](https://github.com/rust-lang/rust-clippy/pull/12186) ;
[rust-clippy #12637](https://github.com/rust-lang/rust-clippy/issues/12637).
