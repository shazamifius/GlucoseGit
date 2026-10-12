//! PARTAGE-2 — sans sélecteur de fichiers, exporter et enregistrer une image passent par la
//! feuille de partage : le fichier s'écrit dans `partages/`, et lui seul s'offre.

use super::*;
use crate::epreuve::Dossier;
use crate::persist::disque::tests::image_suivante;
use std::sync::{Arc, Mutex};

type Offerts = Arc<Mutex<Vec<(PathBuf, String)>>>;

/// Une feuille de partage d'épreuve, branchée comme `glucose-android` la branche : elle retient
/// ce qu'on lui offre.
fn feuille() -> Offerts {
    let offerts: Offerts = Arc::new(Mutex::new(Vec::new()));
    let retenus = Arc::clone(&offerts);
    crate::plateforme::partager::installer_le_partageur(Arc::new(move |chemin, type_mime| {
        retenus
            .lock()
            .expect("la liste")
            .push((chemin.to_path_buf(), type_mime.to_string()));
    }));
    offerts
}

/// Une application qui habite un dossier à elle : ses partages ne croisent ceux d'aucune autre
/// épreuve.
fn application(d: &Dossier) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.habiter(d);
    app
}

fn offerts(o: &Offerts) -> Vec<(PathBuf, String)> {
    o.lock().expect("la liste").clone()
}

fn rang(format: Format) -> Reponse {
    Reponse::Choix(
        Format::TOUS
            .iter()
            .position(|f| *f == format)
            .expect("un format"),
    )
}

/// **Exporter, sans sélecteur, demande le format, puis offre le fichier** — le bouton qui ne
/// faisait rien au téléphone. Le fichier est dans les partages, et c'est lui qui s'offre.
#[test]
fn test_partage_2_exporter_sans_selecteur_demande_le_format_puis_offre_le_fichier() {
    let d = Dossier::nouveau("partage-exporter");
    let mut app = application(&d);
    let o = feuille();
    app.export_board();
    assert_eq!(
        app.ui.question.as_ref().map(|(_, s)| s.clone()),
        Some(Suite::Exporter),
        "le format se demande"
    );
    app.ui.question = None;
    app.exporter_au_format(rang(Format::Markdown));
    let attendu = app
        .disque
        .partages
        .join(format!("{}.md", app.document_label()));
    assert!(attendu.exists(), "le tableau est écrit dans les partages");
    assert_eq!(offerts(&o), vec![(attendu, "text/markdown".to_string())]);
}

/// **L'export en image s'offre quand son fil a fini**, pas avant : l'application choisie
/// recevrait un fichier à moitié écrit.
#[test]
fn test_partage_2_l_export_en_image_s_offre_quand_son_fil_a_fini() {
    let d = Dossier::nouveau("partage-png");
    let mut app = application(&d);
    let o = feuille();
    app.exporter_au_format(rang(Format::Png));
    assert!(app.exporte_encore(), "le fil de l'export travaille");
    assert!(offerts(&o).is_empty(), "rien ne s'offre avant la fin");
    let debut = std::time::Instant::now();
    while app.exporte_encore() {
        assert!(debut.elapsed().as_secs() < 120, "l'export ne finit pas");
        app.suivre_l_export();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let attendu = app
        .disque
        .partages
        .join(format!("{}.png", app.document_label()));
    assert_eq!(
        offerts(&o),
        vec![(attendu.clone(), "image/png".to_string())]
    );
    assert!(image::open(&attendu).is_ok(), "une image entière");
}

/// **Une image enregistrée s'offre telle qu'elle a été posée** — ses octets scellés, sous leur
/// extension —, sans sélecteur.
#[test]
fn test_partage_2_une_image_enregistree_s_offre_telle_quelle() {
    let d = Dossier::nouveau("partage-image");
    let mut app = application(&d);
    app.save_to(d.join("doc.glucose"));
    let photo = crate::persist::disque::tests::png(&d, "photo.png", 90);
    let octets = std::fs::read(&photo).expect("ses octets");
    let b = app.store.project.active_board_id.clone();
    app.place_image_file(&b, &photo, (0.0, 0.0)).expect("posée");
    image_suivante(&mut app);
    let o = feuille();
    app.enregistrer_l_image_sous();
    let recus = offerts(&o);
    assert_eq!(recus.len(), 1, "l'image s'offre : {recus:?}");
    let (chemin, type_mime) = &recus[0];
    assert_eq!(type_mime, "image/png");
    assert_eq!(chemin.parent(), Some(app.disque.partages.as_path()));
    assert_eq!(
        std::fs::read(chemin).expect("relue"),
        octets,
        "octet pour octet"
    );
}

/// **Les partages ne gardent que le dernier fichier**, et rien ne s'offre de ce qui s'écrit
/// ailleurs — ni sans feuille de partage.
#[test]
fn test_partage_2_le_dossier_ne_garde_que_le_dernier_et_seul_lui_s_offre() {
    let d = Dossier::nouveau("partage-dossier");
    let app = application(&d);
    let premier = app.a_partager("premier.md");
    std::fs::write(&premier, b"un").expect("ecrit");
    let second = app.a_partager("second.md");
    assert!(!premier.exists(), "le partage d'avant est parti");
    std::fs::write(&second, b"deux").expect("ecrit");

    app.offrir_si_partage(&second);
    let o = feuille();
    let ailleurs = d.join("ailleurs.md");
    std::fs::write(&ailleurs, b"x").expect("ecrit");
    app.offrir_si_partage(&ailleurs);
    assert!(offerts(&o).is_empty(), "hors des partages, rien ne s'offre");
    app.offrir_si_partage(&second);
    assert_eq!(offerts(&o).len(), 1);
}
