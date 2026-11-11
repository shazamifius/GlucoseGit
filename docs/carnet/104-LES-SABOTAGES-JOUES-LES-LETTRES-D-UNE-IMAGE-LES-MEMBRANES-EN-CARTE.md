# 104 — Les sabotages joués, les lettres d'une image, et les membranes en carte

> Session du 11/11/2026, de 2 h 33 à 5 h 45, sans lui (il dormait). La passation (fiche 103 § 6)
> laissait 66 sabotages écrits et pas joués, et les membranes emboîtées mesurées, un prototype en image,
> rien de décidé. Envoyé : `f8b188e` (les sabotages), `465794a` (LETTRES-1) ; son Glucose mis à jour.

## 1. Les 66 sabotages

`retour-2-103.json` (14), `lien-4-103.json` (29), `lien-1-102.json` (23), dans un arbre détaché, après
un témoin sur le code intact (`retour_` : 22 épreuves passées ; `lien_1 lien_4` : 20). **64 tombés,
deux aveugles** — et aucun des deux ne cachait un défaut du programme ; tous deux cachaient une épreuve
qui ne regardait pas où la garde agit.

* **« Tout ce qui est visible compte »** (LIEN-4 : compter les nœuds à l'écran au lieu des liens). Les
  scènes de l'épreuve avaient vingt-six liens et vingt-huit textes : 26 ou 54, sa droite donne zéro
  dans les deux cas. L'entrée qui le fait tomber : **un lien entre deux textes, seul, est plein** (1
  contre 3, `test_lien_4_seuls_les_liens_comptent`).
* **« Un bout de lien n'est pas reconnu au survol »** (le noyau, `arrow::touche_un_lien`). Depuis LIEN-4,
  le survol ne la lit plus : il demande si la main change un lien (`la_main_change_un_lien`). La garde a
  un autre lecteur, le menu (« Montrer ses liens » sur un nœud qui en a, LIEN-3), et une épreuve qui le
  voit déjà (`test_lien_3_le_menu_ouvre_mene_et_ferme` : « eau » est la **source** d'un lien) ; le
  sabotage ne lançait que `lien_1`. Le commentaire du noyau nommait encore le survol : corrigé ; le
  sabotage vise son vrai lecteur.

**Rejoués** (l'épreuve neuve, le sabotage réorienté) : les deux tombent — **66 sur 66**.

## 2. Les membranes emboîtées — une loi de carte

### Ce que le contraste dit

Les lettres d'un texte (`#dce1eb`) n'ont, en APCA, qu'un contraste de **88 sur le canevas nu** : le
niveau « préféré » de 90 est hors d'atteinte dans ce thème, le **minimum de 75 pour un corps de texte**
est la seule borne qui ait un sens. Sous la loi d'avant (chaque membrane pose 11,3 % de sa teinte sur
ses mères), le corps passe sous 75 **à cinq membranes emboîtées** (74,3), et vaut 68 à sept : son « on
arrive à lire, mais… », chiffré. Les liens bleus des textes (`#60a5fa`) ne valent que 52 sur le canevas
nu — un défaut à part.

### La recherche (consultée le 11/11/2026)

* GMap (Gansner, Hu, Kobourov, PacificVis 2010, [l'article](https://graphviz.org/documentation/GHK09.pdf)) :
  un graphe dessiné comme une carte, chaque pays d'**une** couleur « de carte », les voisins aussi
  différents que possible — aucune superposition.
* Le motif « territoire » de la cartographie ([Howarth](https://jeffhowarth.github.io/cart-patterns/patterns/territory)) :
  pas de remplissage plein qui gêne la lecture de l'intérieur ; l'accent près de la frontière ; la
  hiérarchie des frontières (pays, région, commune) par l'épaisseur et la valeur du trait.
* L'effet de taille (une couleur change-t-elle avec l'aire qu'elle couvre ?) : les sources **se
  contredisent** — un guide de coloristes dit qu'une grande surface paraît plus saturée
  ([artience](https://www.artiencegroup.com/en/column/chromatics/basic/007.html)), une étude
  d'appariement de 1995 trouve qu'il faut au contraire **plus** de saturation à la grande surface pour
  égaler la petite ([J. Gen. Psychol.](https://informahealthcare.com/doi/abs/10.1080/00221309.1995.9921229)).
  Pas de quoi écrire une loi qui ferait varier la couleur d'une région avec sa taille : je n'en écris
  pas.
* Tree Colors révisé (Mertz et Kohlhammer, VIS 2024, [arXiv 2407.08287](https://arxiv.org/html/2407.08287v3)) :
  chroma et clarté selon la profondeur. **Refusé ici** : chez Glucose, la clarté veut déjà dire
  « accessible » (sa parole du 9/11) ; une région profonde plus pâle dirait « plus facile », faux.
* Bubble Sets (Collins, Penn, Carpendale, 2009) : des contours pour des ensembles qui se recouvrent
  — c'est le domaine (un recouvrement), pas la membrane (un arbre).

### La loi

**La couleur en un point est celle de la plus petite région qui le contient** (la règle des cartes) :
le corps d'une membrane est **opaque**, du ton qu'elle aurait **seule sur le canevas** — une couche de sa
teinte d'opacité `α*` sur le noir. Les mères se peignent avant leurs filles (la profondeur, que le noyau
calcule : `Board::profondeurs_des_membranes`). `α*` ne se règle pas : c'est **la plus grande force qui
laisse un texte lisible (Lc 75) dans la région la plus défavorable** — une membrane blanche, sous la
brume blanche d'une carte (`couleur::contraste::force_des_regions`). Ici, **23 %** : la force qu'atteint
aujourd'hui une membrane à deux niveaux, tenue à toute profondeur. Les fonds colorés gardent 80 à 85.

Ce que la loi garde du sens (sa parole du 9/11) : la teinte (la proximité des domaines) et la clarté
(l'accessibilité) sont celles de la couleur de la membrane, intactes ; l'opacité (l'appartenance)
multipliera `α*` quand le modèle la portera — jamais au-dessus. La profondeur ne s'y lit pas : elle est
déjà dans les frontières et les titres.

Une membrane devient trois champs de la même loi (MEMB-FORME-1), dans l'ordre : ses halos, son corps
opaque, son contour. Le processeur, la carte graphique, l'export et le papier des textes lisent la même
liste : aucune voie n'a de code à elle.

### Le prototype, regardé (pas dans le dépôt)

`~/.cache/glucose-bancs/membranes-104/` : `prototype-104.patch` (sur `da48279` ; il porte aussi les
compteurs de lettres et un plafond relevé, qui ne servaient qu'à la mesure du § 3), `vues.sh` (les quatre
vues de la fiche 103, voie du processeur forcée), les captures de chaque variante, `cmp-*.png`. Les
variantes se choisissent par des variables : `PROTO_FRONTIERE`, `PROTO_NEUTRE`, `PROTO_PLANCHER`,
`PROTO_SANS_COULEUR` ; la loi des régions, elle, est toujours là.

* **Vue d'ensemble** : se lit comme une carte, chaque thème d'une couleur franche, aucune boue. **Défaut** :
  le grand cadre « Wikipédia » devient la surface la plus vive de l'écran.
* **« Eau », de près** : plus de boue, le texte se détache. Mais les sections ont la couleur de l'article
  (même domaine) : un seul brun, et seules les frontières séparent les niveaux.
* **« Philosophie »** : Philosophie, Éthique et Logique ont enfin chacune leur couleur.
* **Les frontières à la loi du titre** (`PROTO_FRONTIERE` : le trait et ses tirets × `√(A/A₀)`, TITRE-1 ;
  la hiérarchie des frontières des cartes) : l'emboîtement se lit d'un coup d'œil ; les grands pointillés
  sont lourds, presque du ruban de chantier.
* **Les textes sans domaine neutres** (`PROTO_NEUTRE`, le chantier A) : plus de lueurs arc-en-ciel.
* **Une membrane sans couleur choisie devient neutre** (`PROTO_SANS_COULEUR` : le gris de même clarté que
  le bleu par défaut). Trouvé en regardant le défaut de la vue d'ensemble : le cadre « Wikipédia » **n'a
  pas de couleur choisie** — c'est le bleu de Glucose Tauri, qui ne dit rien de son domaine. Neutre, il se
  calme, et les régions qui ont une couleur ressortent : la règle de Imhof (les grandes surfaces sourdes)
  obtenue par le sens, sans loi de taille. C'est la marque de Tauri sur tous ses documents : sa parole.
* **Le plancher des liens, deux lectures** (`PROTO_PLANCHER`) : `max(lecture × présence, 8 %)`
  (aujourd'hui, ses mots tels quels) garde une brume des 1 128 liens sur toute la carte ;
  `lecture × max(présence, 8 %)` les efface presque de loin, comme lors de sa validation de LIEN-1.

**Ce que la loi change aussi, dit dans la page** : la grille de points ne se voit plus dans une membrane
(l'aplat la couvre, comme une carte couvre son quadrillage), ni une zone de storyboard posée dessous.

### Sa parole, attendue

Une page avant/après, privée, avec une visionneuse (quatre vues, sept versions) et cinq questions en
prose : https://claude.ai/artifact/4a7Uj11JxSbEmctaoRV6Lf. Mes avis : oui à la loi des régions ; oui aux
frontières sur l'idée, plus légères sur l'allure ; oui aux textes neutres (un lien entre deux textes
neutres deviendrait gris : la carte de Wikipédia n'a aucun domaine posé) ; ses mots tels quels pour le
plancher ; oui aux membranes sans couleur neutres. **Rien n'est écrit dans Glucose pour les membranes.**

## 3. LETTRES-1 — le cache des lettres tient une image, sans plafond

La fiche 103 avait nommé le poste (les lettres détourées des titres, 40 % de chaque image sur la carte
de Wikipédia) sans en trouver la cause. **La mesure** (deux compteurs au prototype, le vrai Glucose au
pilote, trois secondes de souris sans rien faire, voie de la carte graphique comme chez lui) : de loin,
**une image demande 26 282 lettres et en rate 7 727 ; le cache plafonne à 4 020 sur 4 096**. Le plafond
(R-40) avait été mesuré à 531 lettres pour une image d'un document ordinaire, cinq fois la marge ; puis
chaque titre a pris la taille de sa région (TITRE-1) : 190 titres, chacun à son corps, chaque lettre à
sa phase, plus son halo — **5 654 variantes par image**. Un cache plus petit que ce qu'une image demande
ne garde rien : chaque image chassait ce que la suivante redemandait (le profil de la veille le disait
sans qu'on le lise : `shifted_glyph` et `halo` ne tournent qu'à un raté).

Sans plafond, le même cache gonfle à 42 275 variantes après trois zooms (chaque image d'un zoom pose des
tailles neuves). **La loi** est celle que Glucose tient déjà pour ses mises en page et ses textures :
**une image qui commence oublie ce que la précédente n'a pas demandé** (`Typography::nouvelle_image`).
Le plafond et l'éviction par quarts disparaissent ; aucune capacité n'est choisie.

**Mesuré au pilote, en paires entrelacées** contre son Glucose installé (`826c0e0`), sur la copie de sa
carte : de loin, la souris seule — **25,5 → 15,5 ms par image** (deux paires : 25,58/15,50, 25,46/15,56),
les membranes 16,5 → 4,9 ms, 45 → 62 images en trois secondes ; dans « Philosophie », rien ne change
(5,4 ms) ; pendant les zooms, la médiane ne bouge pas (18,5 ms) et le p90 descend de 31 à 24 ms.

**Ce qui reste, dit** : les titres ne suivent pas encore la loi du gris (GRIS-2). Ce n'est plus une
question de coût ; c'est une question de forme : un titre détouré porte un halo de huit copies, décalées
d'un pixel et demi quel que soit le corps — son encre ne croît pas comme le carré du corps, et le gris
exact doit la compter.

## 4. Les leçons

* **Un sabotage aveugle ne dit pas toujours « garde inutile »** : les deux de cette session avaient une
  garde juste et une épreuve qui regardait ailleurs — une scène où deux lois donnaient le même nombre, et
  une garde dont le lecteur avait changé de main. Chercher l'entrée qui la fait tomber, et le lecteur.
* **Un cache plus petit qu'une image ne garde rien** : sa capacité était juste quand elle a été mesurée ;
  TITRE-1 a multiplié par dix ce qu'une image demande sans que personne la relise. Une borne choisie se
  périme ; la loi « ce que l'image d'avant n'a pas demandé s'oublie » ne se périme pas.
* **Compter avant de soupçonner** : deux compteurs au prototype (demandes, ratés) ont tranché en une
  mesure ce que le profil de la veille montrait sans qu'on le lise (`shifted_glyph`, `halo` : des
  fonctions de raté).
* **Regarder le défaut de sa propre proposition jusqu'à sa cause** : « le cadre bleu est trop vif » n'était
  pas une affaire de force ni de taille, mais une couleur que personne n'a choisie.
* **Les cliquets, une troisième fois** : `renderer.rs` à 601 lignes ; le décodage des couleurs
  hexadécimales n'avait rien à y faire (`renderer/hexa.rs`).
