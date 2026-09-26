//! **La fenêtre de l'éditeur, peinte** — l'habit de `ArrowTextEditor.tsx` : un voile, une
//! fenêtre sombre bordée de la couleur de la carte, l'étape en capitales à cette couleur, le
//! texte à sélectionner sur un fond noir, ce qui est choisi en puces, et les boutons.
//!
//! La couleur est celle de la carte de l'étape, comme chez Tauri (`stepColor`) : la source et la
//! cible se reconnaissent à leur teinte avant qu'on lise leur titre.

use super::fenetre::{
    titre, Fenetre, CONSIGNE_1, CONSIGNE_2, CORPS, CORPS_BOUTON, RAYON, SOUS_TITRE, TOUCHE,
};
use super::Ancrage;
use crate::renderer::PaintKit;
use crate::typography::{Face, TextStyle};
use crate::ui::action_bar::fond_arrondi;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, PixmapMut, Stroke, Transform};

/// Une teinte de la carte, à cette opacité.
fn teinte_a((r, g, b): (u8, u8, u8), opacite: f32) -> Color {
    Color::from_rgba8(r, g, b, (opacite * 255.0).round() as u8)
}

/// **Peint la fenêtre** de l'édition en cours, pour le texte `texte` de la carte de l'étape,
/// à sa `teinte`.
pub fn dessiner(
    pixmap: &mut PixmapMut,
    (fenetre, ancrage): (&Fenetre, &Ancrage),
    (texte, teinte): (&str, (u8, u8, u8)),
    kit: PaintKit<'_>,
) {
    let theme = kit.theme;
    let s = fenetre.s;
    // Le voile : le canevas recule, la fenêtre seule attend.
    let (l, h) = (pixmap.width() as f32, pixmap.height() as f32);
    fond_arrondi(
        pixmap,
        (0.0, 0.0, l, h),
        0.0,
        Color::from_rgba8(0, 0, 0, 150),
        None,
    );
    fond_arrondi(
        pixmap,
        fenetre.rect,
        RAYON * s,
        Color::from_rgba8(0x14, 0x14, 0x14, 255),
        Some(teinte_a(teinte, 0.27)),
    );
    dessiner_l_entete(pixmap, (fenetre, ancrage), teinte, kit);
    dessiner_la_consigne(pixmap, fenetre, kit);
    dessiner_la_zone(pixmap, (fenetre, ancrage), (texte, teinte), kit);
    dessiner_le_choisi(pixmap, fenetre, teinte, kit);
    for b in &fenetre.boutons {
        let (fond, filet, encre) = if b.principal {
            (
                teinte_a(teinte, 0.13),
                teinte_a(teinte, 0.33),
                teinte_a(teinte, 1.0),
            )
        } else {
            (theme.btn_bg, theme.btn_border, theme.text_muted)
        };
        fond_arrondi(pixmap, b.rect, 5.0 * s, fond, Some(filet));
        let largeur = kit
            .typography
            .measure_text(b.libelle, CORPS_BOUTON * s, Face::Regular)
            .0;
        kit.typography.draw_text(
            pixmap,
            b.libelle,
            b.rect.0 + (b.rect.2 - largeur) / 2.0,
            b.rect.1 + (b.rect.3 - CORPS_BOUTON * s) / 2.0,
            style(CORPS_BOUTON * s, encre, Face::Regular),
        );
    }
}

fn style(size: f32, color: Color, face: Face) -> TextStyle {
    TextStyle { size, color, face }
}

/// La pastille de l'étape et sa lueur, son titre, la consigne, et les numéros d'étape.
fn dessiner_l_entete(
    pixmap: &mut PixmapMut,
    (fenetre, ancrage): (&Fenetre, &Ancrage),
    teinte: (u8, u8, u8),
    kit: PaintKit<'_>,
) {
    let s = fenetre.s;
    let e = &fenetre.entete;
    let (haut, x_titre, x_sous) = (e.haut, e.x_titre, e.x_sous_titre);
    let centre = (x_titre - 10.0 * s, haut + 9.0 * s);
    disque(pixmap, centre, 8.0 * s, teinte_a(teinte, 0.18));
    disque(pixmap, centre, 4.0 * s, teinte_a(teinte, 1.0));
    let y = haut + (18.0 - CORPS) / 2.0 * s;
    kit.typography.draw_text(
        pixmap,
        titre(ancrage.etape),
        x_titre,
        y,
        style(CORPS * s, teinte_a(teinte, 1.0), Face::Bold),
    );
    kit.typography.draw_text(
        pixmap,
        SOUS_TITRE,
        x_sous,
        y,
        style(CORPS * s, kit.theme.text_muted, Face::Regular),
    );
    for (numero, rect, en_cours) in &e.etapes {
        let (fond, encre) = if *en_cours {
            (teinte_a(teinte, 1.0), Color::WHITE)
        } else {
            (kit.theme.bg_active, kit.theme.text_muted)
        };
        fond_arrondi(pixmap, *rect, 4.0 * s, fond, None);
        let l = kit
            .typography
            .measure_text(numero, (CORPS - 1.0) * s, Face::Bold)
            .0;
        kit.typography.draw_text(
            pixmap,
            numero,
            rect.0 + (rect.2 - l) / 2.0,
            rect.1 + (rect.3 - (CORPS - 1.0) * s) / 2.0,
            style((CORPS - 1.0) * s, encre, Face::Bold),
        );
    }
}

/// Les deux lignes de la consigne, la touche `Ctrl` dessinée comme une touche.
fn dessiner_la_consigne(pixmap: &mut PixmapMut, fenetre: &Fenetre, kit: PaintKit<'_>) {
    let s = fenetre.s;
    let gauche = fenetre.zone.rect.0;
    let encre = kit.theme.text_muted;
    let decale = (18.0 - CORPS) / 2.0 * s;
    kit.typography.draw_text(
        pixmap,
        CONSIGNE_1,
        gauche,
        fenetre.entete.consigne.0 + decale,
        style(CORPS * s, encre, Face::Regular),
    );
    let y = fenetre.entete.consigne.1 + decale;
    kit.typography.draw_text(
        pixmap,
        CONSIGNE_2.0,
        gauche,
        y,
        style(CORPS * s, encre, Face::Regular),
    );
    let t = fenetre.entete.touche;
    fond_arrondi(
        pixmap,
        (t.0, t.1 + 1.0 * s, t.2, t.3 - 2.0 * s),
        3.0 * s,
        kit.theme.bg_hover,
        Some(kit.theme.border_accent),
    );
    kit.typography.draw_text(
        pixmap,
        TOUCHE,
        t.0 + 5.0 * s,
        t.1 + (t.3 - (CORPS - 1.0) * s) / 2.0,
        style((CORPS - 1.0) * s, kit.theme.text_secondary, Face::Mono),
    );
    kit.typography.draw_text(
        pixmap,
        CONSIGNE_2.1,
        t.0 + t.2 + 5.0 * s,
        y,
        style(CORPS * s, encre, Face::Regular),
    );
}

/// **La zone du texte** : un fond noir bordé de la teinte, et le texte de la carte peint à part
/// puis posé — ce qui le découpe à la zone quand il défile.
fn dessiner_la_zone(
    pixmap: &mut PixmapMut,
    (fenetre, ancrage): (&Fenetre, &Ancrage),
    (texte, teinte): (&str, (u8, u8, u8)),
    kit: PaintKit<'_>,
) {
    let zone = fenetre.zone;
    let s = fenetre.s;
    fond_arrondi(
        pixmap,
        zone.rect,
        6.0 * s,
        Color::from_rgba8(0x0d, 0x0d, 0x0d, 255),
        Some(teinte_a(teinte, 0.13)),
    );
    let (l, h) = (zone.rect.2.ceil() as u32, zone.rect.3.ceil() as u32);
    let Some(mut image) = Pixmap::new(l.max(1), h.max(1)) else {
        return;
    };
    use crate::renderer::passages::{self, Eclaires, Pose, DANS_L_EDITEUR};
    let vue = zone.vue();
    let choisies = ancrage.plages_choisies(texte);
    let eclaires = Eclaires {
        plages: &choisies,
        teinte,
        style: &DANS_L_EDITEUR,
    };
    let outils = (kit.typography, kit.math);
    let largeur = zone.largeur_monde;
    let ouverte = passages::mise_en_page(
        outils,
        (texte, largeur),
        crate::renderer::richtext::TextMode::Rendered,
        Some(&eclaires),
    );
    let pose = Pose {
        origine: (0.0, 0.0),
        vp: vue,
        echelle: crate::renderer::scale::WorldScale::new(vue.scale, s),
    };
    // Le glisser en cours, sous le texte, comme toute sélection : rien ne s'écarte sous la
    // souris pendant qu'on choisit. Les passages choisis ont leur cadre, dans le texte.
    if let Some((a, b, _)) = ancrage.glisse.filter(|g| g.0 != g.1) {
        let glisse =
            passages::troncons(outils, (&ouverte, texte, largeur), &[(a.min(b), a.max(b))]);
        passages::peindre_une_selection(
            &mut image.as_mut(),
            pose,
            &glisse,
            (teinte, &DANS_L_EDITEUR),
        );
    }
    crate::renderer::card::peindre_le_texte_seul(
        kit,
        &mut image.as_mut(),
        (texte, largeur, teinte),
        (vue, s),
        Some(eclaires),
    );
    let choisis = passages::troncons(outils, (&ouverte, texte, largeur), &choisies);
    passages::peindre_les_lueurs(
        &mut image.as_mut(),
        pose,
        &choisis,
        (teinte, &DANS_L_EDITEUR),
    );
    pixmap.draw_pixmap(
        zone.rect.0.round() as i32,
        zone.rect.1.round() as i32,
        image.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        Transform::identity(),
        None,
    );
}

/// L'encadré de ce qui est choisi : son titre, « Effacer », et une puce par passage.
fn dessiner_le_choisi(
    pixmap: &mut PixmapMut,
    fenetre: &Fenetre,
    teinte: (u8, u8, u8),
    kit: PaintKit<'_>,
) {
    let Some(choisi) = &fenetre.choisi else {
        return;
    };
    let s = fenetre.s;
    fond_arrondi(
        pixmap,
        choisi.rect,
        4.0 * s,
        teinte_a(teinte, 0.07),
        Some(teinte_a(teinte, 0.2)),
    );
    let y = choisi.titre.1 + (18.0 - CORPS) / 2.0 * s;
    kit.typography.draw_text(
        pixmap,
        &choisi.titre.0,
        choisi.rect.0 + 10.0 * s,
        y,
        style(CORPS * s, teinte_a(teinte, 1.0), Face::Bold),
    );
    let e = choisi.effacer;
    let effacer_l = kit
        .typography
        .measure_text("Effacer", CORPS * s, Face::Regular)
        .0;
    kit.typography.draw_text(
        pixmap,
        "Effacer",
        e.0 + e.2 - effacer_l - 20.0 * s,
        y,
        style(CORPS * s, kit.theme.text_muted, Face::Regular),
    );
    croix(
        pixmap,
        (e.0 + e.2 - 8.0 * s, e.1 + e.3 / 2.0),
        3.5 * s,
        kit.theme.text_muted,
    );
    for puce in &choisi.puces {
        fond_arrondi(
            pixmap,
            puce.rect,
            3.0 * s,
            teinte_a(teinte, 0.12),
            Some(teinte_a(teinte, 0.25)),
        );
        kit.typography.draw_text(
            pixmap,
            &puce.texte,
            puce.rect.0 + 8.0 * s,
            puce.rect.1 + (puce.rect.3 - CORPS * s) / 2.0,
            style(CORPS * s, kit.theme.text_secondary, Face::Italic),
        );
        let c = puce.croix;
        croix(
            pixmap,
            (c.0 + c.2 / 2.0, c.1 + c.3 / 2.0),
            c.2 / 2.0,
            kit.theme.text_muted,
        );
    }
}

/// Un disque plein.
fn disque(pixmap: &mut PixmapMut, (cx, cy): (f32, f32), rayon: f32, couleur: Color) {
    let mut pb = PathBuilder::new();
    pb.push_circle(cx, cy, rayon);
    let Some(chemin) = pb.finish() else {
        return;
    };
    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };
    paint.set_color(couleur);
    pixmap.fill_path(
        &chemin,
        &paint,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );
}

/// Une croix de demi-côté `r` : la police de l'interface n'a pas de « ✗ », on la trace.
fn croix(pixmap: &mut PixmapMut, (cx, cy): (f32, f32), r: f32, couleur: Color) {
    let mut pb = PathBuilder::new();
    pb.move_to(cx - r, cy - r);
    pb.line_to(cx + r, cy + r);
    pb.move_to(cx + r, cy - r);
    pb.line_to(cx - r, cy + r);
    let Some(chemin) = pb.finish() else {
        return;
    };
    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };
    paint.set_color(couleur);
    let stroke = Stroke {
        width: (r / 3.0).max(1.0),
        line_cap: tiny_skia::LineCap::Round,
        ..Stroke::default()
    };
    pixmap.stroke_path(&chemin, &paint, &stroke, Transform::identity(), None);
}
