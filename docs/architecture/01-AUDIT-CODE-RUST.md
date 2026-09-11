# 01 — Audit du code Rust actuel

> **Portée** : `crates/glucose-core` (5 546 l.) + `crates/glucose-desktop` (3 466 l.).
> **Méthode** : lecture intégrale du code, `cargo check --workspace --all-targets` (0 erreur),
> vérification des sémantiques de `tiny-skia` dans la source vendue du crate.
> **Audit initial** : commit `7a13c86` — **re-vérifié** au commit `81aea31`.
>
> Chaque constat porte un identifiant stable (`R-xx`), une gravité, un emplacement exact,
> et un correctif. **Aucun constat n'est une supposition** : tout ce qui est affirmé ici a été
> vérifié dans le code ou dans la source de la dépendance concernée.

> ### 🔍 Vérification indépendante au commit `3bd9bda`
>
> `cargo check` : 0 erreur. `cargo test --workspace` : **230 tests, 15 binaires, tous verts**.
> `cargo clippy --workspace --all-targets -- -D warnings` : **code de sortie 0**.
>
> **Les 3 « mensonges » du tour précédent ont été traités, et bien traités :**
>
> | Constat | État vérifié |
> |---|---|
> | **R-34 — DPI** | ✅ **RÉSOLU.** `scale()`, `topbar_height()`, `tabs_height()`, `header_height()` ; layouts, marges **et polices** (`14.0 * s`) scalés ; `dock.rs` reçoit `scale: f32` ; `renderer` utilise `ui.header_height()`. Test à 150 % qui valide dimensions **et** hit-testing. |
> | **R-37 — `dock.rs` géométrie double** | ✅ **RÉSOLU.** `WidgetRect` + `layout_organize_panel` → `sort_buttons: Vec<OrganizeSortButton>`, consommé par le rendu (l. 987) **et** par le clic (l. 1724). Le `len() as f32 * 6.0` a disparu. Test UTF-8 à l'échelle 1,0 **et** 1,5. |
> | **R-38 — boucle d'animation** | ✅ **RÉSOLU, et exactement comme il fallait.** Plus de `mark_dirty()` inconditionnel ; curseur repeint uniquement au basculement de phase (2 fps) ; **plateau du toast à 0 repaint** ; Pomodoro sur changement de seconde ; `ControlFlow::Wait` au repos. |
> | **R-40 — typographie** | ✅ **RÉSOLU sur les 4 points.** LRU réelle (compteur d'accès mis à jour **aussi sur hit**), `Rc` au lieu d'`Arc`, `data_mut()` hissé hors des boucles, `measure_text` via `horizontal_line_metrics().new_line_size`. |
> | **R-35 — erreurs** | ⚠️ **PARTIEL, mais nettement avancé** *(revérifié à `138357c`)*. `DesktopError` câblé (5 sites dans `clipboard.rs`, toasts `⚠️` visibles) ✅. `let _ =` : 7 → **3, tous en tests** ✅. `.unwrap()` hors tests : **0** ✅. `CoreError`/`CoreResult` sont désormais **réellement câblés** : **32 références dans 6 fichiers** (`store.rs`, `store/{annotations,boards,catalog,folders,navigation}.rs`) ✅ — `try_remove_board`, `try_update_annotation` et le catalogue retournent de vraies erreurs typées. **Reste** : le chargement d'images de `renderer.rs` échoue toujours en silence, et aucune barre de statut n'affiche ces erreurs à l'utilisateur. |
> | **R-36 — thème** | ⚠️ **PARTIEL.** `dock.rs` 134 → **41** littéraux (93 usages du thème), `ui.rs` 35 → **4** (43 usages) ✅. **Mais `renderer.rs` est intact : 37 littéraux, 3 usages**, et **`Theme::light()` n'est jamais appelé** — thème clair mort-né. |
> | **R-39 — allocations** | ✅ **RÉSOLU** *(revérifié à `138357c`)*. Cache de teintes corrigé (`retain` + `get_mut`) ✅. `query_rect_refs() -> HashSet<&str>` utilisé **des deux côtés** : `renderer.rs:279` **et** `hit_priority/candidates.rs:321` ✅ — la version `String` a disparu du chemin chaud. `index_board()` n'est plus un rebuild : `sync_at` compare l'entrée mise en cache et n'appelle `upsert` que si le nœud a changé de cellule, et le balayage O(n) n'a lieu **que** si un nœud a disparu ✅. **Réserve honnête** : la traversée reste O(n) sur le nombre de nœuds à chaque changement de `store.version`, même si chaque nœud ne coûte plus qu'une comparaison. Invérifiable à 10 000 nœuds tant que le banc de la tâche 0.8 n'existe pas. |
>
> **Nouveau constat : R-41** — la taille des fichiers et des fonctions continue de croître.
>
> **Bilan : 7 critères de sortie de la phase 1 sur 9 sont verts.** Les 2 restants ne sont pas
> « presque faits » : ils sont **non mesurables** faute d'outillage (tâches 0.5, 0.6, 0.8).

> ### 🔍 Vérification précédente au commit `8444e8b`
>
> **Tout ce qui suit a été relu ligne à ligne dans le code, pas sur déclaration.**
> `cargo check` : 0 erreur. `cargo test --workspace` : **224 tests, 14 suites, tous verts**.
> `cargo clippy --workspace --all-targets` : **12 warnings**.
>
> **Réellement fait et vérifié (14)** : 1.3, 1.5, 1.6, 1.7, 1.9, 1.11, 1.15, 1.17, 1.18,
> 1.20, 1.21, 1.22, 1.25, 1.26 — plus R-12 (flèches aux deux bouts) et R-27 (alpha prémultiplié).
> `app.rs` est passé de **1 126 à 331 lignes**, avec un module `interactions/` propre.
> C'est du vrai travail d'architecture.
>
> **Annoncé fait, mais faux (3)** — voir R-34, R-35, R-36 :
>
> | Tâche | Annoncé | Réalité vérifiée |
> |---|---|---|
> | 1.10 / R-16 — DPI | « appliqué dynamiquement » | `ui.scale_factor` est **écrit deux fois et jamais lu**. Zéro effet. |
> | 1.23 / R-21 — erreurs | « hiérarchie `GlucoseError`… » | `CoreError` + `DesktopError` = 115 lignes avec **0 référence hors de leur propre fichier**. |
> | 1.24 / R-31 — thème | « structure `Theme` et jetons » | `Theme` lu **3 fois dans 1 fichier**. **211 littéraux de couleur hors thème**, dont 134 dans `dock.rs`. |
>
> **Partiellement fait (4)** : 1.4 (cache OK mais `n` allocations/frame), 1.8 (unique mais O(n)
> par id), 1.12 (`WaitUntil` OK mais boucle à 33 fps), 1.14 (cache OK mais éviction totale).
>
> **Nouveaux constats introduits par ces commits** : **R-34 → R-40** (voir section dédiée).
> Le plus grave est **R-37** : `dock.rs` (1 710 lignes, écrit après l'audit) reproduit R-20
> à l'identique, avec un bug de divergence confirmé.

> ### ⚠️ Re-vérification au commit `81aea31`
>
> Deux commits (`2da029f`, `81aea31`) sont arrivés pendant la rédaction de cet audit.
> **Quatre constats ont été corrigés entre-temps**, et sont marqués ✅ **CORRIGÉ** ci-dessous
> avec le commit qui les a réglés :
>
> | Constat | État | Par |
> |---|---|---|
> | R-02 — grille en O(surface) | ✅ **CORRIGÉ** — pas adaptatif + un seul `fill_path` | `81aea31` |
> | R-05 — aucun culling | ✅ **CORRIGÉ** (niveau naïf) — rejet par frustum dans les 4 passes | `81aea31` |
> | R-06 — transform des images inversée | ✅ **CORRIGÉ** — `from_scale().post_translate()` | `81aea31` |
> | R-10 — sélection élastique inerte | ✅ **CORRIGÉ** — le rectangle sélectionne | `2da029f` |
> | R-20 — géométrie d'UI en double | 🟡 **PARTIEL** — `layout_topbar()` + 2 tests pour la barre ; **les onglets restent en double** |
>
> **Deux constats ont empiré** : `window_event` est passé de 490 à **728 lignes** (R-19), et les
> appels directs à `redraw()` de 30 à **45** (R-15).
>
> Les 27 autres constats sont **inchangés**, re-vérifiés un par un dans le code actuel.

---

## Verdict en une page

Le code compile, il est commenté en français, il a des tests. Ce n'est pas du code « débutant ».
Le problème n'est pas la syntaxe, c'est **l'architecture**. Trois maladies :

| # | Maladie | Symptôme que tu ressens |
|---|---------|-------------------------|
| 1 | **Le noyau est débranché de l'application** | Tu écris des algorithmes propres et testés, et l'app ne les utilise pas. 13 modules sur 18 sont morts. |
| 2 | **La logique métier vit dans le gestionnaire d'événements** | `app.rs` fait 1 029 lignes et mélange fenêtre, souris, clavier, presse-papiers, décodage d'images et algorithme de mise en page. Toute modification touche tout. |
| 3 | **L'UI calcule sa géométrie deux fois** | Le dessin et le clic sont deux fonctions séparées avec les mêmes nombres magiques recopiés. Elles ont déjà divergé → des boutons qui ne cliquent pas là où ils sont dessinés. |

À cela s'ajoutait un trio de fautes de performance de classe algorithmique. Deux viennent d'être
corrigées (grille en O(surface), absence de culling) ; **la troisième reste** : la teinte
symbiotique est recalculée en O(n) **par nœud visible, par frame, deux fois**. Et surtout,
**l'absence totale de persistance** demeure.

**Le chiffre qui résume tout : l'application ne sait pas enregistrer.** Il n'existe aucune
écriture de projet dans tout `glucose-desktop`. Tout ce que tu poses sur le canvas disparaît à
la fermeture de la fenêtre. Ce n'est pas encore un logiciel, c'est une démo.

---

## Table des constats

### Bloquants — l'app est inutilisable ou perd des données
- **R-01** — Aucune persistance
- **R-02** — La grille de points est en O(surface) — ✅ **CORRIGÉ** (`81aea31`)
- **R-03** — Teinte symbiotique en O(n²) par frame, deux fois — 🟡 **(traité à vérifier)**
- **R-04** — L'undo clone le projet entier, blobs compris, 200 fois — 🟡 **(traité à vérifier)**
- **R-05** — Aucun culling : tout est dessiné même hors écran — ✅ **CORRIGÉ** (`81aea31`)

### Majeurs — comportement faux, visible par l'utilisateur
- **R-06** — Images placées au mauvais endroit (transform inversée) — ✅ **CORRIGÉ** (`81aea31`)
- **R-07** — Les onglets ne cliquent pas où ils sont dessinés — 🟡 **(traité à vérifier)**
- **R-08** — La minimap est morte (code inatteignable) — 🟡 **(traité à vérifier)**
- **R-09** — Le moteur d'alignement n'est jamais appelé — 🟡 **(traité à vérifier)**
- **R-10** — La sélection élastique n'est jamais appliquée — ✅ **CORRIGÉ** (`2da029f`)
- **R-11** — `organize_layout` mélange deux conventions de coordonnées — 🟡 **(traité à vérifier)**
- **R-12** — Les flèches ne suivent que leur source, jamais leur cible — 🟡 **(traité à vérifier)**
- **R-13** — Dupliquer deux fois crée deux éléments de même id — 🟡 **(traité à vérifier)**
- **R-14** — Les ids de boards se recollisionnent après suppression — 🟡 **(traité à vérifier)**
- **R-15** — Le curseur ne clignote pas, les toasts ne disparaissent pas — 🟡 **(traité à vérifier)**
- **R-16** — Aucune gestion du DPI — 🟡 **(traité à vérifier)**
- **R-17** — Pas d'IME : pas d'accents au clavier mort, pas de CJK

### Structurels — la dette qui te fait perdre des heures
- **R-18** — 12 modules du noyau sur 20 sont du code mort — ⚠️ *(remesuré à `e2cd410` : 2 branchés, 2 modules créés)*
- **R-19** — `app.rs` est un objet-dieu — 🟡 **(traité à vérifier)**
- **R-20** — L'UI calcule sa géométrie deux fois — 🟡 **(traité à vérifier)**
- **R-21** — Les erreurs sont jetées à la poubelle — 🟡 **(traité à vérifier)**
- **R-22** — Toute recherche est un scan linéaire
- **R-23** — Le modèle répète 14 champs par variante
- **R-24** — Des champs du modèle sont ignorés au rendu — 🟡 **(traité à vérifier)**
- **R-25** — Deux énumérations `ActiveTool` concurrentes — 🟡 **(traité à vérifier)**

### Rendu & typographie
- **R-26** — Aucun cache de glyphes — 🟡 **(traité à vérifier)**
- **R-27** — Le mélange alpha du texte est faux — 🟡 **(traité à vérifier)**
- **R-28** — Pas de retour à la ligne, pas de crénage, pas de shaping
- **R-29** — Le cache d'images est sans limite et sans mipmaps — 🟡 **(traité à vérifier)** *(cache négatif)*
- **R-30** — Décodage complet d'une image juste pour lire ses dimensions — 🟡 **(traité à vérifier)**
- **R-31** — Le thème est ~130 nombres magiques éparpillés — 🟡 **(traité à vérifier)**

### Vérité du dépôt
- **R-32** — Le message de commit ne correspond pas au code
- **R-33** — 10 boutons sur 19 ne font qu'afficher un toast

### Introduits après la première passe
- **R-34** — Le DPI est câblé mais jamais appliqué — 🟡 **(traité à vérifier)**
- **R-35** — Les types d'erreur sont du code mort neuf — 🟡 **(traité à vérifier)**
- **R-36** — Le thème est ignoré par le code écrit en même temps que lui — 🟡 **(traité à vérifier)**
- **R-37** — `dock.rs` reproduit R-20, avec un bug de divergence confirmé — 🟡 **(traité à vérifier)**
- **R-38** — La boucle d'animation repeint tout l'écran 33 fois par seconde — 🟡 **(traité à vérifier)**
- **R-39** — Allocations par frame dans le cache de teintes et l'index spatial — 🟡 **(traité à vérifier)**
- **R-40** — Défauts ponctuels de la typographie — 🟡 **(traité à vérifier)**
- **R-41** — Les fichiers et les fonctions continuent de grossir — 🟡 *(inversé dans `glucose-core` seulement)*

### Introduits par la mesure du budget de frame (`def3800`)
- **R-42** — Les docks sont le premier poste de rendu (43 % de la frame) et ne changent jamais
- **R-43** — `draw_grid` reconstruit un `PathBuilder` complet à chaque frame (14 % de la frame)
- **R-44** — 15 lints clippy désactivés à l'échelle du crate, dont celui qui aurait évité le gel — ⚠️ **PARTIEL** *(`glucose-desktop` propre ; `glucose-core` en éteint encore 3)*

### Introduits par la confrontation au code TypeScript
- **R-45** — Le texte ne suit pas le zoom : 11 `clamp` bornent le contenu, pas la boîte
- **R-46** — Les glyphes sont posés à des positions entières tronquées (flou, tremblement)
- **R-47** — Le panneau DOMAINES écrit dans une liste fantôme, jamais dans le document

---

## Bloquants

### R-01 — Aucune persistance

**Gravité : BLOQUANT.** C'est le constat le plus grave du document.

Il n'existe, dans tout `crates/glucose-desktop`, **aucune écriture ni lecture de projet**.
Le seul accès disque du crate est [app.rs:238](../../crates/glucose-desktop/src/app.rs#L238),
qui écrit une image collée dans `%TEMP%`. Pas de `Ctrl+S`, pas de `Ctrl+O` de projet, pas de
format de fichier, pas d'autosave, pas de récupération après crash.

`glucose-core` ne sait pas non plus sérialiser : `Project` ne dérive aucun trait de
sérialisation, il n'y a ni `to_bytes` ni `from_bytes`. `bundle.rs` sait calculer un SHA-256 et
décoder du base64, mais n'assemble aucun conteneur.

**Conséquence** : chaque session repart de zéro. Aucun test utilisateur réel n'est possible.
Aucun bug de « gros projet » n'est observable, parce qu'aucun gros projet ne peut exister.

**Correctif** : phase 2 de la roadmap, **avant toute nouvelle fonctionnalité**.
Voir `02-ARCHITECTURE-CIBLE.md` § « Format `.glucose` v2 ».

---

### R-02 — La grille de points est en O(surface) — ✅ CORRIGÉ

> **Résolu par `81aea31`.** Le pas est maintenant adaptatif (`while effective_step * scale < 32.0 { *= 2 }`),
> tous les points vont dans **un seul `PathBuilder`** et un seul `fill_path`, et les points hors
> écran sont ignorés. Le nombre de points est désormais **borné à ~1 300 quel que soit le zoom**,
> avec **1 allocation de chemin par frame** au lieu de 7,2 millions.
>
> **Reste à faire (mineur)** : la grille est toujours recalculée intégralement à chaque frame.
> Une tuile 256×256 mise en cache par palier de zoom la rendrait quasi gratuite.
>
> *L'analyse ci-dessous est conservée : elle documente la faute, et sert de référence si le
> problème réapparaît.*

**Gravité initiale : BLOQUANT.** [renderer.rs:157-190](../../crates/glucose-desktop/src/renderer.rs#L157-L190) *(numérotation du commit `7a13c86`)*

```rust
let grid_step = 60.0;                       // ← pas FIXE en unités monde
while gx <= end_x {
    let mut gy = start_y;
    while gy <= end_y {
        let mut pb = PathBuilder::new();     // allocation
        pb.push_circle(sx as f32, sy as f32, 1.2);
        if let Some(path) = pb.finish() {     // allocation
            pixmap.fill_path(&path, &dot_paint, ...);  // rastérisation anti-aliasée
        }
        gy += grid_step;
    }
    gx += grid_step;
}
```

Les bornes viennent de `screen_to_world`, donc **le nombre de points croît comme l'inverse du
carré du zoom**. Le zoom descend jusqu'à `0.01`
([canvas.rs:23](../../crates/glucose-desktop/src/canvas.rs#L23)).

| Zoom | Monde visible (fenêtre 1440×900) | Points dessinés | Allocations / frame |
|------|----------------------------------|----------------:|--------------------:|
| 1.0  | 1 440 × 900                      | 24 × 15 = **360** | 720 |
| 0.25 | 5 760 × 3 600                    | 96 × 60 = **5 760** | 11 520 |
| 0.05 | 28 800 × 18 000                  | 480 × 300 = **144 000** | 288 000 |
| 0.01 | 144 000 × 90 000                 | 2 400 × 1 500 = **3 600 000** | **7 200 000** |

À `0.01`, c'est un gel de plusieurs dizaines de secondes — **par frame**. Et cette frame est
déclenchée à chaque mouvement de souris (voir R-15). Rien n'est mis en cache : les mêmes points
sont recalculés intégralement au mouvement suivant.

**Correctif**
1. Pas de grille **adaptatif** : `step = 60 · 2^round(log2(1/scale))`, pour que le pas *à l'écran*
   reste dans `[20 px, 80 px]`. Le nombre de points devient **constant** (~1 500) à tout zoom.
2. Ne plus construire un `Path` par point : un point de 1,2 px de rayon est un motif 3×3
   pré-calculé une fois, écrit directement dans le buffer.
3. Mieux encore : rendre la grille dans une tuile de 256×256 une fois par niveau de zoom, puis
   la répéter. Coût quasi nul.

---

### R-03 — Teinte symbiotique en O(n²) par frame, deux fois

**Gravité : BLOQUANT — toujours ouvert.**
[renderer.rs:228](../../crates/glucose-desktop/src/renderer.rs#L228) et
[renderer.rs:548](../../crates/glucose-desktop/src/renderer.rs#L548)

> **Re-vérifié au commit `81aea31`** : les **deux** sites d'appel subsistent. Le culling ajouté
> par ce commit atténue le symptôme — le coût passe de O(n²) à **O(visible × n)** — mais
> n'attaque pas la cause. Sur un board de 1 000 cartes dont 50 visibles, cela reste
> **100 000 évaluations par frame** (2 × 50 × 1 000), chacune avec 4 appels à `sin()`,
> **pour un résultat identique à la frame précédente**.

`get_symbiotic_hue(ann, &board.annotations)` parcourt **toutes** les annotations pour calculer
la moyenne circulaire du voisinage. Elle est appelée :

- une fois par annotation `Text` dans `draw_halos` ;
- une fois par annotation `Text` dans `draw_annotations`.

Soit **2·n²** évaluations par frame. Chaque évaluation appelle `get_zone_hue`, qui fait 4 appels
à `random2d`, qui contient un `f64::sin()` — la fonction transcendante la plus lente du lot.

| Cartes texte | Appels `sin()` par frame |
|-------------:|-------------------------:|
| 50           | 20 000 |
| 200          | 320 000 |
| 500          | 2 000 000 |
| 1 000        | **8 000 000** |

Un canvas Glucose sérieux, c'est plusieurs centaines de cartes. À 500 cartes l'app est déjà
figée — et ce, **sans qu'aucune donnée n'ait changé** : la teinte d'une carte immobile est
recalculée à l'identique 60 fois par seconde.

**Correctif** : la teinte est une **fonction pure de (position, id, positions des voisines)**.
Elle doit être mémorisée dans le document, invalidée uniquement quand une annotation bouge,
naît ou meurt — et le recalcul limité aux annotations dans un rayon de 1 200 px de celle qui a
bougé, trouvées via l'index spatial (qui existe déjà : R-18).
Coût réel : **0 appel par frame** en régime stationnaire, quelques dizaines pendant un drag.

---

### R-04 — L'undo clone le projet entier, blobs compris, 200 fois

**Gravité : BLOQUANT.** [store.rs:247-254](../../crates/glucose-core/src/store.rs#L247-L254)

```rust
pub fn push_undo(&mut self) {
    if self.in_live_edit { return; }
    self.undo_stack.push(self.project.clone());   // ← clone PROFOND du projet entier
    if self.undo_stack.len() > self.max_undo {
        self.undo_stack.remove(0);                 // ← O(n) memmove de 200 Project
    }
    self.redo_stack.clear();
}
```

`Project` contient [`blobs: HashMap<String, Vec<u8>>`](../../crates/glucose-core/src/types.rs#L523)
— **les octets bruts des images**. `max_undo = 200`.

Un projet modeste de 40 images à 2 Mo = 80 Mo de blobs. Chaque frappe, chaque ajout, chaque
déplacement pousse une copie. Après 200 actions : **16 Go de pile d'undo**. Bien avant ça,
l'application aura été tuée par l'OS.

Trois défauts empilés :

1. **Snapshot au lieu de journal** — coût mémoire en O(taille du document × profondeur) au lieu
   de O(taille de l'édition × profondeur).
2. **Les binaires sont dans le document** — ils ne devraient jamais entrer dans un snapshot.
3. **`Vec::remove(0)`** au lieu d'une `VecDeque` — chaque éviction déplace 199 `Project`.

Corollaire : chaque `add_image` appelle `push_undo`. Déposer 20 images d'un coup
([app.rs:184](../../crates/glucose-desktop/src/app.rs#L184)) crée **20 snapshots complets** et
laisse **une seule image sélectionnée**, parce que `add_image` appelle `select_image(id, false)`
qui vide la sélection ([store.rs:329](../../crates/glucose-core/src/store.rs#L329)).

**Correctif** : undo par **journal de commandes inversibles**.
Voir `02-ARCHITECTURE-CIBLE.md` § « Undo : journal, pas snapshot ».

---

### R-05 — Aucun culling : tout est dessiné même hors écran — ✅ CORRIGÉ

> **Résolu par `81aea31`** au niveau naïf : un rejet par frustum a été ajouté dans les quatre
> passes (`draw_halos` l. 218, `draw_membranes` l. 278, `draw_images` l. 443,
> `draw_annotations` l. 542). Les éléments hors écran ne sont plus rastérisés.
>
> **Reste à faire (structurel)** : chaque passe **itère toujours sur tous les nœuds** pour les
> rejeter un par un. C'est O(n) par frame, pas O(visible). À 100 000 nœuds, on paie
> 400 000 tests de rectangle par frame. La version définitive interroge l'index spatial
> (`quadtree.rs`, toujours mort — R-18) et n'itère que sur ce qui est visible.

**Gravité initiale : BLOQUANT.** `draw_images`, `draw_annotations`, `draw_membranes` et
`draw_halos` itéraient sur la **totalité** du board sans aucun test de visibilité.

Sur un canvas de 2 000 éléments dont 15 sont à l'écran, on paie pour 2 000. Pour les images, on
paie en plus une `draw_pixmap` avec filtrage bilinéaire complet, dont `tiny-skia` ne peut pas
deviner qu'elle est entièrement clippée.

Le comble : **l'index spatial qui règle exactement ce problème existe déjà**, testé, dans
[quadtree.rs](../../crates/glucose-core/src/quadtree.rs) — et il n'est appelé nulle part (R-18).

**Correctif** : `SpatialHash::query_ids(viewport)` en tête de chaque passe de dessin.
Une ligne. Le gain est immédiat et massif.

---

## Majeurs

### R-06 — Images placées au mauvais endroit (transform inversée) — ✅ CORRIGÉ

> **Résolu par `81aea31`** — [renderer.rs:453](../../crates/glucose-desktop/src/renderer.rs#L453)
> est désormais `Transform::from_scale(scale_x, scale_y).post_translate(sx, sy)`, ce qui est
> l'ordre correct.
>
> **Ce constat est le meilleur argument du dossier en faveur des tests de rendu.** Le bug était
> invisible à la lecture (le code *semblait* juste), silencieux à la compilation, et n'a été
> attrapé que par la vérification de la sémantique de `post_scale` dans la source de `tiny-skia`.
> **Une comparaison de PNG de référence l'aurait signalé en 30 secondes.** C'est pourquoi la
> phase 0 de la roadmap met en place ce test.

**Gravité initiale : MAJEUR.** *(code du commit `7a13c86`)*

```rust
let ts = Transform::from_translate(sx as f32, sy as f32)
    .post_scale(sw / loaded_pixmap.width() as f32, sh / loaded_pixmap.height() as f32);
pixmap.draw_pixmap(0, 0, loaded_pixmap.as_ref(), &pp, ts, None);
```

**Vérifié dans la source de `tiny-skia-path-0.11.4`** :
`post_scale(sx,sy)` ≡ `post_concat(from_scale(sx,sy))` ≡ `concat(scale, self)` — l'échelle est
appliquée **après** la translation, et **multiplie donc la translation**. Le test unitaire du
crate le confirme (`transform.rs:493-495`) : une transform de translation `(1.2, 3.4)` devient
`(2.4, -13.6)` après `post_scale(2.0, -4.0)`.

L'image atterrit donc à `(sx · sw/w_src, sy · sh/h_src)` au lieu de `(sx, sy)`.

**Ce n'est correct que si l'image est affichée exactement à sa taille source en pixels.**
Or l'import réduit systématiquement les grandes images à 600 px de côté
([app.rs:194-201](../../crates/glucose-desktop/src/app.rs#L194-L201)) : une photo de 1200 px
importée est affichée avec un facteur 0,5, donc **dessinée à la moitié de sa position**.
Et comme le facteur dépend du zoom, l'image **glisse** quand tu zoomes.

**Correctif** : `Transform::from_scale(kx, ky).post_translate(sx, sy)`.

---

### R-07 — Les onglets ne cliquent pas où ils sont dessinés

**Gravité : MAJEUR — toujours ouvert.**
[ui.rs:496](../../crates/glucose-desktop/src/ui.rs#L496) contre
[ui.rs:882](../../crates/glucose-desktop/src/ui.rs#L882)

```rust
// Dessin  (l. 496) :
let (tw, _) = typo.measure_text(&board.name, 12.0, is_active);   // gras si actif
// Clic    (l. 882) :
let (tw, _) = typo.measure_text(&board.name, 12.0, false);       // JAMAIS gras
```

> **Re-vérifié au commit `81aea31`.** Ce commit a introduit `layout_topbar()` — une vraie
> fonction de mise en page qui produit une liste de boutons, consommée par le dessin **et** par
> le clic, avec deux tests qui vérifient qu'aucun bouton ne se chevauche à 8 résolutions.
> **C'est exactement le bon correctif** (loi L4).
>
> Mais il n'a été appliqué qu'à la **barre d'outils**. La **barre d'onglets** garde ses deux
> mesures divergentes, et le bug est intact. C'est la démonstration parfaite que R-20 est un
> problème d'architecture et non de vigilance : tant qu'il reste **un** endroit où la géométrie
> est calculée deux fois, le bug y survit.

L'onglet actif est dessiné en gras — donc plus large — mais mesuré en maigre pour le test de
clic. Le décalage s'accumule : **tous les onglets situés après l'onglet actif ont une zone de
clic décalée vers la gauche**. Plus le nom est long, plus l'erreur est grande. Cliquer sur le
4ᵉ onglet peut sélectionner le 3ᵉ.

C'est le symptôme direct de R-20. Ce n'est pas une inattention isolée : **c'est ce que
l'architecture actuelle rend inévitable**.

---

### R-08 — La minimap est morte (code inatteignable)

**Gravité : MAJEUR.**

`handle_ui_click` contient une branche minimap
([ui.rs:845-859](../../crates/glucose-desktop/src/ui.rs#L845-L859)) qui retourne
`UiAction::MinimapPan`. Mais son unique appelant ne l'invoque **que si le clic est dans
l'en-tête** :

```rust
// app.rs:475
if my < TOTAL_HEADER_HEIGHT {
    if let Some(action) = handle_ui_click(mx, my, ...) { ... }
}
```

La minimap est dessinée en **bas à droite** (`mm_y = h - mm_h - 16.0`). La branche `else` de
`handle_ui_click` est donc **strictement inatteignable**. `UiAction::MinimapPan` n'est jamais
produit. La minimap est un dessin décoratif.

Et si on la rebranchait telle quelle, elle serait fausse : le calcul de cible
([ui.rs:854-855](../../crates/glucose-desktop/src/ui.rs#L854-L855)) suppose un monde de
2 000 unités en dur, alors que le rendu calcule les vraies bornes `min_x/max_x` du board.
Les deux ne se parlent pas — encore R-20.

Note annexe : `render_minimap` n'affiche **que les images**, jamais les annotations, membranes
ou dossiers. La minimap d'un board fait de cartes texte est vide.

---

### R-09 — Le moteur d'alignement n'est jamais appelé

**Gravité : MAJEUR.**

`smart_align.rs` fait 458 lignes, expose `snap_move`, `snap_resize`, `snap_point`,
`collect_align_targets`, et possède 346 lignes de tests d'intégration. C'est du bon travail.

Dans `app.rs`, `active_guides` n'est **jamais** assigné à autre chose que `SnapGuides::default()`
— les seules occurrences sont la déclaration (l. 44), l'initialisation (l. 90), le passage au
renderer (l. 156) et une remise à zéro (l. 788). **Aucun appel à `snap_move`.**

Résultat : le bouton « Aimant » bascule un booléen, affiche « ✨ Aimant activé », et **ne fait
rien**. `draw_guides` dessine toujours une liste vide.

---

### R-10 — La sélection élastique n'est jamais appliquée — ✅ CORRIGÉ

> **Résolu par `2da029f`.** [app.rs:819-850](../../crates/glucose-desktop/src/app.rs#L819-L850)
> teste maintenant l'intersection du rectangle avec les images et les annotations, avec un
> seuil anti-clic de 3 px. Le rectangle sélectionne enfin.

**Gravité initiale : MAJEUR.** `selection_box` était créée au clic dans le vide, mise à jour au
mouvement, dessinée par `draw_selection_box`… et jetée au relâchement :

```rust
if self.selection_box.is_some() {
    self.selection_box = None;      // ← on jette, sans rien sélectionner
}
```

---

### R-11 — `organize_layout` mélange deux conventions de coordonnées

**Gravité : MAJEUR.** [app.rs:314](../../crates/glucose-desktop/src/app.rs#L314) contre
[app.rs:332](../../crates/glucose-desktop/src/app.rs#L332) et
[app.rs:346](../../crates/glucose-desktop/src/app.rs#L346)

```rust
// Images — ancrées au CENTRE :
img.x = cur_x + img.width / 2.0;
// Annotations — ancrées en HAUT-GAUCHE :
*x = cur_x;
```

C'est correct au regard du modèle (`BoardImage` est centré, les annotations sont en haut-gauche),
mais cela révèle le vrai problème : **le modèle a deux conventions de coordonnées**, et chaque
fonction qui touche aux deux doit s'en souvenir. Le renderer y pense
([renderer.rs:394](../../crates/glucose-desktop/src/renderer.rs#L394) : `x - w/2.0`), la minimap
y pense, mais `create_folder` [n'y pense pas](../../crates/glucose-core/src/store.rs#L676) : il
teste `inside(img.x, img.y)` sur le **centre** et `inside(ann.x(), ann.y())` sur le **coin** —
deux sémantiques de capture différentes pour le même geste.

C'est une **classe de bugs**, pas un bug. Il faut une convention unique.

---

### R-12 — Les flèches ne suivent que leur source, jamais leur cible

**Gravité : MAJEUR.** [store.rs:528-537](../../crates/glucose-core/src/store.rs#L528-L537)

```rust
if let Annotation::Arrow { ref source_id, ref mut x, ref mut y, .. } = a {
    if source_id.as_deref() == Some(id) {
        *x = nx;  *y = ny;
    }
}
```

Aucun traitement symétrique de `target_id` / `x2` / `y2`. Déplacer le nœud cible d'une flèche
laisse la pointe en arrière.

Pire : `move_selected` — le chemin réellement emprunté pendant un drag — ne fait **aucun** suivi
de flèche. Il translate seulement les flèches déjà sélectionnées. Déplacer une carte reliée à
une flèche casse visuellement le lien.

---

### R-13 — Dupliquer deux fois crée deux éléments de même id

**Gravité : MAJEUR.** [store.rs:447](../../crates/glucose-core/src/store.rs#L447),
[460](../../crates/glucose-core/src/store.rs#L460), [465](../../crates/glucose-core/src/store.rs#L465)

```rust
dup.id = format!("{}-dup", dup.id);
```

`duplicate_selected` resélectionne les copies, donc `Ctrl+D` deux fois d'affilée donne `x-dup`
puis `x-dup-dup` : ça passe. Mais **resélectionner l'original et dupliquer à nouveau crée un
second `x-dup`**. À partir de là, tous les `find(|a| a.id() == id)` du store — c'est-à-dire la
sélection, le déplacement, la suppression, l'édition — touchent arbitrairement l'un des deux.

Même faute dans `mirror_annotation` ([l. 566](../../crates/glucose-core/src/store.rs#L566)) et
`mirror_folder` ([l. 793](../../crates/glucose-core/src/store.rs#L793)) :
`format!("{}-mirror", ...)`.

**Correctif** : générateur d'id monotone unique par document, jamais dérivé d'un autre id.

---

### R-14 — Les ids de boards se recollisionnent après suppression

**Gravité : MAJEUR.** [store.rs:636](../../crates/glucose-core/src/store.rs#L636) et
[store.rs:665](../../crates/glucose-core/src/store.rs#L665)

```rust
let id = format!("board-{}", self.project.boards.len() + 1);
```

Scénario : 3 boards (`main`, `board-2`, `board-3`). On supprime `board-2` → il en reste 2.
On crée un board → `len()+1` = **3** → `board-3`, **déjà pris**.

Dès lors, `active_board()` fait `iter().find()` et retourne le premier : le second `board-3`
devient inaccessible, ses onglets pointent vers le mauvais contenu, et `build_folder_stack`
peut construire des chaînes de parenté fausses.

`create_folder` utilise la même formule pour `child_board_id`, avec la même faille.

---

### R-15 — Le curseur ne clignote pas, les toasts ne disparaissent pas

**Gravité : MAJEUR.** [main.rs:14](../../crates/glucose-desktop/src/main.rs#L14)

```rust
event_loop.set_control_flow(ControlFlow::Wait);
```

`Wait` signifie : ne rien faire tant qu'aucun événement OS n'arrive. Or :

- `TextEditSession::blink_timer` est lu **au moment du dessin**
  ([renderer.rs:530](../../crates/glucose-desktop/src/renderer.rs#L530)) ;
- `Toast::is_expired` / `Toast::alpha` de même
  ([ui.rs:59-68](../../crates/glucose-desktop/src/ui.rs#L59-L68)).

Personne ne planifie de réveil. **Sans mouvement de souris, le curseur ne clignote jamais et le
toast reste affiché indéfiniment.** L'animation de fondu (400 ms) n'est visible que si tu bouges
la souris pile à ce moment-là.

Le symétrique est aussi vrai et aussi grave : **chaque événement provoque un repaint intégral
synchrone**. `redraw()` est appelé directement depuis les handlers — **45 occurrences** dans
`app.rs` au commit `81aea31`, contre 30 au commit `7a13c86` — au lieu de
`window.request_redraw()`. Un simple survol de la barre d'outils
([app.rs:445](../../crates/glucose-desktop/src/app.rs#L445)) redessine la totalité de la scène.

> **Re-vérifié au commit `81aea31` : ce constat a empiré.** Les correctifs de performance des
> deux derniers commits (grille adaptative, culling) réduisent le coût *d'une* frame — mais
> tant que chaque événement souris en déclenche une **complète et synchrone**, le gain est
> partiellement consommé. C'est le correctif à plus fort effet de levier restant :
> il multiplie l'efficacité de tous les autres.

**Correctif** : `ControlFlow::WaitUntil(prochaine_échéance_d_animation)`, **un seul** point de
rendu (`RedrawRequested`), et des handlers qui se contentent de marquer `dirty`.

---

### R-16 — Aucune gestion du DPI

**Gravité : MAJEUR.**

La fenêtre est créée en `LogicalSize(1440, 900)` mais tout le reste travaille en pixels
physiques : `window.inner_size()` (physique), `position` de `CursorMoved` (physique), et
**toutes** les constantes d'UI (`TOPBAR_HEIGHT: f32 = 44.0`, polices 12–15 px, boutons 30×30).

Sur un écran à 150 % — courant sous Windows 11 — la barre fait 44 px physiques ≈ 29 px logiques :
texte minuscule, cibles de clic trop petites. À 200 % c'est illisible.
Aucun `scale_factor()`, aucun `ScaleFactorChanged`.

---

### R-17 — Pas d'IME : pas d'accents au clavier mort, pas de CJK

**Gravité : MAJEUR.** [app.rs:797-880](../../crates/glucose-desktop/src/app.rs#L797-L880)

La saisie repose uniquement sur `WindowEvent::KeyboardInput` + `Key::Character`.
`WindowEvent::Ime` n'est jamais traité, `window.set_ime_allowed(true)` jamais appelé.

Sur un clavier français, et pour une app qui se veut internationale :

- les **touches mortes** (`^`, `¨`) ne composent pas — `^` puis `e` ne donne pas `ê` ;
- aucune saisie chinoise, japonaise ou coréenne ;
- aucun retour visuel de composition (texte de pré-édition souligné).

Par ailleurs le curseur est indexé en **octets**, avec un parcours manuel de `is_char_boundary`.
Ça évite le panic sur UTF-8, mais reste faux pour les **grappes de graphèmes** : un emoji
composé (`👨‍👩‍👧`) ou un `e` + accent combinant se supprime morceau par morceau, en laissant des
demi-caractères à l'écran.

---

## Structurels

### R-18 — 12 modules du noyau sur 20 sont du code mort

**Gravité : STRUCTUREL — c'est la cause directe de ta frustration.**

Vérification par recherche des références `glucose_core::<module>` depuis `glucose-desktop` :

| Module | Lignes | Utilisé par l'app ? |
|--------|-------:|---------------------|
| `types` | 597 | ✅ 18 références |
| `layout` | 375 | ✅ 12 références *(module créé depuis)* |
| `store` | 96 + 9 modules | ✅ 5 références |
| `smart_align` | 458 | ✅ 5 références — **branché depuis** (R-09 corrigé) |
| `symbiotic_hue` | 169 | ✅ 2 références |
| `hit_priority` | 136 + 3 modules | ✅ 2 références |
| `quadtree` | 451 | ✅ 1 référence — **branché depuis** |
| `error` | 58 | ✅ 1 référence *(module créé depuis)* |
| `membrane_space` | 773 | ❌ **mort** |
| `membrane_focus` | 438 | ❌ **mort** |
| `export` | 430 | ❌ **mort** |
| `bundle` | 354 | ❌ **mort** |
| `timeline` | 256 | ❌ **mort** |
| `membrane_stretch` | 231 | ❌ **mort** |
| `curtain_model` | 216 | ❌ **mort** |
| `geometry` | 215 | ❌ **mort** |
| `text_anchors` | 211 | ❌ **mort** |
| `curtain_panel` | 201 | ❌ **mort** |
| `arrow_anchor` | 166 | ❌ **mort** |
| `mirror_graph` | 110 | ❌ **mort** |

**3 601 lignes de noyau — la moitié des modules — sont inatteignables depuis l'interface.**

*Remesuré au commit `e2cd410` par recherche de `glucose_core::<module>` dans `glucose-desktop`.
Depuis la première passe, `quadtree` et `smart_align` ont été branchés et deux modules ont été
créés (`layout`, `error`), tous deux vivants. Le nombre de modules morts est passé de 13 à 12
pendant que le noyau grossissait de 18 à 20 modules : **le rythme de branchement ne suit pas le
rythme d'écriture.***

C'est exactement pour ça que tu as l'impression d'être à 3 % : tu as porté les **algorithmes**
de la version TypeScript, mais **pas les fonctionnalités**. Entre un algorithme et une
fonctionnalité il y a l'interface, les commandes, les états et la persistance — et c'est ça qui
manque.

Le cas le plus douloureux : **`quadtree.rs` résout R-05, `smart_align.rs` résout le magnétisme,
`export.rs` résout l'export. Tout est écrit et testé. Il ne manque que le branchement.**

---

### R-19 — `app.rs` est un objet-dieu

**Gravité : STRUCTUREL — en aggravation.** **1 126 lignes** au commit `81aea31` (contre 1 029),
une structure `GlucoseApp` à 17 champs publics, et **une seule fonction `window_event` de
728 lignes** (contre 490) avec 7 niveaux d'imbrication.

> **Re-vérifié : ce constat a empiré de 49 %.** Les corrections de R-10 (sélection élastique) et
> de la navigation ont été ajoutées **dans** `window_event`, parce qu'il n'existe aucun autre
> endroit où les mettre. C'est le mécanisme exact par lequel un objet-dieu grossit : chaque
> bonne correction l'alourdit, faute de structure d'accueil. C'est aussi pourquoi la phase 1
> de la roadmap éclate ce fichier **avant** d'ajouter des fonctionnalités.

Ce qui y est mélangé :

- la boucle d'événements et la présentation du framebuffer ;
- l'état d'interaction (pan, drag, sélection, édition de texte) ;
- la **logique métier** — `organize_layout`, un algorithme de mise en page (l. 296-360) ;
- les **entrées/sorties** — décodage d'images, écriture dans `%TEMP%`, dialogues de fichiers ;
- le **presse-papiers** ;
- la **construction d'annotations** — 4 blocs de ~30 lignes de littéraux de struct ;
- l'**édition de texte** — parcours UTF-8, gestion du curseur (l. 800-880).

Aucune de ces responsabilités n'est testable isolément. Il n'existe **aucun test** dans
`glucose-desktop`, hormis 2 dans `canvas.rs`.

C'est le symptôme direct de « on doit se démerder pendant super longtemps pour trouver et
rectifier » : quand tout est dans une fonction de 490 lignes, il n'y a aucune frontière où poser
un point d'arrêt mental.

---

### R-20 — L'UI calcule sa géométrie deux fois — 🟡 PARTIELLEMENT CORRIGÉ

> **Résolu pour la barre d'outils par `81aea31`**, et de la bonne façon : une fonction
> `layout_topbar(screen_w, ui, typo, img_count) -> Layout` produit une liste de boutons
> (`x, y, w, h, action, label`) ; `render_topbar` la dessine, `handle_ui_click` la parcourt.
> Deux tests ont été ajoutés (`test_topbar_no_overlap_across_all_resolutions` sur 8 résolutions,
> `test_topbar_responsive_collapse`). **C'est exactement la loi L4.**
>
> **Toujours ouvert pour la barre d'onglets** (R-07) et pour la minimap (R-08), qui gardent leur
> géométrie dupliquée. Tant qu'il reste un seul endroit non converti, la classe de bugs survit.

**Gravité initiale : STRUCTUREL — la faute la plus coûteuse du projet.**

`render_topbar` et `handle_ui_click` contenaient **deux copies indépendantes de la même mise en
page** *(numérotation du commit `7a13c86`)* :

| Élément | Dessin | Test de clic |
|---|---|---|
| Origine des outils | `cur_x = 108.0` | `cur_x = 108.0` |
| Pas d'outil | `cur_x += 32.0` | `cur_x += 32.0` |
| Séparateur | `cur_x += draw_separator(...)` *(retourne 9.0)* | `cur_x += 32.0 + 9.0` |
| Bouton Images | `cur_x += 92.0` | `cur_x += 92.0 + 9.0` |
| Largeur d'onglet | `measure_text(name, 12.0, **is_active**)` | `measure_text(name, 12.0, **false**)` |

Toute la géométrie est dupliquée à la main, et **elle a déjà divergé** (R-07). Ajouter un bouton
demande d'éditer deux fonctions à deux endroits, avec des nombres magiques recopiés.
Ce n'est pas un problème de discipline : **l'architecture rend l'erreur inévitable**.

**Correctif** : la mise en page produit une **liste de widgets** (`Vec<(WidgetId, Rect)>`) ;
le dessin la consomme, le test de clic la consomme. Une seule source de vérité.
Voir `02-ARCHITECTURE-CIBLE.md` § « UI : layout une seule fois ».

---

### R-21 — Les erreurs sont jetées à la poubelle

**Gravité : STRUCTUREL.**

`let _ = ...` : **5 occurrences** dans `app.rs` (redimensionnement de surface, présentation du
buffer, création de dossier temporaire). `.unwrap()` : **8 occurrences**, dont
`SystemTime::duration_since`. `.expect()` sur le chargement des polices
([typography.rs:17-19](../../crates/glucose-desktop/src/typography.rs#L17-L19)).

**Aucun type d'erreur n'est défini dans aucun des deux crates. Aucune erreur n'atteint
l'utilisateur.**

Effet concret : `import_image_files` remplace silencieusement une image illisible par
`(300.0, 200.0)` ([app.rs:191](../../crates/glucose-desktop/src/app.rs#L191)), puis
`get_or_load_image` retourne `None`, et l'utilisateur voit un rectangle gris
« Image [img-173…] » sans jamais savoir pourquoi. Comme il n'y a **pas de cache négatif**, le
décodage raté est **retenté à chaque frame**.

---

### R-22 — Toute recherche est un scan linéaire

**Gravité : STRUCTUREL.**

`store.rs` contenait **34 occurrences** de `iter().find(...)` / `iter_mut().find(...)`. Après
le découpage en modules (`def3800`), elles sont **40**, réparties sur 7 fichiers — le découpage
a amélioré la lisibilité, **il n'a rien changé à la complexité**. C'est attendu, mais il faut le
dire : déplacer un scan linéaire ne l'accélère pas.

`active_board()` scanne les boards **à chaque appel** — et il est appelé plusieurs fois par
frame par le renderer.

`move_selected` appelle `selection_sets()`, qui reconstruit **deux `HashSet` complets** en
clonant chaque identifiant sélectionné
([store/images.rs:17-23](../../crates/glucose-core/src/store/images.rs#L17-L23)) — donc à chaque
événement `CursorMoved` pendant un drag. Sur une sélection de 500 images, c'est 500 `String`
clonées par mouvement de souris.

**Ce qui est en revanche correct, et mérite d'être noté** : `push_undo()` est bien neutralisé
pendant un glissement par le garde `in_live_edit`
([store/undo.rs:25](../../crates/glucose-core/src/store/undo.rs#L25)), et ce garde est réellement
appelé depuis [`drag.rs:17`](../../crates/glucose-desktop/src/interactions/drag.rs#L17) et
relâché en `drag.rs:97`. Le projet ne clone donc **pas** le document entier à chaque pixel de
déplacement. C'est l'un des rares endroits où un mécanisme a été écrit **et** branché.

`remove_images` fait une boucle `while grew` sur **tous les boards × toutes les images** jusqu'à
point fixe pour propager la cascade de miroirs : O(n²) sur une suppression.

---

### R-23 — Le modèle répète 14 champs par variante

**Gravité : STRUCTUREL.** [types.rs:227-303](../../crates/glucose-core/src/types.rs#L227-L303)

`Annotation` est une énumération à variantes-structures. `x`, `y`, `membrane_id`, `domains`,
`mirror_of`, `temporal_anchor` sont **recopiés dans les 4 variantes**. Conséquences :

- **Construire une annotation demande 14 lignes de littéraux.** `app.rs` en contient 4
  exemplaires quasi identiques (l. 57-71, 601-615, 630-647, 665-690).
- Chaque accesseur est un `match` à 4 bras — `id()`, `x()`, `y()`, `membrane_id()`,
  `set_membrane_id()`, `domains_mut()`
  ([types.rs:305-357](../../crates/glucose-core/src/types.rs#L305-L357)).
- Ajouter un champ commun = éditer 4 variantes + tous les `match` + tous les sites de
  construction.
- `store.rs` contient **12 `match ann { … }`** dont l'unique objet est d'atteindre `x` et `y`.

**Correctif** : `struct Node { common: NodeCommon, kind: NodeKind }`. Les champs communs sont
écrits une fois, `node.common.x` est direct, et les `match` ne subsistent que là où le
comportement diffère réellement.

---

### R-24 — Des champs du modèle sont ignorés au rendu

**Gravité : STRUCTUREL.** Le modèle promet, le renderer ne tient pas.

| Champ | Défini | Rendu ? |
|---|---|---|
| `Sticky.bg_color` | types.rs:255 | ❌ couleur **codée en dur** `rgba8(254,240,138,245)` ([renderer.rs:646](../../crates/glucose-desktop/src/renderer.rs#L646)) |
| `Sticky.color` | types.rs:254 | ❌ texte toujours `rgba8(28,25,23)` |
| `Sticky.operator` | types.rs:257 | ⚠️ affiché via `format!("{:?}")` → « And », « Because » |
| `Arrow.color` | types.rs:268 | ❌ ignoré |
| `Arrow.stroke_width` | types.rs:272 | ❌ ignoré (dur : 1.8 / 2.5) |
| `Arrow.waypoints` | types.rs:273 | ❌ ignoré |
| `Arrow.text` | types.rs:266 | ❌ jamais affiché |
| `Arrow.arrow_type` (`curved`) | types.rs:270 | ❌ toujours droite |
| `Arrow.arrow_bidirectional` | types.rs:271 | ❌ ignoré |
| `Arrow.predicate` | types.rs:272 | ❌ ignoré |
| `Text.font_size` | types.rs:236 | ❌ dur à `14.0 * scale` |
| `BoardImage.rotation` | types.rs:74 | ❌ ignoré |
| `BoardImage.locked` | types.rs:75 | ⚠️ respecté par `move_selected`, invisible à l'écran |
| `Board.folders` | types.rs:463 | ❌ jamais dessiné |
| `Board.panels`, `Board.zones` | types.rs:462-464 | ❌ jamais dessinés |
| `MembraneMode`, `curtains` | types.rs:293-294 | ❌ jamais dessinés |

C'est le pire genre de dette : **le modèle ment**. On lit le code, on croit que la
fonctionnalité existe, on cherche pourquoi elle ne marche pas.

---

### R-25 — Deux énumérations `ActiveTool` concurrentes

**Gravité : STRUCTUREL.**

- [`glucose_core::store::ActiveTool`](../../crates/glucose-core/src/store.rs#L15-L25) —
  8 variantes (avec `ZoneSelect`), stockée dans `Store::active_tool`.
- [`glucose_desktop::ui::ActiveTool`](../../crates/glucose-desktop/src/ui.rs#L13-L22) —
  7 variantes, stockée dans `UiState::active_tool`.

**Seule celle de l'UI est lue.** Celle du store n'est jamais écrite ni consultée par l'app : elle
est du bruit qui sera cloné 200 fois dans la pile d'undo (R-04). Deux sources de vérité pour la
même notion, c'est une désynchronisation qui attend son heure.

---

## Rendu & typographie

### R-26 — Aucun cache de glyphes

**Gravité : BLOQUANT en pratique.**
[typography.rs:37-40](../../crates/glucose-desktop/src/typography.rs#L37-L40)

```rust
for ch in text.chars() {
    let (metrics, bitmap) = font.rasterize(ch, size);   // ← alloue un Vec<u8> par glyphe
```

`fontdue::Font::rasterize` **rastérise le contour et alloue un bitmap à chaque appel**. Aucun
cache. Chaque caractère de chaque carte est re-rastérisé 60 fois par seconde.

Amplifié par le contour des titres de membranes
([renderer.rs:296-315](../../crates/glucose-desktop/src/renderer.rs#L296-L315)) : le libellé est
dessiné **9 fois** (8 décalages d'ombre + 1) — soit 9 rastérisations complètes de la même
chaîne, à la même taille, par membrane, par frame.

**Correctif** : atlas de glyphes en cache `(font_id, char, taille_quantifiée) → bitmap`, alloué
une fois. Pour le contour, rastériser une fois et compositer 9 fois.

---

### R-27 — Le mélange alpha du texte est faux

**Gravité : MAJEUR.**
[typography.rs:53-70](../../crates/glucose-desktop/src/typography.rs#L53-L70)

```rust
data[idx]     = (r * effective_alpha + dr * inv_alpha).min(255.0) as u8;
// ...
data[idx + 3] = 255;   // ← écrase l'alpha de destination
```

Trois fautes :

1. **`data[idx+3] = 255` détruit l'alpha du pixmap.** Dessiner du texte sur une zone transparente
   la rend opaque. Ça passe aujourd'hui parce que le fond est rempli opaque en premier — toute
   couche transparente future casse.
2. **Le mélange se fait en espace prémultiplié en traitant les valeurs comme non prémultipliées.**
   Le buffer de `tiny-skia` est en RGBA prémultiplié ; ce code y écrit du non-prémultiplié.
   Sur les bords anti-aliasés au-dessus d'un fond coloré, ça produit des franges.
3. **Aucune correction gamma.** Le mélange est linéaire sur des valeurs sRGB : texte clair sur
   fond sombre trop maigre, texte sombre sur fond clair trop gras. C'est ce qui donne
   l'impression de « texte pas net » sans savoir pourquoi.

Accessoirement, `measure_text` retourne `(largeur, size)` : la hauteur retournée est la taille de
police, **pas** la hauteur de ligne réelle (ascendante + descendante + interligne). Toute mise en
page verticale qui s'y fie est fausse.

---

### R-28 — Pas de retour à la ligne, pas de crénage, pas de shaping

**Gravité : MAJEUR.**

- **`draw_text` ignore `\n`** (`if ch == '\n' { continue; }`,
  [typography.rs:34](../../crates/glucose-desktop/src/typography.rs#L34)) : le retour à la ligne
  est géré à l'extérieur, par `content.lines()`, dans **deux endroits différents** du renderer,
  chacun avec sa propre hauteur de ligne (`× 1.35` pour Text, `× 1.3` pour Sticky).
- **Aucun retour à la ligne automatique.** Une carte de 240 px avec un paragraphe long déborde
  horizontalement sans fin. La largeur de la carte est décorative.
- **Aucun crénage** : la largeur est la somme des `advance_width`. « AV », « To », « Ta »
  s'affichent avec des trous.
- **Aucun shaping** : pas de ligatures, pas de bidirectionnel (arabe, hébreu), pas de
  réordonnancement indien.
- **Ligne de base approximée** : `gy = y + size - ymin - height` utilise `size` comme substitut
  de l'ascendante réelle. Deux polices de même `size` n'auront pas la même ligne de base.

Effet visible aujourd'hui : le curseur de la sticky
([renderer.rs:697-699](../../crates/glucose-desktop/src/renderer.rs#L697-L699)) mesure **tout le
contenu**, pas le préfixe avant le curseur. Sur un texte multiligne, il part loin à droite et se
fait plaquer contre le bord par le `.min(...)`.

Le rendu Markdown, lui, se réduit à `starts_with("# ")` / `starts_with("- ")`
([renderer.rs:557-558](../../crates/glucose-desktop/src/renderer.rs#L557-L558)). Pas de gras, pas
d'italique, pas de code, pas de lien, pas de citation, pas de tableau, pas de LaTeX — alors que
la version TypeScript rend du Markdown complet **avec KaTeX**.

---

### R-29 — Le cache d'images est sans limite et sans mipmaps

**Gravité : MAJEUR.** [renderer.rs:59-102](../../crates/glucose-desktop/src/renderer.rs#L59-L102)

```rust
pub image_cache: HashMap<String, Pixmap>,
```

- **Aucune borne de taille, aucune éviction.** 200 photos de 24 Mpx = **19 Go** de `Pixmap` RGBA
  en mémoire. Rien ne les libère, jamais.
- **Aucun mipmap.** Une image 6000×4000 affichée à 100 px est ré-échantillonnée bilinéairement
  depuis la pleine résolution : lent **et** crénelé (le bilinéaire ne suffit pas au-delà d'un
  facteur 2).
- **Décodage synchrone dans la boucle de rendu.** `get_or_load_image` est appelée depuis
  `draw_images`. Ouvrir un projet de 50 images gèle la fenêtre pendant tout le décodage.
- **Aucun cache négatif.** Un fichier absent ou corrompu est retenté à chaque frame.
- **La clé est le chemin absolu.** Deux copies du même fichier = deux entrées. Il y a un
  `sha256` dans `bundle.rs`… non utilisé.

---

### R-30 — Décodage complet d'une image juste pour lire ses dimensions

**Gravité : MAJEUR.** [app.rs:189-193](../../crates/glucose-desktop/src/app.rs#L189-L193)

```rust
let (w, h) = if let Ok(dyn_img) = image::open(path_buf) {
    (dyn_img.width() as f64, dyn_img.height() as f64)
} else { (300.0, 200.0) };
```

`image::open` **décode intégralement** le fichier. Pour un JPEG de 24 Mpx c'est ~100 Mo alloués
et plusieurs centaines de millisecondes — **pour lire deux entiers présents dans les 20 premiers
octets du fichier**. Le résultat est ensuite jeté ; le renderer redécodera la même image un peu
plus tard (R-29).

Déposer 50 photos = 50 décodages complets inutiles, puis 50 décodages utiles. La fenêtre est
gelée pendant tout ce temps, sans aucun retour visuel.

**Correctif** : lire l'en-tête seul pour les dimensions, et rendre l'import asynchrone avec une
vignette de remplacement.

---

### R-31 — Le thème est ~130 nombres magiques éparpillés

**Gravité : STRUCTUREL.**

`Color::from_rgba8(...)` apparaît **73 fois** avec des littéraux : 39 dans `renderer.rs`,
33 dans `ui.rs`, 1 dans `icons.rs`. Le bleu de sélection `(56, 189, 248)` est recopié **11 fois**.
Le fond `(26, 26, 26)` **6 fois**. Aucune constante, aucune structure `Theme`.

Changer la couleur d'accent = 11 remplacements manuels sans filet. Un thème clair est impossible
sans réécrire les deux fichiers.

Idem pour la géométrie : `TOPBAR_HEIGHT` est une constante (bien), mais `7.0`, `30.0`, `32.0`,
`8.0`, `12.0` sont partout en dur, et **en double** (R-20).

---

## Vérité du dépôt

### R-32 — Le message de commit ne correspond pas au code

**Gravité : MAJEUR (confiance).**

Le commit `d7e460a` s'intitule :

> `feat(desktop): elimination de tiny-skia au profit d'un rasterizer 2D logiciel en pur Rust std`

**tiny-skia n'a pas été éliminé.** Il est déclaré dans
[Cargo.toml](../../crates/glucose-desktop/Cargo.toml) et importé dans **5 des 6 modules** du
crate (`app.rs`, `icons.rs`, `renderer.rs`, `ui.rs`, `typography.rs`). Aucun rastériseur maison
n'existe dans le dépôt.

De même, `glucose-desktop` dépend aujourd'hui de **7 crates externes** (`winit`, `softbuffer`,
`tiny-skia`, `fontdue`, `arboard`, `rfd`, `image`), qui tirent **plus de 200 crates transitives**
(`Cargo.lock` fait 80 Ko). L'objectif « 0 boîte noire » n'est pas atteint, et le dépôt ne le dit
pas.

Ce n'est pas cosmétique : **si l'historique ment, tu ne peux plus t'y fier pour savoir où tu en
es** — et c'est exactement le sentiment que tu décris.

---

### R-33 — 10 boutons sur 19 n'ont aucun effet observable

**Gravité : MAJEUR (perception).** Décompte bouton par bouton, au commit `81aea31` :

| Agissent réellement (9) | Sans effet observable (10) |
|---|---|
| Sélection, Pan, Texte, Sticky, Flèche, Membrane (outils) | **Dossier** — sélectionne l'outil, mais l'outil n'affiche qu'un toast |
| **+ Images** — ouvre le dialogue | **Timer**, **Storyboard**, **Domaines**, **Preset**, **Plugins**, **Exporter** — toast uniquement |
| **Ordonner** — réorganise | **Aimant** — bascule un booléen ; les guides sont toujours vides (R-09) |
| **+** (nouveau board) | **Trans-domaines** — bascule un booléen jamais relu |
| | **Collaborer** — bascule un booléen qui ne sert qu'à colorer le bouton |

[app.rs:521-566](../../crates/glucose-desktop/src/app.rs#L521-L566)

```rust
UiAction::ToggleTimer      => { self.ui.show_toast("⏱ Timer démarré (25m)"); }
UiAction::ToggleStoryboard => { self.ui.show_toast("🎬 Mode Storyboard"); }
UiAction::ExportMenu       => { self.ui.show_toast("💾 Exportation du canvas"); }
UiAction::TogglePlugins    => { self.ui.show_toast("🧩 Extensions & Plugins"); }
UiAction::TogglePreset     => { self.ui.show_toast("🎨 Préréglage PureRef appliqué"); }
UiAction::ToggleDomains    => { self.ui.show_toast("🏷 Domaines thématiques"); }
```

Aucun de ces toasts ne décrit une action réelle. « 🎨 Préréglage PureRef appliqué » est
factuellement faux — rien n'est appliqué. Il y a **24 appels à `show_toast`** dans `app.rs`.

C'est une **UI de maquette présentée comme une UI fonctionnelle**, et c'est ce qui rend
l'estimation d'avancement impossible : la barre suggère 19 fonctionnalités, 9 existent.
Un utilisateur — toi compris — conclut « c'est cassé » là où la vérité est « c'est inachevé ».

---

## Ce qui est bon, et qu'il faut garder

L'audit serait malhonnête s'il ne le disait pas.

| Élément | Pourquoi c'est bon |
|---|---|
| `glucose-core` sans aucune dépendance | Le `Cargo.toml` est réellement vide. C'est un vrai actif. |
| Les tests du noyau | 3 194 lignes de tests d'intégration, avec de vrais cas limites (`undo_redo_suite` fait 707 lignes). |
| `hit_priority.rs` | Modèle de priorité de sélection explicite et testé. La partie la plus mûre du code. |
| `smart_align.rs` | API propre (`snap_move` / `snap_resize` / `snap_point`), pure, testée. Il ne manque que le branchement. |
| `mirror_graph.rs` | Détection de cycle par BFS, correcte, testée. |
| `preserve_view` dans l'undo | L'idée que la caméra ne doit pas téléporter à l'undo est juste, et rarement implémentée. |
| `symbiotic_hue` | L'algorithme (bruit de valeur + moyenne circulaire) est élégant. C'est son **appel** qui est mal placé, pas son contenu. |
| Les commentaires d'invariants | `MEMB-1 — membrane propriétaire. STOCKÉE, jamais redérivée…` : ce genre de commentaire vaut de l'or. Il en faut plus. |

**Rien de tout cela n'est à jeter.** L'architecture proposée dans `02-ARCHITECTURE-CIBLE.md`
**réutilise** ces modules ; elle change ce qui les entoure.

---

## Récapitulatif chiffré

*Mesures au commit `81aea31`, avec l'évolution depuis `7a13c86`.*

| Indicateur | Valeur | Évolution |
|---|---:|:--:|
| Lignes Rust totales | 14 322 (dont 3 194 de tests) | ↗ +286 |
| **Modules de noyau morts** | **13 / 18** | → |
| Lignes de noyau inutilisées par l'app | ≈ 3 700 | → |
| Plus grosse fonction (`window_event`) | **728 lignes**, 7 niveaux | ↗ **+238** ⚠️ |
| Plus gros fichier (`app.rs`) | **1 126 lignes** | ↗ +97 ⚠️ |
| Appels directs à `redraw()` | **45** | ↗ +15 ⚠️ |
| Tests dans `glucose-desktop` | **4** | ↗ +2 ✅ |
| Types d'erreur définis | **0** | → |
| `let _ =` / `.unwrap()` dans `app.rs` | 5 / 8 | → |
| `iter().find()` dans `store.rs` | 34 | → |
| Littéraux `Color::from_rgba8` | 73 | → |
| Appels à `show_toast` | 24 | → |
| Dépendances directes du desktop | 7 (≈ 200 transitives) | → |
| **Chemins d'écriture de projet sur disque** | **0** | → |
| Boutons de la barre qui agissent | **9 / 19** | → |

### Lecture de cette évolution

Les deux derniers commits ont fait du **bon travail de correction** : 4 constats réglés, dont
les deux fautes de performance les plus visibles, et l'introduction de `layout_topbar()` — qui
est précisément le bon patron architectural.

Mais dans le même temps, `window_event` a grossi de 238 lignes et `redraw()` a gagné
15 appels. **Les corrections s'entassent dans l'objet-dieu, faute d'un endroit où les mettre.**

C'est la dynamique centrale à comprendre : à structure constante, chaque correction rend la
suivante plus coûteuse. C'est pour cela que la roadmap place l'éclatement de `app.rs` et la
boucle de rendu unique en **phase 1**, avant toute nouvelle fonctionnalité — sinon la dette
croît plus vite qu'on ne la rembourse.

---

# Nouveaux constats — vérification `8444e8b`

*Introduits par les commits de la phase 1. Même méthode : lecture du code, rien sur déclaration.*

## R-34 — Le DPI est câblé mais jamais appliqué — 🟡 (traité à vérifier)

**Gravité : MAJEUR.** Annoncé comme fait (tâche 1.10), **ne l'est pas**.

```rust
// app.rs:219 et 259 — les DEUX seules occurrences hors déclaration
self.ui.scale_factor = scale_factor as f32;
```

`UiState::scale_factor` est **écrit deux fois et lu zéro fois**. Recherche exhaustive sur tout
`crates/glucose-desktop/src/` : aucune lecture. `layout_topbar`, `layout_tabs`,
`layout_minimap`, `compute_panel_layouts` et l'ensemble de `dock.rs` travaillent toujours en
constantes physiques (`TOPBAR_HEIGHT: f32 = 44.0`, boutons 30×30, polices 10–15 px).

**R-16 est donc intact.** Sur un écran à 150 % — le réglage par défaut de beaucoup de portables
Windows 11 — la barre fait toujours 44 px physiques ≈ 29 px logiques.

Le `ScaleFactorChanged` a bien été ajouté, et c'est utile : il ne manque que la moitié qui agit.

**Correctif** : passer `scale_factor` à chaque fonction de layout et multiplier toutes les
constantes. Ou mieux — un type `Dip(f32)` qui ne se convertit en pixels qu'au dernier moment,
pour que l'oubli devienne impossible à compiler.

---

## R-35 — Les types d'erreur sont du code mort neuf — 🟡 (traité à vérifier)

**Gravité : MAJEUR.** Annoncé comme fait (tâche 1.23), **ne l'est pas**.

`crates/glucose-core/src/error.rs` (51 l.) et `crates/glucose-desktop/src/error.rs` (64 l.)
définissent `CoreError`, `CoreResult`, `DesktopError`, `DesktopResult`, avec des `Display`
soignés, des conversions `From`, et deux tests.

Recherche exhaustive de `CoreError|CoreResult|DesktopError|DesktopResult` dans tout `crates/`,
en excluant leurs fichiers de définition : **zéro résultat.**

Aucune fonction ne retourne ces types. Aucune ne les construit. Il n'y a pas de barre de statut.
Il reste **7 `let _ =`** (4 dans `app.rs`, 3 dans `dock.rs`) et les `.unwrap()` sur
`SystemTime::duration_since` sont toujours là (`clipboard.rs:74`).

**C'est R-18 reproduit en direct** — et c'est la leçon la plus importante de cette vérification :
*le réflexe du projet est d'écrire le module propre, et de s'arrêter avant de le brancher.*
Les 115 lignes sont bonnes ; elles ne servent à rien tant qu'aucun `Result` ne les traverse.

**Correctif** : convertir d'abord **un seul** chemin de bout en bout — le décodage d'image, qui a
déjà sa variante `ImageDecodeFailed` — jusqu'à un toast d'erreur visible. Une fois ce chemin
vivant, les autres suivent naturellement.

---

## R-36 — Le thème est ignoré par le code écrit en même temps que lui — 🟡 (traité à vérifier)

**Gravité : MAJEUR.** Annoncé comme fait (tâche 1.24), **ne l'est pas**.

`theme.rs` (153 l.) définit 25 jetons nommés. Nombre de lectures de `theme.` par fichier :

| Fichier | Lectures du thème | Littéraux `from_rgba8` |
|---|---:|---:|
| `renderer.rs` | 3 | 37 |
| `ui.rs` | **0** | 35 |
| `dock.rs` | **0** | **134** |
| `icons.rs` | **0** | 1 |
| `typography.rs` | **0** | 4 |
| **Total hors `theme.rs`** | **3** | **211** |

L'audit initial comptait **73** littéraux. Il y en a maintenant **211**.
**Le problème a triplé**, parce que `dock.rs` — 1 710 lignes écrites *après* la création du
thème — a été rédigé intégralement en couleurs codées en dur.

Un thème clair reste impossible. `theme.rs` est aujourd'hui un 26ᵉ module mort.

---

## R-37 — `dock.rs` reproduit R-20, avec un bug de divergence confirmé — 🟡 (traité à vérifier)

**Gravité : MAJEUR — le constat le plus grave de cette vérification.**

`dock.rs` fait **1 710 lignes** (limite fixée : 500) et contient 6 fonctions de rendu de
87 à 192 lignes (limite : 60). Mais le vrai problème est structurel.

`compute_panel_layouts()` produit bien une liste de boîtes de panneaux partagée — bonne idée.
**Mais le contenu de chaque panneau a sa géométrie écrite deux fois**, exactement comme
`render_topbar` / `handle_ui_click` avant le correctif :

```rust
// dock.rs:690 — RENDU
let (tw, _) = typo.measure_text(lbl, 10.0, false);
let bw = tw + 14.0;

// dock.rs:1433 — CLIC
let bw = lbl.len() as f32 * 6.0 + 14.0;
```

**Deux formules différentes pour la même largeur de bouton.** Et la seconde est doublement
fausse :

1. `len() * 6.0` est une estimation monospace arbitraire, sans rapport avec la police réelle ;
2. `str::len()` compte des **octets, pas des caractères**. Les libellés du panneau ORDONNER
   contiennent `→` (3 octets en UTF-8) : « Sombre → Clair » fait 14 caractères mais
   **16 octets**.

Conséquence mesurable : largeur au clic ≈ 16 × 6 + 14 = **110 px**, largeur au rendu ≈ **84 px**.
**26 px de dérive par bouton**, cumulés sur 8 boutons, avec en prime un retour à la ligne
(`if sx + bw > px + pw - 14.0`) qui ne se déclenche pas au même endroit dans les deux passes —
donc des **rangées entières décalées**. Les boutons de tri du panneau ORDONNER ne cliquent pas
où ils sont dessinés.

Tout le reste de `handle_dock_click` (169 lignes) est du même tissu : `py + 38.0 + 32.0 + 14.0`,
`cy += 32.0`, `py + b.height - 38.0`… recopiés à la main depuis les fonctions de rendu.

**Ce constat vaut plus que sa gravité technique** : `layout_topbar()` a démontré le bon patron,
il a ses tests, il marche — et le fichier suivant a été écrit sans lui. Tant que le patron n'est
pas *le seul chemin possible*, il ne sera pas suivi.

**Correctif** : `dock.rs` doit produire, comme la barre d'outils, une liste
`Vec<DockWidget { id, rect }>` consommée par le rendu **et** par le clic. Et l'éclater en un
fichier par panneau.

---

## R-38 — La boucle d'animation repeint tout l'écran 33 fois par seconde — 🟡 (traité à vérifier)

**Gravité : MAJEUR.** [app.rs:300-330](../../crates/glucose-desktop/src/app.rs#L300-L330)

```rust
if self.ui.current_toast.is_some() {
    need_anim = true;
    min_timeout_ms = min_timeout_ms.min(30);   // ← 33 fps
}
// ...
if need_anim {
    event_loop.set_control_flow(ControlFlow::WaitUntil(next_deadline));
    self.mark_dirty();                          // ← redessine TOUT, inconditionnellement
}
```

Un toast dure 2 500 ms → **83 rendus plein écran** (fond + grille + halos + membranes + images +
annotations + UI + docks) pour animer une pilule de 36 px de haut en bas d'écran.
Une session d'édition de texte tourne à 10 fps en permanence, pour un curseur dont la période
est de 500 ms — **2 redessins par seconde suffiraient**.

`mark_dirty()` est appelé **sans condition** dans `about_to_wait` : rien ne vérifie que quelque
chose a réellement changé depuis la frame précédente.

Le correctif 1.11 (`request_redraw` au lieu de `redraw()` direct) est réel et bon. Mais sans
rectangles sales (1.13, honnêtement marqué partiel), il déplace le problème au lieu de le
résoudre : on est passé de « repeint plein écran à chaque événement souris » à « repeint plein
écran 33 fois par seconde ».

**Correctif** : cadencer sur la vraie période de l'animation (500 ms pour le curseur, ~16 ms
seulement pendant les 400 ms de fondu du toast), et n'appeler `mark_dirty()` que si l'état
visuel a effectivement changé.

---

## R-39 — Allocations par frame dans le cache de teintes et l'index spatial — 🟡 (traité à vérifier)

**Gravité : MAJEUR.** Les deux optimisations de la phase 1 réallouent à chaque frame ce
qu'elles sont censées éviter de recalculer.

**Cache de teintes** — [renderer.rs:97-101](../../crates/glucose-desktop/src/renderer.rs#L97-L101),
appelé à chaque frame :

```rust
self.last_positions.clear();
for ann in annotations {
    if let Annotation::Text { id, x, y, .. } = ann {
        self.last_positions.insert(id.clone(), (*x, *y));   // ← String allouée par carte
    }
}
```

Vide et reconstruit intégralement la table à chaque frame : **n allocations `String` + n
insertions de hachage par frame**, même quand rien ne bouge. Sur 1 000 cartes immobiles, c'est
1 000 allocations/frame pour découvrir qu'il n'y a rien à faire.

**Index spatial** — [quadtree.rs:169-172](../../crates/glucose-core/src/quadtree.rs#L169-L172) :

```rust
for id in ids {
    out.insert(id.clone());   // ← String allouée par élément visible, par requête
}
```

`query_rect` retourne `HashSet<String>`. Chaque requête de visibilité alloue une `String` par
élément trouvé, puis hache des chaînes au lieu d'entiers. Il y a une requête par frame pour le
culling, plus une par test de clic.

Enfin, `index_board()` fait un **rebuild complet en O(n)** dès que `store.version` change —
c'est-à-dire **à chaque événement de souris pendant un drag**. Sur 10 000 nœuds, déplacer une
carte réindexe les 10 000.

**Correctif** : `Vec<NodeId>` d'entiers au lieu de `HashSet<String>` (déjà prescrit dans
`02-ARCHITECTURE-CIBLE.md` § 3.3), tampons réutilisés d'une frame à l'autre, et
`insert`/`remove` incrémentaux au lieu du rebuild.

---

## R-40 — Défauts ponctuels de la typographie — 🟡 (traité à vérifier)

**Gravité : MINEUR à MAJEUR selon le point.**
[typography.rs](../../crates/glucose-desktop/src/typography.rs)

Le cache de glyphes (1.14) est réel et le mélange alpha prémultiplié (R-27) est **correct** —
vérifié : `dst = src·a + dst·(1−a)` sur une destination prémultipliée, avec l'alpha préservé.
Restent quatre défauts :

1. **Éviction par table rase** (l. 53) : `if cache.len() > 4096 { cache.clear(); }`.
   Au 4 097ᵉ glyphe, on jette **tout** et on re-rastérise depuis zéro. Comme la clé inclut la
   taille au dixième de pixel, un zoom continu génère une clé nouvelle par palier : le cache se
   remplit vite et l'application repart de zéro périodiquement, en plein geste. Il faut une LRU.

2. **`pixmap.data_mut()` appelé une fois par pixel** (l. 103, dans la double boucle
   `row × col`). À sortir de la boucle : c'est un appel de fonction et une revérification
   d'emprunt par pixel, soit des centaines de milliers d'appels par frame.

3. **`Arc<GlyphEntry>` cloné à chaque glyphe** : une opération atomique par caractère dessiné,
   dans un moteur strictement mono-thread. `Rc`, ou mieux un index dans un `Vec`, suffirait.

4. **`measure_text` retourne toujours `(largeur, size)`** (l. 234) : la hauteur annoncée est la
   taille de police, pas la hauteur de ligne réelle. Toute mise en page verticale qui s'y fie
   est fausse — R-28 reste entier.

---

## R-41 — Les fichiers et les fonctions continuent de grossir

**Gravité : STRUCTUREL — la seule courbe qui va encore dans le mauvais sens.**

Malgré l'éclatement réussi d'`app.rs` (1 126 → 331 l.), la masse s'est déplacée, pas dissoute.

| Fichier | `7a13c86` | `8444e8b` | `3bd9bda` | `def3800` | Limite fixée |
|---|---:|---:|---:|---:|---:|
| `dock.rs` | — | 1 710 | 2 025 | **2 067** | 500 |
| `ui.rs` | 862 | 1 178 | 1 348 | **1 356** | 500 |
| `renderer.rs` | 807 | 1 026 | 1 143 | **1 179** | 500 |
| `store.rs` | 909 | 1 043 | 1 043 | **96** ✅ | 500 |
| `hit_priority.rs` | 920 | 1 026 | 1 026 | **136** ✅ | 500 |
| `membrane_space.rs` | — | — | 773 | **773** | 500 |
| `types.rs` | — | — | 597 | **597** | 500 |

**La courbe s'est enfin inversée, mais seulement dans `glucose-core`.** Le découpage de
`store.rs` (1 043 → 96 l. + 9 modules) et de `hit_priority.rs` (1 026 → 136 l. + 3 modules)
est réel, l'API publique n'a pas bougé et `glucose-desktop` a compilé sans retouche.

**4 fichiers dépassent encore la limite**, et les trois de `glucose-desktop` continuent de
grossir à chaque commit — y compris `dock.rs`, qui vient pourtant d'être *refactorisé*.
Seul `renderer.rs` a reculé (1 242 → 1 179) parce que le halo en est sorti : c'est la preuve
que la méthode fonctionne quand on l'applique.

**35 fonctions dépassent 60 lignes** dans `glucose-desktop`. Les pires :

| Lignes | Fonction |
|---:|---|
| **366** | `interactions/mouse.rs` — gestionnaire principal |
| **340** | `renderer.rs::draw_annotations` |
| **316** | `icons.rs` — fonction de dessin d'icônes |
| **244** | `ui.rs::layout_topbar` |
| 183 | `dock.rs::render_organize_content` |
| 154 | `dock.rs::render_pomodoro_content` |
| 141 | `interactions/shortcuts.rs` |
| 139 | `renderer.rs::draw_membranes` |
| 138 | `dock.rs::render_preset_content` |
| 135 | `dock.rs::handle_dock_click` |

`mouse.rs` à 366 lignes est particulièrement notable : c'est l'ancien `window_event` qui a
simplement changé de fichier. Le découpage a été fait **par catégorie d'événement**, pas par
niveau d'abstraction — chaque module reste un `impl GlucoseApp` avec accès à ses 24 champs
publics. C'est une amélioration réelle de la lisibilité, mais pas encore une décomposition :
aucun de ces modules n'est testable sans construire une fenêtre.

**Correctif** : appliquer la règle 1.7 des standards — l'événement est traduit en **intention**,
et l'intention est traitée par une fonction pure prenant l'état minimal dont elle a besoin.
C'est ce qui rendra `interactions/` testable, et c'est ce qui arrêtera la croissance.

---

## Synthèse de la vérification `8444e8b`

| | Nombre |
|---|---:|
| Tâches annoncées de la phase 1 | 21 |
| **Réellement faites et vérifiées** | **14** |
| Partiellement faites | 4 |
| **Annoncées faites mais fausses** | **3** (R-34, R-35, R-36) |
| Nouveaux constats introduits | **7** (R-34 → R-40) |
| Critères de sortie de la phase 1 atteints | **5 / 9** |
| Tests | 224, 14 suites, **tous verts** |
| Warnings clippy | 12 |
| Fichiers > 500 lignes | 5 (`dock.rs` 1710, `ui.rs` 1178, `store.rs` 1043, `renderer.rs` 1026, `hit_priority.rs` 1026) |
| Fonctions > 60 lignes dans `dock.rs` | 8 |


### Ce qui reste ouvert dans R-44 : `glucose-core` éteint encore trois alarmes

`glucose-desktop` est propre, mais le crate censé être exemplaire ne l'est pas. Trois
`allow(clippy::…)` subsistent dans `glucose-core`, et **deux d'entre eux sont exactement le lint
qui a laissé passer le gel** :

| Emplacement | Lint | Portée |
|---|---|---|
| `bundle.rs:3` | `manual_is_multiple_of`, `chunks_exact_to_as_chunks` | **module entier** |
| `hit_priority/handles.rs:17` | `too_many_arguments` | une fonction |
| `membrane_space.rs:159` | `too_many_arguments` | une fonction |

Les deux derniers ne sont pas théoriques. Chacune de ces fonctions présente **trois scalaires
flottants consécutifs et interchangeables**, la forme précise du défaut qui a rendu le logiciel
indémarrable :

```rust
fn push_handles(out, owner, id, z, corners, wx: f64, wy: f64, slop: f64)
fn walk(item, ox: f64, oy: f64, s: f64, in_focus, members, children, focus_id, out)
```

Dans `walk`, `s` est une **échelle** posée juste après deux **coordonnées** — littéralement le
couple `scale` / `mouse_x` qui a coûté une fenêtre gelée, cette fois dans le noyau. Intervertir
`oy` et `s` compile sans un mot et déplace silencieusement toute une membrane.

**Correctif** : appliquer à `glucose-core` le remède déjà écrit pour `glucose-desktop` —
`push_handles` prend un point et une tolérance nommés, `walk` prend une transformation nommée
(origine + échelle) — puis supprimer les trois `allow`. Le `#![allow]` de `bundle.rs` est de
portée module, donc à traiter en priorité : il éteint l'alarme pour du code qui n'existe pas
encore.


---

## Vérification `def3800` — le premier budget de frame mesuré

C'est la première fois que ce projet dispose d'un **budget de frame chiffré** plutôt que d'une
impression. Il a fallu deux corrections pour l'obtenir.

### Le gel au démarrage : une erreur d'ordre des arguments

La fenêtre restait blanche et Windows affichait « ne répond pas ». Cause : `render_docks()`
était appelée avec ses arguments dans le mauvais ordre, et le paramètre `scale` recevait
`mouse_x`, soit **170**. Les glyphes étaient rastérisés à 2 040 px. Coût : **10 697 ms par
frame**, soit une image toutes les onze secondes.

Deux choses méritent d'être retenues, parce qu'elles se reproduiront :

1. **254 tests verts et clippy propre n'ont rien vu.** Le défaut n'était pas dans une fonction
   mais dans un *site d'appel*. Les tests unitaires ne testent pas les sites d'appel.
2. **`clippy::too_many_arguments` aurait signalé cette fonction** — 11 paramètres dont 5 `f32`
   consécutifs et interchangeables par le compilateur. Ce lint était désactivé à la ligne 2 de
   `main.rs`. La dette de configuration a directement coûté un logiciel qui ne démarre plus.

### Le profil `dev` : 240 ms par frame, soit 4 images par seconde

Le profil de compilation par défaut rendait l'application inutilisable, et masquait complètement
le profil de rendu réel. Mesure en trois étapes :

| Configuration | Frame | dont `blit` |
|---|---:|---:|
| dépendances opt 0 + notre code opt 0 | 240,00 ms | — |
| dépendances opt 3 + notre code opt 0 | 55,00 ms | 31,50 ms |
| **dépendances opt 3 + notre code opt 1** | **22,30 ms** | **1,52 ms** |

Optimiser les seules dépendances ne donnait que ×4,4, parce que `blit` — une boucle scalaire sur
1,3 M de pixels, **notre** code — représentait alors 57 % de la frame. Il a fallu monter aussi
nos deux crates à `opt-level = 1`, ce qui conserve les symboles de debug. Gain total : **×10,8**.

### Le budget de frame après le cache de halos

Board par défaut, quasi vide, profil dev, `GLUCOSE_PERF=2` :

| Poste | Avant `def3800` | Après `def3800` | Part |
|---|---:|---:|---:|
| `docks` | 7,11 ms | **7,57 ms** | **43 %** |
| `ui` | 3,27 ms | 3,50 ms | 20 % |
| `grid` | 2,42 ms | 2,49 ms | 14 % |
| `blit` | 1,52 ms | 1,78 ms | 10 % |
| `halos` | **7,19 ms** | **1,50 ms** | 9 % |
| `present` | 0,75 ms | 0,76 ms | 4 % |
| **Total** | **22,30 ms** | **17,50 ms** | |

---

## R-42 — Les docks sont le premier poste de rendu, et ils ne changent jamais

**Gravité : MAJEUR — 43 % de la frame dépensés à redessiner ce qui n'a pas bougé.**

`render_docks()` coûte **7,6 ms à chaque image**, mesuré sur un board vide. C'est désormais le
poste le plus cher du rendu, devant l'UI, la grille et les halos réunis.

Or un dock ne change presque jamais : ni au déplacement de la souris sur le canvas, ni au zoom,
ni au défilement, ni pendant une animation de carte. Tout son contenu — panneaux, boutons,
libellés, icônes — est pourtant reconstruit et recomposé intégralement soixante fois par seconde.

**Correctif** : rendre chaque dock une fois dans son propre `Pixmap`, et ne le recomposer que
lorsque son état change. C'est la tâche 1.13 de la feuille de route (dirty rects), qui devient
le prochain gain de performance le plus rentable du projet.

---

## R-43 — `draw_grid` reconstruit un chemin complet à chaque frame

**Gravité : MAJEUR — viole la règle 4.3 des standards (aucune allocation dans la boucle de rendu).**

`renderer.rs:376` alloue un `PathBuilder` neuf à chaque image, y empile **un cercle par point de
grille visible** via une double boucle `while`, puis remplit le tout en un seul `fill_path`
anti-aliasé.

```rust
let mut pb = PathBuilder::new();        // alloué à chaque frame
while gx <= end_x {
    while gy <= end_y {
        pb.push_circle(sx as f32, sy as f32, 1.2);   // 4 courbes de Bézier par point
```

Chaque `push_circle` produit quatre courbes de Bézier, que le rastériseur doit ensuite aplatir en
segments puis couvrir en anti-aliasé — pour dessiner un point de 2,4 px de diamètre. Le nombre de
points dépend du viewport et du zoom : plusieurs centaines au minimum, davantage en dézoomant.

Coût mesuré : **2,49 ms par frame sur un board vide**, soit 14 % de la frame.

**Correctif** : un point de grille n'a pas besoin du pipeline de chemins. Un `fill_rect` de 2×2 px
par point, ou mieux, une composition directe des pixels comme celle qui vient d'être écrite pour
les halos dans `renderer/halo.rs`, coûterait une fraction de ce prix — et supprimerait l'allocation.

---

## R-44 — 15 lints clippy désactivés à l'échelle du crate

**Gravité : MAJEUR — dont celui qui aurait évité le gel au démarrage.**

`crates/glucose-desktop/src/main.rs:2` désactive **15 lints** pour tout le crate :

```
too_many_arguments, field_reassign_with_default, manual_strip, manual_is_multiple_of,
manual_range_contains, unnecessary_cast, collapsible_if, collapsible_else_if,
collapsible_match, single_match, chunks_exact_to_as_chunks, derivable_impls,
new_without_default, type_complexity, unwrap_or_default
```

Tant que ce bloc existe, `cargo clippy --all-targets -- -D warnings` **ne prouve rien** sur
`glucose-desktop` : il valide un crate dont on a préalablement éteint les alarmes. Le premier de
la liste, `too_many_arguments`, aurait signalé la fonction qui a rendu le logiciel indémarrable.

**Correctif** : supprimer le bloc, puis traiter les avertissements un par un. Les lints de style
(`collapsible_if`, `single_match`, `manual_range_contains`) se corrigent mécaniquement. Les deux
qui demandent un vrai travail — `too_many_arguments` et `type_complexity` — sont précisément ceux
qui désignent les fonctions à refactoriser. Aucun `#[allow]` ne doit être réintroduit ailleurs
pour compenser : ce serait déplacer la dette, pas la payer.

### Correctif appliqué — 🟡 (traité à vérifier)

Le bloc `#![allow(…)]` est supprimé de `main.rs`. Les 46 avertissements qu'il masquait sont
corrigés, aucun `#[allow(clippy::…)]` n'a été réintroduit **dans `glucose-desktop`**, et
`cargo clippy --workspace --all-targets -- -D warnings` sort en 0 sur un crate dont plus aucune
alarme n'est éteinte.

Cinq des quinze lints ne supprimaient plus rien au moment de la suppression
(`manual_is_multiple_of`, `manual_range_contains`, `collapsible_if`, `collapsible_else_if`,
`single_match`) : ils étaient devenus du bruit qui protégeait les dix autres.

Répartition des 46 avertissements : `too_many_arguments` 17, `unnecessary_cast` 12,
`field_reassign_with_default` 7, `type_complexity` 2, `collapsible_match` 2, `manual_strip` 2,
`chunks_exact_to_as_chunks` 1, `derivable_impls` 1, `new_without_default` 1, `unwrap_or_default` 1.

Les 17 `too_many_arguments` sont traités par regroupement en **types nommés**, pas par découpage :
`crates/glucose-desktop/src/params.rs` déclare `Pointer`, `ScreenFrame`, `ScaledRect`,
`ButtonState`, `SceneOverlay` et `ViewPass`, construits avec leurs champs explicites au point
d'appel. `render_docks` — la fonction du gel — reçoit désormais `ScreenFrame` et `Pointer` :
intervertir `scale` et `mouse_x` ne compile plus.

---

## R-45 — Le texte ne suit pas le zoom : la carte grandit, son contenu non

**Gravité : BLOQUANT pour la fidélité — c'est le « LOD » visible en dézoomant.**

Dans `renderer.rs`, la **boîte** d'une carte est mise à l'échelle linéairement, sans borne :

```rust
let sw = (width.unwrap_or(DEFAULT_TEXT_CARD_WIDTH)  * vp.scale) as f32;   // libre
let sh = (height.unwrap_or(DEFAULT_TEXT_CARD_HEIGHT) * vp.scale) as f32;  // libre
```

Mais **tout ce qu'elle contient est borné** :

| Valeur | Ligne | Expression | Plage où elle reste fidèle |
|---|---:|---|---|
| Police du corps | 684 | `(14.0 * scale).clamp(8.0, 24.0)` | `scale ∈ [0,57 ; 1,71]` |
| Marge horizontale | 686 | `(18.0 * scale).clamp(4.0, 24.0)` | `scale ∈ [0,22 ; 1,33]` |
| Marge verticale | 687 | `(12.0 * scale).clamp(4.0, 16.0)` | `scale ∈ [0,33 ; 1,33]` |
| Rayon de coin | 698 | `(24.0 * scale).clamp(4.0, 28.0)` | `scale ∈ [0,17 ; 1,17]` |
| Police de titre | 484 | `(16.0 * scale).clamp(12.0, 22.0)` | `scale ∈ [0,75 ; 1,37]` |
| Police secondaire | 854 | `(12.0 * scale).clamp(8.0, 20.0)` | `scale ∈ [0,67 ; 1,67]` |

**Onze `clamp`** au total sur des valeurs dérivées du zoom, et **aucun** sur la boîte.

Conséquence, exactement celle décrite par l'utilisateur :

- **En dézoomant** sous `scale = 0,57`, la boîte continue de rétrécir pendant que le texte reste
  figé à 8 px. Le texte déborde, les marges mangent la carte, la mise en page se réorganise. Ce
  n'est pas un niveau de détail choisi : c'est l'effet de bord de six bornes indépendantes qui se
  déclenchent à **six seuils différents** — d'où l'impression de « petits trucs » qui apparaissent
  les uns après les autres.
- **En zoomant**, les rayons de coin se figent à 1,17, puis les marges à 1,33, puis le titre à
  1,37, puis le corps à 1,71. La carte se déforme par paliers successifs.
- **La carte n'est fidèle qu'à `scale ≈ 1`** : c'est la seule valeur où aucune borne n'est atteinte.

**Pourquoi la version TypeScript n'a pas ce problème.** `HtmlAnnotationLayer.tsx` (1 333 l.) pose
le texte en DOM et laisse **une** transformation CSS mettre l'ensemble à l'échelle. Le navigateur
re-rend les glyphes à la bonne taille et tout se met à l'échelle **ensemble, par construction**.
Le portage Rust a remplacé une transformation unique par onze bornes manuelles.

### La règle existait, écrite, dans le code d'origine

Ce n'est pas une subtilité qu'on pouvait rater : l'auteur du TypeScript avait **anticipé
exactement ce défaut** et laissé le raisonnement en commentaire
([`HtmlAnnotationLayer.tsx:203-210`](../../src/canvas/HtmlAnnotationLayer.tsx#L203-L210)) :

> *« Une transformation d'échelle plutôt qu'une largeur divisée : **réduire la boîte sans réduire
> la police ferait déborder le texte**, alors que `scale` emporte tout d'un coup — cadre, police,
> marges, badges. »*

Et la couche entière est mise à l'échelle d'un seul geste (l. 395) :

```js
el.style.transform = `translate(${x}px, ${y}px) scale(${scale})`;
```

**La seule exception admise est une contre-échelle, jamais une borne.** Pour ce qui doit garder une
taille constante à l'écran — les guides d'alignement — le TSX divise explicitement par l'échelle
(l. 676-682) :

```js
width: 1 / scale,
borderLeft: `${1 / scale}px dashed …`,
```

D'où la règle à appliquer, et qui devient la règle **§ 4.4** des standards :

> Tout ce qui appartient au monde subit **une seule** transformation, ensemble. Ce qui doit garder
> une taille écran constante est divisé par l'échelle (`1 / scale`). **Aucune valeur dérivée du
> zoom n'est jamais bornée individuellement** : une borne sur la police sans borne équivalente sur
> la boîte casse la mise en page, et six bornes à six seuils la cassent six fois.

**Correctif** : une carte se dessine **une fois en coordonnées locales**, puis subit **une seule**
transformation. Si l'on veut des bornes de lisibilité, elles s'appliquent à la transformation
entière, jamais valeur par valeur.

---

## R-46 — Les glyphes sont posés à des positions entières tronquées

**Gravité : MAJEUR — source du flou et de l'irrégularité du texte.**

`typography.rs` rastérise correctement chaque glyphe à sa taille réelle (`font.rasterize(ch, size)`,
clé de cache au dixième de pixel près). Puis il le pose ainsi :

```rust
let px = (gx + col as f32) as i32;
let py = (gy + row as f32) as i32;
```

`as i32` **tronque**. Ni positionnement sous-pixel, ni rééchantillonnage, ni même un arrondi. Un
glyphe dont l'origine calculée tombe à `x = 100,6` est écrit à `x = 100`.

Trois conséquences visibles :

1. **L'espacement des lettres est faux de façon irrégulière** — chaque glyphe est tronqué
   indépendamment, donc l'erreur varie de 0 à 1 px d'une lettre à l'autre dans un même mot.
2. **Le texte tremble** pendant un déplacement ou un zoom : chaque glyphe franchit son seuil à un
   moment différent.
3. **Le rendu paraît pixelisé** alors que la rastérisation, elle, est bonne. Le défaut n'est pas
   dans le glyphe, il est dans l'endroit où on le pose.

À noter : l'en-tête d'`icons.rs` annonce « anti-aliasing et **subpixel rendering** ». C'est vrai
des icônes, qui sont des tracés vectoriels. **Ce n'est pas vrai du texte.** Le commentaire décrit
une intention, pas le code — même motif que R-32.

**Réserve honnête** : l'utilisateur signale aussi les **icônes** comme pixelisées. Je ne l'ai pas
reproduit : 6 appels sur 7 passent par `draw_icon_scaled`, vectoriel et sensible au DPI. Il faut
une capture comparée avant de conclure — c'est précisément ce que permettraient les tâches 0.5/0.6.

---

## R-47 — Le panneau DOMAINES écrit dans une liste fantôme

**Gravité : BLOQUANT — l'illustration la plus nette de la maladie chronique du projet.**

`glucose-core` implémente les domaines **au complet**, et c'est testé
([`e2e_workflows_suite.rs:326-339`](../../crates/glucose-core/tests/e2e_workflows_suite.rs#L326-L339)) :

```rust
store.add_domain(d);
store.update_domain(id, name);
store.remove_domain(id);
store.try_assign_domain_to_node("main", "T1", "D1", 0.7)?;   // avec pondération
```

Le panneau de l'interface entretient sa **propre liste**, sans rapport avec le document
([`dock.rs:196`](../../crates/glucose-desktop/src/dock.rs#L196)) :

```rust
pub struct DomainsState { pub domains: Vec<DomainItem> }
```

Et le bouton « + Nouveau domaine » ([`dock.rs:1754`](../../crates/glucose-desktop/src/dock.rs#L1754)) :

```rust
dock.domains.domains.push(DomainItem {
    id:    format!("domain-{}", dock.domains.domains.len() + 1),
    name:  format!("Domaine {}", dock.domains.domains.len() + 1),
    color: colors[idx],
});
```

**`store.add_domain()` n'est jamais appelé.** Les domaines créés n'entrent pas dans le document :
ni assignables à un nœud, ni pondérables, ni renommables, ni supprimables, ni enregistrables. La
capture de l'utilisateur montre « Domaine 1 » à « Domaine 20 » — vingt clics, vingt lignes
décoratives.

Écart avec `DomainsPanel.tsx` (211 l.), qui sait : créer, renommer en ligne, supprimer avec
cascade de désassignation confirmée, choisir parmi **12 couleurs et 12 icônes**, assigner à la
sélection **avec un poids**, et afficher le poids moyen agrégé sur la sélection courante.

**Correctif** : supprimer `DomainsState` et faire du panneau une vue de `store.project.domains`.
Le noyau est prêt — il n'y a rien à écrire côté modèle, seulement à brancher.

---

**Suite** : [`02-ARCHITECTURE-CIBLE.md`](02-ARCHITECTURE-CIBLE.md) — ce qu'on met à la place.
