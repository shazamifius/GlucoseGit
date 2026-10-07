//! **QUESTION-1 — la question que Glucose dessine lui-même** (fiche 56).
//!
//! # Pourquoi
//!
//! Glucose pose cinq questions — un document neuf, un travail non enregistré, une mise à jour,
//! le journal technique deux fois — par les dialogues du système (`rfd`). Android n'en a
//! aucun qu'on puisse appeler ainsi, et `dialogue::oui_ou_non` y répond « non » : sur son
//! téléphone, « Nouveau document » ne faisait rien, et la question de la télémétrie ne se
//! posait jamais.
//!
//! La question se dessine donc ici, dans le langage de `style.md` — un voile, une carte plate,
//! un filet, du texte, des réponses en rangées de 48 points, la cible d'un doigt. Elle ne
//! **bloque** rien : elle s'affiche, et la réponse arrive plus tard, au toucher. Ce qu'elle
//! décide s'exécute alors par sa **suite** ([`crate::interactions::question`]).
//!
//! # Ce module est pur
//!
//! Il place ([`placer`]), dit quelle réponse est sous le doigt ([`reponse_sous`]) et dessine
//! ([`dessiner`]) : le dessin et le toucher lisent la même mise en page (loi L4).

use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Transform};

/// L'écart minimal entre la carte et le bord de l'écran.
const MARGE_ECRAN: f32 = 16.0;
/// La carte ne s'élargit pas au-delà : une ligne plus longue se lit mal.
const LARGEUR_MAX: f32 = 420.0;
/// La marge intérieure de la carte.
const PAD: f32 = 20.0;
/// Le corps du titre, puis celui du texte — ceux des dialogues d'Android.
const TITRE: f32 = 17.0;
const CORPS: f32 = 15.0;
/// L'interligne, en corps.
const INTERLIGNE: f32 = 1.4;
/// Une réponse : la cible d'un doigt (Android, Material), qui convient aussi à la souris.
const BOUTON: f32 = 48.0;
/// Le rayon des coins, celui du menu et de la barre d'action.
const RAYON: f32 = 6.0;

/// Ce qu'on peut répondre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reponse {
    Oui,
    Non,
    Annuler,
    /// Le n-ième élément d'une liste — un document à ouvrir.
    Choix(usize),
}

/// **Ce qu'une réponse déclenche** : la suite de la question, que
/// [`crate::interactions::question`] exécute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Suite {
    /// « Nouveau document » : oui, il naît (NOUVEAU-1).
    NouveauDocument,
    /// Le journal technique : « oui » l'envoie, « non » l'arrête et l'efface.
    Telemetrie,
    /// Ouvrir l'un de ces documents (DOCUMENTS-1), dans l'ordre de la liste.
    Ouvrir(Vec<std::path::PathBuf>),
}

/// **Une question** : son titre, son texte, et ses réponses, dans l'ordre où elles s'affichent
/// — chacune avec son libellé, qui dit ce qu'elle fait.
#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    pub titre: String,
    pub texte: String,
    pub choix: Vec<(String, Reponse)>,
}

/// Un rectangle de l'écran : gauche, haut, largeur, hauteur.
pub type Rangee = (f32, f32, f32, f32);

/// **La question, placée à l'écran.**
#[derive(Clone, Debug, PartialEq)]
pub struct Placee {
    /// La carte.
    pub carte: (f32, f32, f32, f32),
    /// Chaque ligne du titre, et le haut de son texte.
    pub titre: Vec<(String, f32)>,
    /// Chaque ligne du texte, et le haut de son texte.
    pub texte: Vec<(String, f32)>,
    /// Chaque réponse : sa rangée, son libellé, ce qu'elle répond.
    pub boutons: Vec<(Rangee, String, Reponse)>,
    /// Les corps du titre et du texte, et le bord gauche du texte.
    pub corps: (f32, f32),
    pub gauche: f32,
}

/// **Place la question** au centre de l'écran, à la largeur que l'écran laisse.
pub fn placer(q: &Question, typo: &Typography, (w, h): (f32, f32), scale: f32) -> Placee {
    let s = crate::theme::clamp_ui_scale(scale);
    let (corps_titre, corps_texte) = (TITRE * s, CORPS * s);
    let largeur = (w - 2.0 * MARGE_ECRAN * s).min(LARGEUR_MAX * s).max(1.0);
    let interieur = (largeur - 2.0 * PAD * s).max(1.0);
    let titre = lignes(typo, &q.titre, (corps_titre, Face::Bold), interieur);
    let texte = lignes(typo, &q.texte, (corps_texte, Face::Regular), interieur);
    let (pas_titre, pas_texte) = (corps_titre * INTERLIGNE, corps_texte * INTERLIGNE);
    let entre = if titre.is_empty() { 0.0 } else { PAD * s / 2.0 };
    let haut_du_texte = titre.len() as f32 * pas_titre + entre + texte.len() as f32 * pas_texte;
    let hauteur = PAD * s * 2.0 + haut_du_texte + q.choix.len() as f32 * BOUTON * s;
    let x = (w - largeur) / 2.0;
    let y = ((h - hauteur) / 2.0).max(MARGE_ECRAN * s);
    let (titre, apres_le_titre) = superposer(titre, y + PAD * s, pas_titre);
    let (texte, _) = superposer(texte, apres_le_titre + entre, pas_texte);
    let mut haut = y + PAD * s * 2.0 + haut_du_texte;
    let boutons = q
        .choix
        .iter()
        .map(|(libelle, reponse)| {
            let rangee = (x, haut, largeur, BOUTON * s);
            haut += BOUTON * s;
            (rangee, libelle.clone(), *reponse)
        })
        .collect();
    Placee {
        carte: (x, y, largeur, hauteur),
        titre,
        texte,
        boutons,
        corps: (corps_titre, corps_texte),
        gauche: x + PAD * s,
    }
}

/// Les lignes, l'une sous l'autre depuis `haut`, et le haut de ce qui les suit.
fn superposer(lignes: Vec<String>, haut: f32, pas: f32) -> (Vec<(String, f32)>, f32) {
    let n = lignes.len() as f32;
    let posees = lignes
        .into_iter()
        .enumerate()
        .map(|(i, l)| (l, haut + i as f32 * pas))
        .collect();
    (posees, haut + n * pas)
}

/// **Le texte coupé en lignes** qui tiennent dans `largeur` (WRAP-1). Un retour à la ligne
/// du texte en commence une ; une ligne vide sépare deux paragraphes ; les blancs d'une ligne
/// se lisent comme un seul.
fn lignes(typo: &Typography, texte: &str, (corps, face): (f32, Face), largeur: f32) -> Vec<String> {
    let mut sortie = Vec::new();
    for brute in texte.lines() {
        let ligne = brute.split_whitespace().collect::<Vec<_>>().join(" ");
        if ligne.is_empty() {
            if sortie.last().is_some_and(|l: &String| !l.is_empty()) {
                sortie.push(String::new());
            }
            continue;
        }
        let avance = |_: usize, c: char| typo.advance(c, corps, face);
        for (debut, fin) in crate::renderer::wrap::wrap_paragraph(&ligne, largeur, avance) {
            sortie.push(ligne[debut..fin].to_string());
        }
    }
    while sortie.last().is_some_and(String::is_empty) {
        sortie.pop();
    }
    sortie
}

/// La réponse sous ce point, s'il y en a une.
pub fn reponse_sous(p: &Placee, px: f32, py: f32) -> Option<Reponse> {
    p.boutons
        .iter()
        .find(|(r, _, _)| dans(*r, px, py))
        .map(|(_, _, reponse)| *reponse)
}

fn dans((x, y, w, h): (f32, f32, f32, f32), px: f32, py: f32) -> bool {
    px >= x && px <= x + w && py >= y && py <= y + h
}

/// **Dessine la question** : le voile sur tout l'écran, la carte, le texte, les réponses — la
/// première en blanc, les autres en gris ; celle sous le pointeur, éclairée.
pub fn dessiner(
    pixmap: &mut PixmapMut,
    p: &Placee,
    (typo, theme): (&Typography, &Theme),
    pointer: (f32, f32),
    scale: f32,
) {
    let s = crate::theme::clamp_ui_scale(scale);
    // Le voile : ce qui est derrière attend la réponse.
    let (w, h) = (pixmap.width() as f32, pixmap.height() as f32);
    if let Some(tout) = Rect::from_xywh(0.0, 0.0, w, h) {
        crate::renderer::scale::fill_crisp(pixmap, tout, Color::from_rgba8(0, 0, 0, 150));
    }
    let (x, _, largeur, _) = p.carte;
    remplir(
        pixmap,
        p.carte,
        RAYON * s,
        theme.bg_panel,
        Some(theme.border_medium),
    );
    let ecrire = |pixmap: &mut PixmapMut, texte: &str, (gauche, haut), style: TextStyle| {
        typo.draw_text(pixmap, texte, gauche, haut, style);
    };
    for (ligne, haut) in &p.titre {
        let style = TextStyle {
            size: p.corps.0,
            color: theme.text_primary,
            face: Face::Bold,
        };
        ecrire(pixmap, ligne, (p.gauche, *haut), style);
    }
    for (ligne, haut) in &p.texte {
        let style = TextStyle {
            size: p.corps.1,
            color: theme.text_secondary,
            face: Face::Regular,
        };
        ecrire(pixmap, ligne, (p.gauche, *haut), style);
    }
    for (rang, (rangee, libelle, _)) in p.boutons.iter().enumerate() {
        filet(pixmap, (x, rangee.1, largeur), theme.border_medium);
        if dans(*rangee, pointer.0, pointer.1) {
            remplir(pixmap, *rangee, 0.0, theme.bg_hover, None);
        }
        let encre = if rang == 0 {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        let (lw, _) = typo.measure_text(libelle, p.corps.1, Face::Regular);
        let gauche = rangee.0 + (rangee.2 - lw) / 2.0;
        let haut = rangee.1 + (rangee.3 - p.corps.1) / 2.0;
        let style = TextStyle {
            size: p.corps.1,
            color: encre,
            face: Face::Regular,
        };
        ecrire(pixmap, libelle, (gauche, haut), style);
    }
}

/// Un filet d'un pixel sur toute la largeur de la carte, sur la grille de pixels.
fn filet(pixmap: &mut PixmapMut, (x, y, largeur): (f32, f32, f32), couleur: Color) {
    if let Some(rect) = Rect::from_xywh(x, y, largeur, 1.0) {
        crate::renderer::scale::fill_crisp(pixmap, rect, couleur);
    }
}

/// Un rectangle aux coins arrondis, rempli, et cerné d'un filet s'il en a un.
fn remplir(
    pixmap: &mut PixmapMut,
    (x, y, w, h): (f32, f32, f32, f32),
    rayon: f32,
    fond: Color,
    bord: Option<Color>,
) {
    let mut pb = PathBuilder::new();
    crate::renderer::push_rounded_rect(&mut pb, x, y, w, h, rayon);
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(fond);
    pixmap.fill_path(
        &path,
        &paint,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );
    if let Some(bord) = bord {
        paint.set_color(bord);
        let trait_fin = tiny_skia::Stroke {
            width: 1.0,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &trait_fin, Transform::identity(), None);
    }
}

#[cfg(test)]
mod tests;
