//! **La porte des vidéos d'Android** (VIDEO-ANDROID-1, fiche 172) : le lecteur du système
//! (`MediaPlayer`) décode, cadence et joue le son ; ses images arrivent dans un `ImageReader`, et
//! Glucose les lit plan par plan. Sa réponse du 25/11 (`relais-q-videos`) : les lecteurs natifs
//! des appareils.
//!
//! Java ne fait que ce que seul Java peut faire : ouvrir le lecteur, tenir une image le temps qu'on
//! la lise, rendre la première image (`MediaMetadataRetriever`). Ce qui décide — les couleurs, le
//! recadrage, le quart de tour, ce que veut dire chaque nombre que Java rend — est en Rust, et
//! s'éprouve ici sans téléphone.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// **Où sont les octets d'une vidéo** : un fichier (entier, ou la tranche d'un document), ou la
/// mémoire.
pub enum Ou<'a> {
    Fichier {
        chemin: &'a Path,
        debut: u64,
        /// `None` : jusqu'au bout du fichier.
        longueur: Option<u64>,
    },
    Octets(&'a Arc<[u8]>),
}

/// **Un plan d'une image** (`Image.Plane`) : ses octets, le pas d'une rangée à la suivante, et
/// d'un échantillon au suivant (2 quand les deux plans de couleur s'entrelacent, NV12 ou NV21).
pub struct Plan<'a> {
    pub octets: &'a [u8],
    pub rangee: usize,
    pub pas: usize,
}

/// **Une image du lecteur**, en YUV 4:2:0 (`YUV_420_888`), telle qu'il la tient.
pub struct Plans<'a> {
    pub y: Plan<'a>,
    pub u: Plan<'a>,
    pub v: Plan<'a>,
    /// Le rectangle à garder (`Image.getCropRect`) : x, y, largeur, hauteur. Un décodeur range
    /// souvent l'image dans une toile plus grande (1088 rangées pour 1080).
    pub recadrage: [u32; 4],
    /// L'espace de couleurs que le système dit (`Image.getDataSpace`, Android 13) ; 0 s'il ne dit
    /// rien.
    pub espace: i32,
    /// Les quarts de tour, dans le sens des aiguilles d'une montre, que la vidéo demande (ses
    /// métadonnées) : un `ImageReader` rend l'image telle que le capteur l'a filmée.
    pub quarts: u8,
}

/// **Une image fixe**, en RGBA.
pub struct ImageFixe {
    pub largeur: u32,
    pub hauteur: u32,
    pub rgba: Vec<u8>,
}

/// **Ce que l'attente d'une image a donné.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attente {
    /// Une image, prêtée le temps de la lire.
    Image,
    /// Rien encore.
    Rien,
    /// La vidéo est allée au bout.
    AuBout,
    /// Le système n'a pas su la lire.
    Faute,
}

/// **Une vidéo ouverte par le système.** Elle se ferme quand on la lâche.
pub trait Lecteur: Send {
    /// Attend une image au plus `delai`, et la prête à `voir` le temps de la lire.
    fn attendre(&self, delai: Duration, voir: &mut dyn FnMut(&Plans<'_>)) -> Attente;
    /// Joue, ou s'arrête sur l'image où elle est.
    fn jouer(&self, oui: bool);
    /// Coupe ou rend le son.
    fn couper_le_son(&self, oui: bool);
    /// Repart du début, et joue ; `false` si le système refuse.
    fn revenir_au_debut(&self) -> bool;
}

/// **Ce que le système sait faire d'une vidéo.**
pub trait Porte: Send + Sync {
    /// Sa première image, tournée comme elle se regarde.
    fn image_fixe(&self, ou: &Ou<'_>) -> Option<ImageFixe>;
    /// L'ouvre, prête à jouer, le son coupé ou non.
    fn ouvrir(&self, ou: &Ou<'_>, avec_le_son: bool) -> Option<Box<dyn Lecteur>>;
}

#[cfg(target_os = "android")]
static PORTE: std::sync::Mutex<Option<Arc<dyn Porte>>> = std::sync::Mutex::new(None);

/// **Branche les vidéos du système**, avant le lancement. Chaque activité a la sienne : la
/// dernière née remplace la précédente.
pub fn installer(porte: Arc<dyn Porte>) {
    #[cfg(target_os = "android")]
    {
        *PORTE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(porte);
    }
    #[cfg(not(target_os = "android"))]
    drop(porte);
}

/// La porte branchée, s'il y en a une.
#[cfg(target_os = "android")]
pub(crate) fn porte() -> Option<Arc<dyn Porte>> {
    PORTE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// Ce que `LecteurVideo.tenue` rend en nombres, dans cet ordre.
pub const MESURES: usize = 12;

/// **Les plans d'une image que Java tient** : ses trois tampons (Y, U, V), et ses [`MESURES`] —
/// pour chaque plan, le pas d'une rangée puis d'un échantillon ; le recadrage (x, y, largeur,
/// hauteur) ; l'espace de couleurs ; les quarts de tour. `None` si un nombre ne se lit pas : une
/// image qu'on ne sait pas lire ne se devine pas.
pub fn plans<'a>(mesures: &[i32], [y, u, v]: [&'a [u8]; 3]) -> Option<Plans<'a>> {
    let &[ry, py, ru, pu, rv, pv, x, h0, l, h, espace, quarts] = mesures else {
        return None;
    };
    let n = |v: i32| usize::try_from(v).ok().filter(|&v| v > 0);
    let plan = |octets, rangee, pas| {
        Some(Plan {
            octets,
            rangee: n(rangee)?,
            pas: n(pas)?,
        })
    };
    let cote = |v: i32| u32::try_from(v).ok();
    Some(Plans {
        y: plan(y, ry, py)?,
        u: plan(u, ru, pu)?,
        v: plan(v, rv, pv)?,
        recadrage: [cote(x)?, cote(h0)?, cote(l)?, cote(h)?],
        espace,
        quarts: u8::try_from(quarts.rem_euclid(4)).ok()?,
    })
}

/// **L'image fixe que Java rend** : sa largeur, sa hauteur, puis chaque pixel en ARGB
/// (`Bitmap.getPixels`). `None` si les nombres ne s'accordent pas.
pub fn image_fixe_de_java(pixels: &[i32]) -> Option<ImageFixe> {
    let (&[l, h], argb) = pixels.split_first_chunk::<2>()?;
    let (largeur, hauteur) = (u32::try_from(l).ok()?, u32::try_from(h).ok()?);
    if largeur == 0 || hauteur == 0 || argb.len() != (largeur as usize) * (hauteur as usize) {
        return None;
    }
    let rgba = argb
        .iter()
        .flat_map(|&p| {
            let [a, r, g, b] = p.to_be_bytes();
            [r, g, b, a]
        })
        .collect();
    Some(ImageFixe {
        largeur,
        hauteur,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_android_1_les_mesures_de_java_se_lisent_dans_leur_ordre() {
        let (y, u, v) = ([1u8; 4], [2u8; 2], [3u8; 2]);
        let m = [8, 1, 4, 2, 4, 2, 0, 2, 6, 4, 0x1_0000, 5];
        let p = plans(&m, [&y, &u, &v]).expect("lisibles");
        assert_eq!((p.y.rangee, p.y.pas, p.y.octets[0]), (8, 1, 1));
        assert_eq!((p.u.rangee, p.u.pas, p.u.octets[0]), (4, 2, 2));
        assert_eq!((p.v.rangee, p.v.pas, p.v.octets[0]), (4, 2, 3));
        assert_eq!(p.recadrage, [0, 2, 6, 4]);
        assert_eq!((p.espace, p.quarts), (0x1_0000, 1), "cinq quarts font un");
    }

    #[test]
    fn video_android_1_des_mesures_fausses_ne_se_devinent_pas() {
        let (y, u, v) = ([0u8; 4], [0u8; 2], [0u8; 2]);
        let bonnes = [8, 1, 4, 2, 4, 2, 0, 0, 6, 4, 0, 0];
        assert!(plans(&bonnes, [&y, &u, &v]).is_some());
        assert!(plans(&bonnes[..11], [&y, &u, &v]).is_none(), "trop court");
        for i in [0, 1, 2, 3, 4, 5] {
            let mut m = bonnes;
            m[i] = 0;
            assert!(plans(&m, [&y, &u, &v]).is_none(), "pas nul en {i}");
        }
        let mut m = bonnes;
        m[8] = -6;
        assert!(plans(&m, [&y, &u, &v]).is_none(), "largeur négative");
    }

    #[test]
    fn video_android_1_l_image_fixe_de_java_passe_d_argb_a_rgba() {
        let rouge_opaque = i32::from_be_bytes([0xff, 0xff, 0x00, 0x00]);
        let bleu_mi = i32::from_be_bytes([0x80, 0x00, 0x00, 0xff]);
        let image = image_fixe_de_java(&[2, 1, rouge_opaque, bleu_mi]).expect("lisible");
        assert_eq!((image.largeur, image.hauteur), (2, 1));
        assert_eq!(image.rgba, [0xff, 0, 0, 0xff, 0, 0, 0xff, 0x80]);
        assert!(
            image_fixe_de_java(&[2, 2, 0, 0, 0]).is_none(),
            "un pixel manque"
        );
        assert!(image_fixe_de_java(&[0, 0]).is_none(), "vide");
        assert!(image_fixe_de_java(&[1]).is_none(), "sans hauteur");
    }
}
