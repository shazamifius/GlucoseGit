# 99 — Ce qu'on écrit se voit ; la couleur d'un texte

> Session du 9/11/2026, à partir de 14 h 42, après sa parole de l'après-midi (VISION, fin de
> « Wikipédia sur le canevas »). Le prochain lot de SUITE, dans l'ordre : les défauts qu'il a vus
> (le double-clic, la couleur d'un texte), la roue de couleur, `Alt+Z`, les liens, le sens des
> couleurs des membranes.

## 1. Reproduire d'abord : un banc qui clique et qui tape

Sa parole : *« le système de double-clic fonctionne très bizarrement — on tape, les lettres
n'apparaissent pas ; ce n'est qu'après avoir cliqué ailleurs que le texte s'enregistre et
s'affiche »*. Avant toute hypothèse, il fallait le voir dans le vrai programme, et le banc hors
écran ne savait ni cliquer ni taper (le moteur de scénarios ne joue que des commandes).

**Le pilote** (`~/.cache/glucose-bancs/edition/piloter.sh`, hors du dépôt) : le compositeur sans
écran du banc (`sway`, moteur `headless`), où le vrai Glucose reçoit de **vrais** événements Wayland —
des frappes par `wtype` (un clavier virtuel), des clics par un **pointeur virtuel persistant**
(`wl/pointeur.py`, le protocole `wlr-virtual-pointer`, par `pywayland`), des captures par `grim`.
Rien sur son écran, rien ne vole son focus. Deux pièges payés :

* sway sans écran n'a **aucun** périphérique : ses propres commandes de curseur (`seat - cursor`)
  bougent un curseur que nul client ne reçoit — Glucose n'avait pas de souris. Un pointeur virtuel
  donne au siège sa capacité « pointeur » ; il doit vivre **avant** Glucose et entre deux ordres ;
* `wtype` perd la première touche (le clavier virtuel naît, sa disposition arrive après) : une
  pause d'amorce (`-s 300`) avant de taper.

**Ce que le banc a montré** (`run2/`) : sur un **texte** et un **post-it**, la frappe se voit
pendant qu'on tape. Sur le **titre d'une membrane**, rien — ni lettre, ni curseur — jusqu'au clic
ailleurs. Exactement sa parole, et précisément le geste qui fait une région : `Ctrl+G`, puis un
double-clic pour la nommer.

## 2. SAISIE-1 — le titre d'une membrane montre ce qu'on tape

**La cause** : le titre se dessinait toujours depuis le document (`MembraneVue::text`). La session
d'édition remplissait son tampon ; le texte et le post-it la recevaient, la flèche aussi — la
membrane, seule, avait été oubliée. Un oubli de branchement, pas un défaut du clavier.

**La réparation** : la saisie remplace le titre **à la source** (`pour_chaque_membrane`), donc le
titre, sa mesure et son ajustement à la largeur ne connaissent qu'un texte, celui qu'on voit ; un
titre vide qu'on écrit a sa place (le curseur s'y pose avant la première lettre) ; la sélection se
surligne sous les lettres (`Ctrl+A` dans un titre ne choisit plus un mot qu'on ne voit pas) ; le
curseur suit la phase qui se lit (BLINK-1). `renderer/scene/saisie.rs`.

**Une seule loi du curseur** (`renderer/curseur.rs`) : le texte et le post-it en avaient chacun une
copie — même largeur, même hauteur, mais l'une posée sur la grille des pixels, l'autre non (deux
demi-traits gris sur un post-it). Le titre l'emprunte ; les deux copies ont disparu.

**Prouvé** : écrire « salée » après « Eau » ne diffère, au pixel près, du titre « Eau salée » au
repos que d'une barre de curseur — de la hauteur d'une ligne de lettres, au bout du mot — et la
membrane voisine ne bouge pas ; un titre vide qu'on écrit montre son curseur ; une sélection se
voit. Le curseur fait 1,2 fois le corps, jambages compris : il mord d'un pixel sur le bord, comme
un « g » du titre — c'est l'épreuve qui était trop stricte, pas le curseur.

## 3. SAISIE-2 — ce qu'on écrit est à l'écran

**Ce que le banc a montré ensuite** (`run3/`) : le correctif marchait — et l'on ne voyait toujours
rien. `Ctrl+G` entoure ce qu'on a choisi, avec la place du titre au-dessus ; on choisit ce qu'on
voit, et quand cela touche le haut de l'écran, **la place du titre tombe sous la barre des
onglets**. On n'apercevait que le jambage du « g ». C'est, très probablement, aussi ce qu'il a vécu.

**La règle de tout éditeur** : le curseur qu'on écrit est à l'écran. Quand une saisie s'ouvre sur un
titre de membrane que l'écran ne montre pas en entier, la vue glisse **du strict nécessaire**, à la
même échelle, par le vol de `F` (`interactions/text_edit/montrer.rs`). Le titre garde autour de lui
**sa propre hauteur** d'air : aucune marge à choisir. Trop large pour l'écran, il en montre le début.
Un texte et un post-it ne bougent pas la vue : on les ouvre là où l'on vient de cliquer, et les
ramener entiers ferait sauter la vue sur un long texte dont on a double-cliqué un mot du milieu.
La bande du titre est une loi du noyau (`membrane_titre::bande`), que l'encre réutilise.

**Vu dans le vrai programme** (`run4/`) : tout choisir, `Ctrl+G`, double-clic dans la membrane
neuve — la vue descend, le curseur paraît à la place du titre, « Région » s'écrit lettre à lettre,
et reste après le clic ailleurs.

## 4. COULEUR-3 — la couleur d'un texte se voit, et son texte se lit pareil

Sa parole : *« les couleurs de text box ne fonctionnent pas »*. Le banc l'a confirmé (`run5/`) :
colorer « Un texte » en rose ou en orange ne changeait presque rien. La couleur d'un texte ne
touchait que sa lueur, ses puces et sa brume — à 3 % — ; sans puces, il ne restait que la lueur, et
chaque texte en a déjà une, à sa teinte « symbiotique » (sa place, ses voisins). Une couleur choisie
se perdait dans un bruit de couleurs qui ne dit rien. **Un bouton dont l'effet ne se voit pas ment.**

**La forme** : une couleur choisie **teinte le papier du texte sans changer sa clarté perçue**
(`renderer/card/habit.rs`, `papier`). Dans Oklch : la clarté du papier qu'on avait, la teinte
choisie, sa chroma — ramenée s'il le faut à la plus forte que l'écran sait montrer à cette clarté
(`glucose_core::couleur::papier_teinte`). La clarté fait le contraste : les lettres se lisent
exactement aussi bien, la couleur se voit de près comme de loin (le papier se peint sous le gris des
lignes), et **aucune opacité n'est à choisir** — la constante qu'il aurait fallu régler disparaît.
C'est aussi la règle du post-it, dont la couleur est le papier (Tauri).

**Le noyau gagne le retour** (`couleur/ecran.rs`) : Oklab → sRGB (les matrices inverses
d'Ottosson), et le bord du gamut — la plus forte chroma qu'un écran sRGB montre à une clarté et
une teinte données, par bissection. La roue de couleur en aura besoin.

**Regardé** (`run6/`) : « Un texte » prend un papier prune (rose), brun (orange), marine (bleu) ;
ses lettres blanches se lisent pareil. **La limite, dite** : à la clarté d'un papier sombre, l'orange
devient brun et le rose prune — c'est la nature de ces teintes dans le sombre —, et le rose, le
rouge et le violet s'y ressemblent plus qu'en clair. Si l'on voulait des papiers plus vifs, la
route serait de les éclaircir jusqu'au contraste « AAA » des guides d'accessibilité (7 : 1 avec des
lettres blanches) — une seule clarté pour tous, donnée par la norme et non réglée à l'œil ; mais
alors un texte coloré serait aussi plus clair qu'un autre, et cette clarté dirait « coloré » : à
peser avec lui.

**Prouvé** : dans le vrai rendu, le papier d'un texte rose a la teinte du rose, une chroma qu'on
distingue d'un gris, et la clarté du papier d'un texte sans couleur (à 0,015 près) ; sa texture
(la voie de la carte graphique) se repose au bit près sur le dessin du processeur ; l'aller-retour
Oklab rend chaque couleur, la chroma maximale est un bord (sur l'écran à elle, hors un millième
au-delà, sur 19 clartés × 36 teintes).

## 5. Une utilité pour la couleur des textes — trois propositions, pour sa parole

Sa parole : *« c'est toute une question aussi — au sein d'une membrane il y a souvent du texte, et du
texte de couleur : il faut proposer une utilité »*. Et sa règle : *« il faut que TOUT ait une
signification »*. Une couleur ne porte bien **qu'un** sens à la fois ; les voici.

**A — La couleur d'un texte dit le domaine dont il parle** (mon avis). La membrane dit **où** l'on
est (sa teinte : la proximité des domaines, sa parole du 9/11) ; le texte dirait **de quoi** il
parle — son domaine, celui qui « traverse tout » (sa définition du 22/10). Un texte qui parle du
sujet de sa région a la teinte de sa membrane ; un texte qui parle d'autre chose porte la teinte de
cet autre domaine : « l'eau dans les jeux vidéo », dans la membrane « Eau », prend la teinte du jeu
vidéo, et les ponts entre domaines se voient d'un regard. Choisir la couleur d'un texte, ce serait
choisir son domaine ; une IA le lit déjà (`glucose_lire` dit les domaines). Le domaine reçoit enfin
sa « couleur » (VISION : *le domaine, une infinité de dimensions, plus une couleur*) sans une seconde
machinerie. Et la teinte symbiotique — une couleur que la place décide, qui ne dit rien — céderait
la sienne : un texte sans domaine resterait neutre. **C'est une décision de vision** : la lueur
symbiotique est une marque de Tauri (« Teintes symbiotiques dynamiques », dit le nœud d'accueil).

**B — La couleur dit le rôle du texte dans un raisonnement** : question, idée, argument pour, contre,
décision, source — les nœuds d'IBIS (Rittel, années 1970) et de Compendium, l'outil de
« dialogue mapping ». C'est la grammaire des objectifs 3 et 4 (des graphes de questions et de
réponses, le document de recherche) ; une légende fixe de six ou sept rôles, qu'une IA lit comme un
type.

**C — La couleur dit la sûreté** : établi et sourcé, hypothèse, à vérifier, contredit — le
« référence nécessaire » de Wikipédia, et, pour le pont humain–IA, ce qu'une IA a écrit sans en
être sûre.

Mon ordre : **A** pour la couleur ; B et C pourraient prendre une forme qui n'est pas une couleur (une
pastille, un bord), le jour venu. La clarté et l'opacité d'un texte, elles, restent au service de
sa lecture : seule sa teinte porterait un sens.

## 6. RETOUR-1 — `Alt+Z`, revenir où l'on était

Sa parole : *« tu focus sur un bloc qu'il ne fallait pas, Alt+Z, et tu es de retour instantanément
à ton lieu d'origine »*. C'est le retour d'un navigateur, et les touches `[` et `]` de Maya
(« Undo View Change », « Redo View Change »,
[aide d'Autodesk](https://help.autodesk.com/cloudhelp/2025/ENU/Maya-KeyboardShortcuts/files/GUID-F5379746-7A11-48C0-AFBA-D4B333D3B63A.htm)).

* **On retient le lieu qu'on quitte à chaque départ du vol** — `F`, tout cadrer, un signet, un
  résultat de `Ctrl+F`, la place d'un domaine, le mode Focus, la minimap, SAISIE-2 : tous passent par
  `Vol::voler_vers` ou `Vol::viser`. Le vol note qu'il part (`parti`), l'image suivante — qui
  connaît la vue d'où il part — la range (`app/mouvement.rs`). **Un seul endroit**, quel que soit le
  geste : un saut ajouté demain sera retenu sans que personne y pense. Un vol redirigé en route
  n'ajoute rien : en plein vol, on n'est nulle part qu'on voudrait retrouver.
* **Le défilement et le zoom à la main ne sont pas des départs** : on ne revient pas d'un tour de
  molette, comme un navigateur ne revient pas d'un défilement. (À peser à son écran : un long voyage
  à la main — dézoomer, traverser, rezoomer — ne se remonte pas.)
* **`Alt+Z` est instantané** (son mot) : la vue se pose, sans vol, et ne se note donc pas comme un
  départ ; en plein vol, il arrête le vol. **`Alt+Maj+Z`** repart. Une navigation neuve efface
  l'avant. Un lieu dont le tableau a disparu (un onglet fermé) se saute.
* **L'histoire vit dans le magasin** (`store/retour.rs`), à côté de la vue de chaque tableau et de la
  pile des dossiers : c'est de la navigation, hors de l'annulation du document ; un document qu'on
  charge l'oublie (chaque document nomme son premier tableau `main` : un lieu de l'ancien
  s'appliquerait au nouveau) ; et le menu sait s'il y a où revenir. **Aucune borne** : un lieu pèse
  quelques dizaines d'octets.
* **Au doigt** (pas d'`Alt`) : « Vue précédente » au menu du vide, seulement s'il y a où revenir,
  comme « Annuler ». Les deux commandes sont à la palette (`vue.precedente`, `vue.suivante`) et se
  répètent au maintien, comme `Ctrl+Z`.

## 7. Les liens — la forme, à lui présenter avant de bâtir

Sa parole : *« c'est la première fois que j'autorise les niveaux de LOD »* — pour les liens seuls.
Très transparents de loin, ils apparaissent à l'approche d'un bloc de texte ; un clic sur un lien :
un focus sur ce qu'il rattache, ou un « show ».

**La recherche.** Ce qu'il décrit existe, et a été mesuré : **Bring & Go** (Moscovich, Chevalier,
Henry, Pietriga et Fekete, *Topology-Aware Navigation in Large Networks*, CHI 2009 —
[article](https://www.microsoft.com/en-us/research/publication/topology-aware-navigation-large-networks/),
[projet](https://www.aviz.fr/Research/Topology-aware-navigation)). Choisir un nœud **amène ses voisins
près de lui**, dans la vue : rangés sur des anneaux concentriques, **chacun dans la direction de sa
vraie place**, les plus proches d'abord sur l'anneau intérieur, sans chevauchement ; les liens
suivent, aplatis. Choisir un voisin **y va** : la vue vole (en dézoomant puis rezoomant) pendant que
tout reprend sa vraie place. Et cela s'enchaîne : un nouveau « Bring » sur un voisin amené. Dans leur
étude, **Bring & Go était nettement plus rapide** que le zoom et le défilement, que la vue
d'ensemble, et que **Link Sliding** (glisser le long d'un lien jusqu'à son bout). Leur argument
compte pour nous : des **mandataires** plutôt qu'une déformation de la carte gardent la constance
spatiale — la carte ne bouge pas, ce qu'on amène est une copie temporaire. C'est exactement son
« show » : une fenêtre temporaire, les blocs liés placés à côté même s'ils sont à des kilomètres,
leur couleur d'origine, un clic sur un bloc défait la fenêtre et vole vers lui.

**Ma proposition, en trois pièces.**

1. **La transparence de loin** : un lien suit **la lisibilité des textes qu'il relie** — à peine une
   trace quand leurs lettres ne se lisent pas (sous les 2,8 pixels de la loi du gris, GRIS-1), plein
   quand elles se lisent. Le seuil n'est pas choisi : c'est celui où un texte devient du gris. Ce
   n'est pas une simplification de ce que la carte porte (le lien reste là, en trace) : c'est **la
   seule exception** qu'il a donnée, et elle tient dans une ligne de la loi des flèches.
2. **Le clic sur un lien** : un **focus** qui cadre ses deux bouts (le vol de `F` sur leurs deux
   boîtes) — et `Alt+Z` ramène où l'on était.
3. **Le « show » d'un nœud** (un geste à nommer : `Maj`+clic, ou le menu « Montrer ses liens ») : la
   fenêtre de Bring & Go — le nœud au centre, chaque nœud lié **amené** sur un anneau, dans la
   direction de sa vraie place, à sa taille de lecture, sa couleur d'origine, et **sa provenance**
   écrite dessous (*Sciences › Chimie › Eau*, le fil « où je suis » de la proposition C) ; on y zoome,
   on y lit ; un clic sur un bloc défait la fenêtre et vole vers sa vraie place ; `Échap` la défait
   sur place. Une IA lit déjà la même chose (les flèches d'un nœud, `glucose_lire`) : la fenêtre est
   la vue humaine de ce voisinage.

**Ce que j'aimerais qu'il tranche** : la transparence suit-elle le zoom (la lisibilité, ma
proposition) ou **le curseur** (les liens du bloc survolé s'allument) — ou les deux ; et le « show »
s'ouvre-t-il d'un clic sur un **lien** (comme il l'a dit) ou sur un **nœud** (comme Bring & Go, qui
amène tous ses liens d'un coup) ?

## 8. ROUE-1 — la roue de couleur, « à la Blender », dans Oklch

Sa parole : *« laisser l'utilisateur choisir la couleur qu'il souhaite, si possible le même système de
palette que Blender, avec la roue — elle est extrêmement ergonomique »* ; en revoir l'ergonomie et
les raccourcis.

**Ce que Blender fait** ([manuel, « Color Picker »](https://docs.blender.org/manual/en/5.2/interface/controls/templates/color_picker.html)) :
un champ de deux composantes (cinq formes : cercle TSV ou TSL, carrés), un curseur pour la troisième,
les champs RVB/TSV, l'hexadécimal, la pipette. `Ctrl`-glisser cale la teinte tous les 30°,
`Maj`-glisser affine, la molette règle la valeur ou la clarté, `Retour arrière` rend la valeur par
défaut. **Ce qu'on lui reproche** : son cercle est en TSV — un jaune et un bleu de même « valeur »
n'ont pas la même clarté à l'œil, et les teintes n'y sont pas réparties comme l'œil les distingue
(Ottosson, [*Okhsv and Okhsl*](https://bottosson.github.io/posts/colorpicker/), qui recommande Okhsl
« le meilleur compromis » : une clarté orthogonale).

**Mieux que Blender, sans approximation** (`glucose_core::couleur::roue`) : à la clarté du curseur,
l'angle est la **teinte d'Oklch** (des angles égaux, des écarts que l'œil voit égaux), et le rayon la
**part de la plus forte chroma que l'écran montre à cette clarté et cette teinte** — le bord exact du
gamut, calculé par COULEUR-3. Le bord du disque est donc toujours le bord de l'écran, chaque point
une couleur affichable, et **la clarté ne bouge pas quand on tourne** : c'est l'idée d'Okhsl, calculée
exactement là où Okhsl en donne une approximation lisse. Une conséquence honnête, à lui dire : à la
clarté d'un bleu moyen, le jaune paraît olive — un jaune vif n'existe qu'en clair ; on monte le
curseur. C'est la roue qui dit vrai, là où le cercle TSV mélange teinte et clarté.

**Les gestes** (`interactions/roue.rs`) : la case au disque arc-en-ciel, au bout de la barre de
couleur, l'ouvre (et la commande `couleur.roue`, à la palette) ; glisser dans le disque choisit
teinte et saturation, dans le curseur la clarté — **la couleur se pose en direct** sur tout ce qui est
choisi ; `Ctrl` cale la teinte tous les 30° ; `Maj` affine (le mouvement compte pour un dixième) ;
**la molette change la clarté d'une différence à peine visible** (0,02 en Oklab, le pas que l'œil
distingue : aucun pas à choisir) ; `Retour arrière` rend « Par défaut » ; `Échap` annule tout ;
`Entrée` ou un clic dehors valide — sans désélectionner. L'aperçu montre la couleur d'avant et
celle d'après, et son code `#rrggbb`. La case s'allume quand la couleur commune est hors du
nuancier. **Au doigt** : le doigt passe par les mêmes gestes que la souris (`toucher.rs`).

**Un seul geste du document** : à chaque mouvement, la roue repart des couleurs d'avant
(`cancel_live_edit`) et pose la nouvelle ; le journal ne porte qu'une écriture par nœud, quelle que
soit la longueur du glisser ; valider ferme le geste (un `Ctrl+Z`), annuler l'abandonne sans trace.
Ouverte, la roue tient le clavier et la souris, comme une question.

**Regardé** (`run8/`) : la roue s'ouvre au-dessus de la barre ; glisser dans le disque recolore la
membrane en direct (orange `#e78651`), l'aperçu et le curseur suivent ; un clic dehors valide, la
membrane reste choisie, la case de la roue s'allume.

**Une épreuve aveugle évitée** : le décor posait la membrane par un geste, et « un geste à défaire »
était donc vrai avant la roue. Le journal est vidé après le décor ; et les épreuves passent par les
vrais chemins (le curseur, le bouton, la molette, la touche), pour qu'un branchement oublié tombe.

**Reste, dit** : le champ hexadécimal ne se tape pas encore (il s'affiche) ; la pipette (prendre la
couleur d'un nœud ou d'une photo — la couleur d'une image est déjà le sommet de sa pyramide,
MOYENNE-1) ; les autres formes de Blender (les carrés) n'ont pas été reprises — le disque Oklch les
rend inutiles, à juger à son écran.

## 9. Ce qui n'a pas été fait, et pourquoi

* **Les liens** (SUITE, le prochain lot) : conçus et présentés (§ 7), pas bâtis — deux questions de
  forme lui reviennent (la transparence suit-elle le zoom ou le curseur ; le « show » s'ouvre-t-il
  sur un lien ou sur un nœud).
* **Le sens des couleurs des membranes** (teinte par proximité des domaines, clarté par
  accessibilité, opacité par appartenance) : pas commencé. Il demande d'abord une image faite par
  l'agent sur le squelette — et la proximité des domaines suppose une mesure entre domaines que
  Glucose n'a pas encore (des domaines qui partagent des nœuds ? des voisins sur la carte ?) : à
  concevoir avec la recherche (Tree Colors ne suffit pas, fiche 98).
* **La roue** : le champ hexadécimal ne se tape pas, la pipette n'existe pas (§ 8).
* **Un voyage à la main** (dézoomer, traverser, rezoomer à la molette) ne se remonte pas par
  `Alt+Z` : à peser à son écran ; la règle possible — retenir un arrêt de la vue qui ne partage plus
  rien de ce qu'elle montrait — se dit sans constante.
* **Le nœud d'accueil d'un premier lancement** se pose à l'origine même si le document y a déjà un
  nœud (vu au banc, sur un document d'essai fait par le banc des agents) : sans gêne d'usage
  constatée, noté.
* **GRIS-3, B, C, « Créer un sous-dossier », REPOS-3** : inchangés (fiche 98 § 7).

## 10. Les leçons

* **Reproduire avant de supposer** a payé deux fois : le défaut n'était ni le clavier ni une méthode
  de saisie (vérifié : aucune ne tourne), mais un titre qui se dessinait depuis le document — et une
  fois réparé, l'image du geste entier a montré un second défaut, le titre sous la barre.
* **Vérifier son instrument avant le logiciel** : le premier clic du banc « ne marchait pas » — c'est
  le banc qui n'avait pas de souris. Un simple clic qui choisit, regardé, avant de conclure.
* **Une épreuve est aussi aveugle par son décor** (le journal plein d'avance) **et par son chemin**
  (la fonction appelée directement) : les deux se voient en se demandant quel sabotage passerait.
* **Un cliquet arrête deux fois par session, et a raison deux fois** : un toast de trop (le retour
  sans lieu n'a pas à parler, le menu sait déjà se taire), une fonction trop longue (la molette :
  extraite).

## 11. Prouver — le compte de la session

* **Suite entière** : 3 002 épreuves passées, 7 ignorées (carte graphique absente) ; les quatre
  tombées au dernier passage — le compte des cases de la barre, deux empreintes témoins, le cliquet
  des 80 lignes — réparées et rejouées. `cargo fmt`, clippy et les cliquets à zéro.
* **Sabotages** : SAISIE-1 et 2, 10 sur 10 (`saisie-1-99.json`) ; COULEUR-3, 7 sur 7
  (`couleur-3-99.json`) ; RETOUR-1, 11 sur 11 (`retour-1-99.json`) ; ROUE-1, 15 sur 15 (`roue-1-99.json`).
  **En tout : 43 sabotages, 43 tombés.**
* **Le vrai programme, regardé** (`~/.cache/glucose-bancs/edition/run*/`) : la frappe dans un titre,
  `Ctrl+G` puis le nom, la couleur d'un texte, `F` puis `Alt+Z` et `Alt+Maj+Z`, le menu du vide, la
  roue ouverte, glissée, validée ; les images témoins comparées pixel à pixel avant de changer deux
  empreintes.
* **Pas à son écran** : rien de cette session.

## 12. Le sens des couleurs des membranes, en image — un premier prototype

Sa parole du 9/11 : la **teinte** dit la proximité des domaines (sciences, société, sciences sociales
ont des couleurs proches), la **clarté** l'accessibilité, l'**opacité** l'appartenance ; *« il faut que
TOUT ait une signification »*. Un agent colore le squelette par les seules commandes
(`outils/wikipedia/teintes_sens.py`, `GLUCOSE_SENS_EN_COULEURS=1 … essai_squelette.sh`) ; images :
`~/.cache/glucose-bancs/squelette-sens/images/`.

* **La teinte** : chaque région est le sac des mots de ses textes (TF-IDF), deux régions se
  ressemblent comme leurs sacs (le cosinus). Les neuf thèmes sont rangés sur le cercle par une
  **sériation circulaire exacte** — l'ordre qui rend la plus grande la somme des ressemblances entre
  voisins, les 20 160 cercles essayés —, chaque écart de teinte proportionnel à la dissemblance des
  voisins. Une sous-région prend la moyenne circulaire des teintes des thèmes, pondérée par sa
  ressemblance à chacun. **Le premier essai** (le positionnement multidimensionnel de Torgerson)
  rendait le bruit de neuf thèmes presque également dissemblables : Société et Sciences, les plus
  proches (0,14), tombaient à 200° l'un de l'autre — regardé, rejeté.
* **La clarté** : la lisibilité de Kandel et Moles (1958, Flesch pour le français), **par rang** — le
  plus lisible le plus clair, le moins lisible le plus sombre, les autres à égale distance. **Le
  premier essai**, linéaire, laissait un texte aberrant (−88) écraser l'échelle : tout tombait entre
  0,80 et 0,89, des pastels. Le plus sombre garde un titre lisible sur le fond (le contraste « AA »,
  4,5 : 1, une borne de la norme) ; la saturation, sans sens attribué, est la même pour toutes les
  teintes à une clarté donnée.
* **L'opacité** n'est pas peinte : une membrane n'a pas d'opacité dans Glucose (sa couleur est un
  `#rrggbb`). L'appartenance est **mesurée** — la ressemblance d'une région à sa parente — et écrite
  dans le compte rendu (de 0,09 pour une section qui dérive à 0,98). La peindre demande un champ au
  modèle : à lui de dire s'il le veut ainsi.

**Ce que l'image montre** : Histoire rose, Géographie verte, Arts bleus, Vie quotidienne turquoise,
Philosophie brune, Société orange brûlé, Sciences et médecine olive, Technologie lilas, Mathématiques
violettes ; les voisins du cercle sont les thèmes qui se ressemblent (Philosophie, Société, Sciences,
Géographie se suivent). Dans « Eau », les sections gardent l'olive du thème et varient en clarté :
« Symbolique », la plus lisible, presque blanche.

**Ce qui est faible, dit** : la ressemblance repose sur des résumés de deux phrases — les cosinus
vont de 0,05 à 0,14, une mesure pauvre ; avec les articles entiers, ou les liens de Wikipédia entre
articles, elle serait bien meilleure. La formule de lisibilité est grossière sur des textes courts.
L'olive des sciences n'est pas belle : c'est ce que le calcul donne à cette teinte et cette clarté.
Et les lueurs symbiotiques des textes ajoutent leur bruit sous la couleur qui dit la région (§ 5, A).
