**Glucose {version}** — Glucose, réécrit en Rust.

**Ce qui change depuis la 2.0.3-beta.9**

* **Les liens.** De loin, ils pâlissent et laissent la carte se lire ; ceux du nœud qu'on survole ou qu'on choisit s'allument. `Ctrl`+clic sur une flèche fait voler la vue jusqu'à son autre bout. `B` (ou « Montrer ses liens » au menu) amène autour d'un nœud tous ceux qui lui sont liés, chacun dans la direction de sa vraie place et avec sa provenance ; un clic y mène.
* **Revenir où l'on était** : `Alt+Z` ramène la caméra au lieu d'où elle est partie, `Alt+Maj+Z` y repart (« Vue précédente » au menu).
* **Chercher dans tout le document** : `Ctrl+F`, puis `F3` pour le suivant.
* **La couleur** : une barre de couleur pour les textes, les post-it, les membranes et les dossiers ; une roue de couleur, avec le code qu'on tape et la pipette ; la couleur d'un texte se voit enfin.
* **Ranger** : « Disposer en colonnes » sur une membrane ; `Ctrl+G` entoure la sélection d'une membrane neuve ; « Entrer dans la membrane » (le mode Focus) au menu.
* **Les opérateurs ET, OU, MAIS, PARCE QUE** sur les post-it (`Alt+1` à `Alt+4`, et le menu).
* **Les domaines** traversent les documents, et une membrane donne les siens à ce qu'elle contient.
* **Les grandes cartes** : le nom d'une membrane grandit avec elle, et de loin un texte montre le gris de ses lignes au lieu de disparaître.
* **Sous Linux** : copier et coller (vers Discord, un navigateur…) et le glisser-déposer marchent sous Wayland ; le pincement du pavé zoome ; des commandes au terminal (`glucose aide`).
* **Plus fluide** : déplacer la vue à la souris sans à-coups ; les photos ne clignotent plus ; une carte immobile laisse l'ordinateur au repos ; beaucoup de flèches ne figent plus l'image.
* **Au téléphone** : les barres du bas passent à la ligne et ne couvrent plus la minimap.
* **Corrigé** : le titre d'une membrane montre ce qu'on tape ; le contour d'un texte ne se décroche plus pendant qu'on écrit.

**Encore connu** : sélectionner plusieurs milliers d'images d'un coup reste lent à chaque image ; des membranes emboîtées les unes dans les autres se lisent mal — c'est le prochain chantier.
