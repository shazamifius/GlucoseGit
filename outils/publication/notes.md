**Glucose {version}** — Glucose, réécrit en Rust.

**À lire d'abord : mettez à jour toutes vos machines.** Cette version écrit vos documents dans une forme plus récente (une image peut désormais être retournée) : les versions d'avant disent « ce document vient d'une version plus récente de Glucose » et n'y touchent pas. Rien n'est perdu — il s'ouvrira dès qu'elles seront à jour. Au téléphone, installez l'APK **par-dessus** l'ancien : ne désinstallez jamais Glucose, vos brouillons vivent dans l'application.

**Ce qui change depuis la 2.0.4-beta.1**

* **Le texte et les icônes de l'interface sont nets.** Les lettres de la barre, des menus et des panneaux se posent sur les pixels de l'écran, comme celles du système ; les icônes sont tracées en traits de pixels entiers. Le canevas, lui, ne change pas.
* **Sous Windows, chaque bouton de la barre du haut répond enfin là où il est dessiné.** Le clic cherchait les boutons de droite (Collaborer, Exporter, Plugins, Mood board, Domaines) à côté de leur place, et un clic dans le vide de la barre commençait une sélection qui faisait défiler le canevas. Un clic dans la barre ne touche plus jamais le canevas.
* **Travailler à plusieurs dans le même canevas.** Collaborer → « Partager ce canevas » copie un code ; chez l'autre, Collaborer → « Rejoindre » le lit. Chaque geste se voit chez les autres, chiffré de bout en bout ; deux gestes sur le même nœud, le plus récent l'emporte et Glucose le dit ; `Ctrl+Z` ne défait que vos gestes ; les curseurs des autres sont nommés. Un second code permet de seulement regarder. Les photos passent d'un participant à l'autre : celles que vous posez partent avec votre geste, celles qui vous manquent se demandent quand vous entrez — à ceux qui sont là, car le relais n'en garde aucune.
* **Une IA du nuage dans votre document** : Collaborer → « La porte aux IA publiques » laisse Claude.ai lire et écrire dans le document ouvert, après votre accord, chacun de ses gestes signé de son nom.
* **Dater ce que vous posez, et la frise chronologique** : `Maj+T` date un nœud (une année, une plage, une ère) ; `Maj+R` ouvre la frise, dont la fenêtre fait pâlir ce qui sort de la période. Dans le panneau Domaines, « voir » fait pâlir ce qui n'est pas du domaine.
* **Exporter en page web** : une page autonome, qui se parcourt à deux doigts.
* **Trois modes pour la fenêtre** : `F11` passe en plein écran (la barre reste) ; `Ctrl+Maj+A`, le mode référence, garde Glucose sans cadre et devant les autres applications — désormais aussi sous Linux, avec niri, KDE, Hyprland et sway (sous GNOME, Glucose vous dit le geste à faire) ; le mode zen, plus que le canevas, passe à `Maj+F11`. Glucose s'en souvient au prochain lancement.
* **Sous Windows, la barre blanche du système revient si vous la préférez** : « Barre de Windows » au clic droit dans le vide.
* **Renommer un dossier** : choisissez-le, `F2` (ou « Renommer… » au clic droit), tapez, Entrée.
* **Les tiroirs** (Domaines, Storyboard, Time Machine…) ont la hauteur de ce qu'ils montrent, ne passent plus l'un sur l'autre, et leur contenu ne couvre plus le bouton qui les referme. Le fil d'un dossier se lit en entier.
* **Les flèches** : une poignée à chaque bout, à glisser sur un autre nœud ou dans le vide ; clic droit sur un point orange, « Retirer ce coude » ; « Redresser la flèche » au menu.
* **Retourner une image** : `Alt+Maj+H` (de gauche à droite), `Alt+Maj+V` (de haut en bas), ou le menu d'une image.

**Encore connu** : dans un canevas partagé, une photo dont personne de connecté n'a les octets reste grise jusqu'au retour de quelqu'un qui les a, et il faut rejoindre de nouveau par le code après avoir relancé Glucose ; sur de très grands écrans, le texte de l'interface peut paraître petit (une échelle à votre choix viendra) ; sur les très grandes cartes, le zoom a encore des à-coups.
