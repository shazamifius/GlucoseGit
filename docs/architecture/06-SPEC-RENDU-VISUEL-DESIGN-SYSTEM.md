# 06 — Spécification du rendu visuel et Design System (Pixel-Perfect)

> **Rôle de ce document** : servir de **spécification visuelle absolue** pour la réécriture de Glucose en Rust natif (zéro dépendance / rastériseur logiciel ou GPU minimal).
> Il recense chaque couleur exacte (HEX, RGBA, HSL), chaque épaisseur de trait, chaque rayon de courbure, chaque formule géométrique, chaque icône SVG et chaque comportement graphique identifié dans l'implémentation de référence TypeScript/React/CSS/PixiJS.
>
> **Loi suprême** : **Glucose est BRUTALISTE.** L'interface utilisateur est un papier noir strict et monochrome. La couleur n'appartient **qu'au contenu de l'utilisateur**.
>
> **Méthode (12/09/2026)** : chaque valeur est allée voir le code, puis la référence quand ils divergeaient. Ce qui est tenu par un test est sorti ; ce qui reste est la liste de travail. **Deux constats d'ensemble** : le thème du desktop n'était pas celui de la cible — accent bleu ciel sur chaque état actif, canevas bleuté, une « inspiration PureRef » que la référence ne contient pas ; il l'est maintenant (`theme.rs`, un test par jeton, un test pour la loi elle-même : tout jeton de chrome est un gris neutre). Et **les dossiers n'étaient dessinés nulle part** : c'est corrigé au § 8, canevas et minimap compris.

---

## 1. Philosophie graphique & Principes Brutalistes

> Tenus : la feuille noire `#0d0d0d`, la chrome monochrome (`test_every_chrome_token_is_a_neutral_grey`), l'accent unique `#eab308` réservé et **jamais employé pour décorer** (les quatre usages décoratifs du bleu sont devenus blanc ou gris), les hairlines à taille écran constante (`WorldScale::screen`, exception SCALE-1).

---

## 2. Nuancier & Tokens de Couleur Exacts

> Tenu jeton par jeton (`test_the_theme_is_the_brutalist_palette_of_the_spec`) : surfaces (§ 2.1), filets (§ 2.2), typographie (§ 2.3), l'accent et le rouge d'alerte (§ 2.4).

### 2.4 Accents Sémantiques & Système de Statuts
* **À FAIRE** — **Portail / lien miroir** `#93c5fd` : flèche portail, anneaux de téléportation `↻`. Ni portail ni miroir n'ont de rendu (fiche 08 § 4.4, § 6.3).
* **À FAIRE** — **Succès / en ligne** `#10b981` : la pastille de collaboration. Le jeton est en dur dans `ui.rs`, derrière un état qui ne peut plus être vrai (fiche 09 § 9).

### 2.5 Couleurs des Opérateurs Logiques (Sticky Opérateurs)
* **À FAIRE** — `AND` « ET » `#34d399`, `OR` « OU » `#60a5fa`, `BUT` « MAIS » `#f59e0b`, `BECAUSE` « PARCE QUE » `#a78bfa` ; fond `couleur + "20"`, glow `couleur + "33"`. Le rendu emploie un seul ambre en dur pour les quatre (`note.rs`).

---

## 3. Le Moteur de Grille Infinie (Spécification Mathématique)

> Tenu (`test_the_grid_dot_follows_the_formula_of_the_spec`) : pas de 60 px monde, $R = \text{clamp}(1.2 \times \text{scale}, 0.5, 2.5)$, $\alpha = \text{clamp}(\text{scale} \times 0.3, 0.08, 0.45)$, extinction sous $0.07$, gris `136`.

* **Écart assumé, lié au rastériseur** — sous 32 px d'écran entre deux points, le pas double. La référence dessine le pas nominal jusqu'à l'extinction — jusqu'à 73 000 points par frame en 1440 × 900 — sur GPU. Ce doublement borne le coût CPU et disparaît avec le rendu GPU (plan de marche RQ-2).

---

## 4. Spécification des Cartes & Images

> Tenus : les images sont positionnées par leur centre (`BoardImage`, `rect_of_image`) ; le cadre de sélection blanc pur à 0,80, débordant de 3 px, épais de 1,25 px à l'écran ; les poignées, carrés de 9 px blancs à liseré noir `#111111` à 0,90 de 1,25 px, posées aux coins (`test_a_handle_is_a_nine_pixel_square_with_a_hairline_outline`, `test_handles_are_drawn_where_hit_priority_looks_for_them`) ; le rayon de saisie de 24 px (fiche 07 § 3.2).

### 4.1 Géométrie & Bounding Box
* **À FAIRE** — **`fit: "contain"`** pour les vignettes d'un miroir de dossier : $s = \min(w_{\text{boîte}}/w_{\text{texture}},\ h_{\text{boîte}}/h_{\text{texture}})$, la texture n'est jamais déformée. Ni vignettes ni miroir de dossier (fiche 08 § 5.3).

### 4.2 Cadre de Sélection & Poignées de Redimensionnement
* **À VÉRIFIER** — Curseurs `nwse-resize` / `nesw-resize` selon le coin (le curseur est annoncé au survol : `test_the_cursor_announces_the_gesture_over_a_handle_before_any_click` ; les deux orientations restent à distinguer).

### 4.3 Images Verrouillées
* **À FAIRE** — `locked` : les poignées disparaissent, le cadre de sélection passe au rouge `#f87171` (canevas et minimap). Le verrouillage n'a pas de geste (fiche 08 § 1.3).

---

## 5. Spécification des Textes, Notes et Markdown

### 5.1 Bloc de Texte Libre (Markdown + Math)
> Tenus (`test_the_text_card_metrics_are_those_of_the_spec`) : padding `16px 24px`, coins de 32 px, corps 14 px, interligne 1,4 — le Rust avait 12/18, 24 et 1,35. La hauteur suit le contenu (TEXT-FIT-1). L'édition se fait dans la carte.

* **À FAIRE** — Largeur libre jusqu'à `600px`, ou fixée si redimensionnée.
* **À VÉRIFIER** — L'esthétique « nuage / brume » : fond `color-mix(aura 3 %)`, aura `0 0 60px 30px` à 15 % (40 % au survol ou comme cible de flèche). Un halo radial existe (`halo.rs`, HALO-1) ; ses paramètres ne sont pas dérivés de ce `box-shadow`.
* **À FAIRE** — Sélection : `outline: 1px dashed rgba(255, 255, 255, 0.5)`, décalé de 4 px.
* **À FAIRE** — Police `Inter, system-ui, …`. La typographie embarquée est celle de `typography/`.
* **À FAIRE** — Markdown GFM complet et KaTeX (fiche 08 § 2.1).

### 5.2 Note Adhésive (Sticky Note)
> Tenus (`test_the_sticky_metrics_are_those_of_the_spec`) : 160 × 120, `#f5c542`, coins de 2 px (le Rust avait 6), corps 13 px comme la référence (`ann.fontSize || 13`, le Rust avait 12).

* **À FAIRE** — Ombre `0 4px 6px rgba(0, 0, 0, 0.3)` ; sélection en double anneau blanc `0 0 0 2px #ffffff` ; poignées en **disques** de 8 px (fond `#ffffff`, contour 1 px `#333333`) — le rendu emploie les carrés des images.

### 5.3 Sticky Opérateur Logique
* **À FAIRE** — Pilule de 44 px de haut, 80 px de large (130 pour « PARCE QUE »), `borderRadius: 22px`, bordure 1,5 px de la couleur de l'opérateur, fond à 12 %, glow `0 0 18px`, texte centré majuscule 13 px gras espacé de 1 px (fiche 08 § 3.2).

---

## 6. L'Algorithme de Symbiose Chromatique (Symbiotic Hue)

> Tenu **au bit près** contre la référence (`test_the_hue_matches_the_reference_on_its_vectors`, six vecteurs produits par la source TypeScript exécutée par Node) : cellules de 2 000 px, rayon de 1 200 px, poids $(1 - d/R)^2$, moyenne vectorielle circulaire, influence $0.5 (1 - 1/(1 + \sum w))$, seuls les textes attirent. Deux écarts corrigés : `idHash` est de l'arithmétique JavaScript (`<<` en int32, additions en flottant, le hash dépasse 2³²), et `random2D` rend $\sin - \lfloor \sin \rfloor$. **La fiche écrivait la formule en `f32` et `u32` : la référence est en `f64`, et l'entier modulaire ne reproduit pas JavaScript.** Le code est la spécification ; la copie qui figurait ici est retirée.

---

## 7. Spécification des Flèches & Relations de Graphe

> Ce qui existe : un trait de 1,8 px avec une pointe triangulaire (`note.rs`), l'ancrage au périmètre (`arrow_anchor.rs`, testé). Le reste :

### 7.1 Système Multi-Couches de la Flèche
* **À FAIRE** — Trois passes : hitbox invisible de 24 px `rgba(0, 0, 0, 0.01)` (les flèches ne sont pas cliquables, fiche 07 § 3.1) ; halo de `strokeWidth + 4` (`+ 10` sélectionnée) à 0,18 (0,45) en dégradé source → cible ; ligne d'âme de `strokeWidth` (2 px nominal) à 0,92, dégradé ou blanc si sélectionnée ; pointillé `6 4` pour les liens trans-domaines.

### 7.2 Ancrage au Périmètre sans Inversion
* **À VÉRIFIER** — Décollement de 12 px (`ANCHOR_MARGIN`), réduit à $\min(12, (d - 4)/2)$ sous 24 px de distance ; la flèche ne s'inverse jamais. `arrow_anchor.rs` a trois tests ; les chiffres 12 et 24 restent à tenir.

### 7.3 Pointes & Extrémités
* **À FAIRE** — Pointe : **disque** de rayon $1.5 \times sw$, fond `#111111`, contour 2 px à la couleur terminale (le Rust dessine un triangle) ; disque aux deux bouts si bidirectionnelle ; portail : anneau pointillé `#93c5fd` à 0,35 de rayon $4 \times sw$ tournant en 6 s, disque intérieur $2.6 \times sw$ fond `rgba(15, 15, 25, 0.85)` bordure `#93c5fd` 1,8 px, glyphe `↗` à $2.2 \times sw$.

### 7.4 Pastilles Sémantiques & Étiquettes
* **À FAIRE** — Libellé central en pilule (`#111111` à 75 %, arrondi 3 px, 22 px de haut, padding 8 px, texte 11 px à la couleur médiane) ; badge de prédicat (disque de 10 px, `#111111` à 90 %, liseré 1,5 px couleur de domaine, symbole 9 px) ; pastille « i » (disque de 9 px, liseré `#93c5fd` 1,5 px, *i* italique gras).

---

## 8. Spécification des Dossiers (Sub-canvases)

> Le cadre est dessiné (`renderer/folder.rs`), et ses valeurs sont tenues par
> `test_les_metriques_sont_celles_de_la_fiche` — y compris la conversion des pourcentages
> d'opacité en canal alpha, vérifiée plutôt que recopiée. Le bandeau partage la constante
> `pick_consts::FOLDER_HEADER` avec le picking, donc dessin et clic ne peuvent pas diverger
> (`test_le_bandeau_partage_la_constante_du_picking`). La minimap les dessine en pointillés à
> leur couleur, et compte leurs bornes — un tableau qui n'aurait que des dossiers n'avait
> aucune minimap.

### 8.1 Structure Visuelle du Cadre
* **À VÉRIFIER** — L'icône d'explorateur est une silhouette à deux rectangles arrondis, aux proportions de la fiche ; le tracé vectoriel exact de la référence n'a pas été comparé courbe par courbe.
* **À FAIRE** — Le badge compteur affiche **zéro**. Compter le contenu réel demande d'ouvrir le tableau enfant, ce qu'une frame ne peut pas faire : il faut un compte tenu à jour par le store, et il appartient au noyau.
* **À FAIRE** — Poignée de redimensionnement bas-droit *gravée* : deux traits à 45°, (4, 14)→(14, 4) à 0,7 / 1,5 px et (9, 14)→(14, 9) à 0,4 / 1 px. Le dossier sélectionné porte aujourd'hui les mêmes poignées carrées que les autres nœuds.

### 8.2 Mini-Carte Vectorielle Interne (Folder Preview)
* **À FAIRE** — Marge 12 px ; dégradé radial de fond 0,04 → 0 ; flèches enfants en traits ultra-fins ; images en rectangles pleins ; notes à leur teinte. Rien n'est dessiné du contenu d'un dossier.

---

## 9. Spécification de la Minimap

> Tenus (`test_the_chrome_metrics_are_those_of_the_spec`) : 180 × 120, à 12 px du bas et de la droite (le Rust avait 16), fond `rgba(13, 13, 13, 0.92)`, bordure `#2a2a2a`, cadre de caméra blanc à 0,35.

* **À FAIRE** — Glissement à `332px` de la droite en 0,18 s quand un dock droit s'ouvre (fiche 07 § 1, `MINIMAP_SLIDE`).
* **À VÉRIFIER** — Rayon 4 px, opacité globale 0,85, curseur `crosshair` → `grabbing` ; rendu du contenu : images `#2a2a2a` liseré 0,5 px `#444444`, verrouillées `#3a2a1a` liseré `#f87171`, notes à leur couleur, textes blanc à 0,5, flèches 1,5 px ; remplissage du cadre de caméra blanc à 0,04 ; état vide « Canvas vide » en 10 px `#444444`. Les dossiers, eux, y sont désormais en pointillés à leur couleur.

---

## 10. Chrome de l'Application (Barres & Panneaux)

> Tenus : barre d'outils 44 px `#1a1a1a` bordure `#2a2a2a`, onglets 34 px `#111111`, boutons d'outil 30 × 30 (actif `#2d2d2d`, contour `#444444`, icône blanche), onglet actif souligné de 2 px blanc pur, toasts `rgba(26, 26, 26, 0.97)` bordure `#2a2a2a` texte `#cccccc`, durée 2 400 ms et apparition 180 ms (fiche 07).

* **À VÉRIFIER (fiche 10)** — Le détail des boutons d'action (padding 4 × 10, 12 px, repos `#666666`, survol `#1e1e1e` / `#cccccc`), des séparateurs (1 × 20, `#2a2a2a`, marges 4), des onglets (largeur 100–180, padding 12, séparateur `#1a1a1a`, inactif `#555555`, bouton de fermeture 14 px), du logo (« GLUCOSE » 14 px 700 espacé de 2 px) et des toasts (arrondi 6 px, ombre `0 4px 20px rgba(0, 0, 0, 0.60)`, icône 14 × 14 `#888888`, `bottom: 56px`, quatre empilés au plus).

---

## 11. Catalogue des Icônes Clés (Géométrie SVG Exacte)

> **À VÉRIFIER** — `icons.rs` trace les icônes ; leur géométrie n'a pas été comparée tracé par tracé aux SVG ci-dessous, tous en `viewBox="0 0 14 14"` ou `16 16`, trait monochrome 1,2–1,5 px, sans remplissage.

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
