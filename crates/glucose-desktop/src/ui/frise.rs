//! **FRISE-1 — la frise chronologique** (`Maj+R`, fiche 114) : la frise de Glucose Tauri
//! (`TemporalRuler.tsx`), en bas de l'écran. Une fenêtre ambre entre deux poignées garde une
//! période ; ce qui est daté hors d'elle s'estompe (ESTOMPE-1, `glucose_core::timeline::presence`),
//! ce qui n'est pas daté reste là.
//!
//! * **Ce qu'elle montre** : la bande des ères — chaque ère entière, à ses vraies dates, dans son
//!   couloir (FRISE-2, `glucose_core::timeline::couloirs` : deux ères qui se chevauchent vont sur
//!   deux rangées), en gris neutres : Tauri teintait chaque ère d'un hachage de son nom, une
//!   couleur qui ne disait rien (VISION § 2, *tout a une signification*) — ; les graduations ; et **la répartition des dates du document** le long de
//!   l'échelle (KeyLines, Kepler.gl) : sans elle, sur une échelle de cent millions d'années, on ne
//!   retrouve pas ses trois dates. L'ambre seul dit le temps : la fenêtre, ses poignées, les bornes.
//! * **Sa place** : une bande du bas, sur toute la largeur — l'écran visible s'en raccourcit
//!   ([`super::UiState::sous`]), comme du clavier d'Android : la minimap, les barres d'une
//!   sélection et le message se posent au-dessus d'elle, par leurs propres lois. Sur la capture de
//!   Tauri, la minimap recouvrait le bout de la frise.
//! * **La mise en page est une fonction pure** ([`placer`]) : le dessin et la main lisent la même
//!   (loi L4).

use super::action_bar::{bouton, pastille, BTN_PAD_Y, FONT, GAP, PAD_X, PAD_Y};
use super::minimap::BORD;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::store::Store;
use glucose_core::timeline::{
    eres_de_la_vue, format_year, profondeur, repartition, tick_step, EreDansLaBande,
    TemporalFilter, DEFAULT_ERAS,
};
use tiny_skia::{Color, PixmapMut};

/// La piste de Tauri (36), à l'échelle où les barres du bas ont pris leurs mesures (fiche 101 : un
/// corps de 11 devenu 13).
const PISTE: f32 = 42.0;
/// **La rangée d'un couloir d'ères** : une ligne de [`PETIT`] et son interligne ; un filet d'un
/// point entre deux rangées.
const ERE: f32 = 15.0;
const FILET: f32 = 1.0;
/// L'épaisseur dessinée d'une poignée : un trait ambre, qui ne cache pas la répartition — la main
/// la prend de plus loin ([`PRISE`]).
const POIGNEE: f32 = 5.0;
/// Le corps des graduations et des noms d'ères : celui des barres d'avant la fiche 101.
const PETIT: f32 = 11.0;
/// **Jusqu'où une poignée se prend** : la moitié des 48 points qu'une cible demande au doigt
/// (Material, et le menu de Glucose) — une poignée de 16 points se prend aussi au doigt.
const PRISE: f32 = 24.0;

/// Un rectangle `(x, y, largeur, hauteur)`, en points de l'écran.
pub type Boite = (f32, f32, f32, f32);

/// **La frise ouverte** : l'échelle qu'elle montre — indépendante de la fenêtre, pour zoomer
/// sans changer ce qu'on garde, comme chez Tauri — et ce que la main tient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frise {
    pub vue: TemporalFilter,
    pub tenue: Option<Tenue>,
}

/// Ce que la main tient : quoi, d'où elle est partie (en points), la fenêtre d'alors, et combien
/// d'années vaut un point de la piste — l'échelle ne bouge pas pendant qu'on tient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tenue {
    pub prise: Prise,
    pub depuis: f32,
    pub fenetre: TemporalFilter,
    pub par_point: f64,
}

/// Ce qui se prend dans la piste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prise {
    Debut,
    Fin,
    Fenetre,
}

/// Ce qui est sous la main, dans la frise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endroit {
    Fermer,
    Prise(Prise),
    /// Ailleurs dans la frise : rien ne se passe, mais rien ne descend au canevas.
    Ailleurs,
}

/// **La frise posée** : sa boîte, son en-tête, la croix, la bande des ères et la piste.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placee {
    pub rect: Boite,
    /// Où s'écrivent le titre, puis les bornes de la fenêtre : la ligne de base de l'en-tête.
    pub titre: (f32, f32),
    pub fermer: Boite,
    pub eres: Boite,
    pub piste: Boite,
    pub s: f32,
}

/// La hauteur d'une rangée de boutons des barres du bas.
fn rangee(s: f32) -> f32 {
    (FONT + 2.0 * BTN_PAD_Y) * s
}

/// **La hauteur de la bande des ères** : une rangée par couloir du catalogue ([`profondeur`]) —
/// fixe, calculée sur tout le catalogue plutôt que sur la vue : une frise qui grandirait pendant
/// qu'on la zoome ferait sauter la minimap et les barres au-dessus d'elle.
fn hauteur_des_eres() -> f32 {
    let n = profondeur(DEFAULT_ERAS);
    n as f32 * ERE + n.saturating_sub(1) as f32 * FILET
}

/// **Ce que la frise prend au bas de l'écran** : sa hauteur et le bord sous elle.
pub fn prise_au_bas(s: f32) -> f32 {
    rangee(s) + (2.0 * PAD_Y + 2.0 * GAP + hauteur_des_eres() + PISTE + BORD) * s
}

/// **Pose la frise** au bas de `ecran` — l'écran que le système laisse voir —, sur toute sa
/// largeur, au bord des barres du bas.
pub fn placer(ecran: (f32, f32), s: f32) -> Placee {
    let hauteur = prise_au_bas(s) - BORD * s;
    let rect = (
        BORD * s,
        ecran.1 - BORD * s - hauteur,
        ecran.0 - 2.0 * BORD * s,
        hauteur,
    );
    let (x, l) = (rect.0 + PAD_X * s, rect.2 - 2.0 * PAD_X * s);
    let haut = rect.1 + PAD_Y * s;
    let cote = rangee(s);
    let eres = (x, haut + cote + GAP * s, l, hauteur_des_eres() * s);
    Placee {
        rect,
        titre: (x, haut + BTN_PAD_Y * s),
        fermer: (x + l - cote, haut, cote, cote),
        eres,
        piste: (x, eres.1 + eres.3 + GAP * s, l, PISTE * s),
        s,
    }
}

/// **La boîte de la rangée `rangee`** de la bande des ères, comptée depuis le haut — le dessin et
/// l'épreuve la lisent tous deux.
pub fn rangee_d_ere(p: &Placee, rangee: usize) -> Boite {
    let (x, y, l, _) = p.eres;
    (x, y + rangee as f32 * (ERE + FILET) * p.s, l, ERE * p.s)
}

/// **Les couloirs de cette vue, serrés contre la piste** : la rangée de chaque couloir qui porte
/// une ère dans la vue — leur ordre gardé, le plus large en haut —, les rangées vides sous
/// l'en-tête plutôt qu'entre les ères et la piste qu'elles nomment. Une ère ne change de rangée
/// que quand un couloir paraît ou disparaît de la vue ; la frise, elle, ne change pas de hauteur.
pub fn rangees(eres: &[EreDansLaBande<'_>]) -> impl Fn(usize) -> usize {
    let mut pris: Vec<usize> = eres.iter().map(|e| e.couloir).collect();
    pris.sort_unstable();
    pris.dedup();
    let vides = profondeur(DEFAULT_ERAS).saturating_sub(pris.len());
    move |couloir| vides + pris.binary_search(&couloir).unwrap_or(0)
}

/// L'étendue de la vue, en années — jamais nulle.
fn etendue(vue: TemporalFilter) -> f64 {
    ((vue.end - vue.start) as f64 + 1.0).max(1.0)
}

/// **L'abscisse du début de l'année `an`** dans la piste ; une année occupe `[an, an + 1[`.
pub fn x_de(an: i64, vue: TemporalFilter, piste: Boite) -> f32 {
    piste.0 + ((an - vue.start) as f64 / etendue(vue) * f64::from(piste.2)) as f32
}

/// **L'année sous cette abscisse**, en fraction d'année.
pub fn an_de(x: f32, vue: TemporalFilter, piste: Boite) -> f64 {
    vue.start as f64 + f64::from(x - piste.0) / f64::from(piste.2.max(1.0)) * etendue(vue)
}

/// Les deux bords de la fenêtre dans la piste : le début de sa première année, la fin de sa
/// dernière.
pub fn bords(fenetre: TemporalFilter, vue: TemporalFilter, piste: Boite) -> (f32, f32) {
    (
        x_de(fenetre.start, vue, piste),
        x_de(fenetre.end + 1, vue, piste),
    )
}

fn dans((x, y, l, h): Boite, px: f32, py: f32) -> bool {
    px >= x && px < x + l && py >= y && py < y + h
}

/// **Ce qui est sous la main** : la croix, une poignée — la plus proche, à moins de [`PRISE`] —,
/// la fenêtre, ou ailleurs dans la frise ; rien hors d'elle.
pub fn endroit(
    p: &Placee,
    (vue, fenetre): (TemporalFilter, TemporalFilter),
    (x, y): (f32, f32),
) -> Option<Endroit> {
    if !dans(p.rect, x, y) {
        return None;
    }
    if dans(p.fermer, x, y) {
        return Some(Endroit::Fermer);
    }
    let haut = p.eres.1;
    if y < haut || y >= p.piste.1 + p.piste.3 {
        return Some(Endroit::Ailleurs);
    }
    let (a, b) = bords(fenetre, vue, p.piste);
    let (da, db) = ((x - a).abs(), (x - b).abs());
    let portee = PRISE * p.s;
    if da.min(db) <= portee {
        let debut = da < db || (da == db && x < a);
        return Some(Endroit::Prise(if debut {
            Prise::Debut
        } else {
            Prise::Fin
        }));
    }
    if x > a && x < b {
        return Some(Endroit::Prise(Prise::Fenetre));
    }
    Some(Endroit::Ailleurs)
}

/// **Dessine la frise** : sa pastille, l'en-tête et ses bornes, la croix, la bande des ères, la
/// piste — ses graduations, la répartition des dates, la fenêtre et ses poignées.
pub fn dessiner(
    pixmap: &mut PixmapMut,
    store: &Store,
    (reglette, fenetre): (&Frise, TemporalFilter),
    (typo, theme): (&Typography, &Theme),
    p: &Placee,
) {
    let s = p.s;
    pastille(pixmap, p.rect, s, theme);
    let corps = |taille: f32, color: Color| TextStyle {
        size: taille * s,
        color,
        face: Face::Regular,
    };
    let titre = "Frise chronologique";
    let avance = typo.draw_text(
        pixmap,
        titre,
        p.titre.0,
        p.titre.1,
        corps(FONT, theme.text_muted),
    );
    let bornes = format!(
        "{} → {}",
        format_year(fenetre.start),
        format_year(fenetre.end)
    );
    let x = p.titre.0 + avance.max(typo.measure_text(titre, FONT * s, Face::Regular).0);
    typo.draw_text(
        pixmap,
        &bornes,
        x + 2.0 * GAP * s,
        p.titre.1,
        corps(FONT, theme.temps.encre),
    );
    bouton(pixmap, p.fermer, (s, false), theme);
    let (cl, ch) = typo.measure_text("×", FONT * s, Face::Regular);
    let at = (
        p.fermer.0 + (p.fermer.2 - cl) / 2.0,
        p.fermer.1 + (p.fermer.3 - ch) / 2.0,
    );
    typo.draw_text(pixmap, "×", at.0, at.1, corps(FONT, theme.text_secondary));
    dessiner_les_eres(pixmap, reglette.vue, (typo, theme), p);
    dessiner_la_piste(pixmap, store, (reglette.vue, fenetre), (typo, theme), p);
}

/// Un rectangle plein, net.
fn remplir(pixmap: &mut PixmapMut, (x, y, l, h): Boite, couleur: Color) {
    if let Some(r) = tiny_skia::Rect::from_xywh(x, y, l, h) {
        crate::renderer::scale::fill_crisp(pixmap, r, couleur);
    }
}

/// **La bande des ères** : chaque ère dans la rangée de son couloir ([`rangees`]), d'un gris
/// neutre — dans un couloir, l'une sur deux plus claire, pour qu'on voie où l'une finit ; son nom
/// s'il tient.
fn dessiner_les_eres(
    pixmap: &mut PixmapMut,
    vue: TemporalFilter,
    (typo, theme): (&Typography, &Theme),
    p: &Placee,
) {
    let s = p.s;
    let bande = p.eres;
    let eres = eres_de_la_vue(vue, DEFAULT_ERAS);
    let rangee_de = rangees(&eres);
    for ere in eres.iter() {
        let rangee = rangee_d_ere(p, rangee_de(ere.couloir));
        let (a, b) = bords(ere.paru, vue, bande);
        let (a, b) = (a.max(bande.0), b.min(bande.0 + bande.2));
        let fond = if ere.rang % 2 == 0 {
            theme.btn_bg
        } else {
            theme.bg_hover
        };
        remplir(pixmap, (a, rangee.1, b - a, rangee.3), fond);
        if let Some(nom) = ce_qui_tient(typo, ere.ere.name, PETIT * s, b - a - 2.0 * GAP * s) {
            let h = typo.measure_text(&nom, PETIT * s, Face::Regular).1;
            let style = TextStyle {
                size: PETIT * s,
                color: theme.text_secondary,
                face: Face::Regular,
            };
            let y = rangee.1 + (rangee.3 - h) / 2.0;
            typo.draw_text(pixmap, &nom, a + GAP * s, y, style);
        }
    }
}

/// **Ce qui tient d'un nom dans cette largeur** : le nom entier, ou son début et « … » — comme
/// la frise de Tauri (`text-overflow: ellipsis`) ; rien s'il ne tient pas même une lettre. Une ère
/// coupée par le bord de la vue, ou trop courte à cette échelle, garde de quoi la reconnaître.
fn ce_qui_tient(typo: &Typography, nom: &str, corps: f32, largeur: f32) -> Option<String> {
    let mesure = |t: &str| typo.measure_text(t, corps, Face::Regular).0;
    if mesure(nom) <= largeur {
        return Some(nom.to_string());
    }
    let lettres: Vec<char> = nom.chars().collect();
    (1..lettres.len())
        .rev()
        .map(|n| format!("{}…", lettres[..n].iter().collect::<String>().trim_end()))
        .find(|t| mesure(t) <= largeur)
}

/// **La piste** : son fond, les graduations — une majeure sur cinq, nommée —, la répartition des
/// dates du document en barres grises, puis la fenêtre ambre et ses deux poignées.
fn dessiner_la_piste(
    pixmap: &mut PixmapMut,
    store: &Store,
    (vue, fenetre): (TemporalFilter, TemporalFilter),
    (typo, theme): (&Typography, &Theme),
    p: &Placee,
) {
    let (s, piste) = (p.s, p.piste);
    remplir(pixmap, piste, theme.bg_card);
    let pas = tick_step(vue.end - vue.start);
    let etiquette = (PETIT + 2.0) * s;
    let comptes = repartition(store.dates_du_tableau(), vue, pas);
    let plus = comptes.iter().copied().max().unwrap_or(0).max(1) as f32;
    let premier = vue.start.div_euclid(pas);
    for (k, n) in comptes.iter().enumerate() {
        let debut = (premier + k as i64) * pas;
        let (a, b) = (x_de(debut, vue, piste), x_de(debut + pas, vue, piste));
        let (a, b) = (a.max(piste.0), b.min(piste.0 + piste.2));
        let majeure = (premier + k as i64).rem_euclid(5) == 0;
        let trait_ = if majeure {
            theme.border_medium
        } else {
            theme.border_subtle
        };
        if x_de(debut, vue, piste) >= piste.0 {
            remplir(pixmap, (a, piste.1, 1.0, piste.3), trait_);
            if majeure {
                let style = TextStyle {
                    size: PETIT * s,
                    color: theme.text_muted,
                    face: Face::Regular,
                };
                // Un libellé qui sortirait de la piste ne s'écrit pas.
                let mot = format_year(debut);
                let l = typo.measure_text(&mot, PETIT * s, Face::Regular).0;
                if a + 2.0 * s + l <= piste.0 + piste.2 {
                    typo.draw_text(pixmap, &mot, a + 2.0 * s, piste.1 + s, style);
                }
            }
        }
        let haut = *n as f32 / plus * (piste.3 - etiquette);
        if *n > 0 {
            let marge = ((b - a) * 0.15).min(2.0 * s);
            let barre = (
                a + marge,
                piste.1 + piste.3 - haut,
                (b - a - 2.0 * marge).max(1.0),
                haut,
            );
            remplir(pixmap, barre, theme.text_muted);
        }
    }
    let (a, b) = bords(fenetre, vue, piste);
    let (a2, b2) = (a.max(piste.0), b.min(piste.0 + piste.2));
    if b2 > a2 {
        remplir(pixmap, (a2, piste.1, b2 - a2, piste.3), theme.temps.voile);
        remplir(pixmap, (a2, piste.1, b2 - a2, 1.0), theme.temps.ambre);
        remplir(
            pixmap,
            (a2, piste.1 + piste.3 - 1.0, b2 - a2, 1.0),
            theme.temps.ambre,
        );
    }
    for bord in [a, b] {
        if bord >= piste.0 && bord <= piste.0 + piste.2 {
            let l = POIGNEE * s;
            remplir(
                pixmap,
                (bord - l / 2.0, piste.1, l, piste.3),
                theme.temps.ambre,
            );
        }
    }
}

#[cfg(test)]
mod tests;
