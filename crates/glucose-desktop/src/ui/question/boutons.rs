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
    /// **Ce qu'on cherchait, dans le nom** (fiche 196) : ses tranches, en octets ; elles s'écrivent
    /// en clair sur le reste du nom, plus discret. Vide : le nom s'écrit d'une seule encre.
    pub eclats: Vec<(usize, usize)>,
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
        let ecrire = |pixmap: &mut PixmapMut, texte: &str, gauche: f32, (color, face)| {
            let haut = rangee.1 + (rangee.3 - p.corps.1) / 2.0;
            let style = TextStyle {
                size: p.corps.1,
                color,
                face,
            };
            typo.draw_text(pixmap, texte, gauche, haut, style);
        };
        if let Some((_, precision)) = p.precisions.iter().find(|(r, _)| r == reponse) {
            let marge = p.gauche - x;
            let rangee = (rangee, marge);
            rangee_precisee(
                pixmap,
                (typo, theme),
                p.corps.1,
                rangee,
                (libelle, precision),
                &ecrire,
            );
            continue;
        }
        let encre = if rang == 0 {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        let (lw, _) = typo.measure_text(libelle, p.corps.1, Face::Regular);
        ecrire(
            pixmap,
            libelle,
            rangee.0 + (rangee.2 - lw) / 2.0,
            (encre, Face::Regular),
        );
    }
}

/// L'encre et la graisse d'un morceau de texte.
type Encre = (Color, Face);

/// **Une rangée précisée** : son nom à gauche — ce qu'on cherchait en gras (fiche 196) —, sa
/// précision à droite, plus discrète. Introuvable, la rangée entière se dit en rouge.
fn rangee_precisee(
    pixmap: &mut PixmapMut,
    (typo, theme): (&Typography, &Theme),
    corps: f32,
    (rangee, marge): (&Rangee, f32),
    (libelle, precision): (&str, &Precision),
    ecrire: &dyn Fn(&mut PixmapMut, &str, f32, Encre),
) {
    let interieur = rangee.2 - 2.0 * marge;
    let avance = |c| typo.advance(c, corps, Face::Regular);
    let (libelle, eclats) = rapprochee(libelle, &precision.eclats, interieur, avance);
    let largeur = |t: &str, face| typo.measure_text(t, corps, face).0;
    // Ce que le gras ajoute au nom : la précision lui laisse cette place en plus.
    let surplus = (eclats.iter())
        .map(|&(a, b)| largeur(&libelle[a..b], Face::Bold) - largeur(&libelle[a..b], Face::Regular))
        .sum();
    let place = precisee(
        typo,
        corps,
        (rangee, marge, surplus),
        (&libelle, &precision.texte),
    );
    let (nom, encre) = match precision.introuvable {
        true => (theme.alert, theme.alert),
        false => (theme.text_primary, theme.text_muted),
    };
    let mut gauche = rangee.0 + marge;
    for (morceau, eclat) in morceaux(&place.nom, &eclats) {
        let face = if eclat { Face::Bold } else { Face::Regular };
        ecrire(pixmap, morceau, gauche, (nom, face));
        gauche += largeur(morceau, face);
    }
    let gauche = place.gauche_de_la_precision;
    ecrire(pixmap, &place.precision, gauche, (encre, Face::Regular));
}

/// **Le mot trouvé se lit** : une phrase dont le premier éclat tomberait au-delà du premier tiers de
/// la rangée perd son début, à une frontière de mot, et commence par « … » — le contexte d'avant
/// garde un tiers de la place, la suite le reste. Les éclats suivent le texte coupé.
pub(super) fn rapprochee(
    nom: &str,
    eclats: &[(usize, usize)],
    interieur: f32,
    avance: impl Fn(char) -> f32,
) -> (String, Vec<(usize, usize)>) {
    let Some(&(debut, _)) = eclats.first() else {
        return (nom.to_string(), Vec::new());
    };
    let contexte = interieur / 3.0;
    if nom[..debut].chars().map(&avance).sum::<f32>() <= contexte {
        return (nom.to_string(), eclats.to_vec());
    }
    let (mut depart, mut cumul) = (debut, 0.0);
    for (i, c) in nom[..debut].char_indices().rev() {
        cumul += avance(c);
        if cumul > contexte {
            break;
        }
        if c == ' ' {
            depart = i + 1;
        }
    }
    let coupee = format!("…{}", &nom[depart..]);
    let decale = |o: usize| o - depart + '…'.len_utf8();
    let eclats = (eclats.iter())
        .filter(|e| e.0 >= depart)
        .map(|&(a, b)| (decale(a), decale(b)))
        .collect();
    (coupee, eclats)
}

/// **Le nom en morceaux** : chacun, et s'il est un éclat. Ce que la coupe a ôté de la fin n'en a
/// plus ; « … » n'en est jamais un.
pub(super) fn morceaux<'a>(nom: &'a str, eclats: &[(usize, usize)]) -> Vec<(&'a str, bool)> {
    let garde = nom.strip_suffix('…').map_or(nom.len(), str::len);
    let mut out = Vec::new();
    let mut lu = 0;
    for &(a, b) in eclats {
        let (a, b) = (a.max(lu).min(garde), b.min(garde));
        if a >= b || !nom.is_char_boundary(a) || !nom.is_char_boundary(b) {
            continue;
        }
        out.push((&nom[lu..a], false));
        out.push((&nom[a..b], true));
        lu = b;
    }
    out.push((&nom[lu..], false));
    out.retain(|(m, _)| !m.is_empty());
    out
}

/// Une rangée précisée, placée : son nom et sa précision, coupés à ce qui tient.
struct Precisee {
    nom: String,
    precision: String,
    gauche_de_la_precision: f32,
}

/// **Le partage d'une rangée entre le nom et sa précision** : chacun en entier s'ils tiennent ;
/// sinon le nom garde au moins la moitié de la place, et la précision se coupe la première. Le
/// `surplus` est ce que le gras de ses éclats ajoute au nom, mesuré en romain.
fn precisee(
    typo: &Typography,
    corps: f32,
    (rangee, marge, surplus): (&Rangee, f32, f32),
    (nom, precision): (&str, &str),
) -> Precisee {
    let interieur = (rangee.2 - 2.0 * marge).max(1.0);
    let largeur = |t: &str| typo.measure_text(t, corps, Face::Regular).0;
    let avance = |c| typo.advance(c, corps, Face::Regular);
    let entre = marge;
    let pour_le_nom = (largeur(nom))
        .min((interieur - surplus - entre - largeur(precision)).max(interieur / 2.0 - surplus));
    let nom = crate::renderer::wrap::tronquer(nom, pour_le_nom, avance);
    let reste = (interieur - largeur(&nom) - surplus - entre).max(0.0);
    let precision = crate::renderer::wrap::tronquer(precision, reste, avance);
    let gauche_de_la_precision = rangee.0 + rangee.2 - marge - largeur(&precision);
    Precisee {
        nom,
        precision,
        gauche_de_la_precision,
    }
}
