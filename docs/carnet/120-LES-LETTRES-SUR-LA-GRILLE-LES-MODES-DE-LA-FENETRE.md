# 120 — Sa capture, la suite : les lettres sur la grille, les modes de la fenêtre, nommer un dossier

> Session du 18/11/2026, depuis 10 h 16. Il a renvoyé **la même capture que le 17/11** (le même
> fichier, au pixel près) avec *« voici une capture des choses qui ne fonctionnent pas dans la version
> actuelle de Glucose »*, et le compte rendu de la session d'avant. Sa fiche de questions n'avait pas
> bougé depuis la veille : aucune réponse aux questions 1 à 19. En cours de session, deux demandes
> neuves (§ 7) : nommer un dossier, détourer une image.

## 1. Ce qu'on a lu d'abord

* **La 2.0.4-beta.2 était toujours en brouillon** sur GitHub : la version qu'il a entre les mains (à
  l'école, au téléphone) est la beta.1, qui porte encore les douze défauts. Huit sont corrigés depuis
  la veille dans `main` et dans le brouillon — la capture n'est « actuelle » que pour la version
  publiée. Dit en tête de session : publier la beta.2 tout de suite, ce qui suit ira dans une beta.3.
* **Les quatre défauts qui attendaient sa parole** : trois sont des **options**, que sa règle
  « l'utilisateur choisit toujours » permet de construire avec un défaut argumenté (les modes de la
  fenêtre — sa propre proposition —, la barre de Windows, le texte) ; la quatrième, la structure des
  tiroirs, engage l'ergonomie : prise sur une image (§ 6).

## 2. GRILLE-1 et GRILLE-2 — le texte et les icônes « pixelisés »

**Reproduit** (non reproduit la veille) : au pixel près, la bande du vrai Glucose à 100 % — le haut et
le bas des minuscules à cheval sur deux rangées, chaque fût étalé sur deux colonnes, des icônes floues.
Le grain de sa capture (réduite deux fois : collée dans Glucose, puis recapturée) est le même.

**L'expérience, hors du dépôt** (`fontdue` seul, Inter 12 px, `#cccccc` sur `#1a1a1a`, et FreeType en
référence) — trois causes, mesurées une à une :

1. **Rien n'est aligné sur la grille** : la ligne de base au quart de pixel, la hauteur d'x d'Inter à
   0,545 du corps (6,55 px à 12). Aligner seulement verticalement — ce que FreeType appelle
   l'ajustement « léger » — rend le haut et le bas des minuscules nets.
2. **La position au quart de pixel par ré-échantillonnage** (GLYPH-1, l'interpolation bilinéaire d'une
   lettre déjà rastérisée) étale chaque bord d'un pixel : FreeType *sans aucun* ajustement était plus
   net que Glucose. Une lettre rastérisée **quatre fois plus grande**, puis réduite par blocs de 4 × 4
   décalés de la phase, donne la couverture **exacte** au quart de pixel — la couverture d'un pixel
   est la moyenne de ses seize sous-pixels.
3. Le mélange sans correction de luminosité (le texte clair maigrit sur fond sombre) : regardé
   (linéaire, contraste renforcé à 0,5 et 1), **pas retenu** — un nombre de ressenti, et ce n'était pas
   le « pâté ».

**Les lois** :

* **GRILLE-1** (`typography/ecran.rs`) : l'interface a sa typographie, **posée sur l'écran**
  (`Renderer::typo_ecran`, les mêmes polices partagées par un `Rc`) — la ligne de base sur un pixel
  entier ; la hauteur d'x sur un nombre entier de pixels par un **étirement vertical seul** du maître
  ×4 (la largeur et les avances ne changent pas : aucune mise en page ne bouge, une épreuve le tient) ;
  la position horizontale exacte. Le canevas garde sa typographie : ses lettres changent de taille à
  chaque image d'un zoom, se reposent dans des textures transformées, et une lettre seize fois plus
  chère y coûterait chaque image. Le cache de l'écran n'oublie rien d'une image à l'autre (un petit
  nombre fini de tailles).
* **GRILLE-2** (`icons/grille.rs`) : les icônes des panneaux étaient dessinées sur 16 unités et posées
  à 14 pixels (chaque coordonnée entière à un multiple de 0,875 px), en traits de 1,3 px. **Un trait
  fait un nombre entier de pixels**, et chaque point se pose pour que ses bords tombent sur des
  frontières (au centre d'un pixel pour un trait impair) — puis revient dans les unités du dessin :
  les pointillés gardent leurs longueurs.
* **Six portes** donnent sa typographie à l'interface : le rendu processeur, le rendu réduit, la
  chrome confiée à la carte, les menus et les barres, les panneaux (deux chemins).

**Regardé** : la bande à 100, 125 et 150 % avant/après ; les trois témoins de l'interface (7 115,
14 218 et 25 941 pixels changés, **aucun sur le canevas**, l'écart dessiné en rouge) ; le menu agrandi.
À 100 %, les icônes passent à des traits d'un pixel nets, plus légers ; à 125 et 150 %, à deux pixels.

## 3. FENETRE-1 et DEVANT-1 — les trois modes de la fenêtre

Ses mots : *« un mode fenêtre classique, qui est l'état actuel de Glucose sur Linux ; un mode qui force
le plein écran ; et un mode qui force le fait d'être au-dessus de toutes les applications et permet de
faire exactement comme Windows »*.

* **Une seule valeur** (`ui::fenetre::ModeDeFenetre`) : normale, plein écran, référence — exclusives,
  jamais deux drapeaux qui se contredisent ; le même geste défait son mode, un autre en change ;
  **retenue** (`mode-fenetre`, le souvenir du mode référence d'avant se relit puis disparaît).
* **`F11` est le plein écran** (la touche de tous les logiciels de bureau ; l'interface reste) ; le
  **zen passe à `Maj+F11`** et se combine au plein écran. Le zen était à `F11` depuis Tauri, jamais
  essayé par lui (fiche 115) : le déplacer ne lui coûte rien, et ses raccourcis se changent dans Glucose.
  Sous Windows, un plein écran peut réveiller RTX HDR (fiche 52) : il est demandé, il est ce qu'il dit.
* **DEVANT-1** (`plateforme/devant.rs`) : sous Wayland, **aucune requête** ne garde une fenêtre devant
  (la demande est en débat depuis des années). Chaque bureau par sa voie, la fenêtre désignée par **son
  processus** (jamais « celle qui a le focus ») : **niri** — la couche flottante, toujours au-dessus de
  la mosaïque (`niri msg action move-window-to-floating --id`, le numéro lu par `--json windows`) ;
  **Hyprland** (`setfloating pid:`), **sway** (`[pid=] floating enable`) ; **KDE** — un script de KWin
  chargé par D-Bus, `keepAbove` sur les fenêtres du processus ; **GNOME** et les autres : Glucose **le
  dit**, avec le geste du bureau (`Alt+Espace`, « Toujours au premier plan »). La commande part sur un
  fil. Au lancement, la référence retenue se pose devant **au premier focus** — le moment exact où la
  fenêtre existe chez le bureau, sans attente chronométrée.
* **Le déplacement au clic droit** lisait la position de la fenêtre, que Wayland ne donne jamais : il
  passe désormais par le geste du système (`drag_window`) au premier mouvement ; le système garde la
  main jusqu'au relâchement — que Glucose ne reçoit pas : le déplacement s'oublie aussitôt, seul reste
  que ce relâchement-là n'ouvre pas le menu (sinon chaque mouvement suivant aurait été avalé).
* **Essayé sous un niri imbriqué** (hors de son écran ; le Glucose d'essai ne reçoit que la prise du
  niri de l'essai) : `F11` — la fenêtre de 960 à 1 920 pixels, l'interface entière ; `Ctrl+Maj+A` —
  flottante, puis rendue à la mosaïque ; le clic droit glissé de 300 pixels — **la fenêtre déplacée de
  300 pixels** (la liste de niri ne met pas à jour la position d'une fenêtre flottante : mesuré sur les
  captures). KDE, Hyprland, sway, GNOME : **jamais essayés**, leurs commandes lues dans leur
  documentation.

## 4. BARRE-3 — la barre de Windows au choix

La bande de Glucose par défaut (elle rend au canevas la hauteur d'une barre), ou la barre blanche du
système — retenue (`barre-du-systeme`), **basculée sans relancer** : la procédure glissée devant celle
de `winit` se tait sur un drapeau, et Windows recalcule le cadre (`SWP_FRAMECHANGED`). Au menu du vide
et à la palette, **sous Windows seulement**. Éprouvée ici sur son souvenir ; la voie de Windows ne se
compile et ne s'éprouve que sur GitHub — **jamais essayée sous un vrai Windows**.

## 5. RENOMMER-1 — nommer un dossier

Sa parole, en cours de session : *« il n'y a pas de système pour nommer un dossier »* — vérifié : on
renommait un onglet et un document, jamais un dossier. **`F2`** (la touche de l'explorateur de Windows et
de VS Code), « Renommer… » au menu d'un dossier, la palette : la question de Glucose et son champ, le nom
choisi tout entier ; Entrée renomme, `Échap` non ; **un seul `Ctrl+Z`** ; un nom vide ou le même ne fait
aucun geste. La loi dans le noyau (`Store::renommer_le_dossier`), donc portée par le co-working.

## 6. Les tiroirs — ce qui reste à sa parole

Sa phrase dit quatre choses : des bulles incohérentes avec l'écran, qui s'entrechoquent, trop grosses,
et sans structure qui explique où elles s'ouvrent. Les deux du milieu sont réglées depuis la veille
(TIROIR-1 à 3). Pour les deux autres :

* **l'allure** : la charte (`style.md`) la tranche déjà — *« coins peu ou pas arrondis sur la chrome »*,
  *« pas d'ombre portée molle »* ; les panneaux ont une ombre de 8 px et des coins de 6 px : ce sont ses
  « bulles » ;
* **la structure** : aujourd'hui, Domaines, Mood board et Plugins — dont les boutons sont **à droite**
  de la bande — s'ouvrent **en haut à gauche**. La loi proposée : **un tiroir s'ouvre du côté de son
  bouton**, collé au bord, empilé sous les autres de ce côté.

Un prototype (hors du dépôt, `proto_tiroirs.py`) et son image, à lui montrer avant d'écrire.

## 7. Ses demandes neuves du 18/11 — le détourage

Ses mots sont dans VISION § 3. **La première recherche** :

* **À écarter** : RMBG-1.4 et RMBG-2.0 (BRIA) — licence non commerciale (CC BY-NC 4.0, ou un accord
  avec BRIA) ; c'est **le modèle par défaut de `rembg` depuis sa 2.0.80** : le piège
  ([RMBG-2.0](https://huggingface.co/briaai/RMBG-2.0),
  [rembg, 2026](https://www.creativeainews.com/articles/rembg-background-removal-models-tested-2026/)).
* **Permis** : U²-Net et u2netp (Apache 2.0 ; u2netp 4,7 Mo, rapide, moins fin), ISNet (Apache 2.0,
  176 Mo, meilleur qu'U²-Net), **BiRefNet lite** (MIT, ~44 M de paramètres, entrée 1024², 224 Mo en
  fp32, ~89 Mo en fp16 — [BiRefNet_lite](https://huggingface.co/ZhengPeng7/BiRefNet_lite)).
* **En Rust** : `rembg-rs`, `outline-core` passent par ONNX Runtime (`ort`) — une dépendance lourde ;
  l'IA du sens de Glucose (fiche 106) dit comment le dépôt fait déjà tourner un modèle.
* **Sans IA** : un fond uni (blanc) se détoure exactement par remplissage depuis les bords —
  instantané, sur toutes les machines.
* À concevoir : le fond uni sans IA d'abord ; puis un modèle selon la machine (u2netp au téléphone,
  BiRefNet lite sur un bon PC) — ses « trois versions selon la machine » (VISION § 8 bis).

## 8. Les épreuves, les sabotages

* **GRILLE** : la hauteur d'x et la ligne de base sur des rangées entières (corps 10 à 18, cinq
  fractions), l'encre jamais hors de la boîte exacte du contour décalé, la mesure inchangée, les
  polices partagées ; un trait d'icône de pixels entiers ; **chacune avec sa preuve à l'envers** (l'ancienne
  loi échoue sur les mêmes données). Les trois témoins mis à jour, regardés.
* **Sabotages GRILLE** (`outils/sabotages/grille-120.json`) : 10 sur 12 au premier passage — **deux
  portes aveugles trouvées** (le rendu réduit, les panneaux confiés à la carte) : deux épreuves
  écrites, sabotées avec le second chantier. Une leçon : la bande garde son dessin en cache dans
  l'état de l'interface ; une épreuve qui compare deux rendus doit partir chaque fois d'une interface
  neuve, sinon le cache de l'un fournit l'image de l'autre.
* **Sabotages du second chantier** (`outils/sabotages/fenetre-120.json`) : **18 sur 19**, dont les deux
  portes de GRILLE enfin gardées. **Un aveugle trouvé** : le script de KWin pouvait poser toutes les
  fenêtres devant, l'épreuve ne cherchait que le texte `f.pid === 5123` ; elle exige désormais la
  condition entière (`if (f.pid === 5123) {`) — verte sur le code intact, **le sabotage n'est pas
  rejoué** (la session s'est arrêtée là, à sa demande).
* **Pas fait avant l'arrêt** : la suite entière depuis FENETRE-1 (seules les épreuves des chantiers
  touchés, 280 environ, et les seize cliquets, et clippy strict) ; la CI ; le brouillon de la beta.2
  refait ; son Glucose installé mis à jour ; le prototype des tiroirs (§ 6 : l'image d'aujourd'hui est
  prise, celle du prototype non).

## 9. La recherche (consultée le 18/11/2026)

* [FreeType — le rendu du texte](https://freetype.org/freetype2/docs/hinting/text-rendering-general.html) :
  l'alignement « léger » (vertical seul), le mélange en lumière linéaire, l'épaississement des fûts.
* [Skia — « The Raster Tragedy »](https://docs.skia.org/docs/dev/design/raster_tragedy/) : la
  correction de contraste, réduite pour un texte clair sur fond sombre.
* [Chrome et Edge sous Windows, 2025](https://blogs.windows.com/msedgedev/2025/01/30/better-text-contrast-for-all-chromium-based-browsers-on-windows/) :
  le contraste du texte relevé par défaut.
* [niri — fenêtres flottantes](https://niri-wm.github.io/niri/Floating-Windows.html), sa
  [FAQ](https://github.com/niri-wm/niri/wiki/FAQ), et son
  [`move_grab.rs`](https://github.com/niri-wm/niri/blob/main/src/input/move_grab.rs) : pas de
  « toujours devant » ; un déplacement commence après 8 pixels, quel que soit le bouton.
* [KDE, bogue 460873](https://bugs.kde.org/show_bug.cgi?id=460873) : Wayland n'a pas de « garder
  au-dessus » ; le script de KWin est la voie.
* [Wayland, demande 305](https://gitlab.freedesktop.org/wayland/wayland/-/issues/305) : « toujours
  devant », toujours en débat.
* Le détourage : § 7.

## 10. Les leçons

* **Des épreuves vertes ne disaient rien du grain** : aucune ne regardait une lettre au pixel. Une
  expérience hors du dépôt, image contre image, contre un étalon (FreeType), a nommé les causes avant
  d'écrire une ligne.
* **Une porte sans épreuve passe tous les sabotages** : deux des six portes de la typographie de
  l'écran n'étaient gardées par rien — c'est le sabotage, pas la relecture, qui l'a montré.
* **Un état gardé en cache cache la loi qui l'a écrit** : comparer deux rendus qui partagent leur
  interface compare deux fois le même cache.
* **Un nombre se compte** : ÉTAT disait 109 commandes ; la table en comptait 113 avant cette
  session (62 simples aujourd'hui, 8 poussées, 7 outils, 18 signets, 7 relations, 5 opérateurs,
  9 couleurs : **116**).
* **Ce que la liste d'un compositeur ne dit pas, l'image le dit** : la fenêtre flottante avait bougé,
  la liste de niri disait le contraire.
