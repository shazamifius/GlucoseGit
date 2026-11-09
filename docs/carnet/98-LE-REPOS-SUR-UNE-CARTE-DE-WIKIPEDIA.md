# 98 — Glucose se pose sur une carte de Wikipédia ; un agent ne change que ce qu'il dit

> Session du 9/11/2026, à partir de 10 h 26, en autonomie : il était épuisé, et a demandé qu'on
> avance sans question ni essai de sa part (*« si les défis nécessitent un humain, partir sur autre
> chose »*). Sa parole du 9/11 au matin (VISION) : REPOS-2 est le prochain chantier (*« yep »*) ; B,
> C, D à lui présenter. Ce qui demande sa parole est donc **écrit ici**, en prose, avec des images
> à regarder, et rien n'en a été décidé à sa place.

## 1. REPOS-2 — ce que la mesure a corrigé dans la fiche 97

La fiche 97 disait : *« Glucose ne se pose jamais sur ce tableau »*, 99 % des images « parce que la
netteté revient », et soupçonnait un cercle entre la résolution réduite et la finesse rendue.
**La mesure dit autre chose.** Le banc hors écran (le vrai Glucose, la vraie carte graphique, le
squelette, « tout cadrer » puis rien) sur **120 s** au lieu de 30 : **les 293 images tombent toutes
dans les 30 premières secondes**, puis plus rien. Ce n'était pas un cercle : c'était une
convergence ruineuse.

Une ligne par image (`GLUCOSE_PERF=1`) l'a nommée sans hypothèse : chaque image du repos faisait
**une** texture (`textures_faites=1`, 10 µs, 300 pixels) et en reportait 340 ; la peinture des
membranes (17 ms) avait mangé tout le budget avant elle. Et ce qui redemandait ces images
**complètes**, c'était les teintes : tant qu'une seule restait en retard, CHANTIER-2 (fiche 95)
salissait toute la scène — or la tranche des teintes courait **depuis le début de l'image**, et les
membranes la mangeaient aussi : **une** teinte par image. Trois cent cinquante nœuds, trois cent
cinquante images de 22 ms, trente secondes d'un cœur occupé après chaque « tout cadrer » (et après
chaque zoom qui montre des nœuds neufs : sa séance du 9/11 à 1 h 05 en porte la trace, 115 images
« netteté » et 94 « teintes » sans la main). Une seule cause pour les deux raisons.

**La réparation** (`renderer/hue.rs`, `app/reveil.rs`) suit ce que font les compositeurs et les
éditeurs qui dorment bien — l'état de repos de [niri](https://niri-wm.github.io/niri/Development%3A-Redraw-Loop.html)
(on ne redessine que ce qui est abîmé), les passes sans dommage que
[Chromium](https://grida.co/docs/wg/research/chromium/damage-tracking) saute, la réparation
« quand le système est inactif » de [Pad++](https://www.cs.umd.edu/projects/hcil/pad++/documentation/doc-0.2.7/guide/guide-0-23.html) :
**on ne redessine pas pour travailler.** Le cache des teintes **retient ce qu'il doit** — le rang de
chaque nœud qu'une image a montré d'une teinte d'avant (un rang de l'index : noter une dette
n'alloue rien) ; quand la vue ne bouge pas, le réveil **paie** ces dettes hors de toute image, par
tranches, et ne demande **qu'une** image, celle qui montre le tout. Pendant un mouvement rien ne se
paie entre les images (elles viennent de toute façon) ; un document changé depuis l'image ne se paie
pas (les rangs désignent l'ancien index : une image refait le compte). Les textures, elles, n'ont
rien demandé : une image sans repeinture leur laisse tout le budget, et **les 352 se posent en une
image de 16 ms**.

**Mesuré**, en paires entrelacées avec le programme d'avant (`f967f66`), 30 s de repos sur le
squelette :

| | images | un cœur, au repos |
|---|---|---|
| avant | 295, 293 | 21,8 %, 21,5 % |
| après | 27, 26 | 2,4 %, 2,3 % |

Il reste 18 images : le fondu du message d'accueil, qui **redessine toute la carte** à chaque pas
(17 ms de membranes pour un message qui change). Défaut ancien, hors de ce chantier, sans gêne
d'usage : noté (§ 7).

**Prouvé** : cinq épreuves (le cache paie hors de l'image au bit près du calcul entier ; chaque image
refait le compte de ses dettes ; au repos, aucune image avant la dernière, une seule, due au
chantier, qui ne calcule plus rien, puis le sommeil ; un document changé ne se paie pas ; la vue qui
bouge ne paie pas). **Sabotages** (`outils/sabotages/repos-2-98.json`) : **7 sur 7**.

## 2. AGENT-8 — une mise à jour d'agent ne change que ce que l'appel nomme

Trouvé en préparant l'image de la proposition D (§ 4) : un agent qui **recolore** une membrane par
son identifiant **effaçait son titre** — et son mode, ses rideaux, son ancrage dans le temps. La pose
bâtissait un nœud neuf et le mettait à la place de l'ancien ; seuls l'appartenance et les domaines
étaient gardés, chacun par une ligne à lui. Réécrire un texte effaçait de même la couleur, la taille
de lettre ou l'opérateur que l'humain y avait mis : **un agent pouvait défaire en silence le travail
de l'humain**, contre le pont que VISION demande.

La règle devient générale et dans le bon sens (`agents/executer/partir.rs`) : une mise à jour **part
du nœud tel qu'il est**, et l'outil n'y écrit que ce que l'appel nomme. Un champ qu'on ajoutera au
modèle demain sera gardé sans que personne y pense — l'inverse d'une liste de champs à recopier.
Épreuves : une membrane recolorée garde son titre et son mode, renommée garde sa couleur ; un post-it
réécrit garde sa couleur, son opérateur, sa taille de lettre. **Sabotages**
(`outils/sabotages/agent-8-98.json`) : **3 sur 3**.

## 3. B, C, D — présentées en prose, pour qu'il réponde quand il voudra

Sa parole du 9/11 : *« prochaine session »*. Il était trop fatigué pour qu'on en discute : la
présentation est ici, et dans le message de fin de session ; **rien n'est décidé à sa place.**

**D — une teinte par grande région, ses régions en nuances.** C'est la seule des trois qu'une image
peut montrer sans rien changer à Glucose : un agent colore l'arbre des membranes par les seules
commandes, comme un cartographe (`outils/wikipedia/teintes_arbre.py`, § 4). La méthode est **Tree
Colors** (Tennekes et de Jonge, [IEEE TVCG 2014](https://vis.cs.ucdavis.edu/vis2014papers/TVCG/papers/2072_20tvcg12-tennekes-2346277.pdf),
relue ce jour dans l'article même) : la racine reçoit toute la roue des teintes ; chaque région
partage la sienne entre ses enfants, les permute pour qu'aucun ordre ne se lise (deux sœurs voisines
à 144° l'une de l'autre), retourne une part sur deux (deux feuilles voisines de branches différentes
ne se ressemblent pas), et n'en garde que le milieu (75 %), l'écart qui sépare les branches. La
clarté monte avec la profondeur, la saturation descend — la méthode « additive » de l'article, celle
d'un fond sombre. Déterministe, sans palette choisie à la main.

**Ce que l'image montre** (`~/.cache/glucose-bancs/squelette-d/images/`, à côté de celles d'avant
dans `squelette-wikipedia/images/`) : **au premier niveau, ça marche** — les neuf thèmes se
reconnaissent d'un regard (Histoire rouge, Géographie verte, Arts bleu, Vie quotidienne brune,
Philosophie sarcelle, Société violette, Sciences jaune, Technologie cyan, Mathématiques magenta), là
où tout était du même bleu. **Au-delà, les nuances ne se voient pas** : chaque thème n'a que 30° de
teinte à partager, et Chimie, Biologie, Astronomie diffèrent de 3 ou 4° — la roue est épuisée dès le
deuxième niveau. C'est la limite de la méthode sur un arbre profond (l'article l'illustre sur trois ou
quatre niveaux ; [une suite de 2024](https://arxiv.org/html/2407.08287v3) propose de la corriger).
Et une seconde chose se voit : **les textes gardent leur lueur « symbiotique »** (vert, rose, bleu,
selon leur voisinage) — un bruit de couleurs qui ne dit rien de la région, sous la teinte qui la dit.

Trois suites possibles, à lui de choisir : *(a)* la couleur dit le thème, et rien de plus — ce que
l'image montre déjà ; *(b)* au-delà du premier niveau, les sœurs se distinguent par la **clarté**
(une claire, une sombre), la teinte restant celle du thème ; *(c)* chaque article de Wikipédia
reçoit la roue entière pour ses sections — ses sections se distinguent, mais on ne lit plus le thème
dans un article. Mon avis : *(b)*, et que la lueur des textes prenne la teinte de leur région. Le
jour où il dit oui, D devient un geste de Glucose (« Colorer l'arbre », au menu d'une membrane), pas
seulement un script d'agent.

**C — un fil « où je suis ».** En haut du canevas, la suite des membranes qui contiennent le centre
de l'écran (*Wikipédia › Sciences et médecine › Chimie › Eau › Propriétés › Polarité*), chaque nom
cliquable pour y voler, et la même phrase pour une IA. Pourquoi : le « brouillard du désert » (Jul et
Furnas, 1998) — zoomé dans une région, on ne sait plus où l'on est ; des lecteurs de Wikipédia se
servaient de l'arbre de la hiérarchie plutôt que de la carte (Wu et Vakkari, 2014) ; Muse quitte ses
tableaux emboîtés par un fil d'Ariane. Glucose a déjà un fil pour les dossiers ; celui-ci serait pour
les membranes. Rien ne se cache : le fil **ajoute** une ligne.

**B — une membrane dit ce qu'elle porte.** Sous son titre, « 12 sections · 198 paragraphes »,
calculé, jamais écrit à la main ; l'IA le lit déjà dans le plan. Pourquoi : avec la loi qu'il a gardée
(la surface dit la quantité), un petit pays vu de loin ne dit pas s'il est vide ou dense ; le nombre
le dit sans rien simplifier.

**Mon ordre, s'il le veut** : D *(b)* d'abord (la couleur sert à toutes les distances), puis C (le
fil, à toutes les profondeurs), puis B.

## 4. OPERATEUR-1 — les opérateurs des post-it, enfin un geste

ET, OU, MAIS, PARCE QUE : la grammaire des post-it de Glucose Tauri. Le modèle les portait, le
rendu les dessinait en pilules depuis la fiche 06 — **aucun geste ne les posait** (INVENTAIRE § 3).
Lu dans Tauri (`App.tsx`, branche `tauri-v1.0.1`) : `Alt+1` à `Alt+4` posent l'opérateur sur
chaque post-it choisi, `Alt+0` le retire, rien d'autre.

* **Une commande** (P4) : `operateur.et`, `.ou`, `.mais`, `.parce-que`, `.aucun` — au clavier
  (`Alt+0` à `Alt+4`, dans la couche de Glucose), à la palette (« Post-it : opérateur ET »…), et **au
  menu d'un post-it** : au téléphone, sans `Alt`, c'est le seul chemin. Le menu ne montre que ce qui
  change quelque chose (l'opérateur que tous portent déjà disparaît ; « Post-it ordinaire » n'existe
  que si l'un en porte un).
* **Un geste** : un seul `Ctrl+Z` pour toute la sélection ; ce qui n'est pas un post-it est ignoré
  (une sélection mêlée sert à ce qu'elle peut servir, comme les relations des flèches) ; sans
  post-it, la commande ne s'applique pas — ni geste, ni document modifié.
* **Un opérateur a la taille de son mot** : sa boîte retombe à la naissance d'une pilule (80 de
  large, 130 pour PARCE QUE, 44 de haut) ; redevenu papier, à celle d'un papier neuf — la taille
  d'avant reste à `Ctrl+Z`. Le texte du post-it n'est jamais touché : il reparaît avec le papier.
  (Tauri gardait la taille du papier sous la pilule, que son CSS ignorait ; ici la boîte **est** ce
  qu'on voit — la sélection, le culling et l'export la lisent.)
* **Sans message** : la pilule paraît sous les yeux de celui qui appuie — la règle des relations
  (PRED-1). Tauri en affichait un.

**Prouvé** : le noyau (deux post-it et deux flèches : les post-it seuls changent, à la taille de leur
mot, le texte intact ; le même opérateur ne change rien ; un `Ctrl+Z` rend les papiers et leur
taille) ; l'application au clavier (`Alt+1` à `Alt+4` sur toute la sélection, le texte choisi avec
eux reste un texte, `Alt+0`, un seul `Ctrl+Z`, sans post-it rien ne s'écrit et la commande le dit) ;
le menu (rien sur un texte, les quatre sur un papier, l'opérateur porté disparaît, « Post-it
ordinaire » revient). **Sabotages** (`outils/sabotages/operateur-1-98.json`) : **7 sur 7** — un
premier jet en avait deux aveugles, refaits avant d'être joués (une garde dont le seul effet
observable est ce que la commande rend : l'épreuve l'exige désormais). **Essai réel** : un document
de quatre post-it et un texte posé par les commandes dans le vrai Glucose sans fenêtre, puis, dans le
vrai programme hors écran, un scénario « tout choisir », `operateur.parce-que`, `operateur.et` : la
capture montre quatre pilules ET vertes, le texte ordinaire intact
(`~/.cache/glucose-bancs/operateurs/banc/capture.png`).

## 5. OPERATEUR-2 — une IA lit et pose les opérateurs, comme l'humain

Un trou de principe, trouvé en relisant : un humain **voyait** la pilule « PARCE QUE », une IA qui
lisait le tableau **ne la voyait pas** — la description d'un nœud ne disait pas l'opérateur (et
montrait le texte du post-it, que la pilule cache). VISION l'exige « impérativement » : lu de la même
façon par les deux.

* **Lire** : `glucose_lire` dit `"operateur":"parce-que"` pour un post-it devenu pilule.
* **Poser** : `glucose_poser_texte … operateur=et|ou|mais|parce-que|aucun` — un opérateur fait de
  l'appel un post-it ; il passe par la loi du noyau (la pilule à la taille de son mot), dans le même
  geste ; la boîte rendue est celle de la pilule ; un autre mot est refusé, en disant les mots
  permis ; une mise à jour qui n'en parle pas le garde (AGENT-8). Au terminal : `glucose poser-texte
  texte="car il pleut" operateur=parce-que`.
* **Un seul nom partout** : `StickyOperator::id` (`et`, `ou`, `mais`, `parce-que`), que la table des
  commandes, la palette, les agents et le terminal lisent. Les opérateurs ont quitté
  `annotation.rs` pour `types/operateur.rs` : le cliquet des 600 lignes l'a demandé (601), et
  l'extraction s'imposait, jamais le plafond.

**Sabotages** (`outils/sabotages/operateur-2-98.json`) : **6 sur 6**.

## 6. COULEUR-1 — la barre de couleur

SUITE, étape 2 : *« la couleur : un sélecteur ; la couleur d'un post-it »* ; INVENTAIRE : *« menu,
barre d'action »*. Lu dans Tauri : un post-it ne se colorait **que pendant qu'on l'écrivait**, par un
bouton posé au-dessus de lui qui ouvrait une roue HSV (`ColorPicker.tsx`) ; une membrane naissait
d'une teinte **tirée au hasard** parmi six, et rien ne la changeait ensuite ; un texte n'avait rien.

**Ce qui est fait** : quand la sélection porte un texte, un post-it ou une membrane, une barre se pose
au-dessus de la barre d'action (au-dessus de celle des flèches s'il y en a une) : « Couleur », les
huit teintes du thème (celles des domaines, choisies pour rester distinctes sur le fond sombre ; les
six premières sont celles que Tauri tirait pour une membrane), et « Par défaut ». Un clic colore tout
ce qui est choisi, en un geste ; la teinte que tous portent s'allume. **Chaque teinte est une
commande** (`couleur.rose`, `couleur.par-defaut`…) : la palette les porte, un raccourci peut les
nommer. Ce que « sa couleur » veut dire, seul le noyau le sait (`Store::colorer`) : le papier d'un
post-it (comme Tauri), la teinte d'une membrane, la teinte d'un texte (qui l'emporte sur sa teinte
symbiotique).

**Ce que l'image a corrigé** : la première capture, quatre pilules MAIS choisies, montrait une barre
de couleur dont les teintes ne changeaient rien de visible — une pilule a la couleur de son mot. Un
bouton qui ne fait rien de visible ment : **un opérateur ne se colore pas** (`Annotation::se_colore`),
comme une flèche, dont la couleur dit la relation.

**Le dossier aussi** : Tauri le colorait par sa roue ; la barre le fait par le nuancier, et « Par
défaut » lui rend la couleur de sa naissance (`COULEUR_DE_NAISSANCE`, nommée au lieu d'un `"#888888"`
écrit dans le constructeur). Au passage, **la lecture des couleurs quitte l'interface pour le noyau**
(`Store::couleurs_choisies`, `Annotation::couleur_propre`), à côté de l'écriture
(`Store::colorer_la_selection`) : ce que « sa couleur » veut dire n'est écrit qu'à un endroit.

**Regardé**, dans le vrai programme hors écran (`~/.cache/glucose-bancs/operateurs/banc-couleur*/`) :
quatre post-it devenus roses, la case rose allumée ; puis MAIS, puis « Par défaut » allumé. **Les
images témoins** : seules les deux scènes choisies changent — 15 450 pixels, tous dans la bande de la
barre (701-1098 × 758-796), regardée avant/après, agrandie ; les six autres témoins, au pixel près.

**Prouvé** : le noyau (chaque sorte, l'opérateur ignoré, un geste, « Par défaut », le dossier) ; la
barre (rien pour une flèche, une photo, un opérateur ; un dossier en a une ; la teinte commune
s'allume ; au-dessus des barres du bas, dans l'écran, sans chevauchement) ; la souris (un clic colore,
un clic entre deux cases ne désélectionne rien, un `Ctrl+Z`) ; la commande. **Sabotages**
(`outils/sabotages/couleur-1-98.json`) : **9 sur 9** à la première version, rejoués sur la version
finale (§ 9).

**Reste** : la roue de Tauri, pour une couleur hors du nuancier (à la souris et au doigt) ; l'encre
d'un post-it ; le jour où il le dira, D en geste (« Colorer l'arbre »).

### COULEUR-2 — une IA lit et pose les couleurs avec le sens de l'humain

Le même trou qu'OPERATEUR-2 : la couleur qu'un humain donne à une région, **une IA ne la lisait
pas** — or D (les régions reconnues par leur couleur) ne vaut que si l'IA les reconnaît aussi. Et la
couleur qu'un agent donnait à un post-it peignait **son encre**, quand la barre de l'humain peint
**son papier**. `glucose_lire` dit désormais `"couleur"` (par `Annotation::couleur_propre`), et l'agent
passe par la loi du noyau (`Store::colorer`). Épreuves : un agent colore le papier d'un post-it et
le relit ; l'IA lit la teinte qu'un humain a posée sur une membrane. AGENT-8 en a changé une ligne
(un post-it réécrit garde **son papier**).

## 6 bis. REPOS-2 pendant le mouvement : rien d'abîmé

Le scénario de vols du dépôt (`outils/scenarios/vols-vue-d-ensemble.txt`) sur le squelette, en paires
entrelacées avec le programme d'avant : l'image de zoom médiane 9,7 et 13,8 ms après, 11,6 et 11,6
avant ; le p90 16 et 19 contre 23 et 23 ; le pire 47 et 72 contre 48 et 55 (une seule image sur
cinquante : du bruit à cette taille, pas un verdict). Et sans la main, sur ce scénario : **282 → 76
images**, un cœur 19,5 % → 4,3 %.

## 6 ter. GROUPER-1 — grouper la sélection dans une membrane neuve

Trouvé en relisant P6 (« les menus du clic droit contre Tauri ») : **Tauri n'avait pas de menu du clic
droit** (fiche 03, note 2.16 — son clic droit déplaçait la vue), et la spécification d'origine y
annonçait quatre gestes ; trois existaient, pas « Grouper dans une nouvelle membrane », que le menu
taisait faute de geste. C'est pourtant **le geste qui fait une région** — choisir des nœuds, les
entourer, nommer —, celui de la carte de Wikipédia comme de Miro ou Figma (`Ctrl+G`).

* **La loi** (`core/store/grouper.rs`) : les **racines** de la sélection entrent dans une membrane
  neuve — ce qu'une membrane choisie contient la suit sans changer d'appartenance (on emballe un pays
  avec ses villes) ; le cadre est celui de MEMB-7 (un écart autour, la place d'un titre au-dessus),
  **une seule loi** désormais (`Store::cadre_autour`), que l'agrandissement des agents partage ; le
  groupe naît dans **l'ancêtre commun** des racines, qui grandit s'il le faut (ses parentes avec lui) ;
  sans ancêtre commun, à la racine du tableau. Un geste ; la membrane est choisie (un double-clic la
  nomme, la barre de couleur la teinte).
* **Le geste** : la commande `grouper` — `Ctrl+G`, le menu d'une sélection (« Grouper dans une
  membrane », absent pour des flèches seules, qui n'ont pas de boîte), la palette.
* **Regardé** : dans le vrai programme hors écran, « tout choisir » puis `grouper` entoure les six
  nœuds d'une membrane choisie (`~/.cache/glucose-bancs/operateurs/banc-grouper/capture.png`) ; le
  menu témoin gagne sa ligne — 29 234 pixels, tous dans le menu, regardés avant/après.
* **Une épreuve aveugle évitée** : celle du cadre comparait la membrane à `cadre_autour` lui-même — un
  sabotage de la loi l'aurait laissée passer. Elle vérifie désormais la loi sans elle (la place du
  titre, l'écart de chaque côté).

## 7. Ce qui n'a pas été fait, et pourquoi

* **REPOS-3, le fondu d'un message redessine toute la carte** : sur le squelette, 17 images de
  22 ms pour un message qui paraît puis s'efface. Ancien, sans gêne d'usage mesurée ; le réparer
  demande une salissure « l'interface seule », qui touche la composition des couches — à peser avec
  la mesure d'un usage qui en souffre (sa parole du 7/11 : pas d'église de l'optimisation).
* **GRIS-3, les traits plus fins qu'un pixel hors des nœuds** (SUITE, le reste de GRIS) : examiné,
  pas fait. L'épaississement est **voulu à deux endroits** — le bord d'une zone et d'une case de
  storyboard (`scale.world(1.0).max(1.0)`, `zone.rs`, `storyboard.rs`), et la grille des pixels sur
  les rectangles des zones (`on_pixel_grid`, qui ramène tout côté à un pixel) — et ces rectangles
  passent **aussi par la carte graphique** : les deux voies doivent rester identiques au bit près.
  Les poignées et les anneaux, eux, ont une taille d'écran (jamais sous un pixel) : hors de cause.
  Le chantier juste : une loi « un rectangle du monde plus fin qu'un pixel se peint à sa couverture
  exacte » dans `fill_crisp` **et** dans la voie de la carte, l'épreuve de l'encre de loin (comme
  GRIS-1), et la mise à jour regardée du témoin dézoomé. `fill_crisp` sert aussi l'interface (13
  appels dans la bande) : la loi doit y garder ses filets nets — distinguer le monde de l'écran.
* **La roue de couleur de Tauri** : un chantier entier (la roue, le curseur de clarté, le champ
  hexadécimal, au doigt comme à la souris) ; le nuancier couvre déjà la couleur d'un post-it,
  d'une membrane, d'un texte et d'un dossier.
* **Le budget d'une image déjà en retard** : la cascade des textures et la tranche des teintes se
  mesurent contre le plancher de 10 ms ; une image qui le dépasse déjà (17 ms de membranes) aurait
  pu faire tout son travail jusqu'au balayage suivant (33 ms) sans que l'œil le voie. Une idée de
  modèle de coût (la charte : prédire, pas réagir), notée, pas un chantier : REPOS-2 a supprimé le
  cas qui la rendait coûteuse.

## 8. Les leçons

* **Une mesure trop courte fabrique une conclusion** : 30 s de banc disaient « ne se pose jamais » ;
  120 s ont montré une convergence en 30 s. Mesurer au-delà de ce que l'hypothèse annonce.
* **« Au moins une par image » garantit la fin, pas la vitesse** : quand chaque image coûte plus que
  le budget, la règle devient « une unité de travail par image complète ». Au repos, le travail se
  fait hors des images.
* **Regarder l'image avant de dire fini** a encore payé : la première capture de la barre de couleur
  montrait des teintes qui ne changeaient rien de visible (les pilules).
* **La fenêtre de capture du banc compte depuis le script**, pas depuis le lancement de Glucose
  (`nix develop` l'ouvre plusieurs secondes plus tard) : une capture « à 5,5 s » est tombée avant la
  commande de 4 s. Donner de la marge, et relire la capture.

## 9. Prouver — le compte de la session

* **Suite entière** : 2 965 épreuves, aucune tombée, 7 ignorées (carte graphique absente) ;
  `cargo fmt`, clippy et les cliquets à zéro (le cliquet des 600 lignes a arrêté une fois
  `annotation.rs` : extrait, pas relevé).
* **Sabotages** : REPOS-2 7 sur 7 ; AGENT-8 3 sur 3 ; OPERATEUR-1 7 sur 7 ; OPERATEUR-2 6 sur 6 ;
  COULEUR-1 12 sur 12 (rejoués sur la version finale) ; COULEUR-2 3 sur 3, et AGENT-8 et
  OPERATEUR-2 rejoués après lui (3 sur 3, 6 sur 6) ; GROUPER-1 7 sur 7 (`grouper-1-98.json`).
  **En tout : 45 sabotages, 45 tombés** (dont neuf rejoués sur la version finale de leur code).
* **Le vrai programme, regardé** : le squelette au banc hors écran (repos et vols, en paires
  entrelacées avec `f967f66`) ; le squelette coloré par l'agent (D) ; les opérateurs et la barre de
  couleur par des scénarios hors écran, chaque capture regardée ; les images témoins comparées
  pixel à pixel avant de changer deux empreintes.
* **Pas à son écran** : rien de cette session.
