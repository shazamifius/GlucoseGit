# 106 — Le survol, la petite IA du sens, le halo qui dit la parenté

> Session du 11/11/2026, en deux temps : l'après-midi (de 15 h à 18 h 05, coupée net, sans fiche) et
> le soir (de 19 h 33 à 21 h 28). La passation (fiche 105) laissait les membranes en carte écrites ;
> ses mots du matin (VISION § 5) demandaient un halo qui dise la parenté des textes. Envoyé :
> `93987a9` (le survol), `3622d13` (le halo), `01f450c` (un agent lit la parenté), `e6cbb9d` (les
> épreuves des quatre sabotages aveugles, écrites ce soir-là et rejouées par la session suivante, fiche
> 107) et cette fiche ; son Glucose mis à jour.

## 1. L'après-midi : le survol (SURVOL-1, SURVOL-2)

Sur la carte de Wikipédia, suivre la souris coûtait 8,2 ms (médiane) et 9,7 ms (99ᵉ centile) par
image : chaque mouvement parcourait tout le tableau en hachant chaque identifiant.

* **SURVOL-1** : les nœuds proches du curseur se lisent par leur **rang dans l'index**, vérifié ; des
  tranches qui ne sont pas celles du tableau indexé reprennent la recherche par le nom. 4,9 / 4,9 ms.
* **SURVOL-2** : de loin (la lecture sous le plancher de 8 %), aucune présence ne change le trait d'un
  lien hors lumière ; elle ne se calcule plus, ni au dessin ni à la souris.
* **La bande où l'on saisit une membrane par son nom** est celle où il est dessiné (18 × √(A/A₀)) : la
  visée en prêtait 30. L'épreuve d'équivalence l'a trouvé.

## 2. Ses réponses, et ses mots sur l'IA de Glucose

Devant la page du halo de l'après-midi : **par familles** ; le halo à **76 au repos, 102 au survol** ;
l'**opacité dit la sûreté de l'IA**, la **clarté l'accessibilité** (*« parfaitement d'accord »*) ; la
couleur qu'on choisit soi-même : *« c'est là le truc le plus complexe »*. Puis la méta-compréhension
(VISION § 8 bis, ses mots). La réponse proposée le soir même : apprendre **chez lui**, sans récolter ;
le journal des rectifications comme une seconde colonne de l'histoire ; une seconde interface, le
récit lisible du canevas ; trois IA qui répondent à la même question. **À concevoir avec lui** — une
page de conception avant d'y toucher.

## 3. Ce que la remise en question a changé (le soir)

La session coupée laissait le modèle et ses familles écrits dans le noyau, sans appelant (le cliquet
3 l'avait arrêtée), et la sûreté d'un texte définie comme le centile de son plus proche parent.

* **La sûreté ne disait rien.** Mesurée sur la carte (352 textes, 71 familles, 69 textes seuls) : de
  0,97 à 1 pour les membres, de 0,72 à 0,99 pour les seuls — en opacité, tous les halos auraient eu la
  même force. Chaque texte est comparé à 351 autres : le meilleur de 351 tirages du hasard est
  toujours « bon ». Quatre essais, chacun mesuré sur la carte et sur du hasard pur :
  | | membres (médiane) | seuls | hasard pur |
  |---|---|---|---|
  | le centile du plus proche parent (la session coupée) | 0,997 | 0,968 | — |
  | Šidák, `F(r)ⁿ⁻¹`, le hasard lu sur toutes les paires | 0,35 | 0,000 | uniforme |
  | Šidák, le hasard **robuste** (médiane, écart absolu médian) | 0,96 | 0,03 | uniforme |
  | Benjamini–Hochberg sur ces chances | 0,90 | 0 (jusqu'à 0,62) | ~0 dès 60 textes ; mais 0,88 pour du hasard noyé parmi 15 vraies parentés |
  | **Efron, densité de Grenander, part du hasard à 1** (écrit) | **0,71** | **0 (tous)** | ~0 ; au plus un texte extrême (Daniels) |

  Le hasard **moyen** était pollué par les vraies parentés : les estimateurs robustes ne le sont pas.
  Une chance corrigée du nombre de comparaisons est uniforme sous le pur hasard : la peindre ferait
  paraître à moitié sûrs la moitié des textes d'un document sans parenté. Benjamini–Hochberg dit une
  part **d'ensemble**, dont hérite un texte limite. Le taux local de fausses découvertes dit le
  texte : au-dessus de 0,5, son plus proche parent est juste à 97 % sur la carte.
* **Le seuil du sûr** reste « deux écarts » (la convention), mais au-dessus du hasard robuste.
* **J'ai éteint la lueur des textes seuls, et l'image a montré la faute.** Le papier d'un texte est
  presque le noir du canevas : c'est la lueur qui dessine le bord du bloc (« le texte sur fond noir,
  le contour en lueur »). La page qu'il avait approuvée disait d'ailleurs *« un halo pâle »*, pas
  « aucun ». La loi écrite : **chaque bloc garde la lueur de Tauri (38), en gris** ; la sûreté la monte
  jusqu'à 76 et monte sa vivacité du gris à la couleur de sa famille. Un texte dont l'IA n'est pas
  sûre reste gris — *« pas de liaison de couleur »*.
* **La teinte symbiotique est partie entièrement**, flèches comprises : la garder pour elles seules
  aurait donné des flèches d'une couleur et des halos d'une autre. Une flèche prend la couleur des
  lueurs qu'elle relie, grise vers une photo ou un bout dans le vide. Visible sur la carte : beaucoup de
  flèches grises — **à sa parole**. Le cache, ses dettes au repos (REPOS-2, fiche 98) et la marque de
  mesure « hues » sont partis avec elle (≈ 600 lignes).
* **L'export SVG peint les coins du canevas.** La lueur deux fois plus forte faisait monter l'écart des
  coins arrondis (Tauri, dans le SVG) contre les coins droits (le canevas, HALO-1) à 12,9 sur
  l'épreuve de superposition, aussi haut qu'une teinte faussée (13,3). Un export montre ce que le
  canevas montre : remesuré, 3,3 ; la borne de teinte passe à 8,3.
* **L'épreuve de l'épaisseur des flèches** (DPI-1) mesurait une colonne qui passait sur le point de
  la grille de l'origine, plus gros à 150 % : la flèche, grise, a moins d'encre, et le point faussait le
  rapport (1,534). Elle mesure maintenant entre deux points, contre la même vue la flèche ailleurs.
* **La mémoire** : la parenté compare chaque texte à tous les autres (6 octets par paire au plus fort) ;
  elle suit la règle d'ADAPT-1 — jamais plus que ce qu'elle laisse — et, au-delà, les halos restent
  gris et la chronique le dit.
* **Lu par une IA aussi** (PARENTE-3) : `glucose_lire` dit la famille et la sûreté de chaque texte,
  et seulement l'avis du tableau que l'IA de Glucose a lu.
* **Un modèle plus fort existe** : `potion-multilingual-128M` (Model2Vec, mai 2025, 101 langues) — pas
  mesuré, environ quatre fois plus lourd, un découpeur Unigram à réécrire : le candidat d'une IA
  « portable » (ses trois IA, VISION § 8 bis). Note de décision 15.

## 4. La recherche

Consultée en ligne le 11/11/2026 : la fiche du modèle
(<https://huggingface.co/sentence-transformers/static-similarity-mrl-multilingual-v1> : Apache 2.0,
50 langues, Matryoshka de 1 024 à 32), *Train 400x faster Static Embedding Models*
(<https://huggingface.co/blog/static-embeddings>), Model2Vec et `potion-multilingual-128M`
(<https://github.com/MinishLab/model2vec>).

Les références de statistique, classiques, citées de mémoire et non relues en ligne ce soir : Hampel
(1974) pour l'écart absolu médian ; Šidák (1967) ; Benjamini et Hochberg (1995) ; Efron et al. (2001)
pour le taux local de fausses découvertes, Efron (2004) pour le hasard « empirique » lu sur les
données ; Grenander (1956) pour la densité décroissante ; Langaas, Lindqvist et Ferkingstad (2005)
pour son emploi sur des chances ; Daniels (1945) pour la loi du plus extrême ; Murtagh (1983) pour la
chaîne des plus proches voisins. L'après-midi : Arora, Liang et Ma (2017) ; la sériation circulaire
spectrale ; *Semantic Color Mapping* (IEEE VIS 2022).

## 5. Ce qui est écrit

| invariant | où | ce qu'il dit |
|---|---|---|
| **SENS-1** | `core/sens.rs`, `sens/decoupeur.rs`, `sens/unicode.rs`, `donnees/` | le modèle embarqué (lu une fois), le découpeur à l'identique de la référence, le sens d'un texte, les textes d'un tableau |
| **PARENTE-1** | `core/sens/familles.rs` | le hasard robuste, le seuil du sûr, les familles, les teintes et leur orientation par le document, la sûreté (Šidák puis Efron) |
| **PARENTE-2** | `desktop/parente.rs`, `renderer/sens.rs`, `renderer/halo.rs` | le fil, l'empreinte des textes, la règle de la mémoire ; la grammaire des couleurs ; la lueur du bloc et celle de la sûreté |
| **PARENTE-3** | `agents/executer.rs` (`parente_en_json`) | un agent lit la famille et la sûreté |
| **COULEUR-4** | `renderer/sens.rs` (`lueur`) | la couleur choisie va à la lueur, sûre |
| le cercle | `core/couleur/ecran.rs` (`cercle_des_teintes`) | la plus forte vivacité que toutes les teintes atteignent à une clarté |
| les flèches | `core/arrow/aspect.rs`, `renderer/arrow.rs` | deux couleurs, leur milieu dans Oklch |

## 6. Prouver

* **La suite** : 3 122 épreuves vertes (avec celles des quatre aveugles) ; clippy strict et `cargo fmt` à
  zéro.
* **Les sabotages** (`outils/sabotages/sens-106.json`) : 36 ; 32 tombés au premier tour, **4
  aveugles**, aucun ne cachant un défaut du programme — chaque fois, l'épreuve n'exerçait pas le cas où
  la loi et sa version cassée divergent : un nœud remplacé ailleurs qu'à sa place (SURVOL-1) ; aucun
  mot qui bute au milieu (SENS-1 : la référence rend « [UNK] » entier) ; des familles posées dans
  l'ordre même qu'on cherchait (l'anneau, PARENTE-1) ; l'opacité éprouvée avec une famille, dont la
  vivacité suit aussi la sûreté (PARENTE-2). Réécrites (`e6cbb9d`), les quatre tombent : **36 sur 36**.
* **Regardé** : les quatre vues de la carte (avant, son Glucose installé ; après), par la carte
  graphique ; les cinq images témoins, avant et après, pixel par pixel — c'est là que le bloc perdu
  s'est vu. Les empreintes témoins mises à jour après les avoir regardées.
* **De bout en bout** : cinq textes de musique, quatre de l'eau, neuf sans rapport, par
  l'application et son image : deux familles, deux teintes ; les seuls gris.

## 7. Ce qui attend sa parole

* **La page** : <https://claude.ai/artifact/LXZ89P8vZUiJ1bsqtvEzei> — les quatre vues avant/après, et trois questions : les flèches grises ; la force
  des familles (« Eau, de près ») ; sa couleur et ce que l'IA en apprend (la méta-compréhension, à
  concevoir avec lui).
* **Les petits documents** : sous une quinzaine de textes, l'IA hésite (sûreté au plus ~0,3) ; une
  calibration hors du document (le hasard du modèle, mesuré une fois) le réglerait — à voir à l'usage.
* **La clarté = l'accessibilité** : approuvée, pas encore lue par l'IA ; la clarté ne varie pas.

## 8. Les leçons

* **Mesurer la grandeur qu'on va peindre, sur la vraie carte, avant de l'écrire** : la sûreté écrite
  l'après-midi passait ses épreuves et ne disait rien à l'œil (0,97 à 1 pour tous).
* **Une statistique « contre le hasard » a besoin d'un hasard qui ne contient pas ce qu'on cherche** :
  la moyenne des paires d'un document est faite en partie de ses vraies parentés.
* **Une chance n'est pas une sûreté** : sous le pur hasard, elle est uniforme. Et une part d'ensemble
  n'est pas celle d'un seul.
* **Une image témoin se regarde avant de mettre son empreinte à jour** : l'empreinte disait qu'un rendu
  avait changé, pas qu'il était faux ; seule l'image a montré les blocs disparus.
* **Relire la phrase approuvée** : *« un halo pâle »* n'est pas *« aucun halo »*.
* **Une épreuve qui tombe pour une raison voisine dit souvent un vrai défaut de l'épreuve** : la
  colonne de DPI-1 passait sur un point de la grille depuis le début.
