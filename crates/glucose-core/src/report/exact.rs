//! Le report **à l'identique** : un pixel de source pour un pixel de destination.
//!
//! C'est le chemin de la vignette et de la tuile posée à sa taille, et le cas dominant. La
//! ligne devient une recopie contiguë, que le compilateur ramène à un déplacement de mémoire
//! — sauf en source-over, où il faut savoir **où** copier, où sauter, et où mélanger.
//!
//! Deux façons de le savoir, et elles donnent les mêmes pixels au bit près :
//!
//! * la source porte ses [`super::Plages`], lues une fois au rangement : la ligne se traite plage
//!   par plage, sans lire un seul alpha ;
//! * elle ne les porte pas : la ligne se balaie, une comparaison par pixel, et les plages
//!   se reconnaissent au passage. C'est ce que coûtait chaque composition avant que les
//!   tuiles ne se souviennent des leurs.

use super::plages::Nature;
use super::{Melange, Pixel, Pose, Vue, VueMut};

/// Un pixel de source pour un pixel de destination : la ligne devient une recopie contiguë.
pub(super) fn reporter_tel_quel(
    dest: &mut VueMut<'_>,
    src: &Vue<'_>,
    pose: Pose,
    (x0, y0, x1, y1): (u32, u32, u32, u32),
    melange: Melange,
) -> u64 {
    let (dx, dy) = (pose.x as i64, pose.y as i64);
    let mut ecrits = 0u64;
    for y in y0..=y1 {
        let sy = i64::from(y) - dy;
        if sy < 0 || sy >= i64::from(src.hauteur()) {
            continue;
        }
        // Le décalage est le même sur les deux bords : la partie commune se lit d'un bloc,
        // et ce qui déborde de la source se rogne une fois pour toute la ligne.
        let sx0 = i64::from(x0) - dx;
        let sx1 = i64::from(x1) - dx;
        let rogne_gauche = (-sx0).max(0);
        let rogne_droite = (sx1 - (i64::from(src.largeur()) - 1)).max(0);
        let largeur = (sx1 - sx0 + 1) - rogne_gauche - rogne_droite;
        if largeur <= 0 {
            continue;
        }
        let depart_src = (sx0 + rogne_gauche) as usize;
        let depart_dest = (i64::from(x0) + rogne_gauche) as usize;
        let n = largeur as usize;
        let source = &src.ligne(sy as u32)[depart_src..depart_src + n];
        let cible = &mut dest.ligne_mut(y)[depart_dest..depart_dest + n];
        ecrits += match (melange, src.plages()) {
            (Melange::Remplacer, _) => {
                cible.copy_from_slice(source);
                n as u64
            }
            (Melange::Composer, Some(plages)) => {
                composer_par_plages(cible, source, plages.ligne(sy as u32), depart_src as u32)
            }
            (Melange::Composer, None) => {
                composer_la_ligne(cible, source);
                n as u64
            }
        };
    }
    ecrits
}

/// Compose une ligne dont les plages sont **connues** : copie les opaques, saute les
/// transparentes, mélange les mixtes. Rend combien de pixels ont été écrits.
///
/// `cible` et `source` sont la même tranche de ligne, qui commence à la colonne `depart`
/// de la source ; les plages sont celles de la ligne source entière, et se rognent à la
/// tranche.
fn composer_par_plages(
    cible: &mut [Pixel],
    source: &[Pixel],
    plages: &[super::plages::Plage],
    depart: u32,
) -> u64 {
    let fin = depart + source.len() as u32;
    let mut ecrits = 0u64;
    for plage in plages {
        let a = plage.x0.max(depart);
        let b = plage.x1.min(fin);
        if a >= b {
            continue;
        }
        let (i, j) = ((a - depart) as usize, (b - depart) as usize);
        match plage.nature {
            Nature::Transparente => continue,
            Nature::Opaque => cible[i..j].copy_from_slice(&source[i..j]),
            Nature::Mixte => {
                for (d, s) in cible[i..j].iter_mut().zip(&source[i..j]) {
                    *d = compose(*s, *d);
                }
            }
        }
        ecrits += u64::from(b - a);
    }
    ecrits
}

/// Compose une ligne source par-dessus une ligne cible, **par plages** reconnues au passage.
///
/// # Ce que cela change, et pourquoi c'est exact
///
/// Une tuile de photos est faite de trois sortes de pixels : ceux d'une photo opaque, alpha
/// 255 ; ceux où aucune photo ne passe, alpha 0 ; et une frange de bords anti-aliasés ou de
/// photos translucides, alpha entre les deux. Les deux premières sortes font l'immense
/// majorité, et pour elles le source-over a une réponse fermée :
///
/// * alpha 255 : `d = s + d × 0 = s` — une copie ;
/// * alpha 0 : `d = 0 + d × 1 = d` — rien à faire, puisqu'en prémultiplié un pixel
///   transparent est entièrement nul.
///
/// Calculer quatre multiplications par pixel pour aboutir à « copie » ou « rien » coûtait
/// cinq millisecondes par écran de 2560 × 1600. Reconnaître les plages coûte une comparaison
/// par pixel, et les plages elles-mêmes se traitent d'un bloc.
///
/// **Les pixels sont identiques au bit près** à ceux du calcul général, parce que
/// `mul255(d, 0) = 0` et `mul255(d, 255) = d` pour tout `d` — c'est une propriété de la
/// formule, vérifiée par un test. Le cas général reste pour ce qui n'est ni l'un ni l'autre.
///
/// Et cette comparaison par pixel, sur un octet lu tous les quatre, coûte encore plus que la
/// copie qu'elle prépare : c'est pourquoi les tuiles portent leurs [`super::Plages`], et ne
/// passent plus par ici.
pub(super) fn composer_la_ligne(cible: &mut [Pixel], source: &[Pixel]) {
    let n = cible.len().min(source.len());
    let mut i = 0;
    while i < n {
        let nature = Nature::de(source[i][3]);
        // La longueur de la plage de même nature, à partir d'ici.
        let mut fin = i + 1;
        while fin < n && Nature::de(source[fin][3]) == nature {
            fin += 1;
        }
        match nature {
            Nature::Opaque => cible[i..fin].copy_from_slice(&source[i..fin]),
            Nature::Transparente => {}
            Nature::Mixte => {
                for (d, s) in cible[i..fin].iter_mut().zip(&source[i..fin]) {
                    *d = compose(*s, *d);
                }
            }
        }
        i = fin;
    }
}

/// « Source par-dessus », en prémultiplié : `d = s + d × (1 − a)`.
pub(super) fn compose(s: Pixel, d: Pixel) -> Pixel {
    let inv = 255 - s[3];
    let mut sortie = [0u8; 4];
    for c in 0..4 {
        sortie[c] = s[c].saturating_add(mul255(d[c], inv));
    }
    sortie
}

/// `a × b / 255`, exact pour tous les octets, et sans division.
pub(super) fn mul255(a: u8, b: u8) -> u8 {
    let t = u32::from(a) * u32::from(b) + 128;
    ((t + (t >> 8)) >> 8) as u8
}
