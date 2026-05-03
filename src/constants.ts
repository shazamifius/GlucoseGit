// CLEANUP C-04 — Constantes centralisées du projet.
// Toute valeur "magic" qui apparaît plusieurs fois ou dont la sémantique est
// non-évidente doit être nommée ici plutôt qu'inline.

// ════════════════════════════════════════════════════════════════════════════
// Viewport / world space
// ════════════════════════════════════════════════════════════════════════════

export const VIEWPORT = {
  /** Echelle minimum (très dézoomé). En dessous → coords flottantes instables. */
  MIN_SCALE: 0.005,
  /** Echelle maximum (très zoomé). Au dessus → invariants subpixel cassés. */
  MAX_SCALE: 50,
  /** Bornes ±x/y world-space. Au-delà → erreurs Float32 PixiJS. */
  COORD_LIMIT: 1_000_000,
  /** Pas de pan via le pavé numérique (4/6/8/2) en pixels écran. */
  PAN_STEP: 60,
  /** Pas de zoom via pavé numérique (+/-) — ratio multiplicatif. */
  ZOOM_STEP: 1.1,
  /** Marge de pré-affichage côté culling spatial (50% de la viewport). */
  CULLING_MARGIN_RATIO: 0.5,
} as const;

// ════════════════════════════════════════════════════════════════════════════
// LOD (Level of Detail)
// ════════════════════════════════════════════════════════════════════════════

export const LOD = {
  /** Bascule macro→meso. À scale=1.0 (zoom par défaut), on est déjà en micro. */
  MACRO_TO_MESO: 0.25,
  /** Bascule meso→micro. */
  MESO_TO_MICRO: 0.55,
  /** Truncation des titres en mode meso (en caractères). */
  TITLE_TRUNCATE: 60,
} as const;

// ════════════════════════════════════════════════════════════════════════════
// Animation timings (millisecondes)
// ════════════════════════════════════════════════════════════════════════════

export const TIMING = {
  /** Animation enter/exit folder (Phase 4.5). */
  FOLDER_TRANSITION: 400,
  /** Téléportation viewport vers l'original d'un miroir. */
  MIRROR_TELEPORT: 400,
  /** Fade-in panel description de flèche (Phase 5). */
  ARROW_PANEL_IN: 180,
  /** Glissement minimap quand un panel droit s'ouvre/ferme. */
  MINIMAP_SLIDE: 180,
  /** Fenêtre de double-click (annotation, dossier). */
  DOUBLE_CLICK_WINDOW: 350,
  /** Délai avant fermeture des notifications toast. */
  TOAST_DURATION: 2400,
  /** Animation de dismissal des panels du dock. */
  PANEL_DISMISS: 200,
} as const;

// ════════════════════════════════════════════════════════════════════════════
// Clustering / Membranes (Union-Find + Convex Hull)
// ════════════════════════════════════════════════════════════════════════════

export const CLUSTERING = {
  /** Distance max world-pixels pour fusionner 2 images dans le même cluster. */
  CLUSTER_DIST: 600,
  /** Padding autour des images pour le contour de membrane. */
  MEMBRANE_PAD: 80,
  /** Saturation HSL pour les membranes sans domaine assigné. */
  HUE_SATURATION_FALLBACK: 65,
  /** Saturation HSL pour les membranes dominées par un domaine. */
  HUE_SATURATION_DOMAIN: 70,
  /** Lightness HSL des membranes. */
  HUE_LIGHTNESS: 55,
} as const;

// ════════════════════════════════════════════════════════════════════════════
// Symbiose chromatique (HtmlAnnotationLayer.getSymbioticHue)
// ════════════════════════════════════════════════════════════════════════════

export const SYMBIOSIS = {
  /** Rayon en world-pixels où une annotation influence la teinte des voisines. */
  RADIUS: 1200,
  /** Influence max d'un voisin sur la teinte d'une annotation (0..1). */
  INFLUENCE_MAX: 0.5,
  /** Variation pseudo-aléatoire par ID autour de la teinte de zone. */
  HUE_JITTER: 80,
  /** Échelle Perlin du bruit de zone (pixels world). */
  ZONE_NOISE_SCALE: 2000,
} as const;

// ════════════════════════════════════════════════════════════════════════════
// Pathfinding flèches (ArrowSvgLayer.getDynamicRoute)
// ════════════════════════════════════════════════════════════════════════════

export const PATHFINDING = {
  /** Padding autour d'un obstacle évité. */
  OBSTACLE_PAD: 32,
  /** Profondeur max de récursion (évite les boucles infinies). */
  MAX_DEPTH: 10,
  /** Distance de snap d'une flèche vers le bord d'un nœud. */
  SNAP_DIST: 120,
} as const;

// ════════════════════════════════════════════════════════════════════════════
// Miroirs / Alias (Phase 4)
// ════════════════════════════════════════════════════════════════════════════

export const MIRROR = {
  /** Décalage entre l'original et le miroir créé (Ctrl+Shift+M). */
  OFFSET: 40,
  /** Profondeur max de la chaîne `mirrorOf` (anti-cycle). */
  CHAIN_LIMIT: 16,
  /** Seuil de poids domaine pour afficher un badge sur un nœud. */
  DOMAIN_BADGE_THRESHOLD: 0.4,
} as const;

// ════════════════════════════════════════════════════════════════════════════
// UI / panels
// ════════════════════════════════════════════════════════════════════════════

export const PANEL = {
  /** Largeur panel droit (DomainsPanel, PresetPanel). */
  RIGHT_PANEL_WIDTH: 320,
  /** Décalage minimap quand panel droit ouvert. */
  MINIMAP_OFFSET: 332,
  /** Ouverture/fermeture distance gap. */
  MINIMAP_GAP: 12,
} as const;

// ════════════════════════════════════════════════════════════════════════════
// Limites de tailles (anti-DoS / robustesse)
// ════════════════════════════════════════════════════════════════════════════

export const LIMITS = {
  /** Taille max du texte d'une annotation (en chars). */
  MAX_TEXT_LENGTH: 50_000,
  /** Taille max d'un fichier .glucose chargé (en octets). */
  MAX_PROJECT_FILE_SIZE: 200 * 1024 * 1024,
  /** Taille max d'un fichier image téléchargé (en octets). */
  MAX_IMAGE_FILE_SIZE: 100 * 1024 * 1024,
  /** Niveaux d'undo conservés. */
  UNDO_DEPTH: 50,
} as const;
