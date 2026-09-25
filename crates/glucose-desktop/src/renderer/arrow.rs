//! **La flèche telle que Glucose Tauri la dessine** (FLECHE-1) — ce qu'il trouve *« trop
//! beau »* : un fil en dégradé de la teinte de sa source à celle de sa cible, un halo doux, et
//! une pastille ronde là où elle arrive — et **comment elle se peint** (FLECHE-2).
//!
//! Les nombres viennent de [`glucose_core::arrow::aspect`], la forme de
//! [`glucose_core::arrow::trace`] — le tracé même que le clic vise, droit ou courbe —, et la
//! peinture de [`glucose_core::arrow::champ`] : une loi évaluée en chaque pixel, que le
//! processeur peint ici et que la carte graphique peint à l'identique
//! ([`crate::present::fleches_gpu`]).
//!
//! # Ce qu'une flèche montre
//!
//! 1. le **halo** : le tracé, plus large de quatre pixels (dix sélectionnée), en dégradé, à
//!    18 % (45 %) ;
//! 2. le **trait** : deux pixels, en dégradé à 92 % — blanc quand la flèche est
//!    sélectionnée ;
//! 3. les **disques** : la pastille terminale sombre cerclée de la teinte d'arrivée, une
//!    seconde au départ si la flèche va dans les deux sens ; sélectionnée, deux disques pleins
//!    cerclés de blanc marquent ses bouts.
//!
//! # Ce que le traceur générique coûtait
//!
//! Peinte par `tiny-skia` — deux tracés anticrénelés en dégradé et des disques —, une flèche
//! coûtait 0,43 ms : quatre-vingts flèches, 34 ms par image (`bench_fleches`). La loi du champ
//! ne visite que les pixels que la flèche peut toucher.

use super::hue::SymbioticHueCache;
use super::math::MathRenderer;
use super::pass::Pass;
use super::scale::WorldScale;
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::typography::Typography;
use glucose_core::arrow::aspect::{self, Rgb, Teintes};
use glucose_core::arrow::champ::{self, Champ, Disque};
use glucose_core::arrow::trace::{self, Morceau};
use glucose_core::arrow::{self as noyau};
use glucose_core::quadtree::{noeud_au_rang, Noeud, Visibles};
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Board, Viewport};
use tiny_skia::PixmapMut;

/// Ce qu'il faut savoir d'une flèche pour la dessiner, une fois son tracé et ses teintes
/// connus.
pub(crate) struct Fleche {
    /// Son tracé, en coordonnées du monde.
    pub morceaux: Vec<Morceau>,
    pub teintes: Teintes,
    /// Son épaisseur, en pixels d'écran (`strokeWidth`).
    pub epaisseur: f64,
    pub selectionnee: bool,
    pub double_sens: bool,
}

impl Fleche {
    /// **Ce qu'une flèche du tableau montre** : son tracé ancré à ses nœuds, retrouvés par
    /// l'index en temps constant, et les teintes de ses deux bouts.
    pub(super) fn de(
        hue_cache: &mut SymbioticHueCache,
        ann: &Annotation,
        (pass, noeuds): (ViewPass<'_>, NoeudsDuRendu<'_>),
        selectionnee: bool,
    ) -> Option<Self> {
        let board = noeuds.board;
        let Annotation::Arrow {
            id,
            source_id,
            target_id,
            stroke_width,
            arrow_bidirectional,
            ..
        } = ann
        else {
            return None;
        };
        let morceaux = noyau::morceaux_with(ann, noeuds)?;
        let bouts = (morceaux.first()?.depart(), morceaux.last()?.arrivee());
        let teintes = aspect::teintes(
            teinte_d_un_bout(hue_cache, (board, pass), id, source_id.as_deref(), bouts.0),
            teinte_d_un_bout(hue_cache, (board, pass), id, target_id.as_deref(), bouts.1),
        );
        Some(Self {
            morceaux,
            teintes,
            epaisseur: stroke_width.unwrap_or(aspect::EPAISSEUR),
            selectionnee,
            double_sens: *arrow_bidirectional,
        })
    }

    /// **Son champ à l'écran** — tout en pixels, découpé au cadre de `ecran`.
    pub(crate) fn champ(
        &self,
        vp: &Viewport,
        scale: WorldScale,
        ecran: (f32, f32),
    ) -> Option<Champ> {
        let (premier, dernier) = (self.morceaux.first()?, self.morceaux.last()?);
        let a_l_ecran = |p: (f64, f64)| world_to_screen(p.0, p.1, vp);
        let sw = self.epaisseur;
        let (halo, opacite_du_halo) = aspect::halo(sw, self.selectionnee);
        let (debut, fin) = (a_l_ecran(premier.depart()), a_l_ecran(dernier.arrivee()));
        let t = self.teintes;
        let mut champ = Champ {
            segments: Vec::new(),
            axe: [debut.0 as f32, debut.1 as f32, fin.0 as f32, fin.1 as f32],
            teintes: [unite(t.depart), unite(t.milieu), unite(t.arrivee)],
            halo: [scale.screen(halo as f32) / 2.0, opacite_du_halo],
            ame: [scale.screen(sw as f32) / 2.0, aspect::OPACITE_DU_TRAIT],
            ame_blanche: self.selectionnee,
            disques: self.disques((debut, fin), scale),
        };
        // Un quart de pixel : la ligne brisée ne se distingue pas de la courbe à l'écran.
        let tolerance = 0.25 / vp.scale.max(1e-12);
        let points: Vec<(f64, f64)> = trace::aplatir(&self.morceaux, tolerance)
            .into_iter()
            .map(a_l_ecran)
            .collect();
        let segments: Vec<[f64; 4]> = points
            .windows(2)
            .map(|w| [w[0].0, w[0].1, w[1].0, w[1].1])
            .collect();
        // Le cadre élargi de la portée : un pixel de l'écran touché par un segment qui en sort
        // l'est par ce qui en reste.
        let p = f64::from(champ.portee());
        champ.segments = champ::decouper(
            &segments,
            [-p, -p, f64::from(ecran.0) + p, f64::from(ecran.1) + p],
        );
        Some(champ)
    }

    /// Les disques de ses bouts, en pixels d'écran.
    fn disques(
        &self,
        (debut, fin): ((f64, f64), (f64, f64)),
        scale: WorldScale,
    ) -> [Option<Disque>; champ::DISQUES] {
        let t = self.teintes;
        let sw = self.epaisseur;
        let disque = |centre: (f64, f64), k: f64, (fond, contour): (Rgb, Rgb)| Disque {
            centre: [centre.0 as f32, centre.1 as f32],
            rayon: scale.world((sw * k) as f32),
            demi_contour: scale.screen(aspect::CONTOUR as f32) / 2.0,
            fond: unite(fond),
            contour: unite(contour),
        };
        let blanc = (255, 255, 255);
        if self.selectionnee {
            return [
                Some(disque(debut, aspect::BOUT_SELECTIONNE, (t.depart, blanc))),
                Some(disque(fin, aspect::BOUT_SELECTIONNE, (t.arrivee, blanc))),
            ];
        }
        let fond = aspect::FOND_DE_LA_POINTE;
        [
            self.double_sens
                .then(|| disque(debut, aspect::POINTE, (fond, t.depart))),
            Some(disque(fin, aspect::POINTE, (fond, t.arrivee))),
        ]
    }
}

/// Une couleur du modèle, de 0 à 1.
fn unite((r, g, b): Rgb) -> [f32; 3] {
    [
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
    ]
}

/// La teinte d'un bout de flèche : celle du nœud qu'il touche, si c'est une annotation ; sinon
/// celle qu'aurait la flèche posée là — le choix de Tauri, qui ne cherche la source que parmi
/// les annotations.
fn teinte_d_un_bout(
    hue_cache: &mut SymbioticHueCache,
    (board, pass): (&Board, ViewPass<'_>),
    fleche: &str,
    noeud: Option<&str>,
    point: (f64, f64),
) -> f64 {
    let annotation = noeud.and_then(|id| match noeud_au_rang(board, pass.index.rang_de(id)?)? {
        Noeud::Annotation(a) if a.id() == id => Some(a),
        _ => None,
    });
    match annotation {
        Some(a) => hue_cache.get_or_compute(a, pass.index, board).0,
        None => hue_cache.au_point(fleche, point, pass.index, board),
    }
}

/// **Peint une flèche au processeur**, et rend vrai si elle a posé de l'encre.
pub(super) fn draw_arrow(ctx: &Pass, pixmap: &mut PixmapMut, fleche: &Fleche) -> bool {
    let (l, h) = (pixmap.width(), pixmap.height());
    let Some(champ) = fleche.champ(&ctx.vp, ctx.scale, (l as f32, h as f32)) else {
        return false;
    };
    let (pixels, _) = pixmap.data_mut().as_chunks_mut::<4>();
    champ.peindre(pixels, l, h)
}

/// **Les flèches que la carte doit peindre**, dans l'ordre du tableau (FLECHE-2).
///
/// Sur la voie graphique, la couche du dessus ne porte plus leur trait : une flèche en
/// diagonale y touchait beaucoup de lignes pour peu de pixels, et chacune partait sur le bus.
/// Leurs étiquettes, badges et poignées y restent.
pub(super) fn fleches_a_poser(
    hue_cache: &mut SymbioticHueCache,
    (store, pass): (&Store, ViewPass<'_>),
    (typographie, math): (&Typography, &MathRenderer),
    ecran: (f32, f32),
) -> Vec<Champ> {
    let Some(board) = store.active_board() else {
        return Vec::new();
    };
    let noeuds = NoeudsDuRendu {
        board,
        index: Some(pass.index),
        typographie,
        math,
    };
    let echelle = pass.echelle();
    Visibles::nouvelles(pass.visibles, board)
        .annotations()
        .filter(|a| matches!(a, Annotation::Arrow { .. }))
        .filter_map(|ann| {
            let selectionnee = store.selected_annotation_ids.iter().any(|s| s == ann.id());
            Fleche::de(hue_cache, ann, (pass, noeuds), selectionnee)?
                .champ(&pass.vp, echelle, ecran)
        })
        .collect()
}

mod noeuds;
pub(crate) use noeuds::NoeudsDuRendu;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "arrow/ancres_tests.rs"]
mod ancres_tests;
