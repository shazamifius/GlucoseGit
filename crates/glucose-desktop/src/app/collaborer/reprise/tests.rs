//! **PARTAGE-SURVIT** (fiche 131), par la vraie application : partager, fermer par la croix,
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

/// **Fermé proprement, relancé : il rejoint sa salle au numéro que son fichier contient** — et
/// le souvenir redit « en cours ».
#[test]
fn partage_survit_ferme_proprement_il_rejoint_sa_salle_au_bon_numero() {
    let d = dossier("canevas-survit");
    let (mut app, chemin, code) = document_partage(&d);
    arrive(&mut app, 2, "b0b-1");
    image_suivante(&mut app);
    assert_eq!(
        souvenir(&d, &chemin).and_then(|s| s.confirme),
        None,
        "en cours"
    );
    assert!(fermer(&mut app), "une fermeture propre");
    let attendu = Souvenir {
        code: code.texte_ecrire().unwrap(),
        confirme: Some(2),
    };
    assert_eq!(souvenir(&d, &chemin), Some(attendu));
    drop(app);

    let mut relance = lancer(&d);
    relance.retrouver_le_travail(None);
    relance.reprendre_le_canevas_partage();
    let cp = relance
        .lancement
        .agents
        .partage
        .as_ref()
        .expect("le partage reprend");
    assert_eq!(cp.entree, Entree::Dedans);
    assert!(cp.code.peut_ecrire());
    assert_ne!(cp.marque, "a1b2", "une marque neuve");
    assert_eq!(relance.store.partage.as_ref().map(|p| p.confirme), Some(2));
    assert_eq!(
        souvenir(&d, &chemin).and_then(|s| s.confirme),
        None,
        "de nouveau en cours"
    );
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
    assert_eq!(souvenir(&d, &chemin).and_then(|s| s.confirme), Some(1));
}

/// **Un de mes gestes attend encore son numéro** à la fermeture : il est dans le fichier, pas
/// dans la salle — rien n'est retenu d'exact, et le lancement suivant ne reprend pas tout seul.
/// Il le dit.
#[test]
fn partage_survit_un_geste_en_attente_ne_laisse_pas_de_numero() {
    let d = dossier("canevas-attente");
    let (mut app, chemin, _) = document_partage(&d);
    noter(&mut app, "c2", "parti, pas revenu");
    image_suivante(&mut app);
    assert_eq!(app.store.partage.as_ref().map(|p| p.en_attente()), Some(1));
    assert!(fermer(&mut app));
    assert_eq!(souvenir(&d, &chemin).and_then(|s| s.confirme), None);
    drop(app);

    let mut relance = lancer(&d);
    relance.retrouver_le_travail(None);
    relance.reprendre_le_canevas_partage();
    assert!(
        relance.lancement.agents.partage.is_none(),
        "pas de reprise à l'aveugle"
    );
    let dit = relance.ui.current_toast.as_ref().map(|t| t.message.clone());
    assert_eq!(dit.as_deref(), Some(ARRETE_NET));
    assert_eq!(souvenir(&d, &chemin), None, "dit une fois");
}

/// **Un arrêt net** (aucune fermeture) : le souvenir est resté « en cours » — pas de reprise.
#[test]
fn partage_survit_un_arret_net_ne_reprend_pas() {
    let d = dossier("canevas-net");
    let (app, _chemin, _) = document_partage(&d);
    drop(app);
    let mut relance = lancer(&d);
    relance.retrouver_le_travail(None);
    relance.reprendre_le_canevas_partage();
    assert!(relance.lancement.agents.partage.is_none());
}

/// **« Quitter le partage » l'oublie** : relancé, le document reste à soi seul.
#[test]
fn partage_survit_quitter_le_partage_l_oublie() {
    let d = dossier("canevas-quitter");
    let (mut app, chemin, _) = document_partage(&d);
    app.repondre_a_collaborer(QUITTER);
    assert!(app.lancement.agents.partage.is_none());
    assert!(fermer(&mut app));
    assert_eq!(souvenir(&d, &chemin), None);
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

/// **Fermer attend le retour de mes gestes** (fiche 144 § 2) : un geste parti juste avant la
/// fermeture revient numéroté pendant qu'elle attend, derrière un geste de Bob — le numéro est
/// retenu, le geste de Bob est dans le fichier, et le document relancé rejoindra sa salle au lieu
/// de s'arrêter « net ».
#[test]
fn partage_survit_fermer_attend_le_retour_de_mes_gestes() {
    let d = dossier("canevas-retour");
    let (fil, salle) = lien::d_epreuve();
    let (mut app, chemin, _) = document_partage_par(&d, fil, Code::neuf([7; 32]));
    noter(&mut app, "c2", "parti juste avant la fermeture");
    image_suivante(&mut app);
    assert_eq!(app.store.partage.as_ref().map(|p| p.en_attente()), Some(1));
    let lettre = |marque: &str, nom: &str, geste| {
        Box::new(glucose_core::salle::lettre::Lettre {
            marque: marque.into(),
            nom: nom.into(),
            couleur: 0,
            main: Main::Humain,
            corps: glucose_core::salle::lettre::Corps::Geste(geste),
        })
    };
    let de_bob = geste_de_bob(&app.store.project, "b0b-1");
    let arrivees = [
        (2, "b0b-1", lettre("b0b", "Bob", de_bob)),
        (
            3,
            "a1b2-2",
            lettre("a1b2", "Alice", GestePartage::default()),
        ),
    ];
    for (n, id, lettre) in arrivees {
        let nouvelle = lien::Nouvelle::Lettre {
            n,
            id: id.into(),
            lettre,
        };
        salle.send(nouvelle).expect("la file");
    }
    assert!(fermer(&mut app));
    assert_eq!(souvenir(&d, &chemin).and_then(|s| s.confirme), Some(3));
    drop(app);

    let mut relance = lancer(&d);
    relance.retrouver_le_travail(None);
    let board = relance.store.project.active_board_id.clone();
    assert!(
        relance.store.project.annotation(&board, "b0b-1").is_some(),
        "le geste de Bob, écrit avant le numéro"
    );
}

/// **La loi de l'attente** : seulement des gestes en attente, une salle reliée, au présent, et
/// pas au-delà du silence.
#[test]
fn partage_survit_l_attente_s_arrete_quand_rien_ne_peut_revenir() {
    let tot = Duration::from_millis(1);
    assert!(attendre(1, true, false, tot));
    assert!(!attendre(0, true, false, tot), "rien n'attend");
    assert!(!attendre(1, false, false, tot), "pas reliée");
    assert!(!attendre(1, true, true, tot), "dans le passé");
    assert!(!attendre(1, true, false, SILENCE), "silencieuse");
}

/// **Seul le scribe reprend le partage** (fiche 144 § 2) : une seconde fenêtre ouvre le document
/// que la première écrit — elle le lit sans le verrou, ne rejoint pas la salle, et laisse le
/// numéro retenu tel quel.
#[test]
fn partage_survit_une_seconde_fenetre_ne_reprend_pas_la_salle() {
    let d = dossier("canevas-seconde");
    let (mut app, chemin, _) = document_partage(&d);
    assert!(fermer(&mut app));
    drop(app);
    let retenu = souvenir(&d, &chemin);
    assert_eq!(retenu.as_ref().and_then(|s| s.confirme), Some(1));

    let mut scribe = lancer(&d);
    scribe.retrouver_le_travail(None);
    let mut seconde = lancer(&d);
    seconde.retrouver_le_travail(None);
    seconde.reprendre_le_canevas_partage();
    assert!(
        seconde.lancement.agents.partage.is_none(),
        "la seconde fenêtre a rejoint la salle"
    );
    assert_eq!(souvenir(&d, &chemin), retenu, "le numéro du scribe, intact");
    scribe.reprendre_le_canevas_partage();
    assert!(
        scribe.lancement.agents.partage.is_some(),
        "le scribe, lui, reprend"
    );
}
