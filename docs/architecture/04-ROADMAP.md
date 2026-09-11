# 04 — Roadmap : de 20 % à 100 %

> **Principe directeur** : chaque phase se termine sur une application **utilisable**,
> pas sur un chantier ouvert. À aucun moment tu ne dois te retrouver avec du code à moitié
> migré qui ne tourne pas.
>
> **Règle de non-régression** : une phase n'est pas finie tant que ses critères de sortie ne
> sont pas tous verts. Pas de « je finirai plus tard » — c'est exactement comme ça que naissent
> les 27 modules morts d'aujourd'hui.

---

## Vue d'ensemble

| Ph. | Nom | Objectif | Parité à la sortie |
|:--:|---|---|---:|
| **0** | Vérité & garde-fous | Que le dépôt arrête de mentir. Mesurer. | 20 % |
| **1** | Brancher le noyau | Utiliser ce qui est déjà écrit. Tuer les bugs bloquants. | **30 %** |
| **2** | Persistance | L'app sait enregistrer et ouvrir. **Elle devient un logiciel.** | **37 %** — ⏳ *enregistrer/ouvrir livrés (`ed56e2b`) ; restent journal, autosave, import v1, et l'alerte de fermeture* |
| **3** | Manipulation directe | Redimensionner, pivoter, menu contextuel, panneaux. | **47 %** |
| **4** | Texte professionnel | Retour à la ligne, Markdown, IME, sélection. | **57 %** |
| **5** | Entrée universelle | Glisser-déposer web **et** fichiers. Ton besoin n°1. | **62 %** |
| **6** | Membranes | Modes, focus, adoption, animation. | **69 %** |
| **7** | Dossiers & miroirs | Sous-canvas, miroir de dossier OS, alias vivants. | **77 %** |
| **8** | Flèches sémantiques | Dessin, attaches, prédicats, portails. | **83 %** |
| **9** | Sémantique & organisation | Domaines, temporalité, storyboard, presets. | **90 %** |
| **10** | Export & interopérabilité | SVG, PNG, HTML, Markdown, import v1. | **94 %** |
| **11** | Collaboration & rideaux | Temps réel, curseurs, rideaux, time machine. | **98 %** |
| **12** | Extensions & finition | Plugins, App Bridge, recherche, Pomodoro, mises à jour. | **100 %** |

**Repères de charge** (développeur seul, temps sérieux, non calendaire) :
phases 0-2 ≈ 20 % de l'effort total, phases 3-5 ≈ 20 %, phases 6-9 ≈ 35 %,
phases 10-12 ≈ 25 %.

### Le volume réel derrière ces pourcentages

Les pourcentages de parité comptent des **fonctionnalités**. Voici ce qu'ils représentent en
**lignes de TypeScript à porter**, mesuré sur la source hors tests (`e2cd410`) :

| Phase | Sous-systèmes | TS à porter | Noyau Rust déjà écrit |
|:--:|---|---:|---|
| 2 | Persistance, assets, versions, compaction | **2 572 l.** | 354 l. (`bundle`) — **morte** |
| 4 | Markdown, KaTeX, éditeur, ancres de texte | **2 229 l.** | 211 l. (`text_anchors`) — **morte** |
| 6 | Membranes : espace, focus, tween, étirement | **2 661 l.** | 1 442 l. — **mortes** |
| 7 | Dossiers & miroirs | **1 307 l.** | 110 l. (`mirror_graph`) — **morte** |
| 8 | Flèches : tracé, ancrage, texte, options | **1 386 l.** | 166 l. (`arrow_anchor`) — **morte** |
| 9 | Domaines, temporalité, storyboard, presets | **2 593 l.** | domaines complets — **liste fantôme (R-47)** |
| 10 | Export SVG / HTML / PNG / Markdown | **1 565 l.** | 430 l. (`export`) — **morte** |
| 11 | Collaboration CRDT, curseurs, rideaux | **3 098 l.** | 417 l. (curtains) — **mortes** |
| 12 | Plugins, App Bridge, télémétrie | **2 259 l.** | — |
| | **Total** | **19 670 l.** | **3 601 l. écrites et mortes** |

Sur **32 423 lignes** de TypeScript hors tests, ces sous-systèmes en représentent **61 %**. Le
reste — environ 12 750 lignes — est le cœur du canvas (`GlucoseCanvas.tsx` 4 181 l.,
`store/index.ts` 2 113 l., sélection, glisser-déposer, alignement), qui est la partie réellement
portée aujourd'hui.

**À retenir** : aucune optimisation de performance ne réduit ces 19 670 lignes. Les gains de
rendu obtenus (240 ms → 17 ms par frame) rendent le prototype utilisable pour développer ; ils ne
font avancer la parité d'aucun point.

### Insertion : la fidélité du rendu

Deux constats découverts en confrontant le rendu Rust au TypeScript (R-45, R-46) ne relèvent
d'aucune phase existante, parce que ce ne sont pas des fonctionnalités manquantes : ce sont des
défauts qui **dégradent tout ce qui est déjà acquis**.

| # | Tâche | Constat | Où |
|---|---|---|---|
| 1.27 | Une carte se dessine en coordonnées locales puis subit **une seule** transformation | R-45 | ✅ **VÉRIFIÉ** — **12** `clamp` retirés (l'audit en annonçait 11) + 6 seuils implicites qui faisaient disparaître le texte ; `renderer/scale.rs` centralise `world()` / `screen()` |
| 1.28 | Positionnement **sous-pixel** des glyphes | R-46 | ✅ **VÉRIFIÉ** — 4 phases par axe dans la clé du cache, variantes dérivées par interpolation bilinéaire ; coût frame nul (16,84 → 16,70 ms), cache borné inchangé |
| 1.29 | Capture PNG comparée entre zoom 0,25 / 0,5 / 1 / 2 / 4 pour prouver la fidélité | R-45, R-46 | ✅ **VÉRIFIÉ** — rapport encre/boîte : avant 1,031 → *néant* → 0,245 ; après 0,615 → 0,593. Test permanent `renderer/card/proof.rs` |
| 1.30 | **Plafonner le rayon du halo en unités écran.** Il croît aujourd'hui linéairement avec le zoom, sans borne : à ×3 une seule carte remplit l'écran et coûte 10,1 ms. Un halo est un effet de présentation, il n'a pas à grandir indéfiniment. Utiliser `WorldScale::screen()`. | **R-50** | ⏳ **PRIORITÉ** |

Ces trois tâches passent **avant** toute nouvelle fonctionnalité : il est moins coûteux de
réparer la fidélité sur 20 % du logiciel que sur 100 %.

**Le point de bascule est la phase 2.** Avant, tu construis un prototype. Après, tu construis un
logiciel — et tu peux enfin l'utiliser toi-même tous les jours, ce qui est la meilleure source de
priorités qui existe.

---

## Phase 0 — Vérité & garde-fous

> *« Avant de construire, savoir où on est. »*
> **Aucun code applicatif nouveau.** Uniquement de l'outillage et du nettoyage.

### Pourquoi cette phase existe

Aujourd'hui l'historique dit que tiny-skia a été éliminé alors qu'il est partout (R-32), et la
barre d'outils affiche 19 fonctionnalités dont 9 existent (R-33). **Tu ne peux pas piloter un
projet dont les instruments mentent.**

### Travaux

| # | Tâche | Répare |
|---|---|---|
| 0.1 | Retirer ou griser les **10 boutons sans effet observable** (Dossier, Timer, Storyboard, Aimant, Trans-domaines, Domaines, Preset, Plugins, Exporter, Collaborer). Un bouton grisé + infobulle « bientôt », jamais un toast qui simule. | R-33 |
| 0.2 | Corriger le `README` / `ROADMAP` : lister les dépendances réelles, l'état réel. Ajouter un lien vers ces documents. | R-32 |
| 0.3 | Créer `docs/architecture/decisions/` et y consigner la règle des 2 dépendances. | — |
| 0.4 | Mettre en place un **HUD de diagnostic** (F12) : ms/frame, nœuds visibles / total, allocations, mémoire du cache d'images, taille de la pile d'undo. | mesure de L2 |
| 0.5 | Créer `glucose-cli` avec une commande `render-scene` : charge une scène de référence, produit un PNG. | test visuel |
| 0.6 | CI : `cargo check`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, et **comparaison de PNG** sur 5 scènes de référence. | — |
| 0.7 | Interdire `let _ =` et `unwrap()` hors tests via un lint clippy. | R-21 |
| 0.8 | Ajouter des **bancs d'essai** : 10 / 100 / 1 000 / 10 000 nœuds, zoom 1 / 0,1 / 0,01. Publier les chiffres. | L2 |

### Critères de sortie

- [ ] Aucun bouton de l'UI ne ment sur ce qu'il fait.
- [ ] `README` et `ROADMAP` décrivent l'état réel, vérifiable par `grep`.
- [ ] La CI casse si un warning clippy apparaît.
- [ ] Le HUD affiche les ms/frame en direct.
- [ ] Les chiffres de référence sont écrits noir sur blanc — y compris le gel au zoom 0,01.

---

## Phase 1 — Brancher le noyau

> *« 27 fonctionnalités sont écrites, testées, et jamais appelées. Va les chercher. »*

C'est la phase au **meilleur rapport valeur/effort de tout le projet** : +13 points de parité
pour très peu de code neuf.

### 1A — Les bugs bloquants (à faire en premier, ils sont courts)

| # | Tâche | Répare | Effort / État |
|---|---|---|---|
| ~~1.1~~ | ~~`Transform::from_scale(k).post_translate(sx,sy)`~~ | R-06 | ✅ **fait** (`81aea31`) |
| ~~1.2~~ | ~~Grille adaptative~~ *(reste : tuile en cache, mineur)* | R-02 | ✅ **fait** (`81aea31`) |
| 1.3 | Culling via `SpatialHash` — passer du rejet naïf en O(n) à une vraie requête spatiale en O(visible) | R-05 | ✅ **VÉRIFIÉ** |
| 1.4 | Teinte symbiotique mémorisée, invalidée par voisinage | R-03 | ⚠️ **PARTIEL** — cache OK, mais `n` allocations `String` par frame |
| 1.5 | **Onglets et minimap** : les faire passer par `layout_topbar()` — une seule mesure pour dessin et clic | R-07, R-20 | ✅ **VÉRIFIÉ** — `layout_tabs` partagé, test à l'appui |
| 1.6 | Hit-test de l'UI sur toute la fenêtre → la minimap revit | R-08 | ✅ **VÉRIFIÉ** — hit-test sur toute la fenêtre |
| 1.7 | Minimap : bornes réelles + annotations + membranes | R-08 | ✅ **VÉRIFIÉ** — `layout_minimap` partagé |
| 1.8 | Générateur d'id monotone unique (fin de `-dup`, `-mirror`, `board-{len}`) | R-13, R-14, R-22 | ✅ **VÉRIFIÉ** — `generate_id` est un simple incrément O(1) ; `id_exists` a quitté le chemin de génération et ne sert plus qu'aux tests. L'unique point de scan est `resync_next_id`, documenté comme INVARIANT ID-1 |
| 1.9 | Sticky : utiliser `bg_color` et `color` du modèle | R-24 | ✅ **VÉRIFIÉ** |
| 1.10 | DPI : facteur d'échelle appliqué à toute l'UI + `ScaleFactorChanged` | R-16 | ✅ **VÉRIFIÉ** — layouts, polices et docks scalés ; test 150 % avec hit-testing |

### 1B — La boucle de rendu

| # | Tâche | Répare | État |
|---|---|---|---|
| 1.11 | Un seul point de rendu : les handlers marquent `dirty`, `RedrawRequested` peint | R-15 | ✅ **VÉRIFIÉ** — `mark_dirty()` → `request_redraw()` |
| 1.12 | `ControlFlow::WaitUntil` piloté par la prochaine échéance d'animation → curseur qui clignote, toasts qui s'effacent | R-15 | ✅ **VÉRIFIÉ** — curseur 2 fps, plateau toast à 0 repaint, `Wait` au repos |
| 1.13 | Rectangles sales : ne repeindre que ce qui a changé | L2, **R-42** | ⏳ **PRIORITÉ 1, en cours** — mesuré : `docks` 7,0 ms + `ui` 3,0 ms = **59 % de la frame**, redessinés alors qu'ils ne changent jamais. Contrainte de conception retenue : invalidation par **empreinte recalculée chaque frame**, jamais par un `invalidate()` qu'il faut penser à appeler |
| 1.14 | Cache de glyphes (atlas) ; contour de membrane rastérisé une fois | R-26, R-27 | ✅ **VÉRIFIÉ** — LRU réelle, `Rc`, `data_mut()` hissé, métriques réelles |

### 1C — Le branchement du noyau

| # | Tâche | Débloque | État |
|---|---|---|---|
| 1.15 | Appeler `snap_move` / `snap_resize` pendant le drag → le magnétisme existe enfin | 2.7, 2.8 | ✅ **VÉRIFIÉ** — sans dérive de curseur, bien écrit |
| ~~1.16~~ | ~~Appliquer la sélection élastique au relâchement~~ | — | ✅ **fait** (`2da029f`) |
| 1.17 | `hit_priority` alimenté par l'index spatial | 2.2 | ✅ **VÉRIFIÉ** |
| 1.18 | `handle_cursor()` branché → curseurs contextuels | 2.11 | ✅ **VÉRIFIÉ** |

### 1D — L'undo par journal

| # | Tâche | Répare | État |
|---|---|---|---|
| 1.19 | Introduire `Command` + `Inverse` ; convertir toutes les mutations de `store.rs` | R-04 | *(en cours)* |
| 1.20 | Sortir `blobs` du document → `AssetStore` séparé | R-04 | ✅ **VÉRIFIÉ** — `AssetStore` hors de `Project` |
| 1.21 | `VecDeque` + coalescence de la frappe + libellés d'action | R-04 | ✅ **VÉRIFIÉ** — `VecDeque` + `pop_front` |
| 1.22 | Revalider les invariants existants (navigation, caméra, transaction live) sur le nouveau moteur | 15.4, 15.5 | ✅ **VÉRIFIÉ** — 34 tests verts |

### 1E — Hygiène

| # | Tâche | Répare | État |
|---|---|---|---|
| 1.23 | `GlucoseError` par crate ; barre de statut qui affiche les erreurs | R-21 | ⚠️ **PARTIEL** — `DesktopError` câblé (5 sites, toasts visibles), `let _ =`/`unwrap` nettoyés ✅ ; **`CoreError` désormais câblé pour de bon : 32 références dans 6 fichiers** ✅ ; **reste** la barre de statut, et le chargement d'images qui échoue en silence |
| 1.24 | Structure `Theme` : les ~130 littéraux de couleur deviennent des jetons | R-31 | ⚠️ **PARTIEL** — `dock.rs` 134→41 littéraux, `ui.rs` 35→4 ✅ ; mais **`renderer.rs` intact (37 littéraux, 3 usages)** et `Theme::light()` jamais appelé |
| 1.25 | Supprimer `store::ActiveTool` (doublon) | R-25 | ✅ **VÉRIFIÉ** |
| 1.26 | Sortir `organize_layout` de `app.rs` vers `glucose-model`, avec une convention de coordonnées unique | R-11, R-19 | ✅ **VÉRIFIÉ** — `app.rs` 1126 → 331 l. |

### Critères de sortie

*Vérifiés dans le code au commit `3bd9bda`. Un critère n'est coché que si une preuve
existe dans le dépôt — test vert, ou lecture du code.*

- [ ] **Frame < 8 ms** sur 10 000 nœuds dont 50 visibles, **et** au zoom 0,01.
      → **non mesurable** : aucun banc n'existe (tâche 0.8 non faite).
- [ ] **200 undos sur un projet de 200 images < 1,5 Go** de mémoire.
      → `push_undo` clone toujours `Project` en entier (1.19 en cours). Non mesuré.
- [x] Le curseur clignote sans bouger la souris ; les toasts s'effacent seuls.
      → cadence vérifiée : 2 fps curseur, 0 repaint sur plateau, `Wait` au repos.
- [x] Le magnétisme fonctionne, guides visibles à l'appui.
      → `snap_move` appelé dans `drag.rs:67`, `active_guides` assigné.
- [x] La sélection élastique sélectionne.
      → `selection.rs`, corrigé en `2da029f`.
- [x] Cliquer sur chaque onglet sélectionne le bon board — **test automatisé**.
      → `layout_tabs` partagé + `test_click_tabs_selects_correct_board` vert.
- [ ] Les images se placent au bon endroit à tout zoom — **test PNG de référence**.
      → le correctif est en place, mais **le test PNG de référence n'existe pas** (0.5/0.6 non faits).
- [x] L'UI est lisible et cliquable à 100 %, 150 % et 200 %.
      → `test_ui_dpi_scaling_and_hit_testing_at_150_percent` + `test_organize_layout_exact_utf8_hit_test` verts.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passe.
      → code de sortie 0, 0 avertissement, **et le critère n'est plus creux** : le bloc
      `#![allow(…)]` de `main.rs:2` est supprimé (R-44). Les 46 avertissements qu'il
      masquait sont corrigés, dont les 17 `too_many_arguments` — regroupés en types
      nommés dans `params.rs` — et aucun `#[allow(clippy::…)]` n'a été réintroduit.

### Budget de frame mesuré (`def3800`)

Le premier chiffrage réel du rendu, board par défaut, profil dev, `GLUCOSE_PERF=2`.
Il remplace les impressions par des priorités.

| Poste | Coût | Part | Constat | Tâche |
|---|---:|---:|---|---|
| `docks` | 7,57 ms | 43 % | R-42 — redessiné intégralement chaque frame | **1.13** |
| `ui` | 3,50 ms | 20 % | même cause | **1.13** |
| `grid` | 2,49 ms | 14 % | R-43 — `PathBuilder` réalloué chaque frame | *nouvelle* |
| `blit` | 1,78 ms | 10 % | boucle scalaire sur 1,3 M pixels | — |
| `halos` | 1,50 ms | 9 % | ✅ corrigé (7,19 → 1,50 ms, ×4,8) | — |
| `present` | 0,76 ms | 4 % | — | — |
| **Total** | **17,50 ms** | | cible : < 8 ms | |

**Ce que ça change dans l'ordre des travaux** : les rectangles sales (1.13) ne sont plus une
optimisation de confort, ils portent **63 % de la frame** à eux seuls (`docks` + `ui`). C'est le
prochain chantier de performance le plus rentable du projet, loin devant tout le reste.

Deux réserves sur ce tableau. D'abord il est mesuré sur un board **quasi vide** : c'est le coût
plancher, celui qu'on paie même sans rien afficher. Ensuite il reste une mesure ponctuelle et non
un banc reproductible — la tâche **0.8** est toujours à faire, et tant qu'elle ne l'est pas, le
critère « frame < 8 ms sur 10 000 nœuds » reste invérifiable.

**Bilan de la phase 1 : 7 critères sur 9 sont passés au vert.** Voir la section « Ce qui reste »
ci-dessous et [`01-AUDIT-CODE-RUST.md`](01-AUDIT-CODE-RUST.md).

---

## Phase 2 — Persistance

> ***La phase qui transforme une démo en logiciel.*** Rien d'autre n'a de valeur avant.

### Travaux

| # | Tâche | État |
|---|---|---|
| 2.1 | Sérialisation binaire maison de `Document` (0 dépendance), avec numéro de version en tête | ✅ `glucose-core/src/persist/` |
| 2.2 | `AssetStore` content-addressed sur disque (sha256 → octets), dédup native | ✅ |
| 2.3 | Conteneur `.glucose` v2 : `manifest` + `document` + `assets/` + `journal` | 🟡 les trois premières sections ; la nature `journal` est réservée et sautée |
| 2.4 | Sommes de contrôle par section ; détection et récupération partielle d'un fichier tronqué | 🟡 détection ✅ (sha256 par section) ; récupération partielle non faite |
| 2.5 | **Écriture atomique** : `.tmp` → `fsync` → renommage. Un crash ne détruit jamais le fichier | ✅ `glucose-desktop/src/persist/atomic.rs` |
| 2.6 | `Ctrl+S`, `Ctrl+Maj+S`, `Ctrl+O`, fichiers récents, titre de fenêtre avec indicateur de modification | 🟡 les trois raccourcis ✅ et le titre ✅ ; fichiers récents non faits |
| 2.7 | **Alerte de fermeture** : sur `CloseRequested`, si le document est modifié, proposer Enregistrer / Ne pas enregistrer / Annuler | **R-48** | ✅ **VÉRIFIÉ** — `persist/close.rs`, INVARIANT SAVE-3 : après « Enregistrer », l'état modifié est **relu** plutôt que la réponse du dialogue, donc un enregistrement raté ou un chemin annulé laissent la fenêtre ouverte |
| 2.8 | Sauvegarde automatique par journal (delta uniquement) | ❌ |
| 2.9 | Récupération après crash : au démarrage, proposer de reprendre le journal orphelin | ❌ |
| 2.10 | Ramasse-miettes des assets à la sauvegarde | ✅ le magasin est reconstruit à chaque écriture |
| 2.11 | Chaîne de migrations `v2 → v3 → …`, avec un test par saut de version | 🟡 les deux numéros de version sont en place, aucune migration à écrire pour l'instant |
| 2.12 | **Importeur du `.glucose` v1** (bundle TypeScript), lecture seule | ❌ refusé par un message explicite |
| 2.13 | Chargement paresseux des images : le document s'ouvre d'abord, les images arrivent ensuite | ❌ |

### Critères de sortie

- [x] Créer un projet, le fermer, le rouvrir : **identique au bit près** (test de round-trip).
      → `crates/glucose-core/tests/persist_suite.rs`, `crates/glucose-desktop/src/persist/commands.rs`
- [x] Tuer le processus pendant une sauvegarde : le fichier précédent est intact.
      → écriture atomique, `crates/glucose-desktop/src/persist/atomic.rs`
- [ ] Tuer le processus après 50 modifications : au redémarrage, les 50 sont récupérées.
- [ ] Ouverture d'un projet de 500 nœuds + 200 images : **premier affichage < 400 ms**.
- [ ] Sauvegarde incrémentale **< 30 ms**.
- [ ] Un `.glucose` produit par la version TypeScript s'ouvre sans perte de contenu.
- [x] La même image insérée 10 fois occupe **un seul blob** dans le fichier.
      → `test_a_duplicated_asset_writes_a_single_blob`

---

## Phase 3 — Manipulation directe

> *« Le canvas devient manipulable, pas seulement affichable. »*

| # | Tâche | Fonctions débloquées |
|---|---|---|
| 3.1 | Poignées de redimensionnement interactives, avec magnétisme et préservation de ratio | 2.9, 3.9, 3.10, 5.8, 7.4 |
| 3.2 | Rotation à la poignée | 2.12 |
| 3.3 | Verrouillage : indicateur visuel + bascule | 2.13 |
| 3.4 | Ordre d'empilement : premier/dernier plan, `Ctrl+]` / `Ctrl+[` | 2.17 |
| 3.5 | **Menu contextuel** ; le pan passe au bouton du milieu / `Espace` | 2.16, 19.14 |
| 3.6 | Mipmaps + échantillonnage trilinéaire | 3.12 |
| 3.7 | Cache d'images LRU borné, avec cache négatif | 3.13 |
| 3.8 | Décodage asynchrone + vignette de remplacement + lecture d'en-tête pour les dimensions | 3.14, 3.15, 3.16 |
| 3.9 | Système de panneaux (dock, un panneau actif, en-tête commun, `Échap`) | 18.9 |
| 3.10 | Sélecteur de couleur | 18.10, 5.6 |
| 3.11 | Panneau Organiser (grilles, colonnes, tri, espacement) | 18.11 |
| 3.12 | Boards : renommer, dupliquer, fermer, réordonner | 18.6, 18.7, 22.6 |
| 3.13 | Infobulles + navigation clavier dans l'UI | 18.18, 18.21 |
| 3.14 | `fit: contain` pour les vignettes | 3.11 |

### Critères de sortie

- [ ] Toute image / carte / membrane se redimensionne et pivote à la souris, avec magnétisme.
- [ ] Le clic droit ouvre un menu contextuel adapté à la cible.
- [ ] Importer 50 photos de 24 Mpx : **aucun gel > 100 ms**, cache **< 800 Mo**.
- [ ] Un `Tab` parcourt l'interface dans un ordre sensé.

---

## Phase 4 — Texte professionnel

> *« Glucose est un outil de pensée écrite. Le texte doit être irréprochable. »*

| # | Tâche | Fonctions |
|---|---|---|
| 4.1 | Moteur de mise en page : runs → shaping → césure → lignes de base réelles | 4.8, 4.17 |
| 4.2 | Crénage (`kern` / `GPOS` simple) | 4.17 |
| 4.3 | Retour à la ligne automatique aux limites de mots | 4.8 |
| 4.4 | Curseur et sélection depuis le layout (fin des mesures divergentes) | 4.3, 4.6, 5.7 |
| 4.5 | Navigation ↑ ↓, `Maj`+flèches, `Ctrl`+flèches (mot à mot), glisser pour sélectionner | 4.5, 4.6 |
| 4.6 | Copier / couper / coller dans le texte, presse-papiers en écriture | 4.7, 19.15 |
| 4.7 | **IME complet** + grappes de graphèmes | 4.4, 19.12 |
| 4.8 | Markdown : titres 1-3, gras, italique, barré, code, listes, citations | 4.9 → 4.13 |
| 4.9 | Redimensionnement automatique de la carte selon le contenu | 4.8 |
| 4.10 | Reste des raccourcis clavier (`F11`, `Échap` global, `Ctrl+Maj+*`, pavé numérique, signets 1-9) | 1.7, 1.8, 1.13, 19.6, 19.9, 19.10 |
| 4.11 | Copier / coller d'éléments entre boards | 2.15 |
| 4.12 | Déplacement au clavier | 2.18 |
| 4.13 | Défilement horizontal | 1.12 |

### Critères de sortie

- [ ] Un paragraphe de 500 mots dans une carte de 300 px : **retour à la ligne correct**, pas de débordement.
- [ ] `^` + `e` donne `ê`. Saisie japonaise fonctionnelle.
- [ ] Sélectionner du texte à la souris sur 3 lignes fonctionne, y compris avec des emojis.
- [ ] Supprimer un emoji composé le supprime **entièrement**.
- [ ] Le Markdown s'affiche correctement, et le texte source reste éditable.
- [ ] Rendu de 200 cartes de texte : **< 8 ms/frame** (atlas de glyphes à l'œuvre).

---

## Phase 5 — Entrée universelle

> ***Ton besoin explicite. Et le meilleur argument « from scratch » du projet.***

| # | Tâche |
|---|---|
| 5.1 | `glucose-platform` : cible de dépôt OLE complète sous Windows (`IDropTarget`) |
| 5.2 | Négociation multi-formats : `CF_HDROP`, `CFSTR_INETURL`, `CF_HTML`, `CF_UNICODETEXT`, `CFSTR_FILECONTENTS`, `CF_DIB` |
| 5.3 | XDND sous Linux : `text/uri-list`, `text/html`, `image/png`, `application/x-moz-file` |
| 5.4 | Dépôt **multi-fichiers** (aujourd'hui : un seul à la fois) |
| 5.5 | Retour visuel au survol : contour du canvas, position d'insertion prévisualisée |
| 5.6 | Coller une URL d'image → téléchargement + insertion |
| 5.7 | `source_url` renseignée depuis le dépôt web (champ du modèle enfin utilisé) |
| 5.8 | Presse-papiers en écriture, multi-formats (copier une carte → PNG + texte + HTML) |

### Critères de sortie

- [ ] Glisser une image depuis Firefox / Chrome / Edge → elle arrive, **avec son URL d'origine**.
- [ ] Glisser 20 fichiers depuis l'explorateur → les 20 arrivent, disposés proprement.
- [ ] Glisser depuis un site sans fichier local (image en `<canvas>`, data-URL) → ça marche via `CF_DIB`.
- [ ] Le survol du dépôt donne un retour visuel avant le lâcher.
- [ ] Même comportement sous Linux.

---

## Phase 6 — Membranes

> *« La première fonctionnalité identitaire de Glucose. »*
> 5 modules du noyau (≈ 2 100 lignes + 1 600 de tests) sortent enfin du placard.

| # | Tâche | Module réutilisé |
|---|---|---|
| 6.1 | Dessiner une membrane par glisser (fin de la taille fixe 320×240) | — |
| 6.2 | Redimensionner, avec magnétisme | — |
| 6.3 | **Appartenance stockée** (`membrane_id`) : dépôt = adoption, invariant MEMB-1 | `membrane_space` |
| 6.4 | Mode `minimized` : l'échelle du contenu descend sous 1 | `membrane_space` |
| 6.5 | Mode `stretched` : la membrane s'agrandit pour contenir | `membrane_stretch` |
| 6.6 | Alerte d'étirement | `membrane_stretch` |
| 6.7 | **Mode Focus** | `membrane_focus` |
| 6.8 | Animation de transition | `membrane_tween` |
| 6.9 | Panneau Options de membrane (mode, couleur, titre) | — |
| 6.10 | Membranes imbriquées | `membrane_space` |
| 6.11 | Suppression : contenu libéré ou supprimé (choix explicite) | — |

### Critères de sortie

- [ ] Les 3 modes se comportent conformément aux tests de `membrane_space_suite` (422 l.).
- [ ] Déposer une carte dans une membrane l'adopte ; l'invariant MEMB-1 est respecté (une membrane minimisée ne perd pas son contenu).
- [ ] Le mode Focus entre et sort avec une animation fluide.
- [ ] Membranes imbriquées sur 3 niveaux sans artefact.

---

## Phase 7 — Dossiers & miroirs

> *« La deuxième fonctionnalité identitaire — et la plus impressionnante techniquement. »*

| # | Tâche |
|---|---|
| 7.1 | Rendu d'un dossier sur le canvas (aujourd'hui : jamais dessiné) |
| 7.2 | Créer un dossier qui capture le contenu de son rectangle |
| 7.3 | Entrer / sortir, avec fil d'Ariane |
| 7.4 | Sous-board indépendant (viewport, contenu, undo cohérent) |
| 7.5 | **Miroir d'un dossier OS** : scan → arbre de nœuds |
| 7.6 | Modes `snapshot` / `live` (surveillance du système de fichiers) |
| 7.7 | Scan récursif + **scan paresseux** (tenir 49 000 fichiers sans gel) |
| 7.8 | Filtre glob |
| 7.9 | 7 modes de tri façon explorateur |
| 7.10 | Vignettes des médias du dossier |
| 7.11 | Miroirs d'annotation / image / dossier — **alias vivants** |
| 7.12 | Propagation des modifications original → miroirs |
| 7.13 | Suppression en cascade (en O(n), pas en O(n²)) |
| 7.14 | Détection de cycle anti-inception (`mirror_graph`, déjà écrit) |
| 7.15 | Marquage visuel d'un miroir + « aller à l'original » |

### Critères de sortie

- [ ] Déposer un dossier de 49 000 fichiers : **import instantané**, scan à l'entrée.
- [ ] Mode `live` : ajouter un fichier dans l'explorateur → il apparaît dans Glucose.
- [ ] Modifier un original → tous ses miroirs suivent.
- [ ] Tenter de créer un miroir cyclique → refusé avec un message clair.
- [ ] Navigation sur 5 niveaux de dossiers imbriqués, fil d'Ariane exact.

---

## Phase 8 — Flèches sémantiques

| # | Tâche |
|---|---|
| 8.1 | Dessiner une flèche par glisser |
| 8.2 | Déplacer les extrémités |
| 8.3 | Attache à un nœud, **des deux côtés** (répare R-12) |
| 8.4 | Ancrage géométrique au bord (`arrow_anchor`, déjà écrit) |
| 8.5 | Flèches courbes (Bézier) |
| 8.6 | Points de passage éditables |
| 8.7 | Bidirectionnelle, épaisseur, couleur, style |
| 8.8 | Étiquette sur la flèche + éditeur |
| 8.9 | Description longue en Markdown |
| 8.10 | **Prédicats sémantiques** : est_précurseur, contredit, hérite_de, inspire, dépend_de, illustre |
| 8.11 | Rendu différencié par prédicat (style + couleur + icône) |
| 8.12 | Attache à un sous-bloc de texte |
| 8.13 | Attache à une **sélection de texte** via ancres robustes (`text_anchors`, déjà écrit) |
| 8.14 | Flèche-portail vers un autre board |

### Critères de sortie

- [ ] Déplacer un nœud → les flèches suivent **des deux côtés**, ancrées au bord.
- [ ] Les 6 prédicats sont visuellement distincts.
- [ ] Une ancre de texte survit à l'édition du texte cible (test dédié).
- [ ] Une flèche-portail navigue vers son board cible.

---

## Phase 9 — Sémantique & organisation

| # | Bloc | Contenu |
|---|---|---|
| 9.1 | **Domaines** | CRUD, panneau, affectation pondérée, couleurs de membranes dérivées, badges, liens trans-domaines, filtrage, icônes |
| 9.2 | **Temporalité** | Ancrage, invite de saisie, règle temporelle, filtre, panneau Timeline, plages, années négatives |
| 9.3 | **Storyboard** | Panneaux, 5 ratios, grille, assignation d'images, descriptions, réordonnancement |
| 9.4 | **Presets & zones** | Presets intégrés, panneau, application à un board, zones colorées, outil de sélection de zone, `slot_id` |
| 9.5 | **Compléments texte** | Tableaux, liens cliquables, LaTeX en ligne |
| 9.6 | **Stickies** | Opérateurs logiques avec un vrai rendu (fin de `{:?}`) |
| 9.7 | **Images** | Étiquettes (tags) |

### Critères de sortie

- [ ] Un nœud dans 3 domaines pondérés colore sa membrane par mélange correct.
- [ ] La règle temporelle affiche une plage -3000 → 2026 sans artefact.
- [ ] Un storyboard 16:9 sur 3 colonnes se génère et se réordonne.
- [ ] `$\frac{a}{b}$` s'affiche correctement.

---

## Phase 10 — Export & interopérabilité

| # | Tâche | Module réutilisé |
|---|---|---|
| 10.1 | Menu d'export réel (fin du toast) | — |
| 10.2 | Construction de scène | `export::build_scene` |
| 10.3 | Export SVG | `export::scene_to_svg` |
| 10.4 | Export Markdown | `export::scene_to_markdown` |
| 10.5 | Export HTML | — |
| 10.6 | **Export PNG** via `glucose-raster` (pas de navigateur, pas de dépendance) | `glucose-raster` |
| 10.7 | Options : sélection / board / projet, échelle, fond, marges |
| 10.8 | Écriture sur disque + « révéler dans l'explorateur » |
| 10.9 | Import / export d'un board seul |
| 10.10 | Support vidéo (lecture, vignette, contrôles) |

### Critères de sortie

- [ ] Les 4 formats produisent un fichier ouvrable, fidèle au canvas.
- [ ] L'export PNG d'un board de 200 nœuds en 4K : **< 3 s**.
- [ ] `glucose-cli export` fonctionne **sans fenêtre** (utilisable en CI et en script).

---

## Phase 11 — Collaboration & rideaux

> *La phase la plus risquée. Elle vient tard **exprès**.*

| # | Bloc | Contenu |
|---|---|---|
| 11.1 | **Synchronisation** | Le journal de commandes de la phase 1 devient le protocole réseau. Ordre causal, résolution de conflits par transformation d'opérations sur le modèle en arène. |
| 11.2 | **Transport** | WebSocket maison (le protocole tient en ~300 lignes) ; salon, invitation par lien |
| 11.3 | **Présence** | Identité locale, curseurs des pairs, sélections distantes |
| 11.4 | **Assets** | Canal séparé pour les octets, transfert content-addressed |
| 11.5 | **Rideaux** | Modèle (déjà écrit), canvas de rideau, visibilité privé/partagé, droits owner/everyone, languettes nommées, ratios, visibles en mode Focus |
| 11.6 | **Time machine** | Historique nommé, prévisualisation d'une version, restauration |
| 11.7 | **Robustesse** | Reconnexion automatique, mode hors ligne, résolution à la reconnexion |

### Critères de sortie

- [ ] Deux instances éditent le même board simultanément sans divergence (test automatisé de convergence).
- [ ] Couper le réseau 30 s, éditer des deux côtés, reconnecter : convergence correcte.
- [ ] Un rideau privé n'est pas visible par les autres ; un rideau partagé l'est.
- [ ] Restaurer une version nommée ne perd rien.

---

## Phase 12 — Extensions & finition

| # | Bloc | Contenu |
|---|---|---|
| 12.1 | **Recherche** | Panneau, plein texte multi-boards, navigation vers un résultat, `Ctrl+F` |
| 12.2 | **App Bridge** | Ouvrir un fichier avec son application, overlay de lancement, résolution de chemins, vérification d'existence |
| 12.3 | **Plugins** | Système d'extensions, moteur intégré, bus d'événements, panneau, transformation texte → carte |
| 12.4 | **Pomodoro** | Timer, overlay, notifications |
| 12.5 | **Distribution** | Mise à jour de l'application, installateurs Windows / Linux / macOS |
| 12.6 | **Éditeur de syntaxe** | Coloration pour les blocs de code |
| 12.7 | **Finition** | Thème clair, accessibilité, préférences, écran de bienvenue |

### Critères de sortie

- [ ] Recherche sur un projet de 5 000 nœuds : **< 100 ms**.
- [ ] Un plugin tiers peut créer des nœuds sans accéder aux internes du document.
- [ ] Un installateur produit une application qui démarre sur une machine vierge.

---

## Les cinq règles qui font tenir le plan

### 1. Aucune phase ne commence avant que la précédente soit verte

Les critères de sortie ne sont pas indicatifs. C'est **exactement** le mécanisme qui manquait :
`smart_align` a été écrit et testé, puis on est passé à la suite sans le brancher. Un critère de
sortie « le magnétisme fonctionne » l'aurait empêché.

### 2. Une fonctionnalité n'existe que si elle est branchée

Écrire un module et le laisser hors du chemin d'exécution, c'est produire de la dette qui a
l'apparence du progrès. **Interdit** : merger un module de `glucose-model` sans son appelant.

### 3. Le modèle et l'UI avancent ensemble (loi L9)

Ajouter un champ au modèle sans le rendre ni l'éditer ni le sauvegarder, c'est R-24.
Un champ arrive avec ses trois consommateurs, ou il n'arrive pas.

### 4. Chaque phase ajoute ses tests de non-régression

- **Phase 1** : bancs de performance dans la CI, seuil qui casse la build.
- **Phase 2** : round-trip de sauvegarde, y compris interruption brutale.
- **Phase 3+** : scènes de référence rendues en PNG et comparées.

C'est ce qui aurait attrapé R-06 en 30 secondes au lieu de rester invisible.

### 5. Tu utilises Glucose pour construire Glucose

Dès la fin de la phase 2, tiens **ta propre roadmap dans Glucose**. C'est le meilleur détecteur
de friction qui existe, et ça produit un flux de priorités que personne ne peut te donner de
l'extérieur.

---

## Ce qui pourrait faire dérailler le plan

Un architecte honnête nomme les risques.

| Risque | Probabilité | Conséquence | Parade |
|---|---|---|---|
| **Le rastériseur maison prend 3× plus de temps que prévu** | Moyenne | Phase 1 bloquée | Garder `tiny-skia` derrière le trait `Canvas` jusqu'à ce que le rastériseur maison passe les tests PNG. Le remplacement devient un non-événement. |
| **Le moteur de texte est un puits sans fond** | **Élevée** | Phase 4 s'éternise | Livrer P0 (retour à la ligne, curseur, crénage) puis **s'arrêter**. Bidi/ligatures/indien sont explicitement hors périmètre jusqu'à besoin réel. |
| **La collaboration s'avère plus dure que prévu** | Élevée | Phase 11 | Elle est en phase 11 exprès. Si elle échoue, tu as déjà 93 % d'un excellent logiciel mono-utilisateur. |
| **Lassitude devant l'ampleur** | **Élevée** | Abandon | C'est le vrai risque. Parade : les phases 1 et 2 donnent des gains **visibles et immédiats** (+20 points de parité, l'app qui enregistre). Ne commence pas par le rastériseur. |
| **Le périmètre s'élargit en route** | Élevée | Dérive | Ces documents sont le périmètre. Toute idée nouvelle va dans un backlog, pas dans la phase en cours. |
| **La FFI plateforme casse sur une mise à jour d'OS** | Faible | Phase 5 | `glucose-platform` est le seul crate `unsafe` ; il est petit, isolé et testable séparément. |

---

## La séquence, en une phrase

> **Arrête de mentir sur l'état (0), branche ce qui est déjà écrit (1), apprends à enregistrer (2)** —
> et à ce moment-là tu auras un logiciel que tu peux utiliser tous les jours, à 31 % de parité,
> avec une base saine. Tout le reste, ce sont des fonctionnalités qui s'ajoutent une par une sur
> des fondations qui ne bougent plus.

---

**Suite** : [`05-STANDARDS-DE-CODE.md`](05-STANDARDS-DE-CODE.md) — les règles qui empêchent le
code de redevenir ce qu'il est aujourd'hui.
