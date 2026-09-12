# 08 — Spécification Complète des Nœuds & Fonctionnalités Canvas

> **Rôle de ce document** : fournir la **spécification unitaire et exhaustive** de chaque type d'élément pouvant vivre sur le canvas de Glucose, de ses propriétés, de ses contraintes d'affichage et de ses algorithmes de manipulation.
> Ce document sert de contrat technique pour le portage de toute la couche logique métier en Rust natif.

---

## 1. Les Images (`BoardImage`)

L'image est le premier citoyen de Glucose, conçu comme une table lumineuse pour artistes, chercheurs et concepteurs.

### 1.1 Modèle de Données & Représentation
```rust
pub struct BoardImage {
    pub id: String,
    pub asset: Option<AssetRef>,       // Embed (SHA-256) ou Link (chemin/URL)
    pub x: f64,                        // Centre X en coordonnées monde
    pub y: f64,                        // Centre Y en coordonnées monde
    pub width: f64,                    // Largeur d'affichage monde
    pub height: f64,                   // Hauteur d'affichage monde
    pub original_width: f64,           // Définition native du fichier image
    pub original_height: f64,          // Définition native du fichier image
    pub rotation: f64,                 // Rotation en degrés (0..360)
    pub locked: bool,                  // Verrouillage anti-déplacement
    pub tags: Vec<String>,             // Étiquettes de classement
    pub membrane_id: Option<String>,   // Membrane propriétaire (si capturée)
    pub mirror_of: Option<String>,     // ID de l'original si miroir/alias
    pub is_video: bool,                // Vrai si flux vidéo (MP4/WebM)
    pub fit: Option<ImageFit>,         // Fit::Contain pour vignettes de dossiers
    pub domains: Vec<DomainAssignment>,// Pondération sémantique
    pub temporal_anchor: Option<TemporalAnchor>, // Datation historique
}
```

### 1.2 Pipeline de Culling Spatial & Virtualisation VRAM
Pour afficher 10 000 images sans effondrer la mémoire vive :
1. **Spatial Hashing / Quadtree** :
   Chaque image est enregistrée dans une grille de hachage spatial (`SpatialHash.ts`).
2. **Hystérésis de Visibilité (Anti-Yoyo)** :
   * **Marge de chargement** : $\text{Viewport} + 50\%$ de marge (`loadMargin = base * 0.5`). Toute image entrant dans ce périmètre est décodée.
   * **Marge de conservation** : $\text{Viewport} + 250\%$ de marge (`keepMargin = base * 2.5`). Une image n'est déchargée de la VRAM GPU que si la caméra s'éloigne au-delà de cette distance.
   * *Bénéfice* : Élimine les rafales de chargement/déchargement quand l'utilisateur oscille près du bord de l'écran.
3. **Chargement Progressif par Priorité Radiale** :
   Lors d'un dézoom massif ou d'un "Fit View" (Ctrl+Shift+F), les images ne sont pas décodées toutes ensemble (ce qui créerait un gel de 500ms). Elles sont placées dans une file d'attente triée par distance au **centre de l'écran** : les images centrales s'affichent en premier.
4. **Décodage Multi-Résolution (LOD)** :
   Une image de 8000x6000 affichée sous forme d'une vignette de 100px n'est pas chargée en pleine résolution. Le moteur décode une texture réduite proportionnelle au zoom courant :
   $$\text{pixels}_{\text{texture}} = \text{clamp}(\text{taille}_{\text{écran}} \times 1.25,\, 256,\, \text{résolution}_{\text{native}})$$
   Le ré-affinage vers la haute définition se produit automatiquement si on zoome dessus ($\times 1.8$ du besoin initial).

### 1.3 Manipulation Directe
* **Déplacement** : Glisser-déposer standard au clic gauche (si déverrouillée).
* **Redimensionnement (Poignées de coin)** :
  * **Ancrage par défaut** : Le coin diagonalement opposé à la poignée tirée reste strictement fixe (comportement d'un logiciel de retouche photo).
  * **Ancrage centré (Touche Ctrl)** : Le centre $(x, y)$ reste immobile, la boîte grandit symétriquement dans les 4 directions.
  * **Ratio toujours verrouillé** : L'aspect ratio natif $w / h$ ne peut pas être déformé lors du resize manuel.
* **Verrouillage (Touche 'L')** :
  * Bascule l'état `locked`. Les poignées disparaissent et la carte devient inerte aux gestes de drag. Toast : `"Images verrouillées 🔒"` ou `"déverrouillées 🔓"`.
* **Duplication (Ctrl+D)** :
  * Clone immédiatement les images sélectionnées avec un décalage de $+24\text{px}$ en X et Y.

---

## 2. Les Cartes de Texte & Markdown (`TextAnnotation`)

Glucose traite le texte comme un outil de pensée spatiale, pas comme un simple champ d'étiquette.

### 2.1 Moteur de Rendu Texte
* **Support Markdown GFM** : Titres H1 à H6, listes à puces `-`, citations `>`, tableaux, blocs de code avec syntaxe, texte en gras (`**concept**`) et italique (`*terme*`).
* **Formules Mathématiques KaTeX** :
  * Inline : `$E = mc^2$`
  * Bloc : `$$\int_{-\infty}^{+\infty} e^{-x^2} dx = \sqrt{\pi}$$`
* **Édition In-Place Fluide** :
  * Un double-clic sur le bloc monte un éditeur transparent (`textarea`) superposé exactement à la géométrie de la carte.
  * La police, l'interlignage, les couleurs et le défilement correspondent au millimètre près.
* **Transaction Unique d'Undo (Undo Isolation)** :
  * De l'ouverture de l'éditeur à sa fermeture (`endLiveEdit`), toutes les frappes de touches, suppressions et recalculs automatiques de hauteur forment **une seule et unique entrée dans l'historique d'annulation**. Un seul Ctrl+Z efface la frappe au lieu d'exiger 50 retours arrière lettre par lettre.

### 2.2 Ancrage Fin de Flèches sur le Texte (`TextSelection`)
Une flèche peut être ancrée non pas seulement sur la boîte globale du texte, mais sur un **mot ou passage précis** à l'intérieur du texte (ex. lier la définition d'un concept au mot-clé exact dans un paragraphe).
* **Format des Ancres Textuelles** :
  ```rust
  pub struct TextAnchor {
      pub start: usize,
      pub end: usize,
      pub quote: String,           // Texte exact sélectionné
      pub prefix: Option<String>,  // Contexte avant (résilience aux modifs)
      pub suffix: Option<String>,  // Contexte après
  }
  ```
* En cas de modification du texte, l'algorithme utilise les préfixes et suffixes pour retrouver la citation originale et repositionner la flèche sans casser le graphe de pensée.

---

## 3. Les Notes Adhésives & Opérateurs Logiques (`StickyAnnotation`)

### 3.1 Note Adhésive Classique
* Conçue pour la capture d'idées brèves, brainstorming et tri rapide.
* Palette pastel prédéfinie : Jaune (`#f5c542`), Rose (`#f472b6`), Bleu (`#60a5fa`), Vert (`#4ade80`), Orange (`#fb923c`), Violet (`#c084fc`).
* Coins francs ou très subtilement arrondis, ombre marquée simulant le papier posé sur la table.

### 3.2 Opérateurs Logiques (Phase 5)
En sélectionnant une note et en pressant `Alt+1` à `Alt+4` (ou `Alt+0` pour réinitialiser), la note se métamorphose en un **connecteur logique sémantique** :
* `Alt+1` $\to$ **ET (`AND`)** : Couleur vert menthe `#34d399`. Conjonction positive.
* `Alt+2` $\to$ **OU (`OR`)** : Couleur bleu ciel `#60a5fa`. Alternative / bifurcation.
* `Alt+3` $\to$ **MAIS (`BUT`)** : Couleur ambre `#f59e0b`. Nuance / contre-argument.
* `Alt+4` $\to$ **PARCE QUE (`BECAUSE`)** : Couleur lilas `#a78bfa`. Causalité / justification.
* Format géométrique compact : pilule de $44\text{px}$ de haut, texte gras centré, glow coloré.

---

## 4. Les Flèches & Graphe de Connaissance (`ArrowAnnotation`)

Les flèches relient des idées entre elles et transforment le tableau en un graphe orienté intelligent.

### 4.1 Évitement Dynamique d'Obstacles (Pathfinding)
Quand une flèche relie le nœud A au nœud B, elle ne traverse pas stupidement les cartes intermédiaires.
* **Algorithme (`getDynamicRoute`)** :
  1. Test d'intersection du segment $[A, B]$ avec toutes les boîtes d'obstacles présentes sur le board.
  2. Si un obstacle est heurté, la boîte est étendue d'une marge de sécurité de $32\text{px}$ (`OBSTACLE_PAD = 32`).
  3. Des points de contournement sont générés aux 4 coins étendus (Top-Left, Top-Right, Bottom-Left, Bottom-Right).
  4. L'algorithme choisit récursivement le chemin valide le plus court contournant les obstacles (profondeur max de 10 niveaux).

### 4.2 Coudes & Points de Passage Utilisateur (Waypoints)
* L'utilisateur peut ajouter manuellement des coudes à la flèche en glissant la poignée centrale (`MidHandle`) qui apparaît au survol de chaque segment.
* Les waypoints sont stockés dans `waypoints: Vec<{ x: f64, y: f64 }>`.

### 4.3 Prédicats Sémantiques Prédéfinis
Une flèche peut porter un symbole logique d'épistémologie :
* `"est_precurseur"` : Précède historiquement ou conceptuellement.
* `"contredit"` : Réfute ou s'oppose.
* `"herite_de"` : Dérivation conceptuelle ou taxonomie.
* `"inspire"` : Influence créative ou analogie.
* `"depend_de"` : Dépendance fonctionnelle stricte.
* `"illustre"` : Cas concret ou exemple d'application.

### 4.4 Flèches Portails Inter-Boards
Une flèche dont `target_board_id` est renseigné ne pointe pas sur l'écran courant : elle crée un **portail dimensionnel** vers un autre tableau.
* Visuel : Extrémité ornée d'un double anneau bleu tournant avec la flèche `↗`.
* Interaction : Cliquer sur l'anneau téléporte instantanément l'utilisateur sur le tableau de destination et centre la caméra sur l'élément cible.

---

## 5. Les Dossiers & Sous-Canvases (`CanvasFolder`)

Un dossier n'est pas une simple boîte graphique : c'est un **portail vers un univers complet**.

### 5.1 Architecture des Sous-Canvases
* Chaque `CanvasFolder` possède un identifiant `child_board_id`.
* Ce `child_board_id` pointe vers un tableau Glucose complet contenant ses propres images, textes, flèches, sous-dossiers et caméras.
* L'imbrication est **infinie** (dossiers dans des dossiers dans des dossiers).

### 5.2 Navigation & Fil d'Ariane
* **Entrée** : Double-clic sur le bandeau ou le corps d'un dossier. La caméra plonge doucement en zoom avant ($400\text{ms}$), puis le canvas charge le board enfant.
* **Fil d'Ariane (`FolderBreadcrumb`)** : Affiché en haut à gauche sous la barre d'outils (`Projet > Recherches > Dossier 1`). Cliquer sur un niveau parent remonte immédiatement d'un ou plusieurs crans.
* **Zoom de Sortie Adaptatif** : Si l'utilisateur dézoome fortement à l'intérieur d'un dossier (au-delà des limites du contenu), la caméra fait automatiquement surface vers le tableau parent sans avoir à cliquer sur un bouton.

### 5.3 Dossiers Miroirs du Disque OS (Folder Mirror)
En faisant glisser un dossier depuis l'Explorateur Windows ou le Finder vers Glucose :
1. **Scan Intelligent Sécurisé** :
   Le backend scanne la hiérarchie locale de fichiers. Les fichiers exécutables et scripts dangereux sont masqués (`SCAN_HIDE_EXTS`).
2. **Scan Paresseux (Lazy Tree Scanning)** :
   Pour éviter de geler sur un répertoire de 50 000 fichiers (ex. un dépôt de code ou une bibliothèque d'assets 3D), seul le premier niveau est scanné immédiatement (`pendingScan = true`). Les sous-dossiers ne sont scannés que lorsque l'utilisateur y pénètre.
3. **Tri Explorateur Windows** :
   L'utilisateur peut trier la disposition automatique des cartes à l'intérieur du dossier :
   * Par nom (`A → Z` ou `Z → A`)
   * Par type d'extension
   * Par taille de fichier (`size-desc`, `size-asc`)
   * Par date de dernière modification (`modified-desc`)

---

## 6. Les Miroirs & Alias Vivants (Phase 4)

Un miroir (`mirror_of`) est une instance clonée d'un nœud existant.

### 6.1 Synchronisation Bidirectionnelle Instantanée
* Modifier le texte, la taille, les domaines ou les étiquettes de n'importe quel miroir modifie **instantanément et simultanément l'original et tous les autres miroirs**.
* Seule la position spatiale $(x, y)$ est propre à chaque copie, permettant de classer un même concept sous plusieurs angles ou dans plusieurs dossiers.

### 6.2 Moteur Anti-Cycle Inception (`mirrorGraph.ts`)
* Il est rigoureusement interdit de créer un miroir d'un miroir qui contiendrait l'original (boucle infinie de causalité).
* Une analyse de graphe orienté acyclique (DAG) valide chaque création de lien (limite de profondeur de chaîne fixée à 16). En cas de tentative cyclique, l'action est refusée : `"Cycle Inception refusé ⚠"`.

### 6.3 Téléportation Caméra
* Chaque miroir porte une pastille ronde bleue avec le symbole `↻` dans son coin supérieur gauche.
* Cliquer dessus déclenche un glissement fluide de caméra de $400\text{ms}$ qui transporte l'utilisateur directement à l'emplacement de l'original (y compris en remontant ou descendant dans les dossiers).

---

## 7. Les Membranes Organisationnelles (`MembraneAnnotation`)

Les membranes regroupent des éléments en ensembles sémantiques vivants.

### 7.1 Les Trois Modes de Membrane
1. **Mode Classique (`classic`)** :
   Zone délimitée servant de repère visuel. Les éléments contenus conservent leur échelle naturelle de 100%.
2. **Mode Minimisé (`minimized`)** :
   Le contenu est réduit proportionnellement pour tenir dans le cadre de la membrane. Idéal pour condenser de grands ensembles de référence sans encombrer le canvas principal.
3. **Mode Étiré (`stretched`)** :
   La membrane adapte automatiquement sa taille pour englober son contenu plus une marge d'air de $32\text{px}$ (`STRETCH_PADDING = 32`).

### 7.2 Alerte de Collision Élastique (`MembraneStretchAlert`)
Si une membrane en mode étiré tente de grandir mais qu'un autre élément bloque son expansion :
* L'expansion est stoppée net au bord de l'obstacle.
* Une bannière d'alerte non intrusive apparaît au-dessus de la membrane pour expliquer la contrainte et proposer soit de repousser les voisins, soit de repasser en mode classique.

---

## 8. Tuiles de Lancement d'Applications (App Bridge)

Glucose sert de hub de travail unifié capable de dialoguer avec les logiciels de l'ordinateur.

* **Reconnaissance Native d'Icônes** :
  Un fichier `.blend` affiche le logo officiel Blender avec son halo orange caractéristique (`#ea7600`) ; un fichier `.psd` affiche le logo Photoshop bleu (`#31a8ff`) ; un fichier `.rs` affiche le crabe Rust (`#dea584`).
* **Lancement Sécurisé au Double-Clic (`open_in_app`)** :
  Double-cliquer sur la tuile lance le fichier dans son logiciel dédié via `ShellExecute` OS.
* **Protection RCE Absolue (Deny-List)** :
  Le backend Rust applique une liste de blocage stricte (`FORBIDDEN_OPEN_EXTS`) refusant l'exécution de tout binaire ou script (`.exe`, `.bat`, `.cmd`, `.ps1`, `.vbs`, `.js`, `.sh`, `.scr`, `.com`, `.jar`). Ces fichiers peuvent être affichés sur le canvas mais ne seront jamais exécutés directement.
* **Réassociation de Liens Rompus** :
  Si un fichier local a été renommé ou déplacé sur le disque dur, la tuile passe en bordure pointillée rouge (`#ef4444`). Cliquer sur le badge ouvre un sélecteur de fichier permettant de réassocier le chemin en un geste.
