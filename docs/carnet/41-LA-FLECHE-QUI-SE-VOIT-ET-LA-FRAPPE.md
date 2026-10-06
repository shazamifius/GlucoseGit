# 41 — La flèche qui se voit, et la frappe

> **Rôle de ce document.** Ses retours sur la fiche 40, puis deux sessions : la première a été
> **coupée** par lui au milieu d'un chantier parce qu'elle devenait trop longue ; la seconde
> a repris exactement là. Cette fiche couvre les deux. Ses retours (§ 1) : les flèches trop
> fines, les barres du bas trop petites, l'éditeur d'ancres qui n'est pas une vraie interface,
> les cartes « pleines » là où Tauri montre le texte sur fond noir et le contour en lueur, et
> la flèche **invisible** pendant qu'on la tire. La première session y a répondu (§ 2 à § 5) ;
> la seconde a fini le chantier coupé — ce qu'une frappe coûte (§ 6, § 7) —, puis vérifié son
> registre de Tauri entrée par entrée et corrigé ce qui s'y reproduisait (§ 9, § 10).
>
> **Date** : 2026-09-25 et 26 · commits `d0ba347`, `4c5c7d1`, `0113e8f`, `28a57aa` (la session
> coupée), puis `f2322dd`, `8bd78f8`, `e8abb5c`, `77114bf`, `dd96904`, `35317e2`, et ceux de
> cette fiche.
> **État vérifié** : `cargo test --workspace` exit 0, **1 799 tests verts**, clippy strict à
> zéro, `cargo fmt --check` à zéro, l'application se construit ; après chaque suite,
> `%LOCALAPPDATA%\Glucose` n'a rien reçu.
>
> **Vu à l'écran** : rien de cette fiche encore. Il a lancé l'application le 25/09 à 22 h 19,
> quatre minutes, sans message (§ 1.2) : ni flèche tirée ni texte écrit. Le § 13 dit quoi
> regarder, le § 14 ce qui attend sa parole.

---

## 0. En une page

| | ce qui change pour lui | commit |
|---|---|---|
| **GESTE-1** | la flèche qu'on tire **se voit pendant le glisser** — elle était invisible jusqu'au relâchement | `d0ba347` |
| **LUEUR-1, -2** | la carte : **le texte sur fond noir, le contour en lueur**, comme Tauri ; la carte visée par une flèche **s'avive en douceur** (200 ms) | `d0ba347`, `28a57aa` |
| **DPI-1** | à 150 %, les flèches, poignées, bandes de clic et étiquettes du canevas ont **la taille de Tauri** (elles étaient d'un tiers plus petites) | `4c5c7d1` |
| **BARRES-1** | les barres du bas à la mesure de Tauri : corps 13, boutons encadrés, barre de 37 points | `0113e8f` |
| **ANCRE-UX** | « Éditer le texte lié » ouvre **une vraie fenêtre** : SOURCE puis CIBLE, étapes, puces par passage, croix, molette | `0113e8f` |
| **Passages** | au survol d'une flèche ancrée, le passage brille, **cadre serré sur la police**, texte à la couleur de la carte, une formule prise entière | `0113e8f` |
| **COMPOSANT-3** | écrire : le **curseur ne refait plus la texture** de la carte (il la refaisait deux fois par seconde, et à chaque flèche du clavier) ; une formule sous le curseur ne fait plus retomber la carte au processeur | `f2322dd`, `8bd78f8` |
| **COMPOSANT-4** | **sélectionner une carte ne refait plus sa texture** ; la brume d'une carte coûte un quart de moins ; les deux voies s'accordent **au bit près** | `e8abb5c` |
| **PLACEMENT-1** | sous un outil de création, un **fantôme aimanté** suit le curseur : l'élément naît aligné (registre, 3) | `77114bf` |
| **PICK-2** | **ce qu'on voit sous la souris est ce qu'on prend** : une carte posée sur une photo se prend ; le re-clic descend jusqu'au fond (registre, 8) | `dd96904` |
| **Jalons** | la Time Machine dit **la date exacte** de chaque jalon (registre, 12) | `35317e2` |
| **Le registre** | ses dix-huit remarques sur Tauri ont toutes un état : cinq ne se reproduisent pas dans Rust, trois sont corrigées, Trans-domaines est expliqué | fiche 39 |

---

## 1. Ses retours sur la fiche 40, et son lancement du 25/09 au soir

### 1.1 Ce qu'il a dit

* *« Les flèches sont beaucoup trop fines. »* — *« L'UX en bas est beaucoup trop petite aussi. »*
* Tauri est meilleur pour **le glow** et l'encadré d'un passage, qui *« colle mieux au texte »* ;
  sur Rust, *« des problèmes d'encadrer au niveau du LaTeX, au niveau de la disposition »*.
* L'éditeur d'ancres : Tauri a *« une vraie interface où on peut directement sélectionner le
  texte et dire à quoi il fait référence […] avec un vrai bouton éditer »* ; sur Rust, *« un tout
  petit bouton, on ne sait même pas que c'est un bouton, et le texte est incomplet »* (« Ancrer… »).
* *« Les zones de texte sont pleines, avec la couleur du bloc en fond complet : je trouve ça
  moche. Sur Tauri, le texte sur fond noir avec juste les contours du bloc en glow, c'est
  tellement plus beau. »*
* *« Quand on prend la flèche et qu'on glisse d'un point à un autre, elle est parfaitement
  invisible ; c'est seulement au relâchement qu'elle apparaît. »*
* Et son rappel : *« l'idée n'est pas de tout faire comme Tauri au niveau du script, qui est
  catastrophique ; il faut voir TOUT d'un œil expert neuf et le refaire au propre ; pour le
  rendu visuel, c'est vrai que Tauri est visuellement magnifique. »*

Sa réponse sur le **contournement automatique des flèches** (FLECHE-5) n'est jamais arrivée :
rien n'a été commencé (§ 10).

### 1.2 Sa chronique des flèches, et son lancement de 22 h 19

`sortie-chronique-2026-09-25-fleches.txt` (1 067 s, 5 306 images, 141 nœuds) : **103 images par
seconde** entre deux images consécutives. Le geste le plus cher : **éditer du texte**, image
médiane **19,48 ms**, dont `textures` **11,59 ms**, sur une carte de **798 kpx**. C'est le
chantier du § 6.

À 22 h 19, après la coupure de la première session, il a relancé l'application quatre minutes
(`target/release` reconstruit à 22 h 19, `derniere-session.txt` à 22 h 23) — sans fichier de
sortie ni message. Sa chronique dit : 246 s, 142 puis 255 nœuds, zoomer, déplacer la vue,
sélectionner, décoder des images ; **aucune flèche tirée, aucun texte écrit**. 51 images par
seconde ; des images **au repos** à 25-30 ms vers la quinzième seconde (`blit`, `soumettre`,
`docks`) ; et **« sélectionner » : `textures` 12,35 ms au p99** — cause trouvée, corrigée au
§ 7.

---

## 2. La flèche qui se voit pendant qu'on la tire (GESTE-1)

**Ce qu'on croyait** : le rendu oublie la flèche. **Ce qui était** : l'index spatial ne se
resynchronise que quand `store.version` avance, et la version n'avance qu'**à la fin** d'un
geste (`end_live_edit`). Une flèche née pendant le geste n'avait aucun rang dans l'index,
donc aucune passe ne la dessinait. Même défaut latent : une annotation posée pendant un geste
décalait d'un cran les rangs des dossiers.

`glucose_core::quadtree::geste::SuiviDuGeste` rend l'union de ce que l'index connaît — relu
dans la numérotation du présent grâce aux longueurs que l'index retient — et de ce que le geste
ouvert a touché (`Journal::en_cours`). Le suivi est **incrémental** : chaque image ne lit que
les écritures nouvelles (sinon m mouvements × k nœuds). Un retrait ou une pose au milieu rend
le geste « illisible », et l'index se remet d'accord. Branché dans `renderer/cadrage.rs`.
Huit épreuves du noyau, une de bout en bout ; huit sabotages tombent.

---

## 3. Le texte sur fond noir, le contour en lueur (LUEUR-1, LUEUR-2, HALO-2)

Chez Tauri, l'ombre CSS d'une boîte est **découpée à l'intérieur** de la boîte (la norme de
`box-shadow`), et le fond ne vaut que 3 % de la teinte. Rust peignait la lueur, puis un fond
presque opaque à 12 % et un filet. Désormais : une **brume** à 3 %, aucun filet au repos, et la
lueur multipliée par `1 − couverture` de la carte arrondie — la distance exacte du noyau
(`membrane_forme::Arrondi::distance`), sur les deux voies. Au processeur, la lueur découpée
coûte **moins** (10,7 contre 13,7 ms sur une lueur géante 2560 × 1440) : l'intérieur est sauté.

**La carte désignée** — celle qu'une flèche en train de naître vise, celle dont elle partirait
sous l'outil flèche armé, les bouts d'une flèche survolée — prend la lueur vive de Tauri
(`0 0 80px 40px` à 40 %). LUEUR-2 : elle s'avive en **200 ms** (la transition CSS de Tauri), sur
l'amorti commun, et s'éteint sans sauter ; le réveil demande des images tant qu'une lueur
glisse.

**HALO-2** : la portée du flou s'arrêtait à la dernière *frontière* de pixel au lieu du point
exact où la queue interpolée passe le demi-niveau — une rangée de lueur au niveau 1 coupée,
**sur les deux voies à la fois**, donc invisible à leur épreuve d'accord. Trouvé par une
référence calculée pixel par pixel.

---

## 4. Les tailles d'écran du canevas en pixels logiques (DPI-1)

Sur son écran à 150 %, les barres suivaient l'échelle, le canevas non : flèches, poignées,
bandes de clic, étiquettes étaient **d'un tiers plus petites** que chez Tauri — une part de sa
plainte n° 8 (*« il faut viser PILE »*). `WorldScale::new(zoom, densite)` : une taille
**d'écran** se multiplie par la densité ; une taille du **monde** suit le zoom seul. Le clic
suit : le noyau parle en pixels logiques (`zoom_logique = zoom / densité`). Les filets d'un
pixel restent un pixel physique, délibérément. Épreuves à 150 % ; huit sabotages sur huit.

---

## 5. Les barres, la fenêtre d'ancrage, les passages (BARRES-1, ANCRE-UX)

* **Les barres du bas** : corps 13, boutons encadrés de 23 points, barre de 37 (Tauri ~37), la
  palette de Tauri. « Ancrer… » devient un bouton crayon **« Éditer le texte lié »**, allumé si
  la flèche désigne déjà un passage.
* **La fenêtre d'ancrage** — l'`ArrowTextEditor` de Tauri : un voile, SOURCE puis CIBLE à la
  couleur de la carte, les étapes 1 · 2, la consigne et la touche `Ctrl`, la zone de texte,
  « Sélectionné (n) » avec une puce par passage et sa croix, « Annuler » et le bouton principal,
  la molette qui défile. **La règle** : le texte y est mis en page par la loi même de la carte
  (`card_text_layout`), une unité monde = un point, et un clic rend un octet de la source — ce
  qui faisait diverger les ancres de Tauri n'existe pas. Regard :
  `cargo run --release -p glucose-desktop --example apercu_ancrage -- <dossier>`.
* **Les passages** (`renderer/passages.rs`, réécrit) : le cadre épouse la **police** (montante →
  descente), pas la ligne de 1,4 corps qui le faisait paraître décalé vers le bas — mesuré sur
  l'image à cinq zooms, l'horizontale était déjà juste au pixel près ; le texte du passage est
  repeint à la teinte, comme le `<mark>` de Tauri ; une **formule est un atome** : le cadre prend
  sa boîte dessinée, et un choix qui la touche la prend entière.

**Pas reproduit** : le défaut précis de *sa* capture (un cadre mal placé) n'a pas été retrouvé
au processeur. Il reste possible sur la voie graphique — à regarder s'il le revoit.

---

## 6. Ce qu'une frappe coûte (COMPOSANT-3)

### 6.1 La cause

La phase du curseur et sa place étaient dans la **clé** de la texture de la carte éditée :
chaque clignotement (deux par seconde) et chaque flèche du clavier re-rastérisaient toute la
carte. Hypothèse démentie en chemin, par la session coupée : créer une texture neuve et la
téléverser coûte 0,4 à 0,8 ms sur ses deux cartes graphiques — ce n'est pas l'envoi.

### 6.2 Le curseur est un ornement (`f2322dd`)

La session coupée avait commencé par donner à la texture une copie de la saisie au curseur
éteint. La reprise a préféré la **construction** : le contenu d'une carte **ne peint plus
jamais** le curseur. Il vit avec les poignées (`card/dessus.rs`), et la voie processeur devient
littéralement « contenu, puis ornements » — les deux mêmes fonctions que la voie graphique
(texture, puis couche du dessus). La clé ignore la phase et la place du curseur **parce que le
dessin les ignore**, pas parce qu'on a pensé à éteindre une copie ; elle suit une sélection
étendue et son étendue. Cent images de saisie immobile : **une** texture au lieu de deux.

Ce que les sabotages ont trouvé :

* une passe qui oublierait le curseur ne faisait rien tomber — une épreuve de bout en bout le
  tient désormais (il clignote dans la couche du dessus, sans refaire aucune texture) ;
* **la place du curseur n'était éprouvée nulle part** : décalé d'une ligne, rien ne tombait.
  Elle l'est, lue sur l'image (différence curseur allumé / éteint) contre les bandes de lignes ;
* deux règles du curseur n'avaient **aucun cas** : « un curseur dans le préfixe d'un bloc se
  rattache à sa première ligne » (une première ligne commence toujours au début du paragraphe)
  et « la dernière ligne recueille ce qui dépasse la fin du texte ». Elles disparaissent, et
  l'invariant qui les rend inutiles — **toute position du texte appartient à une ligne** — est
  éprouvé sur chaque genre de bloc (titres, citations, listes, code, tableaux, formules).

### 6.3 La prévisualisation d'une formule aussi (`8bd78f8`)

La pastille qui montre la formule qu'on écrit suit le curseur et se pose hors de la carte :
c'est un ornement. Dans le contenu, elle empêchait la carte d'être une texture, et toute la
carte retombait au processeur **à chaque image** tant que le curseur traversait une formule.
L'exception disparaît avec ses deux fonctions ; une carte qu'on édite est toujours une texture.

Deux défauts trouvés en relisant, chacun avec son épreuve : sortis du contenu, le curseur et
la pastille se peignaient **sous le seuil de détail** (SCALE-2), sur une carte réduite à son
cadre — une régression de mon propre premier commit ; et basculée à gauche, la pastille gardait
un écart au corps de référence quand celui de droite suivait le zoom.

---

## 7. La brume par la loi, l'anneau au-dessus (COMPOSANT-4)

### 7.1 Ce qu'une frappe coûte vraiment — mesuré sur une carte comme les siennes

`bench_composant` ne ressemblait pas à son cas (son texte « long » tient en deux lignes).
`bench_saisie` (nouveau) mesure sa carte type : la plus grosse de son document du 25/09 —
titre, sous-titre, deux paragraphes, 550 unités de large (lu dans son fichier, **sans y
écrire** ; son texte n'est pas recopié) — vue à ×2,3, ce qui donne les ~800 kpx de sa chronique.

| sa carte de notes | avant | après COMPOSANT-4 |
|---|---:|---:|
| rendu à ×1 (131 kpx), minimum | 0,38 ms | **0,26 ms** |
| rendu à ×2,3 (677 kpx), minimum | 2,05 ms | **1,55 ms** |
| dont la brume seule, à ×2,3 | **1,54 ms** | 1,11 ms |
| une frappe entière sur la carte graphique à ×2,3 (rendu + création + envoi), médiane | 2,19 ms | **1,79 ms** |

**Les trois quarts d'une frappe étaient la brume** — un voile uniforme à 3 %, peint par le
remplissage anticrénelé de `tiny-skia`. Le minimum est rapporté à côté de la médiane : pour un
travail déterministe, il dit le coût propre ; la médiane a varié du simple au double d'un
passage à l'autre sur cette machine.

### 7.2 La brume par la loi des membranes

Une brume est une membrane dégénérée : un champ d'une seule couche. `Membrane::plein` la décrit,
et le peintre des membranes la peint — une composition par pixel là où elle est constante, le
calcul exact au seul bord. Une seule loi pour les membranes, la lueur découpée et la brume.

**Pourquoi pas plus loin** : la sortir de la texture (dans le nuanceur qui la pose) l'annulerait
sur la voie graphique. Mais c'est un chantier de nuanceur pour une milliseconde à fort zoom,
alors que le coût mesuré chez lui reste inexpliqué (§ 7.4). Le mesurer d'abord.

### 7.3 L'anneau au-dessus

L'anneau d'une carte sélectionnée ou éditée était peint dans la texture, et la sélection entrait
dans sa clé : **sélectionner une carte la re-rastérisait entière** — les 12,35 ms de son p99. Il
rejoint les ornements ; la texture ne connaît plus la sélection (le type l'interdit : le
sabotage qui voudrait la lui rendre ne compile pas).

**Et une tolérance disparaît avec sa cause.** L'anneau était le dernier trait de `tiny-skia`
qu'une texture portait, et `tiny-skia` arrondit ses bords selon leur position absolue : l'accord
des deux voies tolérait **26 niveaux sur 200 canaux** pour une carte sélectionnée ou en saisie,
et **une sous-ligne de 64 niveaux** au contour des tuiles d'une carte vue de près. Mesuré après :
**zéro pixel différent**, partout. Les épreuves sont devenues des égalités strictes ; la fonction
du « contour » et sa constante ont disparu.

Témoins : 86 000 pixels changent, **au plus deux niveaux**, tous au contour des cartes (l'arrondi
de leur bord) ; l'intérieur est identique. Regardés agrandis, la différence amplifiée cent fois.

### 7.4 Ce qui n'est pas expliqué

Sa chronique donne **11,59 ms** de `textures` par image en écrivant ; le banc, **1,8 à 2,2 ms**
pour la même carte, sur la même machine, envoi compris. Un facteur cinq. Hypothèse testée et
**démentie** : le cache de glyphes, partagé avec toute l'interface et plafonné à 4 096
variantes, déborderait et ferait rendre chaque frappe à froid (7 ms à ×2,3) — une image n'en
laisse qu'environ 400. Restent : un processeur plus lent chez lui pendant l'usage (fréquence,
chaleur, autres programmes), ou un envoi plus lent qu'au banc. **La chronique gagne un compteur,
`rendu_us`**, qui sépare le rendu de l'envoi dans ses images les plus lentes : sa prochaine
session tranchera.

**L'étape 2** — ne refaire que les lignes qu'une frappe change — est **remise** : elle est
devenue possible (la brume par la loi et le texte sont invariants par découpage, ce qui levait
l'objection de la fiche 24), mais la frappe entière coûte 0,26 ms à ×1 et 1,55 à ×2,3 au banc.
Ajouter la complexité des bandes avant de savoir d'où viennent les 11,6 ms du terrain serait
optimiser à l'aveugle.

---

## 8. Ce que la session coupée a cru, et que la mesure a démenti

* « La flèche est invisible parce que le rendu l'oublie » → c'est l'index qui ne connaît que le
  document publié.
* « Le texte d'une carte déborde dans la lueur », vu sur un témoin → ce sont les documents
  **synthétiques** qui n'étaient pas mesurés ; l'application mesure tout document qu'elle ouvre.
* « Le cadre d'un passage est mal placé horizontalement » → mesuré à cinq zooms : l'horizontale
  est juste au pixel près ; c'était la verticale (la ligne au lieu de la police).
* « Créer une texture neuve coûte cher » → 0,4-0,8 ms.
* Et la reprise : « le cache de glyphes déborde » → démenti (§ 7.4).

Les épreuves **aveugles** trouvées en sabotant, sur les deux sessions : la portée exacte du flou
ne tombait qu'avec un balayage de positions fractionnaires ; la voie graphique des flèches n'était
pas éprouvée à 150 % ; la formule-atome n'était éprouvée qu'en fonction isolée ; le numéro
d'étape allumé, pas du tout ; le réveil de la lueur passait grâce au message d'accueil d'une
application neuve ; l'oubli d'une carte éteinte n'était pas observable ; la place du curseur ;
le curseur oublié par la passe ; la hauteur de la pastille ; l'anneau oublié.

---

## 9. L'aimant dès le premier placement (PLACEMENT-1)

L'entrée 3 de son registre : *« le snap intelligent ne s'active qu'une fois qu'on édite le
placement ; pourquoi pas DÈS qu'on souhaite placer une première fois ? »* Un clic avec un outil
posait l'élément sous le curseur, sans aimant.

**Ce que font les autres** : FigJam — l'outil des notes choisi, *un aperçu de la note suit le
curseur*, un clic la dépose ([Figma, *Sticky notes in FigJam*](https://help.figma.com/hc/en-us/articles/1500004414322-Sticky-notes-in-FigJam)).
Miro n'en dit rien dans sa documentation.

Sous un outil qui crée une boîte — carte, pense-bête, membrane, dossier —, un **fantôme** de
l'élément suit le curseur à sa taille de naissance, aimanté comme un glisser (même seuil, en
pixels logiques), guides compris ; le clic le pose là où il est. Une seule fonction dit où
l'élément se poserait : le fantôme la lit au mouvement, le clic au clic, et aucun état ne passe
de l'un à l'autre — le clic ne peut pas poser ailleurs que là où le fantôme était. Les cibles de
l'aimant se relèvent une fois par état du document, pas à chaque mouvement. Le fantôme a l'habit
de la sélection élastique : une affordance monochrome, qui ne ressemble à rien de ce qu'un
document contient.

Huit épreuves par le vrai chemin de la souris ; huit sabotages tombent — le seuil à 150 %
était aveugle avant la sienne.

---

## 10. Le registre de Tauri, vérifié entrée par entrée

Toutes les entrées ont désormais un état (fiche 39). En bref :

| n° | sujet | état dans Rust |
|---|---|---|
| 1 | ancres de texte | corrigé, puis la vraie fenêtre (§ 5) |
| 2 | alignement des membranes et dossiers | **absent** : ils s'aimantent et servent de cibles |
| 3 | aimant au premier placement | **corrigé** (§ 9) |
| 4 | tiroirs en haut à gauche, sans croix | **absent** : c'est déjà ainsi |
| 6 | icône d'aimant en couleur | **absent** : le message n'a pas d'icône |
| 7 | texte en retard quand la vue bouge | **absent par construction** : une seule image |
| 8 | ordre de priorité au clic | **se reproduisait — corrigé** (§ 10.2) |
| 10 | Trans-domaines | **expliqué** (§ 10.1) |
| 11 | ordre des boutons | **absent** : déjà dans l'ordre voulu |
| 12 | Time Machine | pas de « compacter » ; **jalons datés** (§ 10.3) ; l'optimisation reste à faire |
| 17 | prévisualisation de l'écriture | **absent** : on écrit dans la carte |
| 5, 9 | plugins, Ollama | phase 7 |
| 13, 14 | membranes, rideaux | attendent la discussion |
| 15, 16, 18 | recopie d'image, niveaux, « aaaa » | à comprendre avec lui |

### 10.1 Trans-domaines, expliqué (entrée 10)

Il a demandé qu'on lui dise **tout** ce qui y touche. Dans **Glucose Tauri** (`ArrowSvgLayer.tsx`,
`Toolbar.tsx`, `store/index.ts`) :

* un état `transDomainVisible`, vrai au départ, que le bouton bascule ;
* une flèche est **trans-domaine** quand ses deux bouts portent des domaines d'un poids supérieur
  à 0,1 **et n'en partagent aucun** ;
* une telle flèche se dessine **en pointillés** (`6 4`), et **disparaît** quand le bouton est
  éteint. Rien d'autre.

D'où son constat, *« coché ou décoché, rien ne change »* : sans domaines assignés aux **deux**
bouts d'une flèche, aucune flèche n'est trans-domaine, et le bouton n'a rien à masquer. Dans
**Glucose Rust**, le bouton est posé, ne s'allume jamais, et le clic ne fait rien : sa fonction a
été retirée quand il a dit que ce n'était pas du tout la bonne (fiches 29-30). La règle de la
fiche 05 § 5.4 voudrait qu'il soit absent ou grisé ; il n'a pas été touché, faute de sa parole.

### 10.2 L'ordre au clic (entrée 8) — PICK-2 (`dd96904`)

Rust avait porté l'arbitre de Tauri **tel quel**, et son défaut avec : les contenus y étaient
classés par « intention » — image 30, note 40, texte 50 —, le tri se faisait d'abord par rang, et
la profondeur ne départageait qu'à rang égal. Une image sous une carte gagnait toujours. Et le
texte était **terminal** : le re-clic ne descendait jamais sous lui. La raison de Tauri était
écrite — *« un double-clic sur un bloc texte ouvre l'édition ; il ne peut donc pas être une étape
intermédiaire du cycle »*.

Cette prémisse ne tient pas dans Rust : l'édition s'ouvre par un **vrai double-clic** (deux clics
en moins de 350 ms), et le cycle ne descend qu'au relâchement d'un re-clic **plus lent** que
cette fenêtre. Le temps sépare les deux gestes.

PICK-2 : les **affordances fines** d'abord (poignée 0, bord de conteneur 10, trait de flèche 20),
puis **ce qui est peint au-dessus** — Glucose peint les photos, puis les cartes, puis les
pense-bêtes : note 30, texte 40, image 50 —, puis le corps des conteneurs (60). Son cas exact,
éprouvé par la vraie souris : une image sous une carte — le clic prend la carte, le re-clic lent
la photo, le double-clic ouvre la carte. Et un reste du DOM de Tauri disparaît : `dom_hint`, que
rien ne remplissait jamais.

Les poignées « trop petites » : c'était surtout DPI-1 (§ 4) — une prise de 24 pixels logiques,
prioritaire sur tout.

### 10.3 Les jalons datés (entrée 12, `35317e2`)

*« Les jalons : oui, avec la date exacte. »* Un jalon disait « il y a 3 h » ; il dit « nommé ·
25/09/2026 22:19 · geste 12 », le style court `fr-FR` des dates de Tauri. L'heure locale d'une
date passée demande **la règle d'heure d'été de cette date-là** — seul le système la connaît
(`SystemTimeToTzSpecificLocalTime`, une fonctionnalité de plus du paquet `windows` déjà
présent). L'épreuve confronte deux chemins du système (l'heure que `GetLocalTime` donne
maintenant, et la conversion de l'instant présent) : la première version acceptait tout fuseau,
et était aveugle à une conversion restée en temps universel.

---

## 11. Vu en chemin, pas corrigé

* **L'histoire écrit un pas par mouvement de souris.** Pendant un glisser, chaque mouvement ajoute
  une translation à la transaction ouverte, et chacune s'écrit dans le fichier — environ 1 200
  pour cinq secondes à 240 Hz. C'est une part de *« elle enregistre beaucoup de choses
  inutiles »* (entrée 12). **Les fusionner n'est pas anodin** : additionner les pas en virgule
  flottante ne redonne pas bit pour bit la position vivante, et l'épreuve fondatrice de
  l'histoire (« le fichier relu redonne exactement le document ») tomberait. La correction
  juste change la façon dont le glisser applique ses pas : une position = la position de départ
  + le déplacement **total** du geste, une seule addition — ce que rejoue exactement une seule
  translation. À faire.
* **L'optimisation de la Time Machine** (entrée 12, *« définir mathématiquement une
  optimisation, et garder une quarantaine d'étapes clés »*). Une proposition : chaque geste a un
  poids — ce qu'il change (`Transaction::weight`, JRN-1) — et leur cumul est une courbe du
  changement. Les **étapes clés** sont les points qui la découpent en parts **égales de
  changement** : là où l'on a beaucoup travaillé, beaucoup d'étapes ; là où rien ne bougeait,
  presque aucune. Et « une quarantaine » n'a pas à être un nombre écrit : c'est ce que la
  réglette montre lisiblement — sa largeur divisée par l'écart minimal entre deux traits —,
  soit une quarantaine pour son panneau. Aucune donnée n'est perdue : c'est une **vue**. Que
  « optimiser » doive aussi **effacer** les gestes intermédiaires est une décision à lui.
* **Trois parcours du tableau entier à chaque mouvement de souris**, qui ne tiendront pas à dix
  millions de nœuds : `Store::move_selected` (chaque mouvement d'un glisser), l'aimant
  (`snap_move` compare le rectangle à toutes les cibles), et `arrow::snap_to_nearest` sous
  l'outil flèche. Le remède est le même pour les trois : l'index spatial, que le rendu et
  l'arbitre de clic interrogent déjà. Le glisser demande en plus un index par identifiant, que
  le magasin n'a pas.
* **Une formule en ligne** (`$…$` au milieu d'une phrase) reste affichée en source ; seule une
  formule qui occupe tout un paragraphe se rend. Tauri la rendait par KaTeX. C'est peut-être une
  part de ses *« problèmes de LaTeX »*, et c'est un vrai chantier : un atome insécable dans la
  coupe des lignes, une boîte mesurée, un clic atomique.
* **Le bouton Trans-domaines** ne fait rien et ne le dit pas ; la fiche 05 § 5.4 voudrait qu'il
  soit absent ou grisé (§ 10.1).
* Sa session de 22 h 19 : des images **au repos** à 25-30 ms vers la quinzième seconde (`blit`,
  `soumettre`, `docks`), et les panneaux à 11,6 ms au p99 au repos dans la précédente. Pas
  encore départagé.

---

## 12. Ce qui n'est pas fait, ou pas prouvé

* **Vu à l'écran** : rien de cette fiche.
* **Les 11,6 ms de `textures` pendant qu'on écrit** : pas reproduits (1,8-2,2 ms au banc, même
  machine) ; le compteur `rendu_us` départagera (§ 7.4). L'étape 2 — ne refaire que les lignes
  changées — attend cette mesure.
* **Les flèches** : la description longue (le badge « i » et son panneau Markdown), le portail
  vers un autre tableau, l'étiquette dans la barre d'options — pas encore faits. Dans Tauri,
  **rien ne créait un portail** : aucun geste de l'interface n'écrivait `targetBoardId` ; seuls un
  document importé ou le serveur MCP en portaient. Le modèle de Rust a déjà les deux champs
  (`long_text`, `target_board_id`), lus des documents de Tauri, jamais montrés.
* Le défaut précis de **sa capture d'un passage mal encadré** n'a pas été reproduit au
  processeur ; possible sur la voie graphique (§ 5).
* La date des jalons **hors de Windows** : la durée relative, faute de fuseau connu.

---

## 13. Ce qu'il faut regarder à l'écran

```text
cargo run --release > sortie-reprise.txt 2>&1
```

Sa chronique de 22 h 19 est déjà mise de côté. Avant de relancer **après** cet essai, copier
`%TEMP%\glucose-chronique\derniere-session.txt` en `sortie-chronique-2026-09-26-reprise.txt`.

1. **Relier deux cartes** : la flèche doit se voir **pendant** le glisser ; la carte visée
   s'aviver en douceur ; la carte de départ s'aviver déjà sous l'outil Flèche armé.
2. **Les cartes** : le texte sur fond noir, le contour en lueur ; les flèches et les poignées
   plus épaisses (son écran à 150 %).
3. **Sélectionner une flèche** : la barre plus grande, les boutons encadrés ; « Éditer le texte
   lié » ouvre la fenêtre — SOURCE puis CIBLE, les puces et leurs croix, la molette sur un long
   texte.
4. **Survoler une flèche ancrée** : le passage brille, le cadre serré sur le texte, à la couleur
   de la carte ; sur une formule, le cadre épouse la formule dessinée.
5. **Écrire dans une carte** pendant un moment, puis dans une formule `$$…$$` : dire si c'est
   plus fluide ; la pastille de la formule apparaît à côté pendant qu'on l'écrit.
6. **Le fantôme** : choisir l'outil Texte (ou Note, Membrane, Dossier) et approcher le curseur
   d'une carte existante — le fantôme s'aligne sur elle, un guide le montre ; cliquer : l'élément
   naît aligné.
7. **Une carte posée sur une photo** : cliquer sur la carte la prend (et non la photo) ;
   recliquer lentement au même endroit prend la photo ; un double-clic ouvre la carte.
8. **La Time Machine** (`Ctrl+H`) : les jalons disent leur date exacte.

---

## 14. Ce qui attend sa parole

1. **Le contournement automatique des flèches** (FLECHE-5) : sa réponse n'est jamais arrivée.
   Tauri faisait passer une flèche **autour** des cartes qu'elle traverse (`getDynamicRoute`, un
   algorithme glouton récursif sur les coins des obstacles). Mieux : un A* sur une grille non
   uniforme, comme les flèches coudées d'Excalidraw, ou un graphe de visibilité — avec l'index
   spatial et un cache, pour tenir dix millions de nœuds.
2. **Les membranes** (fiche 38 § 8) et **les rideaux** (fiche 39 § 14) : la discussion reste
   ouverte. Il a demandé **des références de logiciels** pour les rideaux ; en voici quatre, chacune
   pour une facette de ce qu'il décrit :
   * **Heptabase** — des *sous-tableaux* posés dans un tableau, imbriqués à volonté : un espace à
     soi, rattaché à un endroit précis de la carte ([wiki Heptabase](https://wiki.heptabase.com/organize-knowledge-and-projects)).
   * **Notion, *Side Peek*** — une page s'ouvre en panneau par-dessus celle où l'on est, sans la
     quitter : la **fenêtre flottante** du rideau ([Notion](https://www.notion.com/help/navigate-with-the-sidebar)).
   * **Miro, *Private mode*** — ce qu'on écrit reste invisible aux autres jusqu'à ce qu'on le
     révèle : le rideau **qui n'est qu'à soi** en collaboration ([Miro](https://help.miro.com/hc/en-us/articles/9794413310482-Private-mode)).
   * **Muse** — des tableaux dans des tableaux, dans une interface qui zoome : la **transition
     fluide** entre le focus et le rideau ([Ink & Switch, *Muse*](https://www.inkandswitch.com/muse/)).
3. **Trans-domaines** (§ 10.1) : ce que le bouton doit être — ou son retrait.
4. **La Time Machine** (§ 11) : que « optimiser » soit une vue des étapes clés, ou qu'il efface.

---

## 15. Les sources

* tldraw, [*Arrow binding options*](https://tldraw.dev/examples/arrow-binding-options) — les
  indices qui disent à quoi une flèche se liera.
* Excalidraw, la distance de liaison qui suit le zoom : [PR 8927](https://github.com/excalidraw/excalidraw/pull/8927),
  et [le système de liaison des éléments](https://deepwiki.com/excalidraw/excalidraw/3.2-element-binding-system).
* Cibles tactiles en unités logiques : [Material 3, 48 dp](https://m3.material.io/foundations/designing/structure),
  [Apple 44 pt](https://blog.logrocket.com/ux-design/all-accessible-touch-target-sizes/) — le
  principe de DPI-1.
* W3C, *CSS Backgrounds and Borders* : l'ombre extérieure d'une boîte est découpée à l'intérieur
  de sa bordure — le principe de LUEUR-1.
* Figma, [*Sticky notes in FigJam*](https://help.figma.com/hc/en-us/articles/1500004414322-Sticky-notes-in-FigJam)
  — l'aperçu qui suit le curseur avant de poser (PLACEMENT-1).
* Microsoft, [`SystemTimeToTzSpecificLocalTime`](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/nf-timezoneapi-systemtimetotzspecificlocaltime)
  — l'heure locale d'une date passée, heure d'été de cette date comprise.
* Pour les rideaux : [Heptabase](https://wiki.heptabase.com/organize-knowledge-and-projects),
  [Notion](https://www.notion.com/help/navigate-with-the-sidebar),
  [Miro](https://help.miro.com/hc/en-us/articles/9794413310482-Private-mode),
  [Muse](https://www.inkandswitch.com/muse/).
* Glucose Tauri : `ArrowSvgLayer.tsx`, `ArrowDescriptionPanel.tsx`, `ArrowOptions.tsx`,
  `ArrowTextEditor.tsx`, `canvas/hitPriority.ts`, `Toolbar.tsx`, `HtmlAnnotationLayer.tsx`.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
