//! Le dessin du panneau STORYBOARD — les couleurs et les mots de `StoryboardControls.tsx`.

use super::lecture::{CaseLayout, Lecture, LectureLayout};
use super::{layout_storyboard_panel, Contenu, Ligne};
use crate::dock::champ::Champ;
use crate::dock::paint::{Brush, ButtonLook};
use crate::dock::DockManager;
use crate::dock::WidgetRect;
use crate::params::ScaledRect;
use crate::typography::Face;
use glucose_core::store::storyboard::Texte;
use tiny_skia::{Color, PixmapMut};

pub fn render_storyboard_panel(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    frame: ScaledRect,
    dock: &DockManager,
) {
    let state = &dock.storyboard;
    let contenu = &state.contenu;
    let layout = layout_storyboard_panel(frame, state, contenu);
    brush.title(pixmap, frame, "STORYBOARD");
    // Plusieurs storyboards : le courant, parmi combien, et de quoi passer de l'un à l'autre.
    if let Some((precedent, suivant, rang, nombre)) = layout.sequences {
        brush.button(pixmap, precedent, "‹", 12.0, ButtonLook::default());
        brush.button(pixmap, suivant, "›", 12.0, ButtonLook::default());
        let texte = format!("{} / {nombre}", rang + 1);
        let milieu = (precedent.x + precedent.w + suivant.x) / 2.0;
        let largeur = brush.typo.measure_text(&texte, brush.px(9.5), Face::Bold).0;
        brush.caption(
            pixmap,
            (milieu - largeur / 2.0, precedent.y + brush.px(4.0)),
            &texte,
        );
    }

    let x = frame.x + brush.px(14.0);
    brush.caption(
        pixmap,
        (x, layout.format_button.y - brush.px(14.0)),
        "FORMAT",
    );
    draw_selecteur(pixmap, brush, layout.format_button, state.format_label());
    for (champ, rect) in [
        (Champ::StoryLargeur, layout.width_input),
        (Champ::StoryColonnes, layout.cols_input),
        (Champ::StoryEcart, layout.gap_input),
    ] {
        crate::dock::champ::paint::peindre(pixmap, brush, dock, champ, rect);
    }
    for (i, cell) in layout.cells.iter().enumerate() {
        draw_cell(pixmap, brush, *cell, i + 1);
    }

    let actif = !contenu.cases.is_empty();
    let label = if actif { "Mettre à jour" } else { "Activer" };
    brush.button(
        pixmap,
        layout.activate_button,
        label,
        12.0,
        ButtonLook::default(),
    );
    if let Some(nouveau) = layout.nouveau {
        brush.button(pixmap, nouveau, "+ Nouveau", 12.0, ButtonLook::default());
    }
    // Le geste qui suit « Activer » ne se devine pas : il se dit ici, pas dans un toast, qui
    // ne dit que ce qui vient d'avoir lieu (cliquet 2).
    brush.caption(pixmap, layout.aide, "UN CLIC POSE, UN GLISSER TRACE");
    if let Some(croix) = layout.effacer {
        // « × » et non « ✕ » : Inter porte le premier, pas le second (vu sur l'aperçu).
        brush.button(pixmap, croix, "×", 14.0, ButtonLook::default());
    }
    if let (Some(l), Some(lecture)) = (&layout.lecture, &contenu.lecture) {
        peindre_la_lecture(pixmap, brush, dock, (l, lecture));
    }
    draw_list(pixmap, brush, (&layout, contenu));
}

/// **La lecture** (fiche 67) : « ▶ Lire » et la durée par case ; puis la case choisie.
fn peindre_la_lecture(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    dock: &DockManager,
    (layout, lecture): (&LectureLayout, &Lecture),
) {
    brush.button(pixmap, layout.lire, "▶ Lire", 12.0, ButtonLook::default());
    let planches = "Exporter en planches…";
    brush.button(
        pixmap,
        layout.planches,
        planches,
        12.0,
        ButtonLook::default(),
    );
    crate::dock::champ::paint::peindre(pixmap, brush, dock, Champ::StoryDuree, layout.duree);
    if let (Some(case), Some(c)) = (&layout.case, &lecture.case) {
        let titre = format!("CASE {}", c.numero);
        brush.caption(pixmap, case.titre, &titre);
        crate::dock::champ::paint::peindre(pixmap, brush, dock, Champ::CaseDuree, case.duree);
        if let Some(rendre) = case.rendre {
            brush.button(pixmap, rendre, "↺", 12.0, ButtonLook::default());
        }
        peindre_la_case(pixmap, brush, (case, c), lecture);
    }
}

/// Le plan de la case, puis une rangée par texte : le nom à gauche, ce qu'elle dit à droite.
fn peindre_la_case(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    (case, c): (&CaseLayout, &super::lecture::Choisie),
    lecture: &Lecture,
) {
    let theme = brush.theme;
    let au_dessus = (case.plan.x, case.plan.y - brush.px(12.0));
    brush.text(pixmap, "PLAN", au_dessus, 9.0, theme.text_muted, Face::Bold);
    draw_selecteur(pixmap, brush, case.plan, c.plan.map_or("—", |p| p.noms().0));
    let milieu = |r: WidgetRect| r.y + (r.h - brush.px(11.0)) / 2.0;
    for rangee in &case.textes {
        let (nom, ecrit) = match rangee.texte {
            Texte::Description => ("Description", c.description.as_str()),
            Texte::Champ(i) => (
                lecture.champs.get(i).map_or("", String::as_str),
                c.textes.get(i).map_or("", String::as_str),
            ),
        };
        let renommable = matches!(rangee.texte, Texte::Champ(_)) && brush.hovered(rangee.nom);
        let couleur_nom = if renommable {
            theme.text_primary
        } else {
            gris(0x66)
        };
        let court = tronquer(brush, nom, rangee.nom.w, 10.0);
        let at = (rangee.nom.x, milieu(rangee.nom));
        brush.text(pixmap, &court, at, 10.0, couleur_nom, Face::Bold);
        brush.fill(pixmap, rangee.ecrit, brush.px(3.0), gris(0x16));
        let (texte, couleur) = if ecrit.is_empty() {
            ("…", gris(0x33))
        } else {
            (ecrit, gris(0x88))
        };
        let couleur = if brush.hovered(rangee.ecrit) {
            theme.text_primary
        } else {
            couleur
        };
        let largeur = rangee.ecrit.w - brush.px(12.0);
        let court = tronquer(brush, texte, largeur, 11.0);
        let at = (rangee.ecrit.x + brush.px(6.0), milieu(rangee.ecrit));
        brush.text(pixmap, &court, at, 11.0, couleur, Face::Regular);
    }
    let look = ButtonLook {
        quiet: true,
        ..ButtonLook::default()
    };
    brush.button(pixmap, case.ajouter_un_champ, "+ Champ", 10.0, look);
}

/// La liste des cases : son titre, « + Ajouter », les pages, chaque case, « Re-numéroter ».
fn draw_list(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    (layout, contenu): (&super::StoryboardPanelLayout, &Contenu),
) {
    let Some((haut, ajouter)) = layout.liste else {
        return;
    };
    let theme = brush.theme;
    let x = layout.activate_button.x;
    let titre = format!("PANELS ({})", contenu.cases.len());
    brush.caption(pixmap, (x, haut), &titre);
    brush.button(pixmap, ajouter, "+ Ajouter", 11.0, ButtonLook::default());
    if let Some((precedente, suivante, page, pages)) = layout.pages {
        brush.button(pixmap, precedente, "‹", 12.0, ButtonLook::default());
        brush.button(pixmap, suivante, "›", 12.0, ButtonLook::default());
        let mot = format!("{}/{}", page + 1, pages);
        let at = (precedente.x - brush.px(30.0), precedente.y + brush.px(3.0));
        brush.text(pixmap, &mot, at, 10.0, theme.text_muted, Face::Regular);
    }
    for ligne in &layout.lignes {
        if let Some(case) = contenu.cases.get(ligne.rang) {
            draw_row(pixmap, brush, ligne, case);
        }
    }
    if let Some(r) = layout.renumeroter {
        let look = ButtonLook {
            quiet: true,
            ..ButtonLook::default()
        };
        brush.button(pixmap, r, "Re-numéroter les panels", 10.0, look);
    }
}

/// Une case : son numéro, sa description — ou « Description... » en retrait —, sa croix.
fn draw_row(pixmap: &mut PixmapMut, brush: &Brush, ligne: &Ligne, case: &(String, i32, String)) {
    let theme = brush.theme;
    brush.fill(pixmap, ligne.ligne, brush.px(3.0), gris(0x16));
    let milieu = |r: WidgetRect| r.y + (r.h - brush.px(11.0)) / 2.0;
    let numero = (case.1 + 1).to_string();
    let at = (ligne.ligne.x + brush.px(8.0), milieu(ligne.ligne));
    brush.text(pixmap, &numero, at, 10.0, gris(0x55), Face::Regular);
    let (texte, couleur) = if case.2.is_empty() {
        ("Description...", gris(0x33))
    } else {
        (case.2.as_str(), gris(0x88))
    };
    let survol = brush.hovered(ligne.description);
    let couleur = if survol { theme.text_primary } else { couleur };
    let at = (ligne.description.x, milieu(ligne.description));
    let court = tronquer(brush, texte, ligne.description.w, 11.0);
    brush.text(pixmap, &court, at, 11.0, couleur, Face::Regular);
    let croix = if brush.hovered(ligne.retirer) {
        theme.alert
    } else {
        gris(0x33)
    };
    brush.text_centered(pixmap, ligne.retirer, "×", 13.0, croix, Face::Regular);
}

/// **Le texte, coupé à `largeur`** avec « … » quand il ne tient pas — une description longue
/// ne déborde pas sur la croix de sa case.
fn tronquer(brush: &Brush, texte: &str, largeur: f32, corps: f32) -> String {
    let mesure = |t: &str| brush.typo.measure_text(t, brush.px(corps), Face::Regular).0;
    if mesure(texte) <= largeur {
        return texte.to_string();
    }
    let mut court: String = texte.to_string();
    while !court.is_empty() && mesure(&format!("{court}…")) > largeur {
        court.pop();
    }
    format!("{}…", court.trim_end())
}

fn gris(v: u8) -> Color {
    Color::from_rgba8(v, v, v, 255)
}

/// **Un sélecteur qui tourne** — le format, le plan : un champ avec un chevron.
fn draw_selecteur(pixmap: &mut PixmapMut, brush: &Brush, rect: WidgetRect, libelle: &str) {
    let theme = brush.theme;
    let bg = if brush.hovered(rect) {
        theme.bg_hover
    } else {
        theme.input_bg
    };
    brush.fill(pixmap, rect, brush.px(4.0), bg);
    brush.stroke(pixmap, rect, brush.px(4.0), theme.btn_border, brush.px(1.0));
    let at = (rect.x + brush.px(8.0), rect.y + brush.px(6.0));
    brush.text(pixmap, libelle, at, 11.0, theme.text_primary, Face::Regular);
    let chevron = (rect.x + rect.w - brush.px(16.0), rect.y + brush.px(4.0));
    brush.text(pixmap, "˅", chevron, 12.0, theme.text_muted, Face::Regular);
}

/// Une cellule numérotée de l'aperçu (Tauri : blanc à 0,04, liseré à 0,18, numéro à 0,25).
fn draw_cell(pixmap: &mut PixmapMut, brush: &Brush, rect: WidgetRect, number: usize) {
    let blanc = |a: u8| Color::from_rgba8(255, 255, 255, a);
    brush.fill(pixmap, rect, brush.px(1.0), blanc(10));
    brush.stroke(pixmap, rect, brush.px(1.0), blanc(46), brush.px(0.8));
    brush.text_centered_if_fits(pixmap, rect, &number.to_string(), 8.0, blanc(64));
}
