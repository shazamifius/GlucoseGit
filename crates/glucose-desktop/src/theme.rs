//! Jetons de design et thème de l'application Glucose — la fiche 06 et `style.md` rendus
//! exécutables : chaque couleur de la chrome vit ici, et nulle part ailleurs.

use glucose_core::types::StickyOperator;
use tiny_skia::Color;

/// Échelle d'interface minimale acceptée.
pub const MIN_UI_SCALE: f32 = 0.5;
/// Échelle d'interface maximale acceptée (au-delà, aucun écran réel n'existe).
pub const MAX_UI_SCALE: f32 = 4.0;

/// Palette proposée aux domaines, **en hexadécimal**.
///
/// Elle est écrite en texte et non en [`Color`] parce que c'est ainsi que le *document* range
/// la couleur d'un domaine (`Domain::color`) : le modèle ne connaît pas tiny-skia, et ne doit
/// pas l'apprendre. Elle vit tout de même ici, parce que le thème est la seule autorité sur
/// les couleurs de l'application (standard § 4.6) et que ces huit teintes sont choisies pour
/// rester distinctes les unes des autres sur le fond sombre du canevas.
pub const DOMAIN_PALETTE: [&str; 8] = [
    "#60a5fa", // bleu
    "#34d399", // émeraude
    "#f472b6", // rose
    "#fbbf24", // ambre
    "#a78bfa", // violet
    "#f87171", // rouge
    "#22d3ee", // cyan
    "#fb923c", // orange
];

/// Sigles proposés aux domaines, dans l'ordre où le panneau les parcourt.
///
/// Trois caractères au plus : c'est ce qu'une colonne de réglette peut porter sans empiéter
/// sur sa voisine. Ils sont en capitales latines, et rien d'autre : la police embarquée
/// (`assets/font.ttf`) ne porte pas d'émoji, et un caractère qu'elle ne connaît pas se
/// rastérise en glyphe vide — un « icône » invisible est pire qu'une absence d'icône.
pub const DOMAIN_SIGILS: [&str; 8] = ["SCI", "ART", "JV", "LNG", "HIS", "TEC", "PHI", "MUS"];

/// Les six teintes des prédicats de flèche, dans l'ordre de `ArrowPredicate::ALL` (PRED-1).
///
/// Reprises de Glucose Tauri (`ArrowSvgLayer.tsx`, `PREDICATE_COLORS`) : ambre pour le
/// précurseur, rouge pour la contradiction, violet pour l'héritage, émeraude pour
/// l'inspiration, bleu pour la dépendance, rose pour l'illustration.
const PREDICATE_HEX: [u32; 6] = [
    0xf5_9e0b, 0xef_4444, 0x8b_5cf6, 0x10_b981, 0x3b_82f6, 0xf4_72b6,
];

/// Normalise un facteur d'échelle d'interface.
///
/// Toute valeur non finie ou hors plage (facteur corrompu, argument inversé)
/// produirait des panneaux de plusieurs milliers de pixels et un temps de rendu
/// non borné : la borne rend ce scénario impossible par construction.
pub fn clamp_ui_scale(scale: f32) -> f32 {
    if scale.is_nan() {
        return 1.0;
    }
    scale.clamp(MIN_UI_SCALE, MAX_UI_SCALE)
}

#[derive(Debug, Clone)]
pub struct Theme {
    // ── Surfaces (fiche 06 § 2.1) ──────────────────────────────────────────
    /// `canvas-bg` — la feuille de papier noire.
    pub bg_canvas: Color,
    /// `surface-toolbar` — la barre d'outils.
    pub bg_header: Color,
    /// Fond des panneaux du dock : `#111` dans la référence (fiche 10 § 5, `--bg-panel`).
    pub bg_panel: Color,
    /// Surface interne relevée d'un panneau — en-têtes, cartes, cellules : `#161616`.
    pub bg_card: Color,
    /// `surface-btn-hover`.
    pub bg_hover: Color,
    /// `surface-btn-active` — gris, jamais coloré : la chrome est monochrome.
    pub bg_active: Color,

    // ── Filets (§ 2.2) ─────────────────────────────────────────────────────
    /// `hairline-dark` — contour des panneaux et séparateurs d'onglets : `#222222`.
    pub border_subtle: Color,
    /// `hairline-base`.
    pub border_medium: Color,
    /// `hairline-active` — contour d'un bouton actif.
    pub border_accent: Color,

    // ── L'unique accent (§ 1.3, § 2.4) ─────────────────────────────────────
    /// Le jaune d'emphase. **Jamais sur un bouton standard, jamais en fond** : jalons,
    /// alerte douce, et rien d'autre. Un usage décoratif est une faute.
    pub accent_primary: Color,
    pub accent_muted: Color,
    pub accent_subtle: Color,

    // ── Typographie (§ 2.3) ────────────────────────────────────────────────
    /// `text-main`.
    pub text_primary: Color,
    /// `text-dim`.
    pub text_secondary: Color,
    /// `text-muted`.
    pub text_muted: Color,
    /// `text-bright` — onglet actif, icône active, sélection.
    pub text_accent: Color,

    /// `hairline-smartguide` — guides magnétiques, pointillés blancs.
    pub snap_guide: Color,
    /// `hairline-selection` — cadre de l'élément sélectionné, blanc pur à 0,80 (§ 2.2, § 4.2).
    pub selection_frame: Color,
    /// **Le rouge de l'interface** : ce qui est verrouillé, et ce qui détruit.
    ///
    /// `#f87171`, une seule valeur pour trois usages — le cadre d'un nœud verrouillé
    /// (fiche 06 § 4.3), le libellé du bouton « Verrouillé » et le survol de « Supprimer »
    /// (fiche 10). La référence n'en a qu'une pour les trois.
    ///
    /// À ne pas confondre avec [`Theme::danger`] (`#ef4444`), qui est le rouge du
    /// **contenu** : un prédicat de flèche « contredit », un fichier rompu, une couleur de
    /// preset. Deux rouges parce que la référence en a deux, et qu'ils ne parlent pas de la
    /// même chose — l'un qualifie ce que l'utilisateur a écrit, l'autre l'état d'un bouton.
    pub alert: Color,
    /// Le fond d'une action en état d'alerte : `#2a1a1a`.
    pub alert_bg: Color,
    /// Son filet : `#553333`.
    pub alert_border: Color,
    /// Sélection élastique (fiche 07 § 7.3) : contour blanc à 0,50, intérieur blanc à 0,03.
    pub rubberband_stroke: Color,
    pub rubberband_fill: Color,

    /// Poignées de redimensionnement (§ 4.2) : carré blanc pur, liseré noir à 0,90.
    pub handle_fill: Color,
    pub handle_outline: Color,

    /// Le fond de l'étiquette d'une flèche : `#111111` à 0,75 dans la référence.
    pub arrow_label_bg: Color,
    /// L'encre d'une étiquette de flèche.
    pub arrow_label_text: Color,

    /// Le fond de la pastille d'un prédicat : `#111111` à 0,90 dans la référence.
    ///
    /// Plus opaque que celui d'une étiquette, et pour une raison : le badge se pose **sur**
    /// le trait de la flèche, alors que l'étiquette se décale au-dessus. À 0,75, le trait
    /// traverse le sigle qui devrait se lire sur un aplat.
    pub arrow_badge_bg: Color,

    /// Le coude d'une flèche (ARROW-3) : l'orange de Glucose Tauri, `#ff8c00`.
    ///
    /// Une couleur à lui, et non l'accent du thème : un coude se manipule au milieu d'un
    /// trait, et il doit se distinguer et de la flèche, et de tout ce qu'elle traverse.
    pub arrow_bend: Color,

    /// Les six prédicats d'une flèche, dans l'ordre de [`ArrowPredicate`] (PRED-1).
    ///
    /// Un tableau et non six champs : ils forment **une** famille, et les parcourir dans
    /// l'ordre est ce que fait tout ce qui les propose. Les valeurs sont celles de Glucose
    /// Tauri (`ArrowSvgLayer.tsx`, `PREDICATE_COLORS`).
    pub predicates: [Color; 6],

    // ── Note adhésive (§ 5.2) ──────────────────────────────────────────────
    /// Le papier d'un pense-bête que le document ne colore pas : `#f5c542`.
    pub sticky_yellow_bg: Color,
    /// L'encre d'un pense-bête que le document ne colore pas : `#222` dans la référence.
    pub sticky_yellow_text: Color,
    /// L'ombre portée du papier : `0 4px 6px rgba(0, 0, 0, 0.3)` dans la référence — le noir
    /// à 0,30 ; le décalage est une longueur, il vit dans le rendu.
    pub sticky_shadow: Color,

    // ── Dock (fiche 10 § 4) ────────────────────────────────────────────────
    /// L'ombre portée d'un panneau au repos : `0 4px 20px rgba(0, 0, 0, 0.5)`.
    pub panel_shadow: Color,
    /// Celle d'un panneau tiré, plus dense : `0 16px 48px rgba(0, 0, 0, 0.9)`.
    pub panel_shadow_dragged: Color,

    // ── Carte de texte (§ 5.1) ────────────────────────────────────────────
    /// Un titre `#` : blanc pur, comme le texte de la référence (`color: #ffffff`).
    pub card_heading: Color,
    /// Un sous-titre `##`.
    pub card_subheading: Color,
    /// Le corps, les puces et les formules.
    pub card_body: Color,
    /// Le fond d'un texte sélectionné pendant une saisie. Bleu, et non blanc : la chrome est
    /// monochrome (fiche 06 § 1.3) mais une sélection de texte est du **contenu**, et le blanc
    /// translucide se confondrait avec l'encre du texte qu'elle doit laisser lisible.
    pub text_selection: Color,
    /// Un signe de Markdown pendant l'édition : `**`, `` ` ``, `# `. Il doit se lire sans se
    /// confondre avec le texte qu'il commande — assez pâle pour s'effacer du regard, assez
    /// présent pour qu'on vise le bon octet en le corrigeant (MODE-1).
    pub card_marker: Color,
    /// Le texte d'un `[lien](url)` : `#60a5fa`, le bleu de la référence.
    ///
    /// Un état du **contenu**, comme `text_selection` et la famille `alert*` : la chrome est
    /// monochrome, ce qui l'est ici ne l'est pas.
    pub link: Color,
    /// Le fond d'un `` `code` `` : `rgba(255, 255, 255, 0.08)`, la valeur de la référence.
    /// Le code n'a pas d'encre à lui — il hérite de celle de sa ligne, comme dans la
    /// référence, et sa chasse fixe suffit à le distinguer.
    pub code_bg: Color,

    // ── Flèches (§ 7) ──────────────────────────────────────────────────────
    /// La couleur d'une flèche que le document ne colore pas — **provisoire** : la référence
    /// la teinte du dégradé symbiotique de ses deux extrémités (§ 7.1), qui arrive avec le
    /// chantier des flèches (fiche 12, 2.B).
    pub arrow_default: Color,

    // ── Minimap (§ 9) ──────────────────────────────────────────────────────
    pub minimap_bg: Color,
    pub minimap_border: Color,
    /// Liseré du cadre de caméra : blanc à 0,35.
    pub minimap_viewport: Color,
    /// Un nœud ordinaire dans la minimap.
    pub minimap_element: Color,

    // ── Toasts (§ 10.3) ────────────────────────────────────────────────────
    pub toast_bg: Color,
    pub toast_border: Color,
    pub toast_text: Color,

    // ── Docks & boutons (§ 10.1) ───────────────────────────────────────────
    /// `text-deep` — poignée de dock au repos.
    pub dock_grip_inactive: Color,
    /// `text-muted` — glyphe de poignée `⠿⠿`.
    pub dock_grip_active: Color,
    pub input_bg: Color,
    pub btn_bg: Color,
    /// `hairline-muted`.
    pub btn_border: Color,
    pub badge_bg: Color,
    pub badge_text: Color,

    /// Accent sémantique « succès » : la fin d'un Pomodoro (fiche 10 § 1, `#4ade80` dans la
    /// référence). Vert, et seulement pour cela.
    pub success: Color,

    /// Accent sémantique « alerte / erreur / destruction » (§ 2.4) : image verrouillée,
    /// lien rompu, bouton de suppression survolé. Rouge, et seulement pour cela.
    pub danger: Color,

    /// Teinte de repli d'un domaine dont la couleur du document est illisible : un gris
    /// neutre, pour que le domaine reste visible sans que l'application choisisse une couleur.
    pub domain_fallback: Color,

    /// Les couleurs de la Time Machine.
    pub temps: CouleursDuTemps,
}

/// **L'ambre du temps** (fiche 06 § 1, fiche 10 § 5.7) : les jalons nommés, le curseur du
/// passé, et le liseré qui dit qu'on regarde un état passé. Pour cela seulement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CouleursDuTemps {
    /// `#fbbf24`.
    pub ambre: Color,
    /// Le voile ambré à l'intérieur du liseré d'aperçu (`rgba(251, 191, 36, 0.18)`).
    pub voile: Color,
}

impl CouleursDuTemps {
    fn sombres() -> Self {
        Self {
            ambre: hex(0xfbbf24),
            voile: hexa(0xfbbf24, 46),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

/// `#rrggbb` en couleur opaque — les jetons de la fiche 06 sont écrits ainsi.
fn hex(rgb: u32) -> Color {
    Color::from_rgba8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255)
}

/// `#rrggbb` avec une opacité en 0..=255.
fn hexa(rgb: u32, a: u8) -> Color {
    Color::from_rgba8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, a)
}

/// La couleur d'un opérateur logique (fiche 06 § 2.5) : le vert de la conjonction, le bleu de
/// l'alternative, l'ambre de la nuance, le lilas de la causalité. Une couleur de **contenu**,
/// pas de chrome — c'est le sens du mot qui la porte.
pub fn operator_color(op: StickyOperator) -> Color {
    match op {
        StickyOperator::And => hex(0x34d399),
        StickyOperator::Or => hex(0x60a5fa),
        StickyOperator::But => hex(0xf59e0b),
        StickyOperator::Because => hex(0xa78bfa),
    }
}

impl Theme {
    /// La couleur d'un prédicat de flèche (PRED-1).
    ///
    /// Le rang vient du modèle ([`ArrowPredicate::rank`]) : la couleur, le sigle et le
    /// raccourci désignent ainsi tous le même prédicat par le même nombre, et aucun des
    /// trois ne peut dériver tout seul.
    pub fn predicate_color(&self, predicate: glucose_core::types::ArrowPredicate) -> Color {
        self.predicates[predicate.rank()]
    }

    /// Le thème de Glucose — et il n'y en a qu'un (fiche 06, `style.md`) : une feuille de
    /// papier noire, une chrome monochrome stricte, un seul accent jaune employé avec une
    /// parcimonie extrême. **La couleur appartient au contenu de l'utilisateur.**
    ///
    /// Le thème précédent — « sombre sleek inspiré de PureRef moderne », accent bleu ciel
    /// `#38bdf8` sur chaque état actif, canevas bleuté — était une invention du port : ce
    /// bleu n'apparaît qu'une fois dans toute la référence, comme couleur de curseur
    /// multijoueur.
    pub fn dark() -> Self {
        Self {
            bg_canvas: hex(0x0d0d0d),
            bg_header: hex(0x1a1a1a),
            bg_panel: hex(0x111111),
            bg_card: hex(0x161616),
            bg_hover: hex(0x1e1e1e),
            bg_active: hex(0x2d2d2d),

            border_subtle: hex(0x222222),
            border_medium: hex(0x2a2a2a),
            border_accent: hex(0x444444),

            accent_primary: hex(0xeab308),
            accent_muted: hexa(0xeab308, 180),
            accent_subtle: hexa(0xeab308, 40),

            text_primary: hex(0xe6e6e6),
            text_secondary: hex(0xcccccc),
            text_muted: hex(0x888888),
            text_accent: hex(0xffffff),

            snap_guide: hexa(0xffffff, 77),
            selection_frame: hexa(0xffffff, 204),
            alert: hex(0xf87171),
            alert_bg: hex(0x2a1a1a),
            alert_border: hex(0x553333),
            rubberband_stroke: hexa(0xffffff, 128),
            rubberband_fill: hexa(0xffffff, 8),

            arrow_label_bg: Color::from_rgba8(0x11, 0x11, 0x11, 191),
            arrow_label_text: hex(0xe8e8ea),
            arrow_badge_bg: Color::from_rgba8(0x11, 0x11, 0x11, 230),
            arrow_bend: hex(0xff_8c00),
            predicates: PREDICATE_HEX.map(hex),
            handle_fill: hex(0xffffff),
            handle_outline: hexa(0x111111, 230),

            sticky_yellow_bg: hex(0xf5c542),
            sticky_yellow_text: hex(0x222222),
            sticky_shadow: hexa(0x000000, 77),

            panel_shadow: hexa(0x000000, 128),
            panel_shadow_dragged: hexa(0x000000, 230),

            card_heading: hex(0xffffff),
            card_subheading: hex(0xf0f0f5),
            card_body: hex(0xdce1eb),
            card_marker: hex(0x7a8296),
            text_selection: Color::from_rgba8(74, 127, 181, 115),
            link: hex(0x60a5fa),
            code_bg: Color::from_rgba8(255, 255, 255, 20),

            arrow_default: hexa(0x94a3b8, 220),

            minimap_bg: hexa(0x0d0d0d, 235),
            minimap_border: hex(0x2a2a2a),
            minimap_viewport: hexa(0xffffff, 89),
            minimap_element: hex(0x2a2a2a),

            toast_bg: hexa(0x1a1a1a, 247),
            toast_border: hex(0x2a2a2a),
            toast_text: hex(0xcccccc),

            dock_grip_inactive: hex(0x333333),
            dock_grip_active: hex(0x888888),
            input_bg: hex(0x111111),
            btn_bg: hex(0x1a1a1a),
            btn_border: hex(0x333333),
            badge_bg: hex(0x1e1e1e),
            badge_text: hex(0x888888),

            success: hex(0x4ade80),
            danger: hex(0xef4444),

            domain_fallback: hex(0x888888),
            temps: CouleursDuTemps::sombres(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_ui_scale_rejects_absurd_factors() {
        assert_eq!(clamp_ui_scale(1.25), 1.25);
        assert_eq!(clamp_ui_scale(0.0), MIN_UI_SCALE);
        assert_eq!(clamp_ui_scale(-3.0), MIN_UI_SCALE);
        assert_eq!(clamp_ui_scale(170.0), MAX_UI_SCALE);
        assert_eq!(clamp_ui_scale(f32::NAN), 1.0);
        assert_eq!(clamp_ui_scale(f32::INFINITY), MAX_UI_SCALE);
    }

    fn rgba(c: Color) -> (u8, u8, u8, u8) {
        let c = c.to_color_u8();
        (c.red(), c.green(), c.blue(), c.alpha())
    }

    /// Fiche 06 § 2 — chaque jeton à sa valeur exacte. Ce test remplace celui qui
    /// verrouillait l'accent bleu `#38bdf8` et le canevas bleuté `(13, 14, 18)`.
    #[test]
    fn test_the_theme_is_the_brutalist_palette_of_the_spec() {
        let t = Theme::dark();
        // § 2.1 surfaces
        assert_eq!(rgba(t.bg_canvas), (0x0d, 0x0d, 0x0d, 255), "canvas-bg");
        assert_eq!(
            rgba(t.bg_header),
            (0x1a, 0x1a, 0x1a, 255),
            "surface-toolbar"
        );
        assert_eq!(
            rgba(t.bg_panel),
            (0x11, 0x11, 0x11, 255),
            "--bg-panel (fiche 10)"
        );
        assert_eq!(
            rgba(t.bg_card),
            (0x16, 0x16, 0x16, 255),
            "surface interne relevée"
        );
        assert_eq!(
            rgba(t.bg_hover),
            (0x1e, 0x1e, 0x1e, 255),
            "surface-btn-hover"
        );
        assert_eq!(
            rgba(t.bg_active),
            (0x2d, 0x2d, 0x2d, 255),
            "surface-btn-active"
        );
        assert_eq!(
            rgba(t.toast_bg),
            (0x1a, 0x1a, 0x1a, 247),
            "surface-toast 0.97"
        );
        assert_eq!(
            rgba(t.minimap_bg),
            (0x0d, 0x0d, 0x0d, 235),
            "surface-minimap 0.92"
        );
        // § 2.2 filets
        assert_eq!(
            rgba(t.border_subtle),
            (0x22, 0x22, 0x22, 255),
            "--border-panel (fiche 10)"
        );
        assert_eq!(
            rgba(t.border_medium),
            (0x2a, 0x2a, 0x2a, 255),
            "hairline-base"
        );
        assert_eq!(
            rgba(t.btn_border),
            (0x33, 0x33, 0x33, 255),
            "hairline-muted"
        );
        assert_eq!(
            rgba(t.border_accent),
            (0x44, 0x44, 0x44, 255),
            "hairline-active"
        );
        assert_eq!(
            rgba(t.snap_guide),
            (255, 255, 255, 77),
            "hairline-smartguide 0.30"
        );
        assert_eq!(
            rgba(t.selection_frame),
            (255, 255, 255, 204),
            "hairline-selection 0.80"
        );
        // Fiche 06 § 4.3 — le cadre d'un nœud verrouillé, et le même rouge que l'état
        // « Verrouillé » de la barre d'action (fiche 10).
        assert_eq!(rgba(t.link), (0x60, 0xa5, 0xfa, 255), "link #60a5fa");
        assert_eq!(rgba(t.alert), (0xf8, 0x71, 0x71, 255), "alert #f87171");
        assert_eq!(
            rgba(t.alert_bg),
            (0x2a, 0x1a, 0x1a, 255),
            "alert-bg #2a1a1a"
        );
        assert_eq!(
            rgba(t.alert_border),
            (0x55, 0x33, 0x33, 255),
            "alert-border #553333"
        );
        assert_ne!(
            rgba(t.alert),
            rgba(t.danger),
            "deux rouges, deux rôles : l'interface et le contenu"
        );
        assert_eq!(
            rgba(t.rubberband_stroke),
            (255, 255, 255, 128),
            "lasso 0.50"
        );
        assert_eq!(rgba(t.rubberband_fill), (255, 255, 255, 8), "lasso 0.03");
        // § 2.3 typographie
        assert_eq!(rgba(t.text_accent), (255, 255, 255, 255), "text-bright");
        assert_eq!(rgba(t.text_primary), (0xe6, 0xe6, 0xe6, 255), "text-main");
        assert_eq!(rgba(t.text_secondary), (0xcc, 0xcc, 0xcc, 255), "text-dim");
        assert_eq!(rgba(t.text_muted), (0x88, 0x88, 0x88, 255), "text-muted");
        assert_eq!(
            rgba(t.dock_grip_inactive),
            (0x33, 0x33, 0x33, 255),
            "text-deep"
        );
        // § 2.4 l'unique accent
        assert_eq!(
            rgba(t.accent_primary),
            (0xea, 0xb3, 0x08, 255),
            "le jaune, et rien d'autre"
        );
        assert_eq!(
            rgba(t.danger),
            (0xef, 0x44, 0x44, 255),
            "alerte / erreur / destruction"
        );
        assert_eq!(
            rgba(t.success),
            (0x4a, 0xde, 0x80, 255),
            "validation Pomodoro"
        );
        // § 4.2 poignées, § 5.2 note, § 9 minimap, § 10.3 toast
        assert_eq!(rgba(t.handle_fill), (255, 255, 255, 255));
        assert_eq!(
            rgba(t.handle_outline),
            (0x11, 0x11, 0x11, 230),
            "liseré noir à 0,90"
        );
        assert_eq!(
            rgba(t.sticky_yellow_bg),
            (0xf5, 0xc5, 0x42, 255),
            "le papier du pense-bête"
        );
        assert_eq!(
            rgba(t.sticky_yellow_text),
            (0x22, 0x22, 0x22, 255),
            "l'encre du pense-bête, celle de la référence"
        );
        assert_eq!(
            rgba(t.sticky_shadow),
            (0, 0, 0, 77),
            "l'ombre du papier à 0,30"
        );
        assert_eq!(rgba(t.minimap_border), (0x2a, 0x2a, 0x2a, 255));
        assert_eq!(
            rgba(t.minimap_viewport),
            (255, 255, 255, 89),
            "cadre de caméra à 0,35"
        );
        assert_eq!(
            rgba(t.minimap_element),
            (0x2a, 0x2a, 0x2a, 255),
            "image ordinaire"
        );
        assert_eq!(rgba(t.toast_border), (0x2a, 0x2a, 0x2a, 255));
        assert_eq!(rgba(t.toast_text), (0xcc, 0xcc, 0xcc, 255));
    }

    /// La loi de la couleur : la chrome est monochrome. Tout jeton de surface, de filet, de
    /// texte ou de bouton a ses trois composantes égales — hors le seul accent.
    ///
    /// N'y figurent pas les jetons qui colorent un **état** plutôt qu'une surface :
    /// `text_selection` et `link` (bleus), et la famille `alert*` (rouge). Ce ne sont pas des exceptions
    /// arbitraires : les fiches 06 et 10 leur donnent leur valeur explicitement, parce qu'un
    /// état qui doit se lire d'un coup d'œil ne peut pas le faire en gris parmi des gris.
    #[test]
    fn test_every_chrome_token_is_a_neutral_grey() {
        let t = Theme::dark();
        let chrome = [
            ("bg_canvas", t.bg_canvas),
            ("bg_header", t.bg_header),
            ("bg_panel", t.bg_panel),
            ("bg_card", t.bg_card),
            ("bg_hover", t.bg_hover),
            ("bg_active", t.bg_active),
            ("border_subtle", t.border_subtle),
            ("border_medium", t.border_medium),
            ("border_accent", t.border_accent),
            ("text_primary", t.text_primary),
            ("text_secondary", t.text_secondary),
            ("text_muted", t.text_muted),
            ("text_accent", t.text_accent),
            ("snap_guide", t.snap_guide),
            ("selection_frame", t.selection_frame),
            ("rubberband_stroke", t.rubberband_stroke),
            ("rubberband_fill", t.rubberband_fill),
            ("handle_fill", t.handle_fill),
            ("handle_outline", t.handle_outline),
            ("minimap_bg", t.minimap_bg),
            ("minimap_border", t.minimap_border),
            ("minimap_viewport", t.minimap_viewport),
            ("minimap_element", t.minimap_element),
            ("toast_bg", t.toast_bg),
            ("toast_border", t.toast_border),
            ("toast_text", t.toast_text),
            ("dock_grip_inactive", t.dock_grip_inactive),
            ("dock_grip_active", t.dock_grip_active),
            ("input_bg", t.input_bg),
            ("btn_bg", t.btn_bg),
            ("btn_border", t.btn_border),
            ("badge_bg", t.badge_bg),
            ("badge_text", t.badge_text),
            ("domain_fallback", t.domain_fallback),
        ];
        for (name, c) in chrome {
            let (r, g, b, _) = rgba(c);
            assert!(
                r == g && g == b,
                "{name} n'est pas un gris neutre : ({r}, {g}, {b})"
            );
        }
    }

    /// Fiche 10 § 5.4 — les huit couleurs proposées aux domaines sont celles de la référence
    /// (`PRESET_COLORS` de `DomainsPanel.tsx`), dans le même ordre : le premier domaine créé
    /// ici a la couleur du premier domaine créé là-bas.
    #[test]
    fn test_the_domain_palette_is_that_of_the_reference() {
        assert_eq!(
            DOMAIN_PALETTE,
            [
                "#60a5fa", "#34d399", "#f472b6", "#fbbf24", "#a78bfa", "#f87171", "#22d3ee",
                "#fb923c"
            ]
        );
    }

    /// § 4.6 — la palette des domaines vit dans le thème, et chacune de ses entrées est un
    /// hexadécimal que le modèle peut ranger tel quel dans `Domain::color`.
    #[test]
    fn test_domain_palette_entries_are_readable_hex_colours() {
        let mut seen = Vec::new();
        for hex in DOMAIN_PALETTE {
            assert_eq!(hex.len(), 7, "{hex} n'est pas un #RRGGBB");
            assert!(hex.starts_with('#'), "{hex}");
            assert!(hex[1..].chars().all(|c| c.is_ascii_hexdigit()), "{hex}");
            assert!(
                !seen.contains(&hex),
                "{hex} apparaît deux fois dans la palette"
            );
            seen.push(hex);
        }
    }
}

#[cfg(test)]
mod operator_tests {
    use super::*;

    fn rgb(c: Color) -> (u8, u8, u8) {
        let c = c.to_color_u8();
        (c.red(), c.green(), c.blue())
    }

    /// Fiche 06 § 2.5 — chaque opérateur a sa couleur, et elles sont distinctes.
    #[test]
    fn test_operator_colors_are_those_of_the_spec_and_distinct() {
        assert_eq!(rgb(operator_color(StickyOperator::And)), (0x34, 0xd3, 0x99));
        assert_eq!(rgb(operator_color(StickyOperator::Or)), (0x60, 0xa5, 0xfa));
        assert_eq!(rgb(operator_color(StickyOperator::But)), (0xf5, 0x9e, 0x0b));
        assert_eq!(
            rgb(operator_color(StickyOperator::Because)),
            (0xa7, 0x8b, 0xfa)
        );
    }
}
