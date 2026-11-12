# 108 — Le lag de loin divisé par deux, la Time Machine socle de l'IA

> Session du 12/11/2026, à partir de 8 h 11. La passation (fiche 107) laissait un correctif des
> flèches non commité, le lag de loin à 15-20 ms par image, une page des flèches en attente, et la
> méta-compréhension à concevoir avec lui. Envoyé : `9ef8c54` (les flèches), `d623daa` (les titres),
> `2efa8de` (les mémoires) ; la suite plus bas.

## 1. Les flèches sans ornement (FLECHE-1)

Le correctif d'hier (`porte_des_ornements`) : sur la voie graphique, une flèche sans poignée,
sans étiquette ni badge ne recalcule plus son tracé au processeur. **Une épreuve manquait** : celle
de la fonction seule ne voyait pas un appel qui lui passerait la mauvaise chose, et le témoin des
deux voies non plus (sa flèche choisie porte un badge). L'épreuve neuve passe par la couche du
dessus avec une flèche nue — choisie, ses poignées ; en saisie, sa pastille. **6 sabotages sur 6**
(`outils/sabotages/fleches-108.json`).

## 2. Les titres des membranes par la carte graphique (TITRE-3)

Le profil d'hier : 26 % d'une image de loin à redessiner les titres lettre à lettre au processeur,
dans la couche du dessous, puis 9 % à renvoyer cette couche. Un titre est désormais un **composant**
(COMPOSANT-1, la loi des blocs de texte) :

* rendu une fois par **le dessin même** du processeur, posé juste après les frontières, sous les
  photos ; à l'arrêt les mêmes pixels (au bit près pour des lettres posées ; pour des lettres
  tracées, au-delà de 128 pixels, l'écart de tiny-skia sous une translation entière, seulement aux
  bords) ; en mouvement au palier, la plume à sa place ;
* sa texture contient toute son encre (`Typography::encre_detouree` : les lettres, le détourage, la
  cellule et la phase) ;
* son **ajustement à la largeur se mesure dans le monde** : mesuré à l'écran, il variait d'un zoom
  à l'autre (de loin, une lettre de moins d'un pixel se mesure comme d'un pixel), et la texture se
  serait refaite à chaque image d'un zoom ;
* **une seule règle** (`titre_par_la_carte`, de la vue et de la fenêtre seules) laisse au processeur
  le titre qu'on écrit et celui dont la texture dépasserait la fenêtre ; la couche du dessous et la
  récolte la lisent pareil — un titre se dessine une fois, jamais zéro ni deux ;
* au processeur, **tous les titres, puis toutes les poignées et réglettes** (une affordance reste
  attrapable) ; le titre qu'on écrit en dernier. Le seul écart d'ordre entre les deux voies : un
  titre plus grand que la fenêtre qui en chevauche un autre.

**Regardé** sur sa carte (quatre vues, par la carte graphique et par le processeur) : au processeur,
l'ancienne et la neuve donnent la même image au pixel près ; entre les deux voies, l'écart passe de
5 500 à 3 107 pixels — les petits titres, qui différaient nettement entre les voies, n'y sont plus.

**Une épreuve aveugle trouvée par son sabotage** : celle de la clé d'un titre pendant un zoom
comparait ×0,004 et ×0,0042 — sous le plus petit zoom (`Viewport::MIN_SCALE`, 0,005), donc deux fois
la même vue. Elle regarde maintenant ×0,03 et ×0,033 et vérifie qu'elle voit deux vues. **9 sabotages
sur 9** (`outils/sabotages/titres-108.json`).

## 3. Deux mémoires pures

Le profil de la version aux titres : chaque titre mesuré lettre à lettre quatre fois par image
(13 %), le dégradé de chacune des 1 700 flèches remélangé dans Oklch à chaque image (8 %). Deux
fonctions de leurs entrées seules, retenues d'une image à l'autre, et oubliées quand une image ne
les demande pas (la loi des lettres et des mises en page) : `Typography::ligne_detouree`,
`SensDesTextes::degrade`. **5 sabotages sur 5** (`outils/sabotages/memoires-108.json`).

## 4. Le lag, mesuré

Sa carte de Wikipédia, par la carte graphique, hors écran (sway sans écran), en paires entrelacées ;
le **travail par image**, hors des attentes de l'écran (`tempo`, `acquerir`) :

| | son Glucose installé (`5e48642`) | + titres (TITRE-3) | + mémoires |
|---|---|---|---|
| de loin, la souris (médiane / p99) | 16,3 / 21,3 ms | 8,6-9,1 / 11-13 ms | **7,5-7,9 / 11,1 ms** |
| de loin, déplacer | 17,4 / 20,9 ms | 9,2-9,7 / 13 ms | **7,5-7,8 / 11,5-13 ms** |
| de loin, zoomer | 19,0 ms | 18,6-20,0 ms | 19,1-19,5 ms |
| au fond d'« Eau », déplacer | 4,6 ms | 3,0 ms | **2,8-2,9 ms** |

**Le zoom n'a pas gagné** : pendant un zoom à la molette, chaque cran refait la texture de tous les
blocs et titres (le cran se montre au palier, puis l'arrêt les refait exacts), et la cascade y passe
son budget (`textures` 12-13 ms). C'est la prochaine piste du lag.

Ce qui reste de loin (8,4 ms en moyenne) : le tracé des flèches pour la carte graphique (2,5 ms),
puis des postes de moins d'une milliseconde.

## 5. Le cercle de l'IA — sa réponse, et la Time Machine pour socle

La page de conception (<https://claude.ai/artifact/8WJTkJLvv7HvDDrJMng7CL>) proposait six choix ; sa
réponse (VISION § 8 bis) : tous acceptés, l'apprentissage fédéré plus tôt que je ne le proposais, et
une correction de fond — **on ne fait pas un récit « comme » la Time Machine, on utilise la Time
Machine** ; un geste peut porter **un écrit** (ce qu'une IA en comprend, et pourquoi il compte),
« comme des push GitHub ». Le fonctionnement d'abord ; les couleurs et l'interface de la Time
Machine ensuite. Ses flèches : l'IA ne les juge pas, elle **comprend leur importance**.

Envoyé : `b2a71c9` (la Time Machine), `1c3c754` et `365c9a8` (les flèches).

* **PERSIST-5 — des entrées facultatives.** Une nature d'entrée à bit haut ne change pas l'état du
  document : une version qui ne la connaît pas l'enjambe (les morceaux ancillaires du PNG). **Le
  prix, vérifié dans le code de la beta 10** : les versions publiées refusent un document qui en
  porte une, avec un message net (« ce document vient d'une version plus récente de Glucose »), et
  la reprise laisse le brouillon où il est — rien ne se perd, mais un document qu'un agent a touché,
  ou qui porte un écrit, ne s'ouvre plus sur son téléphone (beta 4) ni au PC de l'école (beta 9)
  avant leur mise à jour. Un document que seul l'humain a touché n'en porte aucune.
* **MAIN-1 — chaque geste signé.** Chaque transaction porte sa main : l'humain, ou un agent sous le
  nom que son client se donne (déclaré, jamais vérifié, comme la question « Autoriser » le montre).
  Le scribe signe chaque geste et déclare un agent une fois par document (AUTEUR) ; défaire est un
  geste de l'humain. Le terminal signe « le terminal » : un script et une commande tapée à la main
  y sont la même chose, un programme qui parle par la porte de Glucose.
* **La provenance des nœuds** (`Journal::des_agents`, `suivre_les_mains`) : ce qu'un agent a posé
  et que l'humain n'a pas touché depuis. Un geste de l'humain le reprend ; un agent qui retouche ce
  que l'humain a fait ne le lui prend pas. Relue dans l'histoire à l'ouverture (`Ouvert::des_agents`)
  seulement quand un agent y est déclaré : un document de l'humain seul ne relit rien de plus. Les
  annotations seulement (pas les photos) ; pendant qu'on regarde le passé dans la Time Machine, le
  journal est neuf et aucune flèche n'est jugée — le retour au présent rend le journal entier.
* **NOTE-1 — un écrit attaché à un geste**, par son rang, sans le toucher (comme une note de git).
  `glucose_noter` l'écrit ; `glucose_histoire` relit l'histoire — chaque geste, son auteur, ce qu'il
  a fait en mots (`Transaction::en_mots`), ses écrits. La Time Machine montre en aperçu qui a fait
  le geste regardé et ses écrits, une ligne chacun : son interface viendra après le fonctionnement
  (sa règle).
* **PARENTE-4 dans `main`** : le prototype de la fiche 107, branché sur MAIN-1 — **seules se jugent
  les flèches qu'un agent ou un script a posées et que l'humain n'a pas touchées** ; la collecte des
  liens est passée au noyau (`sens::liens::liens_du_tableau`). Les cinq épreuves de LIEN-4 tombées
  hier (des flèches de l'humain entre des textes identiques) passent de nouveau sans être réécrites :
  ses flèches ne sont plus jugées.
* **Prouvé** : 3 152 épreuves, clippy et fmt à zéro ; **16 sabotages sur 16**
  (`outils/sabotages/provenance-108.json`) — dont **trois épreuves aveugles** trouvées et écrites :
  un document chargé qui garderait la provenance de l'autre (un agent nomme ses nœuds lui-même,
  « p1 » peut revenir), une membrane qui ne compterait pas les textes de ses filles, la souris qui
  redessinerait pour un lien d'habitude. L'épreuve de l'agent rouvrait son document pendant que la
  première fenêtre le tenait (un seul scribe par fichier) : elle la ferme d'abord.

## 6. La recherche (consultée le 12/11/2026)

* Les étiquettes à halo des cartes géographiques : MapLibre et Mapbox, glyphes en champ de distance
  signée (SDF), le halo par un seuil (<https://maplibre.org/maplibre-style-spec/glyphs>) ; Patel,
  *Guide to SDF+MSDF Fonts* (Red Blob Games, 2026, <https://www.redblobgames.com/articles/sdf-fonts/>) :
  des coins arrondis, une largeur de halo bornée par l'étendue du champ. Gardé : la texture (COMPOSANT-1),
  qui rend **les mêmes pixels** que le processeur ; un halo SDF aurait changé le dessin.
* Slug (Lengyel, JCGT 2017 ; *A Decade of Slug*, <https://terathon.com/blog/decade-slug.html>) : le
  rendu exact des lettres sur la carte, déjà employé de près ; il ne sait pas détourer.
* WebRender (Firefox), le *picture caching* : ce qui ne change pas se garde en textures, invalidé par
  ce qui le touche (<https://searchfox.org/firefox-main/source/gfx/wr/webrender/src/tile_cache/mod.rs>).
* L'IA qui aide et que l'humain corrige : Amershi et al., *Guidelines for Human-AI Interaction*
  (CHI 2019, <https://www.microsoft.com/en-us/research/publication/guidelines-for-human-ai-interaction/>) —
  G9 corriger efficacement, G13 apprendre du comportement, G14 s'adapter prudemment, G15 un retour
  fin, G18 prévenir des changements ; Kulesza et al., *Principles of Explanatory Debugging*
  (IUI 2015) ; FaLA (EMNLP 2023 Findings) ; McMahan et al. (2017) et Hard et al. (2018) pour
  l'apprentissage fédéré (<https://arxiv.org/abs/1602.05629>, <https://arxiv.org/abs/1811.03604>).
* Shipman et McCall, *Supporting Knowledge-Base Evolution with Incremental Formalization* (CHI 1994,
  <https://people.engr.tamu.edu/shipman/chi94-hos/chi94hos_abstract.html>) : l'humain pose librement,
  le système propose une structure, la forme libre d'origine se garde.
* La provenance : W3C, *PROV Primer* (2013, <https://www.w3.org/TR/2013/NOTE-prov-primer-20130430/>) —
  un objet « attribué » à un agent, humain ou logiciel.
* Les entrées facultatives : PNG, les morceaux « ancillaires » qu'un décodeur « peut ignorer sans
  risque » (<https://www.w3.org/TR/png-3/#5Chunk-naming-conventions>) ; les écrits attachés sans
  toucher : *git notes* (<https://git-scm.com/docs/git-notes>).

## 7. Les leçons

* **Deux cas qu'une épreuve croit distincts, et qui ne le sont pas** — deux fois dans la journée :
  la clé d'un titre comparée à ×0,004 et ×0,0042, sous le plus petit zoom (`Viewport::MIN_SCALE`,
  0,005), donc la même vue ; l'étoile des épreuves de PARENTE-4, où le lien `k` part dans la
  direction `k × 0,4833` — or 13 × 0,4833 ≈ 2π, et les liens `k`, `k + 13`, `k + 26` tombent sur le
  même rayon : un lien non jugé y cachait la différence, et j'ai d'abord accusé la loi. Une épreuve
  qui compare vérifie qu'elle voit deux cas ; mesurer avant de supposer.
* **Relire la signature avant de conclure** : une trace de débogage passait des coordonnées
  d'écran à une fonction qui attend le monde.
* **Un remplacement par script échoue en silence quand `cargo fmt` a changé le texte** : relire le
  fichier, puis remplacer (déjà dans MÉTHODE § 5 ; payé encore deux fois). Le dernier, d'une ligne,
  s'est fait à la main.
* **`git apply --3way` met le correctif dans l'index** : pour ne changer que le message d'un commit,
  `git commit --amend --only`, sinon l'index entre dans le commit.
* **Le profileur cherche `pgrep -x glucose-desktop`** : une copie du programme à profiler doit
  porter ce nom.
* **Une durée annoncée se mesure aussi** : j'ai dit « 45 minutes à une heure » pour seize sabotages,
  il en a fallu six — seule la partie touchée se recompile à chaque sabotage.

