//! **Les pixels d'une pyramide** : la prémultiplication, et la réduction par deux.
//!
//! # MOYENNE-1 — chaque pixel d'un niveau est la moyenne de ce qu'il couvre
//!
//! Un pixel du rang `k` couvre un bloc de `2^k × 2^k` pixels natifs — moins, au bord droit et
//! au bord bas, quand un côté n'est pas une puissance de deux. La réduction moyennait ses
//! quatre pixels sources **à poids égaux**, et tronquait :
//!
//! * un bloc du bord, plus petit, pesait autant qu'un bloc entier. Au premier étage, cela ne
//!   change rien ; aux derniers, c'est grave — une image de trois pixels de large réduite à
//!   deux puis à un donnait au tiers droit la moitié du poids ;
//! * chaque étage arrondissait vers le bas : l'image s'assombrissait d'environ 0,4 unité par
//!   étage, cinq au sommet d'une photo.
//!
//! Chaque source pèse désormais **la surface native qu'elle couvre**, et la moyenne s'arrondit
//! au plus proche. Un pixel d'un niveau est alors la moyenne exacte de son bloc natif, à un
//! arrondi par étage près — et le sommet, le niveau d'un seul pixel, **est la couleur moyenne
//! de l'image**, que le tri par couleur lit (ORDONNER-2).

use tiny_skia::PixmapRef;

/// Prémultiplie des pixels RGBA droits dans `dest`, en entiers et arrondis au plus proche.
///
/// Sur le fil de fond, parce que la prémultiplication coûte autant que le décodage sur une
/// grande photo — la laisser au fil de rendu aurait déplacé le problème d'un mètre.
pub(super) fn premultiplier(rgba: &[u8], dest: &mut [u8]) {
    let (source, _) = rgba.as_chunks::<4>();
    let (destination, _) = dest.as_chunks_mut::<4>();
    for (px, out) in source.iter().zip(destination.iter_mut()) {
        let a = u32::from(px[3]);
        // `t = c·a + 128`, puis `(t + (t >> 8)) >> 8` : le décalage remplace la division par
        // 255 exactement sur toute la plage — c'est vérifié canal par canal, et `a = 255`
        // redonne `c` sans écart.
        //
        // La version flottante qui était ici tronquait : un canal à 255 sur un pixel opaque
        // ressortait à 254. Invisible sur une photo, visible sur un aplat près d'une bordure.
        // Ma première réécriture se trompait dans l'autre sens — 128 devenait 129 — et c'est
        // le test qui l'a dit, pas la relecture.
        let premultiplie = |c: u8| {
            let t = u32::from(c) * a + 128;
            ((t + (t >> 8)) >> 8) as u8
        };
        *out = [
            premultiplie(px[0]),
            premultiplie(px[1]),
            premultiplie(px[2]),
            px[3],
        ];
    }
}

/// **Ce que couvre chaque colonne (ou rangée) d'un niveau**, en pixels natifs : `2^rang`,
/// sauf la dernière, qui garde ce qui reste.
fn couvertures(natif: u32, cote: u32, rang: usize) -> Vec<u64> {
    let pas = 1u64 << rang;
    (0..u64::from(cote))
        .map(|i| pas.min(u64::from(natif) - i * pas))
        .collect()
}

/// Réduit le niveau `source`, de rang `rang` d'une image aux dimensions `natives`, de moitié
/// sur chaque côté : chaque pixel produit est la moyenne de ses (au plus) quatre sources,
/// **pondérées par la surface native qu'elles couvrent** (MOYENNE-1), arrondie au plus proche.
///
/// Les côtés impairs sont arrondis vers le haut : le dernier pixel d'une ligne impaire n'a pas
/// de voisin de droite, et la moyenne porte sur ce qui existe.
///
/// La moyenne se fait sur les octets tels quels, donc sur des valeurs prémultipliées en sRGB.
/// C'est ce que fait déjà le filtre bilinéaire du rasteriseur : le niveau réduit et la
/// réduction à la volée sont ainsi cohérents entre eux, ce qui est ce qui compte ici.
pub(super) fn reduire_par_deux(
    source: PixmapRef<'_>,
    dst: &mut [u8],
    (dw, dh): (u32, u32),
    natives: (u32, u32),
    rang: usize,
) {
    let (sw, sh) = (source.width(), source.height());
    let (cx, cy) = (
        couvertures(natives.0, sw, rang),
        couvertures(natives.1, sh, rang),
    );
    let src = source.data();
    // Un bloc **entier** — quatre sources qui couvrent chacune `2^rang` de côté — se moyenne
    // sans poids : ils sont tous égaux. Seuls la dernière colonne et la dernière rangée
    // peuvent en demander.
    let pas = 1u64 << rang;
    let entier = |c: &[u64], i: u32| c.get(2 * i as usize + 1).is_some_and(|&d| d == pas);
    for y in 0..dh {
        let rangee_entiere = entier(&cy, y);
        for x in 0..dw {
            let j = ((y * dw + x) * 4) as usize;
            if rangee_entiere && entier(&cx, x) {
                let (i, k) = (((2 * y * sw + 2 * x) * 4) as usize, (sw * 4) as usize);
                for c in 0..4 {
                    let s = u32::from(src[i + c])
                        + u32::from(src[i + 4 + c])
                        + u32::from(src[i + k + c])
                        + u32::from(src[i + k + 4 + c]);
                    dst[j + c] = ((s + 2) >> 2) as u8;
                }
                continue;
            }
            let mut somme = [0u64; 4];
            let mut poids = 0u64;
            for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let (sx, sy) = (x * 2 + ox, y * 2 + oy);
                if sx >= sw || sy >= sh {
                    continue;
                }
                let p = cx[sx as usize] * cy[sy as usize];
                let i = ((sy * sw + sx) * 4) as usize;
                for c in 0..4 {
                    somme[c] += p * u64::from(src[i + c]);
                }
                poids += p;
            }
            for c in 0..4 {
                dst[j + c] = ((somme[c] + poids / 2) / poids) as u8;
            }
        }
    }
}
