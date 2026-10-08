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
use glucose_core::text::Selection;
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
    /// Ce qu'on fait de ce document, après un appui long dans la liste (DOCUMENTS-2).
    Gerer(std::path::PathBuf),
    /// Le renommer, du nom écrit dans le champ.
    Renommer(std::path::PathBuf),
    /// Le supprimer — c'est définitif.
    Supprimer(std::path::PathBuf),
}

/// **Un champ de texte** dans une question — le nom d'un document qu'on renomme
/// (DOCUMENTS-2). Le clavier du système y écrit par le miroir (CLAVIER-1) ; la question le
/// dessine.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Champ {
    pub texte: String,
    pub selection: Selection,
}

/// **Une question** : son titre, son texte, et ses réponses, dans l'ordre où elles s'affichent
/// — chacune avec son libellé, qui dit ce qu'elle fait.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Question {
    pub titre: String,
    pub texte: String,
    pub choix: Vec<(String, Reponse)>,
    /// **La réponse pressée**, qui ne part que si le bouton se relève dessus (APPUI-1) —
    /// comme un bouton d'Android : un doigt qui glisse ailleurs, ou qui tient jusqu'à l'appui
    /// long, n'a rien répondu. Et le doigt qui ouvre la question depuis un menu ne lui répond
    /// pas en se levant.
    pub sous_le_doigt: Option<Reponse>,
    /// Le champ où l'on écrit, sous le texte, s'il y en a un.
    pub champ: Option<Champ>,
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
    /// Le champ, s'il y en a un : sa rangée, et ce qu'il porte.
    pub champ: Option<(Rangee, Champ)>,
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
    // Le champ prend la hauteur d'une réponse : la cible d'un doigt.
    let du_champ = if q.champ.is_some() { BOUTON * s } else { 0.0 };
    let hauteur = PAD * s * 2.0 + haut_du_texte + du_champ + q.choix.len() as f32 * BOUTON * s;
    let x = (w - largeur) / 2.0;
    let y = ((h - hauteur) / 2.0).max(MARGE_ECRAN * s);
    let (titre, apres_le_titre) = superposer(titre, y + PAD * s, pas_titre);
    let (texte, _) = superposer(texte, apres_le_titre + entre, pas_texte);
    let champ = q.champ.as_ref().map(|c| {
        let haut = y + PAD * s * 1.5 + haut_du_texte;
        let rangee = (x + PAD * s, haut, largeur - 2.0 * PAD * s, BOUTON * s);
        (rangee, c.clone())
    });
    let mut haut = y + PAD * s * 2.0 + haut_du_texte + du_champ;
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
        champ,
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

/// Ce point tombe-t-il dans le champ ?
pub fn champ_sous(p: &Placee, px: f32, py: f32) -> bool {
    p.champ.as_ref().is_some_and(|(r, _)| dans(*r, px, py))
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
    if let Some((rangee, champ)) = &p.champ {
        dessiner_le_champ(pixmap, (*rangee, champ), (typo, theme), p.corps.1, s);
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

/// **Le champ** : un cadre plat, le texte qui glisse pour garder la tête de la sélection en
/// vue, la sélection surlignée, le curseur quand elle est vide.
fn dessiner_le_champ(
    pixmap: &mut PixmapMut,
    ((x, y, w, h), champ): (Rangee, &Champ),
    (typo, theme): (&Typography, &Theme),
    corps: f32,
    s: f32,
) {
    remplir(
        pixmap,
        (x, y, w, h),
        RAYON * s,
        theme.bg_canvas,
        Some(theme.border_medium),
    );
    let texte = champ.texte.as_str();
    let borne = |i: usize| {
        let mut i = i.min(texte.len());
        while !texte.is_char_boundary(i) {
            i -= 1;
        }
        i
    };
    let (ancre, tete) = (borne(champ.selection.anchor), borne(champ.selection.head));
    let mesure = |a: usize, b: usize| typo.measure_text(&texte[a..b], corps, Face::Regular).0;
    let marge = PAD * s / 2.0;
    let (debut, fin) = fenetre(texte, tete, (w - 2.0 * marge).max(1.0), mesure);
    let (gauche, haut) = (x + marge, y + (h - corps) / 2.0);
    let (bas, haut_sel) = (
        ancre.min(tete).clamp(debut, fin),
        ancre.max(tete).clamp(debut, fin),
    );
    let trait_ = |a: f32, b: f32| Rect::from_xywh(a, haut - corps * 0.15, b - a, corps * 1.3);
    if bas < haut_sel {
        let (a, b) = (
            gauche + mesure(debut, bas),
            gauche + mesure(debut, haut_sel),
        );
        if let Some(rect) = trait_(a, b) {
            crate::renderer::scale::fill_crisp(pixmap, rect, theme.text_selection);
        }
    }
    let style = TextStyle {
        size: corps,
        color: theme.text_primary,
        face: Face::Regular,
    };
    typo.draw_text(pixmap, &texte[debut..fin], gauche, haut, style);
    if ancre == tete {
        let a = gauche + mesure(debut, tete);
        if let Some(rect) = trait_(a, a + s.max(1.0)) {
            crate::renderer::scale::fill_crisp(pixmap, rect, theme.text_primary);
        }
    }
}

/// **La part du texte qui tient dans `place` et montre la tête** : on retire au début ce qui
/// pousse la tête hors du champ, puis à la fin ce qui dépasse. Des bornes de caractères.
pub fn fenetre(
    texte: &str,
    tete: usize,
    place: f32,
    mesure: impl Fn(usize, usize) -> f32,
) -> (usize, usize) {
    let suivant = |i: usize| texte[i..].chars().next().map_or(i, |c| i + c.len_utf8());
    let precedent = |i: usize| {
        texte[..i]
            .chars()
            .next_back()
            .map_or(i, |c| i - c.len_utf8())
    };
    let mut debut = 0;
    while debut < tete && mesure(debut, tete) > place {
        debut = suivant(debut);
    }
    let mut fin = texte.len();
    while fin > tete && mesure(debut, fin) > place {
        fin = precedent(fin);
    }
    (debut, fin)
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
