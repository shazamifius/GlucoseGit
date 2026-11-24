# 137 — La sélection en équerres, et le reste du canevas flouté (Q12)

> Le 24/11/2026, à partir de 2 h, session du relais (compte b, sa semaine à 27 %). Sa consigne au
> relais (VISION § 9) ; la passation du compte a (23 h 21) : Q12 dans `main`, puis TEXT-FIT-3, puis
> l'archive. Sa réponse finale à Q12 (VISION § 8, sous l'image D) : *« les carrés avec le flou, mais
> alors au moins 60 % moins que ce qu'il y a actuellement »*.

## 1. Le dernier fichier de sabotages jamais joué : `detourage-121`, 25 sur 25

Le compte a l'avait lancé à 23 h 24 et fut coupé au mur après dix sabotages (tous tombés), **sans
restaurer** : l'arbre détaché gardait le onzième (`modele.rs`, « un tenseur se lit avant d'être
écrit »). Remis au propre (`git checkout`), les quinze restants rejoués d'un fichier tiré du premier
(`detourage-121.json`, rangs 10 à 24), sous `CARGO_PROFILE_DEV_OPT_LEVEL=1` : **15 sur 15**, donc
**25 sur 25**, chacun nommant son épreuve tombée. La cible optimisée est retirée.

**Le piège payé** : une session coupée au mur pendant `saboter.py` laisse son sabotage dans l'arbre.
La session suivante regarde `git -C ~/.cache/glucose-bancs/sabotage-arbre status` avant tout.

Et deux `pointeur.py` du pilote vivaient depuis trois et quatre heures, orphelins : leur compositeur
était mort sans que `piloter.sh arreter` soit appelé (des sessions coupées, elles aussi). Arrêtés.

## 2. Les équerres : un seul langage pour tous les nœuds (`927c809`, `8e018fc`)

La « 0 » de la fiche 133 disait la sélection de cinq façons : l'anneau blanc du texte, le fil
presque invisible du post-it, le cadre décollé de la photo (blanc, ou rouge verrouillée), le trait
plein qui remplaçait le pointillé d'une membrane, le halo et le voile doublé d'un dossier — et le
fil du groupe autour de tout. Sa lettre B (« ça j'aime bien au niveau design ») les remplace par
**une seule loi** (`renderer/equerres.rs`) : quatre coins de viseur autour de chaque nœud choisi,
posés sur un seul calque au-dessus de tout, comme tldraw pose ses indicateurs. Le nœud reste tel
quel.

* **Ce que l'équerre vise** : le nœud tel que l'écran le montre — la boîte montrée d'un texte
  (celle qui grandit pendant l'écriture, `card::boite_montree`), les quatre coins tournés d'une
  photo (`rotate::place`, la formule de ses poignées), la mise en page d'un dossier (son minimum
  de 180 × 120 compris). Le texte ou le post-it **qu'on écrit** est visé, choisi ou non (leur
  anneau se montrait) ; le titre d'une membrane qu'on écrit ne la vise pas (son bord ne changeait
  pas). Une flèche n'a pas d'équerres : ses deux bouts sont ses prises.
* **Les mesures** : 5 pixels d'écart, 14 de bras (au plus le quart du côté), 2 de trait — les
  nombres de l'image B, sauf le trait (2,5 anti-crénelé dans le prototype) posé en filets sur la
  grille, à 2 pixels, le minimum du W3C (WCAG 2.2, 2.4.13). Un cadre anti-crénelé coûtait 9,9 ms
  pour 243 photos choisies (`bench_ornements`) : un nœud droit pose huit filets, une photo penchée
  un tracé. **Plus fin qu'un pixel, le trait s'estompe** (POIGNEE-1, la loi du cadre des photos) :
  sinon, mille vignettes choisies de loin font la grille blanche de son retour du 07/10.
* **Les poignées se logent dans les angles** : une poignée de coin (9 pixels, centrée sur le coin)
  déborde de 5,1 pixels, l'équerre vit entre 3 et 5. Posées après les équerres, les poignées de la
  photo les recouvraient presque entières (des moignons, vu au pilote) ; les équerres se posent donc
  **après toutes les poignées** — l'angle de l'équerre enveloppe la poignée, sur chaque sorte.
* **Retirés** : l'anneau (et la marge que les textures des cartes lui réservaient), les cadres des
  photos (`Cadres`), le fil du groupe (ses poignées restent), le style choisi de la membrane et du
  dossier.

**Prouvé** : 6 épreuves neuves (chaque sorte porte ses équerres et rien d'autre ne change, le long
des bords comme au centre ; une flèche n'est pas visée ; le texte qu'on écrit l'est ; la vignette
estompe ; la verrouillée est rouge ; la penchée tourne ; la poignée se loge dans l'angle) ; les
épreuves des langages retirés réécrites selon la loi neuve, **deux aveugles retirées** (l'anneau
d'une carte et celui d'un post-it « s'estompaient de loin » : leur plage de lecture tombait sur une
poignée, elles passaient sans anneau) ; les empreintes de la sélection et du menu mises à jour,
l'écart regardé (8 017 pixels, tous autour des nœuds choisis). **10 sabotages sur 10**, après deux
reprises : l'épreuve de la flèche lisait un seul pixel où une équerre réduite ne passait pas (elle
lit désormais les deux coins de la boîte opposés au trait), et le sabotage du dossier touchait le
bandeau, que la loi plafonne — il ne changeait rien. **Regardé dans le vrai Glucose** au pilote
(la scène Q12, `~/.cache/glucose-bancs/selection-q12/q12-a137.png`).

## 3. Le flou du reste : une gaussienne exacte, la même sur les deux voies

**Combien.** Sa réponse : *« au moins 60 % moins que ce qu'il y a actuellement »*. L'image D
floutait à σ ≈ 14 pixels — réduite au quart (variance (4² − 1)/12 = 1,25), trois flous de boîte de
rayon 3 sur la petite grille (3 × 4 = 12, soit 192 sur la grande), agrandie au quart en bilinéaire
(4²/6 ≈ 2,67) : √195,9 ≈ 14,0. Au moins 60 % de moins : **σ = 40 % × 14 = 5,6 pixels logiques**
(`renderer/flou.rs`, `SIGMA`), multipliés par la densité de l'écran — un nombre de ressenti, le
sien, recompté par une épreuve.

**La loi** (`glucose_core::flou`, la recherche : Vello, `vello_common/filter/gaussian_blur.rs`, et
le cours de Stanford CS448F sur les filtres rapides). Flouter par une gaussienne, c'est convoluer,
et des convolutions qui se suivent **additionnent leurs variances**. Chaque étage réduit de moitié
par le binomial `[1, 3, 3, 1] / 8` (variance 0,75 sur la grille fine) et s'agrandit au retour par
l'interpolation `[¾, ¼]` (0,75) : 1,5 par étage, le reste divisé par 4 sur la grille deux fois plus
grossière, où une gaussienne séparable fait le reste. **Combien d'étages, sans seuil choisi** : on
réduit tant que le flou qui resterait sur la grille réduite efface encore sa propre fréquence de
Nyquist sous un niveau de huit bits, `exp(−π²σ²/2) ≤ 1/255`, soit `σ² ≥ 2 ln 255 / π²` ≈ 1,12
(Vello s'arrête à une variance de 4, choisie). Pour σ = 5,6 : deux étages, puis une gaussienne de
neuf coefficients (σ ≈ 1,22) sur la grille au quart. **Mesuré** : la réponse à une impulsion a la
variance annoncée à 1,5 % près (la troncature du noyau à 3σ), pour σ de 2 à 20, à quatre places sur
la grille, et garde sa masse.

**Ce qui reste net** : chaque nœud que ses équerres visent, et chaque flèche choisie — son tracé
ancré (ANCRE-1) épais de son halo ou de ses disques —, élargis d'une marge **déduite** : l'écart des
équerres plus une maille et demie de la grille du masque, car la lecture bilinéaire d'une grille
grossière ne vaut entièrement 1 qu'à une maille et demie de son bord. Le masque se rastérise une
fois, au processeur, sur la grille la plus grossière (tiny-skia, anti-crénelé) ; **les deux voies
lisent les mêmes octets**. Rien de visé à l'écran : aucun flou (un écran entièrement flou ne dirait
rien de la sélection). La bande de l'interface ne se floute jamais, ni la chrome.

**Les deux voies.** Au processeur (`renderer/flou.rs`), après la scène et avant l'interface : la
première réduction lit le tampon de l'écran tel quel (sans copie flottante de l'image entière), le
mélange `net × m + flou × (1 − m)` remplace les octets. Sur la carte (`present/flou_gpu.rs`,
`flou.wgsl`) : quand le flou vit, la scène — jusqu'à la couche du dessus — se compose dans une
texture à part, s'y réduit (quatre lectures bilinéaires à trois quarts de texel du centre : chacune
rend `(a + 3b)/4`), la gaussienne en deux passes par lectures exactes, remonte, et une dernière
passe mêle sur l'écran ; la chrome se pose par-dessus. `couches::composer`, que l'écran et le banc
des épreuves partagent, porte les deux chemins : sans flou, rien ne change. **Les deux voies
s'accordent** : sur la scène témoin choisie et floutée, pire écart 4 niveaux, 3 canaux sur 6,5
millions au-delà de 3 (la scène sans flou en admet 26).

**Quand.** La main au repos (`app/flou.rs`) : ni glisser, ni redimensionnement, ni tracé, ni coude,
ni rectangle de sélection, ni écriture, ni lecture — pendant qu'on déplace ce qu'on a choisi, il
faut voir où le poser. La caméra peut bouger : ce qu'on a choisi reste net. **Tant que le flou vit,
l'image se repeint entière** sur la voie du processeur (et celle qui le retire aussi) : une région
repeinte seule se peindrait nette dans un écran flouté.

**Le réglage** (*« TOUJOURS CHOISIR »*, 15/10) : « Flouter le reste » / « Ne plus flouter le
reste » au menu de la sélection et à la palette — deux commandes dont une seule s'applique
(APPLIQUE-1), l'entrée dit donc ce qu'elle va faire ; le choix se retient (`flou-de-la-selection`,
à côté de l'échelle de l'interface). Oui par défaut.

## 4. Prouvé, et ce que les sabotages ont appris

* **Les épreuves** : cinq dans le noyau (la variance d'une impulsion, la loi de Nyquist et son
  envers — le seuil de Vello réduit une fois un flou de σ = 2,3 que la loi garde entier —, le
  binomial, les quarts de l'agrandissement, le mélange à masque plein, vide et à moitié) ; trois au
  bureau (le choisi, ses équerres, la flèche et les poignées du groupe restent nets au pixel près, le
  témoin se floute, la bande jamais ; rien de visé, rien de flou ; σ recompté depuis l'image D) ;
  trois dans l'application (le flou s'efface pendant un glisser, un rectangle, une écriture ; le
  réglage l'éteint, le menu dit l'autre geste, le choix se retient ; une région salie ne se repeint
  pas seule sous le flou) ; et **l'accord des deux voies** sur la scène témoin floutée.
* **Les sabotages, 16 sur 16** (`flou-137.json`), après quatre reprises : l'épreuve du mélange ne
  mêlait qu'à masque plein ou vide (le raccourci `m ≥ 1` cachait la formule) ; « rien de visé »
  comparait deux rendus qui passaient tous deux par le flou (un témoin indépendant : le contraste
  des lettres) ; une réduction à mi-texel sur la carte restait sous la borne générale de l'accord
  des voies (1 ‰ au-delà de 3 niveaux : 6 248 canaux sous 6 480) — **le flou a sa borne, mesurée** :
  15 canaux sur 6,5 millions au-delà de 2 niveaux, au plus un sur cent mille (le sabotage : 17 160) ;
  et la garde de la bande ne protégeait rien — la chrome se pose après le flou sur les deux voies et
  la couvre — : **retirée** (principe 16), remplacée par le sabotage qui floute après l'interface.
* **Regardé au pilote**, par la carte graphique (la scène Q12, quatre nœuds choisis) : un défaut que
  les épreuves n'avaient pas — **les deux poignées du groupe qui ne tombent sur aucun nœud se
  floutaient en taches blanches** ; elles entrent désormais dans ce qui reste net (épreuve et
  sabotage). L'image finale, **E**, est posée dans son Glucose à côté de D (`relais-q12-e`).
* **Les grilles de la carte** : seize bits flottants par canal si l'adaptateur sait les cibler et les
  filtrer, sinon huit (certains pilotes OpenGL ES) — mesuré au banc en huit bits : le même accord.

## 5. Vu en chemin : l'arbitre condamne la RTX à tort (ARBITRE-DÉMARRAGE)

Sa sortie du 23/11 à 21 h 50 : *« arbitre : cette carte gèle — la carte économe s'ouvrira au
prochain lancement »* — sur un PC qui ne démarrait pas. Sa chronique (copiée,
`chronique-sienne.txt` du scratchpad) : dans les images lentes, `present` vaut **12 à 16 ms**, jamais
plus de 15,9. À 60 Hz, c'est l'attente du battement suivant de l'écran (la chaîne pleine quand Glucose
dessine plus vite que l'écran n'affiche), pas une carte qui gèle — et l'arbitre compte comme perdu
tout ce qui dépasse le plancher de la charte, 10 ms. **Sans effet sur son poste, vérifié** : Vulkan
n'y voit qu'une carte, la RTX (`vulkaninfo`, et `/sys/class/drm` ne montre que `card1`, 10de:2783,
l'écran sur DP-1) — la puce du Ryzen n'est pas exposée sous Linux, « économe » et « rapide »
ouvrent la même. **Son Glucose a donc été relancé** à 3 h 26 (la consigne : les relancements sont
au relais) et vérifié : la session d'avant fermée proprement, la RTX, son document, la porte des IA
qui répond à travers le relancement. Sur une machine à deux cartes, en revanche, le faux gel fait
balancier : le chantier suivant (fiche 138) — le plancher au battement de l'écran, pas d'arbitre
pour une seule carte, ECRAN-1 sous Linux par `/sys/class/drm`.

## 6. Choix pris seul, à contester

* **Toutes les sortes de nœuds prennent les équerres**, membranes et dossiers compris, et perdent leur
  style choisi (le trait plein de la membrane, le halo et le voile du dossier, conçus la veille pour
  OPACITE-1) : B disait « le nœud reste tel quel ». S'il regrette le dossier plus voyant, son style
  peut revenir à côté des équerres.
* **Le trait des équerres à 2 pixels** posés sur la grille, au lieu des 2,5 anti-crénelés de l'image B.
* **Le flou s'efface pendant toute la main au travail** (glisser, redimensionner, tracer, écrire),
  pas seulement pendant un glisser ; la caméra, elle, peut bouger sous le flou.
* **Rien de visé à l'écran, aucun flou** : une sélection hors de la vue ne floute pas l'écran entier.
* **« Flouter le reste » vit au menu de la sélection** (et à la palette), pas au menu du vide : c'est
  là qu'on est quand le flou gêne.
* **Le texte ou le post-it qu'on écrit porte ses équerres** (leur anneau se montrait) ; le titre d'une
  membrane qu'on écrit, non.

## 7. Vu, pas réglé

* **Aucune marque de mesure « flou »** : le serveur de télémétrie refuse un poste qu'il ne connaît pas,
  et se déploie avant la version qui l'écrit. Le coût du flou au processeur reste dans `img_scene_us`.
  À ajouter avec le prochain déploiement du serveur.
* **`test_queue_1_la_cascade_compte_la_fin_de_l_image` est tombée une fois** sous la suite entière, et
  passe seule trois fois sur trois : l'épreuve qui croit l'horloge (fiche 133 § 6).
* **Le coût du flou au processeur** n'est pas mesuré sur une vraie machine sans carte : une
  réduction, une gaussienne de neuf coefficients au quart, un mélange de l'écran entier, à chaque
  image où le flou vit et que la scène change — sur un fil.
