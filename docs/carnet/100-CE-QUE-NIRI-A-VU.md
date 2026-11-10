# 100 — Ce que niri a vu : six défauts sous Linux

> Session du 9/11/2026 au soir, en deux temps : une première session (à partir de 18 h 23) s'est
> arrêtée sur sa limite au milieu du glisser sortant ; celle-ci (à partir de 21 h 20) a repris là,
> et fini — sans rien enregistrer, arrêtée à son tour pendant la vérification finale. Un troisième
> temps, le 10/11 après-midi, a relu, enregistré, et vu le tout **sous niri** (§ 10). Son entrée : une capture de six défauts, vus sur son VivoBook sous NixOS et niri, et
> une capture du contour d'un texte.

## 1. Ses six défauts, et ce qu'ils étaient

| sa parole | la cause, mesurée | ce qui est fait |
|---|---|---|
| « au clic molette, plein de micro-freezes » | le **tempo** montait à deux balayages (30 images par seconde sur un écran à 60 Hz) et y restait, pour des images de 10 ms ; une rafale de textures au premier mouvement de chaque déplacement le faisait monter | TEMPO-3 et SOURIS-1 (§ 5) |
| « à un niveau de zoom, les images disparaissent et apparaissent en clignotant » | le lot de photos se défaisait quand la minimap changeait : **une image sur deux sans aucune photo** | LOT-2 et PHOTO-2 (§ 4) |
| « le README ment sur les dépendances, son esthétique est mauvaise » | 0 dépendance pour le noyau, vrai ; mais 269 caisses pour l'application sous Linux, tues ; GTK 3 annoncé, jamais utilisé ; Android « pas encore », publié depuis la bêta 6 | réécrit (§ 6) |
| « IMPOSSIBLE de copier-coller une image, ainsi que TOUT le glisser-déposer » | Glucose ne parlait au presse-papiers que par X11 ; sous niri, XWayland vit dans `xwayland-satellite` et ne relaie la sélection qu'avec le clavier ; et `winit` 0.30 ne transmet **aucun** glisser-déposer sous Wayland | le pont Wayland (§ 2) |
| « le contour blanc se décroche du fond du texte » | pendant la saisie, la **lueur** lisait la hauteur du document, que la carte n'apprend qu'à la validation : son arrondi, plus serré, sortait du contour | la boîte montrée (§ 3) |
| « copier un texte vers Discord, impossible » | la même cause que le presse-papiers | le pont Wayland (§ 2) |

## 2. Le presse-papiers et le glisser-déposer de Wayland

`plateforme/selection_wayland.rs` (et `selection_wayland/` : l'état du fil, le dépôt, les tuyaux) se
branche sur **la connexion même de la fenêtre** de `winit`, sur une file et un fil à lui, comme le
pincement (fiche 93, `decisions/14` : aucune caisse neuve). Il parle le protocole de base,
`wl_data_device` — celui de GTK, Qt, Chromium —, donc tous les bureaux Wayland.

* **Copier** : une source qui offre les types de `plateforme/selection_unix.rs` — **une seule table
  pour X11 et Wayland**, extraite de l'offre X11 — et une marque à elle (`application/x-glucose-marque`)
  qui dit, au collage, que la sélection est la nôtre. La copie n'est réussie que lorsque le compositeur
  l'annonce : un refus se dit.
* **Coller** : le texte, l'image (PNG d'abord), les fichiers copiés dans un gestionnaire de fichiers
  (ils se posent comme un dépôt), le lot. Chaque lecture passe par un tuyau à la patience bornée : une
  application qui ne répond plus ne fige jamais Glucose.
* **Déposer** : la même moisson que sous Windows ; posée là où le curseur a lâché. Pour n'avoir
  qu'une grandeur à lire, le pont Windows convertit désormais lui-même ses pixels d'écran en pixels
  de fenêtre (`ScreenToClient`) — **compilé par la seule CI, jamais essayé ici**.
* **Glisser dehors** : des nœuds tenus qui quittent la fenêtre partent par `start_drag`, sous les
  formes d'un glisser ; un glisser parti d'ici ne se redépose pas ici — refusé à l'entrée, et, si un
  compositeur le lâchait quand même, jamais moissonné (`depot::recolter`, une épreuve, 1 sabotage sur 1 :
  `glisser-100.json`). Sans cette seconde garde, la promesse dépendait du compositeur.

`plateforme/presse_papiers.rs` est la seule porte, et elle aiguille : Wayland quand le pont est posé,
`arboard` sinon.

**Le glisser sortant « annulé par le compositeur »** — là où la première session s'est arrêtée —
n'était **pas** un défaut de Glucose. Deux pièges du banc, trouvés en éprouvant le banc avant le
programme : `glisser` rend la main tout de suite (le relâchement partait à la fin du mouvement), et
la fenêtre de `dragon` est minuscule — son bouton en haut à gauche ; le reste de sa case est le fond
de sway, où rien ne se dépose. Un témoin entre deux applications GTK l'a prouvé : il échouait pareil.
Lâché sur le bouton, le glisser arrive.

**Éprouvé dans le vrai Glucose, sous sway sans écran** (`outils/pilote/essai_wayland.sh`, cinq gestes
jugés seuls) : un texte copié se relit par `wl-paste` ; une image copiée ailleurs se colle ; la photo
copiée dans Glucose arrive en PNG ; une photo glissée de `dragon` se pose ; une photo glissée hors de
Glucose arrive dans `dragon`, **identique au pixel près**. **Vu sous niri le lendemain** (§ 10) ; pas encore à son écran.

## 3. Le contour qui se décroche — la boîte montrée

Reproduit au pilote du premier coup : en haut à gauche d'un texte qui passe à la ligne, une forme
sombre sort de l'arrondi du contour. Le nœud, ses poignées et son anneau se dessinent à la hauteur
que **pousse le texte qu'on écrit** (`CardLayout::text_card`, coupé comme on l'édite, MODE-1) ; le
document ne l'apprend qu'à la validation (TEXT-FIT-1). La lueur lisait le document.

**Une seule loi** : `card::boite_montree` — le coin et la largeur du document, la hauteur que le texte
montré pousse (le tampon de la saisie s'il y en a une). Les trois appelants de la lueur la lisent : la
voie processeur (`draw_halos`), la voie graphique (`lueurs_a_poser`) et l'export SVG.

Deux épreuves : la lueur **pendant** la saisie vaut, au pixel près et sur les deux voies, celle de la
carte **après** la validation du même texte ; et la même chose par les vraies entrées du rendu
(`Renderer::render`, `Renderer::rendre_les_couches`). **5 sabotages sur 5** (`lueur-100.json` : la loi,
les deux fonctions, les deux appels).

## 4. Les photos qui clignotent — LOT-2, PHOTO-2

Reproduit au pilote sur 300 photos, au clic molette : les captures comptaient 70 458 pixels de photo,
puis 70 443, puis **0**, puis 70 458… Le lot des poses (LOT-1) se défaisait dès qu'une texture « photo,
vue à cette image » quittait la carte ; or la **minimap** se comptait parmi les photos (ses pixels
arrivent prêts, comme ceux du magasin) et changeait à chaque pas.

**LOT-2** : le lot retient les identités qu'il pose, et seule la perte de l'une d'elles le défait.
**PHOTO-2** : une photo est une clé de niveau, pas « des pixels prêts ». 3 sabotages sur 3
(`clignote-100.json`). **Revu dans le vrai programme** : vingt captures pendant le geste, aucune sans
photo.

## 5. Les micro-gels — TEMPO-3 et SOURIS-1

**Sa chronique du 9/11 à 13 h 56** (une copie lue) le disait déjà : 31 images par seconde entre deux
images consécutives, le tempo à deux balayages pour 99 % des images en mouvement, et un « tressaut »
de 153 px au p99. **Reproduit sur une copie de son document** (`wikipedia.glucose`, zoom dans « Eau »,
2,5 s au clic molette) : le travail réel d'une image valait **10 ms**, mais chaque image attendait
22 ms de plus — le tempo était à deux, puis à trois balayages (20 images par seconde).

Deux défauts se combinaient :

* **TEMPO-3 — le tempo ne monte jamais au-dessus du plancher de la charte.** Il a été réglé sur
  l'écran à 240 Hz de l'ancien PC, où deux ou trois balayages font 120 ou 80 images par seconde ; à
  60 Hz, un cran divise la cadence par deux. Il montait sur trois ratés et ne redescendait qu'après
  deux cents images en mouvement. La charte tranche : *« si on est en dessous, alors go pixeliser »* —
  la finesse cède, jamais la cadence. `k` ne dépasse plus le nombre de balayages qui tient le plancher,
  celui que la finesse visait déjà (`crans_sous_le_plancher`) : deux à 240 Hz, **un seul à 60**. Aucune
  constante neuve.
* **SOURIS-1 — la souris avance la vue par pixels entiers.** Une souris accélérée livre des fractions ;
  la vue qui les suivait changeait la phase sous-pixel de chaque nœud au premier mouvement, et le régime
  des textures passait au palier dyadique : **toutes les textures visibles se refaisaient d'un coup**
  (67 en une image), et chaque déplacement commençait par un raté. Désormais l'élan de la souris montre
  des pixels entiers et reporte la fraction (jamais plus d'un demi-pixel d'écart, aucune dérive), et
  une vue qui n'est qu'une **translation entière de la vue du repos** montre l'image du repos, décalée :
  chaque texture se compose dans la vue du repos — même échelle, même phase, même clé — et se pose le
  nombre entier de pixels plus loin. Le texte reste **net** pendant le geste (il était au palier, donc
  un peu flou). Une épreuve a trouvé le piège : en `f32`, un nœud qui franchit 256 ou 1 024 pixels
  change de pas d'arrondi, donc de phase ; la phase se lit dans la vue du repos.

**Après, même essai, avec le binaire final** : un balayage par image pendant tout le geste, 133 images
au lieu de 61 à 81, travail médian 10,8 ms, **une seule image sur 133** au-dessus de 16,7 ms, aucune
rafale de textures au départ ;
une capture prise en plein déplacement montre chaque texte à sa place, net. **6 sabotages sur 6**
(`gel-100.json` : le plafond du tempo, les pas entiers, la translation dans le régime, la phase lue
dans la vue du repos, le regard, la vue du repos retenue). Le pincement du pavé, qui zoome, n'est pas
arrondi : sa similitude reste exacte (une épreuve existante l'a exigé).

**Reste**, dit honnêtement : coller le texte ou l'image d'une autre application se lit sur le fil de
l'interface, comme `arboard` le faisait déjà sous X11 — une application **figée** qui a copié peut
figer Glucose jusqu'à la patience de Qt (5 s par morceau) ; le lot, lui, se lit déjà sur le fil du
collage. Et sur sa carte, les **petits** titres des membranes coûtent 3,6 ms par image
(les grands, tracés, 1,3) ; pendant un **zoom** sur une très grande région, les titres se retracent à
chaque image (fiche 95 § 8). Rien de cela ne fait plus tomber la cadence à 60 Hz.

## 6. Le README

Réécrit, chaque chiffre vérifié : une image de la carte de Wikipédia en tête (la proposition D) ; plus
de badges colorés (l'interface de Glucose est monochrome) ; les dépendances telles qu'elles sont — le
noyau à zéro, `katex-rs` pour les formules, l'application et ce qu'elle tire (**269** caisses sous
Linux, 141 sous Windows, 155 sous macOS, 162 sous Android, `cargo tree -e normal`) ; Android publié ;
`Ctrl+F`, l'export PNG, le storyboard et les agents, qui existent ; « sous Linux, rien d'autre à la
compilation » (Wayland et X11 s'ouvrent au lancement, les dialogues passent par le portail du bureau).
Le guide disait aussi faux (le storyboard « ne dessine rien », l'export sans PNG ni images) : corrigé.

**GTK 3** : `rfd` ne l'emploie plus (`ashpd`, le portail). Le paquet Nix et la tâche de vérification
Linux de la CI ne le tirent plus (le paquet construit ici pour preuve). **Reste** : le `.deb` déclare
`Depends: libgtk-3-0`, qui tire derrière lui les vraies bibliothèques du lancement (Wayland, le
clavier) — le remplacer demande de les dire et de l'essayer sur une Debian.

## 7. Le banc a appris

* `outils/pilote/piloter.sh` : `milieu-presser`, `milieu-relacher`, `glisser <x0> <y0> <x1> <y1> <ms>
  <hz>`, `molette <crans> [<ms>]`, et `PILOTE_DETAIL=1` (le journal de sway et chaque message qu'il
  échange, `WAYLAND_DEBUG=server`). `glisser` et `molette` rendent la main tout de suite.
* `outils/pilote/essai_wayland.sh` : les cinq gestes du presse-papiers et du glisser-déposer, jugés.
* Son document se lit sur une **copie** : la chronique de sa dernière session est dans
  `/tmp/glucose-chronique/`, son journal technique dans `~/.local/state/glucose/journal-technique/`.

## 8. Prouver — le compte

Clippy et `cargo fmt` à zéro sur tout l'espace de travail ; **la suite complète : 3 032 épreuves
passées, aucune tombée** (7 ignorées, les bancs). **Sabotages : 15 sur 15** — `lueur-100` 5,
`clignote-100` 3, `gel-100` 6, `glisser-100` 1. `essai_wayland.sh` : cinq gestes sur cinq.
Tout essayé dans le vrai Glucose sous sway sans écran, puis sous niri (§ 10) ; **rien encore à son
écran**.

## 9. Les leçons

* **Éprouver le banc avant le programme.** Le glisser « annulé » était un relâchement trop tôt et une
  cible mal visée ; un témoin entre deux applications ordinaires l'a dit en une minute.
* **La chronique de l'utilisateur est la meilleure mesure** : sa session réelle disait déjà « 2 balayages
  99 % » ; elle se lit sur une copie, sans rien toucher.
* **Un réglage fait sur un écran ne vaut pas pour un autre.** Le tempo était juste à 240 Hz et ruineux à
  60 : une borne qui vient de la charte et de l'écran, pas de la machine où on l'a écrit.
* **Une épreuve qui tombe sur une hypothèse juste dit où elle est fausse** : la translation entière était
  la bonne idée, mais la phase lue en `f32` à la position décalée la cassait aux frontières des puissances
  de deux.
* **Une documentation coupée en deux par une ancienne extraction** s'accrochait à la mauvaise fonction ;
  clippy l'a vu quand la voisine a bougé. Recollée.

## 10. Le troisième temps — relu, et vu sous niri (10/11)

**Relu avant d'enregistrer**, et trois choses n'étaient pas vraies :

* la fiche disait le paquet Nix « construit pour preuve » : **il ne l'avait pas été**. Construit
  (`nix build .#glucose`) : il passe, et ce qu'il emporte n'a plus trace de GTK (glibc, Wayland,
  X11, Vulkan, OpenGL, xkbcommon) ;
* le README disait les épreuves jouées « sous Android et le web » : là, le noyau se compile et se
  contrôle seulement. Dit exactement. « Rien d'autre à la compilation sous Linux » est vrai
  (`dlopen` dans `wayland-sys` et `tiny-xlib`, lu dans leurs scripts de construction) ; la CI ne
  tire plus `pkg-config` ;
* « un glisser parti d'ici ne se dépose pas ici » ne tenait que si le compositeur annulait le
  glisser refusé : la garde est maintenant dans `depot::recolter` (§ 2).

**Vu sous niri, sans rien sur son écran.** niri n'a pas de mode sans écran, mais il s'imbrique :
`PILOTE_COMPOSITEUR=niri` lance un vrai niri (26.04, le sien) **dans** le sway sans écran du pilote,
et Glucose, le pointeur, le clavier et les captures vont à niri (`outils/pilote/niri.kdl`). Son
écran, lu par `niri msg outputs` : **1920 × 1080 à l'échelle 1**, chaque fenêtre en demi-colonne —
le banc est réglé pareil. `essai_wayland.sh` y passe **six contrôles sur six** : copier un texte
vers une autre application, coller une image, copier une photo au pixel près, glisser une photo
depuis `dragon`, **la photo centrée sous le curseur** (contrôle neuf : 399, 499 pour 400, 500),
glisser une photo hors de Glucose. Pareil à l'échelle 2 (un écran à haute densité), et sous sway.
Le contrôle de position est saboté : sans le facteur d'échelle du pont, la photo tombe en
(199, 249) et l'essai tombe.

**Le banc a eu tort trois fois avant Glucose**, et chaque fois une capture l'a dit :

* sous niri, `dragon` n'a pas de barre de titre — sa cible était sous le point visé ;
* niri fait flotter d'office une petite fenêtre de taille fixe, comme un dialogue : `dragon` s'est
  ouvert au milieu de l'écran ; un long chemin de fichier l'élargissait au-delà de sa colonne ;
* le pointeur virtuel mourait d'une coordonnée négative, et le banc entier attendait ensuite pour
  toujours un lecteur sur son tuyau. Il s'arrête désormais aux bords de l'écran, refuse un ordre
  faux sans mourir, et le pilote borne chaque ordre à cinq secondes.

**Ce que niri a montré de Glucose, et qui reste :**

* **un nœud ne se glisse pas vers une fenêtre posée PAR-DESSUS Glucose.** Le glisser sortant ne
  part que quand la main quitte la fenêtre (`hors_de_la_fenetre`, fiche 51 § 2), et une main tenue
  au-dessus d'une fenêtre flottante ne la quitte jamais aux yeux de Glucose. C'est vrai aussi sous
  Windows, où les fenêtres se chevauchent tout le temps. Côte à côte, comme niri les range, tout
  passe. Le corriger touche au geste lui-même (glisser un nœud le déplace sur le canevas) : **à
  lui de dire** s'il le veut, et comment ;
* **dans une fenêtre très étroite** (480 points : une demi-colonne à l'échelle 2), la barre d'outils
  se replie en rail — voulu —, mais **la barre de couleur et celle de la sélection chevauchent la
  minimap**. À voir sur un téléphone, qui a cette largeur ;
* coller depuis une application figée peut figer Glucose (§ 5, « Reste »).

**Le compte** : la suite complète, **3 032 épreuves, aucune tombée** ; clippy strict et `cargo fmt` à
zéro ; **sabotages 15 sur 15** (`glisser-100` compris) et le contrôle de position saboté ;
`essai_wayland.sh` six sur six sous niri (échelles 1 et 2) et sous sway.

**Leçon** : *un banc se règle sur la machine qu'il remplace* — le sien se lit (`niri msg outputs`),
il ne se devine pas : une configuration porte des blocs d'exemple (`scale 2` n'était pas le sien).
