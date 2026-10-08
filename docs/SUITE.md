# La suite, dans l'ordre

> **Au 08/10/2026, dans la nuit** (fiche 58). La feuille de route de Glucose Rust après la V2. Elle remplace les plans des
> fiches 36, 44 et 46 du [carnet](carnet/00-INDEX.md), qui en gardent le raisonnement. Chaque
> chantier dit **pourquoi il est là**, et ce qui le déclare fini. Tout y est à remettre en question.
>
> **Comment s'en servir** : prendre le premier chantier non fait. Quand il est fini (prouvé, pas
> « ça compile »), le cocher ici, mettre à jour [ÉTAT](ETAT.md), et passer au suivant.

> **Son ordre, le 07/10 au soir** (fiche 53 § 7) : *« une version ultra stable avec tout le mode
> Ctrl+Maj+A, compatible tablette, Linux, Windows et Mac ; un build complet pour tous les Linux,
> tous les Mac, et SURTOUT Android ; et une télémétrie de tous les utilisateurs, quel que soit
> leur appareil »*. D'où l'ordre ci-dessous : finir le chantier 1 (le déplacement au pavé, la
> sélection, la CI, la 2.0.2), puis **Android** (chantier 2), les versions complètes pour Linux
> et Mac (chantier 3), la télémétrie (chantier 4, qui attend sa décision sur le serveur).
>
> **Le 08/10 (fiche 54)** : la télémétrie est écrite des deux côtés et n'attend que son compte
> Cloudflare ; Android compile, avec les doigts, et attend son premier téléphone — l'APK se
> construira sur GitHub au prochain envoi avec CI.

---

## 0. Ses demandes du 08/10 au soir — **avant tout le reste**

Ses mots, après la fiche 57 : les 12 commits envoyés **avec la CI** ; la clé Android copiée hors
du PC (« c'est fait, c'est enregistré ») ; la publication de la 2.0.2-beta.1 seulement **si la
mise à jour automatique fonctionne pour tout le monde**.

| | chantier | pourquoi, et ce qui le déclare fini |
|---|---|---|
| [~] | **POPUP-1 — toutes les questions dessinées dans Glucose, partout** (urgent) — **écrit, éprouvé hors écran (fiche 58), à juger à son écran** : la croix, `Ctrl+N`, `Ctrl+O`, la mise à jour, le journal technique ; Entrée, Tab, Échap ; une seule boîte du système reste, la mise à jour cherchée avant la fenêtre (DIAL-5) | *« souvent des black screen total et des freeze ; il faut voyager dans le noir total, faire Tab puis Entrée pour autoriser la télémétrie »*. Les dialogues du système (`rfd`, `dialogue::oui_ou_non`, `oui_non_ou_annuler`, `poser`, appelés sous `sous_un_dialogue`) bloquent la boucle pendant qu'ils sont ouverts : la fenêtre ne se repeint plus. La question dessinée (QUESTION-1, `ui::question`) existe déjà et marche au téléphone : la faire servir partout (`ui.questions_dessinees` à vrai), et convertir les questions encore synchrones — le travail non enregistré (`persist/close.rs`, BROUILLON-1, qui attend une réponse avant de fermer), la mise à jour (`app/lancement.rs`). Les sélecteurs de **fichiers** (ouvrir, enregistrer sous, exporter, ajouter des images) restent ceux du système : à lui de dire s'ils posent le même problème. Fini quand plus aucune question oui/non ne passe par `rfd`, et quand il l'a jugé à son écran. |
| [ ] | **GESTES-1 — des raccourcis à deux et trois doigts** — **la carte lui est proposée** (fiche 58 § 4) : deux doigts annulent, trois rétablissent, un doigt qui tient un nœud pendant qu'un autre touche ajoute à la sélection, un appui long sur le vide puis glisser trace le rectangle ; **attend sa parole** | *« impossible de faire une multi-sélection et un clic droit ; avec 2 doigts on crée des raccourcis, et avec 3 aussi ; le plus simple possible, le moins de boutons et de pages possible »*. Attention : deux doigts **déplacent et pincent** déjà la vue (la similitude, fiche 54) — un toucher à deux doigts (sans mouvement) est libre, comme dans Procreate (deux doigts = annuler, trois = rétablir) ; chercher ce que font Procreate, Infinite Painter, Concepts, tldraw, Figma au doigt, et lui proposer **une** carte de gestes courte avant d'écrire. |
| [~] | **Les noms dans le rail** : une icône et son nom — il est d'accord | **écrit (fiche 58 § 3)** : colonne après colonne, sans défiler, les icônes seules quand la place manque ; **à son téléphone** |
| [~] | **Les emojis** : la police monochrome Noto Emoji (OFL) en repli d'Inter — d'accord | **écrit (fiche 58 § 5, `decisions/10`)** : 887 Ko, Unicode 15 ; les familles et les drapeaux se décomposent ; **à son téléphone** |
| [~] | **La mise à jour automatique, vérifiée pour tout le monde** avant de publier : Windows et Linux (la bascule est répétée par la CI) ; **Android : écrite (MAJ-ANDROID-1, fiche 58 § 6)** — l'APK confié à `PackageInstaller`, signé de la clé de Glucose et inscrit au manifeste. **Ne se prouve qu'à une publication** : le premier Glucose qui la porte s'installe à la main, le suivant arrive seul ; la 2.0.2-beta.1 en brouillon ne la porte pas (à republier) | sa condition à la publication |

## 1. Ses retours sur la V2, et la 2.0.2 bêta — **d'abord**

Fiches 51 et 52. Au soir du 07/10, **presque tout est jugé à son écran** : « tout est absolument
parfait ». Restent le pincement, le dessin de la sélection, et la publication.

> **La CI n'a pas vu les commits du 07/10 après `20deb6f`.** Il a demandé l'envoi **sans**
> déclencher la vérification (le dernier commit porte `[skip ci]`). Or ils touchent du code
> propre aux autres systèmes (`plateforme/ecran.rs`, `plateforme::glisser_un_lot`) : **avant la
> publication**, lui proposer un envoi qui lance la CI, et en lire le résultat.

`[x]` fini et prouvé · `[~]` écrit, éprouvé hors écran, **attend son jugement à l'écran** · `[ ]` à faire

| | chantier | où en est-il |
|---|---|---|
| [x] | **La version de travail en `2.0.2-dev`** | l'épreuve compare à tout ce qui est publié (sabotée : elle tombe) |
| [x] | **La souris instantanée** : deux portes dans l'élan, la souris montrée à l'image suivante | essayée à la souris le 07/10 : « tout est absolument parfait » ; il navigue surtout au pavé |
| [x] | **Copier, couper, coller des nœuds, d'une fenêtre à l'autre** : le lot `.glucose`, le format « Glucose.Lot », le collage au curseur en un geste ; le glisser entre fenêtres (`DoDragDrop`) | **« absolument parfait »**, et le glisser entre deux fenêtres aussi (07/10) |
| [x] | **Clic droit sur une image : « Copier l'image » et « Enregistrer l'image sous… »** | **« parfait »**, Discord compris (07/10) |
| [x] | **`Ctrl+N` : un nouveau document**, et l'entrée du menu | **demande toujours** (NOUVEAU-1) : « fonctionne parfaitement » (07/10) |
| [x] | **Le mode référence, à la PureRef** : `Ctrl+Maj+A`, sans interface, sans cadre, au premier plan, retenu ; **au pavé, `Alt` + glisser déplace la fenêtre et `Alt` + pincer la redimensionne** (REFERENCE-2) ; il se désagrandit en entrant | « absolument et vraiment sublime » (07/10), signets compris. Plus tard, s'il le veut : opacité, clics qui traversent |
| [x] | **Le repos à zéro image** : la chronique du 07/10 le dit raison par raison | **aucune raison spontanée** : les messages qui s'effacent et les glissades qui finissent |
| [x] | **La CI de `20deb6f`** (la voie hors Windows du presse-papiers) | neuf tâches vertes |
| [x] | **ECRAN-1 — la carte qui tient l'écran** : le lag du 07/10 (gels de 500 ms dans `present`) venait d'un balancier de l'arbitre entre la RTX, qui tient son écran, et l'Intel (fiche 52 § 1) | **vérifié** sur sa session suivante : la RTX, « celle qui tient l'écran », `mailbox`, pire image 48 ms au lieu de 505 |
| [x] | **POIGNEE-1 — les poignées, le cadre et l'anneau de sélection suivent la place du nœud** à l'écran (son retour du 07/10, deux fois) | « parfait » (07/10) |
| [x] | **Le pincement au pavé** : *Direct Manipulation* (chantier 1 bis, fiche 53 § 2) | « dans l'idée c'est bien, le zoom maintenant » (07/10) |
| [~] | **Le déplacement à deux doigts** : dix-neuf déplacements pris pour des pincements et bloqués (sa chronique du 07/10) ; la similitude entière des doigts, sans bascule (fiche 54 § 3) | Windows zoome sous le curseur (**vérifié**, 2 px) ; **son ressenti** sur le biais |
| [~] | **GROUPE-1 — le cadre du groupe suit le groupe** pendant le geste (fiche 54 § 2) | **à son écran** |
| [ ] | **La coupure du pavé de 7 à 10 s**, aussi dans Blender | la boîte noire en note chaque étage (fiche 54 § 4) : **la prochaine coupure le dira** |
| [~] | **Pinterest, DEPOT-WEB-6** : la page compressée, un repli qui dit pourquoi, « Remplacer par l'image », une session gardée, les variantes en course, **la copie posée tout de suite puis remplacée par l'original** (fiche 53 § 1, 7, 9) | la copie en 0,3 à 0,9 s ; **à son écran** |
| [~] | **`F` cadre la sélection (ou tout), `Ctrl+F` tout** — sa demande (fiche 53 § 7) | éprouvé par la vraie touche ; **à son écran**. `Ctrl+F` prend la place de la future recherche : à lui dire |
| [x] | **La molette** : un tiers d'octave par cran (« pas du tout assez rapide » au huitième) | « parfait » (07/10) |
| [x] | **Le mode référence, suite** : les panneaux retirés aussi de la voie graphique ; SIGNET-2 (les signets retiennent le centre) ; REFERENCE-3 (la fenêtre se redimensionne une fois par image — le gel qui a « failli planter » son PC, cause probable) | « parfait » (07/10) ; le gel n'est pas revenu |
| [~] | **Transformer une sélection entière**, origine commune ou individuelle, comme Blender (fiche 53 § 10) : poignées du groupe, `Alt` + coin pour tourner, bouton dans la barre | **à son écran** |
| [ ] | **Le dessin de la sélection** : il penche pour **E** (un cadre et un fil), C et B ses préférées — mais « leur design ne me convient pas encore » (fiche 53 § 10) | **à concevoir avec lui** : planches de variantes de E |
| [x] | **Le glisser vers une autre fenêtre** : la fenêtre d'origine se peint avant de partir, et ne fait plus tourner un cœur à vide | « parfait » (07/10) |
| [ ] | **Publier la 2.0.2-beta.1** : notes écrites (`outils/publication/notes.md`) ; son geste (Actions → Publier, `2.0.2-beta.1`, brouillon coché, relire, publier) | **après** ses essais à l'écran |

## 1 bis. Le pincement par *Direct Manipulation* — **écrit le 07/10, à juger à son écran**

**Pourquoi** : son jugement du 07/10, après trois réglages — « légèrement trop lent, et pas
fluide, comme s'il sautait », alors que PureRef et un navigateur pincent « instantanément, de
manière fluide ». Glucose reçoit le pincement d'un pavé de précision sous la forme que Windows
donne aux applications qui ne savent pas mieux : un `Ctrl` + molette fractionné, par paquets
(fiche 33, `interactions/pincement.rs`). Aucun réglage de gain ni de lissage ne rend la
continuité qu'il a perdue en route. Chromium et Blender prennent le pavé par *Direct
Manipulation* (`IDirectManipulationManager`, viewport, `DM_POINTERHITTEST`) : l'échelle, le
déplacement et l'inertie arrivent tels que le doigt les fait, à la cadence du pavé (fiche 51
§ 1, ses sources).

**Fini quand** : à son écran, le pincement est « instantané et fluide » comme dans PureRef ; et
tout ce qui arrive encore en molette vient d'une molette (ce qui règle aussi le doute des roues
libres). **Fait** (fiche 53 § 2, `decisions/07`) : reste son écran — le geste, le sens du
déplacement, l'élan. Une recherche d'abord : le code de Chromium (`direct_manipulation_helper_win.cc`) et le
commit de Blender cité en fiche 51 § 1 ; puis une dépendance éventuelle à défendre dans
`carnet/decisions/` (la caisse `windows` a la fonctionnalité `Win32_Graphics_DirectManipulation`).

## 1 ter. Le dessin de la sélection — **la planche lui est envoyée (fiche 53 § 3)**

**Pourquoi** : « pas esthétique du tout » (07/10). `style.md` impose une chrome monochrome,
héritée de Glucose Tauri. Quatre directions lui ont été dessinées hors écran : `docs/carnet/
screens/52-selection-quatre-directions.png` — A aujourd'hui, B des équerres aux coins, C un fil
et quatre points, D un cadre unique pour le groupe. **Il ne sait pas où regarder les fichiers** :
la lui **montrer** (l'envoyer à l'écran), recueillir son choix, puis l'écrire dans la loi des
ornements (`hit_priority::chrome_ratio`, `renderer/handles.rs`, `scene/image/ornement.rs`) et
dans `style.md`. Si c'est la couleur qui lui manque, c'est `style.md` qui change — sa décision.

## 2. Le toucher : **Android d'abord**, et l'iPad par le web — ce qu'il attend le plus

**Fait les 07 et 08/10 (fiches 54 à 56, `decisions/08` et `09`)** : Glucose compile pour Android et
**tourne sur son Redmi 9** ; la couche de gestes ; la clé de signature et l'APK dans la
publication ; **le rail** (la barre sur le côté quand elle ne tient plus — à juger à son écran).
**Sa liste du 08/10, dans son ordre** (fiche 55 § 4), la vidéo remise à plus tard — et ce que
la session du 08/10 (fiche 56) en a fait :

| | chantier | où en est-il |
|---|---|---|
| [~] | **VIE-1 — la surface qu'Android reprend et rend** : sans elle, revenir d'une autre application (le chemin même du partage) tombait sur une surface morte | écrit, éprouvé hors écran ; **à son téléphone** |
| [~] | **PARTAGE-1 — le partage d'images vers Glucose** : `SEND`, `SEND_MULTIPLE`, les images en un geste, un lien au rapatriement, Java à la frontière seulement (`decisions/09`) | **à son téléphone** : Pinterest, la galerie, Chrome |
| [~] | **Le double toucher** sur le vide : le menu à la taille du doigt, « Ajouter des images… » par le sélecteur de photos ; le bouton « images » du rail, qui ne faisait rien, aussi | **à son téléphone** |
| [~] | **QUESTION-1** — la question que Glucose dessine (aucun dialogue sous Android) ; **VUE-1** — le refus du journal technique donné à sa place se repose | **à son téléphone** |
| [~] | **DOCUMENTS-1** — les documents du téléphone : un travail sans nom se range sous « Canevas N », « Nouveau document » marche, « Ouvrir un document… » liste les rangés | **à son téléphone** |
| [~] | **CLAVIER-1 à 3 — le clavier du téléphone** (fiche 57) : la saisie et le clavier tiennent le même état (le miroir) ; les positions UTF-16 de Java ; l'état d'avant l'envoi ignoré ; la ligne qu'on écrit reste au-dessus du clavier ; la touche retour valide | écrit, éprouvé hors écran (treize sabotages) ; **à son téléphone** |
| [~] | **APPUI-1 — l'appui long** (fiche 57) : au délai que règle l'accessibilité d'Android, le menu du clic droit à la taille du doigt, le mot sous le doigt dans un texte, une vibration ; la question répond au relâchement | **à son téléphone** |
| [~] | **DOCUMENTS-2 — renommer, dupliquer, supprimer** par un appui long dans la liste (fiche 57) ; un champ dans la question, que le clavier remplit | **à son téléphone** |
| [~] | **Les emojis** : Noto Emoji en repli (EMOJI-1) | fiche 58 § 5 — **à son téléphone** |
| [~] | **BORD-1 — le bord à bord** (fiche 57 § 4 bis), avancé devant la mise à jour : partout, les marges du système lues dans `MainActivity`, la barre qui couvre la barre d'état, *l'écran visible* au-dessus de la navigation et du clavier | écrit, éprouvé hors écran ; **à son téléphone** |
| [~] | La **mise à jour automatique** sous Android — écrite (fiche 58 § 6), à prouver à la prochaine publication | |
| [ ] | Le presse-papiers du système ; la batterie sous Android (la télémétrie la lit nulle) ; les panneaux qui s'écartent des bords | |
| [ ] | Les doigts sous Windows (écarter la souris simulée, `GetMessageExtraInfo`) et l'iPad par le web. La vidéo (fiche 55 § 4) quand il la redemandera ; les photos **HEIC** par le décodeur d'Android | |

Le plan d'origine (fiche 50) ; l'ordre :

1. **L'exécuteur des tranches** : dans un navigateur, il n'y a pas de fils. Le travail de fond
   (scribe, atelier, boîte noire) doit pouvoir tourner **en tranches, dans le temps libre de chaque
   image**, comme une voie de plein droit, éprouvée d'abord sous Windows (mêmes résultats par les
   deux voies).
2. **La couche de gestes** commune : doigts, pincement, aucun survol, puis le crayon.
3. **Le web** (WebGPU dans Safari 26 sur iPad à puce A12 et après, WebGL2 en dessous) : chaque
   document écrit **aussi** dans les Fichiers de l'iPad, parce que Safari efface les données d'un
   site inactif et recharge une page trop gourmande. Une version web qui perd un document serait
   pire que rien.
4. **Android** : `GameActivity` (le clavier virtuel ne marche pas avec `NativeActivity`), la vie
   d'une application suspendue, l'APK et **sa clé à ne jamais perdre**, de vrais téléphones anciens
   par Firebase Test Lab (gratuit).

## 3. Des versions complètes : tous les Linux, tous les Mac, le mode référence partout

**Pourquoi** : sa demande du 07/10 — « une version ultra stable avec tout le mode Ctrl+Maj+A,
compatible tablette, Linux, Windows et Mac ». Linux a déjà ses trois formes et NixOS (fiche 48) ;
**le Mac n'a aucune version publiée** : la CI le compile (puce Apple et Intel), rien ne
l'empaquette. Sans compte Apple (aucun argent), un `.dmg` non signé s'ouvre par « clic droit →
Ouvrir », comme PureRef l'a longtemps fait.

* Le mode référence hors de Windows : sans cadre et au premier plan passent par `winit` partout ;
  `Alt` + glisser (`drag_window`) est refusé par certains bureaux Wayland — à vérifier, et à dire
  quand il ne marche pas.
* Le pavé hors de Windows : macOS donne le pincement et le déplacement nativement à `winit` ;
  Linux selon le bureau (libinput sous Wayland, rien de propre sous X11).
* **Fini quand** : la CI verte sur les neuf tâches, un `.dmg` à la release, et un essai du mode
  référence sous Linux et Mac (par lui ou un testeur).

## 4. La boîte noire qui voyage — la télémétrie de tous ses utilisateurs

**Pourquoi maintenant** : deux installeurs téléchargés, zéro paquet Linux, et aucun moyen de savoir
si la bascule a échoué en silence chez quelqu'un. Sans elle, on ne voit que sa machine.

Conçu, pas écrit (fiche 49 § 2) : éteint par défaut, une question claire au premier lancement,
« voir ce qui part » (exactement les lignes de la boîte noire, qui ne peuvent porter aucun mot de
l'utilisateur), des lots envoyés au lancement suivant en HTTPS, un identifiant tiré au hasard et
renouvelable, effacement sur demande, **aucune adresse IP gardée**, une page publique qui dit tout.

* **En ligne le 08/10 (fiches 54 § 6 et 55 § 1)** : le Worker et sa base dans son compte
  Cloudflare, l'adresse dans Glucose. **Le 08/10 au soir, la base est vide** : personne n'a
  encore lancé un Glucose qui la porte et répondu « oui ». Reste : vérifier la première session
  reçue (`npx wrangler d1 execute glucose-boite-noire --remote --command "SELECT …"`, depuis
  `outils/telemetrie/`). **La question sous Android est faite** (QUESTION-1, VUE-1, fiche 56).
* Les plantages hors de Windows : Linux en a deux registres (`systemd-coredump` chez Fedora, `apport`
  chez Ubuntu) ; Android a `ApplicationExitInfo` (11 et plus).

## 5. Ce qui manque de Glucose Tauri

D'abord **remesurer la parité** (le tableau de la fiche 03 date du 10/09). Puis, dans l'ordre de la
fiche 36 § 2 (phase 4) : ce qui est écrit dans le noyau et pas branché (`timeline`, `mirror_graph`,
rideaux, `membrane_stretch` : le cliquet 3 les nomme), naviguer dans l'immense (`Ctrl+F`, la couleur
des domaines), dossiers et miroirs, rideaux et temporalité, storyboard, presets et zones,
l'interface (sélecteur de couleur, infobulles), vidéos et provenance (fiche 27), exports PNG et HTML.

**Le registre de Tauri** (fiche 39) garde ses défauts relevés, à ne pas reproduire. Encore ouverts :
les membranes de Mary (13), les rideaux (14), « recopie image » (15) et la carte « aaaa » (18) à
comprendre avec lui, les niveaux de boards (16) à confirmer, l'optimisation de la Time Machine (12).

## 6. La fluidité

* Les zooms chargés à 11-21 ms (session du 05/10) : le plancher de 100 images par seconde.
* **Sur batterie** (fiche 54 § 5) : le tempo à 7-8 balayages (30 images par seconde) sur 11 % des
  images en mouvement, « effacer » jusqu'à 27 ms. Une session branchée et une débranchée, le
  même geste, pour séparer le bridage de la carte de Glucose ; puis pixeliser plutôt que céder.
* Les gels de la carte Intel Arc (fiche 43 § 8) : très probablement l'écran branché sur la RTX
  (fiche 52 § 1) — à confirmer, puis à mesurer en mode Optimus (l'écran sur l'Intel).
* **La boîte noire ne note ni la carte ni la succession des images** : à ajouter, c'est elle
  qui voyage (fiche 52 § 1).
* Un plantage reproductible **dans les épreuves** : trois fois dans `vulkan-1.dll` au même octet
  (fiche 49 § 4), probablement une épreuve graphique qui referme la carte.
* ~~Le pavé par *Direct Manipulation*~~ : fait (fiche 53), à juger à son écran.
* **Le dépôt hors de Windows demande la page nue** : `ureq` sans sa fonctionnalité `gzip` (qui
  tirerait `flate2`, une note à écrire). Sous Linux, une page d'épingle peut mettre 20 s.
* Puis l'étalonnage : le modèle de coût qui **prédit** ce qu'une image va coûter, appris de ce que la
  boîte noire aura vu sur les vraies machines.

## 7. Plus loin

Le co-working (sur le journal du document) · le MCP réécrit en Rust (`rmcp`) · les plugins selon sa
vision ([PLUGINS.md](../PLUGINS.md)) et l'IA locale · le Mac · la fondation des 10⁷ nœuds (l'arène,
écrite et en attente) · et le but : le canevas de Wikipédia de `glucose-brain`.

---

## Ce qui n'attend que lui

1. **Le clic « Publish release »** sur le brouillon de la 2.0.2-beta.1 (fiche 55 § 1).
1 bis. ~~Une copie de la clé Android hors du PC~~ : faite (08/10 au soir). Garder ou non le
   sous-domaine `ferme-nilslamber` du serveur de la boîte noire.
2. **Une copie de sa clé de signature hors de ce PC.**
3. **Le modèle de l'iPad**, et **les téléphones de ses testeurs Android** (le plus ancien
   d'abord) : un « Xiaomi 9 » — Mi 9 ou Redmi 9 ? — et une clé de signature Android à garder.
4. Le modèle de sa souris : **il en a une** (07/10), mais navigue surtout au pavé tactile.
5. Sait-il qui, parmi ses cinq utilisateurs, a basculé ? Surtout sous Linux.
6. Les membranes de Mary, les rideaux, Trans-domaines, « optimiser » dans la Time Machine.

## Les essais à l'écran jamais faits

Écrits dans les fiches 42, 43 et 47, jamais rapportés. À lui redonner en **une liste courte** à la
fin du chantier 1 (avec le copier-coller entre deux fenêtres et le collage dans Discord) : les flèches qui contournent (et leur coude qu'on glisse), les formules seules sur
leur ligne, la Time Machine ouverte pendant qu'on zoome, `Ctrl+Z` après un glisser ou un
redimensionnement, l'aimant sur un tableau chargé, coller puis `Ctrl+C`/`Ctrl+V` une image, glisser
un PDF depuis une page web.
