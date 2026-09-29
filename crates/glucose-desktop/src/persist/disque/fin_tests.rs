//! **Rien ne se recouvre sans avoir été mis de côté** (FIN-1).
//!
//! Une entrée abîmée au milieu de l'histoire rend « ignoré » tout ce qui la suit ; l'écriture
//! le recouvrait. Ces épreuves passent par la vraie application : enregistrer, écrire,
//! abîmer le fichier sur le disque, le rouvrir.

use super::tests::{application, dossier, image_suivante, noter, rouvrir};
use glucose_core::persist::histoire;
use std::path::{Path, PathBuf};

/// Un document de trois gestes, fermé ; un octet change au milieu du deuxième. Rend le
/// chemin, les octets abîmés, et l'endroit où la chaîne se rompt.
fn document_abime(d: &Path) -> (PathBuf, Vec<u8>, u64) {
    let chemin = d.join("abime.glucose");
    let mut app = application(d);
    app.save_to(chemin.clone());
    for i in 0..3 {
        noter(&mut app, &format!("n{i}"), &format!("note {i}"));
        image_suivante(&mut app);
    }
    assert!(app.fermer_le_document());
    drop(app);
    let o = histoire::ouvrir(&mut std::fs::File::open(&chemin).unwrap()).unwrap();
    assert_eq!(o.gestes.len(), 3);
    let t = o.gestes[1].tranche;
    let mut octets = std::fs::read(&chemin).unwrap();
    octets[usize::try_from(t.offset + t.longueur / 2).unwrap()] ^= 0x5a;
    std::fs::write(&chemin, &octets).unwrap();
    // La chaîne se rompt à l'en-tête de ce geste : son contenu ne suit plus sa somme.
    (chemin, octets, t.offset - histoire::ENTETE as u64)
}

/// **Tout ce qui suit l'entrée abîmée est mis de côté**, puis recouvert par la suite de
/// l'écriture — et le fichier d'avant se reconstitue exactement : ce qui reste, puis ce qui a
/// été mis de côté.
#[test]
fn test_une_entree_abimee_au_milieu_ne_detruit_rien_de_ce_qui_la_suit() {
    let d = dossier("fin-abimee");
    let (chemin, octets, coupure) = document_abime(&d);

    let mut autre = rouvrir(&d, &chemin);
    let toast = autre.ui.current_toast.as_ref().expect("l'ouverture parle");
    assert!(toast.message.contains("mise de côté"), "{}", toast.message);
    noter(&mut autre, "apres", "écrit par-dessus la coupure");
    image_suivante(&mut autre);
    assert!(autre.fermer_le_document());
    drop(autre);

    let cote: Vec<PathBuf> = std::fs::read_dir(d.join("brouillons").join("recuperation"))
        .expect("le dossier de ce qui a été mis de côté")
        .flatten()
        .map(|e| e.path())
        .collect();
    assert_eq!(cote.len(), 1, "une fin mise de côté : {cote:?}");
    let reste = std::fs::read(&chemin).unwrap();
    let mut refait = reste[..usize::try_from(coupure).unwrap()].to_vec();
    refait.extend(std::fs::read(&cote[0]).unwrap());
    assert!(
        refait == octets,
        "ce qui reste, puis ce qui a été mis de côté : le fichier d'avant, octet pour octet"
    );
}

/// **Une mise de côté impossible refuse la troncature** : le fichier reste tel quel, et le
/// document s'ouvre quand même — ce qu'on y change part dans un brouillon.
#[test]
fn test_une_fin_qu_on_ne_peut_pas_mettre_de_cote_n_est_pas_touchee() {
    let d = dossier("fin-intouchee");
    let (chemin, octets, _) = document_abime(&d);
    // Un fichier à la place du dossier : rien ne peut s'y ranger.
    std::fs::create_dir_all(d.join("brouillons")).unwrap();
    std::fs::write(d.join("brouillons").join("recuperation"), b"un fichier").unwrap();

    let mut autre = rouvrir(&d, &chemin);
    let toast = autre.ui.current_toast.as_ref().expect("l'ouverture parle");
    assert!(
        toast.message.contains("laissée intacte"),
        "{}",
        toast.message
    );
    assert!(toast.message.contains("brouillon"), "{}", toast.message);
    noter(&mut autre, "ailleurs", "écrit dans un brouillon");
    image_suivante(&mut autre);
    drop(autre);
    assert!(
        std::fs::read(&chemin).unwrap() == octets,
        "le document n'a pas été touché"
    );
}
