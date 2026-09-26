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

## 5. Le coude qu'on ne pouvait pas déplacer (ARROW-3, `6194109`)

**Ce qu'il a vu** : cliquer le point blanc d'une flèche crée un point de déformation, *« mais on
ne peut RIEN en faire : il est là, mais on ne peut pas le déplacer »*.

**Reproduit** par le vrai chemin de la souris, à 150 % : le coude naît à l'appui et reste au
milieu du tronçon pendant le glisser. **La cause** : `update_bend` et `finish_bend` n'étaient
appelés **nulle part** dans l'application — seulement par les épreuves, qui les appelaient
directement. `git log -S` le confirme : le câblage n'a jamais existé depuis le commit des coudes
du 15/09. Le geste d'annulation que l'appui ouvrait ne se refermait pas non plus. C'est la forme
exacte de R-18 (un module écrit, éprouvé, et débranché), et la raison pour laquelle une épreuve
doit passer par le vrai chemin de la souris.

**Une hypothèse démentie** : j'ai d'abord cru que le relâchement faisait descendre le cycle du
clic (PICK-2) et déselectionnait la flèche. Faux : le cycle ne descend qu'au relâchement d'un
appui que l'arbitre a armé, et un appui pris par une poignée ne passe pas par lui. La remise à
zéro que j'avais écrite pour ça est retirée (une règle sans cas) ; l'épreuve garde le scénario.

---

## 6. Les pastilles de relation (BADGE-1, BADGE-2, `678edd1`)

Ses trois retours sur les « logos » :

1. **Ils ne rapetissaient pas au dézoom** — sa capture : une pastille orange plus grande qu'un
   groupe entier d'images. Une session précédente les avait mises en pixels d'écran, par un choix
   argumenté (« une légende doit rester lisible quand on prend du recul »). Chez Tauri, tout le
   calque des flèches est mis à l'échelle du zoom (`translate(…) scale(…)`) : la pastille de rayon
   10 et l'étiquette de corps 11 sont des longueurs **du monde**, seul le trait garde son épaisseur.
   Elles y reviennent, sous le seuil de détail commun (SCALE-2), comme le texte d'une carte.
2. **Ils se posaient exactement où l'on crée les coudes.** La pastille survolée **s'efface en
   deux cents millisecondes**, et revient quand la souris part — la même mécanique que la lueur
   des cartes désignées (LUEUR-2) ; `Vivacites` réunit les deux, et le réveil ne demande qu'à elle
   si quelque chose glisse encore. Le dessin et le survol lisent le même centre de pastille
   (`centre_du_badge`), et une seule règle dit si l'étiquette se montre — il y en avait deux. Le
   survol ne cherche que parmi ce que l'index spatial trouve autour du point.
3. **Rien ne disait ce qu'ils veulent dire.** La liste de Tauri portait des mots ; les sigles
   seuls les avaient perdus — et le noyau nommait pourtant les relations
   (`ArrowPredicate::label`), sans que personne le lise. Chaque bouton de relation montre
   désormais son sigle **et** son mot : « est précurseur de », « contredit », « hérite de »,
   « inspire », « dépend de », « illustre ». Trop étroite pour tout tenir, la barre retombe sur les
   sigles seuls — du plus riche au plus sobre, sans largeur écrite en dur.

Au passage : une **troisième copie** du traceur de rectangle arrondi, en paraboles, survivait
dans l'étiquette (ARC-1 disait qu'il n'en restait qu'une) : elle passe au traceur commun.

**Ce qui le tient** : la pastille et l'étiquette à ×0,5 couvrent la moitié de ce qu'elles
couvrent à ×1 (par l'image) ; la pastille survolée s'efface — dans l'image, pas seulement dans
sa valeur — et revient ; la barre dit ses mots et retombe sur les sigles. Huit sabotages tombent.

---

## 7. Les formules (FORMULE-1, `b3acf72`)

**Ce qu'il a vu**, *« depuis le tout début du LaTeX »* : des caractères qui entrent dans
d'autres, des positionnements qui ne fonctionnent pas du tout. Sa capture montre une formule de
la carte d'essai où les lettres se chevauchent.

**La cause, une seule, au fond.** KaTeX écrit sa mise en page en deux endroits : dans son arbre
(les hauteurs d'un empilement, les marges de l'espacement) et dans **sa feuille de style** — qu'une
fraction centre ses étages, qu'un délimiteur vide borde une fraction de 0,12 em, que l'indice
d'une racine recule de 0,5556 em. Le navigateur de Tauri appliquait la feuille ; le pont de
Glucose Rust ne lisait que l'arbre. Et le navigateur mesure lui-même chaque lettre dans sa
fonte, là où le pont lisait les largeurs de KaTeX — qui, pour les lettres qu'il fusionne
(« or », « nj »), ne gardent que **la largeur de la première**. D'où « bonjours » avec le « o »
sous le « j », « fjord » avec le « d » sur le « r ».

**La méthode : mesurer contre le vrai.** Le KaTeX de Glucose Tauri (`node_modules/katex`) rend
44 formules dans Chromium (Playwright), et Glucose les rend hors écran par le moteur des cartes
(`examples/apercu_formules.rs`), au même corps. Chaque rendu est recadré sur son encre, et l'on
compte la part d'encre franche d'un rendu **sans aucune encre de l'autre à moins de 2 pixels** :

| | pire formule | formules à 0 % |
|---|---|---|
| le pont d'origine | **76 %** (`\sqrt[3]{x+1}`) | — |
| FORMULE-1 | **0,38 %** (`\begin{bmatrix}…`) | 43 sur 44 |

Les 44 : fractions imbriquées, racines à indice, intégrales simples, doubles, de contour,
sommes, produits, limites, matrices (`pmatrix`, `bmatrix`, `vmatrix`, tableau à filets), `cases`,
`aligned`, binômes, accents simples et larges, flèches extensibles, accolades, `\boxed`,
`\cancel`, `\rule`, grands délimiteurs empilés, les familles `\mathbb`, `\mathfrak`, `\mathcal`,
`\mathscr`, `\mathbf`, `\boldsymbol`, du texte. Ce qui reste est d'un pixel : Chromium arrondit
l'épaisseur d'un filet au pixel entier (1,6 px devient 1), nous la lissons.

**Ce que le pont fait désormais** (`glucose-math/src/layout/`) :

* **`feuille.rs`** — les règles géométriques de `katex.css` (0.16), vocabulaire **fermé** et cité
  ligne à ligne : alignement des empilements, familles de fontes (famille, inclinaison et graisse
  sont trois propriétés distinctes, comme en CSS — `\mathbb` et `\mathfrak` manquaient), marges et
  bourrages de classe, portions de forme (demi-flèches, accolades en trois morceaux), largeurs
  imposées, bordures (`.fbox`, `.angl`), recouvrements (`llap`, `rlap`, `clap`).
* **`pont.rs`** — le parcours : un glyphe par caractère ; la correction d'italique en marge
  droite (les bornes de `\int_0^\infty` se posaient sur le signe) ; les largeurs écrites (l'écart
  entre deux colonnes, les morceaux de grand délimiteur) ; les bordures tracées sans se recouvrir
  (`\boxed`, le filet vertical d'un tableau, `\rule` et sa levée) ; les formes qui attendent la
  largeur de leur étage ; la transparence de `\phantom`, qui réserve sa place sans se dessiner.
* **`chemin.rs`** — les formes SVG de KaTeX **lues**, pas reconstruites : treize commandes de
  chemin, la boîte de vue, la règle `preserveAspectRatio`. Le radical était construit à la main
  (trois segments) et **aucune** autre forme ne se dessinait : ni flèche longue, ni accolade, ni
  chapeau large. Les ratures de `\cancel` sont des `<line>` : elles se tracent, à leur épaisseur.

**Une découverte en chemin : les métriques de KaTeX ne sont pas la fonte.** Sur les 2 035
glyphes des vingt fontes, 25 divergent — dont `∬` et `∭`, larges de 0,556 em dans les métriques
et de 1,084 et 1,592 dans la fonte (`D` de `\iint_D` se posait sur le signe), `°`, et les accents
combinants. Plutôt qu'une table de 25 exceptions, le pont **se fait mesurer par la fonte même
que le dessin emploie** (`layout_avec`) : le rendu tient les fontes, il répond ; le crate seul
garde les métriques pour recours. Mesure et dessin ne peuvent plus choisir deux fontes.

**Mes erreurs, trouvées en route** :

* ma première mesure d'écart (par quantiles de la masse d'encre) donnait 38 px d'écart sur une
  intégrale **identique** à l'œil : elle mesurait les différences de lissage, pas de position.
  La superposition rouge/vert l'a montré ; la mesure retenue compte l'encre orpheline ;
* j'avais écrit une lecture du `calc(100% - …)` que KaTeX pose sur l'étage d'un accent large.
  En sabotant, elle s'est révélée **redondante** : ce `calc` vaut exactement la largeur qu'un bloc
  prend de lui-même (son conteneur moins ses marges). Retirée ; la règle générale suffit ;
* un ancien test affirmait que la barre d'une fraction fait toute la largeur de la formule — il
  gardait le défaut. Il dit désormais la règle de KaTeX (entre les deux vides de 0,12 em).

**Les témoins** : la carte « Une formule, seule sur sa ligne » montrait exactement son défaut —
la borne « 0 » de l'intégrale absente, le « π » avalé par la racine. 908 pixels changent, tous
dans cette formule ; regardés avant/après, agrandis.

**Ce qui le tient** : 24 épreuves du pont et de ses chemins (chaque lettre avance de sa largeur,
bornes d'intégrale, avance de la fonte, centrage des fractions, colonnes, familles, cadre sans
recouvrement, `\rule`, filet de tableau, rature, accent d'une lettre penchée, indice d'une racine,
fantôme, toutes les formes tracées et bornées, tous les chemins nommés de KaTeX lisibles…) et
deux du dessin (une rature se trace ; l'avance vient de la fonte). **27 sabotages tombent** ;
un seul passait au premier tour — les marges de classe — et a reçu son épreuve.

**Ce qui n'est pas fait** : les **couleurs** (`\color{red}`, `\colorbox`) ne se peignent pas — la
formule sort dans la couleur de la carte ; `\tag` (la numérotation à droite) ; `\sout` ; les
filets pointillés (`\hdashline`) sortent pleins.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
