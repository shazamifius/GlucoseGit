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
