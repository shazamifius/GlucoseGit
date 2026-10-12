//! Le panneau PRESETS : les gabarits qu'on pose sur un tableau, et celui qu'il suit (PRESET-1,
//! fiche 62).
//!
//! # Ce qu'il montre — celui de Glucose Tauri (`PresetPanel.tsx`)
//!
//! * **Le gabarit que suit le tableau**, s'il en suit un : son nom, un bouton « Retirer », et une
//!   pastille par catégorie — pleine dès qu'une image est posée dans sa zone. Chez Tauri, la
//!   pastille lisait un `slotId` qu'aucune ligne n'écrivait : elle ne s'allumait jamais. Ici
//!   l'appartenance se lit dans la géométrie (`Store::zone_de`).
//! * **Les gabarits à poser** : les six de Tauri, puis ceux du document — leur nom, la rangée de
//!   leurs catégories en couleur, leur phrase. Un clic **arme** le gabarit : le panneau se
//!   ferme, le gabarit suit le curseur, un clic sur le canevas le pose
//!   ([`crate::interactions::gabarit`]).
//! * « Créer un preset custom », grisé : Glucose Tauri n'en faisait rien non plus — un message
//!   « bientôt ». Il reste à concevoir.
//!
//! # Une seule géométrie (loi L4)
//!
//! [`layout_presets_panel`] produit la liste des rectangles ; le dessin la lit, le clic la lit.
//! Le panneau a la hauteur de ce qu'il peut montrer de plus grand ([`hauteur_naturelle`]),
//! mesurée sur la même mise en page — aucune hauteur écrite à la main.

use super::WidgetRect;
use crate::params::ScaledRect;
use glucose_core::gabarits::{a_choisir, suivi, Choix};
use glucose_core::store::Store;

pub mod paint;

/// Le bouton de création, grisé tant que créer un gabarit n'existe pas.
pub const CREATE_LABEL: &str = "+ Créer un preset custom";

/// Combien de pastilles par rangée dans la section du gabarit suivi.
const PASTILLES_PAR_RANGEE: usize = 3;

/// Une catégorie du gabarit suivi : son nom, sa couleur, et si une image est posée dans sa
/// zone.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pastille {
    pub nom: String,
    pub couleur: u32,
    pub remplie: bool,
}

/// **Ce que le panneau montre**, lu dans le document : le tableau, le gabarit qu'il suit et
/// l'état de ses catégories, les gabarits à choisir.
#[derive(Debug, Clone)]
pub struct Contenu<'a> {
    pub tableau: String,
    /// Le gabarit suivi : son nom, et chaque catégorie avec sa couleur et si une image y est.
    pub suivi: Option<(String, Vec<Pastille>)>,
    pub choix: Vec<Choix<'a>>,
}

impl<'a> Contenu<'a> {
    /// Ce que le document dit au panneau.
    pub fn du(store: &'a Store) -> Self {
        let tableau = store
            .active_board()
            .map_or(String::new(), |b| b.name.clone());
        let board = store.project.active_board_id.clone();
        let choix = a_choisir(&store.project);
        let zones = store.zones(&board);
        let suivi = suivi(zones).and_then(|id| {
            let gabarit = choix.iter().find(|c| c.id == id)?;
            let images = store
                .active_board()
                .map_or(&[][..], |b| b.images.as_slice());
            let pastilles = gabarit
                .categories
                .iter()
                .map(|(cat, nom, couleur)| {
                    let zone = glucose_core::gabarits::identifiant(id, cat);
                    let remplie = images
                        .iter()
                        .any(|i| store.zone_de(&board, i).is_some_and(|z| z.slot_id == zone));
                    Pastille {
                        nom: nom.to_string(),
                        couleur: *couleur,
                        remplie,
                    }
                })
                .collect();
            Some((gabarit.nom.to_string(), pastilles))
        });
        Self {
            tableau,
            suivi,
            choix,
        }
    }
}

/// La section du gabarit suivi : sa légende, son nom, « Retirer », ses pastilles.
#[derive(Debug, Clone)]
pub struct SectionSuivie {
    pub legende_at: (f32, f32),
    pub nom_at: (f32, f32),
    pub retirer: WidgetRect,
    pub pastilles: Vec<WidgetRect>,
}

/// Un gabarit dans le panneau : la rangée entière (ce qu'on clique), son titre, la rangée de
/// ses catégories, sa phrase.
#[derive(Debug, Clone)]
pub struct PresetRowLayout {
    pub index: usize,
    pub rect: WidgetRect,
    pub title_at: (f32, f32),
    pub slots: Vec<WidgetRect>,
    pub summary_at: (f32, f32),
}

#[derive(Debug, Clone)]
pub struct PresetsPanelLayout {
    pub suivie: Option<SectionSuivie>,
    pub caption_at: (f32, f32),
    pub rows: Vec<PresetRowLayout>,
    pub create_button: WidgetRect,
    /// Le bas de ce qui est posé : ce qui mesure la hauteur naturelle du panneau.
    pub bas: f32,
}

/// La géométrie du panneau — la seule, lue par le dessin et par le clic.
pub fn layout_presets_panel(frame: ScaledRect, contenu: &Contenu<'_>) -> PresetsPanelLayout {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (x, inner_w) = (frame.x + 14.0 * s, frame.w - 28.0 * s);
    let mut cy = frame.y + 38.0 * s;
    let suivie = contenu.suivi.as_ref().map(|(_, pastilles)| {
        let section = section_suivie((x, inner_w, cy, s), pastilles.len());
        cy = section.1;
        section.0
    });
    let caption_at = (x, cy);
    cy += 16.0 * s;
    let mut rows = Vec::with_capacity(contenu.choix.len());
    for (index, choix) in contenu.choix.iter().enumerate() {
        let haut = cy;
        let title_at = (x, cy);
        cy += 14.0 * s;
        let n = choix.categories.len().max(1) as f32;
        let gap = 2.0 * s;
        let slot_w = (inner_w - (n - 1.0) * gap) / n;
        let slot_h = 32.0 * s;
        let slots = (0..choix.categories.len())
            .map(|i| WidgetRect::new(x + i as f32 * (slot_w + gap), cy, slot_w, slot_h))
            .collect();
        cy += slot_h + 3.0 * s;
        let summary_at = (x, cy);
        cy += 16.0 * s;
        rows.push(PresetRowLayout {
            index,
            rect: WidgetRect::new(x, haut, inner_w, cy - haut),
            title_at,
            slots,
            summary_at,
        });
    }
    let create_button = WidgetRect::new(x, cy, inner_w, 24.0 * s);
    PresetsPanelLayout {
        suivie,
        caption_at,
        rows,
        create_button,
        bas: cy + 24.0 * s + 14.0 * s - frame.y,
    }
}

/// La section du gabarit suivi, posée à partir de `haut` ; rend aussi où la suite commence.
fn section_suivie(
    (x, inner_w, haut, s): (f32, f32, f32, f32),
    combien: usize,
) -> (SectionSuivie, f32) {
    let legende_at = (x, haut);
    let nom_at = (x, haut + 16.0 * s);
    let retirer = WidgetRect::new(x + inner_w - 64.0 * s, haut + 12.0 * s, 64.0 * s, 20.0 * s);
    let mut cy = haut + 40.0 * s;
    let (gap, h) = (4.0 * s, 18.0 * s);
    let w = (inner_w - (PASTILLES_PAR_RANGEE as f32 - 1.0) * gap) / PASTILLES_PAR_RANGEE as f32;
    let pastilles = (0..combien)
        .map(|i| {
            let (col, rang) = (i % PASTILLES_PAR_RANGEE, i / PASTILLES_PAR_RANGEE);
            WidgetRect::new(
                x + col as f32 * (w + gap),
                cy + rang as f32 * (h + gap),
                w,
                h,
            )
        })
        .collect();
    cy += combien.div_ceil(PASTILLES_PAR_RANGEE) as f32 * (h + gap) + 10.0 * s;
    (
        SectionSuivie {
            legende_at,
            nom_at,
            retirer,
            pastilles,
        },
        cy,
    )
}

/// **La hauteur naturelle du panneau** : celle de ce qu'il peut montrer de plus grand — les six
/// gabarits, et le plus long suivi —, mesurée sur sa propre mise en page.
pub fn hauteur_naturelle() -> f32 {
    let projet = glucose_core::types::Project::new("mesure");
    let choix = a_choisir(&projet);
    let plus_long = choix.iter().map(|c| c.categories.len()).max().unwrap_or(0);
    let contenu = Contenu {
        tableau: String::new(),
        suivi: Some((String::new(), vec![Pastille::default(); plus_long])),
        choix,
    };
    let unite = ScaledRect {
        x: 0.0,
        y: 0.0,
        w: super::TabId::Preset.default_width(),
        h: 0.0,
        scale: 1.0,
    };
    layout_presets_panel(unite, &contenu).bas
}

/// Ce qu'un clic dans le panneau demande.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GabaritIntent {
    /// Armer ce gabarit : il suivra le curseur jusqu'au clic qui le pose.
    Armer(String),
    /// Retirer le gabarit du tableau : toutes ses zones.
    Retirer,
}

/// Le clic : « Retirer », ou une rangée ; ailleurs, rien.
pub fn click_presets_panel(
    layout: &PresetsPanelLayout,
    contenu: &Contenu<'_>,
    (px, py): (f32, f32),
) -> Option<GabaritIntent> {
    if layout
        .suivie
        .as_ref()
        .is_some_and(|s| s.retirer.contains(px, py))
    {
        return Some(GabaritIntent::Retirer);
    }
    let row = layout.rows.iter().find(|r| r.rect.contains(px, py))?;
    Some(GabaritIntent::Armer(
        contenu.choix[row.index].id.to_string(),
    ))
}

#[cfg(test)]
mod tests;
