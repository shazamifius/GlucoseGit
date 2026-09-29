//! **COLLER-2 — une image collée ne se scelle jamais vide**, et celle qui l'a été se répare.
//!
//! Le 28/09, trois images collées se sont scellées dans son document avec **zéro octet** :
//! l'ouvrier écrivait leur PNG en place, et le scribe l'a lu au moment où il venait d'être créé,
//! vide. Le document en gardait une copie vide, et l'image ne se montrait plus. Leurs fichiers,
//! eux, étaient entiers. Désormais l'ouvrier pose le PNG d'un bloc, l'écriture n'attend qu'un
//! fichier non vide, le scribe refuse le vide — et l'ouverture répare ce que le défaut a laissé.

use super::tests::{application, dossier, image_suivante, png, rouvrir};
use glucose_core::hash::sha256;
use glucose_core::persist::histoire;
use glucose_core::types::BoardImage;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

/// Pose dans le tableau actif une image dont les octets viennent de `cle`.
pub(crate) fn poser_l_image(app: &mut crate::app::GlucoseApp, cle: &str) {
    let b = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new("collee", 0.0, 0.0, 8.0, 6.0);
    img.src = Some(cle.to_string());
    app.store.add_image(&b, img);
}

/// Ce que le défaut laissait : un objet de zéro octet au bout de la chaîne du document, et le
/// lien de l'image vers lui.
pub(crate) fn sceller_du_vide(document: &Path, cle: &str) {
    let ouvert = histoire::ouvrir(&mut std::io::BufReader::new(
        std::fs::File::open(document).expect("document"),
    ))
    .expect("lisible");
    let mut chaine = ouvert.chaine;
    let vide = sha256(&[]);
    let objet = chaine.entete_d_objet(&vide, 0).expect("entête");
    let lien = chaine.encadrer(histoire::nature::LIEN, &histoire::contenu_lien(cle, &vide));
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .open(document)
        .expect("document");
    f.seek(SeekFrom::Start(ouvert.fin)).expect("sa fin");
    f.write_all(&objet).expect("l'objet vide");
    f.write_all(&lien).expect("son lien");
}

/// **Un document où une image s'est scellée vide se répare à l'ouverture** : elle se relit
/// depuis son fichier, entière, et se rescelle — dans le fichier, le lien mène à ses octets.
#[test]
fn test_une_image_scellee_vide_se_rescelle_a_l_ouverture() {
    let d = dossier("coller-2-reparation");
    let chemin = d.join("colle.glucose");
    let cle = d.join("colle.png").to_string_lossy().into_owned();
    let mut app = application(&d);
    app.save_to(chemin.clone());
    poser_l_image(&mut app, &cle);
    image_suivante(&mut app);
    assert!(app.fermer_le_document());
    sceller_du_vide(&chemin, &cle);
    let lu = histoire::ouvrir(&mut std::io::BufReader::new(
        std::fs::File::open(&chemin).unwrap(),
    ))
    .unwrap();
    assert_eq!(
        lu.liens.get(&cle),
        Some(&sha256(&[])),
        "le défaut, reproduit"
    );

    let octets = std::fs::read(png(&d, "colle.png", 70)).unwrap();
    let mut autre = rouvrir(&d, &chemin);
    assert_eq!(
        autre.disque.objets.lire(&cle).as_deref(),
        Some(&octets[..]),
        "l'image se relit, entière, depuis son fichier"
    );
    image_suivante(&mut autre);
    assert!(autre.disque.objets.est_scellee(&cle), "et se rescelle");
    assert!(autre.fermer_le_document());
    let repare = histoire::ouvrir(&mut std::io::BufReader::new(
        std::fs::File::open(&chemin).unwrap(),
    ))
    .unwrap();
    assert_eq!(
        repare.liens.get(&cle),
        Some(&sha256(&octets)),
        "dans le fichier, le lien mène à ses octets"
    );
}

/// **Un fichier qui paraît vide n'est pas encore l'image** — celui qu'on est en train d'écrire :
/// il ne se scelle pas, et se scelle entier une fois écrit.
#[test]
fn test_un_fichier_encore_vide_ne_se_scelle_pas() {
    let d = dossier("coller-2-vide");
    let chemin = d.join("colle.glucose");
    let fichier = d.join("en-cours.png");
    let cle = fichier.to_string_lossy().into_owned();
    let mut app = application(&d);
    app.save_to(chemin.clone());
    std::fs::File::create(&fichier).expect("créé, encore vide");
    poser_l_image(&mut app, &cle);
    image_suivante(&mut app);
    assert!(
        !app.disque.objets.est_scellee(&cle),
        "le vide ne se scelle pas"
    );

    let octets = std::fs::read(png(&d, "en-cours.png", 30)).unwrap();
    image_suivante(&mut app);
    assert!(app.disque.objets.est_scellee(&cle), "écrit, il se scelle");
    assert_eq!(app.disque.objets.lire(&cle).as_deref(), Some(&octets[..]));
}

/// **Le scribe refuse de sceller zéro octet**, et le dit : l'image reste à sceller.
#[test]
fn test_le_scribe_refuse_le_vide() {
    let d = dossier("coller-2-scribe");
    let chemin = d.join("colle.glucose");
    let mut app = application(&d);
    app.save_to(chemin.clone());
    let e = app.disque.ecriture.as_mut().expect("le document s'écrit");
    e.sceller_des_octets("rien", Vec::new());
    e.synchroniser().expect("le disque suit");
    assert!(!app.disque.objets.est_scellee("rien"));
    assert!(
        e.prendre_l_erreur()
            .is_some_and(|m| m.contains("aucun octet")),
        "le refus se dit"
    );
}
