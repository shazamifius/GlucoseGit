//! Le panneau PRESETS : les quatre gabarits de la référence, avec leurs zones colorées.
//!
//! # Ce qui est vrai, et ce qui ne l'est pas encore
//!
//! Les gabarits sont ceux de la référence (`defaultPresets`, fiche 03 § 14.1), et ce sont des
//! **données**, déclarées ici et non recopiées dans le dessin. Le noyau sait poser les zones
//! d'un preset sur un tableau (`Store::apply_preset_to_board`, journalisé) — mais aucun clic
//! ne l'appelle encore : cliquer un gabarit ne fait rien, et le bouton de création non plus
//! (fiche 10 § 5.5, fiche 12 chantier 3.G). Le panneau ne prétend pas le contraire : il n'a
//! pas de bouton « Appliquer », et son bouton de création est grisé.

use super::WidgetRect;
use crate::params::ScaledRect;
use tiny_skia::Color;

pub mod paint;

/// Une zone d'un gabarit : son sigle et sa couleur — une couleur de contenu, celle d'une
/// zone posée sur le tableau, pas de la chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresetSlot {
    pub label: &'static str,
    /// `#rrggbb` — la couleur se construit au dessin, où elle reçoit son opacité.
    pub rgb: u32,
}

impl PresetSlot {
    /// L'opacité d'une zone sur le tableau, sur 255.
    const ALPHA: u8 = 200;

    pub fn color(self) -> Color {
        let rgb = self.rgb;
        Color::from_rgba8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, Self::ALPHA)
    }
}

/// Un gabarit de tableau.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PresetSpec {
    pub name: &'static str,
    pub summary: &'static str,
    pub slots: &'static [PresetSlot],
}

const fn slot(label: &'static str, rgb: u32) -> PresetSlot {
    PresetSlot { label, rgb }
}

/// Les quatre gabarits de la référence.
pub const PRESETS: [PresetSpec; 4] = [
    PresetSpec {
        name: "CharaDesign",
        summary: "Workflow complet de création de personnage",
        slots: &[
            slot("Réf", 0x3b82f6),
            slot("Sketch", 0x8b5cf6),
            slot("Lineart", 0xa855f7),
            slot("Face", 0x10b981),
            slot("3/4", 0x14b8a6),
            slot("Profil", 0x34d399),
            slot("Dos", 0xf59e0b),
            slot("Poses", 0xef4444),
            slot("Final", 0x6b7280),
        ],
    },
    PresetSpec {
        name: "Environment",
        summary: "Conception d'environnement et décors",
        slots: &[
            slot("Réf", 0x3b82f6),
            slot("Mood", 0x6366f1),
            slot("Thumb", 0x10b981),
            slot("Layout", 0x14b8a6),
            slot("Détails", 0xf59e0b),
            slot("Lumière", 0xfbbf24),
            slot("Final", 0x6b7280),
        ],
    },
    PresetSpec {
        name: "Creature Design",
        summary: "Conception de créature / monstre",
        slots: &[
            slot("Réf", 0x3b82f6),
            slot("Silh", 0x8b5cf6),
            slot("Anatom", 0xef4444),
            slot("Textur", 0x10b981),
            slot("Turn", 0x14b8a6),
            slot("Action", 0xf59e0b),
            slot("Final", 0x6b7280),
        ],
    },
    PresetSpec {
        name: "Props & Items",
        summary: "Design d'objets, armes, accessoires",
        slots: &[
            slot("Réf", 0x3b82f6),
            slot("Sketch", 0x8b5cf6),
            slot("Ortho", 0x10b981),
            slot("Détails", 0xf59e0b),
            slot("Final", 0x6b7280),
        ],
    },
];

/// Le bouton de création, grisé tant que créer un gabarit n'existe pas.
pub const CREATE_LABEL: &str = "+ Créer un preset custom";

#[derive(Debug, Clone, Default)]
pub struct PresetsState {
    pub active_preset: Option<String>,
}

/// Un gabarit dans le panneau : son titre, la rangée de ses zones, son résumé.
#[derive(Debug, Clone)]
pub struct PresetRowLayout {
    pub index: usize,
    pub title_at: (f32, f32),
    pub slots: Vec<WidgetRect>,
    pub summary_at: (f32, f32),
}

#[derive(Debug, Clone)]
pub struct PresetsPanelLayout {
    pub caption_at: (f32, f32),
    pub rows: Vec<PresetRowLayout>,
    pub create_button: WidgetRect,
}

/// La géométrie du panneau — la seule, lue par le dessin et par le clic.
pub fn layout_presets_panel(frame: ScaledRect) -> PresetsPanelLayout {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (px, py, pw) = (frame.x, frame.y, frame.w);
    let x = px + 14.0 * s;
    let inner_w = pw - 28.0 * s;
    let caption_at = (x, py + 38.0 * s);
    let mut cy = caption_at.1 + 16.0 * s;

    let mut rows = Vec::with_capacity(PRESETS.len());
    for (index, preset) in PRESETS.iter().enumerate() {
        let title_at = (x, cy);
        cy += 14.0 * s;
        let n = preset.slots.len() as f32;
        let gap = 2.0 * s;
        let slot_w = (inner_w - (n - 1.0) * gap) / n;
        let slot_h = 32.0 * s;
        let slots = (0..preset.slots.len())
            .map(|i| WidgetRect::new(x + i as f32 * (slot_w + gap), cy, slot_w, slot_h))
            .collect();
        cy += slot_h + 3.0 * s;
        let summary_at = (x, cy);
        cy += 16.0 * s;
        rows.push(PresetRowLayout {
            index,
            title_at,
            slots,
            summary_at,
        });
    }
    let create_button = WidgetRect::new(x, cy, inner_w, 24.0 * s);

    PresetsPanelLayout {
        caption_at,
        rows,
        create_button,
    }
}
