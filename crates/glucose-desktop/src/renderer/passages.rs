//! **Faire briller un passage d'une carte** (FLECHE-4) — ce qu'une flèche survolée y désigne,
//! ou ce que l'éditeur d'ancres y sélectionne.
//!
//! # Ce que Tauri faisait
//!
//! Au survol d'une flèche ancrée, le passage visé s'enveloppait d'un `<mark>` à la couleur de
//! sa carte (`HtmlAnnotationLayer.tsx`) : un fond à 20 %, un liseré à 45 % décalé de deux
//! pixels, une lueur floutée de 16 et étalée de 6 à 25 %, des coins de 3. Ce sont ces nombres,
//! en unités du monde : la carte de Tauri est mise à l'échelle avec tout ce qu'elle porte.
//!
//! Son défaut était ailleurs, et c'est lui que ce module ne reproduit pas : le passage est
//! désigné par ses **octets** ([`glucose_core::text_anchors`]), jamais par son mot. Deux
//! « bonjours » ne s'allument pas ensemble parce qu'on en a désigné un.
//!
//! # PASSAGE-2 — le cadre a sa place, le passage est du contenu
//!
//! Le cadre se posait par-dessus une mise en page qui ne savait rien de lui : il mordait sur
//! les lettres voisines, et la teinte, repeinte dans un rectangle découpé, en prenait la moitié
//! d'une (sa capture du 26/09). Désormais :
//!
//! * la mise en page **ouvre la place** du cadre ([`super::richtext::place`]) : ses voisins
//!   s'écartent d'exactement ce qui manque, sans que la coupe des lignes change ;
//! * le passage est du **contenu** de la carte : son fond et son liseré se peignent sous le
//!   texte, ses lettres à la teinte, fragment par fragment ([`peindre_les_cadres`]) — dans sa
//!   texture sur la voie graphique ;
//! * seule la **lueur** passe au-dessus ([`eclairer`]), découpée dans le cadre comme l'ombre
//!   CSS de Tauri : elle déborde de la carte, et une texture s'arrête à la carte.

use super::card::{text_box, MisesEnPage, TEXT_ORIGIN};
use super::halo::{draw_halo, HaloBox};
use super::hue::SymbioticHueCache;
use super::math::MathRenderer;
use super::richtext::hit::{offset_to_x, x_du_caractere};
use super::richtext::place::ouvrir_la_place;
use super::richtext::{font_of, indent_of, mode_of, TextLayout, TextMode, VisualLine};
use super::scale::WorldScale;
use crate::canvas::world_to_screen;
use crate::params::{Eclairage, ViewPass};
use crate::typography::{Face, Typography};
use glucose_core::membrane_forme::{Arrondi, Bord, Membrane};
use glucose_core::quadtree::{noeud_au_rang, Noeud};
use glucose_core::store::Store;
use glucose_core::text::BlockKind;
use glucose_core::types::{Annotation, Viewport};
use std::rc::Rc;
use tiny_skia::{Color, PixmapMut};

/// **Comment un passage brille** — les nombres de Tauri, en unités du monde : la carte de
/// Tauri se met à l'échelle avec tout ce qu'elle porte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Surlignage {
    /// Le fond, et le liseré, en opacité de la teinte.
    fond: f32,
    lisere: f32,
    /// L'épaisseur du liseré, et son décalage hors de la boîte (`outline-offset`).
    epaisseur: f32,
    decalage: f32,
    /// La marge autour du texte (`padding`), et les coins.
    marge: (f32, f32),
    coins: f32,
    /// La lueur (`box-shadow`) : son écart-type — la moitié du flou CSS —, son étalement, son
    /// opacité sur 255.
    lueur: (f32, f32, u8),
}

impl Surlignage {
    /// **De combien le cadre déborde de son passage**, de chaque côté : sa marge, son décalage,
    /// et la moitié extérieure de son liseré — ce que la ligne doit lui laisser (PASSAGE-2).
    pub(crate) fn etendue(&self) -> f32 {
        self.marge.0 + self.decalage + self.epaisseur / 2.0
    }
}

/// Au survol d'une flèche, sur la carte (`HtmlAnnotationLayer.tsx`) : fond à 20 %, liseré de
/// 1,5 à 45 % décalé de 2, marge `1px 3px`, coins de 3, lueur `0 0 16px 6px` à 25 %.
pub(crate) const SUR_LA_CARTE: Surlignage = Surlignage {
    fond: 0.20,
    lisere: 0.45,
    epaisseur: 1.5,
    decalage: 2.0,
    marge: (3.0, 1.0),
    coins: 3.0,
    lueur: (8.0, 6.0, 64),
};

/// Dans la fenêtre de l'éditeur (`ArrowTextEditor.tsx`) : fond à 25 %, liseré de 1 à 40 %
/// décalé de 1, marge `1px 4px`, lueur `0 0 8px` à 15 %.
pub(crate) const DANS_L_EDITEUR: Surlignage = Surlignage {
    fond: 0.25,
    lisere: 0.40,
    epaisseur: 1.0,
    decalage: 1.0,
    marge: (4.0, 1.0),
    coins: 3.0,
    lueur: (4.0, 0.0, 38),
};

/// **Des passages qui brillent dans une carte** : leurs plages en octets de sa source, leur
/// teinte, leur style.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Eclaires<'a> {
    pub plages: &'a [(usize, usize)],
    pub teinte: (u8, u8, u8),
    pub style: &'a Surlignage,
}

impl Eclaires<'_> {
    /// Une plage touche-t-elle `[debut, fin)` ?
    pub(crate) fn touchent(&self, debut: usize, fin: usize) -> bool {
        self.plages
            .iter()
            .any(|&(a, b)| a < fin.max(debut + 1) && b > debut)
    }
}

/// **Les passages qu'une image fait briller dans cette carte**, s'il y en a : à la teinte de
/// la carte, sauf si l'éclairage porte la sienne.
pub(crate) fn eclaires_de<'a>(
    eclairages: &'a [Eclairage],
    carte: &str,
    teinte: (u8, u8, u8),
) -> Option<Eclaires<'a>> {
    let e = eclairages.iter().find(|e| e.carte == carte)?;
    Some(Eclaires {
        plages: &e.plages,
        teinte: e.teinte.unwrap_or(teinte),
        style: &SUR_LA_CARTE,
    })
}

/// **La mise en page du texte d'une carte, la place de ses cadres ouverte** — la seule que le
/// tracé, le clic et les cadres lisent. Sans passage, ou pendant l'édition, c'est celle de la
/// carte, inchangée.
pub(crate) fn mise_en_page(
    (typographie, math, memoire): (&Typography, &MathRenderer, &MisesEnPage),
    (texte, largeur): (&str, f32),
    mode: TextMode,
    eclaires: Option<&Eclaires>,
) -> Rc<TextLayout> {
    let brute = memoire.de((typographie, math), texte, largeur, mode);
    match eclaires {
        Some(e) if mode == TextMode::Rendered && !e.plages.is_empty() => Rc::new(ouvrir_la_place(
            &brute,
            typographie,
            (texte, text_box(largeur)),
            e.plages,
            e.style.etendue(),
        )),
        _ => brute,
    }
}

/// **La part d'un passage sur une ligne** : sa boîte, en unités du monde depuis le coin de la
/// carte, et la ligne qui la porte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Troncon {
    pub rect: (f32, f32, f32, f32),
    pub ligne: usize,
}

/// **Les tronçons d'un passage**, un par ligne qu'il touche, dans `mise_en_page`.
///
/// # La boîte épouse le texte, pas la ligne
///
/// Sa hauteur est celle de la **police** — de sa montante à sa descente —, là où la ligne vaut
/// 1,4 corps : la typographie pose la base à `haut + corps`, donc tout ce qui dépasse la police
/// tombe sous le texte. Une boîte haute comme la ligne paraissait décalée vers le bas ; celle-ci,
/// comme le `<mark>` de Tauri, colle au texte. Les métriques sont celles d'Inter, communes à ses
/// quatre graisses.
///
/// # Une formule est un atome
///
/// Au repos, une formule ne se lit pas caractère par caractère : on ne désigne pas la moitié
/// d'une fraction. Un passage qui touche un paragraphe de formule en prend la boîte **dessinée**
/// — sa largeur et sa hauteur rendues —, et non la largeur de sa source, qu'on ne voit pas.
pub(crate) fn troncons(
    (typographie, math): (&Typography, &MathRenderer),
    (mise_en_page, texte, largeur): (&TextLayout, &str, f32),
    plages: &[(usize, usize)],
) -> Vec<Troncon> {
    let boite = text_box(largeur);
    let mut out = Vec::new();
    for (rang, ligne) in mise_en_page.lines.iter().enumerate() {
        let corps = font_of(ligne.kind, boite.body);
        let gauche = TEXT_ORIGIN.0 + indent_of(ligne.kind, boite.bullet_indent);
        let haut = TEXT_ORIGIN.1 + rang as f32 * boite.line_height;
        for &(debut, fin) in plages {
            if fin <= ligne.start || debut > ligne.end {
                continue;
            }
            if let Some(rect) = boite_d_une_formule(math, texte, ligne, (gauche, haut, corps)) {
                out.push(Troncon { rect, ligne: rang });
                break;
            }
            let (a, b) = (debut.max(ligne.start), fin.min(ligne.end));
            // Le cadre commence là où la première lettre est dessinée — après l'écart que sa
            // place a ouvert —, et finit là où finit la dernière, avant l'écart suivant.
            let x0 = x_du_caractere(typographie, mise_en_page, ligne, texte, a, corps);
            let x1 = offset_to_x(typographie, mise_en_page, ligne, texte, b, corps);
            if x1 > x0 {
                let (y, h) = cadre_de_la_police(typographie, haut, corps);
                out.push(Troncon {
                    rect: (gauche + x0, y, x1 - x0, h),
                    ligne: rang,
                });
            }
        }
    }
    out
}

/// **Les rectangles d'un passage** dans une carte large de `largeur` : ceux de ses tronçons,
/// dans la mise en page ouverte pour lui — ce que les épreuves visent.
#[cfg(test)]
pub(crate) fn rectangles(
    outils: (&Typography, &MathRenderer),
    texte: &str,
    largeur: f32,
    plages: &[(usize, usize)],
) -> Vec<(f32, f32, f32, f32)> {
    let eclaires = Eclaires {
        plages,
        teinte: (255, 255, 255),
        style: &SUR_LA_CARTE,
    };
    let memoire = MisesEnPage::default();
    let ouverte = mise_en_page(
        (outils.0, outils.1, &memoire),
        (texte, largeur),
        TextMode::Rendered,
        Some(&eclaires),
    );
    troncons(outils, (&ouverte, texte, largeur), plages)
        .into_iter()
        .map(|t| t.rect)
        .collect()
}

/// La boîte verticale de la police sur une ligne dont le haut est `haut` : de sa montante à sa
/// descente, autour de la base que la typographie pose à `haut + corps`.
fn cadre_de_la_police(typographie: &Typography, haut: f32, corps: f32) -> (f32, f32) {
    let (montante, descente) = typographie
        .font(Face::Regular)
        .horizontal_line_metrics(corps)
        .map_or((corps, 0.0), |m| (m.ascent, -m.descent));
    (haut + corps - montante, montante + descente)
}

/// La boîte **dessinée** d'une formule, si cette ligne en est la première : là où la carte la
/// pose (`draw_formula`), haute de ce qu'elle monte et descend.
fn boite_d_une_formule(
    math: &MathRenderer,
    texte: &str,
    ligne: &VisualLine,
    (gauche, haut, corps): (f32, f32, f32),
) -> Option<(f32, f32, f32, f32)> {
    let BlockKind::Math { display } = ligne.kind else {
        return None;
    };
    let source = &texte[ligne.start..ligne.end];
    let (formule, _) = glucose_core::text::block::formula(source)?;
    let (l, h, d) = math.measure(&source[formule], mode_of(display), corps)?;
    ligne.first.then_some((gauche, haut, l, h + d))
}

/// Où se pose ce qu'on peint : la carte en `origine` dans le monde, vue par `vp` à l'échelle
/// `echelle`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Pose {
    pub origine: (f64, f64),
    pub vp: Viewport,
    pub echelle: WorldScale,
}

impl Pose {
    /// La boîte d'un tronçon à l'écran, sans sa marge.
    fn a_l_ecran(&self, (rx, ry, rw, rh): (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
        let (sx, sy) = world_to_screen(
            self.origine.0 + f64::from(rx),
            self.origine.1 + f64::from(ry),
            &self.vp,
        );
        let s = self.echelle;
        (sx as f32, sy as f32, s.world(rw), s.world(rh))
    }

    /// La boîte d'un tronçon à l'écran, **marge comprise** : celle que le fond remplit, et dans
    /// laquelle la lueur se découpe.
    fn boite(&self, rect: (f32, f32, f32, f32), style: &Surlignage) -> (f32, f32, f32, f32) {
        let (x, y, l, h) = self.a_l_ecran(rect);
        let s = self.echelle;
        (
            x - s.world(style.marge.0),
            y - s.world(style.marge.1),
            l + s.world(2.0 * style.marge.0),
            h + s.world(2.0 * style.marge.1),
        )
    }
}

/// L'opacité d'une part du cadre, sur 255 : celle que les deux voies composent.
fn opacite(f: f32) -> u8 {
    (f * 255.0).round() as u8
}

/// **Le cadre d'un tronçon : une membrane** — un fond et un liseré d'une même teinte, le liseré
/// étant le trait de demi-largeur `demi` sur le bord du fond repoussé de `décalage`.
///
/// # CADRE-1 — une seule loi pour le cadre, sur les deux voies (fiche 73)
///
/// Le processeur le remplissait par `tiny-skia` (le liseré en anneau pair-impair) ; de près, la
/// carte le dessine par la loi des membranes. Deux lois pour un même bord : de dix niveaux sur
/// le liseré, et le bord du fond qui débordait de la lueur découpée par la loi exacte — plus
/// clair que le fond et que le dehors, sur toute sa longueur. C'est la loi de la brume
/// ([`super::card`]) : une membrane d'une couche, et un bord.
fn membrane_du_cadre(
    pose: Pose,
    troncon: &Troncon,
    ((r, g, b), style): ((u8, u8, u8), &Surlignage),
) -> Membrane {
    let s = pose.echelle;
    let (x, y, l, h) = pose.boite(troncon.rect, style);
    let fond = Arrondi::nouveau(x, y, l, h, s.world(style.coins));
    let teinte = [r, g, b].map(|v| f32::from(v) / 255.0);
    let mut cadre = Membrane::plein(fond, teinte, f32::from(opacite(style.fond)) / 255.0);
    cadre.bord = Bord {
        forme: fond.dilate(s.world(style.decalage)),
        demi_largeur: s.world(style.epaisseur) / 2.0,
        pointille: None,
        alpha: f32::from(opacite(style.lisere)) / 255.0,
    };
    cadre
}

/// **Le fond et le liseré de chaque tronçon** — ce que la carte porte sous le texte de ses
/// passages (PASSAGE-2), par la loi des membranes (CADRE-1). Leurs lettres se peignent ensuite à
/// la teinte, avec le reste du texte.
pub(crate) fn peindre_les_cadres(
    pixmap: &mut PixmapMut,
    pose: Pose,
    troncons: &[Troncon],
    teinte_et_style: ((u8, u8, u8), &Surlignage),
) {
    let (largeur, hauteur) = (pixmap.width(), pixmap.height());
    let (pixels, _) = pixmap.data_mut().as_chunks_mut::<4>();
    for t in troncons {
        let cadre = membrane_du_cadre(pose, t, teinte_et_style);
        glucose_core::membrane_forme::peindre(&cadre, pixels, largeur, hauteur);
    }
}

/// **Le fond et le liseré de chaque tronçon, en dessin vectoriel** (fiche 73) : la membrane même
/// de [`peindre_les_cadres`], sa couche puis son bord (CADRE-1).
pub(crate) fn cadres_en_vecteurs<V: super::vectoriel::Vectoriel>(
    v: &mut V,
    pose: Pose,
    troncons: &[Troncon],
    teinte_et_style: ((u8, u8, u8), &Surlignage),
) {
    for t in troncons {
        let cadre = membrane_du_cadre(pose, t, teinte_et_style);
        let [r, g, b] = cadre.teinte;
        let encre = |alpha: f32| Color::from_rgba(r, g, b, alpha).unwrap_or(Color::TRANSPARENT);
        let (fond, alpha) = cadre.remplissages[2];
        let boite = (
            fond.gauche,
            fond.haut,
            fond.droite - fond.gauche,
            fond.bas - fond.haut,
        );
        v.rectangle(boite, fond.rayon, encre(alpha));
        let bord = cadre.bord;
        v.anneau(bord.forme, bord.demi_largeur, encre(bord.alpha));
    }
}

/// **La lueur de chaque tronçon**, découpée à l'intérieur de sa boîte comme l'ombre CSS de
/// Tauri (`box-shadow` ne se peint que hors de la boîte qui la porte) : c'est ce qui passe
/// au-dessus de la carte, et déborde d'elle.
pub(crate) fn peindre_les_lueurs(
    pixmap: &mut PixmapMut,
    pose: Pose,
    troncons: &[Troncon],
    (teinte, style): ((u8, u8, u8), &Surlignage),
) {
    let s = pose.echelle;
    let (ecart_type, etalement, opacite) = style.lueur;
    let etale = s.world(etalement);
    for t in troncons {
        let (gauche, haut, largeur, hauteur) = pose.boite(t.rect, style);
        draw_halo(
            pixmap,
            HaloBox {
                left: gauche - etale,
                top: haut - etale,
                right: gauche + largeur + etale,
                bottom: haut + hauteur + etale,
                sigma: s.world(ecart_type),
                carte: Some(Arrondi::nouveau(
                    gauche,
                    haut,
                    largeur,
                    hauteur,
                    s.world(style.coins),
                )),
            },
            teinte,
            opacite,
        );
    }
}

/// **Une sélection en cours** — le glisser de l'éditeur d'ancres : un fond sous les lettres
/// exactement, sans marge, comme toute sélection de texte. Rien ne s'y écarte : le texte ne
/// bouge pas sous la souris pendant qu'on choisit.
pub(crate) fn peindre_une_selection(
    pixmap: &mut PixmapMut,
    pose: Pose,
    troncons: &[Troncon],
    ((r, g, b), style): ((u8, u8, u8), &Surlignage),
) {
    let couleur = Color::from_rgba8(r, g, b, (style.fond * 255.0).round() as u8);
    for t in troncons {
        let (x, y, l, h) = pose.a_l_ecran(t.rect);
        if let Some(rect) = tiny_skia::Rect::from_xywh(x, y, l, h) {
            super::scale::fill_crisp(pixmap, rect, couleur);
        }
    }
}

/// **La lueur des passages que la flèche survolée désigne**, chacun à la couleur de sa carte —
/// ou à la sienne, s'il en porte une. Le fond, le liseré et les lettres sont déjà dans la carte.
pub(super) fn eclairer(
    hue_cache: &mut SymbioticHueCache,
    (typographie, math, memoire): (&Typography, &MathRenderer, &MisesEnPage),
    pixmap: &mut PixmapMut,
    (store, eclairages, pass): (&Store, &[Eclairage], ViewPass<'_>),
) {
    let Some(board) = store.active_board() else {
        return;
    };
    for eclairage in eclairages {
        let par_l_index = pass
            .index
            .rang_de(&eclairage.carte)
            .and_then(|r| noeud_au_rang(board, r));
        let Some(Noeud::Annotation(carte)) = par_l_index else {
            continue;
        };
        let Annotation::Text { x, y, text, .. } = carte else {
            continue;
        };
        let Some((largeur, _)) = carte.size() else {
            continue;
        };
        let teinte = eclairage
            .teinte
            .unwrap_or_else(|| super::card::teinte_de_carte(hue_cache, carte, (pass.index, board)));
        let eclaires = Eclaires {
            plages: &eclairage.plages,
            teinte,
            style: &SUR_LA_CARTE,
        };
        let largeur = largeur as f32;
        let ouverte = mise_en_page(
            (typographie, math, memoire),
            (text, largeur),
            TextMode::Rendered,
            Some(&eclaires),
        );
        let t = troncons(
            (typographie, math),
            (&ouverte, text, largeur),
            eclaires.plages,
        );
        let pose = Pose {
            origine: (*x, *y),
            vp: pass.vp,
            echelle: pass.echelle(),
        };
        peindre_les_lueurs(pixmap, pose, &t, (teinte, &SUR_LA_CARTE));
    }
}

#[cfg(test)]
mod tests;
