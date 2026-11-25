//! **Une image YUV 4:2:0 en RGBA** (VIDEO-ANDROID-1, fiche 172) : ce que le lecteur d'Android
//! tend (`YUV_420_888`), recadré, mis en couleurs par la matrice que la vidéo dit, puis tourné
//! comme elle se regarde. GStreamer fait tout cela lui-même sous Linux (`videoconvert`,
//! `videoflip`) ; Android le laisse à qui lit l'image.
//!
//! # La matrice
//!
//! Android 13 dit l'espace de couleurs de chaque image (`Image.getDataSpace`). Avant lui, ou s'il
//! ne dit rien, celle que les décodeurs d'Android eux-mêmes prennent par défaut
//! (`ColorUtils::getColorConfigFromDataSpace`, résumée) : BT.709 pour une image de 1280 de côté ou
//! plus — ce que filment les téléphones —, BT.601 en dessous, l'étendue réduite (16-235).

use crate::plateforme::video::{Plan, Plans};
use crate::renderer::gif::Toile;
use std::time::Duration;

// `android.hardware.DataSpace` : les bits du standard et de l'étendue.
const STANDARD: i32 = 63 << 16;
const BT709: i32 = 1 << 16;
const BT601_625: i32 = 2 << 16;
const BT601_625_NON_AJUSTE: i32 = 3 << 16;
const BT601_525: i32 = 4 << 16;
const BT601_525_NON_AJUSTE: i32 = 5 << 16;
const BT2020: i32 = 6 << 16;
const BT2020_LUMINANCE_CONSTANTE: i32 = 7 << 16;
const ETENDUE: i32 = 7 << 27;
const ETENDUE_PLEINE: i32 = 1 << 27;

/// Le côté à partir duquel une image sans espace dit est tenue pour HD.
const COTE_HD: u32 = 1280;

/// Les nombres d'une matrice, en virgule fixe sur [`UN`].
const DECALAGE: u32 = 14;
const UN: f32 = (1 << DECALAGE) as f32;

/// **Une matrice YUV → RGB**, en virgule fixe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Matrice {
    noir: i32,
    y: i32,
    rv: i32,
    gu: i32,
    gv: i32,
    bu: i32,
}

impl Matrice {
    /// La matrice de ces poids (Kr, Kb), à l'étendue pleine ou réduite.
    fn de((kr, kb): (f32, f32), pleine: bool) -> Self {
        let kg = 1.0 - kr - kb;
        let (y, c, noir) = if pleine {
            (1.0, 1.0, 0)
        } else {
            (255.0 / 219.0, 255.0 / 224.0, 16)
        };
        let f = |v: f32| (v * UN).round() as i32;
        Self {
            noir,
            y: f(y),
            rv: f(2.0 * (1.0 - kr) * c),
            gu: f(2.0 * kb * (1.0 - kb) / kg * c),
            gv: f(2.0 * kr * (1.0 - kr) / kg * c),
            bu: f(2.0 * (1.0 - kb) * c),
        }
    }

    /// La matrice que cette image dit, ou celle qu'Android prend pour elle.
    fn pour(espace: i32, (largeur, hauteur): (u32, u32)) -> Self {
        const P709: (f32, f32) = (0.2126, 0.0722);
        const P601: (f32, f32) = (0.299, 0.114);
        const P2020: (f32, f32) = (0.2627, 0.0593);
        let poids = match espace & STANDARD {
            BT709 => P709,
            BT601_625 | BT601_625_NON_AJUSTE | BT601_525 | BT601_525_NON_AJUSTE => P601,
            BT2020 | BT2020_LUMINANCE_CONSTANTE => P2020,
            _ if largeur.max(hauteur) >= COTE_HD => P709,
            _ => P601,
        };
        Self::de(poids, espace & ETENDUE == ETENDUE_PLEINE)
    }

    /// Un pixel.
    #[inline]
    fn rgba(&self, y: u8, u: u8, v: u8) -> [u8; 4] {
        let y = (i32::from(y) - self.noir) * self.y + (1 << (DECALAGE - 1));
        let (u, v) = (i32::from(u) - 128, i32::from(v) - 128);
        let c = |x: i32| (x >> DECALAGE).clamp(0, 255) as u8;
        [
            c(y + self.rv * v),
            c(y - self.gu * u - self.gv * v),
            c(y + self.bu * u),
            255,
        ]
    }
}

/// **L'octet du plan en (x, y)** : la place se vérifie une fois pour toute l'image
/// ([`assez_long`]).
#[inline]
fn octet(plan: &Plan, x: usize, y: usize) -> u8 {
    plan.octets[y * plan.rangee + x * plan.pas]
}

/// Le plan porte-t-il ses `largeur` × `hauteur` échantillons, depuis (x, y) ?
fn assez_long(plan: &Plan, (x, y): (usize, usize), (largeur, hauteur): (usize, usize)) -> bool {
    let dernier = (y + hauteur - 1) * plan.rangee + (x + largeur - 1) * plan.pas;
    dernier < plan.octets.len()
}

/// **L'image en RGBA**, recadrée et tournée ; `None` si ses plans ne portent pas ce qu'elle dit
/// — une image qu'on ne sait pas lire ne se devine pas.
pub(crate) fn en_rgba(p: &Plans) -> Option<Toile> {
    let [x0, y0, l, h] = p.recadrage.map(|v| v as usize);
    if l == 0 || h == 0 {
        return None;
    }
    let chroma = (x0 / 2, y0 / 2);
    let cote_chroma = (
        (x0 + l - 1) / 2 - chroma.0 + 1,
        (y0 + h - 1) / 2 - chroma.1 + 1,
    );
    if !assez_long(&p.y, (x0, y0), (l, h))
        || !assez_long(&p.u, chroma, cote_chroma)
        || !assez_long(&p.v, chroma, cote_chroma)
    {
        return None;
    }
    let (largeur, hauteur) = (u32::try_from(l).ok()?, u32::try_from(h).ok()?);
    let m = Matrice::pour(p.espace, (largeur, hauteur));
    let mut rgba = Vec::with_capacity(l * h * 4);
    for y in y0..y0 + h {
        for x in x0..x0 + l {
            let (cx, cy) = (x / 2, y / 2);
            rgba.extend(m.rgba(octet(&p.y, x, y), octet(&p.u, cx, cy), octet(&p.v, cx, cy)));
        }
    }
    tourner(rgba, (largeur, hauteur), p.quarts)
}

/// **Tourne l'image** de ces quarts de tour, dans le sens des aiguilles d'une montre.
fn tourner(rgba: Vec<u8>, (l, h): (u32, u32), quarts: u8) -> Option<Toile> {
    use image::imageops::{rotate180, rotate270, rotate90};
    let image = image::RgbaImage::from_raw(l, h, rgba)?;
    let image = match quarts % 4 {
        1 => rotate90(&image),
        2 => rotate180(&image),
        3 => rotate270(&image),
        _ => image,
    };
    Some(Toile {
        largeur: image.width(),
        hauteur: image.height(),
        rgba: image.into_raw(),
        delai: Duration::ZERO,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une image `l` × `h` d'une seule couleur YUV, ses deux plans de couleur entrelacés (NV12)
    /// ou séparés (I420), dans des rangées plus longues qu'elle.
    fn unie(
        (l, h): (usize, usize),
        (y, u, v): (u8, u8, u8),
        entrelaces: bool,
    ) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let rangee = l + 8;
        let plan_y = vec![y; rangee * h];
        if entrelaces {
            let uv: Vec<u8> = (0..rangee * h / 2)
                .map(|i| if i % 2 == 0 { u } else { v })
                .collect();
            let v_seul = uv[1..].to_vec();
            (plan_y, uv, v_seul)
        } else {
            let c = vec![0; rangee / 2 * h / 2];
            (
                plan_y,
                c.iter().map(|_| u).collect(),
                c.iter().map(|_| v).collect(),
            )
        }
    }

    fn plans<'a>(
        (py, pu, pv): (&'a [u8], &'a [u8], &'a [u8]),
        (l, h): (u32, u32),
        entrelaces: bool,
        espace: i32,
    ) -> Plans<'a> {
        let rangee = l as usize + 8;
        let (rc, pas) = if entrelaces {
            (rangee, 2)
        } else {
            (rangee / 2, 1)
        };
        Plans {
            y: Plan {
                octets: py,
                rangee,
                pas: 1,
            },
            u: Plan {
                octets: pu,
                rangee: rc,
                pas,
            },
            v: Plan {
                octets: pv,
                rangee: rc,
                pas,
            },
            recadrage: [0, 0, l, h],
            espace,
            quarts: 0,
        }
    }

    fn proche(a: [u8; 4], b: [u8; 4]) -> bool {
        a.iter().zip(b).all(|(&x, y)| x.abs_diff(y) <= 2)
    }

    /// Les couleurs de référence des deux matrices, à l'étendue réduite : le rouge, le vert et le
    /// bleu purs de BT.601 et de BT.709 (leurs tables), le noir et le blanc.
    #[test]
    fn video_android_1_les_couleurs_de_reference() {
        let bt601 = Matrice::pour(BT601_625, (640, 480));
        assert!(proche(bt601.rgba(81, 90, 240), [255, 0, 0, 255]));
        assert!(proche(bt601.rgba(145, 54, 34), [0, 255, 0, 255]));
        assert!(proche(bt601.rgba(41, 240, 110), [0, 0, 255, 255]));
        let bt709 = Matrice::pour(BT709, (640, 480));
        assert!(proche(bt709.rgba(63, 102, 240), [255, 0, 0, 255]));
        assert!(proche(bt709.rgba(173, 42, 26), [0, 255, 0, 255]));
        assert!(proche(bt709.rgba(32, 240, 118), [0, 0, 255, 255]));
        for m in [bt601, bt709] {
            assert_eq!(m.rgba(16, 128, 128), [0, 0, 0, 255]);
            assert_eq!(m.rgba(235, 128, 128), [255, 255, 255, 255]);
        }
        let pleine = Matrice::pour(BT601_625 | ETENDUE_PLEINE, (640, 480));
        assert_eq!(pleine.rgba(0, 128, 128), [0, 0, 0, 255]);
        assert_eq!(pleine.rgba(255, 128, 128), [255, 255, 255, 255]);
        // Le gris du milieu : réduite, l'étendue l'éclaircirait (130) ; le noir et le blanc, eux,
        // se rejoignent aux bornes.
        assert_eq!(pleine.rgba(128, 128, 128), [128, 128, 128, 255]);
    }

    /// Sans espace dit, la taille décide : HD en BT.709, en dessous BT.601 — et un espace dit
    /// l'emporte sur la taille.
    #[test]
    fn video_android_1_sans_espace_la_taille_decide() {
        let bt709 = Matrice::de((0.2126, 0.0722), false);
        let bt601 = Matrice::de((0.299, 0.114), false);
        assert_eq!(Matrice::pour(0, (1920, 1080)), bt709);
        assert_eq!(Matrice::pour(0, (720, 1280)), bt709, "debout");
        assert_eq!(Matrice::pour(0, (640, 480)), bt601);
        assert_eq!(Matrice::pour(BT601_525, (1920, 1080)), bt601);
        assert_eq!(Matrice::pour(BT709, (640, 480)), bt709);
        assert_ne!(Matrice::pour(BT2020, (640, 480)), bt601);
    }

    /// Le rouge de BT.709, en NV12 comme en I420, donne la même image rouge.
    #[test]
    fn video_android_1_entrelaces_ou_separes_meme_image() {
        for entrelaces in [true, false] {
            let (y, u, v) = unie((6, 4), (63, 102, 240), entrelaces);
            let t = en_rgba(&plans((&y, &u, &v), (6, 4), entrelaces, BT709)).expect("lisible");
            assert_eq!((t.largeur, t.hauteur), (6, 4));
            for px in t.rgba.as_chunks::<4>().0 {
                assert!(proche(*px, [255, 0, 0, 255]), "{px:?}");
            }
        }
    }

    /// Un échantillon de couleur vaut pour deux pixels sur deux : une image dont la moitié gauche
    /// est rouge et la droite bleue (BT.601) se lit ainsi, et non colonne par colonne.
    #[test]
    fn video_android_1_la_couleur_se_lit_a_sa_demi_place() {
        let rangee = 4;
        let y: Vec<u8> = (0..rangee * 2)
            .map(|i| if i % rangee < 2 { 81 } else { 41 })
            .collect();
        let uv = [90, 240, 240, 110];
        let plan = |octets| Plan {
            octets,
            rangee,
            pas: 2,
        };
        let p = Plans {
            y: Plan {
                octets: &y,
                rangee,
                pas: 1,
            },
            u: plan(&uv),
            v: plan(&uv[1..]),
            recadrage: [0, 0, 4, 2],
            espace: BT601_625,
            quarts: 0,
        };
        let t = en_rgba(&p).expect("lisible");
        let px = t.rgba.as_chunks::<4>().0;
        for x in [0, 1, 4, 5] {
            assert!(proche(px[x], [255, 0, 0, 255]), "{x} : {:?}", px[x]);
        }
        for x in [2, 3, 6, 7] {
            assert!(proche(px[x], [0, 0, 255, 255]), "{x} : {:?}", px[x]);
        }
    }

    /// Le recadrage garde le rectangle dit, et lit la couleur à sa place : une image dont la
    /// moitié gauche est noire et la droite blanche, recadrée sur la droite, est blanche.
    #[test]
    fn video_android_1_le_recadrage_garde_son_rectangle() {
        let (mut y, u, v) = unie((8, 4), (16, 128, 128), true);
        let rangee = 16;
        for r in 0..4 {
            y[r * rangee + 4..r * rangee + 8].fill(235);
        }
        let mut p = plans((&y, &u, &v), (8, 4), true, BT709);
        p.recadrage = [4, 2, 4, 2];
        let t = en_rgba(&p).expect("lisible");
        assert_eq!((t.largeur, t.hauteur), (4, 2));
        assert!(t.rgba.as_chunks::<4>().0.iter().all(|px| *px == [255; 4]));
    }

    /// Un plan trop court pour ce qu'il dit, ou un recadrage vide : rien, jamais une image
    /// devinée ni une panique.
    #[test]
    fn video_android_1_un_plan_trop_court_ne_se_lit_pas() {
        let (y, u, v) = unie((6, 4), (63, 102, 240), false);
        let p = plans((&y[..y.len() - 9], &u, &v), (6, 4), false, 0);
        assert!(en_rgba(&p).is_none(), "Y court");
        let p = plans((&y, &u, &v[..v.len() - 8]), (6, 4), false, 0);
        assert!(en_rgba(&p).is_none(), "V court");
        let mut p = plans((&y, &u, &v), (6, 4), false, 0);
        p.recadrage = [0, 0, 0, 4];
        assert!(en_rgba(&p).is_none(), "vide");
        p.recadrage = [2, 0, 20, 4];
        assert!(en_rgba(&p).is_none(), "au-delà des rangées");
    }

    /// Un quart de tour : une vidéo filmée debout (6 × 4 dans le capteur) se regarde 4 × 6, et
    /// son coin haut-gauche part en haut à droite.
    #[test]
    fn video_android_1_le_quart_de_tour() {
        let (mut y, u, v) = unie((6, 4), (16, 128, 128), true);
        y[0] = 235; // le coin haut-gauche, blanc
        let mut p = plans((&y, &u, &v), (6, 4), true, BT709);
        p.quarts = 1;
        let t = en_rgba(&p).expect("lisible");
        assert_eq!((t.largeur, t.hauteur), (4, 6));
        let blanc = |x: usize, y: usize| t.rgba[(y * 4 + x) * 4] == 255;
        assert!(blanc(3, 0), "en haut à droite");
        assert!(!blanc(0, 0));
        p.quarts = 2;
        let t = en_rgba(&p).expect("lisible");
        assert_eq!((t.largeur, t.hauteur), (6, 4));
        assert_eq!(t.rgba[(3 * 6 + 5) * 4], 255, "en bas à droite");
    }
}
