//! La taille d'un JPEG lue dans son en-tête : la même que celle du décodeur entier, sans lire ses
//! données.

use super::*;
use std::io::Cursor;

/// Un lecteur qui compte ce qu'on lui prend vraiment.
struct Compte {
    dedans: Cursor<Vec<u8>>,
    lus: u64,
}

impl Read for Compte {
    fn read(&mut self, tampon: &mut [u8]) -> std::io::Result<usize> {
        let n = self.dedans.read(tampon)?;
        self.lus += n as u64;
        Ok(n)
    }
}

impl Seek for Compte {
    fn seek(&mut self, ou: SeekFrom) -> std::io::Result<u64> {
        self.dedans.seek(ou)
    }
}

fn segment(marqueur: u8, charge: &[u8]) -> Vec<u8> {
    let longueur = u16::try_from(charge.len() + 2).expect("un segment tient en 64 Ko");
    let mut s = vec![0xFF, marqueur];
    s.extend_from_slice(&longueur.to_be_bytes());
    s.extend_from_slice(charge);
    s
}

/// Le début d'un SOF : la précision, la hauteur, la largeur, un composant.
fn sof(hauteur: u16, largeur: u16) -> Vec<u8> {
    let mut c = vec![8];
    c.extend_from_slice(&hauteur.to_be_bytes());
    c.extend_from_slice(&largeur.to_be_bytes());
    c.extend_from_slice(&[1, 1, 0x11, 0]);
    c
}

/// Un JPEG d'appareil, en squelette : EXIF de 60 Ko, des octets de remplissage, un profil, une table
/// de Huffman qui partage la plage des SOF, puis le SOF progressif — et deux mégaoctets de données.
fn squelette() -> Vec<u8> {
    let mut f = vec![0xFF, 0xD8];
    f.extend(segment(0xE1, &vec![7u8; 60_000]));
    f.extend([0xFF, 0xFF]); // remplissage
    f.extend(segment(0xE2, &[1, 2, 3]));
    f.extend(segment(0xC4, &sof(11, 22))); // DHT : pas une taille
    f.extend(segment(0xC2, &sof(3000, 4000)));
    f.extend(segment(0xDA, &[0; 8]));
    f.extend(vec![0x55u8; 2_000_000]);
    f.extend([0xFF, 0xD9]);
    f
}

#[test]
fn test_la_taille_d_un_jpeg_se_lit_sans_lire_ses_donnees() {
    let fichier = squelette();
    let taille = fichier.len() as u64;
    let mut r = BufReader::new(Compte {
        dedans: Cursor::new(fichier),
        lus: 0,
    });
    assert_eq!(dimensions_jpeg(&mut r), Some((4000, 3000)));
    let lus = r.into_inner().lus;
    assert!(
        lus < 64 * 1024,
        "{lus} octets lus sur {taille} : l'EXIF et les données se sautent"
    );
}

#[test]
fn test_un_flux_sans_taille_n_en_invente_pas() {
    let lire = |octets: Vec<u8>| dimensions_jpeg(&mut Cursor::new(octets));
    assert_eq!(lire(b"\x89PNG\r\n\x1a\n".to_vec()), None, "pas un JPEG");
    let mut donnees_avant = vec![0xFF, 0xD8];
    donnees_avant.extend(segment(0xDA, &[0; 8]));
    donnees_avant.extend(segment(0xC0, &sof(10, 10)));
    assert_eq!(lire(donnees_avant), None, "les données avant la taille");
    let mut tronque = vec![0xFF, 0xD8];
    tronque.extend(segment(0xE1, &[0; 100]));
    tronque.truncate(50);
    assert_eq!(lire(tronque), None, "coupé au milieu d'un segment");
    let mut differee = vec![0xFF, 0xD8];
    differee.extend(segment(0xC0, &sof(0, 10)));
    assert_eq!(lire(differee), None, "la hauteur différée au marqueur DNL");
}

#[test]
fn test_un_vrai_jpeg_a_la_taille_que_dit_le_decodeur_entier() {
    let mut octets = Vec::new();
    let pixels = vec![90u8; 37 * 23 * 3];
    image::codecs::jpeg::JpegEncoder::new(&mut octets)
        .encode(&pixels, 37, 23, image::ExtendedColorType::Rgb8)
        .expect("encodé");
    let entier = ImageReader::with_format(Cursor::new(&octets), ImageFormat::Jpeg)
        .into_dimensions()
        .expect("le décodeur entier");
    let dir = std::env::temp_dir().join(format!("glucose-dimensions-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dossier");
    // Nommé .png : c'est la signature qui dit le format.
    let chemin = dir.join("photo.png");
    std::fs::write(&chemin, &octets).expect("écrit");
    let lu = dimensions_du_fichier(&chemin).expect("lu");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(lu, (37, 23));
    assert_eq!(lu, entier);
}

#[test]
fn test_une_photo_du_disque_se_mesure_par_son_en_tete() {
    let dir = std::env::temp_dir().join(format!("glucose-squelette-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dossier");
    let chemin = dir.join("DSC_0001.JPG");
    std::fs::write(&chemin, squelette()).expect("écrit");
    let lu = dimensions_du_fichier(&chemin);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        lu.ok(),
        Some((4000, 3000)),
        "la marche, pas le décodeur entier"
    );
}
