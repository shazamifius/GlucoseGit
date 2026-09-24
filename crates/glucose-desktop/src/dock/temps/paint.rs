//! Le dessin du panneau TIME MACHINE : l'en-tête et son badge, la réglette, les boutons de
//! l'aperçu, les jalons, le pied.
//!
//! Deux couleurs seulement disent l'état, comme chez Glucose Tauri : le **vert** du présent
//! (le succès du thème) et l'**ambre** du passé (fiche 06 § 1 : les jalons de la Time
//! Machine). Pas d'émoji : la police n'en a pas (R-51).

use super::{il_y_a, layout_temps_panel, TempsLayout, TempsUi};
use crate::dock::paint::{Brush, ButtonLook};
use crate::dock::WidgetRect;
use crate::params::ScaledRect;
use crate::typography::Face;
use tiny_skia::PixmapMut;

pub fn render_temps_panel(pixmap: &mut PixmapMut, brush: &Brush, frame: ScaledRect, ui: &TempsUi) {
    let layout = layout_temps_panel(frame, ui);
    let marge = brush.px(14.0);
    brush.text(
        pixmap,
        "TIME MACHINE",
        (frame.x + marge, frame.y + brush.px(22.0)),
        10.0,
        brush.theme.text_muted,
        Face::Bold,
    );
    badge(pixmap, brush, &layout, ui);
    compte(pixmap, brush, frame, ui);
    reglette(pixmap, brush, layout.reglette, ui);
    if let Some((maintenant, restaurer)) = layout.boutons {
        brush.button(
            pixmap,
            maintenant,
            "← Maintenant",
            10.5,
            ButtonLook::default(),
        );
        brush.fill(pixmap, restaurer, brush.px(3.0), brush.theme.temps.ambre);
        brush.text_centered(
            pixmap,
            restaurer,
            "Restaurer cet état",
            11.0,
            brush.theme.bg_panel,
            Face::Bold,
        );
    }
    jalons(pixmap, brush, &layout, ui);
    pied(pixmap, brush, layout.pied, ui);
}

/// « EN DIRECT » au présent, « APERÇU » dans le passé.
fn badge(pixmap: &mut PixmapMut, brush: &Brush, layout: &TempsLayout, ui: &TempsUi) {
    let (texte, couleur) = match ui.regarde {
        Some(_) => ("APERÇU", brush.theme.temps.ambre),
        None => ("EN DIRECT", brush.theme.success),
    };
    brush.stroke(pixmap, layout.badge, brush.px(8.0), couleur, brush.px(1.0));
    brush.text_centered(pixmap, layout.badge, texte, 8.5, couleur, Face::Bold);
}

/// « N GESTES », et où l'on est : « k/N · il y a … ».
fn compte(pixmap: &mut PixmapMut, brush: &Brush, frame: ScaledRect, ui: &TempsUi) {
    let n = ui.gestes.len();
    let y = frame.y + brush.px(46.0);
    let gauche = format!("{n} GESTE{}", if n > 1 { "S" } else { "" });
    brush.text(
        pixmap,
        &gauche,
        (frame.x + brush.px(14.0), y),
        9.0,
        brush.theme.text_muted,
        Face::Bold,
    );
    let k = ui.point();
    let quand = match k {
        0 => "au début".to_string(),
        k => il_y_a(ui.maintenant, ui.gestes[k - 1]),
    };
    let droite = format!("{k}/{n} · {quand}");
    let (w, _) = brush
        .typo
        .measure_text(&droite, brush.px(10.0), Face::Regular);
    brush.text(
        pixmap,
        &droite,
        (frame.x + frame.w - brush.px(14.0) - w, y),
        10.0,
        brush.theme.text_secondary,
        Face::Regular,
    );
}

/// L'abscisse d'un point de la réglette, de 0 (le début) à `n` (le présent).
fn abscisse(r: WidgetRect, n: usize, point: usize) -> f32 {
    if n == 0 {
        return r.x + r.w;
    }
    r.x + r.w * point as f32 / n as f32
}

/// Un trait par geste, les jalons en ambre, et le curseur.
fn reglette(pixmap: &mut PixmapMut, brush: &Brush, r: WidgetRect, ui: &TempsUi) {
    let theme = brush.theme;
    brush.fill(pixmap, r, brush.px(4.0), theme.bg_hover);
    brush.stroke(pixmap, r, brush.px(4.0), theme.border_subtle, brush.px(1.0));
    let n = ui.gestes.len();
    let (haut, h) = (r.y + brush.px(4.0), r.h - brush.px(8.0));
    // Un trait par pixel au plus : dix mille gestes sur trois cents pixels ne font pas dix
    // mille tracés.
    let mut dernier = f32::NEG_INFINITY;
    for point in 1..=n {
        let x = abscisse(r, n, point);
        if x - dernier < 1.0 {
            continue;
        }
        dernier = x;
        brush.fill(
            pixmap,
            WidgetRect::new(x - 0.5, haut, 1.0, h),
            0.0,
            theme.border_medium,
        );
    }
    // Les jalons, plus épais qu'un geste : ambre s'ils sont nommés, gris s'ils viennent de
    // Ctrl+S — la même couleur que leur pastille dans la liste.
    for j in &ui.jalons {
        let x = abscisse(r, n, j.apres.min(n));
        let couleur = if j.nomme {
            theme.temps.ambre
        } else {
            theme.text_muted
        };
        brush.fill(pixmap, WidgetRect::new(x - 1.0, haut, 2.0, h), 0.0, couleur);
    }
    let couleur = if ui.regarde.is_some() {
        theme.temps.ambre
    } else {
        theme.success
    };
    let x = abscisse(r, n, ui.point());
    let curseur = WidgetRect::new(
        x - brush.px(1.5),
        r.y - brush.px(3.0),
        brush.px(3.0),
        r.h + brush.px(6.0),
    );
    brush.fill(pixmap, curseur, brush.px(1.5), couleur);
}

/// Les jalons, du plus récent au plus ancien.
fn jalons(pixmap: &mut PixmapMut, brush: &Brush, layout: &TempsLayout, ui: &TempsUi) {
    let theme = brush.theme;
    let x = layout.reglette.x;
    brush.caption(pixmap, (x, layout.liste_y), "JALONS");
    if ui.jalons.is_empty() {
        brush.text(
            pixmap,
            "Aucun jalon : Ctrl+S en pose un, le bouton ci-dessous en nomme un.",
            (x, layout.liste_y + brush.px(20.0)),
            9.5,
            theme.text_muted,
            Face::Regular,
        );
        return;
    }
    for (ligne, bouton, rang) in &layout.lignes {
        let j = &ui.jalons[*rang];
        let regarde = ui.regarde == Some(j.apres);
        if regarde || brush.hovered(*ligne) {
            brush.fill(pixmap, *ligne, brush.px(4.0), theme.bg_hover);
        }
        let pastille = (ligne.x + brush.px(6.0), ligne.y + brush.px(10.0));
        let couleur = if j.nomme {
            theme.temps.ambre
        } else {
            theme.text_muted
        };
        brush.circle(pixmap, pastille, brush.px(3.5), couleur);
        let libelle = if j.libelle.is_empty() {
            "Enregistré"
        } else {
            &j.libelle
        };
        brush.text(
            pixmap,
            libelle,
            (ligne.x + brush.px(16.0), ligne.y + brush.px(2.0)),
            11.0,
            theme.text_primary,
            Face::Bold,
        );
        let genre = if j.nomme { "nommé" } else { "Ctrl+S" };
        let detail = format!(
            "{genre} · geste {} · {}",
            j.apres,
            il_y_a(ui.maintenant, j.instant)
        );
        brush.text(
            pixmap,
            &detail,
            (ligne.x + brush.px(16.0), ligne.y + brush.px(17.0)),
            9.0,
            theme.text_muted,
            Face::Regular,
        );
        brush.button(pixmap, *bouton, "Restaurer", 9.5, ButtonLook::default());
    }
}

/// « + Marquer un jalon », ou le champ où son nom se tape.
fn pied(pixmap: &mut PixmapMut, brush: &Brush, r: WidgetRect, ui: &TempsUi) {
    match &ui.nom {
        Some(entree) => {
            brush.field(pixmap, r, entree.text());
            brush.stroke(
                pixmap,
                r,
                brush.px(3.0),
                brush.theme.temps.ambre,
                brush.px(1.0),
            );
            let (avant, _) =
                brush
                    .typo
                    .measure_text(entree.before_cursor(), brush.px(11.0), Face::Regular);
            let x = r.x + brush.px(8.0) + avant;
            let caret =
                WidgetRect::new(x, r.y + brush.px(7.0), brush.px(1.0), r.h - brush.px(14.0));
            brush.fill(pixmap, caret, 0.0, brush.theme.text_primary);
            brush.text(
                pixmap,
                "Entrée pour poser le jalon, Échap pour renoncer",
                (r.x, r.y - brush.px(15.0)),
                9.0,
                brush.theme.text_muted,
                Face::Regular,
            );
        }
        None => brush.button(pixmap, r, "+ Marquer un jalon", 11.5, ButtonLook::default()),
    }
}
