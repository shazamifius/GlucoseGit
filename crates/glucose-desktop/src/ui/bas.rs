//! **Le bas de l'écran : où se posent les barres qu'une sélection fait naître** (fiche 101).
//!
//! La barre d'action, celle des flèches et celle de couleur se centrent en bas, l'une sur l'autre ;
//! la minimap tient le coin bas-droit. Sur un écran large, elles ne se croisent pas. Sur un
//! téléphone debout — le sien a 393 points de large, le plus ancien de ses testeurs 360 —, la
//! barre de couleur était plus large que l'écran (« Couleur » et la roue coupées), et les barres
//! couvraient la minimap. Deux lois, ici seulement :
//!
//! * **Une barre ne sort jamais de l'écran** : ce qui ne tient pas sur une rangée passe à la
//!   suivante, comme un texte ([`ranger`]). Aucun bouton caché, aucun défilement — les deux
//!   réponses des boîtes à outils (un menu de débordement, une rangée qui défile) cachent un
//!   bouton, et un bouton caché n'existe pas pour qui ne le cherche pas.
//! * **La minimap ne bouge jamais** : elle est là tout le temps, la main sait où elle est. Une
//!   barre, qui naît et meurt avec la sélection, monte au-dessus d'elle quand elle la rencontrerait
//!   ([`poser`]), et celles qu'elle porte montent avec elle. Material 1 faisait l'inverse (le
//!   bouton flottant montait devant la barre passagère) ; ses règles d'aujourd'hui déconseillent
//!   d'animer les autres éléments avec elle. Ici, ce qui ne fait que passer ne déplace pas ce qui
//!   reste.
//!
//! Aucune constante neuve : le bord est celui de la minimap ([`super::minimap::BORD`]), l'écart
//! entre deux barres celui qui les séparait déjà ([`AU_DESSUS`]).

use super::minimap;

/// L'écart, en points, entre deux barres posées l'une sur l'autre — et entre une barre et la
/// minimap qu'elle enjambe.
pub const AU_DESSUS: f32 = 8.0;

/// **La largeur qu'une barre peut prendre** : l'écran visible, moins le bord de chaque côté.
pub fn largeur_utile(visible: (f32, f32), s: f32) -> f32 {
    visible.0 - 2.0 * minimap::BORD * s
}

/// **Pose une barre** de cette taille, son bas sur `sol` — le bord de l'écran, ou le haut de la
/// barre qu'elle surmonte : centrée sur l'écran visible et, si elle rencontrerait la minimap,
/// juste au-dessus d'elle. Rend son coin haut-gauche.
pub fn poser(visible: (f32, f32), s: f32, (largeur, hauteur): (f32, f32), sol: f32) -> (f32, f32) {
    let x = (visible.0 - largeur) / 2.0;
    let (mx, my, mw, mh) = minimap::emprise(visible, s);
    let rencontre = x < mx + mw && x + largeur > mx && sol - hauteur < my + mh && sol > my;
    let bas = if rencontre {
        sol.min(my - AU_DESSUS * s)
    } else {
        sol
    };
    (x, bas - hauteur)
}

/// **Range des éléments en rangées, comme un texte** : chacun à la suite du précédent, son écart
/// avant lui, et une rangée neuve quand il dépasserait `largeur_max` — l'écart d'un élément qui
/// ouvre sa rangée ne compte pas. `elements` : `(écart avant, largeur)`.
///
/// Rend, pour chacun, sa rangée et son abscisse depuis le début de la rangée ; et la largeur de la
/// plus longue rangée. Un élément plus large que `largeur_max` a sa rangée à lui.
pub fn ranger(elements: &[(f32, f32)], largeur_max: f32) -> (Vec<(usize, f32)>, f32) {
    let mut places = Vec::with_capacity(elements.len());
    let (mut rangee, mut curseur, mut plus_longue) = (0, 0.0_f32, 0.0_f32);
    for (i, &(ecart, largeur)) in elements.iter().enumerate() {
        let x = if i == 0 {
            0.0
        } else if curseur + ecart + largeur > largeur_max {
            rangee += 1;
            0.0
        } else {
            curseur + ecart
        };
        places.push((rangee, x));
        curseur = x + largeur;
        plus_longue = plus_longue.max(curseur);
    }
    (places, plus_longue)
}

#[cfg(test)]
mod tests;
