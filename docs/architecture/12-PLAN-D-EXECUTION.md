# 12 — Plan d'exécution : tout ce qui reste, dans l'ordre où je veux le faire

> **Rôle de ce document.** La fiche [`11`](11-PLAN-DE-MARCHE.md) a posé l'ordre de marche
> A → G. Les étapes A et B sont faites. Ce document dit **ce que je fais ensuite, et pourquoi
> ce n'est pas l'étape C.** Il remplace le § 4 de la fiche 11 à partir de l'étape C ; tout le
> reste de la fiche 11 (les remises en question RQ-0 à RQ-6) reste valable et n'est pas rejoué
> ici.
>
> **Date** : 2026-09-12 · mesures prises sur l'arbre de travail au commit `acceb0d`.
> **État vérifié** : `cargo test --workspace` exit 0, **641 tests verts**, 0 warning.

---

## 1. Ce que « continuer après la phase B » voulait dire

La fiche 11 § 4 est explicite. Après B vient :

| Étape | Contenu | Sortie annoncée |
|---|---|---|
| **C** | Cache pyramidal de tuiles, invalidation exacte, couche d'arêtes longues, présentation GPU, non-régression visuelle au pixel | ≥ 100 fps en 4K sur 10⁷ nœuds |
| D | Les fonctionnalités de navigation — domaines, couleur, membranes, recherche, les modules morts branchés | retrouver une information en < 30 s sans connaître sa position |
| E | Le texte — mise en page, Markdown, maths, sous-pixel | texte net à tout zoom, prouvé par capture |
| F | Persistance, glisser-déposer universel, jonction `glucose-brain` | 10⁷ nœuds ouverts en < 1 s |
| G | Le reste : flèches, dossiers, miroirs, temporalité, storyboard, export, rideaux, collaboration | — |

**Donc : littéralement, « après la phase B » = construire le moteur de rendu par tuiles et le
porter sur le GPU.**

---

## 2. Pourquoi je ne fais pas ça, et pourquoi je pense avoir raison

Trois arguments, du plus faible au plus fort.

### 2.1 La charte le dit déjà, et je ne l'avais pas appliquée

Mot pour mot, dans la charte permanente du projet :

> **Le blocage actuel n'est pas la performance, c'est le manque de fonctionnalités** : Glucose
> n'en a pas encore assez pour rendre une telle carte navigable. À garder en tête quand on
> priorise.

La fiche 11 a été écrite **avec** cette charte sous les yeux et a quand même placé le rendu avant
les fonctionnalités. C'est une contradiction que personne n'a relevée — moi le premier.

Cet argument seul ne suffirait pas : une charte peut se tromper. Les deux suivants sont
techniques.

### 2.2 Un cache de tuiles se conçoit quand on sait ce qu'il y a dans une tuile

C'est l'argument décisif, et il est structurel.

L'étape C.2 — l'invalidation exacte — demande de savoir, pour chaque nœud modifié, **exactement**
quelles tuiles il salit, à tous les paliers. Cette relation dépend entièrement de ce qu'un nœud
sait dessiner :

| Ce qui arrive plus tard | Ce que ça change dans une tuile |
|---|---|
| Gras, italique, code inline | la largeur d'une ligne cesse d'être une somme uniforme → la boîte d'une carte change |
| LaTeX | un bloc de formule a sa propre métrique, et peut déborder sa ligne |
| Flèches dessinables, courbes, waypoints | un trait traverse **n** tuiles au lieu d'une ; l'invalidation devient une requête de segment |
| Étiquette de flèche | un texte flottant qui n'appartient à aucune boîte |
| Membranes étirées | une boîte qui change de taille **quand son contenu bouge** — invalidation en cascade |
| Rideaux | un board entier composé par-dessus un autre |
| Vidéos | une tuile qui se salit toute seule, 30 fois par seconde |

Chacune de ces lignes modifie la clé du cache, l'unité d'invalidation, ou les deux. Construire C
maintenant, c'est **garantir de le reconstruire** — et l'étape C est justement celle où la fiche
11 elle-même écrit qu'« une erreur devient un artefact permanent ».

L'ordre juste est l'inverse : **on sait d'abord dessiner, ensuite on optimise le dessin.**

### 2.3 La fondation de l'étape B ne porte encore rien

L'arène existe : `NodeId(u32)`, SoA, 22 octets par nœud, index CSR, 423,8 Mo pour 10⁷ nœuds.
Elle est écrite, testée, et **elle ne contient aucun document réel** — le modèle vivant reste
`Vec<Annotation>`.

Bâtir le cache de tuiles au-dessus du modèle actuel, puis glisser l'arène dessous, c'est écrire
l'invalidation deux fois. La substitution doit passer **avant** C, jamais après.

### 2.4 Ce que l'argument ne dit pas, et qu'il faut couvrir

La contre-objection sérieuse : *si le rendu ne tient pas les 100 fps, tout ce qu'on empile
au-dessus est à refaire.*

Elle ne tient pas, pour une raison précise : **les fonctionnalités vivent dans `glucose-core`,
qui ne dessine rien.** La mise en page d'une carte, la géométrie d'une membrane, l'ancrage d'une
flèche sont des fonctions pures, testées sans écran. Changer de rastériseur ne les touche pas.

Mais elle contient une vraie exigence, que j'adopte :

> **La performance cesse d'être une étape et devient une contrainte permanente mesurée.**
> Le banc A.5 — jamais construit — est le tout premier travail de ce plan, et il tourne à chaque
> chantier. Si une fonctionnalité coûte plus de 10 % du budget de frame, on le sait le jour où
> elle est écrite, pas deux ans plus tard.

C'est plus sûr que l'étape C, pas moins : une mesure continue attrape une régression, une étape
ponctuelle attrape un instant.

---

## 3. La règle qui rend cet ordre possible

Le risque de repousser la substitution de l'arène est réel : chaque fonctionnalité écrite d'ici
là est une fonctionnalité à migrer. Une seule règle le désamorce.

> **RÈGLE S — Toute fonctionnalité passe par l'API du `Store`, jamais par les champs du modèle.**
>
> Un module qui écrit `board.annotations.push(...)` est couplé à la représentation. Un module qui
> appelle `store.add_annotation(...)` est couplé à un **contrat**, et survit intact à la
> substitution.

Elle se mesure, donc elle se tient :

| Où | Accès directs aujourd'hui | Cible |
|---|---:|---|
| `glucose-desktop/src` | **66** | ne jamais augmenter ; descendre à 0 |
| `glucose-core/src/store*` | 209 | c'est **là** que doit vivre la connaissance de la représentation |
| `glucose-core/src` hors store | 72 | descendre à 0 |

Un test de CI compte ces accès et échoue si le chiffre monte. C'est le même principe que A.3 et
A.4 : mécanique, pas disciplinaire.

Avec cette règle, la substitution devient une opération **locale au `store`** — 209 lignes à
réécrire dans un seul dossier, avec 641 tests comme filet — au lieu d'une réécriture du programme.

---

## 4. L'état mesuré aujourd'hui

Mesures prises à l'instant, pas héritées de la fiche 03.

### Les modules du noyau que personne n'appelle

| Module | l. | Ce qu'il débloque |
|---|---:|---|
| `membrane_space` | 782 | dépôt → adoption, mode minimisé, l'invariant MEMB-1 |
| `export` | 501 | SVG et Markdown, prêts, sans écriture disque |
| `timeline` | 406 | toute la temporalité |
| `membrane_stretch` | 234 | la membrane qui grandit avec son contenu |
| `text_anchors` | 224 | flèches attachées à une **sélection de texte** |
| `curtain_model` | 222 | géométrie des rideaux |
| `curtain_panel` | 204 | panneau de rideau |
| `arrow_anchor` | 177 | flèche ancrée au **bord** d'un nœud, pas à son centre |
| `mirror_graph` | 118 | détection de cycle (anti-inception) des miroirs |
| **Total** | **2 868** | **+ 7 768 lignes de suites de tests qui les couvrent déjà** |

À quoi s'ajoutent `arena` (2 100 l.) et `fixed` (240 l.), qui sont la fondation en attente de
substitution — morts par construction, pas par oubli.

### Ce que l'écran montre, comparé à Glucose Tauri

Lecture faite de la capture de production (`docs/architecture/screens/00_user_production_board.png`)
et des 45 345 lignes de TypeScript. Ce qui manque **visuellement**, dans l'ordre où l'œil le voit :

1. **Les poignées de sélection** — huit par élément dans Tauri, dessinées mais inertes en Rust.
2. **La barre d'action contextuelle** flottante en bas (« 11 sélectionnés · Verrouiller ·
   Supprimer ») — absente.
3. **Le panneau Ordonner** — 8 tris (dont *Couleur*, *Sombre → Clair*) × 5 dispositions, largeur
   cible et espacement réglables. Le Rust a un bouton et une grille.
4. **Le texte riche** — le paragraphe de la capture a un titre gras, un corps, une puce. Le Rust
   sait faire `#`, `##`, `- ` et le retour à la ligne ; il ne sait pas **gras, italique, code,
   citation, liste numérotée, lien, tableau, LaTeX**.
5. **Le compteur d'images** en haut à droite — absent.
6. **Le dock de panneaux** avec son rebond et son FLIP — les courbes existent depuis `acceb0d`,
   les gestes non.

---

## 5. Le plan, en cinq vagues

Chaque chantier porte : **ce qui existe déjà**, **ce qu'il faut écrire**, et **sa sortie
mesurable**. Une vague ne s'ouvre pas avant que la précédente soit verte.

### Vague 0 — Le banc permanent *(quelques heures, et tout le reste en dépend)*

| # | Travail |
|---|---|
| 0.1 | Banc de frame : 10³ / 10⁵ / 10⁶ nœuds × zoom 1 / 0,1 / 0,01, en 1080p **et 4K**, mesuré contre le budget de 10 ms. C'est A.5, jamais fait. |
| 0.2 | Capture PNG déterministe d'une scène de référence, comparée octet pour octet. Le filet de sécurité de tout le reste : il attrape une régression visuelle que 641 tests unitaires ne voient pas. |
| 0.3 | Test de CI comptant les accès directs au modèle (règle S) et refusant leur augmentation. |

> **Sortie** : un tableau de chiffres publié, et un build qui échoue au-delà de 10 % de
> régression. À partir de là, chaque chantier se mesure au lieu de s'affirmer.

---

### Ce que la vague 0 a trouvé, et que rien ne disait

Le banc a existé une journée et a rapporté quatre choses. Aucune n'était dans une fiche, et
deux vont contre ce que j'avais écrit.

#### 1. Le rendu n'était pas déterministe

Deux images de la même scène différaient d'environ **sept mille pixels**. `UiState::new` posait
un toast de bienvenue, donc un `Instant`, donc une opacité qui change à chaque image. Rien ne le
voyait parce que rien ne comparait jamais deux images.

Au-delà de la mesure, c'est une faute de conception : un constructeur d'état ne déclenche pas une
notification. Le mot d'accueil est désormais posé au démarrage de l'application.

#### 2. Vingt-trois mesures sur vingt-sept hors budget

| Définition | 1 000 nœuds | 10 000 | 100 000 |
|---|---:|---:|---:|
| 1080p | 2,45 ms | 17,3 ms | 96 ms |
| 1440p | 8,15 ms | 37,3 ms | 137 ms |
| 4K | **20,6 ms** | 27,2 ms | 108 ms |

La 4K dépasse dè **mille** nœuds. Ce chiffre met en difficulté l'argument du § 2 — celui qui dit
que les tuiles et le GPU peuvent attendre — et il faut le dire avant de l'expliquer.

#### 3. La décomposition déplace la conclusion

Le côt n'est pas là où le total le laissait croire.

| Étape | 1080p / 10 000 | 4K / 1 000 |
|---|---:|---:|
| interface (dont **minimap**) | **5,13 ms** | 5,56 ms |
| halos | 1,98 ms | 1,81 ms |
| grille de points | 0,73 ms | **4,35 ms** |
| effacement du fond | 0,74 ms | 3,45 ms |
| cartes, images, membranes | 0,32 ms | 1,52 ms |

Deux des trois premiers postes sont des **défauts algorithmiques**, pas des coûts de surface :

* **la minimap redessinait un rectangle par nœud, à chaque image**, dans une vignette de
  180 × 120 pixels où la plupart tombent les uns sur les autres. Corrigé par un cache dont la clé
  est le document et le cadrage : **5,13 → 0,89 ms** en 1080p, **5,56 → 0,64 ms** en 4K ;
* **la grille de points** construit un cercle par point — environ huit mille en 4K — puis les
  rasterise d'un trait. Elle reste à traiter (voir plus bas).

Seul l'effacement du fond est un coût de surface irréductible en CPU. **L'ordre du § 2 tient
donc, mais il n'était pas gratuit** : il se paie par des corrections précises, chiffrées, que
personne n'avait vues parce que personne ne mesurait.

Le banc relancé après la correction de la minimap, sur les mêmes documents :

| Une image, à dix mille nœuds, au zoom 1 | Avant | Après | |
|---|---:|---:|---|
| 1080p | 17,28 ms | **4,58 ms** | ×3,8 |
| 1440p | 37,28 ms | **10,89 ms** | ×3,4 |
| 4K | 27,19 ms | **17,58 ms** | ×1,5 |

Vingt-trois mesures hors budget sont devenues vingt et une, mais le compte brut dit mal ce qui
s'est passé : les cas de travail réel — un document de quelques milliers de nœuds, lu à l'échelle
1 — sont **tous** passés dans le budget en 1080p et 1440p. Ce qui reste dehors est le très fort
dézoom, où le culling ne sert plus puisque tout est visible, et la 4K, où la surface domine.

#### 4. Un test de la suite échouait au hasard

Un test de budget de halos chronomètre dans une suite paralèlle : il mesure la contention, pas le
code. Il a échoué pendant une suite complète puis passé trois fois de suite lancé seul. Un test
qui échoue au hasard ne dit plus rien et abîme la valeur des six cent quatre-vingt-dix autres :
il est marqué ignoré, avec la commande pour le lancer seul, et le banc prend le relais.

#### Ce qui reste à corriger, chiffré

| # | Défaut | Coût mesuré | État |
|---|---|---:|---|
| a | La minimap redessine tout, chaque image | 5,1 ms | **corrigé** |
| b | La grille construit un cercle par point | 4,35 ms en 4K | à faire |
| c | L'effacement du fond | 3,45 ms en 4K | irréductible en CPU — relève de la vague 4 |
| d | La touche `F` ne cadre rien (fiche 03 § 1.6) | — | l'API existe (`Store::content_bounds`), le raccourci ne reçoit pas la taille de la fenêtre |
| e | Le badge d'un dossier affiche toujours zéro | — | à faire |

---

### Vague 1 — Le geste quotidien

*C'est ce que l'utilisateur touche mille fois par jour, et c'est là que « ça ne ressemble pas à
Glucose » se décide.*

#### Chantier 1.A — Le texte, cœur de Glucose

| # | Travail | Déjà écrit |
|---|---|---|
| 1.A.1 | **Markdown inline** : gras, italique, barré, code. Le moteur ne lit aujourd'hui que les préfixes de ligne ; il lui faut une passe d'inline qui produise des **runs stylés**, et une mesure qui en tienne compte. | `export::strip_inline_markdown` sait déjà les reconnaître |
| 1.A.2 | **Blocs** : citations `>`, listes numérotées, blocs de code, séparateurs `---`, et `-# ` (petit texte, syntaxe maison) | la structure `LineKind` est prête à s'étendre |
| 1.A.3 | **L'éditeur avec prévisualisation** — ce que l'utilisateur a nommé en premier. Trois couches : symboles Markdown grisés, délimiteurs LaTeX **verts si la formule compile, rouges sinon**, et une fenêtre flottante à droite qui rend le résultat en direct | rien |
| 1.A.4 | **Édition réelle** : sélection au clavier et à la souris, ↑ ↓, Début/Fin, copier-coller **dans** le texte, et **IME** — sans quoi taper `é` est impossible | navigation ← → par octets |
| 1.A.5 | **LaTeX** : `katex-rs` derrière une crate `glucose-math`, plus l'interprète de son arbre vers des positions absolues (§ 7.1) | les douze familles de fontes KaTeX sont **déjà dans le dépôt** |
| 1.A.6 | **Courbes et graphes** à syntaxe PGFPlots, tracés nativement (§ 7.2) | le rastériseur |
| 1.A.7 | Liens cliquables, tableaux | rien |
| 1.A.8 | **Ancres de texte** : brancher `text_anchors` (224 l. mortes) — c'est ce qui permet à une flèche de partir d'une **phrase** et non d'une carte | 224 l. + tests |

> **Sortie** : un document de la capture de production s'affiche **à l'identique**, vérifié par
> capture PNG ; taper un accent fonctionne ; la prévisualisation suit la frappe sans lag mesurable
> au banc 0.1.

#### Chantier 1.B — La manipulation directe

| # | Travail | Déjà écrit |
|---|---|---|
| 1.B.1 | **Poignées de redimensionnement vivantes** — 8 par élément, avec ratio préservé sous `Maj` | `resize.rs` et `hit_priority::handles` sont écrits et testés |
| 1.B.2 | **Curseurs contextuels** sur les poignées | `handle_cursor` écrite, jamais appelée |
| 1.B.3 | **Barre d'action contextuelle** flottante : compte, verrouiller, supprimer | rien |
| 1.B.4 | **Menu contextuel** au clic droit (aujourd'hui le clic droit fait le pan) | rien |
| 1.B.5 | **Cycle de profondeur au clic** — `pick_at_down` / `advance_on_release` sont écrits, testés, et **n'ont aucun appelant** : le desktop prend toujours le premier candidat | complet |
| 1.B.6 | Ordre d'empilement, rotation, déplacement au clavier, verrouillage visible | `rotation` ignorée |

> **Sortie** : redimensionner une image à la souris, la faire tourner, cliquer deux fois au même
> endroit pour atteindre le nœud du dessous — tout cela annulable par `Ctrl+Z`, en une seule
> entrée d'historique par geste.

---

### Vague 2 — Les objets identitaires de Glucose

#### Chantier 2.A — Membranes *(1 442 lignes déjà écrites et testées)*

| # | Travail |
|---|---|
| 2.A.1 | Dessiner une membrane **par glisser** (aujourd'hui : 320 × 240 fixe) |
| 2.A.2 | Écrire `membrane_id` au dépôt — l'invariant MEMB-1 n'est **jamais** écrit aujourd'hui, donc aucune membrane ne possède rien |
| 2.A.3 | Brancher `membrane_space` : dépôt → adoption, coordonnées relatives, mode minimisé |
| 2.A.4 | Brancher `membrane_stretch` : la membrane grandit avec son contenu, et l'alerte d'étirement |
| 2.A.5 | Le **mode focus** au geste (le cadrage `fit_viewport` sert déjà aux vols de dossier) |
| 2.A.6 | Le tween de membrane — 200 ms, courbe prête depuis `acceb0d`, geste absent |
| 2.A.7 | Panneau d'options, membranes imbriquées, suppression en cascade |
| 2.A.8 | **Couleur dérivée des domaines** — `symbiotic_hue` × poids de domaine. C'est la fonctionnalité que la charte désigne pour naviguer dans l'immense |

#### Chantier 2.B — Flèches *(12 % de parité, le plus bas hors les 0 %)*

| # | Travail |
|---|---|
| 2.B.1 | **Dessiner par glisser** (aujourd'hui : 120 × 80 fixe, non dessinable) |
| 2.B.2 | **Les rendre cliquables** — `arrow_id: None` aux deux seuls endroits qui picorent : une flèche n'est aujourd'hui **jamais** sélectionnable |
| 2.B.3 | Brancher `arrow_anchor` : la flèche s'accroche au **bord** du nœud et le suit |
| 2.B.4 | Déplacer les extrémités, courbes, waypoints, bidirectionnelle, épaisseur, couleur |
| 2.B.5 | Étiquette sur la flèche + son éditeur |
| 2.B.6 | **Prédicats sémantiques** (6 types) — l'autre outil de navigation que la charte réclame |
| 2.B.7 | Attache à un sous-bloc / à une sélection de texte (dépend de 1.A.8) ; flèche-portail vers un autre board |

#### Chantier 2.C — Dossiers et miroirs *(commencé en `56737b8` et `acceb0d`)*

| # | Travail |
|---|---|
| 2.C.1 | **Miroirs** : le geste, le marquage visuel, la propagation des modifications, la remontée à l'original — le vol de caméra `MIRROR_TELEPORT` (400 ms) est prêt et attend |
| 2.C.2 | Brancher `mirror_graph` : détection de cycle |
| 2.C.3 | **Miroir d'un dossier OS** : scan snapshot/live, récursif, paresseux (49 k fichiers), glob, 7 modes de tri, vignettes |
| 2.C.4 | Finir le dossier : badge compteur (affiche zéro), poignée gravée à 45°, mini-carte de contenu (fiche 06 § 8.2) |

> **Sortie de la vague 2** : sur un jeu synthétique « forme Wikipédia » de 10⁶ nœuds, retrouver
> une information ciblée en moins de 30 secondes sans connaître sa position. C'est le critère
> d'usage de l'étape D de la fiche 11, conservé tel quel.

---

### Vague 3 — Le reste du portage

| Chantier | Contenu | Déjà écrit |
|---|---|---|
| 3.A | **Rideaux** — la fonctionnalité la plus originale de Glucose, 0 % : créer, languette nommée et colorée, rideau = board complet, visibilité privé/partagé, droits, ratios | 426 l. + 511 l. de tests |
| 3.B | **Export** — SVG, Markdown, HTML, PNG, **et l'écriture sur le disque** qui manque même aux deux moteurs prêts | 501 l. |
| 3.C | **Temporalité** — ancrage, invite de saisie, règle, filtre, plages, années négatives | 406 l. |
| 3.D | **Images** — glisser-déposer multi-fichiers **et depuis un navigateur** (le besoin n° 1), mipmaps, cache borné, décodage asynchrone, chargement progressif, lecture d'en-tête, vidéos, dédup, tags | `sha256` branché |
| 3.E | **Interface** — dock animé (rebond + FLIP, courbes prêtes), sélecteur de couleur, panneau Ordonner complet, infobulles, renommer/fermer/réordonner les boards, thème centralisé (~130 littéraux), barre de statut, HUD | courbes prêtes |
| 3.F | **Recherche** `Ctrl+F` plein texte + navigation vers un résultat | rien |
| 3.G | Storyboard, presets, zones | rien |
| 3.H | **Persistance** — autosave, versions automatiques, compaction, récupération après crash, migration du v1 TypeScript | format v2 complet |

---

### Vague 4 — La fondation et la performance

*Ici, et pas avant : on sait enfin ce qu'il y a dans une tuile.*

| # | Travail |
|---|---|
| 4.1 | **Substitution du noyau vers l'arène** — 209 accès dans `store/`, avec 641 tests comme filet. Rendue locale par la règle S. |
| 4.2 | Étaler la reconstruction de l'index sur plusieurs frames (dette du commit `c85b0e0` : 16 ms tombent aujourd'hui d'un coup) |
| 4.3 | **L'étape C de la fiche 11** : cache pyramidal de tuiles, invalidation exacte, couche d'arêtes longues, présentation GPU avec repli CPU testé par défaut, non-régression au pixel |
| 4.4 | Le protocole de mesure RQ-3 (paliers par octave, rendu direct vs réduction) — c'est lui qui tranche, pas un document |

> **Sortie** : pan et zoom ≥ 100 fps en 4K sur 10⁷ nœuds, 0 % de CPU au repos, netteté vérifiée
> par capture à l'arrêt.

---

### Vague 5 — Le lointain

Collaboration (CRDT, curseurs, canal d'assets — 1 920 l. de TS), plugins et App Bridge
(1 471 l.), Pomodoro, télémétrie, mise à jour. Rien ici ne conditionne quoi que ce soit d'autre.

---

## 6. Ce que je retire du plan, et pourquoi

Deux choses que la fiche 11 promettait et que je ne compte pas faire telles quelles :

- **Les crates séparées `glucose-geom` et `glucose-model`** (B.1, B.2). Elles sont nées comme
  modules de `glucose-core` (`fixed`, `arena`). Trois crates pour un code que rien d'autre ne
  consomme, c'est de la cérémonie : la frontière qui compte est celle entre le noyau et le
  dessin, et elle existe déjà. Décision assumée, déjà documentée en phase B.
- **La courbe de Hilbert** (B.3). Le bénéfice devait être « mesuré d'abord, adopté ensuite ». Il
  n'a pas été mesuré, et l'index CSR répond en 0,21 µs sans elle. Elle reste une option si le banc
  0.1 montre un jour un défaut de localité — pas un travail planifié.

---

## 7. Les questions ouvertes, et comment je compte les fermer

### 7.1 Le LaTeX — tranché, et mesuré

**Décision prise : la dépendance est assumée.** Le moteur maison est abandonné — pas par
paresse, mais parce que la mesure a montré qu'un portage complet de KaTeX existe déjà en Rust,
et qu'écrire à la main un sous-ensemble moins bon serait de l'orgueil, pas de l'élégance.

#### Ce que la mesure a établi

Deux candidats ont été essayés pour de vrai, sur les mêmes huit formules.

**`latex-rust` 1.0.2 — écarté.** Sa fiche promet « TeX-faithful layout on exact rationals, emit
SVG, PNG, or egui shapes », et sa sortie PNG passe par `tiny-skia`, le rastériseur du projet.
C'était le candidat idéal sur le papier. À l'image, il est inutilisable :

| Formule | Ce qui sort |
|---|---|
| `\begin{pmatrix} a & b \\ c & d \end{pmatrix}` | `(a bc d)` — **le saut de ligne est ignoré, la matrice est aplatie** |
| `\int_0^\infty` | la borne basse et la borne haute **se superposent** |
| `\binom{n}{k}` | dessiné **avec une barre de fraction**, qu'un binôme n'a pas |
| `\sum_{i=1}^{n} i = \frac{n(n+1)}{2}` | boîte englobante trop petite : le `n`, le `i=1` et la fraction sont **coupés** |

Une bibliothèque en version 1.0 peut être immature ; c'est précisément pourquoi on essaie avant
d'adopter.

**`katex-rs` 0.3.0 — retenu.** C'est le portage du vrai KaTeX, pas une réécriture : le crate
contient `lexer`, `macro_expander`, `functions`, `font_metrics`, `build_html`, `build_mathml`,
`dom_tree`. Ses dépendances non optionnelles sont six crates légères et sans JavaScript
(`bon`, `phf`, `rapidhash`, `strum`, `strum_macros`, `thiserror`) ; celles qui touchent au
navigateur sont derrière la feature `wasm`, qu'on ne prend pas.

#### Le seul vrai travail : KaTeX rend du HTML, et Glucose n'a pas de navigateur

`render_to_dom_tree` donne un arbre où **toute la typographie est déjà faite** — c'est KaTeX qui
calcule les hauteurs, les profondeurs et les décalages, le CSS ne fait que les appliquer. Extrait
réel pour `\int_0^\infty x\,dx` :

```text
SYM "∫"  h=0.805 d=0.306 w=0.472 it=0.194  classes=["mop","op-symbol","small-op"]
SPAN vlist                                  style="top:-2.3442em; margin-left:-0.1945em"
  SYM "0"  h=0.644 w=0.500                  classes=["mord","mtight"]
SPAN mspace                                 style="margin-right:0.1667em"
```

Le vocabulaire de positionnement est **fermé** : des boîtes posées horizontalement, des `vlist`
empilées avec un `top` explicite, des marges, et des symboles avec leurs métriques. Aucun
flottant, aucune flexbox, aucun retour à la ligne automatique. Écrire l'interprète de cet arbre
vers des positions absolues est donc un travail borné et **exactement testable** — les nombres
sont des `em`, on les compare.

Et les douze familles de fontes KaTeX sont **déjà dans le dépôt** en TTF (`dist/assets/`),
héritées du build de l'ancienne version : Main, Math, AMS, Size1 à 4, Caligraphic, Fraktur,
SansSerif, Script, Typewriter.

#### Où ça vit

Une crate `glucose-math` dédiée, qui dépend de `katex-rs` et rend de la **géométrie** — des
glyphes positionnés et des filets, en `em` — sans connaître ni pixel ni rastériseur.

Ce n'est pas la cérémonie que la fiche reproche par ailleurs à `glucose-geom` : celle-là aurait
isolé du code sans dépendance, donc n'aurait rien isolé. Une crate qui **contient une dépendance
externe** derrière une frontière, c'est exactement ce à quoi sert une crate — et le jour où l'on
change de moteur, un seul endroit bouge.

### 7.2 Les courbes et les graphes — la question qui reste ouverte

La demande est explicite : « le système de LaTeX **complet** avec le système de création aussi de
courbe et de graphe ». En LaTeX, cela s'appelle **TikZ** et **PGFPlots**. KaTeX ne les fait pas,
et rien en Rust ne les fait.

Il n'existe qu'une seule façon d'avoir le vrai TikZ : embarquer une vraie distribution TeX, ce
que fait `tectonic` (XeTeX + TeXLive, empaqueté en crate). Mon avis d'ingénieur, et il est
tranché :

| | TikZ par `tectonic` | Traceur natif à syntaxe PGFPlots |
|---|---|---|
| Empreinte | TeXLive, des centaines de Mo, ou téléchargement des paquets **à la demande par le réseau** | quelques milliers de lignes |
| Coût d'un graphe | une compilation LaTeX → PDF, puis rastériser le PDF (encore une dépendance) | un tracé, comme le reste de la scène |
| Au zoom | une image figée qu'on agrandit | **retracé net à toute échelle** |
| **Android** | **inatteignable** — TeXLive n'y est pas packagé, et le réseau au moment du rendu est exclu | fonctionne |

La dernière ligne est décisive, et elle ne vient pas de moi : la charte pose macOS, Android,
Windows et Linux comme **contrainte de conception, pas comme portage ultérieur**. TikZ complet
entre en collision frontale avec elle.

Ce que je propose à la place vise le **but** plutôt que le moyen : accepter la syntaxe
`\begin{axis} \addplot {x^2}; \end{axis}` — donc écrire du LaTeX, comme demandé — et la tracer
nous-mêmes. Dans un canva, c'est **meilleur** que TikZ, pas seulement plus simple : une courbe
tracée nativement se redessine à la résolution du zoom, là où un PDF compilé est une image morte.

**Ce qui reste à décider, et que je ne peux pas décider seul :** jusqu'où va « courbe et graphe ».
Tracer des fonctions, des nuages de points, des barres et des axes gradués est un chantier net.
Dessiner des diagrammes arbitraires — nœuds, flèches courbes, décorations, le TikZ des articles
de recherche — en est un autre, et Glucose sait déjà faire des nœuds et des flèches sur son
canva. La réponse dépend de ce qui doit vraiment se retrouver dans une carte.

### 7.3 Ce qui reste ouvert depuis la fiche 11

Inchangé : zoom entre paliers (tranché par C.5), OpenGL ou Vulkan (mesure sur matériel faible),
rastériseur maison ou `tiny-skia`, combien d'arêtes Wikipédia matérialiser, coût réel d'un
document Automerge.

---

## 8. Ce qui pourrait faire dérailler ce plan

| Risque | Signal précoce | Réponse |
|---|---|---|
| La règle S est oubliée et la substitution redevient une réécriture | le compteur d'accès directs monte | 0.3 est mécanique : le build échoue |
| Le texte riche s'avère être un puits sans fond | 1.A dépasse deux semaines sans capture conforme | Découper par capacité livrée (inline, puis blocs, puis maths), chacune testée seule |
| Une fonctionnalité de la vague 1 coûte le budget de frame | le banc 0.1 régresse de 10 % | On le sait le jour même, pas à la vague 4 |
| Repousser le GPU se paie plus cher que prévu | le banc plafonne sous 100 fps **sans** tuiles sur un document réel | C'est exactement ce que 0.1 existe pour attraper ; la vague 4 remonte alors dans l'ordre |
| Le sous-ensemble LaTeX choisi ne couvre pas les documents réels | une formule s'affiche en rouge | 7.1 : mesurer sur ses documents **avant** d'écrire une ligne |

---

## 9. Ce que ce document ne fait pas

Il ne donne pas de dates. Le projet n'a jamais tenu une estimation en jours, et une estimation
fausse est pire que pas d'estimation : elle transforme une décision technique en retard.

Ce qu'il donne, c'est un **ordre**, une **justification par chantier**, et une **sortie mesurable**
par vague. Le jour où une mesure contredit ce document, c'est ce document qu'on corrige.

---

*Suite logique : la vague 0, puis le chantier 1.A — le texte, que l'utilisateur a nommé en
premier et qui est le cœur de Glucose.*
