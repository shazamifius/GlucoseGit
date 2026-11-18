# Où en est Glucose

> **Au 18/11/2026** (fiches 108-120). **Le présent, en une page** : ce qui est publié, ce qui marche, ce qui ne
> va pas, ce qu'on sait de ses utilisateurs. Il se **réécrit** quand la réalité change — il ne garde pas
> l'histoire, qui vit dans le [carnet](carnet/00-INDEX.md) (une fiche par session). La direction et ses
> paroles : [VISION](VISION.md). Ce qu'on fait ensuite : [SUITE](SUITE.md). Chaque fonction et son
> bouton : [INVENTAIRE](INVENTAIRE.md).

---

## 1. En bref

* **En brouillon, à son clic** : la **2.0.4-beta.2** (refait le 17/11 à 23 h 12 pour porter les photos
  du co-working : la CI verte sur `c8d7585`, dix tâches, puis le flux « Publier », onze fichiers) — la
  correction de Windows, le co-working et ses photos, la porte publique, la frise, le miroir ; laisser « Set as the latest release » coché, **ne pas** cocher « pre-release ».
* **Publiée** : la **2.0.4-beta.1** (son clic, le 13/11 à 18 h 23 ; la CI verte sur `f9027fd`, dix
  tâches ; fiche 111) — installeur Windows, AppImage, `.deb`, `.rpm`, APK Android, et `nix run
  github:shazamifius/GlucoseGit`. Chaque Glucose Tauri installé passe de lui-même à Glucose Rust ; ses
  documents s'ouvrent, ses données ne sont jamais touchées. Elle porte, depuis la beta 10 : `Alt+Z`
  d'un geste (RETOUR-2), les liens près des blocs (LIEN-4), les titres qui ne refont plus leurs
  lettres (LETTRES-1 : de loin, sur la carte de Wikipédia, 25,5 → 15,5 ms par image) ; **les
  membranes en carte** (REGION-1, fiche 105 : la plus petite région l'emporte, le papier des textes
  toujours gris, une membrane sans couleur grise ; au fond de « Eau », 7,8 ms par image au lieu de
  17,6) ; la souris qui suit les nœuds par les rangs de l'index (SURVOL-1 et 2, fiche 106 : sur la
  carte, 8,2 → 4,9 ms) ; **le halo d'un texte dit sa parenté** (PARENTE, fiche 106 : une petite IA du
  sens embarquée, des familles, une sûreté ; la teinte de bruit de Tauri partie, flèches comprises) ;
  **les membranes deux fois plus transparentes** (11 % — *« vraiment vraiment parfait »*, fiche 107) ;
  **les titres des membranes posés par la carte graphique** et deux mémoires pures (TITRE-3, fiche
  108 : de loin, 17 → 7,5 ms par image) ; **la Time Machine, socle de l'IA** (fiche 108) : chaque
  geste signé — l'humain, ou un agent sous son nom —, un écrit qui s'attache à un geste, une IA qui
  relit toute l'histoire avec ses écrits ; **l'IA ne juge que les flèches qu'un agent ou un script a
  posées** (PARENTE-4) ; **le zoom qui réutilise le repos** (REPOS-4, fiche 109) ; **le journal des
  rectifications** (RECTIF-1, fiche 110) : quand il fait autrement que l'IA, l'IA de Glucose l'écrit
  dans l'histoire, avec ce qu'elle pensait et **pourquoi** ; **la cascade qui compte la fin de
  l'image** (QUEUE-1, fiche 110).
* **Sur `main`, depuis la 2.0.4-beta.1** : **ce qui a paru à l'écran** (PARU-1, fiche 112 : sous
  Wayland, le compositeur dit pour chaque image si elle a paru, à quel battement, ou si elle a été
  jetée) ; **une seule loi dit si une commande s'applique** (APPLIQUE-1, fiche 113 : la palette, le
  menu du clic droit et l'exécution la lisent — plus de « Couleur » proposée sur une flèche ; le menu
  rangé par sens) ; **la date des nœuds** (DATE-1, fiches 113-114 : `Maj+T`, la palette ou le menu
  d'une sélection datent un nœud — une année, une plage, une ère ; une pastille ambre à son coin, sur
  les deux voies et dans l'export SVG ; une IA date par `glucose_dater` et lit les dates par
  `glucose_lire`) ; **le bleu des liens** éclairci (`#a8cfff`) ; **la frise chronologique et
  l'estompe** (FRISE-1 et 2, ESTOMPE-1 et 2, fiches 114-115 : `Maj+R`, une bande du bas qui pousse
  la minimap comme le clavier, les ères entières dans leurs couloirs, la répartition des dates, une
  fenêtre ambre — ce qui en sort pâlit à 12 %, sur les deux voies ; une flèche sans date pâlit avec
  ce qu'elle relie ; regardée dans le vrai Glucose, au PC et à la taille du téléphone) ; **regarder
  un domaine** (DOMAINE-1 et LENTILLE-1, fiche 115 : « voir » dans le panneau Domaines — ce qui
  n'appartient pas au domaine pâlit selon son poids, de 12 % à plein ; avec la frise, le minimum
  des deux lentilles) ; **la porte aux IA publiques** (PUBLIQUE-1, fiche 116 : **Collaborer** l'ouvre ;
  une IA du nuage — Claude.ai — entre par son clic dans Glucose, devant le même code que sa page, ne
  voit que le document ouvert et signe ses gestes de son nom vérifié ; **le relais est en ligne**,
  `https://glucose-porte.ferme-nilslamber.workers.dev`, sur son compte Cloudflare ; essayée de bout
  en bout par le vrai Glucose et un faux claude.ai — **pas encore avec son compte**) ; **le co-working**
  (CLIENT-1, fiche 117 : Collaborer → « Partager ce canevas », le code copié ; « Rejoindre » par le code
  collé ; chaque geste vu chez les autres, chiffré de bout en bout, la salle sur le même relais **en
  ligne** ; deux gestes sur le même nœud — le plus récent l'emporte, et le message le dit ; `Ctrl+Z`
  défait **ses** gestes ; les curseurs des autres ; essayé en ligne avec deux vrais Glucose, l'un
  derrière un réseau lent — **pas encore à son écran**) ; **l'export en page web** (EXPORT-HTML-1,
  fiche 115 § 7 ter : une page autonome, deux doigts, une flèche se lit — Safari jamais essayé) ;
  **sa capture d'écran du 17/11** (fiche 118) : **sous Windows, les boutons de la bande répondent où
  ils sont dessinés** (BARRE-2 — le clic les cherchait 138 px plus loin ; la CI le prouve sous un vrai
  Windows, lui l'avait constaté), un clic dans le vide de la bande ne sélectionne plus rien (VIDE-1),
  le fil d'Ariane d'un dossier se lit en entier (ARIANE-1), **les tiroirs ont la hauteur de ce qu'ils
  montrent et ne se chevauchent plus** (TIROIR-1 à 3), **les bouts d'une flèche se prennent** (BOUT-1),
  « Redresser la flèche » et « Retirer ce coude » au clic droit (COUDE-1), **retourner une image**
  (MIROIR-1 : `Alt+Maj+H`, `Alt+Maj+V` ; le document passe au schéma 6, qu'une 2.0.4-beta.1 refuse par
  son message) ; **les photos du co-working** (B1, fiche 119 : celles qu'on pose partent avec le geste,
  celles qui manquent se demandent en entrant ; prouvées par deux vrais Glucose à travers le relais en
  ligne, redéployé — **pas encore à son écran**) ; **la suite de sa capture, le 18/11** (fiche 120) :
  **le texte et les icônes de l'interface nets** (GRILLE-1 et 2 : la ligne de base et la hauteur d'x
  sur la grille, la position exacte au quart de pixel, des traits d'icône de pixels entiers — le
  canevas inchangé), **les trois modes de la fenêtre** (FENETRE-1 : `F11` le plein écran,
  `Ctrl+Maj+A` la référence, le zen à `Maj+F11`), **toujours devant sous Wayland** par la voie de
  chaque bureau (DEVANT-1 : niri, KDE, Hyprland, sway ; GNOME le dit — essayé sous niri seulement),
  **la barre de Windows au choix** (BARRE-3, jamais essayée sous Windows), **renommer un dossier**
  (RENOMMER-1, `F2`).
  **Son Glucose installé** (profil Nix `GlucoseGit`) : `aeea3ef` (17/11, 22 h 49) — la porte
  publique, le co-working et ses photos, la page web et toute sa capture compris.
* **La machine** : son VivoBook sous **NixOS** (i7-8550U, 7,6 Go, Intel UHD 620 et MX150, écran 60 Hz,
  Wayland, niri), depuis le 6/11 — son PC Windows est cassé. **Windows ne s'essaie plus qu'à travers la
  CI**, alors que la plupart de ses utilisateurs y sont.
* **Le co-working** (sa parole du 14/11) : **fait jusqu'au bout**, photos comprises depuis la fiche 119,
  sauf **le partage qui survit à un
  relancement** (on rejoint de nouveau par le code). Le reste attend : son nom changé dans l'interface, le conflit par
  propriété, le téléphone (fiche 117 § 7).
* **Le chantier du lag** : **son lag, mesuré à son écran** (fiche 112) — le « 32 % d'images
  ratées » de la fiche 111 était le compteur du tempo, pas l'écran : niri montre une image neuve à
  93-99 % des battements ; le prototype où niri donne le rythme est écarté (il saute jusqu'à 16 %
  des battements). Ce qui reste : **deux battements (33 à 42 ms) entre l'envoi d'une image et
  l'écran**, avec une gigue d'un battement — un calage sur le battement l'efface (prototype, à
  écrire) — et le travail quand il zoome. **Sa première vraie session lue par PARU-1** (14/11, 1 h 45, fiche 116 § 1) : 19 % des
  battements sautés alors qu'une image était due, 46 ms de l'envoi à l'écran en médiane, 93 au p90.
  Et la carte de Wikipédia — ses cinq défauts (VISION § 7) —, et le cercle de
  l'IA bâti sur la Time Machine (VISION § 8 bis : elle propose, il corrige, Glucose note, elle
  apprend — les trois premiers temps sont écrits). L'allure des frontières des membranes attend sa
  parole. **Sa carte de Wikipédia signée** : `~/Documents/Glucose-essai-wikipedia-signee.glucose`,
  reposée par la version neuve (1 870 gestes de « agent le terminal ») — l'IA en juge les flèches ;
  l'ancienne (`…-liens.glucose`), posée avant MAIN-1, reste telle quelle.

## 2. Ce que fait Glucose Rust

Le détail, fonction par fonction et bouton par bouton : [INVENTAIRE](INVENTAIRE.md).

| domaine | ce qui marche |
|---|---|
| **Naviguer** | canevas infini, zoom jusqu'à ×256 ; souris, pavé (pincement sous Windows, Linux et macOS), doigts ; `F` cadre la sélection, `Alt+F` tout ; **`Alt+Z`** le `Ctrl+Z` de la caméra ; signets ; minimap ; onglets ; **`Ctrl+F`** cherche dans tout le document ; le mode Focus par un geste choisi |
| **Le document** | `.glucose` écrit geste après geste, brouillons, reprise au lancement, Time Machine (`Ctrl+H`) — **chaque geste signé**, ses écrits, **le journal des rectifications** de l'IA de Glucose —, les documents de Tauri lus ; au terminal, `glucose document.glucose` et une commande par outil des agents (`glucose aide`) |
| **Images** | PNG, JPEG, WebP, GIF (figé), BMP ; coller, déposer ; rotation, recadrage ; la mémoire par étages |
| **Texte** | Markdown, tableaux, liens, LaTeX ; net à tout zoom (de près, la carte graphique dessine les lettres depuis leur forme) ; **de loin, le gris de ses lignes** ; **son papier, toujours le gris sombre des blocs** (PAPIER-1) ; **sa lueur dit sa parenté** : une IA du sens embarquée (15 Mo, multilingue, sans réseau) range les textes en familles — même couleur, d'autant plus forte et vive qu'elle est sûre ; un texte sans parent garde le contour gris de son bloc ; la couleur choisie va à sa lueur (PARENTE, fiche 106) |
| **Flèches** | droites ou courbes, six relations, coudes, contournement, ancres de texte ; **la couleur des lueurs qu'elles relient** (grises vers une photo) ; **de loin, une trace** ; leur présence suit leur nombre et la souris — **la souris ne rapproche que les flèches de sens** parmi celles qu'un agent ou un script a posées (PARENTE-4) ; `Ctrl`+clic suit un lien ; la **fenêtre des liens** (`B`) amène ses voisins |
| **Organiser** | membranes (**en carte** : la plus petite région l'emporte, titre à la taille de la région), dossiers (**renommés** par `F2`), **domaines** (qui traversent les fichiers), `Ctrl+G` grouper, disposer en colonnes, aimant, Ordonner, Mood board, Storyboard, les opérateurs des post-it ; la barre de couleur et **la roue** (à la Blender, dans Oklch) ; **dater** un nœud (`Maj+T`), **la frise chronologique** (`Maj+R`) et **regarder un domaine** (« voir ») : deux lentilles qui estompent ce qu'elles écartent, jamais ne cachent |
| **Exporter** | Markdown, SVG complet, PNG, planches de storyboard (PDF), **la page web** (deux doigts, une flèche se lit) |
| **L'interface** | 116 commandes dans une table (`Commande::toutes`), la palette (`Ctrl+Maj+P`), les raccourcis comme donnée, changés dans Glucose ; **ses lettres et ses icônes posées sur la grille de l'écran** (GRILLE-1 et 2) ; **les trois modes de la fenêtre** (normale, plein écran `F11`, référence `Ctrl+Maj+A`, toujours devant sous Wayland par la voie du bureau) ; **le mode zen** (`Maj+F11`) ; sous Windows, la bande ou la barre du système, au choix |
| **Collaborer** | **le co-working** (fiche 117) : partager ce canevas, le rejoindre par le code ; chaque geste vu chez les autres, chiffré ; le conflit dit ; `Ctrl+Z` par main ; les curseurs ; un code pour regarder seulement |
| **Les agents** | **une IA du nuage par Collaborer** (Claude.ai : la porte publique, son accord devant un code, le document ouvert seulement, *Lire seulement* au choix, son nom vérifié qui signe — fiche 116) ; `glucose --mcp` : une IA lit le tableau (par niveaux, ses domaines, ce que l'humain a choisi, la parenté que l'IA de Glucose donne aux textes et aux flèches) et y pose textes, membranes, flèches, domaines — chaque geste annulable, signé de son nom, rien sans son accord ; elle attache un écrit à un geste (`glucose_noter`) et relit l'histoire (`glucose_histoire`) |
| **Mise à jour** | automatique et signée sous Windows et Linux ; sous Android, à prouver |

## 3. Ce qui ne va pas

**Ce qu'il voit, ses mots du 11/11** (la carte de Wikipédia, son usage réel) — le détail dans VISION § 5 à 7 :

* **pas encore lisible** : les membranes emboîtées s'empilaient en boue — **réglé** (fiche 105), et deux
  fois plus transparentes (fiche 107, sa parole sur les images) ; pas encore à son écran ; l'allure des
  frontières attend sa parole ;
* **trop de flèches**, qui partent des blocs et non des mots, au tracé brouillon (surtout en courbe) ;
  l'IA ne juge que les flèches d'un agent ou d'un script — celles de sa carte actuelle n'en sont pas
  (posées avant MAIN-1) ;
* **il lague** — **mesuré à son écran, par ses gestes** (fiche 112 : un pavé tactile virtuel, le
  pincement et les deux doigts) : l'écran montre une image neuve à 93-99 % des battements ; le
  « 32 % ratées » de son essai du 12/11 à 18 h 16 était le compteur du tempo. Ce qui se voit et se
  sent : **deux battements de l'envoi à l'écran** (33 à 42 ms ; la carte graphique est partagée
  avec niri), une gigue d'un battement d'une image à l'autre, 5 % de battements sautés zoomé dans
  « Eau », et le travail quand il zoome (dans sa session, les flèches à 9-12 ms). Avant : sur son VivoBook, la télémétrie dit **13,8 ms l'image typique, 2,6 s la pire** (avant
  la fiche 105) ; remesuré le 12/11 sur sa carte (fiche 108) : **de loin 7,5 ms** (17 avant), au fond de
  « Eau » 2,8 ms ; **au zoom** : une image du mouvement 4-5 ms au lieu de 19-21 (fiche 109), et les
  images qui ratent un battement de l'écran 8-10 → 3 par série de crans, p90 24 → 17 ms (fiche 110) ;
  reste une image sur six autour de 18-22 ms ; pas encore à son écran. **Sa session du 14/11 à 4 h 37, sur
  sa carte et sur batterie** (fiche 117 § 1) : 46,5 % de battements sautés, « ornements » jusqu'à 33 ms ;
* **le halo d'un texte** dit sa parenté depuis la fiche 106 — pas encore à son écran ; beaucoup de
  flèches de la carte sont grises (leurs bouts n'ont pas de famille sûre) : acceptées *« seulement si
  c'est temporaire »* ; sous une quinzaine de textes, l'IA hésite ; la clarté (l'accessibilité) n'est
  pas encore lue.

**Connus, mesurés, pas réglés** :

* **les versions d'avant la 2.0.4-beta.1 refusent un document qui porte une entrée neuve** (un geste
  d'agent signé, un écrit — PERSIST-5, fiche 108) : message net, rien de perdu ; son téléphone et le PC
  de l'école sont passés à la 2.0.4-beta.1 (le 16/11), les autres utilisateurs le feront par la mise à
  jour. **Depuis RECTIF-1, un document ordinaire en porte dès qu'il colore un texte que l'IA rangeait
  dans une famille** ;
* **sa machine** : une suite d'épreuves sans borne l'a figée deux heures et demie le 12/11 (fiche
  110) — toute commande lourde passe désormais par `outils/borne.sh`, et `earlyoom` veille sur toute
  la machine (arrêter le plus gros programme avant l'étouffement) ;
* l'étiquette d'une photo en chemin ne suit pas le zoom (fiche 114 § 5) ;
* choisir 8 000 images coûte 28 ms par image (fiche 87 § 7 bis) ;
* le fondu d'un message redessine toute la carte (REPOS-3, fiche 98 § 7) ;
* les titres des membranes ne suivent pas encore la loi du gris de loin (fiche 104 § 3) ; les traits plus
  fins qu'un pixel hors des nœuds non plus (GRIS-3) ;
* **la parité avec Glucose Tauri n'est pas atteinte** : ce qui manque, [INVENTAIRE](INVENTAIRE.md) § 3 ;
* **jamais essayé à son écran** : presque tout ce qui est venu depuis la fiche 62 — la liste unique est
  dans [SUITE](SUITE.md) § 3.

## 4. La qualité

* **Les épreuves** : 3 339 vertes le 18/11 au matin (GRILLE compris, avant les témoins mis à jour) ;
  depuis FENETRE-1, seules les épreuves des chantiers touchés et les cliquets — **la suite entière
  reste à relancer** (`bash outils/borne.sh nix develop -c cargo test --workspace --no-fail-fast`) ; clippy strict et `cargo fmt` à
  zéro ; les **cliquets** (`tests/cliquets_suite.rs` : 80 lignes par fonction, 600 par fichier, le
  couplage au modèle…) ne se relèvent jamais.
* **Les sabotages** : chaque garde est cassée exprès et doit faire tomber une épreuve
  (`outils/sabotages/`, un fichier par chantier) ; les derniers : 66 sur 66 (fiches 102-103), 17 sur 17
  (REGION-1, fiche 105), 36 sur 36 (le sens et la parenté, fiche 106), 3 sur 3 (le voile des membranes,
  fiche 107), 46 sur 46 (fiche 108 : les flèches, les titres, les mémoires, la Time Machine, la
  provenance — trois épreuves aveugles trouvées et écrites), 15 sur 15 (le zoom, fiche 109), 22 sur
  22 (RECTIF-1 et le pourquoi, fiche 110 — une épreuve aveugle écrite), 4 sur 4 (QUEUE-1), 18 sur 18
  (PARU-1, fiche 112), 13 sur 13 (APPLIQUE-1, fiche 113 — deux sabotages qui passaient ont
  désigné une garde redondante, retirée), 21 sur 21 (DATE-1, fiche 114), 51 sur 51 (la frise et
  l'estompe), 16 sur 16 (le domaine regardé) et 12 sur 12 (le mode zen, fiche 115) ; 32 sur 32 (la porte
  publique, fiche 116 — dont une épreuve aveugle trouvée et écrite, en deux passes) ; 8 sur 8 (la page
  web), 29 sur 29 (le client du co-working, fiche 117), plus 1 du relais ; **34 sur 34 (sa capture et le
  miroir, fiche 118 — un sabotage aveugle trouvé et son épreuve écrite)** ; **fiche 120** : GRILLE 10 sur
  12 puis les deux portes aveugles gardées, le second chantier 18 sur 19 (l'épreuve du script de KWin
  resserrée, son sabotage pas encore rejoué).
* **Les bancs** : sous sway sans écran, présenter ne bloque jamais, et niri imbriqué bloque sur son
  hôte : **le rythme ne se juge qu'à son écran**, par `outils/pilote/essai_ecran.sh` (ses gestes,
  pavé compris) et la section PARU-1 de la chronique (fiche 112).
* **Sur GitHub, dix tâches** (Windows, Linux, les deux Mac, le noyau pour Android et le web, l'APK, NixOS,
  l'installeur et la mise à jour, la répétition de la bascule depuis Tauri, les paquets Linux) : **verte
  sur `4dfba2a`** (17/11 au soir, le co-working, sa capture et le miroir compris : le cadre de Windows
  éprouvé sous un vrai Windows).
  GitHub compile avec **Rust 1.100**, Nix ici avec 1.99 : un clippy plus récent peut trouver ce
  que celui d'ici ne voit pas (fiche 116 § 7). La CI se lance avant une publication, pas à chaque envoi.

## 5. Ses utilisateurs, et ce qu'on en sait

* **La télémétrie** (avec l'accord de chacun ; un serveur Cloudflare de son compte,
  [TELEMETRIE](TELEMETRIE.md)) au 17/11 : **190 sessions, 10 installations, 40 fins anormales**.
  * son VivoBook (Intel UHD 620, la version `dev` de son profil Nix) : 9 sessions du 7 au 12/11 ;
  * une GTX 1660 SUPER sous Windows, 51 sessions jusqu'au 16/11, **en 2.0.4-beta.1** — **très
    probablement lui, au PC de l'école** ;
  * un Linux à RTX 4070 SUPER, en 2.0.4-beta.1 (21 sessions, la dernière le 17/11) — ses fins anormales :
    l'appareil redémarré des heures après, un PC éteint Glucose ouvert ; un Windows à AMD Radeon
    660M, passé en beta 10 (4 sessions, aucune fin anormale) ;
  * **son téléphone** (moto g54, PowerVR) : 26 sessions, **19 fins anormales** ; **en 2.0.4-beta.1 depuis
    le 16/11**, sa première session dans cette version finie proprement ; 16,4 ms l'image typique ;
  * un Windows neuf (carte inconnue), deux sessions le 12/11 en **2.0.2-beta.2** — sa mise à jour
    automatique lui proposera la dernière ;
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
