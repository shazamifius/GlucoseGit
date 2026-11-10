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
//! déroulante : on reconnaît ce qu'on voit sur la flèche.
//!
//! # BADGE-2 — chaque sigle dit son mot
//!
//! Son essai du 26/09 : *« on a plein de logos, plein de couleurs, mais absolument rien qui nous
//! dit que ça, ça dépend de ça, ou que ça contredit »*. La liste de Tauri portait ces mots ; les
//! sigles seuls les avaient perdus, et le noyau les avait pourtant (`ArrowPredicate::label`),
//! lus par personne. Chaque bouton de relation montre désormais son sigle **et** son mot. Si la
//! fenêtre est trop étroite pour tout tenir, la barre retombe sur les sigles seuls — du plus
//! riche au plus sobre, comme la barre d'action, et sans largeur écrite en dur.
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

/// Ce qu'un bouton de la barre montre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Contenu {
    Texte(&'static str),
    /// Le sigle d'une relation, dessiné comme sur la flèche.
    Sigle(ArrowPredicate),
    /// Le sigle d'une relation **et son mot** (BADGE-2) : « contredit », « dépend de ».
    Relation(ArrowPredicate),
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
    /// « Flèche », puis « Épaisseur » : où leurs mots s'écrivent `(x, y)`.
    pub mots: Vec<(&'static str, (f32, f32))>,
    /// Les filets qui séparent deux groupes d'une même rangée : `(x, y, hauteur)`.
    pub filets: Vec<(f32, f32, f32)>,
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
    fn groupes(&self, avec_mots: bool) -> [Vec<(Contenu, bool, Reglage)>; 4] {
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
                    let contenu = if avec_mots {
                        Contenu::Relation(p)
                    } else {
                        Contenu::Sigle(p)
                    };
                    (contenu, allume, reglage)
                })
                .collect(),
        ]
    }
}

/// **Les boutons de la barre**, par groupe — ou `None` s'il n'y a pas de flèche sélectionnée.
/// Quels boutons : cette fonction ; où les poser : [`layout_options_de_fleche`].
fn groupes_de_boutons(store: &Store, avec_mots: bool) -> Option<Vec<Vec<(Contenu, bool, Action)>>> {
    let fleches = store.selected_arrows();
    if fleches.is_empty() {
        return None;
    }
    let mut groupes: Vec<Vec<(Contenu, bool, Action)>> = EnCommun::de(&fleches)
        .groupes(avec_mots)
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
/// Fonction pure : elle ne lit que le store et la typographie, et ne dessine rien. Elle se pose
/// avec les mots des relations si elle tient dans l'écran, avec leurs sigles seuls sinon
/// (BADGE-2).
pub fn layout_options_de_fleche(
    store: &Store,
    typography: &Typography,
    screen: (f32, f32),
    scale: f32,
) -> Option<OptionsDeFleche> {
    let utile = super::bas::largeur_utile(screen, crate::theme::clamp_ui_scale(scale));
    // Les mots des relations, sur une rangée qui tient entre les bords de l'écran (BADGE-2) ;
    // sinon les sigles — sur une rangée s'ils y tiennent, et seulement alors passés à la ligne,
    // groupe par groupe : un téléphone debout n'a pas 700 points (fiche 101).
    let riche = poser_la_barre(store, typography, (screen, scale), (true, f32::INFINITY))?;
    if riche.rect.2 <= utile {
        return Some(riche);
    }
    let interieur = utile - PAD_X * crate::theme::clamp_ui_scale(scale) * 2.0;
    poser_la_barre(store, typography, (screen, scale), (false, interieur))
}

/// La barre posée, avec ou sans les mots des relations, ses rangées jamais plus longues que
/// `rangee_max`.
fn poser_la_barre(
    store: &Store,
    typography: &Typography,
    (screen, scale): ((f32, f32), f32),
    (avec_mots, rangee_max): (bool, f32),
) -> Option<OptionsDeFleche> {
    let groupes = groupes_de_boutons(store, avec_mots)?;
    // Seule sa hauteur compte ici, que l'origine ne change pas : un bouton à deux états a la
    // largeur de son plus long mot (fiche 101), donc la barre ses rangées.
    let dessous = super::action_bar::layout_action_bar(
        store,
        typography,
        (screen, scale),
        glucose_core::groupe::Origine::Commune,
    )?;
    let s = crate::theme::clamp_ui_scale(scale);
    let font = FONT * s;
    let mesure = |t: &str| typography.measure_text(t, font, Face::Regular).0;
    let hauteur_btn = font + BTN_PAD_Y * s * 2.0;
    let largeur_de = |c: &Contenu| match c {
        Contenu::Texte(t) => mesure(t) + BTN_PAD_X * s * 2.0,
        Contenu::Sigle(_) => hauteur_btn,
        Contenu::Relation(p) => (ICON + ICON_GAP) * s + mesure(p.label()) + BTN_PAD_X * s * 2.0,
        Contenu::Action(_, t) => (ICON + ICON_GAP) * s + mesure(t) + BTN_PAD_X * s * 2.0,
    };

    // Le titre, puis chaque groupe d'un bloc, son filet devant lui — comme un texte : un groupe
    // qui ne tient plus passe à la ligne, jamais coupé, et aucun filet n'ouvre une rangée.
    let (titre, epaisseur) = ("Flèche", "Épaisseur");
    let elements: Vec<(f32, f32)> = std::iter::once((0.0, mesure(titre)))
        .chain(groupes.iter().enumerate().map(|(rang, groupe)| {
            let mot = if rang == 2 {
                mesure(epaisseur) + GAP * s
            } else {
                0.0
            };
            let boutons: f32 = groupe.iter().map(|(c, _, _)| largeur_de(c) + GAP * s).sum();
            (GAP * s * 2.0 + 1.0, mot + boutons - GAP * s)
        }))
        .collect();
    let (places, plus_longue) = super::bas::ranger(&elements, rangee_max);
    let rangees = places.last().map_or(1, |&(r, _)| r + 1) as f32;
    let largeur = plus_longue + PAD_X * s * 2.0;
    let hauteur = rangees * hauteur_btn + (rangees - 1.0) * GAP * s + PAD_Y * s * 2.0;
    // Centrée, sur la barre d'action — et au-dessus de la minimap si elle la rencontrerait.
    let sol = dessous.rect.1 - super::bas::AU_DESSUS * s;
    let (x, y) = super::bas::poser(screen, s, (largeur, hauteur), sol);
    let a = |&(rangee, dx): &(usize, f32)| {
        let haut = y + PAD_Y * s + rangee as f32 * (hauteur_btn + GAP * s);
        (x + PAD_X * s + dx, haut)
    };
    let texte = |haut: f32| haut + (hauteur_btn - font) / 2.0;

    let (tx, ty) = a(&places[0]);
    let mut mots = vec![(titre, (tx, texte(ty)))];
    let (mut filets, mut boutons) = (Vec::new(), Vec::new());
    for ((rang, groupe), place) in groupes.into_iter().enumerate().zip(&places[1..]) {
        let (mut curseur, haut) = a(place);
        if place.1 > 0.0 {
            filets.push((curseur - GAP * s - 1.0, haut, hauteur_btn));
        }
        if rang == 2 {
            mots.push((epaisseur, (curseur, texte(haut))));
            curseur += mesure(epaisseur) + GAP * s;
        }
        let rangee = ((curseur, haut), (hauteur_btn, GAP * s));
        poser_le_groupe(groupe, rangee, &largeur_de, &mut boutons);
    }
    Some(OptionsDeFleche {
        rect: (x, y, largeur, hauteur),
        mots,
        filets,
        boutons,
    })
}

/// **Pose les boutons d'un groupe à la suite**, depuis `(x, haut)`, `ecart` entre deux.
fn poser_le_groupe(
    groupe: Vec<(Contenu, bool, Action)>,
    ((mut curseur, haut), (hauteur, ecart)): ((f32, f32), (f32, f32)),
    largeur_de: &impl Fn(&Contenu) -> f32,
    boutons: &mut Vec<Bouton>,
) {
    for (i, (contenu, allume, action)) in groupe.into_iter().enumerate() {
        if i > 0 {
            curseur += ecart;
        }
        let w = largeur_de(&contenu);
        boutons.push(Bouton {
            rect: (curseur, haut, w, hauteur),
            contenu,
            allume,
            action,
        });
        curseur += w;
    }
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
    let style = |color| TextStyle {
        size: font,
        color,
        face: Face::Regular,
    };
    for &(mot, (x, y)) in &barre.mots {
        typography.draw_text(pixmap, mot, x, y, style(theme.text_muted));
    }
    for &(x, y, h) in &barre.filets {
        filet_vertical(pixmap, x, y, h, theme.border_medium);
    }
    for b in &barre.boutons {
        dessiner_un_bouton(pixmap, (typography, theme), b, s);
    }
}

/// **Un bouton de la barre** : son cadre, puis ce qu'il montre — un mot, une icône et son mot,
/// un sigle, ou un sigle et son mot.
fn dessiner_un_bouton(
    pixmap: &mut PixmapMut,
    (typography, theme): (&Typography, &Theme),
    b: &Bouton,
    s: f32,
) {
    let font = FONT * s;
    bouton(pixmap, b.rect, (s, b.allume), theme);
    let encre = if b.allume {
        theme.text_accent
    } else {
        theme.text_secondary
    };
    let (bx, by, bw, bh) = b.rect;
    {
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
            // Le sigle à la place d'une icône, puis le mot : ce que la relation veut dire.
            Contenu::Relation(p) => {
                crate::renderer::predicate::draw_sigil(
                    pixmap,
                    p,
                    (bx + (BTN_PAD_X + ICON / 2.0) * s, by + bh / 2.0),
                    bh * 0.3,
                    theme.predicate_color(p),
                );
                Some((bx + (BTN_PAD_X + ICON + ICON_GAP) * s, p.label()))
            }
        };
        if let Some((x, t)) = texte {
            let style = TextStyle {
                size: font,
                color: encre,
                face: Face::Regular,
            };
            typography.draw_text(pixmap, t, x, by + (bh - font) / 2.0, style);
        }
    }
}

#[cfg(test)]
mod tests;
