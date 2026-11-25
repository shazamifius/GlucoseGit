# 155 — L'ombre des nœuds sous la main (25/11 vers 2 h 10, le relais, compte a)

## 1. D'où il vient

Q10 (fiche 149 § 1) : `Ctrl` + glisser un nœud l'emporte vers une autre fenêtre. **Restait** : pendant
le geste, seul le curseur disait le glisser — rien ne montrait ce qu'on emportait. Sous Wayland, le
pont du presse-papiers lançait `start_drag(source, origine, None, bouton)` : une icône vide.

## 2. Ce qui est fait (OMBRE-1)

**Les nœuds tenus se dessinent sous la main, translucides.** Rien n'est inventé pour leur forme :
c'est **l'export en image de la sélection** (EXPORT-PNG-1, fiche 62 : `copie_a_exporter`, la marge
exacte de `debord`), rendu par un moteur d'export dont le fond du canevas est **transparent** — on ne
voit que les nœuds et leur lueur.

* `interactions/clipboard/ombre.rs` : `prise_des_noeuds` lit où la main tient les nœuds **avant**
  qu'ils ne reviennent à leur place ; `ombre_des_noeuds` rend l'ombre à la taille où on les voit, en
  pixels entiers à la densité de l'écran (`ceil` de son échelle : 2 sur son écran 4K à 150 %), et le
  point de la main en pixels de l'ombre.
* **Deux nombres, empruntés** : le plus grand côté, **400 points** — celui de Safari (WebCore,
  `DragController::maxDragImageSize` : 400 × 400 sous macOS, 200 ailleurs ; Chromium plafonne un
  lien glissé à 150 et un fichier à 200) ; l'opacité, **0,8** — celle du fantôme d'une fenêtre qu'on
  glisse sous ChromeOS (`kDragPhantomMaxOpacity`). Au-delà du plafond, l'ombre se **réduit** (jamais
  rognée) : la marge croissant comme l'échelle (la lueur s'étale et se floute d'autant), le contenu et
  ses deux marges y tiennent ensemble, `plafond / (côté + 2 × debord(1))`.
* `plateforme/selection_wayland/icone.rs` : la surface de l'icône, ses pixels dans un fichier en
  mémoire (`memfd_create`, `wl_shm`, `ARGB8888` prémultiplié, petit-boutiste), donnée à
  `start_drag`, puis montrée — et **retirée** à `dnd_finished` comme à `cancelled`, une seule fois.
  Le pont lie désormais `wl_compositor` et `wl_shm` ; sans eux, le glisser part sans ombre.
* **La prudence de la connexion** (une faute de protocole fermerait Glucose) : `attach` avec un
  décalage est une faute dès la version 5 de la surface — `offset` prend sa place ; `set_buffer_scale`
  attend la version 3, `damage_buffer` la 4 ; la taille du tampon est un multiple de son échelle
  (`invalid_size`). Chaque garde est écrite à côté de sa requête.
* `plateforme::sait_montrer_une_ombre` : l'ombre ne se dessine que là où elle sert.

## 3. Ce qui reste

* **Windows** : `IDragSourceHelper::InitializeFromBitmap` (un `HBITMAP` 32 bits prémultiplié, le point
  de la main en `ptOffset`). **Pas écrit** : rien ne compile pour Windows sur ce poste (la cible n'est
  pas dans `nix develop`), et du COM jamais compilé avant la CI serait un pari. **À faire d'abord** :
  ajouter la bibliothèque standard de `x86_64-pc-windows-gnu` au `flake.nix`, pour un `cargo check
  --target` ici — utile bien au-delà de l'ombre (la voie WinHTTP, `glisser_windows.rs`).
* La version 4 de la surface (`attach` décalé) n'a pas été vue : sway et niri offrent la 6.
* Au téléphone, l'appui long ; sous macOS, `⌥` — inchangés (fiche 149 § 1).

## 4. La preuve

* Cinq épreuves (`ombre/tests.rs`, `icone.rs`) : la taille où on les voit et la main à sa place ; un
  grand nœud réduit (jamais rogné) sous 400 points, à la densité de l'écran ; une largeur brute
  impaire devenue paire à la densité 2 ; translucide sur un fond transparent ; les octets de Wayland.
* **8 sabotages sur 8** (`outils/sabotages/ombre-155.json`) — dont trois d'abord **aveugles** : le
  plafond ôté, le plafond qui oublie les marges, la parité oubliée passaient, parce que la taille,
  bornée en dernier, cachait un contenu rogné. Les épreuves demandent désormais que le bord du nœud
  reste dans l'ombre, et qu'une largeur brute impaire existe.
* clippy propre, **3 694 épreuves vertes**.
* **Vu au pilote, sous sway et sous niri** (son compositeur ; la capture `grim -c`, car niri pose le
  curseur et l'icône à part) : `Ctrl` tenu, le texte d'accueil glissé — son ombre réduite suit la
  main, le point tenu sous elle ; au lâcher, elle disparaît, le nœud est à sa place, rien n'est
  doublé ; un second glisser de même ; Glucose vivant.

Sources : WebCore `DragController` (https://opensource.apple.com/source/WebCore/WebCore-855.7/page/mac/DragControllerMac.mm.auto.html,
et l'ancien `DragController` de Chromium) ; Chromium `button_drag_utils.cc`
(https://chromium.googlesource.com/chromium/src/+/HEAD/ui/views/button_drag_utils.cc) et
`drag_utils.cc` ; ChromeOS `drag_window_controller.cc`
(https://chromium.googlesource.com/chromium/src/+/e76b8604b39fc007f822142871df47364830ad34/ash/wm/drag_window_controller.cc) ;
Wayland, `wl_surface.offset` et `start_drag` (https://gitlab.freedesktop.org/wayland/wayland/-/merge_requests/344).
