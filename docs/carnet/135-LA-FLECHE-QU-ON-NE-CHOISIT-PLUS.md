# 135 — La flèche qu'on ne choisit plus : une seule source pour sa géométrie (ANCRE-1)

> Le 23/11/2026, de 22 h 28 à 23 h, compte b, session du relais. Sa consigne : *« fixer TOUS les
> problèmes de Glucose et continuer son développement de manière intelligente et réfléchie »*. Sa
> note du soir (`text-kpll-375`, sa capture `img-paste-kpll-374`, membrane « Groupe » de `membrane-299`) :
> *« trop chelou, j'ai l'impression que lorsqu'on sélectionne une flèche attachée à un bloc texte MAIS
> qui est sélectionnée seule avec un autre bloc texte, eh bien on peut faire bouger une espèce de zone
> de clic invisible dans le canva, et ensuite faire en sorte que la flèche soit impossible à
> sélectionner »*.

## 1. Ce qui se passait

**Reproduit au pilote** (`~/.cache/glucose-bancs/fleche-zone-134/`, images 4 à 8) : A → B reliés, C
dessous ; la flèche choisie, `Maj`+clic sur C (le pilote sait désormais `maj-clic` : Maj tenu par le
clavier virtuel le temps du clic), puis C glissé de (+400, −300) pixels. La flèche reste dessinée de A
à B, mais **le cadre du groupe part avec C** — jusqu'au centre de B déplacé, hors de l'écran (image 5).
Déjà avant le glisser, le cadre allait jusqu'au **centre** de B et non au bout dessiné (image 4).

**La cause, lue dans le code.** Une flèche garde `(x, y, x2, y2)`. Pour un bout accroché, ce n'est
qu'une **copie** de la place de son nœud : le dessin et la visée la ramènent sur le nœud
(`path_with`, ARROW-1), mais **sept lecteurs lisaient la copie** : l'index spatial (`annotation_bbox` —
donc le clic, qui ne regarde que ce que l'index trouve près du curseur, et le culling), le cadre du
groupe (la « zone invisible » qu'on attrape), la sélection élastique (`pris_par`), `F` (`emprise`,
`selection_bounds`), `Alt+F` et l'export (`content_bounds`), la zone redessinée d'un glisser
(`Emport::encre`) et la boîte dite aux IA (`decrire_le_tableau`). Trois gestes laissaient la copie
loin du dessin :

* **la flèche choisie avec un autre nœud, puis glissée** : `translation_de_la_selection` la translate
  en entier — la copie part, les nœuds restent ;
* **le rangement dans un dossier** (RANGER-1, fiche 134 § 10) : `vers_le_dossier` change le nœud visé,
  pas la copie, restée là où était le nœud parti — **sa capture** : la flèche de « archive », le cadre
  sur le vide à droite ;
* **un dossier déplacé** : `end_follows` ne regarde que photos et annotations, jamais les dossiers ;
  et un texte qui grandit beaucoup (EDITION-LONGUE) déplace son bord sans toucher la copie.

Quand la copie sort des cellules (1 000 unités) où la flèche est dessinée, l'index ne la propose plus
au clic : **la flèche ne se choisit plus**, et le culling peut cesser de la dessiner.

## 2. La recherche

* **tldraw déduit** : les bornes d'une flèche liée se calculent depuis ses liaisons, et son index
  spatial (`SpatialIndexManager`, un R-tree par page) re-vérifie les flèches dont les bornes dépendent
  d'autres formes ; une épreuve y vérifie que les bornes d'une flèche se rafraîchissent quand le contour
  de la forme liée change sans que sa boîte change
  ([SpatialIndexManager.ts](https://cdn.jsdelivr.net/npm/@tldraw/editor@4.5.9/src/lib/editor/managers/SpatialIndexManager/SpatialIndexManager.ts),
  [spatialIndex.test.ts](https://cdn.jsdelivr.net/npm/tldraw@5.1.1/src/test/spatialIndex.test.ts)).
* **Excalidraw tient une copie à jour** (`updateBoundElements`) et en paie une famille de bogues :
  flèches liées non mises à jour à la rotation de plusieurs éléments (#6662), bornes en cache périmées
  après l'édition d'une étiquette (#6893)
  ([changelog](https://unpkg.com/@peakpay/excalidraw@1.0.0-ad9a6b6/CHANGELOG.md),
  [5191cdb](https://gitea.maison43.duckdns.org/gilles/excalidraw/src/commit/5191cdbe267cea32e3189d2900a2d84bb24effaa)).

Corriger les trois gestes un par un, c'était la voie d'Excalidraw : trois rustines, et la quatrième à
venir (un texte qui grandit). **MÉTHODE § 4, principe 15 : une seule source pour un état.**

## 3. ANCRE-1 — tout se lit sur le tracé ancré

* `arrow/enveloppe.rs` : `enveloppe_with` (l'enveloppe du tracé ancré), `encre_vue` (l'encre d'une
  annotation, une flèche par son tracé), `boites_du_tableau` (un parcours pour toutes les flèches
  d'un coup, au lieu d'un par bout). Branchée dans les sept lecteurs.
* **L'index** (`quadtree/boites.rs`) : une flèche accrochée **garde sa plage** pendant la passe (ses
  nœuds n'y sont pas tous encore), puis `recaler_les_fleches` la pose sur les boîtes de ses nœuds et
  ses coudes — l'enveloppe de son tracé droit y tient. Les nœuds de ses bouts sont retenus d'une passe
  à l'autre et relus **par leur nom** : **aucun hachage, aucune écriture** quand rien ne change (SPAT-1
  et SPAT-3 tenus, éprouvés). `quadtree.rs` passait 600 lignes : ses boîtes du modèle sont parties
  dans `boites.rs`.
* La copie reste ce qu'elle est : la place d'un bout **libre**, et le repli d'un bout dont le nœud a
  disparu.

**Choix pris seul, à contester** : la translation d'une flèche choisie reste ce qu'elle était (ses
coudes suivent la sélection) ; la copie de ses bouts accrochés part encore avec elle, mais plus rien ne
la lit.

## 4. Prouvé

* **Quatre épreuves** (`arrow/enveloppe_tests.rs`, `ancre_1`), chacune avec sa preuve à l'envers (la
  scène exerce le cas : la copie est loin du trait, et l'index par la copie ne la trouvait plus) : son
  geste (le clic par l'index, le cadre du groupe, la zone redessinée) ; le rangement dans un dossier
  (le clic, l'élastique, `F`, `Alt+F`, la boîte dite à l'IA) ; un coude ; le coût de l'index.
* **LIEN-4 reposait sur la copie** : sa scène posait la copie de chaque flèche en (0, 0) — les treize
  liens de « z », à un million d'unités, comptaient « à l'écran » par elle. Un coude les fait passer par
  l'écran ; « z » reste loin (`presence_tests.rs`).
* Clippy strict à zéro ; la suite entière avant LIEN-4 corrigée : 3 564 sur 3 569 (les quatre LIEN-4
  et QUEUE-1, qui passe seule — elle tombe parfois sous charge) ; LIEN-4 et QUEUE-1 vertes ensuite.
* **Au pilote** avec le correctif (images 7 et 8) : après le même glisser, le cadre tient C et la flèche
  telle qu'on la voit ; recliquée, elle se choisit.
* **Sabotages** : `outils/sabotages/ancre-135.json`, **10 sur 10** (§ 5).

## 5. Les sabotages

`ancre-135.json`, joué dans l'arbre détaché avec son témoin : **9 sur 10 au premier passage** — « une
flèche accrochée reprend sa plage de copie à chaque passe » **passait** : dans la scène de l'épreuve du
coût, la copie et le tracé couvraient les mêmes cellules, donc relire la copie n'écrivait rien de plus.
L'épreuve fait d'abord son geste (la copie envoyée loin), puis exige deux passes sans écriture
(`046a5ca`) : rejoué, il **tombe**. **10 sur 10.**

## 6. Reste

* **Son écran** : refaire son geste dans son Glucose relancé.
* Une flèche **sans coude qui contourne** (FLECHE-5) peut sortir de la boîte de ses deux nœuds ; une
  **courbe** peut bomber au-delà de ses points : l'index ne la propose alors pas au clic sur ce morceau.
  Vrai avant ANCRE-1 aussi ; l'itinéraire vit dans le rendu, l'index dans le noyau.
* L'arbitre appelé **sans** mesureur de nœuds (`noeuds: None`) sur une tranche filtrée perd les nœuds
  loin du curseur et retombe sur la copie : seul `Store::emprise_du_groupe` l'appelle ainsi, sur le
  tableau entier ; le vrai clic passe toujours le sien.
