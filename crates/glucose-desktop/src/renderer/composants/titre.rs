//! **TITRE-3 — le titre d'une membrane est un composant** (fiche 108) : rendu une fois, posé à
//! chaque image par la carte graphique, comme une carte de texte (COMPOSANT-1).
//!
//! # Ce que la mesure a dit
//!
//! Sur sa carte de Wikipédia, de loin, par la voie graphique, le profil de la fiche 107 trouvait
//! **26 %** du temps d'une image dans les titres des membranes, redessinés lettre à lettre au
//! processeur dans la couche du dessous à chaque image (`draw_text_with_outline`, dont 20 % à
//! poser des pixels de lettres) — puis cette couche renvoyée à la carte (9 %). Un titre ne
//! change pourtant que si son texte, sa couleur ou son corps changent.
//!
//! # La loi
//!
//! Le titre se rend dans sa texture par **le même dessin** que la voie processeur pose en place
//! ([`crate::typography::Typography::draw_text_with_outline`]) : à l'arrêt, à la même phase,
//! posé sur un pixel entier — les mêmes pixels. En mouvement, au palier dyadique, posé à sa
//! place fractionnaire. Sa texture contient toute son encre
//! ([`crate::typography::Typography::encre_detouree`]).
//!
//! Son corps est celui **du monde**, ramené à la largeur de sa membrane en unités du monde :
//! il ne change pas pendant un zoom, et la texture ne se refait qu'à chaque palier.
//!
//! # Ce qui reste au processeur
//!
//! Le titre **qu'on écrit** (sa sélection et son curseur bougent à chaque frappe, et il passe
//! au-dessus des autres), et celui dont la texture **dépasserait la fenêtre** — de très près
//! d'une très grande membrane : une texture plus grande que l'écran coûterait des pixels que
//! personne ne voit. La règle ([`titre_par_la_carte`]) ne dépend que de la vue et de la
//! fenêtre : la couche du dessous et la récolte la lisent pareil, et un titre se dessine donc
//! une fois, jamais zéro ni deux.

use super::contenu::Contenu;
use super::{palier_dyadique, Composant, Regime};
use crate::canvas::world_to_screen;
use crate::renderer::scale::WorldScale;
use crate::renderer::PaintKit;
use crate::typography::{Face, TextStyle};
use std::sync::Arc;
use tiny_skia::{Color, Pixmap};

/// **Un titre tel que le monde le dit** : de quelle membrane, quel texte, où et à quelle taille
/// — ses mesures en unités du monde, déjà ramenées à la largeur de sa membrane (TITRE-1).
pub(in crate::renderer) struct TitreDuMonde<'a> {
    /// L'identifiant de sa membrane : l'identité de la texture.
    pub membrane: &'a str,
    pub texte: &'a str,
    /// Le coin haut-gauche de sa membrane, dans le monde.
    pub coin: (f64, f64),
    /// Son corps, son retrait depuis le bord gauche, son élévation au-dessus du bord haut.
    pub mesures: [f32; 3],
    /// La teinte de ses lettres, et celle de son détourage.
    pub teintes: ((u8, u8, u8), Color),
}

/// **Le titre se pose-t-il par la carte graphique ?** (TITRE-3) — vrai si sa texture tient dans
/// la fenêtre à la plus grande échelle où elle peut se rendre dans cette vue : l'exacte, ou son
/// palier en mouvement. Le titre qu'on écrit n'y passe jamais : l'appelant l'écarte.
///
/// Une fonction de la vue et de la fenêtre seules : la couche du dessous, qui dessine ce que la
/// carte ne pose pas, et la récolte, qui confie le reste, ne peuvent pas répondre autrement.
pub(in crate::renderer) fn titre_par_la_carte(
    kit: PaintKit<'_>,
    (texte, corps): (&str, f32),
    vue: f64,
    fenetre: (u32, u32),
) -> bool {
    let echelle = vue.max(palier_dyadique(vue));
    let (g, h, d, b) = kit
        .typography
        .encre_detouree(texte, corps_a(corps, echelle), Face::Bold);
    // La même taille que la texture : le pixel de plus porte sa phase (`Regime::assembler`).
    (d - g + 1.0).ceil() <= fenetre.0 as f32 && (b - h + 1.0).ceil() <= fenetre.1 as f32
}

/// **Le corps d'un titre à cette échelle** : la seule mise à l'échelle (SCALE-1), celle que la
/// voie processeur emploie — à l'arrêt, les mêmes bits. La densité de l'écran n'y entre pas.
fn corps_a(corps: f32, echelle: f64) -> f32 {
    WorldScale::new(echelle, 1.0).world(corps)
}

impl Regime {
    /// **La texture d'un titre de membrane**, à son rang sous les photos (TITRE-3).
    ///
    /// Sa plume tombe là où la voie processeur l'écrit — le coin de la membrane, plus son
    /// retrait, moins son élévation, par la même mise à l'échelle (SCALE-1) — dans la vue des
    /// textures, décalée comme une carte pendant une translation entière (fiche 100). Le coin
    /// de la texture est celui de son encre mesurée **à l'échelle de rendu**, ramenée à la vue :
    /// en mouvement, la plume reste exactement à sa place.
    pub(in crate::renderer) fn titre(&self, kit: PaintKit<'_>, t: &TitreDuMonde<'_>) -> Composant {
        let s = WorldScale::new(self.vue.scale, self.densite);
        let (vue_des_textures, (dx, dy)) = self.textures;
        let (sx, sy) = world_to_screen(t.coin.0, t.coin.1, &vue_des_textures);
        let [corps, retrait, elevation] = t.mesures;
        let plume = (sx as f32 + s.world(retrait), sy as f32 - s.world(elevation));
        let (g, h, d, b) =
            kit.typography
                .encre_detouree(t.texte, corps_a(corps, self.echelle), Face::Bold);
        let rapport = self.rapport();
        let (phase, pose) = self.phase_et_coin(plume.0 + g * rapport, plume.1 + h * rapport);
        let contenu = Arc::new(Contenu::Titre {
            texte: t.texte.to_string(),
            corps,
            teinte: t.teintes.0,
            contour: t.teintes.1,
        });
        self.assembler(
            (format!("titre:{}", t.membrane), &contenu),
            self.echelle,
            (phase, (pose.0 + dx, pose.1 + dy)),
            (d - g, b - h, 0.0),
        )
    }
}

/// **Rend un titre dans sa texture** : le dessin même de la voie processeur, la plume placée
/// pour que son encre commence au coin, à sa phase.
pub(super) fn rendre(kit: PaintKit<'_>, composant: &Composant, pixmap: &mut Pixmap) {
    let Contenu::Titre {
        texte,
        corps,
        teinte: (r, g, b),
        contour,
    } = composant.contenu.as_ref()
    else {
        return;
    };
    let corps = corps_a(*corps, composant.echelle);
    let (gauche, haut, _, _) = kit.typography.encre_detouree(texte, corps, Face::Bold);
    let style = TextStyle {
        size: corps,
        color: Color::from_rgba8(*r, *g, *b, 255),
        face: Face::Bold,
    };
    let (x, y) = (composant.phase.0 - gauche, composant.phase.1 - haut);
    kit.typography
        .draw_text_with_outline(&mut pixmap.as_mut(), texte, x, y, style, *contour);
}

#[cfg(test)]
mod tests;
