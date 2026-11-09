//! **COULEUR-1 — la barre de couleur** (fiche 98) : au-dessus de la barre d'action quand la
//! sélection porte un texte, un post-it, une membrane ou un dossier, le nuancier de Glucose — huit
//! teintes —, et « Par défaut ».
//!
//! # Ce que Tauri faisait, et ce qui change
//!
//! Glucose Tauri ne colorait un post-it que **pendant qu'on l'écrivait**, par un bouton posé
//! au-dessus de lui qui ouvrait une roue de couleurs ; une membrane naissait d'une teinte tirée
//! au hasard parmi six, et rien ne la changeait ensuite ; un texte n'avait rien. Ici la couleur
//! est un réglage de la **sélection**, comme la forme d'une flèche (FLECHE-3) : un clic colore
//! tout ce qui est choisi, un `Ctrl+Z` le défait, et une teinte s'allume quand tous la portent.
//! Chaque teinte est une commande (`couleur.rose`…) : la palette et le clavier les ont aussi.
//! La roue de Tauri, pour une couleur hors du nuancier, reste à faire.
//!
//! # Le nuancier
//!
//! Les huit teintes du thème (`DOMAIN_PALETTE`) : celles des domaines, choisies pour rester
//! distinctes sur le fond sombre ; les six premières sont celles que Tauri tirait pour une
//! membrane. Une seule palette pour tout ce qui se colore.
//!
//! # La mise en page est une fonction pure
//!
//! [`layout_barre_de_couleur`] rend les rectangles sans rien dessiner ; le tracé et le clic
//! lisent la même liste (loi L4), comme les deux barres au-dessus desquelles elle se pose.

use super::action_bar::{
    bouton, dans, fond_arrondi, pastille, BTN_PAD_X, BTN_PAD_Y, FONT, GAP, PAD_X, PAD_Y,
};
use crate::theme::{Theme, DOMAIN_PALETTE};
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::store::Store;
use tiny_skia::{Color, PixmapMut};

/// Le nom de chaque teinte du nuancier, dans l'ordre de `DOMAIN_PALETTE` : l'identifiant de sa
/// commande, puis ce que la palette en dit.
pub const NOMS: [(&str, &str); 8] = [
    ("bleu", "bleu"),
    ("emeraude", "émeraude"),
    ("rose", "rose"),
    ("ambre", "ambre"),
    ("violet", "violet"),
    ("rouge", "rouge"),
    ("cyan", "cyan"),
    ("orange", "orange"),
];

/// L'écart entre cette barre et celle qu'elle surmonte.
const AU_DESSUS: f32 = 8.0;
/// Le mot de la barre, et celui du bouton qui rend la couleur par défaut.
const MOT: &str = "Couleur";
pub const PAR_DEFAUT: &str = "Par défaut";

/// Une case de la barre : sa boîte, la teinte qu'elle pose (`None` : par défaut), et si tous
/// les nœuds choisis la portent déjà.
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    pub rect: (f32, f32, f32, f32),
    /// Le rang de la teinte dans le nuancier, ou `None` pour « Par défaut » — et pour la roue.
    pub teinte: Option<u8>,
    pub allume: bool,
    /// La case qui ouvre la roue de couleur (ROUE-1, fiche 99), pour une couleur hors du nuancier.
    pub roue: bool,
}

/// La barre, une fois placée.
#[derive(Clone, Debug, PartialEq)]
pub struct BarreDeCouleur {
    pub rect: (f32, f32, f32, f32),
    pub mot_x: f32,
    pub cases: Vec<Case>,
}

/// **La teinte de rang `rang`**, en `#rrggbb`.
pub fn teinte(rang: u8) -> Option<&'static str> {
    DOMAIN_PALETTE.get(usize::from(rang)).copied()
}

/// La barre pour la sélection courante, ou `None` si rien de colorable n'est choisi (un texte, un
/// post-it, une membrane, un dossier). Elle se pose
/// au-dessus de la barre des flèches s'il y en a une, sinon au-dessus de la barre d'action.
pub fn layout_barre_de_couleur(
    store: &Store,
    typography: &Typography,
    screen: (f32, f32),
    scale: f32,
) -> Option<BarreDeCouleur> {
    let couleurs = store.couleurs_choisies();
    let (premiere, autres) = couleurs.split_first()?;
    let commune = autres.iter().all(|c| c == premiere).then_some(*premiere);
    let dessous =
        super::options_de_fleche::layout_options_de_fleche(store, typography, screen, scale)
            .map(|b| b.rect.1)
            .or_else(|| {
                super::action_bar::layout_action_bar(
                    store,
                    typography,
                    (screen, scale),
                    glucose_core::groupe::Origine::Commune,
                )
                .map(|b| b.rect.1)
            })?;
    let s = crate::theme::clamp_ui_scale(scale);
    let font = FONT * s;
    let cote = font + BTN_PAD_Y * s * 2.0;
    let hauteur = cote + PAD_Y * s * 2.0;
    let mesure = |t: &str| typography.measure_text(t, font, Face::Regular).0;
    let defaut = mesure(PAR_DEFAUT) + BTN_PAD_X * s * 2.0;
    let n = DOMAIN_PALETTE.len() as f32;
    let largeur = PAD_X * s * 2.0
        + mesure(MOT)
        + GAP * s * 2.0
        + n * (cote + GAP * s)
        + defaut
        + GAP * s
        + cote;
    let (x, y) = (
        (screen.0 - largeur) / 2.0,
        dessous - AU_DESSUS * s - hauteur,
    );
    let mut curseur = x + PAD_X * s + mesure(MOT) + GAP * s * 2.0;
    let mut cases = Vec::with_capacity(DOMAIN_PALETTE.len() + 1);
    for (rang, hex) in DOMAIN_PALETTE.iter().enumerate() {
        cases.push(Case {
            rect: (curseur, y + PAD_Y * s, cote, cote),
            teinte: Some(rang as u8),
            allume: commune == Some(Some(hex)),
            roue: false,
        });
        curseur += cote + GAP * s;
    }
    cases.push(Case {
        rect: (curseur, y + PAD_Y * s, defaut, cote),
        teinte: None,
        allume: commune == Some(None),
        roue: false,
    });
    // La roue : allumée quand la couleur commune n'est pas une teinte du nuancier.
    let hors_du_nuancier = commune
        .flatten()
        .is_some_and(|c| !DOMAIN_PALETTE.iter().any(|t| t.eq_ignore_ascii_case(c)));
    cases.push(Case {
        rect: (curseur + defaut + GAP * s, y + PAD_Y * s, cote, cote),
        teinte: None,
        allume: hors_du_nuancier,
        roue: true,
    });
    Some(BarreDeCouleur {
        rect: (x, y, largeur, hauteur),
        mot_x: x + PAD_X * s,
        cases,
    })
}

/// La teinte qu'un clic en `(px, py)` pose — `Some(None)` pour « Par défaut », `None` à côté.
pub fn teinte_sous(barre: &BarreDeCouleur, px: f32, py: f32) -> Option<Option<u8>> {
    barre
        .cases
        .iter()
        .find(|c| !c.roue && dans(c.rect, px, py))
        .map(|c| c.teinte)
}

/// Le clic tombe-t-il sur la case de la roue ?
pub fn roue_sous(barre: &BarreDeCouleur, px: f32, py: f32) -> bool {
    barre.cases.iter().any(|c| c.roue && dans(c.rect, px, py))
}

/// Le clic tombe-t-il sur la barre ? Ce qui tombe dessus ne doit jamais atteindre le canevas :
/// la sélection s'y perdrait, et la barre avec elle.
pub fn couvre(barre: &BarreDeCouleur, px: f32, py: f32) -> bool {
    dans(barre.rect, px, py)
}

/// Dessine la barre, si quelque chose de colorable est choisi.
pub fn draw_barre_de_couleur(
    pixmap: &mut PixmapMut,
    store: &Store,
    typography: &Typography,
    theme: &Theme,
    (screen, scale): ((f32, f32), f32),
) {
    let Some(barre) = layout_barre_de_couleur(store, typography, screen, scale) else {
        return;
    };
    let s = crate::theme::clamp_ui_scale(scale);
    let font = FONT * s;
    pastille(pixmap, barre.rect, s, theme);
    let (_, y, _, h) = barre.rect;
    let style = |color| TextStyle {
        size: font,
        color,
        face: Face::Regular,
    };
    typography.draw_text(
        pixmap,
        MOT,
        barre.mot_x,
        y + (h - font) / 2.0,
        style(theme.text_muted),
    );
    for case in &barre.cases {
        let (cx, cy, cw, ch) = case.rect;
        if case.roue {
            dessiner_la_case_de_la_roue(pixmap, case, (s, theme));
            continue;
        }
        match case.teinte.and_then(teinte) {
            Some(hex) => {
                // Allumée, la case porte l'anneau des boutons allumés autour de sa teinte.
                if case.allume {
                    bouton(pixmap, case.rect, (s, true), theme);
                }
                let marge = 3.0 * s;
                let (r, g, b) = crate::renderer::parse_hex_color(hex, 0, 0, 0);
                fond_arrondi(
                    pixmap,
                    (cx + marge, cy + marge, cw - 2.0 * marge, ch - 2.0 * marge),
                    3.0 * s,
                    Color::from_rgba8(r, g, b, 255),
                    None,
                );
            }
            None => {
                bouton(pixmap, case.rect, (s, case.allume), theme);
                let encre = if case.allume {
                    theme.text_accent
                } else {
                    theme.text_secondary
                };
                typography.draw_text(
                    pixmap,
                    PAR_DEFAUT,
                    cx + BTN_PAD_X * s,
                    cy + (ch - font) / 2.0,
                    style(encre),
                );
            }
        }
    }
}

/// La case de la roue : un petit disque de la roue elle-même, à la clarté des teintes du nuancier.
fn dessiner_la_case_de_la_roue(pixmap: &mut PixmapMut, case: &Case, (s, theme): (f32, &Theme)) {
    if case.allume {
        bouton(pixmap, case.rect, (s, true), theme);
    }
    let marge = 3.0 * s;
    let (cx, cy, cw, _) = case.rect;
    let d = (cw - 2.0 * marge).round().max(1.0) as u32;
    // Gardé une fois par taille : la barre se redessine à chaque image de l'interface, le disque
    // ne change jamais (une clarté fixe, celle des teintes du nuancier).
    thread_local! {
        static PETIT: std::cell::RefCell<Option<(u32, tiny_skia::Pixmap)>> = const { std::cell::RefCell::new(None) };
    }
    let disque = PETIT.with_borrow_mut(|garde| {
        if garde.as_ref().is_none_or(|(taille, _)| *taille != d) {
            *garde = Some((d, super::roue::disque(0.75, d)));
        }
        garde.as_ref().map(|(_, p)| p.clone())
    });
    if let Some(disque) = disque {
        let paint = tiny_skia::PixmapPaint::default();
        let (x, y) = ((cx + marge).round() as i32, (cy + marge).round() as i32);
        pixmap.draw_pixmap(
            x,
            y,
            disque.as_ref(),
            &paint,
            tiny_skia::Transform::identity(),
            None,
        );
    }
}

#[cfg(test)]
mod tests;
