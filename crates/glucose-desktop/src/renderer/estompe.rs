//! **ESTOMPE-1 — ce que la frise chronologique écarte s'estompe** (fiche 114) : un nœud daté hors
//! de la période qu'elle garde ne se cache pas, il se dessine à la présence que le noyau lui donne
//! (`Store::presence`, `glucose_core::timeline::ESTOMPE`) — sur les deux voies.
//!
//! * **Ce qui est d'une seule couleur** — le sol et la frontière d'une région, une lueur, le champ
//!   d'une flèche, une pastille — multiplie son alpha : c'est exact.
//! * **Ce qui est composé** — une carte de texte, un titre, une photo — prend une opacité de
//!   **groupe** : la carte graphique pose sa texture avec `Pose::opacite`.
//! * **La couche du dessus**, que les deux voies dessinent par le même code (les post-it, les
//!   étiquettes et les pastilles des flèches, les réglettes des domaines, les dates), multiplie
//!   les alphas qu'elle emploie : les deux voies y restent identiques par construction.

use glucose_core::quadtree::{noeud_au_rang, noeud_par_id, Noeud, SpatialHash};
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Board};
use tiny_skia::Color;

/// **Cette couleur, à cette présence** : son alpha multiplié — rien d'autre ne change.
pub(crate) fn estomper(c: Color, presence: f32) -> Color {
    if presence >= 1.0 {
        return c;
    }
    Color::from_rgba(c.red(), c.green(), c.blue(), c.alpha() * presence.max(0.0)).unwrap_or(c)
}

/// **ESTOMPE-2 — ce que la frise laisse à une flèche** (fiche 115) : sa date, ou celles de ses deux
/// bouts (`Store::presence_d_une_relation`), retrouvés par l'index en temps constant — sans index,
/// dans le tableau. Sans frise, rien à chercher. Les deux chemins qui dessinent une flèche (son
/// champ, et la couche du dessus : son étiquette, sa pastille) l'appellent.
pub(crate) fn de_la_fleche(
    store: &Store,
    (board, index): (&Board, Option<&SpatialHash>),
    ann: &Annotation,
) -> f32 {
    if store.temporal_filter.is_none() {
        return 1.0;
    }
    let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = ann
    else {
        return store.presence(ann.temporal_anchor());
    };
    let date = |id: &Option<String>| {
        let id = id.as_deref()?;
        let noeud = index
            .and_then(|i| noeud_au_rang(board, i.rang_de(id)?))
            .filter(|n| n.id() == id)
            .or_else(|| noeud_par_id(board, id))?;
        match noeud {
            Noeud::Annotation(a) => a.temporal_anchor(),
            Noeud::Image(i) => i.temporal_anchor.as_ref(),
            Noeud::Dossier(_) => None,
        }
    };
    store.presence_d_une_relation(ann.temporal_anchor(), [date(source_id), date(target_id)])
}

/// **Une surface qui estompe** : ce qu'on lui demande de remplir, de tracer ou d'écrire part sur
/// la surface d'en dessous, chaque couleur à cette présence. Tout dessin écrit sur une
/// [`super::surface::Surface`] — la réglette des domaines, la pastille d'une date — s'estompe
/// ainsi sans qu'on le touche.
pub(crate) struct Estompee<'a, S> {
    pub dessous: &'a mut S,
    pub presence: f32,
}

impl<S: super::surface::Surface> super::surface::Surface for Estompee<'_, S> {
    fn remplir(&mut self, chemin: &tiny_skia::Path, couleur: Color) {
        self.dessous
            .remplir(chemin, estomper(couleur, self.presence));
    }

    fn tracer(&mut self, chemin: &tiny_skia::Path, couleur: Color, trait_: super::surface::Trait) {
        self.dessous
            .tracer(chemin, estomper(couleur, self.presence), trait_);
    }

    fn remplir_net(&mut self, rect: tiny_skia::Rect, couleur: Color) {
        self.dessous
            .remplir_net(rect, estomper(couleur, self.presence));
    }

    fn ecrire(&mut self, at: (f32, f32), texte: &str, style: crate::typography::TextStyle) {
        let style = crate::typography::TextStyle {
            color: estomper(style.color, self.presence),
            ..style
        };
        self.dessous.ecrire(at, texte, style);
    }

    fn typographie(&self) -> &crate::typography::Typography {
        self.dessous.typographie()
    }
}

/// **L'opacité de groupe, au processeur** : ce que `dessin` pose dans `boite` (en pixels de
/// l'écran) se dessine d'abord dans une couche à part, à la taille de la boîte, puis se compose à
/// `presence` — exactement ce que la carte graphique fait d'une texture posée avec
/// `Pose::opacite`. Entier, il se dessine en place, sans détour. `dessin` reçoit la couche et
/// l'origine de la boîte dans l'écran, à retirer de ses coordonnées.
pub(crate) fn en_groupe(
    pixmap: &mut tiny_skia::PixmapMut,
    presence: f32,
    (x, y, l, h): (f32, f32, f32, f32),
    dessin: impl FnOnce(&mut tiny_skia::PixmapMut, (f32, f32)),
) {
    if presence >= 1.0 {
        dessin(pixmap, (0.0, 0.0));
        return;
    }
    // La boîte, aux pixels entiers et dans l'écran : la couche se compose sans rééchantillonner.
    let (x0, y0) = (x.floor().max(0.0), y.floor().max(0.0));
    let x1 = (x + l).ceil().min(pixmap.width() as f32);
    let y1 = (y + h).ceil().min(pixmap.height() as f32);
    if presence <= 0.0 || x1 <= x0 || y1 <= y0 {
        return;
    }
    let Some(mut couche) = tiny_skia::Pixmap::new((x1 - x0) as u32, (y1 - y0) as u32) else {
        return;
    };
    dessin(&mut couche.as_mut(), (x0, y0));
    let paint = tiny_skia::PixmapPaint {
        opacity: presence,
        blend_mode: tiny_skia::BlendMode::SourceOver,
        quality: tiny_skia::FilterQuality::Nearest,
    };
    let ici = tiny_skia::Transform::identity();
    pixmap.draw_pixmap(x0 as i32, y0 as i32, couche.as_ref(), &paint, ici, None);
}
