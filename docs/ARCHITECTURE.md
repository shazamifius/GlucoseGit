# L'architecture de Glucose Rust

> **Au 08/10/2026** (fiches 51 à 57 comprises). Comment le code est fait **aujourd'hui** : où vit chaque chose, et pourquoi.
> Ce n'est pas un plan : c'est une carte. Pour le *pourquoi* détaillé d'un mécanisme, chaque module
> porte son histoire en tête de fichier, et le code cite les fiches du [carnet](carnet/00-INDEX.md)
> par leur numéro (« fiche 22 § 5 » se lit dans `docs/carnet/22-…`).

---

## 1. Quatre crates, et la règle des dépendances

```
crates/
├── glucose-core/     le noyau : modèle, géométrie, journal, format, texte, index — AUCUNE dépendance
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
  (rastériseur de la voie processeur), `softbuffer`, `fontdue` (glyphes), `image` (décodeurs),
  `arboard` (presse-papiers) et `rfd` (dialogues) **hors d'Android seulement** — chacune derrière
  sa porte, `presse_papiers` et `dialogue` (DIAL-4, cliquet 12) —, `minisign-verify`
  (signatures), `pollster`, et `windows` sous Windows, `ureq` + `rustls` ailleurs. Une dépendance nouvelle exige une note dans
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
  quand se réveiller (**`ControlFlow::Wait`** : sans raison, Glucose dort).
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
  cartes de texte des **composants** rendus une fois puis posés (`renderer/composants.rs`) ; le fond,
  les lueurs, les membranes, les flèches et le liseré de la Time Machine sont des nuanceurs qui
  évaluent la même loi que le processeur. Autour des photos, deux couches (`couches.rs`) qui ne
  s'effacent et ne partent que par les **bandes** réellement écrites (`bandes.rs`), envoyées par des
  tampons qui restent (`envoi.rs`).
* **La garantie** : les deux voies rendent la même scène, au bit près ou à un écart **mesuré et
  borné** (`tests/voies_suite.rs`). L'adaptation change le chemin, jamais le résultat.
* **La surface qu'Android reprend** (VIE-1, `app/vie.rs`, `present/gpu/vie.rs`) : au
  `Suspended`, la présentation lâche sa surface et garde son périphérique et ses textures ; au
  `Resumed`, une surface neuve se crée sur le même. « En arrière-plan » se lit sur elle
  (`a_sa_surface`) : rien ne se dessine entre les deux.
* **La carte qui tient l'écran** (`plateforme/ecran.rs`, ECRAN-1) : sous Windows, Glucose dessine
  sur la carte dont une sortie porte le moniteur de la fenêtre — dessiner ailleurs fait recopier
  chaque image par le compositeur. L'ordre : `GLUCOSE_CARTE` › l'écran › le souvenir › l'économe.
* **L'arbitre** (`present/arbitre.rs`) ne s'installe que quand l'écran n'a pas tranché : il
  choisit la carte en la regardant travailler, et retient son verdict pour le lancement suivant
  (`carte.txt`) : changer de carte en cours de route figeait l'affichage (ARBITRE-4).

## 5. Les images et la mémoire

* **L'atelier** (`renderer/atelier.rs`) décode hors de l'image, sur des ouvriers qui **cèdent** au
  fil qui dessine — et à Blender ou Adobe (CEDER-1).
* **Le magasin** (`renderer/magasin.rs`, `photo.rs`) garde pour chaque image une pyramide de
  réductions par deux. Chaque niveau est **tenu**, **offert** au système (il le reprend s'il en a
  besoin, sans rien écrire sur le disque), en chemin ou perdu (ETAGES-1).
* **La carte graphique** reçoit le niveau qui couvre la taille posée ; son budget se lit chez le
  système et se suit même quand Glucose dort (ETAGES-2, 3).
* **Le disque** : la vue d'ensemble de chaque image (`renderer/apercu.rs`), nommée par l'empreinte
  de ses octets ; un document s'ouvre déjà montré.

## 6. L'interface

`ui/` : la bande du haut, rendue une fois par changement ; ses boutons placés par une seule mise en
page que le dessin et le clic lisent (loi L4) ; **le rail** (`rail.rs`, fiche 55), la même liste de
boutons reposée en grille sur le côté quand la barre ne tient plus, décidé avant la scène ; onglets, minimap, menu contextuel, barre d'action,
options de flèche, éditeur du texte lié, **un seul** toast (coupé en lignes à la largeur de
l'écran) ; **la question** (`question.rs`, QUESTION-1), que Glucose dessine là où le système
n'a pas de dialogues — Android —, et dont la suite (`interactions/question.rs`) est la même
qu'au bureau — elle répond au relâchement, sur la réponse pressée, et peut porter un **champ**
de texte (renommer un document) ; le menu contextuel **à la taille du doigt** quand deux touchers l'ouvrent. `dock/` : les panneaux (Ordonner,
Timer, Domaines, Time Machine ; Storyboard, Presets et Plugins sont des façades honnêtes), chacun
avec son cache. Les couleurs viennent du thème (`theme.rs`, fiche 06, [`style.md`](../style.md)).

## 7. La plateforme

Tout ce qui parle au système vit dans `plateforme/` (et le `unsafe` avec), une voie par système —
et trois **portes** que le téléphone branche au lancement : `clavier` (lire et écrire l'état du
clavier virtuel), `doigt` (le délai de l'appui long, la vibration) et `marges` (ce que la barre
d'état, la navigation et le clavier recouvrent — BORD-1 : la barre du haut englobe la barre
d'état, et le bas de l'interface se pose dans `UiState::ecran_visible`) :
le dépôt depuis un navigateur (COM, Windows seulement), le **pavé par *Direct Manipulation***,
la carte qui tient l'écran (DXGI), le **glisser de nœuds vers une autre
fenêtre** (`DoDragDrop`) et le **presse-papiers** de ce qu'`arboard` ne sait pas dire (le lot de
nœuds « Glucose.Lot », l'image en PNG et `CF_DIBV5`, fiche 51), le téléchargement (WinHTTP sous Windows,
`ureq` ailleurs), l'offre de mémoire (`OfferVirtualMemory`, `madvise` sous Linux), le budget de la
carte (DXGI), la priorité des fils, l'identité pour la barre des tâches, l'heure locale, le journal
des plantages de Windows, la batterie.

## 8. Mesurer, et voir ce qui arrive

* **La chronique** (`chronique/`) : ce que chaque image coûte poste par poste, ce que l'écran a
  montré, les gels décomposés, le verdict. Écrite à la fin de chaque session dans
  `%TEMP%\glucose-chronique\derniere-session.txt`.
* **La boîte noire** (`boite_noire/`) : un fichier par session dans `%LOCALAPPDATA%\Glucose\boite-noire\`,
  écrit au fil de l'eau, qui survit à un plantage ; au lancement, comment la session d'avant a fini
  (et, sous Windows, dans quel module elle a planté). Elle garde la session précédente et
  toutes celles qui ont mal fini, efface le reste.
* **La boîte noire qui voyage** (`telemetrie.rs`, fiche 54) : **avec l'accord de chacun**, les
  sessions closes partent au lancement, sur un fil à part, vers un Worker de Cloudflare
  (`outils/telemetrie/`, qui n'accepte que les noms de ses listes fermées). Rien ne part tant que
  `telemetrie::ADRESSE` est vide. La page publique : `docs/TELEMETRIE.md`.
* **Les bancs** (`crates/*/examples/bench_*`) mesurent hors écran ; `capture_temoin` rend la scène
  témoin en PNG ; `verifier_images` et `lire_histoire` lisent un document **sans l'écrire**.
* **Les instruments** (variables d'environnement, jamais des réglages de production) :
  `GLUCOSE_PERF`, `GLUCOSE_CARTE`, `GLUCOSE_IMAGES`, `GLUCOSE_PRESENT`, `GLUCOSE_ARBITRE`,
  `GLUCOSE_DEPOT`, `GLUCOSE_BORDURES`.

## 9. Distribuer et se mettre à jour

* **`mise_a_jour/`** : lit le `latest.json` **au format de Tauri**, vérifie la signature minisign
  **avant** d'écrire quoi que ce soit, ne propose qu'une version qui monte, sait comment ce programme
  est installé (NSIS, AppImage, `.deb`, `.rpm`) ; un Glucose qui ne sait pas se remplacer (NixOS,
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
  temporaire et a son propre presse-papiers ; seul `main.rs` ouvre le vrai dossier.
