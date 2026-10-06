# L'architecture de Glucose Rust

> **Au 06/10/2026.** Comment le code est fait **aujourd'hui** : où vit chaque chose, et pourquoi.
> Ce n'est pas un plan : c'est une carte. Pour le *pourquoi* détaillé d'un mécanisme, chaque module
> porte son histoire en tête de fichier, et le code cite les fiches du [carnet](carnet/00-INDEX.md)
> par leur numéro (« fiche 22 § 5 » se lit dans `docs/carnet/22-…`).

---

## 1. Trois crates, et la règle des dépendances

```
crates/
├── glucose-core/     le noyau : modèle, géométrie, journal, format, texte, index — AUCUNE dépendance
├── glucose-math/     les formules LaTeX en géométrie pure — une dépendance, katex-rs
└── glucose-desktop/  l'application : fenêtre, deux voies de rendu, plateforme, mise à jour
```

* **Le noyau n'a aucune dépendance**, et se teste sans écran. C'est ce qui lui permet de compiler
  pour Android et le web à chaque envoi.
* **L'application a les siennes**, chacune défendue par une impossibilité de faire sans :
  `winit` (fenêtre), `wgpu` (Vulkan, Metal, Direct3D, GL ES derrière une interface), `tiny-skia`
  (rastériseur de la voie processeur), `softbuffer`, `fontdue` (glyphes), `image` (décodeurs),
  `arboard` (presse-papiers), `rfd` (dialogues), `minisign-verify` (signatures), `pollster`, et
  `windows` sous Windows, `ureq` + `rustls` ailleurs. Une dépendance nouvelle exige une note dans
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
  Time Machine (`temps.rs`), les raccourcis (`shortcuts.rs`, chiffres lus sur la touche
  **physique** pour les claviers AZERTY).
* **La navigation** : `pan_zoom.rs` décide si un défilement est un zoom ou un déplacement, et s'il
  vient d'une molette ou d'un doigt ; `elan.rs` tient une **dette** (ce que la main a demandé et
  que l'écran n'a pas montré) remboursée à chaque image ; `vol.rs` suit le chemin de van Wijk et
  Nuij (`F`, signets, dossiers) ; `horloge.rs` avance du temps que l'écran **montre** ; `tempo.rs`
  donne à chaque image un nombre entier de balayages.

## 3. Le document

| | où | ce qu'il fait |
|---|---|---|
| **Le modèle** | `core/types/` | projet → tableaux → images, annotations (carte, pense-bête, flèche, membrane), dossiers ; coordonnées monde en `f64`, origine haut-gauche |
| **Le magasin** | `core/store/` | la seule porte d'écriture. Chaque modification est une **transaction du journal** (`journal.rs`), inversible : l'annulation coûte la taille du changement, jamais celle du document. Un geste = une entrée (`undo.rs`) ; un glisser s'écrit en **une** translation (GLISSER-1), un redimensionnement en une édition (FONDRE-1). La navigation et la sélection n'entrent jamais dans l'annulation |
| **Le fichier** | `core/persist/` | `.glucose` : une **base** (conteneur versionné, sections avec somme de contrôle) puis une **histoire en ajout seul** (`histoire.rs`) — objets (octets d'image par empreinte SHA-256), gestes, instantanés, jalons, vues — chaque entrée chaînée à la précédente : une fin déchirée s'arrête à la dernière entrée saine |
| **Le scribe** | `desktop/persist/scribe.rs` | un fil qui ajoute les gestes au fichier et synchronise le disque ; la cadence suit le disque. `Ctrl+S` pose un **jalon**. Un document sans nom vit dans un **brouillon** |
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
* **L'arbitre** (`present/arbitre.rs`) choisit la carte graphique en la regardant travailler, et
  retient son verdict pour le lancement suivant (`carte.txt`) : changer de carte en cours de route
  figeait l'affichage (ARBITRE-4).

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
page que le dessin et le clic lisent (loi L4) ; onglets, minimap, menu contextuel, barre d'action,
options de flèche, éditeur du texte lié, **un seul** toast. `dock/` : les panneaux (Ordonner,
Timer, Domaines, Time Machine ; Storyboard, Presets et Plugins sont des façades honnêtes), chacun
avec son cache. Les couleurs viennent du thème (`theme.rs`, fiche 06, [`style.md`](../style.md)).

## 7. La plateforme

Tout ce qui parle au système vit dans `plateforme/` (et le `unsafe` avec), une voie par système :
le dépôt depuis un navigateur (COM, Windows seulement), le téléchargement (WinHTTP sous Windows,
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
  toutes celles qui ont mal fini, efface le reste, et **n'envoie rien**.
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
* **La vérification** (`.github/workflows/ci.yml`) : neuf tâches à chaque envoi, dont la bascule
  depuis un vrai Glucose Tauri.

## 10. Les épreuves

* `crates/*/tests/*_suite.rs` et les `tests.rs` de chaque module. Une épreuve par invariant nommé
  (MEMB-1, PICK-2, SNAP-4…).
* **`tests/cliquets_suite.rs`** : des nombres que le dépôt ne peut que faire descendre — fonctions de
  80 lignes au plus, fichiers de 600, le couplage au modèle, les toasts, les modules sans appelant,
  les marques de mesure, l'unique endroit qui remplace un fichier. Le correctif d'un cliquet n'est
  jamais de relever son plafond.
* **Rien ne touche la machine de l'utilisateur** : chaque application d'épreuve habite un dossier
  temporaire et a son propre presse-papiers ; seul `main.rs` ouvre le vrai dossier.
