//! **Les réponses d'une question, dessinées** : une rangée par réponse, sous un filet ; le survol,
//! l'évidence d'Entrée (POPUP-1) ; et, pour une réponse qui se situe (RECENTS-1, fiche 174), son
//! nom à gauche et sa précision à droite.

use super::{cerner, dans, filet, remplir, Placee, Rangee};
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use tiny_skia::{Color, PixmapMut};

/// La largeur d'une liste qui situe ses réponses : celle de la palette de VS Code (600 points),
/// où un nom et son dossier tiennent sur une ligne.
pub(super) const LARGEUR_LISTE: f32 = 600.0;

/// **Ce qu'une réponse dit de plus**, à droite de son nom, en encre discrète : le dossier d'un
/// document et depuis quand il n'a pas été touché. Introuvable, la rangée entière se dit en rouge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Precision {
    pub texte: String,
    pub introuvable: bool,
}

/// **Dessine les réponses.**
pub(super) fn dessiner(
    pixmap: &mut PixmapMut,
    p: &Placee,
    (typo, theme): (&Typography, &Theme),
    pointer: (f32, f32),
    s: f32,
) {
    let (x, _, largeur, _) = p.carte;
    for (rang, (rangee, libelle, reponse)) in p.boutons.iter().enumerate() {
        filet(pixmap, (x, rangee.1, largeur), theme.border_medium);
        if dans(*rangee, pointer.0, pointer.1) {
            remplir(pixmap, *rangee, 0.0, theme.bg_hover, None);
        }
        // Ce qu'Entrée donnerait : le filet blanc de la sélection, en retrait des filets.
        if rang == p.focus {
            cerner(pixmap, *rangee, (3.0 * s).round(), theme.selection_frame);
        }
        let ecrire = |pixmap: &mut PixmapMut, texte: &str, gauche: f32, color: Color| {
            let haut = rangee.1 + (rangee.3 - p.corps.1) / 2.0;
            let style = TextStyle {
                size: p.corps.1,
                color,
                face: Face::Regular,
            };
            typo.draw_text(pixmap, texte, gauche, haut, style);
        };
        if let Some((_, precision)) = p.precisions.iter().find(|(r, _)| r == reponse) {
            let marge = p.gauche - x;
            let place = precisee(
                typo,
                p.corps.1,
                (rangee, marge),
                (libelle, &precision.texte),
            );
            let (nom, encre) = match precision.introuvable {
                true => (theme.alert, theme.alert),
                false => (theme.text_primary, theme.text_muted),
            };
            ecrire(pixmap, &place.nom, rangee.0 + marge, nom);
            ecrire(
                pixmap,
                &place.precision,
                place.gauche_de_la_precision,
                encre,
            );
            continue;
        }
        let encre = if rang == 0 {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        let (lw, _) = typo.measure_text(libelle, p.corps.1, Face::Regular);
        ecrire(pixmap, libelle, rangee.0 + (rangee.2 - lw) / 2.0, encre);
    }
}

/// Une rangée précisée, placée : son nom et sa précision, coupés à ce qui tient.
struct Precisee {
    nom: String,
    precision: String,
    gauche_de_la_precision: f32,
}

/// **Le partage d'une rangée entre le nom et sa précision** : chacun en entier s'ils tiennent ;
/// sinon le nom garde au moins la moitié de la place, et la précision se coupe la première.
fn precisee(
    typo: &Typography,
    corps: f32,
    (rangee, marge): (&Rangee, f32),
    (nom, precision): (&str, &str),
) -> Precisee {
    let interieur = (rangee.2 - 2.0 * marge).max(1.0);
    let largeur = |t: &str| typo.measure_text(t, corps, Face::Regular).0;
    let avance = |c| typo.advance(c, corps, Face::Regular);
    let entre = marge;
    let pour_le_nom =
        largeur(nom).min((interieur - entre - largeur(precision)).max(interieur / 2.0));
    let nom = crate::renderer::wrap::tronquer(nom, pour_le_nom, avance);
    let reste = (interieur - largeur(&nom) - entre).max(0.0);
    let precision = crate::renderer::wrap::tronquer(precision, reste, avance);
    let gauche_de_la_precision = rangee.0 + rangee.2 - marge - largeur(&precision);
    Precisee {
        nom,
        precision,
        gauche_de_la_precision,
    }
}
