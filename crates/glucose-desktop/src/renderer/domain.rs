//! La **réglette de domaines** : ce qui rend une assignation visible sur le canevas.
//!
//! # Pourquoi une réglette, et pas une teinte
//!
//! Une assignation porte deux informations, et il faut lire les deux : *quel* domaine, et
//! *avec quelle force*. Les encodages qu'on essaie d'abord échouent chacun sur l'une des deux :
//!
//! | Essai | Ce qu'il perd |
//! |---|---|
//! | Teinter le nœud à la couleur du domaine | la force ; et un nœud à trois domaines n'a qu'une couleur |
//! | Moduler l'opacité du nœud selon le poids | la lisibilité du contenu, et l'identité du domaine |
//! | Un segment par domaine sur toute la largeur, longueur ∝ poids | des poids **relatifs** : 0,2 et 0,4 se dessinent comme 0,4 et 0,8 |
//! | Une pastille par domaine | la force, encore |
//!
//! La réglette garde les deux : **une colonne par domaine**, dans l'ordre du catalogue (DOM-2),
//! chacune avec sa piste de hauteur constante — sa couleur en sourdine — et son remplissage
//! monté depuis le bas à `poids × hauteur`. La piste est la clé de la lecture : elle donne à
//! chaque colonne le **même plafond**, donc deux remplissages se comparent à l'œil comme deux
//! barres d'un histogramme, et un poids de 0 reste visible (piste vide) au lieu de se confondre
//! avec « pas de domaine ». Trois domaines font trois colonnes, pas trois nuances mélangées :
//! la lisibilité ne se dégrade pas avec le nombre, elle occupe seulement plus de largeur.
//!
//! Le sigle du domaine (`Domain::icon`) se pose au-dessus de sa colonne dès que le niveau de
//! détail l'autorise (SCALE-2) : la couleur seule demanderait d'aller lire le panneau, le sigle
//! nomme le domaine sur place.
//!
//! # SCALE-1 — la réglette appartient au monde, elle suit le zoom en bloc
//!
//! [`GaugeLayout`] décrit la réglette à l'échelle 1 — piste, barre, écart, marge, rayon, corps
//! du sigle — et [`GaugeLayout::scaled`] applique le **même** facteur à tous ces champs d'un
//! coup. Aucune borne par valeur : une réglette dézoomée rétrécit entièrement, elle ne se
//! réorganise pas (standard § 4.4).
//!
//! # DOMAIN-TINT-1 — le renderer lit, il ne cherche pas
//!
//! Résoudre `domain_id → (couleur, sigle)` est une recherche dans le catalogue. La faire par
//! nœud et par frame ferait dépendre le coût d'une frame du contenu du document (loi L2) et
//! mettrait un calcul dans le renderer (§ 4.4). [`DomainTints`] la fait **une fois par version
//! du document**, exactement comme l'index spatial, et la passe de dessin ne fait plus que
//! lire une table.

use super::scale::WorldScale;
use super::{parse_hex_color, push_rounded_rect};
use crate::theme::Theme;
use crate::typography::{TextStyle, Typography};
use glucose_core::store::Store;
use glucose_core::types::DomainAssignment;
use std::collections::HashMap;
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Transform};

// ── Mesures, en unités monde ────────────────────────────────────────────────

/// Hauteur d'une piste — le plafond commun à toutes les colonnes.
const TRACK_HEIGHT: f32 = 30.0;
/// Largeur d'une colonne.
///
/// Elle est réglée sur le sigle, pas l'inverse : [`SIGIL_MAX_CHARS`] caractères au corps
/// [`SIGIL_FONT`] mesurent un peu moins que cette largeur, donc deux sigles voisins ne se
/// touchent jamais. Rétrécir la colonne sans rétrécir le sigle les ferait se chevaucher.
const BAR_WIDTH: f32 = 18.0;
/// Écart entre deux colonnes.
const BAR_GAP: f32 = 6.0;
/// Écart entre le bas de la réglette et le bord haut du nœud.
const NODE_GAP: f32 = 10.0;
/// Rayon des coins d'une colonne.
const BAR_RADIUS: f32 = 4.0;
/// Corps du sigle posé au-dessus d'une colonne.
const SIGIL_FONT: f32 = 10.0;
/// Écart entre le haut de la piste et le bas du sigle.
const SIGIL_GAP: f32 = 4.0;

/// Opacité d'une piste vide : présente, mais discrète.
const TRACK_ALPHA: u8 = 46;
/// Opacité d'un remplissage : c'est lui qu'on lit.
const FILL_ALPHA: u8 = 235;
/// Opacité d'un sigle.
const SIGIL_ALPHA: u8 = 225;

/// Nombre de caractères retenus d'un sigle.
///
/// Le sigle se dessine centré sur une colonne large de [`BAR_WIDTH`] unités monde ; au-delà de
/// trois caractères il déborde sur sa voisine et les deux deviennent illisibles. La troncature
/// se fait au remplissage de la table, jamais dans la passe de dessin.
const SIGIL_MAX_CHARS: usize = 3;

/// Mise en page d'une réglette. **En unités monde tant que `scaled` n'a pas été appelée** ;
/// en pixels écran après, et rien d'autre ne dérive du zoom entre les deux (SCALE-1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GaugeLayout {
    pub track_height: f32,
    pub bar_width: f32,
    pub bar_gap: f32,
    pub node_gap: f32,
    pub bar_radius: f32,
    pub sigil_font: f32,
    pub sigil_gap: f32,
}

impl GaugeLayout {
    /// La réglette telle qu'elle serait dessinée à l'échelle 1.
    pub(super) fn world() -> Self {
        Self {
            track_height: TRACK_HEIGHT,
            bar_width: BAR_WIDTH,
            bar_gap: BAR_GAP,
            node_gap: NODE_GAP,
            bar_radius: BAR_RADIUS,
            sigil_font: SIGIL_FONT,
            sigil_gap: SIGIL_GAP,
        }
    }

    /// L'UNIQUE transformation d'échelle de la réglette (SCALE-1) : tous les champs, le même
    /// facteur, au même instant. Ajouter un champ ici sans le mettre à l'échelle, ou le borner
    /// au passage, c'est ramener R-45.
    pub(super) fn scaled(self, s: WorldScale) -> Self {
        Self {
            track_height: s.world(self.track_height),
            bar_width: s.world(self.bar_width),
            bar_gap: s.world(self.bar_gap),
            node_gap: s.world(self.node_gap),
            bar_radius: s.world(self.bar_radius),
            sigil_font: s.world(self.sigil_font),
            sigil_gap: s.world(self.sigil_gap),
        }
    }

    /// Largeur occupée par `count` colonnes.
    pub(super) fn block_width(self, count: usize) -> f32 {
        match count {
            0 => 0.0,
            n => n as f32 * self.bar_width + (n - 1) as f32 * self.bar_gap,
        }
    }
}

// ── La table des teintes ────────────────────────────────────────────────────

/// Ce que le dessin a besoin de savoir d'un domaine, et rien de plus.
#[derive(Clone, Debug, PartialEq)]
pub struct DomainTint {
    pub rgb: (u8, u8, u8),
    /// Sigle déjà tronqué à [`SIGIL_MAX_CHARS`] et mis en capitales.
    pub sigil: String,
}

/// `domain_id → teinte`, reconstruite une fois par version du document (DOMAIN-TINT-1).
#[derive(Debug, Default)]
pub struct DomainTints {
    built_for: Option<u64>,
    entries: HashMap<String, DomainTint>,
}

impl DomainTints {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconstruit la table si le document a changé depuis la dernière frame.
    ///
    /// `store.version` n'avance que sur une vraie mutation (UNDO-1) : la navigation, le pan et
    /// le zoom ne paient donc rien ici, et une frame de document inchangé ne fait qu'une
    /// comparaison d'entiers.
    pub fn refresh(&mut self, store: &Store, theme: &Theme) {
        if self.built_for == Some(store.version) {
            return;
        }
        self.built_for = Some(store.version);
        self.entries.clear();
        let fallback = (
            (theme.domain_fallback.red() * 255.0) as u8,
            (theme.domain_fallback.green() * 255.0) as u8,
            (theme.domain_fallback.blue() * 255.0) as u8,
        );
        for domain in &store.project.domains {
            let rgb = parse_hex_color(&domain.color, fallback.0, fallback.1, fallback.2);
            self.entries.insert(
                domain.id.clone(),
                DomainTint { rgb, sigil: shorten_sigil(&domain.icon) },
            );
        }
    }

    pub fn get(&self, domain_id: &str) -> Option<&DomainTint> {
        self.entries.get(domain_id)
    }
}

/// Ramène un sigle à ce qu'une colonne peut porter.
fn shorten_sigil(icon: &str) -> String {
    icon.trim()
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(SIGIL_MAX_CHARS)
        .flat_map(|c| c.to_uppercase())
        .collect()
}

// ── Dessin ──────────────────────────────────────────────────────────────────

/// Ce qu'une passe de réglettes garde constant.
struct GaugePass<'a> {
    typography: &'a Typography,
    tints: &'a DomainTints,
    scale: WorldScale,
}

/// Largeur écran d'une réglette de `count` colonnes.
///
/// Ce que doit connaître un appelant qui veut l'aligner autrement qu'à gauche — la membrane,
/// dont le bord haut-gauche est déjà pris par son titre protecteur.
pub(super) fn gauge_width(scale: WorldScale, count: usize) -> f32 {
    GaugeLayout::world().scaled(scale).block_width(count)
}

/// Dessine la réglette d'un nœud, ancrée sur le coin haut-gauche de sa boîte écran.
///
/// `node_top_left` est en pixels écran ; la réglette monte au-dessus de ce point. Un nœud sans
/// domaine ne coûte qu'un test de vacuité.
pub(super) fn draw_domain_gauge(
    typography: &Typography,
    tints: &DomainTints,
    pixmap: &mut PixmapMut,
    scale: WorldScale,
    node_top_left: (f32, f32),
    assignments: &[DomainAssignment],
) {
    if assignments.is_empty() {
        return;
    }
    let pass = &GaugePass { typography, tints, scale };
    let layout = GaugeLayout::world().scaled(pass.scale);
    let track_bottom = node_top_left.1 - layout.node_gap;
    let track_top = track_bottom - layout.track_height;

    for (column, assignment) in assignments.iter().enumerate() {
        let Some(tint) = pass.tints.get(&assignment.domain_id) else {
            // DOM-1 rend ce cas impossible dans un document sain ; le renderer ne dessine pas
            // ce qu'il ne sait pas nommer, il ne panique pas pour autant.
            continue;
        };
        let x = node_top_left.0 + column as f32 * (layout.bar_width + layout.bar_gap);
        draw_column(pixmap, (x, track_top), &layout, tint, assignment.weight);
        draw_sigil(pass, pixmap, (x, track_top), &layout, tint);
    }
}

/// La piste d'une colonne et son remplissage.
fn draw_column(
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &GaugeLayout,
    tint: &DomainTint,
    weight: f64,
) {
    let (r, g, b) = tint.rgb;
    fill_rounded(pixmap, at, (layout.bar_width, layout.track_height), layout.bar_radius, Color::from_rgba8(r, g, b, TRACK_ALPHA));

    // Le poids vient du document, pas du zoom : ce test n'est pas une borne sur une longueur
    // mise à l'échelle (§ 4.4), c'est le refus de dériver une hauteur d'un nombre qui n'en est
    // pas un. Le noyau valide déjà la plage à l'écriture et à l'ouverture d'un fichier.
    if !weight.is_finite() || weight <= 0.0 {
        return;
    }
    let fill_height = layout.track_height * weight as f32;
    let top = at.1 + layout.track_height - fill_height;
    fill_rounded(pixmap, (at.0, top), (layout.bar_width, fill_height), layout.bar_radius, Color::from_rgba8(r, g, b, FILL_ALPHA));
}

/// Le sigle du domaine, centré au-dessus de sa colonne.
fn draw_sigil(
    pass: &GaugePass,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &GaugeLayout,
    tint: &DomainTint,
) {
    // SCALE-2 — sous le seuil, la réglette garde ses colonnes et laisse tomber son texte,
    // exactement comme une carte garde son cadre et laisse tomber le sien.
    if !pass.scale.draws_detail() || tint.sigil.is_empty() {
        return;
    }
    let (width, _) = pass.typography.measure_text(&tint.sigil, layout.sigil_font, true);
    let (r, g, b) = tint.rgb;
    pass.typography.draw_text(
        pixmap,
        &tint.sigil,
        at.0 + (layout.bar_width - width) / 2.0,
        at.1 - layout.sigil_gap - layout.sigil_font,
        TextStyle {
            size: layout.sigil_font,
            color: Color::from_rgba8(r, g, b, SIGIL_ALPHA),
            bold: true,
        },
    );
}

fn fill_rounded(pixmap: &mut PixmapMut, at: (f32, f32), size: (f32, f32), radius: f32, color: Color) {
    if !(at.0.is_finite() && at.1.is_finite() && size.0 > 0.0 && size.1 > 0.0) {
        return;
    }
    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, at.0, at.1, size.0, size.1, radius);
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint { anti_alias: true, ..Default::default() };
    paint.set_color(color);
    pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
}

#[cfg(test)]
mod proof;

#[cfg(test)]
mod tests {
    use super::*;
    use glucose_core::store::DomainPatch;
    use glucose_core::types::Domain;

    fn domain(id: &str, color: &str, icon: &str) -> Domain {
        Domain {
            id: id.into(),
            name: id.into(),
            color: color.into(),
            icon: icon.into(),
            created_at: 0,
        }
    }

    #[test]
    fn test_scale_1_the_gauge_is_self_similar_at_every_zoom() {
        let world = GaugeLayout::world();
        for zoom in [0.25_f64, 0.5, 1.0, 2.0, 4.0, 16.0] {
            let screen = world.scaled(WorldScale::new(zoom));
            let pairs = [
                (screen.bar_width, world.bar_width),
                (screen.bar_gap, world.bar_gap),
                (screen.node_gap, world.node_gap),
                (screen.bar_radius, world.bar_radius),
                (screen.sigil_font, world.sigil_font),
                (screen.sigil_gap, world.sigil_gap),
                (screen.block_width(5), world.block_width(5)),
            ];
            for (on_screen, in_world) in pairs {
                let expected = in_world / world.track_height;
                let observed = on_screen / screen.track_height;
                assert!(
                    (observed - expected).abs() < 1e-6,
                    "zoom {zoom} : rapport {observed} au lieu de {expected}"
                );
            }
        }
    }

    #[test]
    fn test_a_gauge_widens_by_one_column_per_domain() {
        let layout = GaugeLayout::world();
        assert_eq!(layout.block_width(0), 0.0);
        assert_eq!(layout.block_width(1), BAR_WIDTH);
        assert_eq!(layout.block_width(2), BAR_WIDTH * 2.0 + BAR_GAP);
        assert_eq!(layout.block_width(4), BAR_WIDTH * 4.0 + BAR_GAP * 3.0);
    }

    /// Chaque sigle de la palette doit tenir dans le pas d'une colonne, sinon deux voisins se
    /// chevauchent et la réglette cesse de nommer quoi que ce soit. C'est la palette qui est
    /// testée, pas les 17 576 triplets de capitales : un sigle venu d'un autre outil peut
    /// déborder, ce qui est laid mais pas faux — la couleur, elle, reste lisible.
    #[test]
    fn test_every_sigil_of_the_palette_fits_in_a_column_pitch() {
        let typo = Typography::new();
        let pitch = BAR_WIDTH + BAR_GAP;
        for sigil in crate::theme::DOMAIN_SIGILS {
            let (width, _) = typo.measure_text(sigil, SIGIL_FONT, true);
            assert!(width <= pitch, "« {sigil} » mesure {width:.1} pour un pas de {pitch}");
        }
    }

    #[test]
    fn test_a_sigil_is_trimmed_to_what_a_column_can_carry() {
        assert_eq!(shorten_sigil("sci"), "SCI");
        assert_eq!(shorten_sigil("  jv "), "JV");
        assert_eq!(shorten_sigil("Conlang"), "CON");
        assert_eq!(shorten_sigil(""), "");
        assert_eq!(shorten_sigil("   "), "");
    }

    /// DOMAIN-TINT-1 — la table se reconstruit quand le document change, et seulement là.
    #[test]
    fn test_the_tint_table_follows_the_document_version_and_nothing_else() {
        let theme = Theme::dark();
        let mut store = Store::new("Teintes");
        store.try_add_domain(domain("d-1", "#38bdf8", "SCI")).expect("catalogue vide");
        let mut tints = DomainTints::new();

        tints.refresh(&store, &theme);
        assert_eq!(tints.get("d-1").expect("d-1").rgb, (0x38, 0xbd, 0xf8));
        assert_eq!(tints.get("d-1").expect("d-1").sigil, "SCI");
        let built = tints.built_for;

        // Une passe de navigation ne fait pas avancer la version : rien n'est reconstruit.
        store.pan(120.0, 40.0);
        store.zoom(2.0, 0.0, 0.0);
        tints.refresh(&store, &theme);
        assert_eq!(tints.built_for, built, "le pan ne doit pas relancer la construction");

        // Une vraie mutation, elle, est vue à la frame suivante.
        store
            .try_update_domain("d-1", DomainPatch::new().with_color("#f472b6").with_icon("ART"))
            .expect("d-1 est au catalogue");
        tints.refresh(&store, &theme);
        assert_eq!(tints.get("d-1").expect("d-1").rgb, (0xf4, 0x72, 0xb6));
        assert_eq!(tints.get("d-1").expect("d-1").sigil, "ART");
    }

    /// Un domaine dont la couleur est illisible garde une teinte, celle du thème : il reste
    /// visible au lieu de disparaître sans dire pourquoi.
    #[test]
    fn test_an_unreadable_colour_falls_back_to_the_theme() {
        let theme = Theme::dark();
        let mut store = Store::new("Repli");
        store.try_add_domain(domain("d", "pas une couleur", "X")).expect("catalogue vide");
        let mut tints = DomainTints::new();
        tints.refresh(&store, &theme);

        let expected = (
            (theme.domain_fallback.red() * 255.0) as u8,
            (theme.domain_fallback.green() * 255.0) as u8,
            (theme.domain_fallback.blue() * 255.0) as u8,
        );
        assert_eq!(tints.get("d").expect("d").rgb, expected);
    }

    /// Un domaine supprimé disparaît de la table : la cascade du noyau (DOM-1) et la table du
    /// rendu ne peuvent pas diverger, puisque la seconde se relit de la première.
    #[test]
    fn test_a_removed_domain_leaves_the_tint_table_too() {
        let theme = Theme::dark();
        let mut store = Store::new("Cascade");
        store.try_add_domain(domain("d", "#38bdf8", "SCI")).expect("catalogue vide");
        let mut tints = DomainTints::new();
        tints.refresh(&store, &theme);
        assert!(tints.get("d").is_some());

        store.try_remove_domain("d").expect("d est au catalogue");
        tints.refresh(&store, &theme);
        assert!(tints.get("d").is_none(), "la table doit suivre la cascade du noyau");
    }
}
