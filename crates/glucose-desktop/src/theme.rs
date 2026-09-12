//! Jetons de design et thème de l'application Glucose — la fiche 06 et `style.md` rendus
//! exécutables : chaque couleur de la chrome vit ici, et nulle part ailleurs.

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
    "#38bdf8", // ciel
    "#34d399", // émeraude
    "#f472b6", // rose
    "#fbbf24", // ambre
    "#a78bfa", // violet
    "#fb7185", // corail
    "#4ade80", // vert
    "#f0abfc", // orchidée
];

/// Sigles proposés aux domaines, dans l'ordre où le panneau les parcourt.
///
/// Trois caractères au plus : c'est ce qu'une colonne de réglette peut porter sans empiéter
/// sur sa voisine. Ils sont en capitales latines, et rien d'autre : la police embarquée
/// (`assets/font.ttf`) ne porte pas d'émoji, et un caractère qu'elle ne connaît pas se
/// rastérise en glyphe vide — un « icône » invisible est pire qu'une absence d'icône.
pub const DOMAIN_SIGILS: [&str; 8] = ["SCI", "ART", "JV", "LNG", "HIS", "TEC", "PHI", "MUS"];

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
    /// `surface-panel` — les panneaux du dock.
    pub bg_panel: Color,
    /// `surface-root` — conteneurs sombres, barre d'onglets, cellules.
    pub bg_card: Color,
    /// `surface-btn-hover`.
    pub bg_hover: Color,
    /// `surface-btn-active` — gris, jamais coloré : la chrome est monochrome.
    pub bg_active: Color,

    // ── Filets (§ 2.2) ─────────────────────────────────────────────────────
    /// `hairline-dark`.
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
    /// Sélection élastique (fiche 07 § 7.3) : contour blanc à 0,50, intérieur blanc à 0,03.
    pub rubberband_stroke: Color,
    pub rubberband_fill: Color,

    /// Poignées de redimensionnement (§ 4.2) : carré blanc pur, liseré noir à 0,90.
    pub handle_fill: Color,
    pub handle_outline: Color,

    // ── Note adhésive (§ 5.2) ──────────────────────────────────────────────
    pub sticky_yellow_bg: Color,
    pub sticky_yellow_text: Color,
    pub sticky_yellow_border: Color,

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

    /// Accent sémantique « alerte / erreur / destruction » (§ 2.4) : image verrouillée,
    /// lien rompu, bouton de suppression survolé. Rouge, et seulement pour cela.
    pub danger: Color,

    /// Teinte de repli d'un domaine dont la couleur du document est illisible : un gris
    /// neutre, pour que le domaine reste visible sans que l'application choisisse une couleur.
    pub domain_fallback: Color,
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

impl Theme {
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
            bg_panel: hex(0x161616),
            bg_card: hex(0x111111),
            bg_hover: hex(0x1e1e1e),
            bg_active: hex(0x2d2d2d),

            border_subtle: hex(0x1c1c1c),
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
            rubberband_stroke: hexa(0xffffff, 128),
            rubberband_fill: hexa(0xffffff, 8),

            handle_fill: hex(0xffffff),
            handle_outline: hexa(0x111111, 230),

            sticky_yellow_bg: hex(0xf5c542),
            sticky_yellow_text: hex(0x1c1917),
            sticky_yellow_border: hex(0xf5c542),

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

            danger: hex(0xef4444),

            domain_fallback: hex(0x888888),
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
        assert_eq!(rgba(t.bg_header), (0x1a, 0x1a, 0x1a, 255), "surface-toolbar");
        assert_eq!(rgba(t.bg_panel), (0x16, 0x16, 0x16, 255), "surface-panel");
        assert_eq!(rgba(t.bg_card), (0x11, 0x11, 0x11, 255), "surface-root");
        assert_eq!(rgba(t.bg_hover), (0x1e, 0x1e, 0x1e, 255), "surface-btn-hover");
        assert_eq!(rgba(t.bg_active), (0x2d, 0x2d, 0x2d, 255), "surface-btn-active");
        assert_eq!(rgba(t.toast_bg), (0x1a, 0x1a, 0x1a, 247), "surface-toast 0.97");
        assert_eq!(rgba(t.minimap_bg), (0x0d, 0x0d, 0x0d, 235), "surface-minimap 0.92");
        // § 2.2 filets
        assert_eq!(rgba(t.border_subtle), (0x1c, 0x1c, 0x1c, 255), "hairline-dark");
        assert_eq!(rgba(t.border_medium), (0x2a, 0x2a, 0x2a, 255), "hairline-base");
        assert_eq!(rgba(t.btn_border), (0x33, 0x33, 0x33, 255), "hairline-muted");
        assert_eq!(rgba(t.border_accent), (0x44, 0x44, 0x44, 255), "hairline-active");
        assert_eq!(rgba(t.snap_guide), (255, 255, 255, 77), "hairline-smartguide 0.30");
        assert_eq!(rgba(t.selection_frame), (255, 255, 255, 204), "hairline-selection 0.80");
        assert_eq!(rgba(t.rubberband_stroke), (255, 255, 255, 128), "lasso 0.50");
        assert_eq!(rgba(t.rubberband_fill), (255, 255, 255, 8), "lasso 0.03");
        // § 2.3 typographie
        assert_eq!(rgba(t.text_accent), (255, 255, 255, 255), "text-bright");
        assert_eq!(rgba(t.text_primary), (0xe6, 0xe6, 0xe6, 255), "text-main");
        assert_eq!(rgba(t.text_secondary), (0xcc, 0xcc, 0xcc, 255), "text-dim");
        assert_eq!(rgba(t.text_muted), (0x88, 0x88, 0x88, 255), "text-muted");
        assert_eq!(rgba(t.dock_grip_inactive), (0x33, 0x33, 0x33, 255), "text-deep");
        // § 2.4 l'unique accent
        assert_eq!(rgba(t.accent_primary), (0xea, 0xb3, 0x08, 255), "le jaune, et rien d'autre");
        assert_eq!(rgba(t.danger), (0xef, 0x44, 0x44, 255), "alerte / erreur / destruction");
        // § 4.2 poignées, § 5.2 note, § 9 minimap, § 10.3 toast
        assert_eq!(rgba(t.handle_fill), (255, 255, 255, 255));
        assert_eq!(rgba(t.handle_outline), (0x11, 0x11, 0x11, 230), "liseré noir à 0,90");
        assert_eq!(rgba(t.sticky_yellow_bg), (0xf5, 0xc5, 0x42, 255), "jaune pastel");
        assert_eq!(rgba(t.minimap_border), (0x2a, 0x2a, 0x2a, 255));
        assert_eq!(rgba(t.minimap_viewport), (255, 255, 255, 89), "cadre de caméra à 0,35");
        assert_eq!(rgba(t.minimap_element), (0x2a, 0x2a, 0x2a, 255), "image ordinaire");
        assert_eq!(rgba(t.toast_border), (0x2a, 0x2a, 0x2a, 255));
        assert_eq!(rgba(t.toast_text), (0xcc, 0xcc, 0xcc, 255));
    }

    /// La loi de la couleur : la chrome est monochrome. Tout jeton de surface, de filet, de
    /// texte ou de bouton a ses trois composantes égales — hors le seul accent.
    #[test]
    fn test_every_chrome_token_is_a_neutral_grey() {
        let t = Theme::dark();
        let chrome = [
            ("bg_canvas", t.bg_canvas), ("bg_header", t.bg_header), ("bg_panel", t.bg_panel),
            ("bg_card", t.bg_card), ("bg_hover", t.bg_hover), ("bg_active", t.bg_active),
            ("border_subtle", t.border_subtle), ("border_medium", t.border_medium),
            ("border_accent", t.border_accent), ("text_primary", t.text_primary),
            ("text_secondary", t.text_secondary), ("text_muted", t.text_muted),
            ("text_accent", t.text_accent), ("snap_guide", t.snap_guide),
            ("selection_frame", t.selection_frame), ("rubberband_stroke", t.rubberband_stroke),
            ("rubberband_fill", t.rubberband_fill),
            ("handle_fill", t.handle_fill), ("handle_outline", t.handle_outline),
            ("minimap_bg", t.minimap_bg), ("minimap_border", t.minimap_border),
            ("minimap_viewport", t.minimap_viewport), ("minimap_element", t.minimap_element),
            ("toast_bg", t.toast_bg), ("toast_border", t.toast_border), ("toast_text", t.toast_text),
            ("dock_grip_inactive", t.dock_grip_inactive), ("dock_grip_active", t.dock_grip_active),
            ("input_bg", t.input_bg), ("btn_bg", t.btn_bg), ("btn_border", t.btn_border),
            ("badge_bg", t.badge_bg), ("badge_text", t.badge_text), ("domain_fallback", t.domain_fallback),
        ];
        for (name, c) in chrome {
            let (r, g, b, _) = rgba(c);
            assert!(r == g && g == b, "{name} n'est pas un gris neutre : ({r}, {g}, {b})");
        }
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
            assert!(!seen.contains(&hex), "{hex} apparaît deux fois dans la palette");
            seen.push(hex);
        }
    }
}
