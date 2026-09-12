# 06 — Spécification du rendu visuel et Design System (Pixel-Perfect)

> **Rôle de ce document** : servir de **spécification visuelle absolue** pour la réécriture de Glucose en Rust natif (zéro dépendance / rastériseur logiciel ou GPU minimal).
> Il recense chaque couleur exacte (HEX, RGBA, HSL), chaque épaisseur de trait, chaque rayon de courbure, chaque formule géométrique, chaque icône SVG et chaque comportement graphique identifié dans l'implémentation de référence TypeScript/React/CSS/PixiJS.
>
> **Loi suprême** : **Glucose est BRUTALISTE.** L'interface utilisateur est un papier noir strict et monochrome. La couleur n'appartient **qu'au contenu de l'utilisateur**.

---

## 1. Philosophie graphique & Principes Brutalistes

1. **La « feuille de papier noire »** :
   Le canvas (`#0d0d0d`) est un espace de réflexion infini. L'interface n'a ni fioritures, ni gradients décoratifs criards, ni glassmorphisme, ni ombres molles sur les fenêtres.
2. **Monochrome strict pour la Chrome** :
   Barre d'outils, onglets, docks, menus, boutons, poignées de sélection et séparateurs sont composés exclusivement de noir, de blancs et de gris neutres.
3. **L'unique accent : le Jaune d'emphase (`~#eab308`)** :
   Utilisé avec une parcimonie extrême, exclusivement pour signaler un point d'attention capital (jalon d'historique, alerte non bloquante). Jamais sur un bouton standard, jamais en fond d'écran.
4. **La couleur appartient au contenu** :
   Ce sont les images de l'utilisateur, les domaines sémantiques qu'il crée, les notes adhésives qu'il dépose et le moteur de **symbiose chromatique** qui apportent la couleur à la feuille.
5. **Hairlines à échelle constante à l'écran** :
   Toutes les bordures, cadres de sélection, poignées et guides d'alignement conservent une épaisseur constante en pixels écran (1px à 1.5px), quel que soit le niveau de zoom de la caméra :
   $$\text{épaisseur}_{\text{monde}} = \frac{\text{épaisseur}_{\text{écran}}}{\text{scale}}$$

---

## 2. Nuancier & Tokens de Couleur Exacts

### 2.1 Surfaces & Arrière-plans
| Token | Valeur Hex / RGBA | Rôle & Composant |
|---|---|---|
| `canvas-bg` | `#0d0d0d` (`rgb(13, 13, 13)`) | Arrière-plan infini de l'espace de travail |
| `surface-root` | `#111111` (`rgb(17, 17, 17)`) | Fond de la barre d'onglets (`BoardTabs`), conteneurs sombres |
| `surface-toolbar` | `#1a1a1a` (`rgb(26, 26, 26)`) | Fond de la barre d'outils supérieure (`Toolbar`) |
| `surface-panel` | `#161616` (`rgb(22, 22, 22)`) | Fond des panneaux latéraux déroulants (`PanelDock`) |
| `surface-card-file` | `#14141c` (`rgb(20, 20, 28)`) | Fond des cartes de fichiers texte du Folder Mirror |
| `surface-btn-hover` | `#1e1e1e` (`rgb(30, 30, 30)`) | État survolé des boutons d'action de la barre d'outils |
| `surface-btn-active`| `#2d2d2d` (`rgb(45, 45, 45)`) | État actif/enfoncé des outils et boutons de docks |
| `surface-toast` | `rgba(26, 26, 26, 0.97)` | Fond opaque des notifications toast |
| `surface-minimap` | `rgba(13, 13, 13, 0.92)` | Fond de la minimap flottante |
| `surface-dialog` | `rgba(0, 0, 0, 0.50)` | Fond des overlays modaux et inputs en édition |

### 2.2 Bordures, Filets & Séparateurs
| Token | Valeur Hex / RGBA | Utilisation |
|---|---|---|
| `hairline-dark` | `#1c1c1c` / `#222222` | Séparateur fin entre onglets, bordure basse onglets |
| `hairline-base` | `#2a2a2a` (`rgb(42, 42, 42)`) | Bordure de Toolbar, séparateurs verticaux 1x20px, cadre minimap, cadre toast |
| `hairline-card` | `#2c2c3a` | Bordure par défaut des tuiles de code/fichier |
| `hairline-muted` | `#333333` | Bordure des inputs d'édition, contours de boutons fermer |
| `hairline-active` | `#444444` | Contour de bouton actif (`outline: 1px solid #444`) |
| `hairline-highlight`| `#555555` | Accentuation d'onglet survolé, liseré lors du glisser |
| `hairline-selection`| `rgba(255, 255, 255, 0.80)` | Cadre de sélection blanc pur de l'élément sélectionné |
| `hairline-smartguide`| `rgba(255, 255, 255, 0.30)`| Lignes pointillées infinies des guides magnétiques |

### 2.3 Typographie & Niveaux de Contraste
| Token | Valeur Hex | Usage |
|---|---|---|
| `text-bright` | `#ffffff` | Titre "Glucose", texte sélectionné, onglet actif, icônes actives |
| `text-main` | `#e8e8f0` / `#e6e6e6` | Noms de cartes, titres de fichiers, corps de texte principal |
| `text-content` | `#c8cce0` | Corps Markdown dans les tuiles, texte secondaire lisible |
| `text-dim` | `#cccccc` | Libellés des boutons d'actions, texte des toasts |
| `text-muted` | `#888888` | Icônes inactives, glyphes de drag poignée (`⠿⠿`) |
| `text-subtle` | `#666666` | Raccourcis clavier d'outils, boutons neutres |
| `text-dark` | `#444444` | Compteur d'images, hint "Canvas vide", bouton fermer onglet |
| `text-deep` | `#333333` / `#2a2a2a` | Poignées de dock au repos, indicateur discret du Mode Zen |

### 2.4 Accents Sémantiques & Système de Statuts
| Rôle | Couleur | Usage |
|---|---|---|
| **Accent Unique** | `#eab308` (Jaune ambré) | Jalons essentiels, avertissement doux, emphasis |
| **Succès / En ligne** | `#10b981` (Vert émeraude) | Pastille active de collaboration multi-joueur (`boxShadow: 0 0 5px #10b981`) |
| **Alerte / Erreur / Verrou** | `#ef4444` / `#f87171` (Rouge) | Image verrouillée, lien de fichier rompu, destruction d'éléments |
| **Portail / Lien miroir** | `#93c5fd` (Bleu ciel) | Flèche portail inter-boards, anneaux de téléportation miroir `↻` |
| **Sélection Clavier (Focus)**| `#93c5fd` | Anneau d'accessibilité `outline: 2px solid #93c5fd; offset: 2px` |

### 2.5 Couleurs des Opérateurs Logiques (Sticky Opérateurs)
| Opérateur | Label | Couleur Hex | Teinte d'accompagnement |
|---|---|---|---|
| `AND` | `"ET"` | `#34d399` (Vert menthe) | Fond : `#34d39920`, Glow : `#34d39933` |
| `OR` | `"OU"` | `#60a5fa` (Bleu clair) | Fond : `#60a5fa20`, Glow : `#60a5fa33` |
| `BUT` | `"MAIS"` | `#f59e0b` (Ambre) | Fond : `#f59e0b20`, Glow : `#f59e0b33` |
| `BECAUSE` | `"PARCE QUE"` | `#a78bfa` (Violet lilas) | Fond : `#a78bfa20`, Glow : `#a78bfa33` |

---

## 3. Le Moteur de Grille Infinie (Spécification Mathématique)

La grille en arrière-plan est un maillage de points réguliers (dot grid) calculé en coordonnées d'écran par rapport à la position de la caméra $(x, y)$ et de son zoom $\text{scale}$.

```
•   •   •   •   •   •   •
  Distance de base : 60 px monde
•   •   •   •   •   •   •
```

### Formule mathématique exacte :
1. **Pas de la grille à l'écran** :
   $$G = 60 \times \text{scale}$$
2. **Rayon du point à l'écran ($R$)** :
   $$R = \text{clamp}(1.2 \times \text{scale},\, 0.5,\, 2.5)$$
3. **Opacité du point ($\alpha$)** :
   $$\alpha = \text{clamp}(\text{scale} \times 0.3,\, 0.08,\, 0.45)$$
4. **Déphasage écran (modulo positif)** :
   $$P_x = ((x \pmod G) + G) \pmod G$$
   $$P_y = ((y \pmod G) + G) \pmod G$$
5. **Seuil d'extinction (LOD)** :
   Si $\text{scale} < 0.07$, la grille s'éteint complètement ($\alpha = 0$).
6. **Rendu par point** :
   Chaque point est un disque centré sur $(P_x + i \cdot G,\, P_y + j \cdot G)$ de rayon $R$, peint en `rgba(136, 136, 136, \alpha)`.

---

## 4. Spécification des Cartes & Images

### 4.1 Géométrie & Bounding Box
* **Repère interne** : Les images sont positionnées par leur centre $(x, y)$, avec une largeur $w$ et une hauteur $h$.
* **Coin haut-gauche calculé** :
  $$x_{\text{tl}} = x - \frac{w}{2}, \quad y_{\text{tl}} = y - \frac{h}{2}$$
* **Affichage avec ratio préservé (`fit: "contain"`)** :
  Pour les vignettes issues d'un miroir de dossier OS, la texture ne doit jamais être déformée :
  $$s = \min\left(\frac{w_{\text{boîte}}}{w_{\text{texture}}},\, \frac{h_{\text{boîte}}}{h_{\text{texture}}}\right)$$
  $$w_{\text{sprite}} = w_{\text{texture}} \times s, \quad h_{\text{sprite}} = h_{\text{texture}} \times s$$

### 4.2 Cadre de Sélection & Poignées de Redimensionnement
Quand une image est sélectionnée :
1. **Cadre de sélection hairline** :
   * Débord : 3 pixels autour de la texture.
   * Coordonnées locales :
     $$X = -\frac{w}{2} - 3, \quad Y = -\frac{h}{2} - 3, \quad W = w + 6, \quad H = h + 6$$
   * Contour : Blanc pur (`#ffffff`), opacité `0.80`, épaisseur constante à l'écran de `1.25px` ($\frac{1.25}{\text{scale}}$ en unités monde).
2. **Poignées de coin (4 coins)** :
   * Positionnées exactement sur les coins de la texture :
     $$\left(-\frac{w}{2}, -\frac{h}{2}\right), \quad \left(\frac{w}{2}, -\frac{h}{2}\right), \quad \left(\frac{w}{2}, \frac{h}{2}\right), \quad \left(-\frac{w}{2}, \frac{h}{2}\right)$$
   * **Carré visuel dessiné** :
     * Taille : Carré de $9\text{px}$ constant à l'écran ($hs = \frac{9}{\text{scale}}$).
     * Remplissage : Blanc pur opaque (`#ffffff`).
     * Liseré extérieur : Noir presque pur (`#111111`), opacité `0.90`, épaisseur $\frac{1.25}{\text{scale}}$.
     * *Raison du design* : Le blanc se détache du fond sombre du canvas, le liseré noir se détache des images très claires (« découpe papier »).
   * **Zone de préhension invisible (Hitbox)** :
     * Rayon effectif : $24\text{px}$ à l'écran (`PICK.HANDLE_SLOP_PX = 24`).
     * Diamètre de saisie : $\approx 48\text{px}$ à l'écran.
     * Curseurs : `nwse-resize` pour haut-gauche / bas-droit, `nesw-resize` pour haut-droit / bas-gauche.

### 4.3 Images Verrouillées
* Quand `image.locked === true` :
  * Les poignées de coin **disparaissent** entièrement.
  * Le cadre de sélection prend une teinte rouge d'avertissement : `#f87171` (sur le canvas ou la minimap).

---

## 5. Spécification des Textes, Notes et Markdown

### 5.1 Bloc de Texte Libre (Markdown + Math)
* **Dimensions** :
  * Largeur : libre jusqu'à `600px` max, ou fixée si redimensionnée (`ann.width`).
  * Hauteur : s'adapte au contenu (`fit-content`).
* **Esthétique « Nuage / Brume »** :
  * Padding : `16px 24px`.
  * Rayon des coins : `borderRadius: 32px`.
  * Couleur de fond : `color-mix(in srgb, var(--aura-color) 3%, transparent)`.
  * Ombre portée diffuse (aura) :
    * Normal : `box-shadow: 0 0 60px 30px color-mix(in srgb, var(--aura-color) 15%, transparent)`.
    * Survol / Cible de flèche : `box-shadow: 0 0 80px 40px color-mix(in srgb, var(--aura-color) 40%, transparent)`.
  * Sélection : `outline: 1px dashed rgba(255, 255, 255, 0.5)`, décalé de `4px` (`outline-offset: 4px`).
* **Typographie** :
  * Police : `Inter, system-ui, -apple-system, sans-serif`.
  * Taille : par défaut `14px` (ou `ann.fontSize`), interlignage `1.4`.
  * Markdown : Support standard GFM (titres H1..H6, listes `- `, citations `> `, code inline en monospace).
  * KaTeX : Formules inline `$x^2$` et bloc `$$\int f(x) dx$$` rendues sans débordement.

### 5.2 Note Adhésive (Sticky Note)
* **Dimensions par défaut** : Largeur `160px`, Hauteur `120px`.
* **Couleur par défaut** : Jaune pastel `#f5c542`.
* **Coins & Ombres** :
  * Coins : angle droit ou léger arrondi `2px`.
  * Ombre : `box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3)`.
  * Sélection : double anneau blanc `box-shadow: 0 0 0 2px #ffffff`.
* **Poignées de redimensionnement** :
  * 4 disques blancs aux coins : diamètre visuel `8px`, fond `#ffffff`, contour 1px `#333333`, zone tactile `24x24px`.

### 5.3 Sticky Opérateur Logique
* **Dimensions** : Hauteur fixe `44px`, Largeur `80px` (ou `130px` pour "PARCE QUE").
* **Forme** : Pilule parfaite (`borderRadius: 22px`).
* **Bordure** : Trait de `1.5px` solide de la couleur de l'opérateur.
* **Fond** : Couleur avec 12% d'opacité (`color + "20"`).
* **Glow** : `box-shadow: 0 0 18px color + "33"`.
* **Texte** : Centré, majuscule, `13px`, `fontWeight: 700`, espacement de lettres `1px`.

---

## 6. L'Algorithme de Symbiose Chromatique (Symbiotic Hue)

La teinte d'un bloc de texte ou d'une membrane émerge naturellement de sa position spatiale sur le canvas (bruit 2D par cellules de 2000px), puis subit l'attraction vectorielle des blocs voisins dans un rayon de 1200px.

```
       Cellule Perlin (2000 px)
+-----------------------------------+
|               • Bloc voisin (Hue2)|
|                 \                 |
|                  \ Rayon 1200px   |
|                   \               |
|      • Mon Bloc --> Attirance     |
|      (BaseHue)     circulaire     |
+-----------------------------------+
```

### Formule mathématique complète (implémentable directement en Rust) :

```rust
// 1. Hash déterministe de chaîne (djb2)
fn id_hash(id: &str) -> u32 {
    let mut h: u32 = 5381;
    for b in id.bytes() {
        h = h.wrapping_shl(5).wrapping_add(h).wrapping_add(b as u32);
    }
    h
}

// 2. Bruit 2D pseudo-aléatoire continu sur grille de 2000px
fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn random_2d(ix: f32, iy: f32) -> f32 {
    let dot = ix * 12.9898 + iy * 78.233;
    let sin = (dot.sin() * 43758.5453).fract();
    if sin < 0.0 { sin + 1.0 } else { sin }
}

fn get_zone_hue(x: f32, y: f32) -> f32 {
    let scale = 2000.0;
    let cx = x / scale;
    let cy = y / scale;

    let x0 = cx.floor();
    let x1 = x0 + 1.0;
    let y0 = cy.floor();
    let y1 = y0 + 1.0;

    let sx = smooth_step(cx - x0);
    let sy = smooth_step(cy - y0);

    let nx0 = random_2d(x0, y0) * (1.0 - sx) + random_2d(x1, y0) * sx;
    let nx1 = random_2d(x0, y1) * (1.0 - sx) + random_2d(x1, y1) * sx;
    let val = nx0 * (1.0 - sy) + nx1 * sy;

    (val * 360.0) % 360.0
}

// 3. Calcul complet de la teinte symbiotique
pub fn get_symbiotic_hue(ann_id: &str, x: f32, y: f32, neighbors: &[(String, f32, f32)]) -> f32 {
    let jitter = ((id_hash(ann_id) % 80) as f32) - 40.0;
    let mut base_hue = (get_zone_hue(x, y) + jitter + 360.0) % 360.0;

    const RAYON: f32 = 1200.0;
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut weight_sum = 0.0;

    for (other_id, ox, oy) in neighbors {
        if other_id == ann_id { continue; }
        let dx = x - ox;
        let dy = y - oy;
        if dx.abs() > RAYON || dy.abs() > RAYON { continue; }

        let dist = (dx * dx + dy * dy).sqrt();
        if dist < RAYON {
            let w = (1.0 - dist / RAYON).powi(2);
            let other_jitter = ((id_hash(other_id) % 80) as f32) - 40.0;
            let other_hue = (get_zone_hue(*ox, *oy) + other_jitter + 360.0) % 360.0;

            let rad = other_hue.to_radians();
            sum_x += rad.cos() * w;
            sum_y += rad.sin() * w;
            weight_sum += w;
        }
    }

    if weight_sum > 0.0 {
        let mut env_hue = sum_y.atan2(sum_x).to_degrees();
        if env_hue < 0.0 { env_hue += 360.0; }

        let mut diff = env_hue - base_hue;
        if diff > 180.0 { diff -= 360.0; }
        if diff < -180.0 { diff += 360.0; }

        let influence = 0.5 * (1.0 - 1.0 / (1.0 + weight_sum));
        base_hue += diff * influence;
    }

    (base_hue + 360.0) % 360.0
}
```

---

## 7. Spécification des Flèches & Relations de Graphe

### 7.1 Système Multi-Couches de la Flèche
Une flèche entre deux points n'est pas un simple trait : elle se compose de **3 passes de tracé géométrique identique** :
1. **Passe 1 : Hitbox invisible de capture** :
   * Épaisseur : `24px`.
   * Couleur : `rgba(0, 0, 0, 0.01)`.
   * Permet à l'utilisateur de cliquer facilement la flèche sans devoir viser un trait fin de 2px.
2. **Passe 2 : Halo lumineux (Glow)** :
   * Épaisseur : $\text{strokeWidth} + 4$ (non sélectionnée) ou $\text{strokeWidth} + 10$ (sélectionnée).
   * Opacité : `0.18` (repos) ou `0.45` (sélectionnée).
   * Couleur : Dégradé linéaire reliant la couleur source à la couleur cible.
3. **Passe 3 : Ligne d'âme (Core Stroke)** :
   * Épaisseur : $\text{strokeWidth}$ (nominal $2\text{px}$, modifiable).
   * Opacité : `0.92`.
   * Couleur : Dégradé linéaire, ou blanc `#ffffff` franc si sélectionnée.
   * Lignes trans-domaines : motif pointillé `strokeDasharray: "6 4"`.

### 7.2 Ancrage au Périmètre sans Inversion
La flèche s'arrête précisément au bord du conteneur parent (image, bloc texte, note), décollée de $12\text{px}$ (`ANCHOR_MARGIN = 12`).
* **Invariant absolu anti-croisement** :
  Si deux boîtes sont distantes de moins de $24\text{px}$, les marges des deux extrémités sont réduites symétriquement à la moitié de la place disponible :
  $$\text{marge} = \min\left(12,\, \frac{\text{distance} - 4}{2}\right)$$
  La flèche ne s'inverse **jamais**, même quand les blocs se chevauchent presque.

### 7.3 Pointes & Extrémités
* **Pointe standard (fin de flèche)** :
  * Disque de rayon $R = \text{strokeWidth} \times 1.5$.
  * Remplissage sombre `#111111`, contour de $2\text{px}$ à la couleur terminale de la flèche.
* **Flèche bidirectionnelle** :
  * Même disque dessiné à l'origine $(p_0)$ et à la terminaison $(p_n)$.
* **Flèche Portail (vers un autre Board)** :
  * Anneau extérieur en pointillés tournants (`#93c5fd`, opacité `0.35`, rayon $4 \times sw$, rotation 360° en 6s).
  * Disque intérieur central : rayon $2.6 \times sw$, fond `rgba(15, 15, 25, 0.85)`, bordure `#93c5fd` $1.8\text{px}$.
  * Glyphe au centre : `↗` en taille $sw \times 2.2$.

### 7.4 Pastilles Sémantiques & Étiquettes
* **Libellé textuel central** :
  * Boîte pilule : fond `#111111` à 75% d'opacité, arrondi 3px, hauteur 22px, padding horizontal 8px.
  * Texte : 11px, police sans-serif système, couleur médiane de la flèche.
* **Badge de prédicat sémantique** :
  * Disque de rayon $10\text{px}$ centré sur la flèche.
  * Fond `#111111` à 90% d'opacité, liseré $1.5\text{px}$ couleur de domaine.
  * Symbole interne en 9px.
* **Pastille d'information Markdown ("i")** :
  * Disque de rayon $9\text{px}$, liseré `#93c5fd` de $1.5\text{px}$.
  * Lettre *i* italique grasse au centre.

---

## 8. Spécification des Dossiers (Sub-canvases)

Un dossier est un conteneur spatial navigable ouvrant sur un sous-monde infini.

### 8.1 Structure Visuelle du Cadre
* **Dimensions minimales** : Largeur $180\text{px}$, Hauteur $120\text{px}$.
* **Coins** : Rayon $10\text{px}$ (`RADIUS = 10`).
* **Bandeau d'en-tête** : Hauteur fixe de $38\text{px}$ (`HEADER = 38`).
* **Corps du conteneur** :
  * Remplissage : Couleur du dossier à $2.5\%$ d'opacité (ou $5\%$ si sélectionné).
  * Bordure : Trait de $1\text{px}$ en pointillés `3 5` à $14\%$ d'opacité (devient continu de $1.4\text{px}$ à $45\%$ d'opacité si sélectionné).
  * Glow de sélection : Rectangle extérieur agrandi de $3\text{px}$ (`x: -3, y: -3, w: W+6, h: H+6`), rayon $13\text{px}$, contour $1.5\text{px}$ à $35\%$ d'opacité.
* **Icône de dossier** :
  * Dessinée à $(12, 11)$ dans l'en-tête.
  * Tracé SVG de dossier d'explorateur (largeur 16px, hauteur 14px), opacité $45\%$ ($70\%$ si sélectionné).
* **Titre du dossier** :
  * Coordonnées : $x = 34$, $y = 23$ (centré verticalement dans l'en-tête).
  * Police : `14px`, `fontWeight: 600`, lettre-spacing $0.3\text{px}$.
  * Tronqué avec points de suspension au-delà de 28 caractères.
* **Badge compteur d'éléments** :
  * Coordonnées : Coin supérieur droit ($x = W - 38$, $y = 11$).
  * Forme : Pilule $30\text{px} \times 16\text{px}$, arrondi $8\text{px}$.
  * Fond : Couleur du dossier à $18\%$ d'opacité, contour $0.8\text{px}$ à $40\%$ d'opacité.
  * Texte : Chiffre centré en `10px`, `fontWeight: 600`.
* **Poignée de redimensionnement (coin bas-droit)** :
  * Visible uniquement si le dossier est sélectionné.
  * Deux traits diagonaux gravés à 45° dans le coin :
    * Trait 1 : de $(4, 14)$ à $(14, 4)$, opacité `0.7`, largeur `1.5px`.
    * Trait 2 : de $(9, 14)$ à $(14, 9)$, opacité `0.4`, largeur `1.0px`.

### 8.2 Mini-Carte Vectorielle Interne (Folder Preview)
Quand le dossier contient des éléments, son corps affiche un rendu vectoriel miniature à l'échelle :
* Marge intérieure : $12\text{px}$ tout autour.
* Dégradé radial de fond : du centre vers les bords (opacité 0.04 à 0.00).
* Les flèches enfants sont rendues en traits ultra-fins en arrière-plan.
* Les images sont rendues en rectangles pleins miniatures.
* Les notes adhésives conservent leur teinte pastel miniature.

---

## 9. Spécification de la Minimap

* **Dimensions** : Largeur fixe $180\text{px}$, Hauteur fixe $120\text{px}$.
* **Positionnement écran** :
  * `bottom: 12px`.
  * `right: 12px` (se déplace avec transition fluide de `0.18s` à `332px` quand un dock latéral droit s'ouvre).
* **Style du boîtier** :
  * Fond : `rgba(13, 13, 13, 0.92)`.
  * Rayon des coins : `4px`.
  * Bordure : `1px solid #2a2a2a`.
  * Opacité globale : `0.85`.
  * Curseur : `crosshair` (au repos) $\to$ `grabbing` pendant le déplacement.
* **Rendu du contenu miniaturisé** :
  * Images normales : rectangle `#2a2a2a`, liseré `0.5px` `#444444`.
  * Images verrouillées : rectangle `#3a2a1a`, liseré `0.5px` `#f87171`.
  * Sticky notes : rectangle plein à la couleur de la note (`#f5c542` par défaut).
  * Textes : rectangle blanc translucide `rgba(255, 255, 255, 0.5)`.
  * Flèches : traits de $1.5\text{px}$ reliant les positions relatives des nœuds.
  * Dossiers : rectangles pointillés à la couleur du dossier.
* **Cadre indicateur de viewport (Caméra actuelle)** :
  * Rectangle calculé d'après les bornes d'écran inversées.
  * Remplissage intérieur : `rgba(255, 255, 255, 0.04)`.
  * Liseré extérieur : `rgba(255, 255, 255, 0.35)`, épaisseur $1\text{px}$.
* **État vide** :
  * Si le canvas ne contient aucun nœud : texte `"Canvas vide"` centré, en taille `10px`, couleur `#444444`.

---

## 10. Chrome de l'Application (Barres & Panneaux)

### 10.1 Barre d'Outils Supérieure (Toolbar)
* **Hauteur** : Fixe $44\text{px}$.
* **Fond** : `#1a1a1a`, bordure inférieure $1\text{px}$ `#2a2a2a`.
* **Composants** :
  1. Logo : Texte `"GLUCOSE"`, blanc pur, `fontSize: 14px`, `fontWeight: 700`, espacement de lettres `2px`, marge droite `12px`.
  2. Boutons d'outils (`ToolBtn`) :
     * Taille : Carré $30\text{px} \times 30\text{px}$, arrondi $4\text{px}$.
     * Repos : Fond transparent, icône `#666666`.
     * Actif : Fond `#2d2d2d`, icône `#ffffff`, contour `1px solid #444444`.
  3. Séparateurs verticaux : Filet de $1\text{px} \times 20\text{px}$, fond `#2a2a2a`, marges latérales $4\text{px}$.
  4. Boutons d'action (`ActionBtn`) :
     * Padding : $4\text{px} \times 10\text{px}$, arrondi $4\text{px}$, taille police `12px`.
     * Repos : Fond transparent, texte `#666666`.
     * Survol : Fond `#1e1e1e`, texte `#cccccc`.
     * Actif : Fond `#2d2d2d`, texte `#cccccc`, contour `1px solid #444444`.

### 10.2 Barre d'Onglets (BoardTabs)
* **Hauteur** : Fixe $34\text{px}$.
* **Fond** : `#111111`, bordure inférieure $1\text{px}$ `#222222`.
* **Onglet** :
  * Hauteur : $34\text{px}$, largeur min $100\text{px}$, max $180\text{px}$, padding $0\text{px} \times 12\text{px}$.
  * Séparateur droit : $1\text{px}$ `#1a1a1a`.
  * Onglet inactif : Fond transparent, texte `#555555`.
  * Onglet actif : Fond `#1a1a1a`, texte `#ffffff`, bordure inférieure $2\text{px}$ blanc pur `#ffffff`.
  * Bouton fermeture : Cercle de diamètre $14\text{px}$, croix `×` en `11px`, invisible ou gris `#444444`, virant à `#aaa` au survol.

### 10.3 Notifications Flottantes (Toasts)
* **Position** : Flottant, centré horizontalement, `bottom: 56px`, `zIndex: 99999`.
* **Dimensions** : Hauteur auto, padding $7\text{px} \times 16\text{px}$, espacement vertical entre toasts $6\text{px}$, max 4 toasts empilés.
* **Boîtier** :
  * Fond : `rgba(26, 26, 26, 0.97)`.
  * Bordure : `1px solid #2a2a2a`.
  * Arrondi : `6px`.
  * Ombre portée : `0 4px 20px rgba(0, 0, 0, 0.60)`.
* **Contenu** :
  * Icône vectorielle monochrome à gauche (taille $14\text{px} \times 14\text{px}$, couleur `#888888`).
  * Texte du message : `12px`, couleur `#cccccc`.

---

## 11. Catalogue des Icônes Clés (Géométrie SVG Exacte)

Toutes les icônes de la barre d'outils et des toasts sont des tracés vectoriels stricts en `viewBox="0 0 14 14"` ou `16 16`, trait monochrome `1.2px` à `1.5px` sans remplissage (`fill="none"`).

### 1. Sélection (V)
```svg
<svg width="12" height="12" viewBox="0 0 14 14" fill="none">
  <path d="M2 2L6.5 12L8 8L12 6.5L2 2Z" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round"/>
</svg>
```

### 2. Main / Pan (Espace)
```svg
<svg width="13" height="13" viewBox="0 0 20 20" fill="none">
  <circle cx="10" cy="10" r="3" stroke="currentColor" strokeWidth="1.5"/>
  <path d="M10 2v3M10 15v3M2 10h3M15 10h3" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
</svg>
```

### 3. Texte (T)
```svg
<svg width="12" height="12" viewBox="0 0 14 14" fill="none">
  <path d="M2 3h10M7 3v8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
</svg>
```

### 4. Note Sticky (N)
```svg
<svg width="12" height="12" viewBox="0 0 14 14" fill="none">
  <rect x="1.5" y="1.5" width="11" height="11" rx="1" stroke="currentColor" strokeWidth="1.3"/>
  <path d="M4 5h6M4 7.5h4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/>
</svg>
```

### 5. Flèche (A)
```svg
<svg width="12" height="12" viewBox="0 0 14 14" fill="none">
  <path d="M2 12L12 2M12 2H7M12 2V7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
</svg>
```

### 6. Dossier (F)
```svg
<svg width="12" height="12" viewBox="0 0 14 14" fill="none">
  <path d="M1 4.5V11.5a1 1 0 001 1h10a1 1 0 001-1V5.5a1 1 0 00-1-1H7L5.5 2.5H2a1 1 0 00-1 2z" stroke="currentColor" strokeWidth="1.2" strokeLinejoin="round"/>
</svg>
```

### 7. Membrane (M)
```svg
<svg width="13" height="13" viewBox="0 0 14 14" fill="none">
  <rect x="1.5" y="1.5" width="11" height="11" rx="3" stroke="currentColor" strokeWidth="1.3" strokeDasharray="3 2"/>
</svg>
```

### 8. Aimant / Smart Guides (G)
```svg
<svg width="12" height="12" viewBox="0 0 16 16" fill="none">
  <path d="M2 2l12 12M14 2L2 14" stroke="currentColor" strokeWidth="1" strokeOpacity="0.4"/>
  <rect x="3" y="3" width="10" height="10" rx="1" stroke="currentColor" strokeWidth="1.3" strokeDasharray="2 1"/>
</svg>
```
