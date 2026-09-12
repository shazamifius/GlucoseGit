# 10 — Plan de marche : de « ça marche » à « c'est juste »

> **Rôle de ce document.** Les fiches 01 à 09 décrivent *ce que Glucose est* et *ce qu'il
> devrait être*. Celle-ci répond à une autre question : **dans quel ordre, et qu'est-ce qu'on
> refuse ?**
>
> Elle est écrite contre la charte (§ 1), et elle **conteste** plusieurs décisions de
> [`02-ARCHITECTURE-CIBLE.md`](02-ARCHITECTURE-CIBLE.md) et de
> [`04-ROADMAP.md`](04-ROADMAP.md) — avec des mesures, pas des opinions.
>
> **Date** : 2026-09-12 · mesures prises sur l'arbre de travail (`c6fcaf6` + specs 06-09).
> **État vérifié à la main** : 168 tests verts, 0 warning, `cargo test --workspace` exit 0.

---

## 1. La charte — ce contre quoi tout se juge

| # | Exigence | Conséquence opérationnelle |
|:-:|---|---|
| **C1** | **Le moins de dépendances possible — mais le zéro n'est pas un objectif.** Si une capacité est inatteignable autrement, on prend la dépendance et on la prend *intelligemment*. | Une dépendance se défend par une impossibilité ou un coût démesuré. Jamais par le confort, jamais par la pureté inverse non plus. |
| **C2** | **Architecture et code quasi irréprochables.** | Pas d'objet-dieu, pas de module mort, pas de géométrie calculée deux fois, pas de bouton qui ment. |
| **C3** | **Minimum 100 fps sur n'importe quelle machine, sur n'importe quel écran.** | **Budget : 10 ms par frame**, 4K comprise, sur du matériel faible. |
| **C4** | **À l'arrêt, net à quasi 100 %.** Pendant le mouvement : à peser, tester, mesurer. | Le rendu se fait à la résolution physique. Et l'arbitrage du mouvement se **mesure**, il ne se tranche pas sur le papier. |
| **C5** | **Élégance mathématique** du fonctionnement et des optimisations. | Une constante arbitraire qui peut **disparaître** ne se règle pas : elle disparaît. |
| **C6** | **Viser la perfection.** | Une solution qui « marche » n'en est pas une si elle ferme la porte à la bonne. |

**Échelle cible** : des **dizaines de millions** de nœuds — pas des milliards — et **pas tout de
suite**. Le cas d'usage qui fixe le cap est le canva Wikipédia produit par `glucose-brain`
(§ 3.5). Le blocage actuel pour ce cas d'usage **n'est pas la performance, c'est le manque de
fonctionnalités de navigation**. Ce plan doit en tenir compte, et le § 4 en tient compte.

**Contrainte de fond** : les optimisations sont *exactes*. On ne dégrade jamais ce qui est
montré ; on évite de recalculer ce qui n'a pas changé. Naviguer dans l'immense se règle par des
outils sémantiques — domaines, couleur — pas par l'appauvrissement du rendu.

---

## 1 bis. Ce qu'on fait de tout ce qui est déjà écrit

Les fiches 01 à 09 ne sont pas un décor : elles ont chacune un statut, et ce statut décide de
l'autorité qu'on leur accorde.

| Fiche | Statut | Ce qu'on en fait |
|---|---|---|
| **01** Audit | **Diagnostic daté, partiellement périmé.** Re-mesuré ici : 11 modules morts et non 13, 44 toasts et non 24, et son verdict « panneaux qui simulent » ne vaut plus (RQ-0, § 2b). | Historique de dette. **On re-mesure avant de croire.** Jamais une source d'action directe. |
| **02** Architecture cible | **Décisions sur le *comment*.** Les 10 lois, le DAG de crates, le modèle en composition sont bons et on les garde. Trois points contestés : L7 (`f64`), « pas de GPU », rastériseur maison obligatoire. | Base de travail, amendée par RQ-1, RQ-2 et RQ-6. |
| **03** Parité | **Inventaire précieux, étalon invalide.** | Liste de ce qu'il ne faut pas oublier. Jamais une mesure du but (RQ-4). |
| **04** Roadmap | **Bon découpage, mauvais ordre.** | Le contenu des phases reste ; la séquence est révisée au § 4. |
| **05** Standards | **R1 (rien à moitié) et R2 sont excellentes.** Leur faiblesse est de reposer sur la discipline. | Gardées mot pour mot, et **rendues mécaniques** par les tests A.2–A.4. |
| **06 à 09** Specs | **La cible.** Elles font autorité sur le *quoi* : chaque couleur, chaque durée, chaque rang, chaque formule. | **Transformées en oracle exécutable** (ci-dessous). |
| **10** Ce document | Subordonné aux précédents sur le *quoi*, en désaccord assumé avec 02 et 04 sur le *comment* et l'ordre. | Se corrige dès qu'une mesure le contredit — deux corrections déjà inscrites. |

### Les fiches 06–09 deviennent des tests, pas de la documentation

C'est la leçon de RQ-0 : **le Rust a dérivé vers un thème cyan que personne n'a décidé, et la
fiche a dérivé vers un accent jaune qui n'a jamais existé.** Les deux dérives ont la même cause —
rien ne les vérifiait. La réponse n'est pas de mieux relire, c'est de rendre la vérification
mécanique, dans les **deux** sens :

```
   TypeScript (la vérité de référence : c'est lui qu'on veut à l'identique)
        ↑  test de fidélité de la spec   (aurait attrapé #eab308, absent du code)
   Fiches 06–09 (la cible écrite)
        ↑  test de conformité du Rust    (aurait attrapé le cyan #38bdf8)
   Rust
```

Concrètement, chaque fiche produit ses assertions :

| Fiche | Devient |
|---|---|
| **06** § 2 | Une table de tokens unique, et un test qui compare chaque champ de `Theme` à la valeur écrite. Un token qui change casse le build. |
| **06** § 3, 4, 6 | Tests de valeurs sur la grille ($G, R, lpha, P$), le cadre à +3 px, les poignées de 9 px, et la teinte symbiotique (djb2 → bruit 2000 px → moyenne vectorielle 1200 px). |
| **07** § 1, 2 | Les 11 constantes de `TIMING` et les courbes d'amortissement : une seule définition, testée. |
| **07** § 3 | Les rangs PICK 0→60, le cycle à 8 px / 2 500 ms : table de cas, comparée à `hit_priority`. |
| **07** § 4 | Le seuil de snap à 8 px écran, et l'invariant de décrochage franc de la § 4.1. |
| **08, 09** | Une ligne de spec = un test de parité fonctionnelle, coché seulement quand R1 est satisfaite (branché, visible, annulable, sauvegardé). |

**Et la référence ultime reste le TypeScript**, puisque l'objectif est de le reproduire à
l'identique : une capture PNG de la même scène dans les deux versions, comparée pixel à pixel,
est le seul juge qui ne peut pas dériver. C'est ce que `glucose-cli render-scene` (tâche 0.5 de
la fiche 04) permettrait, et c'est la raison pour laquelle cette tâche vaut bien plus que sa
place dans la liste ne le suggère.

> **Ce qui ne change pas, en revanche :** l'identique porte sur **le résultat visible**, jamais
> sur l'implémentation. Le code, lui, doit être meilleur — plus propre, plus optimisé, mieux
> structuré. Reproduire le rendu de `GlucoseCanvas.tsx` n'autorise à reproduire aucune de ses
> 4 181 lignes.

---

## 2. Ce que j'ai mesuré moi-même

Je n'ai pas repris les chiffres de l'audit sur parole.

| Mesure | Valeur | Méthode |
|---|---:|---|
| Tests | **168 verts, 0 échec** | `cargo test --workspace` |
| Warnings de compilation | **0** | idem |
| Modules de `glucose-core` **jamais appelés** par le desktop | **11 / 21** | `grep` sur chaque chemin de module |
| Appels à `show_toast` | **44** (l'audit en comptait 24) | `grep -r` |
| Taille de `dock.rs` / plus grosse fonction | **1 921 l. / 166 l.** | `wc`, comptage AST |
| Plus grosse fonction de `app.rs` | **75 lignes** | idem |
| `size_of::<BoardImage>()` | **392 octets** | test `size_of` compilé et exécuté |
| `size_of::<Annotation>()` | **512 octets** | idem |
| `size_of::<Id>()` (= `String`) | **24 octets** + tas | idem |
| Fichiers du desktop important `tiny_skia` | **20** | `grep -rl` |

### Trois dynamiques que l'audit ne dit pas encore

**a) `app.rs` est guéri ; la maladie a déménagé.** L'audit alertait sur `window_event` à 728
lignes : c'est réglé, la plus grosse fonction de `app.rs` fait 75 lignes. Mais `dock.rs` pèse
maintenant **1 921 lignes** avec quatre fonctions de 95 à 166 lignes. L'objet-dieu n'a pas été
dissous, il a changé d'adresse.

**b) Les toasts sont passés de 24 à 44 (+83 %) — mais le dock, lui, travaille vraiment.**
Vérification faite fonction par fonction : le Pomodoro décompte pour de bon (`tick_pomodoro`,
`Instant`, arrêt à zéro), Organize applique un vrai layout (`ApplyLayout`), Storyboard bascule
un état réel, et **`dock.rs` ne contient aucun `show_toast`**. Le verdict de l'audit — « des
panneaux qui simulent » — **ne s'applique pas au dock tel qu'il est aujourd'hui**. Les 44 toasts
sont ailleurs (`app.rs`, `interactions/`), et c'est là qu'il faut regarder, un par un.

**c) 11 modules du noyau sur 21 restent morts** : `arrow_anchor`, `curtain_model`,
`curtain_panel`, `export`, `geometry`, `membrane_focus`, `membrane_space`, `membrane_stretch`,
`mirror_graph`, `text_anchors`, `timeline`. ≈ 3 200 lignes écrites, testées, inatteignables.

---

## 3. Les remises en question

### RQ-0 — Le thème du Rust n'est pas celui de Glucose, et c'est l'écart le plus visible

La cible est **exactement** Glucose Tauri, visuellement et fonctionnellement. Or en comparant
`crates/glucose-desktop/src/theme.rs` (264 lignes) aux tokens de la fiche
[`06`](06-SPEC-RENDU-VISUEL-DESIGN-SYSTEM.md) § 2 et au code TypeScript réel, **aucun token ne
correspond**. Le commentaire du fichier le dit lui-même : *« Thème sombre sleek inspiré de
PureRef moderne »*. Ce n'est pas une implémentation imparfaite du design Glucose, c'est **un
autre thème**.

| Token | Cible (fiche 06 / TS) | `theme.rs` actuel | Écart |
|---|---|---|---|
| `canvas-bg` | `#0d0d0d` = (13, 13, 13) | (13, 14, 18) | fond bleuté |
| `surface-toolbar` | `#1a1a1a` = (26, 26, 26) | (20, 21, 26) α 250 | plus sombre, bleuté, translucide |
| `surface-panel` | `#161616` = (22, 22, 22) | (24, 24, 28) α 240 | plus clair, bleuté |
| `surface-btn-hover` | `#1e1e1e` = (30, 30, 30) | (39, 39, 42) | trop clair |
| **`surface-btn-active`** | **`#2d2d2d` = gris** | **(56, 189, 248) α 45 — cyan** | **couleur au lieu de gris** |
| `hairline-base` | `#2a2a2a` = (42, 42, 42) | (40, 42, 50) | bleuté |
| `hairline-active` | `#444444` = (68, 68, 68) | (60, 65, 75) | bleuté |
| **`hairline-smartguide`** | **blanc à 30 %, pointillé** | **(236, 72, 153) α 200 — rose** | **couleur et opacité fausses** |
| `text-bright` | `#ffffff` | (245, 245, 245) | légèrement gris |
| `text-muted` | `#888888` = (136, 136, 136) | (113, 113, 122) | plus sombre, bleuté |
| `surface-minimap` | `rgba(13, 13, 13, 0.92)` | (15, 16, 20) α 220 | bleuté |
| sticky jaune | `#f5c542` = (245, 197, 66) | (254, 240, 138) | jaune pastel très différent |
| **accent de chrome** | **aucun — monochrome strict** | **`#38bdf8` cyan, partout** | **viole la loi visuelle n° 2** |

Le cyan `#38bdf8` n'est pas un détail : il est branché sur `bg_active`, `border_accent`,
`text_accent`, `minimap_viewport` et `toast_border`. C'est lui qu'on voit sur chaque bouton
actif, sur le cadre de la minimap et sur les surlignages de panneaux — et **il n'existe nulle
part dans le TypeScript**, dont les boutons actifs sont `background: #2d2d2d; color: #fff;
outline: 1px solid #444`, strictement monochromes.

**Mais la fiche 06 n'est pas exacte non plus.** Elle proclame un « accent unique : le jaune
d'emphase `#eab308` ». Vérification : **`#eab308` a 0 occurrence dans tout `src/`.** La couleur
n'existe pas dans le logiciel qu'elle prétend décrire. À l'inverse, `#60a5fa` y apparaît
**40 fois** — deuxième couleur la plus fréquente du projet — et la fiche ne la mentionne que
comme teinte de l'opérateur logique « OU ».

> **Ce que ça dit de la méthode, et c'est le point le plus important de tout ce document :**
> une spécification que rien ne vérifie mécaniquement dérive — dans les deux sens. Le Rust a
> dérivé vers un thème cyan que personne n'a décidé ; la fiche a dérivé vers un accent jaune qui
> n'a jamais existé. Aucun des deux n'était malhonnête ; il manquait simplement un test.

**Conséquence pratique :** c'est le meilleur rapport résultat/effort du projet. `theme.rs` est
**un seul fichier de 264 lignes**. Le corriger contre les tokens exacts — et poser le test qui
compare chaque valeur à la fiche — change l'écran en entier pour un coût dérisoire, et **ne
dépend d'aucune des autres étapes**.

---

### RQ-1 — Le modèle de données ne tient pas les dizaines de millions

```
Annotation ..... 512 octets  ×  10⁷  =  5,1 Go de RAM
BoardImage ..... 392 octets  ×  10⁷  =  3,9 Go de RAM
```

Et ce sont les tailles **de pile seulement**. Chaque nœud porte en plus, sur le tas : un `String`
d'identifiant, un `Vec<String>` de tags, un `Vec<DomainAssignment>`, jusqu'à cinq
`Option<String>` — soit **500 à 700 octets réels et 3 à 8 allocations** par nœud posé.

> Un canva Wikipédia de 10 à 30 millions de nœuds demanderait **5 à 20 Go de RAM rien que pour
> le modèle**, avant tout rendu. Sur une machine modeste, le plafond réel du modèle actuel est
> de l'ordre de **1 à 2 millions de nœuds**.

La cause première tient en une ligne, `types.rs:6` :

```rust
pub type Id = String;
```

24 octets de pile, 16 de tas, une allocation, et une **comparaison en O(n) avec indirection
mémoire** à chaque test d'égalité — dans un logiciel dont la boucle chaude ne fait que comparer
des identifiants.

**Trois décisions, toutes structurantes, toutes irrattrapables si elles arrivent tard :**

1. **Identifiants entiers.** `NodeId(u32)` = un indice dans une arène. Comparaison en une
   instruction, 4 octets, aucune allocation, aucune collision par construction. Les chaînes
   lisibles ne survivent que dans le format de fichier.

2. **Tableaux de champs plutôt que tableau de structures** (*SoA*). Le culling ne lit que
   `x, y, w, h` : il doit balayer un tableau contigu, pas sauter de 392 en 392 octets à travers
   des champs dont il n'a que faire. Sur 10⁷ nœuds, c'est 160 Mo lus au lieu de 5 Go — sur un
   CPU faible, la différence entre quelques millisecondes et plusieurs secondes.

3. **Coordonnées en entiers à virgule fixe**, et non en `f64`.

Ensemble : **~32 octets par nœud au lieu de ~512**, soit **320 Mo pour 10⁷ nœuds**. Ça tient en
RAM sur une machine modeste — et le `mmap` cesse d'être obligatoire pour devenir un confort
(ouverture instantanée), ce qui **simplifie beaucoup** la persistance par rapport à une cible à
10⁹.

#### Pourquoi les entiers fixes plutôt que `f64` (loi L7 à réviser)

La fiche 02 pose : *« unités monde, `f64`, aucune exception »*. Bon réflexe sur l'unicité,
mauvais choix de type.

Un `i32` en unités de **1/256 pixel** donne une portée de ±8,4 millions de pixels — huit fois les
bornes anti-crash de ±10⁶ déjà retenues en [`09`](09-SPEC-SYSTEMES-METIER-ET-PERSISTANCE.md) —
pour une précision de **0,004 px**, invisible à tout zoom utile, et **4 octets au lieu de 8**.

Mais la mémoire est le moindre des trois gains. Les deux autres touchent C5 directement :

- **Les epsilons disparaissent.** Deux nœuds sont alignés, ou ils ne le sont pas. Le magnétisme
  accroche, ou il n'accroche pas. Plus aucun `< 1e-9` à calibrer nulle part.
- **L'associativité des déplacements devient un théorème.** `(x + d₁) + d₂ = x + (d₁ + d₂)`
  est exact en arithmétique entière ; en `f64`, il ne l'est qu'à l'arrondi près, et l'erreur
  s'accumule sur les milliers de petits pas d'un glisser long.

> **Correction d'une erreur de la première version de ce document.** J'avais présenté la
> *« session anti-dérive figée au grab »* de la fiche [`07`](07-SPEC-ANIMATIONS-ET-INTERACTIONS.md)
> § 4.1 comme une rustine contre l'erreur flottante, que les entiers feraient disparaître.
> **C'est faux.** En relisant la fiche en entier : la correction y est calculée depuis la boîte
> d'origine du geste et non depuis la frame précédente, pour que *« si on écarte la souris de
> plus de 8 px, la carte se décroche immédiatement et revienne exactement sous le curseur »*.
> C'est un invariant de **comportement** — le décrochage doit être franc — et il reste
> nécessaire en entiers, parce que re-snapper depuis une position déjà snappée crée une dérive
> *sémantique* qu'aucun type numérique ne corrige. L'argument des epsilons ci-dessus tient ;
> celui-là ne tenait pas.

Le `f64` reste à sa place : dans le transform caméra et le rastériseur, où l'on calcule des
pixels, pas des positions de document.

---

### RQ-2 — À 100 fps, le rendu purement CPU est arithmétiquement exclu

La fiche 02 conclut : *« ne fais pas de GPU maintenant : le CPU tient largement ce budget »*.
C'était défendable à 60 fps sur un écran de 1,3 Mpx. **À 100 fps, ce n'est plus un arbitrage,
c'est une impossibilité**, et le chiffre vient de votre propre `Cargo.toml` :

```
blit de 1,3 Mpx (≈ 1440×900) ......... 1,5 ms      →  1,15 ns / pixel
```

Budget C3 : **10 ms par frame**.

| Écran | Blit seul | Part du budget | Machine 3× plus lente |
|---|---:|---:|---:|
| 1440×900 | 1,5 ms | 15 % | 45 % |
| 1920×1080 | 2,4 ms | 24 % | **72 %** |
| **3840×2160** | **9,6 ms** | **96 %** | **287 % — hors budget** |

La lecture qui compte : **en 4K, la simple recopie du tampon vers l'écran consomme 96 % du
budget avant qu'un seul pixel de contenu ne soit dessiné.** Et sur une machine faible, même en
1080p, elle en consomme 72 %. Il n'y a aucune marge, à aucune définition intéressante.

Les rectangles sales (fiche 02 § 5.1) ne sauvent pas ce cas, et c'est le point que le plan actuel
rate : ils traitent le survol d'un bouton et le déplacement d'une carte — les gestes *rares*.
**Le geste dominant d'un canvas infini est le pan, et le pan invalide 100 % de l'écran à chaque
frame.** La loi L2 est écrite ; rien dans le plan actuel ne la rend vraie pendant le geste le
plus fréquent du logiciel.

#### Ce que le GPU est vraiment ici

Avec C1 tel que corrigé — le zéro n'est pas l'objectif — l'objection « c'est une dépendance »
tombe d'elle-même : c'est exactement le cas « on ne peut pas faire autrement, alors on fait
intelligemment avec ». Restent deux objections techniques, et la réponse est la même pour les
deux :

> **Le rastériseur produit les pixels. Le GPU ne fait que les transporter et les composer.**

Notre anti-aliasing reste le nôtre, exact, testable en CI par comparaison de PNG. Le GPU fait ce
qu'aucun CPU ne sait faire : déplacer des dizaines de mégapixels par frame pour presque rien.

**Sur le choix d'API — à mesurer, pas à décréter.** OpenGL 3.3 s'atteint par
`wglGetProcAddress` / `glXGetProcAddress` : ~300 lignes de `extern "C"`, aucune crate, disponible
sur tout matériel depuis 2010 — ce qui est précisément la définition de « n'importe quelle
machine ». Vulkan offre un meilleur contrôle mais impose en pratique un chargeur et de la
plomberie. Pour un usage qui se réduit à « téléverser des tuiles et dessiner des quads
texturés », OpenGL 3.3 paraît suffisant et nettement moins cher ; si la mesure dit le contraire,
Vulkan est parfaitement acceptable. **Le repli CPU reste obligatoire et testé en CI** — c'est lui
qui garantit le « n'importe quelle machine ».

---

### RQ-3 — La pyramide de tuiles : l'optimisation exacte qui rend C3 et C4 compatibles

C'est la pièce qui manque au plan actuel, et elle résout RQ-2 en même temps.

**Le mécanisme.** L'espace est découpé en tuiles de 256 × 256 pixels, à des paliers de zoom
dyadiques (…, ½, 1, 2, 4, …). Chaque tuile est rastérisée **une fois**, exactement, avec tout son
contenu, puis conservée. Une frame = composer les tuiles couvrant le viewport.

**Chiffré, sur un écran 4K, contre le budget de 10 ms :**

| Geste | Aujourd'hui | Avec tuiles |
|---|---:|---:|
| Rien ne bouge | 8,3 Mpx redessinés | **0** |
| Pan rapide (une rangée découverte / frame) | 8,3 Mpx | **≈ 1,0 Mpx** |
| Zoom sur un palier dyadique | 8,3 Mpx | **0** |
| Un nœud déplacé | 8,3 Mpx | 1 à 4 tuiles |

Le coût d'une frame devient **borné par la surface découverte**, strictement indépendant du
nombre de nœuds. La loi L2 cesse d'être un vœu et devient une propriété du pipeline.

**Ce n'est pas de la LOD**, et la distinction doit rester écrite noir sur blanc parce que le
vocabulaire dérive vite :

> Une tuile cachée contient le **rendu complet et exact** de son contenu, sans simplification,
> substitution ni seuil. On ne montre jamais une approximation d'un nœud ; on **évite de
> recalculer un résultat identique.** Dégrader *ce qui est montré* trahit la carte ; mémoïser ne
> touche qu'*au moment du calcul*.

**Le point de C4 — à mesurer, pas à trancher.** À l'arrêt, sur un palier, le rendu est exact :
net à 100 %, C4 satisfaite sans discussion. La question ouverte est le zoom *entre* deux paliers,
et elle appelle une expérience, pas une décision :

| À mesurer | Protocole |
|---|---|
| Combien de paliers par octave ? 1 (dyadique), 2 (√2), 4 ? | Plus de paliers = moins d'écart d'échelle donc moins d'adoucissement transitoire, mais mémoire de cache × n. Mesurer l'écart perceptible sur capture, et l'occupation réelle. |
| Le rendu direct à l'échelle exacte tient-il 10 ms pendant un zoom ? | Bancs sur 10⁵ / 10⁶ / 10⁷ nœuds, en 1080p et 4K, avec et sans GPU. Si oui, la question de l'adoucissement transitoire ne se pose même pas. |
| Un palier grossier construit par réduction de 4 tuiles fines est-il visuellement supérieur ou inférieur à un rendu direct ? | Comparaison de PNG. La réduction est un supersampling ×2 — elle pourrait être **meilleure** que le rendu direct, pas seulement moins chère. |

> Aucune de ces trois réponses ne se devine depuis un document. Elles se mesurent en étape C, et
> c'est exactement ce que l'étape C doit produire : **des chiffres, avant une décision.**

---

### RQ-4 — La roadmap se mesure à un étalon qu'elle déclare invalide

**Contradiction logique.** La règle R2 de la fiche 05 dit : *« l'ancien code TypeScript n'est pas
un modèle […] on refait tout en Rust, mieux »*. La fiche 04 gradue l'avancement en **pourcentage
de parité avec ce même TypeScript**, jusqu'à 100 %. On ne peut pas refuser un étalon et graduer
sa règle dessus.

Ce n'est pas qu'un problème de mot : un tableau qui plafonne à 100 % **interdit structurellement**
d'y inscrire ce qui ferait de la version Rust autre chose qu'une copie — les dizaines de millions
de nœuds, les 100 fps, la netteté 4K, le glisser-déposer web, le canva Wikipédia. **Aucune de ces
choses n'existe dans le TypeScript, donc aucune ne peut apparaître dans un score de parité.**
L'inventaire TS reste précieux comme liste de ce qu'il ne faut pas oublier ; il est disqualifié
comme mesure du but.

**Contradiction économique, plus coûteuse.** La roadmap prévoit de porter ~19 670 lignes de
TypeScript en phases 4 à 12, pendant que l'architecture cible reste diffuse dans ces mêmes
phases. Or RQ-1 établit que `Id`, les coordonnées et la disposition mémoire doivent changer.
**Tout code écrit avant ce changement sera réécrit après.**

```
Fondation d'abord  :  coût(fondation) + coût(portage)
Fondation ensuite  :  coût(portage) + coût(fondation) + coût(réécriture du portage)
```

Le troisième terme est du travail volontairement gaspillé. L'objection honnête — des semaines
sans rien de visible, sur un projet où le ressenti « je suis à 3 % » est déjà un problème nommé —
se traite en **livrant la fondation par substitution** : chaque étape du § 4 remplace une pièce
derrière une interface stable, et l'application tourne à la fin de chacune.

---

### RQ-5 — Ce que le canva Wikipédia impose vraiment, et que personne n'a encore chiffré

C'est le cas d'usage qui fixe la cible, donc il mérite d'être regardé en face.

**a) Le volume des nœuds est confortable ; celui des arêtes ne l'est pas.** Wikipédia anglais
compte de l'ordre de **7 millions d'articles** — soit, selon le découpage en sections ou en faits,
**10 à 30 millions de nœuds**. C'est exactement la cible, et RQ-1 la rend atteignable.

Mais le nombre de **liens internes entre articles est de l'ordre du milliard**. C'est là qu'est le
milliard, et il n'est pas dans les nœuds :

```
10⁹ arêtes × 32 octets  =  32 Go
```

Trois conséquences, toutes à trancher avant d'écrire le moteur de flèches :

- Les arêtes ne peuvent pas toutes être matérialisées. Il faudra **filtrer** (liens réciproques,
  liens de la section principale, pondération par co-occurrence) ou **dériver à la demande**.
- La spécification des flèches en [`08`](08-SPEC-FONCTIONNALITES-CANVAS-ET-NOEUDS.md) prévoit un
  **évitement dynamique d'obstacles** (`getDynamicRoute`, marge 32 px). C'est une belle
  fonctionnalité à l'échelle humaine et une **impossibilité algorithmique** à 10⁸ arêtes. Il
  faudra deux régimes de tracé, explicitement séparés — pas un seul qu'on espère voir tenir.
- **Une arête longue traverse des milliers de tuiles.** C'est le point faible du design tuilé
  de RQ-3 : déplacer un nœud relié à l'autre bout de la carte invaliderait des milliers de tuiles.
  Il faut probablement une **couche d'arêtes longues séparée**, composée par-dessus les tuiles
  plutôt que gravée dedans. À concevoir en étape C, pas à découvrir en étape G.

**b) `glucose-brain` contient la même contradiction qu'il cherche à éviter.** Sa décision
d'architecture n° 1 est excellente et bien argumentée : *un document Automerge par nœud, jamais
un graphe monolithique*. Mais l'**index**, lui, est un document Automerge unique
(`register_node` écrit dans une seule `Map` sous `ROOT`). À 10⁷ nœuds, **l'index est exactement le
monolithe que la décision n° 1 refuse** — avec son historique CRDT, ses métadonnées par clé et sa
sérialisation entière à chaque chargement.

La sortie est nette, et elle réconcilie les deux projets :

> **L'arène compacte de Glucose *est* l'index.** Projection dense, ids entiers, positions en
> entiers fixes, conçue pour 10⁷ entrées. Les documents Automerge restent la vérité du *contenu*
> d'un nœud, chargés à la demande quand on l'ouvre — jamais pour dessiner la carte.

Ce qui veut dire que la persistance de Glucose (étape F) et l'index de `glucose-brain` ne sont
pas deux chantiers, mais un seul. À vérifier avec une mesure avant de s'y engager : combien pèse
réellement un document Automerge minimal, sur disque et en mémoire une fois chargé ?

**c) Et le vrai blocage n'est aucun de ces chiffres.** Vous l'avez dit : ce qui manque pour
naviguer dans Wikipédia, ce sont **les fonctionnalités** — les domaines, l'usage approfondi de la
couleur, les membranes. Aucune optimisation ne les remplace. Le § 4 en tire la conséquence.

---

### RQ-6 — Auto-critique : la version précédente de ce plan sur-investissait dans le zéro dépendance

À écrire honnêtement, puisque la règle vaut aussi pour moi. La première version de ce document
ordonnait les étapes autour du retrait de `winit`, `softbuffer`, `arboard`, `rfd`, `fontdue` et
`tiny-skia`. Avec C1 corrigé, **cette justification tombe**, et l'ordre change :

| Pièce | Justification v1 (caduque) | Justification qui reste |
|---|---|---|
| `winit` → plateforme maison | « retirer une dépendance » | **Aucune** pour la fenêtre. Mais le glisser-déposer web exige un `IDropTarget` maison : on l'**ajoute à côté** de `winit`, on ne le remplace pas. Bien moins cher. |
| `softbuffer` | « retirer une dépendance » | Remplacé de fait par la présentation GPU (RQ-2), pas pour la pureté. |
| `tiny-skia` → rastériseur maison | « c'est le morceau from-scratch le plus satisfaisant » | **À justifier par la mesure.** `tiny-skia` sait rastériser dans un `Pixmap` de 256×256 : il peut alimenter le cache de tuiles dès le premier jour. Le rastériseur maison devient un gain de qualité et de contrôle à démontrer, **pas un prérequis**. |
| `fontdue` | « retirer une dépendance » | À réévaluer si la qualité du texte l'exige. Pas avant. |
| `image` | déjà admise comme exception | Inchangée : décodeur = octets hostiles, risque pour gain nul. |

Conséquence directe : **l'étape la plus lourde de la v1 (réécrire la plateforme) disparaît**, et
le temps libéré va aux fonctionnalités de navigation, qui sont le vrai blocage (RQ-5c).

---

## 4. L'ordre de marche

Six étapes. Chacune se termine sur une application qui tourne et sur un **critère mesurable qui
échoue le build s'il régresse**.

### Étape A — Arrêter l'hémorragie *(courte, et elle conditionne tout le reste)*

La phase 0 de la fiche 04, mais avec des garde-fous **automatisés** — sans quoi elle sera défaite
comme elle l'a déjà été (24 → 44 toasts).

| # | Travail |
|---|---|
| A.1 | **Inventaire fonction par fonction du dock, avant toute suppression.** Vérification faite : le Pomodoro **fonctionne réellement** (`tick_pomodoro`, `Instant`, arrêt à zéro), Organize applique un vrai layout (`ApplyLayout`), Storyboard bascule un état — et `dock.rs` ne contient **aucun** `show_toast`. La recommandation « supprimer 550 lignes qui mentent », héritée de l'audit et reprise telle quelle dans la v1 de ce document, **aurait détruit du code qui marche**. Ce qui reste à traiter : les boutons dont le clic n'a effectivement aucun effet observable, identifiés un par un. Grisé + infobulle, jamais un toast qui simule. |
| A.2 | Test CI : `show_toast` ne décrit que ce qui vient d'avoir lieu — liste blanche des messages, revue à chaque ajout. |
| A.3 | Test CI : **aucun module de `glucose-core` sans appelant.** Le graphe d'appel est vérifié, pas affirmé. |
| A.4 | Test CI : aucune fonction > 80 lignes, aucun fichier > 600 lignes. `dock.rs` tombe le premier. |
| A.5 | Bancs d'essai : 10³ / 10⁵ / 10⁶ / 10⁷ nœuds × zoom 1 / 0,1 / 0,01, en 1080p **et 4K**, **mesurés contre le budget de 10 ms**. Chiffres publiés, build en échec au-delà de 10 % de régression. |

> **Sortie** : la CI refuse mécaniquement les cinq dérives du § 2. Tant que A.3 et A.4 ne sont pas
> là, toute autre étape ne fait qu'ajouter à la dette.

---

### Étape B — La fondation : géométrie entière, arène compacte, index spatial

La seule étape irrattrapable. Tout ce qui suit en dépend.

| # | Travail |
|---|---|
| B.1 | `glucose-geom` : coordonnées `i32` en 1/256 px, rectangles, transform caméra, prédicats d'intersection exacts. |
| B.2 | `glucose-model` : arène SoA, `NodeId(u32)`, tronc commun + `kind` (le modèle en composition de la fiche 02 § 3.1, enfin appliqué), invariants testés. |
| B.3 | Index spatial adapté à 10⁷ entrées, remplaçant `quadtree.rs` — dont les 451 lignes servent de banc de comparaison. Ordre de stockage à localité spatiale (courbe de Hilbert) : bénéfice mesuré d'abord, adopté ensuite. |
| B.4 | Migration mécanique des 10 modules vivants du noyau. **Les algorithmes ne changent pas** : `smart_align`, `hit_priority`, `resize`, `symbiotic_hue` sont bons et restent. Seuls leurs types changent. Les 168 tests doivent repasser verts. |
| B.5 | Générateur de documents synthétiques reproductibles : 10⁶, 10⁷ nœuds, et un jeu « forme Wikipédia » (distribution des degrés en loi de puissance, arêtes longues). |

> **Sortie** : 10⁷ nœuds chargés, **mémoire résidente < 500 Mo**, requête de viewport < 1 ms, les
> 168 tests verts, `cargo tree -p glucose-model` ne listant que `glucose-geom`.

---

### Étape C — Tuiles, présentation GPU, et les mesures qui tranchent C4

| # | Travail |
|---|---|
| C.1 | Cache pyramidal de tuiles 256×256 : clé = (palier, tuile x, tuile y, empreinte du contenu), éviction LRU bornée **en octets**. Rastérisation par `tiny-skia` dans un premier temps — le rastériseur maison viendra s'il se justifie (RQ-6). |
| C.2 | Invalidation exacte : un nœud modifié salit exactement les tuiles qu'il recouvre, à tous les paliers. C'est l'endroit où une erreur devient un artefact permanent — donc le plus durement testé. |
| C.3 | **Couche d'arêtes longues** composée par-dessus les tuiles plutôt que gravée dedans (RQ-5a). |
| C.4 | Présentation GPU par FFI (OpenGL 3.3 d'abord, Vulkan si la mesure l'impose), **avec repli CPU obligatoire et testé en CI par défaut**. |
| C.5 | **Le protocole de mesure de RQ-3** : paliers par octave, rendu direct vs réduction, tenue des 10 ms pendant le zoom. Produit un tableau de chiffres — c'est lui qui décide, pas ce document. |
| C.6 | Non-régression visuelle : tuiles CPU composées en GPU ≡ scène rendue entièrement en CPU, **octet pour octet**. |

> **Sortie** : pan et zoom **≥ 100 fps en 4K sur 10⁷ nœuds** ; 0 % de CPU au repos ; netteté
> vérifiée par capture à l'arrêt ; l'égalité C.6 tenue ; **et le tableau de mesures de C.5 publié**.

---

### Étape D — Les fonctionnalités de navigation *(le vrai blocage)*

Remontée ici, et pas en fin de parcours, parce que c'est ce qui manque réellement pour le canva
Wikipédia : **domaines**, usage approfondi de la **couleur**, **membranes**, recherche, et les
11 modules morts enfin branchés — dont `membrane_space`, `membrane_focus`, `membrane_stretch`
(1 442 lignes déjà écrites et testées).

> **Sortie** : sur un jeu synthétique de 10⁶ nœuds « forme Wikipédia », on retrouve une
> information ciblée en moins de 30 secondes sans connaître sa position. **Critère d'usage, pas
> de parité.**

---

### Étape E — Le texte, cœur de Glucose

Glucose est un outil de pensée écrite avant d'être un moodboard. Mise en page pure et testable
sans écran, Markdown, maths, positionnement sous-pixel (l'acquis 1.28 est conservé).

> **Sortie** : texte net à tout zoom, prouvé par capture PNG ; la mise en page testée sans
> dessiner.

---

### Étape F — Persistance, entrée universelle, et la jonction avec `glucose-brain`

Format de fichier conçu en B, implémenté ici ; journal d'ajout pour l'undo et l'autosave ;
écriture atomique ; bundle portable dédupliqué. Plus le `IDropTarget` maison (RQ-6) qui débloque
**le glisser-déposer web et fichiers dans le même geste** — le besoin n° 1, et la démonstration
de la fiche 02 § 0.1 reste valable telle quelle.

Et la jonction de RQ-5b : l'arène de Glucose devient l'index de `glucose-brain`, les documents
Automerge restant la vérité du contenu, chargés à la demande.

> **Sortie** : 10⁷ nœuds ouverts en moins d'une seconde ; glisser une image depuis un navigateur
> pose l'image **et** renseigne `source_url` ; un aller-retour `glucose-brain` → Glucose →
> `glucose-brain` sans perte, mesuré.

---

### Étape G — Le reste du portage fonctionnel

Flèches sémantiques, dossiers et miroirs, temporalité, storyboard, export, rideaux,
collaboration. Le gros du volume, désormais **écrit une seule fois**. L'ordre interne des phases
6 à 12 de la fiche 04 reste pertinent ; seule sa place change. Et la mesure change avec : on
compte des fonctionnalités **livrées selon R1** — branchées, visibles, annulables, sauvegardées —
pas des pourcentages de ressemblance au TypeScript.

---

## 5. Ce qui pourrait faire dérailler ce plan

| Risque | Signal précoce | Réponse |
|---|---|---|
| La migration B.4 s'éternise | Les 168 tests restent rouges plusieurs jours | Module par module derrière un adaptateur de types, jamais tout d'un coup |
| Bug subtil d'invalidation de tuiles (C.2) | Une capture CI diverge d'un pixel | C.6 est la ligne de défense : ne jamais l'assouplir pour faire passer un build |
| Le repli CPU est négligé puis pourrit | Il n'est plus dans la CI | Le faire tourner **par défaut**, le GPU en second |
| L'étape A est défaite une fois de plus | Un panneau apparaît pour une fonctionnalité absente | A.2–A.4 sont mécaniques : ils ne dépendent pas de la discipline |
| Les arêtes longues font s'effondrer le cache | Le taux de tuiles invalidées explose sur le jeu « forme Wikipédia » | C'est précisément ce que B.5 et C.3 existent pour attraper — en étape C, pas en étape G |
| 10⁷ s'avère hors d'atteinte sur le matériel réel | Les bancs de B.5 plafonnent avant | Le savoir en B, pas en F. C'est tout l'intérêt de mesurer tôt. |

---

## 6. Ce qui reste ouvert, et comment on le fermera

Aucune de ces questions ne se répond depuis un document. Chacune a son protocole.

1. **Le zoom entre paliers** (C4 pendant le mouvement) → tableau de mesures C.5.
2. **OpenGL 3.3 ou Vulkan** → mesure sur matériel faible réel, avec le repli CPU comme témoin.
3. **Rastériseur maison ou `tiny-skia`** → comparaison de qualité et de débit sur tuiles, une
   fois le cache en place. Décidé par chiffres, pas par goût du from-scratch.
4. **Combien d'arêtes Wikipédia matérialiser** → dépend du filtre retenu ; à instruire avec
   `glucose-brain` avant l'étape G.
5. **Coût réel d'un document Automerge** (disque et mémoire chargée) → une mesure d'une heure,
   qui conditionne toute l'architecture de la jonction RQ-5b.

---

*Ce document se juge comme les autres : à sa capacité à rester vrai. Chaque chiffre du § 2 est
reproductible en une commande. Le jour où l'un d'eux devient faux, c'est ce document qu'il faut
corriger — pas la mesure.*
