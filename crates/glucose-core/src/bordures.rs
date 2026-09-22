//! **BORDURES-1** : trouver les bandes unies qui entourent une image, pour les retirer.
//!
//! # Ce que l'utilisateur demande
//!
//! *« Un bouton qui retire les bordures d'un lot d'images sélectionnées — détecter les bandes
//! noires ou blanches, puis rogner. »* Une capture d'écran de vidéo en boîte aux lettres, une
//! photo scannée avec sa marge, une image de site web sur son fond : la bande est du bruit
//! autour de ce qu'on voulait garder, et la retirer à la main sur cinquante images est
//! exactement le genre de corvée qu'un logiciel doit faire seul.
//!
//! # Ce que la référence fait, et ce qu'on en garde
//!
//! `cropdetect`, le filtre de FFmpeg qui fait ce travail sur des millions de vidéos depuis
//! vingt ans, procède ainsi : depuis chaque bord, ligne après ligne, il fait la **moyenne** de
//! la ligne et s'arrête à la première dont la moyenne dépasse un seuil de noir — **24 sur
//! 255** par défaut, soit le noir du signal vidéo (16) plus une marge pour le bruit.
//!
//! On garde la structure — quatre balayages depuis les bords, un arrêt à la première ligne
//! qui n'est plus bande — et on change deux choses, que deux tests ont imposées.
//!
//! **La bande n'est pas noire, elle est unie.** Ce qu'on mesure n'est donc pas la clarté de la
//! ligne, mais sa **distance à la couleur du bord**. Une bande blanche, grise ou d'une couleur
//! de fond se détecte alors exactement comme une noire, et le seuil de FFmpeg garde son sens :
//! c'est l'écart qu'un noir vidéo bruité atteint sans cesser d'être du noir.
//!
//! **Le pire pixel de la ligne, et non sa moyenne.** FFmpeg moyenne la ligne, et c'est juste
//! pour une vidéo : ce qu'il coupe est du noir bruité, ce qu'il garde est de l'image. Glucose
//! coupe des captures d'écran, et ce qu'il garde est **du texte fin sur un fond uni** — le
//! contenu le plus fréquent du canevas. La première ligne de pixels du haut des lettres porte
//! trois pour cent d'encre ; sa moyenne reste sous le seuil, et la moyenne la mange. Deux
//! essais l'ont dit : la moyenne perd le haut des lettres, et la médiane — essayée pour tenir
//! malgré un filigrane — prend un damier entier pour une bande, puisque la moitié de ses pixels
//! sont d'une même couleur.
//!
//! Une ligne est donc de la bande si **aucun** de ses pixels ne s'écarte du bord de plus que le
//! bruit. Le défaut de ce critère est de s'arrêter une ligne trop tôt sur un pixel de bruit
//! isolé, ce qui laisse une ligne de bande et ne se voit pas ; le défaut de la moyenne est de
//! manger du contenu, ce qui se voit. **On choisit le défaut qui ne se voit pas.** Un filigrane
//! posé dans la bande l'arrête à sa ligne, comme chez FFmpeg — c'est assumé, et un test le dit.
//!
//! # Les quatre bords se rognent ensemble, jusqu'à ce que plus rien ne bouge
//!
//! Le bord droit d'une image encadrée traverse les bandes du haut et du bas, qui n'ont aucune
//! raison d'être de sa couleur : mesuré sur toute la hauteur, il n'est pas uni, et on ne le
//! trouve pas. Le second test l'a dit. Chaque bord se mesure donc **à l'intérieur de ce que les
//! autres ont déjà retiré**, et on recommence tant qu'un bord recule encore. Deux passes
//! suffisent en pratique ; la boucle s'arrête d'elle-même parce qu'un rectangle ne peut que
//! rétrécir.
//!
//! # Ce qui trompe, et qu'on ne cache pas
//!
//! Une photo dont le **contenu** commence par une zone unie — un ciel sans nuage en haut, un
//! mur blanc à gauche — se fera rogner de cette zone. FFmpeg a le même défaut et le
//! documente. Il n'y a pas de critère local qui distingue un ciel uni d'une bande, parce que
//! ce sont les mêmes pixels ; seule l'intention les sépare. L'action est donc **explicite et
//! annulable** — un geste, un `Ctrl+Z` — et jamais automatique.
//!
//! Et une image entièrement unie proposerait de tout retirer : c'est l'invariant du
//! [`Recadrage`] qui garantit alors qu'il reste quelque chose, pas ce module.

use crate::report::{Pixel, Vue};
use crate::types::Recadrage;

/// **L'écart au-delà duquel un pixel n'est plus de la bande**, en niveaux sur 255.
///
/// C'est le seuil de noir de `cropdetect` — `24.0 / 255` dans `vf_cropdetect.c` — et il n'est
/// pas à nous : c'est le noir du signal vidéo, seize, plus ce que le bruit de compression y
/// ajoute sans qu'on cesse de le voir noir. Appliqué à une distance plutôt qu'à une clarté, il
/// dit la même chose d'une bande blanche que d'une bande noire.
pub const ECART_DE_BANDE: f64 = 24.0;

/// **Les bandes unies qui entourent cette image**, comme recadrage à lui appliquer.
///
/// Rend [`Recadrage::ENTIER`] quand il n'y en a aucune. Le coût est celui des bandes, pas de
/// l'image : le balayage s'arrête à la première ligne qui n'en est plus, donc une photo sans
/// bordure ne coûte que quatre lignes.
pub fn detecter(image: &Vue<'_>) -> Recadrage {
    let (l, h) = (image.largeur(), image.hauteur());
    // Le rectangle qui reste : [x0, x1[ × [y0, y1[. Il ne peut que rétrécir.
    let (mut x0, mut y0, mut x1, mut y1) = (0u32, 0u32, l, h);
    loop {
        let avant = (x0, y0, x1, y1);
        y0 += compter(y0..y1, |y| ligne(image, y, x0..x1));
        y1 -= compter((y0..y1).rev(), |y| ligne(image, y, x0..x1));
        x0 += compter(x0..x1, |x| colonne(image, x, y0..y1));
        x1 -= compter((x0..x1).rev(), |x| colonne(image, x, y0..y1));
        if (x0, y0, x1, y1) == avant {
            break;
        }
    }
    Recadrage::depuis_les_marges(
        f64::from(x0) / f64::from(l),
        f64::from(y0) / f64::from(h),
        f64::from(l - x1) / f64::from(l),
        f64::from(h - y1) / f64::from(h),
    )
}

/// Combien de lignes, prises dans cet ordre, sont de la bande.
///
/// La couleur de référence est celle de la **première** ligne parcourue — le bord —, prise
/// comme médiane par canal pour qu'un logo dans le coin ne la fausse pas. Une ligne est de la
/// bande si **aucun** de ses pixels ne s'écarte de cette couleur de plus que
/// [`ECART_DE_BANDE`].
fn compter<I, F>(ordre: I, mut ligne_de: F) -> u32
where
    I: IntoIterator<Item = u32>,
    F: FnMut(u32) -> Vec<Pixel>,
{
    let mut reference: Option<Pixel> = None;
    let mut combien = 0u32;
    for indice in ordre {
        let pixels = ligne_de(indice);
        if pixels.is_empty() {
            break;
        }
        let couleur = *reference.get_or_insert_with(|| mediane(&pixels));
        if pire_ecart(&pixels, couleur) > ECART_DE_BANDE {
            break;
        }
        combien += 1;
    }
    combien
}

/// La ligne `y`, sur les colonnes `xs`, copiée : les colonnes ne sont pas contiguës, et une
/// seule forme pour les deux vaut mieux que deux boucles.
fn ligne(image: &Vue<'_>, y: u32, xs: std::ops::Range<u32>) -> Vec<Pixel> {
    xs.map(|x| image.pixel(x, y)).collect()
}

/// La colonne `x`, sur les lignes `ys`, copiée.
fn colonne(image: &Vue<'_>, x: u32, ys: std::ops::Range<u32>) -> Vec<Pixel> {
    ys.map(|y| image.pixel(x, y)).collect()
}

/// La médiane de chaque canal, canal par canal.
///
/// Pas la moyenne : un filigrane blanc sur vingt pixels d'une bande noire de mille la
/// déplacerait de cinq niveaux, et la médiane, pas d'un.
fn mediane(pixels: &[Pixel]) -> Pixel {
    let mut sortie = [0u8; 4];
    let mut canal = Vec::with_capacity(pixels.len());
    for (c, valeur) in sortie.iter_mut().enumerate() {
        canal.clear();
        canal.extend(pixels.iter().map(|p| p[c]));
        canal.sort_unstable();
        *valeur = canal.get(canal.len() / 2).copied().unwrap_or(0);
    }
    sortie
}

/// **Le pixel de la ligne le plus éloigné de cette couleur**, sur son canal le plus éloigné.
///
/// Le seul pixel d'encre d'une ligne suffit à dire qu'elle n'est plus de la bande — c'est ce
/// qui empêche de manger le haut des lettres, là où une moyenne passait dessous.
fn pire_ecart(pixels: &[Pixel], couleur: Pixel) -> f64 {
    pixels
        .iter()
        .map(|p| (0..3).map(|c| p[c].abs_diff(couleur[c])).max().unwrap_or(0))
        .max()
        .map_or(0.0, f64::from)
}

#[cfg(test)]
mod tests;
