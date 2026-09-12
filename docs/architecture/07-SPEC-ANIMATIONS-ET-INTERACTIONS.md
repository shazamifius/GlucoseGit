# 07 — Spécification des Animations, Physique & Micro-Interactions

> **Rôle de ce document** : définir avec une précision chronométrique et mathématique absolue l'intégralité des **animations, courbes d'accélération, cinétiques, transitions d'état et règles d'interaction** de Glucose.
> Il constitue la référence indispensable pour recréer en Rust natif le ressenti fluide, réactif et organique ("feel & polish") de la version originale.
>
> **Méthode (12/09/2026)** : chaque chiffre est allé voir le code, puis la référence TypeScript quand les deux divergeaient. Ce qui est implémenté **et tenu par un test** est sorti ; ce qui reste est la liste de travail. Constat d'ensemble : l'arbitre de clic, l'aimantation, la géométrie des membranes et des rideaux sont exacts dans le noyau. **Le mécanisme d'animation existe désormais** (`glucose_core::anim` pour les courbes et les durées, `glucose-desktop/animation.rs` pour l'horloge) et la première transition de caméra — la plongée dans un dossier — est branchée. Les autres attendent leur geste, plus leur mécanisme.

---

## 1. Table Chronométrique & Délais de Référence (TIMING)

> Tenus par test : `DOUBLE_CLICK_WINDOW` 350 ms (une seule constante, `pick_consts::DBLCLICK_MS` — la référence en avait deux, 350 et 400, avec 50 ms entre les deux où un clic n'était ni double-clic ni re-clic) ; `CYCLE_TTL_MS` 2 500 ms ; `TOAST_DURATION` 2 400 ms et `TOAST_ANIM_IN` 180 ms (la référence déclarait 2 400 et codait 2 200 en dur ; le Rust codait 2 500 et 150 ; les constantes nommées font foi). Extension gardée : le toast a un fondu sortant de 400 ms, là où la référence le retire d'un coup.

| Constante | Durée (ms) | État | Description & Comportement |
|---|---:|---|---|
| `MEMBRANE_TWEEN` | **200 ms** | **À FAIRE** — la constante existe, le geste non | Passage fluide d'une image/texte entrant ou sortant d'une membrane minimisée |
| `FOLDER_TRANSITION`| **400 ms** | **Fait** — plongée à l'entrée, remontée à la sortie, un clic l'abrège | Plongée fluide de la caméra lors de l'entrée ou la sortie d'un dossier |
| `MIRROR_TELEPORT` | **400 ms** | **À FAIRE** — les miroirs n'ont pas de geste (fiche 08 § 6) ; le vol de caméra, lui, est prêt | Téléportation animée de la caméra vers l'original d'un miroir (`↻`) |
| `PANEL_DISMISS` | **200 ms** | **À FAIRE** — le panneau disparaît d'un coup | Glissement d'éviction d'un panneau du dock tiré vers sa sortie |
| `MINIMAP_SLIDE` | **180 ms** | **À FAIRE** — pas de translation | Translation horizontale de la minimap quand un panneau droit s'ouvre/se ferme |
| `ARROW_PANEL_IN` | **180 ms** | **À FAIRE** — pas de panneau de flèche | Fondu et déploiement du panneau de description Markdown d'une flèche |
| `AUTOSAVE_DEBOUNCE` | **2 000 ms** | **À FAIRE** (fiche 09 § 3.3) | Temporisation d'inactivité avant l'écriture disque du projet |

---

## 2. Fonctions d'Amortissement & Courbes d'Accélération (Easing)

> Les courbes existent et sont tenues par test (`glucose_core::anim`) : l'amorti universel $1 - (1-t)^3$, et les Béziers CSS résolues comme un navigateur les résout — l'ordonnée se lit à l'**abscisse** voulue, et non au paramètre, ce qu'un témoin par bissection indépendante vérifie sur six courbes. Le rebond du dock dépasse bien 1 avant de se caler, et un test l'exige : une implémentation qui rabattrait la sortie dans $[0, 1]$ le détruirait en silence.
>
> La fiche renvoyait ce mécanisme au rendu GPU. Les deux n'ont rien à voir : une animation est une valeur qui dépend du temps, et qui la dessine ensuite ne change ni la courbe, ni la durée, ni la façon de la tester. Le module ne connaît d'ailleurs pas l'heure — l'appelant lui donne le temps écoulé — ce qui rend **toute courbe de Glucose vérifiable sans attendre une milliseconde**.

* **À FAIRE** — Les courbes sont là ; il leur manque leurs gestes : rebond de préhension du dock, transition FLIP de réordonnancement sur 250 ms, slide de la minimap, apparition des popovers.
* **À FAIRE** — **Entrée du toast** : opacité 0 → 1 et `translateY(8px)` → 0 en 180 ms ease-out. Le fondu d'opacité existe (linéaire) ; la translation et la courbe manquent, alors que la courbe, elle, est maintenant disponible.
* **À FAIRE** — **Pulsation de fantôme** : opacité 0,55 ↔ 0,35, pour les fantômes de placement et opérations en cours.

---

## 3. L'Arbitre de Clic & Priorité de Sélection (PICK-1)

> Tenus par test (`hit_priority_suite`, `hit_priority_pick_suite`, et par la souris dans `interactions/resize/tests.rs`) : l'échelle de priorité complète — poignée 0, bords de conteneur 10, flèche 20, image 30, note 40, texte 50, corps de conteneur 60 — avec ses règles de départage (le plus petit conteneur gagne, une membrane ne gagne jamais sur son contenu, le texte est terminal) ; la bande de 14 px des contours ; le rayon de saisie des poignées de 24 px, plafonné à 35 % du petit côté, plancher 6 px.

### 3.1 Échelle de Priorité Absolue (PICK_RANK)
* **À FAIRE** — **Les flèches ne sont pas cliquables.** Le rang 20 et la zone tampon de $24\text{ px}$ autour du tracé sont prévus par l'arbitre, mais le desktop ne fournit jamais de candidat flèche (`arrow_id: None` partout) : aucun test de distance au segment n'existe.
* **Dette** — `collect_candidates_indexed` clone les nœuds proches du curseur à chaque clic pour les présenter en tranches contiguës. En O(local), mais des clones tout de même ; à faire travailler sur des références.

### 3.3 Mécanique du Cycle de Profondeur (« Switch Priority »)
* **À FAIRE** — **Le cycle est écrit dans le noyau et n'est pas branché** (`pick_at_down`, `advance_on_release` : aucun appelant). Le desktop prend toujours le premier candidat. À brancher : premier clic → rang le plus prioritaire ; re-clic au même endroit ($< 8\text{ px}$, $< 2{,}5\text{ s}$, au **relâchement** pour ne jamais gêner un glisser) → l'élément derrière ; arrêt sur un texte ou une note éditable (`terminal`) ; réinitialisation au-delà de 8 px, de 2,5 s, ou sur double-clic.

---

## 4. Alignement Intelligent & Guides Magnétiques (SNAP-1)

> Tenus par test (`smart_align_suite`, 17 tests, et `interactions/resize/tests.rs::test_snap_1_*`) : le seuil de $8$ pixels écran (`SNAP_SCREEN_PX = 8`, seuil monde $= 8 / \text{scale}$, même sensation à tout zoom — `test_snap_move_scale_constant_on_screen`) ; les six axes (bords et centres, `test_snap_move_left_edge`, `test_snap_move_centers`) ; la session figée au départ du geste — `drag.rs` calcule chaque correction depuis la boîte d'origine, jamais depuis la frame précédente (`drag_applied_delta`).

### 4.2 Lignes de Guidage Infinies
* **À VÉRIFIER (fiche 06)** — Ligne guide infinie à travers tout l'écran, $1\text{ px}$ constant à l'écran, pointillés blancs `rgba(255, 255, 255, 0.30)`.

---

## 5. Physique et Tweening des Membranes (MEMB-1 à MEMB-6)

> Tenu (`membrane_space_suite`) : la loi d'échelle déduite $k = \min(1,\ W/\text{étendue}_X,\ H/\text{étendue}_Y)$, jamais stockée ; plafond à 1 ; plancher $0{,}08$ (`MIN_CONTENT_SCALE`) ; l'étirement d'un seul axe ne déforme rien. Et le cadrage du mode focus (`membrane_focus_suite`) : marge de **6 %** (`FIT_PADDING = 0.06`, comme la référence — la fiche disait 10 %), en 320 ms.

### 5.2 Le Tweening Fluide (MEMB-6)
* **À FAIRE** — À l'entrée ou la sortie d'une membrane minimisée : 200 ms, `ease_out_cubic`, interpolation linéaire de la position et de l'échelle sous la courbe ; déclenché **uniquement** sur changement d'appartenance ou de mode (`membershipSignature`), jamais pendant un redimensionnement à la poignée.

### 5.3 Mode Focus & Assombrissement Extérieur
* **À FAIRE** — Le mode focus n'est pas branché (aucun appel de `membrane_focus` dans le desktop) : ni le cadrage animé au double-clic, ni le fondu des éléments extérieurs vers une opacité de $0{,}05$.

---

## 6. Mécanique des Rideaux de Comparaison (Curtains)

> Tenu dans le noyau (`curtain_suite`, 12 tests) : le modèle, les permissions, la géométrie, les ratios par défaut **0,1 replié / 0,9 déployé** (comme la référence — la fiche disait ≈ 0,05 et ≈ 0,60), et la progression exponentielle $r(t + \Delta t) = r(t) + (r_{\text{cible}} - r(t))(1 - e^{-\Delta t / \tau})$ (`test_curtain_panel_animation_monotone`).

* **À FAIRE** — **Le desktop ne dessine aucun rideau** et n'a aucun geste pour en ouvrir : languettes permanentes de $30\text{ px}$ (`TAB_COL`) à la couleur du propriétaire, déploiement au survol du bord droit, repère de caméra et arbitre de sélection propres (`curtainScope`), glisser le rideau sans déplacer la scène.

---

## 7. Gestuelle de la Caméra & Navigation

> Tenus par test : le zoom ancré — le point du monde sous le curseur ne bouge pas (`test_zoom_at_keeps_the_world_point_under_the_cursor_still`), formule unique sur `Viewport::zoom_at`, bornes du geste $0{,}02$–$20$ à la molette (`test_the_wheel_zoom_is_bounded_between_0_02_and_20`) ; le pan à l'espace + clic gauche, au bouton du milieu ou au bouton droit (`mouse.rs`) ; la sélection élastique — seuil de $4\text{ px}$ écran, sélection par boîte englobante (`test_the_rubberband_needs_more_than_four_pixels_and_selects_by_bounding_box`).

### 7.2 Pan & Bouclage de Curseur (Cursor Wrap)
* **À FAIRE** — **Bouclage infini** : le curseur qui atteint un bord de l'écran pendant un pan est téléporté au bord opposé (`set_cursor_position`), pour un défilement continu sans heurter le moniteur.

### 7.3 Sélection Élastique (Lasso / Rubberband)
* **À VÉRIFIER (fiche 06)** — Contour `rgba(255, 255, 255, 0.50)` de $1\text{ px}$, remplissage `rgba(255, 255, 255, 0.03)`.

---

## 8. Rendu à la Demande (Zero-GPU Idle Engine)

> Tenu, et au-delà : la boucle passe en `ControlFlow::Wait` dès qu'aucune animation ni minuterie n'est en cours — elle ne dessine que sur événement ou à l'échéance qu'un toast, un curseur clignotant ou le minuteur lui donnent, sans la fenêtre de 250 ms de la référence. Le toast dit lui-même quand le redessiner (`Toast::repaint_need`, testé) : la boucle n'a plus aucun chiffre à connaître — les siens, dupliqués, avaient divergé une fois.
