# HANDOFF — Glucose (pour le prochain Claude)

> Réécrit le **2026-07-02** à la fin de la session « bundle portable ».
> Branche : **`checkpoint/avant-A`** · HEAD : **`2d2a9f1`** · **TOUT est poussé** sur `origin`.
> Ce fichier est **non suivi par git** — il ne part pas dans le repo/PR. Supprime-le quand tu veux.

---

## 0. TL;DR — où on en est

Glucose = app **Tauri v2** (Rust + WebView2) / **React 19** / **PixiJS 8** / **Zustand** / **Automerge 3** (CRDT).
North star : rendre le `.glucose` **indestructible & incorruptible** (« mieux qu'une feuille »).

- **Tout est fait, testé, poussé, CI relancée** sur la branche. Rien en attente de push.
- **État technique** : `typecheck` 0 · **437 tests TS** verts · **5 tests Rust** verts · `biome` 0 erreur
  (14 warnings `any` pré-existants) · `cargo fmt`+`clippy -D warnings` OK · build prod OK.
- Dernière grosse livraison : **le bundle portable** (déplacer un `.glucose` emporte ses images) —
  **validé en réel par l'user (« fonctionne trop bien »)** après le fix Rust.

### La première chose à faire la prochaine fois
1. **Lire `MEMORY.md` puis ce fichier** (voir §1). Rien n'est cassé, rien n'est à finir en urgence.
2. Demander à l'user quelle **route** on prend (voir §3 « reste à faire »). Les 2 candidats chauds :
   **bundle approche 2** (magasin co-localisé auto) ou **le cap nord P5** (exporteur sémantique + MCP).
3. Lui rappeler (une fois) de **révoquer son token GitHub** (voir §6).

---

## 1. D'OÙ VIENNENT MES RESSOURCES — quoi lire, dans quel ordre (⭐ ta question)

**A) Ma mémoire persistante** (auto-chargée à CHAQUE session) vit dans :
`C:\Users\Administrator\.claude\projects\c--Users-Administrator-Documents-GlucoseGit-main\memory\`
- **`MEMORY.md`** = l'index (une ligne/souvenir). **À lire EN PREMIER.**
- Chaque `*.md` = un fait. Les plus importants pour repartir :
  - **`checkpoint-avant-A-wip-state.md`** → l'état de la branche + TOUS les commits + les pièges. **Lis-le tôt.**
  - **`bundle-portable-mechanism.md`** → le dernier gros chantier (bundle) : comment il marche, ce qui reste.
  - **`compaction-mechanism.md`** → la compaction d'historique (Git#1 Phase 4-p2).
  - **`glucose-design-monochrome.md`** → RÈGLE DE DESIGN : UI noir/blanc/gris, couleur = statut seul.
  - **`indestructible-incorruptible-north-star.md`** → la vision + le plan Git#1 (4 phases, toutes faites).
  - **`glucose-hub-ia-plugin-architecture.md`** → l'archi cible 3 couches + les DEUX « git » distincts.
  - **`undo-forward-revert-wasm-panic.md`** + **`undo-architecture-invariants.md`** → l'undo (règle porteuse).
  - **`collab-automerge-repo.md`** + **`collab-silent-reconnect-disabled.md`** + **`collab-images-embed-vs-link.md`** → la collab.
  - **`graphify-architecture-map.md`** → il existe un graphe du repo dans `graphify-out/graph.json`.

**B) Les docs de VISION** (les plus à jour, écrits par/pour l'user), dans **`C:\Users\Administrator\Documents\`** :
- **`Glucose-Vision-et-Etat.md`** + **`Presentation_Glucose.md`** (26/06/2026) → l'idée, les 4 sens de
  communication Humain↔IA, les « 3 pierres » du pont IA. **À lire pour le CAP.**

**C) Dans le repo (`C:\Users\Administrator\Documents\GlucoseGit-main\`) :**
- **`ROADMAP.md`** = LE plan canonique **P1→P7** (audit 2026-06-10). Priorités, vision, fichiers critiques.
- `README.md` ; `git ls-files "*.md"` pour le reste.
- **`graphify-out/graph.json`** = carte du repo (interroge-la pour naviguer vite ; 2 chokepoints :
  `validate_scope` côté Rust, `useGlucoseStore` = store monolithe ~1900 l.).

**D) L'ÉCOSYSTÈME (4 dossiers, pas 1) — tous dans `Documents\` :**
- **`GlucoseGit-main`** = l'app (le canvas). C'est ici qu'on code.
- **`glucose-notes`** = moteur Rust séparé « texte → cours spatialisé » (cœur du pont IA→Humain), itéré
  jusqu'à v10. Cf. mémoire `glucose-notes-plugin.md`.
- **`glucose-plugins`** = packaging du plugin « Cours magistral » (déjà branché DANS l'app, Phase 8).
- **`glucose-pipeline-v1…v10`** = bancs d'essai réels (cours de neurobio de la musique).

**E) Les vrais fichiers `.glucose` de test de l'user** (pour vérifier un résultat sur disque, façon compaction) :
- `Desktop\Blender\Projet\en cours\tst.glucose` (a servi à valider la compaction).
- `Downloads\Nouveau projet-portable\` (bundle complet, 129 images — a servi à valider le bundle).

**Règle d'or mémoire** : une mémoire reflète ce qui était vrai à l'écriture. Si elle cite un
fichier/une fonction, **vérifie qu'il existe encore** avant de t'appuyer dessus. Le CODE est la vérité ;
les mémoires disent le POURQUOI non-déductible du code.

---

## 2. Ce que CETTE session a fait (6 commits, tous poussés)

Au-dessus de `14f0087` :
- **`4b95ed3`** — Git#1 Phase 4-p2 = **COMPACTION** de l'historique (poussée + **testée en réel** :
  `tst.glucose` 209 Ko → 17 Ko, −91,8 %, historique 4241→1 change, zéro perte). Cf. `compaction-mechanism`.
- **`4833439`** — **fixes review Gemini** (bot auto sur la PR #13) : `kill_on_drop(true)` sur 3 spawns Rust,
  export `toAbsolute`, `setScale` throttlé. Écartés à raison : isMounted (React 19 = no-op), IPv6 SSRF (pas une faille).
- **`9e0aecc`** — **BUNDLE PORTABLE (approche 1)** : dossier auto-suffisant `project.glucose` + `objects/<hash>`
  + `bundle.json`. UI : « Projet portable » dans ExportMenu + **Ctrl+Maj+O** pour ouvrir. +14 tests.
- **`28f6004`** — fix import : message d'erreur RÉEL (Tauri jette des **strings**, pas des `Error`).
- **`338d830`** — **⚠️ LE fix qui compte** : la copie d'assets passe **côté RUST** (`bundle_export_assets` /
  `bundle_import_assets`, disque→disque). L'ancien « tout JS via base64/IPC » **calait à ~34 images sur 129**
  (gros projet 190 Mo). **Validé en réel par l'user après rebuild.** Cf. `bundle-portable-mechanism`.
- **`2d2a9f1`** — style : icônes ExportMenu violet→gris (règle `glucose-design-monochrome`).

---

## 3. Ce qu'il RESTE à faire (rien d'urgent — proposer, laisser l'user choisir)

| Priorité | Tâche | Détail |
|---|---|---|
| ○ | **Bundle approche 2** | Magasin co-localisé auto `mon.glucose.assets/` à côté du doc (comme `.versions/`) → fichier DU QUOTIDIEN portable sans export manuel. Suite prévue de l'approche 1. |
| ⭐ | **Cap nord P5** | Exporteur sémantique `glucose → graphe lisible par une IA` (pur/testable, `buildScene` fait 80%) PUIS serveur **MCP** (l'IA lit/édite le canvas en direct). Le chaînon manquant du projet — attirant pour l'user. |
| ○ | Déclencheur **AUTO** de compaction | Au-delà d'un seuil d'historique. Reporté (manuel d'abord = plus sûr). Réutiliser `runCompaction`, gate solo. |
| ○ | **PR #13** `checkpoint/avant-A → main` | Fusionner ou non = décision user. |
| 🔐 | **Révoquer le token** GitHub | L'user l'a collé + réutilisé plusieurs fois ce jour → à révoquer ; puis `gh auth login`. |
| ○ | Bundle : inclure les jalons `.versions/` | v1 n'embarque que l'état courant. |

Voir `ROADMAP.md` (P1→P7) pour le reste. Revue stratégique complète faite ce jour (l'user a choisi de
FINIR l'arc « indestructible » avant le cap nord).

---

## 4. Organisation du code (l'essentiel)

**Pipeline de sauvegarde / Git#1** (`src/utils/`) :
- `project.ts` — `saveProject`/`loadProject` (⚠️ **`loadProject(pathArg?)`** accepte désormais un chemin →
  saute le dialogue ; utilisé par l'ouverture de bundle). Réécriture de chemins **SOLO only**.
- `saveState.ts` — PUR. `planSave`/`commitSave`/`markLoaded`. `autoVersion.ts` — jalons AUTO à l'ampleur (32 Ko).
- `versions.ts` — jalons DURABLES (`<path>.versions/`). `loadLatestHealthyVersion` = filet anti-corruption.
- `compaction.ts` — `compactDoc` (pur, roundtrip) + `runCompaction` (I/O, garde-fous solo/atomique).
- **`bundle.ts`** (NEW) — bundle portable : PUR (`collectReferencedAssets`/`buildBundleManifest`/`assetBytesMatch`)
  + `exportBundle`/`importBundle` qui appellent les commandes RUST `bundle_export_assets`/`bundle_import_assets`.
  **`bundleActions.ts`** = glue UI (dialogues + toast). Tests : `bundle.test.ts` (pur) + `bundle.integration.test.ts`.
- `assets.ts` / `assetRef.ts` — magasin global content-addressed `app_data_dir/assets/<hash16>.<ext>` ; le doc
  ne porte que des refs `asset:<name>` (mode "link" + sha256). C'est POURQUOI déplacer un `.glucose` perdait
  les images → d'où le bundle. `currentPath.ts` — singleton `getCurrentPath()`.

**Store** : `src/store/index.ts` (~1900 l., monolithe = gros chokepoint). Source de vérité = `_doc: Doc<Project>`.
`src/store/automerge.ts` = wrapper `A.*` (create/change/save/load/**loadResilient**/asPlain/…).

**UNDO (règle porteuse)** : forward-revert. `undo()` lit `A.asPlain(snapshot)` et le RÉ-APPLIQUE EN AVANT →
lignée-agnostique (c'est pourquoi la compaction ne casse pas l'undo). Jamais d'`A.change` brut sur le doc d'un handle collab.

**Rust** : `src-tauri/src/lib.rs`. Commandes clés : assets (`save_asset`/`load_asset`/`get_assets_dir`),
**bundle** (`bundle_export_assets`/`bundle_import_assets` = copie disque→disque + intégrité + `validate_scope`),
plugins (Phase 8), fetch web anti-SSRF. `validate_scope(path, app)` = frontière disque (canonicalize + roots autorisés).

**UI** : `src/components/ExportMenu.tsx` (menu Exporter + « Projet portable »), `TimelinePanel.tsx` (Time Machine +
Compacter), `PluginPanel.tsx`. `src/App.tsx` = raccourcis clavier (Ctrl+O ouvrir, **Ctrl+Maj+O ouvrir bundle**,
Ctrl+S save, Ctrl+H time machine, Ctrl+Maj+L collab).

**CI** : `.github/workflows/ci.yml` — tourne **uniquement sur `main` et PR vers `main`** (d'où la PR #13).
Frontend (typecheck+lint+vitest) + Rust (`cargo check --release` + `fmt --check` + `clippy -D warnings`, **bloquants**).

---

## 5. Commandes utiles

```bash
# Frontend (racine)
npm run typecheck        # tsc --noEmit
npm run lint             # biome check src   (⚠️ biome, PAS eslint)
npm test                 # vitest run (toute la suite, 437)
npx vitest run src/utils/bundle.test.ts   # un fichier
npm run build            # tsc && vite build (prod)
npm run tauri dev        # app desktop RÉELLE — recompile le Rust si besoin (nécessaire après un changement de commande Rust)

# Rust (via --manifest-path pour éviter un cd)
cargo fmt   --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --release -- -D warnings
cargo fmt   --manifest-path src-tauri/Cargo.toml --all            # applique le formatage
```

**Pousser** (seul moyen sur ce PC — `gh` pas connecté, helper git = GUI inutilisable en non-interactif) :
```bash
git push "https://<TOKEN>@github.com/shazamifius/GlucoseGit.git" checkpoint/avant-A 2>&1 | sed -E 's/github_pat_[A-Za-z0-9_]+/***REDACTED***/g'
git update-ref refs/remotes/origin/checkpoint/avant-A HEAD   # OBLIGATOIRE : le push par URL inline n'avance PAS origin/…
```
🔐 **Le token vient de l'user à CHAQUE fois**, JAMAIS écrit sur disque/`.git/config`, JAMAIS ré-affiché
(filtre `sed`). Ne pousse QUE quand il le demande. **Ordre de travail de l'user : coder → tester → PUIS pousser.**

---

## 6. Pièges & gotchas (mordu dessus)

1. **Copier beaucoup/de gros fichiers : JAMAIS via base64 à travers l'IPC Tauri.** L'export bundle « tout JS »
   (load_asset base64 → JS → writeFile) **calait à 34 images sur 129** (190 Mo). → faire la copie **en Rust**
   (`std::fs`/`tokio::fs`, disque→disque). Modèle : `bundle_export_assets`/`bundle_import_assets` dans lib.rs.
2. **Tauri jette des STRINGS, pas des `Error`.** `(e as Error).message` → « undefined » et cache la vraie cause.
   Utilise **`String(e)`** dans les catch d'appels Tauri.
3. **Push par URL inline n'avance pas `origin/…`** → `git update-ref refs/remotes/origin/checkpoint/avant-A HEAD`
   après, sinon `git status` ment (« en avance de N commits »).
4. **`A.change` sur le doc d'un handle collab = panic WASM** (fatal). Toute mutation gatée `!getCollabHandle()`.
   `A.save`/`A.asPlain` (lecture) sont sûrs en collab.
5. **Design : noir/blanc/gris SEULEMENT**, couleur = statut (vert=ok, rouge=pas ok). Pas de violet/bleu déco
   (cf. `glucose-design-monochrome`). Palette grise : texte `#d4d4dd`, secondaire `#7d7d8c`, icône `#9a9aa0`,
   bordures `#26262e`/`#34343e`, fonds `#16161a`/`#23232b`.
6. **Nouvelle commande Rust = rebuild** (`npm run tauri dev` recompile). Un simple HMR frontend ne suffit pas.
7. **Tests : pas de `@types/node`.** Pour tester du code plugin-fs, **mocker le module avec une Map** (modèle :
   `bundle.integration.test.ts`, `compaction.integration.test.ts`). Pour un test JETABLE qui lit un vrai fichier
   disque, `import { readFile } from "node:fs"` marche sous vitest (esbuild ne typecheck pas) — supprime-le après.
8. **CI ne voit la branche que via la PR #13** (workflow sur `main`/PR→main). Un push seul ne déclenche RIEN d'autre.
9. **Seuil auto-version = 32 Ko** de delta (images = liens `asset:` quasi gratuits → doc grossit lentement).
10. **PowerShell = shell primaire** (Windows). Bash dispo. `git commit -F -` + heredoc `<<'EOF'` pour les messages
    multi-lignes (le heredoc évite les soucis d'accents/quotes). `$var` est mangé par bash si on lance
    `powershell.exe` via l'outil Bash → utilise l'outil PowerShell directement.

---

## 7. Bosser avec l'user (shazamifius, FR)

- **Parle français.** Direct, collaboratif, **honnête sur prouvé vs supposé** (il remercie pour ça). Distingue
  🟢 testé-réel / 🟡 logique-testée-I/O-mockée / 🔵 typé-buildé / 👤 validé-par-lui.
- Il aime **détailler un chantier AVANT de coder**, puis choisir une option (souvent via une question à choix).
  Mais quand il dit « **en autonomie** » / « **go** » → **fonce et code**, ne redemande pas.
- **Il veut que tu TESTES DE TON CÔTÉ d'abord**, avant de lui demander de tester. Tu ne pilotes pas la fenêtre
  Tauri, MAIS tu peux : lancer les tests, ET **inspecter/rejouer sur les vrais fichiers disque** (charger un
  `.glucose`, recalculer des hash, rejouer un algo en Node) — c'est comme ça qu'on a validé compaction ET bundle.
- **Ordre** : coder → tester → **et SEULEMENT après on pousse** (quand il le dit).
- **CI 100 % vert, zéro X.** Ne casse jamais ça.
- **North star** = boussole : « mieux qu'une feuille », indestructible & incorruptible. Chaque décision Git#1 se
  juge là-dessus (jalon avant tout risque, vérifier avant de remplacer, écriture atomique).
- Ne re-narre pas ce qui est établi : agis. `/graphify` existe (skill) ; `graphify-out/graph.json` = carte du repo.

---

_Fin du handoff. Tout est vert, tout est poussé. Bonne session._
