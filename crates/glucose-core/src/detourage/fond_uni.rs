//! **Retirer le fond uni** (DETOURER-3, fiche 121) — sans IA, exact, instantané.
//!
//! Sa parole du 18/11 parle d'une image *« à fond uni blanc ou complexe »*. Sur un dessin au trait,
//! l'IA garde tout le blanc entre les traits (mesuré : 32 % d'erreur, fiche 121 § 3) — elle cherche
//! un objet, et le dessin entier en est un. Ici, rien à deviner : **la couleur du fond** est celle du
//! bord de l'image (la médiane de son cadre d'un pixel, par canal), et chaque pixel devient **aussi
//! transparent que possible, tel que, reposé sur cette couleur, il redonne exactement l'original** —
//! la « couleur vers alpha » de GIMP, à ses réglages par défaut (aucun seuil :
//! <https://docs.gimp.org/2.10/en/gimp-filter-color-to-alpha.html>). Un trait gris d'anticrénelage
//! devient un trait noir à demi transparent : le bord reste lisse sur n'importe quel fond.
//!
//! Pour chaque canal `c` de couleur `I`, sur le fond `B` (entre 0 et 1) :
//!
//! ```text
//! α_c = (I_c − B_c) / (1 − B_c)   si I_c > B_c        α = max α_c
//! α_c = (B_c − I_c) / B_c         si I_c < B_c        F = B + (I − B) / α
//! ```
//!
//! Ce qui, dans l'objet, a exactement la couleur du fond devient transparent aussi : c'est ce que le
//! geste dit (« le fond uni »), et `Ctrl+Z` le rend.

/// **La couleur du fond** : la médiane, canal par canal, des pixels opaques du cadre de l'image —
/// `None` si le cadre est déjà transparent.
pub fn couleur_du_bord(rvba: &[u8], (l, h): (usize, usize)) -> Option<[u8; 3]> {
    let cadre = (0..l)
        .flat_map(|x| [(x, 0), (x, h - 1)])
        .chain((1..h.saturating_sub(1)).flat_map(|y| [(0, y), (l - 1, y)]));
    let mut canaux: [Vec<u8>; 3] = Default::default();
    for (x, y) in cadre {
        let p = &rvba[(y * l + x) * 4..][..4];
        if p[3] == 255 {
            (0..3).for_each(|c| canaux[c].push(p[c]));
        }
    }
    if canaux[0].is_empty() {
        return None;
    }
    Some(std::array::from_fn(|c| {
        canaux[c].sort_unstable();
        canaux[c][canaux[c].len() / 2]
    }))
}

/// **L'image sans son fond uni**, en RVBA non prémultiplié — ou `None` si son bord est déjà
/// transparent (il n'y a pas de fond à retirer).
pub fn retirer(rvba: &[u8], (l, h): (usize, usize)) -> Option<Vec<u8>> {
    let fond = couleur_du_bord(rvba, (l, h))?.map(|v| f32::from(v) / 255.0);
    let mut sortie = Vec::with_capacity(rvba.len());
    for p in rvba.as_chunks::<4>().0 {
        let i: [f32; 3] = std::array::from_fn(|c| f32::from(p[c]) / 255.0);
        let alpha = (0..3).map(|c| part(i[c], fond[c])).fold(0.0_f32, f32::max);
        if alpha <= 0.0 {
            sortie.extend_from_slice(&[0, 0, 0, 0]);
            continue;
        }
        for c in 0..3 {
            let f = (fond[c] + (i[c] - fond[c]) / alpha).clamp(0.0, 1.0);
            sortie.push((f * 255.0).round() as u8);
        }
        // L'opacité qu'avait déjà le pixel se garde : on retire le fond, pas davantage.
        sortie.push((alpha * f32::from(p[3])).round() as u8);
    }
    Some(sortie)
}

/// L'opacité qu'exige un canal : la part du chemin entre le fond et le bout de l'échelle.
fn part(i: f32, b: f32) -> f32 {
    if i > b {
        (i - b) / (1.0 - b)
    } else if i < b {
        (b - i) / b
    } else {
        0.0
    }
}
