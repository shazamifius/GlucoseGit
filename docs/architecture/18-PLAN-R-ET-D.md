# 18 — Cent quarante images par seconde, toujours : le plan

> **Rôle de ce document.** Les fiches [`15`](15-PLAN-DE-PERFORMANCE.md) et
> [`17`](17-SESSION-DU-18-09.md) disaient comment aller plus vite avec l'architecture qu'on a.
> Celui-ci dit **pourquoi cette architecture ne peut pas tenir la promesse**, et par quoi la
> remplacer.
>
> **Date** : 2026-09-19. Écrit après une session de terrain où l'utilisateur a tranché sur
> capture d'écran : « c'est ultra pixelisé, sur un écran comme le mien ça passe pas ; déjà ça
> lag, et ensuite c'est moche ». Puis : « la façon dont on procède est complètement contraire
> à nos règles, rien d'élégant mathématiquement, complètement bourrin ».
>
> **Il a raison, et ce document part de là.**

---

## 1. Ce qui est faux aujourd'hui, et ce n'est pas un réglage

Quatre défauts, et ils sont de **conception**, pas d'exécution.

### 1.1 On réagit là où la charte demande de prédire

Le modèle de coût COUT-1 prévoit bien ce qu'une image va coûter — il est juste à 98-101 %.
Mais ce qu'on en fait est réactif : *si ça dépasse, on abîme*. La prévision sert de
déclencheur à une dégradation, jamais à **préparer du travail à l'avance**.

La charte demande l'inverse, mot pour mot : « fais en sorte que mathématiquement le logiciel
développe une intelligence et comprenne la machine afin de pré-optimiser des rendus ».

### 1.2 La dégradation est globale, donc brutale

Réduire la résolution divise **toute la scène**, y compris ce qui ne coûte rien et ce que
l'œil fixe. C'est le levier le plus grossier qui existe : il traite une carte de texte lisible
et un mur de quatre cents photos exactement pareil.

### 1.3 La finesse ne suit pas la vitesse — le défaut que l'utilisateur a nommé

Mot pour mot : « le fait qu'on a un smooth qui nous ralentit peu à peu, et bien la
pixelisation continue ».

C'est exact, et c'est une faute de modèle. La finesse est décidée par un **budget de temps**.
Or ce qui rend une image grossière acceptable n'est pas le temps qu'elle a coûté : c'est la
**vitesse à laquelle elle défile sous l'œil**. Pendant un glissement rapide, l'œil ne résout
pas les hautes fréquences ; pendant la fin d'un amortissement exponentiel, il les résout de
nouveau — et la pixelisation, elle, reste tant que le budget reste tendu.

Deux grandeurs sans rapport ont été confondues. Il faut les séparer.

### 1.4 Rien n'est réutilisé d'une image à l'autre

Toutes les images portent `redessine 100%`. Le cache de vignettes sert à 6 %. Le mécanisme de
rendu par région existe, et **un seul appelant sur soixante-huit** sait désigner sa zone.

Chaque image repart donc de zéro sur une scène qui, entre deux images, a bougé de trois
pixels. C'est là que passe l'essentiel du travail, et aucun réglage de budget n'y touche.

---

## 2. Le mur arithmétique, mesuré

Avant toute conception, le plancher physique. `bench_bande_passante`, sur la machine de
l'utilisateur :

| définition | effacer | recopier | **incompressible** | reste à 140 fps |
|---|---:|---:|---:|---:|
| 1440 × 900 | 0,19 ms | 0,19 ms | 0,38 ms | 6,76 ms |
| 2560 × 1600 | 0,69 ms | 0,65 ms | **1,34 ms** | 5,80 ms |
| 3840 × 2160 (4K) | 2,55 ms | 2,22 ms | **4,77 ms** | 2,37 ms |
| 7680 × 4320 (8K) | 11,24 ms | 11,99 ms | **23,23 ms** | **impossible** |

Ces colonnes ne dessinent **rien**. Elles remplissent l'écran d'une couleur et le recopient.

Trois conséquences qui ferment le débat :

1. **En 4K, il reste 2,37 ms pour tout le contenu.** Toute architecture qui repeint l'écran
   entier à chaque image est hors-jeu avant d'avoir commencé.
2. **En 8K, aucune cadence n'est atteignable** si l'on repeint tout — et la charte interdit
   d'exclure une machine.
3. Donc **le rendu par région n'est pas une optimisation, c'est une condition d'existence.**

---

## 3. Les trois lois, et elles remplacent toutes les constantes

L'exigence de la charte est explicite : une constante arbitraire qui peut **disparaître** doit
disparaître. Chacun des trois mécanismes ci-dessous remplace un seuil choisi par une loi
mesurable.

### Loi I — la finesse se déduit de la vitesse, pas du budget

La psychophysique donne le seuil, et il n'est pas de notre invention : **l'acuité visuelle
commence à chuter au-delà d'environ 2°/s de glissement rétinien, et les hautes fréquences
spatiales sont perdues au-delà de 5 à 6°/s.** Au-delà, une image plus fine n'est pas vue —
la produire est du travail jeté.

La poursuite oculaire complique à peine les choses : quand l'œil suit le contenu (gain 0,9 à
1,0 jusqu'à 20°/s), le glissement rétinien est **faible même à grande vitesse**, donc le
contenu reste net à l'œil. Mais l'œil ne peut suivre qu'**une** trajectoire à la fois : ce
qu'il poursuit reste net, tout le reste défile.

D'où la règle, qui n'a aucun paramètre libre :

```
    glissement(x) = ‖ v_apparente(x) − v_poursuivie ‖   en degrés par seconde
    finesse(x)    = celle que ce glissement laisse voir
```

Trois propriétés tombent d'elles-mêmes, et ce sont exactement celles qui manquent :

* **la netteté revient en même temps que l'amortissement s'éteint** — continûment, sans
  seuil, puisque `v → 0` ;
* la finesse est **locale** : ce que l'œil suit reste net pendant que le reste défile ;
* les degrés par seconde se calculent depuis les pixels par seconde et la **densité réelle**
  de l'écran, que le système annonce. Rien à régler d'un appareil à l'autre.

### Loi II — ne jamais recalculer ce qui n'a pas changé (TUILE-1)

Le canevas devient un **quadtree dyadique** dont chaque nœud porte l'empreinte du contenu
qu'il recouvre. Le rendu d'un nœud est **mémoïsé par cette empreinte**.

C'est la structure de Hashlife, et son avantage se transpose mot pour mot : *le travail est
proportionnel au nombre de nœuds, pas au nombre de cellules*. Dans notre cas : proportionnel
au nombre de **tuiles dont le contenu a changé**, pas au nombre de pixels de l'écran.

Quatre conséquences, et chacune répond à un défaut nommé plus haut :

| propriété | ce qu'elle règle |
|---|---|
| deux régions au contenu identique partagent un seul rendu | le mur de quatre cents photos |
| se déplacer sur du vide ne rend rien | `redessine 100%` pendant un pan |
| les échelles sont dyadiques, donc stables sur une octave entière | les vignettes périmées à chaque fraction de pixel |
| une tuile invalidée n'invalide que ses ancêtres | la salissure globale des 68 appelants |

C'est aussi ce que font, sous une forme ou une autre, tous les canevas qui tiennent :
tuiles de 256 px, cache par tuile, paliers de zoom en puissances de deux, invalidation par
tuile sale.

**Ce que cela remplace :** le cache de vignettes par nœud, dont la clé est un point d'un
espace continu (taille × phase sous-pixel) et qui ne peut donc servir que si la vue est
*exactement* immobile. Six pour cent d'utilisation mesurés, et c'est structurel.

### Loi III — on observe la machine, on ne l'interroge jamais (VOIES-1)

Déjà dans la charte, et déjà à moitié tenue par le débit mesuré de l'atelier. À généraliser :
chaque opération a plusieurs **voies** — scalaire, SIMD détecté à l'exécution, GPU — et
Glucose mesure le débit constaté de chacune.

Cela répond d'un seul mécanisme à trois questions que la charte pose séparément :

* **le bridage thermique** — un téléphone qui chauffe voit son débit baisser ; les tranches de
  travail raccourcissent sans que rien n'ait eu à lire une température ;
* **un jeu ou un rendu 3D en arrière-plan** — la voie GPU s'effondre, la voie CPU tient, on
  bascule sur ce qui est réellement libre ;
* **big.LITTLE et l'hétérogénéité** — les cœurs lents se dénoncent par leur débit.

La garantie qui rend tout cela sûr, et elle est déjà écrite dans la charte : **deux voies
d'une même opération produisent les mêmes pixels, au bit près.** C'est ce qu'on vient de
prouver pour la composition par REPORT-1 contre `tiny-skia`, et c'est le modèle à suivre.

---

## 4. L'arbre de possibilités — PREDICTION-1

C'est la demande explicite de l'utilisateur : « avoir mathématiquement des arbres de
possibilités qui se créent et se détruisent en permanence ».

### Ce que l'on prédit, et pourquoi c'est calculable

La caméra n'a que trois degrés de liberté — deux de translation, un d'échelle — et son
mouvement est déjà **régi par une équation connue** : l'amortissement exponentiel de l'élan.
On ne devine donc pas la trajectoire : **on l'intègre**.

```
    v(t) = v₀ · e^(−t/τ)        ⟹    position future exacte, à geste constant
```

À l'horizon d'une demi-seconde, l'enveloppe des vues possibles est donc un cône étroit autour
de cette trajectoire — d'autant plus étroit que l'amortissement est avancé. Les tuiles qui
entreront dans la vue s'en déduisent **par le calcul**, pas par une heuristique.

C'est le même principe que la prédiction de pose en réalité virtuelle (rendre pour l'instant
où l'image sera *affichée*, pas celui où elle est demandée) et que le préchargement de tuiles
dans les cartes (extrapoler où la vue va être pour charger avant).

### La forme de l'arbre

* **la racine** : la vue courante ;
* **les branches** : la trajectoire intégrée, plus les gestes que l'utilisateur peut initier
  (poursuivre, freiner, inverser, zoomer) ;
* **le poids d'une branche** : la probabilité observée de ce geste — mesurée sur la session,
  pas postulée ;
* **l'élagage** : une branche coûte des tuiles ; on ne garde que ce que le temps libre et la
  mémoire disponible permettent de préparer. Le critère est le **gain espéré par
  milliseconde**, et il se calcule.

Une branche non réalisée n'a rien coûté d'autre que du temps libre — celui qui, sinon,
n'aurait rien fait du tout.

### Ce que cela rend possible, et rien de moins

Aujourd'hui, un geste trouve les tuiles à faire et les fait **pendant** l'image. Avec l'arbre,
il les trouve **déjà faites**. C'est la seule voie par laquelle 140 images par seconde se
tiennent sans dégrader : non pas dessiner plus vite, mais **avoir déjà dessiné**.

---

## 5. Le plan, par étapes, et ce que chacune débloque

L'ordre n'est pas négociable : chaque étape rend la suivante mesurable.

### Étape 0 — la finesse suit la vitesse *(petite, immédiate, visible)*

Remplacer le seuil de budget par la loi I. La pixelisation s'éteint alors **en même temps que
le mouvement**, ce qui est exactement le défaut signalé.

*Ce qui prouve que c'est fait :* la part d'images pixelisées devient proportionnelle au temps
passé en mouvement rapide, et tombe à zéro dans la dernière demi-seconde de chaque
amortissement.

### Étape 1 — la salissure devient la règle et non l'exception *(moyenne)*

Les soixante-huit appelants qui disent « tout est sale » doivent devenir des appelants qui
disent **ce** qui est sale. C'est mécanique, sans invention, et c'est la condition
arithmétique du § 2.

*Ce qui prouve que c'est fait :* `redessine` descend nettement sous 100 % hors zoom, et la
part d'images évitées monte.

### Étape 2 — TUILE-1, le quadtree canonique *(grande — le cœur du chantier)*

Le cache pyramidal de tuiles, mémoïsé par empreinte de contenu, aux échelles dyadiques. Il
remplace le cache de vignettes par nœud, qui disparaît.

*Ce qui prouve que c'est fait :* se déplacer sur une zone déjà visitée ne produit **aucun**
rendu de tuile ; deux régions identiques n'en produisent qu'un.

### Étape 3 — PREDICTION-1, l'arbre de possibilités *(grande)*

Le temps libre sert à préparer les tuiles de la trajectoire intégrée. L'atelier cesse d'être
un rattrapage pour devenir une avance.

*Ce qui prouve que c'est fait :* la part des tuiles trouvées déjà prêtes au moment d'être
demandées — une mesure qui n'existe pas encore et qu'il faudra écrire d'abord.

### Étape 4 — VOIES-1, l'adaptativité totale *(grande)*

Plusieurs voies par opération, débit mesuré, bascule sur ce qui est libre. Avec la preuve
d'égalité au bit près comme garde-fou systématique.

*Ce qui prouve que c'est fait :* lancer un jeu en arrière-plan fait basculer les voies sans
que la cadence de Glucose bouge.

### Étape 5 — les pics de présentation *(inconnue, et elle inquiète)*

`present` à 334 ms, `blit` à 323 ms, sur des images qui ne dessinent rien. Entièrement dans
la carte graphique, jamais élucidé. Tant que ça dure, **aucune garantie de cadence n'est
tenable** : un gel d'un tiers de seconde annule tout le reste.

À traiter en parallèle des autres étapes, parce que rien de ce qui précède ne le touche.

---

## 6. Ce qui décide que c'est gagné

Trois mesures, et aucune n'est une moyenne.

1. **Le pire centile de la latence geste → écran** reste sous 7,14 ms pendant toute une
   session réelle. Pas la médiane : c'est le pire qui se ressent.
2. **Aucune image au-dessus de 7,14 ms**, y compris les pics de présentation.
3. **La netteté à l'arrêt est intégrale**, et la finesse en mouvement ne dépend que de la
   vitesse — vérifiable en rejouant deux gestes de même vitesse sur deux machines
   différentes : ils doivent produire la même finesse, quelle que soit la machine.

---

## 7. Ce que ce plan remet en cause de mon propre travail

Par honnêteté, et parce que la charte l'exige.

* **Le cache de vignettes par nœud** (MIP-2) est structurellement incapable de servir : sa clé
  est un point d'un espace continu. TUILE-1 le remplace, il disparaît.
* **La réduction de résolution globale** est le levier le plus grossier possible. La loi I la
  remplace par une finesse locale et continue.
* **Le modèle de coût** reste juste et utile, mais son usage était faux : il doit financer une
  **avance de travail**, pas déclencher une dégradation.
* **Trois sessions de travail sur les budgets** (tranche de fond, plancher, cible) ont déplacé
  un problème que ces trois lois suppriment. Elles n'étaient pas inutiles — elles ont produit
  les mesures qui rendent ce document possible — mais elles ne sont pas la direction.

---

## 8. Sources

Travaux et références consultés pour ce plan :

- [Hashlife — Wikipedia](https://en.wikipedia.org/wiki/Hashlife) et
  [une implémentation commentée](https://johnhw.github.io/hashlife/index.md.html) : quadtree
  canonique, mémoïsation par empreinte, coût proportionnel aux nœuds et non aux cellules.
- [An infinite canvas tutorial](https://antv.vision/infinite-canvas-tutorial/guide/what-is-an-infinite-canvas)
  et [le dépôt associé](https://github.com/xiaoiver/infinite-canvas-tutorial) : architecture
  de canevas infini, tuiles, paliers de zoom.
- [Tile-based caching for rendering complex artwork](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/9235873)
  et [Method for rendering using a progressive cache](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/7190367) :
  tuiles rendues en fond par palier d'échelle, sélection de la meilleure tuile disponible,
  raffinement progressif.
- [Predictive tiling](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/9383917)
  et [la discussion de préchargement de MapLibre](https://github.com/maplibre/maplibre-gl-js/issues/116) :
  extrapolation de la trajectoire de vue pour préparer les tuiles à venir.
- [Asynchronous reprojection](https://en.wikipedia.org/wiki/Asynchronous_reprojection) et
  [Predictability-Aware Motion Prediction for Edge XR](https://arxiv.org/pdf/2507.13179) :
  prédiction de pose, rendu pour l'instant d'affichage plutôt que de demande.
- [Unity Adaptive Performance](https://docs.unity3d.com/Packages/com.unity.adaptiveperformance@1.0/manual/index.html) :
  réaction au régime thermique, choix du levier selon le goulot constaté.
- [Contention-Aware Scheduling for Asymmetric Multicore Processors](https://yuleisui.github.io/publications/icpads15.pdf)
  et [Dynamic Adaptive Scheduling for Heterogeneous SoC](https://doi.org/10.3390/jlpea13040056) :
  ordonnancement sur cœurs hétérogènes, détection de contention par les grandeurs observées.
- [Smooth pursuit eye movement — synthèse](https://www.sciencedirect.com/topics/immunology-and-microbiology/smooth-pursuit-eye-movement)
  et [Detection of speed changes during pursuit eye movements](https://link.springer.com/article/10.1007/s00221-005-0216-6) :
  chute d'acuité au-delà de 2°/s de glissement rétinien, perte des hautes fréquences au-delà
  de 5-6°/s, gain de poursuite de 0,9 à 1,0 jusqu'à 20°/s.

Mesures propres au projet : `bench_bande_passante` (plancher d'une image, § 2),
`bench_chrome` (composition des tampons, 4×), `bench_occlusion` et `bench_zone` (rendu par
région, 10 à 12× sur une petite zone), et les chroniques de terrain des 18 et 19/09.
