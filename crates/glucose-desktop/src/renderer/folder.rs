//! Le dessin des dossiers de canevas (fiche 06 § 8).
//!
//! # Ce que ce module répare
//!
//! Un dossier est décrit dans la fiche 08 § 5 comme *« un portail vers un univers complet »*.
//! Il existait dans le document, répondait au clic — `hit_priority` connaît son bandeau et son
//! bord depuis longtemps — et **n'avait aucun pixel**. Le renderer ne lisait pas
//! `board.folders`. C'est le constat le plus visible de la relecture du dossier 06–10.
//!
//! # Les proportions
//!
//! Tout vient de la fiche 06 § 8.1, et la seule mesure que le noyau connaissait déjà —
//! [`pick_consts::FOLDER_HEADER`] — reste la source du bandeau : le dessin et le clic ne
//! peuvent pas diverger s'ils lisent la même constante.
//!
//! Le corps est teinté par la couleur du dossier à très faible opacité, comme la référence :
//! c'est un cadre qui contient, pas un objet qui s'affirme. La sélection double cette
//! opacité, remplace le pointillé par un trait plein et ajoute un halo — la même grammaire
//! que les membranes, qui sont l'autre conteneur du canevas.
//!
//! # Ce que la relecture contre Tauri a corrigé (fiche 70)
//!
//! `FolderSvgLayer.tsx` écrit le titre **sur sa ligne de base**, à `HEADER / 2 + 4` = 23, et le
//! chiffre du badge à 11,5 dans sa pastille. Le portage avait pris ces ordonnées pour des
//! **hauts** de texte : le titre tombait d'un corps entier et touchait le trait du bandeau, le
//! chiffre passait sous sa pastille. Le badge ne se montre que si le dossier contient quelque
//! chose, et un dossier vide dit « Double-clic pour ouvrir » ; l'icône est le tracé même de
//! Tauri, plus deux rectangles ; le titre a les opacités de Tauri (55 %, 85 % choisi).
//!
//! Le dessin est écrit **une fois**, sur une [`Surface`] : les pixels du canevas, ou le SVG
//! d'un export (P5).

use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::renderer::handles::draw_resize_handles;
use crate::renderer::pass::Clip;
use crate::renderer::scale::WorldScale;
use crate::renderer::surface::{Pixels, Surface, Trait, Vecteurs};
use crate::renderer::svg::Svg;
use crate::renderer::{parse_hex_color, push_rounded_rect, PaintKit};
use crate::typography::{Face, TextStyle};
use glucose_core::hit_priority::pick_consts;
use glucose_core::quadtree::Visibles;
use glucose_core::resize::Handle;
use glucose_core::store::Store;
use glucose_core::types::CanvasFolder;
use tiny_skia::{Color, LineCap, Path, PathBuilder, PixmapMut, Rect};

/// Rayon des coins du cadre, en unités monde (fiche 06 § 8.1).
pub(super) const CORNER_RADIUS: f32 = 10.0;
/// Largeur minimale d'un dossier, en unités monde.
pub(super) const MIN_WIDTH: f32 = 180.0;
/// Hauteur minimale d'un dossier, en unités monde.
pub(super) const MIN_HEIGHT: f32 = 120.0;
/// Opacité du corps, sur 255 : 2,5 % au repos, 5 % sélectionné.
const BODY_ALPHA: (u8, u8) = (6, 13);
/// Opacité de la bordure : 14 % au repos, 45 % sélectionnée.
const BORDER_ALPHA: (u8, u8) = (36, 115);
/// Épaisseur de la bordure, en unités monde : 1 px au repos, 1,4 px sélectionnée.
const BORDER_WIDTH: (f32, f32) = (1.0, 1.4);
/// Le pointillé de la bordure au repos : trois unités de trait, cinq de vide.
const BORDER_DASH: (f32, f32) = (3.0, 5.0);
/// Le halo de sélection : débord, rayon supplémentaire, épaisseur, opacité.
const GLOW: (f32, f32, f32, u8) = (3.0, 13.0, 1.5, 89);

/// Le coin de l'icône d'explorateur, en unités monde — le `translate(12, 11)` de Tauri.
const ICON: (f32, f32) = (12.0, 11.0);
/// Opacité de l'icône : 45 % au repos, 70 % sélectionnée.
const ICON_ALPHA: (u8, u8) = (115, 179);
/// Le titre dans le bandeau : son abscisse, **sa ligne de base**, son corps — en unités monde.
const TITLE: (f32, f32, f32) = (34.0, 23.0, 14.0);
/// Opacité du titre : 55 % au repos, 85 % sélectionné (Tauri, `fillOpacity`).
const TITLE_ALPHA: (u8, u8) = (140, 217);
/// Longueur au-delà de laquelle le titre est tronqué.
const TITLE_MAX_CHARS: usize = 28;
/// Le badge compteur : retrait depuis le bord droit, ordonnée, largeur, hauteur, rayon.
const BADGE: (f32, f32, f32, f32, f32) = (38.0, 11.0, 30.0, 16.0, 8.0);
/// Opacité du fond du badge (18 %) et de son contour (40 %).
const BADGE_ALPHA: (u8, u8) = (46, 102);
/// Épaisseur du contour du badge, taille de son chiffre, et **la ligne de base** du chiffre
/// dans la pastille, en unités monde.
const BADGE_STROKE: f32 = 0.8;
const BADGE_FONT: f32 = 10.0;
const BADGE_BASELINE: f32 = 11.5;
/// Opacité du chiffre du badge : 95 %.
const BADGE_TEXT_ALPHA: u8 = 242;
/// Ce qu'un dossier vide dit, au milieu de son corps, sa taille et son gris (Tauri : `#444`).
const VIDE: (&str, f32, (u8, u8, u8)) = ("Double-clic pour ouvrir", 11.0, (0x44, 0x44, 0x44));

/// Les longueurs d'un dossier à l'échelle d'une passe.
struct Layout {
    width: f32,
    height: f32,
    radius: f32,
    header: f32,
}

impl Layout {
    fn new(f: &CanvasFolder, scale: WorldScale) -> Self {
        // Le minimum appartient au modèle, pas au dessin : un dossier plus petit que 180 × 120
        // serait illisible, et la fiche 06 § 8.1 le pose comme une propriété du cadre.
        let w = (f.width as f32).max(MIN_WIDTH);
        let h = (f.height as f32).max(MIN_HEIGHT);
        Self {
            width: scale.world(w),
            height: scale.world(h),
            radius: scale.world(CORNER_RADIUS),
            header: scale.world(pick_consts::FOLDER_HEADER as f32),
        }
    }
}

/// Ce qu'un dossier sait de lui-même pour se dessiner : sa teinte, s'il est choisi, l'échelle,
/// et ce qu'il contient.
struct Etat {
    tint: (u8, u8, u8),
    selected: bool,
    scale: WorldScale,
    compte: usize,
}

/// **Chaque dossier visible du tableau actif**, avec son coin à l'écran, sa mise en page et
/// son état.
///
/// Deux filtres, et ils ne font pas double emploi. L'index donne la tranche des dossiers
/// proches, sans parcourir les autres : c'est lui qui rend le coût indépendant de la taille
/// du document (CULL-1). Le test de bord qui suit reste nécessaire parce que l'index
/// interroge une zone élargie de 200 px — marge utile au picking, superflue pour dessiner.
fn visibles<'a>(
    store: &'a Store,
    pass: ViewPass<'a>,
    clip: Clip,
) -> impl Iterator<Item = (&'a CanvasFolder, (f32, f32), Layout, Etat)> + 'a {
    let scale = pass.echelle();
    let board = store.active_board();
    let dossiers = board
        .into_iter()
        .flat_map(move |b| Visibles::nouvelles(pass.visibles, b).dossiers());
    dossiers.filter_map(move |f| {
        let layout = Layout::new(f, scale);
        let (wx, wy) = world_to_screen(f.x, f.y, &pass.vp);
        let (sx, sy) = (wx as f32, wy as f32);
        if clip.rejects(sx, sy, layout.width, layout.height) {
            return None;
        }
        let etat = Etat {
            tint: parse_hex_color(&f.color, 136, 136, 136),
            selected: store.selected_folder_id.as_deref() == Some(f.id.as_str()),
            scale,
            compte: store.folder_child_count(&f.child_board_id),
        };
        Some((f, (sx, sy), layout, etat))
    })
}

/// Dessine les dossiers visibles du tableau actif. Rend **vrai** si au moins un dossier a reçu
/// de l'encre (voir `draw_membranes`).
pub(super) fn draw_folders(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) -> bool {
    let clip = Clip {
        width: pixmap.width() as f32,
        height: pixmap.height() as f32,
        top: pass.header_h,
    };
    let mut encre = false;
    let groupe = store
        .active_board()
        .and_then(|b| store.emprise_du_groupe(&b.id));
    for (f, at, layout, etat) in visibles(store, pass, clip) {
        encre = true;
        let mut pixels = Pixels {
            pixmap: &mut *pixmap,
            typographie: kit.typography,
        };
        dessiner(&mut pixels, f, at, &layout, &etat);
        if etat.selected && groupe.is_none() {
            let boite = (at.0, at.1, layout.width, layout.height);
            draw_resize_handles(pixmap, kit.theme, etat.scale, boite, &Handle::ALL);
        }
    }
    encre
}

/// **Écrit les dossiers visibles en SVG** (P5, fiche 70) — le même dessin.
pub(super) fn dossiers_en_svg(
    kit: PaintKit<'_>,
    svg: &mut Svg,
    (store, pass): (&Store, ViewPass<'_>),
    taille: (u32, u32),
) {
    let clip = Clip {
        width: taille.0 as f32,
        height: taille.1 as f32,
        top: pass.header_h,
    };
    let mut vecteurs = Vecteurs {
        svg,
        typographie: kit.typography,
    };
    for (f, at, layout, etat) in visibles(store, pass, clip) {
        dessiner(&mut vecteurs, f, at, &layout, &etat);
    }
}

/// Un dossier : son cadre, puis — quand l'échelle dessine le détail — son bandeau.
fn dessiner(p: &mut impl Surface, f: &CanvasFolder, at: (f32, f32), layout: &Layout, etat: &Etat) {
    draw_frame(p, at, layout, etat);
    if etat.scale.draws_detail() {
        draw_header(p, at, layout, f, etat);
    }
}

/// Un rectangle aux coins arrondis, en tracé.
fn arrondi((x, y, w, h): (f32, f32, f32, f32), rayon: f32) -> Option<Path> {
    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, x, y, w, h, rayon);
    pb.finish()
}

/// Le cadre : halo de sélection, corps teinté, bordure, séparation du bandeau.
fn draw_frame(p: &mut impl Surface, at: (f32, f32), layout: &Layout, etat: &Etat) {
    let Etat {
        tint: (r, g, b),
        selected,
        scale,
        ..
    } = *etat;
    let pick = |paire: (u8, u8)| if selected { paire.1 } else { paire.0 };

    if selected && scale.draws_detail() {
        let pad = scale.world(GLOW.0);
        let boite = (
            at.0 - pad,
            at.1 - pad,
            layout.width + pad * 2.0,
            layout.height + pad * 2.0,
        );
        if let Some(halo) = arrondi(boite, scale.world(GLOW.1)) {
            let trait_ = Trait::plein(scale.world(GLOW.2));
            p.tracer(&halo, Color::from_rgba8(r, g, b, GLOW.3), trait_);
        }
    }

    let boite = (at.0, at.1, layout.width, layout.height);
    let Some(cadre) = arrondi(boite, layout.radius) else {
        return;
    };
    p.remplir(&cadre, Color::from_rgba8(r, g, b, pick(BODY_ALPHA)));
    let trait_ = Trait {
        largeur: scale.world(if selected {
            BORDER_WIDTH.1
        } else {
            BORDER_WIDTH.0
        }),
        tirets: (!selected).then(|| (scale.world(BORDER_DASH.0), scale.world(BORDER_DASH.1))),
        bout: LineCap::Butt,
    };
    let bord = Color::from_rgba8(r, g, b, pick(BORDER_ALPHA));
    p.tracer(&cadre, bord, trait_);

    // Le trait qui sépare le bandeau du corps : il rend le portail lisible comme une fenêtre.
    // Net (SCALE-3) — un trait horizontal n'a pas de bord oblique, et anticrénelé il avait fait
    // paniquer le rastériseur.
    let ligne = Rect::from_xywh(at.0, at.1 + layout.header, layout.width, scale.world(1.0));
    if let Some(ligne) = ligne {
        p.remplir_net(ligne, bord);
    }
}

/// L'icône d'explorateur de Tauri, en unités monde depuis son coin : un onglet sur un corps.
fn icone((x, y): (f32, f32), scale: WorldScale) -> Option<Path> {
    let point = |px: f32, py: f32| (x + scale.world(px), y + scale.world(py));
    let mut pb = PathBuilder::new();
    let aller = |pb: &mut PathBuilder, px, py| {
        let (a, b) = point(px, py);
        pb.line_to(a, b);
    };
    let courbe = |pb: &mut PathBuilder, (cx, cy), (px, py)| {
        let ((a, b), (c, d)) = (point(cx, cy), point(px, py));
        pb.quad_to(a, b, c, d);
    };
    // « M0 3 Q0 1 2 1 L7 1 L9 3 L14 3 Q16 3 16 5 L16 13 Q16 15 14 15 L2 15 Q0 15 0 13 Z »
    let (a, b) = point(0.0, 3.0);
    pb.move_to(a, b);
    courbe(&mut pb, (0.0, 1.0), (2.0, 1.0));
    aller(&mut pb, 7.0, 1.0);
    aller(&mut pb, 9.0, 3.0);
    aller(&mut pb, 14.0, 3.0);
    courbe(&mut pb, (16.0, 3.0), (16.0, 5.0));
    aller(&mut pb, 16.0, 13.0);
    courbe(&mut pb, (16.0, 15.0), (14.0, 15.0));
    aller(&mut pb, 2.0, 15.0);
    courbe(&mut pb, (0.0, 15.0), (0.0, 13.0));
    pb.close();
    pb.finish()
}

/// Le bandeau : icône, titre tronqué, puis le badge compteur — ou, vide, ce qu'il faut faire.
fn draw_header(
    p: &mut impl Surface,
    at: (f32, f32),
    layout: &Layout,
    f: &CanvasFolder,
    etat: &Etat,
) {
    let Etat {
        tint: (r, g, b),
        selected,
        scale,
        compte,
    } = *etat;
    let pick = |paire: (u8, u8)| if selected { paire.1 } else { paire.0 };
    let coin = (at.0 + scale.world(ICON.0), at.1 + scale.world(ICON.1));
    if let Some(forme) = icone(coin, scale) {
        p.remplir(&forme, Color::from_rgba8(r, g, b, pick(ICON_ALPHA)));
    }
    let corps = scale.world(TITLE.2);
    let titre = TextStyle {
        size: corps,
        color: Color::from_rgba8(r, g, b, pick(TITLE_ALPHA)),
        face: Face::Bold,
    };
    let haut = at.1 + scale.world(TITLE.1) - corps;
    let nom = truncate(&f.name, TITLE_MAX_CHARS);
    p.ecrire((at.0 + scale.world(TITLE.0), haut), &nom, titre);
    if compte > 0 {
        draw_badge(p, at, layout, etat);
    } else {
        dire_qu_il_est_vide(p, at, layout, scale);
    }
}

/// Le badge compteur, en haut à droite du bandeau : ce que le dossier contient
/// ([`Store::folder_child_count`] — ses images, ses notes, ses dossiers).
///
/// Combien de chiffres le badge sait montrer, borné par sa largeur : au-delà, `999+`.
const BADGE_CHIFFRES: usize = 4;

/// Le compte tel que le badge l'écrit, dans un tampon de pile.
///
/// Rien n'est alloué : un badge se dessine par dossier visible et à chaque image.
fn badge_label(n: usize, buf: &mut [u8; BADGE_CHIFFRES]) -> &str {
    if n > 999 {
        buf.copy_from_slice(b"999+");
        return "999+";
    }
    let mut fin = BADGE_CHIFFRES;
    let mut reste = n;
    loop {
        fin -= 1;
        buf[fin] = b'0' + (reste % 10) as u8;
        reste /= 10;
        if reste == 0 {
            break;
        }
    }
    // Les octets écrits sont des chiffres ASCII : la tranche est de l'UTF-8 valide.
    std::str::from_utf8(&buf[fin..]).unwrap_or("0")
}

fn draw_badge(p: &mut impl Surface, at: (f32, f32), layout: &Layout, etat: &Etat) {
    let Etat {
        tint: (r, g, b),
        scale,
        compte,
        ..
    } = *etat;
    let (bx, by) = (
        at.0 + layout.width - scale.world(BADGE.0),
        at.1 + scale.world(BADGE.1),
    );
    let (bw, bh) = (scale.world(BADGE.2), scale.world(BADGE.3));
    let Some(pastille) = arrondi((bx, by, bw, bh), scale.world(BADGE.4)) else {
        return;
    };
    p.remplir(&pastille, Color::from_rgba8(r, g, b, BADGE_ALPHA.0));
    let contour = Trait::plein(scale.world(BADGE_STROKE));
    p.tracer(
        &pastille,
        Color::from_rgba8(r, g, b, BADGE_ALPHA.1),
        contour,
    );

    let mut chiffres = [0u8; BADGE_CHIFFRES];
    let texte = badge_label(compte, &mut chiffres);
    let corps = scale.world(BADGE_FONT);
    // Centré sur la pastille par sa largeur mesurée (`textAnchor="middle"`), posé sur sa ligne
    // de base.
    let (large, _) = p.typographie().measure_text(texte, corps, Face::Bold);
    let haut = by + scale.world(BADGE_BASELINE) - corps;
    let style = TextStyle {
        size: corps,
        color: Color::from_rgba8(r, g, b, BADGE_TEXT_ALPHA),
        face: Face::Bold,
    };
    p.ecrire((bx + (bw - large) / 2.0, haut), texte, style);
}

/// Un dossier vide dit ce qu'il faut faire, au milieu de son corps — centré comme Tauri le
/// centre (`dominantBaseline="middle"` : la ligne de base à une demi-hauteur d'x sous le
/// milieu).
fn dire_qu_il_est_vide(p: &mut impl Surface, at: (f32, f32), layout: &Layout, scale: WorldScale) {
    let (texte, taille, (r, g, b)) = VIDE;
    let corps = scale.world(taille);
    let typo = p.typographie();
    let (large, _) = typo.measure_text(texte, corps, Face::Regular);
    let hauteur_d_x = typo.font(Face::Regular).metrics('x', corps).height as f32;
    let milieu = at.1 + layout.header + (layout.height - layout.header) / 2.0;
    let haut = milieu + hauteur_d_x / 2.0 - corps;
    let style = TextStyle {
        size: corps,
        color: Color::from_rgba8(r, g, b, 255),
        face: Face::Regular,
    };
    p.ecrire((at.0 + (layout.width - large) / 2.0, haut), texte, style);
}

/// Tronque un titre à `max` caractères, ellipse comprise.
///
/// Le compte porte sur les **caractères** et non sur les octets : un titre accentué serait
/// coupé au milieu d'un caractère par une troncature sur les octets, et la chaîne cesserait
/// d'être du texte.
pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
}

#[cfg(test)]
mod tests;
