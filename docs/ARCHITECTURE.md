# L'architecture de Glucose Rust

> **Revue le 11/11/2026** (fiches 51 à 105). Comment le code est fait **aujourd'hui** : où vit chaque chose, et pourquoi.
> Ce n'est pas un plan : c'est une carte. Pour le *pourquoi* détaillé d'un mécanisme, chaque module
> porte son histoire en tête de fichier, et le code cite les fiches du [carnet](carnet/00-INDEX.md)
> par leur numéro (« fiche 22 § 5 » se lit dans `docs/carnet/22-…`).

---

## 0. Ce que la vision demande au code

> Venu de [VISION](VISION.md) le 11/11/2026 (fiche 104) : ce sont des lectures de ses paroles par des
> sessions, pas ses paroles — elles vivent donc ici.

### Ce que la vision demande au code (la session du 15/10, à remettre en question)

* **L'interface devient une donnée.** Un preset ne peut exister que si les raccourcis, le thème, la
  disposition des panneaux et les comportements se lisent dans **une seule table** chacun, et non
  dans le code de chaque geste. Chaque fonctionnalité écrite avec un raccourci en dur ajoute au
  travail de demain.
* **Les plugins** veulent une frontière nette entre le noyau, l'application et ce qui s'y greffe —
  le noyau sans dépendance l'est déjà.
* **L'échelle** : 10⁷ nœuds navigables (l'arène, écrite, attend son branchement) — et une
  navigation sémantique (domaines, couleur), jamais la dégradation du sens.
* **Le graphe avant le texte** : les flèches, les ancres de texte et les membranes sont déjà la
  grammaire des objectifs 3 et 4.
* **L'ergonomie se fait à la fin** : une grosse session, quand toutes les fonctionnalités seront
  là. Avant, chaque fonction a son bouton, au PC et au téléphone (INVENTAIRE).

### Membrane et domaine : deux structures (la session du 22/10, à remettre en question)

* **Membrane et domaine sont deux structures différentes, et c'est ce qui fait le domaine.** Une
  membrane **contient** : un nœud est dans une seule membrane à la fois, et les membranes
  s'emboîtent — un arbre. Un domaine **traverse** : un nœud peut appartenir à plusieurs domaines, à
  des degrés divers, quelle que soit sa membrane ou son fichier — un recouvrement, pas un arbre. Le
  noyau a déjà cette forme pour un document (`Project.domains`, chaque nœud porte ses domaines avec
  un poids de 0 à 1 : une appartenance graduée) ; il lui manque de **traverser les fichiers**, d'être
  **vu** (filtrer, colorer) et d'être **lu par une IA** (`glucose_lire` ne dit pas les domaines).
  **Depuis la fiche 88** : il traverse les fichiers (chaque fichier garde les siens ; Glucose met côte à
  côte ceux qu'il connaît), et une IA le lit et le pose comme l'humain ; reste qu'on le **voie**.
* **« Lu de la même façon » oblige à une règle** : le sens d'un tableau ne doit jamais tenir
  **seulement** dans la position. Un humain lit la proximité ; une IA reçoit des coordonnées qu'elle
  n'interprète pas. Ce qui compte s'écrit donc aussi dans une structure explicite — une membrane, une
  flèche, un domaine —, que les deux lisent pareil.
* **Les dimensions, comptées honnêtement** : x et y sont deux dimensions continues ; la couleur
  perçue en a trois (clarté, deux axes de teinte), et l'œil ne distingue de façon sûre qu'une
  dizaine de couleurs **catégorielles** (ordre de grandeur des guides de visualisation, à vérifier) —
  au-delà d'une dizaine de domaines, la couleur seule ne suffira plus à les reconnaître ; chaque
  domaine est une dimension à lui (appartient, et à quel degré) : leur nombre n'a pas de borne.
  Ce qui rend Glucose plus exploitable qu'un espace latent n'est pas de le **projeter** en 2D (une
  projection de 4 096 dimensions en perd presque tout), c'est que ses dimensions sont **explicites et
  nommées** — « eau », « contredit » — là où celles d'un modèle sont anonymes.

## 1. Quatre crates, et la règle des dépendances

```
crates/
├── glucose-core/     le noyau : modèle, géométrie, journal, format, texte, index, courbes (Slug) — AUCUNE dépendance
├── glucose-math/     les formules LaTeX en géométrie pure — une dépendance, katex-rs
├── glucose-desktop/  l'application : fenêtre, deux voies de rendu, plateforme, mise à jour
└── glucose-android/  le point d'entrée du téléphone (`android_main`) et la frontière JNI
                      (le partage, le sélecteur de photos) — vide hors d'Android
android/              l'enveloppe Java de l'APK (Gradle, GameActivity, le partage), decisions/08 et 09
```

* **Le démarrage** (`demarrage::lancer`) est commun : `main.rs` (le bureau) et `glucose-android`
  fabriquent chacun leur boucle et connaissent leur dossier, et lui confient tout le reste.

* **Le noyau n'a aucune dépendance**, et se teste sans écran. C'est ce qui lui permet de compiler
  pour Android et le web à chaque envoi.
* **L'application a les siennes**, chacune défendue par une impossibilité de faire sans :
  `winit` (fenêtre), `wgpu` (Vulkan, Metal, Direct3D, GL ES derrière une interface), `tiny-skia`
  (rastériseur de la voie processeur), `softbuffer`, `fontdue` (glyphes — Inter, et Noto Emoji
  en repli, EMOJI-1), `image` (décodeurs), `arboard` (presse-papiers) et `rfd` (les sélecteurs
  de fichiers, ouverts sur un fil à eux — DIAL-2, `persist::choix` —, et la seule boîte posée
  avant la fenêtre, DIAL-5) **hors d'Android seulement** — chacune derrière
  sa porte, `presse_papiers` et `dialogue` (DIAL-4, cliquet 12) —, `minisign-verify`
  (signatures), `png` (l'export en image, écrit par bandes au fil de l'eau, `decisions/12`),
  `pollster`, et `windows` sous Windows, `ureq` + `rustls` ailleurs, `x11rb` sous
  Linux (la sélection de X11, déjà là par `arboard`, `decisions/11`). Une dépendance nouvelle exige une note dans
  [`carnet/decisions/`](carnet/decisions/).

## 2. La vie d'une image

```
événement winit ──► interactions (une intention) ──► Store (une transaction du journal)
                                                         │
          salissure (ce qui a changé) ◄──────────────────┘
                     │
   renderer : culling ──► voie processeur (tiny-skia, tuiles, bandes)  ─┐
                       └► voie graphique (wgpu : photos, cartes, fond…) ─┤
                                                                        ▼
            present : tempo (quand soumettre) ──► l'écran ──► chronique (ce que l'œil a reçu)
```

* **`app/`** : la boucle (`evenements.rs`) ne contient aucune logique, elle traduit et confie.
  `peinture.rs` décide quoi repeindre, `presentation.rs` remet l'image à l'écran, `reveil.rs` dit
  quand se réveiller (**`ControlFlow::Wait`** : sans raison, Glucose dort). **Une image qui n'est
  pas finie demande la suivante** (NETTETE-1, `Raison::Nettete`, fiche 63) : la dernière image
  d'un geste appelle son image d'arrêt, la finesse réduite remonte jusqu'à un, et les textures
  que la cascade a reportées (`SceneGpu::inachevee`, lu par `Presenter::scene_inachevee`) se
  redemandent **sans rien repeindre** (`demander_une_image`). Chacune finit d'elle-même.
* **`interactions/`** : chaque geste a son module. La souris (`mouse.rs`) pose sa question aux
  couches du haut vers le bas ; le clic (`pick.rs`) passe par l'arbitre du noyau (`hit_priority`,
  PICK-2) ; le glisser (`drag.rs`), l'aimant (`aimant.rs`, SNAP-4), le placement aimanté
  (`placement.rs`), le redimensionnement, le recadrage, les coudes, les ancres, les onglets, la
  Time Machine (`temps.rs`), le copier-coller de nœuds (`clipboard/lot.rs` : le lot est un
`.glucose`, le noyau l'extrait et le recolle, `core/store/lot.rs`), le mode référence
(`reference.rs`), les raccourcis (`shortcuts.rs`, chiffres lus sur la touche
  **physique** pour les claviers AZERTY).
* **La navigation** : `pan_zoom.rs` décide si un défilement est un zoom ou un déplacement, et s'il
  vient d'une molette ou d'un doigt ; `elan.rs` tient une **dette**, par deux portes — la souris la montre en entier à l'image
suivante, le doigt la rembourse et glisse (fiche 51 § 1) —  (ce que la main a demandé et
  que l'écran n'a pas montré) remboursée à chaque image ; `vol.rs` suit le chemin de van Wijk et
  Nuij (`F`, signets, dossiers) ; `horloge.rs` avance du temps que l'écran **montre** ; `tempo.rs`
  donne à chaque image un nombre entier de balayages.
* **Le pavé de précision** (`pave.rs`, fiches 53 et 54) : sous Windows, par *Direct Manipulation*
  (`plateforme/pave_windows.rs`, un *viewport* fictif à la taille de la fenêtre) — d'une image à
  l'autre, le système rend une **similitude** `q ↦ r · q + b`, que Glucose applique entière, par
  la porte de la souris ; sans classement ni bascule. Ses signes de vie vont à la boîte noire.
  Un pavé que Windows ne donne pas ainsi, et `Ctrl` + molette, passent toujours par
  `pan_zoom.rs` et `pincement.rs`.
* **Les doigts sur un écran** (`toucher.rs`, fiche 54) : un doigt agit comme la souris, sauf sur
  le vide où il déplace le canevas avec son élan ; deux doigts font leur similitude. Toutes les
  similitudes d'une image — pavé et doigts — se **composent** en une seule (`Toucher::attente`).
  Hors de Windows seulement : Windows y simule aussi la souris du premier doigt. **L'appui
  long** (APPUI-1, fiche 57) : un doigt immobile qui tient le délai du système
  (`plateforme::doigt`, le réglage d'accessibilité d'Android) ouvre le menu du clic droit, ou
  choisit le mot sous le doigt dans un texte ; la boucle se réveille à son échéance.
  **Les gestes à plusieurs doigts** (GESTES-1, `toucher/plusieurs.rs`, fiche 58) : deux doigts
  touchés annulent, trois rétablissent ; l'appui long pris, ce qui suit décide — lever ouvre le
  menu, glisser trace un rectangle, un autre doigt ajoute des nœuds.
* **Le clavier du téléphone** (CLAVIER-1, `text_edit/miroir.rs`, fiche 57) : le clavier du
  système tient le texte qu'il réécrit ; la saisie (un nœud, ou le champ d'une question) et lui
  tiennent le même état, comparé à chaque tour de boucle — ce qu'il réécrit devient une commande
  d'écriture (annulable mot par mot), ce que Glucose change repart chez lui.
* **Le partage vers Glucose** (fiche 56, PARTAGE-1) : sous Android, `MainActivity.java` ouvre les
  fichiers partagés ou choisis dans le sélecteur de photos et confie leurs descripteurs à
  `glucose-android` (JNI), qui lit et remet un `Partage` à `plateforme/partage.rs` : des images
  posées en un geste, ou des adresses au rapatriement — le chemin du dépôt. Une boîte aux
  lettres garde ce qui arrive avant la fenêtre.
* **Le dépôt** (`drop.rs`, `depot_web.rs`) : un lot, un geste, un compte-rendu ; une image
  rapatriée d'une page (`plateforme/rapatrier.rs`) qui ne vient pas laisse un lien qui **dit
  pourquoi** et se rattrape au clic droit (`depot_web/relance.rs`, DEPOT-WEB-6) ; la première
  copie qui arrive se pose tout de suite, et l'original prend sa place (`depot_web/apercu.rs`).
* **Tracer un rectangle** (TRACE-1, `interactions/trace.rs`, fiche 65) : sous l'outil Membrane
  ou Dossier et sous la grille armée du Storyboard, l'appui ne pose rien ; un clic (la main
  dans le tremblement d'un double-clic) pose comme avant, un glisser trace la taille — la loi
  de tldraw. La poignée du storyboard est un tracé parti du coin de sa première case. **Le
  fantôme part toujours du curseur** (FANTOME-1, fiche 66) : au survol comme au clic, le coin
  haut gauche de la première case — ou de la membrane, du dossier — est sous la main, là d'où
  un glisser tracerait ; Tauri centrait la grille, et le fantôme sautait d'une demi-grille dès
  que la main glissait.
* **L'animatique** (`interactions/lecture.rs`, `renderer/lecture.rs`, fiche 67) : « ▶ Lire »
  **cadre le canevas lui-même** sur une case après l'autre — aucun second moteur, les images à
  leur netteté par les deux voies —, voile de noir ce qui n'est pas elle, pose ses textes en
  sous-titre, et **coupe** à chaque durée ; entre deux coupes, la boucle dort
  (`Raison::Animation`). La lecture prend toute la main, comme une question. L'interface se
  cache par **un seul prédicat**, `UiState::sans_interface()` (le mode référence, ou une
  lecture), que la bande, les panneaux, les clics et le dessin lisent.
* **Le canevas défile au bord** (DEFILE-1, `bord.rs`, fiche 59) : un nœud, un coin ou un
  rectangle de sélection tenu dans la bande du bord — la cible d'un doigt — fait avancer la vue
  à chaque image (`bouger_la_camera`, une raison de réveil à lui), d'autant plus vite qu'il s'y
  enfonce ; et **ce que la main tient suit la vue qui bouge**, d'où que vienne le mouvement
  (`les_gestes_suivent_la_camera`). Sortir de la fenêtre fait toujours partir le glisser.
* **Le copier unique** (COPIER-1, `clipboard/formes.rs`, fiches 59 et 61) : `Ctrl+C` et le
  glisser rassemblent la sélection sous toutes ses formes — le lot, le texte, l'image seule en PNG
  et en pixels, chaque image en fichier (octets d'origine) —, et **une seule règle** dit lesquelles
  partent (`Formes::offertes` : une image seule en image, jamais aussi en fichier) ; chaque
  système la dit dans ses formats (`selection_windows`, `selection_x11`). Un lot collé se lit
  **sur le fil du collage** (`LotALire`) : sous X11, celui d'un autre Glucose vient par le
  serveur.
* **Transformer la sélection entière** (`resize.rs`, fiche 53 § 10) : le noyau calcule
  (`core/groupe/` — mise à l'échelle par un coin, rotation, origine commune ou individuelle) et le
  magasin réécrit tout depuis la pose de départ (`core/store/groupe.rs`) ; le cadre du groupe se
  garde sur le geste en cours (GROUPE-1).

## 3. Le document

| | où | ce qu'il fait |
|---|---|---|
| **Le modèle** | `core/types/` | projet → tableaux → images, annotations (carte, pense-bête, flèche, membrane), dossiers ; coordonnées monde en `f64`, origine haut-gauche |
| **Le magasin** | `core/store/` | la seule porte d'écriture. Chaque modification est une **transaction du journal** (`journal.rs`), inversible : l'annulation coûte la taille du changement, jamais celle du document. Un geste = une entrée (`undo.rs`) ; un glisser s'écrit en **une** translation (GLISSER-1), un redimensionnement en une édition (FONDRE-1). La navigation et la sélection n'entrent jamais dans l'annulation |
| **Le fichier** | `core/persist/` | `.glucose` : une **base** (conteneur versionné, sections avec somme de contrôle) puis une **histoire en ajout seul** (`histoire.rs`) — objets (octets d'image par empreinte SHA-256), gestes, instantanés, jalons, vues — chaque entrée chaînée à la précédente : une fin déchirée s'arrête à la dernière entrée saine |
| **Le scribe** | `desktop/persist/scribe.rs` | un fil qui ajoute les gestes au fichier et synchronise le disque ; la cadence suit le disque. `Ctrl+S` pose un **jalon**. Un document sans nom vit dans un **brouillon** |
| **Les documents du téléphone** | `persist/documents.rs`, `documents/gestion.rs` | DOCUMENTS-1 : sans dialogues, un travail sans nom qu'on quitte se range sous « Canevas N » dans `documents/` ; « Ouvrir un document… » les liste. DOCUMENTS-2 : un appui long dans la liste renomme (par « Enregistrer sous » pour le document ouvert), duplique, supprime |
| **Les filets** | `persist/frappe.rs`, `reprise.rs`, `recuperation.rs`, `verrou.rs`, `atomic.rs` | le texte en cours de frappe survit à un plantage ; le lancement rouvre le dernier travail ; ce qu'on va recouvrir est mis de côté (FIN-1) ; un seul scribe par fichier ; **un seul endroit** pose un fichier à la place d'un autre (cliquet 11) |
| **Les gabarits et leurs zones** | `core/gabarits.rs`, `core/store/zones.rs`, `core/layout/zones.rs` | PRESET-1 (fiche 62) : les six gabarits de Tauri ; une zone porte `gabarit/catégorie` ; une image appartient à la zone où est son centre — rien ne s'écrit sur elle ; « Par mood board » range chaque zone en rangées justifiées, qui tiennent dans la zone |
| **Le storyboard** | `core/types/storyboard.rs`, `core/store/storyboard.rs` | STORYBOARD-1 et 2 (fiches 64, 65) : **une suite** de cases et **sa grille, enregistrée dans le tableau** (`Board::storyboard`, schéma 3, `Edit::Storyboard`) ; **une seule loi** place une case (`Grille::coin` : la case `i` à la colonne `i mod c`) ; chaque geste — poser d'un clic, tracer un rectangle (les colonnes et rangées dont l'étendue est la plus proche, `tient_dans`), tirer la poignée, ajouter, retirer, lâcher une case à une autre place — donne une `Disposition`, et `Store::ranger` seul en tire chaque case, numérotée dans l'ordre, et ses images, qui suivent par la similitude de l'ancien cadre au nouveau ; une case qui porte quelque chose ne se retire jamais d'un tracé. Une image lâchée dans une case s'y ajuste ; une case glissée emporte ses images (GLISSER-1, refermé avant le rangement). **Schéma 4** (fiche 67, `persist/storyboard.rs`) : une case porte sa durée, son plan et le texte de chaque champ ; le storyboard, la durée d'une case qui n'a pas la sienne (`Storyboard::duree`, la seule loi) et **le nom de ses champs** — une donnée, comme les légendes de Toon Boom. Chaque réglage est un geste (`store/storyboard/case.rs`) ; c'est le journal qui jette ce qui ne change rien. **Schéma 5** (fiche 68) : un tableau porte **plusieurs storyboards** (`Board::storyboards`), une séquence chacun, et **chaque case nomme le sien** — la liste des cases reste plate, comme l'arène ; **SEQUENCE-1** : toute case nomme un storyboard de son tableau, tout storyboard a au moins une case — ramené à la lecture par une seule règle (`types::rattacher` : un document d'avant, ou de Tauri, met son unique storyboard sous `STORYBOARD_D_AVANT`), tenu par chaque geste. Ranger un storyboard ne touche que ses places dans la liste (`store/storyboard/ranger.rs`, `remplacer_la_suite`) ; au journal, un storyboard est une case de la liste (`Edit::Storyboard`, un `Slot`) — l'ancien « avant / après » des schémas 3 et 4 en est la case de rang 0 |
| **L'arène** | `core/arena/` | la fondation des 10⁷ nœuds, en attente de branchement : un tronc de champs contigus (22 octets par nœud) et des **tables éparses** pour ce qui est propre à un genre ; `NodeId` est un indice. **Une seule liste de ses tables**, `Doc::tables()`, tenue par une déstructuration exhaustive — une table oubliée ne compile pas —, que le poids (`Doc::bytes`) et la vérification (DOC-1) lisent tous deux (fiche 67 : la liste écrite deux fois avait perdu `crop`) |
| **Les domaines** | `core/store/domains.rs`, `core/store/domains/nom.rs`, `core/recensement.rs`, `desktop/recensement.rs`, `desktop/interactions/domains/connus.rs` | fiche 88 : un sujet qui traverse les nœuds, les membranes et les fichiers. **Pas de catalogue central** : chaque fichier garde les domaines qu'il porte. **DOM-4**, un nom par domaine (sa règle) ; **DOM-5**, l'identité née une fois (`eau@<instant>`), le nom qui change, la date la plus récente qui l'emporte. Le **recensement** lit, sur un fil à lui et sans jamais écrire, les documents connus (`documents-connus.txt`) et leurs voisins de dossier ; jamais le fichier ouvert (RECENSE-2) ; la vue (`Recensement::vue`) est pure, et deux créations du même nom y restent deux homonymes (RECENSE-1). **DOM-6** (fiche 94, `store/agents/decrit.rs`) : un nœud hérite à la lecture les domaines de ses membranes, la plus proche d'abord, son propre poids l'emportant ; rien ne s'écrit |
| **Les membranes en carte** | `core/membrane_forme/regions.rs` (`Regions`), `core/membrane_forme.rs` (`couvre`, `touche`), `desktop/renderer/scene/membranes.rs`, `membranes/champs.rs` | REGION-1 (fiche 105) : la couleur en un point est celle de **la plus petite région qui le contient**. Chaque membrane donne son **sol** — ses halos, puis son corps **opaque**, du ton qu'elle aurait seule sur le canevas (la force des régions : le voile de Tauri posé deux fois, un nombre de ressenti) — et sa **frontière** ; les régions se posent de la plus grande à la plus petite (la surface, non le modèle). Les deux voies et l'export posent le sol, puis la grille de points et les zones, puis les frontières. Un corps qui couvre l'écran cache ce qui le précède : rien de cela ne se pose (au fond de « Eau », l'attente de la carte graphique de 9,7 à 0,1 ms). NEUTRE-1 : sans couleur choisie, une membrane est grise, de la clarté du bleu de Tauri |
| **Le titre des membranes** | `core/membrane_titre.rs`, `desktop/renderer/scene/membranes.rs` (`MembraneVue::titre`), `desktop/typography/glyph.rs` (`halo`) | TITRE-1 (fiche 95) : chaque mesure du titre × `√(A/A₀)`, jamais moins que 1 ; aucun ne se cache ; ramené au plus grand qui tient dans sa largeur ; **l'encre** (`Annotation::encre`, la boîte et la bande du titre) pour le culling, le cadrage, l'export, le vol, la sélection et le glisser — la boîte pour le reste ; la réserve des colonnes (`surcroit`). TITRE-2 : un texte détouré en deux passes, le halo des huit copies gardé dans le cache des lettres ; une lettre tracée, un trait puis son remplissage. **TITRE-3** (fiche 108, `desktop/renderer/composants/titre.rs`) : sur la voie graphique, le titre est un **composant** — rendu une fois par le dessin même du processeur, posé à chaque image juste après les frontières, sous les photos ; sa texture contient toute son encre (`Typography::encre_detouree`) ; son ajustement à la largeur se mesure **dans le monde** (`MembraneVue::titre_du_monde`), sa clé ne change donc pas pendant un zoom ; une seule règle (`titre_par_la_carte`, de la vue et de la fenêtre seules) laisse au processeur le titre qu'on écrit et celui dont la texture dépasserait la fenêtre ; au processeur, tous les titres, puis toutes les poignées et réglettes |
| **Le gris de loin** | `desktop/typography/encre.rs` (`LETTRE_LISIBLE`, `forme_grise`, `peindre_exact`, `griser`), `desktop/renderer/card/gris.rs` (`formes_de_loin`), `desktop/renderer/vectoriel.rs` (`a_des_pixels`) | GRIS-1 et GRIS-2 (fiche 96) : une lettre de moins de 2,8 pixels montre le gris de son encre — le rectangle de sa ligne à l'opacité de son encre sur sa place, à la couverture exacte ; dans la typographie du canevas pour tout texte, ligne par ligne pour un nœud (ornements compris) ; une géométrie, deux peintres (le canevas, la carte graphique) ; le SVG garde ses lettres ; le seuil de zoom des nœuds s'en déduit |
| **Disposer en colonnes** | `core/layout/colonnes.rs`, `core/store/colonnes.rs`, `desktop/interactions/colonnes.rs` | COLONNES-1 (fiche 92) : une membrane range ce qu'elle porte en colonnes de journal, de bas en haut, dans l'ordre de lecture ; deux bissections (la plus basse hauteur pour k colonnes ; le k qui approche la proportion de l'écran), l'optimum exact ; l'appartenance écrite ne change pas ; un seul geste |
| **Les couleurs** | `core/couleur.rs`, `core/store/annotations.rs` (`colorer`), `desktop/ui/couleurs.rs` | Oklab et Oklch, les trois tris par la couleur (ORDONNER-2) ; la couleur d'une image est le sommet de sa pyramide (MOYENNE-1). **COULEUR-1** (fiche 98) : ce que « sa couleur » veut dire, le noyau seul le lit et l'écrit (`Annotation::couleur_propre`, `Store::couleurs_choisies`, `colorer_la_selection`) — le papier d'un post-it, la teinte d'une membrane, d'un texte, d'un dossier ; un opérateur et une flèche ne se colorent pas ; la barre de couleur et les commandes `couleur.*` y passent. **Fiche 99** : `core/couleur/ecran.rs` — le retour Oklab → sRGB, le bord exact du gamut (`Oklch::chroma_maximale`, par bissection) ; **PAPIER-1** (fiche 105, qui défait COULEUR-3) : le papier d'un texte est le gris sombre et neutre des blocs, partout (`Theme::card_paper`, `renderer/card/habit.rs`, `papier`) — la couleur choisie d'un texte va à sa lueur (VISION § 5, COULEUR-4 : `SensDesTextes::lueur`) ; **ROUE-1**, `core/couleur/roue.rs` (la place teinte × saturation × clarté, le disque en pixels), `ui/roue.rs` (mise en page pure, dessin), `interactions/roue.rs` (les gestes ; un geste du journal, repris à chaque mouvement par `cancel_live_edit`) |
| **Le sens des textes** | `core/sens.rs` (`embarque`, `textes_du_tableau`), `core/sens/familles.rs`, `core/donnees/`, `desktop/parente.rs`, `renderer/sens.rs`, `renderer/halo.rs` | **SENS-1** (fiche 106) : une IA du sens embarquée — `static-similarity-mrl-multilingual-v1`, 128 dimensions en octets, 15 Mo, lue sans dépendance (le découpeur WordPiece réécrit à l'identique de la référence ; `decisions/15`). **PARENTE-1** : le hasard d'un document par la médiane et l'écart absolu médian ; les familles par liaison moyenne coupée à deux écarts au-dessus ; les teintes par sériation circulaire, l'origine fixée par le premier texte du document ; la sûreté d'un texte, la chance de son plus proche parent corrigée du nombre de comparaisons (Šidák), puis le taux local de fausses découvertes (Efron, densité de Grenander). **PARENTE-2** : un fil la calcule quand les **textes** changent (une empreinte ; déplacer ne relance rien), dans la règle de la mémoire (ADAPT-1) ; sans fil (épreuves, bancs), sur place, avant l'image (`poursuivre_les_attentes`) ; le rendu la peint dans la grammaire des membranes (`SensDesTextes::lueur` — la clarté du gris neutre, la vivacité du cercle des teintes à la mesure de la sûreté), la lueur d'un bloc (38) montant jusqu'à 76 avec elle ; une flèche prend la couleur des lueurs qu'elle relie. **PARENTE-3** : `glucose_lire` dit la famille et la sûreté de chaque texte du tableau que l'IA a lu. **PARENTE-4** (fiches 107-108, `core/sens/liens.rs`, `desktop/parente/liens.rs`, `renderer/arrow/presence.rs`) : la sûreté d'une flèche — que ses deux bouts parlent de la même chose, une membrane par ce qu'elle porte et son nom, le hasard de la cible lu hors d'elle ; la souris ne rapproche que les liens de sens (`max(d, proximité × s)`), la densité ne change pas ; **seules se jugent les flèches qu'un agent ou un script a posées** (`Journal::des_agents`, MAIN-1). La teinte symbiotique de Tauri est partie |
| **La Time Machine, socle de l'IA** | `core/persist/histoire.rs` (`nature`, `Auteur`, `Note`), `histoire/ouvrir.rs`, `core/store/journal.rs` (`Main`, `Ecrit`, `suivre_les_mains`), `journal/en_mots.rs`, `desktop/persist/ecriture.rs` (`signer`, `noter`), `agents/executer/temps.rs`, `dock/temps.rs` | Fiche 108, sa parole du 12/11 (*« on UTILISE la Time Machine […] tout le système IA se basera là-dessus »*). **PERSIST-5** : une nature d'entrée à bit haut est facultative — une version qui ne la connaît pas l'enjambe (les morceaux ancillaires du PNG). **MAIN-1** : chaque transaction de la file porte sa main (l'humain, ou un agent sous le nom que son client se donne ; la boucle des agents la pose et la rend) ; le scribe signe chaque geste et déclare un agent une fois par document (AUTEUR) ; le journal suit les nœuds qu'un agent a posés et que l'humain n'a pas touchés (`des_agents`), relus dans l'histoire à l'ouverture quand un agent y est déclaré. **NOTE-1** : un écrit s'attache à un geste par son rang, sans le toucher (NOTE, comme une note de git) ; `glucose_noter` l'écrit, `glucose_histoire` relit l'histoire — chaque geste, son auteur, ce qu'il a fait en mots (`Transaction::en_mots`), ses écrits ; la Time Machine montre en aperçu qui a fait le geste regardé et ses écrits. **RECTIF-1** (fiche 110, `desktop/parente/rectifications.rs`) : le journal des rectifications — quand l'humain colore un texte que l'IA rangeait dans une famille, retouche, retire ou suit une flèche d'agent qu'elle jugeait, l'IA de Glucose (`Auteur::de_glucose`, un identifiant qu'aucun nom d'agent ne prend) attache un écrit au geste (ou au dernier geste, pour un suivi), avec ses **sujets** (champ de queue de NOTE) : ce qu'elle pensait, sa sûreté, le texte qui ressemble le plus et **les mots qui les rapprochent** (B3, `core/sens/pourquoi.rs` : la ressemblance d'un modèle statique se découpe exactement, mot par mot). Aucun seuil ; suivre ne change pas le jugement (ce n'est pas un geste). Un auteur se dit par une seule loi (`Auteur::en_mots`) |
| **Les liens et leur fenêtre** | `desktop/renderer/scale.rs` (`lecture`), `renderer/arrow.rs` (`en_lumiere`), `interactions/survol.rs`, `interactions/suivre.rs`, `core/amener.rs`, `core/store/fenetre.rs`, `interactions/fenetre.rs` | Fiche 102. **LIEN-1** : plus loin que la taille réelle, un lien rétrécit avec le monde et pâlit d'autant (un facteur, `WorldScale::lecture`, sur ses longueurs d'écran et son opacité — les pastilles restent du monde) ; en lumière (choisi, survolé, dans la fenêtre), plein. **LIEN-2** : `Ctrl`+clic vole jusqu'au bout le plus loin. **LIEN-3** : la fenêtre des liens est un **tableau de copies à côté du projet**, que `Store::active_board` rend tant qu'elle est ouverte — aucune écriture (elles nomment un tableau du projet) ni aucun enregistrement (la vue s'écrit depuis le projet) ne l'atteint ; ses copies ont des identités à elles (`original` les rend) ; la disposition de Bring & Go, exacte (`amener::disposer`, un point d'accroche par voisin) ; la main y est passagère : toute commande qui ne la regarde pas la ferme d'abord (`executer`) |
| **L'histoire de la vue** | `core/store/retour.rs`, `desktop/interactions/retour.rs`, `interactions/vol.rs` (`parti`) | RETOUR-1 (fiche 99) : un départ du vol note le lieu quitté, l'image suivante le range dans le magasin ; `Alt+Z` le rend, instantanément, sans se noter ; un document chargé l'oublie ; hors de l'annulation du document |
| **Exporter** | `desktop/persist/export.rs`, `export/png.rs`, `export/svg.rs`, `renderer/export.rs` | **Le SVG** (P5, fiches 69 à 71) : ce que le canevas montre, dans le cadre de l'export PNG, dans **l'ordre même du canevas** (`Renderer::rendre_en_svg` : sous les photos, les photos, toutes les lueurs avant toutes les cartes, les annotations, les réglettes des photos) ; **le texte en tracés** (`renderer/svg/glyphes.rs` : chaque glyphe défini une fois, posé par un `<use>`, les mots dans un `aria-label` — aucune police demandée au lecteur, `decisions/13`), les formules par les fontes de KaTeX (`renderer/math/svg.rs`), les photos par leurs octets d'origine, chaque flou borné à sa portée ; **éprouvé par superposition** à l'export PNG, rendu par `resvg` (l'oracle, dans les épreuves seulement). Markdown, et PNG par bandes de la surface d'un écran, sur un fil à soi (EXPORT-PNG-1) ; ce qui est exporté est une copie (`Store::copie_a_exporter` — la sélection seule, sans les cases ni les zones). **Les planches d'un storyboard** (P2.7, fiche 68, `export/planches.rs`) : chaque planche est un rectangle du canevas (`Disposition::planches`, ses rangées déduites du format A, √2), rendu par le même écrivain (`rendre_en_png`, qui écrit dans n'importe quelle sortie), d'une copie qui ne garde que ce storyboard (`Store::copie_du_storyboard`) ; en images, ou en **PDF écrit à la main** (`export/pdf.rs` : les blocs `IDAT` posés tels quels sous le prédicteur PNG — rendu au pixel près par MuPDF et PDFium) |
| **Glucose Tauri** | `core/persist/tauri.rs`, `desktop/persist/import.rs` | un lecteur d'Automerge écrit ici, sans dépendance, identique à la bibliothèque de référence ; le fichier Tauri n'est jamais réécrit |

## 4. Le rendu : deux voies, un socle

**Le socle commun** décide *quoi* dessiner : le culling par l'index spatial (`core/quadtree`, et
GESTE-1 pour ce qu'un geste en cours a touché), les niveaux dyadiques, la perception (`perception.rs` :
ce qu'on a le droit d'abîmer en mouvement), le cadrage (`renderer/cadrage.rs`).

**Deux exécutants vraiment distincts** décident *comment* :

* **La voie processeur** (`renderer/`, tiny-skia et nos propres primitives) : un cache de **tuiles**
  ancrées au monde et indexées par l'empreinte de leur contenu (`tuiles.rs`, `grille.rs`),
  composées en **bandes** sur tous les cœurs (`fils.rs`) ; les lueurs, les membranes et les flèches
  par la loi exacte du noyau.
* **La voie graphique** (`present/`, wgpu) : les photos sont des textures (`scene_gpu.rs`), les
  cartes de texte et les titres des membranes des **composants** rendus une fois puis posés
  (`renderer/composants.rs`, TITRE-3) ; le fond,
  les lueurs, les membranes, les flèches et le liseré de la Time Machine sont des nuanceurs qui
  évaluent la même loi que le processeur. Autour des photos, deux couches (`couches.rs`) qui ne
  s'effacent et ne partent que par les **bandes** réellement écrites (`bandes.rs`), envoyées par des
  tampons qui restent (`envoi.rs`). Ces passes de la carte naissent, se préparent et se rangent
  dans les cinq temps **en un seul endroit** (`present/passes.rs`), que la présentation et le banc
  des épreuves emploient tous deux : une passe oubliée ne peut plus se cacher derrière une copie.
* **La chrome est posée par la carte** (CHROME-1, fiche 64) : la bande, le fil d'Ariane, le fond
  de la minimap et le rail sont des **textures** tenues par leurs caches, que la carte ne reçoit
  qu'à leur changement (`ui/chrome.rs` : `parcourir` dit l'ordre une fois, la voie processeur
  compose chaque morceau, `ChromeConfiee` les confie, et leurs pixels se lisent sans copie, par
  `Rc`) ; le rectangle de la vue sur la minimap est un **nombre** — quatre filets et un voile que
  la carte pose comme les zones. La couche du dessus ne porte plus que les ornements de la scène :
  sans rien de choisi, elle est vide (297 lignes de moins par image au PC, 556 à 1 865 au
  téléphone). Ce qui attend une décision passe toujours dans son tampon, au-dessus de la chrome,
  et ne se peint que s'il y a quelque chose à décider (`UiState::attend_une_decision`).
* **De près, la carte dessine depuis les formes** (P8, fiche 73) : un composant **plus grand que
  l'écran** ne se découpe plus en carrés (DE-PRES-1 retiré) — la carte le dessine depuis sa
  description vectorielle, la même que l'export SVG (`renderer/vectoriel.rs`, le trait
  `Vectoriel` ; `card/vecteurs.rs`), relevée en primitives (`vectoriel/releve.rs`) :
  `present/formes_gpu` pose des **arrondis** par la loi des membranes (`present/arrondi.wgsl`,
  partagé avec elles) et des **courbes** par **Slug** — la loi dans `glucose-core::courbes`
  (`couverture`, transcrite par `formes.wgsl`), ses contours en quadratiques (CUBIQUE-1), ses
  bandes (BANDES-1, BANDES-2), un atlas qui se resserre. Chaque suite de formes se pose à son rang
  parmi les textures (`couches::poser_une_tranche`). La texture du nœud se **prépare** dans le
  temps qui reste, sans se poser, pour le dézoom (`Confie::a_preparer`). Une ligne qui ne peut
  pas toucher l'écran ne se parcourt pas (LIGNE-1, `Vectoriel::voit_la_bande`).
* **Le zoom réutilise le repos** (REPOS-4, fiche 109, `renderer/composants/passage.rs`) : tant que
  la vue reste à **une demi-octave** de l'échelle du dernier repos — la garantie même du palier —,
  chaque texture se compose dans la vue du repos (même échelle, même phase, donc même clé), et la
  carte la pose par la transformation exacte du repos à l'écran (`écran = k × point + d`) ; la
  translation entière de la fiche 100 en est le cas exact (`Regard::repos`).
* **La cascade** (`present/scene_gpu/cascade.rs`) rend les textures qui manquent, en deux tours
  dont l'ordre est la priorité — ce que la carte n'a pas du tout, puis ce qui a vieilli —, au
  moins une par image, chacune **prévue avant d'être entamée** : un prix fixe plus un prix par
  pixel, la droite des moindres carrés cumulés (CASCADE-4, Welford). Son budget vient de
  `Cadence::tranche_de_fond`.
* **Les mises en page des nœuds de texte sont gardées** (`card/memoire.rs`, `MisesEnPage`) : elles
  ne dépendent pas du zoom (CARD-1). Possédées par le `Renderer`, passées par le `PaintKit` et la
  `Pass`, oubliées au début de chaque image si la précédente ne les a pas demandées — la loi des
  textures et de l'atlas, sans capacité choisie.
* **La garantie** : les deux voies rendent la même scène, au bit près ou à un écart **mesuré et
  borné** (`tests/voies_suite.rs`). L'adaptation change le chemin, jamais le résultat. De près,
  SLUG-1 : elles ne diffèrent qu'au bord des formes (le quart de pixel et l'aire au processeur,
  la place exacte et deux rayons sur la carte), et les lettres sont à la même place à un
  huitième de pixel près (la borne du quart de pixel). Une forme qui n'a pas de raison de
  différer a **une seule loi**, que les deux voies appellent : le cadre d'un passage est une
  membrane (CADRE-1), peinte par `membrane_forme::peindre` comme la brume.
* **La surface** (`renderer/surface.rs`, fiche 70) : un dessin qui n'emploie que remplir un
  tracé, tracer un contour, remplir un rectangle net et écrire est **écrit une fois**, générique
  sur `Surface` — `Pixels` (le canevas) ou `Vecteurs` (le SVG d'un export) —, la forme de Cairo
  et de Skia : les deux reçoivent le même `tiny_skia::Path`. Y sont passés le dossier, la réglette
  des domaines, les textes des zones et des cases ; le texte riche, les post-it, les flèches,
  les badges, les photos et les membranes gardent encore une « sœur » vectorielle à côté de leur
  dessin (fiche 69).
* **La surface qu'Android reprend** (VIE-1, `app/vie.rs`, `present/gpu/vie.rs`) : au
  `Suspended`, la présentation lâche sa surface et garde son périphérique et ses textures ; au
  `Resumed`, une surface neuve se crée sur le même. « En arrière-plan » se lit sur elle
  (`a_sa_surface`) : rien ne se dessine entre les deux.
* **Les moteurs de la carte, dans l'ordre** (FENETRES-1, `present/moteurs.rs`, fiche 82) : Vulkan,
  Metal, Direct3D 12 ; OpenGL seulement si aucun n'offre de carte — son moteur WGL pose, sous
  Windows, une fenêtre cachée qui ne lit jamais ses messages, et bloquait le Restart Manager et
  l'extinction. La présentation (`gpu/ouverture.rs`, `par_le_premier_moteur`) et le banc
  l'appellent tous deux.
* **La carte qui tient l'écran** (`plateforme/ecran.rs`, ECRAN-1) : sous Windows, Glucose dessine
  sur la carte dont une sortie porte le moniteur de la fenêtre — dessiner ailleurs fait recopier
  chaque image par le compositeur. L'ordre : `GLUCOSE_CARTE` › l'écran › le souvenir › l'économe.
* **Les zones des gabarits** : le socle en dresse une seule liste de rectangles
  (`renderer/zone.rs`) ; le processeur les remplit, la carte graphique les pose
  (`present/zones_gpu.rs`). **L'ordre du sol** (REGION-1, fiche 105), le même sur les deux voies : le
  fond, le sol des régions, la grille de points (`fond_gpu` en deux tracés : le fond, puis les points
  composés sur ce qui est dessous), les zones et les cases, les frontières des régions.
* **L'arbitre** (`present/arbitre.rs`) ne s'installe que quand l'écran n'a pas tranché : il
  choisit la carte en la regardant travailler, et retient son verdict pour le lancement suivant
  (`carte.txt`) : changer de carte en cours de route figeait l'affichage (ARBITRE-4).

## 5. Les images et la mémoire

* **L'atelier** (`renderer/atelier.rs`) décode hors de l'image, sur des ouvriers qui **cèdent** au
  fil qui dessine — et à Blender ou Adobe (CEDER-1). **Un décodage n'entre qu'admis**
  (MEMOIRE-3, fiche 66, `atelier/admission.rs`) : son pic, lu dans l'en-tête avant de décoder,
  plus ceux des décodages en vol, tient dans la moitié de ce que la machine laisse maintenant —
  la porte est celle du processus, et ne s'ouvre que par `pendant(octets, travail)`. Il **décode
  dans la page du niveau natif** (`photo/decodage.rs`), étalée en RGBA de la fin vers le début :
  le pic d'une photo est sa pyramide, et rien d'autre (445 → 254 Mo pour 50 Mpx). **Ce qui rend
  la mémoire passe devant ce qui en prend** (MEMOIRE-4, fiche 83) : les files se lisent
  promesses, reprises, offres et écritures d'aperçus, puis décodages — derrière six mille
  décodages, l'offre attendait, et le téléphone mourait ; un niveau « en chemin » se compte
  (`Magasin::octets_en_memoire`).
* **Le magasin** (`renderer/magasin.rs`, `photo.rs`) garde pour chaque image une pyramide de
  réductions par deux. Chaque niveau est **tenu**, **offert** au système (il le reprend s'il en a
  besoin, sans rien écrire sur le disque), en chemin ou perdu (ETAGES-1).
* **La carte graphique** reçoit le niveau qui couvre la taille posée ; son budget se lit chez le
  système et se suit même quand Glucose dort (ETAGES-2, 3) ; et **jamais un niveau plus grand que
  la plus grande texture qu'elle accepte** (PLAFOND-1, fiche 59) — la présentation dit ce plafond
  au rendu avant chaque image (`EtatDeLaCarte`), et le niveau se réduit jusqu'à tenir.
* **Le disque** : la vue d'ensemble de chaque image (`renderer/apercu.rs`), nommée par l'empreinte
  de ses octets ; un document s'ouvre déjà montré.
* **Quand le système demande de la mémoire** (MEMOIRE-2, `app/memoire.rs`) : sous Android,
  `onTrimMemory` arrive sans son niveau (`memory_warning`) ; au premier plan, la carte rend ce que
  l'écran ne pose pas ; derrière une autre application, elle rend tout, et le magasin offre au
  système chacun de ses niveaux (`Magasin::offrir_tout`). Depuis Android 14, une application au
  premier plan ne reçoit plus aucun niveau de pression : c'est l'admission et la borne du
  magasin qui la tiennent.

## 6. L'interface

**L'ordre des couches, écrit une fois** (DECISION-1, `renderer/decision.rs`, fiche 59) : le
canevas, la chrome permanente (la bande, le fil d'Ariane, la minimap, le rail — dans l'ordre de
`ui::chrome::parcourir`, CHROME-1), les panneaux,
puis **ce qui attend une décision** (`ui/decision.rs` : la barre d'action, les options de flèche,
la fenêtre des ancres, le toast, le menu, la question) — l'ordre même où le clic descend. Les
deux peintres le posent après les panneaux ; sur la voie graphique, avec un panneau ouvert, dans
un tampon que la carte pose après eux (`app/decision.rs`).

`ui/` : la bande du haut, rendue une fois par changement ; ses boutons placés par une seule mise en
page que le dessin et le clic lisent (loi L4) ; **le rail** (`rail.rs`, fiche 55), la même liste de
boutons reposée en grille sur le côté quand la barre ne tient plus, décidé avant la scène,
colonne après colonne, chaque icône avec son nom quand la largeur le permet (fiche 58) ; onglets, minimap, menu contextuel, barre d'action,
options de flèche, éditeur du texte lié, **un seul** toast (coupé en lignes à la largeur de
l'écran) ; **la question** (`question.rs`, QUESTION-1, POPUP-1), que Glucose dessine
**partout** — aucune boîte du système ne tient plus sa boucle — et dont la réponse déclenche une
**suite** (`interactions/question.rs` ; la fermeture en a une, `persist::close::Apres`) : elle
répond au relâchement, sur la réponse pressée, et au clavier (Entrée, Tab, Échap), prend tout
ce qui la touche, et peut porter un **champ** de texte (renommer un document) ; le menu contextuel **à la taille du doigt** quand deux touchers l'ouvrent. **Le menu porte chaque commande** (ACCES-1, fiche 63) — ce qu'il propose dans `ui/context_menu/entrees.rs`, sa pose dans `context_menu.rs`, **en colonnes** quand l'écran est trop bas (un groupe reste entier) ; **ce qui s'applique, une seule loi le dit** (APPLIQUE-1, fiche 113, `commandes/applique.rs`) : la palette n'offre, le menu ne montre et l'exécution ne fait que ce qu'elle dit — chaque commande rangée sans joker (toujours, selon le document, selon le choix), absente plutôt que grisée ; les signets vivent dans `interactions/signets.rs` (le clavier et la question « Signets… » appellent les mêmes gestes). `dock/` : les panneaux (Ordonner,
Timer, Domaines, Time Machine, **Mood board** (les gabarits de zones ; l'ancien « Presets », fiche 71), **Storyboard** — sa liste de cases par pages, fiche 64 ;
Plugins est encore une façade honnête). **Le panneau du storyboard est un miroir** du document
(`StoryboardState::suivre`, fiche 67), recopié à chaque image : son dessin, son clic et ses
champs lisent ce miroir et lui seul (L4) ; une durée qu'on tient reste à la main jusqu'au lâcher
(un glisser, un `Ctrl+Z`), et après avoir écrit, le panneau **relit aussitôt la source** — un
`Ctrl+Z` avant l'image suivante ne laisse pas la valeur défaite dans le champ. Les durées se
règlent **en rapport** (une règle logarithmique, deux chiffres significatifs) ;
chacun avec son cache ; `dock/clic.rs` aiguille le clic, `dock/champ.rs` porte les **champs de
nombre** (CHAMP-1 : glisser, ou cliquer et taper — tout le panneau est la règle). Les zones des
gabarits se prennent par leur bandeau (`interactions/zone.rs`), un gabarit armé suit le curseur
(`interactions/gabarit.rs`) ; **les cases du storyboard** se prennent par leur bande, une grille
armée suit le curseur (`interactions/storyboard.rs`) — leurs rectangles au premier rang avec les
zones (`renderer/storyboard.rs`, `premier_rang`). **Le storyboard courant** (fiche 68, `interactions::storyboard::storyboard_courant`) : le dernier touché — choisir une case le règle — ou posé, sinon le premier ; le panneau, la lecture, « + Ajouter », « Mettre à jour » et la croix agissent sur lui, et « ‹ 2 / 3 › » passe de l'un à l'autre. La grille armée sait sa **cible** (`Placement::cible`) : aucune, elle fait naître un storyboard ; un storyboard, elle le range (« Mettre à jour », la poignée). Les couleurs viennent du thème (`theme.rs`, fiche 06, [`style.md`](../style.md)).

## 7. La plateforme

**La fin que le système demande** (FIN-SESSION-1, `plateforme/fin.rs`, fiche 82) : sous Windows,
une procédure devant celle de `winit` marque la fenêtre à `WM_ENDSESSION` et lui passe un
`WM_CLOSE`, que l'application lit comme une fin sans question (`app/fin.rs`,
`finir_avec_la_session`) ; sous Linux, `SIGTERM`, `SIGINT` et `SIGHUP` passent par un tuyau et un
fil, et la boucle part (un second signal fait partir sur-le-champ). Partout, la fin s'écrit une
fois : `GlucoseApp::finir` (FIN-1), qu'`exiting` appelle.

Tout ce qui parle au système vit dans `plateforme/` (et le `unsafe` avec), une voie par système —
et trois **portes** que le téléphone branche au lancement : `clavier` (lire et écrire l'état du
clavier virtuel), `doigt` (le délai de l'appui long, la vibration) et `marges` (ce que la barre
d'état, la navigation et le clavier recouvrent — BORD-1 : la barre du haut englobe la barre
d'état, et le bas de l'interface se pose dans `UiState::ecran_visible`) :
le dépôt depuis un navigateur (COM, Windows seulement), le **pavé par *Direct Manipulation***,
la carte qui tient l'écran (DXGI), le **glisser de nœuds vers une autre
fenêtre** (`DoDragDrop`) et le **presse-papiers** de ce qu'`arboard` ne sait pas dire — un seul
objet, `selection_windows` (COPIER-1, `IDataObject`), qui part au presse-papiers
(`OleSetClipboard`) et dans le glisser : le lot « Glucose.Lot », le texte, l'image seule en PNG et
`CF_DIBV5`, plusieurs images en `CF_HDROP` — des fichiers écrits au premier collage qui les veut,
dans `%TEMP%\glucose-copies` (`copies.rs`, partagé), et rendus à la fermeture (`OleFlushClipboard`) ; **sous X11, la sélection servie par Glucose** (`selection_x11`, fiche 61) — sa connexion, sa fenêtre marquée et son fil : le lot (`application/x-glucose-lot`), le texte, `image/png` ou `text/uri-list` et `x-special/gnome-copied-files`, `TARGETS`, `TIMESTAMP`, `MULTIPLE`, `INCR` au-delà de la plus grande requête, la prise à l'heure du serveur, la remise au gestionnaire à la fermeture (`SAVE_TARGETS`) ; sous macOS, `arboard` ne pose encore que le lot, en HTML ; l'objet d'`arboard` est **tenu** toute la vie de l'application (TENU-1 : sous X11, le copieur sert le collage) —, le téléchargement (WinHTTP sous Windows,
`ureq` ailleurs), **la feuille de partage** (PARTAGE-2, fiche 63 : `plateforme::partager`, une
porte que `glucose-android` branche sur `MainActivity.partagerUnFichier` ; là où elle est
branchée, exporter et enregistrer une image écrivent dans `partages/` — `persist/partages.rs`,
le seul dossier que le `FileProvider` ouvre — et **tout fichier qui vient d'y être écrit
s'offre**), l'offre de mémoire (`OfferVirtualMemory`, `madvise` sous Linux), le budget de la
carte (DXGI), la priorité des fils, l'identité pour la barre des tâches, l'heure locale, le journal
des plantages de Windows, la batterie.

## 8. Mesurer, et voir ce qui arrive

* **La chronique** (`chronique/`) : ce que chaque image coûte poste par poste, ce que l'écran a
  montré, les gels décomposés, le verdict. Écrite à la fin de chaque session dans
  `%TEMP%\glucose-chronique\derniere-session.txt`.
* **Ce qui a paru à l'écran** (PARU-1, fiche 112, `chronique/ecran.rs`) : le rythme (RYTHME-1)
  date l'instant où la présentation rend la main — sous Wayland, en `mailbox`, tout de suite, et
  l'image paraît deux battements plus tard, ou jamais. Le compositeur, lui, le sait : une
  **sonde** (`plateforme/presentation_wayland.rs`, `wp_presentation`, sur la connexion de la
  fenêtre comme le pincement) lui demande, juste avant chaque présentation, ce qu'il fera de
  l'image ; ses verdicts (montrée à tel battement, ou jetée) disent dans la chronique les images
  jetées, les **battements sautés alors qu'une image était due** (son échéance, GEL-1, précède
  l'instant où la précédente a paru) et le temps de l'envoi à l'écran avec sa gigue. Sans
  compositeur qui le dise (Windows, macOS, X11), la section se tait. Les images « qui manquent
  l'heure du tempo » sont celles du tempo, sur SA grille : ce n'est pas l'écran.
* **La boîte noire** (`boite_noire/`) : le **journal technique**, un fichier par session dans
  `%LOCALAPPDATA%\Glucose\journal-technique\`, nommé à l'heure de la session et lisible d'un
  double clic, avec un `LISEZ-MOI.txt` en colonnes (`boite_noire::fichiers`, fiche 58),
  écrit au fil de l'eau, qui survit à un plantage ; au lancement, comment la session d'avant a fini
  (et, sous Windows, dans quel module elle a planté). Elle garde la session précédente et
  toutes celles qui ont mal fini, efface le reste. Une session se reconnaît par sa **clé** (son
  début et son processus), pas par le nom de son fichier. Ses lignes ne portent que des nombres
  et des mots fixés à la compilation ; la carte graphique y entre par ses numéros (CARTE-1,
  fiche 60), que la présentation rend avec son nom (`Presenter::carte`). **Un épisode plus lent
  que le plancher dit où son temps est parti** (POSTES-1, fiche 63) : une ligne `poste` pour
  chacun des postes sans lesquels son image ordinaire tiendrait dans les 10 ms — leurs noms sont
  les marques de mesure du code, une liste fermée du serveur qu'une épreuve lie au code. Les
  distributions par geste de la chronique vivent dans `chronique/par_geste.rs`. **La mémoire**
  (MEMOIRE-1, fiche 66) : une ligne `memoire` à chaque record — l'empreinte la plus haute, ou ce
  que la machine laisse au plus bas —, relevée au rythme de la veille (`veiller_sur_la_machine`),
  et une ligne `alerte_memoire` quand le système en demande. Le `LISEZ-MOI.txt` du journal dit
  chaque champ de chaque type que le serveur accepte — une épreuve les lie.
* **La boîte noire qui voyage** (`telemetrie.rs`, fiche 54) : **avec l'accord de chacun**, les
  sessions closes partent au lancement, sur un fil à part, vers un Worker de Cloudflare
  (`outils/telemetrie/`, qui n'accepte que les noms de ses listes fermées). Rien ne part tant que
  `telemetrie::ADRESSE` est vide. La page publique : `docs/TELEMETRIE.md`.
* **Les bancs** (`crates/*/examples/bench_*`) mesurent hors écran ; `capture_temoin` rend la scène
  témoin en PNG ; `verifier_images` et `lire_histoire` lisent un document **sans l'écrire**.
* **Les instruments** (variables d'environnement, jamais des réglages de production) :
  `GLUCOSE_PERF`, `GLUCOSE_CARTE`, `GLUCOSE_IMAGES`, `GLUCOSE_PRESENT`, `GLUCOSE_ARBITRE`,
  `GLUCOSE_DEPOT`, `GLUCOSE_BORDURES`.

## 8 bis. La porte des agents (fiche 75)

Un agent — Protéo, Claude, tout client MCP — travaille dans le Glucose ouvert, sous les yeux de
l'utilisateur. `glucose --mcp` (`agents/relais.rs`) est le programme qu'un client lance : il
répond seul à la découverte et à la liste des outils, et ne se branche — et n'ouvre Glucose —
qu'au premier appel d'outil (RELAIS-1). La **porte** (`agents/porte.rs`) n'est pas un port
réseau : un tuyau nommé sous Windows, réservé au compte (AGENTS-2), dont le bout du relais est en
E/S superposée (`porte/tuyau.rs`, TUYAU-1) ; une prise Unix ailleurs ; aucune au téléphone. Le
**protocole** (`agents/protocole.rs`, MCP du 28/07/2026 et les versions d'avant) est pur ; le JSON
est écrit ici (`agents/json.rs`, JSON-1). Un appel d'outil arrive à la boucle par une boîte aux
lettres (`agents::Boite`) ; la boucle demande l'accord (AGENTS-1, `app/agents.rs`), puis
l'exécute (`agents/executer.rs`) — quatorze outils (`glucose_disposer`, fiche 92 ; `glucose_grouper`, fiche 98), dont la recherche de `Ctrl+F` (`executer/chercher.rs`, AGENT-7, fiche 90) et les domaines (`executer/domaines.rs`, fiche 88 :
`glucose_poser_domaine`, `glucose_domaines`, AGENT-5 et 6) — en **un geste du journal** : le noyau
(`core/store/agents.rs`) tient AGENT-1 à 4 (un geste atomique, la sélection rendue, le passage mot
pour mot, l'identifiant qui met à jour) ; et **MEMB-8** (fiche 97, `core/store/agents/rattacher.rs`) : ce qu'un agent pose `dans` une membrane lui appartient, même là où un voisin grandi la recouvre — la géométrie (MEMB-1) ne décide que lorsque rien n'est nommé. **AGENT-8** (fiche 98, `agents/executer/partir.rs`) : une mise à jour part du nœud tel qu'il est, et n'écrit que ce que l'appel nomme. **OPERATEUR-2** : une IA lit l'opérateur d'un post-it (`Decrit::operateur`) et le pose (`operateur=`), par un seul nom (`StickyOperator::id`) que le clavier partage. **COULEUR-2** : elle lit la couleur d'un nœud (`Decrit::couleur`) et la pose par la loi du noyau (`Store::colorer`). **GROUPER-1** : `glucose_grouper` (`agents/executer/grouper.rs`, `core/store/grouper.rs`), le `Ctrl+G` de l'humain. Une page web est une photo qui garde son adresse
(`source_url`) : le double-clic l'ouvre (`interactions/links.rs`). Une flèche ancrée à un passage arrive à sa hauteur, par le côté qui fait face (FLECHE-6, `core/arrow_anchor.rs`, fiche 76). Côté Protéo, chaque demande est triée (`tri.rs`) et posée sur une toile par demande (`toile/dessin.rs`). Le banc
`bench/porte.rs` (exemple `porte_sans_fenetre`) sert la porte sans fenêtre et rend le tableau en
images (et les membranes que `GLUCOSE_VUES` nomme ; `outils/wikipedia/essai_squelette.sh`).

**Le terminal, la seconde porte** (fiche 90, `agents/terminal.rs`) : `glucose <commande>` est un
outil de la même table (TERMINAL-1 — `poser-texte` pour `glucose_poser_texte`, ses arguments lus par le
schéma de l'outil, `glucose aide` tirée de sa description), envoyé par la même porte en **une** ligne du
protocole moderne, sous le nom « le terminal », donc par le même accord (TERMINAL-2). `main.rs` aiguille
la ligne de commande avant toute fenêtre ; un document demandé (`glucose document.glucose`, le
double-clic du type `application/x-glucose`, `outils/paquets/`) passe devant la reprise (OUVRIR-1,
`persist/reprise.rs`).

## 9. Distribuer et se mettre à jour

* **`mise_a_jour/`** : lit le `latest.json` **au format de Tauri**, vérifie la signature minisign
  **avant** d'écrire quoi que ce soit, ne propose qu'une version qui monte, sait comment ce programme
  est installé (NSIS, AppImage, `.deb`, `.rpm`, et l'APK — sous Android, « poser » c'est confier
  l'APK à `PackageInstaller`, `plateforme::installation`, fiche 58) ; un Glucose qui ne sait pas se remplacer (NixOS,
  `cargo run`) ne cherche rien. Après une session qui a mal fini, il cherche **avant tout**.
* **`outils/installeur/`** : l'installeur NSIS (par utilisateur, sans `RMDir /r`, il attend
  Glucose au lieu de le tuer) ; **`outils/paquets/`** : les trois formes Linux ;
  **`.github/workflows/publier.yml`** : un bouton qui construit, signe et prouve, en brouillon s'il
  est coché ; `flake.nix` pour NixOS.
* **La vérification** (`.github/workflows/ci.yml`) : dix tâches à chaque envoi, dont la bascule
  depuis un vrai Glucose Tauri, et l'APK d'Android (`outils/android/construire.sh`), déposé.

## 10. Les épreuves

* `crates/*/tests/*_suite.rs` et les `tests.rs` de chaque module. Une épreuve par invariant nommé
  (MEMB-1, PICK-2, SNAP-4…).
* **`tests/cliquets_suite.rs`** : des nombres que le dépôt ne peut que faire descendre — fonctions de
  80 lignes au plus, fichiers de 600, le couplage au modèle, les toasts, les modules sans appelant,
  les marques de mesure, l'unique endroit qui remplace un fichier. Le correctif d'un cliquet n'est
  jamais de relever son plafond.
* **Rien ne touche la machine de l'utilisateur** : chaque application d'épreuve habite un dossier
  temporaire (`crate::epreuve::Dossier`, retiré quand il sort de portée, qu'elle réussisse ou
  tombe) et a son propre presse-papiers ; seul `main.rs` ouvre le vrai dossier. Les épreuves
  de la sélection X11 ne visent qu'un serveur X nommé (`GLUCOSE_ECRAN_X_D_EPREUVE`, un Xvfb) —
  sur GitHub comme dans la distribution `Glucose-Linux` de ce PC (METHODE § 5).
