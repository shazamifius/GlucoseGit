// =============================================================================
//  CERVEAU v2 — Raspberry Pi Pico (RP2040) Texture Brain
//  Image → Analyse mathématique → Shader HLSL procédural (one-shot, sans IA)
//
//  Compilez (PC / debug) :
//      g++ -O2 -std=c++17 main.cpp -o cerveau && ./cerveau
//  Compilez (Pico) :
//      arm-none-eabi-gcc -O2 -mthumb -mcpu=cortex-m0plus -DPICO_BUILD ...
//
//  CONTRAINTES MATÉRIELLES :
//   - Cortex-M0+ : ZERO FPU matériel → arithmétique entière pure (aucun float)
//   - 264 Ko SRAM totaux → allocations statiques uniquement (ZERO malloc)
//   - Lecture pixels en flux continu → ZERO stockage de l'image complète
// =============================================================================

#include <stdint.h>
#include <string.h>

// ---- Pont de compilation PC ↔ Pico ----------------------------------------
#ifndef PICO_BUILD
  #include <stdio.h>
  // DBG_PRINTF → stderr : le stdout est réservé au shader HLSL pur (redirection > fichier)
  #define DBG_PRINTF(...) fprintf(stderr, __VA_ARGS__)
  // stb_image — chargeur JPEG/PNG header-only (PC uniquement, jamais sur Pico)
  #define STB_IMAGE_IMPLEMENTATION
  #include "stb_image.h"
#else
  #include "pico/stdlib.h"
  // Sur Pico, redirigez vers stdio USB ou UART selon votre config :
  //   stdio_init_all() + printf redirige automatiquement si PICO_STDIO_USB=1
  #define DBG_PRINTF printf
#endif

// =============================================================================
//  CONFIGURATION IMAGE
//  Adaptez IMG_WIDTH / IMG_HEIGHT au flux réel (capteur, carte SD, UART host)
// =============================================================================
#define IMG_WIDTH        320u
#define IMG_HEIGHT       240u
#define TOTAL_PIXELS     ((uint32_t)(IMG_WIDTH) * (IMG_HEIGHT))   // 76 800

// ---- GLCM : quantification 4 bits (16 niveaux, matrice 16×16) ---------------
#define GLCM_LEVELS      16u
#define GLCM_SHIFT       4u    // uint8_t lum >> 4  →  index 0–15

// ---- Gabor : ring buffer de scanlines pour la convolution glissante ---------
#define GABOR_ANGLES     4u    // 0°, 45°, 90°, 135°
#define GABOR_RING_ROWS  5u    // 5 lignes = fenêtre de convolution 5×5
#define GABOR_KERN_SIZE  5u    // largeur du noyau (= GABOR_RING_ROWS, carré)

// ---- Mode de test du flux simulé -------------------------------------------
// Décommentez UNE des lignes ci-dessous pour changer le pattern de test.
// En production Pico, ces modes sont inactifs (get_next_pixel lit l'UART).
//
// [commenté]         → MODE BOIS  : rayures horizontales + LCG 4-bit
//                       Attendu : has_stripes=1, has_grain=0, FBM directionnel
//
// #define TEST_MODE_STONE  → MODE PIERRE/SABLE : bruit LCG isotrope
//                            Attendu : has_grain=1, has_stripes=0, Voronoï
//
// #define TEST_MODE_STONE

// ---- Taille max des images (PC uniquement) ----------------------------------
// Sur Pico : IMG_MAX_WIDTH = IMG_WIDTH (pas d'image réelle, flux UART)
// Sur PC   : 1024 couvre image.png 950×868 et les image de test 512×512
#ifdef PICO_BUILD
  #define IMG_MAX_WIDTH  IMG_WIDTH
  #define IMG_MAX_HEIGHT IMG_HEIGHT
#else
  #define IMG_MAX_WIDTH  2048u
  #define IMG_MAX_HEIGHT 2048u
#endif

// ---- Pyramide Gabor 3 niveaux -----------------------------------------------
//
//  Niveau 0 — MICRO  (GaborState)     décimation ×1  λ=4px   grain, JPEG, fibres
//  Niveau 1 — LOW    (GaborStateLow)  décimation ×4  λ=16px  strates moyennes
//  Niveau 2 — MACRO  (GaborStateMacro)décimation ×16 λ=64px  strates géologiques
//
//  Le même noyau 5×5 à λ=4px sur des pixels décimés ×D voit λ×D dans l'original.
//  Coût CPU décroît comme D² : MACRO = ÷256 du coût MICRO. Négligeable.
//
//  RAM MACRO sur Pico : 5 × (320/16) = 5 × 20 = 100 B
//  RAM MACRO sur PC   : 5 × (1024/16) = 5 × 64 = 320 B
//
#define GABOR_DECIM       4u   // facteur entre niveaux consécutifs
#define GABOR_DECIM_MACRO 16u  // facteur total MICRO→MACRO (= DECIM²)
#define IMG_MAX_WIDTH_LOW   (IMG_MAX_WIDTH / GABOR_DECIM)        // PC:256  Pico:80
#define IMG_MAX_WIDTH_MACRO (IMG_MAX_WIDTH / GABOR_DECIM_MACRO)  // PC:64   Pico:20


// =============================================================================
//  RAPPORT RAM (statique, vérifié à la conception)
//
//  Structure          sizeof   Notes
//  ─────────────────────────────────────────────────────────────────────────
//  WelfordState          32    (padding 4 B après sum_b → alignement int64_t)
//  GLCMState          1 032    (1024 matrix + 3 B padding + 4 total_pairs + 1 prev)
//  GaborState         1 640    (rows 1600 B en tête → energy[4] aligné sans gap)
//  TextureSignature      32    (~10 uint8_t + 2 B pad + 4×uint32_t)
//  gabor_row_buf (stack) 320   (uint8_t[IMG_WIDTH], boucle principale)
//  ─────────────────────────────────────────────────────────────────────────
//  TOTAL               3 056 octets  = 1.16 % des 264 Ko disponibles
// =============================================================================


// =============================================================================
//  STRUCTURES DE DONNÉES
// =============================================================================

// -----------------------------------------------------------------------------
/**
 * WelfordState — Statistiques streaming (mean + variance) en une passe O(n)
 *
 * IMPLÉMENTATION : accumulation de sommes exactes (König-Huygens).
 *
 * Pourquoi PAS le Welford online pur (mean += delta/n) ?
 *   Sur M0+, `delta / n` est une division entière. Pour n = 76 800,
 *   un delta de 200 donne 200/76800 = 0 → la moyenne se "bloque" sur la
 *   valeur des premiers pixels et ne converge jamais. Bug de troncature fatal.
 *
 * Solution retenue : accumuler Σlum et Σlum², calculer mean/variance en fin.
 *   mean     = sum_lum / n
 *   variance = sum_sq_lum / n  −  (sum_lum / n)²     [formule König-Huygens]
 *
 * Stabilité numérique pour données 8 bits (0–255) :
 *   Annulation catastrophique impossible : sum_sq/n ≥ mean² toujours (Jensen).
 *   Différence max utile ≈ 65025 − 0 = 65025 → tient dans int32_t. ✓
 *
 * Overflow :
 *   sum_lum    max = 255 × 76 800 = 19 584 000        → uint32_t ✓
 *   sum_sq_lum max = 65025 × 76 800 = 4 993 920 000   → déborde uint32_t ! uint64_t. ✓
 */
typedef struct {
    uint32_t n;           // nombre de pixels traités
    uint32_t sum_lum;     // Σ luminance     (max 19.6 M → uint32_t ✓)
    uint64_t sum_sq_lum;  // Σ luminance²    (max 4.99 B → uint64_t obligatoire)
    uint32_t sum_r;       // Σ canal R (pour couleur dominante en fin de passe)
    uint32_t sum_g;       // Σ canal G
    uint32_t sum_b;       // Σ canal B
    // Histogramme 4 bins (remplace _pad) — max 4×2048² = 16 M par bin → uint32_t ✓
    uint32_t hist_rsum[4];  // Σ R par bin de luminance
    uint32_t hist_gsum[4];  // Σ G par bin
    uint32_t hist_bsum[4];  // Σ B par bin
    uint32_t hist_cnt[4];   // nb pixels par bin
} WelfordState;             // sizeof = 80 B

// -----------------------------------------------------------------------------
/**
 * GLCMState — Matrice de Co-occurrence des Niveaux de Gris (GLCM / Haralick)
 *
 * Direction : horizontale (paires de pixels adjacents sur la même ligne).
 * Symétrie  : chaque paire (i,j) incrémente matrix[i][j] ET matrix[j][i].
 *
 * Quantification 4 bits : luminance 0–255 → index 0–15 (>> 4).
 * Matrice 16×16 = 256 cellules. Coût : 256 × 4 = 1 024 octets (uint32_t).
 *
 * Pourquoi uint32_t et non uint16_t ?
 *   Pairs horizontaux max = 319 × 240 = 76 560 > uint16_t max (65 535).
 *   Cas diagonal (i == j) : deux incréments par paire → max 153 120.
 *   uint32_t est obligatoire. ✓
 *
 * Produit Phase 2 (Haralick) :
 *   Contraste    = Σ (i−j)² × p(i,j)          → texture rude/creusée
 *   Homogénéité  = Σ p(i,j) / (1 + |i−j|)     → texture lisse/uniforme
 *   Entropie     = −Σ p(i,j) × log₂(p(i,j))   → désordre / complexité
 */
typedef struct {
    uint32_t matrix[GLCM_LEVELS][GLCM_LEVELS]; // 1 024 octets
    uint32_t total_pairs;                        // paires accumulées
    uint8_t  prev_q4;                            // lum quantifiée du pixel précédent
    uint8_t  _pad[3];                            // padding explicite → sizeof = 1 032
} GLCMState;

// -----------------------------------------------------------------------------
/**
 * GaborState — Banque de filtres de Gabor (détection d'anisotropie)
 *
 * Les filtres de Gabor (sinusoïde × gaussienne) imitent les cellules simples
 * du cortex visuel : chaque orientation détecte les structures dans sa direction.
 *   0° = rayures horizontales (bois, strates rocheuses)
 *  45° = diagonales
 *  90° = rayures verticales (écorce, fibres)
 * 135° = anti-diagonales
 *
 * Implémentation streaming : ring buffer de GABOR_RING_ROWS scanlines.
 * Quand le buffer est plein, on applique la convolution 5×5 sur la fenêtre.
 * Noyaux pré-calculés en flash (const → ROM sur Pico), format Q7 (×128).
 *
 * NOTE : la convolution réelle est implémentée en Phase 1b (prochaine étape).
 *        Cette phase-ci stocke les scanlines et prépare la structure.
 *
 * Astuce d'alignement : rows[] en premier → energy[4] (int64_t) aligné sans gap. ✓
 */
typedef struct {
    uint8_t  rows[GABOR_RING_ROWS][IMG_MAX_WIDTH]; // ring buffer scanlines (PC:5120B / Pico:1600B)
    int64_t  energy[GABOR_ANGLES];             // énergie par orientation (32 B)
    uint32_t n_windows;                         // fenêtres de convolution traitées
    uint8_t  head;                             // tête du ring buffer (0–RING_ROWS−1)
    uint8_t  filled;                           // lignes valides (0–RING_ROWS)
    uint8_t  _pad[2];
} GaborState;

// -----------------------------------------------------------------------------
/**
 * GaborStateLow — Banque Gabor couche basse fréquence (pyramide décimée ×GABOR_DECIM)
 *
 * Même noyau 5×5 à λ=4px, mais appliqué sur un flux sous-échantillonné ×4.
 * Résolution effective : img_w/4 × img_h/4.
 * λ_perçu dans l'image originale = 4px × GABOR_DECIM = 16px.
 *
 * On détecte ainsi les STRATES LARGES (bois ≈20px, roche ≈40-80px) que le
 * Gabor haute résolution est aveugle à détecter (myopie de λ=4px).
 *
 * Coût RAM Pico : 5 × 80 = 400 B + 32 B energy + overhead ≈ 440 B.
 * Coût CPU Pico : ÷16 par rapport au Gabor haute résolution. Négligeable.
 */
typedef struct {
    uint8_t  rows[GABOR_RING_ROWS][IMG_MAX_WIDTH_LOW]; // PC:1280B / Pico:400B
    int64_t  energy[GABOR_ANGLES];
    uint32_t n_windows;
    uint8_t  head;
    uint8_t  filled;
    uint8_t  _pad[2];
} GaborStateLow;

// -----------------------------------------------------------------------------
/**
 * GaborStateMacro — Couche MACRO (décimation ×16, λ=64px dans l'image originale)
 *
 * Détecte les structures géologiques larges : strates épaisses, bancs de roche,
 * draperies, alternances stratigraphiques de 40-80px d'épaisseur dans l'image.
 *
 * RAM Pico  : 5 × 20 = 100 B + 32 B energy + overhead ≈ 140 B total.
 * RAM PC    : 5 × 64 = 320 B.
 * Coût CPU  : (img_w/16) colonnes × (img_h/16) lignes = ÷256 du niveau MICRO.
 */
typedef struct {
    uint8_t  rows[GABOR_RING_ROWS][IMG_MAX_WIDTH_MACRO]; // PC:320B / Pico:100B
    int64_t  energy[GABOR_ANGLES];
    uint32_t n_windows;
    uint8_t  head;
    uint8_t  filled;
    uint8_t  _pad[2];
} GaborStateMacro;

// -----------------------------------------------------------------------------
/**
 * LBPState — Histogramme LBP (Local Binary Patterns) uniforme 8 voisins rayon 1
 *
 * Calcul en streaming via le ring buffer du Gabor MICRO (aucun RAM supplémentaire
 * pour le stockage des lignes — réutilisation des 5 rows déjà bufférisées).
 *
 * Patron uniforme LBP8,1 : au plus 2 transitions 0↔1 en parcourant les 8 bits.
 *   Bin 0..8 → patrons uniformes (0..8 bits à 1 formant un arc connexe)
 *   Bin 9    → patrons non-uniformes (toutes les autres configurations)
 *
 * Pourquoi LBP ?
 *   - Capture la micro-structure locale : fibres, grains, craquelures, pores
 *   - Invariant au seuil de luminance (comparaison relative)
 *   - Très discriminant pour séparer : bois fin / béton / tissu / crépi / cuir
 *
 * RAM Pico : 10×4 + 4 = 44 octets (hors LUT de 256 B en BSS).
 */
typedef struct {
    uint32_t hist[10];   // histogramme LBP (bins 0..8 = uniforme, bin 9 = non-uniforme)
    uint32_t n_pixels;   // pixels LBP traités
} LBPState;              // 44 octets

// -----------------------------------------------------------------------------
/**
 * AutoCorrState — Autocorrélation horizontale, décalages 0..8 pixels
 *
 * Pour chaque scanline, accumule les produits lum[x]×lum[x+k] pour k=0..8.
 * Permet de détecter :
 *   - Périodicité directe  (k = période → pic de corrélation)
 *   - Quasi-périodicité    (plusieurs pics successifs)
 *   - Aplatissement spectral (bruit blanc → pas de pic)
 *
 * Implémentation : ring buffer circulaire de 8 valeurs lum par ligne.
 *   O(8) multiplications par pixel = ~32 cycles @ 133 MHz → négligeable.
 *
 * ATTENTION OVERFLOW : lum[x]×lum[x+k] max = 255² = 65025
 *   Pour 76 800 pixels : Σ max = 65025 × 76800 = 4.99 × 10⁹ > uint32_t max.
 *   → acorr[] est uint64_t obligatoirement.
 *
 * RAM Pico : 9×8 + 9×4 + 8 + 4 = 72 + 36 + 12 = 120 octets.
 */
typedef struct {
    uint64_t acorr[9];   // Σ lum[x]×lum[x+k], k=0..8  (k=0 = Σlum²)
    uint32_t n_pairs[9]; // nombre de paires valides par décalage
    uint8_t  lag_buf[8]; // ring buffer des 8 dernières lum (par scanline)
    uint8_t  buf_head;   // tête d'écriture (0..7)
    uint8_t  buf_count;  // valeurs valides dans le ring buffer (0..8)
    uint8_t  _pad[2];
} AutoCorrState;          // 120 octets

// -----------------------------------------------------------------------------
/**
 * SpatialState — Tuilage spatial 4×4 pour détecter la non-stationnarité
 *
 * L'image est divisée en 16 tuiles (4 colonnes × 4 lignes).
 * Pour chaque tuile : somme de luminance + compteur de pixels.
 *
 * Produit : spatial_var_q8 = variance inter-tuiles de la lum moyenne.
 *   Haute valeur → texture non-stationnaire (veines, salissures, gradients)
 *   Basse valeur → texture homogène (béton uniforme, tissu régulier)
 *
 * RAM Pico : 2 × 4×4 × 4 = 128 octets.
 */
typedef struct {
    uint32_t sum_lum[4][4]; // Σ luminance par tuile (max 4800×255 = 1.2 M → uint32_t ✓)
    uint32_t cnt[4][4];     // nombre de pixels par tuile
} SpatialState;              // 128 octets

// -----------------------------------------------------------------------------
/**
 * TextureSignature — Empreinte mathématique finale de la texture
 *
 * Produit de la Phase 2 (Comprendre) à partir des trois analyseurs ci-dessus.
 * Entrée unique de la Phase 3 (Dessiner / générateur HLSL).
 * Taille ~32 octets : tient dans deux lignes de cache M0+ (16 B chacune).
 */
typedef struct {
    // Welford
    uint8_t  mean_lum;          // luminance moyenne (0–255)
    uint8_t  mean_r;            // couleur dominante canal R
    uint8_t  mean_g;            // couleur dominante canal G
    uint8_t  mean_b;            // couleur dominante canal B
    // Haralick (depuis GLCM, Phase 2)
    uint8_t  contrast_q;        // contraste normalisé (0=plat, 255=très rugueux)
    uint8_t  homogeneity_q;     // homogénéité normalisée (255=lisse, 0=chaos)
    uint8_t  entropy_q;         // entropie normalisée (0=uniforme, 255=maximum)
    // Gabor MICRO  (λ=4px)  — grain, fibre, artefacts JPEG
    uint8_t  dominant_angle;     // 0=0°, 1=45°, 2=90°, 3=135°
    uint8_t  anisotropy_q;       // force orientation micro  (0=iso, 255=fort)
    // Gabor LOW    (λ=16px) — strates moyennes, planches de bois
    uint8_t  dominant_angle_low; // angle dominant couche low
    uint8_t  anisotropy_q_low;   // force orientation low    (0=iso, 255=fort)
    // Gabor MACRO  (λ=64px) — strates géologiques, bancs de roche
    uint8_t  dominant_angle_mac; // angle dominant couche macro
    uint8_t  anisotropy_q_mac;   // force orientation macro  (0=iso, 255=fort)
    // Exposant fractal (Phase 2, approximé depuis entropie GLCM)
    uint8_t  fractal_H_q8;       // H × 256 pour FBM (ex: H=0.7 → 179)
    // Drapeaux décisionnels sémantiques (Phase 2) — sortie de l'arbre
    uint8_t  has_stripes;        // micro OU low OU macro orienté
    uint8_t  has_grain;          // contraste GLCM Q10 > seuil
    uint8_t  has_chaos;          // entropie élevée OU variance forte
    uint8_t  macro_stripes;      // 1 = strates géologiques larges (macro orienté)
    uint8_t  micro_chaos;        // 1 = désordre à petite échelle (micro non orienté)
    // Variance
    uint32_t variance_lum;       // variance luminance (unités²)
    uint32_t haralick_contrast;  // contraste Haralick en Q10 (×1024)
    uint32_t haralick_homog_q16; // homogénéité Haralick ×65536 (Q16)
    // Histogramme 4 bins : couleurs réelles extraites de l'image
    // bin 0 = pixels sombres (lum 0-63), bin 3 = pixels clairs (lum 192-255)
    uint8_t  palette[4][3];     // palette[bin][R/G/B]  — 12 octets
    // Drapeaux texture avancés
    uint8_t  has_grid;          // 1 = structure biaxiale (tissus, rattan, grille)
    uint8_t  is_fine_grain;     // 1 = grain fin régulier (bois, fibre lisse)
    // Angle secondaire MICRO (pour détection grille)
    uint8_t  second_angle_mic;  // angle secondaire à l'échelle MICRO
    uint8_t  second_aniso_mic;  // force de cet angle secondaire
    uint8_t  is_isotropic;      // 1 = surface isotrope (béton, plâtre, enduit uniforme)

    // ── LBP / micro-structure locale (v3) ─────────────────────────────────────
    uint8_t  lbp_uniformity_q8; // fraction de patrons uniformes (255=structure fine, 0=chaos)
    uint8_t  lbp_entropy_q8;    // entropie histogramme LBP (0=monotone, 255=très varié)
    uint8_t  microtexton_q8;    // force du patron LBP dominant (ex: bords/coins/taches)

    // ── Autocorrélation horizontale (v3) ──────────────────────────────────────
    uint8_t  period_x_q8;       // période horizontale détectée (0=aucune, 32=1px, 255=8px)
    uint8_t  spectral_flatness_q8; // 0=périodique/pic, 255=bruit blanc plat

    // ── Tuilage spatial 4×4 (v3) ──────────────────────────────────────────────
    uint8_t  spatial_var_q8;    // variance inter-tuiles de lum moyenne (0=uniforme, 255=très variable)
    uint8_t  nonstationarity_q8;// alias pour is_nonstationary quantifié

    // ── Chromatique / phases (v3) ─────────────────────────────────────────────
    uint8_t  chroma_strength_q8;// max(|R-G|,|R-B|,|G-B|) → couleur porteuse de structure
    uint8_t  bimodal_q8;        // (bin_dark+bin_bright)/total → matériau 2 phases
    uint8_t  n_phases;          // nb de phases matérielles détectées (1..4)

    // ── Drapeaux décisionnels v3 ───────────────────────────────────────────────
    uint8_t  has_periodic;      // 1 = périodicité détectée (grille, tissu régulier)
    uint8_t  is_nonstationary;  // 1 = variation spatiale lente (veines, salissures)
    uint8_t  is_chromatic;      // 1 = texture portée par la couleur (pas la luminance seule)

    uint8_t  _pad2[3];          // padding → alignement 4 octets
} TextureSignature;              // sizeof ≈ 80 octets


// =============================================================================
//  INITIALISATIONS
// =============================================================================

static void welford_init(WelfordState* s) {
    s->n          = 0u;
    s->sum_lum    = 0u;
    s->sum_sq_lum = 0u;
    s->sum_r      = 0u;
    s->sum_g      = 0u;
    s->sum_b      = 0u;
    memset(s->hist_rsum, 0, sizeof(s->hist_rsum));
    memset(s->hist_gsum, 0, sizeof(s->hist_gsum));
    memset(s->hist_bsum, 0, sizeof(s->hist_bsum));
    memset(s->hist_cnt,  0, sizeof(s->hist_cnt));
}

static void glcm_init(GLCMState* s) {
    memset(s->matrix, 0, sizeof(s->matrix));
    s->total_pairs = 0u;
    s->prev_q4     = 0u;
    s->_pad[0] = s->_pad[1] = s->_pad[2] = 0u;
}

static void gabor_init(GaborState* s) {
    memset(s->rows,   0, sizeof(s->rows));
    memset(s->energy, 0, sizeof(s->energy));
    s->n_windows = 0u;
    s->head      = 0u;
    s->filled    = 0u;
    s->_pad[0] = s->_pad[1] = 0u;
}

static void signature_init(TextureSignature* sig) {
    memset(sig, 0, sizeof(*sig));
}

static void lbp_init(LBPState* s) {
    memset(s->hist, 0, sizeof(s->hist));
    s->n_pixels = 0u;
}

static void acorr_init(AutoCorrState* s) {
    memset(s->acorr,   0, sizeof(s->acorr));
    memset(s->n_pairs, 0, sizeof(s->n_pairs));
    memset(s->lag_buf, 0, sizeof(s->lag_buf));
    s->buf_head  = 0u;
    s->buf_count = 0u;
    s->_pad[0] = s->_pad[1] = 0u;
}

static void spatial_init(SpatialState* s) {
    memset(s->sum_lum, 0, sizeof(s->sum_lum));
    memset(s->cnt,     0, sizeof(s->cnt));
}


// =============================================================================
//  BOUCHON — FLUX DE PIXELS SIMULÉ
//  Remplacez par uart_read_blocking() / usb_read / SPI / SD selon votre hardware.
//
//  Pattern simulé : rayures horizontales (onde triangulaire, période 20 px)
//                 + bruit LCG 4 bits déterministe
//                 = approximation d'une texture "bois / tranche de roche"
//
//  Pourquoi une onde TRIANGULAIRE et non sinusoïdale ?
//   Le M0+ n'a pas de sin() matériel. Calculer sin en logiciel (libm) coûte
//   ~100+ cycles et 20 Ko de flash. L'onde triangulaire = 3 opérations entières.
//   La morphologie des rayures est conservée pour les tests de GLCM et Welford.
// =============================================================================

static uint32_t _stream_pos  = 0u;          // position dans le flux (pixel courant)
static uint32_t _lcg_state   = 0xDEADBEEFu; // état du générateur pseudo-aléatoire

#ifndef PICO_BUILD
// Pointeur sur les données d'image réelle (nullptr = bouchon LCG actif)
// Format : RGB entrelacé, 1 octet par canal, ligne par ligne
static const uint8_t* _img_data    = nullptr;
static uint32_t       _img_n_pixels = 0u;    // width × height (pour clamp)
#endif

/**
 * get_next_pixel — Retourne le prochain pixel RGB du flux simulé.
 *
 * En PRODUCTION, remplacez le corps par :
 *   uint8_t buf[3];
 *   uart_read_blocking(uart0, buf, 3);   // RP2040 Pico SDK
 *   *r_out = buf[0]; *g_out = buf[1]; *b_out = buf[2];
 *
 * Format attendu du flux réel : pixels consécutifs, ligne par ligne,
 * 3 octets par pixel (R, G, B), sans en-tête.
 */
static void get_next_pixel(uint8_t* r_out, uint8_t* g_out, uint8_t* b_out) {

#ifndef PICO_BUILD
    // ── Mode image réelle (PC) ─────────────────────────────────────────────────
    // Activé si _img_data != nullptr (chargé par main via stb_image)
    if (_img_data != nullptr) {
        const uint32_t idx = _stream_pos * 3u;
        if (_stream_pos < _img_n_pixels) {
            *r_out = _img_data[idx + 0u];
            *g_out = _img_data[idx + 1u];
            *b_out = _img_data[idx + 2u];
        } else {
            *r_out = *g_out = *b_out = 0u;
        }
        _stream_pos++;
        return;
    }
#endif

    // Avance le LCG dans les deux modes (paramètres Knuth, déterministe)
    _lcg_state = _lcg_state * 1664525u + 1013904223u;

#ifdef TEST_MODE_STONE
    // -------------------------------------------------------------------------
    // MODE PIERRE / SABLE
    // -------------------------------------------------------------------------
    // Objectif statistique :
    //   • Contraste GLCM élevé  → transitions brusques entre niveaux adjacents
    //   • Gabor isotrope        → aucune direction dominante → has_stripes = 0
    //   • Variance élevée       → has_chaos = 1
    //
    // Implémentation : LCG pur par pixel (pixels indépendants entre eux).
    //
    // Pourquoi le LCG donne-t-il Gabor isotrope ?
    //   Pour un bruit blanc, l'espérance d'énergie Gabor vaut :
    //     E[E_a] = σ² × ‖kernel_a‖²_F
    //   Or ‖kernel_a‖²_F est identique pour les 4 angles (rotations/symétries
    //   du même kernel). Donc toutes les énergies convergent vers la même valeur.
    //   → dom_pct ≈ 25% → anisotropy_q ≈ 0 → has_stripes = 0 ✓
    //
    // Plage lumineuse [50..209] → variance ≈ 160²/12 ≈ 2133 >> seuil 1000 ✓
    // Contraste GLCM E[d²] ≈ 42.5 >> seuil 30                               ✓
    // -------------------------------------------------------------------------
    _stream_pos++;  // position non utilisée pour pierre (texture sans structure)

    // Luminance de base : 8 bits hauts du LCG → [50..209]
    const uint8_t base = (uint8_t)(50u + ((_lcg_state >> 24u) * 160u >> 8u));

    // Variation fine : 4 bits intermédiaires [bits 19..16] → [0..15]
    // Bits différents des 8 bits hauts → décorrélés dans la séquence LCG
    const uint8_t detail = (uint8_t)((_lcg_state >> 16u) & 0x0Fu);

    const uint32_t lv = (uint32_t)base + (uint32_t)detail;
    const uint8_t  lum = (lv > 224u) ? (uint8_t)224u : (uint8_t)lv; // clamp max

    // Palette pierre : gris chaud (tons sable/granit). R légèrement > G > B.
    // Les offsets sont constants pour préserver la randomness de lum.
    const uint32_t rv = (uint32_t)lum + 14u;
    const uint32_t gv = (uint32_t)lum +  6u;
    const uint32_t bv = (uint32_t)lum;          // B = base, pas d'underflow

    *r_out = (rv > 255u) ? (uint8_t)255u : (uint8_t)rv;
    *g_out = (gv > 255u) ? (uint8_t)255u : (uint8_t)gv;
    *b_out = (uint8_t)bv;

#else
    // -------------------------------------------------------------------------
    // MODE BOIS / STRATES (défaut)
    // -------------------------------------------------------------------------
    const uint32_t y = _stream_pos / IMG_WIDTH;  // bouchon : IMG_WIDTH est la largeur simulée
    _stream_pos++;

    // Onde triangulaire (remplace sin, zéro float, 3 opérations entières)
    const uint32_t PERIOD   = 20u;
    const uint32_t HALF     = PERIOD / 2u;
    const uint32_t phase    = y % PERIOD;
    const uint8_t  stripe   = (phase < HALF)
        ? (uint8_t)(phase * 255u / HALF)
        : (uint8_t)((PERIOD - phase) * 255u / HALF);

    // 4 bits hauts du LCG → bruit [0..15]
    const uint8_t noise = (uint8_t)(_lcg_state >> 28u);

    // Palette "bois beige" : tons chauds + modulation rayures
    const uint32_t rv = 75u + (uint32_t)stripe * 180u / 255u + (uint32_t)(noise >> 1u);
    const uint32_t gv = 60u + (uint32_t)stripe * 140u / 255u + (uint32_t)(noise >> 2u);
    const uint32_t bv = 40u + (uint32_t)stripe *  90u / 255u + (uint32_t)(noise >> 3u);

    *r_out = (rv > 255u) ? (uint8_t)255u : (uint8_t)rv;
    *g_out = (gv > 255u) ? (uint8_t)255u : (uint8_t)gv;
    *b_out = (bv > 255u) ? (uint8_t)255u : (uint8_t)bv;

#endif // TEST_MODE_STONE
}


// =============================================================================
//  PHASE 1 — FONCTIONS D'ACCUMULATION
// =============================================================================

// -----------------------------------------------------------------------------
/**
 * lum_from_rgb — Luminance ITU-R BT.601 en arithmétique entière pure
 *
 * Y ≈ (77·R + 150·G + 29·B) >> 8
 * Approximation entière de : Y = 0.299·R + 0.587·G + 0.114·B
 *   77/256 ≈ 0.301   150/256 ≈ 0.586   29/256 ≈ 0.113
 * Erreur max par rapport à la formule flottante : < 1 LSB sur [0–255]. ✓
 *
 * Sur M0+ : 3 MUL + 2 ADD + 1 SHR = ~6 cycles. Aucun accès mémoire.
 */
static inline uint8_t lum_from_rgb(uint8_t r, uint8_t g, uint8_t b) {
    return (uint8_t)(
        ((uint32_t)r * 77u + (uint32_t)g * 150u + (uint32_t)b * 29u) >> 8u
    );
}

// -----------------------------------------------------------------------------
/**
 * welford_update — Mise à jour Welford (O(1), aucun float)
 *
 * Coût sur M0+ (pas de FPU) :
 *   lum_from_rgb : ~6 cycles
 *   delta = lum - mean : 1 SUB
 *   mean += delta / n  : 1 SDIV (soft-div ~20 cycles sur RP2040 ROM)
 *   delta2 = lum - mean : 1 SUB
 *   M2 += delta * delta2 : __aeabi_lmul (~10 cycles pour 32×32→64 bits)
 *   Total estimé : ~40 cycles @ 133 MHz = ~0.3 µs/pixel
 *
 * OPTIMISATION PICO (Phase 1b) :
 *   Remplacer `delta / n` par hw_divider_s32_quotient_wait(delta, n)
 *   depuis <hardware/divider.h> → utilise le divider matériel du SIO.
 */
static inline void welford_update(WelfordState* s,
                                   uint8_t lum, uint8_t r, uint8_t g, uint8_t b)
{
    s->n++;
    s->sum_lum    += (uint32_t)lum;
    // uint64_t += uint32_t×uint32_t : sur M0+ = __aeabi_ladd(uint64_t, uint64_t)
    // ~4 cycles. Acceptable pour une puce alimentée par des patates.
    s->sum_sq_lum += (uint64_t)lum * (uint64_t)lum;

    s->sum_r += (uint32_t)r;
    s->sum_g += (uint32_t)g;
    s->sum_b += (uint32_t)b;

    // Accumulation histogramme 4 bins (lum >> 6 = division entière par 64)
    const uint8_t bin = lum >> 6u;   // bin 0..3
    s->hist_rsum[bin] += r;
    s->hist_gsum[bin] += g;
    s->hist_bsum[bin] += b;
    s->hist_cnt[bin]++;
}

// -----------------------------------------------------------------------------
/**
 * glcm_accumulate — Accumule une paire (prev_lum, lum) dans la matrice GLCM
 *
 * Direction : horizontale stricte (paires consécutives sur la même ligne).
 * Passer is_line_start=1 pour le premier pixel de chaque nouvelle ligne,
 * afin d'éviter les paires fantômes inter-lignes (fin ligne N, début ligne N+1).
 *
 * Symétrie : on incrémente matrix[i][j] ET matrix[j][i].
 *   → Si i == j (pixels identiques), matrix[i][i] reçoit 2 incréments.
 *      C'est le comportement standard d'une GLCM symétrique. ✓
 *
 * Coût : 2 accès mémoire uint32_t, 2 incréments, 1 comparaison = ~5 cycles.
 */
static inline void glcm_accumulate(GLCMState* s, uint8_t lum, uint8_t is_line_start) {
    const uint8_t q = lum >> GLCM_SHIFT;  // quantification 0–255 → 0–15

    if (!is_line_start) {
        s->matrix[s->prev_q4][q]++;         // paire (prev → curr)
        s->matrix[q][s->prev_q4]++;         // symétrie (curr → prev)
        s->total_pairs++;
    }
    s->prev_q4 = q;
}

// =============================================================================
//  LBP (Local Binary Patterns) — LUT + calcul par fenêtre 3×3
//
//  LBP8,1 uniforme : pour chaque pixel centre, compare ses 8 voisins (rayon 1).
//  Le code 8 bits résultant est mappé vers un bin [0..9] via une LUT précalculée.
//
//  Bins 0..8 = patrons uniformes (≤ 2 transitions 0→1 ou 1→0 dans le ring) :
//    Bin k = patrons avec exactement k bits à 1 formant un arc connexe.
//    Exemple : 00001111 (bin 4), 00000011 (bin 2), 11000001 (bin 3 en circulaire).
//  Bin 9 = non-uniforme (> 2 transitions, ex: 01010101 = 4 transitions).
//
//  Précalculé au démarrage (lbp_lut_build) → 256 B en SRAM (BSS).
//  Coût : ~256 × 16 itérations = ~4000 cycles au boot. Négligeable.
// =============================================================================

static uint8_t LBP_LUT[256]; // 256 B — initialisé une fois par lbp_lut_build()

static void lbp_lut_build(void) {
    for (uint32_t v = 0u; v < 256u; v++) {
        // Comptage des transitions circulaires
        uint32_t trans = 0u;
        for (uint32_t i = 0u; i < 8u; i++) {
            const uint32_t b0 = (v >>  i       ) & 1u;
            const uint32_t b1 = (v >> ((i + 1u) & 7u)) & 1u;
            if (b0 != b1) trans++;
        }
        // Comptage des bits à 1 (popcount)
        uint32_t ones = 0u;
        uint32_t tmp  = v;
        while (tmp) { ones += tmp & 1u; tmp >>= 1u; }
        LBP_LUT[v] = (uint8_t)((trans <= 2u) ? ones : 9u);
    }
}

// -----------------------------------------------------------------------------
// lbp_compute_from_rows — Calcule LBP sur la ligne CENTRALE d'une fenêtre 3×3
//
// Appel depuis gabor_push_row quand le ring buffer est plein :
//   row_top = ring_rows[1], row_ctr = ring_rows[2], row_bot = ring_rows[3]
//
// Convention de bit-assignment (classique, sens horloge depuis voisin droit) :
//   bit0=droite, bit1=bas-droite, bit2=bas, bit3=bas-gauche,
//   bit4=gauche,  bit5=haut-gauche, bit6=haut, bit7=haut-droite
//
// Coût : ~(8 comparaisons + 1 LUT + 2 incréments) × (width-2) colonnes
//       ≈ 11 × 318 = 3498 opérations par ligne ≈ 26 µs @ 133 MHz. ✓
// -----------------------------------------------------------------------------
static void lbp_compute_from_rows(LBPState*       s,
                                   const uint8_t*  row_top,
                                   const uint8_t*  row_ctr,
                                   const uint8_t*  row_bot,
                                   uint16_t        width)
{
    for (uint16_t c = 1u; c < (uint16_t)(width - 1u); c++) {
        const uint8_t ctr = row_ctr[c];
        uint8_t code = 0u;
        if (row_ctr[c + 1u] >= ctr) code |= 0x01u;
        if (row_bot[c + 1u] >= ctr) code |= 0x02u;
        if (row_bot[c     ] >= ctr) code |= 0x04u;
        if (row_bot[c - 1u] >= ctr) code |= 0x08u;
        if (row_ctr[c - 1u] >= ctr) code |= 0x10u;
        if (row_top[c - 1u] >= ctr) code |= 0x20u;
        if (row_top[c     ] >= ctr) code |= 0x40u;
        if (row_top[c + 1u] >= ctr) code |= 0x80u;
        s->hist[LBP_LUT[code]]++;
        s->n_pixels++;
    }
}

// -----------------------------------------------------------------------------
// acorr_update — Accumule l'autocorrélation horizontale pour un pixel
//
// Appelez avec is_line_start=1 pour le premier pixel de chaque nouvelle ligne
// (vide le ring buffer inter-lignes → évite les paires fantômes bord-à-bord).
//
// Coût : 8 multiplications × ~4 cycles = ~32 cycles/pixel sur M0+.
//        Pour 320×240 : 8 × 76800 ≈ 614 400 cycles ≈ 4.6 ms @ 133 MHz. ✓
// -----------------------------------------------------------------------------
static inline void acorr_update(AutoCorrState* s, uint8_t lum, uint8_t is_line_start)
{
    if (is_line_start) {
        s->buf_head  = 0u;
        s->buf_count = 0u;
        return;
    }
    const uint32_t lv = (uint32_t)lum;
    // lag 0 = Σlum²
    s->acorr[0]   += (uint64_t)lv * lv;
    s->n_pairs[0]++;
    // lags 1..8 : produit avec les k dernières valeurs
    for (uint8_t k = 1u; k <= s->buf_count && k <= 8u; k++) {
        const uint8_t prev_idx = (uint8_t)((s->buf_head + 8u - k) & 7u);
        s->acorr[k]   += (uint64_t)lv * (uint64_t)s->lag_buf[prev_idx];
        s->n_pairs[k]++;
    }
    // Stockage de lum dans le ring buffer
    s->lag_buf[s->buf_head] = lum;
    s->buf_head = (uint8_t)((s->buf_head + 1u) & 7u);
    if (s->buf_count < 8u) s->buf_count++;
}

// -----------------------------------------------------------------------------
// spatial_update — Accumule la lum dans la tuile 4×4 correspondante
//
// tc0/tc1/tc2 = seuils de colonne précalculés hors de la boucle interne.
// row_quad     = 0..3, précalculé hors de la boucle interne (une fois par ligne).
// -----------------------------------------------------------------------------
static inline void spatial_update(SpatialState*  s,
                                   uint8_t        lum,
                                   uint32_t       x,
                                   uint32_t       row_quad,
                                   uint32_t       tc0,
                                   uint32_t       tc1,
                                   uint32_t       tc2)
{
    const uint32_t col_quad = (x < tc0) ? 0u : (x < tc1) ? 1u : (x < tc2) ? 2u : 3u;
    if (row_quad < 4u) {
        s->sum_lum[row_quad][col_quad] += lum;
        s->cnt[row_quad][col_quad]++;
    }
}

// =============================================================================
//  NOYAUX DE GABOR — Stockés en Flash (ROM sur Pico)
//
//  Paramètres mathématiques :
//    σ = 1.5 px  — largeur de l'enveloppe gaussienne
//    λ = 4.0 px  — longueur d'onde de la porteuse (période spatiale)
//    γ = 1.0     — aspect ratio (enveloppe isotrope)
//    Format Q7   — valeurs × 128, arrondies et clampées à [-128, 127]
//
//  Formule (partie réelle du filtre de Gabor complexe) :
//    G(r,c) = exp(-(r²+c²)/(2σ²)) × cos(2π·x_prime/λ)
//    où x_prime = projection (r,c) sur la direction θ
//
//  Vérification overflow convolution :
//    réponse max = Σ|K[r][c]| × 255 = 25 × 127 × 255 = 809 625 → int32_t ✓
//    énergie max = 76 800 × 809 625² ≈ 5×10¹⁶            → int64_t ✓
//
//  CONVENTION D'ORIENTATION (CRITIQUE pour interpréter les résultats) :
//    La porteuse oscille DANS la direction θ.
//    Le filtre répond donc aux structures qui varient DANS cette direction.
//    Exemples avec notre image de test (rayures HORIZONTALES, variation en Y) :
//      GABOR_KERN[0] (θ= 0°, porteuse en X) → faible réponse
//      GABOR_KERN[2] (θ=90°, porteuse en Y) → FORTE réponse  ← attendu
//    L'angle dominant sera donc 90° pour les rayures horizontales.
//
//  Noyaux (r=ligne, c=colonne, origine au centre) :
//                          c: -2   -1    0   +1   +2
// =============================================================================

//  CENTRAGE DC des noyaux (correction critique) :
//    Un noyau brut a un biais DC = Σ(kernel) / 25.
//    Pour des régions uniformes (plat d'une strate), la réponse serait DC × luminance,
//    masquant complètement l'anisotropie réelle.
//    Correction : soustraire round(Σ/25) de chaque élément → Σ ≈ 0.
//
//    Σ brut (avant centrage) :
//      a=0 :  45  →  correction = 2 par élément   (Σ après : -5 ≈ 0)
//      a=1 : 177  →  correction = 7 par élément   (Σ après :  2 ≈ 0)
//      a=2 :  75  →  correction = 3 par élément   (Σ après :  0)
//      a=3 : 177  →  correction = 7 par élément   (Σ après :  2 ≈ 0)
//
//  Propriétés vérifiables dans les tableaux ci-dessous :
//    a=2 est la transposée de a=0 (rotation 90°)
//    a=3 est le miroir horizontal de a=1 (symétrie de a=135° vs a=45°)
//                               c: -2  -1    0  +1   +2
static const int8_t GABOR_KERN[GABOR_ANGLES][GABOR_KERN_SIZE][GABOR_KERN_SIZE] = {

    // ── a=0 : θ=0° — centré DC (−2/elem, Σ=-5≈0) ───────────────────────────
    // Porteuse le long de X. Répond aux structures variant en X (rayures verticales).
    {
        { -24,  -2,  51,  -2, -24 },   // r = -2
        { -44,  -2, 100,  -2, -44 },   // r = -1
        { -55,  -2, 125,  -2, -55 },   // r =  0  ← centre
        { -44,  -2, 100,  -2, -44 },   // r = +1
        { -24,  -2,  51,  -2, -24 },   // r = +2
    },

    // ── a=1 : θ=45° — centré DC (−7/elem, Σ=2≈0) ───────────────────────────
    // Porteuse diagonale ↗. Répond aux transitions anti-diagonales (↙↗).
    {
        { -13, -48, -39,  11,  15 },   // r = -2
        { -48, -56,  36,  75,  11 },   // r = -1
        { -39,  36, 120,  36, -39 },   // r =  0  ← centre
        {  11,  75,  36, -56, -48 },   // r = +1
        {  15,  11, -39, -48, -13 },   // r = +2
    },

    // ── a=2 : θ=90° — centré DC (−3/elem, Σ=0) ─────────────────────────────
    // Porteuse le long de Y. Répond aux structures variant en Y (rayures horizontales).
    // = transposée de a=0 ✓
    {
        { -25, -45, -56, -45, -25 },   // r = -2
        {  -3,  -3,  -3,  -3,  -3 },   // r = -1
        {  50,  99, 124,  99,  50 },   // r =  0  ← centre
        {  -3,  -3,  -3,  -3,  -3 },   // r = +1
        { -25, -45, -56, -45, -25 },   // r = +2
    },

    // ── a=3 : θ=135° — centré DC (−7/elem, Σ=2≈0) ──────────────────────────
    // Porteuse diagonale ↖. Répond aux transitions diagonales (↖↘).
    // = miroir horizontal de a=1 ✓
    {
        {  15,  11, -39, -48, -13 },   // r = -2
        {  11,  75,  36, -56, -48 },   // r = -1
        { -39,  36, 120,  36, -39 },   // r =  0  ← centre
        { -48, -56,  36,  75,  11 },   // r = +1
        { -13, -48, -39,  11,  15 },   // r = +2
    },
};

// -----------------------------------------------------------------------------
/**
 * gabor_convolve_window — Convolution 5×5 sur la fenêtre du ring buffer
 *
 * Appelée après chaque push_row quand le buffer est plein (RING_ROWS lignes).
 * Traite toutes les colonnes c ∈ [2, width−3] : bordure 2 px ignorée.
 * Accumule l'ÉNERGIE QUADRATIQUE de chaque orientation dans s->energy[].
 *
 * ── Stratégie cache et GCC ──────────────────────────────────────────────────
 *  ① TOUS les angles calculés en UN SEUL parcours 5×5 → pixel lu une seule fois
 *  ② row_win = &rows[ri[dr]][c-2] → accès SÉQUENTIEL dans SRAM sur dc=0..4
 *  ③ Les 4 noyaux = 100 octets → tiennent entièrement dans le cache XIP (4 Ko)
 *  ④ GCC -O2 + bornes constantes → déroulage automatique du double for 5×5
 *
 * ── Ring buffer après push_row ──────────────────────────────────────────────
 *  head pointe sur la PROCHAINE case libre (déjà incrémenté dans push_row).
 *  rows[(head+0) % 5] = ligne la plus ANCIENNE (haut de la fenêtre)
 *  rows[(head+2) % 5] = ligne CENTRALE (image_row = N-2)
 *  rows[(head+4) % 5] = ligne la plus RÉCENTE (bas de la fenêtre)
 *
 * ── Coût estimé sur RP2040 @ 133 MHz ────────────────────────────────────────
 *  ~100 mul-add/colonne × ~316 col × 236 lignes = ~7.5 M multiply-adds
 *  + 4 × __aeabi_lmul (soft 64-bit mul) par colonne pour energy[]
 *  Total : ~75 M cycles ≈ 560 ms (one-shot, largement acceptable)
 *
 * OPTIMISATION FUTURE (Phase 1c) :
 *  Kernels 0° et 90° sont séparables → réduire à 2×10 ops/px (gain ×1.5 total).
 *  Sur Pico : utiliser Core1 pour lire les pixels pendant que Core0 convolue.
 */
// gabor_convolve_raw — noyau de convolution 5×5 indépendant de la struct.
//
// Prend un tableau de pointeurs de lignes (ring_rows[5]) plutôt qu'un tableau
// 2D dont la stride serait encodée dans le type. Cela permet de réutiliser
// la même fonction pour GaborState (stride=IMG_MAX_WIDTH) et GaborStateLow
// (stride=IMG_MAX_WIDTH_LOW) sans cast UB ni duplication de code.
//
// ring_rows[dr] = pointeur vers la ligne dr du ring buffer (déjà réordonnée
//                 oldest→newest par l'appelant).
static void gabor_convolve_raw(const uint8_t* const ring_rows[GABOR_RING_ROWS],
                                int64_t energy[GABOR_ANGLES],
                                uint16_t width)
{
    for (uint16_t c = 2u; c < (uint16_t)(width - 2u); c++) {

        int32_t sums[GABOR_ANGLES] = { 0, 0, 0, 0 };

        for (uint8_t dr = 0u; dr < GABOR_RING_ROWS; dr++) {
            const uint8_t* const row_win = &ring_rows[dr][c - 2u];
            for (uint8_t dc = 0u; dc < GABOR_KERN_SIZE; dc++) {
                const int32_t pix = (int32_t)row_win[dc];
                sums[0] += (int32_t)GABOR_KERN[0][dr][dc] * pix;
                sums[1] += (int32_t)GABOR_KERN[1][dr][dc] * pix;
                sums[2] += (int32_t)GABOR_KERN[2][dr][dc] * pix;
                sums[3] += (int32_t)GABOR_KERN[3][dr][dc] * pix;
            }
        }

        for (uint8_t a = 0u; a < GABOR_ANGLES; a++) {
            energy[a] += (int64_t)sums[a] * (int64_t)sums[a];
        }
    }
}

// gabor_convolve_window — wrapper pour GaborState (couche haute résolution)
static void gabor_convolve_window(GaborState* s, uint16_t width) {
    const uint8_t* ring_rows[GABOR_RING_ROWS];
    for (uint8_t dr = 0u; dr < GABOR_RING_ROWS; dr++)
        ring_rows[dr] = s->rows[(s->head + dr) % GABOR_RING_ROWS];
    gabor_convolve_raw(ring_rows, s->energy, width);
}

// gabor_convolve_window_low — wrapper pour GaborStateLow (décimée ×4)
static void gabor_convolve_window_low(GaborStateLow* s, uint16_t width) {
    const uint8_t* ring_rows[GABOR_RING_ROWS];
    for (uint8_t dr = 0u; dr < GABOR_RING_ROWS; dr++)
        ring_rows[dr] = s->rows[(s->head + dr) % GABOR_RING_ROWS];
    gabor_convolve_raw(ring_rows, s->energy, width);
}

// gabor_convolve_window_mac — wrapper pour GaborStateMacro (décimée ×16)
static void gabor_convolve_window_mac(GaborStateMacro* s, uint16_t width) {
    const uint8_t* ring_rows[GABOR_RING_ROWS];
    for (uint8_t dr = 0u; dr < GABOR_RING_ROWS; dr++)
        ring_rows[dr] = s->rows[(s->head + dr) % GABOR_RING_ROWS];
    gabor_convolve_raw(ring_rows, s->energy, width);
}

// -----------------------------------------------------------------------------
/**
 * gabor_push_row — Insère une scanline dans le ring buffer et lance la convolution
 *
 * Flux de données :
 *   1. memcpy(320 B) → ~80 cycles (LDM/STM sur M0+)
 *   2. head avancé, filled incrémenté
 *   3. gabor_convolve_window() si buffer plein (chaque ligne après les 5 premières)
 *
 * La ligne analysée est TOUJOURS la ligne centrale (ring_pos[2]) — c'est-à-dire
 * la ligne IMAGE numéro (N-2) quand on pousse la ligne N.
 * Conséquence : les 2 premières et 2 dernières lignes ne sont jamais centre de noyau.
 * Perte = 4/240 = 1.7 % des lignes → négligeable pour la classification de texture.
 */
// gabor_push_row — version v3 : calcule aussi LBP depuis les mêmes lignes bufférisées.
// lbp_s peut être NULL (layers LOW/MACRO n'ont pas besoin de LBP).
// Quand le ring buffer est plein, ring_rows[1/2/3] = top/centre/bottom pour LBP.
static inline void gabor_push_row(GaborState* s, LBPState* lbp_s, const uint8_t* row, uint16_t width)
{
    memcpy(s->rows[s->head], row, (size_t)width);
    s->head = (uint8_t)((s->head + 1u) % GABOR_RING_ROWS);
    if (s->filled < (uint8_t)GABOR_RING_ROWS) s->filled++;
    s->n_windows++;
    if (s->filled == (uint8_t)GABOR_RING_ROWS) {
        gabor_convolve_window(s, width);
        // LBP : réutilise les 3 lignes centrales du ring buffer (top/ctr/bot)
        if (lbp_s) {
            const uint8_t* row_top = s->rows[(s->head + 1u) % GABOR_RING_ROWS];
            const uint8_t* row_ctr = s->rows[(s->head + 2u) % GABOR_RING_ROWS];
            const uint8_t* row_bot = s->rows[(s->head + 3u) % GABOR_RING_ROWS];
            lbp_compute_from_rows(lbp_s, row_top, row_ctr, row_bot, width);
        }
    }
}

static inline void gabor_push_row_low(GaborStateLow* s, const uint8_t* row, uint16_t width)
{
    memcpy(s->rows[s->head], row, (size_t)width);
    s->head = (uint8_t)((s->head + 1u) % GABOR_RING_ROWS);
    if (s->filled < (uint8_t)GABOR_RING_ROWS) s->filled++;
    s->n_windows++;
    if (s->filled == (uint8_t)GABOR_RING_ROWS)
        gabor_convolve_window_low(s, width);
}

static inline void gabor_push_row_mac(GaborStateMacro* s, const uint8_t* row, uint16_t width)
{
    memcpy(s->rows[s->head], row, (size_t)width);
    s->head = (uint8_t)((s->head + 1u) % GABOR_RING_ROWS);
    if (s->filled < (uint8_t)GABOR_RING_ROWS) s->filled++;
    s->n_windows++;
    if (s->filled == (uint8_t)GABOR_RING_ROWS)
        gabor_convolve_window_mac(s, width);
}


// =============================================================================
//  PHASE 1 — BOUCLE PRINCIPALE : UNE SEULE PASSE O(n)
// =============================================================================

/**
 * phase1_decompose — Lit le flux complet en UNE passe et accumule tous les analyseurs.
 *
 * Garanties :
 *   ✓ Zéro copie de l'image entière en RAM
 *   ✓ Zéro float
 *   ✓ Zéro malloc
 *   ✓ RAM supplémentaire sur le stack : 320 octets (gabor_row_buf)
 *   ✓ Complexité temporelle : O(IMG_WIDTH × IMG_HEIGHT) = O(n)
 */
static void phase1_decompose(WelfordState*     wf,
                              GLCMState*        gc,
                              GaborState*       gb,
                              GaborStateLow*    gb_low,
                              GaborStateMacro*  gb_mac,
                              LBPState*         lbp,
                              AutoCorrState*    ac,
                              SpatialState*     sp,
                              uint32_t          img_w,
                              uint32_t          img_h)
{
    uint8_t gabor_row_buf[IMG_MAX_WIDTH];            // MICRO  : toutes colonnes
    uint8_t gabor_row_buf_low[IMG_MAX_WIDTH_LOW];    // LOW    : 1/4 colonnes
    uint8_t gabor_row_buf_mac[IMG_MAX_WIDTH_MACRO];  // MACRO  : 1/16 colonnes (Pico: 20B)

    const uint32_t low_w = img_w / GABOR_DECIM;
    const uint32_t mac_w = img_w / GABOR_DECIM_MACRO;

    // Seuils de colonnes pour le tuilage spatial 4×4 (précalculés hors boucle)
    const uint32_t tc0 = img_w / 4u;
    const uint32_t tc1 = img_w / 2u;
    const uint32_t tc2 = img_w * 3u / 4u;
    const uint32_t tr0 = img_h / 4u;
    const uint32_t tr1 = img_h / 2u;
    const uint32_t tr2 = img_h * 3u / 4u;

    for (uint32_t y = 0u; y < img_h; y++) {

        uint32_t low_x = 0u;   // index dans gabor_row_buf_low (reset chaque ligne)
        uint32_t mac_x = 0u;   // index dans gabor_row_buf_mac (reset chaque ligne)

        // Quadrant de ligne (0..3) — précalculé une fois par ligne
        const uint32_t row_quad = (y < tr0) ? 0u : (y < tr1) ? 1u : (y < tr2) ? 2u : 3u;

        for (uint32_t x = 0u; x < img_w; x++) {

            // -----------------------------------------------------------------
            // 1. Lecture du prochain pixel (bouchon ou UART/USB en prod)
            // -----------------------------------------------------------------
            uint8_t r, g, b;
            get_next_pixel(&r, &g, &b);

            // -----------------------------------------------------------------
            // 2. Luminance — calculée UNE SEULE FOIS, partagée entre analyseurs
            // -----------------------------------------------------------------
            const uint8_t lum = lum_from_rgb(r, g, b);

            // -----------------------------------------------------------------
            // 3. Welford : mean + variance streaming
            // -----------------------------------------------------------------
            welford_update(wf, lum, r, g, b);

            // -----------------------------------------------------------------
            // 4. GLCM : paire horizontale
            // -----------------------------------------------------------------
            glcm_accumulate(gc, lum, (x == 0u) ? 1u : 0u);

            // -----------------------------------------------------------------
            // 5a. Gabor haute fréquence : collecte pixel courant (toutes colonnes)
            // -----------------------------------------------------------------
            gabor_row_buf[x] = lum;

            // -----------------------------------------------------------------
            // 5b. Gabor basse fréquence : décimation ×GABOR_DECIM en X
            // -----------------------------------------------------------------
            if ((x % GABOR_DECIM == 0u) && (low_x < IMG_MAX_WIDTH_LOW)) {
                gabor_row_buf_low[low_x] = lum;
                low_x++;
            }
            // 5c. Gabor MACRO : décimation ×16 en X
            if ((x % GABOR_DECIM_MACRO == 0u) && (mac_x < IMG_MAX_WIDTH_MACRO)) {
                gabor_row_buf_mac[mac_x] = lum;
                mac_x++;
            }

            // -----------------------------------------------------------------
            // 6. Autocorrélation horizontale (lags 1..8, O(8)/pixel)
            // -----------------------------------------------------------------
            acorr_update(ac, lum, (x == 0u) ? 1u : 0u);

            // -----------------------------------------------------------------
            // 7. Tuilage spatial 4×4
            // -----------------------------------------------------------------
            spatial_update(sp, lum, x, row_quad, tc0, tc1, tc2);
        }

        // ---------------------------------------------------------------------
        // 8a. Fin de ligne haute résolution → push ring buffer MICRO + LBP
        // ---------------------------------------------------------------------
        gabor_push_row(gb, lbp, gabor_row_buf, (uint16_t)img_w);

        // ---------------------------------------------------------------------
        // 8b. Fin de ligne basse fréquence → push ring buffer décimé
        // ---------------------------------------------------------------------
        if ((y % GABOR_DECIM) == 0u)
            gabor_push_row_low(gb_low, gabor_row_buf_low, (uint16_t)low_w);

        if ((y % GABOR_DECIM_MACRO) == 0u)
            gabor_push_row_mac(gb_mac, gabor_row_buf_mac, (uint16_t)mac_w);
    }
}


// =============================================================================
//  PHASE 2 — COMPRENDRE
//  Calcul des descripteurs Haralick, gabor finalization, arbre décisionnel.
//  Zéro float. Zéro stdlib. Approx log2 par CLZ (1 cycle M0+).
// =============================================================================

// -----------------------------------------------------------------------------
// Seuils décisionnels — ajustez en calibration sur votre dataset
// -----------------------------------------------------------------------------
// haralick_contrast est en Q10 (×1024) pour éviter la troncature sur images réelles.
// LCG bouchon pierre : contrast_q10≈33792  → bien au-dessus du seuil
// image.png roche    : contrast_q10≈692    → doit déclencher has_grain
// Images lisses JPEG : contrast_q10≈240    → en-dessous
static constexpr uint32_t THRESH_CONTRAST_GRAIN    = 500u;  // en Q10 : image.png=692 ✓, JPEG lisse=240 ✗
static constexpr uint32_t THRESH_VARIANCE_CHAOS     = 1000u;// variance_lum
static constexpr uint32_t THRESH_ENTROPY_COMPLEX    = 160u; // entropy_q (0–255)
static constexpr uint32_t THRESH_GABOR_DOMINANT_PCT = 35u;  // % énergie angle dom.
static constexpr uint32_t THRESH_ANISO_STRIPES      = 28u;  // anisotropy_q (0–255)
static constexpr uint32_t THRESH_FINE_GRAIN_ANISO  = 150u; // très forte orientation → bois/fibre
static constexpr uint32_t THRESH_FINE_ENTROPY       = 50u;  // très basse entropie → structure régulière
static constexpr uint32_t THRESH_BIAXIAL_SECONDARY  = 28u;  // force angle secondaire → grille

// Seuils v3 — LBP, autocorrélation, spatiale, chromatique, bimodalité
static constexpr uint32_t THRESH_LBP_UNIF           = 160u; // lbp_uniformity_q8 > 160 → structure fine (bois, fibre)
static constexpr uint32_t THRESH_PERIOD_DETECTED    =  24u; // period_x_q8 > 24 → périodicité détectée (k=1 → 32 > 24 ✓)
static constexpr uint32_t THRESH_SPECTRAL_FLAT      = 200u; // spectral_flatness < 200 → pas du bruit blanc
static constexpr uint32_t THRESH_SPATIAL_VAR        =  60u; // spatial_var_q8 > 60 → non-stationnaire (~8 lum d'écart)
static constexpr uint32_t THRESH_CHROMA             =  40u; // chroma_strength_q8 > 40 → couleur porteuse de structure
static constexpr uint32_t THRESH_BIMODAL            = 130u; // bimodal_q8 > 130 → matériau 2 phases (granite, liège)

// -----------------------------------------------------------------------------
// ilog2_u32 — floor(log2(x)), utilise l'instruction CLZ (1 cycle sur ARMv6-M)
// Précondition : x > 0
// -----------------------------------------------------------------------------
static inline uint32_t ilog2_u32(uint32_t x) {
    return 31u - (uint32_t)__builtin_clz(x);
}

// -----------------------------------------------------------------------------
// welford_to_sig — extrait mean + variance + couleurs depuis WelfordState
// -----------------------------------------------------------------------------
static void welford_to_sig(const WelfordState* wf, TextureSignature* sig) {
    if (wf->n == 0u) return;

    const uint32_t mean_lum = wf->sum_lum / wf->n;
    const uint32_t sq_mean  = (uint32_t)(wf->sum_sq_lum / (uint64_t)wf->n);
    const uint32_t variance = (sq_mean >= mean_lum * mean_lum)
                            ? (sq_mean - mean_lum * mean_lum)
                            : 0u;

    sig->mean_lum    = (uint8_t)(mean_lum & 0xFFu);
    sig->mean_r      = (uint8_t)(wf->sum_r / wf->n);
    sig->mean_g      = (uint8_t)(wf->sum_g / wf->n);
    sig->mean_b      = (uint8_t)(wf->sum_b / wf->n);
    sig->variance_lum = variance;

    // Palette 4 bins : couleur moyenne de chaque tranche de luminance
    for (uint8_t b = 0u; b < 4u; b++) {
        if (wf->hist_cnt[b] > 0u) {
            sig->palette[b][0] = (uint8_t)(wf->hist_rsum[b] / wf->hist_cnt[b]);
            sig->palette[b][1] = (uint8_t)(wf->hist_gsum[b] / wf->hist_cnt[b]);
            sig->palette[b][2] = (uint8_t)(wf->hist_bsum[b] / wf->hist_cnt[b]);
        } else {
            sig->palette[b][0] = sig->mean_r;
            sig->palette[b][1] = sig->mean_g;
            sig->palette[b][2] = sig->mean_b;
        }
    }
    // Propagation des bins vides : forward (sombre vers clair) puis backward
    for (uint8_t b = 1u; b < 4u; b++) {
        if (wf->hist_cnt[b] == 0u) {
            sig->palette[b][0] = sig->palette[b-1u][0];
            sig->palette[b][1] = sig->palette[b-1u][1];
            sig->palette[b][2] = sig->palette[b-1u][2];
        }
    }
    for (int8_t b = 2; b >= 0; b--) {
        if (wf->hist_cnt[(uint8_t)b] == 0u) {
            sig->palette[(uint8_t)b][0] = sig->palette[(uint8_t)(b+1)][0];
            sig->palette[(uint8_t)b][1] = sig->palette[(uint8_t)(b+1)][1];
            sig->palette[(uint8_t)b][2] = sig->palette[(uint8_t)(b+1)][2];
        }
    }

    // ── Bimodalité : (bin sombre + bin clair) / total ──────────────────────────
    // Haute valeur → deux phases matérielles séparées (granite, liège, speckle)
    {
        const uint32_t flanks = wf->hist_cnt[0] + wf->hist_cnt[3];
        const uint32_t middle = wf->hist_cnt[1] + wf->hist_cnt[2];
        const uint32_t total_h = flanks + middle;
        sig->bimodal_q8 = (total_h > 0u)
            ? (uint8_t)((flanks * 255u) / (total_h + 1u))
            : 0u;
    }

    // ── Nombre de phases : bins significatifs (> 12.5% des pixels chacun) ──────
    {
        const uint32_t thresh_bin = wf->n / 8u;
        uint8_t n_sig = 0u;
        for (uint8_t b = 0u; b < 4u; b++) {
            if (wf->hist_cnt[b] > thresh_bin) n_sig++;
        }
        sig->n_phases = (n_sig < 1u) ? 1u : n_sig;
    }

    // ── Force chromatique : max(|R-G|,|R-B|,|G-B|) → couleur porteuse de struct ─
    {
        const uint32_t rg = (sig->mean_r >= sig->mean_g)
                            ? (sig->mean_r - sig->mean_g)
                            : (sig->mean_g - sig->mean_r);
        const uint32_t rb = (sig->mean_r >= sig->mean_b)
                            ? (sig->mean_r - sig->mean_b)
                            : (sig->mean_b - sig->mean_r);
        const uint32_t gb = (sig->mean_g >= sig->mean_b)
                            ? (sig->mean_g - sig->mean_b)
                            : (sig->mean_b - sig->mean_g);
        const uint32_t chroma = (rg > rb) ? (rg > gb ? rg : gb) : (rb > gb ? rb : gb);
        sig->chroma_strength_q8 = (chroma >= 255u) ? 255u : (uint8_t)chroma;
    }
}

// -----------------------------------------------------------------------------
// Lookup table de dénominateurs Homogénéité : 1/(1+d²) × 65536, d=|i-j|, d=0..15
// 65536/(1+d²) pour d=0..15, précalculé en compile time
// -----------------------------------------------------------------------------
static const uint32_t HOMDENOM[16] = {
    65536u,  // d=0 : 1/(1+0)   ×65536
    32768u,  // d=1 : 1/(1+1)   ×65536
    13107u,  // d=2 : 1/(1+4)   ×65536
     6554u,  // d=3 : 1/(1+9)   ×65536
     3641u,  // d=4 : 1/(1+16)  ×65536
     2228u,  // d=5 : 1/(1+25)  ×65536
     1489u,  // d=6 : 1/(1+36)  ×65536
     1057u,  // d=7 : 1/(1+49)  ×65536
      784u,  // d=8 : 1/(1+64)  ×65536
      601u,  // d=9 : 1/(1+81)  ×65536
      474u,  // d=10: 1/(1+100) ×65536
      384u,  // d=11: 1/(1+121) ×65536
      317u,  // d=12: 1/(1+144) ×65536
      267u,  // d=13: 1/(1+169) ×65536
      228u,  // d=14: 1/(1+196) ×65536
      197u,  // d=15: 1/(1+225) ×65536
};

// -----------------------------------------------------------------------------
// haralick_compute — Haralick features depuis GLCM 16×16
//
//   Contrast    = Σ d² × p(i,j)         (d = |i-j|)
//   Homogeneity = Σ p(i,j) / (1 + d²)   → via HOMDENOM[d] × 65536 sans division
//   Energy/ASM  = Σ p(i,j)²             (uniformité, remplace log pour entropie)
//   Entropy_q   = approximation : 255 - (Energy normalisée)
//                 + correction CLZ sur les bins les plus peuplés
//
// Toutes les divisions finales par total_pairs utilisent le hardware divider SIO
// (disponible sur RP2040) ou la division 32 bits GCC (M0+ optimisé -O2).
// -----------------------------------------------------------------------------
static void haralick_compute(const GLCMState* gc, TextureSignature* sig) {
    if (gc->total_pairs == 0u) return;

    uint64_t contrast_acc   = 0u;  // Σ d² × count(i,j)
    uint64_t homog_acc      = 0u;  // Σ HOMDENOM[d] × count(i,j)
    uint64_t energy_acc     = 0u;  // Σ count(i,j)²  (avant normalisation)

    for (uint32_t i = 0u; i < GLCM_LEVELS; i++) {
        for (uint32_t j = 0u; j < GLCM_LEVELS; j++) {
            const uint32_t cnt = gc->matrix[i][j];
            if (cnt == 0u) continue;

            const uint32_t d  = (i >= j) ? (i - j) : (j - i);
            contrast_acc  += (uint64_t)(d * d) * (uint64_t)cnt;
            homog_acc     += (uint64_t)HOMDENOM[d] * (uint64_t)cnt;
            energy_acc    += (uint64_t)cnt * (uint64_t)cnt;
        }
    }

    // Contraste Q10 : (contrast_acc × 1024) / total_pairs
    // Évite la troncature entière pour les images réelles où contrast_raw < 1.
    // Max théorique : d²_max=225 × total_pairs → Q10_max = 225×1024 = 230400
    // Sur M0+ : contrast_acc est uint64_t, total_pairs uint32_t → division 64/32 bits
    const uint32_t contrast_q10 = (gc->total_pairs > 0u)
        ? (uint32_t)((contrast_acc * 1024u) / (uint64_t)gc->total_pairs)
        : 0u;
    sig->haralick_contrast = contrast_q10;
    // contrast_q normalisé sur [0,255].
    // Saturation à 10000 Q10 (≈9.77 brut) — max empirique pour textures réelles très rugueuses.
    // LCG pierre ≈33792 → 255 (saturé). image.png ≈692 → 17. JPEG lisse ≈240 → 6.
    sig->contrast_q = (contrast_q10 >= 10000u)
                    ? 255u
                    : (uint8_t)((contrast_q10 * 255u) / 10000u);

    // Homogénéité Q16 : homog_raw = Σ HOMDENOM[d]×cnt / total_pairs
    // HOMDENOM[0]=65536 → si texture parfaitement uniforme : homog_raw = 65536
    const uint32_t homog_raw = (uint32_t)(homog_acc / (uint64_t)gc->total_pairs);
    sig->haralick_homog_q16  = homog_raw;
    sig->homogeneity_q = (homog_raw >= 65536u)
                       ? 255u
                       : (uint8_t)(homog_raw >> 8u);  // Q16 → Q8 (>>8)

    // Energy normalisée (ASM = Angular Second Moment) :
    // energy_normalized = Σcnt² / tp²  ∈ [1/tp, 1]
    // 255 = parfaitement uniforme (1 seul bin plein), ~0 = chaos total
    // Calcul : (energy_acc × 255) / tp²  — multiplier avant de diviser !
    // energy_acc × 255 max ≈ 5.86e9 × 255 ≈ 1.5e12  → uint64_t ✓
    const uint32_t tp         = gc->total_pairs;
    const uint32_t energy_q   = (tp > 0u)
        ? (uint32_t)(((uint64_t)energy_acc * 255u) / ((uint64_t)tp * (uint64_t)tp))
        : 0u;
    // Entropy_q = inverse de l'énergie (forte énergie = faible entropie)
    // Approximation robuste sans log : entropy = 255 - min(energy_q, 255)
    const uint8_t  energy_u8  = (energy_q >= 255u) ? 255u : (uint8_t)energy_q;
    sig->entropy_q = 255u - energy_u8;

    // Exposant fractal H approché depuis entropie :
    // H ∈ [0.1, 0.9] → plus l'entropie est haute, plus la texture est complexe/fractale
    // H_q8 = 26 + entropy_q × (204 - 26) / 255
    //       = 26 + entropy_q × 178 / 255
    // (26/256 ≈ 0.10, 204/256 ≈ 0.80)
    sig->fractal_H_q8 = (uint8_t)(26u + ((uint32_t)sig->entropy_q * 178u) / 255u);
}

// -----------------------------------------------------------------------------
// lbp_to_sig — extrait les descripteurs LBP depuis LBPState
// -----------------------------------------------------------------------------
static void lbp_to_sig(const LBPState* lbp, TextureSignature* sig) {
    if (lbp->n_pixels == 0u) {
        sig->lbp_uniformity_q8 = 128u;
        sig->lbp_entropy_q8    = 128u;
        sig->microtexton_q8    =   0u;
        return;
    }
    // Uniformité : fraction de patrons uniformes (bins 0..8) sur total
    uint32_t uniform_count = 0u;
    for (uint8_t i = 0u; i < 9u; i++) uniform_count += lbp->hist[i];
    sig->lbp_uniformity_q8 = (uint8_t)((uniform_count * 255u) / (lbp->n_pixels + 1u));

    // Entropie LBP via proxy énergie (ASM) : haute énergie = faible entropie
    // energy = Σ hist[i]² / n² ∈ [0,255] → entropy_q = 255 - energy_q
    uint64_t energy_acc = 0u;
    for (uint8_t i = 0u; i < 10u; i++)
        energy_acc += (uint64_t)lbp->hist[i] * (uint64_t)lbp->hist[i];
    const uint64_t n2 = (uint64_t)lbp->n_pixels * (uint64_t)lbp->n_pixels;
    const uint32_t energy_q = (n2 > 0u)
        ? (uint32_t)((energy_acc * 255u) / (n2 + 1u))
        : 0u;
    const uint32_t eq_clamped = (energy_q >= 255u) ? 255u : energy_q;
    sig->lbp_entropy_q8 = (uint8_t)(255u - eq_clamped);

    // Microtexton : force du patron dominant (bin avec le plus de pixels)
    uint8_t  dom_bin   = 0u;
    uint32_t dom_count = lbp->hist[0];
    for (uint8_t i = 1u; i < 10u; i++) {
        if (lbp->hist[i] > dom_count) { dom_count = lbp->hist[i]; dom_bin = i; }
    }
    (void)dom_bin; // pourrait servir plus tard à identifier le type de micro-structure
    sig->microtexton_q8 = (uint8_t)((dom_count * 255u) / (lbp->n_pixels + 1u));
}

// -----------------------------------------------------------------------------
// acorr_to_sig — extrait périodicité depuis AutoCorrState + WelfordState
// -----------------------------------------------------------------------------
static void acorr_to_sig(const AutoCorrState* ac, const WelfordState* wf,
                          TextureSignature* sig) {
    if (ac->n_pairs[0] == 0u || wf->n == 0u) {
        sig->period_x_q8         = 0u;
        sig->spectral_flatness_q8 = 255u;
        return;
    }
    const uint32_t mean_lum  = wf->sum_lum / wf->n;
    const uint32_t mean_sq   = mean_lum * mean_lum;
    const uint32_t e_lum_sq  = (uint32_t)(ac->acorr[0] / (uint64_t)ac->n_pairs[0]);
    // variance : toujours >= 1 pour éviter la division par zéro
    const uint32_t var_0 = (e_lum_sq >= mean_sq) ? (e_lum_sq - mean_sq) : 1u;

    uint32_t max_ratio = 0u;
    uint8_t  best_k    = 0u;
    for (uint8_t k = 1u; k <= 8u; k++) {
        if (ac->n_pairs[k] == 0u) continue;
        const uint32_t e_cross = (uint32_t)(ac->acorr[k] / (uint64_t)ac->n_pairs[k]);
        const uint32_t cov_k   = (e_cross >= mean_sq) ? (e_cross - mean_sq) : 0u;
        const uint32_t ratio   = (cov_k * 256u) / (var_0 + 1u);
        if (ratio > max_ratio) { max_ratio = ratio; best_k = k; }
    }
    // Encode la période : best_k en pixels × 32 → [32..255]
    if (max_ratio > 128u && best_k > 0u) {
        const uint32_t pq = (uint32_t)best_k * 32u;
        sig->period_x_q8 = (pq > 255u) ? 255u : (uint8_t)pq;
    } else {
        sig->period_x_q8 = 0u;
    }
    // Flatness spectrale : 0=périodique, 255=bruit blanc
    const uint32_t flat = (max_ratio >= 255u) ? 0u : 255u - max_ratio;
    sig->spectral_flatness_q8 = (flat > 255u) ? 255u : (uint8_t)flat;
}

// -----------------------------------------------------------------------------
// spatial_to_sig — extrait variance spatiale depuis SpatialState
// -----------------------------------------------------------------------------
static void spatial_to_sig(const SpatialState* sp, TextureSignature* sig) {
    // Calcul de la moyenne par tuile, puis variance inter-tuiles
    uint32_t grand_sum = 0u;
    uint32_t tile_mean[4][4];
    for (uint8_t r = 0u; r < 4u; r++) {
        for (uint8_t c = 0u; c < 4u; c++) {
            tile_mean[r][c] = (sp->cnt[r][c] > 0u)
                ? sp->sum_lum[r][c] / sp->cnt[r][c]
                : (uint32_t)sig->mean_lum;
            grand_sum += tile_mean[r][c];
        }
    }
    const uint32_t grand_mean = grand_sum / 16u;

    uint32_t tile_var = 0u;
    for (uint8_t r = 0u; r < 4u; r++) {
        for (uint8_t c = 0u; c < 4u; c++) {
            const uint32_t d = (tile_mean[r][c] >= grand_mean)
                                ? (tile_mean[r][c] - grand_mean)
                                : (grand_mean - tile_mean[r][c]);
            tile_var += d * d;
        }
    }
    tile_var >>= 4u; // ÷ 16 tuiles

    // Échelle : variance de 50 lum entre tuiles → tile_var≈2500 → q8≈250
    // Donc : q8 = min(255, tile_var / 10)
    sig->spatial_var_q8     = (tile_var >= 2550u) ? 255u : (uint8_t)(tile_var / 10u);
    sig->nonstationarity_q8 = sig->spatial_var_q8;
}

// -----------------------------------------------------------------------------
// gabor_energy_to_aniso — helper commun : extrait dominant + anisotropy depuis
//                          un tableau d'énergies Gabor (haute ou basse couche).
// Sorties : *dom_out = angle dominant [0..3], retourne anisotropy_q [0..255]
// -----------------------------------------------------------------------------
static uint8_t gabor_energy_to_aniso(const int64_t energy[GABOR_ANGLES],
                                      uint8_t* dom_out,
                                      uint8_t* second_dom_out,
                                      uint8_t* second_q_out) {
    uint8_t dom   = 0u;
    int64_t max_e = energy[0];
    int64_t total = 0;

    for (uint8_t a = 0u; a < GABOR_ANGLES; a++) {
        total += energy[a];
        if (energy[a] > max_e) { max_e = energy[a]; dom = a; }
    }

    *dom_out = dom;

    // Second angle dominant
    uint8_t second_dom = 0u;
    int64_t second_max = -1;
    for (uint8_t a = 0u; a < GABOR_ANGLES; a++) {
        if (a != dom && energy[a] > second_max) {
            second_max = energy[a];
            second_dom = a;
        }
    }
    if (second_dom_out) *second_dom_out = second_dom;
    if (second_q_out) {
        if (total > 0) {
            const uint32_t sec_pct = (uint32_t)((second_max * 100LL) / total);
            const uint32_t sq = (sec_pct <= 25u) ? 0u : ((sec_pct - 25u) * 255u) / 75u;
            *second_q_out = (sq >= 255u) ? 255u : (uint8_t)sq;
        } else {
            *second_q_out = 0u;
        }
    }

    if (total <= 0) return 0u;

    const uint32_t dom_pct = (uint32_t)((max_e * 100LL) / total);

    if (dom_pct <= 25u) return 0u;

    const uint32_t q = ((dom_pct - 25u) * 255u) / 75u;
    return (q >= 255u) ? 255u : (uint8_t)q;
}

// -----------------------------------------------------------------------------
// gabor_to_sig — remplit les deux couches Gabor dans TextureSignature
// -----------------------------------------------------------------------------
static void gabor_to_sig(const GaborState*      gb,
                          const GaborStateLow*   gb_low,
                          const GaborStateMacro* gb_mac,
                          TextureSignature*       sig) {
    sig->anisotropy_q     = gabor_energy_to_aniso(gb->energy,     &sig->dominant_angle,
                                                   &sig->second_angle_mic, &sig->second_aniso_mic);
    sig->anisotropy_q_low = gabor_energy_to_aniso(gb_low->energy, &sig->dominant_angle_low, NULL, NULL);
    sig->anisotropy_q_mac = gabor_energy_to_aniso(gb_mac->energy, &sig->dominant_angle_mac, NULL, NULL);
}

// -----------------------------------------------------------------------------
// decision_tree — remplit les drapeaux has_* depuis TextureSignature
//
// Logique :
//   has_stripes → anisotropie forte ET angle dominant clair
//   has_grain   → contraste Haralick élevé ET variance modérée (texture granuleuse)
//   has_chaos   → entropie élevée OU variance très haute (sans structure)
//
// On évite les if/else imbriqués : chaque flag est indépendant, ce qui permet
// une texture d'avoir plusieurs flags actifs (ex: bois = stripes + grain).
// -----------------------------------------------------------------------------
static void decision_tree(TextureSignature* sig) {
    // --- has_stripes : orientation sur AU MOINS une des 3 couches
    sig->has_stripes = ((sig->anisotropy_q     > (uint8_t)THRESH_ANISO_STRIPES) ||
                        (sig->anisotropy_q_low > (uint8_t)THRESH_ANISO_STRIPES) ||
                        (sig->anisotropy_q_mac > (uint8_t)THRESH_ANISO_STRIPES)) ? 1u : 0u;

    // --- has_grain : contraste GLCM Q10 élevé
    sig->has_grain = (sig->haralick_contrast > THRESH_CONTRAST_GRAIN) ? 1u : 0u;

    // --- has_chaos : entropie élevée OU variance forte
    sig->has_chaos = ((sig->entropy_q    > (uint8_t)THRESH_ENTROPY_COMPLEX) ||
                      (sig->variance_lum > THRESH_VARIANCE_CHAOS))           ? 1u : 0u;

    // --- macro_stripes : strates géologiques larges (MACRO orienté)
    //     La couche MACRO voit les structures ≥64px → roche sédimentaire, draperies
    sig->macro_stripes = (sig->anisotropy_q_mac > (uint8_t)THRESH_ANISO_STRIPES) ? 1u : 0u;

    // --- micro_chaos : désordre relatif à l'échelle du grain.
    //     Condition : anisotropy MACRO significativement plus forte que MICRO.
    //     Ratio : si macro_stripes ET micro est < 2×seuil → micro est "trop faible"
    //     pour confirmer une structure fine → la surface est chaotique par rapport
    //     aux strates larges. Seuil souple (2× THRESH) pour capturer la roche
    //     où λ=4px ne voit qu'un léger biais sans vraie structure.
    sig->micro_chaos = (sig->macro_stripes &&
                        (sig->anisotropy_q < (uint8_t)(THRESH_ANISO_STRIPES * 2u))) ? 1u : 0u;

    // --- has_grid : structure biaxiale (rattan, tissus, grille)
    //     Condition : angle secondaire MICRO aussi fort ET perpendiculaire au dominant
    {
        const uint8_t adiff = (sig->dominant_angle >= sig->second_angle_mic)
                               ? (sig->dominant_angle - sig->second_angle_mic)
                               : (sig->second_angle_mic - sig->dominant_angle);
        sig->has_grid = (sig->second_aniso_mic > (uint8_t)THRESH_BIAXIAL_SECONDARY
                         && adiff == 2u) ? 1u : 0u;
    }

    // --- is_fine_grain : grain fin régulier (bois, fibre lisse, fourrure)
    //     Condition : très forte orientation MICRO + entropie très basse
    sig->is_fine_grain = (sig->anisotropy_q     > (uint8_t)THRESH_FINE_GRAIN_ANISO &&
                          sig->anisotropy_q_low  > (uint8_t)THRESH_ANISO_STRIPES    &&
                          sig->entropy_q         < (uint8_t)THRESH_FINE_ENTROPY)    ? 1u : 0u;

    // --- is_isotropic : béton, plâtre, enduit — surface sans structure directionnelle
    //     Deux cas :
    //     A) Surface très uniforme (haralick_contrast Q10 < 150, brut < 0.15) :
    //        béton usé, plâtre → Gabor peut montrer anisotropie élevée par artefacts JPEG/bruit,
    //        mais la texture réelle est quasi-plate → forcer le régime isotrope
    //     B) Réellement isotrope à toutes les échelles (aniso < 130 partout)
    //     Toujours exclure : roche (micro_chaos=1), rattan/bois (haralick élevé + aniso élevé)
    //     IMPORTANT : exclure aussi has_stripes — une texture à strates horizontales a un
    //     haralick_contrast HORIZONTAL très bas (strates = peu de transitions horiz),
    //     ce qui déclenchait faussement la condition (A) avant ce correctif.
    sig->is_isotropic = (!sig->micro_chaos && !sig->has_stripes &&
                          (sig->haralick_contrast < 150u ||     // (A) surface très unie
                           (sig->anisotropy_q     < 130u &&     // (B) réellement isotrope
                            sig->anisotropy_q_low  < 130u &&
                            sig->anisotropy_q_mac  < 130u))) ? 1u : 0u;

    // Override granulaire : béton, plâtre, mortier — bimodal fort (pores/matrice) +
    // pas de chroma + pas de chaos. Le Gabor peut montrer de l'anisotropie (artefact
    // baking, compression JPEG) mais ce matériau est fondamentalement isotrope.
    if (!sig->is_isotropic && !sig->micro_chaos && !sig->is_chromatic &&
        sig->bimodal_q8 > 160u) {
        sig->is_isotropic  = 1u;
        sig->has_stripes   = 0u;
        sig->macro_stripes = 0u;
    }

    // ── Drapeaux v3 ─────────────────────────────────────────────────────────────

    // --- has_periodic : périodicité détectée dans l'autocorrélation horizontale
    //     spectral_flatness < THRESH_SPECTRAL_FLAT → pas du bruit blanc
    //     period_x_q8 > THRESH_PERIOD_DETECTED → pic de corrélation significatif
    sig->has_periodic = ((sig->spectral_flatness_q8 < (uint8_t)THRESH_SPECTRAL_FLAT) &&
                         (sig->period_x_q8          > (uint8_t)THRESH_PERIOD_DETECTED)) ? 1u : 0u;

    // --- is_nonstationary : variation spatiale lente entre tuiles
    //     Indique des veines, gradient d'usure, zones de dépôt, salissures
    sig->is_nonstationary = (sig->spatial_var_q8 > (uint8_t)THRESH_SPATIAL_VAR) ? 1u : 0u;

    // --- is_chromatic : texture portée par la couleur (marbre coloré, bois teinté, rouille)
    //     Exclure les textures grises qui ont de l'entropie : elles ne sont pas "chromatiques"
    sig->is_chromatic = (sig->chroma_strength_q8 > (uint8_t)THRESH_CHROMA) ? 1u : 0u;
}

// -----------------------------------------------------------------------------
// phase2_understand — orchestrateur Phase 2 (v3)
// -----------------------------------------------------------------------------
static void phase2_understand(const WelfordState*     wf,
                               const GLCMState*        gc,
                               const GaborState*       gb,
                               const GaborStateLow*    gb_low,
                               const GaborStateMacro*  gb_mac,
                               const LBPState*         lbp,
                               const AutoCorrState*    ac,
                               const SpatialState*     sp,
                               TextureSignature*       sig) {
    welford_to_sig(wf, sig);
    haralick_compute(gc, sig);
    gabor_to_sig(gb, gb_low, gb_mac, sig);
    lbp_to_sig(lbp, sig);
    acorr_to_sig(ac, wf, sig);
    spatial_to_sig(sp, sig);
    decision_tree(sig);
}


// =============================================================================
//  PHASE 3 — DESSINER
//  Générateur HLSL — consomme TextureSignature, émet le shader ligne par ligne.
//  Zéro float C++. Zéro malloc. Chaque ligne = un printf() streamé.
//
//  HLSL_EMIT est TOUJOURS actif (Pico + PC) : c'est la sortie produit finale,
//  pas du debug. Sur Pico il sort par USB/UART vers le moteur Unity.
// =============================================================================

#define HLSL_EMIT printf

// --- Helpers de dérivation des paramètres visuels (entiers purs) -------------

// Fréquence spatiale des strates : [6..24] proportionnelle à l'anisotropie
static inline uint32_t hlsl_stripe_freq(uint8_t aniso_q) {
    return 6u + (uint32_t)aniso_q / 14u;
}

// Octaves FBM : [3..6], texture complexe → plus d'octaves
static inline uint32_t hlsl_fbm_octaves(uint8_t entropy_q) {
    return 3u + (uint32_t)(entropy_q >> 6u);
}

// Approximation gain = 2^(-H) × 256, linéaire, erreur < 5% sur H∈[0.1,0.9]
static inline uint32_t hlsl_fbm_gain_q8(uint8_t H_q8) {
    return 255u - (uint32_t)H_q8 / 2u;
}

// --- A : Header Unity SubShader ----------------------------------------------
static void hlsl_emit_header(void) {
    HLSL_EMIT("Shader \"Cerveau/ProceduralTexture\"\n");
    HLSL_EMIT("{\n");
    HLSL_EMIT("    SubShader\n");
    HLSL_EMIT("    {\n");
    HLSL_EMIT("        Tags { \"RenderType\"=\"Opaque\" }\n");
    HLSL_EMIT("        Pass\n");
    HLSL_EMIT("        {\n");
    HLSL_EMIT("            CGPROGRAM\n");
    HLSL_EMIT("            #pragma vertex vert\n");
    HLSL_EMIT("            #pragma fragment frag\n");
    HLSL_EMIT("            #include \"UnityCG.cginc\"\n");
    HLSL_EMIT("\n");
    HLSL_EMIT("            struct appdata { float4 vertex : POSITION; float2 uv : TEXCOORD0; };\n");
    HLSL_EMIT("            struct v2f    { float2 uv : TEXCOORD0; float4 vertex : SV_POSITION; };\n");
    HLSL_EMIT("\n");
    HLSL_EMIT("            v2f vert(appdata v)\n");
    HLSL_EMIT("            {\n");
    HLSL_EMIT("                v2f o;\n");
    HLSL_EMIT("                o.vertex = UnityObjectToClipPos(v.vertex);\n");
    HLSL_EMIT("                o.uv     = v.uv;\n");
    HLSL_EMIT("                return o;\n");
    HLSL_EMIT("            }\n");
    HLSL_EMIT("\n");
}

// --- B : Bibliothèque mathématique (Smart Include) ---------------------------
static void hlsl_emit_mathlib(const TextureSignature* sig) {

    // FBM nécessaire pour : strates, chaos, OU roche sédimentaire (domain warp micro),
    //                       OU grille/grain_fin (warp micro des brins)
    const bool need_fbm = sig->has_chaos || sig->has_stripes ||
                          (sig->macro_stripes && sig->micro_chaos) ||
                          sig->has_grid || sig->is_fine_grain || sig->is_isotropic;
    if (need_fbm) {
        const uint32_t oct     = hlsl_fbm_octaves(sig->entropy_q);
        const uint32_t gain_q8 = hlsl_fbm_gain_q8(sig->fractal_H_q8);

        HLSL_EMIT("            // === Hash + ValueNoise + FBM ===\n");
        HLSL_EMIT("            float hash(float2 p)\n");
        HLSL_EMIT("            {\n");
        HLSL_EMIT("                return frac(sin(dot(p, float2(127.1, 311.7))) * 43758.5453);\n");
        HLSL_EMIT("            }\n");
        HLSL_EMIT("\n");
        // valueNoise : interpolation bilinéaire de hash → FBM organique (vs sparkle du hash brut)
        HLSL_EMIT("            float valueNoise(float2 p)\n");
        HLSL_EMIT("            {\n");
        HLSL_EMIT("                float2 i = floor(p), f = frac(p);\n");
        HLSL_EMIT("                float2 u = f * f * (3.0 - 2.0 * f);\n");
        HLSL_EMIT("                return lerp(\n");
        HLSL_EMIT("                    lerp(hash(i),              hash(i + float2(1.0, 0.0)), u.x),\n");
        HLSL_EMIT("                    lerp(hash(i + float2(0.0, 1.0)), hash(i + float2(1.0, 1.0)), u.x),\n");
        HLSL_EMIT("                    u.y);\n");
        HLSL_EMIT("            }\n");
        HLSL_EMIT("\n");
        HLSL_EMIT("            float fbm(float2 p)\n");
        HLSL_EMIT("            {\n");
        HLSL_EMIT("                float v = 0.0, a = 0.5;\n");
        HLSL_EMIT("                float2 shift = float2(100.0, 100.0);\n");
        HLSL_EMIT("                for (int i = 0; i < %u; i++)\n", oct);
        HLSL_EMIT("                {\n");
        // valueNoise au lieu de hash brut → résultat lisse et organique
        HLSL_EMIT("                    v += a * (valueNoise(p) * 2.0 - 1.0);\n");
        HLSL_EMIT("                    p  = p * 2.0 + shift;\n");
        // gain par octave injecté comme entier / 256 — zéro float C++
        HLSL_EMIT("                    a *= (%u.0 / 256.0);\n", gain_q8);
        HLSL_EMIT("                }\n");
        HLSL_EMIT("                return v * 0.5 + 0.5;\n");
        HLSL_EMIT("            }\n");
        HLSL_EMIT("\n");
    }

    const bool need_voronoi = sig->has_grain || sig->has_grid || sig->is_fine_grain || sig->is_isotropic;
    // F2-F1 uniquement si matériau bimodal détecté (sinon défini mais jamais appelé = overhead inutile)
    const bool need_f2f1    = need_voronoi && (sig->bimodal_q8 > (uint8_t)THRESH_BIMODAL);

    if (need_voronoi) {
        HLSL_EMIT("            // === Voronoi F1 + F2-F1 (has_grain=%u, bimodal=%u) ===\n",
                  sig->has_grain, sig->bimodal_q8);
        HLSL_EMIT("            float2 hash2(float2 p)\n");
        HLSL_EMIT("            {\n");
        HLSL_EMIT("                p = float2(dot(p, float2(127.1, 311.7)),\n");
        HLSL_EMIT("                           dot(p, float2(269.5, 183.3)));\n");
        HLSL_EMIT("                return frac(sin(p) * 43758.5453);\n");
        HLSL_EMIT("            }\n");
        HLSL_EMIT("\n");
        HLSL_EMIT("            float voronoi(float2 p)\n");
        HLSL_EMIT("            {\n");
        HLSL_EMIT("                float2 i = floor(p), f = frac(p);\n");
        HLSL_EMIT("                float  d = 1.0;\n");
        HLSL_EMIT("                for (int y = -1; y <= 1; y++)\n");
        HLSL_EMIT("                for (int x = -1; x <= 1; x++)\n");
        HLSL_EMIT("                {\n");
        HLSL_EMIT("                    float2 n  = float2(x, y);\n");
        HLSL_EMIT("                    float2 r  = n + hash2(i + n) - f;\n");
        HLSL_EMIT("                    d = min(d, dot(r, r));\n");
        HLSL_EMIT("                }\n");
        HLSL_EMIT("                return sqrt(d);\n");
        HLSL_EMIT("            }\n");
        HLSL_EMIT("\n");
    }
    if (need_f2f1) {
        // Worley F2-F1 : met en évidence les BORDS de cellules → joints, pores, grains
        // F2-F1 ≈ 0 aux centres de cellules, ≈ 1 aux bords → aspect "joints entre grains"
        HLSL_EMIT("            // === Worley F2-F1 (matériau %u phases) ===\n", sig->n_phases);
        HLSL_EMIT("            float worley_f2f1(float2 p)\n");
        HLSL_EMIT("            {\n");
        HLSL_EMIT("                float2 i = floor(p), f = frac(p);\n");
        HLSL_EMIT("                float d1 = 8.0, d2 = 8.0;\n");
        HLSL_EMIT("                for (int y = -1; y <= 1; y++)\n");
        HLSL_EMIT("                for (int x = -1; x <= 1; x++)\n");
        HLSL_EMIT("                {\n");
        HLSL_EMIT("                    float2 n = float2(x, y);\n");
        HLSL_EMIT("                    float2 r = n + hash2(i + n) - f;\n");
        HLSL_EMIT("                    float  d = dot(r, r);\n");
        HLSL_EMIT("                    if (d < d1) { d2 = d1; d1 = d; }\n");
        HLSL_EMIT("                    else if (d < d2) { d2 = d; }\n");
        HLSL_EMIT("                }\n");
        HLSL_EMIT("                return saturate(sqrt(d2) - sqrt(d1));\n");
        HLSL_EMIT("            }\n");
        HLSL_EMIT("\n");
    }
}

// --- C : Corps du Fragment Shader --------------------------------------------
//
// Trois régimes de rendu selon les flags :
//
//   has_stripes + has_chaos → DOMAIN WARPING DOUBLE COUCHE (roche organique)
//     Le même fbm() est utilisé deux fois avec des offsets différents pour
//     "tordre l'espace" avant d'appliquer le sinus. Résultat : strates
//     irrégulières aux crêtes plates (smoothstep), aucune bande droite visible.
//
//   has_stripes seulement  → WARP SIMPLE (tissu, bois)
//     Un seul fbm() dévie légèrement le sinus. Bandes nettes mais pas rigides.
//
//   has_chaos seulement    → DISTORSION UV (texture chaotique)
//     Pas de strates. L'UV est perturbé avant le Voronoï ou le rendu solide.
//
//   has_grain (tous cas)   → VORONOÏ EN COUCHE SURFACE
//     Ajouté après les strates, toujours sur l'UV warpé si disponible.
//     Mix piloté par entropy_q (chaos élevé → grain très visible).
//
// --- Calcul des couleurs de crête/creux pour les strates organiques ----------
// La roche sédimentaire a des crêtes claires (calcaire, quartz) et des creux
// sombres (argile, oxyde de fer). On dérive deux tons depuis la moyenne Welford.
// Offsets asymétriques (crête +40R/+30G/+20B, creux -30R/-20G/-15B) correspondent
// à l'écart typique d'une roche ocre/brun photographiée en lumière naturelle.
// -----------------------------------------------------------------------------
static void hlsl_emit_frag(const TextureSignature* sig) {

    // Expressions UV selon l'angle Gabor dominant (sur l'UV warpé dans le shader)
    static const char* const UV_WARP[4] = {
        "warped_uv.x",
        "warped_uv.x + warped_uv.y",
        "warped_uv.y",
        "warped_uv.x - warped_uv.y",
    };
    // UV_PLAIN : réservé pour le mode sans warp (futur fallback si aniso_q=0)
    static const char* const UV_PLAIN[4] = {
        "uv.x", "uv.x + uv.y", "uv.y", "uv.x - uv.y",
    };
    (void)UV_PLAIN; // supprime le warning -Wunused quand tous les régimes warpent
    static const char* const ANGLE_COMMENT[4] = {
        "a=0 strates verticales",
        "a=1 strates diagonales ↗",
        "a=2 strates horizontales",
        "a=3 strates diagonales ↖",
    };

    const uint8_t dom   = (sig->dominant_angle < 4u) ? sig->dominant_angle : 2u;
    const uint32_t freq = hlsl_stripe_freq(sig->anisotropy_q);

    // Couleurs crête/creux pour le mode domain warp (calcul en C++, zéro float)
    const uint32_t r_hi = (sig->mean_r + 40u > 255u) ? 255u : sig->mean_r + 40u;
    const uint32_t g_hi = (sig->mean_g + 30u > 255u) ? 255u : sig->mean_g + 30u;
    const uint32_t b_hi = (sig->mean_b + 20u > 255u) ? 255u : sig->mean_b + 20u;
    const uint32_t r_lo = (sig->mean_r > 30u)  ? sig->mean_r - 30u : 0u;
    const uint32_t g_lo = (sig->mean_g > 20u)  ? sig->mean_g - 20u : 0u;
    const uint32_t b_lo = (sig->mean_b > 15u)  ? sig->mean_b - 15u : 0u;

    // Échelle Voronoï : valeur de base 2 (cellules larges → texture roche, pas sable)
    // haralick_contrast en Q10 : ÷8192 donne un bonus max de 1-2
    const uint32_t grain_scale = 2u + sig->haralick_contrast / 8192u;
    // Force du grain : entropy_q/4 → [0..63] → grain discret sur roche
    const uint32_t grain_mix   = (uint32_t)sig->entropy_q / 4u;

    // RÉGIME ISOTROPE : béton, plâtre, enduit — aucune direction dominante
    // Sous-régime bimodal (bimodal_q8 > THRESH_BIMODAL) : granite, liège, speckle 2 phases
    if (sig->is_isotropic) {
        const uint32_t r0=sig->palette[0][0], g0=sig->palette[0][1], b0=sig->palette[0][2];
        const uint32_t r1=sig->palette[1][0], g1=sig->palette[1][1], b1=sig->palette[1][2];
        const uint32_t r2=sig->palette[2][0], g2=sig->palette[2][1], b2=sig->palette[2][2];
        const uint32_t r3=sig->palette[3][0], g3=sig->palette[3][1], b3=sig->palette[3][2];
        const uint32_t blob_scale  = 3u + (uint32_t)sig->entropy_q / 64u;    // [3..6]
        const uint32_t mix_blob_q  = (uint32_t)sig->entropy_q / 3u;          // [0..85]
        const uint32_t pit_scale   = 35u + sig->haralick_contrast / 200u;    // [35..60] micro-pores fins
        const uint32_t pit_int_q   = 65u + (uint32_t)sig->entropy_q / 5u;    // [65..116] pores visibles
        const uint32_t warp_scale  = blob_scale + 1u;
        // Granulométrie bimodale : taille des inclusions depuis le contraste
        // Matériau non-chromatique (béton, plâtre) → inclusions petites, scale élevé
        // Matériau chromatique (granite coloré, liège) → inclusions larges, scale modéré
        const uint32_t incl_scale  = sig->is_chromatic
                                     ? (4u + sig->haralick_contrast / 3000u)   // [4..14]  granite
                                     : (25u + sig->haralick_contrast / 2000u);  // [25..36] béton — granulats minuscules
        // Force de la séparation de phases : bimodal_q8 / 255 ∈ [0..1]
        const uint32_t phase_sep_q = (sig->bimodal_q8 > (uint8_t)THRESH_BIMODAL)
                                      ? (uint32_t)sig->bimodal_q8 : 0u;
        // Non-stationnarité : module lent la teinte (veines, zones)
        const uint32_t nonstat_q   = (uint32_t)sig->spatial_var_q8;          // [0..255]

        HLSL_EMIT("            fixed4 frag(v2f i) : SV_Target\n");
        HLSL_EMIT("            {\n");
        HLSL_EMIT("                float2 uv = i.uv;\n\n");
        HLSL_EMIT("                // Palette histogramme (4 bins lum) — bimodal=%u, n_phases=%u\n",
                  sig->bimodal_q8, sig->n_phases);
        HLSL_EMIT("                float3 col0 = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n", r0,g0,b0);
        HLSL_EMIT("                float3 col1 = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n", r1,g1,b1);
        HLSL_EMIT("                float3 col2 = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n", r2,g2,b2);
        HLSL_EMIT("                float3 col3 = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n", r3,g3,b3);
        HLSL_EMIT("\n");
        HLSL_EMIT("                // Micro-warp isotrope\n");
        HLSL_EMIT("                float2 mwarp = float2(\n");
        HLSL_EMIT("                    fbm(uv * %u.0 + float2(1.7, 9.2)),\n", warp_scale);
        HLSL_EMIT("                    fbm(uv * %u.0 + float2(8.3, 2.8)));\n", warp_scale);
        HLSL_EMIT("                float2 warped = uv + mwarp * 0.025;\n");
        HLSL_EMIT("\n");

        if (phase_sep_q > 0u) {
            // ── Sous-régime bimodal : matériau 2 phases (granite, speckle, liège) ──
            // Phase mask depuis FBM à deux échelles → inclusion claires/sombres
            HLSL_EMIT("                // === 2-PHASE MATERIAL (bimodal=%u, incl_scale=%u) ===\n",
                      sig->bimodal_q8, incl_scale);
            HLSL_EMIT("                // Masque de phases : FBM lissé → seuil (inclusions)\n");
            HLSL_EMIT("                float phase_fbm = fbm(warped * %u.0);\n", incl_scale);
            HLSL_EMIT("                float phase_fbm2 = fbm(warped * %u.0 + float2(3.3, 7.1));\n",
                      incl_scale * 2u);
            // Transition : douce pour béton/plâtre, nette pour granite/liège
            if (sig->is_chromatic) {
                HLSL_EMIT("                float phase_mask = smoothstep(0.38, 0.62,\n");
                HLSL_EMIT("                    phase_fbm * 0.7 + phase_fbm2 * 0.3);\n");
                HLSL_EMIT("\n");
                HLSL_EMIT("                // Phase A (fond) ↔ Phase B (inclusions colorées)\n");
                HLSL_EMIT("                float3 col = lerp(col0, col3, phase_mask);\n");
                HLSL_EMIT("                col = lerp(col, col1, (1.0-phase_mask)*(%u.0/255.0));\n",
                          mix_blob_q);
                HLSL_EMIT("                col = lerp(col, col2, phase_mask * (%u.0/255.0));\n",
                          mix_blob_q / 2u + 10u);
            } else {
                // Béton/plâtre : transition très graduelle, amplitude réduite (surface fine, peu visible)
                // smoothstep large [0.35..0.65] + blend atténué → aspect "poussière de surface"
                HLSL_EMIT("                float phase_mask = smoothstep(0.20, 0.80,\n");
                HLSL_EMIT("                    phase_fbm * 0.60 + phase_fbm2 * 0.40);\n");
                HLSL_EMIT("\n");
                // Béton : surface quasi-uniforme → séparation de phases quasi-invisible
                HLSL_EMIT("                // Phase A (surface claire) ↔ Phase B (agrégats sombres)\n");
                HLSL_EMIT("                float3 col = lerp(col3, col2, phase_mask * 0.04);\n");
                HLSL_EMIT("                col = lerp(col, col2 * 0.95, (1.0-phase_mask) * (%u.0/2040.0));\n",
                          mix_blob_q / 2u + 8u);
            }
            HLSL_EMIT("\n");
            if (sig->is_chromatic) {
                // Joints Worley : uniquement pour matériaux chromatiques (granite, liège)
                // Le béton n'a pas de joints visibles à cette échelle
                HLSL_EMIT("                // Joints inter-grains (F2-F1) + micro-pores (Voronoi fin)\n");
                HLSL_EMIT("                float joints = worley_f2f1(warped * %u.0);\n", incl_scale);
                HLSL_EMIT("                col = lerp(col, col0 * 0.78, joints * (%u.0/510.0));\n",
                          pit_int_q);
            }
            // Micro-porosité : très atténuée pour béton (surface lisse)
            HLSL_EMIT("                float micro_pit = voronoi(warped * %u.0);\n", pit_scale);
            HLSL_EMIT("                col = lerp(col, col0 * 0.86, (1.0-micro_pit) * (%u.0/2040.0));\n",
                      pit_int_q / 2u);
            // Variation nuageuse grande échelle : c'est le vrai pattern du béton usé
            // Fréquence basse (0.35 UV) → taches de ~30-50% de l'image = nuage FBM
            HLSL_EMIT("                float tint_var = fbm(uv * 0.35 + float2(3.3, 7.1)) * 0.20 - 0.10;\n");
            HLSL_EMIT("                col = saturate(col + tint_var);\n");
        } else {
            // ── Régime béton/plâtre standard ──────────────────────────────────────
            HLSL_EMIT("                // FBM blobs macro (blob_scale=%u)\n", blob_scale);
            HLSL_EMIT("                float b0v = fbm(warped * %u.0);\n", blob_scale);
            HLSL_EMIT("                float b1v = fbm(warped * %u.0 + float2(5.1, 3.7));\n",
                      blob_scale * 2u);
            HLSL_EMIT("                float blob = b0v * 0.6 + b1v * 0.4;\n");
            // Béton/plâtre : base claire (col3/col2) → pas de lerp qui assombrit
            HLSL_EMIT("                float3 col = lerp(col3, col2, blob * 0.50);\n");
            HLSL_EMIT("                col = lerp(col, col2 * 0.90, (1.0-blob) * (%u.0/510.0));\n", mix_blob_q);
            HLSL_EMIT("                float vgrain_mac = voronoi(warped * 4.0);\n");
            HLSL_EMIT("                col = lerp(col, col1, (1.0 - vgrain_mac) * 0.10);\n");
            HLSL_EMIT("                float vgrain_pit = voronoi(warped * %u.0);\n", pit_scale);
            HLSL_EMIT("                col = lerp(col, col1 * 0.78, (1.0-vgrain_pit)*(%u.0/510.0));\n",
                      pit_int_q);
        }

        // Non-stationnarité : variation lente de teinte si spatial_var élevée
        if (nonstat_q > (uint32_t)THRESH_SPATIAL_VAR) {
            HLSL_EMIT("\n");
            HLSL_EMIT("                // Non-stationnarité : variation lente (spatial_var=%u)\n",
                      nonstat_q);
            HLSL_EMIT("                float slow_var = fbm(uv * 0.8 + float2(4.1, 2.7));\n");
            HLSL_EMIT("                col = lerp(col, col * (0.8 + slow_var * 0.4),\n");
            HLSL_EMIT("                          %u.0 / 255.0);\n", nonstat_q / 2u);
        }

        HLSL_EMIT("                return fixed4(col, 1.0);\n");
        HLSL_EMIT("            }\n");
        return;
    }

    // RÉGIME GRILLE : structure biaxiale (rattan, tissus, pavé)
    if (sig->has_grid) {
        // dominant_angle == 2 (transitions horiz) → brins horizontaux dominants → fy dense, fx sparse
        // dominant_angle == 0 (transitions vert)  → brins verticaux dominants   → fx dense, fy sparse
        const int horiz_dom = (sig->dominant_angle == 2u);

        // Axe dense (brins principaux) : fréquence depuis aniso_q_mac (structure grande échelle)
        const uint32_t freq_dense  = 12u + (uint32_t)sig->anisotropy_q_mac / 18u;  // [12..19]
        // Axe sparse (connecteurs) : fréquence depuis second_aniso_mic
        const uint32_t freq_sparse = 4u  + (uint32_t)sig->second_aniso_mic / 25u;  // [4..14]

        const uint32_t fy = horiz_dom ? freq_dense  : freq_sparse;
        const uint32_t fx = horiz_dom ? freq_sparse : freq_dense;

        // Duty cycles séparés : brin dense = très large (~78%), connecteur = étroit (~28%)
        // Recalibré sur source rattan : les brins horizontaux sont ~4-5× plus larges que les interstices
        const uint32_t duty_h_q8 = horiz_dom ? 200u : 72u;
        const uint32_t duty_v_q8 = horiz_dom ? 72u  : 200u;

        const uint32_t r0=sig->palette[0][0], g0=sig->palette[0][1], b0=sig->palette[0][2];
        const uint32_t r1=sig->palette[1][0], g1=sig->palette[1][1], b1=sig->palette[1][2];
        const uint32_t r2=sig->palette[2][0], g2=sig->palette[2][1], b2=sig->palette[2][2];
        const uint32_t r3=sig->palette[3][0], g3=sig->palette[3][1], b3=sig->palette[3][2];

        // Grain fibres : toujours actif pour réseaux tressés, amplitude calibrée sur entropie
        const uint32_t fiber_grain_q = 80u + (uint32_t)sig->entropy_q / 5u;  // [80..131]

        HLSL_EMIT("            fixed4 frag(v2f i) : SV_Target\n");
        HLSL_EMIT("            {\n");
        HLSL_EMIT("                float2 uv = i.uv;\n\n");
        HLSL_EMIT("                // Palette histogramme (4 bins lum)\n");
        HLSL_EMIT("                float3 col0 = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n", r0,g0,b0);
        HLSL_EMIT("                float3 col1 = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n", r1,g1,b1);
        HLSL_EMIT("                float3 col2 = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n", r2,g2,b2);
        HLSL_EMIT("                float3 col3 = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n", r3,g3,b3);
        HLSL_EMIT("\n");
        // Amplitude de warp : très régulier (spectral_flat < 30) → quasi-rigide
        // Organique (spectral_flat > 100) → légère ondulation de tissage
        const uint32_t warp_amp_q4 = (sig->spectral_flatness_q8 < 30u) ? 8u :
                                     (sig->spectral_flatness_q8 < 80u) ? 14u : 22u;
        HLSL_EMIT("                // Brins quasi-droits : warp calibré sur spectral_flat=%u\n",
                  sig->spectral_flatness_q8);
        HLSL_EMIT("                float2 micro_q = float2(\n");
        HLSL_EMIT("                    fbm(uv * 6.0 + float2(1.7, 9.2)),\n");
        HLSL_EMIT("                    fbm(uv * 6.0 + float2(8.3, 2.8)));\n");
        HLSL_EMIT("                float2 warped = uv + micro_q * (%u.0 / 1000.0);\n",
                  warp_amp_q4);
        HLSL_EMIT("\n");
        HLSL_EMIT("                // BRINS ANISOTROPES (fx=%u, fy=%u, duty_h=%.2f, duty_v=%.2f)\n",
                  fx, fy, (double)duty_h_q8/256.0, (double)duty_v_q8/256.0);
        HLSL_EMIT("                float duty_h = %u.0 / 256.0;\n", duty_h_q8);
        HLSL_EMIT("                float duty_v = %u.0 / 256.0;\n", duty_v_q8);
        HLSL_EMIT("                float brin_h = smoothstep(0.0, 0.05, duty_h - frac(warped.y * %u.0));\n", fy);
        HLSL_EMIT("                float brin_v = smoothstep(0.0, 0.05, duty_v - frac(warped.x * %u.0));\n", fx);
        HLSL_EMIT("\n");
        HLSL_EMIT("                // Alternance over/under + décalage brique (stagger)\n");
        HLSL_EMIT("                float row_idx = floor(warped.y * %u.0);\n", fy);
        HLSL_EMIT("                float over_h  = step(0.5, frac(row_idx * 0.5));\n");
        HLSL_EMIT("                // Décalage X alterné : rangées paires/impaires décalées d'une demi-période\n");
        HLSL_EMIT("                float row_shift = over_h * (0.5 / %u.0);\n", fx);
        HLSL_EMIT("                brin_v = smoothstep(0.0, 0.05, duty_v - frac(warped.x * %u.0 + row_shift));\n", fx);
        HLSL_EMIT("                float brin_v_vis = brin_v * (1.0 - over_h * brin_h);\n");
        HLSL_EMIT("                float brin_h_vis = brin_h * (1.0 - (1.0 - over_h) * brin_v);\n");
        HLSL_EMIT("\n");
        // Variation dramatique per-strip : certains brins très sombres (rotin naturel)
        // strip_dark : 30% de chance qu'un brin soit quasi-noir (hash < 0.30 → très sombre)
        HLSL_EMIT("                // Couleur : variation par brin — row ET col pour casser la répétition\n");
        HLSL_EMIT("                float col_idx    = floor(warped.x * %u.0);\n", fx);
        HLSL_EMIT("                float strip_hue  = hash(float2(row_idx + col_idx * 0.13, 7.31));\n");
        HLSL_EMIT("                float strip_dark = hash(float2(row_idx + col_idx * 0.07, 13.77));\n");
        HLSL_EMIT("                float strip_mid  = hash(float2(row_idx + col_idx * 0.09, 5.02));\n");
        // Base : lerp pleine amplitude col0 (quasi-noir) → col3 (ambre)
        HLSL_EMIT("                float3 brin_color = lerp(col0, col3, strip_hue);\n");
        // 35% des brins → poussés vers col0 * 0.50 = quasi-noir profond
        HLSL_EMIT("                brin_color = lerp(brin_color, col0 * 0.50, step(0.65, strip_dark) * (strip_dark - 0.65) * 2.86);\n");
        // Variation de teinte intermédiaire (col2 = marron moyen)
        HLSL_EMIT("                brin_color = lerp(brin_color, col2, strip_mid * 0.40);\n");
        HLSL_EMIT("                float3 col = col0 * 0.80;\n");  // fond inter-brins = marron sombre (pas noir)
        HLSL_EMIT("                col = lerp(col, brin_color, brin_h_vis);\n");
        HLSL_EMIT("                col = lerp(col, brin_color * 0.88, brin_v_vis);\n");
        HLSL_EMIT("\n");
        // Gradient interne par brin : plus clair au centre, plus sombre aux bords
        HLSL_EMIT("                // Gradient interne : centre clair, bords sombres (arrondi physique)\n");
        HLSL_EMIT("                float ph = frac(warped.y * %u.0);\n", fy);
        HLSL_EMIT("                float pv = frac(warped.x * %u.0 + row_shift);\n", fx);
        HLSL_EMIT("                float grad_h = 1.0 - abs(ph - 0.5) * 2.0;\n");
        HLSL_EMIT("                float grad_v = 1.0 - abs(pv - 0.5) * 2.0;\n");
        HLSL_EMIT("                col = lerp(col, col * 1.18, brin_h_vis * grad_h * grad_h * 0.4);\n");
        HLSL_EMIT("                col = lerp(col, col * 1.12, brin_v_vis * grad_v * grad_v * 0.3);\n");
        HLSL_EMIT("\n");
        // Ombre de contact aux croisements (occlusion ambiante)
        HLSL_EMIT("                // Occlusion ambiante croisement : ombre portée graduée\n");
        HLSL_EMIT("                float knot = brin_h * brin_v;\n");
        HLSL_EMIT("                float shadow_h = smoothstep(0.7, 1.0, frac(warped.y * %u.0));\n", fy);
        HLSL_EMIT("                float shadow_v = smoothstep(0.7, 1.0, frac(warped.x * %u.0 + row_shift));\n", fx);
        HLSL_EMIT("                float contact_shadow = knot * (shadow_h * 0.5 + shadow_v * 0.5);\n");
        HLSL_EMIT("                col = lerp(col, col * 0.55, contact_shadow * 0.65);\n");
        HLSL_EMIT("\n");
        // Grain de bois orienté selon l'axe de la lamelle
        HLSL_EMIT("                // Grain de bois orienté selon l'axe de la lamelle\n");
        if (horiz_dom) {
            HLSL_EMIT("                float fiber_g = fbm(float2(warped.x * 10.0, warped.y * 0.8));\n");
        } else {
            HLSL_EMIT("                float fiber_g = fbm(float2(warped.x * 0.8, warped.y * 10.0));\n");
        }
        HLSL_EMIT("                float fiber_g2 = fbm(float2(warped.x * 18.0, warped.y * 1.5) + float2(5.1, 3.7));\n");
        HLSL_EMIT("                float fiber_total = fiber_g * 0.6 + fiber_g2 * 0.4;\n");
        // Amplitude augmentée : 0.60 .. 1.40 (au lieu de 0.72..1.28) pour grain de surface visible
        HLSL_EMIT("                col = lerp(col, col * (0.60 + fiber_total * 0.80), %u.0/255.0);\n",
                  fiber_grain_q);
        HLSL_EMIT("\n");
        HLSL_EMIT("                return fixed4(col, 1.0);\n");
        HLSL_EMIT("            }\n");
        return;
    }

    // RÉGIME GRAIN_FIN : bois, fibre, tissu régulier
    if (sig->is_fine_grain) {
        // Freq grain bois : [20..46] — plaqué veiné large (chêne) à grain serré (érable)
        // Calibré sur source oak veneer ~43 fils / UV au niveau preview 512px
        const uint32_t freq_mac = 40u + (uint32_t)sig->anisotropy_q / 9u;  // [40..68]
        // Palette bois : dérivée depuis mean_rgb avec plage physique réelle du bois
        // Évite le problème de palette compressée (entropy_q~0 → bins quasi-identiques)
        // Dark  = mean * 0.82 (séparations inter-fibres sombres — ratio ~25% vs mean)
        // Mid   = mean * 1.00 (fibre neutre)
        // Light = mean * 1.15 (reflet spéculaire du fil de bois)
        const uint32_t rd = (uint32_t)sig->mean_r * 82u / 100u;
        const uint32_t gd = (uint32_t)sig->mean_g * 82u / 100u;
        const uint32_t bd = (uint32_t)sig->mean_b * 82u / 100u;
        const uint32_t rl = ((uint32_t)sig->mean_r * 115u / 100u > 255u) ? 255u : (uint32_t)sig->mean_r * 115u / 100u;
        const uint32_t gl = ((uint32_t)sig->mean_g * 115u / 100u > 255u) ? 255u : (uint32_t)sig->mean_g * 115u / 100u;
        const uint32_t bl = ((uint32_t)sig->mean_b * 115u / 100u > 255u) ? 255u : (uint32_t)sig->mean_b * 115u / 100u;
        const uint32_t r2 = sig->mean_r, g2 = sig->mean_g, b2 = sig->mean_b;
        // Axe du grain depuis l'angle dominant MICRO
        const uint8_t dom_fg = sig->dominant_angle < 4u ? sig->dominant_angle : 0u;
        // a=0 → transitions verticales → fils horizontaux → coord X
        // a=2 → transitions horizontales → fils verticaux → coord Y
        static const char* const GRAIN_COORD[4] = {
            "uv.x", "uv.x + uv.y", "uv.y", "uv.x - uv.y"
        };
        const uint32_t grain_scale_fg = 7u + sig->haralick_contrast / 6000u;  // micro-pores fins
        const uint32_t grain_mix_fg   = ((uint32_t)sig->contrast_q * 3u > 30u)
                                         ? (uint32_t)sig->contrast_q * 3u : 30u;

        HLSL_EMIT("            fixed4 frag(v2f i) : SV_Target\n");
        HLSL_EMIT("            {\n");
        HLSL_EMIT("                float2 uv = i.uv;\n\n");
        HLSL_EMIT("                // Palette bois : sombre (bin1) + clair (mean+offset)\n");
        HLSL_EMIT("                float3 col_dark  = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n", rd,gd,bd);
        HLSL_EMIT("                float3 col_light = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n", rl,gl,bl);
        HLSL_EMIT("                float3 col_mid   = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n", r2,g2,b2);
        HLSL_EMIT("\n");
        // ═══ DOMAIN WARP 2 COUCHES : veines de bois organiques ═══════════════════
        // Principe : les veines de bois = anneaux de croissance projetés sur coupe longitudinale
        // → chaque anneau est une courbe iso-distance depuis le centre de l'arbre
        // On simule ça avec un domain warp basse fréquence à grande amplitude.
        //
        // COUCHE MACRO (basse fréq 0.80 UV, amplitude 25%) → grandes courbes de croissance
        // COUCHE MICRO (haute fréq 3.50 UV, amplitude 6%) → irrégularités locales du grain
        // Les deux s'additionnent avant le sin() → veines vraiment organiques
        //
        // Axe transversal = GRAIN_COORD[(dom_fg+2)%4] (direction perpendiculaire au grain)
        // Axe principal   = GRAIN_COORD[dom_fg]

        // Warp macro-scale 2D (25-30% UV) : courbes de croissance annuelle
        HLSL_EMIT("                // === DOMAIN WARP 2-COUCHES : veines organiques ===\n");
        HLSL_EMIT("                float wx1 = fbm(uv * 0.80 + float2(1.3, 0.7)) * 0.05;\n");
        HLSL_EMIT("                float wy1 = fbm(uv * 0.80 + float2(8.1, 4.2)) * 0.02;\n");
        // Warp micro-scale 2D (6% UV) : imperfections de grain + nœuds
        HLSL_EMIT("                float wx2 = fbm(uv * 3.50 + float2(2.4, 9.1)) * 0.06;\n");
        HLSL_EMIT("                float wy2 = fbm(uv * 3.50 + float2(6.7, 1.3)) * 0.03;\n");
        HLSL_EMIT("                float2 warped_wood = uv + float2(wx1+wx2, wy1+wy2);\n");
        // grain_coord = distance depuis le pôle (centre du tronc hors UV)
        // Coupe longitudinale du bois = anneaux de croissance vus de côté
        // → le pôle est hors de l'espace UV, les anneaux sont des ellipses quasi-plates
        // → pattern cathédrale : légère courbure vers le haut au centre (veneer flat-sawn)
        if (dom_fg == 0u || dom_fg == 1u) {
            // Grain horizontal : pôle au-dessus de l'UV
            HLSL_EMIT("                // Pôle du tronc hors UV : pattern cathédrale (flat-sawn veneer)\n");
            HLSL_EMIT("                float2 pole = float2(0.5, 2.5);\n");
            HLSL_EMIT("                float2 dp   = warped_wood - pole;\n");
            HLSL_EMIT("                float grain_coord = sqrt(dp.x*dp.x*0.60 + dp.y*dp.y);\n");
        } else {
            // Grain vertical : pôle à droite de l'UV
            HLSL_EMIT("                // Pôle du tronc hors UV : pattern cathédrale (flat-sawn veneer)\n");
            HLSL_EMIT("                float2 pole = float2(2.5, 0.5);\n");
            HLSL_EMIT("                float2 dp   = warped_wood - pole;\n");
            HLSL_EMIT("                float grain_coord = sqrt(dp.x*dp.x + dp.y*dp.y*0.60);\n");
        }
        HLSL_EMIT("\n");
        HLSL_EMIT("                // Anneaux de croissance : freq=%u, micro-variation naturelle\n", freq_mac);
        // Variation très légère (veneer poncé = espacement quasi-uniforme)
        HLSL_EMIT("                float freq_mod = 0.95 + fbm(float2(warped_wood.x*0.30, warped_wood.y*0.20+5.1))*0.05;\n");
        HLSL_EMIT("                float raw = abs(sin(grain_coord * %u.0 * freq_mod * 3.14159));\n", freq_mac);
        // Bois tardif (late wood) = lignes sombres fines aux bords de l'anneau (raw→0)
        // Bois précoce (early wood) = zone claire large (raw→1) — physiquement correct
        HLSL_EMIT("                float band = 1.0 - smoothstep(0.10, 0.60, raw);\n");
        HLSL_EMIT("\n");
        // Variation de teinte par fil : réduite pour veneer — tons proches, pas de saut brutal
        // lid sans phase → hash stable (pas de décalage FBM sur l'ID = pas de patches de couleur)
        HLSL_EMIT("                // Couleur par fil : variation forte (chaque fibre a sa teinte)\n");
        // Couleur par fibre : basée sur grain_coord warpé → la couleur suit les courbes
        HLSL_EMIT("                float lid = floor(grain_coord * %u.0);\n", freq_mac);
        HLSL_EMIT("                float lhue  = hash(float2(lid, 0.73));\n");
        HLSL_EMIT("                float lhue2 = hash(float2(lid, 3.11));\n");
        // Variation inter-fibres faible (veneer chêne ~5-10% de variation)
        HLSL_EMIT("                float3 base_col = lerp(col_dark, col_light, lhue * 0.30 + 0.35);\n");
        HLSL_EMIT("                base_col = lerp(base_col, col_mid, lhue2 * 0.35);\n");
        // band=0 → cœur de fibre clair / band=1 → séparation sombre entre fibres
        HLSL_EMIT("                float3 col = lerp(base_col, col_dark, band * 0.28);\n");
        HLSL_EMIT("\n");
        // Micro-pores : seulement si entropie élevée (surface rugueuse)
        // entropy_q=0 (veneer lisse, plaqué poncé) → pas de Voronoi visible
        if (sig->entropy_q > 60u) {
            HLSL_EMIT("                // Micro-pores + grain de surface (deux échelles)\n");
            HLSL_EMIT("                float grain_fine = voronoi(uv * %u.0);\n", grain_scale_fg);
            HLSL_EMIT("                float grain_mac  = voronoi(uv * %u.0);\n", grain_scale_fg / 2u + 2u);
            HLSL_EMIT("                col = lerp(col, col * (0.78 + grain_fine * 0.44), %u.0/255.0);\n",
                      grain_mix_fg);
            HLSL_EMIT("                col = lerp(col, col * (0.88 + grain_mac * 0.24), %u.0/255.0);\n",
                      grain_mix_fg / 2u + 15u);
        }
        HLSL_EMIT("                return fixed4(col, 1.0);\n");
        HLSL_EMIT("            }\n");
        return;
    }

    // RÉGIME 0 (roche sédimentaire) émet son propre frag complet et fait return.
    // Les régimes 1/2/3 partagent l'ouverture ci-dessous.
    // On ouvre le frag générique SEULEMENT si on n'est pas en roche sédimentaire.
    if (!(sig->macro_stripes && sig->micro_chaos)) {
        HLSL_EMIT("            fixed4 frag(v2f i) : SV_Target\n");
        HLSL_EMIT("            {\n");
        HLSL_EMIT("                float2 uv = i.uv;\n");
        HLSL_EMIT("                float2 warped_uv = uv;\n");
        HLSL_EMIT("\n");
    }

    // Couleur de base — seulement pour les régimes 1/2/3 (pas RÉGIME 0 qui gère ses propres couleurs)
    if (!(sig->macro_stripes && sig->micro_chaos)) {
        HLSL_EMIT("                // Couleur de base (palette histogramme bin 2 = teinte principale)\n");
        HLSL_EMIT("                float3 col = float3(%u.0/255.0, %u.0/255.0, %u.0/255.0);\n",
                  sig->palette[2][0], sig->palette[2][1], sig->palette[2][2]);
        HLSL_EMIT("\n");
    }

    if (sig->macro_stripes && sig->micro_chaos) {
        // ── RÉGIME 0 : ROCHE SÉDIMENTAIRE — DOMAIN WARP GÉOLOGIQUE ─────────
        // Signature : strates larges (MACRO orienté) + surface rugueuse (MICRO isotrope)
        // Architecture shader :
        //   1. Un FBM "micro" (haute fréquence, entropy_q élevé) génère le désordre
        //      de surface → sert de perturbateur d'espace.
        //   2. Les UV sont warpés par ce FBM avant d'être injectés dans une fonction
        //      de strates basse fréquence (freq dérivée de MACRO, donc très basse).
        //   3. Le résultat = strates géologiques larges et irrégulières avec une
        //      rugosité de surface superposée. Exactement ce qu'on voit dans image.png.
        //
        // Fréquence macro : anisotropy_q_mac est faible (structure large) → freq basse
        // On utilise directement freq=2..5 (vs 8..23 pour le micro).
        // Fréquences multi-échelles : base 5 (min roche lisible) + bonus aniso
        const uint32_t freq_mac   = 5u + (uint32_t)sig->anisotropy_q_mac / 32u;  // [5..12]
        const uint32_t freq_meso  = freq_mac * 3u;                                 // [15..36]
        // Phase FBM : ondule les strates (entropy élevée → plus organique)
        const uint32_t phase_amp  = 2u + (uint32_t)sig->entropy_q / 64u;          // [2..5] strates plus organiques
        // Netteté des bords : haralick élevé → transitions plus dures
        const uint32_t soft       = (sig->haralick_contrast / 500u < 25u)
                                     ? 40u - sig->haralick_contrast / 500u : 15u;  // [15..40]
        const uint32_t smlo       = 50u - soft / 2u;   // smoothstep lo × 100
        const uint32_t smhi       = 50u + soft / 2u;   // smoothstep hi × 100
        // Fissures verticales : variance élevée → plus de fissures
        const uint32_t crack_x    = 4u + (uint32_t)sig->variance_lum / 250u;      // [4..]
        const uint32_t crack_vis  = (sig->variance_lum > 80u)
                                     ? 130u + sig->haralick_contrast / 120u : 0u;  // [0..255]
        // Palette 3 tons
        const uint32_t r_hi  = (sig->mean_r + 40u > 255u) ? 255u : sig->mean_r + 40u;
        const uint32_t g_hi  = (sig->mean_g + 30u > 255u) ? 255u : sig->mean_g + 30u;
        const uint32_t b_hi  = (sig->mean_b + 20u > 255u) ? 255u : sig->mean_b + 20u;
        const uint32_t r_lo  = sig->mean_r / 2u;   // creux argile très sombre
        const uint32_t g_lo  = sig->mean_g / 2u;
        const uint32_t b_lo  = sig->mean_b / 2u;
        // Grain de surface
        const uint32_t grain_scale = 2u + sig->haralick_contrast / 8192u;
        const uint32_t grain_mix   = (uint32_t)sig->entropy_q / 4u;

        HLSL_EMIT("            fixed4 frag(v2f i) : SV_Target\n");
        HLSL_EMIT("            {\n");
        HLSL_EMIT("                float2 uv = i.uv;\n\n");

        // Utiliser la palette histogramme pour des couleurs réelles extraites de l'image
        HLSL_EMIT("                // Palette histogramme (4 bins réels de l'image)\n");
        HLSL_EMIT("                float3 col_hi  = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n",
                  sig->palette[3][0], sig->palette[3][1], sig->palette[3][2]);
        HLSL_EMIT("                float3 col_mid = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n",
                  sig->palette[2][0], sig->palette[2][1], sig->palette[2][2]);
        HLSL_EMIT("                float3 col_lo  = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n",
                  sig->palette[0][0], sig->palette[0][1], sig->palette[0][2]);
        HLSL_EMIT("\n");

        // Micro-rugosité (micro_chaos) : FBM haute freq + warp subtil
        HLSL_EMIT("                // FBM micro-rugosité (entropy_q=%u, micro_chaos)\n",
                  sig->entropy_q);
        HLSL_EMIT("                float2 micro_q = float2(\n");
        HLSL_EMIT("                    fbm(uv * 8.0 + float2(1.7, 9.2)),\n");
        HLSL_EMIT("                    fbm(uv * 8.0 + float2(8.3, 2.8)));\n");
        HLSL_EMIT("                // Grain de surface : haute fréquence (poussière, micro-rugosité)\n");
        HLSL_EMIT("                float surf_grain = valueNoise(uv * 52.0) * 0.55 + valueNoise(uv * 98.0) * 0.28;\n");
        HLSL_EMIT("                float2 warped = uv + micro_q * 0.16 + float2(surf_grain, surf_grain) * 0.012;\n");
        HLSL_EMIT("\n");

        // Phase FBM : ondule les strates sans les détruire
        HLSL_EMIT("                // Modulation de phase : strates organiques ondulées\n");
        HLSL_EMIT("                float strat_phase = fbm(float2(warped.x * 1.5, warped.y * 0.5)) * %u.0;\n",
                  phase_amp);
        HLSL_EMIT("\n");

        // Strates multi-échelles : 2 fréquences harmoniques
        HLSL_EMIT("                // Strates multi-échelles (freq_mac=%u, freq_meso=%u)\n",
                  freq_mac, freq_meso);
        HLSL_EMIT("                // FBM lent sur Y → largeur des strates variable (géologie réelle)\n");
        HLSL_EMIT("                float strat_y = warped.y + fbm(float2(warped.x * 0.3, warped.y * 0.15)) * 0.16;\n");
        HLSL_EMIT("                float s_macro = abs(sin(strat_y * %u.0 * 3.14159 + strat_phase));\n",
                  freq_mac);
        HLSL_EMIT("                float s_meso  = abs(sin(strat_y * %u.0 * 3.14159 + strat_phase * 2.0));\n",
                  freq_meso);
        HLSL_EMIT("\n");

        // Ridged FBM : module les strates macro avec des crêtes dures
        HLSL_EMIT("                // Ridged FBM : crêtes dures (roche sédimentaire réelle)\n");
        HLSL_EMIT("                float ridged = abs(fbm(float2(warped.x * 2.0, warped.y * 4.0)) - 0.5) * 2.0;\n");
        HLSL_EMIT("                s_macro = s_macro * (0.65 + ridged * 0.35);\n");
        HLSL_EMIT("\n");

        // Transitions calibrées sur haralick_contrast
        HLSL_EMIT("                // Transitions (soft=%u → smoothstep %u/%u)\n",
                  soft, smlo, smhi);
        HLSL_EMIT("                float band_macro = smoothstep(%u.0/100.0, %u.0/100.0, s_macro);\n",
                  smlo, smhi);
        HLSL_EMIT("                float band_meso  = smoothstep(0.40, 0.60, s_meso);\n");
        HLSL_EMIT("\n");

        // Couleur à 3 tons
        HLSL_EMIT("                // Couleur : lerp 3 tons selon les deux échelles\n");
        HLSL_EMIT("                float3 strat_col = lerp(col_lo, col_hi, band_macro);\n");
        HLSL_EMIT("                strat_col = lerp(strat_col, col_mid, band_meso * 0.35);\n");
        HLSL_EMIT("\n");
        HLSL_EMIT("                // Couleur par couche : variation 4-bins (ocre, rouille, sédiment)\n");
        HLSL_EMIT("                float lid = floor(warped.y * %u.0 + strat_phase * 0.3);\n", freq_mac);
        HLSL_EMIT("                float lhue  = hash(float2(lid, 0.91));\n");
        HLSL_EMIT("                float lhue2 = hash(float2(lid, 2.47));\n");
        HLSL_EMIT("                float3 col_p1 = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n",
                  sig->palette[1][0], sig->palette[1][1], sig->palette[1][2]);
        // Layer tone : pleine amplitude col_lo→col_hi, enrichi de col_p1 (rouille/ocre)
        // Poids 0.70 au lieu de 0.50 → couleur de couche très présente
        HLSL_EMIT("                float3 layer_tone = lerp(col_lo, col_hi, lhue);\n");
        HLSL_EMIT("                layer_tone = lerp(layer_tone, col_p1, lhue2 * 0.80);\n");
        HLSL_EMIT("                // Injection d'une teinte rouille/rougeatre depuis le bin1 réel\n");
        HLSL_EMIT("                float lhue3 = hash(float2(lid, 8.53));\n");
        HLSL_EMIT("                layer_tone = lerp(layer_tone, col_p1 * 0.70, lhue3 * 0.45);\n");
        HLSL_EMIT("                strat_col = lerp(strat_col, layer_tone, 0.72);\n");
        HLSL_EMIT("\n");

        // Fissures verticales par valueNoise anisotrope
        if (crack_vis > 0u) {
            HLSL_EMIT("                // Fissures verticales (crack_x=%u, vis=%u/255)\n",
                      crack_x, crack_vis);
            // Vraies fissures minces : frac() + seuil 1.5% de la période
            HLSL_EMIT("                // Fissures irrégulières : warp fort = érosion angulaire réelle\n");
            HLSL_EMIT("                float crack_warp = fbm(warped * 2.3 + float2(4.1, 2.9));\n");
            HLSL_EMIT("                float crack_f = abs(frac(warped.x * %u.0\n", crack_x);
            HLSL_EMIT("                    + crack_warp * 0.62) - 0.5);\n");
            HLSL_EMIT("                float cracks = 1.0 - smoothstep(0.0, 0.028, crack_f);\n");
            HLSL_EMIT("                // Fissures profondes + bords oxydés (plus sombre que col_lo)\n");
            HLSL_EMIT("                strat_col = lerp(strat_col, col_lo * 0.4, cracks * (%u.0/255.0));\n",
                      crack_vis);
            HLSL_EMIT("\n");
        }

        // Grain de surface Voronoï ou F2-F1 si bimodal
        const bool rock_use_f2f1 = (sig->bimodal_q8 > (uint8_t)THRESH_BIMODAL);
        if (sig->has_grain) {
            HLSL_EMIT("                // Grain intra-strate (bimodal=%u → %s)\n",
                      sig->bimodal_q8, rock_use_f2f1 ? "F2-F1 joints" : "voronoi");
            HLSL_EMIT("                float strat_hash = hash(float2(lid, 3.17));\n");
            HLSL_EMIT("                float grain_sc   = %u.0 + strat_hash * 3.0;\n", grain_scale);
            HLSL_EMIT("                float grain_int  = 0.4 + strat_hash * 0.5;\n");
            if (rock_use_f2f1) {
                HLSL_EMIT("                float grain = worley_f2f1(warped * grain_sc);\n");
            } else {
                HLSL_EMIT("                float grain = voronoi(warped * grain_sc);\n");
            }
            HLSL_EMIT("                float3 col = lerp(strat_col,\n");
            HLSL_EMIT("                               strat_col * (0.55 + grain * grain_int),\n");
            HLSL_EMIT("                               %u.0/255.0);\n", grain_mix);
        } else {
            HLSL_EMIT("                float3 col = strat_col;\n");
        }
        // Non-stationnarité : gradient lent de tons (veines minérales)
        if (sig->is_nonstationary) {
            HLSL_EMIT("\n");
            HLSL_EMIT("                // Non-stationnarité : variation lente de tons (spatial_var=%u)\n",
                      sig->spatial_var_q8);
            HLSL_EMIT("                float vein_mod = fbm(uv * 0.6 + float2(5.3, 1.9));\n");
            HLSL_EMIT("                col = lerp(col, col * (0.75 + vein_mod * 0.5),\n");
            HLSL_EMIT("                          %u.0 / 255.0);\n", (uint32_t)sig->spatial_var_q8 / 2u);
        }
        HLSL_EMIT("\n");
        HLSL_EMIT("                return fixed4(col, 1.0);\n");
        HLSL_EMIT("            }\n");
        return; // shader complet, on sort
    }

    if (sig->has_stripes && sig->has_chaos) {
        // ── RÉGIME 1 : DOMAIN WARPING DOUBLE COUCHE ─────────────────────────
        // Principe : on warp l'espace UV avec fbm(), puis on warp encore le
        // résultat avec un second fbm() décalé. Le sinus voit un espace si
        // tordu que les bandes deviennent des couches organiques irrégulières.
        // Les offsets (1.7/9.2, 8.3/2.8) sont des constantes irrationnelles
        // qui évitent la répétition symétrique visible à l'oeil.
        HLSL_EMIT("                // === DOMAIN WARP DOUBLE (roche/strates organiques) ===\n");
        HLSL_EMIT("                // has_stripes=%u + has_chaos=%u : entropy_q=%u\n",
                  sig->has_stripes, sig->has_chaos, sig->entropy_q);
        HLSL_EMIT("                // Couche 1 : deux fbm décalés → vecteur de déplacement q\n");
        HLSL_EMIT("                float2 q = float2(\n");
        HLSL_EMIT("                    fbm(uv * 2.0 + float2(1.7, 9.2)),\n");
        HLSL_EMIT("                    fbm(uv * 2.0 + float2(8.3, 2.8)));\n");
        HLSL_EMIT("\n");
        // Force de la couche 1 : entropy_q/255 (chaos élevé → déformation forte)
        HLSL_EMIT("                // Couche 2 : fbm du fbm (domain warp récursif)\n");
        HLSL_EMIT("                float warp_str = %u.0 / 255.0; // entropy_q normalisé\n",
                  sig->entropy_q);
        HLSL_EMIT("                float2 r = float2(\n");
        HLSL_EMIT("                    fbm(uv + q * warp_str + float2(1.7, 9.2)),\n");
        HLSL_EMIT("                    fbm(uv + q * warp_str + float2(8.3, 2.8)));\n");
        HLSL_EMIT("\n");
        // Force de la couche 2 : entropy_q/510 (moitié moins fort pour éviter la bouillie)
        HLSL_EMIT("                warped_uv = uv + r * (%u.0 / 510.0);\n",
                  sig->entropy_q);
        HLSL_EMIT("\n");
        HLSL_EMIT("                // Strates sur espace warpé (%s, freq=%u)\n",
                  ANGLE_COMMENT[dom], freq);
        HLSL_EMIT("                float coord_w = %s;\n", UV_WARP[dom]);
        // abs(sin) donne des crêtes symétriques [0..1] avec 0=creux, 1=crête
        // smoothstep aplatit le milieu : les crêtes des couches sédimentaires
        // sont plates, pas sinusoïdales. edge0/edge1 autour de 0.5.
        HLSL_EMIT("                float raw_s  = abs(sin(coord_w * %u.0 * 3.14159));\n",
                  freq);
        HLSL_EMIT("                float band   = smoothstep(0.30, 0.70, raw_s);\n");
        HLSL_EMIT("\n");
        // Deux couleurs dérivées de la moyenne Welford (calcul entier en C++)
        HLSL_EMIT("                // Crête = ton clair, creux = ton sombre (écart Welford)\n");
        HLSL_EMIT("                float3 col_hi = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n",
                  r_hi, g_hi, b_hi);
        HLSL_EMIT("                float3 col_lo = float3(%u.0/255.0,%u.0/255.0,%u.0/255.0);\n",
                  r_lo, g_lo, b_lo);
        HLSL_EMIT("                col = lerp(col_lo, col_hi, band);\n");
        HLSL_EMIT("\n");

    } else if (sig->has_stripes) {
        // ── RÉGIME 2 : WARP SIMPLE (bois, tissu) ────────────────────────────
        // Un seul fbm() dévie légèrement la coordonnée du sinus.
        // Les bandes restent lisibles et rythmées, mais ne sont pas rigides.
        HLSL_EMIT("                // === WARP SIMPLE (strates nettes, has_stripes=%u) ===\n",
                  sig->has_stripes);
        HLSL_EMIT("                float warp   = fbm(uv * 3.0) * (%u.0 / 255.0);\n",
                  sig->anisotropy_q);
        HLSL_EMIT("                warped_uv = uv + float2(warp, warp) * 0.3;\n");
        HLSL_EMIT("                float coord_w = %s;\n", UV_WARP[dom]);
        HLSL_EMIT("                float band = sin(coord_w * %u.0 * 6.2832) * 0.5 + 0.5;\n",
                  freq);
        HLSL_EMIT("                col = lerp(col * (1.0 - %u.0/510.0),\n",
                  sig->anisotropy_q);
        HLSL_EMIT("                          col * (1.0 + %u.0/510.0), band);\n",
                  sig->anisotropy_q);
        HLSL_EMIT("\n");

    } else if (sig->has_chaos) {
        // ── RÉGIME 3 : CHAOS PUR (pas de strates, distorsion UV seule) ──────
        HLSL_EMIT("                // === DISTORSION UV (has_chaos=%u, entropy_q=%u) ===\n",
                  sig->has_chaos, sig->entropy_q);
        HLSL_EMIT("                warped_uv = uv + (fbm(uv * 8.0) - 0.5) * (%u.0/255.0);\n",
                  sig->entropy_q);
        HLSL_EMIT("\n");
    }

    if (sig->has_grain) {
        // ── GRAIN VORONOÏ EN COUCHE SURFACE ─────────────────────────────────
        // Appliqué sur l'UV warpé (si available) → le grain suit les strates.
        // grain_scale dérivé du contraste GLCM Q10 : plus c'est rugueux, plus
        // les cellules Voronoï sont petites (grain serré).
        // grain_mix = entropy_q/2 : chaos maximal → grain très présent.
        HLSL_EMIT("                // === GRAIN VORONOÏ (rugosité de surface) ===\n");
        HLSL_EMIT("                // grain_scale=%u (Q10=%u), grain_mix=%u (entropy_q=%u)\n",
                  grain_scale, sig->haralick_contrast, grain_mix, sig->entropy_q);
        HLSL_EMIT("                float grain = voronoi(warped_uv * %u.0);\n", grain_scale);
        // Modulation : grain renforce les zones sombres (aspect grenu/poreux)
        HLSL_EMIT("                col = lerp(col, col * (0.65 + grain * 0.7),\n");
        HLSL_EMIT("                          %u.0 / 255.0);\n", grain_mix);
        HLSL_EMIT("\n");
    }

    HLSL_EMIT("                return fixed4(col, 1.0);\n");
    HLSL_EMIT("            }\n");
}

// --- D : Footer --------------------------------------------------------------
static void hlsl_emit_footer(void) {
    HLSL_EMIT("\n");
    HLSL_EMIT("            ENDCG\n");
    HLSL_EMIT("        }\n");
    HLSL_EMIT("    }\n");
    HLSL_EMIT("}\n");
}

// --- Orchestrateur Phase 3 ---------------------------------------------------
static void phase3_draw(const TextureSignature* sig) {
    hlsl_emit_header();
    hlsl_emit_mathlib(sig);
    hlsl_emit_frag(sig);
    hlsl_emit_footer();
}


// =============================================================================
//  DÉBOGAGE PC — Affichage des résultats Phase 1
//  Ces fonctions sont compilées uniquement hors Pico (printf coûte trop en flash)
// =============================================================================

#ifndef PICO_BUILD

static void debug_print_welford(const WelfordState* wf) {
    if (wf->n == 0u) return;

    // mean = Σlum / n   (exact, pas de troncature accumulée)
    const uint32_t mean_lum = wf->sum_lum / wf->n;

    // variance = Σlum²/n − mean²   (formule König-Huygens, stable pour uint8_t)
    // sum_sq_lum/n : max 65025, mean² : max 65025 → différence dans uint32_t ✓
    const uint32_t sq_mean  = (uint32_t)(wf->sum_sq_lum / (uint64_t)wf->n);
    const uint32_t variance = (sq_mean >= mean_lum * mean_lum)
                            ? (sq_mean - mean_lum * mean_lum)
                            : 0u;  // garde-fou numérique

    const uint32_t mean_r = wf->sum_r / wf->n;
    const uint32_t mean_g = wf->sum_g / wf->n;
    const uint32_t mean_b = wf->sum_b / wf->n;

    DBG_PRINTF("┌─ WELFORD ─────────────────────────────────────┐\n");
    DBG_PRINTF("│  Pixels traites : %u\n", wf->n);
    DBG_PRINTF("│  Lum. moyenne   : %u  (attendu ~137 pour bois beige)\n", mean_lum);
    DBG_PRINTF("│  Variance lum   : %u  (attendu ~2000-3000 pour strates)\n", variance);
    DBG_PRINTF("│  Couleur moy    : R=%-3u G=%-3u B=%-3u\n", mean_r, mean_g, mean_b);
    DBG_PRINTF("└───────────────────────────────────────────────┘\n");
}

static void debug_print_glcm(const GLCMState* gc) {
    DBG_PRINTF("┌─ GLCM ────────────────────────────────────────┐\n");
    DBG_PRINTF("│  Paires horizontales : %u\n", gc->total_pairs);
    DBG_PRINTF("│  Diagonale (auto-transitions) :\n│  ");
    for (uint32_t i = 0u; i < GLCM_LEVELS; i++) {
        DBG_PRINTF("%5u ", gc->matrix[i][i]);
        if (i == 7u) DBG_PRINTF("\n│  ");
    }
    DBG_PRINTF("\n");
    DBG_PRINTF("│  Lecture : valeurs hautes = pixels très similaires\n");
    DBG_PRINTF("│  → concentré sur quelques cases = texture régulière\n");
    DBG_PRINTF("└───────────────────────────────────────────────┘\n");
}

static void debug_print_gabor(const GaborState* gb) {

    // Trouve l'angle dominant et calcule l'énergie totale
    uint8_t  dom   = 0u;
    int64_t  max_e = gb->energy[0];
    int64_t  total = 0;
    for (uint8_t a = 0u; a < GABOR_ANGLES; a++) {
        total += gb->energy[a];
        if (gb->energy[a] > max_e) { max_e = gb->energy[a]; dom = a; }
    }

    // Noms d'angles (convention : θ = direction de la porteuse)
    static const char* const NAMES[4] = {
        " 0deg (porteuse X → detecte transitions verticales)",
        "45deg (porteuse diag ↗)",
        "90deg (porteuse Y → detecte transitions horizontales)",
        "135deg (porteuse diag ↖)"
    };

    DBG_PRINTF("┌─ GABOR ─────────────────────────────────────────────────┐\n");
    DBG_PRINTF("│  Fenetres convoluees : %u  (lignes 2..%u analysees)\n",
               gb->n_windows, gb->n_windows > 3u ? gb->n_windows - 3u : 0u);
    DBG_PRINTF("│\n");
    DBG_PRINTF("│  Energies par orientation :\n");

    for (uint8_t a = 0u; a < GABOR_ANGLES; a++) {
        // Pourcentage de l'énergie totale (×100 puis division entière)
        const uint32_t pct = (total > 0)
            ? (uint32_t)((gb->energy[a] * 100LL) / total)
            : 0u;
        // Barre ASCII proportionnelle (20 caractères max)
        const uint32_t bar_len = pct / 5u;
        char bar[21];
        for (uint32_t i = 0u; i < 20u; i++) bar[i] = (i < bar_len) ? '#' : '.';
        bar[20] = '\0';

        DBG_PRINTF("│  a=%u  %-52s  E=%2u%%  [%s]\n",
                   a, NAMES[a], pct, bar);
    }

    DBG_PRINTF("│\n");
    DBG_PRINTF("│  Angle DOMINANT : a=%u  theta=%s\n", dom, NAMES[dom]);
    DBG_PRINTF("│\n");
    // Validation bouchon : uniquement si on n'est pas en mode image réelle
#ifndef PICO_BUILD
    if (_img_data == nullptr) {
        DBG_PRINTF("│  Attendu pour bouchon bois : a=2 (strates horizontales)\n");
        if (dom == 2u) {
            DBG_PRINTF("│  VALIDATION : SUCCES - la patate voit les strates !\n");
        } else {
            DBG_PRINTF("│  ATTENTION  : bouchon bois - resultat inattendu\n");
        }
    }
#endif
    DBG_PRINTF("└─────────────────────────────────────────────────────────┘\n");
}

static void debug_print_signature(const TextureSignature* sig) {
    static const char* const ANGLE_NAMES[4] = {
        " 0deg (transitions verticales)",
        "45deg (diag ↗)",
        "90deg (transitions horizontales)",
        "135deg (diag ↖)"
    };
    DBG_PRINTF("┌─ TEXTURE SIGNATURE ────────────────────────────────────────┐\n");
    DBG_PRINTF("│  Luminance : mean=%-3u  variance=%-6u\n",
               sig->mean_lum, sig->variance_lum);
    DBG_PRINTF("│  Couleur   : R=%-3u G=%-3u B=%-3u\n",
               sig->mean_r, sig->mean_g, sig->mean_b);
    DBG_PRINTF("│\n");
    DBG_PRINTF("│  Haralick (GLCM) :\n");
    DBG_PRINTF("│    contrast_q    = %-3u  (Q10=%u, brut=%.2f)\n",
               sig->contrast_q, sig->haralick_contrast,
               (double)sig->haralick_contrast / 1024.0);
    DBG_PRINTF("│    homogeneity_q = %-3u  (Q16=%u)\n",
               sig->homogeneity_q, sig->haralick_homog_q16);
    DBG_PRINTF("│    entropy_q     = %-3u\n", sig->entropy_q);
    DBG_PRINTF("│    fractal_H_q8  = %-3u  (H=%.3f)\n",
               sig->fractal_H_q8, (double)sig->fractal_H_q8 / 256.0);
    DBG_PRINTF("│\n");
    DBG_PRINTF("│  Gabor couche fine  (λ=4px)  :\n");
    DBG_PRINTF("│    angle dominant = a=%u  %s\n",
               sig->dominant_angle, ANGLE_NAMES[sig->dominant_angle]);
    DBG_PRINTF("│    anisotropy_q   = %-3u  (0=iso, 255=fort)\n",
               sig->anisotropy_q);
    DBG_PRINTF("│  Gabor couche LOW   (λ=16px) :\n");
    DBG_PRINTF("│    angle dominant = a=%u  %s\n",
               sig->dominant_angle_low, ANGLE_NAMES[sig->dominant_angle_low]);
    DBG_PRINTF("│    anisotropy_q_low= %-3u\n", sig->anisotropy_q_low);
    DBG_PRINTF("│  Gabor couche MACRO (λ=64px) :\n");
    DBG_PRINTF("│    angle dominant = a=%u  %s\n",
               sig->dominant_angle_mac, ANGLE_NAMES[sig->dominant_angle_mac]);
    DBG_PRINTF("│    anisotropy_q_mac= %-3u\n", sig->anisotropy_q_mac);
    // Lecture croisée des 3 couches
    const uint8_t t = (uint8_t)THRESH_ANISO_STRIPES;
    if (sig->anisotropy_q_mac > t && sig->anisotropy_q <= t) {
        DBG_PRINTF("│    ↳ MACRO orienté + MICRO isotrope → ROCHE SEDIMENTAIRE (strates larges)\n");
    } else if (sig->anisotropy_q_mac > t && sig->anisotropy_q_low > t && sig->anisotropy_q > t) {
        DBG_PRINTF("│    ↳ PYRAMIDE COHERENTE 3 niveaux → structure forte multi-échelle\n");
    } else if (sig->anisotropy_q_low > t || sig->anisotropy_q_mac > t) {
        DBG_PRINTF("│    ↳ strate détectée couche LOW/MACRO uniquement\n");
    } else if (sig->anisotropy_q > t) {
        DBG_PRINTF("│    ↳ structure FINE seulement (λ=4px) — grain/fibre\n");
    } else {
        DBG_PRINTF("│    ↳ isotrope aux 3 échelles → texture homogene\n");
    }
    DBG_PRINTF("│  Flags sémantiques :\n");
    DBG_PRINTF("│    macro_stripes=%u  micro_chaos=%u\n",
               sig->macro_stripes, sig->micro_chaos);
    DBG_PRINTF("│    has_grid=%u  is_fine_grain=%u\n",
               sig->has_grid, sig->is_fine_grain);
    {
        const uint8_t ad = (sig->dominant_angle >= sig->second_angle_mic)
                           ? (sig->dominant_angle - sig->second_angle_mic)
                           : (sig->second_angle_mic - sig->dominant_angle);
        DBG_PRINTF("│    second_angle_mic=%u  second_aniso_mic=%u  adiff=%u\n",
                   sig->second_angle_mic, sig->second_aniso_mic, (unsigned)ad);
    }
    DBG_PRINTF("│    Palette 4 bins : ");
    for (uint8_t b=0; b<4; b++)
        DBG_PRINTF("[%u,%u,%u] ", sig->palette[b][0], sig->palette[b][1], sig->palette[b][2]);
    DBG_PRINTF("\n");
    DBG_PRINTF("│\n");
    DBG_PRINTF("│  Flags Phase 3 :\n");
    DBG_PRINTF("│    has_stripes=%u  has_grain=%u  has_chaos=%u\n",
               sig->has_stripes, sig->has_grain, sig->has_chaos);
    DBG_PRINTF("│\n");
    DBG_PRINTF("│  Descripteurs v3 :\n");
    DBG_PRINTF("│    LBP       : uniformity=%-3u  entropy=%-3u  microtexton=%-3u\n",
               sig->lbp_uniformity_q8, sig->lbp_entropy_q8, sig->microtexton_q8);
    DBG_PRINTF("│    AutoCorr  : period_x=%-3u  spectral_flat=%-3u  has_periodic=%u\n",
               sig->period_x_q8, sig->spectral_flatness_q8, sig->has_periodic);
    DBG_PRINTF("│    Spatial   : spatial_var=%-3u  nonstat=%u  is_nonstationary=%u\n",
               sig->spatial_var_q8, sig->nonstationarity_q8, sig->is_nonstationary);
    DBG_PRINTF("│    Chroma    : chroma_strength=%-3u  is_chromatic=%u\n",
               sig->chroma_strength_q8, sig->is_chromatic);
    DBG_PRINTF("│    Phases    : bimodal=%-3u  n_phases=%u\n",
               sig->bimodal_q8, sig->n_phases);
    DBG_PRINTF("│\n");
    DBG_PRINTF("│  Diagnostic : ");
    if (sig->macro_stripes && sig->micro_chaos) {
        DBG_PRINTF("ROCHE SEDIMENTAIRE → Domain Warp double couche");
        if (sig->is_nonstationary) DBG_PRINTF(" + veines");
        if (sig->bimodal_q8 > (uint8_t)THRESH_BIMODAL) DBG_PRINTF(" + joints F2-F1");
        DBG_PRINTF("\n");
    } else if (sig->macro_stripes && sig->has_grain) {
        DBG_PRINTF("roche/bois grenu → strates macro + Voronoi surface\n");
    } else if (sig->is_isotropic && sig->bimodal_q8 > (uint8_t)THRESH_BIMODAL) {
        DBG_PRINTF("MATERIAU 2 PHASES → granite/speckle/liege (bimodal=%u)\n", sig->bimodal_q8);
    } else if (sig->is_isotropic) {
        DBG_PRINTF("beton/platre → isotrope (n_phases=%u)\n", sig->n_phases);
    } else if (sig->has_grid) {
        DBG_PRINTF("tissu biaxial → grille (fx/fy depuis Gabor)\n");
    } else if (sig->has_stripes && sig->has_chaos) {
        DBG_PRINTF("strates + chaos → FBM directionnel warpe\n");
    } else if (sig->has_stripes) {
        DBG_PRINTF("texture orientee (tissu, bois lisse) → warp simple\n");
    } else if (sig->has_grain && sig->has_chaos) {
        DBG_PRINTF("surface rugueuse (pierre, beton) → FBM turbulence\n");
    } else if (sig->has_grain) {
        DBG_PRINTF("grain regulier (sable, liege) → Voronoi\n");
    } else if (sig->has_chaos) {
        DBG_PRINTF("texture chaotique → distorsion UV\n");
    } else {
        DBG_PRINTF("texture lisse/uniforme → couleur solide\n");
    }
    if (sig->is_chromatic)     DBG_PRINTF("│  + couleur porteuse (chroma=%u)\n", sig->chroma_strength_q8);
    if (sig->has_periodic)     DBG_PRINTF("│  + periodicite detectee (period_x=%u)\n", sig->period_x_q8);
    if (sig->is_nonstationary) DBG_PRINTF("│  + non-stationnaire (spatial_var=%u)\n", sig->spatial_var_q8);
    DBG_PRINTF("└────────────────────────────────────────────────────────────┘\n");
}

#endif // !PICO_BUILD


// =============================================================================
//  MAIN
// =============================================================================

int main(int argc, char* argv[]) {

#ifdef PICO_BUILD
    stdio_init_all();
    (void)argc; (void)argv;
#endif

    // =========================================================================
    //  Dimensions de l'image — runtime sur PC, compile-time sur Pico
    // =========================================================================
    uint32_t actual_width  = IMG_WIDTH;
    uint32_t actual_height = IMG_HEIGHT;

#ifndef PICO_BUILD
    // ── Chargement image réelle si un fichier est passé en argument ───────────
    // Usage : ./cerveau image.png
    //         ./cerveau           (bouchon LCG si aucun argument)
    if (argc >= 2) {
        int w = 0, h = 0, ch = 0;
        // stbi_load force 3 canaux (RGB) quelle que soit la source (RGBA, grey…)
        uint8_t* pixels = stbi_load(argv[1], &w, &h, &ch, 3);
        if (pixels == nullptr) {
            fprintf(stderr, "ERREUR : impossible de charger '%s'\n  %s\n",
                    argv[1], stbi_failure_reason());
            return 1;
        }
        if ((uint32_t)w > IMG_MAX_WIDTH || (uint32_t)h > IMG_MAX_HEIGHT) {
            fprintf(stderr, "ERREUR : image %d×%d dépasse IMG_MAX %u×%u\n",
                    w, h, IMG_MAX_WIDTH, IMG_MAX_HEIGHT);
            stbi_image_free(pixels);
            return 1;
        }
        actual_width   = (uint32_t)w;
        actual_height  = (uint32_t)h;
        _img_data      = pixels;
        _img_n_pixels  = (uint32_t)w * (uint32_t)h;
        DBG_PRINTF("\n=== IMAGE REELLE : %s  (%d × %d, %d canaux source) ===\n",
                   argv[1], w, h, ch);
    } else {
        DBG_PRINTF("\n=== BOUCHON LCG (aucun fichier fourni) ===\n");
#ifdef TEST_MODE_STONE
        DBG_PRINTF("    Mode : PIERRE/SABLE\n");
#else
        DBG_PRINTF("    Mode : BOIS/STRATES\n");
#endif
    }
#endif

    // =========================================================================
    //  Allocation statique — ZÉRO malloc, ZÉRO fragmentation mémoire
    //  'static' au scope fonction → segment BSS (initialisé à 0 au boot Pico)
    // =========================================================================
    // ── Analyseurs v1/v2 ──────────────────────────────────────────────────────
    static WelfordState     wf;
    static GLCMState        gc;
    static GaborState       gb;      // MICRO  λ=4px
    static GaborStateLow    gb_low;  // LOW    λ=16px (décimée ×4)
    static GaborStateMacro  gb_mac;  // MACRO  λ=64px (décimée ×16)
    // ── Analyseurs v3 ─────────────────────────────────────────────────────────
    static LBPState         lbp;     // LBP histogram (44 B)
    static AutoCorrState    ac;      // Autocorr horizontale 0..8 (120 B)
    static SpatialState     sp;      // Tuilage spatial 4×4 (128 B)
    static TextureSignature sig;

    welford_init(&wf);
    glcm_init(&gc);
    gabor_init(&gb);
    memset(&gb_low, 0, sizeof(gb_low));
    memset(&gb_mac, 0, sizeof(gb_mac));
    lbp_init(&lbp);
    acorr_init(&ac);
    spatial_init(&sp);
    signature_init(&sig);

    // LBP LUT : précalculée une seule fois au boot (stockée en BSS, ~4000 cycles)
    lbp_lut_build();

    // =========================================================================
    //  PHASE 1 — DÉCOMPOSER
    // =========================================================================
    DBG_PRINTF("\n=== CERVEAU v3 — DÉMARRAGE PHASE 1 ===\n");
    DBG_PRINTF("Flux : %u × %u = %u pixels\n\n",
               actual_width, actual_height, actual_width * actual_height);

    phase1_decompose(&wf, &gc, &gb, &gb_low, &gb_mac, &lbp, &ac, &sp,
                     actual_width, actual_height);

#ifndef PICO_BUILD
    DBG_PRINTF("\n=== PHASE 1 TERMINÉE ===\n\n");
    debug_print_welford(&wf);
    debug_print_glcm(&gc);
    debug_print_gabor(&gb);
    DBG_PRINTF("LBP pixels traites : %u\n", lbp.n_pixels);
    DBG_PRINTF("AutoCorr paires[0] : %u\n", ac.n_pairs[0]);
#endif

    // =========================================================================
    //  PHASE 2 — COMPRENDRE
    // =========================================================================
    DBG_PRINTF("\n=== PHASE 2 — COMPRENDRE ===\n\n");

    phase2_understand(&wf, &gc, &gb, &gb_low, &gb_mac, &lbp, &ac, &sp, &sig);

#ifndef PICO_BUILD
    debug_print_signature(&sig);
#endif

    // =========================================================================
    //  PHASE 3 — DESSINER
    // =========================================================================
#ifndef PICO_BUILD
    DBG_PRINTF("\n=== PHASE 3 — SHADER HLSL GÉNÉRÉ ===\n");
    DBG_PRINTF("─────────────────────────────────────────────────────────────\n");
#endif

    phase3_draw(&sig);

#ifndef PICO_BUILD
    DBG_PRINTF("─────────────────────────────────────────────────────────────\n");
#endif

    return 0;
}
