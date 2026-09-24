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
    let avant = chemin(&d, &src).expect("un chemin");
    assert_eq!(
        chemin(&d, &src),
        Some(avant.clone()),
        "la meme source, le meme nom"
    );
    std::fs::write(&source, [1u8; 11]).expect("reecrit");
    assert_ne!(
        chemin(&d, &src),
        Some(avant),
        "une autre taille, un autre nom"
    );
    assert!(
        chemin(&d, "nulle/part.png").is_none(),
        "une source absente n'a pas d'apercu"
    );
}
