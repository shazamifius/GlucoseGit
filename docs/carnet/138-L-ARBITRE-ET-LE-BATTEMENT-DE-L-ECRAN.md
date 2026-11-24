# 138 — L'arbitre et le battement de l'écran (ARBITRE-5)

> Le 24/11/2026, vers 3 h 40, session du relais (compte b, au bord de sa fenêtre de 5 h). Vu à la
> fiche 137 § 5 : sa sortie du 23/11 à 21 h 50 — *« arbitre : cette carte gèle »* — sur un PC qui
> ne démarrait pas.

## 1. Ce qui se passait

L'arbitre (ARBITRE-1 et 2) compte comme **perdu** tout ce qu'une présentation dépasse le plancher de
la charte, 10 ms. Or sur un écran à 60 Hz, quand Glucose dessine plus vite que l'écran n'affiche, la
chaîne d'images se remplit et `present` attend le battement suivant : 12 à 16 ms dans sa session du
23/11 au soir (sa chronique, copiée), jamais plus de 15,9. Ce n'est pas la carte qui gèle, c'est
l'écran qui fait attendre — et l'arbitre condamnait la carte.

**Sans effet sur son poste** : Vulkan n'y voit qu'une carte, la RTX (fiche 137 § 5) — « économe » et
« rapide » ouvrent la même. **Un balancier sur une machine à deux cartes** : chaque condamnation fait
ouvrir l'autre au lancement suivant (ARBITRE-4), et la fiche 131 § 6 y voyait déjà le démarrage du PC.

## 2. ARBITRE-5 — rien n'est perdu en deçà d'un battement

Le seuil de ce qui est perdu devient **le plus long du plancher et du battement de l'écran**
(`Arbitre::battre`, nourri par la cadence à chaque image). Aucune constante nouvelle : la période se
lit (CADENCE-1). Un vrai gel — le terrain de la fiche 21, 194 ms — passe le battement et fait toujours
essayer l'autre carte.

**Prouvé** : `test_arbitre_5_attendre_le_battement_n_est_pas_geler` — 600 présentations à 15 ms sur
un écran à 60 Hz : l'arbitre ne bouge pas ; **à l'envers**, la loi d'avant (le plancher seul) condamne
la carte sur les mêmes images ; un gel de 194 ms conclut aussitôt. Un sabotage (`arbitre-138.json`).

## 3. Reste (ACCÈS-H24, point 2)

* **Pas d'arbitre quand les deux préférences ouvrent la même carte** : son poste n'a rien à arbitrer,
  et le dit pourtant (« cette carte gèle ») ; comparer les deux adaptateurs au lancement.
* **ECRAN-1 sous Linux** : la carte qui tient l'écran se lit dans `/sys/class/drm` (`cardN-<sortie>/
  status`, et `cardN/device/vendor`, `device/device` — les identifiants que `wgpu` donne aussi), la
  sortie de la fenêtre par le nom de son moniteur. Quand l'écran a tranché, il n'y a plus d'arbitre.
* L'appel `battre` dans `arbitrer_la_carte` n'a pas d'épreuve qui passe par l'application.
