# 103 — `Alt+Z` d'un seul geste, les liens près des blocs, et les membranes qui s'empilent

> Session du 11/11/2026, à partir de 0 h 09. La passation (fiche 102 § 11) laissait RETOUR-2 et
> LIEN-4 écrits, éprouvés, **ni sabotés ni regardés**. La beta 10 était publiée (son clic, le 10/11
> à 23 h 59). Ordre changé, avec une raison : **regarder d'abord, saboter ensuite** — saboter une loi
> que l'image va corriger est du travail perdu (la leçon de la fiche 102). L'image a corrigé les
> deux.

## 1. Ses paroles exactes, relues

Dans la conversation du 10/11 au soir : *« alt Z c'est vraiment juste le ctrl Z mais pour la
caméra »* — réponse à la question « `Alt+Z` après un voyage à la molette ». Pour les liens, sa droite
(`f(x) = 100 − 70 (x − 1) / 19`, *« jamais en dessous de 8 % »*) et sa proximité `100 / (1 + d)`,
posée sur des **points** A, B, C ; aucune unité de distance. La droite est codée telle qu'il l'a
écrite ; l'unité et la façon de grouper les crans étaient des choix de la session d'avant.

## 2. RETOUR-2 — cinq crans faisaient cinq pas

**Au pilote** (le vrai Glucose sans fenêtre, la carte de Wikipédia) : cinq crans de molette, un
toutes les 80 ms, puis `Alt+Z` — la vue ne reculait que d'un cran ; le second `Alt+Z`, d'un cran
encore. La loi d'avant fermait le pas à la première image immobile ; or à la souris chaque cran se
montre d'un coup (fiche 51 § 1), et entre deux crans d'une même main le vrai Glucose dessine des
images immobiles (les textures qui reviennent nettes, les flèches en cascade). L'épreuve ne le voyait
pas : elle enchaînait les crans image après image.

**La recherche** : une molette n'a ni début ni fin de geste. Les navigateurs bornent leur
« transaction de molette » par une durée — 1,5 s chez Firefox, *« un nombre assez arbitraire suggéré
par un fabricant de matériel »*, dit son propre code ([Gecko](https://wiki.mozilla.org/Gecko:Mouse_Wheel_Scrolling)).
AutoCAD regroupe en un pas d'annulation tous les zooms et déplacements consécutifs
([UNDO, Combine](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-LT/files/GUID-2729A466-B199-4840-B92B-4D8A38A8ADB8.htm)) —
mais sa caméra n'a pas d'histoire à elle. Maya et Rhino ont un « annuler la vue » sans dire comment
ils groupent la molette.

**La loi** (`app/mouvement.rs`, `noter_le_pas_de_la_camera`) : un pas s'ouvre quand la vue quitte
son lieu et y laisse ce lieu ; il reste ouvert **tant que la main tient la vue** (un glisser, deux
doigts, le pavé en geste — un drapeau, puisqu'une main immobile ne fait dessiner aucune image) et
**tant que la vue rebouge en moins qu'un double-clic** : `DBLCLICK_MS`, 350 ms, la constante que
Glucose avait déjà pour dire « le même geste » (fiche 07 § 1). Aucune constante neuve. Un retour
ferme le pas. Le lieu noté est la vue **d'avant le mouvement de cette image** (une épreuve l'a
trouvé : un `Alt+Z` posé entre deux images n'en a dessiné aucune, et l'image d'avant était fausse),
noté **avant** le départ d'un vol — un cran et `F` dans la même image laissent deux lieux, dans
l'ordre. La garde « un vol ne se note pas deux fois » n'avait aucun effet (l'histoire ignore déjà
un même lieu noté deux fois) : retirée.

**Rejoué au pilote** : cinq crans, un `Alt+Z` — 207 pixels différents du départ sur deux millions
(le curseur et le survol) ; le second remonte au lieu d'avant « tout cadrer ».

## 3. LIEN-4 — la souris et le coût, l'image et la pelote

### Le coût

Au pilote, trois secondes de souris sans rien faire sur la carte : **110 images, 26 ms chacune** —
toute la scène refaite à chaque mouvement, dès que deux liens étaient à l'écran. Le profil
(`perf`, avec les appelants) nomme le poste : **les lettres des titres des membranes**, détourées,
refaites au processeur à chaque image (`shifted_glyph`, `blend_glyph`, `halo` : 40 % du temps). Les
flèches, elles, coûtent 0,7 ms.

**La règle** : l'image ne se refait pour la main que si un lien visible changerait **d'au moins un
niveau d'écran** (un 255ᵉ d'opacité) depuis ce que la dernière image montre — en deçà, l'image
refaite serait la même au pixel près (un pixel vaut le fond plus `(couleur − fond) × opacité`). Le
seuil n'est pas choisi : c'est l'écran. **Une seule question pour le survol et la souris**
(`la_main_change_un_lien`) : le nœud survolé et la souris de la dernière image entière sont retenus.
La règle du survol de LIEN-1 (« à la taille réelle, tout lien est déjà plein ») n'était plus vraie
depuis LIEN-4 — vingt liens ne sont qu'à 30 % — : survoler un nœud lié de près ne redessinait plus.
L'ancienne règle, « tout redessiner à chaque mouvement », le cachait.

**Ce que ça gagne, honnêtement** : de loin, beaucoup (sous le plancher, rien ne bouge) ; à distance
de lecture, presque rien — la souris y change vraiment les liens à chaque mouvement, c'est ce qu'il
a demandé, et l'écran doit le montrer. Ce qui coûte reste la scène entière : **les titres des
membranes** (§ 5).

### La pelote

Au pilote, à distance de lecture dans « Philosophie » : **l'écran couvert de liens vifs**, souris
loin, sur un texte ou entre deux — trois images presque identiques. La cause : la distance se
comptait en tailles de nœud, et les liens de Wikipédia arrivent sur des **membranes** d'article de
mille unités ; à 170 unités de « Philosophie », on était à 0,13 de sa taille : 88 %. La souris était
partout « tout près ».

**La loi** : la souris rapproche un lien des **blocs** qu'il relie — une carte, une note, une photo,
un dossier, ce qu'une flèche contourne déjà (`obstacle`) — pas des membranes, régions qu'on habite.
Un lien sans bloc à ses bouts se rapproche par lui-même (la règle d'avant). **Regardé** : souris
loin, les liens se calment ; sur « Philosophie », ses liens ressortent ; sur « Logique », les siens.
L'effet est **doux** : c'est sa courbe, `1 / (1 + d)` — un bloc à deux tailles garde 33 %.

## 4. Prouver

* **La suite complète** : 3 102 épreuves passées, 0 tombée, 7 ignorées (les bancs) ; clippy strict et
  `cargo fmt` à zéro. Le premier passage avait fait tomber cinq épreuves, toutes justes : le cliquet du
  couplage (deux lectures de `board.images` et `board.folders` depuis le bureau — le noyau gagne
  `quadtree::noeud_par_id`, et `arrow::node_rect` s'écrit par lui), le cliquet des 80 lignes
  (`peindre_ce_qui_a_change` : extrait `UiState::retenir_la_main_des_liens`), et **trois empreintes
  témoins** — changées par LIEN-4 dès la veille (la suite n'avait pas tourné) : la scène a trois liens,
  sa droite les met à 92,6 % ; 5 866 pixels, tous sur les flèches, quatorze niveaux au plus, regardés
  avec l'image de l'écart contre la beta 10 (`aebb3f7`). Empreintes mises à jour, la raison écrite.
* **Épreuves neuves** : RETOUR-2, dix (la rafale malgré les images immobiles, deux gestes séparés,
  le glisser tenu, tenir sans bouger, lâcher puis reprendre, un retour ferme le pas, deux doigts, le
  pavé, un cran et un vol dans la même image, un vol) ; LIEN-4, quatre (rien ne change → rien ne se
  redessine, moins d'un niveau et les pas qui s'additionnent, les blocs pas les membranes, au doigt
  pas de souris) ; l'épreuve « la souris fait redessiner » va désormais sur le vide, à côté du nœud —
  sur lui, le survol l'allumait de toute façon et elle ne prouvait pas que la souris arrive à l'image.
* **Sabotages écrits, PAS JOUÉS** (la session s'est arrêtée à sa demande) : `retour-2-103.json` (14),
  `lien-4-103.json` (29), et `lien-1-102.json` **ré-ancré** (23 — neuf ne s'appliquaient plus depuis
  LIEN-4 et la question unique de la main). Chaque passage vérifié unique dans le code.
* **Envoyé** : `826c0e0`. **Son Glucose mis à jour** (`nix profile upgrade --refresh GlucoseGit`, le
  profil verrouillé sur `826c0e0`) : il a enfin ITINERAIRE-2 (ses 0 fps), RETOUR-2 et LIEN-4.

## 5. Les membranes emboîtées — la mesure, la recherche, un prototype en image

Sa parole du 10/11 : *« c'est principalement le texte… plus on met des membranes dans des membranes,
moins on voit grand-chose ; on arrive à lire, mais… c'est vraiment pas beau »*.

### La mesure

Chaque membrane pose **trois couches translucides sur tout son intérieur** (le fond à 8/255, deux halos
à 8 et 14/255 qui débordent mais couvrent aussi l'intérieur) : environ 11,3 % de sa teinte. Emboîtées,
elles se multiplient, `1 − 0,887ᵏ` : **45 % à cinq niveaux, 57 % à sept** (Wikipédia › thème ›
sous-thème › article › section › sous-section…), chaque niveau d'une autre teinte — **une boue**. Et le
papier d'un texte est exactement ce fond composé (`Contenants::fond_en`, LUEUR-3) plus 3 % de sa teinte
symbiotique ; autour de chaque texte, une lueur arc-en-ciel (violette, rose, verte) qui ne dit rien.

### La recherche

* Imhof et la cartographie : les grandes surfaces calmes et sourdes, les couleurs fortes aux petites
  ([Ordnance Survey](https://docs.os.uk/more-than-maps/geographic-data-visualisation/guide-to-cartography/colour),
  [USGS](https://pubs.usgs.gov/tm/2005/11B01/05tm11b01.html)) — une carte ne superpose pas les fonds :
  chaque point a la couleur de la plus petite région qui le contient.
* Les treemaps : van Wijk et van de Wetering remplacent les bordures emboîtées, « labyrinthiques », par
  un ombrage ([Cushion Treemaps, 1999](https://www.win.tue.nl/~wstahw/papers/ctm.pdf)), jugé ensuite
  difficile à lire ([Lü et Fogarty, Cascaded Treemaps, 2008](https://homes.cs.washington.edu/~jfogarty/publications/gi2008.pdf)) ;
  les visualisations « à la carte » battent les treemaps emboîtés ([arXiv 1909.09351](https://ar5iv.labs.arxiv.org/html/1909.09351)).
* Material 3 a quitté les surfaces qui s'additionnent avec l'élévation pour une échelle de **tons
  absolus** ([Material](https://material.io/blog/tone-based-surface-color-m3)).
* Tree Colors révisé (Mertz et Kohlhammer, VIS 2024, [arXiv 2407.08287](https://arxiv.org/html/2407.08287v3)) :
  chroma et clarté interpolées avec la profondeur.
* Le contraste du texte : APCA, Lc 75 au minimum, 90 de préférence pour un texte courant
  ([APCA](https://github.com/Myndex/apca-introduction)).

### Le prototype, en image (pas dans le dépôt)

`~/.cache/glucose-bancs/membranes-103/` : `prototype-103.patch` (127 lignes, sur `826c0e0`), `vues.sh`
(les quatre mêmes vues, voie du processeur forcée par `moteurs.txt`), et pour chaque variante les
captures ; `comparaison-*.png` (avant / région / région + neutre) et `comparaison2-*.png` (avant /
profondeur / profondeur + neutre). **Voie du processeur seulement** : la carte graphique (ses nuanceurs
des membranes) n'est pas touchée.

1. **La région la plus intérieure l'emporte** : chaque membrane efface sous son corps les fonds de ses
   mères (`draw_membranes`, `Contenants::fond_en`). **En profondeur, gagné** : plus de boue, la
   structure se lit. **En vue d'ensemble, perdu** : les couleurs des thèmes s'éteignent presque (11 %
   d'une teinte sur du noir). L'empilement colorait fort les petites régions — Imhof sans le vouloir ;
   le défaut n'est pas la force, c'est **le mélange des teintes**.
2. **La teinte seule, à la force de sa profondeur** (`PROTO_PROFONDEUR` : `1 − 0,887ᵏ` d'une seule
   teinte) : **en vue d'ensemble, la meilleure des trois** — chaque région pure, mieux distinguée
   qu'aujourd'hui. **En profondeur, perdu** : les régions profondes pâlissent jusqu'au presque blanc
   (« Solvant »), et le gris du texte y perd son contraste.
3. **Les textes sans domaine neutres** (`PROTO_NEUTRE`, le chantier qu'il a approuvé, A) : les lueurs
   arc-en-ciel disparaissent, tout se calme. Attention : les **flèches** prennent la *teinte*
   symbiotique de leurs bouts — le prototype la leur laisse ; décider leur couleur fait partie du
   chantier.

**La piste qui en sort** : la teinte de la seule région la plus intérieure (variante 2), mais une force
**bornée par le contraste du texte** — le papier d'un texte gardant un Lc APCA d'au moins 75 avec ses
lettres —, plutôt qu'un plafond choisi à l'œil. Et l'emboîtement dit par les frontières et les titres
(TITRE-1 fait déjà grandir le titre comme `√(A/A₀)` ; le trait du bord pourrait suivre la même loi —
la hiérarchie des frontières d'une carte). **Rien de tout ceci n'est décidé : à lui montrer en image.**

## 6. Les leçons

* **Une épreuve qui joue une image à chaque pas ne voit pas ce que le vrai programme dessine entre
  deux** : la rafale de molette passait, et faisait cinq pas au pilote.
* **Une règle trop large en cachait une fausse** : « tout redessiner » masquait que le survol de près
  ne redessinait plus.
* **Une distance « en tailles de nœud » n'a de sens que pour des choses qu'on approche** : une
  région qu'on habite est à distance zéro de tout ce qu'elle contient.
* **Le profil nomme le coupable** : j'accusais les pointillés des grandes membranes ; c'étaient les
  lettres de leurs titres.
* **Une suite qui n'a pas tourné laisse des empreintes en retard** : LIEN-4 avait changé trois images
  témoins la veille ; la comparaison à la version publiée (`aebb3f7`) l'a prouvé.
* **Un prototype qui corrige une vue peut en casser une autre** : regarder la vue d'ensemble ET la
  profondeur, toujours ensemble.
* **La mémoire impose l'ordre** : une seule grosse compilation à la fois — la suite, puis Nix, puis le
  prototype ; les sabotages en dernier.
