# 42 — Son compte rendu, et ce qu'il a changé

> **Rôle de ce document.** Il a fait, le 26/09, les huit essais que la fiche 41 § 13 lui
> demandait, et il en a rendu compte en détail — avec cinq captures, sa réponse sur le
> contournement des flèches, et de nouveaux retours sur les flèches. Cette fiche dit ce qu'il a
> vu, ce que la mesure en dit, et ce qui a été corrigé, dans l'ordre où cela a été fait.
> **Écrite au fil de l'eau.**
>
> **Date** : 2026-09-26 · commits `aa469c1` et suivants.

---

## 1. Son compte rendu

| essai | ce qu'il a vu |
|---|---|
| 1. relier deux cartes | *« parfait, on voit bel et bien la flèche »* |
| 2. l'habit des cartes | *« absolument magnifique »* |
| 3. la barre, la fenêtre d'ancrage | *« parfait »* |
| 4. le passage qui brille | tout fonctionne, **mais le surlignage rose dépasse sur les autres caractères**, et c'est moche (capture) |
| 5. écrire, puis une formule | *« tout est fluide pour le LaTeX »* — mais **depuis le tout début, plein de problèmes de disposition** : des caractères qui rentrent dans d'autres, des positionnements qui ne fonctionnent pas (capture) ; il demande des formules à écrire |
| 6. le fantôme aimanté | *« parfait !!! »* |
| 7. une carte sur une photo | le clic marche, **mais le texte ne se voit pas** : *« la zone de blur est derrière les images »* (capture) |
| 8. les jalons datés | *« tout fonctionne »* |

**Sur les flèches**, en plus :

* **Le contournement (FLECHE-5)** — sa réponse : *« tout le système que possède Tauri pour éviter
  que la flèche traverse une image n'existe pas ; on devrait avoir un algorithme super bien fait
  pour éviter les zones de texte, les autres images, ou TOUT autre élément de Glucose »*.
* **Les points manuels** : cliquer le point blanc crée un point de déformation, mais on ne peut
  **rien** en faire — pas de zone de clic pour le déplacer.
* **Les logos de relation** (« affilier, ignorer… ») se posent exactement là où l'on crée les
  points manuels. Il veut qu'ils restent affichés, mais **s'effacent progressivement** quand la
  souris passe dessus, pour pouvoir éditer la flèche.
* **Rien ne dit ce que les logos veulent dire** : plein de couleurs, aucun vocabulaire
  (« dépend de », « contredit »…).
* **Les logos ne rapetissent pas au dézoom** (capture : une pastille orange plus grande que tout
  un groupe d'images).

### 1.1 Ses journaux

`sortie-reprise.txt` porte la chronique de sa session d'essais (854 s, 6 955 images) : la
console la recopie à la fermeture. `derniere-session.txt`, lui, vient d'un **second lancement**
de 204 s sans écriture, qui a remplacé le premier — rien n'est perdu, les deux sont gardés.

---

## 2. La carte sur une photo (LUEUR-3, `aa469c1`)

**Ce qu'il a vu** : une carte posée sur une peinture claire ; son texte blanc ne se lit pas.
**Son pourquoi** : la zone de flou est derrière les images. **Les deux étaient vrais**, et un
troisième défaut était caché dessous.

1. **La lueur se posait sous les photos.** Elle se peignait juste après le fond — sur la carte
   graphique comme au processeur. Chez Tauri, c'est l'ombre CSS d'un bloc HTML posé sur la
   toile, donc devant toute image (la toile d'images est sous la couche des membranes, elle-même
   sous celle des cartes). Elle se pose désormais **entre les photos et les cartes** : la pose
   des textures sur la carte graphique se coupe en deux autour d'elle (`Retenues` sait, par le
   rang de chaque texture dans la liste demandée, où finissent les photos — même quand l'une
   d'elles n'est pas encore là).
2. **Mais même devant, elle n'aurait rien changé au texte** : l'ombre CSS est découpée à
   l'intérieur de la boîte, et l'intérieur d'une carte ne portait que 3 % de sa teinte. La
   photo passait au travers — chez Tauri aussi. Et la grille avec : sur une autre de ses
   captures, un point de la grille tombe entre « que » et « on », et se lit comme une
   ponctuation.
3. **La règle retenue : une carte cache ce qu'elle recouvre, jamais ce qui la contient.** Son
   fond est peint : le canevas **vu au travers des membranes** qui la contiennent — leur loi même
   (`Membrane::alpha`), lue au centre de la carte —, puis la brume à 3 %. Sur la toile vide,
   rien ne change à l'œil, sinon que la grille s'arrête au bord de la carte ; dans une membrane,
   la carte garde sa teinte violette, comme chez Tauri ; sur une photo, le texte retrouve son
   fond. Aucune opacité n'est choisie : « fond noir » veut dire le fond.

**Le défaut caché.** Opaque, le fond a fait tomber l'égalité stricte des deux voies : un pixel
de coin, 75 contre 76. `Arrondi` range son bord droit comme `gauche + largeur`, et cette somme ne
s'arrondit pas pareil à 80,8 pixels (la carte en place) et à 2,8 (dans sa texture) — aucune
écriture en virgule flottante n'est exacte par translation. À 3 % d'opacité, l'écart
disparaissait dans l'arrondi. La forme de la carte se pose donc sur **la grille de ses glyphes**
(le quart de pixel de GLYPH-1) : sur une grille dyadique les sommes sont exactes, et les deux
voies calculent les mêmes nombres par construction. Le fond et le texte avancent désormais par
les mêmes pas, là où ils pouvaient glisser l'un contre l'autre d'un huitième de pixel.

**Ce qui le tient** : deux épreuves par le vrai rendu des deux voies (`voies_suite`) — une photo
blanche et une carte posée dessus (la photo cachée, la lueur devant elle) ; une membrane et une
carte en son milieu (au même pixel, ce qui était là, voilé de 3 % de la teinte de la carte, et
rien d'autre). Six sabotages tombent, dont la lueur posée avant les photos sur la carte
graphique, et le fond évalué hors de la carte sur chacune des deux voies.

**Les témoins**, regardés côte à côte : dans la scène sélectionnée, une photo en chemin penchée
traverse la carte « Une formule, seule sur sa ligne », et son libellé **se lisait à travers la
formule** — le défaut de sa capture était dans nos propres témoins, et personne ne l'avait vu.
Il est caché. Ailleurs : la grille sous les cartes, la lueur devant le bord de la membrane et le
dossier, et la lueur d'une voisine qui ne déborde plus dans une carte.

---

## 3. Les colonnes d'un tableau suivaient mal le zoom (TABLE-2, `884b694`)

Vu en lisant le code des passages, pas à l'écran : le taquet d'une colonne de tableau — son
abscisse — était une longueur du monde, et le tracé l'ajoutait à une position d'**écran** sans
la mettre à l'échelle. À ×2, la seconde colonne se posait à la moitié de sa place, sur la
première ; en dézoomant, elle partait trop loin. L'épreuve, écrite avant la correction, l'a
mesuré : la dernière colonne finissait à 236 pixels au lieu de 268.

Le taquet se dit désormais **en multiples du corps de sa ligne** : le tracé le multiplie par le
corps de l'écran, le clic par celui du monde, et il suit le zoom sans le connaître.

**Une épreuve aveugle, trouvée en sabotant** : la première version mesurait seulement que le
bord de la dernière colonne double avec le zoom. Or une colonne tombée sur la première laisse
le bord de la première — qui double aussi. Elle mesure maintenant que la colonne finit **là où
la mise en page la pose** ; une seconde épreuve vise le clic dans la colonne. Trois sabotages
tombent.

---

## 4. Le cadre d'un passage ne touche plus ses voisins (PASSAGE-2)

**Ce qu'il a vu** (sa capture) : dans « testetsetetstetsetes », un passage pris au milieu du mot ;
le cadre rose commence au milieu du « s » voisin et finit au milieu du « e » suivant — et ce
« e » sort **à moitié rose**.

**Pourquoi.** Deux causes distinctes :

1. Le cadre — sa marge, son liseré décalé — se posait **par-dessus** une mise en page qui ne
   savait rien de lui : il mordait forcément sur les lettres collées. Chez Tauri, le `<mark>`
   était un élément HTML dans la ligne : sa marge (`padding: 1px 3px`) prenait de la place et
   les voisins s'écartaient. Mais seulement de sa marge : son liseré, qui en CSS ne prend aucune
   place, mordait encore sur un voisin collé — Tauri avait le même défaut au milieu d'un mot, en
   plus petit.
2. La teinte se repeignait en redessinant **toute la ligne** dans une image de la taille du cadre,
   élargie d'un à deux pixels, puis posée : un rectangle découpé, qui prenait le bord d'une
   lettre voisine.

**La règle retenue : un cadre ne couvre jamais l'encre d'une autre lettre.** La mise en page
**ouvre la place** qui manque (`richtext/place.rs`), par un parcours glouton de la ligne :

* un cadre déborde de son passage de son **étendue** — marge, décalage et demi-liseré, lus dans
  son style : aucun nombre nouveau ;
* avant un passage, si le bord gauche du cadre toucherait la dernière lettre voisine, la suite
  de la ligne glisse d'exactement ce qui manque ; après, une voisine qui toucherait le bord droit
  glisse de même ;
* une espace est une place libre (entre deux mots, le cadre la prend avant d'écarter quoi que ce
  soit) ; un début ou une fin de ligne aussi (la marge de la carte est là pour ça) ; deux cadres
  voisins écartent chacun leur part ;
* **la coupe des lignes ne change jamais** : survoler une flèche ne fait pas gagner une ligne à
  une carte, donc ni sa hauteur, ni sa boîte, ni ses flèches ne bougent. La ligne ouverte déborde
  au plus dans la marge de la carte ;
* le décalage se pose par les **taquets** des fragments (TABLE-2), que le tracé comme le clic
  lisent déjà.

Et le passage devient **du contenu** : les fragments sont coupés aux bords des plages, chacun
dedans ou dehors, jamais à moitié — la teinte suit les lettres ; le fond et le liseré se
peignent sous le texte, **dans la texture** de la carte sur la voie graphique ; seule la lueur
passe au-dessus, découpée dans le cadre comme l'ombre CSS de Tauri, parce qu'elle déborde de la
carte. Survoler une flèche refait donc la texture de ses deux cartes — une fois en entrant, une
fois en sortant.

**La fenêtre d'ancrage** lit la même mise en page pour dessiner **et** pour viser : les passages
choisis y ouvrent leur place, et le clic tombe là où le texte est dessiné. Le glisser en cours,
lui, s'y montre comme une sélection ordinaire, sous le texte, sans rien écarter — sinon le texte
bougerait sous la souris pendant qu'on choisit. Sur la carte, seul le choix brille.

**Une erreur à moi, trouvée par les épreuves** : `offset_to_x`, à la frontière de deux fragments
qui se suivent, rend la **fin du premier** — l'abscisse d'avant l'écart. Mes premiers cadres se
posaient donc à l'ancienne place pendant que les lettres, elles, s'étaient écartées. Il y a
désormais deux lectures, et chacune dit laquelle elle est : où **se dessine** un caractère
(`x_du_caractere`, après l'écart), où **finit** ce qui le précède (`offset_to_x`, avant).

**Au passage** : `teinte_de_carte` réunit la teinte d'une carte, recopiée à deux endroits (la
lueur d'un passage prenait la teinte symbiotique là où la carte portait une couleur) ; l'ancien
`draw_line_ink` (la ligne repeinte) disparaît ; l'habit d'une carte — teinte, fond, contenants,
brume, forme — quitte `card.rs`, qui dépassait le cliquet des 600 lignes.

**Ce qui le tient** : sept épreuves de la mise en page (au milieu d'un mot la place est
exactement l'étendue ; une espace est une place libre ; un début de ligne aussi ; deux cadres
voisins ; la coupe inchangée ; la teinte qui suit les lettres ; rien sans plage), une par l'image
(**là où l'encre des deux voisines se pose, aucun pixel teinté**), une par la vraie souris dans
la fenêtre (un second choix visé après l'écart prend exactement ses octets), une des deux voies
avec un passage qui brille. Treize sabotages tombent ; deux premiers « passaient » — c'était du
code redondant, retiré.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
