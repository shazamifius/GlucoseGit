# Où en est Glucose

> **Au 19/11/2026, 18 h 40** (fiches 108-126). **Le présent, en une page** : ce qui est publié, ce qui marche, ce
> qui ne va pas, ce qu'on sait de ses utilisateurs. Il se **réécrit** quand la réalité change — il ne
> garde pas l'histoire, qui vit dans le [carnet](carnet/00-INDEX.md) (une fiche par session). La
> direction et ses paroles : [VISION](VISION.md). Ce qu'on fait ensuite : [SUITE](SUITE.md). Chaque
> fonction et son bouton : [INVENTAIRE](INVENTAIRE.md).

---

## 1. En bref

* **Publiée** : la **2.0.4-beta.3** (son clic, le 19/11 à 16 h 20, construite sur `15beb94`) — sa
  capture du 17/11 en partie : le texte et les icônes de l'interface nets (GRILLE-1 et 2), les trois
  modes de la fenêtre (`F11`, `Ctrl+Maj+A`, le zen à `Maj+F11`), toujours devant sous Wayland
  (DEVANT-1), la barre de Windows au choix, renommer un dossier (`F2`) (fiches 120-121). **Son PC de
  l'école s'y est mis à jour tout seul** le jour même. Avant elle, la 2.0.4-beta.2 (18/11 : le
  co-working et ses photos, la porte aux IA publiques, la frise, le miroir des images). Installeur
  Windows, AppImage, `.deb`, `.rpm`, APK Android, `nix run github:shazamifius/GlucoseGit`.
* **À préparer** : la **2.0.4-beta.4** — tout ce qui suit ; ses notes sont écrites (`outils/publication/notes.md`,
  « depuis la 2.0.4-beta.3 », vérifiées) ; la CI de `main` tourne (fiche 126). Le numéro change :
  une machine en beta.3 ne se mettrait pas à jour vers un autre programme du même numéro.
* **Sur `main` depuis la beta.3** :
  * **les tiroirs du côté de leur bouton** (TIROIR-4, sa parole du 19/11 :
    collés au bord sous la bande, plats, rangés en colonnes d'une largeur), regardés dans le vrai
    Glucose à 100, 110 et 125 % (fiche 124) ;
  * **l'échelle de l'interface, à son choix** (ECHELLE-1 : « Agrandir l'interface » au menu du vide,
    de 50 à 300 %, retenue d'une session à l'autre), regardée dans le vrai Glucose ;
  * **le détourage** (DETOURER-1 à 3, sa demande du 18/11) : « Détourer : la forme principale » par
    u2netp lu par un moteur à nous (1 s sur son VivoBook), et « Retirer le fond uni », exact ;
    annulables, envoyés au co-working — regardés au pilote (une boule, une pomme) : un liseré clair reste au bord du sujet ;
  * **ce que Claude.ai écrivait et que Glucose dessinait mal** (fiches 123-125) : les tableaux
    tiennent dans leur nœud (TABLE-3, la répartition de *CSS Tables 3*) et **un filet discret
    sépare deux rangées** (TABLE-4), les listes s'imbriquent (LISTE-1, la règle de CommonMark), le
    témoin dit l'onglet où l'IA travaille (TEMOIN-2), **un texte qui grandit pousse sa membrane**
    (TEXT-FIT-2), et **les émojis sont en couleur** (EMOJI-2 : Twemoji 17 construit par nous en
    COLRv0, `decisions/18`) — **les signes d'encre (✔ ✖ ➕ ♠ ⚫ 🖤, invisibles sur le papier d'un
    nœud) dans la palette du fond sombre que la police déclare, l'encre du texte disant le papier**
    (EMOJI-3, fiche 126 : sans elle, la version suivante les aurait effacés) — regardés sur une copie de son
    document ;
  * **la porte publique, vraie** : l'identité du document ne part plus au nuage, le code de la
    question d'accord est dans son titre.
* **La porte aux IA publiques marche avec son compte** (19/11, fiche 122) : **Claude.ai** est entré
  par son clic et a posé, signé « agent Claude (claude.ai) », l'onglet « Textes optimisés » de son
  document ; **Mistral** est entré aussi (Le Chat gratuit était saturé, il n'a rien posé) ; et
  **Claude Code** lit ses réponses par elle. Le critère de SUITE est rempli.
* **Son Glucose installé** (profil Nix `GlucoseGit`) : `aeea3ef`, du 17/11 au soir — sans rien de ce
  qui précède.
* **La machine** : son VivoBook sous **NixOS** (i7-8550U, 7,6 Go, Intel UHD 620 et MX150, écran 60 Hz,
  Wayland, niri), depuis le 6/11 — son PC Windows est cassé. **Windows, macOS et Android ne
  s'éprouvent qu'à travers la CI.** Quand il travaille, Brave, VS Code et d'autres sessions de Claude
  laissent souvent moins de 3,5 Go : rien ne se compile alors ici (la borne), et la vérification
  passe par GitHub, sur une branche retirée après la fusion.

## 2. Ce que fait Glucose Rust

Le détail, fonction par fonction et bouton par bouton : [INVENTAIRE](INVENTAIRE.md).

| domaine | ce qui marche |
|---|---|
| **Naviguer** | canevas infini, zoom jusqu'à ×256 ; souris, pavé (pincement sous Windows, Linux et macOS), doigts ; `F` cadre la sélection, `Alt+F` tout ; **`Alt+Z`** le `Ctrl+Z` de la caméra ; signets ; minimap ; onglets ; **`Ctrl+F`** cherche dans tout le document ; le mode Focus par un geste choisi |
| **Le document** | `.glucose` écrit geste après geste, brouillons, reprise au lancement, Time Machine (`Ctrl+H`) — **chaque geste signé**, ses écrits, **le journal des rectifications** de l'IA de Glucose —, les documents de Tauri lus ; au terminal, `glucose document.glucose` et une commande par outil des agents (`glucose aide`) |
| **Images** | PNG, JPEG, WebP, GIF (figé), BMP ; coller, déposer ; rotation, recadrage, **retourner** (`Alt+Maj+H`, `Alt+Maj+V`) ; **détourer** la forme principale, **retirer le fond uni** ; la mémoire par étages |
| **Texte** | Markdown, **tableaux qui tiennent dans leur nœud**, leurs rangées séparées d'un filet, **listes imbriquées**, liens, LaTeX ; net à tout zoom (de près, la carte graphique dessine les lettres depuis leur forme) ; de loin, le gris de ses lignes ; son papier toujours le gris sombre des blocs ; **sa lueur dit sa parenté** (une IA du sens embarquée, sans réseau) ; un texte qui grandit pousse sa carte, **et sa membrane** ; **les émojis en couleur** (Twemoji), un caractère à la fois |
| **Flèches** | droites ou courbes, six relations, coudes (« Retirer ce coude », « Redresser la flèche »), bouts qu'on prend, contournement, ancres de texte ; la couleur des lueurs qu'elles relient ; de loin, une trace ; leur présence suit leur nombre et la souris (PARENTE-4) ; `Ctrl`+clic suit un lien ; la fenêtre des liens (`B`) |
| **Organiser** | membranes **en carte** (la plus petite région l'emporte), dossiers (renommés par `F2`), **domaines** (qui traversent les fichiers), `Ctrl+G` grouper, disposer en colonnes, aimant, Ordonner, Mood board, Storyboard, les opérateurs des post-it ; la barre de couleur et la roue (Oklch) ; **dater** un nœud (`Maj+T`), la frise chronologique (`Maj+R`), regarder un domaine — deux lentilles qui estompent, jamais ne cachent |
| **Exporter** | Markdown, SVG complet, PNG, planches de storyboard (PDF), la page web |
| **L'interface** | 116 commandes dans une table, la palette (`Ctrl+Maj+P`), les raccourcis comme donnée ; ses lettres et ses icônes sur la grille de l'écran ; **son échelle, au choix** ; **les tiroirs du côté de leur bouton** ; les trois modes de la fenêtre et le zen ; sous Windows, la bande ou la barre du système |
| **Collaborer** | **le co-working** (fiches 117 et 119) : partager un canevas, le rejoindre par le code ; chaque geste vu chez les autres, chiffré, ses photos comprises ; le conflit dit ; `Ctrl+Z` par main ; les curseurs |
| **Les agents** | **une IA du nuage par Collaborer** — Claude.ai et Mistral, essayés depuis son compte : son accord devant un code, le document ouvert seulement, son nom vérifié qui signe, le témoin qui dit l'onglet ; `glucose --mcp` pour une IA de sa machine — chaque geste annulable, signé, rien sans son accord |
| **Mise à jour** | automatique et signée sous Windows et Linux ; sous Android, à prouver |

## 3. Ce qui ne va pas

**Vu par lui** (VISION § 5 à 7) : la carte de Wikipédia — pas encore assez lisible, trop de flèches
qui partent des blocs et non des mots, un tracé brouillon, et **il lague** : l'écran montre une image
neuve à 93-99 % des battements (PARU-1, fiche 112), mais il reste deux battements de l'envoi à l'écran
et une gigue d'un battement, et le travail quand il zoome (sa session sur batterie du 14/11 : 46,5 %
de battements sautés). Le halo de la parenté, les membranes à 11 % : pas encore à son écran.

**Vu par nous, pas encore réglé** :

* **une famille d'émojis se décompose, un drapeau devient deux lettres** : sans moteur de mise en
  forme, Glucose dessine un caractère à la fois (`decisions/10` et `18`) ;
* **la télémétrie compte une mise en veille comme une image** : 415 s chez un utilisateur Windows à
  carte AMD, la machine à 0 % pendant ces sept minutes — la « pire image » en est faussée. **Sous
  Windows seulement** : l'`Instant` de Rust y compte la veille, pas sous Linux, Android et macOS
  (fiche 126 § 2) ;
* **les tiroirs ne défilent pas** : quand la hauteur manque même en colonnes, deux se croisent
  (Storyboard et Domaines à 1440 × 900 avec huit domaines) ;
* **ses deux serveurs vivent sur le compte Cloudflare d'un autre projet** (`ferme-nilslamber`, celui de
  `ferme-tiles`) : leur nom est dans l'adresse que chaque utilisateur colle dans Claude.ai, et les
  quotas gratuits sont partagés — le déménagement attend son clic (§ 6) ;
* les versions d'avant la 2.0.4-beta.1 refusent un document qui porte un geste signé (PERSIST-5) :
  message net, rien de perdu ;
* l'étiquette d'une photo en chemin ne suit pas le zoom ; choisir 8 000 images coûte 28 ms par
  image ; le fondu d'un message redessine toute la carte (REPOS-3) ; les titres des membranes ne
  suivent pas encore la loi du gris de loin ;
* **la parité avec Glucose Tauri n'est pas atteinte** : ce qui manque, [INVENTAIRE](INVENTAIRE.md) § 3 ;
* **jamais essayé à son écran** : presque tout ce qui est venu depuis la fiche 62 — la liste unique est
  dans [SUITE](SUITE.md) § 3.

## 4. La qualité

* **Les épreuves** : sur GitHub, `36adb41` **vert sur les dix tâches**, le Mac à puce Apple compris
  (le 19/11 à 18 h 58 : tout le travail des fiches 122 à 126 — `gh run view 44578416447`). Ici, la
  bibliothèque du bureau (2 163) et du noyau (805) passent. Clippy strict et `cargo fmt` à zéro ; les
  **cliquets** (80 lignes par fonction, 600 par fichier, le couplage au modèle — 38 lignes, au
  plafond) ne se relèvent jamais : ce qui déborde s'extrait.
* **Les sabotages** : chaque garde est cassée exprès et doit faire tomber une épreuve
  (`outils/sabotages/`, un fichier par chantier) ; **`saboter.py` joue d'abord le code intact (le
  témoin), exige une épreuve réellement tombée et la nomme** (fiches 125-126). Joués avec témoin :
  `texte-fit-124` 9 sur 9, `emoji-125` 8 sur 8, `table-125` 7 sur 7, `liste-123` 7 sur 8, `temoin-123`
  2 sur 2, `table-123` 10 sur 10, `tiroirs-122` 6 sur 6, `porte-122` 3 sur 3, `applique-122` 2 sur 2,
  `echelle-121` 5 sur 5 (trois gardes étaient aveugles : leurs épreuves sont écrites), `emoji-126`
  9 sur 9 ; **un échantillon de 24 anciens sabotages** tirés au hasard dans 18 fichiers : 24 sur 24
  (fiche 126 § 3). **Pas encore joués** : le dernier de `liste-123` (la puce en vecteurs),
  `detourage-121` (25), `veille-126` (2). 1 380 des 1 527 sabotages écrits s'appliquent encore au
  code d'aujourd'hui ; les autres gardaient du code réécrit.
* **Regarder** : le vrai Glucose au pilote, hors écran, sur une copie de ses documents
  (`outils/pilote/piloter.sh`) ; **le rythme ne se juge qu'à son écran** (`outils/pilote/essai_ecran.sh`,
  la section PARU-1 de la chronique).
* **Sur GitHub, dix tâches** (Windows, Linux, les deux Mac, le noyau pour Android et le web, l'APK,
  NixOS, l'installeur et la mise à jour, la répétition de la bascule depuis Tauri, les paquets Linux) :
  la dernière verte est `36adb41` (19/11, 18 h 58 ; ci-dessus). Elle se lance avant une
  publication, ou comme voie de vérification quand sa machine est pleine.

## 5. Ses utilisateurs, et ce qu'on en sait

* **La télémétrie** (avec l'accord de chacun ; [TELEMETRIE](TELEMETRIE.md)) au 19/11, 18 h 30 : **208
  sessions, 10 installations, 40 fins anormales** — aucune neuve depuis le 17/11.
  * **son PC de l'école** (GTX 1660 SUPER, Windows) : 61 sessions, **en 2.0.4-beta.3 depuis le
    19/11** (la mise à jour automatique), rien d'anormal (pire image 1,1 s, au repos) ;
  * **son téléphone** (moto g54, PowerVR) : 30 sessions, 19 fins anormales — **aucune depuis la
    2.0.4-beta.1** (6 sessions, finies en arrière-plan comme Android le fait) ; prometteur pour
    MEMOIRE-4, pas encore une preuve ; 16,4 ms l'image typique ;
  * son VivoBook (UHD 620, la version `dev`) : 12 sessions jusqu'au 18/11 ;
  * un Linux à RTX 4070 SUPER, en 2.0.4-beta.1 (21 sessions, la dernière le 17/11) ;
  * **un Windows à AMD Radeon 660M**, en 2.0.4-beta.2 (9 sessions, aucune fin anormale) — l'image de
    415 s, très probablement une veille (§ 3) ;
  * un Windows à RTX 5070 portable, resté en 2.0.3-beta.6 (57 sessions, la dernière le 23/10) ; un
    Windows à GTX 1660 SUPER resté en 2.0.3-beta.3 ; un Windows neuf en 2.0.2-beta.2 ; un Android à
    Mali, une session ; un Windows à pilote Intel tué par `igvk64.dll` (le 29/10, en beta 6).
* **Ses utilisateurs connus** : 5 de Glucose Tauri (Windows, Linux) ; une vingtaine de testeurs qui
  attendent Glucose Rust, dont 10 sous Android ; une personne qui compte énormément pour lui, sur **iPad**
  (Glucose dans Safari, plus tard).
* **Il ne peut dépenser aucun argent** : pas de certificat Windows, pas de compte Apple, pas de compte
  Play, pas de nom de domaine. Android passe par un APK installé à la main.

## 6. Points de vigilance

* **Ses données avant tout** : son dossier Glucose (`~/.local/state/glucose`) ne se touche jamais ;
  tout diagnostic se fait sur une copie ; son document se lit par sa porte.
* **Le compte Cloudflare de Glucose** : proposé (fiche 123 § 7) et demandé dans son Glucose (le
  19/11, à 14 h) — son clic *Create Account*, et le nom (`glucose-canevas` proposé). Ensuite, **le
  serveur avant le client** : la boîte noire et la porte sur le compte neuf, l'ancienne adresse de la
  boîte noire qui transmet (le Worker est prêt), la base recopiée, puis les adresses neuves dans la
  beta.3 ; l'ancienne porte reste ouverte pour les versions installées — pendant la transition, deux
  personnes ne partagent un canevas que si elles ont la même version.
* **La clé Android** : dans les secrets du dépôt, et chiffrée dans
  `/stockage/ancien-windows/bureau/sauvegarde-documents-2026-10-31.7z`.
* **La clé de mise à jour** (`glucose_updater.key`) : le secret du dépôt et deux copies sur ce PC —
  **aucune hors de cet ordinateur** : une copie sur une clé USB à lui reste à faire.
* **Deux branches, pas une de plus** : `main` et `tauri-v1.0.1`.
* **Publier reste son geste** : le brouillon se prépare, le clic « Publish release » est le sien.
