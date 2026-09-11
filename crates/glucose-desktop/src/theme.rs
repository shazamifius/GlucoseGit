//! Jetons de design unifiés et thème de l'application Glucose (Roadmap 1.24, R-31).
//! Centralise les littéraux de couleur et les dimensions pour assurer la cohérence visuelle.

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
    // Toile & Arrière-plans
    pub bg_canvas: Color,
    pub bg_header: Color,
    pub bg_panel: Color,
    pub bg_card: Color,
    pub bg_hover: Color,
    pub bg_active: Color,

    // Bordures
    pub border_subtle: Color,
    pub border_medium: Color,
    pub border_accent: Color,

    // Accent (PureRef cyan / sky)
    pub accent_primary: Color,
    pub accent_muted: Color,
    pub accent_subtle: Color,

    // Typographie
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_accent: Color,

    // Guides magnétiques
    pub snap_guide: Color,

    // Post-it / Sticky notes
    pub sticky_yellow_bg: Color,
    pub sticky_yellow_text: Color,
    pub sticky_yellow_border: Color,

    // Minimap
    pub minimap_bg: Color,
    pub minimap_border: Color,
    pub minimap_viewport: Color,
    pub minimap_element: Color,

    // Notifications Toasts
    pub toast_bg: Color,
    pub toast_border: Color,
    pub toast_text: Color,

    // Docks & Boutons
    pub dock_grip_inactive: Color,
    pub dock_grip_active: Color,
    pub input_bg: Color,
    pub btn_bg: Color,
    pub btn_border: Color,
    pub badge_bg: Color,
    pub badge_text: Color,

    /// Teinte de repli d'un domaine dont la couleur du document est illisible.
    /// Un domaine sans teinte lisible reste visible plutôt que de disparaître.
    pub domain_fallback: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Thème sombre sleek inspiré de PureRef moderne
    pub fn dark() -> Self {
        Self {
            bg_canvas: Color::from_rgba8(13, 14, 18, 255),
            bg_header: Color::from_rgba8(20, 21, 26, 250),
            bg_panel: Color::from_rgba8(24, 24, 28, 240),
            bg_card: Color::from_rgba8(24, 24, 27, 248),
            bg_hover: Color::from_rgba8(39, 39, 42, 255),
            bg_active: Color::from_rgba8(56, 189, 248, 45),

            border_subtle: Color::from_rgba8(40, 42, 50, 255),
            border_medium: Color::from_rgba8(60, 65, 75, 255),
            border_accent: Color::from_rgba8(56, 189, 248, 200),

            accent_primary: Color::from_rgba8(56, 189, 248, 255),
            accent_muted: Color::from_rgba8(56, 189, 248, 180),
            accent_subtle: Color::from_rgba8(56, 189, 248, 40),

            text_primary: Color::from_rgba8(245, 245, 245, 255),
            text_secondary: Color::from_rgba8(161, 161, 170, 255),
            text_muted: Color::from_rgba8(113, 113, 122, 255),
            text_accent: Color::from_rgba8(56, 189, 248, 255),

            snap_guide: Color::from_rgba8(236, 72, 153, 200),

            sticky_yellow_bg: Color::from_rgba8(254, 240, 138, 245),
            sticky_yellow_text: Color::from_rgba8(28, 25, 23, 255),
            sticky_yellow_border: Color::from_rgba8(202, 138, 4, 180),

            minimap_bg: Color::from_rgba8(15, 16, 20, 220),
            minimap_border: Color::from_rgba8(45, 48, 58, 200),
            minimap_viewport: Color::from_rgba8(56, 189, 248, 200),
            minimap_element: Color::from_rgba8(100, 110, 130, 200),

            toast_bg: Color::from_rgba8(24, 24, 27, 245),
            toast_border: Color::from_rgba8(56, 189, 248, 120),
            toast_text: Color::from_rgba8(245, 245, 245, 255),

            dock_grip_inactive: Color::from_rgba8(75, 75, 80, 255),
            dock_grip_active: Color::from_rgba8(160, 160, 160, 255),
            input_bg: Color::from_rgba8(26, 26, 30, 255),
            btn_bg: Color::from_rgba8(30, 30, 34, 255),
            btn_border: Color::from_rgba8(50, 52, 60, 255),
            badge_bg: Color::from_rgba8(30, 30, 34, 255),
            badge_text: Color::from_rgba8(160, 160, 170, 255),

            domain_fallback: Color::from_rgba8(148, 163, 184, 255),
        }
    }

    /// Thème clair optionnel pour la future personnalisation
    pub fn light() -> Self {
        Self {
            bg_canvas: Color::from_rgba8(245, 245, 248, 255),
            bg_header: Color::from_rgba8(255, 255, 255, 250),
            bg_panel: Color::from_rgba8(250, 250, 252, 245),
            bg_card: Color::from_rgba8(255, 255, 255, 250),
            bg_hover: Color::from_rgba8(235, 235, 240, 255),
            bg_active: Color::from_rgba8(14, 165, 233, 40),

            border_subtle: Color::from_rgba8(220, 222, 230, 255),
            border_medium: Color::from_rgba8(195, 200, 210, 255),
            border_accent: Color::from_rgba8(14, 165, 233, 220),

            accent_primary: Color::from_rgba8(14, 165, 233, 255),
            accent_muted: Color::from_rgba8(14, 165, 233, 180),
            accent_subtle: Color::from_rgba8(14, 165, 233, 35),

            text_primary: Color::from_rgba8(24, 24, 27, 255),
            text_secondary: Color::from_rgba8(82, 82, 91, 255),
            text_muted: Color::from_rgba8(140, 140, 150, 255),
            text_accent: Color::from_rgba8(14, 165, 233, 255),

            snap_guide: Color::from_rgba8(219, 39, 119, 200),

            sticky_yellow_bg: Color::from_rgba8(254, 240, 138, 245),
            sticky_yellow_text: Color::from_rgba8(28, 25, 23, 255),
            sticky_yellow_border: Color::from_rgba8(202, 138, 4, 180),

            minimap_bg: Color::from_rgba8(240, 242, 248, 220),
            minimap_border: Color::from_rgba8(200, 205, 215, 200),
            minimap_viewport: Color::from_rgba8(14, 165, 233, 200),
            minimap_element: Color::from_rgba8(160, 170, 185, 200),

            toast_bg: Color::from_rgba8(255, 255, 255, 245),
            toast_border: Color::from_rgba8(14, 165, 233, 140),
            toast_text: Color::from_rgba8(24, 24, 27, 255),

            dock_grip_inactive: Color::from_rgba8(180, 180, 185, 255),
            dock_grip_active: Color::from_rgba8(100, 100, 105, 255),
            input_bg: Color::from_rgba8(240, 240, 245, 255),
            btn_bg: Color::from_rgba8(235, 235, 240, 255),
            btn_border: Color::from_rgba8(210, 212, 220, 255),
            badge_bg: Color::from_rgba8(230, 230, 235, 255),
            badge_text: Color::from_rgba8(80, 80, 90, 255),

            domain_fallback: Color::from_rgba8(100, 116, 139, 255),
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

    #[test]
    fn test_theme_tokens_valid() {
        let theme = Theme::dark();
        assert_eq!(theme.accent_primary, Color::from_rgba8(56, 189, 248, 255));
        assert_eq!(theme.bg_canvas, Color::from_rgba8(13, 14, 18, 255));

        let light = Theme::light();
        assert_eq!(light.bg_canvas, Color::from_rgba8(245, 245, 248, 255));
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
