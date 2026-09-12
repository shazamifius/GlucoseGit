# Glucose — Dossier d'architecture

> **Rôle de ce dossier** : servir d'architecte. Il ne contient aucun code. Il contient l'état
> réel du projet, les problèmes nommés et vérifiés, l'architecture cible et le plan pour
> y arriver.
>
> **Date** : 2026-09-10 — audit au commit `7a13c86`, **re-vérifié** au commit `81aea31`
> **Périmètre audité** : `crates/glucose-core` + `crates/glucose-desktop` (14 322 lignes Rust)
> **Référence de parité** : la version TypeScript (`src/`, 45 218 lignes, 180 fichiers)

---

## Les documents

| # | Document | Ce qu'il contient | Quand le lire |
|---|---|---|---|
| **01** | [Audit du code Rust](01-AUDIT-CODE-RUST.md) | **33 constats vérifiés** (dont **4 déjà corrigés** pendant la rédaction), avec gravité, fichier:ligne et correctif. Ce qui n'est pas pro, ce qui n'est pas optimisé, et pourquoi. | Maintenant. C'est le diagnostic. |
| **02** | [Architecture cible](02-ARCHITECTURE-CIBLE.md) | Le découpage en crates, les 10 lois, la politique de dépendances, le modèle, l'undo, le rendu, la persistance. **Et la vraie solution à ton problème de glisser-déposer.** | Avant d'écrire la première ligne. |
| **03** | [Parité fonctionnelle](03-PARITE-FONCTIONNELLE.md) | **279 fonctionnalités inventoriées**, avec leur état exact. Le chiffre réel : **22 %**, pas 3 %. | Pour savoir où tu en es, vraiment. |
| **04** | [Roadmap](04-ROADMAP.md) | 13 phases, chacune avec ses travaux et ses **critères de sortie**. De 20 % à 100 %. | Pour savoir quoi faire lundi matin. |
| **05** | [Standards de code](05-STANDARDS-DE-CODE.md) | Les règles qui empêchent le code de redevenir ce qu'il est. Diagnostic précis de « pourquoi c'est moche ». | Avant chaque merge. |
| **06** | [Rendu visuel & Design System](06-SPEC-RENDU-VISUEL-DESIGN-SYSTEM.md) | **La bible visuelle pixel-perfect** : tokens HEX/RGBA exacts, grille infinie ($G, R, \alpha, P$), cadre 3px, poignées 9px (hitbox 48px), symbiose chromatique (djb2 + Perlin 2D + moyenne vectorielle 1200px), flèches multicouches, minimap, chrome. | Indispensable pour coder le rendu Rust identique. |
| **07** | [Animations & Interactions](07-SPEC-ANIMATIONS-ET-INTERACTIONS.md) | **La cinétique exacte** : table TIMING, cubic ease-out, rebond de dock (overshoot 156%), FLIP 250ms, l'arbitre de clic PICK-1 (rangs 0 à 60, cycle 8px), magnétisme SNAP-1 (8px écran), tweening des membranes (200ms). | Pour régler le "feel & polish" et les animations. |
| **08** | [Nœuds & Fonctionnalités Canvas](08-SPEC-FONCTIONNALITES-CANVAS-ET-NOEUDS.md) | **Inventaire technique exhaustif** : images (culling hystérésis, LOD progressif), texte GFM + KaTeX + undo unique, stickies + opérateurs (AND/OR/BUT/BECAUSE), flèches (évitement d'obstacles), dossiers miroir OS, miroirs vivants anti-cycle, membranes. | Pour implémenter chaque entité sans rien oublier. |
| **09** | [Systèmes Métier & Persistance](09-SPEC-SYSTEMES-METIER-ET-PERSISTANCE.md) | **Les moteurs sous le capot** : schéma universel, undo/redo (transactions atomiques), format binaire `.glucose` v2, bundle portable dédupliqué SHA-256, filet anti-corruption, domaines sémantiques, timeline temporelle, collaboration CRDT. | Pour la persistance et la robustesse globale. |
| **11** | [Plan de marche](11-PLAN-DE-MARCHE.md) | **La charte confrontée aux fiches 02 et 04** : 100 fps (budget 10 ms), netteté à l’arrêt, cible 10^7 nœuds (canva Wikipédia), dépendances minimales mais assumees. Mesures refaites a la main, 6 remises en question chiffrées (modèle 512 o/noeud, blit 4K a 9,6 ms = 96 % du budget, tuiles vs LOD, parité TS invalide comme étalon, les arêtes Wikipédia = le vrai milliard, auto-critique du zéro-dépendance), ordre de marche A→G. | **Avant de choisir quoi faire ensuite.** |

---

## Le diagnostic en cinq phrases

1. **Le code n'est pas mauvais, il est débranché.** 13 des 18 modules de `glucose-core` sont
   écrits, testés — et jamais appelés par l'application. C'est ≈ 3 700 lignes de travail réel
   qui ne sert à rien pour l'utilisateur.

2. **Tu n'es pas à 3 %, tu es à 22 %.** Et si tu ne faisais que **brancher** ce qui existe déjà,
   sans écrire un seul algorithme nouveau, tu passerais à **26 %**.

3. **L'application ne sait pas enregistrer.** Il n'existe aucune écriture de projet sur disque
   dans tout `glucose-desktop`. Ce n'est pas encore un logiciel.

4. **La teinte symbiotique est recalculée en O(n) par nœud visible, par frame, deux fois** —
   pour un résultat identique à la frame précédente. *(Les deux autres fautes de performance,
   la grille en O(surface) et l'absence de culling, viennent d'être corrigées par le commit
   `81aea31`.)*

5. **L'interface calcule sa géométrie deux fois** — une fois pour dessiner, une fois pour
   cliquer, avec les mêmes nombres recopiés. Le commit `81aea31` vient de régler ça **pour la
   barre d'outils**, avec des tests, et c'est exactement le bon patron. Mais la **barre
   d'onglets** garde ses deux mesures divergentes (les onglets ne cliquent pas là où ils sont
   dessinés), et la minimap reste dans du code strictement inatteignable.

---

## Tes questions, mes réponses d'architecte

### « Est-ce que je dois continuer, ou tout reprendre ? »

**Continue — mais restructure.** Ce n'est pas la même chose que « réécris ».

Ce qui est bon et doit être **gardé tel quel** : `hit_priority`, `smart_align`, `symbiotic_hue`,
`quadtree`, `mirror_graph`, `bundle`, `export`, les modules de membranes et de rideaux, et
surtout les **3 194 lignes de tests**. Soit ≈ 2 800 lignes de logique métier valide.

Ce qui doit être **refait** : `store.rs` (snapshots d'undo, scans linéaires, collisions d'ids),
`app.rs` (objet-dieu de 1 029 lignes), `ui.rs` + `renderer.rs` (géométrie en double, aucun cache).

**Ce n'est pas un redémarrage à zéro. C'est la même matière, dans une structure qui la laisse
enfin servir.**

### « Le "0 dépendance", c'est réaliste ? »

**Presque. Vise 2, pas 0** — et surtout, place-les au bon endroit.

Écrire ton propre rastériseur 2D (2 000-3 000 lignes) : **oui**, c'est faisable, agréable et
ça t'appartient. Écrire ton propre décodeur WebP : **non** — c'est un décodeur VP8 complet,
plusieurs mois, et une surface d'attaque sur des octets hostiles pour un gain de compréhension
nul.

Détail complet dans [`02-ARCHITECTURE-CIBLE.md` § 0](02-ARCHITECTURE-CIBLE.md).

Note au passage : le commit `d7e460a` affirme avoir éliminé `tiny-skia`. **C'est faux** — il est
importé dans 5 des 6 modules du desktop, et l'application dépend aujourd'hui de 7 crates directs
et ≈ 200 transitifs. Un historique qui ment t'empêche de savoir où tu en es (R-32).

### « Le glisser-déposer web ET fichiers, c'est impossible ? »

**Non. Et c'est le meilleur argument "from scratch" de tout ton projet.**

Ce n'est pas une limite de Rust ni de Tauri : c'est une limite de `winit`, qui n'expose que
`DroppedFile` avec un chemin, et **jette tout le reste**.

Or, quand tu fais glisser une image depuis un navigateur, Windows te propose **simultanément**
`CF_HDROP`, `CFSTR_INETURL`, `CF_HTML`, `CF_UNICODETEXT`, `CFSTR_FILECONTENTS` et `CF_DIB`.
En implémentant toi-même `IDropTarget` — environ 400 lignes — tu obtiens **fichiers locaux ET
images web dans le même geste, avec l'URL d'origine préservée** (le champ `source_url` de ton
modèle, qui n'a jamais été rempli).

Ici, écrire soi-même n'est pas une question de fierté : **c'est la seule façon d'avoir la
fonctionnalité.** Détail en [`02-ARCHITECTURE-CIBLE.md` § 0.1](02-ARCHITECTURE-CIBLE.md),
phase 5 de la roadmap.

### Les deux règles qui priment sur tout le reste

Elles sont détaillées dans [`05-STANDARDS-DE-CODE.md`](05-STANDARDS-DE-CODE.md), en tête des règles.

**R1 — Rien à moitié.** Une fonctionnalité livrée à moitié est pire que pas livrée : elle a l'air
de marcher. Une fonctionnalité n'est finie que si elle est **branchée**, **visible**,
**annulable** et **conservée à l'enregistrement**. Quatre conditions. Ce dépôt est un catalogue de
ce que coûte l'inverse — voir R-18, R-33, R-47.

**R2 — L'ancien code TypeScript n'est pas un modèle.** Il sert d'inventaire de ce qui existe, et
à rien d'autre. Son implémentation n'est ni portée, ni imitée, ni citée comme justification. Une
décision se défend par un raisonnement et une mesure. On refait tout en Rust, **mieux**.

### « Pourquoi j'ai l'impression d'être à 3 % ? »

Deux raisons, toutes les deux mesurées :

1. **10 boutons sur 19 n'ont aucun effet observable** — ils affichent un toast qui *décrit* une
   action qui n'a pas lieu. « 🎨 Préréglage PureRef appliqué » : rien n'est appliqué. Il y a
   **24 appels à `show_toast`** dans `app.rs`. Tu regardes l'écran et tu vois Glucose ; tu
   cliques et il n'y a rien derrière.

2. **24 fonctionnalités sont écrites mais débranchées.** Tu as fait le travail, il ne compte pas.
   C'est démoralisant précisément parce que l'effort a été fourni. Mesure exacte au commit
   `e2cd410` : **12 des 20 modules de `glucose-core` ne sont appelés par personne**, soit
   **3 601 lignes** écrites, testées, inatteignables (R-18).

3. **Ce qui s'affiche s'affiche mal.** Deux défauts dégradent *tout* le rendu, indépendamment des
   fonctionnalités : le texte ne suit pas le zoom (onze `clamp` bornent le contenu des cartes
   pendant que leur boîte se met à l'échelle librement — R-45), et les glyphes sont posés à des
   coordonnées entières tronquées, sans positionnement sous-pixel (R-46). Résultat : le
   « LOD » involontaire en dézoomant, et l'impression de flou.

4. **Une ligne du tableau de parité peut valoir 2 000 lignes de code.** « Collaboration : 0 % »
   pèse autant visuellement que « Storyboard : 0 % », alors que la première représente 1 920
   lignes de CRDT et la seconde 464. Le décompte par volume est dans
   [`03-PARITE-FONCTIONNELLE.md`](03-PARITE-FONCTIONNELLE.md) § *Ce que le score ne dit PAS* :
   **quatorze sous-systèmes, 19 670 lignes de TypeScript, dont aucun n'est utilisable.**

**Décision** : un bouton dont la fonction n'existe pas est retiré ou grisé. Jamais un toast qui
simule. C'est la phase 0, et elle ne coûte presque rien.

### « Pourquoi c'est si long de trouver et corriger un bug ? »

Parce que `window_event` fait **490 lignes avec 7 niveaux d'imbrication**, et que `app.rs`
mélange fenêtre, souris, clavier, presse-papiers, décodage d'images et algorithmes de mise en
page. Il n'existe aucune frontière où poser un point d'arrêt mental.

Et parce qu'il n'y a **aucun test** dans `glucose-desktop` (2 dans `canvas.rs`), **aucun type
d'erreur** dans tout le projet, et **aucune erreur visible** par l'utilisateur : quand une image
ne s'affiche pas, tu vois un rectangle gris et rien ne te dit pourquoi.

Règles correctives en [`05-STANDARDS-DE-CODE.md`](05-STANDARDS-DE-CODE.md).

### « Le code est moche. C'est réparable ? »

Oui, et le diagnostic est précis : **ce n'est ni le nommage ni l'indentation** — ils sont
corrects. C'est :

- **60 %** de répétition structurelle : créer une annotation demande 14 lignes dont 8 disent
  « rien », et ce bloc apparaît 4 fois dans `app.rs`. Cause : le modèle recopie ses champs
  communs dans les 4 variantes.
- **30 %** d'imbrication profonde.
- **10 %** de nombres magiques en double.

**La laideur est un symptôme de la structure, pas du style.** Une fois le modèle en composition
et l'UI en layout-unique, écrire du code beau devient le chemin le plus facile — pas un effort
en plus.

---

## Par où commencer, concrètement

Les points 1 à 8 de la liste d'origine sont **faits** : transform des images, grille adaptative,
culling, sélection élastique, layout unique, teinte mémorisée, boucle de rendu unique,
`smart_align` branché, générateur d'id en O(1), `app.rs` éclaté (1 126 → 438 l.). La frame est
passée de 240 ms à 17 ms. Voici la suite réelle, mesurée au commit `e2cd410`.

| Ordre | Action | Pourquoi maintenant | Poids |
|:--:|---|---|---|
| 1 | **La persistance** (R-01, phase 2) | **Rien n'est encore écrit sur disque.** Tant que ça dure, tout le reste est du travail qu'on perd en fermant la fenêtre. C'est la seule tâche qui transforme un prototype en logiciel. | 2 572 l. TS |
| 2 | **La fidélité du rendu** (R-45, R-46) | Ce que tu vois est faux à tout zoom ≠ 1, et le texte est flou. Ça dégrade les 22 % déjà acquis, donc ça coûte plus cher que ça n'en a l'air. | ~200 l. |
| 3 | **Brancher les 12 modules morts** (R-18, R-47) | **3 601 lignes déjà écrites et testées.** Les domaines sont l'exemple type : le noyau sait tout faire, l'interface écrit dans une liste fantôme. Meilleur rapport résultat/effort du projet. | déjà payé |
| 4 | **Retirer les 10 boutons qui mentent** (R-33, phase 0) | Sans ça tu ne peux pas mesurer ton avancement — et c'est la cause directe du ressenti « 3 % ». | quelques heures |
| 5 | **Rectangles sales** (R-42, tâche 1.13) | `docks` + `ui` = 59 % de la frame passés à redessiner l'immobile. | ~300 l. |
| 6 | **Texte riche : Markdown + KaTeX** | 2 229 l. TS, aucune ligne portée. C'est ce qui fait de Glucose un outil de pensée et pas un tableau d'images. | 2 229 l. TS |
| 7 | **Membranes** (3 modules morts + tween) | 2 661 l. TS ; 1 442 l. de noyau déjà écrites mais mortes. | 2 661 l. TS |
| 8 | **Flèches, dossiers, miroirs, temporalité, rideaux** | 5 167 l. TS ; noyau partiellement écrit, intégralement mort. | 5 167 l. TS |
| 9 | **Collaboration, plugins, télémétrie** | 4 179 l. TS, rien de porté. À traiter en dernier : ce sont les seuls sous-systèmes qui ne bloquent aucun usage solo. | 4 179 l. TS |

**Ce que cette table dit, et qu'il faut regarder en face** : les points 1 à 5 représentent
quelques milliers de lignes et rendraient le logiciel *utilisable*. Les points 6 à 9 représentent
**environ 14 000 lignes de TypeScript à porter** — c'est là qu'est le gros du chemin restant, et
aucune optimisation de performance ne le raccourcira.

Les points 2 à 6 représentent **moins de 400 lignes** et changent radicalement la sensation du
logiciel. Ne commence pas par le rastériseur maison : commence par les gains visibles.

⚠️ **Le point 7 mérite une alerte.** En deux commits, `window_event` a gagné 238 lignes et
`redraw()` 15 appels : les bonnes corrections s'entassent dans l'objet-dieu **faute d'un endroit
où les mettre**. À structure constante, chaque correction rend la suivante plus coûteuse.
C'est la seule dynamique de ce dossier qui va dans le mauvais sens.

---

## Les chiffres à retenir

*Au commit `81aea31`.*

| | |
|---|---:|
| Lignes Rust | 14 322 |
| Lignes de tests | 3 194 |
| **Modules de noyau morts** | **13 / 18** |
| Lignes de noyau inutilisées | ≈ 3 700 |
| **Parité fonctionnelle réelle** | **22 %** |
| Parité si on branche l'existant | **26 %** |
| Fonctionnalités inventoriées | 279 |
| Fonctionnalités « maquette » | 18 |
| Plus grosse fonction (`window_event`) | **728 lignes** ⚠️ |
| Appels directs à `redraw()` | **45** ⚠️ |
| Types d'erreur définis | **0** |
| **Chemins d'écriture de projet** | **0** |
| Boutons qui agissent | **9 / 19** |

---

## Comment maintenir ce dossier

- **`03-PARITE-FONCTIONNELLE.md`** est mis à jour à chaque fin de phase, avec les vrais chiffres.
  C'est ton tableau de bord.
- **`01-AUDIT-CODE-RUST.md`** : chaque constat `R-xx` est barré quand il est corrigé, avec le
  commit qui l'a fait. Le document devient un historique de dette remboursée.
- **`04-ROADMAP.md`** : les critères de sortie sont cochés. Une phase n'avance pas sans eux —
  c'est exactement le mécanisme qui manquait quand `smart_align` a été écrit puis oublié.
- **`docs/architecture/decisions/`** : une note par décision structurante, notamment toute
  nouvelle dépendance.

---

*Ce dossier ne vaut que s'il reste vrai. Un document d'architecture qui ment est pire que pas de
document du tout — c'est exactement le problème que ce dossier existe pour corriger.*
