//! **PARTAGE-SURVIT** (fiche 131) et **REVENIR-2** (fiche 148), par la vraie application : partager, fermer par la croix,
//! relancer. Le fil vers la salle part vers une adresse morte — jamais le réseau, jamais le
//! presse-papiers de l'utilisateur.

use super::super::{lien, CanevasPartage, Entree, QUITTER};
use super::*;
use crate::persist::close::tests::fermer;
use crate::persist::disque::tests::{application, dossier, image_suivante, noter};
use glucose_core::store::journal::partage::GestePartage;
use glucose_core::store::journal::Main;
use glucose_core::store::Arrivee;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const MORTE: &str = "http://127.0.0.1:9";

/// Une application de ce dossier, son souvenir des canevas dans ce dossier, sa porte branchée sur
/// une salle morte.
fn lancer(d: &Path) -> GlucoseApp {
    let mut app = application(d);
    app.disque.canevas = d.join(FICHIER);
    app.lancement.agents.boite = Some(crate::agents::Boite::nouvelle(Arc::new(|| {})));
    app.lancement.agents.relais = Some(MORTE.into());
    app
}

/// Un document enregistré dans ce dossier, puis partagé — sa base revenue numérotée (n° 1) : il
/// est dans la salle.
fn document_partage(d: &Path) -> (GlucoseApp, PathBuf, Code) {
    let code = Code::neuf([7; 32]);
    let boite = crate::agents::Boite::nouvelle(Arc::new(|| {}));
    document_partage_par(d, lien::lancer(MORTE, code.clone(), 0, boite), code)
}

/// Le même, par ce fil.
fn document_partage_par(d: &Path, lien: lien::Lien, code: Code) -> (GlucoseApp, PathBuf, Code) {
    let chemin = d.join("partagé.glucose");
    let mut app = lancer(d);
    noter(&mut app, "c1", "à plusieurs");
    app.save_to(chemin.clone());
    image_suivante(&mut app);
    assert!(app.store.partager_ce_canevas("a1b2"));
    let qui = ("a1b2".into(), "Alice".into(), 0);
    let objets = Arc::clone(&app.disque.objets);
    app.lancement.agents.partage = Some(CanevasPartage::neuf(
        lien,
        code.clone(),
        qui,
        Entree::Partage,
        objets,
    ));
    app.retenir_le_canevas_en_cours();
    arrive(&mut app, 1, "a1b2-1");
    image_suivante(&mut app);
    (app, chemin, code)
}

/// Un geste de la salle arrive au numéro `n`, sous ce nom, et s'applique — sans s'écrire encore
/// (l'image suivante l'écrira) : vide (le retour de ma base), ou un vrai geste de Bob.
fn arrive(app: &mut GlucoseApp, n: u64, id: &str) {
    let main = Main::Participant {
        marque: "b0b".into(),
        nom: "Bob".into(),
    };
    let geste = if id.starts_with("b0b") {
        geste_de_bob(&app.store.project, id)
    } else {
        GestePartage::default()
    };
    app.store.recevoir(Arrivee {
        n,
        id: id.into(),
        main,
        geste,
    });
    app.store.appliquer_les_arrivees();
}

/// Le geste que Bob fait chez lui, dans le même canevas : il pose un texte.
fn geste_de_bob(projet: &glucose_core::types::Project, id: &str) -> GestePartage {
    let mut bob = glucose_core::store::Store::new("bob");
    bob.load_project(projet.clone());
    assert!(bob.rejoindre("b0b", true, 1));
    let t = bob.project.active_board_id.clone();
    let texte = glucose_core::types::Annotation::text(id, 0.0, 300.0, "posé de loin");
    bob.add_annotation(&t, texte);
    bob.partager_les_gestes();
    match bob.prendre_les_sortants().pop().map(|s| s.envoi) {
        Some(glucose_core::store::Envoi::Geste(g)) => g,
        _ => panic!("le geste de Bob"),
    }
}

fn souvenir(d: &Path, chemin: &Path) -> Option<Souvenir> {
    lire(&d.join(FICHIER), &chemin.display().to_string())
}

/// Le document de ce dossier, relancé : il revient dans sa salle, s'il y était.
fn relancer(d: &Path) -> GlucoseApp {
    let mut relance = lancer(d);
    relance.retrouver_le_travail(None);
    relance.reprendre_le_canevas_partage();
    relance
}

/// Où en est l'entrée du partage, s'il y en a un.
fn entree(app: &GlucoseApp) -> Option<Entree> {
    app.lancement.agents.partage.as_ref().map(|cp| cp.entree)
}

/// **Fermé proprement, relancé : il revient dans sa salle**, depuis le dernier numéro que son
/// fichier contient — la borne, retenue telle quelle.
#[test]
fn revenir_ferme_proprement_il_revient_depuis_sa_borne() {
    let d = dossier("canevas-survit");
    let (mut app, chemin, code) = document_partage(&d);
    arrive(&mut app, 2, "b0b-1");
    image_suivante(&mut app);
    assert!(fermer(&mut app), "une fermeture propre");
    let attendu = Souvenir {
        code: code.texte_ecrire().unwrap(),
        retenu: 2,
        dedans: true,
    };
    assert_eq!(souvenir(&d, &chemin), Some(attendu.clone()));
    drop(app);

    let relance = relancer(&d);
    let cp = relance
        .lancement
        .agents
        .partage
        .as_ref()
        .expect("il revient");
    assert_eq!(cp.entree, Entree::Revient);
    assert!(cp.code.peut_ecrire());
    assert_ne!(cp.marque, "a1b2", "une marque neuve");
    assert_eq!(cp.borne.retenue, 2);
    assert!(relance.store.partage.is_none(), "pas avant la salle");
    assert_eq!(souvenir(&d, &chemin), Some(attendu));
}

/// **Le partage suit son document** : après la croix, plus rien de la salle ne s'y applique, et
/// aucun brouillon ne naît — le lancement suivant le rouvrirait avant le document (BROUILLON-1).
/// Le numéro retenu reste celui du fichier.
#[test]
fn partage_survit_apres_la_croix_rien_de_la_salle_ne_s_applique() {
    let d = dossier("canevas-croix");
    let (mut app, chemin, _) = document_partage(&d);
    assert!(fermer(&mut app));
    assert!(
        app.lancement.agents.partage.is_none(),
        "le partage s'arrête avec son document"
    );
    arrive(&mut app, 2, "b0b-1");
    image_suivante(&mut app);
    app.finir();
    let brouillons = std::fs::read_dir(d.join("brouillons"))
        .map(|l| {
            l.flatten()
                .filter(|e| e.path().extension().is_some_and(|x| x == "glucose"))
                .count()
        })
        .unwrap_or(0);
    assert_eq!(brouillons, 0, "aucun brouillon parasite");
    assert_eq!(souvenir(&d, &chemin).map(|s| s.retenu), Some(1));
}

/// **Un de mes gestes attend encore son numéro** à la fermeture : il est dans le fichier — la
/// borne reste au dernier numéro confirmé, et le document revient quand même (la fusion le rejoint).
#[test]
fn revenir_un_geste_en_attente_n_empeche_rien() {
    let d = dossier("canevas-attente");
    let (mut app, chemin, _) = document_partage(&d);
    noter(&mut app, "c2", "parti, pas revenu");
    image_suivante(&mut app);
    assert_eq!(app.store.partage.as_ref().map(|p| p.en_attente()), Some(1));
    assert!(fermer(&mut app));
    assert_eq!(souvenir(&d, &chemin).map(|s| s.retenu), Some(1));
    drop(app);
    assert_eq!(entree(&relancer(&d)), Some(Entree::Revient));
}

/// **Un arrêt net** (aucune fermeture) : dedans, à la borne que le disque a rendue sûre — il
/// revient aussi.
#[test]
fn revenir_un_arret_net_revient_aussi() {
    let d = dossier("canevas-net");
    let (app, chemin, _) = document_partage(&d);
    drop(app);
    assert_eq!(
        souvenir(&d, &chemin).map(|s| (s.retenu, s.dedans)),
        Some((0, true))
    );
    assert_eq!(entree(&relancer(&d)), Some(Entree::Revient));
}

/// **« Quitter le partage » garde le code, dehors** (fiche 146 § 4, le 1) : relancé, le document
/// reste à soi seul ; son « Partager » revient dans la même salle, au lieu d'en ouvrir une seconde.
#[test]
fn revenir_quitter_garde_le_code_et_partager_y_revient() {
    let d = dossier("canevas-quitter");
    let (mut app, chemin, code) = document_partage(&d);
    arrive(&mut app, 2, "b0b-1");
    app.repondre_a_collaborer(QUITTER);
    assert!(app.lancement.agents.partage.is_none());
    let dehors = Souvenir {
        code: code.texte_ecrire().unwrap(),
        retenu: 2,
        dedans: false,
    };
    assert_eq!(
        souvenir(&d, &chemin),
        Some(dehors.clone()),
        "écrit, synchronisé, retenu"
    );
    assert!(fermer(&mut app));
    assert_eq!(
        souvenir(&d, &chemin),
        Some(dehors),
        "la croix ne le remet pas dedans"
    );
    drop(app);

    let mut relance = relancer(&d);
    assert!(
        relance.lancement.agents.partage.is_none(),
        "dehors : il ne revient pas seul"
    );
    relance.montrer_collaborer_avec(None);
    let premier = relance
        .ui
        .question
        .as_ref()
        .map(|(q, _)| q.choix[0].0.clone());
    assert_eq!(
        premier.as_deref(),
        Some("Revenir dans sa salle"),
        "le bouton dit ce qu'il fera (fiche 148 § 3, le 1)"
    );
    relance.partager_ce_canevas();
    let cp = relance
        .lancement
        .agents
        .partage
        .as_ref()
        .expect("il revient");
    assert_eq!(cp.entree, Entree::Revient);
    assert_eq!(cp.code.texte_ecrire(), code.texte_ecrire(), "la même salle");
    assert_eq!(souvenir(&d, &chemin).map(|s| s.dedans), Some(true));
}

/// **Revenir, jusqu'au bout** : fermé au n° 1 ; Bob pose un texte (n° 2) ; relancé, Alice écrit
/// pendant que la salle arrive. La salle donnée, les deux se rejoignent : le texte de Bob est là, le
/// sien aussi — et part comme son geste.
#[test]
fn revenir_la_salle_arrive_et_les_deux_se_rejoignent() {
    let d = dossier("canevas-revenir");
    let (mut app, _chemin, code) = document_partage(&d);
    let base = app.store.project.clone();
    assert!(fermer(&mut app));
    drop(app);

    let mut relance = relancer(&d);
    let (fil, salle) = lien::d_epreuve();
    relance
        .lancement
        .agents
        .partage
        .as_mut()
        .expect("il revient")
        .lien = fil;
    noter(&mut relance, "c2", "écrit pendant le retour");
    image_suivante(&mut relance);
    let lettre = |corps| {
        Box::new(glucose_core::salle::lettre::Lettre {
            marque: "b0b".into(),
            nom: "Bob".into(),
            couleur: 0,
            main: Main::Humain,
            corps,
        })
    };
    use glucose_core::salle::lettre::Corps;
    let de_bob = geste_de_bob(&base, "b0b-1");
    let nouvelles = [
        lien::Nouvelle::Lettre {
            n: 1,
            id: "a1b2-1".into(),
            lettre: lettre(Corps::Base(Box::new(base))),
        },
        lien::Nouvelle::Lettre {
            n: 2,
            id: "b0b-1".into(),
            lettre: lettre(Corps::Geste(de_bob)),
        },
        lien::Nouvelle::AJour(2),
    ];
    for nouvelle in nouvelles {
        salle.send(nouvelle).expect("la file");
    }
    relance.suivre_le_canevas_partage();

    assert_eq!(entree(&relance), Some(Entree::Dedans));
    let p = relance.store.partage.as_ref().expect("il participe");
    assert_eq!(
        (p.confirme, p.en_attente()),
        (2, 1),
        "au n° 2, son texte en attente"
    );
    let board = relance.store.project.active_board_id.clone();
    assert!(
        relance.store.project.annotation(&board, "b0b-1").is_some(),
        "le texte de Bob"
    );
    assert!(
        relance.store.project.annotation(&board, "c2").is_some(),
        "le sien"
    );
    assert!(code.peut_ecrire());
}

/// **La borne ne dépasse jamais le disque** : la salle confirme le n° 2, mais tant que le scribe
/// ne l'a pas écrit, le souvenir reste à 0 ; relâché, la synchronisation revient, la borne monte.
#[test]
fn revenir_la_borne_attend_le_disque() {
    let d = dossier("canevas-borne");
    let (mut app, chemin, _) = document_partage(&d);
    let relacher = app.disque.ecriture.as_ref().expect("le scribe").retenir();
    arrive(&mut app, 2, "b0b-1");
    for _ in 0..3 {
        app.tenir_la_borne();
    }
    assert_eq!(
        souvenir(&d, &chemin).map(|s| s.retenu),
        Some(0),
        "pas avant le disque"
    );
    relacher.send(()).expect("le scribe attend");
    let mut monte = false;
    for _ in 0..200 {
        app.tenir_la_borne();
        monte = souvenir(&d, &chemin).is_some_and(|s| s.retenu == 2);
        if monte {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(monte, "la borne monte une fois le disque sûr");
}

/// **Le code « écrire » est un secret** : dans le dossier de Glucose, lisible de lui seul — jamais
/// dans le `.glucose`, qu'on envoie.
#[test]
fn partage_survit_le_code_reste_un_secret_hors_du_document() {
    let d = dossier("canevas-secret");
    let (mut app, chemin, code) = document_partage(&d);
    assert!(fermer(&mut app));
    let secret = code.texte_ecrire().unwrap();
    let document = std::fs::read(&chemin).expect("le document");
    assert!(
        !document
            .windows(secret.len())
            .any(|w| w == secret.as_bytes()),
        "pas dans le .glucose"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(d.join(FICHIER))
            .expect("le souvenir")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "lisible de lui seul");
    }
}

/// **Rejoindre** : la base reçue devient le document — et ferme celui qu'on quitte, par la même
/// porte — sans couper le partage qui attendait cette base : on y entre.
#[test]
fn partage_survit_la_base_recue_ne_coupe_pas_le_partage_qu_on_rejoint() {
    let d = dossier("canevas-rejoindre");
    let mut app = lancer(&d);
    noter(&mut app, "c1", "le document qu'on quitte");
    image_suivante(&mut app);
    let code = Code::neuf([8; 32]);
    let lien = lien::lancer(
        MORTE,
        code.clone(),
        0,
        app.lancement.agents.boite.clone().unwrap(),
    );
    let objets = Arc::clone(&app.disque.objets);
    let qui = ("c0c".into(), "Carla".into(), 1);
    app.lancement.agents.partage = Some(CanevasPartage::neuf(
        lien,
        code,
        qui,
        Entree::AttendLaBase,
        objets,
    ));
    let base = glucose_core::types::Project::new("la salle");
    app.entrer_par_la_base(1, base);
    let cp = app
        .lancement
        .agents
        .partage
        .as_ref()
        .expect("le partage reste");
    assert_eq!(cp.entree, Entree::Dedans);
    assert_eq!(app.store.partage.as_ref().map(|p| p.confirme), Some(1));
}

/// **Seul le scribe reprend le partage** (fiche 144 § 2) : une fenêtre ouvre le document sans
/// pouvoir l'écrire — le verrou pris entre sa vérification et son ouverture (deux lancements à la
/// même seconde, 14 h 02), ou le fichier en lecture seule, ce qu'on fait ici. Elle ne rejoint pas
/// la salle et laisse le numéro retenu tel quel ; celle qui l'écrit, ensuite, la rejoint.
#[test]
fn partage_survit_une_seconde_fenetre_ne_reprend_pas_la_salle() {
    let d = dossier("canevas-seconde");
    let (mut app, chemin, _) = document_partage(&d);
    assert!(fermer(&mut app));
    drop(app);
    let retenu = souvenir(&d, &chemin);
    assert_eq!(retenu.as_ref().map(|s| s.retenu), Some(1));

    let droits = std::fs::metadata(&chemin).unwrap().permissions();
    let mut lecture_seule = droits.clone();
    lecture_seule.set_readonly(true);
    std::fs::set_permissions(&chemin, lecture_seule).unwrap();
    let mut seconde = lancer(&d);
    seconde.retrouver_le_travail(Some(chemin.clone()));
    // La prémisse, dite : elle a le document, sans pouvoir l'écrire.
    assert!(
        seconde.disque.objets.document().is_some(),
        "elle l'a ouvert"
    );
    assert!(seconde.disque.ecriture.is_none(), "sans l'écrire");
    seconde.reprendre_le_canevas_partage();
    assert!(
        seconde.lancement.agents.partage.is_none(),
        "la fenêtre qui n'écrit pas a rejoint la salle"
    );
    assert_eq!(souvenir(&d, &chemin), retenu, "le numéro retenu, intact");
    drop(seconde);

    std::fs::set_permissions(&chemin, droits).unwrap();
    let mut scribe = lancer(&d);
    scribe.retrouver_le_travail(Some(chemin.clone()));
    assert!(scribe.disque.ecriture.is_some(), "le scribe l'écrit");
    scribe.reprendre_le_canevas_partage();
    assert!(
        scribe.lancement.agents.partage.is_some(),
        "le scribe, lui, reprend"
    );
}

/// **SYNCHRO-1** (sa note du 25/11, `text-utzs-2464` : *« je note plein de trucs dans le vide »*) :
/// la rangée des onglets dit le lien — relié ; coupé, et ce qui attend d'en partir ; rien hors
/// d'un partage.
#[test]
fn synchro_le_temoin_dit_le_lien_et_ce_qui_attend() {
    use crate::agents::publique::lien::EtatDuLien;
    use crate::ui::Synchro;
    let d = dossier("canevas-temoin");
    let (fil, _salle) = lien::d_epreuve();
    let (mut app, _, _) = document_partage_par(&d, fil, Code::neuf([7; 32]));
    app.suivre_le_canevas_partage();
    assert_eq!(app.ui.synchro, Some(Synchro::Reliee));
    let cp = app.lancement.agents.partage.as_ref().expect("partagé");
    cp.lien
        .poser_l_etat(EtatDuLien::Hors("le téléphone dort".into()));
    noter(&mut app, "c2", "écrit pendant la coupure");
    image_suivante(&mut app);
    app.suivre_le_canevas_partage();
    assert_eq!(app.ui.synchro, Some(Synchro::Coupee(1)));
    app.quitter_le_canevas_partage();
    app.suivre_le_canevas_partage();
    assert_eq!(app.ui.synchro, None);
}

/// Sous un code qui ne fait que regarder, ses gestes ne partent jamais, même relié : le témoin le
/// dit, au lieu de « synchro ».
#[test]
fn synchro_un_code_qui_regarde_se_dit() {
    let d = dossier("canevas-temoin-lecteur");
    let (fil, _salle) = lien::d_epreuve();
    let lecteur =
        Code::lire_le_texte(&Code::neuf([7; 32]).texte_lire()).expect("un code de lecture");
    let (mut app, _, _) = document_partage_par(&d, fil, lecteur);
    app.suivre_le_canevas_partage();
    assert_eq!(app.ui.synchro, Some(crate::ui::Synchro::Regarde));
}

/// **SYNCHRO-2** : Glucose revient au premier plan (Android) — le lien du canevas partagé est
/// éprouvé tout de suite, pour que le témoin dise vrai dès qu'il écrit.
#[test]
fn synchro_2_au_retour_le_lien_est_eprouve() {
    use crate::app::vie::tests::Temoin;
    let d = dossier("canevas-retour");
    let (fil, _salle) = lien::d_epreuve();
    let (mut app, _, _) = document_partage_par(&d, fil, Code::neuf([7; 32]));
    app.presenter = Some(Box::new(Temoin(Arc::new(Default::default()), false)));
    app.reprendre();
    let cp = app.lancement.agents.partage.as_ref().expect("partagé");
    assert!(cp.lien.presse(), "le lien n'a pas été éprouvé au retour");
}
