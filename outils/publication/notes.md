**Glucose {version}** — Glucose, réécrit en Rust.

**Si vous venez de la 2.0.4-beta.1 ou d'avant : mettez à jour toutes vos machines.** Depuis la 2.0.4-beta.2, Glucose écrit vos documents dans une forme plus récente (une image peut être retournée) : les versions d'avant disent « ce document vient d'une version plus récente de Glucose » et n'y touchent pas. Rien n'est perdu — il s'ouvrira dès qu'elles seront à jour. Au téléphone, installez l'APK **par-dessus** l'ancien : ne désinstallez jamais Glucose, vos brouillons vivent dans l'application.

**Ce qui change depuis la 2.0.4-beta.2**

* **Les émojis sont en couleur** : 🟢 et 🔴 ne sont plus le même rond. Les dessins sont ceux de Twemoji, nets à tout zoom, dans le canevas comme dans l'export SVG ; vos textes gardent exactement leur mise en page. Les signes dessinés en noir (✔, ➕, ♠…) s'éclaircissent sur le fond sombre de vos textes, et gardent leur couleur sur un post-it.
* **Les tableaux tiennent dans leur nœud** : une cellule trop longue passe à la ligne dans sa colonne, au lieu de sortir du bloc ; un filet discret sépare deux rangées.
* **Les listes s'imbriquent** : un élément décalé sous un autre se range un cran plus loin, sa puce avec lui.
* **Un texte qui grandit pousse sa membrane** : quand un texte s'allonge, le bas de la membrane qui le contient le suit — à la saisie comme à l'ouverture d'un document.
* **Les tiroirs s'ouvrent du côté de leur bouton**, collés au bord, sans ombre, rangés en colonnes : plus aucun ne se pose sur un autre.
* **Agrandir ou réduire toute l'interface** : « Agrandir l'interface » au clic droit dans le vide, de 50 à 300 % ; Glucose s'en souvient.
* **Détourer une photo** : clic droit sur une photo, « Détourer : la forme principale » garde le sujet et rend le fond transparent ; « Retirer le fond uni » retire un fond d'une seule couleur. `Ctrl+Z` rend la photo d'origine.
* **Le texte et les icônes de l'interface sont nets.** Les lettres de la barre, des menus et des panneaux se posent sur les pixels de l'écran, comme celles du système ; les icônes sont tracées en traits de pixels entiers. Le canevas, lui, ne change pas.
* **Trois modes pour la fenêtre** : `F11` passe en plein écran (la barre reste) ; `Ctrl+Maj+A`, le mode référence, garde Glucose sans cadre et devant les autres applications — désormais aussi sous Linux, avec niri, KDE, Hyprland et sway (sous GNOME, Glucose vous dit le geste à faire) ; le mode zen, plus que le canevas, passe à `Maj+F11`. Glucose s'en souvient au prochain lancement.
* **Sous Windows, la barre blanche du système revient si vous la préférez** : « Barre de Windows » au clic droit dans le vide.
* **Renommer un dossier** : choisissez-le, `F2` (ou « Renommer… » au clic droit), tapez, Entrée.
* **Une IA qui travaille dans votre document par la porte** dit dans quel onglet elle travaille, et le code à vérifier est dans le titre de la question d'accord.

**Encore connu** : dans un canevas partagé, une photo dont personne de connecté n'a les octets reste grise jusqu'au retour de quelqu'un qui les a, et il faut rejoindre de nouveau par le code après avoir relancé Glucose ; une famille d'émojis (👨‍👩‍👧) s'affiche en personnes côte à côte, et un drapeau en deux lettres ; quand l'écran manque de hauteur, deux tiroirs peuvent encore se croiser ; sur les très grandes cartes, le zoom a encore des à-coups.
