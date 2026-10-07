# 52 — La carte de l'écran, les poignées au dézoom, le glisser qui se tait

> Session du 07/10/2026. La CI de `20deb6f` lue (neuf tâches vertes) ; puis trois de ses
> retours arrivés pendant le travail — les poignées au dézoom, le pincement trop lent, et
> **« je lag énormément et je ne sais pas du tout à quoi c'est dû »**. Rien de ce qui suit n'a
> encore été vu à son écran : la liste du § 7 le dira.

---

## 0. Ce qui précédait

* La CI de `20deb6f` : **neuf tâches vertes**, Linux et Mac compris. La voie hors Windows du
  presse-papiers (fiche 51) compile : c'est vérifié sur GitHub.
* `layout::organize_board_grid` n'avait plus d'appelant hors de ses épreuves, et Glucose Tauri
  n'a rien de tel (« ranger tout le tableau, textes compris ») : retirée avec ses trois aides
  et ses deux épreuves. Les dispositions d'Ordonner (`calculate_image_layout`) restent.

## 1. Le lag de sa session du matin : un balancier entre deux cartes (ECRAN-1)

**Ce qu'il a vécu** : la 2.0.1-beta.1 installée, lancée à 10 h 40, « je lag énormément ».

**Ce que sa boîte noire dit** (copiée, jamais touchée) : une image coûte **3 ms en médiane**,
mais 47 épisodes contiennent une image de 100 à 505 ms, et la cadence tombe à 20-40 images
par seconde par moments. Le gel tombe presque toujours sur **la première image après un
repos** : après un repos de deux secondes et plus, elle coûte ~50 ms à chaque fois ; les gros
gels (300-500 ms) suivent des repos courts, d'une demi-seconde à deux.

**Ce que la chronique dit** (écrite à 11 h 01) : les douze pires images passent **tout** leur
temps dans `present` (430 à 502 ms) ; la succession est **`fifo`**, alors qu'elle était
`mailbox` la veille au soir ; le tempo monte jusqu'à 8 balayages (30 images par seconde) sur
9 % des mouvements, à cause des ratés.

**Ce que Windows dit** :

* l'écran (2560 × 1600 à 240 Hz) est branché sur la **RTX** ; l'Intel n'affiche rien — son
  portable est en mode « carte dédiée directe » (le commutateur d'Armoury Crate) ;
* `dwm` tourne sur la RTX ; Glucose a 1,3 Go sur l'**Intel** et y fait l'essentiel de son
  travail 3D : il dessine sur l'Intel, et chaque image traverse vers la RTX.

**Ce que le banc dit** (`bench_reveil`, hors écran) : après un repos, l'Intel met **~50 ms**
à rendre une image dès 1,25 s de repos (7 ms à 0,75-1 s, moins d'une milliseconde avant) ; la
RTX répond en 0,4 ms quel que soit le repos. C'est la signature des ~50 ms de sa boîte noire.
Les gels de 450 ms ne se reproduisent pas hors écran : ils vivent dans la copie entre cartes et
la composition, que seul l'écran exerce.

**Pourquoi l'Intel** : l'arbitre (ARBITRE-1) part de l'économe et ne change de carte qu'au
**lancement suivant** (ARBITRE-4). La veille au soir, sur la RTX, un hoquet lui a fait retenir
l'économe — son propre code l'avouait : « deux cartes qui gèlent toutes deux le feraient
alterner ». Ce matin, tout s'est fait sur l'Intel, qui a gelé, et l'arbitre a retenu la RTX
pour la fois suivante. **Une session sur deux**. C'est déduit (`mailbox` hier, `fifo` et la
mémoire sur l'Intel ce matin, et seul l'arbitre écrit `carte.txt`), pas observé ligne à ligne.

**Relu avec ce savoir, le carnet s'éclaire** : les gels de l'Arc du 29/09 (fiche 43 § 8, 274 à
473 ms dans `soumettre` et `present`) et la mesure fondatrice de l'arbitre (Intel `present`
pire 487 ms, RTX 2,35 ms) ont très probablement la même cause — l'écran sur la RTX. L'arbitre
compensait une topologie qu'on ne lisait pas.

**La réponse** (`plateforme/ecran.rs`) : sous Windows, la carte de départ est **celle dont une
sortie porte le moniteur de la fenêtre** (DXGI, `MonitorFromWindow`), rapportée à la
préférence que Windows range en tête (`EnumAdapterByGpuPreference`, ce que `wgpu` emploie).
L'ordre devient : `GLUCOSE_CARTE` › **l'écran** › le souvenir › l'économe. Quand l'écran a
tranché, **l'arbitre ne s'installe pas**.

**Pourquoi ce n'est pas une étiquette**, et pourquoi la charte le permet : on ne demande rien
des capacités de la carte, on lit où vont les pixels. La carte de l'écran est sur le chemin de
toute image — c'est elle qui compose ([Microsoft, *Cross Adapter Scan-Out*](https://devblogs.microsoft.com/directx/optimizing-hybrid-laptop-performance-with-cross-adapter-scan-out-caso/) :
dessiner pour un écran branché sur une autre carte fait recopier chaque image par le
compositeur). La quitter n'évite jamais sa charge — un Blender qui la sature ralentit la
composition quoi qu'on fasse — et ajoute une copie à travers une carte qui s'endort. Se nicher
là où il reste de la place reste juste ; s'en aller de l'écran n'en est pas un moyen.

**Sur sa machine**, la lecture rend `10de:2d58` (la RTX) → rapide. La chronique et la bannière
disent désormais la carte qui dessine, et si c'est celle de l'écran — ce qui manquait ce matin.

**Ce qui reste ouvert** :

* la topologie se lit **au lancement** : une fenêtre glissée sur un écran branché sur l'autre
  carte garde la sienne jusqu'au lancement suivant ;
* hors de Windows, rien ne se lit : l'arbitre garde la main ;
* **la boîte noire ne note ni la carte ni la succession** : un trou à fermer (elle voyage, la
  chronique non) ;
* l'Intel qui dessine pour son propre écran (le mode Optimus) n'a jamais été mesuré ici ;
* `effacer` monte à 427 ms sur une image de glisser : dans la même session sur l'Intel,
  probablement la même cause, à revoir sur la RTX.

## 2. Les poignées au dézoom (POIGNEE-1)

**Son retour**, capture à l'appui : « tu vois les points là, lorsqu'on dézoome on ne voit plus
que ça, or ça devrait être cohérent avec la taille du zoom ». Huit carrés blancs de 9 px
entouraient des photos de 30 px. Glucose Tauri avait le même défaut, en quatre carrés.

**La recherche** : Excalidraw garde des poignées de taille fixe et retire celles des côtés
sous cinq fois leur taille (`transformHandles.ts`) — les coins restent ; tldraw expose
`hideResizeHandles` par forme. Aucun ne lie le dessin à la prise.

**La loi** (`hit_priority::handle_reach_px`, `handle_side_px`) : la **prise** d'une poignée
vaut 24 px sans dépasser 35 % du petit côté du nœud à l'écran (la règle de Tauri, déjà là) ;
le **carré** en est le cœur visible, dans la proportion 9 : 24. Ils rétrécissent ensemble ;
quand le carré n'a plus 3 px (un pixel blanc entre deux de liseré), la poignée disparaît **du
dessin et du clic**. La constante `HANDLE_SLOP_MIN_PX` (6 px) disparaît : avec elle, une
vignette sélectionnée était couverte de prises, et on la redimensionnait en voulant la
déplacer. La loi se lit en pixels logiques (la densité ne change pas la décision).

**Regardé** : `apercu_poignees` rend 300 photos sélectionnées à quatre zooms. Avant, au
dixième : le mur blanc de sa capture ; après : les cadres seuls, et des poignées à la taille
des photos à mi-zoom. Quatre sabotages, quatre chutes.

## 3. Le glisser vers une autre fenêtre : la fenêtre d'origine se tait

En relisant le chemin jamais vu à l'écran (fiche 51 § 2), dans les sources de `winit` 0.30.13 :
`DoDragDrop` tient sa boucle **dans** notre gestionnaire. `RedrawRequested` contourne le tampon
de réentrance de `winit`, mais `WM_PAINT` est gardé (pas de plantage) — et **redemandé aussitôt**
(`RDW_INTERNALPAINT` après `DefWindowProcW`). La boucle de `DoDragDrop` tire par `GetMessage` :
elle le reçoit dès que sa file est vide, et un cœur tournait à vide pendant tout le geste. La
fenêtre restait aussi figée sur les nœuds tenus au bord. Le crate `drag` de CrabNebula appelle
`DoDragDrop` de la même façon : la forme est la bonne, il fallait soigner l'avant.

L'image où les nœuds sont revenus **se peint avant le départ**, puis `RDW_NOINTERNALPAINT`
retire la demande en attente. L'épreuve ouvre une fenêtre à −32 000 pixels, jamais montrée.
**Sa première version était aveugle** : `PeekMessage`, même sans retirer, consomme la peinture
interne ; elle lit désormais `GetQueueStatus(QS_PAINT)`. Le saboteur l'a dit.

Le relâchement du bouton gauche n'arrive pas à `winit` pendant `DoDragDrop` (OLE tient la
souris) : sans effet ici, le glisser des nœuds est abandonné avant le départ.

## 4. Le pincement, un tiers plus rapide

« Le pincement sur le pavé tactile est trop lent sur le zoom, il faudrait juste augmenter
légèrement » : `OCTAVES_PAR_UNITE_DE_DOIGT` passe de 1/16 à **1/12** (douze unités pour
doubler), loin des 0,25 jugés « trop trop vite ». La molette n'est pas touchée. Le journal du
réglage le note.

## 5. La chronique du 06/10, relue

« 713 pincements, 723 déplacements, **0 pris pour un cran de souris** » : cette session-là s'est
faite au pavé ; elle ne dit rien de sa molette. Celle du 07/10 : 3 056 pincements, 7 072
déplacements, **4 pris pour un cran** — trop peu pour juger la roue libre.

## 6. Prouvé, et pas prouvé

* **Prouvé** : la CI de `20deb6f` ; 2 017 épreuves vertes, clippy strict à zéro ; onze
  sabotages, onze chutes (une épreuve aveugle trouvée et corrigée) ; la lecture de la carte de
  l'écran sur sa machine.
* **Pas prouvé** : que la session suivante soit fluide (son écran le dira) ; les commits de
  cette fiche n'ont pas vu la CI (rien n'est envoyé sans son accord), et `plateforme/ecran.rs`
  a une voie hors Windows que seule la CI jugera.

## 7. Ce que seul son écran dira

La liste de la fiche 51 § 9 tient toujours ; s'y ajoutent :

1. **Glucose fluide** dès le lancement, une session entière : la chronique doit dire « carte
   graphique : NVIDIA … — celle qui tient l'écran » et `mailbox`.
2. **Les poignées** au dézoom, sur un tableau chargé : ne plus voir que les cadres de loin, des
   poignées à la taille des photos de près ; et une vignette sélectionnée qu'on **déplace**.
3. **Le pincement** au pavé : légèrement plus rapide, pas trop.
4. **Glisser des nœuds vers une autre fenêtre** : les nœuds reviennent à leur place **dès** la
   sortie de la fenêtre, pas au lâcher.

## 8. Son écran, le 07/10 à 11 h 24 — et ce qui en a suivi

La chronique de sa session (859 s) : **la RTX, « celle qui tient l'écran », en `mailbox`** ; la
pire image coûte 48 ms (505 le matin), et aucun gel de `present`. **ECRAN-1 est vérifié.** Les
gels qui restent (jusqu'à 321 ms) sont dans « entretien » et « écouter la main » : des gestes
lourds — ouvrir une fenêtre, copier, changer de mode —, pas l'affichage. **Le repos** est
tranché : au repos, seuls dessinent les messages qui s'effacent (519 images) et les glissades
qui finissent (336) ; aucune raison spontanée.

Ses verdicts, et ce qui a été fait :

1. **Couper-coller une sélection mêlée** : « absolument parfait ». **Deux fenêtres, copier
   puis glisser** : « parfait aussi ». **Copier l'image dans Discord, l'enregistrer** :
   « parfait ».
2. **Il n'a pas de souris.** Toute la navigation se juge au pavé ; la voie de la souris attend
   d'autres mains.
3. **Les poignées** : parties, mais **une grille blanche restait** — le cadre de sélection
   (3 px de débord, 1,25 px de trait, fixes). Le cadre des photos, l'anneau des textes et celui
   des post-its suivent désormais la même loi (`chrome_ratio`, la prise effective rapportée à
   la prise pleine) ; plus fin qu'un pixel, le cadre droit reçoit l'encre de sa couverture —
   ce que l'anti-crénelage aurait donné. De loin, une trace grise ; de près, rien ne change.
4. **Le pincement** : « trop d'effet de smooth, et pas du tout rapide, ce qui crée un vrai
   sentiment de lag ». Il passait par la conduite (10 ms) et la glissade (0,28 s) du doigt.
   **Tout zoom est désormais direct** — montré en entier à l'image suivante, rien après — ;
   la porte `pousser_zoom` et `TAU_LIBRE_ZOOM` disparaissent. Gain : 1/5 d'octave par unité,
   montré tout de suite (le 1/12 lissé valait un sixième, servi en retard ; le 0,25 lissé, une
   demi-octave, « trop trop vite »). Le déplacement à deux doigts garde son élan. Un piège
   évité : un zoom arrivé entre deux poussées du doigt dormait dans la dette, et l'élan ne
   finissait jamais — il se montre en entier quelle que soit la porte (épreuve, sabotée).
5. **`Ctrl+N`** : « il crée instantanément une nouvelle session sans même demander
   d'enregistrer ». Le document quitté était **nommé** (`fuser.glucose`, son dernier document) :
   chaque geste y était déjà écrit, et ses brouillons ne montrent aucune perte. Demander
   « enregistrer ? » mentirait ; se taire l'a inquiété. Le message dit maintenant ce qui vient
   d'avoir lieu, à la place de « Supprimé » (une suppression se voit, comme un collage). Le cas
   « travail sans nom » pose bien la question : une épreuve le joue de bout en bout.
6. **Le mode référence** :
   * **les couleurs** : les journaux de l'application NVIDIA (lecture seule) montrent qu'à
     11 h 34 min 50 s elle a appliqué à `glucose-desktop.exe` **RTX HDR** (`AIHDR`, crête 613
     nits) et **RTX Dynamic Vibrance** (`AIDVC`, intensité et saturation 50) — des filtres de
     jeu. Son écran est en HDR. Hypothèse, **probable, à confirmer** : agrandie et sans cadre,
     la fenêtre ressemblait à un jeu en plein écran fenêtré. Entrer dans le mode la
     désagrandit désormais : une référence flotte, elle ne couvre pas l'écran ;
   * **les signets `1`, `2`, `3`, `Ctrl+1`, `Ctrl+2` ne répondaient pas** : en épreuve, en mode
     référence, `Ctrl+1` pose et `1` vole. **Non reproduit** — la touche se perd avant Glucose,
     ou dans un cas non joué (une carte en cours d'édition prend les chiffres) ;
   * **déplacer et redimensionner sans souris** (REFERENCE-2) : `Alt` + glisser déplace la
     fenêtre par le geste natif du système ; `Alt` + pincer l'agrandit autour de son centre,
     bornée entre 160 px logiques et l'écran. `Alt` au clic fouille ailleurs une pile de
     nœuds : en mode référence, la fenêtre passe d'abord. Le bouton droit et les bordures
     restent pour la souris.

Prouvé : 2 026 épreuves, clippy strict ; dix nouveaux sabotages, dix chutes, **une épreuve
aveugle de plus trouvée** (celle du message de `Ctrl+N` lisait le message de l'enregistrement
d'avant). Pas prouvé : tout ce qui précède à son écran.

## 9. Son écran, une deuxième fois

* **Le mode référence et les signets** : « absolument et vraiment sublime ». Les signets
  répondent ; le défaut de la première fois n'est pas revenu.
* **Le pincement** : direct, il allait « strate par strate, comme une molette » ; un zoom au
  pavé est « smooth et instantané ». Il reprend la **conduite** du déplacement (10 ms, qui fond
  les paquets du pavé) sans **glissade** au lâcher, et 10 % de gain de plus (0,22). Deux
  épreuves aveugles trouvées par le saboteur, corrigées : la route du doigt jouée dans
  l'application, et la glissade remise au lâcher qui fait tomber l'épreuve de l'élan.
* **`Ctrl+N`** (NOUVEAU-1) : le message après coup ne suffisait pas. *« Que quoi qu'il se
  passe, lorsqu'on fait Ctrl+N, on voie une popup : voulez-vous créer un nouveau document ? »*
  — Oui / Non, le document nommé dit enregistré ; un travail sans nom garde sa question à trois
  réponses, qui vaut confirmation. DIAL-3 : sous `cfg(test)`, une épreuve fournit la réponse,
  et tombe sans elle — aucune épreuve ne peut plus ouvrir une boîte sur son écran.
* **La sélection** : « pas esthétique du tout ». Le style (`style.md`) est hérité de Glucose
  Tauri. Une planche hors écran lui montre quatre directions monochromes — l'actuelle, des
  équerres aux coins, un fil et quatre points, un cadre unique pour le groupe — : **son choix**.

## 10. Son écran, une troisième fois

* **`Ctrl+N`** : « fonctionne parfaitement ». **Le pincement** : « une nette amélioration, mais
  ce n'est pas encore ça » — sans plus de détail ; la réponse de fond reste *Direct
  Manipulation* (SUITE, chantier 5), qui livre les gestes du pavé avec leur vraie échelle.
* **La molette, à la souris pour la première fois** : « pas du tout assez rapide ». Un tiers
  d'octave par cran au lieu d'un huitième : trois crans doublent.
* **Les panneaux en mode référence** : Ordonner, la Time Machine et le Pomodoro restaient
  posés **par la carte graphique** — la voie du processeur les retirait, celle de la carte non
  —, visibles et morts puisque leurs clics étaient coupés. Ils partent, et reviennent tels
  quels quand le mode se défait.
* **Les signets dans une petite fenêtre** (SIGNET-2) : « on tombe dans le vide ». Un signet
  retenait le décalage du coin haut-gauche ; il retient désormais le décalage au centre du
  canevas, et se rappelle autour du centre d'aujourd'hui. Le format ne change pas — la 2.0.1
  installée relit les mêmes fichiers — ; les signets posés avant se rappellent décalés d'une
  demi-fenêtre, une fois.
* **« Un énorme freeze, qui a failli faire planter mon PC »**. Windows n'a noté ni erreur de
  pilote ni plantage ; la pire image de Glucose a coûté 36 ms. Mais sa boîte noire finit sur
  des dizaines de pincements à 5 ms d'écart, puis 29 images à 12 ms dans `present`, puis rien
  pendant 42 s : la signature d'une fenêtre redimensionnée en rafale. `Alt` + pincer la
  redimensionnait **à chaque événement** — chaque taille reconstruit la surface de la carte et
  le tampon de l'écran. REFERENCE-3 : les pincements s'additionnent, et la fenêtre change de
  taille au plus une fois par image, quand Windows a appliqué la précédente (ou que la demande
  est trop vieille pour être encore en route). **Cause probable, à confirmer avec lui.**
* **La sélection** : la planche des quatre directions attend son choix.

