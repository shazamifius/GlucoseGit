# Où en est Glucose

> **Au 11/11/2026** (fiche 105). **Le présent, en une page** : ce qui est publié, ce qui marche, ce qui ne
> va pas, ce qu'on sait de ses utilisateurs. Il se **réécrit** quand la réalité change — il ne garde pas
> l'histoire, qui vit dans le [carnet](carnet/00-INDEX.md) (une fiche par session). La direction et ses
> paroles : [VISION](VISION.md). Ce qu'on fait ensuite : [SUITE](SUITE.md). Chaque fonction et son
> bouton : [INVENTAIRE](INVENTAIRE.md).

---

## 1. En bref

* **Publié** : la **2.0.3-beta.10** (son clic, le 10/11 à 23 h 59 ; la CI verte sur `aebb3f7`, dix
  tâches) — installeur Windows, AppImage, `.deb`, `.rpm`, APK Android, et `nix run
  github:shazamifius/GlucoseGit`. Chaque Glucose Tauri installé passe de lui-même à Glucose Rust ; ses
  documents s'ouvrent, ses données ne sont jamais touchées.
* **Sur `main`, depuis la beta 10** : `Alt+Z` d'un geste (RETOUR-2), les liens près des blocs (LIEN-4),
  les titres qui ne refont plus leurs lettres (LETTRES-1 : de loin, sur la carte de Wikipédia, 25,5 →
  15,5 ms par image) ; **les membranes en carte** (REGION-1, fiche 105 : la plus petite région
  l'emporte, le papier des textes toujours gris, une membrane sans couleur grise ; au fond de « Eau »,
  7,8 ms par image au lieu de 17,6). **Son Glucose installé** (profil Nix `GlucoseGit`) : à jour avec `main` (fiche 105, les membranes
  en carte comprises).
* **La machine** : son VivoBook sous **NixOS** (i7-8550U, 7,6 Go, Intel UHD 620 et MX150, écran 60 Hz,
  Wayland, niri), depuis le 6/11 — son PC Windows est cassé. **Windows ne s'essaie plus qu'à travers la
  CI**, alors que la plupart de ses utilisateurs y sont.
* **Le chantier en cours** : la carte de Wikipédia — ses cinq défauts (VISION § 7). Les membranes en carte
  sont écrites (fiche 105) ; l'allure de leurs frontières attend sa parole. Puis le halo des textes.

## 2. Ce que fait Glucose Rust

Le détail, fonction par fonction et bouton par bouton : [INVENTAIRE](INVENTAIRE.md).

| domaine | ce qui marche |
|---|---|
| **Naviguer** | canevas infini, zoom jusqu'à ×256 ; souris, pavé (pincement sous Windows, Linux et macOS), doigts ; `F` cadre la sélection, `Alt+F` tout ; **`Alt+Z`** le `Ctrl+Z` de la caméra ; signets ; minimap ; onglets ; **`Ctrl+F`** cherche dans tout le document ; le mode Focus par un geste choisi |
| **Le document** | `.glucose` écrit geste après geste, brouillons, reprise au lancement, Time Machine (`Ctrl+H`), les documents de Tauri lus ; au terminal, `glucose document.glucose` et une commande par outil des agents (`glucose aide`) |
| **Images** | PNG, JPEG, WebP, GIF (figé), BMP ; coller, déposer ; rotation, recadrage ; la mémoire par étages |
| **Texte** | Markdown, tableaux, liens, LaTeX ; net à tout zoom (de près, la carte graphique dessine les lettres depuis leur forme) ; **de loin, le gris de ses lignes** ; **son papier, toujours le gris sombre des blocs** (PAPIER-1) ; la couleur choisie va à sa lueur |
| **Flèches** | droites ou courbes, six relations, coudes, contournement, ancres de texte ; **de loin, une trace** ; leur présence suit leur nombre et la souris ; `Ctrl`+clic suit un lien ; la **fenêtre des liens** (`B`) amène ses voisins |
| **Organiser** | membranes (**en carte** : la plus petite région l'emporte, titre à la taille de la région), dossiers, **domaines** (qui traversent les fichiers), `Ctrl+G` grouper, disposer en colonnes, aimant, Ordonner, Mood board, Storyboard, les opérateurs des post-it ; la barre de couleur et **la roue** (à la Blender, dans Oklch) |
| **Exporter** | Markdown, SVG complet, PNG, planches de storyboard (PDF) |
| **L'interface** | 105 commandes dans une table (`Commande::toutes`), la palette (`Ctrl+Maj+P`), les raccourcis comme donnée, changés dans Glucose ; le mode référence |
| **Les agents** | `glucose --mcp` : une IA lit le tableau (par niveaux, ses domaines, ce que l'humain a choisi) et y pose textes, membranes, flèches, domaines — chaque geste annulable, rien sans son accord |
| **Mise à jour** | automatique et signée sous Windows et Linux ; sous Android, à prouver |

## 3. Ce qui ne va pas

**Ce qu'il voit, ses mots du 11/11** (la carte de Wikipédia, son usage réel) — le détail dans VISION § 5 à 7 :

* **pas encore lisible** : les membranes emboîtées s'empilaient en boue — **réglé sur `main`** (fiche 105),
  pas encore à son écran ; l'allure des frontières attend sa parole ;
* **trop de flèches**, qui partent des blocs et non des mots, au tracé brouillon (surtout en courbe) ;
* **il lague** : sur son VivoBook, la télémétrie dit **13,8 ms l'image typique, 2,6 s la pire** (au-dessus
  du plancher de 10 ms) ;
* **le halo d'un texte** ne dit rien : la teinte symbiotique de Tauri, « temporaire » ; il doit dire la
  parenté des textes (une IA de similarité), et porter la couleur qu'on choisit pour un texte — qui,
  depuis que le papier est toujours gris, ne se voit plus que dans sa lueur.

**Connus, mesurés, pas réglés** :

* les liens bleus dans les textes (`#60a5fa`) : contraste APCA 52, sous le minimum de 60 (fiche 104) ;
* choisir 8 000 images coûte 28 ms par image (fiche 87 § 7 bis) ;
* le fondu d'un message redessine toute la carte (REPOS-3, fiche 98 § 7) ;
* les titres des membranes ne suivent pas encore la loi du gris de loin (fiche 104 § 3) ; les traits plus
  fins qu'un pixel hors des nœuds non plus (GRIS-3) ;
* **la parité avec Glucose Tauri n'est pas atteinte** : ce qui manque, [INVENTAIRE](INVENTAIRE.md) § 3 ;
* **jamais essayé à son écran** : presque tout ce qui est venu depuis la fiche 62 — la liste unique est
  dans [SUITE](SUITE.md) § 3.

## 4. La qualité

* **Les épreuves** : 3 110, toutes vertes au 11/11 (`cargo test --workspace`) ; clippy strict et
  `cargo fmt` à zéro ; les **cliquets** (`tests/cliquets_suite.rs` : 80 lignes par fonction, 600 par
  fichier, le couplage au modèle…) ne se relèvent jamais.
* **Les sabotages** : chaque garde est cassée exprès et doit faire tomber une épreuve
  (`outils/sabotages/`, un fichier par chantier) ; les derniers : 66 sur 66 (fiches 102-103), 10 sur 10
  (LETTRES-1).
* **Sur GitHub, dix tâches** (Windows, Linux, les deux Mac, le noyau pour Android et le web, l'APK, NixOS,
  l'installeur et la mise à jour, la répétition de la bascule depuis Tauri, les paquets Linux) : **verte
  sur `aebb3f7`** (la beta 10). La CI se lance avant une publication, pas à chaque envoi.

## 5. Ses utilisateurs, et ce qu'on en sait

* **La télémétrie** (avec l'accord de chacun ; un serveur Cloudflare de son compte,
  [TELEMETRIE](TELEMETRIE.md)) au 11/11 : **160 sessions, 9 installations, 32 fins anormales**.
  * son VivoBook (Intel UHD 620, la version `dev` de son profil Nix) : 8 sessions depuis le 7/11 ;
  * une GTX 1660 SUPER sous Windows, 47 sessions jusqu'au 9/11, en beta 9 — **très probablement lui, au PC
    de l'école** ;
  * **deux neuves** : un Linux à RTX 4070 SUPER, déjà en beta 10 (9 sessions, la dernière le 11/11) ; un
    Windows à AMD Radeon 660M (beta 9, le 8/11) ;
  * **son téléphone** (moto g54, PowerVR) : 22 sessions, **17 fins anormales**, encore en **beta 4**,
    dernière session le 6/11 ; 16,4 ms l'image typique ;
  * un Windows à pilote Intel tué par `igvk64.dll` (le 29/10, en beta 6) : MOTEUR-1 l'a réparé depuis,
    sans preuve sur ce pilote.
* **Ses utilisateurs connus** : 5 de Glucose Tauri (Windows, Linux) ; une vingtaine de testeurs qui
  attendent Glucose Rust, dont 10 sous Android ; une personne qui compte énormément pour lui, sur **iPad**
  (Glucose dans Safari, plus tard).
* **Il ne peut dépenser aucun argent** : pas de certificat Windows, pas de compte Apple, pas de compte
  Play. Android passe par un APK installé à la main.

## 6. Points de vigilance

* **Ses données avant tout** : son dossier Glucose (`~/.local/state/glucose`) ne se touche jamais ; tout
  diagnostic se fait sur une copie.
* **La clé Android** : dans les secrets du dépôt, et chiffrée dans
  `/stockage/ancien-windows/bureau/sauvegarde-documents-2026-10-31.7z`.
* **La clé de mise à jour** (`glucose_updater.key`) : le secret du dépôt et deux copies sur ce PC
  (`~/.local/share/glucose-cles/`, `/stockage/ancien-windows/bureau/`) — **aucune hors de cet
  ordinateur** : une copie sur une clé USB à lui reste à faire.
* **Deux branches, pas une de plus** : `main` et `tauri-v1.0.1`.
* **Publier reste son geste** : le brouillon se prépare, le clic « Publish release » est le sien.
