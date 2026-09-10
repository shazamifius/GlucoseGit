# 01 — Audit du code Rust actuel

> **Portée** : `crates/glucose-core` (5 546 l.) + `crates/glucose-desktop` (3 466 l.).
> **Méthode** : lecture intégrale du code, `cargo check --workspace --all-targets` (0 erreur),
> vérification des sémantiques de `tiny-skia` dans la source vendue du crate.
> **Audit initial** : commit `7a13c86` — **re-vérifié** au commit `81aea31`.
>
> Chaque constat porte un identifiant stable (`R-xx`), une gravité, un emplacement exact,
> et un correctif. **Aucun constat n'est une supposition** : tout ce qui est affirmé ici a été
> vérifié dans le code ou dans la source de la dépendance concernée.

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
- **R-03** — Teinte symbiotique en O(n²) par frame, deux fois
- **R-04** — L'undo clone le projet entier, blobs compris, 200 fois
- **R-05** — Aucun culling : tout est dessiné même hors écran — ✅ **CORRIGÉ** (`81aea31`)

### Majeurs — comportement faux, visible par l'utilisateur
- **R-06** — Images placées au mauvais endroit (transform inversée) — ✅ **CORRIGÉ** (`81aea31`)
- **R-07** — Les onglets ne cliquent pas où ils sont dessinés
- **R-08** — La minimap est morte (code inatteignable)
- **R-09** — Le moteur d'alignement n'est jamais appelé
- **R-10** — La sélection élastique n'est jamais appliquée — ✅ **CORRIGÉ** (`2da029f`)
- **R-11** — `organize_layout` mélange deux conventions de coordonnées
- **R-12** — Les flèches ne suivent que leur source, jamais leur cible
- **R-13** — Dupliquer deux fois crée deux éléments de même id
- **R-14** — Les ids de boards se recollisionnent après suppression
- **R-15** — Le curseur ne clignote pas, les toasts ne disparaissent pas
- **R-16** — Aucune gestion du DPI
- **R-17** — Pas d'IME : pas d'accents au clavier mort, pas de CJK

### Structurels — la dette qui te fait perdre des heures
- **R-18** — 13 modules du noyau sur 18 sont du code mort
- **R-19** — `app.rs` est un objet-dieu
- **R-20** — L'UI calcule sa géométrie deux fois
- **R-21** — Les erreurs sont jetées à la poubelle
- **R-22** — Toute recherche est un scan linéaire
- **R-23** — Le modèle répète 14 champs par variante
- **R-24** — Des champs du modèle sont ignorés au rendu
- **R-25** — Deux énumérations `ActiveTool` concurrentes

### Rendu & typographie
- **R-26** — Aucun cache de glyphes
- **R-27** — Le mélange alpha du texte est faux
- **R-28** — Pas de retour à la ligne, pas de crénage, pas de shaping
- **R-29** — Le cache d'images est sans limite et sans mipmaps
- **R-30** — Décodage complet d'une image juste pour lire ses dimensions
- **R-31** — Le thème est ~130 nombres magiques éparpillés

### Vérité du dépôt
- **R-32** — Le message de commit ne correspond pas au code
- **R-33** — 15 boutons sur 19 ne font qu'afficher un toast

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

### R-18 — 13 modules du noyau sur 18 sont du code mort

**Gravité : STRUCTUREL — c'est la cause directe de ta frustration.**

Vérification par recherche des références `glucose_core::<module>` depuis `glucose-desktop` :

| Module | Lignes | Tests | Utilisé par l'app ? |
|--------|-------:|------:|---------------------|
| `types` | 583 | — | ✅ |
| `store` | 909 | 707 | ✅ |
| `hit_priority` | 920 | 348 | ✅ — uniquement `collect_candidates` |
| `smart_align` | 458 | 346 | ⚠️ importé, **jamais appelé** (R-09) |
| `symbiotic_hue` | 169 | — | ✅ |
| `membrane_space` | 772 | 422 | ❌ **mort** |
| `export` | 436 | 222 | ❌ **mort** |
| `membrane_focus` | 438 | 337 | ❌ **mort** |
| `bundle` | 353 | 246 | ❌ **mort** |
| `timeline` | 256 | 174 | ❌ **mort** |
| `membrane_stretch` | 231 | 199 | ❌ **mort** |
| `curtain_model` | 216 | 287 | ❌ **mort** |
| `geometry` | 215 | — | ❌ **mort** |
| `text_anchors` | 211 | 135 | ❌ **mort** |
| `curtain_panel` | 201 | — | ❌ **mort** |
| `arrow_anchor` | 166 | 87 | ❌ **mort** |
| `mirror_graph` | 110 | 93 | ❌ **mort** |
| `quadtree` | 79 | — | ❌ **mort** |

**≈ 3 700 lignes de noyau, dont ~2 200 de tests, ne servent à rien dans l'application.**

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

`store.rs` contient **34 occurrences** de `iter().find(...)` / `iter_mut().find(...)`.
`active_board()` scanne les boards **à chaque appel** — et il est appelé plusieurs fois par
frame par le renderer.

`move_selected` reconstruit **deux `HashSet` complets** à chaque appel
([store.rs:384-385](../../crates/glucose-core/src/store.rs#L384-L385)) — donc à chaque événement
`CursorMoved` pendant un drag.

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

**Suite** : [`02-ARCHITECTURE-CIBLE.md`](02-ARCHITECTURE-CIBLE.md) — ce qu'on met à la place.
