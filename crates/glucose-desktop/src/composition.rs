//! Poser un tampon déjà dessiné dans l'image, au meilleur chemin disponible.
//!
//! # Ce que ce module répare, et pourquoi personne ne l'avait vu
//!
//! Trois endroits de Glucose gardent un tampon d'une image à l'autre — les panneaux du dock,
//! le fond de la minimap, et la région repeinte du rendu partiel. Tous les trois le
//! reposaient par `Pixmap::draw_pixmap`, c'est-à-dire par le pipeline générique de
//! `tiny-skia`.
//!
//! Le coût de ce pipeline annulait exactement le gain du cache. `bench_chrome` le mesurait
//! sans qu'on en tire la conclusion : les six panneaux du dock coûtaient 3,82 ms dessinés et
//! 3,48 ms depuis leur tampon — **un gain de 1,0×**, et deux panneaux y perdaient. La note
//! écrite alors concluait qu'un tampon ne valait pas le détour, « composer trois cent mille
//! pixels en source-over coûte à peu près ce que coûte le dessin qu'il remplace ».
//!
//! L'observation était juste, la conclusion trop large : elle mesurait un **rasteriseur**, pas
//! une machine. Le projet a depuis sa propre primitive de report — celle qui a fait passer
//! vingt-sept photos empilées de 340 ms à 3,4 ms — et elle a un chemin exact pour ce cas
//! précis : même taille, position entière, un pixel pour un pixel. Elle le **constate** sur
//! les valeurs plutôt que de le faire déclarer, donc aucun appelant ne peut s'y tromper.
//!
//! Le cache du dock n'était donc pas une idée mal mesurée. Il était branché sur le mauvais
//! moteur de composition. Mesuré après le changement : **3,44 ms dessinés, 0,85 ms depuis les
//! tampons — 4×**, et plus aucun panneau qui perde.
//!
//! # Pourquoi une fonction et non trois appels
//!
//! Les trois appelants posent le même objet de la même façon, et deux d'entre eux se sont
//! trompés de mode de mélange au moins une fois dans l'histoire du dépôt. Un seul endroit qui
//! sache poser un tampon, c'est un seul endroit où cette erreur peut exister.

use glucose_core::occlusion::Boite;
use glucose_core::report::{reporter, Filtre, Melange, Pose, Vue, VueMut};
use tiny_skia::{Pixmap, PixmapMut};

/// Pose `tampon` dans `dest` avec son coin haut-gauche en `(x, y)`, entiers.
///
/// Rend `false` — et n'écrit rien — quand les vues ne se construisent pas, c'est-à-dire quand
/// une taille est nulle. L'appelant retombe alors sur le rasteriseur : ne rien dessiner
/// effacerait ce qu'il voulait poser, ce qui serait pire que le coût évité.
pub fn poser(
    dest: &mut PixmapMut<'_>,
    tampon: &Pixmap,
    (x, y): (f32, f32),
    melange: Melange,
) -> bool {
    let (dw, dh) = (dest.width(), dest.height());
    let (sw, sh) = (tampon.width(), tampon.height());
    let (octets_src, _) = tampon.data().as_chunks::<4>();
    let Some(src) = Vue::nouvelle(octets_src, sw, sh) else {
        return false;
    };
    let (octets_dest, _) = dest.data_mut().as_chunks_mut::<4>();
    let Some(mut vue) = VueMut::nouvelle(octets_dest, dw, dh) else {
        return false;
    };
    // La position est arrondie ici, et c'est ce qui ouvre le chemin exact : un tampon posé
    // à une fraction de pixel repasserait par l'interpolation, pour un décalage que personne
    // n'a demandé. Les trois appelants calculent déjà des coins entiers.
    let pose = Pose {
        x: x.round(),
        y: y.round(),
        largeur: sw as f32,
        hauteur: sh as f32,
    };
    reporter(
        &mut vue,
        &src,
        pose,
        Boite::nouvelle(0.0, 0.0, dw as f32, dh as f32),
        melange,
        Filtre::Lisse,
    );
    true
}

#[cfg(test)]
mod tests;
