# 07 — Spécification des Animations, Physique & Micro-Interactions

> **Rôle de ce document** : définir avec une précision chronométrique et mathématique absolue l'intégralité des **animations, courbes d'accélération, cinétiques, transitions d'état et règles d'interaction** de Glucose.
> Il constitue la référence indispensable pour recréer en Rust natif le ressenti fluide, réactif et organique ("feel & polish") de la version originale.

---

## 1. Table Chronométrique & Délais de Référence (TIMING)

L'application repose sur des constantes temporelles unifiées (extraites de `src/constants.ts` et des composants canvas) :

| Constante | Durée (ms) | Description & Comportement |
|---|---:|---|
| `MEMBRANE_TWEEN` | **200 ms** | Passage fluide d'une image/texte entrant ou sortant d'une membrane minimisée |
| `FOLDER_TRANSITION`| **400 ms** | Plongée fluide de la caméra lors de l'entrée ou la sortie d'un dossier (sous-canvas) |
| `MIRROR_TELEPORT` | **400 ms** | Téléportation animée de la caméra vers l'original d'un miroir (`↻`) |
| `PANEL_DISMISS` | **200 ms** | Glissement d'éviction et disparition d'un panneau du dock tiré vers sa sortie |
| `MINIMAP_SLIDE` | **180 ms** | Translation horizontale de la minimap quand un panneau droit s'ouvre/se ferme |
| `ARROW_PANEL_IN` | **180 ms** | Fondu et déploiement du panneau de description Markdown d'une flèche |
| `TOAST_DURATION` | **2 400 ms** | Temps de vie à l'écran d'une notification toast avant son auto-destruction |
| `TOAST_ANIM_IN` | **180 ms** | Trajectoire d'entrée verticale et apparition en opacité d'un toast |
| `DOUBLE_CLICK_WINDOW`| **350–400 ms**| Fenêtre temporelle maximale pour valider un double-clic (édition texte, dossier) |
| `CYCLE_TTL_MS` | **2 500 ms** | Délai d'expiration du cycle de sélection : au-delà, le clic repart au rang prioritaire |
| `AUTOSAVE_DEBOUNCE` | **2 000 ms** | Temporisation d'inactivité avant déclenchement de l'écriture disque du projet |

---

## 2. Fonctions d'Amortissement & Courbes d'Accélération (Easing)

Toutes les animations de Glucose refusent les mouvements linéaires brutaux. Elles emploient des fonctions d'amortissement spécifiques :

```
          1.0 +-------------+ (Arrivée douce)
              |            /
              |           /
              |         /
              |       /
          0.0 +------+
              0.0   0.5   1.0
```

### 2.1 Cubic Ease-Out (Amorti Universel)
Utilisé pour le tweening des membranes (`membraneTween.ts`), le cadrage caméra du mode Focus (`membraneFocus.ts`), le slide de la minimap et les apparitions de popovers :
$$f(t) = 1 - (1 - t)^3 \quad \text{pour } t \in [0, 1]$$

*Implémentation Rust :*
```rust
pub fn ease_out_cubic(t: f32) -> f32 {
    let c = t.clamp(0.0, 1.0);
    1.0 - (1.0 - c).powi(3)
}
```

### 2.2 Rebond Élastique de Préhension (Dock Panel Bounce)
Lorsqu'un panneau du dock s'ouvre ou est relâché après un léger glissement (`PanelDock.tsx`), il utilise une courbe de Bézier cubique avec dépassement élastique (overshoot de 156%) :
$$\text{cubic-bezier}(0.34,\, 1.56,\, 0.64,\, 1.0)$$

*Effet ressenti* : Le panneau monte légèrement au-dessus de sa position finale avant de se caler fermement, donnant une sensation tactile mécanique très gratifiante.

### 2.3 Transition FLIP de Réordonnancement (Dock Swap)
Lorsqu'on glisse un panneau horizontalement pour échanger sa place avec un voisin :
$$\text{cubic-bezier}(0.22,\, 1.0,\, 0.36,\, 1.0) \quad \text{sur } 250\text{ ms}$$

### 2.4 Animation Toast (Slide-Up)
Entrée du toast :
```css
@keyframes toastIn {
  from { opacity: 0; transform: translateY(8px); }
  to   { opacity: 1; transform: translateY(0); }
}
/* transition: 180ms ease-out */
```

### 2.5 Pulsation de Fantôme (Ghost Pulse)
Utilisé pour les fantômes de placement et les opérations en cours :
```css
@keyframes ghostPulse {
  0%, 100% { opacity: 0.55; }
  50%       { opacity: 0.35; }
}
```

---

## 3. L'Arbitre de Clic & Priorité de Sélection (PICK-1)

Dans une interface canvas multi-couches, les éléments se superposent (images, textes, membranes, flèches).
**Le problème fondamental** : Si le clic dépend de l'ordre d'affichage, une grande membrane transparente avale tous les clics sur les images qu'elle contient.
**La solution de Glucose (PICK-1)** : Un arbitrage géométrique strict basé sur **l'intention de l'utilisateur**.

### 3.1 Échelle de Priorité Absolue (PICK_RANK)

```
[Rang 0]  Poignées de redimensionnement de la sélection active  (Intention absolue)
   |
[Rang 10] Bords de conteneurs (Pointillés de membrane, header de dossier)
   |
[Rang 20] Flèches (Tracé fin, difficile à viser)
   |
[Rang 30] Images
   |
[Rang 40] Notes Sticky
   |
[Rang 50] Blocs de Texte (Dernier parmi les contenus, ouvre l'édition)
   |
[Rang 60] Corps intérieur de conteneur (Une membrane ne gagne jamais sur son contenu)
```

| Rang | Type d'élément | Règle de départage |
|:---:|---|---|
| **0** | **Poignée de sélection (`handle`)** | Gagne **toujours**. Permet de redimensionner même si un autre élément passe sous la poignée. |
| **10** | **Bord de conteneur (`membrane-edge`, `folder-edge`)** | Bande de $14\text{px}$ écran autour du contour. Entre 2 conteneurs, **le plus petit en surface gagne**. |
| **20** | **Flèche (`arrow`)** | Ligne vectorielle avec zone tampon de $24\text{px}$. |
| **30** | **Image (`image`)** | Rectangle de texture. Entre 2 images superposées, celle peinte au-dessus (z-index) gagne. |
| **40** | **Note adhésive (`sticky`)** | Rectangle de la note. |
| **50** | **Bloc de texte (`text`)** | Toujours **dernier des contenus**. Double-clic = ouverture édition (état terminal). |
| **60** | **Corps intérieur de conteneur (`membrane-body`, `folder-body`)** | Remplissage intérieur. Ne capture le clic que si aucun contenu n'est sous le curseur. |

### 3.2 Tolérance de Préhension des Poignées (Hitbox Magnétique)
Le carré visuel d'une poignée de coin ne mesure que $9\text{px}$ à l'écran. Viser précisément $9\text{px}$ à la souris est irritant.
* **Rayon de saisie étendu** : $24\text{px}$ écran (`PICK.HANDLE_SLOP_PX = 24`).
* **Plafond de sécurité proportionnel** : Une poignée ne doit jamais couvrir plus de $35\%$ du petit côté de la boîte (`HANDLE_SLOP_MAX_RATIO = 0.35`), avec un plancher de $6\text{px}$ (`HANDLE_SLOP_MIN_PX = 6`). Cela empêche les poignées d'une carte miniature dézoomée de saturer tout l'intérieur de la carte.

### 3.3 Mécanique du Cycle de Profondeur (« Switch Priority »)
Quand plusieurs éléments sont empilés sous le curseur :
1. **Premier clic** : Sélectionne la cible de rang le plus prioritaire (ex. l'image au premier plan).
2. **Re-clic au même endroit sans bouger** (déplacement $< 8\text{px}$ écran) :
   * L'arbitre avance d'un cran dans la liste des candidats ordonnés et sélectionne l'élément situé derrière (ex. la membrane qui englobe l'image).
   * **Le saut s'effectue au RELÂCHEMENT du bouton (pointerup)**, jamais à l'enfoncement (pointerdown). Ainsi, si l'utilisateur appuie et glisse, il déplace l'élément courant sans déclencher de saut intempestif.
3. **Terminus du cycle (`terminal: true`)** :
   * Si la cible atteinte est un texte ou un sticky éditable, le cycle s'arrête là : le clic suivant doit ouvrir l'éditeur de texte et non basculer sur un conteneur d'arrière-plan.
4. **Réinitialisation du cycle** :
   * Le cycle est réinitialisé si le curseur bouge de plus de $8\text{px}$, si plus de $2.5\text{ s}$ s'écoulent (`CYCLE_TTL_MS = 2500`), ou lors d'un double-clic ($< 400\text{ ms}$).

---

## 4. Alignement Intelligent & Guides Magnétiques (SNAP-1)

L'alignement intelligent assiste le déplacement, le redimensionnement et la création de tout élément sur le canvas.

### 4.1 Invariants du Moteur de Snap
1. **Seuil d'accroche fixe à l'écran** :
   L'aimant s'active dès que la distance à une cible est inférieure à **$8\text{ pixels écran}$** (`SNAP_SCREEN_PX = 8`). En coordonnées monde :
   $$\text{seuil}_{\text{monde}} = \frac{8}{\text{scale}}$$
   *Conséquence* : L'aimant a exactement la même sensation physique de force à $10\%$ de zoom comme à $500\%$ de zoom.
2. **Session figée au départ du geste (Anti-Drift)** :
   Lorsqu'un déplacement commence à $(x_0, y_0)$, la boîte de départ est mémorisée. À chaque frame de déplacement, la correction $(\Delta x, \Delta y)$ est calculée par rapport à la position d'origine absolue, **jamais par rapport à la frame précédente**.
   *Conséquence* : L'aimant n'accumule aucune dérive sous la souris. Si on écarte la souris de plus de 8px, la carte se décroche immédiatement et revient exactement sous le curseur.

### 4.2 Lignes de Guidage Infinies
Quand un bord s'aligne magnétiquement sur un voisin :
* Une ligne guide infinie apparaît à travers tout l'écran.
* Épaisseur : $1\text{px}$ constant à l'écran ($\frac{1}{\text{scale}}$ en monde).
* Style : Pointillés blancs semi-transparents : `border: 1px dashed rgba(255, 255, 255, 0.30)`.
* Les 6 axes alignables :
  * Horizontaux : bord haut ($y_1$), centre vertical ($y_c$), bord bas ($y_2$).
  * Verticaux : bord gauche ($x_1$), centre horizontal ($x_c$), bord droit ($x_2$).

---

## 5. Physique et Tweening des Membranes (MEMB-1 à MEMB-6)

Une membrane est une région vivante qui adapte la taille de son contenu.

### 5.1 La Loi d'Échelle Déduite (Sans Stockage d'Échelle)
Le taux d'échelle $k$ d'une membrane minimisée n'est **jamais stocké dans la base de données** ; il est recalculé dynamiquement :
$$k = \min\left(1.0,\, \frac{W_{\text{membrane}}}{\text{étendue}_X},\, \frac{H_{\text{membrane}}}{\text{étendue}_Y}\right)$$
où $\text{étendue}_X$ et $\text{étendue}_Y$ mesurent l'encombrement du contenu naturel.
* **Règle du Min** : Étirer la membrane sur un seul axe ne déforme pas les images ; le ratio reste isotrope.
* **Plafond à 1.0** : Agrandir une membrane minimisée ramène d'abord son contenu à $100\%$ de sa taille d'origine, puis crée du vide autour.
* **Plancher de lisibilité** : $k \ge 0.08$ (`MIN_CONTENT_SCALE = 0.08`).

### 5.2 Le Tweening Fluide (MEMB-6)
Quand une image entre ou sort d'une membrane minimisée :
* **Durée** : $200\text{ ms}$ (`MEMBRANE_TWEEN.MS = 200`).
* **Courbe** : `ease_out_cubic`.
* **Interpolation** :
  $$\text{pos}(t) = \text{pos}_{\text{départ}} + (\text{pos}_{\text{arrivée}} - \text{pos}_{\text{départ}}) \times \text{ease}(t)$$
  $$\text{scale}(t) = \text{scale}_{\text{départ}} + (\text{scale}_{\text{arrivée}} - \text{scale}_{\text{départ}}) \times \text{ease}(t)$$
* **Déclencheur discret** : L'animation démarre **uniquement** sur changement d'appartenance ou changement de mode (`membershipSignature`), jamais pendant le redimensionnement manuel à la poignée (qui reste instantané pour ne pas introduire de lag).

### 5.3 Mode Focus & Assombrissement Extérieur
Au zoom ou double-clic sur une membrane :
* La caméra glisse doucement pour cadrer la boîte de membrane au centre de l'écran avec une marge de $10\%$.
* Tous les éléments extérieurs à la membrane subissent un fondu d'opacité vers $0.05$ (assombrissement au noir), isolant visuellement l'utilisateur dans son espace de travail courant.

---

## 6. Mécanique des Rideaux de Comparaison (Curtains)

Le rideau est un panneau latéral coulissant attaché à une membrane (mode Focus), permettant de confronter deux versions ou de tenir un carnet privé.

```
+---------------------------+====+ (Poignée / Languette 30px)
|                           | R  |
|      Canvas Membrane      | I  | <-- Se déploie de droite à gauche
|                           | D  |     au survol ou glisser
|                           | E  |
+---------------------------+====+
```

1. **Languettes permanentes** :
   * Largeur fixe : $30\text{px}$ écran (`TAB_COL = 30`).
   * Reste visible même replié, à la couleur du propriétaire du rideau.
2. **Déploiement au survol** :
   * Seuil de déclenchement : approche du bord droit.
   * Progression continue du ratio : de `collapsedRatio` ($\approx 0.05$) à `expandedRatio` ($\approx 0.60$).
   * Calcul par frame :
     $$r(t + \Delta t) = r(t) + (r_{\text{cible}} - r(t)) \times \left(1 - e^{-\frac{\Delta t}{\tau}}\right)$$
3. **Monde de clic indépendant** :
   * Le rideau possède son propre repère de caméra (`glucose:curtain-viewport`) et son propre arbitre de sélection (`curtainScope`).
   * Faire glisser le rideau ne déplace jamais la scène arrière.

---

## 7. Gestuelle de la Caméra & Navigation

### 7.1 Zoom Ancré sous le Curseur
Le zoom à la molette ou trackpad maintient le point du monde situé sous le curseur de souris rigoureusement immobile à l'écran.

*Formule mathématique :*
Soit $(c_x, c_y)$ les coordonnées de la souris en pixels écran, $(v_x, v_y)$ le décalage de la caméra, et $s$ son échelle courante :
$$s_{\text{nouveau}} = \text{clamp}(s \times \text{facteur},\, 0.02,\, 20.0)$$
$$v'_x = c_x - (c_x - v_x) \times \frac{s_{\text{nouveau}}}{s}$$
$$v'_y = c_y - (c_y - v_y) \times \frac{s_{\text{nouveau}}}{s}$$

### 7.2 Pan & Bouclage de Curseur (Cursor Wrap)
* Déplacement activé par : barre d'espace + clic gauche, clic molette (bouton 1) ou clic droit (bouton 2).
* **Bouclage infini aux bords de l'écran** :
  Quand le curseur atteint le bord gauche de l'écran pendant un pan, il est téléporté instantanément au bord droit (et inversement) via `setCursorPosition` OS, permettant un défilement continu sur des millions de pixels sans heurter les limites physiques du moniteur.

### 7.3 Sélection Élastique (Lasso / Rubberband)
* Déclenchée par clic gauche maintenu sur le vide du canvas.
* **Seuil d'activation** : Déplacement $> 4\text{px}$ écran (évite les micro-sélections accidentelles sur simple clic).
* **Rendu visuel** :
  * Rectangle en coordonnées écran.
  * Contour : `rgba(255, 255, 255, 0.50)`, épaisseur $1\text{px}$.
  * Remplissage intérieur : `rgba(255, 255, 255, 0.03)`.
* **Sélection AABB** : Tout nœud dont la boîte englobante intersecte le rectangle élastique est ajouté à la sélection multi-éléments.

---

## 8. Rendu à la Demande (Zero-GPU Idle Engine)

L'application ne tourne **jamais** en boucle de rendu continue à 60 ou 144 FPS quand l'utilisateur ne fait rien :
1. Une variable `renderUntil` mémorise l'horodatage futur jusqu'auquel le rafraîchissement d'écran est requis.
2. Chaque déplacement de souris, touche clavier, frame d'animation ou réception de message réseau repousse `renderUntil = now + 250ms`.
3. Dès que $now > renderUntil$ et qu'aucune vidéo n'est en cours de lecture : la boucle d'affichage s'endort totalement. **Consommation GPU = 0%**.
