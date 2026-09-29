# 48 — La mise à jour et la bascule

> **Rôle de ce document.** La partie B du plan de la fiche [`46`](46-LA-SUITE-DANS-L-ORDRE.md),
> commencée le 29/09 au soir : la mise à jour de Glucose Rust branchée dans l'application,
> l'installeur Windows, et la **bascule** de ses utilisateurs de Glucose Tauri — éprouvée de bout
> en bout sur une machine de GitHub, jamais sur la sienne. **Écrite au fil de l'eau.**
>
> **Date** : 2026-09-29. **État à la fin de cette étape** : sous Windows, **1 962 épreuves
> vertes** (8 de plus), clippy strict à zéro ; `%LOCALAPPDATA%\Glucose` **identique**, fichier
> pour fichier, avant et après chaque suite d'épreuves. **Rien n'est publié** : aucune release,
> aucun `latest.json`.

---

## 0. En une page

| | ce qui change pour lui |
|---|---|
| **Un piège désamorcé** | Glucose Rust portait la version **1.0.0**, sous celle de Glucose Tauri (1.0.2-beta.1) : branché, il aurait proposé de **réinstaller Glucose Tauri**. Il porte désormais **2.0.0-dev**, et une épreuve l'empêche de redescendre (§ 2) |
| **La mise à jour, branchée** | au lancement, un fil lit le `latest.json` — le même que Tauri ; si plus récent, **le popup habituel** ; oui : téléchargé, **signature vérifiée avant que rien ne touche le disque**, le document s'écrit, l'installeur démarre, Glucose se ferme. Après un arrêt brutal, la recherche passe **avant tout** (§ 3) |
| **L'installeur Windows** | pour l'utilisateur seul, sans droits d'administrateur, dans `%LOCALAPPDATA%\Programs\Glucose` ; 4,8 Mo (celui de Tauri : 8) (§ 4) |
| **La bascule depuis Tauri** | il retire les **deux** fichiers de Tauri — ceux que son désinstalleur retire, lu sur son binaire —, reprend raccourcis, épingle et liste des programmes ; **ses données ne sont jamais touchées** (§ 4) |
| **Glucose ouvert pendant l'installation** | l'installeur **attend** qu'il se ferme — Windows lui dit qui tient `glucose.exe` —, sans jamais le fermer de force (celui de Tauri, lui, le tue) (§ 5) |
| **L'épingle de la barre des tâches** | Glucose déclare à Windows **qui il est** (`com.glucose.app`, l'identité de Tauri), et chaque raccourci la porte : la fenêtre et son épingle se reconnaissent (§ 6) |
| **Plus de console noire** | Glucose Rust était un programme de console : lancé d'un raccourci, une fenêtre noire se serait ouverte à côté. Une version publiée est désormais fenêtrée (§ 7) |
| **Prouvé sur GitHub, à chaque envoi** | Glucose Tauri installé **et ouvert** → la bascule → la mise à jour par un `latest.json` signé **par l'outil même de Tauri** → la désinstallation ; ses données intactes à chaque pas (§ 8) |

---

## 1. Ce que fait l'updater de Glucose Tauri — lu dans sa source

La bascule passe par le programme de mise à jour que ses utilisateurs ont **déjà**. Il a été lu,
pas supposé (`tauri-plugin-updater` 2.10.1, `src/updater.rs` et `src/config.rs`) :

* il lit `…/releases/latest/download/latest.json`, choisit l'entrée de sa plateforme —
  `windows-x86_64-nsis` ou `windows-x86_64` ; sous Linux `linux-x86_64-appimage`, `-deb` ou
  `-rpm` selon ce qui est installé —, et vérifie la signature minisign de ce qu'il télécharge ;
* sous Windows, il lance l'installeur par `ShellExecuteW` avec **`/P /R /UPDATE`** (le mode
  passif par défaut : sans question, relancer ensuite), **puis quitte aussitôt**
  (`std::process::exit(0)`) — sans dire son numéro de processus à l'installeur ;
* Glucose Tauri ne déclare **aucune association de fichiers, aucun protocole, aucun lancement
  automatique** (`tauri.conf.json`) : seuls l'exécutable, les raccourcis et l'entrée du registre
  sont à reprendre.

Le `latest.json` publié de la 1.0.2-beta.1 annonce huit entrées : Windows (2), Linux (AppImage,
`.deb`, `.rpm`, et l'entrée générique), Mac à puce Apple (2). **Ses utilisateurs sont sous
Windows et Linux** (fiche 46 § 0) : la bascule devra servir l'AppImage, le `.deb` **et** le
`.rpm` — l'updater prend celui qui correspond à l'installation (§ 9).

## 2. La version : un piège désamorcé

La release « latest » de ce dépôt est encore celle de Glucose Tauri, **1.0.2-beta.1**. Glucose
Rust portait **1.0.0** : la veille des mises à jour, branchée, aurait trouvé
`1.0.2-beta.1 > 1.0.0` et proposé, à qui lançait Glucose Rust, d'installer Glucose Tauri — et,
après un arrêt brutal, **avant même d'ouvrir la fenêtre**.

Il porte désormais **2.0.0-dev** : au-dessus de toute version de Tauri, en dessous de la bascule
(2.0.1-beta.1), et « -dev » dit que ce n'est pas une version publiée. L'épreuve
`test_la_version_depasse_celle_de_glucose_tauri` empêche d'y retomber (sabotée : elle tombe).
**À la publication**, la version prend son vrai nom ; après, la version de travail repasse
au-dessus de la dernière publiée.

## 3. Le cycle de la mise à jour, dans l'application

`mise_a_jour::cycle` fait ce que fait l'updater de Tauri, au même fichier :

1. **chercher** : lire `latest.json`, ne proposer qu'une version qui **monte** (semver) ;
2. **préparer** : télécharger l'installeur, **vérifier sa signature, puis seulement l'écrire**
   (`%LOCALAPPDATA%\Glucose\mises-a-jour\`) — un installeur altéré ne touche jamais le disque ;
3. **lancer** avec `/P /R /UPDATE`, les arguments de Tauri, puis fermer Glucose — le document
   s'écrit d'abord ; s'il porte un travail sans nom, la question habituelle est posée, et
   « Annuler » reporte la mise à jour au lancement suivant.

Dans l'application (`app/lancement.rs`) : un fil cherche au lancement, un autre prépare quand
l'utilisateur a dit oui ; la boucle récolte ce qu'ils disent **sans jamais les attendre**, et
tout passe par **un seul toast** et un dialogue natif (DIAL-2).

**Après une session qui a mal fini** (la boîte noire le dit), la recherche se fait **avant
tout** — avant la carte graphique et le document : une version qui tombe au démarrage doit
pouvoir recevoir sa correction. Le prix, assumé et écrit dans la note
[`decisions/04`](decisions/04-CHERCHER-LES-MISES-A-JOUR.md) : ce lancement-là attend la réponse
du réseau — immédiate hors ligne, bornée par les délais de WinHTTP sur un réseau muet.

Deux commandes sans fenêtre servent les épreuves : `--version` et `--mettre-a-jour` (chercher,
préparer, lancer en silence). **L'adresse et la clé se fixent à la compilation** : une
construction d'épreuve y met un serveur local et une clé d'essai ; un programme installé ne se
laisse convaincre de rien par son environnement.

**La recherche est une requête de fond** : la note 02 promettait qu'il n'y en aurait aucune.
Elle est corrigée, et la note 04 dit exactement ce qui part, à qui et quand.

## 4. L'installeur Windows

`outils/installeur/glucose.nsi`, construit par `construire.py` (qui lit la version dans
l'exécutable lui-même : un installeur ne peut pas annoncer une autre version que celle qu'il
pose). NSIS, comme Tauri : c'est l'outil que ses utilisateurs ont déjà vu.

* **Pour l'utilisateur seul**, sans droits d'administrateur, dans le dossier que Windows réserve
  à cela (`FOLDERID_UserProgramFiles` : `%LOCALAPPDATA%\Programs\Glucose`).
* **Pas dans le dossier de Tauri.** Glucose Tauri s'installait dans `%LOCALAPPDATA%\Glucose` —
  là où Glucose Rust garde ses brouillons, sa boîte noire et ses aperçus. Y mêler le programme
  aurait rendu la bascule triviale (rien ne change de chemin), mais aurait gardé pour toujours
  un programme au milieu de ses données. Le prix de la séparation — reprendre raccourcis,
  épingle et registre — est payé une fois, et éprouvé (§ 8).
* **Ce qu'il retire de Tauri** : `glucose.exe` et `uninstall.exe`, **exactement** ce que le
  désinstalleur de Tauri retire de ce dossier (lu sur son binaire, fiche 47 § 2) ; puis le
  dossier **seulement s'il est vide** (`RMDir` sans `/r`) — il ne l'est jamais : ce qui y reste
  est à lui.
* **Ce qu'il reprend** : le raccourci du menu Démarrer, celui du bureau s'il existait, l'épingle
  de la barre des tâches si elle existe ; l'entrée de la liste des programmes, **écrite à neuf**
  (celle de Tauri portait des valeurs que ce programme n'a pas).
* **Ce qu'il ne touche jamais** : `%LOCALAPPDATA%\Glucose` au-delà de ces deux fichiers ; le
  magasin d'images de Tauri (`%APPDATA%\com.glucose.app`), que ses documents Tauri demandent.
* **Son désinstalleur** ne retire que ce que l'installeur a posé. **Aucun `RMDir /r`, nulle
  part** : `lire_les_suppressions.py --exiger-aucune-recursive` lit le binaire à chaque envoi —
  l'installeur **et** le désinstalleur qu'il embarque — et fait tomber la vérification si une
  suppression récursive y apparaît un jour. Sur l'installeur de Tauri, la même exigence tombe :
  son désinstalleur porte deux `RMDir /r`, ceux de la case « supprimer les données » (fiche 47).

La seule suppression qui reste à expliquer, `Delete $0` avec le drapeau `DEL_SIMPLE`, est celle
de NSIS lui-même : il crée un fichier temporaire, l'efface, et met à sa place le dossier de ses
greffons. Vérifié sur les ordres voisins (`GetTempFileName`, `CreateDirectory`, `$PLUGINSDIR`),
et le lecteur le dit désormais de lui-même.

**Il reste une clé** que Tauri laisse aussi après sa propre désinstallation :
`HKCU\Software\<éditeur>\Glucose` (son dossier d'installation et la langue de l'installeur).
Inoffensive ; on ne la touche pas.

## 5. Glucose ouvert pendant l'installation : attendre, jamais tuer

L'updater de Tauri lance l'installeur puis quitte — **sans dire son numéro**. Le premier
installeur attendait un numéro passé par `/ATTENDRE=` : Tauri ne le passe pas, et quelqu'un peut
aussi installer à la main Glucose ouvert.

**La solution qui a remplacé les deux cas** (`outils/installeur/attendre.nsh`) : on demande au
**Gestionnaire de redémarrage de Windows** (`RmGetList`) quels processus tiennent
`glucose.exe` — celui de Tauri comme celui de Glucose Rust —, et on attend **exactement** leur
fin. Aucune durée choisie ; **rien n'est fermé de force** : un Glucose ouvert à la main peut
porter du travail. `/ATTENDRE` a disparu, des deux côtés. L'installeur de Tauri, dans le même
cas, **tue** l'application (`CheckIfAppIsRunning`, `RmShutdown` forcé).

Éprouvé sur cette machine **sans rien installer**, par un programme d'essai qui inclut le même
fichier : personne ne tient le fichier → fini en 181 ms ; un processus le tient → Windows le
désigne (1 trouvé) et l'essai attend **exactement** sa fin (11,6 s, le processus était fini
avant). Deux sabotages — ne plus attendre, ne plus lire la liste — font tomber l'essai. Sur
GitHub, l'épreuve le refait avec **Glucose Tauri réellement ouvert** (§ 8).

## 6. L'épingle de la barre des tâches, et l'identité de Glucose

Windows rattache une fenêtre à son épingle par une identité, l'**AppUserModelID**. Microsoft
est net : une identité explicite se porte **partout à la fois** — processus, raccourcis,
épingles — sans quoi la barre des tâches les sépare.

* L'installeur de Tauri pose `com.glucose.app` sur ses raccourcis (`SetLnkAppUserModelId`) ;
  **son programme ne la déclare jamais** (vérifié dans le code de Tauri et de `tao` : seul le
  greffon des notifications s'en sert). Une fenêtre de Glucose Tauri n'était rattachée à son
  épingle que si elle avait été lancée par ce raccourci-là.
* **Glucose Rust déclare `com.glucose.app` au lancement**, avant toute fenêtre
  (`plateforme::identite`, `SetCurrentProcessExplicitAppUserModelID`), et **son installeur la
  pose sur chaque raccourci** qu'il écrit — le menu Démarrer, le bureau, l'épingle. Une épreuve
  exige que le programme et l'installeur portent la même (sabotée : elle tombe).
* Garder l'identité de Tauri, plutôt qu'en créer une : **les épingles de ses utilisateurs la
  portent déjà**.

**Ce qui n'est pas prouvé** : le comportement de l'Explorateur lui-même — qu'une épingle
réécrite se rafraîchisse sans fermer la session, que les épingles du menu Démarrer de
Windows 11 suivent. L'épreuve vérifie les fichiers (cible, identité), pas l'écran. C'est un
point de la répétition générale (§ 9).

## 7. Plus de console noire

Glucose Rust était compilé comme un **programme de console** : lancé depuis un raccourci,
Windows aurait ouvert une fenêtre noire à côté de Glucose, à chaque lancement, chez chacun de
ses utilisateurs basculés. Une construction publiée est désormais **fenêtrée**
(`windows_subsystem = "windows"`, sous-système 2 vérifié dans l'en-tête du binaire) ; la
construction de travail garde sa console. Les journaux ne se perdent pas : redirigés vers un
fichier, ils s'y écrivent (`--version` répond, redirigé, vérifié).

## 8. La vérification de GitHub : la vie entière de Glucose sous Windows

Une tâche de plus à chaque envoi, **« Windows, l'installeur et la mise à jour »**
(`outils/installeur/eprouver.ps1`), sur une machine jetable — l'épreuve installe dans le compte
courant ; ailleurs, **elle refuse avant de toucher à quoi que ce soit** (il lui faut
`GITHUB_ACTIONS` **et** `RUNNER_ENVIRONMENT=github-hosted` ; essayé ici : refus, code 2).

1. **Glucose Tauri** 1.0.2-beta.1 — la vraie release publique — s'installe. On vérifie qu'il
   pose l'identité sur son raccourci (la prémisse du § 6), et on plante des données : des
   brouillons, un fichier de récupération, une session de boîte noire, un aperçu, le dernier
   document dans `%LOCALAPPDATA%\Glucose` ; une image dans le magasin de Tauri ; un document ;
   et l'épingle que Windows crée quand on épingle.
2. **La bascule, Glucose Tauri ouvert**, avec les arguments de son updater : l'installeur doit
   **attendre** (vingt secondes d'observation : il n'a rien posé ni rien retiré), puis, Tauri
   fermé, s'installer ; les deux fichiers de Tauri partis ; chaque raccourci mène à Glucose Rust
   et porte l'identité ; la liste des programmes dit la nouvelle version, sans les valeurs de
   Tauri.
3. **La mise à jour** : la version N+1 est construite, **signée par l'outil de Tauri** avec une
   clé d'essai tirée sur place, et servie par un serveur local. Un installeur **altéré** est
   refusé, et rien n'est posé ; l'intact installe N+1, et l'installeur posé est exactement celui
   qui est signé.
4. **La désinstallation** : le programme, ses raccourcis et son entrée partent.

**À chaque pas, les données plantées sont intactes, octet pour octet**, et rien n'a disparu de
`%LOCALAPPDATA%\Glucose`.

**Et une épreuve permanente de plus** : notre vérificateur n'avait jamais lu une signature du
**vrai** signataire de Tauri — celles des tests étaient faites d'après la RFC. Une clé et une
signature produites par `tauri-cli` 2.12.0 sont désormais figées dans les tests
(`test_ce_que_signe_l_outil_de_tauri_passe`) : si Tauri change sa forme un jour, c'est là que ça
tombe.

## 9. Ce qui reste de B, dans l'ordre

1. **Linux** : ses utilisateurs Linux basculent par l'AppImage, le `.deb` ou le `.rpm` — il faut
   les trois, et la voie réseau de Glucose Rust sous Linux (note 04) pour ses mises à jour
   suivantes. Le paquet `.deb`/`.rpm` doit porter le nom de celui de Tauri pour le remplacer.
2. **La construction et la publication automatiques** — en brouillon ou pré-version, **jamais
   « latest » avec un `latest.json`** tant que la bascule n'est pas décidée par lui. Elles
   attendent sa réponse sur sa clé de signature (fiche 46 § 6).
3. **La répétition générale** : l'updater de Tauri lui-même — le popup, le téléchargement, la
   vérification par sa clé — mène à Glucose Rust, avec une clé d'essai. L'épreuve du § 8 lance
   l'installeur comme lui ; il reste à le voir faire, et à regarder l'épingle à l'écran (§ 6).

## 10. Les sabotages

| garde | sabotage | résultat |
|---|---|---|
| la version dépasse celle de Tauri | `version = "1.0.0"` | **tombe** |
| le programme et l'installeur portent la même identité | l'installeur pose `com.glucose.autre` | **tombe** |
| l'installeur attend qui tient `glucose.exe` | plus d'attente | **tombe** (l'essai finit avant) |
| — | la liste des processus n'est plus lue | **tombe** |
| `--exiger-aucune-recursive` | l'exigence ne fait plus rien | **tombe** (sur l'installeur de Tauri) |
| ce que signe l'outil de Tauri passe | une autre clé | **tombe** |

## 11. Ce que le premier envoi a appris

La vérification de `e8183bd` (run `36625698372`) :

* **« Windows, l'installeur et la mise à jour » : verte du premier coup.** Chaque constat de
  l'épreuve a été lu dans son journal — Glucose Tauri ouvert, l'installeur qui l'attend sans rien
  poser ni rien retirer, l'installeur altéré refusé (« la signature ne correspond pas »), la N+1
  installée, les sept fichiers plantés intacts après la bascule, la mise à jour et la
  désinstallation.
* **Linux et les deux Mac : clippy tombé.** Les épreuves de bout en bout du cycle n'étaient
  compilées pour rien hors de Windows (leurs outils restaient, sans usage). Elles vivent dans
  un sous-module — et, puisque Linux et Mac ont désormais leur téléchargeur (`ureq`, note
  [`decisions/05`](decisions/05-LE-RESEAU-HORS-DE-WINDOWS.md)), elles y tournent aussi.
* **Windows : une épreuve de la phase A est tombée**, `test_une_promesse_abandonnee…` —
  « l'échec se dit ». Diagnostic plutôt que supposition : elle tombait **aussi ici, 2 fois sur
  40**, l'erreur absente. Cause : l'application elle-même, à la fin de chaque image, prend
  l'erreur du scribe et **l'affiche** (le bon comportement) ; quand le scribe allait plus vite
  que le fil qui dessine, l'épreuve cherchait au scribe une erreur déjà dite. Elle lit désormais
  **le toast**, ce qu'il verrait : **60 sur 60**, et deux sabotages (l'erreur qui ne s'affiche
  plus, le message qui change) la font tomber. Ma première hypothèse — l'antivirus de GitHub —
  était fausse : l'erreur n'était pas écrasée, elle était absente.

## 12. Linux : trois formes, et le geste de Tauri

Ses utilisateurs Linux ont installé Glucose Tauri sous l'une de trois formes, et l'updater de
Tauri remplace chacune à sa façon (lu dans sa source) : **l'AppImage** est réécrit par-dessus
lui-même ; **le `.deb`** s'installe par `pkexec dpkg -i`, **le `.rpm`** par `pkexec rpm -U` —
puis Tauri relance le même exécutable. Les paquets de Tauri, ouverts : le paquet **`glucose`**,
le programme **`/usr/bin/glucose`**, `Glucose.desktop` (`StartupWMClass=glucose`), les icônes
`glucose` ; et **glibc 2.34** exigée — il était construit sur Ubuntu 22.04.

* **Les paquets de Glucose Rust** (`outils/paquets/construire.sh`) reprennent cette disposition
  exacte, construits **sur Ubuntu 22.04** : sur une machine plus récente, Glucose exigerait une
  bibliothèque C que leurs systèmes n'ont peut-être pas.
* **Leurs dépendances** : `dpkg -i` et `rpm -U` n'installent rien de ce qui manque — et un
  `dpkg -i` aux dépendances manquantes remplace quand même les fichiers, puis laisse le paquet à
  moitié configuré. **Seul GTK 3 est exigé**, que Glucose Tauri exigeait déjà ; le reste est
  recommandé. **Une limite, dite** : sur une session **X11** sans `libxkbcommon-x11`, Glucose
  ne s'ouvrirait pas (winit en a besoin pour le clavier) ; GNOME, KDE et Cinnamon l'installent,
  et une session Wayland — le défaut d'aujourd'hui — n'en a pas besoin.
* **La fenêtre porte la classe `glucose`** (X11) et l'`app_id` `glucose` (Wayland — vérifié dans
  la source de winit 0.30.13) : son fichier de bureau la reconnaît — son nom, son icône, son
  épingle. C'est l'équivalent Linux de l'identité Windows (§ 6).
* **Glucose Rust se met à jour lui-même sous Linux** (`mise_a_jour::installation`) : il sait
  comment il est installé — `APPIMAGE`, que pose le lanceur des AppImage ; sinon `dpkg-query -S`
  ou `rpm -qf` sur son exécutable — et lit dans `latest.json`, **dans l'ordre de Tauri**, l'entrée
  de sa forme puis celle de la plateforme. Il pose **pendant qu'il tourne** (Linux le permet :
  un programme ouvert garde l'ancien fichier), puis se ferme et se relance. L'AppImage se
  remplace d'un bloc, **exécutable avant de prendre la place de l'ancien** (`atomic`, le seul
  endroit qui pose un fichier à la place d'un autre). Un Glucose installé autrement — NixOS, qui
  le met à jour lui-même ; `cargo run` — **ne se propose aucune mise à jour** qu'il ne saurait
  pas poser.
* **La vérification de GitHub** (`outils/paquets/eprouver.sh`) : pour chaque forme — le `.deb`
  dans un conteneur Ubuntu 22.04, le `.rpm` dans un conteneur Fedora 40, l'AppImage sur la
  machine —, Glucose Tauri 1.0.2-beta.1 réel installé, des données plantées, **le geste exact de
  l'updater de Tauri**, puis la mise à jour de Glucose Rust par un `latest.json` signé par l'outil
  de Tauri : un paquet altéré refusé **pour sa signature**, l'intact posé ; les données intactes à
  chaque pas.

## 13. Les sources

* Tauri, `tauri-plugin-updater` 2.10.1 — [`src/updater.rs`](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/updater/src/updater.rs)
  (`install_inner` sous Windows : `ShellExecuteW`, puis `std::process::exit(0)`) et
  `src/config.rs` (`nsis_args` : `/P /R` en mode passif).
* Tauri, le modèle d'installeur NSIS —
  [`installer.nsi`](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi)
  et [`utils.nsh`](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/utils.nsh)
  (`SetLnkAppUserModelId`, `CheckIfAppIsRunning` et son `RmShutdown` forcé).
* Glucose Tauri, sa configuration (`src-tauri/tauri.conf.json`) et son `latest.json` publié
  ([v1.0.2-beta.1](https://github.com/shazamifius/GlucoseGit/releases/tag/v1.0.2-beta.1)).
* Microsoft, [*Application User Model IDs*](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
  — l'identité portée partout, ou nulle part ; et
  [`SetCurrentProcessExplicitAppUserModelID`](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-setcurrentprocessexplicitappusermodelid)
  — avant toute interface.
* Microsoft, [`RmGetList`](https://learn.microsoft.com/en-us/windows/win32/api/restartmanager/nf-restartmanager-rmgetlist)
  — `ERROR_MORE_DATA` (234) et le nombre de processus.
* Microsoft, [`KNOWNFOLDERID`](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
  — `FOLDERID_UserProgramFiles`.
* NSIS 3, les en-têtes livrés `Include\Win\RestartManager.nsh`, `COM.nsh`, `Propkey.nsh`, et le
  [greffon System](https://nsis.sourceforge.io/Docs/System/System.html).
* Rust, [l'attribut `windows_subsystem`](https://doc.rust-lang.org/reference/runtime.html#the-windows_subsystem-attribute).
* AppImage, [`appimagetool` 1.9.1](https://github.com/AppImage/appimagetool/releases/tag/1.9.1)
  et le [runtime type 2 du 08/11/2025](https://github.com/AppImage/type2-runtime/releases/tag/20251108),
  épinglés par leur empreinte SHA-256 publiée.
* winit 0.30.13, `src/platform/x11.rs` et `wayland.rs` (`with_name`), et
  `platform_impl/linux/wayland/window/mod.rs` (l'`app_id`).
* GitHub, [les variables des machines](https://docs.github.com/en/actions/reference/workflows-and-actions/variables)
  — `GITHUB_ACTIONS`, `RUNNER_ENVIRONMENT=github-hosted`, `RUNNER_TEMP`.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
