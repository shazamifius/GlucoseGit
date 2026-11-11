# 105 — Les membranes en carte, le papier gris des blocs

> Session du 11/11/2026, de 6 h 20 à __FIN__. La passation (fiche 104) laissait la loi des régions
> approuvée et prototypée, rien d'écrit ; ses paroles du matin (VISION § 5) : le fond d'un texte
> toujours gris, les frontières plus légères, une membrane sans couleur neutre. Envoyé : __COMMITS__.

## 1. Ce que la remise en question a changé au plan

* **La force des régions ne se déduisait plus de rien.** La fiche 104 la tirait de la lecture d'un
  texte posé sur la couleur de sa région (APCA, Lc 75 : 23 %). Le papier d'un texte devenant toujours
  le gris des blocs, aucune lecture ne la borne plus. J'ai cherché un principe (le bloc qui se
  détache de sa région, le halo qui doit rester visible dessus, la lisibilité du titre détouré) : aucun
  ne fixe un nombre, seulement des minimums. Elle est donc dite **nombre de ressenti**, ancrée à sa
  parole : *le voile d'une membrane de Tauri posé deux fois* — le ton qu'avait un thème dans la racine
  sous la loi d'avant, celui de l'image qu'il a trouvée « magnifique » le 9/11, sans le mélange des
  teintes (21,3 %, la même chose à l'œil que les 23 % approuvés). Écrite avec les constantes de Tauri,
  sans en ajouter (`champs.rs`, `force`).
* **Le corps opaque cachait la grille de points et les cases d'un storyboard.** La page de la fiche 104
  le disait, et il a dit oui à la loi ; mais un storyboard rangé dans une membrane (`Ctrl+G` sur une
  séquence) aurait perdu ses cases — rien à moitié. Et FigJam, dont les sections opaques cachent la
  grille, reçoit les plaintes de ceux qui s'en servent pour aligner ([forum de Figma](https://forum.figma.com/suggest-a-feature-11/figjam-transparent-background-section-41196)).
  D'où un ordre en trois temps : **le sol des régions** (halos, corps opaque), **puis la grille et les
  zones**, **puis les frontières** — une carte pose son quadrillage sur les pays.
* **Ordonner par la surface, non par la profondeur.** La loi dit « la plus petite région qui contient
  le point » : peindre de la plus grande à la plus petite la donne exactement, pour des membranes
  emboîtées comme pour deux qui se chevauchent, sans parcourir l'arbre (le prototype ajoutait
  `Board::profondeurs_des_membranes` au noyau : inutile). Une fille posée avant sa mère dans le
  modèle passe quand même par-dessus.
* **La loi coûte moins que l'ancienne.** Aujourd'hui, chaque membrane emboîtée qui couvre l'écran est un
  quad plein écran que la carte graphique compose ; au fond de « Eau », six. Un corps opaque qui couvre
  l'écran cache tout ce qui le précède : ces champs ne se posent plus (`Membrane::couvre`), ni une
  frontière dont l'écran est loin à l'intérieur (`Membrane::touche`). __MESURE__
* **La bande « territoire » essayée, écartée.** Voir § 3.

## 2. La recherche (consultée le 11/11/2026)

* Le motif « territoire » ([J. Howarth, Cartographic Design Patterns](https://jeffhowarth.github.io/cart-patterns/patterns/territory)) :
  *« un remplissage de tout le territoire réduit souvent la lisibilité de ce qui est dedans ; les
  cartographes affaiblissent l'intérieur et renforcent l'accent près de la frontière »* — le bord en
  bandes (une large bande intérieure, des bandes plus pâles vers l'intérieur) et le bord lumineux (le
  même, en dégradé). La *buffalo tint* des cartes politiques (Esri).
* Les frontières hiérarchisées : la hiérarchie par l'épaisseur, la valeur et le détail du trait
  ([USGS, spécifications des limites administratives](https://www.usgs.gov/ngp-standards-and-specifications/cartographic-specifications-governmental-units)) ;
  un trait plus clair et plus large placé sous l'autre marche mal quand il faut une hiérarchie.
* La règle d'Imhof : les grandes surfaces sourdes et claires, les couleurs saturées aux petites
  ([Ordnance Survey](https://docs.os.uk/more-than-maps/geographic-data-visualisation/guide-to-cartography/colour)).
* Les cadres opaques des canevas : Miro (un cadre blanc cache la grille, sauf à le rendre transparent),
  FigJam (les sections opaques la cachent, et des utilisateurs le regrettent).

## 3. La frontière « plus légère » — regardée, à sa parole

Trois essais sur les quatre vues de sa carte, au prototype (`~/.cache/glucose-bancs/membranes-105/`) :

* **la bande « territoire »**, aussi large que le titre est haut, du voile de la région posé une fois de
  plus : la nidification se lit, mais chaque région devient un cadre de tableau épais — lourd autrement ;
* **le trait continu** : épais comme la loi du titre (`2 × √(A/A₀)`), à l'intérieur du bord, avec
  **l'encre du pointillé d'aujourd'hui** (la moitié de son opacité, sans tirets — ce que Glucose fait déjà
  quand un tiret devient plus petit qu'un pixel). De loin, la carte se lit par ses frontières fines ; de
  près, le bord d'une immense région devient une large bande (la même loi que son titre) ;
* **le pointillé de la fiche 104**, pour mémoire : *« ruban de chantier »*.

La page, avec les images et deux questions : https://claude.ai/artifact/7L4zAY5U4mnebV2qvHgSK4. **Le trait
continu ferait disparaître le pointillé de toutes les membranes** — la marque de Tauri — et toute sa
mécanique (le noyau, le nuanceur) deviendrait du code mort, à retirer : rien n'est retiré avant sa
parole. __FRONTIERE__

## 4. Ce qui est écrit

* **REGION-1** (`glucose-core/src/membrane_forme/regions.rs`) : `Regions`, le sol puis les frontières ;
  `Membrane::couvre` et `Membrane::touche`, deux questions exactes (convexité : les quatre coins de
  l'écran suffisent). Au bureau, `renderer/scene/membranes.rs` (extrait de la scène, qui approchait des
  600 lignes) et `membranes/champs.rs` : chaque membrane donne ses halos, son corps opaque — sa teinte
  à la force des régions sur le fond du canevas, celui que le mode Focus teinte —, et sa frontière ; de
  la plus grande à la plus petite. Le processeur (`draw_membranes`, avec ce qui se pose entre), la carte
  graphique (`poser_le_sol`, `poser_les_frontieres` : deux tracés du même tampon), l'export SVG : le même
  ordre.
* **La grille sur le sol** : la passe du fond se fait en deux tracés (`fs`, le fond seul ; `points`, les
  points prémultipliés, composés sur ce qui est dessous) ; au processeur, `poser_le_fond` ne pose plus la
  grille, `dessiner_sous_les_photos` la pose entre le sol et les frontières.
* **PAPIER-1** : `Theme::card_paper`, le gris sombre et neutre des blocs, partout ; `Contenants` (le
  papier vu au travers des membranes, LUEUR-3), `papier_teinte` (COULEUR-3) et les 3 % de teinte de la
  brume disparaissent. La texture d'un texte ne se refait plus quand une membrane change.
* **NEUTRE-1** : une membrane sans couleur — ou à la couleur illisible — prend le gris de la clarté du
  bleu de Tauri ; le fond du mode Focus aussi.

## 5. Prouver

__PROUVER__

## 6. Les leçons

__LECONS__
