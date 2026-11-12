# Où en est Glucose

> **Au 12/11/2026** (fiches 108-109). **Le présent, en une page** : ce qui est publié, ce qui marche, ce qui ne
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
  7,8 ms par image au lieu de 17,6) ; la souris qui suit les nœuds par les rangs de l'index (SURVOL-1 et
  2, fiche 106 : sur la carte, 8,2 → 4,9 ms) ; **le halo d'un texte dit sa parenté** (PARENTE, fiche
  106 : une petite IA du sens embarquée, des familles, une sûreté ; la teinte de bruit de Tauri partie,
  flèches comprises) ; **les membranes deux fois plus transparentes** (le voile d'une membrane seule de
  Tauri, 11 % — *« vraiment vraiment parfait »*, fiche 107) ; **les titres des membranes posés par la
  carte graphique** et deux mémoires pures (TITRE-3, fiche 108 : de loin, 17 → 7,5 ms par image) ;
  **la Time Machine, socle de l'IA** (fiche 108) : chaque geste signé — l'humain, ou un agent sous son
  nom —, un écrit qui s'attache à un geste, une IA qui relit toute l'histoire avec ses écrits ;
  **l'IA ne juge que les flèches qu'un agent ou un script a posées**, jamais les siennes (PARENTE-4).
  **Son Glucose installé** (profil Nix `GlucoseGit`) : à jour avec `main` (`365c9a8`).
* **La machine** : son VivoBook sous **NixOS** (i7-8550U, 7,6 Go, Intel UHD 620 et MX150, écran 60 Hz,
  Wayland, niri), depuis le 6/11 — son PC Windows est cassé. **Windows ne s'essaie plus qu'à travers la
  CI**, alors que la plupart de ses utilisateurs y sont.
* **Le chantier en cours** : la carte de Wikipédia — ses cinq défauts (VISION § 7) —, et le cercle de
  l'IA bâti sur la Time Machine (VISION § 8 bis : elle propose, il corrige, Glucose note, elle
  apprend). L'allure des frontières des membranes attend sa parole. **Une limite** : sa carte de
  Wikipédia a été posée par un script avant MAIN-1 — ses gestes y sont signés comme les siens, et l'IA
  n'en juge aucune flèche tant qu'elle n'est pas reposée par la version neuve.

## 2. Ce que fait Glucose Rust

Le détail, fonction par fonction et bouton par bouton : [INVENTAIRE](INVENTAIRE.md).

| domaine | ce qui marche |
|---|---|
| **Naviguer** | canevas infini, zoom jusqu'à ×256 ; souris, pavé (pincement sous Windows, Linux et macOS), doigts ; `F` cadre la sélection, `Alt+F` tout ; **`Alt+Z`** le `Ctrl+Z` de la caméra ; signets ; minimap ; onglets ; **`Ctrl+F`** cherche dans tout le document ; le mode Focus par un geste choisi |
| **Le document** | `.glucose` écrit geste après geste, brouillons, reprise au lancement, Time Machine (`Ctrl+H`) — **chaque geste signé**, ses écrits —, les documents de Tauri lus ; au terminal, `glucose document.glucose` et une commande par outil des agents (`glucose aide`) |
| **Images** | PNG, JPEG, WebP, GIF (figé), BMP ; coller, déposer ; rotation, recadrage ; la mémoire par étages |
| **Texte** | Markdown, tableaux, liens, LaTeX ; net à tout zoom (de près, la carte graphique dessine les lettres depuis leur forme) ; **de loin, le gris de ses lignes** ; **son papier, toujours le gris sombre des blocs** (PAPIER-1) ; **sa lueur dit sa parenté** : une IA du sens embarquée (15 Mo, multilingue, sans réseau) range les textes en familles — même couleur, d'autant plus forte et vive qu'elle est sûre ; un texte sans parent garde le contour gris de son bloc ; la couleur choisie va à sa lueur (PARENTE, fiche 106) |
| **Flèches** | droites ou courbes, six relations, coudes, contournement, ancres de texte ; **la couleur des lueurs qu'elles relient** (grises vers une photo) ; **de loin, une trace** ; leur présence suit leur nombre et la souris — **la souris ne rapproche que les flèches de sens** parmi celles qu'un agent ou un script a posées (PARENTE-4) ; `Ctrl`+clic suit un lien ; la **fenêtre des liens** (`B`) amène ses voisins |
| **Organiser** | membranes (**en carte** : la plus petite région l'emporte, titre à la taille de la région), dossiers, **domaines** (qui traversent les fichiers), `Ctrl+G` grouper, disposer en colonnes, aimant, Ordonner, Mood board, Storyboard, les opérateurs des post-it ; la barre de couleur et **la roue** (à la Blender, dans Oklch) |
| **Exporter** | Markdown, SVG complet, PNG, planches de storyboard (PDF) |
| **L'interface** | 105 commandes dans une table (`Commande::toutes`), la palette (`Ctrl+Maj+P`), les raccourcis comme donnée, changés dans Glucose ; le mode référence |
| **Les agents** | `glucose --mcp` : une IA lit le tableau (par niveaux, ses domaines, ce que l'humain a choisi, la parenté que l'IA de Glucose donne aux textes et aux flèches) et y pose textes, membranes, flèches, domaines — chaque geste annulable, signé de son nom, rien sans son accord ; elle attache un écrit à un geste (`glucose_noter`) et relit l'histoire (`glucose_histoire`) |
| **Mise à jour** | automatique et signée sous Windows et Linux ; sous Android, à prouver |

## 3. Ce qui ne va pas

**Ce qu'il voit, ses mots du 11/11** (la carte de Wikipédia, son usage réel) — le détail dans VISION § 5 à 7 :

* **pas encore lisible** : les membranes emboîtées s'empilaient en boue — **réglé** (fiche 105), et deux
  fois plus transparentes (fiche 107, sa parole sur les images) ; pas encore à son écran ; l'allure des
  frontières attend sa parole ;
* **trop de flèches**, qui partent des blocs et non des mots, au tracé brouillon (surtout en courbe) ;
  l'IA ne juge que les flèches d'un agent ou d'un script — celles de sa carte actuelle n'en sont pas
  (posées avant MAIN-1) ;
* **il lague** : sur son VivoBook, la télémétrie dit **13,8 ms l'image typique, 2,6 s la pire** (avant
  la fiche 105) ; remesuré le 12/11 sur sa carte (fiche 108) : **de loin 7,5 ms** (17 avant), au fond de
  « Eau » 2,8 ms ; **au zoom** (fiche 109, pas encore installé chez lui) : une image du mouvement
  4-5 ms au lieu de 19-21, la latence 7 ms au lieu de 20 — reste l'image de netteté après chaque
  cran (≈ 20 ms) ; pas encore à son écran ;
* **le halo d'un texte** dit sa parenté depuis la fiche 106 — pas encore à son écran ; beaucoup de
  flèches de la carte sont grises (leurs bouts n'ont pas de famille sûre) : acceptées *« seulement si
  c'est temporaire »* ; sous une quinzaine de textes, l'IA hésite ; la clarté (l'accessibilité) n'est
  pas encore lue.

**Connus, mesurés, pas réglés** :

* **les versions publiées refusent un document qui porte une entrée neuve** (un geste d'agent signé, un
  écrit — PERSIST-5, fiche 108) : message net, rien de perdu, mais son téléphone (beta 4) et le PC de
  l'école (beta 9) ne l'ouvriront qu'une fois à jour ;
* les liens bleus dans les textes (`#60a5fa`) : contraste APCA 52, sous le minimum de 60 (fiche 104) ;
* choisir 8 000 images coûte 28 ms par image (fiche 87 § 7 bis) ;
* le fondu d'un message redessine toute la carte (REPOS-3, fiche 98 § 7) ;
* les titres des membranes ne suivent pas encore la loi du gris de loin (fiche 104 § 3) ; les traits plus
  fins qu'un pixel hors des nœuds non plus (GRIS-3) ;
* **la parité avec Glucose Tauri n'est pas atteinte** : ce qui manque, [INVENTAIRE](INVENTAIRE.md) § 3 ;
* **jamais essayé à son écran** : presque tout ce qui est venu depuis la fiche 62 — la liste unique est
  dans [SUITE](SUITE.md) § 3.

## 4. La qualité

* **Les épreuves** : 3 157, toutes vertes au 12/11 (`cargo test --workspace`) ; clippy strict et
  `cargo fmt` à zéro ; les **cliquets** (`tests/cliquets_suite.rs` : 80 lignes par fonction, 600 par
  fichier, le couplage au modèle…) ne se relèvent jamais.
* **Les sabotages** : chaque garde est cassée exprès et doit faire tomber une épreuve
  (`outils/sabotages/`, un fichier par chantier) ; les derniers : 66 sur 66 (fiches 102-103), 17 sur 17
  (REGION-1, fiche 105), 36 sur 36 (le sens et la parenté, fiche 106), 3 sur 3 (le voile des membranes,
  fiche 107), 46 sur 46 (fiche 108 : les flèches, les titres, les mémoires, la Time Machine, la
  provenance — trois épreuves aveugles trouvées et écrites).
* **Sur GitHub, dix tâches** (Windows, Linux, les deux Mac, le noyau pour Android et le web, l'APK, NixOS,
  l'installeur et la mise à jour, la répétition de la bascule depuis Tauri, les paquets Linux) : **verte
  sur `aebb3f7`** (la beta 10). La CI se lance avant une publication, pas à chaque envoi.

## 5. Ses utilisateurs, et ce qu'on en sait

* **La télémétrie** (avec l'accord de chacun ; un serveur Cloudflare de son compte,
  [TELEMETRIE](TELEMETRIE.md)) au 12/11 : **165 sessions, 9 installations, 33 fins anormales**.
  * son VivoBook (Intel UHD 620, la version `dev` de son profil Nix) : 8 sessions du 7 au 11/11 ;
  * une GTX 1660 SUPER sous Windows, 48 sessions jusqu'au 12/11, encore en beta 9 — **très probablement
    lui, au PC de l'école** ;
  * un Linux à RTX 4070 SUPER, en beta 10 (10 sessions, la dernière le 11/11) ; un Windows à AMD Radeon
    660M, passé en beta 10 (4 sessions, aucune fin anormale) ;
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
