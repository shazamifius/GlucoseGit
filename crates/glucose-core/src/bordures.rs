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
//! **Un centile, et non la moyenne ni le pire pixel.** Quatre critères ont été essayés, et
//! chacun a été refusé par une mesure :
//!
//! | critère | ce qu'il fait de faux | ce qui l'a dit |
//! |---|---|---|
//! | la **moyenne** (FFmpeg) | mange le haut des lettres : la première ligne de pixels d'un texte porte trois pour cent d'encre, et sa moyenne reste sous le seuil | un test de texte fin sur fond uni |
//! | la **médiane** | prend un damier entier pour une bande — la moitié des pixels d'une ligne de texte sont du fond | le damier des preuves |
//! | le **pire pixel** | s'arrête trop tôt et **laisse un liseré visible** : sur une illustration réelle, la première ligne refusée portait quatre pixels de plume sur cinq cent six | l'image de l'utilisateur, mesurée |
//! | le **centile 99** | retenu | — |
//!
//! Une ligne est de la bande si **au plus un centième** de ses pixels s'écartent du bord de
//! plus que le bruit. Ce que cela laisse passer est ce qu'on ne perd pas à couper : sur une
//! ligne de cinq cents pixels, un pour cent en fait cinq — le bruit d'un bord compressé, ou le
//! premier rang anti-crénelé d'une forme. Ce que cela arrête est un contenu, qui en occupe
//! davantage dès sa première ligne.
//!
//! **Ce nombre se juge à la main**, comme `TAU_CONDUITE` et `PAN_LIGNE_PX` : aucune loi ne le
//! donne, et son journal est au-dessus de [`PART_ABERRANTE`]. Un filigrane posé dans la bande
//! l'arrête à sa ligne, comme chez FFmpeg — c'est assumé, et un test le dit.
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

/// **La part des pixels d'une ligne qui peuvent s'écarter sans qu'elle cesse d'être de la
/// bande.**
///
/// # Ce nombre se juge à la main, et voici son journal
///
/// * **zéro** — le pire pixel. Refusé par l'image de l'utilisateur : la première ligne refusée
///   du haut de son illustration portait **quatre pixels de plume sur cinq cent six**, et les
///   trois lignes suivantes six, huit et treize. Le recadrage s'arrêtait donc trois lignes
///   avant la vraie frontière, et **le liseré blanc restant se voyait à l'écran** ;
/// * **un pour cent** — retenu. Sur une ligne de cinq cents pixels, cinq pixels : le bruit d'un
///   bord compressé, ou le premier rang anti-crénelé d'une forme. Ce n'est pas ce qu'on perd à
///   couper ;
/// * **deux pour cent** — pas essayé sur le terrain. Il couperait trois lignes de plus sur la
///   même image, et rognerait la première ligne d'un texte fin à moins de deux pour cent
///   d'encre. À reprendre si un cas réel le réclame.
///
/// Ce n'est **pas** [`crate::cadence::PART_TOLEREE`], qui vaut le même nombre : celle-là
/// compte des images ratées par une cadence, celle-ci des pixels dans une ligne. Deux
/// grandeurs sans rapport qui partagent une valeur ne doivent pas partager une constante —
/// c'est exactement la faute d'ARBITRE-1, qui avait repris le seuil du tempo sans reprendre sa
/// grandeur.
pub const PART_ABERRANTE: f64 = 0.01;

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
/// bande si au plus [`PART_ABERRANTE`] de ses pixels s'écartent de cette couleur de plus que
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
        if ecart_au_centile(&pixels, couleur) > ECART_DE_BANDE {
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

/// **L'écart du pixel qui laisse [`PART_ABERRANTE`] de la ligne au-dessus de lui.**
///
/// Chaque pixel donne son écart sur son canal le plus éloigné ; on regarde celui qui sépare le
/// dernier centième du reste. Quelques pixels d'encre ne suffisent donc plus à arrêter une
/// bande — ce qui laissait un liseré —, mais une ligne de contenu, qui en porte bien
/// davantage dès son premier rang, l'arrête toujours.
fn ecart_au_centile(pixels: &[Pixel], couleur: Pixel) -> f64 {
    let mut ecarts: Vec<u8> = pixels
        .iter()
        .map(|p| (0..3).map(|c| p[c].abs_diff(couleur[c])).max().unwrap_or(0))
        .collect();
    if ecarts.is_empty() {
        return 0.0;
    }
    ecarts.sort_unstable();
    // Le rang qui laisse `PART_ABERRANTE` de la ligne au-dessus de lui. Une ligne courte le
    // ramène au dernier pixel, ce qui redonne le pire : sur dix pixels, un centième n'existe
    // pas, et tolérer un pixel sur dix serait tolérer dix pour cent.
    let n = ecarts.len();
    let hors = ((n as f64) * PART_ABERRANTE).floor() as usize;
    f64::from(ecarts[n - 1 - hors.min(n - 1)])
}

#[cfg(test)]
mod tests;
