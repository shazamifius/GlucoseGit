//! Ce que les aperçus promettent : revenir au bit près, se taire quand ils ne se tiennent pas,
//! et ne jamais se relire pour une source qui a changé.

use super::*;
use crate::renderer::photo::{Etat, Pyramide};

/// Un dossier à cette épreuve, vidé : les aperçus d'une autre épreuve n'y sont pas.
fn dossier(nom: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("glucose-apercus-{nom}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// Une image de 300 × 200 aux pixels tous différents.
fn pyramide() -> Pyramide {
    let mut p = tiny_skia::Pixmap::new(300, 200).expect("une image");
    for (i, px) in p.data_mut().as_chunks_mut::<4>().0.iter_mut().enumerate() {
        *px = [
            (i % 251) as u8,
            (i / 300 % 241) as u8,
            (i % 7 * 30) as u8,
            255,
        ];
    }
    Pyramide::nouvelle(p)
}

/// **Un aperçu revient au bit près**, et la pyramide qui en naît tient ses petits niveaux et
/// dit perdus les grands.
#[test]
fn test_un_apercu_revient_au_bit_pres() {
    let p = pyramide();
    let a = p
        .apercu(2)
        .expect("tous les niveaux d'une pyramide neuve sont tenus");
    assert_eq!(a.niveaux.len(), p.niveaux_construits() - 2);
    let chemin = dossier("aller-retour").join("a.apercu");
    ecrire(&chemin, &a).expect("l'ecriture");
    let relu = lire(&chemin).expect("la relecture");
    assert!(relu == a, "l'apercu relu differe de celui ecrit");

    let partielle = Pyramide::depuis_apercu(relu).expect("une pyramide");
    assert_eq!(partielle.niveaux_construits(), p.niveaux_construits());
    assert_eq!(partielle.dimensions_natives(), (300, 200));
    for rang in 0..p.niveaux_construits() {
        let attendu = if rang < 2 { Etat::Perdu } else { Etat::Tenu };
        assert_eq!(partielle.etat(rang), attendu, "le niveau de rang {rang}");
    }
    let ici = partielle.niveau(4).expect("tenu").data().to_vec();
    assert!(
        ici == p.niveau(4).expect("tenu").data(),
        "les octets du niveau 4 different"
    );
    assert!(
        !partielle.entiere(),
        "une pyramide nee d'un apercu n'est pas entiere"
    );
}

/// **Un aperçu qui ne se tient pas se tait** : tronqué, étranger, ou d'une autre version.
#[test]
fn test_un_apercu_qui_ne_se_tient_pas_se_tait() {
    let a = pyramide().apercu(3).expect("tenus");
    let d = dossier("abime");
    let chemin = d.join("a.apercu");
    ecrire(&chemin, &a).expect("l'ecriture");
    let octets = std::fs::read(&chemin).expect("relu");

    let tronque = d.join("tronque.apercu");
    std::fs::write(&tronque, &octets[..octets.len() - 1]).expect("ecrit");
    assert!(lire(&tronque).is_none(), "un octet de moins : rien");

    let long = d.join("long.apercu");
    std::fs::write(&long, [octets.as_slice(), &[0]].concat()).expect("ecrit");
    assert!(lire(&long).is_none(), "un octet de trop : rien");

    let etranger = d.join("etranger.apercu");
    std::fs::write(&etranger, b"PAS UN APERCU DU TOUT").expect("ecrit");
    assert!(lire(&etranger).is_none(), "un autre fichier : rien");
}

/// **Une source modifiée a un autre aperçu** : son nom porte sa taille et sa date, et l'ancien
/// ne se relit jamais.
#[test]
fn test_une_source_modifiee_change_d_apercu() {
    let d = dossier("source");
    std::fs::create_dir_all(&d).expect("le dossier");
    let source = d.join("photo.png");
    std::fs::write(&source, [1u8; 10]).expect("ecrit");
    let src = source.to_string_lossy().to_string();
    let avant = chemin(&d, &src, None).expect("un chemin");
    assert_eq!(
        chemin(&d, &src, None),
        Some(avant.clone()),
        "la meme source, le meme nom"
    );
    std::fs::write(&source, [1u8; 11]).expect("reecrit");
    assert_ne!(
        chemin(&d, &src, None),
        Some(avant),
        "une autre taille, un autre nom"
    );
    assert!(
        chemin(&d, "nulle/part.png", None).is_none(),
        "une source absente n'a pas d'apercu"
    );
}

/// **Une image scellée a l'aperçu de ses octets** (APERCU-5) : le même pour toutes les clés
/// qui les montrent, que leur fichier existe ou non — une image collée n'en a aucun.
#[test]
fn test_une_image_scellee_a_l_apercu_de_ses_octets() {
    let d = dossier("empreinte");
    let e = glucose_core::hash::sha256(b"les octets de l'image");
    let a = chemin(&d, "collee:123", Some(e)).expect("un nom sans fichier");
    let b = chemin(&d, "C:/nulle/part.png", Some(e)).expect("le même");
    assert_eq!(a, b);
    assert!(a.to_string_lossy().ends_with(".apercu"));
}

/// **Un aperçu abîmé se tait** : un seul octet changé dans ses pixels, et son sceau le trahit —
/// il ne montre jamais de faux pixels.
#[test]
fn test_un_apercu_abime_se_tait() {
    let a = pyramide().apercu(3).expect("tenus");
    let d = dossier("sceau");
    let chemin = d.join("a.apercu");
    ecrire(&chemin, &a).expect("l'ecriture");
    let mut octets = std::fs::read(&chemin).expect("relu");
    let milieu = octets.len() / 2;
    octets[milieu] ^= 0x10;
    std::fs::write(&chemin, &octets).expect("abîmé");
    assert!(lire(&chemin).is_none());
}

/// **Les aperçus d'avant se retirent**, et seulement eux : ceux de l'ancienne version, posés
/// à la racine, et le dossier d'une autre version ; rien d'autre n'est touché.
#[test]
fn test_les_apercus_d_avant_se_retirent() {
    let d = dossier("versions");
    std::fs::create_dir_all(d.join("v1")).expect("un ancien dossier");
    std::fs::write(d.join("ancien.apercu"), b"v1").expect("un ancien");
    std::fs::write(d.join("v1").join("x.apercu"), b"v1").expect("un ancien");
    std::fs::write(d.join("note.txt"), b"autre chose").expect("autre chose");
    let ici = dossier_de_cette_version(&d);
    assert_eq!(ici, d.join(format!("v{VERSION}")));
    assert!(!d.join("ancien.apercu").exists());
    assert!(!d.join("v1").exists());
    assert!(d.join("note.txt").exists(), "rien d'autre n'est touché");
}
