//! Le dessin d'un champ de nombre : au repos, tenu, ou ouvert à la frappe.

use super::{Champ, Saisie};
use crate::dock::paint::Brush;
use crate::dock::{DockManager, WidgetRect};
use crate::typography::Face;
use tiny_skia::PixmapMut;

/// **Un champ et son libellé**, tels que la saisie en cours les montre : tenu, il s'entoure ;
/// ouvert à la frappe, il montre ce qu'on tape, la valeur choisie entière ou le curseur.
pub fn peindre(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    dock: &DockManager,
    champ: Champ,
    rect: WidgetRect,
) {
    let theme = brush.theme;
    let au_dessus = (rect.x, rect.y - brush.px(12.0));
    let libelle = champ.libelle(dock);
    brush.text(
        pixmap,
        libelle,
        au_dessus,
        9.0,
        theme.text_muted,
        Face::Bold,
    );
    let saisie = dock.champ.as_ref().filter(|s| s.champ() == champ);
    let Some(Saisie::Frappe {
        entree, entiere, ..
    }) = saisie
    else {
        brush.field(pixmap, rect, &champ.montre(champ.lire(dock)));
        let tenu = saisie.is_some();
        if tenu || brush.hovered(rect) {
            let trait_ = if tenu {
                theme.border_accent
            } else {
                theme.border_medium
            };
            brush.stroke(pixmap, rect, brush.px(3.0), trait_, brush.px(1.0));
        }
        return;
    };
    let gauche = rect.x + brush.px(8.0);
    let mesure = |t: &str| brush.typo.measure_text(t, brush.px(11.0), Face::Regular).0;
    brush.fill(pixmap, rect, brush.px(3.0), theme.input_bg);
    if *entiere {
        let choix = WidgetRect::new(
            gauche - brush.px(2.0),
            rect.y + brush.px(3.0),
            mesure(entree.text()) + brush.px(4.0),
            rect.h - brush.px(6.0),
        );
        brush.fill(pixmap, choix, brush.px(2.0), theme.text_selection);
    }
    brush.field_text(pixmap, rect, entree.text());
    if !*entiere {
        let x = gauche + mesure(entree.before_cursor());
        let caret = WidgetRect::new(
            x,
            rect.y + brush.px(4.0),
            brush.px(1.0),
            rect.h - brush.px(8.0),
        );
        brush.fill(pixmap, caret, 0.0, theme.text_primary);
    }
    brush.stroke(
        pixmap,
        rect,
        brush.px(3.0),
        theme.border_accent,
        brush.px(1.0),
    );
}
