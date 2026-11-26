//! Une vidéo partagée se copie du descripteur au disque, sans passer par la mémoire
//! (VIDEO-ANDROID-2, fiche 172 § 5).

use super::*;
use std::io::Cursor;

/// Le début d'une vidéo MP4 — sa boîte `ftyp`, une marque de vidéo —, puis `corps` octets.
fn video(corps: usize) -> Vec<u8> {
    let mut octets = [&[0, 0, 0, 24][..], b"ftypisom", &[0, 0, 2, 0]].concat();
    octets.extend((0..corps).map(|i| (i % 251) as u8));
    octets
}

/// Un dossier à soi, vide, qu'on retire en sortant.
fn dossier_d_epreuve(nom: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("glucose-partage-{nom}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// Combien de fichiers ce dossier contient.
fn fichiers(d: &Path) -> usize {
    std::fs::read_dir(d).map_or(0, Iterator::count)
}

/// **Une vidéo se copie sur le disque, octet pour octet**, à sa place dans le descripteur — un
/// fournisseur peut n'en donner qu'un morceau.
#[test]
fn video_android_2_une_video_se_copie_sur_le_disque() {
    let d = dossier_d_epreuve("copie");
    let mut fichier = b"ENTETE".to_vec();
    fichier.extend(video(70_000));
    fichier.extend(b"QUEUE");
    let longueur = Some(video(70_000).len() as u64);
    let lu = lire(Cursor::new(fichier), 6, longueur, Some(&d));
    let Some(Lu::Fichier(chemin)) = lu else {
        panic!("une vidéo se copie : {lu:?}");
    };
    assert_eq!(chemin.parent(), Some(d.as_path()));
    let extension = chemin.extension().and_then(|e| e.to_str());
    assert_eq!(
        extension,
        Some("mp4"),
        "son nom dit ce qu'elle est (RECUE-1)"
    );
    assert_eq!(std::fs::read(&chemin).expect("copiée"), video(70_000));
    let _ = std::fs::remove_dir_all(&d);
}

/// **Une source** qui donne `tete`, puis `zeros` octets nuls, puis se tait — ou échoue.
struct Source {
    tete: Vec<u8>,
    zeros: u64,
    echoue: bool,
}

impl Read for Source {
    fn read(&mut self, tampon: &mut [u8]) -> std::io::Result<usize> {
        if !self.tete.is_empty() {
            let n = tampon.len().min(self.tete.len());
            tampon[..n].copy_from_slice(&self.tete[..n]);
            self.tete.drain(..n);
            return Ok(n);
        }
        if self.zeros == 0 && self.echoue {
            return Err(std::io::Error::other("le fournisseur s'est arrêté"));
        }
        let n = tampon
            .len()
            .min(usize::try_from(self.zeros).unwrap_or(usize::MAX));
        tampon[..n].fill(0);
        self.zeros -= n as u64;
        Ok(n)
    }
}

impl Seek for Source {
    fn seek(&mut self, _: SeekFrom) -> std::io::Result<u64> {
        Ok(0)
    }
}

/// **Une vidéo plus lourde que la borne d'un fichier lu se copie entière** : c'est tout l'objet
/// — une minute filmée en 4K dépasse 256 Mo.
#[test]
fn video_android_2_une_video_ne_connait_pas_la_borne() {
    let d = dossier_d_epreuve("lourde");
    let lourde = Source {
        tete: video(0),
        zeros: OCTETS_MAX as u64,
        echoue: false,
    };
    let lu = lire(lourde, 0, None, Some(&d));
    let Some(Lu::Fichier(chemin)) = lu else {
        panic!("une vidéo lourde se copie : {lu:?}");
    };
    let taille = std::fs::metadata(&chemin).expect("copiée").len();
    assert_eq!(taille, video(0).len() as u64 + OCTETS_MAX as u64);
    let _ = std::fs::remove_dir_all(&d);
}

/// **Une image reste en mémoire**, un dossier donné ou non : le document la scelle sans
/// laisser de fichier derrière elle.
#[test]
fn video_android_2_une_image_reste_en_memoire() {
    let d = dossier_d_epreuve("image");
    let lu = lire(Cursor::new(png()), 0, None, Some(&d));
    assert!(matches!(lu, Some(Lu::Octets(_))), "{lu:?}");
    assert_eq!(fichiers(&d), 0);
}

/// **Sans dossier, une vidéo se lit comme avant** : en mémoire, jusqu'à la borne.
#[test]
fn video_android_2_sans_dossier_une_video_se_lit() {
    let lu = lire(Cursor::new(video(100)), 0, None, None);
    assert!(
        matches!(lu, Some(Lu::Octets(ref r)) if r.octets == video(100)),
        "{lu:?}"
    );
}

/// **Une copie interrompue est illisible, et ne laisse rien** : ni fichier vide, ni moitié de
/// vidéo qui passerait pour entière.
#[test]
fn video_android_2_une_copie_interrompue_ne_laisse_rien() {
    let d = dossier_d_epreuve("coupee");
    let coupee = Source {
        tete: video(5_000),
        zeros: 0,
        echoue: true,
    };
    assert_eq!(lire(coupee, 0, None, Some(&d)), None);
    assert_eq!(fichiers(&d), 0, "rien ne reste du fichier réservé");
    let _ = std::fs::remove_dir_all(&d);
}

/// **Un partage compte chaque fichier** : une image lue, une vidéo copiée, un fichier que le
/// système n'a pas ouvert — et la vidéo se pose comme un fichier déposé.
#[test]
fn video_android_2_un_partage_pose_ses_videos() {
    let d = dossier_d_epreuve("partage");
    let mut partage = Partage::default();
    partage.ajouter_dans(Some((Cursor::new(png()), 0, None)), Some(&d));
    partage.ajouter_dans(Some((Cursor::new(video(10)), 0, None)), Some(&d));
    partage.ajouter_dans(None::<(Cursor<Vec<u8>>, u64, Option<u64>)>, Some(&d));
    assert_eq!(
        (
            partage.recus.len(),
            partage.chemins.len(),
            partage.illisibles
        ),
        (1, 1, 1)
    );
    let seule = Partage {
        chemins: partage.chemins.clone(),
        ..Partage::default()
    };
    let Route::Poser(moisson) = router(seule) else {
        panic!("une vidéo se pose");
    };
    assert_eq!(moisson.chemins, partage.chemins);
    assert_eq!(
        moisson.echec, None,
        "une vidéo seule n'est pas un partage vide"
    );
    let _ = std::fs::remove_dir_all(&d);
}
