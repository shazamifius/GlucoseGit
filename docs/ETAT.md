# Où en est Glucose

> **Au 23/11/2026, 19 h** (fiches 108-134). **Le présent, en une page** : ce qui est publié, ce qui marche, ce
> qui ne va pas, ce qu'on sait de ses utilisateurs. Il se **réécrit** quand la réalité change — il ne
> garde pas l'histoire, qui vit dans le [carnet](carnet/00-INDEX.md) (une fiche par session). La
> direction et ses paroles : [VISION](VISION.md). Ce qu'on fait ensuite : [SUITE](SUITE.md). Chaque
> fonction et son bouton : [INVENTAIRE](INVENTAIRE.md).

---

## 1. En bref

* **Publiée** : la **2.0.4-beta.4** (son clic, le 19/11 à 19 h 15, construite sur `9255281`, vérifiée
  par GitHub sur les dix tâches) — les émojis en couleur et les signes d'encre lisibles, les tableaux
  qui tiennent dans leur nœud, les listes imbriquées, les tiroirs du côté de leur bouton, l'échelle
  de l'interface, le détourage, un texte qui pousse sa membrane. Avant elle, la **2.0.4-beta.3** (son clic, le 19/11 à 16 h 20, construite sur `15beb94`) — sa
  capture du 17/11 en partie : le texte et les icônes de l'interface nets (GRILLE-1 et 2), les trois
  modes de la fenêtre (`F11`, `Ctrl+Maj+A`, le zen à `Maj+F11`), toujours devant sous Wayland
  (DEVANT-1), la barre de Windows au choix, renommer un dossier (`F2`) (fiches 120-121). **Son PC de
  l'école s'y est mis à jour tout seul** le jour même. Avant elle, la 2.0.4-beta.2 (18/11 : le
  co-working et ses photos, la porte aux IA publiques, la frise, le miroir des images). Installeur
  Windows, AppImage, `.deb`, `.rpm`, APK Android, `nix run github:shazamifius/GlucoseGit`.
* **Sur `main` depuis la beta.4** (`596e783`, **vert sur les dix tâches de GitHub**, le 20/11 à
  23 h 55 ; **pas encore publié** — la question d'une beta.5 est dans son Glucose) :
  * **les tiroirs qui se rangent dans l'écran et défilent** (TIROIR-5 et 6) — quand la hauteur
    manque, une colonne partage sa place et chaque tiroir fait défiler ce qu'il ne montre plus, à la
    molette ; regardés au pilote, 13 sabotages sur 13 ;
  * **la loi de l'écart** (TIROIR-2) — **deux défauts de la beta.4 publiée**, vus au pilote : à 110 %
    d'interface la Time Machine perdait un jalon, et le Storyboard montrait une case de moins, sur
    deux pages, dès deux cases ;
  * **VEILLE-1** — sous Windows, une image qui contient une mise en veille garde ce qu'elle a coûté
    (la télémétrie ne prend plus une veille pour un gel).
* **Commité depuis, pas encore éprouvé par GitHub** : `f9a8557` (DETOURER-2), `87dfb5a` (IA-1, le
  menu « L'API publique », fiche 128) et `48094df` — **la discussion avec une IA gratuite** (fiche
  129) : le tiroir DISCUSSION, Gemini par sa clé, les outils de la porte, la bascule qui dort le
  temps que Google dit et qui le dit, `plateforme::echanger` sous Windows et Linux. Toute la suite
  du dépôt passe, **25 sabotages sur 25**, **regardée au pilote avec sa clé** (Gemini a posé un
  nœud signé de son nom, fiche 130 § 6) ; la voie WinHTTP jamais compilée (la CI). Puis
  **LECTURE-1 et 2** (fiche 130) : une lecture d'agent n'attend plus jamais le geste de l'humain,
  aucune demande n'attend derrière une autre (le 22/11, son Glucose n'avait servi aucun agent
  treize minutes) ; `tools/list` dit quels outils ne font que lire. 3 481 épreuves, 11 sabotages,
  l'avant et l'après au pilote. Puis **ACCÈS-H24** (fiche 131, le relais seul, après le
  redémarrage du PC) : **« Toujours, dans ce document »** — tout programme de la machine y
  travaille sans question, d'un lancement à l'autre (`c8a3350`) ; **`glucose --mcp` survit à
  Glucose fermé et relancé** (`3d01ce3`) ; **un canevas partagé rejoint sa salle au lancement**,
  au numéro exact que son fichier contient ; et le relais tient compte des deux jauges
  (`6b0007b`). Chacun saboté et regardé au pilote, le binaire d'avant pour preuve à l'envers. Et **DOSSIER-2** (`d86f52f`) : le rectangle de sélection prend les dossiers, plusieurs dossiers se choisissent. Puis, le 23/11 (fiche 133) : **MCP-DOSSIER** (`9accdfe`) — une IA voit les dossiers, les renomme, sait où elle lit ; renommer un dossier nomme son tableau — et **OPACITE-1** (`378239d`) — **l'opacité d'un dossier dans la roue de couleur**, le nom toujours lisible ; regardé au pilote. **`378239d` est vert sur les dix tâches de GitHub** (run 45197129987), et **le brouillon de la 2.0.4-beta.5 est prêt** (le 23/11 à 18 h 03, run 45221747706) — son « oui » est donné, **le clic « Publish » reste le sien**. Depuis, sur `main` seulement : **FLECHE-7** — une flèche vers une membrane va droit à son bord, au lieu de contourner les nœuds qu'elle porte (sa capture du 23/11, fiche 134).
* **Le relais** (fiche 132, sa parole du 23/11 : *« il ne fonctionne tout simplement PAS »*) : il s'était tu de 10 h 08 à 13 h 44 — sa consigne retouchée depuis son autre appareil (« Aigle Émeraude ») passait pour celle d'une IA, la passation fermait la session pour un compte plein, et rien ne lisait ses notes. **Réparé et relancé** : une seule session à la fois (la sienne prend la main, `rendre` la rend), toute main humaine signe sa consigne, jamais une IA, ses notes sont données à la session, jusqu'au mur quand personne ne peut reprendre, la relance sous le plafond de Claude Code (8). Mode d'emploi : `outils/relais/MODE-EMPLOI.md`. **Borné par les quotas** des deux comptes. **Le 23/11 au soir** (fiche 134), deux gels de plus, réglés : la question de Chrome au démarrage (1 h 48) et une permission du mode auto — désormais **`--no-chrome` et le bypass** (sa réponse : *« jamais de mode auto »*) ; le ❤️ dit « figée » ; **la mémoire du modèle à 90 %** relance une conversation neuve, la semaine passe la main à 90 % ; **ses textes** (départ, passation, relance) vivent dans son Glucose, et sa version part dès qu'il l'a écrite.
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
| **Organiser** | membranes **en carte** (la plus petite région l'emporte), dossiers (renommés par `F2`), **domaines** (qui traversent les fichiers), `Ctrl+G` grouper, disposer en colonnes, aimant, Ordonner, Mood board, Storyboard, les opérateurs des post-it ; la barre de couleur et la roue (Oklch) ; **dater** un nœud (`Maj+T`), la frise chronologique (`Maj+R`), regarder un domaine — deux lentilles qui estompent, jamais ne cachent |
| **Exporter** | Markdown, SVG complet, PNG, planches de storyboard (PDF), la page web |
| **L'interface** | 116 commandes dans une table, la palette (`Ctrl+Maj+P`), les raccourcis comme donnée ; ses lettres et ses icônes sur la grille de l'écran ; **son échelle, au choix** ; **les tiroirs du côté de leur bouton**, qui défilent quand la place manque ; les trois modes de la fenêtre et le zen ; sous Windows, la bande ou la barre du système |
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

* **une famille d'émojis se décompose, un drapeau devient deux lettres** : sans moteur de mise en
  forme, Glucose dessine un caractère à la fois (`decisions/10` et `18`) ;
* **la télémétrie des versions publiées compte une mise en veille comme une image** (415 s chez un
  utilisateur Windows à carte AMD) : VEILLE-1, sur `main`, le règle — pas encore publié ;
* **le détourage laisse un liseré clair** au bord d'une photo prise sur fond blanc : le reflet du
  fond sur le sujet, que le masque garde à 0,99 — diagnostiqué, pas corrigé (fiche 127 § 5) ;
* **deux défauts de la beta.4**, réglés sur `main` mais pas publiés : un jalon perdu dans la Time
  Machine à 110 %, une case de moins dans le Storyboard ;
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

* **Les épreuves** : sur GitHub, **`596e783` vert sur les dix tâches**, les deux Mac compris (le
  20/11 à 23 h 55 : VEILLE-1 compilé et éprouvé sous Windows pour la première fois, les tiroirs, la
  loi de l'écart — `gh run view 44830190971`). Ici, la bibliothèque du bureau (2 176) et ses huit
  suites d'intégration passent. Clippy strict et `cargo fmt` à zéro ; les
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
  `veille-126` **2 sur 2**, `liste-123` **8 sur 8** — et `detourage-121`, le dernier jamais joué
  (fiche 127 § 9) ; à la fiche 130, `discussion-129` **25 sur 25** et `lecture-130` **11 sur 11** ;
  à la fiche 131, `relais-131` **12 sur 12**, `agents-toujours-131` **12 sur 12**,
  `relais-2-131` **7 sur 7**, `partage-survit-131` **10 sur 10** (et `saboter.py` lit `unittest`). 1 380 des 1 527 sabotages écrits s'appliquent encore au code d'aujourd'hui ; les
  autres gardaient du code réécrit.
* **Regarder** : le vrai Glucose au pilote, hors écran, sur une copie de ses documents
  (`outils/pilote/piloter.sh`) ; **le rythme ne se juge qu'à son écran** (`outils/pilote/essai_ecran.sh`,
  la section PARU-1 de la chronique).
* **Sur GitHub, dix tâches** (Windows, Linux, les deux Mac, le noyau pour Android et le web, l'APK,
  NixOS, l'installeur et la mise à jour, la répétition de la bascule depuis Tauri, les paquets Linux) :
  la dernière verte est `596e783` (20/11, 23 h 55 ; ci-dessus). Elle se lance avant une
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
