# 51 — Ses retours sur la V2 : la souris, le copier-coller, l'image, Ctrl+N, le mode référence, le repos

> Session des 06 et 07/10/2026. Les six retours qu'il a faits sur la 2.0.1-beta.1, et la
> préparation de la 2.0.2-beta.1. **Rien de ce qui suit n'a encore été vu à son écran** : tout est
> éprouvé hors écran, et la liste d'essais du § 9 dit ce qui reste à juger.

---

## 0. Ce qui précédait

* La CI de `c744505` (l'envoi d'avant) : neuf tâches vertes.
* La version de travail passe de `2.0.0-dev` à **`2.0.2-dev`**. L'épreuve de version compare
  désormais à **tout** ce qui a été publié (1.0.2-beta.1 de Tauri et 2.0.1-beta.1 de Rust) ; elle
  ne comparait qu'à Tauri, et l'oubli de la fiche 48 § 15.4 était passé sans qu'elle tombe.

## 1. La souris instantanée (`interactions/elan.rs`, `pan_zoom.rs`)

**Le défaut** : tout le lissage avait été réglé au pavé tactile (conduite 10 ms, glissades de
0,45 et 0,28 s), et la molette comme le glisser au bouton du milieu en héritaient. Son mot :
*« à la souris, tout algorithme de smooth, c'est un peu horrible »*.

**Deux portes dans l'élan**, pas un réglage commun :

* `pousser_pan` / `pousser_zoom` : le doigt, **inchangé** — conduite, puis glissade ;
* `placer_pan` / `placer_zoom` : la souris — **tout ce qu'elle demande se montre à l'image
  suivante**, sans seuil d'extinction (un dixième de pixel demandé est montré : sinon la vue
  dériverait sous le curseur d'un glisser lent), et **rien ne glisse** au lâcher.

La dette reste le passage obligé, pour une seule raison : la caméra ne bouge qu'une fois par
image, et deux événements arrivés entre deux images s'y rejoignent. La souris arrête net une
glissade du doigt en cours ; le doigt qui reprend retrouve sa conduite et son élan.

**Ce qui a dû changer autour** : à la souris, la dette s'éteint **dans** l'image qui la montre,
donc `elan.en_cours()` est faux pendant que la vue bouge. Le rendu (ce qu'on a le droit
d'abîmer), le tempo (la régularité) et la finesse demandent désormais `la_vue_bouge()` — « cette
image montre-t-elle un mouvement ? » —, et non « reste-t-il une dette ». Un cran isolé, lui, part
tout de suite : le tempo ne retient pas la première image d'un mouvement.

**Le doute qui reste, et c'est le sien à lever** : une molette à **haute résolution** (les roues
libres de Logitech, le multiplicateur de résolution de HID) envoie des fractions de cran,
comme un pavé ; Glucose la prend pour un doigt, et son défilement **déplace** la vue au lieu de
zoomer — comme dans Glucose Tauri (`% 120`). La recherche :

* Windows ne dit pas d'où vient un défilement : `GetCurrentInputMessageSource` répond « souris »
  pour un pavé ([ImageGlass #2465](https://github.com/d2phap/ImageGlass/issues/2465)) ;
* un pavé de précision envoie des deltas de 1 (ultra-haute précision) ou 40, une molette 120
  ([Microsoft, *Windows precision touchpad devices*](https://learn.microsoft.com/en-us/windows/compatibility/precision-touchpad-devices)) ;
* **la seule réponse exacte connue** est celle de Chromium et de Blender : prendre le pavé par
  **Direct Manipulation** ([Blender, *GHOST: precision touchpad gestures*](https://developer.blender.org/rBe58b18888c0e)),
  qui livre les gestes du pavé avec leurs phases, leur point focal et leur inertie — après
  quoi tout ce qui arrive encore en `WM_MOUSEWHEEL` vient d'une molette. C'est un chantier à lui
  seul (COM, `DM_POINTERHITTEST`), noté dans la suite ; **demander d'abord le modèle de sa souris**.

**Épreuves** : six dans `elan/tests_souris.rs`, chacune avec sa preuve à l'envers (le même geste
par la porte du doigt montre le défaut), et NAV-4 dans `pan_zoom/tests.rs`, jouée dans
l'application. Sept sabotages, sept chutes.

## 2. Copier, couper, coller des nœuds (`core/store/lot.rs`, `interactions/clipboard/lot.rs`)

**Le piège d'abord** : `Ctrl+C` ne copiait que du texte (le contenu des cartes, et le *nom
interne* des images), et `Ctrl+X` supprimait toute la sélection. Couper une sélection riche la
détruisait.

**Un lot est un document.** `Store::extraire_la_selection` rend un `Project` d'un tableau : ce
que la sélection emporte (`Emport`, le contenu des membranes compris), **plus les flèches qui
relient deux nœuds emportés**, et les domaines portés. Le bureau l'enveloppe dans un vrai
`.glucose` (`persist::encode`, images dédupliquées par empreinte, sommes de contrôle). Il n'y a
donc pas de « format du presse-papiers » à maintenir : c'est le format du fichier.

**Le canal, après recherche** :

* tldraw et Figma glissent leurs données dans du **HTML** ([tldraw, *clipboard*](https://tldraw.dev/sdk-features/clipboard),
  [Simon Willison, *The web's clipboard*](https://simonwillison.net/2024/Sep/19/the-webs-clipboard/)),
  parce qu'un navigateur n'a pas d'autre canal ; Excalidraw met son JSON en texte ;
* mais un lot pèse ce que pèsent ses photos, et Word ou un client de courrier recevraient des
  mégaoctets de base64. **Sous Windows**, Glucose enregistre donc **son format**,
  « Glucose.Lot » (`RegisterClipboardFormat`), qui porte les octets tels quels, avec le texte des
  nœuds en `CF_UNICODETEXT` dans la même ouverture ; **ailleurs**, `arboard` n'ouvre pas de
  format à soi, et le lot voyage dans du HTML, comme chez tldraw. Huit octets de longueur
  précèdent le lot : `GlobalSize` arrondit.

**Coller** : `Store::coller_un_lot` renomme tout par le renommage de l'import (BOARDS-2, dont
`nommer_le_contenu` a été extrait), fait suivre chaque référence, pose le lot **centré sur le
curseur**, en **un seul geste**, et le sélectionne. Un lot de ce document (reconnu à sa date,
`created_at`, posée à la nanoseconde de la copie) garde ses liens vers les onglets et les
domaines d'ici ; un lot d'ailleurs retrouve un domaine identique en tout, ou apporte le sien.
Chaque image reçoit une clé d'ici : la sienne (même document), celle qui porte déjà les mêmes
octets, ou `lot:<empreinte>` en mémoire, que le scribe scelle comme un dépôt.

**Rien de lourd sur le fil qui dessine** : la copie (lire les images, attendre une promesse,
encoder) et la relecture du collage se font sur un fil ; `Echanges` suit leur retour, la boucle
repasse toutes les 4 ms (raison « une commande attend »). **Couper ne retire rien tant que le
lot n'est pas dans le presse-papiers** ; un `Ctrl+V` tapé pendant la copie attend son tour.

**Deux défauts trouvés en chemin** : `delete_selected` écrivait **deux** gestes pour une
sélection mêlée (un `Ctrl+Z` ne rendait que la moitié) — c'est un seul désormais ; et
`organize_layout`, morte (`allow(dead_code)`), est retirée.

**Les messages** : un collage se voit — les nœuds apparaissent, sélectionnés — et se tait, comme
un vol réussi ; « Texte collé » et « Image collée » se taisent aussi. La copie se dit (« 3 nœuds
copiés »), ses échecs passent par le site unique du presse-papiers. Le cliquet des toasts n'a pas
bougé.

**Le glisser d'une fenêtre à l'autre** (`plateforme/glisser_windows.rs`) : des nœuds tenus qui
sortent de la fenêtre y reviennent (le glisser s'abandonne, comme sous `Échap`), et leur lot part
par `DoDragDrop`, dans un objet de `SHCreateDataObject` (le format du lot et le texte). Glisser
**copie** : rien ne peut se perdre en route. La cible de dépôt reconnaît le lot avant tout autre
format et le colle au point de lâcher ; elle refuse un lot pendant qu'un glisser part de sa propre
fenêtre. Fonctionnalité `Win32_UI_Shell_Common` ajoutée à la caisse `windows`, déjà là.

**Épreuves** : sept dans le noyau (l'aller-retour au bit près — re-extraire ce qui est collé
redonne le lot, identifiants mis à part —, un geste, le centre, les flèches, les domaines des deux
provenances, la suppression d'un coup) ; quatre de bout en bout (`persist/disque/lot_tests.rs` :
une sélection mêlée d'une application à une autre avec les octets de la photo, **l'original
effacé** ; couper puis coller ; `Ctrl+V` pendant la copie ; une coupe qui échoue ne retire rien) ;
l'enveloppe HTML ; et le glisser : l'objet que fabrique la source, lu par la cible, rend le lot
octet pour octet. Quatorze sabotages ; **deux épreuves aveugles trouvées** — l'autre fenêtre
relisait la photo depuis son fichier d'origine resté sur le disque (l'épreuve efface désormais
l'original), et une épreuve de l'élan mesurait la demande au lieu de la glissade.

**Non éprouvé** : la voie Windows réelle du presse-papiers (une épreuve qui l'écrirait remplacerait
ce qu'il vient de copier), et le glisser à la main. C'est son écran qui le dira.

## 3. Clic droit sur une image (`interactions/clipboard/menu_image.rs`)

* **« Copier l'image »** pose ce que pose « Copier l'image » d'un navigateur : un **PNG** sous le
  format enregistré « PNG » — que Chromium, donc Discord et toute page web, lit d'abord
  ([`clipboard_win.cc`](https://chromium.googlesource.com/chromium/src/+/master/ui/base/clipboard/clipboard_win.cc)) —
  et les pixels en **`CF_DIBV5`**, d'où Windows tire le bitmap des autres (rangées du bas vers le
  haut : Word refuse une hauteur négative, arboard l'avait appris). Un PNG part **tel quel** ;
  un JPEG ou un WebP s'encode en PNG, sans perte de ce qu'il montre. Un GIF animé ne garde que sa
  première image, comme dans un navigateur. Ailleurs que sous Windows : `arboard::set_image`.
* **« Enregistrer l'image sous… »** écrit les octets **scellés** dans le document, tels quels,
  avec l'extension que leur signature dit, par la seule porte qui pose un fichier (cliquet 11).

Le menu d'une sélection gagne aussi « Copier » et « Couper ». Les deux entrées de l'image ne
paraissent que pour **une seule** image choisie.

## 4. `Ctrl+N` (`persist/nouveau.rs`)

Le document qu'on quitte passe par `laisser_le_document` (BROUILLON-1 : un document nommé se
quitte sans un mot, du travail sans nom pose la question). Le nouveau est **vierge** — sans carte
d'accueil, sans fichier tant qu'on n'y a rien fait ; son premier geste fait naître son brouillon.
Entrée « Nouveau document » au clic droit sur le vide. Un sabotage a montré qu'une ligne
(`journal.clear()`) était redondante avec `load_project` (JRN-2) : elle est partie.

## 5. Le mode référence, à la PureRef (`interactions/reference.rs`)

Les gestes de PureRef, lus dans son manuel ([raccourcis par défaut](https://www.pureref.com/handbook/shortcuts/all-shortcuts/),
[navigation](https://www.pureref.com/handbook/navigation/)) : **glisser au bouton droit déplace
la fenêtre**, le gauche sur un bord la redimensionne, `Ctrl+Maj+A` la met au premier plan,
`Ctrl+T` laisse passer les clics, `Ctrl+Maj+±` règle l'opacité.

* **`Ctrl+Maj+A`** (et `Alt+T`, l'ancien geste caché, et le menu) bascule tout le mode : plus
  d'interface (bande, onglets, minimap, panneaux, barre d'action — **le menu et les messages
  restent**, sinon on ne saurait plus en sortir), plus de cadre, au premier plan, **retenu** à la
  relance (un fichier `mode-reference` dans le dossier où l'application habite ; la fenêtre naît
  directement sans cadre). Le même geste le défait.
* La hauteur de la bande vaut **zéro** en mode référence : tout ce qui se mesure sur elle — le
  canevas, les clics, les panneaux — la voit disparaître d'un seul nombre.
* **Déplacer** : bouton droit tenu, la fenêtre suit le curseur lu **sur l'écran** ; un clic droit
  sans bouger ouvre toujours le menu. **Redimensionner** : le bouton gauche sur une bordure de
  huit pixels logiques (celle que Windows donne à toute fenêtre), par `drag_resize_window`, avec
  le curseur des bords.
* Pas encore : l'opacité, les clics qui traversent — à lui de dire s'il les veut.

## 6. Le repos

Sa chronique du 06/10 (une heure) dit **0,1 image par seconde** au repos — plus 1 à 2,5 comme au
29/09. La lecture du code ne trouve **aucun réveil spontané** : au repos, la boucle dort
(`ControlFlow::Wait`). Mais la chronique appelle « repos » toute seconde sans la main, et y
range donc **les suites d'un geste** : la glissade du pavé qui s'éteint, le message qui
s'efface, une photo qui finit de se décoder. **Probablement la cause, à confirmer** : la veille
dit désormais, **raison par raison**, ce qui a été dessiné au repos. Sa prochaine session le
dira ; si une raison spontanée y paraît, c'est elle à éteindre.

## 7. La 2.0.2-beta.1, préparée

Les notes de publication (`outils/publication/notes.md`) disent ce qui change. **La publier reste
son geste** (fiche 48 § 15.4) : Actions → Publier, version `2.0.2-beta.1`, brouillon coché,
relire, publier. Après, la version de travail reste `2.0.2-dev`, déjà au-dessus.

## 8. Les cliquets, tenus sans être relevés

`app.rs` était à 600 lignes : `apply_dock_layout` est sortie dans `app/rangement.rs`. Les
épreuves en ligne de `persist/commands.rs` sont sorties dans leur fichier. `GlucoseApp::new` et
`init_window` dépassaient : un commentaire de valeur de naissance est allé à son champ, la
fabrication des attributs de la fenêtre à sa fonction. Le couplage au modèle n'a pas monté (un
`Inventaire` du lot, et `Store::image`).

## 9. Ce que seul son écran dira

1. **La souris** : un cran de molette, une série de crans, le glisser au bouton du milieu —
   instantané, rien ne glisse. Puis le **pavé** : inchangé.
2. **Couper-coller** une sélection mêlée (texte, photo, flèche, membrane) dans la même fenêtre.
3. **Deux Glucose ouverts** : copier dans l'un, coller dans l'autre ; puis **glisser** des nœuds
   de l'un à l'autre.
4. **Clic droit sur une image → Copier l'image**, coller dans **Discord** ; puis **Enregistrer
   l'image sous…**, ouvrir le fichier.
5. **`Ctrl+N`**, sur un document nommé, puis sur un travail sans nom.
6. **Le mode référence** au-dessus de **Blender** : `Ctrl+Maj+A`, déplacer au bouton droit,
   redimensionner par un bord, relancer Glucose, `Ctrl+Maj+A` encore.
7. Une session laissée **au repos** quelques minutes : la chronique dira qui dessine.
