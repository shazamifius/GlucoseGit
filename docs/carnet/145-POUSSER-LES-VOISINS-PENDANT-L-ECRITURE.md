# 145 — Pousser les voisins pendant l'écriture : cadré (24/11, le relais, compte b)

Sa réponse du 24/11 sur `relais-rep-327` : *« le truc dérangeant, c'est justement qu'il n'y ait pas
de mise à jour LORSQU'on écrit »*. Les trois aperçus d'après l'écriture sont retirés du plan.

## 1. Ce qui existe

À chaque lettre, la saisie fait grandir la carte vers le bas (`interactions/resize.rs`, vers la ligne
530) puis, **dans le même geste**, pousse le bas de sa membrane et de celles qui la contiennent
(TEXT-FIT-2, `glucose-core/src/store/pousser.rs`, fiche 124). Les voisins, eux, ne bougent jamais
(MEMB-7) : le texte grandit par-dessus.

## 2. Ce que dit la recherche

Le problème s'appelle *overlap removal* ; le critère qu'il veut — « ce qui était à gauche reste à
gauche » — est **l'ordre orthogonal**, et le moindre déplacement sous cet ordre est NP-difficile
dans le cas général (van Garderen et al., *Minimum-Displacement Overlap Removal*,
<https://diglib.eg.org/handle/10.1111/cgf13199>). VPSC le résout dimension par dimension sous des
contraintes de séparation (Dwyer, Marriott, Stuckey, *Fast Node Overlap Removal*, GD 2005,
<https://link.springer.com/content/pdf/10.1007/11618058_15.pdf>) ; *Force Scan* (Misue et al.)
balaie dans un sens et garde l'ordre ; la comparaison de neuf méthodes :
<https://hal-lirmm.ccsd.cnrs.fr/lirmm-02879677/document>.

**Notre cas est bien plus simple** : un seul nœud grandit, d'un seul côté (le bas). Un balayage à la
Force Scan, vers le bas seulement, suffit et garde l'ordre par construction ; aucun solveur.

## 3. La loi proposée

* **Ce qui grandit pousse ce qui était sous lui, à son niveau** : le texte ses voisins de membrane,
  la membrane qui grandit ses propres voisins — la même loi à chaque étage, comme TEXT-FIT-2.
* Seul ce qui était **entièrement dessous** au début de l'écriture et que le bas neuf recouvre
  descend, de `bas + ECART − haut`, et pousse à son tour (balayage par haut croissant). Ce qui
  chevauchait déjà ne bouge pas (« seul ce qui grandit pousse »).
* **Une fonction pure de l'état au début de l'écriture et du bas courant**, pas une suite de
  poussées : effacer des lettres fait remonter les voisins, jamais plus haut que leur place
  d'origine ; aucune dérive ; le Ctrl+Z unique de la saisie rend l'état de départ.
* Les flèches suivent leurs bouts comme pour tout déplacement.

**Le balayage, précisé** (14 h 22) : à un niveau (les nœuds de même `membrane_id`, flèches
exclues), les candidats sont ceux dont la boîte de départ est entièrement sous le bas de départ du
nœud qui grandit ; triés par haut de départ croissant, chacun prend
`haut = max(haut_départ, max(p.bas_neuf + min(ECART, haut_départ − p.bas_départ)))` sur les pousseurs
`p` déjà vus qui le chevauchent horizontalement — l'écart d'origine gardé s'il était plus petit que
`ECART` : il ne bouge qu'au contact, du strict nécessaire —, puis devient pousseur à son tour. **Puis
l'étage du dessus** : la membrane grandit jusqu'au plus bas de ce qu'elle contient (le texte, les
voisins poussés) + `ECART` (la loi de `membranes_poussees`, élargie aux poussés), et le même balayage
pousse ses voisins à elle. Une membrane poussée emporte son contenu (le déplacement d'une membrane
qu'on tire, déjà là). Pas de branche horizontale : la saisie ne grandit que vers le bas.

**Fait (14 h 25) : la loi d'un niveau**, pure, dans le noyau — `voisins_pousses`
(`glucose-core/src/store/pousser.rs`) ; trois épreuves (`store/pousser/tests.rs` : au contact et en
cascade, effacer fait remonter, seul son niveau) ; `outils/sabotages/pousser-145.json`, 5 sur 5. Le
filtre « entièrement dessous » s'est révélé redondant : un pousseur qui n'a pas bougé ne pousse
jamais (`bas + min(ECART, écart) ≤ haut`) — retiré.

**Puis (14 h 28) l'étage du dessus** : `pousser_en_ecrivant` rend, par rang, le haut neuf de chaque
voisin poussé et le bas neuf de chaque membrane qui grandit (`Bord::Haut`, `Bord::Bas`), étage après
étage ; une épreuve de plus, deux sabotages de plus (7 sur 7). Il remplacera `membranes_poussees` à la
saisie, qu'il contient (un texte sans voisin ne fait que pousser ses membranes).

À faire : l'état
de départ gardé dans la session d'édition, la loi branchée à côté de `pousser_les_membranes` ; épreuves (pousser,
cascade, remonter, ordre gardé, un seul Ctrl+Z, les pairs du partage reçoivent les déplacements),
leurs sabotages, et l'essai au pilote.
