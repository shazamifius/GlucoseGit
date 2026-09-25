//! **La barre d'options d'une flèche** (FLECHE-3) — ce que Glucose Tauri montrait sous une
//! flèche sélectionnée (`ArrowOptions.tsx`) : droite ou courbe, dans les deux sens,
//! l'épaisseur, la relation.
//!
//! # Ce qui change par rapport à Tauri
//!
//! Tauri ne la montrait que pour **une** flèche seule. Les chiffres du clavier qualifiaient
//! déjà toutes les flèches sélectionnées d'un coup (PRED-1) : la barre fait de même. Un réglage
//! s'y allume quand **toutes** les flèches sélectionnées le partagent ; sinon aucun ne
//! s'allume, et un clic le leur donne à toutes.
//!
//! La relation se choisit par ses six sigles — ceux que le tracé porte —, et non par une liste
//! déroulante de libellés : on reconnaît ce qu'on voit sur la flèche.
//!
//! # La mise en page est une fonction pure
//!
//! [`layout_options_de_fleche`] rend les rectangles sans rien dessiner ; le tracé et le clic
//! lisent la même liste (loi L4), comme la barre d'action au-dessus de laquelle elle se pose.

use super::action_bar::{
    bouton, dans, filet_vertical, pastille, BTN_PAD_X, BTN_PAD_Y, FONT, GAP, ICON, ICON_GAP,
    ICON_STROKE, PAD_X, PAD_Y,
};
use crate::icons::{draw_icon_scaled, IconType};
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::arrow::aspect::EPAISSEUR;
use glucose_core::store::{Reglage, Store};
use glucose_core::types::{Annotation, ArrowPredicate};
use tiny_skia::PixmapMut;

/// Les épaisseurs proposées, en pixels d'écran — celles de Tauri —, avec ce que leur bouton
/// écrit.
pub const EPAISSEURS: [(f64, &str); 4] = [(1.0, "1"), (2.0, "2"), (3.0, "3"), (5.0, "5")];

/// L'écart entre cette barre et la barre d'action qu'elle surmonte.
const AU_DESSUS: f32 = 8.0;

/// Ce qu'un bouton de la barre montre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Contenu {
    Texte(&'static str),
    /// Le sigle d'une relation, dessiné comme sur la flèche.
    Sigle(ArrowPredicate),
    /// Une icône et son libellé : une action, pas un réglage.
    Action(IconType, &'static str),
}

/// Ce qu'un clic sur un bouton demande.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    /// Poser ce réglage sur toutes les flèches sélectionnées.
    Regler(Reglage),
    /// Ouvrir l'éditeur d'ancres de la flèche sélectionnée (FLECHE-4).
    Ancrer,
}

/// Un bouton : sa boîte, ce qu'il montre, s'il est allumé, et ce qu'un clic demande.
#[derive(Clone, Debug, PartialEq)]
pub struct Bouton {
    pub rect: (f32, f32, f32, f32),
    pub contenu: Contenu,
    /// Toutes les flèches sélectionnées ont déjà ce réglage.
    pub allume: bool,
    pub action: Action,
}

/// La barre, une fois placée.
#[derive(Clone, Debug, PartialEq)]
pub struct OptionsDeFleche {
    pub rect: (f32, f32, f32, f32),
    /// « Flèche », puis « Épaisseur » : où leurs mots se posent.
    pub mots: Vec<(&'static str, f32)>,
    /// Les filets qui séparent les groupes.
    pub filets: Vec<f32>,
    pub boutons: Vec<Bouton>,
}

/// Ce que les flèches sélectionnées ont en commun : `None` là où elles diffèrent.
#[derive(Debug, Default)]
struct EnCommun {
    courbe: Option<bool>,
    double_sens: Option<bool>,
    epaisseur: Option<f64>,
    relation: Option<Option<ArrowPredicate>>,
}

impl EnCommun {
    fn de(fleches: &[&Annotation]) -> Self {
        fn commun<T: PartialEq + Copy>(mut valeurs: impl Iterator<Item = T>) -> Option<T> {
            let premiere = valeurs.next()?;
            valeurs.all(|v| v == premiere).then_some(premiere)
        }
        let reglages = || {
            fleches.iter().filter_map(|a| match a {
                Annotation::Arrow {
                    arrow_type,
                    arrow_bidirectional,
                    stroke_width,
                    predicate,
                    ..
                } => Some((
                    arrow_type.as_deref() == Some("curved"),
                    *arrow_bidirectional,
                    stroke_width.unwrap_or(EPAISSEUR),
                    *predicate,
                )),
                _ => None,
            })
        };
        Self {
            courbe: commun(reglages().map(|r| r.0)),
            double_sens: commun(reglages().map(|r| r.1)),
            epaisseur: commun(reglages().map(|r| r.2)),
            relation: commun(reglages().map(|r| r.3)),
        }
    }

    /// Les boutons de la barre, par groupe, dans l'ordre : leur contenu, s'ils sont allumés,
    /// et ce qu'un clic pose.
    fn groupes(&self) -> [Vec<(Contenu, bool, Reglage)>; 4] {
        let double = self.double_sens == Some(true);
        [
            vec![
                (
                    Contenu::Texte("Droite"),
                    self.courbe == Some(false),
                    Reglage::Courbe(false),
                ),
                (
                    Contenu::Texte("Courbe"),
                    self.courbe == Some(true),
                    Reglage::Courbe(true),
                ),
            ],
            // Allumé, il éteint ; sinon il allume — pour toutes à la fois.
            vec![(
                Contenu::Texte("Double sens"),
                double,
                Reglage::DoubleSens(!double),
            )],
            EPAISSEURS
                .iter()
                .map(|&(e, texte)| {
                    (
                        Contenu::Texte(texte),
                        self.epaisseur == Some(e),
                        Reglage::Epaisseur(e),
                    )
                })
                .collect(),
            ArrowPredicate::ALL
                .iter()
                .map(|&p| {
                    let allume = self.relation == Some(Some(p));
                    // Recliquer la relation qu'elles portent toutes la retire.
                    let reglage = Reglage::Relation((!allume).then_some(p));
                    (Contenu::Sigle(p), allume, reglage)
                })
                .collect(),
        ]
    }
}

/// **Les boutons de la barre**, par groupe — ou `None` s'il n'y a pas de flèche sélectionnée.
/// Quels boutons : cette fonction ; où les poser : [`layout_options_de_fleche`].
fn groupes_de_boutons(store: &Store) -> Option<Vec<Vec<(Contenu, bool, Action)>>> {
    let fleches = store.selected_arrows();
    if fleches.is_empty() {
        return None;
    }
    let mut groupes: Vec<Vec<(Contenu, bool, Action)>> = EnCommun::de(&fleches)
        .groupes()
        .into_iter()
        .map(|g| {
            g.into_iter()
                .map(|(c, a, r)| (c, a, Action::Regler(r)))
                .collect()
        })
        .collect();
    // L'éditeur du texte lié, pour une flèche seule qui touche une carte de texte : c'est dans
    // une carte qu'on désigne un passage. « Ancrer… » se lisait comme un libellé tronqué, et
    // pas comme un bouton ; il dit maintenant ce qu'il fait, en entier, avec son crayon — et
    // s'allume quand la flèche désigne déjà un passage.
    if let [fleche] = fleches[..] {
        if touche_une_carte(store, fleche) {
            groupes.push(vec![(
                Contenu::Action(IconType::Crayon, EDITER_LE_TEXTE_LIE),
                designe_un_passage(fleche),
                Action::Ancrer,
            )]);
        }
    }
    Some(groupes)
}

/// La barre pour la sélection courante, ou `None` s'il n'y a pas de flèche sélectionnée.
///
/// Fonction pure : elle ne lit que le store et la typographie, et ne dessine rien.
pub fn layout_options_de_fleche(
    store: &Store,
    typography: &Typography,
    screen: (f32, f32),
    scale: f32,
) -> Option<OptionsDeFleche> {
    let groupes = groupes_de_boutons(store)?;
    let dessous = super::action_bar::layout_action_bar(store, typography, screen, scale)?;
    let s = crate::theme::clamp_ui_scale(scale);
    let font = FONT * s;
    let mesure = |t: &str| typography.measure_text(t, font, Face::Regular).0;
    let hauteur_btn = font + BTN_PAD_Y * s * 2.0;
    let hauteur = hauteur_btn + PAD_Y * s * 2.0;
    let largeur_de = |c: &Contenu| match c {
        Contenu::Texte(t) => mesure(t) + BTN_PAD_X * s * 2.0,
        Contenu::Sigle(_) => hauteur_btn,
        Contenu::Action(_, t) => (ICON + ICON_GAP) * s + mesure(t) + BTN_PAD_X * s * 2.0,
    };

    // Une première passe mesure, la seconde place : la barre se centre sur sa largeur.
    let titre = "Flèche";
    let epaisseur = "Épaisseur";
    let mut largeur = PAD_X * s * 2.0 + mesure(titre);
    for (rang, groupe) in groupes.iter().enumerate() {
        largeur += GAP * s * 2.0 + 1.0;
        if rang == 2 {
            largeur += mesure(epaisseur) + GAP * s;
        }
        largeur += groupe
            .iter()
            .map(|(c, _, _)| largeur_de(c) + GAP * s)
            .sum::<f32>()
            - GAP * s;
    }
    let x = (screen.0 - largeur) / 2.0;
    let y = dessous.rect.1 - AU_DESSUS * s - hauteur;

    let mut curseur = x + PAD_X * s;
    let mut mots = vec![(titre, curseur)];
    curseur += mesure(titre);
    let (mut filets, mut boutons) = (Vec::new(), Vec::new());
    for (rang, groupe) in groupes.into_iter().enumerate() {
        curseur += GAP * s;
        filets.push(curseur);
        curseur += 1.0 + GAP * s;
        if rang == 2 {
            mots.push((epaisseur, curseur));
            curseur += mesure(epaisseur) + GAP * s;
        }
        for (i, (contenu, allume, action)) in groupe.into_iter().enumerate() {
            if i > 0 {
                curseur += GAP * s;
            }
            let w = largeur_de(&contenu);
            boutons.push(Bouton {
                rect: (curseur, y + PAD_Y * s, w, hauteur_btn),
                contenu,
                allume,
                action,
            });
            curseur += w;
        }
    }
    Some(OptionsDeFleche {
        rect: (x, y, largeur, hauteur),
        mots,
        filets,
        boutons,
    })
}

/// Ce qu'un clic en `(px, py)` demande — `None` s'il tombe à côté d'un bouton.
pub fn action_sous(barre: &OptionsDeFleche, px: f32, py: f32) -> Option<Action> {
    barre
        .boutons
        .iter()
        .find(|b| dans(b.rect, px, py))
        .map(|b| b.action)
}

/// Le libellé du bouton qui ouvre l'éditeur du texte lié.
pub const EDITER_LE_TEXTE_LIE: &str = "Éditer le texte lié";

/// La flèche désigne-t-elle déjà un passage, dans sa source ou dans sa cible ?
fn designe_un_passage(fleche: &Annotation) -> bool {
    let Annotation::Arrow {
        source_text_sel,
        target_text_sel,
        ..
    } = fleche
    else {
        return false;
    };
    [source_text_sel, target_text_sel]
        .into_iter()
        .any(|sel| glucose_core::text_anchors::has_text_selection(sel.as_ref()))
}

/// La flèche touche-t-elle, par l'un de ses bouts, une carte de texte ?
fn touche_une_carte(store: &Store, fleche: &Annotation) -> bool {
    let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = fleche
    else {
        return false;
    };
    let Some(board) = store.active_board() else {
        return false;
    };
    [source_id, target_id].into_iter().flatten().any(|id| {
        matches!(
            store.project.annotation(&board.id, id),
            Some(Annotation::Text { .. })
        )
    })
}

/// Le clic tombe-t-il sur la barre, bouton ou pas ? Ce qui tombe dessus ne doit jamais
/// atteindre le canevas : cliquer à côté d'un bouton désélectionnerait la flèche, et la barre
/// disparaîtrait sous le doigt.
pub fn couvre(barre: &OptionsDeFleche, px: f32, py: f32) -> bool {
    dans(barre.rect, px, py)
}

/// Dessine la barre, si une flèche est sélectionnée.
pub fn draw_options_de_fleche(
    pixmap: &mut PixmapMut,
    store: &Store,
    typography: &Typography,
    theme: &Theme,
    (screen, scale): ((f32, f32), f32),
) {
    let Some(barre) = layout_options_de_fleche(store, typography, screen, scale) else {
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
    for (mot, x) in &barre.mots {
        typography.draw_text(
            pixmap,
            mot,
            *x,
            y + (h - font) / 2.0,
            style(theme.text_muted),
        );
    }
    for x in &barre.filets {
        filet_vertical(
            pixmap,
            *x,
            y + PAD_Y * s,
            h - PAD_Y * s * 2.0,
            theme.border_medium,
        );
    }
    for b in &barre.boutons {
        bouton(pixmap, b.rect, (s, b.allume), theme);
        let encre = if b.allume {
            theme.text_accent
        } else {
            theme.text_secondary
        };
        let (bx, by, bw, bh) = b.rect;
        // Ce que le bouton écrit, et où : après son icône s'il en a une.
        let texte = match b.contenu {
            Contenu::Texte(t) => Some((bx + BTN_PAD_X * s, t)),
            Contenu::Action(icone, t) => {
                draw_icon_scaled(
                    pixmap,
                    icone,
                    bx + BTN_PAD_X * s,
                    by + (bh - ICON * s) / 2.0,
                    ICON * s,
                    encre,
                    ICON_STROKE,
                );
                Some((bx + (BTN_PAD_X + ICON + ICON_GAP) * s, t))
            }
            Contenu::Sigle(p) => {
                crate::renderer::predicate::draw_sigil(
                    pixmap,
                    p,
                    (bx + bw / 2.0, by + bh / 2.0),
                    bh * 0.3,
                    theme.predicate_color(p),
                );
                None
            }
        };
        if let Some((x, t)) = texte {
            typography.draw_text(pixmap, t, x, by + (bh - font) / 2.0, style(encre));
        }
    }
}

#[cfg(test)]
mod tests;
