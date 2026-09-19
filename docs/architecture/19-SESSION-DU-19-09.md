# 19 — Le temps que l'écran montre : ce que la session du 19/09 a trouvé

> **Rôle de ce document.** La fiche [`18`](18-PLAN-R-ET-D.md) posait le plan. Celle-ci dit ce
> que la journée en a fait — et surtout ce qu'elle a **trouvé en chemin** qui n'y était pas :
> le défaut que quatre tentatives avaient cherché au mauvais endroit, un mode de présentation
> que personne n'avait choisi, et une promesse fausse dans la mesure elle-même.
>
> **Date** : 2026-09-19 · cinq commits, de `6b7c864` à `e32f428`.
> **État vérifié** : `cargo test --workspace` exit 0, **1 294 tests verts**, clippy strict à
> zéro, dix cliquets mécaniques.
>
> **Le point de départ, mot pour mot** : « quand on freine progressivement, on voit tout en
> genre 4 fps », alors que la chronique affiche cent images par seconde. Et : les traces sont
> « pas assez complètes », « trop faciles et superficielles ».

---

## 1. Ce qui a été fait, en une table

| Chantier | Ce que c'est | Vérifié par |
|---|---|---|
| **Le rythme** (`chronique/rythme.rs`) | La chronique mesure enfin ce que l'écran **montre** : l'intervalle entre présentations, la fidélité `temps intégré / temps montré`, le saut de position en pixels, le nombre de balayages par image, et le temps passé à **ne pas dessiner** | tests, et deux sessions réelles |
| **Le verdict** (`chronique/verdict.rs`) | Le rapport **conclut** : chaque défaut comparé à une référence du projet, trié par gravité, muet quand rien ne dépasse | tests |
| **Le mode de présentation** (`present/gpu/succession.rs`) | L'application tournait en `Immediate` — aucune synchronisation verticale — depuis toujours. Elle tourne en `fifo` | bannière de démarrage, chronique |
| **L'horloge** (`horloge.rs`) | La trajectoire avance du temps que l'écran a **montré**, quantifié sur ses balayages, et non du temps de calcul. Deux horloges et deux bornes de 100 ms disparaissent | 9 tests, dont un qui porte sa **limite** |
| **La grille** (`renderer/grille.rs`) | TUILE-1 branché : les photos se posent depuis les tuiles, en trois régimes | 7 tests, `bench_grille`, `bench_freinage` |
| **La composition** (`report.rs`, `tuiles.rs`) | Chaque tuile porte sa boîte utile et son opacité ; le source-over traite les plages opaques par copie et saute les transparentes | test d'égalité au bit près |
| **L'histogramme** (`chronique/histogramme.rs`) | Le découpage était écrit deux fois et promettait 19 % d'erreur en donnant 25. Il est unique, géométrique, et un test balaie l'étendue entière | tests |

---

## 2. Le défaut, et pourquoi quatre tentatives ne pouvaient pas le trouver

La position affichée à l'image `n` vaut `x(n−1) + v · pas(n)`. Elle reste sous les yeux
pendant `Δ(n)`, l'intervalle entre deux présentations. La vitesse que l'œil mesure vaut donc :

```
    v_apparente(n) = v · pas(n) / Δ(n)
```

Elle est constante — le mouvement paraît régulier — **si et seulement si `pas(n) = Δ(n)`**.

Or `pas` valait le temps mural entre deux **débuts de rendu**, et `Δ` est l'intervalle entre
deux **présentations**. Les deux diffèrent exactement de la variation du coût d'une image :
6 ms en médiane, 67 ms au pire, sur le terrain. À mille pixels par seconde, le contenu se
posait jusqu'à soixante pixels à côté de sa trajectoire, pour sept pixels d'avance attendue.

**Le modèle du mouvement était exact. C'est son horloge qui ne l'était pas.** Les quatre
tentatives précédentes ont toutes modifié `elan.rs` — et l'utilisateur avait validé le frein
(« c'est même parfait ») avant la troisième. Elles corrigeaient un modèle juste.

Et aucune mesure de coût ne pouvait le voir : un mouvement parfaitement régulier, rendu par une
machine irrégulière, est vu saccadé pendant que les totaux, les centiles, la cadence et la
latence restent tous excellents.

### Ce qui a été corrigé, et ce qui ne l'est pas

Deux causes se composent, et il faut les nommer séparément.

**L'horloge** — corrigée. Le pas vient de l'intervalle de présentation observé, quantifié sur
la grille de balayage avec report de la fraction (Bresenham sur le temps, sans dérive). La
trajectoire ne dépend plus du tout de ce qu'une image coûte.

**La variance du coût** — réduite, pas supprimée. Tant que `Δ` varie d'une image à l'autre,
`pas(n) = Δ(n−1)` fait encore varier la vitesse apparente, décalée d'un cran. Un test le dit
(`test_une_cadence_qui_varie_fait_encore_varier_la_vitesse_apparente`) plutôt qu'un
commentaire, parce qu'un commentaire ne casse pas la build le jour où quelqu'un croit le
contraire. C'est la grille qui attaque cette cause.

---

## 3. Les chiffres

### Le banc du freinage — `bench_freinage`, nouveau

Trente-six photos, un glissement à 1 200 px/s qui s'éteint sur cent vingt images, 2560 × 1600.
La colonne qui compte est la dernière : l'écart entre deux images **consécutives**, celui que
l'œil reçoit.

| régime | médian | max | pire saut |
|---|---:|---:|---:|
| direct — ce que l'application faisait hier | 23 ms | 35 ms | **18 ms** |
| grille, échelle exacte | 7,5 ms | 13 ms | **7 ms** |
| grille, entre deux niveaux, au plus proche | 7,8 ms | 12 ms | **6 ms** |

Trois fois moins cher, un saut deux fois et demie à trois fois plus petit. Le saut restant
vient de la variance de la composition elle-même — 3,3 à 12 ms pour le même travail — et non
des tuiles à peindre : six tuiles coûtent 1,2 ms.

### Deux sessions réelles, sur canevas vide

| | avant | après |
|---|---:|---:|
| succession des images | `immediate` | `fifo` |
| images par seconde, consécutives | 173 | 205 |
| images sur un seul balayage | 72 % | 92 % |
| pire gel | 706 ms | 1 261 ms *(démarrage, voir § 5)* |

---

## 4. Ce que la mesure a démenti, dans l'ordre

Chaque fois, c'est un test ou un banc qui a protesté, et il avait raison.

### 4.1 L'histogramme promettait 19 % et donnait 25

Quatre tranches par octave bornent l'erreur à `2^(1/4) − 1` — **si les tranches sont
géométriques**. Les deux copies du découpage coupaient l'octave en quatre parts égales : la
première sous-tranche a un rapport de 1,25. Le test qui vérifiait la promesse ne prenait que
quelques valeurs bien choisies ; celui qui balaie l'étendue entière l'a vue au premier essai.

Et un centile pouvait dépasser le maximum observé : « p99 77,9 ms, pire 66,8 ms » était
imprimé. Il est borné.

### 4.2 Le résumé annonçait « au moins 32 images ratées », toujours

Il comptait sur la liste bornée des trente-deux plus lentes. Une session qui en rate mille et
une qui en rate trente-trois disaient le même nombre. Il compte sur toutes.

### 4.3 `ce_que_porte` prenait le centre d'une photo pour son coin

Une photo à cheval sur deux tuiles n'était comptée que dans celle de droite, et sa moitié
gauche disparaissait de l'écran. La preuve de redimensionnement l'a attrapé au premier
branchement : « bord à 500 px, le document dit 400 ». `bench_grille` avait le même défaut et
annonçait « zéro écart aux coutures » — il mesure désormais l'**amplitude** des écarts, qui
vaut 4/255 : l'arrondi de deux compositions, pas une couture.

### 4.4 Le pic du freinage n'était pas le rendu des tuiles

Le poste « occlusion » paraissait coûter trois fois le rendu. Il absorbait tout ce qui se
passait depuis la marque précédente — la composition des tuiles déjà peintes, les empreintes.
Marques posées où les postes changent de nature : six tuiles coûtent 0,2 ms d'occlusion et
1 ms de report. Et le pic de 33 ms était la **première** image du banc, celle qui peint
cinquante-quatre tuiles d'un coup : le démarrage, pas le geste.

### 4.5 L'écriture de la chronique n'était pas le gel

Hypothèse plausible — elle se fait sur le fil de l'interface — et mesurée fausse en une
ligne : 0,5 ms. Elle reste sur ce fil, et c'est une dette nommée, mais elle n'est pas le
sujet.

### 4.6 Un sommeil n'est pas un gel

« Le pire gel : 3 092 ms à la 7,7ᵉ seconde » sur une session où personne ne touchait à rien.
L'application dormait. Le rythme ne mesure un intervalle que si l'image précédente avait
demandé la suivante, ou si un geste attend.

---

## 5. Ce qui reste, et ce que la mesure désigne

### 5.1 Le gel de démarrage — décomposé, pas résolu

À la 1,2ᵉ seconde de chaque session, un gel de 1 250 ms dont **1 000 ms hors de tout code
Glucose**. Ce qui est dans notre code : deux `Resized` qui reconfigurent la surface (191 et
37 ms) et le premier `blit` (228 ms). Le reste est le système ou le pilote. C'est l'étape 5 du
plan 18, et elle est maintenant datée et décomposée.

### 5.2 La composition entre deux niveaux — 4 à 10 ms

L'agrandissement au plus proche de quatre millions de pixels coûte 1 à 2 ns par pixel. Deux
pistes, à mesurer avant de choisir :

* rendre les tuiles au niveau **supérieur** et les réduire au plus proche — meilleure qualité
  visuelle, quatre fois plus de mémoire ;
* rendre les tuiles à l'**échelle exacte de la vue** pendant un glissement à échelle fixe, la
  composition redevenant pixel pour pixel — l'échelle dyadique ne servant plus qu'au zoom.

### 5.3 La variance résiduelle

`clear` varie de 0,7 à 2 ms, `ui` de 0,6 à 2 ms, pour le même travail. Une partie est le
système ; une partie est du travail refait à l'identique — l'interface se redessine à chaque
image alors qu'elle ne change pas. C'est l'étape 1 du plan 18, la salissure, et elle est
désormais la première chose que la variance désigne.

### 5.4 Un choix de ressenti, à juger à l'écran

Entre deux niveaux dyadiques, pendant que la vue bouge, la grille sert **même quand l'œil ne
tolère plus qu'on abîme** — parce que repasser au rendu direct à trente millisecondes ferait
sauter le contenu de soixante pixels, ce qui se voit infiniment plus qu'un agrandissement d'un
facteur un virgule trois. La finesse revient à l'arrêt, d'un coup. C'est écrit dans `Regard`,
et c'est là que ça se défait si l'écran dit le contraire.

---

## 6. Trois leçons de plus, pour la liste de la fiche 17

1. **Une mesure qui ne dit pas où elle a été prise ne se relit pas.** La bannière de démarrage
   écrivait `(Immediate)` à chaque lancement. Personne ne l'a lu, parce qu'aucune trace ne le
   portait à côté des chiffres qu'il rendait faux.
2. **Un test de propriété balaie l'étendue ; un test de valeurs choisies confirme ce qu'on
   croit.** L'histogramme a passé le second pendant des semaines.
3. **Une marque de mesure absorbe tout ce qui la précède.** Un poste qui paraît trois fois
   plus cher que le travail qu'il nomme est une marque mal posée avant d'être un défaut.
