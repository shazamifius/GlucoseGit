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
| 1.A.5 | **LaTeX**. Question ouverte, voir § 7.1 | rien |
| 1.A.6 | Liens cliquables, tableaux | rien |
| 1.A.7 | **Ancres de texte** : brancher `text_anchors` (224 l. mortes) — c'est ce qui permet à une flèche de partir d'une **phrase** et non d'une carte | 224 l. + tests |

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
| 2.B.7 | Attache à un sous-bloc / à une sélection de texte (dépend de 1.A.7) ; flèche-portail vers un autre board |

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

### 7.1 Le LaTeX — la seule vraie inconnue de ce plan

KaTeX, c'est 70 000 lignes de JavaScript et six fontes. Il n'y a pas d'équivalent mûr en Rust.
Trois voies :

1. **Un moteur maison, limité au sous-ensemble réellement utilisé.** La typographie mathématique
   de TeX est un algorithme publié et exact : des boîtes, de la colle, et des règles de style
   (`\displaystyle`, `\textstyle`, indices, exposants). Un sous-ensemble couvrant fractions,
   exposants, indices, racines, sommes, intégrales, matrices, délimiteurs extensibles et symboles
   grecs tient en ~2 000 lignes, se teste **sans écran**, et ne dépend de rien. C'est cohérent
   avec l'élégance mathématique que la charte réclame.
2. Une dépendance Rust existante — à évaluer sur sa maturité et son coût.
3. Ne pas faire, et afficher la source.

**Ce qui décide** : une mesure, pas un goût. Combien de formules dans les documents réels de
l'utilisateur, et **lesquelles** ? Si c'est cinquante formules d'algèbre simple, la voie 1 est
évidente ; si c'est de la théorie des catégories avec des diagrammes commutatifs, elle ne l'est
pas. **Je ne peux pas répondre seul : il me faut ses documents.**

### 7.2 Ce qui reste ouvert depuis la fiche 11

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
