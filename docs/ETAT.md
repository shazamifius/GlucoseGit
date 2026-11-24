# Où en est Glucose

> **Au 24/11/2026, 8 h** (fiches 108-140). **Le présent, en une page** : ce qui est publié, ce qui marche, ce
> qui ne va pas, ce qu'on sait de ses utilisateurs. Il se **réécrit** quand la réalité change — il ne
> garde pas l'histoire, qui vit dans le [carnet](carnet/00-INDEX.md) (une fiche par session). La
> direction et ses paroles : [VISION](VISION.md). Ce qu'on fait ensuite : [SUITE](SUITE.md). Chaque
> fonction et son bouton : [INVENTAIRE](INVENTAIRE.md).

---

## 1. En bref

* **Publiée** : la **2.0.4-beta.5** (son clic, le 23/11 à 19 h 39, construite sur `fcdd59f` — le
  code de `378239d`, **vert sur les dix tâches de GitHub**, run 45197129987) : les tiroirs qui se
  rangent dans l'écran et défilent (TIROIR-5 et 6) ; la loi de l'écart (TIROIR-2 — deux défauts de la
  beta.4 : un jalon perdu dans la Time Machine à 110 %, une case de moins dans le Storyboard) ;
  VEILLE-1 (sous Windows, une veille n'est plus un gel) ; DETOURER-2 ; **la discussion avec une IA
  gratuite** (Collaborer → « L'API publique… », le tiroir DISCUSSION, Gemini par sa clé, la bascule
  d'un modèle à l'autre, fiches 128-130) ; LECTURE-1 et 2 (une lecture d'agent n'attend plus son
  geste) ; ACCÈS-H24 (« Toujours, dans ce document », `glucose --mcp` qui survit à Glucose relancé, le
  partage qui rejoint sa salle au lancement, fiche 131) ; DOSSIER-2 (le rectangle prend les
  dossiers) ; MCP-DOSSIER (une IA voit et renomme les dossiers) ; l'opacité d'un dossier (OPACITE-1,
  fiche 133) ; le relais qui ne se fige plus sur « Claude in Chrome ». Avant elle, la **2.0.4-beta.4**
  (19/11, 19 h 15 ; ce qu'elle a apporté, plus bas) et la **2.0.4-beta.3** (19/11, 16 h 20 : le texte
  et les icônes nets, les trois modes de la fenêtre, `F2` sur un dossier). **Son PC de l'école** se
  met à jour tout seul. Installeur Windows, AppImage, `.deb`, `.rpm`, APK Android, `nix run
  github:shazamifius/GlucoseGit`.
* **Sur `main` depuis la beta.5, pas encore publié** (fiches 134-139 ; vérifié ici, sans la CI) :
  **FLECHE-7** — une flèche vers une membrane va droit à son bord (sa capture du 23/11) ;
  **RANGER-1 et 2** — ranger des nœuds dans un dossier, l'archive : une IA par `glucose_ranger`, sa
  main en les lâchant sur un dossier, un seul `Ctrl+Z` ; **FLECHE-EDITION** — les flèches d'un texte
  qu'on écrit partent du bas qu'on voit ; **MEMB-9** — une membrane qu'on trace prend ce qu'elle
  entoure ; **PICK-3** — le bord d'une membrane ne passe plus devant le texte qu'elle porte ;
  **PIPETTE-1 à 3** — viser la lueur d'un texte prend sa couleur, la roue montre ce qu'elle prendrait ;
  **REJOINDRE-1** — « Rejoindre… » ouvre un champ où coller le code (le téléphone) ; **ANCRE-1** — une
  flèche accrochée se choisit là où elle se dessine ; **EDIT-PRISE** — sélectionner dans un texte qu'on
  écrit ne quitte plus le bloc ; **Q12** — les équerres et le flou, essayés puis **retirés à sa demande** le 24/11 (fiche 142 § 5) ;
  **ARBITRE-5 et 6** — l'arbitre ne condamne plus une carte qui attend l'écran, et ne s'installe plus
  sur une machine à une seule carte (la sienne) (fiches 138-139) ; **DEPLACER-1** — une membrane
  déplacée par une IA emporte ce qu'elle porte, un dossier se déplace, une photo verrouillée non
  (fiche 139) ; **IDENTITE-2** — son nom et sa couleur, demandés la première fois qu'il partage
  ou rejoint un canevas, « Ton nom et ta couleur… » dans Collaborer, dix pastilles (fiche 140, sa
  réponse à Q19) ; **TABLE-5** — un tableau serré dans un nœud étroit ne coupe plus que le mot trop long (fiche 143, sa capture du 22/11) ; **MEMB-SEL-1** — dans le vide d'une membrane non choisie, glisser trace le rectangle de sélection, un clic la choisit (fiche 143 § 3, sa note du 24/11). **MCP-REPONDRE** — une IA répond à une note en un geste : sa réponse à la place libre la plus proche, dans la membrane de la note, la flèche « réponse » (`glucose_repondre`, fiche 143 § 4). **MCP-NEUF** — `glucose_quoi_de_neuf` : chaque nœud posé, changé ou retiré depuis le dernier geste de l'IA, par qui, où (fiche 144). **Son Glucose est un service** (`glucose.service`) : il se relance par `relancer_glucose.py`, qui écrit chez lui l'heure et ce qu'a fait le partage (`relais-relances`) — sa synchronisation de loin a coupé à une relance du relais (12 h 18, fiche 144 § 2) ; **la fermeture attend désormais le retour de ses gestes** (RETOUR-1) : un geste parti juste avant ne l'empêche plus de rejoindre sa salle au lancement. **SERVICE-1** : le relais MCP et la commande `glucose` rouvrent Glucose par son service, plus jamais une seconde fenêtre pendant une relance (vérifié à 14 h 06) ; **SCRIBE-1** : seule la fenêtre qui écrit le document reprend son partage. Chacun saboté et regardé au pilote. **Son Glucose** a tout cela : le relais l'a relancé
  par son service le 24/11 à 14 h 17 sur `88a2fbe` (une seule fenêtre, partage repris au geste n° 435).
* **Le relais** (fiche 132, sa parole du 23/11 : *« il ne fonctionne tout simplement PAS »*) : il s'était tu de 10 h 08 à 13 h 44 — sa consigne retouchée depuis son autre appareil (« Aigle Émeraude ») passait pour celle d'une IA, la passation fermait la session pour un compte plein, et rien ne lisait ses notes. **Réparé et relancé** : une seule session à la fois (la sienne prend la main, `rendre` la rend), toute main humaine signe sa consigne, jamais une IA, ses notes sont données à la session, jusqu'au mur quand personne ne peut reprendre, la relance sous le plafond de Claude Code (8). Mode d'emploi : `outils/relais/MODE-EMPLOI.md`. **Borné par les quotas** des deux comptes. **Le 23/11 au soir** (fiche 134), deux gels de plus, réglés : la question de Chrome au démarrage (1 h 48) et une permission du mode auto — désormais **`--no-chrome` et le bypass** (sa réponse : *« jamais de mode auto »*) ; le ❤️ dit « figée » ; **la mémoire du modèle à 90 %** relance une conversation neuve, la semaine passe la main à 90 % ; **ses textes** (départ, passation, relance) vivent dans son Glucose, et sa version part dès qu'il l'a écrite. **Le 24/11** (fiche 141), il l'a cru en panne : la vue de son VS Code du compte a ne suivait que la session de ce compte, vide toute la nuit (sa semaine pleine) pendant que le b travaillait — elle suit désormais la seule session qui vit, et sans elle dit pourquoi (`voir.sh`). **Les deux comptes sont Pro** : une fenêtre de 5 h ≈ 1 h 30 de travail d'Opus, 22 à 25 h par semaine pour les deux ; le H24 en demande sept fois plus (ses choix : SUITE § 4).
* **Les jetons du relais** (fiches 142 et 147, sa priorité du 24/11) : une conversation neuve à **300 000
  jetons**, un démarrage maigre, aucun compte rendu au terminal, les outils que le relais ne sert jamais
  retirés. **Mais la jauge d'un abonnement ne compte probablement pas la relecture** (fiche 147) : le seuil
  pourrait remonter — l'histoire des jauges (`jauges-histoire.jsonl`, `mesurer_jetons.py`) le tranchera.
* **La 2.0.4-beta.6 : son brouillon est prêt** (24/11, 18 h 06, sur `725411b`, la vérification GitHub verte
  sur les dix tâches) ; le clic « Publish » est le sien.
* **Un document, une salle** (fiche 146 § 4) : fait, sur `main` (pas encore dans une version) —
  « Quitter » garde le code, « Partager » revient dans la même salle, un arrêt net revient aussi, par la
  fusion à trois (REVENIR-1 et 2, fiches 147 § 6 et 148).
  Ce qui a changé des deux côtés : la salle l'emporte, la version d'ici est posée à côté.
* **Ce que la beta.4 a apporté, depuis la beta.3** :
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
* **Son Glucose** : la commande `glucose` lance le binaire compilé du dépôt
  (`target/release/glucose-desktop`), plus une version installée (22/11) — il a toujours ce qu'on a
  compilé en dernier. Son document de travail, `GLUCOSE PROJET.glucose` (rejoint par le code de sa
  salle), est à la racine du dépôt, exclu de Git ; `glucose --mcp` le donne à chaque session de Claude.
* **La machine** : **son poste fixe**, définitivement depuis le 22/11 au soir (Ryzen 5 7600X, 30 Go, RTX
  4070 SUPER, niri) — le dépôt, les mémoires et les journaux y ont été rapatriés du VivoBook à
  l'identique (même `87dfb5a`, même copie de travail, empreinte vérifiée), et `cargo check`, Clippy
  strict et la version optimisée y passent. Du 6 au 22/11, c'était son VivoBook (7,6 Go) ; son PC
  Windows est cassé. **Windows, macOS et Android ne s'éprouvent qu'à travers la CI.** La carte :
  `~/Projets/LISEZ-MOI.md`.

## 2. Ce que fait Glucose Rust

Le détail, fonction par fonction et bouton par bouton : [INVENTAIRE](INVENTAIRE.md).

| domaine | ce qui marche |
|---|---|
| **Naviguer** | canevas infini, zoom jusqu'à ×256 ; souris, pavé (pincement sous Windows, Linux et macOS), doigts ; `F` cadre la sélection, `Alt+F` tout ; **`Alt+Z`** le `Ctrl+Z` de la caméra ; signets ; minimap ; onglets ; **`Ctrl+F`** cherche dans tout le document ; le mode Focus par un geste choisi |
| **Le document** | `.glucose` écrit geste après geste, brouillons, reprise au lancement, Time Machine (`Ctrl+H`) — **chaque geste signé**, ses écrits, **le journal des rectifications** de l'IA de Glucose —, les documents de Tauri lus ; au terminal, `glucose document.glucose` et une commande par outil des agents (`glucose aide`) |
| **Images** | PNG, JPEG, WebP, GIF (figé), BMP ; coller, déposer ; rotation, recadrage, **retourner** (`Alt+Maj+H`, `Alt+Maj+V`) ; **détourer** la forme principale, **retirer le fond uni** ; la mémoire par étages |
| **Texte** | Markdown, **tableaux qui tiennent dans leur nœud**, leurs rangées séparées d'un filet, **listes imbriquées**, liens, LaTeX ; net à tout zoom (de près, la carte graphique dessine les lettres depuis leur forme) ; de loin, le gris de ses lignes ; son papier toujours le gris sombre des blocs ; **sa lueur dit sa parenté** (une IA du sens embarquée, sans réseau) ; un texte qui grandit pousse sa carte, **et sa membrane** ; **les émojis en couleur** (Twemoji), un caractère à la fois |
| **Flèches** | droites ou courbes, six relations, coudes (« Retirer ce coude », « Redresser la flèche »), bouts qu'on prend, contournement, ancres de texte ; la couleur des lueurs qu'elles relient ; de loin, une trace ; leur présence suit leur nombre et la souris (PARENTE-4) ; `Ctrl`+clic suit un lien ; la fenêtre des liens (`B`) |
| **Organiser** | membranes **en carte** (la plus petite région l'emporte), dossiers (renommés par `F2`, **on y range en lâchant dessus**), **domaines** (qui traversent les fichiers), `Ctrl+G` grouper, disposer en colonnes, aimant, Ordonner, Mood board, Storyboard, les opérateurs des post-it ; la barre de couleur et la roue (Oklch) ; **dater** un nœud (`Maj+T`), la frise chronologique (`Maj+R`), regarder un domaine — deux lentilles qui estompent, jamais ne cachent |
| **Exporter** | Markdown, SVG complet, PNG, planches de storyboard (PDF), la page web |
| **L'interface** | **la sélection de la beta.5** (les équerres et le flou de Q12 retirés le 24/11, à sa demande) ; 118 commandes dans une table, la palette (`Ctrl+Maj+P`), les raccourcis comme donnée ; ses lettres et ses icônes sur la grille de l'écran ; **son échelle, au choix** ; **les tiroirs du côté de leur bouton**, qui défilent quand la place manque ; les trois modes de la fenêtre et le zen ; sous Windows, la bande ou la barre du système |
| **Collaborer** | **le co-working** (fiches 117 et 119) : partager un canevas, le rejoindre par le code ; chaque geste vu chez les autres, chiffré, ses photos comprises ; le conflit dit ; `Ctrl+Z` par main ; les curseurs ; **le partage survit au relancement** (le document rejoint sa salle au numéro que son fichier contient, fiche 131) |
| **Les agents** | **une IA du nuage par Collaborer** — Claude.ai et Mistral, essayés depuis son compte : son accord devant un code, le document ouvert seulement, son nom vérifié qui signe, le témoin qui dit l'onglet ; `glucose --mcp` pour une IA de sa machine — chaque geste annulable, signé, rien sans son accord (**« Toujours, dans ce document »** : une fois pour toutes, tout programme de la machine) ; `glucose --mcp` survit à Glucose relancé ; une lecture n'attend jamais son geste, aucun agent n'attend derrière un autre ; **une IA gratuite (Gemini) dans le tiroir DISCUSSION**, qui bascule d'un modèle à l'autre |
| **Mise à jour** | automatique et signée sous Windows et Linux ; sous Android, à prouver |

## 3. Ce qui ne va pas

**Vu par lui** (VISION § 5 à 7) : la carte de Wikipédia — pas encore assez lisible, trop de flèches
qui partent des blocs et non des mots, un tracé brouillon, et **il lague** : l'écran montre une image
neuve à 93-99 % des battements (PARU-1, fiche 112), mais il reste deux battements de l'envoi à l'écran
et une gigue d'un battement, et le travail quand il zoome (sa session sur batterie du 14/11 : 46,5 %
de battements sautés). Le halo de la parenté, les membranes à 11 % : pas encore à son écran.

**Vu par nous, pas encore réglé** :

* **ECRAN-1 sous Linux** : la carte qui tient l'écran ne s'y lit pas ; sur un portable hybride, l'arbitre garde la main. Laissé faute d'une telle machine chez ses utilisateurs (fiche 139 § 1) — l'arbitre ne s'installe plus sur une machine à une seule carte (ARBITRE-6) ;
* **une famille d'émojis se décompose, un drapeau devient deux lettres** : sans moteur de mise en
  forme, Glucose dessine un caractère à la fois (`decisions/10` et `18`) ;
* **la télémétrie des versions publiées compte une mise en veille comme une image** (415 s chez un
  utilisateur Windows à carte AMD) : VEILLE-1 le règle dans la beta.5 ; les versions d'avant le gardent ;
* **le détourage laisse un liseré clair** au bord d'une photo prise sur fond blanc : le reflet du
  fond sur le sujet, que le masque garde à 0,99 — diagnostiqué, pas corrigé (fiche 127 § 5) ;
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

* **Les épreuves** : sur GitHub, **`378239d` vert sur les dix tâches** (le 23/11, run 45197129987 :
  la beta.5 ; l'IA gratuite et sa voie WinHTTP compilées sous Windows pour la première fois). Ici,
  **la suite entière, 3 593 épreuves, passe** (le 24/11, fiche 139 ; `test_queue_1`, qui tombait
  parfois sous la charge, ne lit plus l'horloge). Clippy strict et `cargo fmt` à zéro ; les
  **cliquets** (80 lignes par fonction, 600 par fichier, le couplage au modèle — 38 lignes, au
  plafond) ne se relèvent jamais : ce qui déborde s'extrait.
* **Les sabotages** : chaque garde est cassée exprès et doit faire tomber une épreuve
  (`outils/sabotages/`, un fichier par chantier) ; **`saboter.py` joue d'abord le code intact (le
  témoin), exige une épreuve réellement tombée et la nomme** (fiches 125-126). Joués avec témoin :
  `texte-fit-124` 9 sur 9, `emoji-125` 8 sur 8, `table-125` 7 sur 7, `liste-123` 7 sur 8, `temoin-123`
  2 sur 2, `table-123` 10 sur 10, `tiroirs-122` 6 sur 6, `porte-122` 3 sur 3, `applique-122` 2 sur 2,
  `echelle-121` 5 sur 5 (trois gardes étaient aveugles : leurs épreuves sont écrites), `emoji-126`
  9 sur 9 ; **un échantillon de 24 anciens sabotages** tirés au hasard dans 18 fichiers : 24 sur 24
  (fiche 126 § 3) ; à la fiche 127, `tiroirs-126` **13 sur 13**, `tiroirs-127` **5 sur 5**,
  `veille-126` **2 sur 2**, `liste-123` **8 sur 8** ; `detourage-121`, le dernier jamais joué,
  **25 sur 25** (le 24/11, fiche 137 : son modèle sous `opt-level = 1`) ; à la fiche 137, `equerres-137`
  **10 sur 10** ; à la fiche 130, `discussion-129` **25 sur 25** et `lecture-130` **11 sur 11** ;
  à la fiche 131, `relais-131` **12 sur 12**, `agents-toujours-131` **12 sur 12**,
  `relais-2-131` **7 sur 7**, `partage-survit-131` **10 sur 10** (et `saboter.py` lit `unittest`). 1 380 des 1 527 sabotages écrits s'appliquent encore au code d'aujourd'hui ; les
  autres gardaient du code réécrit.
* **Regarder** : le vrai Glucose au pilote, hors écran, sur une copie de ses documents
  (`outils/pilote/piloter.sh`) ; **le rythme ne se juge qu'à son écran** (`outils/pilote/essai_ecran.sh`,
  la section PARU-1 de la chronique).
* **Sur GitHub, dix tâches** (Windows, Linux, les deux Mac, le noyau pour Android et le web, l'APK,
  NixOS, l'installeur et la mise à jour, la répétition de la bascule depuis Tauri, les paquets Linux) :
  la dernière verte est `378239d` (23/11 ; ci-dessus). Elle se lance avant une
  publication, ou comme voie de vérification quand sa machine est pleine.

## 5. Ses utilisateurs, et ce qu'on en sait

* **La télémétrie** (avec l'accord de chacun ; [TELEMETRIE](TELEMETRIE.md)) au 20/11, 22 h : **219
  sessions, 10 installations, 40 fins anormales** — aucune neuve depuis le 17/11.
  * **son PC de l'école** (GTX 1660 SUPER, Windows) : 64 sessions, **en 2.0.4-beta.4** (la mise à
    jour automatique), rien d'anormal (pire image 1,1 s, au repos) ;
  * **son téléphone** (moto g54, PowerVR) : 32 sessions, en 2.0.4-beta.2, 19 fins anormales — **aucune depuis la
    2.0.4-beta.1** (6 sessions, finies en arrière-plan comme Android le fait) ; prometteur pour
    MEMOIRE-4, pas encore une preuve ; 16,4 ms l'image typique ;
  * son VivoBook (UHD 620, la version `dev`) : 12 sessions jusqu'au 18/11 ;
  * un Linux à RTX 4070 SUPER, en 2.0.4-beta.1 (21 sessions, la dernière le 17/11) ;
  * **un Windows à AMD Radeon 660M**, **en 2.0.4-beta.4** (14 sessions, la dernière le 20/11, aucune
    fin anormale) — l'image de 415 s, très probablement une veille (§ 3) ;
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
