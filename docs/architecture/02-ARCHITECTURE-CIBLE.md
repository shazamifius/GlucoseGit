# 02 — Architecture cible

> Ce document décrit **ce qu'on construit à la place**. Il ne contient aucun code : uniquement
> des décisions, leurs raisons, et les invariants qui en découlent.
>
> Il répond point par point aux constats de [`01-AUDIT-CODE-RUST.md`](01-AUDIT-CODE-RUST.md).

---

## 0. La question des dépendances — réponse d'architecte

Tu dis : « full Rust from scratch, no library », puis « peut-être 1 ou 2 dépendances max ».
Ces deux phrases ne sont pas contradictoires, mais il faut trancher **où** passe la ligne, sinon
tu vas soit réécrire un décodeur WebP pendant trois mois, soit retomber dans les 200 crates
transitives d'aujourd'hui.

Le bon critère n'est pas « est-ce que je *peux* l'écrire », c'est :

> **Est-ce que cette dépendance me coûte plus cher en boîte noire qu'elle ne me coûterait en
> temps à réécrire — sachant que le temps ne se récupère pas, mais que la boîte noire, si.**

### Les quatre catégories

| Catégorie | Verdict | Pourquoi |
|---|---|---|
| **Modèle, géométrie, algorithmes, format de fichier, UI, rastériseur 2D** | **0 dépendance. Non négociable.** | C'est *ton* logiciel. Un rastériseur 2D (chemins, remplissage par scanline, anti-aliasing par couverture analytique, mélange alpha) fait 2 000–3 000 lignes. C'est du travail agréable, entièrement testable, et ça t'appartient pour toujours. |
| **Fenêtre, entrées, presse-papiers, glisser-déposer** | **0 dépendance — mais via les API système, pas via `winit`.** | Voir § 0.1 : c'est là qu'est la solution à ton problème de drag & drop web. |
| **Décodage d'images (PNG / JPEG / WebP / GIF)** | **1 dépendance assumée** | Un décodeur JPEG baseline correct = ~1 500 lignes. Progressif = ×3. WebP = un décodeur VP8 intra complet, plusieurs mois. Et un décodeur d'image est une **surface d'attaque** : c'est du code qui parse des octets hostiles. Écrire le sien, c'est prendre un risque de sécurité pour un gain nul en compréhension. |
| **Police : lecture TTF + rastérisation de glyphes** | **1 dépendance, remplaçable plus tard** | Lire une `glyf`/`loca` TrueType et rastériser des courbes quadratiques est faisable (~1 200 lignes) et intéressant. Mais le *shaping* (crénage `kern`/`GPOS`, ligatures `GSUB`, bidirectionnel) est un puits sans fond. Commence avec une dépendance, garde l'interface derrière un trait, réécris le jour où ça t'amuse. |

### 0.1 — Ton problème de glisser-déposer, et sa vraie solution

Tu écris : *« on ne peut pas faire de glisser-déposer depuis le web et en même temps depuis un
Windows/Linux pour des fichiers »*. C'est exact, et ce n'est **pas** une limite de Rust ni de
Tauri : c'est une limite de l'abstraction que tu utilises.

`winit` n'expose que trois événements : `HoveredFile`, `DroppedFile`, `HoveredFileCancelled` —
et **uniquement des chemins de fichiers**. Il jette tout le reste.

Or, quand tu fais glisser une image depuis un navigateur, Windows te propose en réalité
**plusieurs formats simultanés** dans le même objet de transfert OLE :

| Format Windows | Contenu |
|---|---|
| `CF_HDROP` | Chemins de fichiers — le seul que `winit` te donne |
| `CFSTR_INETURL` / `UniformResourceLocatorW` | L'URL de l'image |
| `CF_HTML` | Le fragment HTML source (avec `<img src=…>`) |
| `CF_UNICODETEXT` | Le texte / l'URL |
| `CFSTR_FILEDESCRIPTOR` + `CFSTR_FILECONTENTS` | **Les octets du fichier eux-mêmes**, sans passer par le disque |
| `CF_DIB` / `CF_DIBV5` | Le bitmap brut |

L'équivalent existe sur X11/Wayland (cibles `text/uri-list`, `text/html`, `image/png`,
`application/x-moz-file`) et sur macOS (`NSPasteboard` : `fileURL`, `URL`, `png`, `html`).

**Donc :** en implémentant toi-même `IDropTarget` (Windows) et XDND (Linux) — ce qui représente
~400 lignes par plateforme — tu obtiens quelque chose qu'**aucune** abstraction portable ne te
donnera : **fichiers locaux ET images web, dans le même geste, avec l'URL d'origine préservée**
(qui alimente `BoardImage.source_url`, un champ déjà prévu par ton modèle et jamais rempli).

C'est le plus bel argument pour « from scratch » de tout ce projet : ici, écrire soi-même n'est
pas de la fierté, c'est **la seule façon d'avoir la fonctionnalité**.

### 0.2 — Budget de dépendances cible

```
glucose-codec   →  décodage d'images        (1 crate)
glucose-text    →  rastérisation de glyphes (1 crate, remplaçable)
──────────────────────────────────────────────────────────────
TOTAL                                          2 dépendances directes
```

Tout le reste — fenêtre, entrées, DnD, presse-papiers, dialogues de fichiers, rastériseur 2D,
UI, modèle, format de fichier, export — est du code Glucose, sans dépendance, sauf les liaisons
système brutes (`windows-sys` sous Windows, `libc` + protocole X11/Wayland sous Linux), qui sont
des **déclarations de FFI**, pas des bibliothèques : elles ne contiennent aucune logique.

**Règle inscrite dans le dépôt** : toute nouvelle dépendance exige une note de décision
argumentée dans `docs/architecture/decisions/`. Pas de note, pas de dépendance.

---

## 1. Les dix lois

Chacune répond à un constat de l'audit. Elles sont non négociables : c'est ce qui empêche le
code de redevenir ce qu'il est aujourd'hui.

| # | Loi | Répare |
|---|-----|--------|
| **L1** | **Rien ne se dessine sans avoir été demandé par l'index spatial.** Chaque passe de rendu commence par une requête de visibilité. | R-02, R-05 |
| **L2** | **Le coût d'une frame ne dépend pas de la taille du document.** Il dépend de la taille de la fenêtre et du nombre d'éléments visibles. Toute violation est un bug de gravité bloquante. | R-02, R-03, R-05 |
| **L3** | **Une modification coûte la taille de la modification, jamais celle du document.** Undo, sauvegarde, synchronisation : tout est incrémental. | R-04 |
| **L4** | **La géométrie de l'interface est calculée une seule fois par frame.** Dessin et test de clic lisent la même liste. Aucun nombre magique n'est écrit deux fois. | R-07, R-08, R-20 |
| **L5** | **La logique métier ne connaît ni fenêtre, ni souris, ni pixel.** Elle est dans des crates sans dépendance, testable sans écran. | R-19 |
| **L6** | **Toute mutation du document passe par une commande.** Il n'existe aucun autre chemin d'écriture. | R-04, R-13, R-14 |
| **L7** | **Une seule convention de coordonnées** : origine en haut-gauche, taille positive, unités monde, `f64`. Aucune exception. | R-11 |
| **L8** | **Aucune erreur n'est silencieuse.** Chaque crate a son type d'erreur ; toute erreur atteignant l'utilisateur passe par la barre de statut. `let _ =` est interdit hors tests. | R-21 |
| **L9** | **Aucun champ du modèle sans consommateur.** Si un champ existe, il est rendu, éditable et sauvegardé — ou il n'existe pas. | R-24 |
| **L10** | **Le message de commit décrit ce que fait le code.** Si le commit dit « suppression de X », `grep X` doit ne rien retourner. | R-32, R-33 |

---

## 2. Découpage en crates

```
glucose/
├── crates/
│   ├── glucose-geom          0 dép.   Points, rects, transforms, index spatial, intersections
│   ├── glucose-model         0 dép.   Document, nœuds, ids, commandes, undo, invariants
│   ├── glucose-raster        0 dép.   Rastériseur 2D : chemins, fills, strokes, AA, blend, blit
│   ├── glucose-text          1 dép.   Fontes, atlas de glyphes, mise en page, Markdown, LaTeX
│   ├── glucose-codec         1 dép.   Décodage/encodage d'images, mipmaps, vignettes
│   ├── glucose-io            0 dép.   Format .glucose, sérialisation, autosave, migration
│   ├── glucose-ui            0 dép.   Widgets, layout, hit-test, thème, panneaux
│   ├── glucose-platform      FFI      Fenêtre, entrées, IME, presse-papiers, DnD, dialogues
│   ├── glucose-app           0 dép.   Outils, états d'interaction, raccourcis, câblage
│   └── glucose-cli           0 dép.   Export headless, tests de non-régression visuelle
└── xtask/                             Scripts de build, bench, snapshots de rendu
```

### Règle de dépendance : le graphe est un DAG strict

```
              geom
             /  |  \
        model  raster  text ──┐
          |      |     |      |
          io    ui ────┘   codec
           \     |   /       /
            \    |  /       /
             glucose-app ──┘
                  |
              platform
```

- Une flèche vers le haut est **interdite**. `glucose-model` ne connaîtra jamais `glucose-raster`.
- `glucose-platform` est feuille : elle ne dépend que de `geom`.
- **Test automatique en CI** : `cargo tree -p glucose-model` ne doit lister que `glucose-geom`.

### Pourquoi ce découpage précis

| Crate | Raison d'être |
|---|---|
| `geom` séparé de `model` | La géométrie est réutilisable par le rastériseur, l'UI et l'export sans traîner le document. |
| `raster` séparé de `ui` | Le rastériseur doit pouvoir produire un PNG d'export sans fenêtre. C'est ce qui rend l'export testable en CI. |
| `text` séparé de `raster` | La mise en page du texte est pure (mesures → lignes) et doit être testable sans dessiner. |
| `platform` isolé | C'est le seul crate contenant de l'`unsafe`. Il doit rester petit et auditable. |
| `cli` | Permet un **test de non-régression visuelle** : rendre une scène de référence en PNG et comparer aux octets attendus. C'est ce qui aurait attrapé R-06 immédiatement. |

---

## 3. Le modèle de données

### 3.1 — Composition, pas répétition (répare R-23)

Aujourd'hui : 4 variantes × 14 champs recopiés, 6 accesseurs en `match`, 12 `match` dans le store
juste pour lire `x`.

Cible : **un tronc commun + un noyau spécifique**.

```
Node
├── id: NodeId                     ← handle typé, pas String
├── transform: { x, y, w, h, rotation }
├── membership: { membrane, domains[], temporal_anchor, mirror_of }
├── style:  { color, opacity, locked, z }
└── kind: NodeKind
        ├── Text     { content, markdown_cache }
        ├── Sticky   { content, operator }
        ├── Arrow    { endpoints, waypoints, predicate, binding }
        ├── Membrane { mode, curtains[] }
        ├── Image    { asset, fit, source_url }
        └── Folder   { child_board, mirror_source }
```

Gains directs :

- `node.transform.x` est direct — plus aucun `match` pour lire une position ;
- ajouter un champ commun = **une ligne**, au lieu de 4 variantes + 6 `match` + N constructions ;
- `Image` et `Folder` rejoignent `Node` : ils ont déjà `x/y/w/h`, un id, un miroir. Aujourd'hui
  ils sont dans des `Vec` séparés, ce qui oblige chaque opération (sélection, déplacement,
  suppression, culling, alignement) à être **écrite trois fois**. C'est la moitié du volume de
  `store.rs`.

### 3.2 — Identité : handles typés, pas des `String` (répare R-13, R-14, R-22)

```
NodeId(u32 index, u32 generation)     ← handle stable, comparaison en 1 instruction
```

- **Arène** : les nœuds vivent dans un `Vec` contigu. Accès O(1). Itération sans indirection —
  c'est aussi ce qui rend le culling rapide.
- **Génération** : supprimer un nœud incrémente la génération de son emplacement. Un handle
  périmé est **détecté**, il ne pointe pas silencieusement sur le nouvel occupant.
  C'est exactement le bug R-13/R-14, rendu impossible par construction.
- **Ids externes** : le format de fichier écrit des ids texte stables (`ULID`, triables par date
  de création). Une table `HashMap<Ulid, NodeId>` fait le pont, uniquement au chargement et à la
  sauvegarde.
- **Génération d'id** : un compteur monotone dans le document. **Jamais** dérivé d'un autre id.

### 3.3 — Index spatial intégré, pas optionnel (répare R-02, R-03, R-05)

L'index n'est pas une structure qu'on pense à mettre à jour : il est **maintenu par les
commandes**. Toute commande qui déplace, crée ou supprime un nœud met à jour l'index dans la même
opération. Il ne peut pas être périmé.

Il sert **quatre** consommateurs, pas un :

1. **Culling du rendu** (L1) ;
2. **Test de clic** — `hit_priority` n'examine que les candidats de la cellule sous le curseur,
   au lieu de tout le board ;
3. **Cibles d'alignement** — `collect_align_targets` ne considère que le voisinage ;
4. **Invalidation de la teinte symbiotique** — voisinage à 1 200 px (R-03).

`quadtree.rs` existe déjà et fait le travail. Il faut le brancher, lui ajouter `insert`/`remove`
incrémentaux, et remplacer `HashSet<String>` par `Vec<NodeId>` (plus d'allocation de chaînes par
requête).

### 3.4 — Les binaires sortent du document (répare R-04)

```
Document          ← nœuds, boards, domaines, presets. Petit. Clonable. Journalisable.
AssetStore        ← sha256 → octets. Séparé. Jamais dans un snapshot d'undo.
```

`AssetStore` est content-addressed : deux fois la même image = un seul blob. Il vit à côté du
document, avec son propre comptage de références et son propre ramasse-miettes au moment de la
sauvegarde. Une commande référence un `AssetId`, jamais des octets.

C'est ce qui fait passer la pile d'undo de 16 Go à quelques mégaoctets.

---

## 4. Undo : journal, pas snapshot (répare R-04)

### Le principe

Toute mutation est une **commande** qui sait faire deux choses : s'appliquer, et produire son
inverse.

```
Command
├── apply(&mut Document) -> Inverse
└── label() -> &str            ← pour l'UI : « Annuler : déplacer 3 éléments »
```

La pile d'undo ne contient plus des documents, elle contient des inverses. Coût mémoire :

| Action | Coût snapshot (actuel) | Coût journal (cible) |
|---|---:|---:|
| Déplacer 3 éléments | taille du projet entier | 3 × (id + 2 `f64`) = **72 octets** |
| Taper un caractère | taille du projet entier | position + 1 caractère = **~16 octets** |
| Importer 20 images | 20 × projet entier | 20 × (id + métadonnées) = **~2 Ko** |

### Les trois invariants que le code actuel a déjà raison de vouloir

Ils sont bons, ils sont testés (`undo_redo_suite.rs`, 707 lignes), il faut les **garder** :

1. **La navigation ne pollue jamais l'undo.** Pan, zoom, changement de board, entrée dans un
   dossier ne sont pas des commandes. → `mutate` vs `mutateView`, déjà présent.
2. **La caméra est préservée à travers undo/redo.** Pas de téléportation. → `preserve_view`,
   déjà écrit.
3. **Une transaction live est atomique.** Un drag de 400 événements souris = **une** entrée
   d'undo. → `begin_live_edit` / `end_live_edit`, déjà présent.

Un quatrième à ajouter :

4. **Les commandes coalescent.** Taper 20 caractères d'affilée dans le même champ = une seule
   entrée. Règle : deux commandes du même type, sur la même cible, à moins de 500 ms d'écart,
   fusionnent.

### Le bonus gratuit

Un journal de commandes te donne, sans travail supplémentaire :

- la **sauvegarde incrémentale** (n'écrire que le delta depuis le dernier point) ;
- la **collaboration** (une commande est un message réseau) ;
- la **time machine** que la version TypeScript avait via Automerge, **sans Automerge** ;
- le **rejeu de session** pour reproduire un bug — inestimable en débogage.

C'est la décision la plus rentable de tout le document.

---

## 5. Le pipeline de rendu

### 5.1 — Une seule frame, dirty rects, jamais de repaint intégral (répare R-15)

**Aujourd'hui** : 30 appels directs à `redraw()` depuis les handlers ; chaque mouvement de souris
redessine tout.

**Cible** — un cycle unique :

```
Événement OS  →  handler : mute l'état, marque des rectangles sales, retourne
                            ↓
             ControlFlow::WaitUntil(prochaine échéance d'animation)
                            ↓
              RedrawRequested → frame unique
                            ↓
   1. Fusionner les rects sales     (union bornée, max 4 régions)
   2. Requête spatiale : nœuds visibles ∩ rects sales
   3. Trier par z
   4. Peindre uniquement ces rects
   5. Présenter uniquement ces rects
```

Conséquences mesurables :

- Survoler un bouton repeint 30×30 px, pas 1440×900.
- Déplacer une carte repeint l'ancien rect ∪ le nouveau, pas la scène.
- Le curseur clignote (échéance à +500 ms) **sans** que la souris bouge.
- Le toast s'efface tout seul.

### 5.2 — Couches, avec cache indépendant

| Couche | Se redessine quand… | Mise en cache |
|---|---|---|
| Fond + grille | le zoom change de palier, ou la fenêtre est redimensionnée | **tuile 256×256 répétée** — coût quasi nul (répare R-02) |
| Halos symbiotiques | un nœud du voisinage bouge | teintes mémorisées dans le document (répare R-03) |
| Contenu (images, cartes, membranes, flèches) | les nœuds sales | culling spatial (répare R-05) |
| Superpositions (sélection, poignées, guides, marquee) | à chaque geste | jamais mise en cache — c'est petit |
| Interface | l'état d'UI change | liste de widgets, réutilisée pour le hit-test (répare R-20) |

### 5.3 — Le rastériseur maison

C'est le morceau « from scratch » le plus satisfaisant du projet, et il est parfaitement à ta
portée. Contenu minimal :

- **Chemins** : `move_to`, `line_to`, `quad_to`, `cubic_to`, `close` ; aplatissement des courbes
  selon une tolérance dépendant du zoom.
- **Remplissage par scanline** avec **couverture analytique** (accumulation d'aire signée par
  cellule) — c'est la technique de FreeType et de `font-rs`, et elle donne un anti-aliasing de
  meilleure qualité que le sur-échantillonnage, pour moins cher.
- **Contours** par génération du chemin décalé (joints : miter/round/bevel ; extrémités :
  butt/round/square ; pointillés).
- **Mélange** en alpha prémultiplié, avec une **rampe gamma sRGB** (répare R-27).
- **Blit d'images** avec échantillonnage trilinéaire sur mipmaps (répare R-29).
- **Clip rectangulaire** — c'est ce qui rend les dirty rects réels.

Volume estimé : 2 000 à 3 000 lignes. Testable par comparaison de PNG en CI.

**Interface à respecter dès le premier jour** : tout passe par un trait `Canvas`. Le jour où tu
veux du GPU, tu écris une seconde implémentation ; **aucun** appelant ne change. Ne fais pas de
GPU maintenant : le CPU tient largement ce budget si L1 et L2 sont respectées, et le GPU
introduirait la plus grosse boîte noire du projet.

---

## 6. Le moteur de texte (répare R-26, R-27, R-28)

Le texte est **la** faiblesse actuelle, et c'est le cœur de Glucose : Glucose est un outil de
pensée écrite avant d'être un moodboard.

### 6.1 — Séparation stricte mise en page / dessin

```
Texte brut
   ↓  parse Markdown            → arbre de blocs (titre, para, liste, code, citation, maths)
   ↓  découpe en runs           → (style, portée) ; gras/italique/code/lien
   ↓  shaping par run           → glyphes + positions (crénage inclus)
   ↓  césure / retour à la ligne → lignes, avec largeur maximale
   ↓  positionnement vertical   → lignes de base réelles (ascendante/descendante/interligne)
   ═  LAYOUT                    ← objet immuable, mis en cache sur le nœud
   ↓
   ├─ dessin        : lit le layout
   ├─ mesure        : lit le layout          ← plus jamais deux mesures divergentes
   ├─ hit-test      : lit le layout          ← curseur à la position exacte du clic
   └─ curseur/sélection : lit le layout      ← répare le curseur de la sticky
```

**Un seul objet, quatre consommateurs.** C'est le même principe que L4, appliqué au texte.

### 6.2 — Atlas de glyphes (répare R-26)

Cache `(font, glyphe, taille quantifiée, sous-pixel) → bitmap`, dans un atlas contigu. Les
tailles sont quantifiées par paliers de 0,25 px pour éviter l'explosion du cache au zoom continu.

Effet sur le contour des titres de membranes : rastériser **une** fois, compositer 9 fois — au
lieu de 9 rastérisations. Et mieux : générer un vrai contour par dilatation du canal de couverture,
ce qui donne un résultat propre au lieu de 8 copies décalées.

### 6.3 — Ce que le layout doit savoir faire, par ordre de priorité

| Priorité | Fonctionnalité | Pourquoi |
|---|---|---|
| P0 | Retour à la ligne aux limites de mots + hauteur de ligne correcte | Sans ça, les cartes débordent. Bloquant. |
| P0 | Curseur et sélection par position de clic | Sans ça, l'édition n'est pas utilisable. |
| P0 | Crénage (`kern` / `GPOS` simple) | Sans ça, le texte a l'air amateur — c'est visible au premier coup d'œil. |
| P1 | Markdown : `#`, `**`, `*`, `` ` ``, `-`, `1.`, `>`, `[]()` | Parité avec la version TS. |
| P1 | IME + grappes de graphèmes | Répare R-17. |
| P2 | Sélection multi-lignes, glisser pour sélectionner | Confort d'édition. |
| P2 | LaTeX en ligne (`$…$`) | La version TS a KaTeX. Cible : un sous-ensemble (fractions, exposants, indices, symboles, racines) rendu vectoriellement. |
| P3 | Bidirectionnel, ligatures, réordonnancement indien | Seulement si un besoin réel apparaît. |

---

## 7. L'interface : layout une seule fois (répare R-04, R-07, R-08, R-20)

### Le principe

```
                   état d'UI
                       ↓
                 layout(état) → Vec<Widget { id, rect, visual }>
                    ↙       ↘
              dessin        hit-test
```

Une seule fonction produit la géométrie. Le dessin et le clic **lisent la même liste**.
Il devient **structurellement impossible** qu'un bouton ne clique pas là où il est dessiné.

### Ce que ça change concrètement

| Aujourd'hui | Cible |
|---|---|
| Ajouter un bouton = éditer 2 fonctions, recopier 4 nombres | Ajouter une ligne dans la description de la barre |
| Onglets décalés (R-07) | Impossible : une seule mesure |
| Minimap inatteignable (R-08) | Impossible : le hit-test balaie la liste complète, sans branche `if y < HEADER` |
| Aucune infobulle malgré les `_title` ignorés | Le survol connaît le widget → infobulle gratuite |
| Aucune navigation clavier | La liste ordonnée donne le parcours `Tab` gratuitement |

### Thème (répare R-31)

Une structure `Theme` avec des jetons nommés — `bg`, `surface`, `border`, `text`, `text_dim`,
`accent`, `danger`, `selection`, `grid` — et des jetons d'espacement/rayon/typographie.
`Color::from_rgba8(56, 189, 248)` recopié 11 fois devient `theme.accent`.

Bénéfice immédiat : le thème clair devient un fichier de données, pas une réécriture.

### Panneaux

La version TypeScript a une douzaine de panneaux (domaines, timeline, storyboard, presets,
plugins, multijoueur, recherche, organisation, options de membrane, options de flèche…).
Il faut **un** système de panneaux : dock latéral, un panneau actif à la fois, en-tête commun,
fermeture au `Échap`. Écrire douze panneaux à la main, c'est douze fois R-20.

---

## 7 bis. Les dépendances, mesurées — et le chemin pour les réduire

*Mesuré au commit `6d48234`. Ce tableau existe parce que « zéro dépendance, zéro boîte noire » est
un objectif du projet, pas un slogan : il faut donc savoir précisément ce qu'on paie et pour quoi.*

### L'état exact

`glucose-core` : **0 dépendance**, et ce n'est pas un vœu — c'est vérifié à chaque rendu d'agent
par `git diff --stat -- '*Cargo.toml' '*Cargo.lock'`. Le SHA-256 de la persistance, le lecteur de
table `cmap` des polices, la sérialisation binaire, le hachage spatial : tout est écrit à la main
sur `std`.

`glucose-desktop` : **7 dépendances directes, 54 crates** dans le graphe complet.

| Dépendance | Crates transitives | Appels dans le code | Ratio |
|---|---:|---:|---|
| `arboard` | 28 | **1** | le pire du projet |
| `image` | 21 | **4** | mauvais |
| `tiny-skia` | 15 | 58 | justifié |
| `winit` | 12 | 22 | justifié |
| `fontdue` | 7 | 4 | à surveiller |
| `softbuffer` | 6 | 7 | justifié |
| `rfd` | 5 | 10 | justifié |

### Ce que les chiffres bruts font croire, et ce qui est vrai

« 28 crates pour un seul appel » invite à supprimer `arboard` en premier. **C'est une erreur de
raisonnement, et la mesure le montre** : les graphes se recouvrent largement.

- Retirer **`image` seul** ferait disparaître **0 crate**. La totalité de son arbre est déjà amenée
  par d'autres. Le gain en nombre de dépendances serait **nul**.
- Retirer **`arboard` seul** ferait disparaître **3 crates**.
- Retirer **les deux** ferait tomber le graphe de **54 à 39**, soit 15 crates :
  `arboard`, `clipboard-win`, `error-code`, `quick-error`, `image`, `image-webp`, `gif`, `weezl`,
  `color_quant`, `zune-core`, `zune-jpeg`, `byteorder-lite`, `moxcms`, `num-traits`, `pxfm`.

**Leçon** : le coût d'une dépendance ne se lit pas sur sa ligne du `Cargo.toml`. Il se mesure
**à la marge**, en retirant effectivement le nœud du graphe. Toute décision de suppression doit
être précédée de cette mesure, sinon on travaille beaucoup pour ne rien gagner.

### Le vrai argument contre `image` n'est pas le nombre de crates

Il est dans ce que fait cette dépendance : **décoder les images**. C'est le cœur d'un moodboard,
et c'est aujourd'hui une boîte noire. Trois conséquences concrètes, toutes déjà constatées :

- **R-30** — lire les dimensions d'une image en appelant `image::open`, qui **décode le fichier
  entier**. Pour une photo de 40 Mpx, c'est 160 Mo alloués pour connaître deux entiers.
- Aucun contrôle sur le **décodage progressif**, les **mipmaps**, ni le **budget mémoire**
  (R-29 : le cache d'images est toujours non borné).
- Aucune prise sur les formats : ce qui est accepté, ce qui est refusé, et comment un fichier
  corrompu échoue.

Un décodeur PNG maison est de l'ordre de 400 à 600 lignes (`inflate` compris) et rend tout ce
contrôle. Le JPEG est un autre ordre de grandeur — c'est le vrai arbitrage à poser, pas le nombre
de crates.

### Ce qui est réellement inévitable

| Besoin | Dépendance | Pourquoi c'est inévitable |
|---|---|---|
| Fenêtre, événements, IME, DPI | `winit` | Réécrire Win32 + X11 + Wayland + Cocoa n'est pas un projet de moodboard, c'est un projet de toolkit. |
| Tampon de pixels vers l'écran | `softbuffer` | Même raison, en plus petit. |
| Dialogues natifs de fichiers | `rfd` | Idem — et l'utilisateur attend les dialogues de **son** système. |
| Rastérisation vectorielle | `tiny-skia` | 58 appels, portage de Skia. Remplaçable en théorie, c'est un projet en soi. |

Les quatre premiers sont des **frontières avec le système d'exploitation**. C'est exactement le
rôle que le § 2 assigne à `glucose-platform` : la dépendance y est isolée derrière une interface
maison, de sorte qu'elle reste remplaçable et qu'elle ne contamine jamais le reste du code.

`fontdue` (4 appels, 7 crates) est le candidat le plus discret : la rastérisation de glyphes est
un problème fini et bien documenté, et le projet sait déjà lire une table `cmap` en Rust pur.

### La règle qui en découle

**Aucune dépendance ne s'ajoute sans un ADR** (`docs/architecture/decisions/`), et cet ADR doit
porter trois chiffres : le nombre de crates **ajoutées à la marge** au graphe existant, le nombre
d'appels prévus, et ce que coûterait l'écrire soi-même. Sans ces trois chiffres, la discussion
n'est pas une décision d'architecture, c'est une préférence.

---

## 8. Persistance : le format `.glucose` v2 (répare R-01)

C'est **la** priorité fonctionnelle. Sans elle, rien d'autre n'a de valeur.

### Structure du conteneur

```
.glucose  (archive, non compressée pour les blobs déjà compressés)
├── manifest.bin      version, ids, horodatages, sommes de contrôle
├── document.bin      nœuds, boards, domaines, presets — sérialisation maison
├── journal.bin       (optionnel) commandes depuis le dernier point — sauvegarde incrémentale
└── assets/
    └── <sha256>      octets bruts, content-addressed, dédupliqués
```

### Décisions

| Décision | Raison |
|---|---|
| **Format binaire maison, pas JSON** | 0 dépendance, contrôle du versionnement, chargement en une passe, pas de flottants réécrits en texte. La sérialisation d'un modèle en arène est mécanique : longueur + champs. |
| **Numéro de version en tête, migrations chaînées** | `v2 → v3 → v4`. Un fichier ancien s'ouvre toujours. La version TS avait déjà `projectMigration.ts` : garder l'idée. |
| **Somme de contrôle par section** | Un fichier tronqué (coupure de courant) est **détecté**, et les sections saines sont récupérées. |
| **Écriture atomique** | Écrire dans `.glucose.tmp`, `fsync`, puis renommer. Un crash pendant la sauvegarde ne détruit jamais le fichier précédent. |
| **Assets content-addressed** | Dédup native ; deux occurrences de la même image = un blob. Ramasse-miettes à la sauvegarde. |
| **Autosave par journal** | Toutes les N secondes, n'écrire que `journal.bin`. Coût proportionnel à l'édition, pas au document. Récupération après crash gratuite. |

### Import du format v1 (TypeScript)

Le `.glucose` actuel est un bundle Automerge. Un importeur **unidirectionnel** doit exister —
sinon tous les projets faits sous la version TS sont perdus, et c'est ce qui condamne les
réécritures. Il est simple d'accepter de le faire en lecture seule.

---

## 9. Entrées et plateforme

### 9.1 — Ce que `glucose-platform` doit exposer

```
Fenêtre       : création, redimensionnement, DPI + ScaleFactorChanged, always-on-top,
                plein écran, curseurs, titre, icône
Entrées       : souris (dont bouton du milieu, molette horizontale), clavier avec
                dispositions, modificateurs, IME complet, tablette (pression), tactile
Presse-papiers: lecture ET écriture, multi-formats (texte, HTML, PNG, DIB, chemins)
DnD           : cible de dépôt complète — fichiers, URL, HTML, octets, bitmap (§ 0.1)
Dialogues     : ouvrir/enregistrer natifs
Système       : ouvrir un fichier avec l'app associée (« App Bridge » de la version TS)
```

### 9.2 — Les manques d'aujourd'hui à corriger

| Manque | Conséquence actuelle |
|---|---|
| DPI (R-16) | UI minuscule à 150 %, illisible à 200 % |
| IME (R-17) | Pas d'accents au clavier mort, pas de CJK |
| Molette horizontale | Pas de défilement latéral au trackpad |
| Bouton du milieu | Confondu avec le clic droit pour le pan |
| Menu contextuel | Le clic droit sert au pan, aucun menu — un canvas sans menu contextuel, c'est une amputation |
| Curseurs | Toujours la flèche, même sur les poignées de redimensionnement (alors que `handle_cursor()` existe dans `hit_priority.rs`… jamais appelé) |
| Tablette | Aucune pression stylet |
| Multi-fenêtres | Une seule fenêtre codée en dur |

---

## 10. Ce qui est réutilisé du code actuel

La réécriture n'est pas un `rm -rf`. Voici ce qui migre, et comment.

| Module actuel | Devenir |
|---|---|
| `hit_priority.rs` (920 l., 348 l. de tests) | **Migré presque tel quel** vers `glucose-model`. Adapter `PickInput` à l'arène + index spatial. C'est le meilleur code du dépôt. |
| `smart_align.rs` (458 l., 346 l. de tests) | **Migré tel quel** vers `glucose-model`, puis **branché** (R-09). |
| `symbiotic_hue.rs` | **Migré**, avec mémorisation + invalidation par voisinage (R-03). L'algorithme ne change pas. |
| `quadtree.rs` | **Migré et étendu** : `insert`/`remove` incrémentaux, `NodeId` au lieu de `String`. Devient le cœur de L1. |
| `mirror_graph.rs` | **Migré tel quel**. |
| `export.rs` (SVG, Markdown) | **Migré** vers `glucose-cli` + branché à une vraie UI d'export. |
| `bundle.rs` (sha256, base64) | **Migré** vers `glucose-io`. Le sha256 maison devient le cœur du store d'assets. |
| `membrane_*.rs`, `curtain_*.rs`, `arrow_anchor.rs`, `text_anchors.rs`, `timeline.rs` | **Conservés en réserve.** Ils ne sont branchés qu'à leur phase (6, 8, 9). Ne pas les jeter, ne pas les brancher trop tôt. |
| `types.rs` | **Refondu** en composition (§ 3.1). Les sémantiques (`MembraneMode`, `CurtainVisibility`, `ArrowPredicate`, `FolderSortMode`) sont **conservées intégralement** — elles sont justes. |
| `store.rs` | **Réécrit.** C'est le seul module vraiment à refaire : snapshots (R-04), scans linéaires (R-22), collisions d'ids (R-13, R-14). |
| `app.rs` | **Éclaté** en `glucose-app` (outils, états, raccourcis) + `glucose-platform`. `organize_layout` part dans `glucose-model`. |
| `renderer.rs`, `ui.rs`, `icons.rs`, `typography.rs` | **Réécrits** sur `glucose-raster` / `glucose-ui` / `glucose-text`. Les *valeurs visuelles* (couleurs, rayons, espacements) sont extraites en `Theme` et conservées — le look actuel est bon, c'est son implémentation qui ne l'est pas. |

**Environ 2 800 lignes de logique testée sont sauvées.** Ce n'est pas un redémarrage à zéro :
c'est la même matière, dans une structure qui la laisse enfin servir.

---

## 11. Comment on saura que ça marche

Des seuils chiffrés, vérifiés en CI. Un dépassement casse la build.

| Indicateur | Seuil | Répare |
|---|---|---|
| Frame, canvas de 10 000 nœuds, 50 visibles | **< 8 ms** | L2, R-02, R-03, R-05 |
| Frame au zoom minimum (0,01) | **< 8 ms** | R-02 |
| Mémoire, 200 images 24 Mpx, 200 undos | **< 1,5 Go** | R-04, R-29 |
| Ouverture d'un projet de 500 nœuds + 200 images | **< 400 ms** jusqu'au premier affichage | R-30 |
| Sauvegarde incrémentale | **< 30 ms** | § 8 |
| Import de 50 images (glisser-déposer) | **aucun gel > 100 ms** | R-30 |
| Cliquer sur un widget | **100 %** des widgets, testé automatiquement sur la liste de layout | L4, R-07, R-08 |
| Couverture de tests de `glucose-model` | **> 80 %** | R-19 |
| `unsafe` hors de `glucose-platform` | **0** | — |
| Dépendances directes | **≤ 2** hors FFI système | § 0.2 |
| Champs du modèle sans consommateur | **0**, vérifié par un test | L9, R-24 |

---

**Suite** : [`03-PARITE-FONCTIONNELLE.md`](03-PARITE-FONCTIONNELLE.md) — l'inventaire exhaustif
de ce que fait Glucose aujourd'hui, et où en est le Rust.
