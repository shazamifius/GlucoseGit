# Glucose — Guide d'utilisation A → Z

> **Glucose** est une surface cognitive infinie : un seul canvas, pas de modes, pour
> poser, relier, zoomer, explorer. Ce guide te ramène à la maîtrise de chaque outil
> sans avoir à relire la roadmap.

**Version :** 0.2.0
**Dernière mise à jour :** 2026-04-30

---

## 0. Vocabulaire

| Terme | Sens |
|---|---|
| **Board** | Une zone de canvas indépendante (onglet en haut). Chaque board a ses images, annotations, dossiers, viewport. |
| **Annotation** | Tout ce qui n'est pas une image : texte, sticky, flèche, membrane fixe. |
| **Sticky** | Note jaune (ou autre couleur). Idéal pour les commentaires courts. |
| **Texte** | Bloc Markdown avec rendu prose : titres, listes, LaTeX, gras, italique. |
| **Flèche** | Lien orienté entre deux nœuds (annotations ou images). Peut porter un prédicat sémantique. |
| **Membrane fixe** | Zone colorée dessinée à la main avec l'outil M. Ne dépend d'aucun cluster. |
| **Membrane implicite** | Halo coloré généré automatiquement autour d'un cluster d'images (Union-Find). |
| **Domaine** | Catégorie sémantique (Science, Art…) avec couleur et icône. Pondère les membranes et tague les nœuds. |
| **Dossier** (CanvasFolder) | Sous-canvas zoomable. Double-clic pour entrer. |
| **Miroir** (Alias ↻) | Copie vivante d'un nœud / dossier. Modifier l'original = tous les miroirs changent. |
| **LOD** | Level of Detail. Macro / Méso / Micro selon le zoom. |
| **Predicate** | Type sémantique d'une flèche : `est_precurseur`, `contredit`, `herite_de`, `inspire`, `depend_de`, `illustre`. |

---

## 1. Démarrage

```bash
bun install
bun run tauri dev
```

L'application s'ouvre. Le canvas par défaut a un board `Board principal` vide.

---

## 2. Outils (toolbar haut)

> **Règle des raccourcis** : tous les raccourcis "touche unique" (V, T, N, A, F, M, G, L) sont **inhibés quand tu tapes dans un champ texte ou un sticky** — taper le mot "Voiture" ne switche pas en Select tool. Les modifiers (Ctrl/Shift/Alt) sont aussi exclus pour éviter les conflits.

| Touche | Outil | Usage |
|---|---|---|
| `V` | Sélectionner | Mode par défaut. Cliquer/glisser pour sélectionner ; rectangle de sélection multiple. |
| `Espace` | Pan | Maintenir pour déplacer la vue. |
| `T` | Texte | Cliquer dans le canvas → crée un bloc texte éditable. |
| `N` | Note sticky | Cliquer → crée une note collante jaune. |
| `A` | Flèche (Arrow) | Cliquer-glisser depuis un nœud vers un autre. |
| `F` | Dossier (Folder) | Cliquer-glisser pour dessiner la zone du dossier. |
| `M` | Membrane | Cliquer-glisser pour dessiner une zone colorée. |

**Sélecteur de zone universel** : `F` et `M` activent automatiquement le mode `zone-select`. La toolbar montre une bannière "Glisse pour définir la zone…" et un label live `W × H` au curseur. Échap annule.

---

## 3. Raccourcis clavier complets

### Navigation / Vue
| Touche | Action |
|---|---|
| `Espace` (maintenu) | Pan |
| `Molette` | Zoom (ancré au curseur) |
| `Ctrl+Shift+F` | Fit-to-view (zoom sur tout le contenu) |
| `F11` | Mode Zen (full-screen, panels cachés). Échap pour sortir. |
| `Échap` | Ferme panels/recherche, sort du Zen, repasse en outil Sélection |
| Pavé num. `4 6 8 2` | Pan par incréments |
| Pavé num. `+` / `-` | Zoom par incréments |

### Édition
| Touche | Action |
|---|---|
| `Ctrl+Z` / `Ctrl+Y` (ou `Ctrl+Shift+Z`) | Undo / Redo (50 niveaux) |
| `Ctrl+S` | Sauvegarder le projet (.glucose) |
| `Ctrl+O` | Ouvrir un projet |
| `Ctrl+A` | Tout sélectionner (board courant) |
| `Ctrl+D` | Dupliquer la sélection |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copier / Couper / Coller |
| `Ctrl+I` | Importer des images |
| `Ctrl+F` | Recherche globale (textes, sticky, tags, boards) |
| `Suppr` / `Backspace` | Supprimer la sélection (ou le dossier sélectionné) |

### Toggles d'état (touche unique)
| Touche | Action |
|---|---|
| `L` | (Dé)verrouiller la sélection d'images |
| `G` | Activer/désactiver l'aimant (smart guides) |

### Phase 4 — Miroirs
| Touche | Action |
|---|---|
| `Ctrl+Shift+M` | Crée un **miroir** de la sélection (offset 40px). Texte, sticky, image. Sur un miroir, badge ↻ apparaît. |
| Click sur `↻` | Téléporte au nœud original avec animation (400ms). Bascule de board si nécessaire. |

### Phase 5 — Opérateurs logiques (sticky)
| Touche | Action |
|---|---|
| `Alt+1` | Transforme le(s) sticky sélectionné(s) en opérateur **ET** (vert) |
| `Alt+2` | → opérateur **OU** (bleu) |
| `Alt+3` | → opérateur **MAIS** (ambre) |
| `Alt+4` | → opérateur **PARCE QUE** (violet) |
| `Alt+0` | Retire l'opérateur (sticky standard restauré) |

---

## 4. Le LOD (zoom sémantique)

Glucose adapte automatiquement ce qui est rendu selon le zoom courant. Trois niveaux :

| Niveau | Scale | Visible |
|---|---|---|
| **Macro** | < 0.25 | Membranes colorées, dossiers (cadre + icône + compteur), pas de texte ni flèche normale. Les zones sémantiques se révèlent. |
| **Méso** | 0.25 – 0.55 | Titres tronqués (1ʳᵉ ligne), miniatures de dossiers, flèches du nœud sélectionné. |
| **Micro** | > 0.55 | Texte intégral, édition, toutes flèches au survol. |

À zoom 1.0 (par défaut) tu es **toujours en micro**. Méso est un bref intervalle de transition. Macro = très dézoomé (vue d'ensemble).

### Règle anti-spaghetti des flèches

Une flèche est rendue uniquement si **au moins une** condition est vraie :
1. Un de ses nœuds est dans la sélection courante
2. Un de ses nœuds est sous le curseur
3. C'est un lien **trans-domaines** (en pointillés) + le toggle "Trans-domaines" est actif (par défaut oui)
4. Elle est **épinglée** (`pinned: true`)

Cela élimine le plat de spaghettis classique des graphes denses.

**Toggle Trans-domaines** : bouton dans la toolbar (icône deux cercles + ligne pointillée). Quand actif, toutes les flèches qui traversent les frontières entre domaines restent visibles à tout zoom.

---

## 5. Domaines (système sémantique)

### À quoi ça sert
Donner aux **membranes** des couleurs qui reflètent le sens du contenu, pas un hash arbitraire de l'ID. Étiqueter les nœuds pour les retrouver.

### Workflow
1. Toolbar → bouton **"Domaines"** (icône triangle de cercles).
2. Clique **"+ Nouveau domaine"**. Donne un nom, une icône (emoji), choisis une couleur dans la palette.
3. Sélectionne un ou plusieurs nœuds (textes, sticky, images).
4. Le panel affiche un slider 0-100% à côté de chaque domaine. Glisse pour assigner.
5. Les **membranes** prennent automatiquement la couleur dominante des domaines de leurs nœuds (somme vectorielle pondérée sur le cercle chromatique).
6. Les nœuds avec poids > 40% affichent un **badge** (icône colorée, coin haut-droit).

### Suppression
Clique le `×` à droite du domaine. Confirmation demandée. Suppression cascade : tous les nœuds qui le portaient sont désassignés automatiquement.

---

## 6. Flèches sémantiques

Une flèche peut porter un **prédicat** (type sémantique) parmi 6 :

| Prédicat | Sens | Couleur | Glyphe |
|---|---|---|---|
| `est_precurseur` | A vient avant B | orange | → |
| `contredit` | A réfute B | rouge | ✗ |
| `herite_de` | A spécialise B | violet | ⊂ |
| `inspire` | A motive B | vert | ✦ |
| `depend_de` | A nécessite B | bleu | ⊕ |
| `illustre` | A est exemple de B | rose | ◎ |

**Affecter un prédicat** : sélectionne la flèche → barre contextuelle (ArrowOptions) → choix du prédicat. Le badge coloré apparaît au milieu de la flèche.

### Pathfinding anti-obstacles
Les flèches **courbes** détectent automatiquement les obstacles entre source et cible et contournent récursivement (algorithme `getDynamicRoute`). Si pas d'obstacle et pas de waypoint manuel, la flèche reste **droite**.

### Waypoints manuels
Sur une flèche sélectionnée :
- Clique sur le losange ◇ au milieu d'un segment → ajoute un waypoint
- Glisse le waypoint orange ● → repositionne
- Double-clic sur un waypoint → supprime

### Sub-block targeting
Trace une flèche depuis un paragraphe spécifique d'un texte vers un autre paragraphe : Glucose retient `sourceBlockId` / `targetBlockId`. Le hover surligne le paragraphe exact.

---

## 7. Dossiers zoomables

### Créer
Outil `F` → glisse pour dessiner la zone. Un dossier vide apparaît avec icône, nom modifiable, compteur "0 éléments".

### Entrer / sortir
- **Double-clic** sur le dossier → entre dedans (le board change). Le breadcrumb apparaît en haut.
- **Échap** ou clic sur breadcrumb parent → remonte d'un niveau.

### Renommer
Double-clic sur le titre du dossier → input inline. Entrée pour valider.

### Couleur
Sélectionne le dossier → barre contextuelle "Couleur du dossier" → palette.

### Aperçu live
Le contenu du dossier s'affiche en miniature dans le cadre (jusqu'à 60 items). Maj automatique à chaque modification du child board.

---

## 8. Miroirs (Alias) — Phase 4

### Concept
Un miroir est une **copie vivante** d'un nœud. Modifier l'original propage à tous les miroirs, à tout zoom, partout dans le projet. Le miroir a sa propre position mais partage le contenu.

### Créer un miroir
1. Sélectionne un ou plusieurs nœuds.
2. `Ctrl+Shift+M` → crée les miroirs avec un offset de 40px.
3. Le badge ↻ bleu apparaît sur chaque miroir (coin haut-gauche).

### Téléporter à l'original
Clique sur le badge ↻ d'un miroir → animation 400ms qui centre le viewport sur l'original. Si l'original est dans un autre board, Glucose bascule automatiquement.

### Garde-fou Inception
Le risque théorique : Dossier A contient un miroir de Dossier B, et Dossier B contient un miroir de Dossier A → entrer dans A montre B montrant A montrant B... à l'infini → crash.

Glucose **refuse net** toute création de miroir de dossier qui fermerait une boucle, à n'importe quelle profondeur. Le check est un BFS strict sur l'arbre des boards (`src/store/mirrorGraph.ts`). Si refus : warning console + aucune mutation du store.

---

## 9. Multimédia & App Bridge

### Import images
- **Glisser-déposer** un ou plusieurs fichiers depuis l'explorateur
- **Coller** une image (Ctrl+V)
- **Coller une URL** d'image → Glucose télécharge à la résolution maximale auto-détectée
- Toolbar → **"+ Images"** → sélection multiple

### Vidéos
- Glisser un `.mp4` / `.mov` / `.webm` local
- Coller une URL YouTube / TikTok / Instagram / Vimeo → Glucose lance `yt-dlp` (auto-téléchargé), import du fichier vidéo en sprite

### App Bridge (`.blend`, `.psd`, `.kra`)
Glisser un fichier source créatif → crée un nœud "Fichier Source" avec l'icône du logiciel.
**Double-clic** → ouvre dans l'application native (Blender, Photoshop, Krita, etc.).

---

## 10. Storyboard

Toolbar → **Storyboard**.
1. Choisis un ratio (16:9, 4:3, etc.), nombre de colonnes, gap.
2. Cliques sur le canvas pour placer la grille.
3. Chaque case devient un panel avec sa zone de description.
4. Drag des images dans les panels = assign automatique.
5. Réordonne les panels par drag.

---

## 11. Presets

Toolbar → **Preset**.
Templates de slots prédéfinis (`CharaDesign`, `Environment`, `Creature`, `Props`, `MoodBoard`, `Storyboard`).
Chaque preset propose des slots colorés (Ex : "Vue de face", "Vue de profil", "Détails costume"…). Glisse une image dans un slot → elle s'aligne et s'épouse automatiquement.

---

## 12. Recherche & navigation

### Ctrl+F — recherche globale
Cherche dans le contenu textuel de tous les boards : texte, sticky, tags d'images, noms de boards.
Résultats listés par board ; clic = navigue + zoom sur le résultat.

### Minimap (bas-droite)
Vue radar 180×120 px du board courant. Clique ou glisse pour naviguer. Se décale à gauche quand un panel droit est ouvert (Domaines, Presets…).

### Bookmarks de viewport
- `Ctrl+1` à `Ctrl+9` → sauve la position/zoom courante
- `1` à `9` → restaure ce viewport

---

## 13. Tags d'images

Sélectionne une image → barre contextuelle → ajoute des tags (pills cliquables).
Recherche par tag via Ctrl+F.

---

## 14. Pomodoro (concentration)

Toolbar → **Timer**.
Cycles 25/15/5 min (travail/pause/long break). Un overlay flottant reste visible si le panel est fermé. Sonne à la fin via WebAudio (silencieux pour les notifications système).

---

## 15. Color picker

Sélectionne un texte ou un sticky → barre contextuelle → roue HSV style Blender. Saturation, luminosité, alpha indépendants.

---

## 16. Export PNG

Toolbar → **Export PNG**.
Glucose rend tout le canvas en haute résolution (WebGL → binaire Tauri natif), pas le viewport courant. Tu obtiens l'image complète du board, pleine fidélité.

---

## 17. Sauvegarde / chargement

Format `.glucose` (JSON pour l'instant — Phase 7 le passera en binaire Automerge CRDT).

- `Ctrl+S` : sauve. Premier appel demande où ; ensuite réutilise le chemin.
- `Ctrl+O` : charge. Les domaines absents (projets legacy) sont auto-normalisés à `[]`.
- Fichier auto-sauvegardé périodiquement dans `~/.glucose/autosave.glucose` (à venir Phase 7).

---

## 18. Workflow type — exemple complet

**Scénario** : tu prépares un dossier sur la Révolution française.

1. **Crée 3 domaines** : `Histoire` (rouge, 📜), `Personnages` (bleu, 👤), `Documents` (vert, 📄).
2. Crée des **textes** pour chaque idée clé (causes économiques, philosophes, événements…).
3. Sélectionne un texte, ouvre Domaines, glisse à 80% sur "Histoire" et 20% sur "Personnages".
4. **Trace des flèches** entre les concepts : un texte sur Voltaire `inspire` → un texte sur les Lumières.
5. Trace une flèche entre un personnage (domaine Personnages) et un événement (domaine Histoire) → la flèche apparaît **en pointillés** (trans-domaine).
6. **Crée un dossier** `Personnages` avec l'outil F. Glisse-y tous tes nœuds Personnages.
7. **Mirror** un texte clé du dossier Personnages dans le board principal avec `Ctrl+Shift+M` → tu peux toujours le voir au niveau global, et toute modification propage.
8. **Dézoome** (molette) → tu vois maintenant uniquement les **membranes colorées** (rouge/bleu/vert mélangés) qui dessinent la topographie sémantique.
9. **Cliquer le badge ↻** d'un miroir te ramène à l'original.

---

## 19. Limites connues / à venir

| Feature | État | Phase |
|---|---|---|
| CRDT Automerge (multi-utilisateur, undo infini) | ⏳ | 7 |
| IA locale (CLIP, Moondream2, RAG) | ⏳ | 8 |
| WikiGit (registre versionné de concepts) | ⏳ | 9 |
| Animation 400ms entrée/sortie de dossier | ⏳ | 4.5 |
| Détection auto de domaine via IA | ⏳ | 8 (fallback manuel pour l'instant) |
| Hook `useZoneSelector` réutilisable | ⏳ | 1.5 |
| Distribution Linux/macOS/Android/iOS | ⏳ | 11 |

Voir [ROADMAP.md](ROADMAP.md) pour le plan complet.

---

## 20. Astuces / gotchas

- **Boîtes déformées au dézoom** : ne devrait plus arriver depuis la Phase 2 (le ResizeObserver ne persiste les dimensions QU'EN MICRO). Si ça arrive, fais Ctrl+Z pour restaurer la dernière taille saine.
- **Flèche courbe qui se courbe sans raison** : depuis la Phase 0, une flèche "courbe" sans obstacle ni waypoint reste droite. Si ça boucle, vérifie qu'elle a bien `arrowType: "curved"` dans le store.
- **Cycle de miroirs** : le check est strict mais ne couvre QUE les dossiers (les miroirs d'annotations/images ne peuvent pas créer de cycles).
- **Performance** : > 1000 nœuds, le rendu reste fluide grâce à la règle anti-spaghetti et au culling spatial. Phase 2.5 branchera le Quadtree pour aller plus loin.
- **PixiJS dev StrictMode warning** : un message console `_cancelResize is not a function` peut apparaître en dev mode. Inoffensif, captured par try/catch.

---

## 21. Architecture rapide (pour les contributeurs)

- **Front** : React 19 + Tailwind 4 + Zustand
- **Rendu raster** : PixiJS 8 (images, vidéos)
- **Rendu vectoriel** : SVG overlay synchronisé sur le viewport PixiJS (texte, flèches, dossiers, membranes fixes)
- **Backend** : Tauri 2 (Rust) — I/O fichiers, yt-dlp, ouverture native
- **Types centraux** : [src/types/index.ts](src/types/index.ts)
- **Store** : [src/store/index.ts](src/store/index.ts) — Zustand + snapshot undo/redo 50 niveaux
- **Cycle detector** : [src/store/mirrorGraph.ts](src/store/mirrorGraph.ts)
- **LOD** : [src/canvas/lod.ts](src/canvas/lod.ts) — fonction pure `computeLOD(scale)` + `shouldRenderArrow()`
- **Membranes** : [src/canvas/MembraneRenderer.ts](src/canvas/MembraneRenderer.ts) — Union-Find + Gift Wrapping + somme vectorielle de domaines

---

**Bonne navigation. Glucose, c'est juste poser, relier, zoomer, explorer.**
