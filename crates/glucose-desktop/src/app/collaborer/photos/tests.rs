//! Les photos d'un participant à l'autre (B1) : le facteur, le bonjour, la relance, la réponse.

use super::super::{lien, CanevasPartage, Entree};
use super::*;
use crate::persist::objets::Promesse;
use glucose_core::salle::code::Code;
use glucose_core::types::BoardImage;
use std::time::Duration;

const CLE: &str = "depot:1-0-chat.png";
/// Assez pour qu'un facteur qui devait écrire l'ait fait.
const ASSEZ: Duration = Duration::from_millis(300);
/// Le plus qu'on attend une lettre qui doit venir.
const AU_PLUS: Duration = Duration::from_secs(5);

fn qui() -> Qui {
    ("b0b".into(), "Bob".into(), 0)
}

/// Une application où la photo `CLE` est posée, sans ses octets.
fn app_sans_les_octets() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new("chat", 0.0, 0.0, 10.0, 10.0);
    img.src = Some(CLE.into());
    app.store.add_image(&board, img);
    app
}

/// Le partage de ce code, relié à une salle qui ne répond pas ; le facteur écrit dans la file rendue.
fn partage(app: &mut GlucoseApp, code: Code, entree: Entree) -> mpsc::Receiver<Passe> {
    let lien = lien::lancer(
        "http://127.0.0.1:9",
        code.clone(),
        0,
        crate::agents::Boite::nouvelle(Arc::new(|| {})),
    );
    let mut cp = CanevasPartage::neuf(lien, code, qui(), entree, Arc::clone(&app.disque.objets));
    let (expediteur, ordres) = mpsc::sync_channel(8);
    cp.photos = Photos::lancer(Arc::clone(&app.disque.objets), expediteur, qui());
    app.lancement.agents.partage = Some(cp);
    ordres
}

fn ecrivain() -> Code {
    Code::neuf([7; 32])
}

/// Ce que le facteur a confié au fil : la lettre lue, et si c'est une photo.
fn lettre_partie(passes: &mpsc::Receiver<Passe>, attente: Duration) -> Option<(Corps, bool)> {
    let Passe { clair, photo, .. } = passes.recv_timeout(attente).ok()?;
    let lettre = Lettre::lire(&clair).expect("une lettre qui se lit");
    Some((lettre.corps, photo))
}

fn demande(cles: &[&str], bonjour: bool) -> Corps {
    Corps::Demande {
        cles: cles.iter().map(|c| c.to_string()).collect(),
        bonjour,
    }
}

/// Une lettre d'Alice.
fn d_alice(corps: Corps) -> Lettre {
    Lettre {
        marque: "a1ce".into(),
        nom: "Alice".into(),
        couleur: 1,
        main: Main::Humain,
        corps,
    }
}

/// **Une photo manque** quand ni le registre ni un fichier ne dit où lire ses octets ; chaque
/// clé ne se compte qu'une fois.
#[test]
fn b1_ce_qui_manque_se_reconnait() {
    let mut app = app_sans_les_octets();
    let board = app.store.project.active_board_id.clone();
    let mut double = BoardImage::new("chat-2", 50.0, 0.0, 10.0, 10.0);
    double.src = Some(CLE.into());
    app.store.add_image(&board, double);
    assert_eq!(
        cles_qui_manquent(&app.store.project, &app.disque.objets),
        vec![CLE.to_string()]
    );
    app.disque
        .objets
        .poser(CLE, Source::Memoire(Arc::new(vec![1, 2, 3])));
    assert!(cles_qui_manquent(&app.store.project, &app.disque.objets).is_empty());
    let fichier = std::env::temp_dir().join(format!("glucose-b1-{}.png", std::process::id()));
    std::fs::write(&fichier, [0u8]).expect("un fichier");
    assert!(
        !manque(&app.disque.objets, &fichier.to_string_lossy()),
        "un fichier se lit"
    );
    let _ = std::fs::remove_file(fichier);
}

/// **Les octets arrivés entrent au registre et sortent des échecs** — sans quoi l'atelier ne
/// retenterait jamais la photo ; et ceux qui arrivent pour une photo qu'on a ne changent rien.
#[test]
fn b1_les_octets_arrives_entrent_au_registre_et_sortent_des_echecs() {
    let mut app = app_sans_les_octets();
    app.renderer.magasin.echecs.insert(CLE.to_string());
    app.une_lettre_qui_passe(d_alice(Corps::Objet {
        cle: CLE.into(),
        octets: vec![1, 2, 3],
    }));
    assert_eq!(
        app.disque.objets.source(CLE),
        Some(Source::Memoire(Arc::new(vec![1, 2, 3])))
    );
    assert!(
        !app.renderer.magasin.echecs.contains(CLE),
        "l'atelier la retentera"
    );
    app.recevoir_la_photo(CLE.into(), vec![9]);
    assert_eq!(
        app.disque.objets.source(CLE),
        Some(Source::Memoire(Arc::new(vec![1, 2, 3]))),
        "la première reste"
    );
}

/// **Le facteur attend les octets promis** : une photo collée part avec son geste avant d'être
/// encodée — ses octets suivent dès qu'ils sont là.
#[test]
fn b1_le_facteur_attend_les_octets_promis() {
    let objets = Objets::nouveau();
    let (promesse, parole) = Promesse::nouvelle();
    objets.poser(CLE, Source::Promise(promesse));
    let (expediteur, ordres) = mpsc::sync_channel(8);
    let photos = Photos::lancer(Arc::clone(&objets), expediteur, qui());
    photos.porter(CLE.into());
    assert!(
        lettre_partie(&ordres, ASSEZ).is_none(),
        "rien avant les octets"
    );
    parole.tenir(vec![4, 5, 6]);
    let partie = lettre_partie(&ordres, AU_PLUS).expect("la photo part");
    assert_eq!(
        partie,
        (
            Corps::Objet {
                cle: CLE.into(),
                octets: vec![4, 5, 6],
            },
            true
        )
    );
}

/// **Le facteur ne lit que le registre** : une clé qui nomme un fichier (`~/.ssh/…`) ne lui fait
/// rien lire ; une promesse abandonnée non plus — et il continue sa tournée.
#[test]
fn b1_le_facteur_ne_lit_que_le_registre() {
    let fichier = std::env::temp_dir().join(format!("glucose-b1-secret-{}", std::process::id()));
    std::fs::write(&fichier, b"secret").expect("un fichier");
    let objets = Objets::nouveau();
    let (promesse, parole) = Promesse::nouvelle();
    objets.poser("collee:1", Source::Promise(promesse));
    drop(parole);
    let (expediteur, ordres) = mpsc::sync_channel(8);
    let photos = Photos::lancer(objets, expediteur, qui());
    photos.porter(fichier.to_string_lossy().into_owned());
    photos.porter("collee:1".into());
    let _ = photos.facteur.send(Course::Demande {
        cles: vec![CLE.into()],
        bonjour: false,
    });
    assert_eq!(
        lettre_partie(&ordres, AU_PLUS),
        Some((demande(&[CLE], false), false)),
        "seule la demande est partie"
    );
    let _ = std::fs::remove_file(fichier);
}

/// **Le bonjour part à l'« à jour » de la salle**, avec ce qui manque — une fois ; **un bonjour
/// reçu fait redemander** ce qui manque encore, sans bonjour : une relance n'en appelle pas
/// d'autre.
#[test]
fn b1_le_bonjour_puis_la_relance() {
    let mut app = app_sans_les_octets();
    let ordres = partage(&mut app, ecrivain(), Entree::Dedans);
    app.salle_a_jour(3);
    app.demander_ce_qui_manque();
    assert_eq!(
        lettre_partie(&ordres, AU_PLUS),
        Some((demande(&[CLE], true), false))
    );
    app.demander_ce_qui_manque();
    assert_eq!(lettre_partie(&ordres, ASSEZ), None, "une fois");

    app.une_lettre_qui_passe(d_alice(demande(&[], true)));
    app.demander_ce_qui_manque();
    assert_eq!(
        lettre_partie(&ordres, AU_PLUS),
        Some((demande(&[CLE], false), false)),
        "Alice entre : on lui redemande"
    );
    app.une_lettre_qui_passe(d_alice(demande(&[], false)));
    app.demander_ce_qui_manque();
    assert_eq!(
        lettre_partie(&ordres, ASSEZ),
        None,
        "une relance n'en appelle pas d'autre"
    );

    app.recevoir_la_photo(CLE.into(), vec![1]);
    app.une_lettre_qui_passe(d_alice(demande(&[], true)));
    app.demander_ce_qui_manque();
    assert_eq!(lettre_partie(&ordres, ASSEZ), None, "plus rien ne manque");
    app.salle_a_jour(4);
    app.demander_ce_qui_manque();
    assert_eq!(
        lettre_partie(&ordres, AU_PLUS),
        Some((demande(&[], true), false)),
        "un bonjour se dit même sans manque"
    );
}

/// **Qui attend la base garde son bonjour** : il part une fois dedans. Celui qui a ouvert le
/// partage — qui ne passe jamais « dedans » — le dit aussi : son réseau revient comme celui des
/// autres, et ce qu'on a posé pendant son absence lui manque.
#[test]
fn b1_le_bonjour_attend_d_etre_dedans() {
    let mut ouvreur = app_sans_les_octets();
    let ordres = partage(&mut ouvreur, ecrivain(), Entree::Partage);
    ouvreur.salle_a_jour(3);
    ouvreur.demander_ce_qui_manque();
    assert_eq!(
        lettre_partie(&ordres, AU_PLUS),
        Some((demande(&[CLE], true), false)),
        "celui qui partage dit bonjour"
    );

    let mut app = app_sans_les_octets();
    let ordres = partage(&mut app, ecrivain(), Entree::AttendLaBase);
    app.salle_a_jour(3);
    app.demander_ce_qui_manque();
    assert_eq!(lettre_partie(&ordres, ASSEZ), None, "rien avant la base");
    if let Some(cp) = app.lancement.agents.partage.as_mut() {
        cp.entree = Entree::Dedans;
    }
    app.demander_ce_qui_manque();
    assert_eq!(
        lettre_partie(&ordres, AU_PLUS),
        Some((demande(&[CLE], true), false))
    );
}

/// **On répond à une demande si l'on écrit** — ce qu'on a au registre ; un lecteur ne répond pas :
/// la salle refuserait ses octets.
#[test]
fn b1_on_repond_si_l_on_ecrit() {
    for (code, repond) in [
        (ecrivain(), true),
        (
            Code::lire_le_texte(&ecrivain().texte_lire()).expect("un code lecteur"),
            false,
        ),
    ] {
        let mut app = GlucoseApp::new();
        app.disque
            .objets
            .poser(CLE, Source::Memoire(Arc::new(vec![4, 5])));
        let ordres = partage(&mut app, code, Entree::Dedans);
        app.une_lettre_qui_passe(d_alice(demande(&[CLE], false)));
        let photo = Corps::Objet {
            cle: CLE.into(),
            octets: vec![4, 5],
        };
        let attente = if repond { AU_PLUS } else { ASSEZ };
        assert_eq!(
            lettre_partie(&ordres, attente),
            repond.then_some((photo, true)),
            "écrivain : {repond}"
        );
    }
}

/// **Ce que je pose part avec mon geste** : le geste d'abord, puis les octets de sa photo.
#[test]
fn b1_ma_photo_part_avec_mon_geste() {
    let mut app = GlucoseApp::new();
    app.disque
        .objets
        .poser(CLE, Source::Memoire(Arc::new(vec![7])));
    let ordres = partage(&mut app, ecrivain(), Entree::Partage);
    app.store.partager_ce_canevas("b0b");
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new("chat", 0.0, 0.0, 10.0, 10.0);
    img.src = Some(CLE.into());
    app.store.add_image(&board, img);
    app.store.partager_les_gestes();
    app.faire_partir();
    assert_eq!(
        lettre_partie(&ordres, AU_PLUS),
        Some((
            Corps::Objet {
                cle: CLE.into(),
                octets: vec![7],
            },
            true
        ))
    );
}

/// **Q18 — Collaborer dit combien de photos attendent quelqu'un qui les a**, et ne le dit plus
/// quand leurs octets sont là.
#[test]
fn q18_collaborer_compte_les_photos_qui_attendent() {
    let mut app = app_sans_les_octets();
    let _ordres = partage(&mut app, ecrivain(), Entree::Dedans);
    let texte = |app: &mut GlucoseApp| {
        app.montrer_collaborer();
        let question = app.ui.question.take().map(|(q, _)| q.texte);
        question.expect("la question de Collaborer")
    };
    assert!(
        texte(&mut app).contains("1 photo attend quelqu'un qui l'a"),
        "le cadre gris se dit"
    );
    app.disque
        .objets
        .poser(CLE, Source::Memoire(Arc::new(vec![1, 2, 3])));
    assert!(!texte(&mut app).contains("attend"), "ses octets sont là");
}
