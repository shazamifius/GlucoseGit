//! **L'affiche d'une vidéo gardée à côté** (fiche 195 ; sa réponse à `relais-q-video-a-cote` :
//! « effectivement je valide »). Le fichier reste sur son disque ; sa première image, réduite à
//! [`COTE`] pixels au plus et encodée en PNG — quelques dizaines de Ko —, entre au document comme
//! une photo. Elle voyage par la salle, et se montre là où le fichier ne se lit pas : un autre
//! appareil, ou le sien quand le fichier a bougé.

use crate::persist::objets::Parole;

/// Le plus grand côté d'une affiche : assez pour un nœud à l'écran, assez peu pour voyager.
pub(crate) const COTE: u32 = 640;

/// **L'affiche de cette vidéo**, en PNG : sa première image, réduite. `None` si ce système ne
/// sait pas la lire.
pub(crate) fn affiche(src: &str) -> Option<Vec<u8>> {
    let t = super::image_fixe(src, None)?;
    let image = image::RgbaImage::from_raw(t.largeur, t.hauteur, t.rgba)?;
    let (l, h) = reduite((t.largeur, t.hauteur));
    let petite = match (l, h) == (t.largeur, t.hauteur) {
        true => image,
        false => image::imageops::thumbnail(&image, l, h),
    };
    crate::renderer::atelier::encoder_en_png(petite.as_raw(), (l, h))
}

/// Les côtés d'une affiche : ceux de l'image, ramenés à [`COTE`] au plus, la forme gardée.
fn reduite((l, h): (u32, u32)) -> (u32, u32) {
    let grand = l.max(h).max(1);
    if grand <= COTE {
        return (l, h);
    }
    let r = |c: u32| ((u64::from(c) * u64::from(COTE) / u64::from(grand)) as u32).max(1);
    (r(l), r(h))
}

/// **Fait l'affiche sur un fil à soi**, et tient la promesse : le scribe et le facteur l'attendent.
/// Une vidéo illisible la laisse non tenue — rien ne s'écrit, rien ne part.
pub(crate) fn promettre(src: String, parole: Parole) {
    let fil = std::thread::Builder::new().name("affiche".into());
    let parti = fil.spawn(move || {
        if let Some(png) = affiche(&src) {
            parole.tenir(png);
        }
    });
    if let Err(e) = parti {
        crate::dire_erreur!("[video] l'affiche ne se fait pas ({e})");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affiche_1_une_affiche_garde_sa_forme_sous_son_cote() {
        assert_eq!(reduite((64, 48)), (64, 48));
        assert_eq!(reduite((1920, 1080)), (640, 360));
        assert_eq!(reduite((1080, 1920)), (360, 640));
        assert_eq!(reduite((4000, 1)), (640, 1));
    }

    /// La vraie vidéo des épreuves : sa première image, rouge, devient un PNG rouge.
    #[test]
    fn affiche_1_l_affiche_est_la_premiere_image_en_png() {
        if !super::super::lisible_ici() {
            return;
        }
        let dossier = std::env::temp_dir().join(format!("glucose-affiche-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).expect("dossier");
        let chemin = dossier.join("rouge.mp4");
        std::fs::write(
            &chemin,
            include_bytes!("../../../tests/donnees/rouge-64x48.mp4"),
        )
        .expect("écrite");
        let png = affiche(chemin.to_str().expect("chemin")).expect("une affiche");
        let image = image::load_from_memory(&png).expect("un PNG").to_rgba8();
        assert_eq!(image.dimensions(), (64, 48));
        let p = image.get_pixel(32, 24).0;
        assert!(p[0] > 200 && p[1] < 60 && p[2] < 60, "rouge : {p:?}");
        let _ = std::fs::remove_dir_all(&dossier);
    }
}
