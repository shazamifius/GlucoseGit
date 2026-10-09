//! COPIER-1 : l'objet de la sélection, interrogé comme Windows et les autres applications
//! l'interrogent — sans presse-papiers, sans écran : le vrai presse-papiers est celui de
//! l'utilisateur, qu'une épreuve ne touche jamais.

use super::*;
use crate::interactions::clipboard::{Fichier, ImagePosee};
use std::sync::Arc;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::ReleaseStgMedium;

/// Une date neuve à chaque appel : chaque épreuve a son dossier, même en parallèle.
fn date() -> i64 {
    static N: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
    let base = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as i64);
    base + N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn fichier(nom: &str, octet: u8) -> Fichier {
    Fichier {
        nom: nom.into(),
        octets: Arc::new(vec![octet; 64]),
    }
}

fn formes(fichiers: Vec<Fichier>, image: bool) -> Formes {
    Formes {
        lot: (0..3000u32).map(|i| (i * 7) as u8).collect(),
        texte: Some("une note".into()),
        image: image.then(|| ImagePosee {
            png: b"\x89PNG faux".to_vec(),
            largeur: 1,
            hauteur: 1,
            rgba: vec![1, 2, 3, 255],
        }),
        fichiers,
        date: date(),
    }
}

fn objet(f: Formes, pour: Pour) -> IDataObject {
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    Selection::nouvelle(f, pour).expect("l'objet").into()
}

fn demande(format: u16) -> FORMATETC {
    FORMATETC {
        cfFormat: format,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    }
}

fn offre(o: &IDataObject, format: u16) -> bool {
    let reponse = unsafe { o.QueryGetData(&demande(format)) };
    reponse == S_OK
}

/// Les octets qu'une application recevrait pour ce format.
fn lire(o: &IDataObject, format: u16) -> Vec<u8> {
    let mut medium = unsafe { o.GetData(&demande(format)) }.expect("la forme");
    let octets = unsafe {
        let bloc = medium.u.hGlobal;
        let taille = GlobalSize(bloc);
        let ou = GlobalLock(bloc).cast::<u8>();
        let v = std::slice::from_raw_parts(ou, taille).to_vec();
        let _ = GlobalUnlock(bloc);
        v
    };
    unsafe { ReleaseStgMedium(&mut medium) };
    octets
}

/// Les chemins d'un `CF_HDROP`, relus comme une cible les relit.
fn chemins(hdrop: &[u8]) -> Vec<PathBuf> {
    let unites: Vec<u16> = hdrop[20..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect();
    unites
        .split(|u| *u == 0)
        .filter(|s| !s.is_empty())
        .map(|s| PathBuf::from(String::from_utf16_lossy(s)))
        .collect()
}

fn png() -> u16 {
    format_nomme("PNG").expect("PNG")
}

/// **Une image seule part en image au presse-papiers** — PNG et pixels —, sans fichier : un
/// fichier de plus ferait une seconde pièce jointe dans Discord. Le lot et le texte sont là.
#[test]
fn test_une_image_seule_part_en_image() {
    let o = objet(formes(vec![fichier("a.png", 1)], true), Pour::PressePapiers);
    assert!(offre(&o, png()) && offre(&o, CF_DIBV5.0));
    assert!(!offre(&o, CF_HDROP.0), "pas de fichier");
    assert!(offre(&o, CF_UNICODETEXT.0));
    assert_eq!(lire(&o, png()), b"\x89PNG faux");
}

/// **Plusieurs images partent en fichiers, écrits au premier collage qui les veut, une fois** :
/// rien sur le disque avant ; puis les originaux, octet pour octet, sous leurs noms.
#[test]
fn test_plusieurs_images_partent_en_fichiers_ecrits_a_la_demande() {
    let f = formes(vec![fichier("a.png", 1), fichier("a.png", 2)], false);
    let dossier = dossier_des_copies().join(f.date.to_string());
    let o = objet(f, Pour::PressePapiers);
    assert!(
        offre(&o, CF_HDROP.0) && !offre(&o, png()),
        "des fichiers, pas d'image"
    );
    assert!(!dossier.exists(), "rien d'écrit avant qu'on le demande");
    let premiers = chemins(&lire(&o, CF_HDROP.0));
    let noms: Vec<_> = premiers
        .iter()
        .map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string()
        })
        .collect();
    assert_eq!(noms, ["a.png", "a 2.png"], "deux noms libres");
    assert_eq!(std::fs::read(&premiers[1]).expect("écrit"), vec![2u8; 64]);
    // Une application a modifié le fichier : un second collage ne le réécrit pas.
    std::fs::write(&premiers[1], b"retouche").expect("modifié");
    assert_eq!(
        chemins(&lire(&o, CF_HDROP.0)),
        premiers,
        "les mêmes chemins"
    );
    assert_eq!(
        std::fs::read(&premiers[1]).expect("lu"),
        b"retouche",
        "une seule écriture"
    );
    let _ = std::fs::remove_dir_all(dossier);
}

/// **Le glisser emporte des fichiers dès une image**, et la cible de Glucose y lit son lot, octet
/// pour octet — avant les fichiers.
#[test]
fn test_le_glisser_emporte_des_fichiers_et_le_lot() {
    let f = formes(vec![fichier("a.png", 1)], true);
    let lot = f.lot.clone();
    let o = objet(f, Pour::Glisser);
    assert!(offre(&o, CF_HDROP.0), "un fichier");
    assert!(!offre(&o, png()), "pas d'image en glisser");
    assert_eq!(super::super::depot_windows::lot_porte(&o), Some(lot));
}

/// **Un glisser n'efface pas ce qu'un collage lira encore** ; une nouvelle copie, si : le
/// presse-papiers ne portait plus que la nouvelle.
#[test]
fn test_le_dossier_d_une_copie_vit_jusqu_a_la_suivante() {
    let copie = objet(
        formes(vec![fichier("a.png", 1)], false),
        Pour::PressePapiers,
    );
    let ecrits = chemins(&lire(&copie, CF_HDROP.0));
    let _glisser = objet(formes(vec![fichier("b.png", 1)], false), Pour::Glisser);
    assert!(ecrits[0].exists(), "le glisser ne l'efface pas");
    let _suivante = objet(
        formes(vec![fichier("c.png", 1)], false),
        Pour::PressePapiers,
    );
    assert!(!ecrits[0].exists(), "la copie suivante l'efface");
}

/// **Un `CF_HDROP` comme l'explorateur l'écrit** : vingt octets d'en-tête, des chemins larges,
/// et deux zéros pour finir.
#[test]
fn test_la_liste_de_fichiers_a_la_forme_de_l_explorateur() {
    let o = liste_de_fichiers(&[PathBuf::from("C:\\a"), PathBuf::from("C:\\b")]);
    assert_eq!(u32::from_le_bytes(o[0..4].try_into().unwrap()), 20);
    assert_eq!(
        u32::from_le_bytes(o[16..20].try_into().unwrap()),
        1,
        "large"
    );
    assert_eq!(
        &o[o.len() - 4..],
        &[0, 0, 0, 0],
        "fin de chemin, fin de liste"
    );
    assert_eq!(
        chemins(&o),
        [PathBuf::from("C:\\a"), PathBuf::from("C:\\b")]
    );
}

/// **Un nom libre ne regarde pas les majuscules** : Windows non plus.
#[test]
fn test_un_nom_libre_ignore_les_majuscules() {
    let mut pris = HashSet::new();
    assert_eq!(nom_libre("Photo.PNG", &mut pris), "Photo.PNG");
    assert_eq!(nom_libre("photo.png", &mut pris), "photo 2.png");
    assert_eq!(nom_libre("sans", &mut pris), "sans");
    assert_eq!(nom_libre("sans", &mut pris), "sans 2");
}
