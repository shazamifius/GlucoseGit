//! **L'habit d'une carte** (LUEUR-1, LUEUR-3, PAPIER-1) : sa teinte, le papier qu'elle peint sous
//! son texte, et la forme qui les porte.
//!
//! Extrait de [`super`] quand les passages qui brillent (PASSAGE-2) y sont entrés : six cent
//! onze lignes là où la fiche 05 en admet six cents. La coupure tombe là où la question change —
//! ici *de quoi la carte est vêtue*, là-bas *comment son texte se pose*.

use super::{CardLayout, TextCard};
use crate::theme::Theme;
use glucose_core::membrane_forme::{Arrondi, Membrane};
use tiny_skia::PixmapMut;

/// **La teinte d'une carte** : sa couleur, si elle en porte une, sinon sa teinte symbiotique.
///
/// Une seule fonction pour tout ce qui la lit — la carte, sa texture, ses passages qui
/// brillent : elle s'écrivait en deux copies, et la lueur d'un passage prenait la symbiotique
/// là où la carte avait une couleur.
pub(crate) fn teinte_de_carte(
    hue_cache: &mut crate::renderer::hue::SymbioticHueCache,
    ann: &glucose_core::types::Annotation,
    (index, board): (
        &glucose_core::quadtree::SpatialHash,
        &glucose_core::types::Board,
    ),
) -> (u8, u8, u8) {
    let (_, symbiose) = hue_cache.get_or_compute(ann, index, board);
    let couleur = match ann {
        glucose_core::types::Annotation::Text { color, .. } => color.as_deref(),
        _ => None,
    };
    couleur
        .map(|c| crate::renderer::parse_hex_color(c, symbiose.0, symbiose.1, symbiose.2))
        .unwrap_or(symbiose)
}

/// **PAPIER-1 — le papier d'un texte** (fiche 105) : le gris sombre et neutre des blocs, partout
/// ([`Theme::card_paper`]).
///
/// Il était le fond vu au travers des membranes qui contiennent la carte (LUEUR-3), puis teinté de
/// la couleur choisie (COULEUR-3, fiche 99). Sa parole du 11/11/2026 : *« il faudrait abandonner
/// l'idée du background d'un texte : je préfère qu'il soit TOUJOURS gris »* — le geste qui colore
/// un texte devait changer le halo autour du bloc, *« PAS son background ! »*. Une membrane se lit
/// maintenant autour de ses blocs, et le papier ne dépend plus de ce qu'il y a dessous : la
/// texture d'un texte ne se refait plus quand une membrane change de couleur.
pub(crate) fn papier(theme: &Theme) -> (u8, u8, u8) {
    let c = theme.card_paper.to_color_u8();
    (c.red(), c.green(), c.blue())
}

/// **Le papier de la carte** (LUEUR-1, LUEUR-3, PAPIER-1).
///
/// Tauri posait sous le texte la teinte de la carte **à 3 %** (`color-mix(AURA 3%)`) et rien
/// d'autre : aucun cadre au repos, et la lueur, découpée à l'intérieur de la boîte, ne passe
/// jamais sous le texte ([`crate::renderer::halo`]). C'est ce qu'il appelle « le texte sur fond noir,
/// le contour en lueur ».
///
/// # Le papier est peint (LUEUR-3)
///
/// Chez Tauri, ce fond noir n'était que la toile, vue au travers. Posée sur une photo, la carte
/// montrait donc la photo sous son texte, qui devenait illisible (sa capture du 26/09) ; et sur la
/// toile, les points de la grille passaient entre les mots, où ils se lisent comme une ponctuation
/// — « que.on », sur une autre de ses captures. Le papier est donc **peint**, opaque.
///
/// # Neutre (PAPIER-1)
///
/// Les 3 % de la teinte de la carte s'y mêlaient encore — la teinte symbiotique, une couleur que le
/// voisinage décide. Sa parole du 11/11/2026 : le fond d'un texte est *« TOUJOURS gris »*, *« neutre »*.
/// Le papier est le jeton du thème, et rien d'autre ; la constante des 3 % disparaît.
///
/// L'anneau qui désigne une carte sélectionnée ou éditée n'est pas ici : c'est une
/// affordance, pas un habit, et il se pose avec les autres (`dessus.rs`, COMPOSANT-4).
///
/// **Le papier se peint par la loi des membranes** (COMPOSANT-4) : un plein d'une couche, une
/// composition par pixel là où il est constant, le calcul exact au seul bord. Le remplissage
/// anticrénelé de `tiny-skia` en coûtait les trois quarts du rendu d'une carte qu'on écrit —
/// 1,54 ms sur 2,05 pour sa carte de notes vue à ×2,3 (`bench_saisie`) —, pour un voile
/// uniforme. Et une loi évaluée pixel par pixel donne le même pixel quel que soit le morceau
/// de carte qu'on peint : c'est ce qui manquait pour ne refaire que les lignes qu'une frappe
/// change.
pub(super) fn draw_card_papier(
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    card: &TextCard,
) {
    let forme = forme_de_la_carte(at, (layout.width, layout.height), layout.radius);
    let papier = Membrane::plein(forme, voile_de_la_carte(card), 1.0);
    let (largeur, hauteur) = (pixmap.width(), pixmap.height());
    let (pixels, _) = pixmap.data_mut().as_chunks_mut::<4>();
    glucose_core::membrane_forme::peindre(&papier, pixels, largeur, hauteur);
}

/// **La couleur du papier**, de 0 à 1 par canal, opaque. Le canevas la peint, le dessin vectoriel
/// la pose.
pub(super) fn voile_de_la_carte(card: &TextCard) -> [f32; 3] {
    let (r, g, b) = card.fond;
    [r, g, b].map(|c| f32::from(c) / 255.0)
}

/// **La forme d'une carte à l'écran** : son rectangle aux coins de `rayon`, posé sur la grille
/// de ses glyphes.
///
/// Opaque, le fond d'une carte montrait ce que sa transparence cachait : posée en place ou
/// dans sa texture, la même carte différait d'un niveau sur un pixel de coin, parce que
/// `gauche + largeur` ne s'arrondit pas pareil à deux abscisses différentes. Sur la grille, les
/// sommes sont exactes (`typography::glyph::sur_la_grille`), et les deux voies calculent les
/// mêmes nombres — par construction, et non par chance. La découpe de la lueur lit la même
/// forme ([`crate::renderer::halo`]).
pub(crate) fn forme_de_la_carte(
    at: (f32, f32),
    (largeur, hauteur): (f32, f32),
    rayon: f32,
) -> Arrondi {
    let (x, y, l, h) = boite_de_la_carte(at, (largeur, hauteur));
    Arrondi::nouveau(x, y, l, h, rayon)
}

/// **La boîte d'une carte à l'écran**, sur la grille de ses glyphes : celle que
/// [`forme_de_la_carte`] arrondit, et que le dessin vectoriel donne à la carte graphique — les
/// mêmes nombres, donc la même forme au bit près.
pub(crate) fn boite_de_la_carte(
    at: (f32, f32),
    (largeur, hauteur): (f32, f32),
) -> (f32, f32, f32, f32) {
    use crate::typography::glyph::sur_la_grille;
    (
        sur_la_grille(at.0),
        sur_la_grille(at.1),
        sur_la_grille(largeur),
        sur_la_grille(hauteur),
    )
}

#[cfg(test)]
mod tests;
