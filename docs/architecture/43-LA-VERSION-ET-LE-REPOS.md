# 43 — La version, et le repos

> **Rôle de ce document.** La session du 28/09 au soir a repris exactement où la fiche 42
> s'arrêtait : ses points ouverts (§ 8.1), puis ce que sa dernière chronique dit du repos.
> **Écrite au fil de l'eau.**
>
> **Date** : 2026-09-28 · commits `0c0d813` et suivants.
> **État au départ** : `e124ac4`, 1 876 épreuves vertes, clippy strict à zéro, arbre propre —
> vérifié.

---

## 1. Ce qui était sur la table

* **Son compte rendu** des essais de la fiche 42 (le contournement, l'interligne des formules) :
  pas encore arrivé.
* **Sa chronique du 28/09 à 18 h 20** (100 s, sa session des formules), que personne n'avait lue :
  **51 images par seconde**, 41 % des images au-dessus du plancher de 10 ms. Les plus lentes
  (38 à 46 ms) n'ont **qu'un seul nœud à l'écran** : `docks` 12-13 ms, `blit` ~10 ms,
  `soumettre` ~9 ms. C'est la troisième chronique qui le montre (25/09, 26/09, 28/09). § 5.
* Les points ouverts de la fiche 42 § 8.1 : l'écart d'un amas lointain par sa boîte englobante
  (§ 3), la colonne de deux cents cartes (§ 4), l'oubli de tous les itinéraires à chaque version
  (§ 2 et § 4).

---

## 2. Un clic publiait une version (VERSION-1, `0c0d813`)

Trouvé en lisant ce qui fait avancer la version, pour l'oubli des itinéraires : `end_live_edit`
la faisait avancer **même quand le geste n'avait rien écrit**. Un simple clic sur un nœud ouvre
un geste à l'appui (pour un glisser éventuel) et le ferme au relâchement : il publiait une
version. La documentation de `begin_live_edit` disait l'inverse — *« un geste resté immobile ne
la touche pas »*.

Ce que la version pilote, et qui se refaisait donc **à chaque clic** : l'état « modifié » du
document (SAVE-2), l'index spatial, le cache des panneaux, la minimap, les teintes, les domaines,
le focus, les cibles de l'aimant, et les itinéraires de **toutes** les flèches.

**La règle retenue : la version avance exactement quand le journal publie.** `Journal::record` et
`Journal::end` disent s'ils publient une transaction ; le magasin ne publie une version que
dans ce cas. Un geste vide, une édition seule qui ne change rien (renommer un onglet de son
propre nom), un geste en bloc qui ne déplace rien : aucune version.

**Ce qui le tient** : un clic par le vrai chemin de la souris (`test_live_5`) ; les trois cas
dans le noyau. Cinq sabotages tombent — le cinquième passait d'abord : le geste en bloc sans
effet n'avait pas de cas ; `move_selected` sans rien de sélectionné lui en donne un.

---

## 3. L'écart d'un amas lointain, mesuré

La fiche 42 § 8.1 : dans `se_voient`, un amas que le segment n'approche pas s'écarte par sa
boîte englobante ; le sabotage ne le voyait pas, et le chronomètre ne tranchait pas.

**Le chronomètre ne tranchera jamais sur cette machine.** Six scènes par le vrai index (un mur
de 20 × 20 et de 100 × 100 photos, deux mosaïques de murs séparés par des couloirs, deux
colonnes de cartes), vingt et une passes chacune, trois fois en alternance : la **même** scène
varie du simple au triple d'une passe à l'autre (le mur de 20 × 20 : 90 à 287 µs).

**Compter le travail tranche.** Les tests de boîtes faits par la visibilité, pour un même
itinéraire :

| scène | avec l'englobante | sans |
|---|---:|---:|
| un mur (un seul amas) | identique | identique |
| mosaïque de 3 × 3 murs de 6 × 6 photos | 1 176 | 1 764 |
| mosaïque de 5 × 5 murs | 2 814 | 5 334 |
| colonne de 50 cartes séparées | 10 550 | 28 504 |

Dès qu'il y a plusieurs amas — ses tableaux : des groupes de photos, des cartes éparses —, l'écart
divise ce travail par 1,5 à 2,7. **Gardé.** Son épreuve compte le travail d'un même test de
visibilité à côté d'un mur de dix blocs de côté puis de vingt : il est le même. Sans l'écart, il
suivrait le pourtour du mur. Deux sabotages tombent.

Les scènes restent dans le dépôt (`arrow_edit/mesures_tests.rs`, ignorées par défaut) :
`cargo test --release -p glucose-desktop --lib mesure_des_scenes -- --ignored --nocapture`.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
