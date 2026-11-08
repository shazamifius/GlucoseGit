//! **L'habit d'une carte** (LUEUR-1, LUEUR-3) : sa teinte, le fond qu'elle peint sous son
//! texte, sa brume, et la forme qui les porte.
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

/// Le fond du canevas en octets — celui du thème, ou celui que le mode Focus teinte.
pub(crate) fn fond_du_canevas(theme: &Theme) -> (u8, u8, u8) {
    let c = theme.bg_canvas.to_color_u8();
    (c.red(), c.green(), c.blue())
}

/// **Ce qu'une carte ne cache pas** (LUEUR-3) : le fond du canevas, et les membranes visibles.
///
/// Une carte cache le **contenu** qu'elle recouvre — une photo, la grille, la lueur d'une
/// voisine —, jamais son **contenant**. Posée dans une membrane, elle en garde la teinte, comme
/// chez Tauri où la membrane se voyait au travers ; posée sur une photo, elle ne montre plus
/// la photo sous son texte.
pub(crate) struct Contenants<'a> {
    fond: (u8, u8, u8),
    membranes: &'a [Membrane],
    /// L'enveloppe de chaque membrane ([`Membrane::enveloppe`]), relevée une fois par image :
    /// hors d'elle, sa loi vaut zéro au bit près.
    enveloppes: Vec<glucose_core::membrane_forme::Arrondi>,
}

impl<'a> Contenants<'a> {
    pub(crate) fn nouveaux(theme: &Theme, membranes: &'a [Membrane]) -> Self {
        Self {
            fond: fond_du_canevas(theme),
            membranes,
            enveloppes: membranes.iter().map(Membrane::enveloppe).collect(),
        }
    }

    /// **Le fond vu au point `(x, y)` de l'écran** : celui du canevas, sur lequel chaque
    /// membrane se compose dans l'ordre du modèle — sa loi même, lue en ce point
    /// ([`Membrane::alpha`]), donc la couleur exacte qu'elle y peint.
    ///
    /// Une membrane dont l'enveloppe ne contient pas le point n'y pose rien : sa loi y vaut zéro,
    /// et la composer ne changerait pas un bit (fiche 97). Sur le squelette de Wikipédia, chaque
    /// nœud lisait les 190 membranes du tableau, deux fois par image.
    pub(crate) fn fond_en(&self, (x, y): (f32, f32)) -> (u8, u8, u8) {
        let (r, g, b) = self.fond;
        let mut c = [r, g, b].map(|v| f32::from(v) / 255.0);
        for (m, e) in self.membranes.iter().zip(&self.enveloppes) {
            if !(x >= e.gauche && x <= e.droite && y >= e.haut && y <= e.bas) {
                continue;
            }
            let a = m.alpha(x, y);
            for (canal, teinte) in c.iter_mut().zip(m.teinte) {
                *canal = *canal * (1.0 - a) + teinte * a;
            }
        }
        let [r, g, b] = c.map(|v| (v * 255.0).round().clamp(0.0, 255.0) as u8);
        (r, g, b)
    }
}

/// **Le fond de la carte, puis sa brume** (LUEUR-1, LUEUR-3).
///
/// Tauri pose sous le texte la teinte de la carte **à 3 %** (`color-mix(AURA 3%)`) et rien
/// d'autre : aucun cadre au repos, et la lueur, découpée à l'intérieur de la boîte, ne passe
/// jamais sous le texte ([`crate::renderer::halo`]). C'est ce qu'il appelle « le texte sur fond noir,
/// le contour en lueur ». Ici, la carte portait un fond presque opaque à 12 % de sa teinte et
/// un filet : un rectangle plein, qu'il trouvait laid.
///
/// # Le fond noir est peint (LUEUR-3)
///
/// Chez Tauri, ce fond noir n'était que la toile, vue au travers de 3 % de teinte. Posée sur
/// une photo, la carte montrait donc la photo sous son texte, qui devenait illisible (sa
/// capture du 26/09) ; et sur la toile, les points de la grille passaient entre les mots, où
/// ils se lisent comme une ponctuation — « que.on », sur une autre de ses captures.
///
/// Le fond noir est désormais **peint** : le fond du canevas lui-même, opaque, puis la brume à
/// 3 % — une seule couleur, un seul plein. Sur la toile vide, rien ne change à l'œil, sinon que
/// la grille s'arrête au bord de la carte ; sur une photo, le texte retrouve son fond. Aucune
/// opacité n'est choisie : « fond noir » veut dire le fond.
///
/// L'anneau qui désigne une carte sélectionnée ou éditée n'est pas ici : c'est une
/// affordance, pas un habit, et il se pose avec les autres (`dessus.rs`, COMPOSANT-4).
///
/// **La brume se peint par la loi des membranes** (COMPOSANT-4) : un plein d'une couche, une
/// composition par pixel là où il est constant, le calcul exact au seul bord. Le remplissage
/// anticrénelé de `tiny-skia` en coûtait les trois quarts du rendu d'une carte qu'on écrit —
/// 1,54 ms sur 2,05 pour sa carte de notes vue à ×2,3 (`bench_saisie`) —, pour un voile
/// uniforme. Et une loi évaluée pixel par pixel donne le même pixel quel que soit le morceau
/// de carte qu'on peint : c'est ce qui manquait pour ne refaire que les lignes qu'une frappe
/// change.
pub(super) fn draw_card_brume(
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    layout: &CardLayout,
    card: &TextCard,
) {
    let forme = forme_de_la_carte(at, (layout.width, layout.height), layout.radius);
    let brume = Membrane::plein(forme, voile_de_la_carte(card), 1.0);
    let (largeur, hauteur) = (pixmap.width(), pixmap.height());
    let (pixels, _) = pixmap.data_mut().as_chunks_mut::<4>();
    glucose_core::membrane_forme::peindre(&brume, pixels, largeur, hauteur);
}

/// **La couleur de la brume** : le fond qu'elle couvre, et la teinte de la carte à 3 % — de 0 à
/// 1 par canal, opaque. Le canevas la peint, le dessin vectoriel la pose.
pub(super) fn voile_de_la_carte(card: &TextCard) -> [f32; 3] {
    let brume = f32::from(BRUME) / 255.0;
    let (t, f) = (card.tint, card.fond);
    [(t.0, f.0), (t.1, f.1), (t.2, f.2)]
        .map(|(t, f)| (f32::from(f) * (1.0 - brume) + f32::from(t) * brume) / 255.0)
}

/// L'opacité de la brume sous le texte, sur 255 : les 3 % de Tauri.
pub(super) const BRUME: u8 = 8;

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
mod tests {
    use super::Contenants;
    use crate::theme::Theme;
    use glucose_core::membrane_forme::{Arrondi, Bord, Membrane};

    /// **Sauter une membrane loin du point ne change pas un bit du fond** (fiche 97) : des
    /// membranes emboîtées et voisines, à lueurs et à bord épais, et le fond lu en des milliers de
    /// points — dedans, au bord, dans les lueurs, dehors — égal au calcul qui les lit toutes.
    #[test]
    fn le_fond_saute_les_membranes_loin_du_point_sans_rien_changer() {
        let membrane = |x: f32, y: f32, l: f32, h: f32| {
            let fond = Arrondi::nouveau(x, y, l, h, 24.0);
            let halo = |pad: f32| {
                Arrondi::nouveau(
                    x - pad,
                    y - pad,
                    l + 2.0 * pad,
                    h + 2.0 * pad,
                    24.0 + pad / 2.0,
                )
            };
            Membrane {
                remplissages: [(halo(20.0), 0.03), (halo(10.0), 0.05), (fond, 0.03)],
                bord: Bord {
                    forme: fond,
                    demi_largeur: 3.0,
                    pointille: None,
                    alpha: 0.45,
                },
                teinte: [0.38, 0.65, 0.98],
            }
        };
        let membranes = [
            membrane(50.0, 40.0, 500.0, 300.0),
            membrane(80.0, 90.0, 200.0, 120.0),
            membrane(300.0, 90.0, 220.0, 200.0),
            membrane(600.0, 50.0, 100.0, 100.0),
        ];
        let theme = Theme::dark();
        let contenants = Contenants::nouveaux(&theme, &membranes);
        let toutes = |(x, y): (f32, f32)| {
            let (r, g, b) = super::fond_du_canevas(&theme);
            let mut c = [r, g, b].map(|v| f32::from(v) / 255.0);
            for m in &membranes {
                let a = m.alpha(x, y);
                for (canal, teinte) in c.iter_mut().zip(m.teinte) {
                    *canal = *canal * (1.0 - a) + teinte * a;
                }
            }
            let [r, g, b] = c.map(|v| (v * 255.0).round().clamp(0.0, 255.0) as u8);
            (r, g, b)
        };
        let mut teintes = std::collections::HashSet::new();
        for i in 0..400 {
            for j in 0..200 {
                let p = (i as f32 * 1.87 + 0.3, j as f32 * 1.91 + 0.7);
                let fond = contenants.fond_en(p);
                assert_eq!(fond, toutes(p), "au point {p:?}");
                teintes.insert(fond);
            }
        }
        assert!(
            teintes.len() > 20,
            "l'épreuve doit traverser lueurs et bords : {}",
            teintes.len()
        );
    }
}
