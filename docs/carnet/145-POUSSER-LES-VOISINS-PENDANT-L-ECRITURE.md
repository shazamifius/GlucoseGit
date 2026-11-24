# 145 — Pousser les voisins pendant l'écriture : cadré (24/11, le relais, compte b)

Sa réponse du 24/11 sur `relais-rep-327` : *« le truc dérangeant, c'est justement qu'il n'y ait pas
de mise à jour LORSQU'on écrit »*. Les trois aperçus d'après l'écriture sont retirés du plan.

## 1. Ce qui existe

À chaque lettre, la saisie fait grandir la carte vers le bas (`interactions/resize.rs`, vers la ligne
530) puis, **dans le même geste**, pousse le bas de sa membrane et de celles qui la contiennent
(TEXT-FIT-2, `glucose-core/src/store/pousser.rs`, fiche 124). Les voisins, eux, ne bougent jamais
(MEMB-7) : le texte grandit par-dessus.

## 2. Ce que dit la recherche

Le problème s'appelle *overlap removal* ; le critère qu'il veut — « ce qui était à gauche reste à
gauche » — est **l'ordre orthogonal**, et le moindre déplacement sous cet ordre est NP-difficile
dans le cas général (van Garderen et al., *Minimum-Displacement Overlap Removal*,
<https://diglib.eg.org/handle/10.1111/cgf13199>). VPSC le résout dimension par dimension sous des
contraintes de séparation (Dwyer, Marriott, Stuckey, *Fast Node Overlap Removal*, GD 2005,
<https://link.springer.com/content/pdf/10.1007/11618058_15.pdf>) ; *Force Scan* (Misue et al.)
balaie dans un sens et garde l'ordre ; la comparaison de neuf méthodes :
<https://hal-lirmm.ccsd.cnrs.fr/lirmm-02879677/document>.

**Notre cas est bien plus simple** : un seul nœud grandit, d'un seul côté (le bas). Un balayage à la
Force Scan, vers le bas seulement, suffit et garde l'ordre par construction ; aucun solveur.

## 3. La loi proposée

* **Ce qui grandit pousse ce qui était sous lui, à son niveau** : le texte ses voisins de membrane,
  la membrane qui grandit ses propres voisins — la même loi à chaque étage, comme TEXT-FIT-2.
* Seul ce qui était **entièrement dessous** au début de l'écriture et que le bas neuf recouvre
  descend, de `bas + ECART − haut`, et pousse à son tour (balayage par haut croissant). Ce qui
  chevauchait déjà ne bouge pas (« seul ce qui grandit pousse »).
* **Une fonction pure de l'état au début de l'écriture et du bas courant**, pas une suite de
  poussées : effacer des lettres fait remonter les voisins, jamais plus haut que leur place
  d'origine ; aucune dérive ; le Ctrl+Z unique de la saisie rend l'état de départ.
* Les flèches suivent leurs bouts comme pour tout déplacement.

**Le balayage, précisé** (14 h 22) : à un niveau (les nœuds de même `membrane_id`, flèches
exclues), les candidats sont ceux dont la boîte de départ est entièrement sous le bas de départ du
nœud qui grandit ; triés par haut de départ croissant, chacun prend
`haut = max(haut_départ, max(p.bas_neuf + min(ECART, haut_départ − p.bas_départ)))` sur les pousseurs
`p` déjà vus qui le chevauchent horizontalement — l'écart d'origine gardé s'il était plus petit que
`ECART` : il ne bouge qu'au contact, du strict nécessaire —, puis devient pousseur à son tour. **Puis
l'étage du dessus** : la membrane grandit jusqu'au plus bas de ce qu'elle contient (le texte, les
voisins poussés) + `ECART` (la loi de `membranes_poussees`, élargie aux poussés), et le même balayage
pousse ses voisins à elle. Une membrane poussée emporte son contenu (le déplacement d'une membrane
qu'on tire, déjà là). Pas de branche horizontale : la saisie ne grandit que vers le bas.

**Fait (14 h 25) : la loi d'un niveau**, pure, dans le noyau — `voisins_pousses`
(`glucose-core/src/store/pousser.rs`) ; trois épreuves (`store/pousser/tests.rs` : au contact et en
cascade, effacer fait remonter, seul son niveau) ; `outils/sabotages/pousser-145.json`, 5 sur 5. Le
filtre « entièrement dessous » s'est révélé redondant : un pousseur qui n'a pas bougé ne pousse
jamais (`bas + min(ECART, écart) ≤ haut`) — retiré.

**Puis (14 h 28) l'étage du dessus** : `pousser_en_ecrivant` rend, par rang, le haut neuf de chaque
voisin poussé et le bas neuf de chaque membrane qui grandit (`Bord::Haut`, `Bord::Bas`), étage après
étage ; une épreuve de plus, deux sabotages de plus (7 sur 7). Il remplacera `membranes_poussees` à la
saisie, qu'il contient (un texte sans voisin ne fait que pousser ses membranes).

À faire : l'état
de départ gardé dans la session d'édition, la loi branchée à côté de `pousser_les_membranes` ; épreuves (pousser,
cascade, remonter, ordre gardé, un seul Ctrl+Z, les pairs du partage reçoivent les déplacements),
leurs sabotages, et l'essai au pilote.

**Le branchement, repéré (14 h 31)** — pour la session qui le fera :
* **L'état de départ** : la saisie s'ouvre dans `start_text_edit_at` (`interactions/text_edit.rs`) ;
  y garder les annotations du tableau (à côté de `TextEditSession`, `renderer.rs`, plutôt que dedans :
  ses constructions littérales sont nombreuses), et l'oublier à la validation.
* **Déplacer un poussé avec ce qu'il emporte** : `translation_de_la_selection`
  (`glucose-core/src/store/images.rs`) sait déjà emporter le contenu d'une membrane et les bouts des
  flèches (`Emport::de(tableau, images, annotations)`, `store/membranes.rs`) — mais pour la
  sélection ; en extraire `translation_de(tableau, ids, delta)` et la partager. Le delta de chacun :
  son haut voulu (celui de `pousser_en_ecrivant`, ou son haut de départ s'il n'y est plus) moins son
  haut courant.
* **Dans le geste ouvert** : `pousser_les_membranes` écrit par `set_annotation_rect` dans la saisie
  ouverte (`begin_live_edit`) ; vérifier qu'une `Edit::Translation` y entre aussi (sinon
  `record_as_one_gesture` ouvrirait une seconde entrée d'annulation) — l'épreuve « un seul Ctrl+Z ».

**Fait (14 h 32)** : `translation_de` extraite (`store/images.rs`, la sélection la partage) ; et
`Store::pousser_en_ecrivant(tableau, départ, déjà, id, bas)` (`store/pousser.rs`) applique la loi dans
le geste ouvert — chaque voisin prend son haut voulu en emportant son contenu, ou revient à son haut de
départ ; `déjà` (ce que cette écriture a poussé) borne le retour à ce qu'elle a déplacé, jamais un nœud
qu'une autre main a bougé. Épreuve `test_pousser_pendant_l_ecriture_par_le_journal` (au contact,
effacer fait remonter jusqu’à sa place, un seul Ctrl+Z) ; 9 sabotages sur 9 (14 h 34). **Reste** : garder `départ` et `déjà` à l'ouverture de la saisie
et appeler la méthode à la place de `pousser_les_membranes` (`interactions/resize.rs`), ses sabotages,
le pilote.

**Deux points avant de brancher (14 h 37)** :
* **La charte** (100 images par seconde) : copier toutes les annotations du tableau à chaque ouverture
  de saisie coûterait cher sur un tableau de 750 000 nœuds (le banc de JRN-4). Ne garder que les
  boîtes (id, boîte, membrane) **des niveaux concernés** — la membrane du texte et ses ancêtres —, et
  pour le niveau du canevas, seulement ce qui est sous le texte dans sa bande horizontale (un index
  spatial s'il existe). À mesurer au banc avant de choisir.
* La saisie ne fait jamais rapetisser la carte (`resize.rs` : la hauteur est un plancher) : « effacer
  fait remonter » ne jouera que le jour où elle rapetissera. Sans un `départ` (une mise en page hors
  saisie), garder `pousser_les_membranes`.

**Une voie sans copie (24/11, 19 h 02, compte b — à éprouver par la session qui branchera)** : le
départ n'a pas besoin d'être copié à l'ouverture de la saisie. Un nœud que cette écriture n'a pas encore
touché est **à sa place de départ** (ou là où une autre main l'a mis pendant l'écriture — et le pousser
depuis là est plus juste que depuis une copie périmée). Il suffit donc de garder **les boîtes d'origine
des seuls nœuds que l'écriture a touchés** — le texte lui-même dès l'ouverture, chaque voisin poussé et
chaque membrane grandie à son premier déplacement — : `origines : id → boîte`, qui remplace aussi `déjà`.
Le départ se lit alors comme « le tableau, chaque touché ramené à son origine ». Coût à l'ouverture :
une boîte ; à chaque changement de hauteur : le balayage du niveau, que `pousser_les_membranes` fait
déjà. Pour cela, `pousser_en_ecrivant` lirait des boîtes `(id, boîte, membrane)` plutôt qu'un
`&[Annotation]` (une annotation pèse ~512 octets : jamais la cloner pour lire sa boîte).

**Fait (19 h 07)** : la voie sans copie, dans le noyau — le trait `Boite` (id, boîte, membrane) que
la loi lit, `AOrigine` (un nœud vu à son origine) ; `Store::pousser_en_ecrivant(tableau, origines, id,
bas)` remplace `départ` et `déjà` : ce qui revient à son origine est oublié (une autre main peut le
bouger, l'épreuve le vérifie), le texte écrit n'est jamais ramené. 13 sabotages sur 13. **Reste** : à
l'ouverture de la saisie (`start_text_edit_at`), garder `origines = [(le texte, sa boîte)]` à côté de
`TextEditSession`, l'oublier à la validation, et appeler la méthode au lieu de `pousser_les_membranes`
(`interactions/resize.rs`, l. 531) ; l'essai au pilote.

**Branché (19 h 15)** : la hauteur d'un texte ne s'écrit dans le document qu'à la validation de la saisie
(et quand on change sa largeur) — `fit_text_card_height` (`interactions/resize.rs`). La loi s'y branche
**sans aucun état de session** : l'origine du texte, c'est sa boîte d'avant l'ajustement ;
`pousser_en_ecrivant` y remplace `pousser_les_membranes` (retiré, sans autre appelant). Épreuve
`test_pousser_145_une_saisie_ecarte_le_voisin_dessous` (l'écart d'origine gardé, un seul Ctrl+Z) ; son
sabotage dans `texte-fit-124.json` (dont les commandes, d'avant NixOS, passent désormais par `nix
develop` et `borne.sh`). **Ce qui reste, honnêtement** : les voisins s'écartent à la validation, pas lettre
à lettre — pendant qu'on tape, la carte du document ne grandit pas encore ; le faire lettre à lettre, c'est
écrire la hauteur pendant la saisie (et alors seulement « effacer fait remonter » jouera). L'essai au
pilote reste à faire.

**Lettre à lettre, conçu (24/11, 19 h 40 ; pas codé, la fenêtre du compte b au mur)**. Trois voies pesées :

* **Un geste ouvert de l'ouverture à la validation** (`begin_live_edit` à `start_text_edit_at`) : écarté.
  Tant qu'un geste est ouvert, **toute écriture d'un agent est refusée** (`Store::agir`,
  `glucose-core/src/store/agents.rs` l. 197 : `Refus::GesteEnCours`) — le relais et ses IA ne pourraient
  plus rien poser pendant qu'il tape, parfois des minutes ; et la version n'avançant qu'à la fin, ni le
  partage ni la sauvegarde ne verraient rien.
* **Déplacer les voisins au seul dessin** (comme `boite_montree`, `renderer/card.rs`, qui fait déjà grandir
  la carte en saisie sans écrire le document) : écarté. Il faudrait que flèches, membranes, sélection et clic
  lisent tous le même décalage — chaque voie du dessin à reprendre, « rien à moitié » impossible.
* **Retenu : écrire quand la hauteur change, et fondre les pas d'une même saisie en un seul geste.** La
  hauteur ne change qu'à une ligne gagnée ou perdue, pas à chaque lettre : à ce moment seulement, un geste
  court (hauteur + `pousser_en_ecrivant`, avec `origines` gardé à côté de `TextEditSession` depuis
  l'ouverture). Le journal fond un geste dans le précédent **s'ils portent la même étiquette et que rien
  n'a été publié entre eux** — la saisie (son numéro) en est une ; la validation (texte + dernier
  ajustement) aussi : un seul Ctrl+Z. Si un agent écrit entre deux pas, la saisie fait deux gestes, dans
  l'ordre vrai : honnête. C'est le regroupement des éditeurs (l'historique de ProseMirror et de CodeMirror
  fond les changements voisins, `newGroupDelay` ; de mémoire, à relire avant de coder), ici par étiquette
  et non par délai — aucune constante. **Échap** (si la saisie s'annule) : la loi étant une fonction pure du
  bas, `pousser_en_ecrivant(…, bas d'origine)` ramène tout ; le geste fondu s'annule de lui-même.
  À vérifier avant de coder : ce que fait un pas reçu du partage (`collaborer`) entre deux pas ; que
  `fondre` (FONDRE-1) sache réunir deux transactions et non seulement les éditions d'une seule.

**Vérifié (19 h 50), pour qui codera** : le journal n'a **aucune** fusion entre deux transactions — `fondre`
(FONDRE-1) ne réunit que les éditions d'une seule, et `remplacer_dans_le_geste` (GLISSER-1) ne vit que
dans un geste ouvert. La fusion par étiquette se loge donc dans `Journal::commit`
(`glucose-core/src/store/journal.rs`, l. 539) : si le sommet de `done` porte la même étiquette et
qu'`undone` est vide, ses éditions s'y ajoutent, puis `fondre` ; sinon on empile. **Ce qui est publié ne
change pas** (`publier` : l'histoire et les pairs reçoivent chaque pas, dans l'ordre) — seule la pile
d'annulation fond. **Un canevas partagé** n'empile pas dans `done` (« la pile est la sienne ») : la pile
par main de `participation.rs` doit fondre de la même façon, sans quoi Ctrl+Z y défait la saisie pas à pas.

**Fait (20 h 05) : la fusion par étiquette, dans le noyau** (FONDRE-145) — `journal/pas.rs` :
`Journal::end_etiquete` ferme un geste sous une étiquette ; `empiler` le fond dans le sommet s'il porte la
même (puis `fondre` ; revenu à son départ, il disparaît de la pile) ; défaire ou refaire efface l'étiquette
du sommet ; l'étiquette d'un geste vide ne reste jamais en attente. `remplacer_dans_le_geste` (GLISSER-1)
y a déménagé (`journal.rs` : 598 → 581 lignes). `Store::end_live_edit_etiquete` (`undo.rs`) l'expose. Six
épreuves (`journal/pas_tests.rs`), **5 sabotages sur 5** (`outils/sabotages/fondre-145.json`) ; le noyau
entier vert. **Reste**, dans l'ordre : la pile par main d'un canevas partagé (`participation.rs`) ;
l'application — `origines` et un numéro de saisie à côté de `TextEditSession`, à chaque frappe la hauteur
du tampon (`boite_montree` la calcule déjà) écrite et poussée **si elle change**, sous l'étiquette de la
saisie, la validation sous la même ; Échap ; puis l'essai au pilote.

**Le partage, repéré (20 h 15), pas codé** : un geste partagé entre dans sa pile à
`Store::partager_les_gestes` (`participation.rs`, l. 219 : `p.defaire.push(geste.inverse())`), lu de
`journal.nommer_ce_qui_part`, qui ne connaît pas l'étiquette. Voie proposée : l'`Ecrit` publié porte
l'étiquette (prise dans `commit`, branche partagée, comme `empiler` la prend) ; `Participation` garde
l'étiquette de son sommet ; même étiquette → l'inverse neuf se place **devant** les éditions du sommet
(défaire B puis A), sinon on empile ; `pas_partage` (défaire, refaire) l'efface. Les gestes reçus entre deux
pas ne coupent pas la fusion : défaire rebase déjà sur eux (`rebaser`). Épreuve à écrire sur deux
participants (comme `participation_tests.rs`), puis ses sabotages.

**Fait (22 h 10) : le partage** (commit `e02a6c5`), par la voie repérée : `Ecrit.etiquette` (posée par
`etiqueter_le_dernier_ecrit` dans la branche partagée de `commit`), rendue par `nommer_ce_qui_part` ;
`Participation::empiler` fond l'inverse neuf **devant** le sommet de même étiquette ; `pas_partage`
efface `sommet`. Deux épreuves (`fondre_145_*`, `participation_tests.rs`, 599 lignes : le cliquet est
atteint, la prochaine épreuve de ce fichier va ailleurs) ; **9 sabotages sur 9**. Note : un `saboter.py`
coupé en pleine compilation laisse le cache incrémental du noyau corrompu (l'éditeur de liens : « undefined
reference to anon… ») — `rm -rf target/debug/incremental/glucose_core-*` le répare. **Reste** :
l'application, Échap, l'essai au pilote.

**Fait (22 h 25) : lettre à lettre dans l'application** (`interactions/text_edit/poussee.rs`). À chaque
changement du tampon (`after_text_change`, et le `Ctrl+Z` du texte), `suivre_la_hauteur` ouvre un pas,
écrit la hauteur si elle change et pousse (`Store::pousser_en_ecrivant`), puis le ferme aussitôt sous le
numéro de la saisie ; la validation se ferme sous le même (`commit_editing`). L'état, `Poussee` (numéro,
`origines`, `laissees`), vit **à côté** de la session, comme l'historique du texte (la session est copiée
à chaque frappe). Quatre décisions :

* **La hauteur, une seule fonction** : `card::hauteur_poussee`, que lisent le dessin (`boite_montree`) et
  le document — en saisie, le texte coupé comme on l'édite (`Source`), sinon comme on le lit. Le plancher
  reste en f64 : passé par f32, il différait de la hauteur écrite et chaque frappe l'aurait récrite.
* **Le plancher est la boîte d'origine**, plus la hauteur courante : écrite pendant la saisie, celle-ci
  aurait empêché d'effacer de faire remonter, et la validation (`Rendered`) de reprendre la hauteur lue.
* **Ce qu'une autre main prend lui reste** : `laissees` garde chaque boîte telle que le dernier pas l'a
  laissée ; une autre, retrouvée au pas suivant (un agent, un pair), et la saisie l'oublie — le nœud écrit,
  lui, reprend son origine là où on le trouve.
* **Un geste déjà ouvert** (un glisser) n'est pas le nôtre : le pas attend le suivant, ou la validation.

**Ce que l'essai a appris : FONDRE-1 ne fondait que des voisines.** Une saisie revenue à son départ
(Entrée puis Effacer) laissait un `Ctrl+Z` mort : le texte, la translation du voisin, le texte, la
translation… jamais deux éditions d'une même case côte à côte. **FONDRE-145, par case**
(`journal/fondre_par_case.rs`) : deux éditions d'une case se rejoignent par-dessus ce qui ne la touche pas
— un changement sur place d'une autre case, une translation qui ne la déplace pas ; une insertion, un
retrait, un tout arrêtent tout (JRN-2). **Dans la pile seulement** (`fondre_les_pas`, appelé par
`empiler`), deux translations du même ensemble s'additionnent : le fichier le refuse (GLISSER-1, le bit
près), la pile n'est ni relue ni écrite, et défaire une translation applique déjà son opposé. Limite
honnête : le retour d'un voisin à son origine est exact quand départ et arrivée sont du même signe et à
moins d'un facteur deux (Sterbenz) ; ailleurs (autour de zéro), il peut rester un ulp, et le geste une
translation d'un ulp — vraie, invisible.

Épreuves : 7 `test_pousser_145_*` (`poussee/tests.rs`) ; 4 dans `fondre_tests.rs` (dont
`test_fondre_1_seules_des_voisines…`, réécrite : des cases entrelacées se fondent désormais) ;
`outils/sabotages/pousser-145-lettre.json`, 15 sabotages. Toutes les épreuves de l'espace de travail
vertes (3 651), clippy vert ; 15 sabotages sur 15. Piège : après des sabotages coupés, le cache incrémental a donné trois pannes du compilateur (éditeur de liens, « dep graph », un SIGSEGV dans LLVM) — `rm -rf target/debug/incremental/glucose_{core,desktop}-*` les a toutes réglées. **Reste** : l'essai au pilote, à son écran ensuite (« Entrée dans un texte
au-dessus d'un autre : l'autre descend pendant qu'on tape »).
