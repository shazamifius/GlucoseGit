# La suite, mise à jour par mise à jour

> **Au 10/11/2026** (fiches 73 à 102 ; **la beta 8 publiée par lui le 6/11, la beta 9 publiée par lui le 7/11** ; Protéo en pause, la partie 1 d'abord ; **sur son VivoBook sous NixOS**). **Le plan complet** de ce qui attend Glucose Rust, rangé en étapes :
> ce que chacune porte, et ce qui la déclare finie. Tout y est à remettre en question — ses
> paroles comprises. La direction vit dans [VISION](VISION.md), l'état vérifié dans
> [ÉTAT](ETAT.md), chaque fonction et son bouton dans [INVENTAIRE](INVENTAIRE.md), la cible
> PureRef dans [PUREREF](PUREREF.md).
>
> **Comment s'en servir** : prendre le premier chantier non fait de la première étape non finie.
> Fini = **prouvé** (sur une copie, sur GitHub, ou à son écran), jamais « ça compile ». Le cocher
> ici, mettre ÉTAT à jour, passer au suivant. Une version se publie quand ses chantiers sont
> prouvés — **le nom et le moment restent les siens**.

`[x]` fini et prouvé · `[~]` écrit, éprouvé, saboté — **attend son écran** · `[ ]` à faire

---

## 0. La partie 1 — sa parole du 25/10

*« J'aimerais mettre une pause à Protéo et essayer de définitivement finir la partie 1 de
Glucose : le logiciel complet, complet, en Rust, ultra optimisé, avec des utilisateurs et une
ergonomie parfaite. »*

**« Parfait » n'a pas de ligne d'arrivée** ; sans elle, rien ne se finit « définitivement ». La
session du 25/10 propose celle-ci (à remettre en question) : la partie 1 est finie, et s'appelle
**2.1.0, la première version stable**, quand ces six choses sont vraies :

1. **Un utilisateur de Glucose Tauri n'y perd plus rien** ([INVENTAIRE](INVENTAIRE.md) § 3 vide,
   sauf ce qu'il repousse lui-même).
2. **Aucun tableau perdu** : la mort du téléphone du 15/10 expliquée et corrigée ; Glucose finit
   proprement quand le système le ferme.
3. **100 images par seconde au PC, 60 sur un téléphone courant**, lues dans la télémétrie au pire
   (le 99ᵉ centile), pas en médiane — ou pixelisées sans que cela se voie.
4. **Chaque fonction a son bouton**, au PC et au téléphone.
5. **L'ergonomie** : le preset de Glucose et l'interface du téléphone refaite, jugés par lui.
6. **Toutes les machines qu'on peut servir sans argent.**

**Cinq étapes, dans cet ordre** : ne rien perdre ; tout ce qui manque ; fluide partout ; toutes
les machines ; l'ergonomie — en dernier, comme il l'a toujours dit (VISION : « l'ergonomie se fait
à la fin, quand toutes les fonctionnalités seront là »). Ce qui garde l'ergonomie possible
pendant les étapes 2 à 4 : chaque fonction neuve entre dans la table des commandes, ses
raccourcis dans la couche de texte, ses couleurs dans le thème (P4) — rien en dur.

**Ordre de grandeur, honnête** : 25 à 35 sessions comme celles d'octobre.

**Ce qui attend sa parole** (les défauts de la session, qu'il peut renverser) :

* **L'iPad par le web** dans la partie 1 ? Défaut : oui, en dernier des machines (la plus grande
  inconnue : Glucose dans Safari).
* **Les presets Figma et Miro** : défaut, la partie 2 ; la partie 1 porte le preset de Glucose et
  celui de PureRef.
* **« Avec des utilisateurs »** : lu comme « des gens qui s'en servent sans rien perdre ». Des
  comptes, une connexion, ce serait la collaboration : défaut, la partie 2 (le bouton dit
  « bientôt »).

### Pourquoi cet ordre — quatre principes, à contredire

1. **Ce qu'il voit d'abord.** Une version publiée sans son essai est un pari : chaque étape
   commence par juger la précédente à son écran, et par lire sa télémétrie.
2. **Ne jamais perdre un tableau** passe avant toute fonction neuve — sa panique devant une perte,
   et deux téléphones l'ont vécu.
3. **Le manque de fonctions bloque plus que la vitesse** (la charte : « le blocage actuel n'est
   pas la performance ») — mais **100 images par seconde, toujours**.
4. **Relire avant de bâtir** : chaque panneau contre Tauri (P6) *avant* que le thème et la
   disposition deviennent des données (les presets) — sinon on transcrit des panneaux qu'il
   faudra refaire.

---

## 1. Le plan

### Publiées

* **La 2.0.3-beta.5** (16/10) : P8, P8 bis, ses raccourcis (fiches 73, 74).
* **La 2.0.3-beta.6** (23/10, sur `4a111b4`) : la porte des agents et tout ce que les fiches 75 à
  81 ont fait (lire par niveaux, ce que l'humain désigne, FLECHE-6, ACCUEIL-1).

| | chantier publié | ce qui le déclare fini |
|---|---|---|
| [~] | **P8 — Le texte de près par la carte graphique** (fiche 73) : Slug, CADRE-1, SLUG-1 ; ~1 ms par image de près, contre ~20 et le flou | de près, aucun flou, sur sa machine et au téléphone |
| [~] | **P8 bis — Un nœud immense, de près** (fiche 73 § 7) : LIGNE-1 et la mémoire des mises en page — 10,5 ms → 0,72 ms à 17 600 mots | un très long texte, de près ; un tableau plein de textes |
| [~] | **P4 — Tes raccourcis** (fiches 72, 74) : la table des commandes, la palette, la couche de l'utilisateur | `Ctrl+Maj+P` ; changer un raccourci, relancer |
| [~] | **P10 — La porte des agents** (fiche 75) : `glucose --mcp`, quatorze outils (les domaines depuis la fiche 88, `glucose_chercher` depuis la fiche 90, `glucose_disposer` 92, `glucose_grouper` 98), et chacun une commande du terminal, chaque geste annulable, rien sans son accord ; lire par niveaux, `selection`, `empreinte`, `document`, `glucose_ouvrir_tableau` (fiches 79, 80) | un agent travaille dans son Glucose, sous ses yeux |

### Étape 1 — Ne rien perdre

| | chantier | ce qui le déclare fini |
|---|---|---|
| [~] | **MOTEUR-1 — Un pilote qui tue Glucose ne l'exclut plus** (fiche 84) : `01ca394b`, le 29/10, tué par `igvk64.dll` à chaque lancement ; l'échelle des moteurs, OpenGL réparé sous Wayland, ACCUEIL-2. Joué en réel sous Linux | la beta 7 publiée ; la télémétrie montre cette installation, ou une autre touchée, qui dessine après sa chute. **À chaque relevé : `plantages.json`** |
| [~] | **FIN-SESSION-1 — Glucose finit quand le système le ferme** (fiche 82) : sous Windows, `winit` ne transmet pas `WM_ENDSESSION` — l'extinction tuait Glucose, et la boîte noire lisait « interrompue » (deux sessions de 3 h 37 et 7 h chez un utilisateur, les 21 et 22/10) ; sous Linux, `SIGTERM` et `Ctrl+C` tuaient aussi. **FIN-1** : la fin, quelle qu'elle soit, s'écrit une fois (`finir`) | éteindre son PC avec Glucose ouvert : la session suivante dit « fermée proprement » ; la télémétrie ne compte plus d'extinctions parmi les fins anormales |
| [~] | **P0 — Ne jamais perdre un tableau au téléphone** : fait côté code (fiche 66 : MEMOIRE-1 à 3 ; fiche 72 : LIBERE-1, SORTIE-1, REPRISE-1). **Le 15/10, sur deux téléphones, un tableau de plus de 6 000 images s'est fermé seul et n'est pas revenu.** **La cause, reproduite le 25/10 sur un téléphone virtuel (fiche 83)** : « Tout cadrer » sur 6 000 images — les niveaux à offrir attendaient derrière les décodages, sans être comptés ; 3,1 Go, et Android a tué Glucose au premier plan. **MEMOIRE-4** la répare : Glucose survit, les 6 000 images dessinées. Reste : que la prochaine version arrive sur son téléphone, et que ses lignes `sortie` ne disent plus de mort par la mémoire ; le « non rouvert » de la beta 3 est défendu par LIBERE-1 (beta 4). **Au 25/10, toujours aucune session de son téléphone depuis le 15/10** (encore en beta 3) : **ouvrir Glucose sur son téléphone**. **Ne pas désinstaller** : le brouillon est peut-être encore là. Relevé : `node outils/telemetrie/tableau.mjs <dossier>` (7403 → `npx wrangler whoami`, puis une fois encore) | reproduit et réparé sur le téléphone virtuel ; puis vécu sur le sien, ses lignes `sortie` sans mort par la mémoire |
| [ ] | **Ses essais de la beta 6** (§ 2) : chaque défaut qu'il voit, corrigé — en premier, le texte de près | sa parole |
| [~] | **Les boutons qui se taisent** : Trans-domaines dit « bientôt — en recherche » **depuis la fiche 71** (épreuve `test_the_trans_domain_button_says_it_is_coming` ; SUITE et INVENTAIRE l'ignoraient jusqu'au 25/10) ; la carte d'accueil ne naît plus choisie (ACCUEIL-1, fiche 81). Tous deux dans la beta 6 | à son écran |
| [x] | **Le repos à zéro** — mesuré le 18/10 : 0,2 image par seconde sans la main, toutes pour le fondu du message de bienvenue | une chronique au repos sans image hors un message qui paraît |
| [x] | **REPOS-2 — sur une carte de Wikipédia, Glucose se pose** (fiche 98 § 1 ; sa parole du 9/11 : « yep ») : pas un cercle — une convergence ruineuse, une teinte et une texture par image complète ; le cache des teintes paie ses dettes au repos, sans image, et une seule image les montre. Banc hors écran, paires entrelacées : 293 → 26 images en 30 s de repos, un cœur 21,5 % → 2,3 % ; 7 sabotages sur 7. **Reste** (ancien, sans gêne d'usage) : le fondu d'un message redessine toute la carte | — |

### Ses six défauts sous Linux — sa capture du 9/11 au soir (fiche 100)

| | chantier | ce qui le déclare fini |
|---|---|---|
| [~] | **Le pont Wayland — copier, coller, glisser-déposer** (fiche 100 § 2 et 10) : un texte vers Discord, une image collée, une photo glissée dans les deux sens et posée sous le curseur ; `outils/pilote/essai_wayland.sh`, six contrôles sur six **sous un vrai niri réglé comme son écran** (et à l'échelle 2, et sous sway) | à son écran sous niri : copier un texte vers Discord, coller une image, glisser depuis Nautilus et depuis le navigateur, glisser une photo hors de Glucose |
| [~] | **La boîte montrée — le contour ne se décroche plus** (fiche 100 § 3) ; 5 sabotages sur 5 | à son écran : écrire un texte qui passe à la ligne |
| [~] | **LOT-2, PHOTO-2 — les photos ne clignotent plus** (fiche 100 § 4) ; revu au pilote, 3 sur 3 | à son écran : déplacer au clic molette sur beaucoup de photos |
| [~] | **TEMPO-3, SOURIS-1 — les micro-gels** (fiche 100 § 5) : le tempo borné au plancher, la souris par pixels entiers, la translation entière garde les textures du repos | à son écran, et sa prochaine chronique : un balayage par image au clic molette |
| [x] | **Le README** (fiche 100 § 6) : réécrit, chiffres vérifiés ; le guide corrigé | — |
| [~] | **GTK 3 retiré** (fiche 100 § 6) : `rfd` ne l'emploie plus ; le paquet Nix (construit ici) et la vérification Linux sans lui, **la CI verte le 10/11** ; **reste le `.deb`** (`Depends: libgtk-3-0` tire les vraies bibliothèques du lancement) | le `.deb` dit ses vraies dépendances, essayé sur une Debian |
| [~] | **Le pont Windows convertit sa position** (`ScreenToClient`, fiche 100 § 2) : **la CI verte le 10/11** (dix tâches sur dix) | un dépôt sous Windows posé sous le curseur |
| [ ] | **Glisser un nœud vers une fenêtre posée PAR-DESSUS Glucose** (fiche 100 § 10) : le glisser sortant ne part que quand la main quitte la fenêtre ; vrai sous Windows aussi | **sa parole** : le veut-il, et par quel geste |
| [~] | **BAS-1 — le bas de l'écran** (fiche 101) : à la taille de son téléphone, la barre de couleur sortait de l'écran et couvrait la minimap ; la barre des flèches faisait 699 points. Une barre passe à la ligne comme un texte et monte au-dessus de la minimap, qui ne bouge jamais ; un bouton à deux états ne bouge plus quand on le bascule. Une propriété de 320 à 2 560 points ; 10 sabotages sur 10 ; regardé au moto g54, au Redmi 9 et au bureau | sur l'APK, à son téléphone : choisir un texte, puis une flèche et un texte ; **sa parole** si la pile des trois barres gêne |

### Le prochain lot — sa parole du 9/11 après-midi (VISION, fin de « Wikipédia sur le canevas »)

| | chantier | ce qui le déclare fini |
|---|---|---|
| [~] | **SAISIE-1 et 2 — ce qu'on écrit se voit** (fiche 99 § 2-3) : reproduit au banc qui clique et tape (`outils/pilote/`) — le titre d'une membrane ne montrait jamais la frappe ; il la montre, son curseur, sa sélection ; la membrane neuve de `Ctrl+G` cachait son titre sous la barre : la vue glisse. 10 sabotages sur 10 | à son écran : choisir deux nœuds, `Ctrl+G`, double-clic dans la membrane, taper |
| [~] | **COULEUR-3 — la couleur d'un texte se voit** (fiche 99 § 4) : elle teinte son papier à sa clarté perçue (Oklch) ; 7 sur 7 | à son écran : choisir un texte, le rose ; **sa parole sur le sens** (§ 5 : A domaine, B rôle, C sûreté) |
| [~] | **ROUE-1 — la roue de couleur** (fiche 99 § 8) : à la Blender, dans Oklch ; la case au bout de la barre ; `Ctrl`, `Maj`, la molette, `Retour arrière`, `Échap` ; **le code qu'on tape, la pipette** ; un geste | à son écran : une membrane, la case au disque, glisser, `Échap`, recommencer, cliquer dehors, `Ctrl+Z` ; un code ; la pipette sur une photo |
| [~] | **RETOUR-1 — `Alt+Z`** (fiche 99 § 6) : la vue revient au lieu d'où la caméra est partie, `Alt+Maj+Z` y repart ; « Vue précédente » au menu du vide ; 11 sur 11. **À peser** : un voyage à la main (molette) ne se remonte pas | à son écran : `F` sur un nœud, `Alt+Z` |
| [~] | **Les liens** (fiche 102) : **LIEN-1**, plus loin que la taille réelle un lien rétrécit avec le monde et pâlit d'autant, ce qu'on regarde reste plein ; **LIEN-2**, `Ctrl`+clic le suit jusqu'à son autre bout ; **LIEN-3**, la fenêtre des liens (`B`, le Bring & Go), un tableau de copies à côté du document, la provenance en membrane. Regardé au pilote sur la carte de Wikipédia et ses 1 128 liens. Bâti sur mes défauts (fiche 102 § 7) | à son écran : `glucose`, ouvrir le squelette (ou tout document à flèches), dézoomer, survoler un nœud, `Ctrl`+clic sur une flèche, `Alt+Z`, choisir un nœud, `B`, cliquer un amené ; **sa parole** sur la forme |
| [~] | **Le sens des couleurs des membranes** — teinte par proximité des domaines, clarté par accessibilité, opacité par appartenance. **Un premier prototype en image** (fiche 99 § 12, `teintes_sens.py`) : la teinte par sériation circulaire des ressemblances, la clarté par le rang de lisibilité ; l'opacité mesurée, pas peinte (pas de champ au modèle). **Version 2** (fiche 102 § 6) : la proximité par les liens que la carte porte (Milne et Witten) — 0,31 à 0,71 entre thèmes, contre 0,05 à 0,14 par les mots. Puis l'option « l'IA de Glucose colore », désactivable | **sa parole** sur l'image (`~/.cache/glucose-bancs/squelette-sens2/images/`) et sur l'opacité |
| [ ] | **B et C** (fiche 98 § 3) : une membrane dit ce qu'elle porte ; un fil « où je suis » | sa parole |

### Étape 2 — Tout ce qui manque (la parité avec Glucose Tauri, et ses demandes)

**Le domaine, `Ctrl+F` et les commandes du terminal sont faits, ils attendent son écran** (fiches 88 à 90). **Un article de Wikipédia posé par les commandes a montré ce qui manque** (fiche 91). **La disposition en colonnes est faite** (fiche 92). **Sa parole du 8/11** : *« travailler sur comment le cerveau humain serait capable de comprendre Wikipédia tout entier sur un canva 2D sans que ce soit trop brouillon, avec le maximum de facilité »* — la fiche 94 en tire quatre principes et l'ordre : DOM-6, l'héritage des domaines par les membranes (**fait**) ; **Sa parole du 8/11 au soir** (VISION) : *« pas de système de LOD et pas de simplification possible »* ; *« j'accepte de revoir le fonctionnement des membranes afin qu'elles soient plus intéressantes »*. **Le prochain : les titres des membranes à la taille de leur région ; puis un squelette de Wikipédia (les articles vitaux) ; puis les liens ; puis P6.

| | chantier | ce qui le déclare fini |
|---|---|---|
| [~] | **BARRE-1 — La barre blanche de Windows disparaît** (sa demande du 6/11, fiche 85) : la bande déplace la fenêtre et porte réduire, agrandir, fermer | à l'écran d'un Windows réel : glisser, coller aux bords, couleurs inchangées une fois agrandie |
| [~] | **Le domaine** (sa définition du 22/10, [VISION](VISION.md) ; fiche 88) : un domaine **traverse** les blocs, les membranes et les fichiers. **Fait** : chaque fichier garde ses domaines, Glucose met côte à côte ceux des documents qu'il connaît (sans y écrire) ; sa règle du 7/11, **deux domaines ne portent jamais le même nom** — on demande « même sujet » ou on précise le mot ; une IA les lit (`glucose_lire`, `glucose_domaines`) et les pose (`glucose_poser_domaine`) ; le panneau montre les autres fichiers, pose la question, et va à chaque place (« où »). **Essai réel hors écran** : trois blocs, deux fichiers, un domaine. **Reste** : les synonymes (« eau », « water ») ; sa visualisation (étape 5) | à son écran : créer « Eau » dans deux documents, « où », la question de deux « Python » ; et un agent qui s'en sert |
| [~] | **Les commandes Linux** (sa demande du 7/11 : *« des commandes Linux directement pour faire des trucs avec Glucose, plein plein de commandes, dont l'ouvrir »* ; fiche 90) : **fait** — `glucose document.glucose` et le double-clic ouvrent ; une commande par outil de la porte des agents, une seule table, la même porte et le même accord (`glucose aide`, `lire`, `chercher`, `domaines`, `poser-texte`…) ; `glucose_chercher`, l'agent cherche comme `Ctrl+F`. Essai réel, tout passe. **Reste** : la sortie dans une console Windows (un second petit programme, la CI pour juge) | à son écran : `glucose aide`, `glucose chercher …` dans son terminal, un double-clic sur un `.glucose` dans Nautilus |
| [~] | **La recherche, `Ctrl+F`** (fiche 89) : la palette, dont les rangées sont des nœuds de tout le document, sans majuscules ni accents ; Entrée y va (son onglet, le nœud choisi, la caméra qui vole), `F3` et `Maj+F3` au suivant et au précédent ; « Chercher… » au menu du vide. Sur le noyau ; l'index de trigrammes pour 10⁷ nœuds viendra sans changer ce qu'elle rend | à son écran, sur un gros tableau |
| [~] | **Un morceau de Wikipédia** (fiche 91) : « Eau » posé par les commandes ; MEMB-7 réparé, le plan pour un agent, COORD-1 ; un prototype de disposition en colonnes | la question de l'héritage des domaines, sa parole |
| [~] | **La disposition en colonnes** (fiche 92) : « Disposer en colonnes » au menu d'une membrane et à la palette, `glucose_disposer`, `glucose disposer` ; récursive, dans l'ordre de lecture, un seul `Ctrl+Z` | à son écran, sur l'article « Eau » ; les titres, sa parole |
| [~] | **DOM-6 — l'héritage des domaines par les membranes** (fiche 94 § 3 et 5) : un nœud appartient aux domaines de ses membranes, sauf son propre poids ; à la lecture, rien d'écrit ; le sommaire de « Eau » de 135 376 à 70 287 caractères ; 8 sabotages sur 8 | sa visualisation (étape 5) ; à son écran, un agent qui pose un article |
| [~] | **Les titres des membranes à la taille de leur région** (fiche 95, TITRE-1 ; **aucun ne se cache**) : la même part de surface qu'une membrane neuve, l'encre, la bande de clic, les colonnes, un nom qui tient dans sa région ; TITRE-2, le halo ; CHANTIER-2 réparé. **Reste** : le coût d'un zoom sur une très grande région (~19 ms contre ~9, fiche 95 § 8) — par la carte (Slug, sans le halo en neuf copies) ou en textures à paliers, à choisir à son écran | à son écran, sur « Eau » ; puis un squelette de Wikipédia |
| [~] | **Revoir les membranes pour qu'elles soient plus intéressantes** (sa parole du 8/11) : **propositions faites** (fiche 95 § 3 et 4) — B, une membrane dit ce qu'elle porte ; C, un fil « où je suis » ; D, une teinte par grande région ; E, plus tard (GMap, Bubble Sets). **Présentées le 9/11 en prose, D en image** (fiche 98 § 3 : Tree Colors par un agent, les neuf thèmes se reconnaissent, la roue s'épuise au deuxième niveau — trois suites possibles) | sa parole sur B, C, D, et sur la suite de D |
| [~] | **FOCUS-1 — le mode Focus par un geste choisi** (sa parole du 8/11 au soir ; fiche 96) : « Entrer dans la membrane » au menu d'une membrane et à la palette ; `Échap`, « Sortir de la membrane » ou dézoomer en sortent ; zoomer n'y fait plus entrer. Éprouvé, 9 sabotages sur 9 | à son écran : clic droit sur une membrane, « Entrer dans la membrane », Échap |
| [~] | **GRIS-1 et GRIS-2 — de loin, le gris de chaque ligne, et rien ne se tait** (sa parole du 8/11 au soir ; fiche 96) : nœuds de texte, post-it, ornements, et neuf dessins qui se taisaient (noms des zones et des dossiers, étiquettes, mots des opérateurs, sigles, cases, prédicats, lueurs) ; la loi dans la typographie (une lettre de moins de 2,8 px montre le gris de son encre) ; le SVG garde ses lettres. Éprouvé, 30 sabotages sur 30. **Reste** : les titres des membranes, et ce qui est plus fin qu'un pixel hors des nœuds — **GRIS-3, examiné le 9/11** (fiche 98 § 7) : l'épaississement est voulu au bord des zones et des cases (`max(1.0)`) et par la grille des pixels, et ces rectangles passent aussi par la carte graphique ; la loi exacte doit valoir sur les deux voies, et épargner les filets de l'interface | à son écran : dézoomer sur « Eau » |
| [x] | **L'épreuve des coupes** — elle avait raison : COLONNES-2, une colonne de trop née d'un arrondi (fiche 96) ; réparé, l'épreuve exige des coupes sur 120 cas ; 3 sabotages sur 3 | — |
| [~] | **Un squelette de Wikipédia** (fiches 94 § 4, 97) : les articles vitaux des niveaux 1 et 2 en membranes, « Eau » et « Hydrogène » dedans — **posé et regardé** (`outils/wikipedia/essai_squelette.sh`) ; MEMB-8 réparé en chemin. **Reste** : **les liens**, choisis par l'agent comme un cartographe — à lui demander, sur ce morceau | à son écran : le squelette ; sa parole sur le déséquilibre, B, C, D |
| [ ] | **P6 — Relire chaque panneau et chaque menu contre Tauri, champ par champ** : Pomodoro, Domaines, Time Machine, la bande, Ordonner, Mood board. **Les menus du clic droit : rien à comparer** — Tauri n'en avait pas (fiche 03, note 2.16) ; des quatre gestes que la spécification d'origine y annonçait, « Grouper dans une membrane » est fait (GROUPER-1, fiche 98 § 6 ter : `Ctrl+G`, le menu, la palette ; 7 sabotages), « Créer un sous-dossier » reste | la liste de ce qui manque, puis fait |
| [~] | **La couleur** (COULEUR-1, fiche 98 § 6) : la barre de couleur — huit teintes du thème et « Par défaut » pour les textes, post-it (leur papier), membranes et dossiers choisis ; un geste ; une commande par teinte (la palette). Un opérateur ne se colore pas. Regardée hors écran ; sabotée. **La roue est faite** (ROUE-1, fiche 99). **Reste** : l'encre d'un post-it | à son écran : choisir un post-it et une membrane, cliquer le rose, `Ctrl+Z` |
| [~] | **Les opérateurs des post-it** (OPERATEUR-1, fiche 98 § 4) : ET, OU, MAIS, PARCE QUE par `Alt+1…4`, `Alt+0` pour le papier, comme Tauri ; la palette ; le menu d'un post-it (le seul chemin au téléphone) ; un `Ctrl+Z` ; un opérateur a la taille de son mot. Essai réel hors écran ; 7 sabotages sur 7. **Une IA les lit et les pose** (OPERATEUR-2 : `glucose_lire` dit `"operateur"`, `glucose_poser_texte … operateur=parce-que` ; 6 sur 6) | à son écran : choisir deux post-it, `Alt+4`, clic droit, « Post-it ordinaire » |
| [ ] | **Les miroirs** (un même nœud à deux endroits, `Ctrl+Maj+M` ; le noyau en partie) | à son écran |
| [ ] | **La temporalité** : ancrer un nœud dans le temps, la réglette de −10 000 à 2 100, le filtre (`Maj+T`, `Maj+R` ; le noyau écrit) | à son écran |
| [ ] | **Les GIF animés et les vidéos** (un GIF se montre figé) | à son écran |
| [ ] | **L'export HTML** (pan, zoom, descriptions, un fichier) ; **le mode zen** (`F11`) | à son écran |
| [ ] | **Les choix à l'utilisateur** (sa parole : « TOUJOURS CHOISIR ») : la planche claire ou sombre, les champs sous les cases, « 2.3 », le numéro dans la bande ou le coin, l'export PNG à 100 % ou aux originaux, les photos du SVG en originaux ou réduites | chaque choix là où l'on agit, enregistré, saboté |
| [ ] | **Ordonner, « très très grandement »** (sa proposition du 15/10) : **à concevoir avec lui**. Ce qui manque déjà (PureRef) : par nom, par date d'ajout, en ligne, en colonne, aligner avec un espacement, distribuer, empiler, normaliser la largeur, la taille, l'aire, l'échelle ; aléatoire | sa parole, puis à son écran |
| [ ] | **Le dossier, à retravailler** (sa parole du 15/10) ; « double-clic + zoom comme Tauri », au PC et au téléphone | à son écran |
| [ ] | **La provenance des images** (SauceNAO) | **sa clé** |
| [ ] | **Ce qui attend sa parole** : les rideaux (un tableau dans une membrane — le noyau écrit), les membranes de Mary (étirées, minimisées — le noyau écrit) | sa parole |

### Étape 3 — Fluide partout (100 images par seconde)

| | chantier | ce qui le déclare fini |
|---|---|---|
| [~] | **Huit mille images : le coût par nœud visible** (fiche 87) : ATLAS-1, PHOTO-1, LOT-1, JETER-1 ; 8 000 images sous 10 ms en vue d'ensemble, sur cette machine, sur secteur. **Arrêté par sa décision du 7/11** : le reste (fiche 87 § 9) n'est repris que si un usage réel le demande | à son écran, sur 8 000 images, à l'école |
| [ ] | **P3 — Ce que son téléphone dira** : 16,4 ms par image ordinaire sur sa PowerVR (la télémétrie), 885 ms au pire ; POSTES-1 (les lignes `poste`), puis DESSUS-1 et RASTER-1 si elles les désignent ; et **P8 au téléphone**, jamais mesuré | les postes lus, le coupable nommé par la mesure, puis tenu |
| [ ] | **La fluidité de son PC** : ses zooms chargés à 11-24 ms (345 images sur 13 886 au-dessus de 10 ms) ; **sur batterie**, 30 images par seconde par moments ; les gels de l'Intel Arc ; la coupure du pavé de 7 à 10 s ; le plantage d'une épreuve dans `vulkan-1.dll` (fiche 49 § 4) | chaque point mesuré, puis tenu ou pixelisé — jamais une dégradation qui se voit |
| [ ] | **Le texte de près, la suite** (fiche 73 § 7) : les titres des membranes, des zones et des cases en formes ; la frontière « plus grand que l'écran » remise en question par la mesure | mesuré, puis aucun flou pendant un zoom |
| [ ] | **Le modèle de coût qui prédit** (la charte) — COUT-1 ne connaît encore que les pixels des photos | l'écart prédit-mesuré lu dans sa chronique |
| [ ] | **La mémoire des formules** (`MathRenderer`) ne s'oublie jamais — la même loi que celle des mises en page | éprouvé, saboté |

### Étape 4 — Toutes les machines

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **Android, ce qui manque** : le presse-papiers du système, les photos HEIC, les champs du dock qui se tapent, les panneaux qui s'écartent des bords ; les rayures de sa PowerVR ; la mise à jour automatique d'Android, prouvée par une version qui arrive d'elle-même | chacun à son écran |
| [ ] | **Les doigts sous Windows** (tablettes) ; le dépôt d'une page compressée hors Windows (`ureq` sans `gzip`) | à son écran, ou sur une copie |
| [ ] | **P9 — Le Mac publié** (un `.dmg`, son copier, sa mise à jour — sans compte Apple : non signé) | une version publiée qui s'installe |
| [ ] | **L'iPad par le web** (Glucose dans Safari — la personne qui compte pour lui) — **à sa parole** pour la partie 1 | **le modèle de l'iPad**, puis à son écran |

### Étape 5 — L'ergonomie (la 2.1.0)

Un preset est **l'interface entière** ([VISION](VISION.md)) : la disposition, le comportement, les
couleurs, les raccourcis, les fenêtres.

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **Le thème comme donnée**, puis **la disposition des panneaux comme donnée** (après P6) | rien n'a changé à l'œil ; un thème changé par la table |
| [ ] | **Le preset de Glucose** — *« extrêmement travaillé, qui n'a jamais été créé »* : toutes les fonctions, ergonomique, rapide pour les professionnels, propre à l'œil | sa parole |
| [ ] | **Le preset PureRef** ([PUREREF](PUREREF.md)) : la hiérarchie, le miroir horizontal et vertical, l'opacité, les niveaux de gris, le plus proche voisin, la pipette, le dessin à main levée, les formes, le magnétisme à 45°, réinitialiser une transformation, image suivante et précédente, un diaporama, les modes de fenêtre | sa liste PureRef, cochée |
| [ ] | **Choisir son preset**, importer et exporter ses réglages (un preset exporté *est* un fichier) ; les raccourcis de la souris et les raccourcis globaux | à son écran |
| [ ] | **La visualisation des domaines** : leur couleur sur le canevas, le filtre par domaine (fiche 03 § 11) — de l'ergonomie, sa parole du 22/10 | à son écran |
| [ ] | **L'interface du téléphone, toute refaite** : ultra rapide, fluide, pratique | à son écran |
| [ ] | **Le dessin de la sélection** (E pour l'instant) | sa parole |

---

## 1 bis. Après la partie 1

### En pause : Protéo (sa parole du 25/10)

Tout ce qui suit est écrit, éprouvé et saboté au point où il est, et attend. La porte des agents
(P10), elle, est dans Glucose et publiée (beta 6) : rien à défaire.

| | chantier | où il en est |
|---|---|---|
| [~] | **L'agent et l'humain, dans les deux sens** (fiches 79, 80) : la relecture à chaque message, le carnet « Ce que Protéo a appris », le « je ne sais pas ». **Reste** : les commentaires ancrés, la provenance par nœud, les membranes repliables, le statut et la preuve écrits par le programme | vu en essai réel hors écran |
| [~] | **La toile, rangée** (fiche 77 § 5) : RANGEE-1, SUITE-2 | à voir en essai réel |
| [~] | **Le tri de la demande** (fiche 76) : réponse courte, outils, ou workflow et sa taille | 23 justes sur 28 |
| [~] | **Le cerveau de Protéo** (fiches 77, 78) : **qwen3.5:4b démontré meilleur que qwen2.5:7b (15/20 contre 3/20, p = 0,0005)**. **Reste** : finir les essais réels, **lui proposer la bascule**, remplacer `proteo.exe` | la bascule reste sa parole |
| [~] | **La machine qui choisit seule** (fiches 77, 78) : écrite et sabotée ; **reste** à la brancher | — |
| [~] | **L'organe de recherche** (fiche 77 § 4) : PROGRES-1, SUJET-1, DEPEND-1 ; **reste** les pages dans une autre langue | — |
| [ ] | **Le flou** (sa parole du 17/10) : quand la recherche découvre qu'elle ne sait pas assez, Protéo se relance seul | — |

### La partie 2 — l'échelle, ensemble

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **GLISSER-2** (défaire un glisser exactement) ; **les glyphes des formules calés au pixel entier** | regardé |
| [ ] | **La fondation des 10⁷ nœuds** : l'arène, écrite, branchée ; l'index, la mémoire, le disque à cette échelle | un tableau de dix millions de nœuds, à 100 images par seconde |
| [ ] | **Le canevas de Wikipédia de `glucose-brain`**, et des jeux de données d'IA (sa parole du 22/10) : naviguer ultra facilement par les domaines et la couleur, jamais en appauvrissant le sens | à son écran |
| [ ] | **Les presets Figma et Miro** (défaut de la session du 25/10 : partie 2) | sa parole |
| [ ] | **Collaborer** (obligatoire) : le co-working sur le journal du document | à son écran, à deux |
| [ ] | **Les plugins** ([PLUGINS.md](../PLUGINS.md)), **l'IA locale**, **le MCP réécrit en Rust** | sa parole |
| [ ] | **D'humain à IA** : des graphes de questions et de réponses au lieu d'un fil ; le plugin de la langue inventée | sa parole |
| [ ] | **Le document de recherche** en mathématiques et en physique, non linéaire | sa parole |
| [ ] | **Trans-domaines** (de la R&D : une vue en perspective 3D de tous les domaines) | sa parole |
| [ ] | **Glucose en briques** : le noyau, le rendu, la typographie, les formules, le format, chacun lisible seul ; **les dépendances vers zéro**, sans jamais exclure une machine | une brique prise par un autre projet |

### La recherche, à côté

* **L'améliorateur d'image à l'import** (sa parole du 15/10) : comprendre une image avant d'y
  toucher (filtres de Gabor, GLCM, LBP — `recherche/cerveau-image/`), pour débruiter, déflouter,
  agrandir sans inventer.

---

## 2. Ce qui attend son écran — une seule liste

Tout est écrit, éprouvé et saboté ; **rien de ceci n'a été jugé par lui**. Sa parole du 15/10 :
*« si j'ai absolument tout testé, normalement 100 % des fonctionnalités sont validées »*.

**Au PC** — `nix develop -c cargo run --release > sortie-essais.txt 2>&1` (ce qui parle de Windows ci-dessous attend une machine Windows) :

* **Les titres des membranes** (fiche 95) : ouvrir un document où une membrane est grande (l'article « Eau » des commandes, ou une membrane tirée très grande avec un titre) ; dézoomer jusqu'au bout : **chaque titre reste**, à la taille de sa région ; zoomer et dézoomer : dire si c'est fluide ; un clic sur un grand titre prend la membrane.
* **Disposer en colonnes** (fiche 92) : choisir une membrane qui porte plusieurs textes (ou un texte dedans), clic droit, « Disposer en colonnes » ; `Ctrl+Z` la remet.
* **Ses commandes** (fiche 90), dans son terminal : `glucose aide` ; `glucose chercher eau` ; la première
  commande demande l'accord dans Glucose, au nom du « terminal » ; `glucose poser-texte "Bonjour"` apparaît
  sous ses yeux, `Ctrl+Z` le défait ; un double-clic sur un `.glucose` dans Nautilus l'ouvre.
* **`Ctrl+F`** (fiche 89), sur un gros tableau : écrire, Entrée y va (même dans un autre onglet, même sur
  une flèche), `F3` au suivant.

* **Le domaine qui traverse les fichiers** (fiche 88) : dans un document, créer un domaine « Eau » au panneau
  Domaines, le poser sur deux nœuds, enregistrer ; un document neuf : « Eau » est dans la liste, en gris,
  « 2 · 1 fichier » ; le poser sur un nœud d'ici ; « où » : les trois places, un clic qui ouvre l'autre fichier
  et y vole. Créer « eau » une seconde fois : refusé, la question. Et un agent (Claude, Protéo) qui
  s'en sert : « range ce tableau par domaines ».
* **Éteindre le PC avec Glucose ouvert** (fiche 82), puis rouvrir Glucose : le travail est là, et
  rien ne doit avoir demandé quoi que ce soit à l'extinction.
* **Le texte de près** (fiches 72 et 73) : zoomer très près d'un texte, d'une formule, d'un
  passage qu'une flèche fait briller, d'une sélection en cours d'écriture, d'un emoji, de
  l'italique — **nets dès la première image**, jusqu'à ×256 ; zoomer et dézoomer sans trou ni
  à-coup ; un titre de membrane (encore peint au processeur) ; **un très long texte** (des
  milliers de mots collés dans un seul nœud), vu de près, qu'on parcourt : fluide.
* **Changer un raccourci** (fiche 74) : `Ctrl+Maj+P`, écrire « raccourci », « Changer un
  raccourci… », choisir « Dupliquer », frapper `Ctrl+B` ; `Ctrl+B` duplique ; fermer et rouvrir
  Glucose : il duplique encore ; puis « Rétablir tous les raccourcis de Glucose ».
* **La palette** : `Ctrl+Maj+P`, écrire « dupl », Entrée ; les flèches pour choisir ; Échap ;
  sans rien écrire, faire défiler toute la liste (molette, pavé, flèches, Page bas).
* **Un agent dans Glucose** (fiches 75, 80) : « Autoriser » ; une demande, une recherche — ses
  pages (le double-clic les ouvre), les flèches qui arrivent à côté de chaque citation ;
  sélectionner un nœud et lui dire « ça ».
* **Le menu du clic droit** : ses raccourcis viennent du clavier — rien ne doit avoir changé à
  l'œil.
* **Le Storyboard** (fiches 64 à 68) : « Activer », la grille qui suit la souris par son coin,
  un clic ou un rectangle tracé ; la poignée qui le retrace ; une case lâchée ailleurs ; décrire
  (deux clics sur la bande), `Tab` vers la suivante ; plusieurs images lâchées sur une case ;
  « Storyboard de la sélection » ; les deux « + » ; « ▶ Lire » (← →, Espace, Échap) ; « CASE n » :
  durée, plan, champs ; « + Nouveau », ‹ 2 / 3 › ; « Exporter en planches » en PDF et en PNG.
* **L'export SVG** : l'ouvrir dans son navigateur, puis dans Inkscape s'il l'a.
* **Le dossier** : son titre et son compteur ; un dossier vide dit « Double-clic pour ouvrir ».
* **`Alt+F`** cadre tout, même un tableau qui ne porte qu'un storyboard.
* **Mood board** : poser un gabarit, ses zones (bandeau, coins, croix), « Par mood board » dans
  Ordonner.
* **Ordonner** : les trois tris par la couleur ; les champs (glisser, cliquer et taper).
* **L'export PNG** : une sélection, puis tout le tableau.
* **Le clic droit** : Annuler, Rétablir, Tout cadrer, Signets…, Enregistrer, Exporter ; le menu
  en colonnes sur un écran bas.
* **La membrane et le dossier tracés** d'un glisser ; « Décrire » écrit au clavier.
* **La chrome** : rien ne doit avoir changé à l'œil, sauf la minimap (un trait fin, un voile).
* De plus loin : transformer une sélection entière (origine commune ou individuelle) ; le cadre
  du groupe qui suit ; Pinterest (la copie d'abord, puis l'original) ; les sélecteurs de fichiers
  qui ne figent plus Glucose ; les flèches qui contournent et leur coude ; les formules seules
  sur leur ligne ; la Time Machine pendant un zoom ; `Ctrl+Z` après un glisser ; l'aimant sur un
  tableau chargé ; un PDF glissé d'une page web.

**Au téléphone** — **ouvrir Glucose** : il proposera la beta 6 (sinon, l'APK zippé sur son
Bureau) :

* l'installer **par-dessus** (ne pas désinstaller : un brouillon perdu est peut-être encore là),
  l'ouvrir une fois, puis une **longue session** ; si Glucose se ferme seul, le rouvrir deux
  fois — ses lignes partent au lancement suivant ;
* le texte de près (P8 sur sa PowerVR, jamais mesuré) ;
* le clavier (écrire dans un texte), l'appui long (le clic droit), deux doigts touchés annulent,
  trois rétablissent ; renommer, dupliquer, supprimer un document (appui long dans la liste) ;
* le Storyboard au doigt ; un dossier choisi (son menu, sa barre) ; le partage vers Glucose
  depuis la galerie ou Pinterest ; les emojis ; les noms du rail ; le bord à bord.

**Sous Linux, désormais sa machine de tous les jours** : à essayer ici d'abord, par nous (`outils/essai_isole.sh`) — le
copier vers Discord, un bloc d'un Glucose à l'autre, après fermeture (fiches 60, 61), les gros
tableaux sur une Intel UHD 620 ; et la liste de « tant de choses ne marchent pas ».

## 3. Ce qui n'attend que lui

* **La beta 8 est publiée par lui le 6/11** (barre blanche, teintes, sélection) : **ses essais à l'école** : le texte, la fenêtre glissée et collée au bord, les couleurs une fois agrandie.
* **La beta 9, publiée par lui le 7/11** (CHARGE-1, la fiche 87, JOURNAL-1). Le domaine (fiche 88) ira dans la suivante.
* **La clé de mise à jour** : récupérée le 7/11 et rangée sur ses deux disques (fiche 87 § 7 sexies). **Une copie hors de cet ordinateur, sur une clé USB à lui, reste à faire.**
* **`sauvegarde-profil-2026-10-31.7z`** (317 Mo : ses clés de serveur, son identité web3, ses brouillons) : où l'a-t-il mise ? Introuvable sur ce PC (fiche 84 § 6).
* **Ouvrir Glucose sur son téléphone** (encore en beta 3, muet depuis le 15/10) : la seule façon
  de savoir pourquoi le tableau de 6 000 images est mort (P0).
* **Ses trois réponses de la partie 1** (§ 0) : l'iPad, les presets Figma et Miro, « avec des
  utilisateurs ». **Sa question du 7/11** : sa copine voudrait Glucose sur son iPad — aujourd'hui
  impossible (aucune version pour iPad ; sans compte Apple payant, seule la voie de Safari reste) ; le
  modèle de l'iPad, et s'il veut avancer l'iPad ; vérifier si l'interface de Glucose Tauri s'ouvre
  dans Safari telle quelle, un raccourci possible.
* **Sa clé SauceNAO** (la provenance des images).
* **Le modèle de l'iPad**, les téléphones de ses testeurs Android (le plus ancien d'abord).
* **Ordonner** : ce qu'il entend par « très très grandement ».
* **Les définitions** : ~~« domaine »~~ (**défini le 22/10**, [VISION](VISION.md) — ne plus le lui
  demander) ; Trans-domaines (R&D), les rideaux, les membranes de Mary (étirées, minimisées),
  « optimiser » dans la Time Machine.
* **Sa liste PureRef** (ce qu'il veut retrouver de PureRef), pour le preset.
* **Le dessin de la sélection** : E pour l'instant, à reprendre avec l'ergonomie.
* **Le défilement au bord** (« moche, pas fluide ») : plus tard, à sa parole.
* **Les modes de la fenêtre** (opacité, clics qui traversent) : pas pour l'instant.
* **La bascule du cerveau de Protéo** (qwen3.5:4b) : quand Protéo reprendra.
* **Le nom des versions** : la 2.1.0 comme première stable, à sa parole.

## 4. Fait (mémoire courte)

* **P5 — L'export SVG complet** (fiches 69 à 71) — reste ses yeux (§ 2).
* **La 2.0.3-beta.3** et **la 2.0.3-beta.4** — publiées par lui le 15/10 ; son essai de la beta 4 :
  plus de 24 000 nœuds au téléphone, « ça n'a pas trop lagé, plutôt cool ».
* **La 2.0.3-beta.5** (16/10) et **la 2.0.3-beta.6** (23/10).
