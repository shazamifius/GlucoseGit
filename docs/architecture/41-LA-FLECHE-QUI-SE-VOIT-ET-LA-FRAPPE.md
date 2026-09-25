# 41 — La flèche qui se voit, et la frappe

> **Rôle de ce document.** Ses retours sur la fiche 40, puis deux sessions : la première a été
> **coupée** par lui au milieu d'un chantier parce qu'elle devenait trop longue ; la seconde
> a repris exactement là. Cette fiche couvre les deux. Ses retours (§ 1) : les flèches trop
> fines, les barres du bas trop petites, l'éditeur d'ancres qui n'est pas une vraie interface,
> les cartes « pleines » là où Tauri montre le texte sur fond noir et le contour en lueur, et
> la flèche **invisible** pendant qu'on la tire. La première session y a répondu (§ 2 à § 5) ;
> la seconde a fini le chantier coupé — ce qu'une frappe coûte (§ 6, § 7).
>
> **Date** : 2026-09-25 et 26 · commits `d0ba347`, `4c5c7d1`, `0113e8f`, `28a57aa` (la session
> coupée), puis `f2322dd`, `8bd78f8`, `e8abb5c`, et la suite.
> **État vérifié** : `cargo test --workspace` exit 0, **1 787 tests verts**, clippy strict à
> zéro, `cargo fmt --check` à zéro, l'application se construit ; après chaque suite,
> `%LOCALAPPDATA%\Glucose` n'a rien reçu.
>
> **Vu à l'écran** : rien de cette fiche encore. Il a lancé l'application le 25/09 à 22 h 19,
> quatre minutes, sans message (§ 1.2) : ni flèche tirée ni texte écrit. Le § 11 dit quoi
> regarder.

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

## 11. Ce qu'il faut regarder à l'écran

*(à compléter en fin de session)*

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
