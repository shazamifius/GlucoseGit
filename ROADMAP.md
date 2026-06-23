# Glucose — Roadmap & Architecture

> **Vision :** une surface cognitive infinie. Une seule interface — pas de modes — capable de soutenir aussi bien la création d'un jeu vidéo, l'élaboration d'une langue construite, que la cartographie versionnée de toute la connaissance humaine.
>
> **Mission (cap nord) :** devenir **la nouvelle feuille blanche entre humains** — la meilleure façon de partager une idée, qu'elle soit artistique, technique, scolaire ou mathématique. Et, plus tard, **le meilleur pont entre l'humain et l'IA** : un espace 2D que l'IA peut générer ET relire, et que l'humain comprend d'un regard.
>
> **Principe :** poser, relier, zoomer, explorer. Rien d'autre.

**Dernière mise à jour :** 2026-06-10 (audit complet du code)
**Version :** 1.0.1-beta.1 · **Tests :** 370 verts / 26 fichiers (vérifié 2026-06-10)
**Architecture :** Tauri 2 (Rust) + React 19 + Tailwind 4 + PixiJS 8 (raster) + SVG overlay (vecteur) + Zustand + Automerge 3 (CRDT, WASM)

---

## 📍 Instantané — où en est Glucose

| Domaine | État |
|---|---|
| Canvas, annotations, flèches typées, domaines, temporel | ✅ stable |
| Dossiers zoomables, miroirs, navigation par zoom | ✅ stable |
| CRDT Automerge, Time Machine, undo infini | ✅ stable |
| Sauvegarde : binaire v2, **incrémentale**, autosave, **versions durables**, chargement résilient | ✅ stable |
| Collaboration internet (chaîne `automerge:…`) | ✅ MVP — images non synchronisées, pas de présence |
| Performance (centaines d'images fluides) | ✅ stable |
| Export **HTML / PNG / SVG / Markdown** | ✅ livré |
| Miroir du système de fichiers (drop d'un dossier OS) | ✅ livré (R-FIL) |
| **Système de plugins + IA locale Ollama** | ✅ **bêta** (plugin n°1 : Cours magistral) |
| Mode web / PWA | 🟡 embryon (`platform.ts`, bannière web) |
| RAG local, recherche sémantique, auto-domaines | ⬜ pas commencé |
| WikiGit / registre de concepts | ⬜ pas commencé |
| Mobile (Android / iOS) | ⬜ pas commencé |

---

## Décisions cadres

- **CRDT Automerge** pour le canvas vivant ; **Git réel** (plus tard, WikiGit) pour les concepts publics partagés. Frontière étanche.
- **L'IA décide le SENS, le code décide la GÉOMÉTRIE.** Règle d'or héritée du moteur `glucose-notes` : un modèle ne donne jamais de coordonnées ; le placement est toujours calculé par du code déterministe.
- **Pas de modes UI.** L'interface reste identique pour tous les usages.
- **La donnée de l'utilisateur passe avant tout** : un `.glucose` doit être incassable (chargement résilient, versions durables, autosave).
- Les **ports typés** (WikiGit) complèteront les prédicats sémantiques — ils ne les remplaceront pas.

## Lois du rendu

> ⚠️ Mise à jour 2026-06-10 : la « Loi du Zoom Sémantique » (LOD macro/méso/micro) et la règle anti-spaghetti des flèches ont été **abandonnées en Phase 7.5** après 10+ heures d'usage terrain (« on ne voit rien »). Le rendu est désormais pleine fidélité à tout zoom. Restent en vigueur :

1. **Loi de la Connexion Latente** — un lien existe toujours dans la donnée ; les liens trans-domaines (pointillés) peuvent être masqués par toggle.
2. **Loi du Domaine Coloré** — un nœud appartient à 1..N domaines pondérés. Membrane = signature primaire, badge = lève l'ambiguïté.
3. **Symbiose chromatique** — sans couleur explicite, la teinte d'une annotation émerge de sa position et de ses voisines ([symbioticHue.ts](src/utils/symbioticHue.ts)).

---

## ✅ Acquis (historique condensé)

### Canvas & annotations (Phases 1–6)
Pan/zoom infini 60 FPS, multi-boards, texte Markdown+LaTeX, sticky (+ opérateurs logiques AND/OR/BUT/BECAUSE), flèches typées (6 prédicats, badges, description longue dépliable, flèches-portails inter-boards), membranes manuelles et implicites (Union-Find + couleurs dérivées des domaines), sélecteur de zone, domaines pondérés avec badges, réglette temporelle zoomable (30 époques nommées, `Shift+R`/`Shift+T`), recherche globale `Ctrl+F`, minimap, Zen mode, Pomodoro, OrganizePanel.

### Dossiers & miroirs (Phases 4 & 7.5)
Dossiers = sous-canvas complets, capture automatique au drag-create, **navigation par zoom** (entrer en zoomant, sortir en dézoomant, indicateur de bordure colorée), breadcrumb façon VSCode avec sauts entre frères, preview riche. Miroirs (alias vivants) d'annotations/images/dossiers avec **garde-fou anti-Inception** (check acyclique [mirrorGraph.ts](src/store/mirrorGraph.ts)), badge ↻, téléportation vers l'original.

### CRDT, Time Machine & sauvegarde (Phase 7 — 2026-05/06)
- Store **CRDT-first** : le doc Automerge est la source de vérité, ~30 actions via `mutate()`, undo/redo infini.
- **Time Machine** (`Ctrl+H`) : slider d'historique, preview live, restauration non destructive, jalons nommés 📌.
- Format `.glucose` v2 **binaire Automerge** ; migration v1 JSON transparente.
- **Enregistrement incrémental** (2026-06-06) : `Ctrl+S` n'écrit que le delta (`append_glucose_binary`), compaction auto, **`loadResilient`** : fin de fichier corrompue → on recharge le plus grand préfixe sain. Jamais de fichier illisible.
- **Autosave** débouncé 1,5 s, actif solo et collab ([useAutosave.ts](src/utils/useAutosave.ts)).
- **Versions durables** : chaque jalon est AUSSI écrit comme un `.glucose` complet et indépendant dans `<fichier>.versions/` (`<time>__<manuel|auto>__<slug>.glucose`) — le filet de secours si le doc vivant casse ([versions.ts](src/utils/versions.ts)).

### Collaboration internet (Phase 7.6 — 2026-06-05)
`automerge-repo` + serveur de synchro always-on + persistance IndexedDB. Créer/rejoindre une **chaîne** par code `automerge:…` (`Ctrl+Shift+L`), catch-up automatique, undo local seulement (forward-revert). Remplace le multijoueur LAN mDNS (7.5bis, code conservé dans [multiplayer.rs](src-tauri/src/multiplayer.rs)).
**Limites MVP :** images non transférées (leçon du 2026-06-06 : ré-embarquer les octets dans le doc = freeze, annulé), pas de curseurs/présence, serveur public par défaut.

### Performance images (2026-06-05)
Assets sur disque (`asset:<hash>`, dédup SHA-256) hors du doc Automerge, virtualisation des textures selon le viewport, downscale adapté au zoom (decode hors thread), rendu à la demande (GPU au repos), abonnements store ciblés, minimap en cache. Des centaines d'images fluides.

### Multimédia, App Bridge & miroir de fichiers
Drag-drop images (upgrade auto résolution CDN : Pinterest, ArtStation…), vidéos locales et YouTube/TikTok/Instagram via yt-dlp embarqué (SHA-256 vérifié), fichiers créatifs (`.blend`, `.psd`, `.kra`…) → sticky source avec icône (30+ extensions) et ouverture native whitelistée.
**R-FIL — miroir du système de fichiers** : drop d'un dossier OS → arbre de CanvasFolders navigable, fichiers en tuiles-launchers, scan paresseux par niveau, tri façon explorateur, médias affichés sans gonfler le `.glucose` ([folderMirror.ts](src/canvas/folderMirror.ts)).
**NAV-2** : geste molette/pavé tactile classé zoom ou pan façon Google Maps (pincement = zoom, deux doigts = pan) ([navigation.ts](src/canvas/navigation.ts)).

### Export (livré — remplace l'ancien PNG unique)
Menu « Exporter ▾ » : **HTML interactif** (1 fichier, pan/zoom dans le navigateur, partage zéro-install), **PNG HD** plein-board, **SVG** vectoriel, **Markdown** structuré ([ExportMenu.tsx](src/components/ExportMenu.tsx), [export.ts](src/utils/export.ts)).

### 🧩 Plugins & IA locale (bêta — anciennement « idée future », désormais réel)
- **Côté Rust** : `list_plugins` / `install_plugin` / `run_plugin` (binaire compagnon + `manifest.json` dans `app_data_dir/plugins/<id>/`), `system_specs` (sonde RAM/cœurs/VRAM → modèle Ollama recommandé), `ollama_status`, `install_ollama` (winget), `pull_model` avec progression.
- **Côté UI** : [PluginPanel.tsx](src/components/PluginPanel.tsx) — installation depuis un dossier, options déclaratives rendues automatiquement (enum/bool), barre de progression par passe du moteur, gestion Ollama intégrée (détection, installation, téléchargement de modèle).
- **Plugin n°1 : « Cours magistral »** — texte long → cours spatialisé en 2D (moteur `glucose-notes`, pipeline multi-passes : nettoyage → triage → extraction → architecture thèmes+liens → géométrie). Le résultat se charge comme un nouveau board, sans écraser le travail en cours.

### Sécurité (Sprint 1 — 2026-05-07)
9 vulnérabilités critiques fermées (RCE, XSS, SSRF, scope FS), validation Zod des fichiers chargés, clamps de coordonnées, capabilities Tauri minimales, CI + Biome.

---

## 🚧 Dette & bugs connus

- [ ] **App Bridge** : `.blend`/`.psd`/`.kra` parfois mal affichés (sticky vide) ; double-clic n'ouvre pas toujours l'app native. *Piste : logger le chemin reçu, vérifier `sourceFile` absolu.*
- [ ] **README** : badge « 304 tests » périmé → 370.
- [ ] **macOS** : app non signée (Gatekeeper → clic-droit Ouvrir).
- [ ] Feedback toast manquant : création dossier, ajout image, import vidéo, application preset (reliquat BUG-5).
- [ ] Animation de capture au drag-create d'un dossier (polish 7.5.1).
- [ ] Refonte des membranes auto/manuelles (évoquée sur le terrain, 7.5.2).
- [ ] Le mode web/PWA existe en embryon mais n'est ni buildé ni documenté.

---

# 🔭 Les chantiers à venir

> Ordonnés par priorité proposée. Les items marqués 🆕 sont des **propositions nouvelles** (audit 2026-06-10) — à valider, amender ou jeter. Le reste vient des plans existants.

## P1 — Consolidation : sortir de la bêta (2-4 semaines)

> Avant d'empiler des features : faire de la 1.0 une version qu'on recommande les yeux fermés.

- [ ] Corriger les bugs App Bridge (affichage + ouverture native)
- [ ] 🆕 **Archive `.glucose` portable** : un format « tout-en-un » (doc + assets content-addressed, zip ou dossier `objects/`) pour qu'un projet survive au changement de machine. *C'est aujourd'hui la plus grosse surprise négative possible pour un utilisateur : il déplace son `.glucose`, les images ne suivent pas.*
- [ ] 🆕 **Onboarding première ouverture** : un board d'exemple pré-rempli (3 cartes, 1 flèche typée, 1 dossier) plutôt qu'un canvas vide — Glucose se comprend en le voyant, pas en lisant.
- [ ] 🆕 Mettre à jour README (badge tests, section plugins, exports 4 formats)
- [ ] 🆕 **Signature de code** : macOS notarization + signature Windows quand le budget existera ; en attendant, documenter proprement le contournement.
- [ ] 🆕 Page « Releases » avec captures animées (GIF de 10 s : poser/relier/zoomer) — le produit est visuel, sa vitrine doit l'être.

## P2 — Collaboration complète (4-6 semaines)

> Le plan « sans budget » (analyse 2026-06-06) reste le bon. Ordre d'attaque proposé :

- [ ] **Canal d'assets hors-document** ⭐ priorité absolue : fetch des images manquantes par `sha256` (style Git-LFS) via le relais — c'est LE manque qui rend la collab décevante aujourd'hui (les images ne se transfèrent pas).
- [ ] **Relais always-on gratuit** : Oracle Cloud Always Free (VM ARM à vie) ou Cloudflare Durable Objects/PartyKit — remplacer le serveur public de test, URL dans [repo.ts](src/multiplayer/repo.ts).
- [ ] **Chiffrement E2E** des changements côté client (la clé vit dans le code de partage) → relais non fiable acceptable, vie privée réglée. *Référence : Ink & Switch « Beehive ».*
- [ ] **Compaction d'historique partagé** : un nouvel arrivant ne télécharge pas tout l'historique (snapshots périodiques côté relais).
- [ ] P2P WebRTC en plus du relais (automerge-repo accepte plusieurs adaptateurs simultanés) — rapide quand les pairs sont en ligne ensemble, zéro charge serveur.
- [ ] Curseurs / présence temps réel des pairs (avatars).
- [ ] 🆕 **Mode « spectateur »** : un lien en lecture seule (pour partager une carte sans risquer qu'on l'édite) — petit effort, gros usage : c'est le « voici ce que je pense » envoyable à n'importe qui.

## P3 — Plugins v2 : du binaire au langage commun (4-8 semaines)

> Le système actuel (binaire + manifest) marche. Prochaine marche : en faire une vraie plateforme.

- [ ] 🆕 **Spécifier le contrat plugin** dans un `PLUGIN.md` : entrées (texte/fichier), sorties (`.glucose`), options déclaratives, événements de progression. C'est la porte d'entrée des contributeurs externes.
- [ ] 🆕 **Sortie en *board fusionnable*** : aujourd'hui un plugin produit un projet qu'on charge ; demain il devrait pouvoir produire un board qui se **fusionne** dans le projet courant (le CRDT le permet naturellement).
- [ ] 🆕 **Plugin « Carte de conversation »** : le mode `mapspace` du moteur existe déjà (idées → embeddings → PCA 2D → clusters) mais n'est pas exposé comme plugin. C'est le complément naturel du « Cours magistral » : moins linéaire, plus spatial.
- [ ] 🆕 **Imports conversationnels** : claude.ai / ChatGPT / Gemini via leurs exports (le moteur ne lit que les sessions Claude Code locales aujourd'hui).
- [ ] 🆕 **Plugin « Wikipédia »** : un article + ses liens → une carte navigable (premier pas concret vers le rêve « remettre Wikipédia dans Glucose » du README).
- [ ] 🆕 **Registre de plugins** (simple : un repo GitHub avec des releases + vérification de hash à l'installation) — éviter le far-west des binaires.
- [ ] 🆕 Sandbox/permissions des plugins (un binaire tiers = du code arbitraire ; au minimum : avertissement clair + hash vérifié + scope disque restreint).

## P4 — IA dans Glucose : RAG local & assistance (6-8 semaines)

> L'infrastructure Ollama est en place (installation, sonde matérielle, pull de modèles). S'en servir DANS le canvas, pas seulement dans les plugins.

- [ ] **Recherche sémantique** : embeddings des cartes (Ollama embed local) + index vectoriel → « retrouve l'idée sur l'architecture japonaise d'il y a 3 mois ». *Simplification 🆕 : commencer avec un index en mémoire sauvé dans `app_data_dir` plutôt que d'embarquer qdrant — des milliers de cartes tiennent sans base dédiée.*
- [ ] **Détection automatique de domaines** (clustering des embeddings → propositions de domaines, l'humain valide) — ferme le reliquat de Phase 3.
- [ ] **Légendes & tags automatiques d'images** (modèle vision local type Moondream) — alimente la recherche.
- [ ] 🆕 **« Suggère des liens »** : sélectionner une carte → l'IA propose 3 flèches typées vers des cartes existantes (avec le prédicat et une justification d'une ligne). L'humain accepte/refuse. *C'est l'assistance la plus « Glucose » qui soit : elle travaille sur les relations, pas sur le contenu.*
- [ ] 🆕 **Résumé de board** : un board entier → une carte de synthèse (l'inverse du plugin Cours magistral).
- [ ] Watchdogs (worker qui surveille un dossier/domaine et pose des post-its d'alerte ambre, jamais bloquants).
- [ ] Ligne de commande visuelle (« range tout ce qui est bleu à gauche ») — après le reste : spectaculaire mais moins structurant.

## P5 — Le pont humain ↔ IA (le cap nord — R&D continue)

> La thèse de Glucose : l'espace 2D est le **langage intermédiaire** entre le latent space de l'IA et l'œil humain. Le sens IA→humain existe (plugins). Tout ce qui suit construit le sens **retour** et le dialogue.

- [ ] 🆕 **Exporteur `glucose → graphe structuré`** ⭐ le chaînon manquant : sérialiser un board en JSON sémantique (nœuds, positions relatives, prédicats, domaines, dates, clusters de membranes) pensé pour être LU par un modèle. Sans lui, toute la communication reste à sens unique.
- [ ] 🆕 **Serveur MCP Glucose** : exposer le projet courant via Model Context Protocol (`read_board`, `add_card`, `link_cards`, `move_card`…). N'importe quel agent (Claude, autre) peut alors lire ET modifier le canvas en direct, avec l'humain qui voit chaque geste. *C'est LA pièce qui transforme Glucose en « plateforme de communication homme-machine » concrète — et le standard existe déjà, pas besoin de l'inventer.*
- [ ] 🆕 **Dialogue spatial v0 (expérience)** : une session où humain et IA éditent le même board à tour de rôle — l'IA via MCP, l'humain au canvas — sur un sujet donné. Mesurer : est-ce qu'on se comprend mieux qu'en chat texte ? C'est l'expérience fondatrice du projet, elle doit exister tôt, même moche.
- [ ] 🆕 **Projection améliorée** dans le moteur de cartes : remplacer PCA par UMAP (préserve les voisinages locaux → les grappes de sens restent des grappes à l'écran).
- [ ] 🆕 **Faire dialoguer deux IA** à travers un board (rêve du README) : techniquement = deux clients MCP sur la même chaîne collab. Devient *trivial* une fois MCP + collab assets livrés — bel exemple de chantiers qui se composent.
- [ ] Apprentissage spatial bidirectionnel (ex-Phase 10) : fine-tuning d'embeddings sous contrainte « proches sur le canvas → proches dans l'espace » et projection inverse. *Reste de la R&D longue ; ne bloque rien d'autre.*
- [ ] 🆕 Veille active (le domaine bouge vite) : Platonic Representation Hypothesis, vec2vec (traduction entre espaces d'embeddings), interprétabilité mécaniste — le « hub latent transvasable » n'est pas pour tout de suite, mais le jour venu il se branchera sur l'exporteur ci-dessus.

## P6 — Plateformes & distribution (parallélisable)

- [ ] **Web/PWA** : finir le mode web embryonnaire (lecture seule d'abord — ouvrir un `.glucose` exporté/partagé dans le navigateur, sans installation). 🆕 *Priorité montée : c'est le meilleur canal de découverte du produit, et `platform.ts` + l'export HTML font déjà 60 % du chemin.*
- [ ] Android / iOS (`tauri android|ios init`) — après le web.
- [ ] Import Obsidian (vaults → boards) ; export PDF storyboard.
- [ ] Compte + cloud sync chiffré (Supabase / Cloudflare R2+D1) — seulement quand la collab P2 est solide.
- [ ] 🆕 Mises à jour automatiques (tauri-plugin-updater) — indispensable dès que la base utilisateurs grandit.

## P7 — WikiGit / Registre de concepts (après P2-P5)

> Inchangé sur le fond (crate `git2`, concepts = dossiers versionnés, forks = branches, ports typés, registre public à terme). 🆕 **Proposition de re-priorisation** : passer APRÈS le pont humain↔IA — le MCP et l'exporteur sémantique serviront de fondation au registre (un concept WikiGit est exactement un « nœud lisible par l'IA »), pas l'inverse.

---

## 💡 Backlog d'idées (non priorisées)

Mode présentation (parcours guidé de nœuds, comme des slides) · thème clair · raccourcis personnalisables · géolocalisation cognitive (concepts sur une carte du monde) · narration liquide (chemin de nœuds → document linéaire) · langage gestuel (communiquer avec l'IA par gestes de souris) · import de flux temps réel (météo, données scientifiques) · export `.ase`/CSS variables · 🆕 mode focus/litterature (suite de cartes en colonne lisible) · 🆕 templates de boards communautaires · 🆕 statistiques de projet (cartes/liens/domaines dans le temps, via l'historique Automerge déjà présent).

---

## 📋 Fichiers critiques (mis à jour 2026-06-10)

| Fichier | Rôle |
|---|---|
| [src/types/index.ts](src/types/index.ts) | Types centraux (`Project`, `Annotation`, `BoardImage`, `CanvasFolder`) |
| [src/store/index.ts](src/store/index.ts) | Store Zustand CRDT-first, `mutate()`, undo/redo |
| [src/store/automerge.ts](src/store/automerge.ts) | Wrapper Automerge (save/load/merge/viewAt/history) |
| [src/canvas/GlucoseCanvas.tsx](src/canvas/GlucoseCanvas.tsx) | Cœur du rendu PixiJS, navigation, virtualisation textures |
| [src/canvas/folderMirror.ts](src/canvas/folderMirror.ts) | Miroir du système de fichiers (R-FIL) |
| [src/utils/plugins.ts](src/utils/plugins.ts) + [src/components/PluginPanel.tsx](src/components/PluginPanel.tsx) | Système de plugins + gestion Ollama |
| [src/utils/export.ts](src/utils/export.ts) | Export HTML/PNG/SVG/Markdown |
| [src/utils/versions.ts](src/utils/versions.ts) | Jalons durables (`.versions/`) |
| [src/utils/saveState.ts](src/utils/saveState.ts) + [useAutosave.ts](src/utils/useAutosave.ts) | Save incrémental + autosave |
| [src/multiplayer/repo.ts](src/multiplayer/repo.ts) + [collabBridge.ts](src/multiplayer/collabBridge.ts) | Collaboration internet (automerge-repo) |
| [src-tauri/src/lib.rs](src-tauri/src/lib.rs) | 24 commandes Tauri (fichiers, assets, plugins, Ollama, scan FS) |

---

## 🧪 Recette de validation continue

| Chantier | Test de bout en bout |
|---|---|
| P1 archive portable | Sauver un projet avec 20 images, le copier sur une autre machine, l'ouvrir : tout s'affiche |
| P2 assets collab | A pose 10 images, B (autre réseau) les voit apparaître ; B ferme, rouvre : tout est là |
| P3 plugin fusionnable | Lancer « Cours magistral » sur un projet existant : le cours arrive en nouveau board, rien d'écrasé |
| P4 recherche sémantique | 200 cartes, requête en langage naturel → les 5 bonnes cartes remontent |
| P5 MCP | Depuis Claude Code : lire le board, ajouter une carte liée par `illustre` → visible instantanément au canvas |
| P5 dialogue spatial | 30 min humain+IA sur un même board : le compte-rendu est-il plus clair qu'un chat équivalent ? |

Tests automatisés : `npm run typecheck` · `npm test` (vitest, 370) · `npm run lint` (Biome).

---

## 📚 Notes techniques

| Sujet | Détail |
|---|---|
| StrictMode | Désactivé intentionnellement (`main.tsx`) — évite la double-init PixiJS |
| Format sauvegarde | `.glucose` v2 binaire Automerge, append incrémental + compaction, `loadResilient` |
| Undo/redo | Infini via CRDT (structural sharing — 50 snapshots ≠ 50× la mémoire) |
| Canvas | PixiJS 8 WebGL (raster) + SVG overlay vectoriel synchronisé |
| IA locale | Ollama (installation/sonde/pull intégrées) ; modèle recommandé selon RAM/VRAM |
| Plugins | Binaire compagnon + `manifest.json` dans `app_data_dir/plugins/<id>/` |
| Collab | automerge-repo + WebSocket + IndexedDB ; LAN mDNS legacy conservé ([multiplayer.rs](src-tauri/src/multiplayer.rs)) |
| Import vidéo | yt-dlp binaire pinné (SHA-256), spawné via Tauri |
| Sécurité | Zod sur tout fichier chargé, whitelist extensions App Bridge, scope checks FS, anti-XSS/SSRF |

---

## Contribuer

Cette roadmap évolue. Retours et propositions bienvenus via les [Issues](../../issues) et [Discussions](../../discussions). Les items 🆕 sont des propositions ouvertes — c'est exactement là que la discussion est utile.
