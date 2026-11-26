//! Ce que garantit la palette des documents (RECENTS-1, fiche 174), telle que `Ctrl+O` l'ouvre.

use crate::app::GlucoseApp;
use crate::commandes::Commande;
use crate::epreuve::Dossier;
use crate::ui::question::{Reponse, Suite};
use std::path::{Path, PathBuf};

/// Un vrai document, que l'ouverture lit.
fn document(chemin: &Path) {
    let s = glucose_core::store::Store::new("Essai");
    let octets =
        glucose_core::persist::encode(&s.project, &glucose_core::types::AssetStore::default(), 0);
    std::fs::write(chemin, octets).expect("un document");
}

/// Une application dont la liste des connus est `connus`, dans `glucose`.
fn application(glucose: &Path, connus: &[&Path]) -> GlucoseApp {
    let liste: String = connus
        .iter()
        .map(|p| format!("{}\n", p.display()))
        .collect();
    std::fs::write(glucose.join(crate::recensement::FICHIER), liste).expect("connus");
    let mut app = GlucoseApp::new();
    app.disque.brouillons = glucose.join("brouillons");
    app.disque.documents = glucose.join("documents");
    let reveil = std::sync::Arc::new(|| {});
    app.disque.recenseur.demarrer(glucose, Vec::new(), reveil);
    app
}

/// Les rangées posées : leur libellé, et si elles se disent introuvables.
fn rangees(app: &GlucoseApp) -> Vec<(String, bool)> {
    let Some((q, Suite::Palette(_))) = &app.ui.question else {
        panic!("la palette n'est pas posée");
    };
    q.choix
        .iter()
        .map(|(libelle, r)| {
            let precision = q.precisions.iter().find(|(p, _)| p == r);
            (
                libelle.clone(),
                precision.is_some_and(|(_, p)| p.introuvable),
            )
        })
        .collect()
}

fn ecrire(app: &mut GlucoseApp, filtre: &str) {
    if let Some((q, _)) = &mut app.ui.question {
        q.champ.as_mut().expect("un champ").texte = filtre.into();
    }
    app.suivre_la_palette();
}

/// **`Ctrl+O` ouvre la palette des documents** : les récents dans leur ordre, l'introuvable en
/// rouge, le voisin jamais ouvert ensuite, « Parcourir ce PC… » en bas ; écrire filtre ; Entrée
/// ouvre celui en évidence.
#[test]
fn recents_1_ctrl_o_montre_les_documents_et_ouvre_le_choisi() {
    let (glucose, ici) = (
        Dossier::nouveau("ouvrir-glucose"),
        Dossier::nouveau("ouvrir-ici"),
    );
    let (a, b, voisin) = (
        ici.join("Alpha.glucose"),
        ici.join("Bêta.glucose"),
        ici.join("Voisin.glucose"),
    );
    for p in [&a, &b, &voisin] {
        document(p);
    }
    let parti: PathBuf = ici.join("Parti.glucose");
    let mut app = application(&glucose, &[&b, &parti, &a]);
    app.executer(Commande::OuvrirUnDocument);
    let attendues = [
        ("Bêta", false),
        ("Parti", true),
        ("Alpha", false),
        ("Voisin", false),
        ("Parcourir ce PC…", false),
    ];
    let attendues: Vec<(String, bool)> =
        attendues.iter().map(|(l, r)| (l.to_string(), *r)).collect();
    assert_eq!(rangees(&app), attendues);

    let liste = std::fs::read_to_string(glucose.join(crate::recensement::FICHIER)).expect("lue");
    assert_eq!(
        liste.matches('\t').count(),
        2,
        "la liste a appris leurs numéros : {liste}"
    );

    // Les lettres éparses ne valent que pour le nom : « gouvi » est dans l'ordre dans le chemin
    // de leur dossier (glucose-ouvrir-ici), et ne trouve rien.
    ecrire(&mut app, "gouvi");
    assert_eq!(rangees(&app).len(), 1, "rien que « Parcourir ce PC… »");
    ecrire(&mut app, "alp");
    assert_eq!(rangees(&app)[0].0, "Alpha", "écrire filtre");
    app.repondre_a_la_question(Some(Reponse::Choix(0)));
    assert_eq!(
        app.project_path.as_deref(),
        Some(a.as_path()),
        "Entrée l'ouvre"
    );
}

/// **Un introuvable choisi demande à être oublié** ; « Retirer de la liste » l'en retire, et la
/// palette revient sans lui.
#[test]
fn recents_1_un_introuvable_s_oublie() {
    let (glucose, ici) = (
        Dossier::nouveau("oublier-glucose"),
        Dossier::nouveau("oublier-ici"),
    );
    let (a, parti) = (ici.join("Alpha.glucose"), ici.join("Parti.glucose"));
    document(&a);
    let mut app = application(&glucose, &[&parti, &a]);
    app.executer(Commande::OuvrirUnDocument);
    app.repondre_a_la_question(Some(Reponse::Choix(0)));
    assert!(
        matches!(&app.ui.question, Some((_, Suite::Oublier(p))) if *p == parti),
        "la question de l'oubli"
    );
    app.repondre_a_la_question(Some(Reponse::Oui));
    let noms: Vec<String> = rangees(&app).into_iter().map(|(l, _)| l).collect();
    assert_eq!(
        noms,
        ["Alpha", "Parcourir ce PC…"],
        "la palette revient sans lui"
    );
    let liste = std::fs::read_to_string(glucose.join(crate::recensement::FICHIER)).expect("lue");
    assert!(!liste.contains("Parti"), "la liste l'a oublié : {liste}");
}

/// **Ce que la fouille trouve paraît** : sa fin réveille la boucle, qui redessine la palette, et
/// la palette se recompose en se dessinant — le document jamais ouvert, loin des connus, est là.
#[test]
fn recents_1_la_trouvaille_de_la_fouille_parait() {
    let (glucose, maison) = (
        Dossier::nouveau("fouille-glucose"),
        Dossier::nouveau("fouille-maison"),
    );
    let loin = maison.join("Images/Lointain.glucose");
    std::fs::create_dir_all(loin.parent().expect("son dossier")).expect("le dossier");
    document(&loin);
    let mut app = application(&glucose, &[]);
    let reveil = std::sync::Arc::new(|| {});
    app.disque
        .fouille
        .brancher(Some(maison.to_path_buf()), reveil);
    app.executer(Commande::OuvrirUnDocument);
    let debut = std::time::Instant::now();
    while app.disque.fouille.en_cours() && debut.elapsed() < std::time::Duration::from_secs(10) {
        std::thread::yield_now();
    }
    let _ = app.prendre_la_salissure();
    app.suivre_le_disque();
    assert_ne!(
        app.prendre_la_salissure(),
        crate::salissure::Salissure::Rien,
        "la trouvaille redessine"
    );
    app.suivre_la_palette();
    let noms: Vec<String> = rangees(&app).into_iter().map(|(l, _)| l).collect();
    assert_eq!(noms, ["Lointain", "Parcourir ce PC…"]);
}
