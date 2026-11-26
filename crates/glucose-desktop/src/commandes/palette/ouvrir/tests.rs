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

/// **Au téléphone, la palette montre les documents rangés** (RECENTS-2) : rien qu'eux — un connu
/// d'ailleurs n'y est pas —, du plus récemment touché au plus ancien, chacun avec son âge et sans
/// dossier ; le clavier attend le doigt, « Annuler » ferme en bas. Le dossier privé de Glucose ne
/// filtre rien ; l'appui long y gère un document, jamais au bureau.
#[test]
fn recents_2_au_telephone_les_documents_ranges() {
    let (glucose, ailleurs) = (
        Dossier::nouveau("telephone-glucose"),
        Dossier::nouveau("telephone-ailleurs"),
    );
    let ranges = glucose.join("documents");
    std::fs::create_dir_all(&ranges).expect("le dossier");
    let (vieux, recent) = (
        ranges.join("Vieux.glucose"),
        ranges.join("Planches.glucose"),
    );
    let connu = ailleurs.join("Connu.glucose");
    for d in [&vieux, &recent, &connu] {
        document(d);
    }
    let il_y_a_deux_jours = std::time::SystemTime::now() - std::time::Duration::from_secs(172_800);
    let fichier = std::fs::File::options().write(true).open(&vieux);
    (fichier.expect("ouvert").set_modified(il_y_a_deux_jours)).expect("daté");
    let mut app = application(&glucose, &[&connu]);
    app.ui.documents_ranges = true;
    app.choisir_un_document();

    assert_eq!(
        rangees(&app),
        [
            ("Planches".to_string(), false),
            ("Vieux".to_string(), false),
            ("Annuler".to_string(), false)
        ]
    );
    let Some((q, Suite::Palette(etat))) = app.ui.question.clone() else {
        panic!("la palette");
    };
    assert!(q.clavier_au_toucher, "le clavier attend le doigt");
    assert_eq!(q.texte, super::AIDE_AU_DOIGT);
    let ages: Vec<&str> = q.precisions.iter().map(|(_, p)| p.texte.as_str()).collect();
    assert_eq!(
        ages,
        ["à l'instant", "il y a 2 j"],
        "leur âge, sans dossier"
    );
    assert_eq!(app.document_a_gerer(&etat, 1), Some(vieux.clone()));

    ecrire(&mut app, "documents");
    assert_eq!(rangees(&app), [("Annuler".to_string(), false)], "aucun");
    ecrire(&mut app, "vieu");
    assert_eq!(rangees(&app)[0].0, "Vieux");
    app.repondre_a_la_question(Some(Reponse::Annuler));
    assert!(app.ui.question.is_none() && app.project_path.is_none());

    app.ui.documents_ranges = false;
    assert_eq!(app.document_a_gerer(&etat, 1), None, "au bureau, rien");
}
