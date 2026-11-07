//! Le panneau, quand un domaine vit aussi dans d'autres fichiers : joué sur le vrai `GlucoseApp`, par
//! les intentions que le panneau produit.

use super::*;
use crate::dock::domains::{hit_domains_panel, layout_domains_panel, Cible, DomainIntent};
use crate::epreuve::Dossier;
use crate::params::{Pointer, ScaledRect};
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Domain};

const CADRE: ScaledRect = ScaledRect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 430.0,
    scale: 1.0,
};

fn app_au_panneau() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.dock_manager.top_tabs = vec![TabId::Domains];
    let board = app.store.project.active_board_id.clone();
    app.store.add_annotation(
        &board,
        Annotation::text("n", 500.0, 500.0, "Un nœud d'ici."),
    );
    app.store.clear_selection();
    app.store.select_annotation("n".to_string(), false);
    app.store.journal.clear();
    app
}

/// Un autre document, où un nœud porte ce domaine ; rend sa définition et le document.
fn un_autre_document(nom: &str, naissance: i64, noeud: &str) -> (Domain, Store) {
    let mut s = Store::new("Autre");
    let board = s.project.active_board_id.clone();
    let texte = format!("Le nœud {noeud}.");
    s.add_annotation(&board, Annotation::text(noeud, 0.0, 0.0, texte));
    let id = s
        .creer_un_domaine(nom, ("#123456".into(), "AUT".into()), naissance)
        .expect("créé");
    s.try_assign_domain_to_node(&board, noeud, &id, 0.9)
        .expect("porté");
    (s.domain(&id).expect("là").clone(), s)
}

fn recense(app: &mut GlucoseApp, chemin: &str, s: &Store) {
    app.disque
        .recenseur
        .etat
        .poser(recenser(chemin, &s.project));
    app.dock_manager.domains.connus = app.domaines_connus();
}

/// Écrit un nom au clavier, dans la saisie ouverte, et valide.
fn taper_le_nom(app: &mut GlucoseApp, nom: &str) {
    for lettre in nom.chars() {
        let lettre = lettre.to_string();
        assert!(app
            .handle_domain_rename_input(&winit::keyboard::Key::Character(lettre.as_str().into())));
    }
    assert!(app.handle_domain_rename_input(&winit::keyboard::Key::Named(
        winit::keyboard::NamedKey::Enter
    )));
}

fn effacer(app: &mut GlucoseApp, n: usize) {
    for _ in 0..n {
        app.handle_domain_rename_input(&winit::keyboard::Key::Named(
            winit::keyboard::NamedKey::Backspace,
        ));
    }
}

/// **DOM-4, sa règle, à la main** : renommer vers un nom pris est refusé, la saisie reste ouverte avec
/// ce qu'on a écrit, et la question est posée. Il n'y a plus qu'à préciser le mot : son exemple même.
/// Changer seulement sa propre casse reste permis.
#[test]
fn test_dom_4_on_n_a_pas_deux_domaines_du_meme_nom() {
    let mut app = app_au_panneau();
    app.apply_domain_intent(DomainIntent::Create);
    effacer(&mut app, 20);
    taper_le_nom(&mut app, "Python");
    app.apply_domain_intent(DomainIntent::Create);
    let second = app.store.project.domains[1].id.clone();
    let profondeur = app.store.undo_depth();

    effacer(&mut app, 20);
    taper_le_nom(&mut app, "python");
    assert_eq!(
        app.store.domain(&second).unwrap().name,
        "Domaine 1",
        "refusé"
    );
    assert_eq!(app.store.undo_depth(), profondeur, "aucun geste");
    let saisie = app
        .dock_manager
        .domains
        .rename
        .as_ref()
        .expect("elle reste ouverte");
    assert_eq!(saisie.entry.text(), "python", "ce qu'on a écrit reste");
    let toast = app.ui.current_toast.as_ref().expect("la question");
    assert!(
        toast.message.contains("même sujet") && toast.message.contains("précise"),
        "{}",
        toast.message
    );

    taper_le_nom(&mut app, " serpent");
    assert_eq!(app.store.domain(&second).unwrap().name, "python serpent");
    assert!(app.dock_manager.domains.rename.is_none());

    app.apply_domain_intent(DomainIntent::StartRename(second.clone()));
    effacer(&mut app, 20);
    taper_le_nom(&mut app, "PYTHON SERPENT");
    assert_eq!(
        app.store.domain(&second).unwrap().name,
        "PYTHON SERPENT",
        "sa casse"
    );
}

/// Le domaine connu d'ailleurs : une rangée sous ceux du document. Son palier le pose sur la
/// sélection : un seul geste entre et assigne, un seul `Ctrl+Z` défait tout. Sa couleur et son nom ne
/// se changent pas d'ici : ils vivent dans l'autre fichier.
#[test]
fn un_domaine_d_ailleurs_se_pose_d_un_seul_geste() {
    let mut app = app_au_panneau();
    let (venu, autre) = un_autre_document("Eau", 7, "x");
    recense(&mut app, "/ailleurs.glucose", &autre);

    let ui = &app.dock_manager.domains;
    let layout = layout_domains_panel(CADRE, &app.store, ui);
    assert_eq!(layout.rows.len(), 1, "une rangée pour lui");
    let rangee = &layout.rows[0];
    assert_eq!(rangee.cible, Cible::Ailleurs(venu.id.clone()));
    let clic = |x: f32, y: f32| Pointer { x, y };
    let centre = |r: crate::dock::WidgetRect| clic(r.x + r.w / 2.0, r.y + r.h / 2.0);
    assert_eq!(
        hit_domains_panel(&layout, &app.store, centre(rangee.swatch), true),
        None
    );
    assert_eq!(
        hit_domains_panel(&layout, &app.store, centre(rangee.name), true),
        None
    );
    assert_eq!(
        hit_domains_panel(&layout, &app.store, centre(rangee.delete), true),
        None
    );
    assert_eq!(
        hit_domains_panel(&layout, &app.store, centre(rangee.unassign), true),
        None,
        "rien ici ne le porte : pas de retrait"
    );
    assert_eq!(
        hit_domains_panel(&layout, &app.store, centre(rangee.steps[0]), false),
        None,
        "sans sélection, un palier ne fait rien"
    );
    let Some(intent) = hit_domains_panel(&layout, &app.store, centre(rangee.steps[4]), true) else {
        panic!("le palier devait répondre");
    };

    app.apply_domain_intent(intent);
    assert_eq!(
        app.store.project.domains,
        std::slice::from_ref(&venu),
        "entière, sous son identité"
    );
    let board = app.store.project.active_board_id.clone();
    let porte = app.store.node_domains(&board, "n").unwrap();
    assert_eq!(
        (porte[0].domain_id.as_str(), porte[0].weight),
        (venu.id.as_str(), 1.0)
    );
    assert_eq!(app.store.undo_depth(), 1, "un seul geste");

    app.dock_manager.domains.connus = app.domaines_connus();
    assert!(
        app.dock_manager.domains.connus.ailleurs.is_empty(),
        "il est ici"
    );
    assert_eq!(
        app.dock_manager.domains.connus.aussi,
        [(venu.id.clone(), 1, 1)]
    );

    app.store.undo();
    assert!(app.store.project.domains.is_empty());
    assert!(app.store.node_domains(&board, "n").unwrap().is_empty());
}

/// **Sa règle, face à un autre fichier** : un nom qui existe ailleurs ne se recrée pas ici.
#[test]
fn un_nom_pris_dans_un_autre_fichier_est_refuse_et_demande() {
    let mut app = app_au_panneau();
    let (_, autre) = un_autre_document("Python", 7, "x");
    recense(&mut app, "/ailleurs.glucose", &autre);
    app.apply_domain_intent(DomainIntent::Create);
    effacer(&mut app, 20);
    for lettre in "python".chars() {
        app.handle_domain_rename_input(&winit::keyboard::Key::Character(
            lettre.to_string().as_str().into(),
        ));
    }
    app.commit_domain_rename();
    assert_eq!(app.store.project.domains[0].name, "Domaine 1", "refusé");
    assert!(
        app.dock_manager.domains.rename.is_some(),
        "la saisie reste ouverte"
    );
    let toast = app.ui.current_toast.as_ref().expect("la question");
    assert!(toast.message.contains("Python") && toast.message.contains("précise"));
    assert!(
        toast.message.contains("1 nœud(s), 1 fichier(s)"),
        "{}",
        toast.message
    );
}

/// **Deux créations séparées se rencontrent** : la rangée pose la question à la place des paliers, et
/// l'homonyme n'a pas de rangée à lui. Tant qu'on n'a pas répondu, aucun palier ne répond.
#[test]
fn un_homonyme_remplace_les_paliers_par_la_question() {
    let mut app = app_au_panneau();
    let ici = app
        .store
        .creer_un_domaine("Python", ("#000000".into(), "P".into()), 1)
        .unwrap();
    let (_, autre) = un_autre_document("python", 7, "x");
    recense(&mut app, "/serpent.glucose", &autre);

    let ui = &app.dock_manager.domains;
    assert_eq!(ui.connus.homonymes, std::slice::from_ref(&ici));
    assert!(ui.connus.ailleurs.is_empty(), "sa rangée pose la question");
    let layout = layout_domains_panel(CADRE, &app.store, ui);
    assert_eq!(layout.rows.len(), 1);
    let rangee = &layout.rows[0];
    let (oui, non) = rangee.question.expect("la question");
    let centre = |r: crate::dock::WidgetRect| Pointer {
        x: r.x + r.w / 2.0,
        y: r.y + r.h / 2.0,
    };
    assert_eq!(
        hit_domains_panel(&layout, &app.store, centre(oui), true),
        Some(DomainIntent::MemeSujet(ici.clone()))
    );
    assert_eq!(
        hit_domains_panel(&layout, &app.store, centre(non), true),
        Some(DomainIntent::Preciser(ici.clone()))
    );
    let premier = rangee.steps[0];
    assert_eq!(
        hit_domains_panel(&layout, &app.store, centre(premier), true),
        None,
        "aucun palier tant qu'on n'a pas répondu"
    );
}

/// **« Oui » : même sujet.** Le domaine rejoint l'autre, la question disparaît.
#[test]
fn oui_reunit_les_deux() {
    let mut app = app_au_panneau();
    let board = app.store.project.active_board_id.clone();
    let ici = app
        .store
        .creer_un_domaine("Eau", ("#000000".into(), "E".into()), 1)
        .unwrap();
    app.store
        .try_assign_domain_to_node(&board, "n", &ici, 0.5)
        .unwrap();
    let (venu, autre) = un_autre_document("eau", 7, "x");
    recense(&mut app, "/ailleurs.glucose", &autre);

    app.apply_domain_intent(DomainIntent::MemeSujet(ici.clone()));
    assert_eq!(app.store.project.domains[0].id, venu.id);
    assert_eq!(
        app.store.node_domains(&board, "n").unwrap()[0].domain_id,
        venu.id
    );
    let connus = app.domaines_connus();
    assert!(connus.homonymes.is_empty());
    assert_eq!(connus.aussi, [(venu.id, 1, 1)]);
}

/// **« Non » : préciser le mot.** La saisie s'ouvre sur son nom ; le mot précisé fait disparaître la
/// question ; le même nom reste refusé.
#[test]
fn non_ouvre_le_nom_a_preciser() {
    let mut app = app_au_panneau();
    let ici = app
        .store
        .creer_un_domaine("Python", ("#000000".into(), "P".into()), 1)
        .unwrap();
    let (_, autre) = un_autre_document("python", 7, "x");
    recense(&mut app, "/serpent.glucose", &autre);

    app.apply_domain_intent(DomainIntent::Preciser(ici.clone()));
    let saisie = app.dock_manager.domains.rename.as_ref().expect("la saisie");
    assert_eq!(saisie.domain_id, ici);
    app.commit_domain_rename();
    assert_eq!(
        app.store.domain(&ici).unwrap().name,
        "Python",
        "le même nom : refusé"
    );
    assert!(app.dock_manager.domains.rename.is_some());

    for lettre in " code".chars() {
        app.handle_domain_rename_input(&winit::keyboard::Key::Character(
            lettre.to_string().as_str().into(),
        ));
    }
    app.commit_domain_rename();
    assert_eq!(app.store.domain(&ici).unwrap().name, "Python code");
    let connus = app.domaines_connus();
    assert!(connus.homonymes.is_empty(), "les deux noms sont distincts");
    assert_eq!(connus.ailleurs.len(), 1, "l'autre a sa rangée");
}

/// **Un nom changé dans un autre fichier s'apprend ici**, par la date la plus récente ; une définition
/// plus ancienne ne remplace rien.
#[test]
fn un_renommage_d_ailleurs_arrive_ici() {
    let mut app = app_au_panneau();
    let (venu, mut autre) = un_autre_document("Python", 7, "x");
    app.store.adopter_un_domaine(&venu).unwrap();
    let id = venu.id.clone();

    // La même définition : rien.
    app.disque
        .recenseur
        .etat
        .poser(recenser("/ailleurs.glucose", &autre.project));
    app.accorder_les_domaines();
    assert_eq!(app.store.domain(&id).unwrap().name, "Python");

    // Une définition plus ancienne que la nôtre, et différente : elle ne défait pas notre renommage.
    let ici = DomainPatch::new().with_name("Python serpent").at(50);
    app.store.try_update_domain(&id, ici).unwrap();
    app.accorder_les_domaines();
    assert_eq!(
        app.store.domain(&id).unwrap().name,
        "Python serpent",
        "la nôtre est plus récente"
    );

    let patch = DomainPatch::new()
        .with_name("Python code")
        .with_color("#654321")
        .at(99);
    autre.try_update_domain(&id, patch).unwrap();
    app.disque
        .recenseur
        .etat
        .poser(recenser("/ailleurs.glucose", &autre.project));
    app.accorder_les_domaines();
    let d = app.store.domain(&id).unwrap();
    assert_eq!(
        (d.name.as_str(), d.color.as_str(), d.defini_a),
        ("Python code", "#654321", 99)
    );
    assert!(app.store.can_undo(), "un geste, annulable");
    let profondeur = app.store.undo_depth();
    app.accorder_les_domaines();
    assert_eq!(app.store.undo_depth(), profondeur, "une seule fois");
}

/// **Pendant un geste de l'utilisateur, rien ne s'accorde** : son `Ctrl+Z` défairait les deux.
#[test]
fn rien_ne_s_accorde_pendant_un_geste() {
    let mut app = app_au_panneau();
    let (venu, mut autre) = un_autre_document("Python", 7, "x");
    app.store.adopter_un_domaine(&venu).unwrap();
    let patch = DomainPatch::new().with_name("Python code").at(99);
    autre.try_update_domain(&venu.id, patch).unwrap();
    app.disque
        .recenseur
        .etat
        .poser(recenser("/ailleurs.glucose", &autre.project));
    app.store.begin_live_edit();
    app.accorder_les_domaines();
    assert_eq!(app.store.domain(&venu.id).unwrap().name, "Python");
    app.store.end_live_edit();
}

/// **« Où ? »** : chaque place, de tous les fichiers, du plus lourd au plus léger ; la photo dit ce
/// qu'elle est ; « Retour » rend les rangées.
#[test]
fn ou_liste_les_places_de_tous_les_fichiers() {
    let mut app = app_au_panneau();
    let board = app.store.project.active_board_id.clone();
    let (venu, autre) = un_autre_document("Eau", 7, "x");
    let id = app.store.adopter_un_domaine(&venu).unwrap();
    app.store
        .try_assign_domain_to_node(&board, "n", &id, 0.5)
        .unwrap();
    app.store.add_image(
        &board,
        glucose_core::types::BoardImage::new("ph", 0.0, 0.0, 9.0, 9.0),
    );
    app.store
        .try_assign_domain_to_node(&board, "ph", &id, 0.2)
        .unwrap();
    recense(&mut app, "/ailleurs.glucose", &autre);

    app.apply_domain_intent(DomainIntent::Ou(venu.id.clone()));
    let ou = app.dock_manager.domains.ou.clone().expect("la liste");
    assert_eq!(ou.nom, "Eau");
    let places: Vec<_> = ou
        .lignes
        .iter()
        .map(|l| (l.noeud.as_str(), l.ici))
        .collect();
    assert_eq!(places, [("x", false), ("n", true), ("ph", true)]);
    assert_eq!(ou.lignes[0].chemin, "/ailleurs.glucose");
    assert_eq!(ou.lignes[0].texte, "Le nœud x.");
    assert_eq!(ou.lignes[2].texte, "photo");

    let layout = layout_domains_panel(CADRE, &app.store, &app.dock_manager.domains);
    let (premier, lignes) = layout.ou.as_ref().expect("les rangées de la liste");
    assert_eq!((*premier, lignes.len()), (0, 3));
    let r = lignes[1];
    let sur = Pointer {
        x: r.x + 1.0,
        y: r.y + 1.0,
    };
    assert_eq!(
        hit_domains_panel(&layout, &app.store, sur, true),
        Some(DomainIntent::Aller(1))
    );
    let b = layout.add_button;
    let sur = Pointer {
        x: b.x + 1.0,
        y: b.y + 1.0,
    };
    assert_eq!(
        hit_domains_panel(&layout, &app.store, sur, true),
        Some(DomainIntent::FermerOu)
    );
    assert!(layout.rows.is_empty(), "elle remplace les rangées");

    app.apply_domain_intent(DomainIntent::FermerOu);
    assert_eq!(app.dock_manager.domains.ou, None);
}

/// **Aller à une place ici** : la caméra vole, le document ne change pas, et la liste reste ouverte
/// pour la suivante. Sur un autre tableau du document, l'onglet suit.
#[test]
fn aller_a_une_place_ici_fait_voler_la_camera() {
    let mut app = app_au_panneau();
    let board = app.store.project.active_board_id.clone();
    let second = app.store.add_board("Second");
    app.store
        .add_annotation(&second, Annotation::text("loin", 9000.0, 9000.0, "Loin."));
    let id = app
        .store
        .creer_un_domaine("Eau", ("#000000".into(), "E".into()), 1)
        .unwrap();
    app.store
        .try_assign_domain_to_node(&second, "loin", &id, 1.0)
        .unwrap();
    let identite = app.store.identite.clone();

    app.apply_domain_intent(DomainIntent::Ou(id.clone()));
    assert_eq!(app.store.project.active_board_id, board);
    app.apply_domain_intent(DomainIntent::Aller(0));
    assert_eq!(app.store.identite, identite);
    assert_eq!(app.store.project.active_board_id, second, "l'onglet suit");
    assert!(app.vol.en_cours(), "la caméra vole");
    assert!(app.dock_manager.domains.ou.is_some());
}

/// **Aller à une place dans un autre fichier**, de bout en bout, sur un vrai fichier : il s'ouvre, et
/// la caméra vole vers le nœud ; rien ne se perd du document qu'on quitte.
#[test]
fn aller_a_une_place_ouvre_le_fichier_et_y_vole() {
    let dossier = Dossier::nouveau("aller-a-une-place");
    let chemin = dossier.join("ailleurs.glucose");
    let (venu, autre) = un_autre_document("Eau", 7, "x");
    let octets = glucose_core::persist::encode(
        &autre.project,
        &glucose_core::types::AssetStore::default(),
        0,
    );
    std::fs::write(&chemin, octets).expect("écrit");

    let mut app = app_au_panneau();
    let texte = chemin.display().to_string();
    recense(&mut app, &texte, &autre);
    app.apply_domain_intent(DomainIntent::Ou(venu.id.clone()));
    app.saved_version = app.store.version;
    app.apply_domain_intent(DomainIntent::Aller(0));
    assert_eq!(app.store.identite, texte, "le fichier s'est ouvert");
    assert_eq!(
        app.dock_manager.domains.ou, None,
        "un autre document : tout geste est fini"
    );
    assert!(app.disque.recenseur.destination.is_some());
    app.suivre_le_recensement();
    assert!(app.disque.recenseur.destination.is_none(), "arrivé");
    assert!(app.vol.en_cours(), "la caméra vole");
    assert!(app.store.domain(&venu.id).is_some(), "son catalogue est là");
}

/// **Un fichier qui ne s'ouvre pas** : la destination s'oublie, et elle ne prend pas de court une
/// ouverture de plus tard.
#[test]
fn une_destination_qui_n_arrive_pas_s_oublie() {
    let mut app = app_au_panneau();
    let dossier = Dossier::nouveau("destination-oubliee");
    let chemin = dossier.join("absent.glucose").display().to_string();
    let (venu, autre) = un_autre_document("Eau", 7, "x");
    recense(&mut app, &chemin, &autre);
    app.apply_domain_intent(DomainIntent::Ou(venu.id.clone()));
    app.saved_version = app.store.version;
    app.apply_domain_intent(DomainIntent::Aller(0));
    app.suivre_le_recensement();
    assert!(app.disque.recenseur.destination.is_none());
    assert!(!app.vol.en_cours());
}

/// **La molette fait défiler les rangées**, seulement sur le panneau, et s'arrête à la dernière
/// rangée montrée : remonter ne demande jamais de crans pour rien.
#[test]
fn la_molette_fait_defiler_les_rangees() {
    let mut app = app_au_panneau();
    assert!(
        !app.molette_sur_les_domaines(-1.0),
        "le pointeur n'y est pas"
    );
    let (l, h) = app.taille_de_la_fenetre();
    let ecran = app.screen_frame(l, h);
    let (_, cadre) = crate::dock::cadres_ouverts(&app.dock_manager, ecran)
        .into_iter()
        .find(|(t, _)| *t == TabId::Domains)
        .expect("ouvert");
    app.mouse_pos = (f64::from(cadre.x + 5.0), f64::from(cadre.y + 5.0));
    // Plus de rangées que le panneau n'en montre.
    for k in 0..12 {
        let (_, s) = un_autre_document(&format!("D{k}"), k, "x");
        recense(&mut app, &format!("/{k}"), &s);
    }
    let montrees = layout_domains_panel(cadre, &app.store, &app.dock_manager.domains)
        .rows
        .len();
    assert!(montrees < 12, "il faut défiler : {montrees} montrées");
    let caches = 12 - montrees;
    for _ in 0..30 {
        assert!(app.molette_sur_les_domaines(-1.0));
    }
    assert_eq!(
        app.dock_manager.domains.decalage, caches,
        "arrêtée à la dernière"
    );
    let layout = layout_domains_panel(cadre, &app.store, &app.dock_manager.domains);
    assert_eq!(
        layout.rows.last().map(|r| r.index),
        Some(11),
        "la dernière se voit"
    );
    app.molette_sur_les_domaines(1.0);
    assert_eq!(
        app.dock_manager.domains.decalage,
        caches - 1,
        "un cran remonte d'une"
    );
    for _ in 0..30 {
        app.molette_sur_les_domaines(1.0);
    }
    assert_eq!(app.dock_manager.domains.decalage, 0, "borné en haut");
    app.dock_manager.top_tabs.clear();
    assert!(
        !app.molette_sur_les_domaines(-1.0),
        "panneau fermé : le canevas la reprend"
    );
}

/// **Le panneau suit le document** : une image après un geste, il sait ce que ce geste a changé.
/// Fermé, il ne calcule rien ; et rien ne se recalcule sans changement.
#[test]
fn le_panneau_suit_le_catalogue() {
    let mut app = app_au_panneau();
    let (_, autre) = un_autre_document("Eau", 7, "x");
    app.disque
        .recenseur
        .etat
        .poser(recenser("/ailleurs.glucose", &autre.project));
    assert!(app.dock_manager.domains.connus.ailleurs.is_empty());
    app.suivre_le_recensement();
    assert_eq!(app.dock_manager.domains.connus.ailleurs.len(), 1);

    // Ce qui est écrit à la main ne peut être gardé que si rien ne recalcule sans raison.
    app.dock_manager.domains.connus.ailleurs.clear();
    app.suivre_le_recensement();
    assert!(
        app.dock_manager.domains.connus.ailleurs.is_empty(),
        "rien n'a changé"
    );

    app.dock_manager.top_tabs.clear();
    app.store
        .creer_un_domaine("Feu", ("#000000".into(), "F".into()), 1)
        .unwrap();
    app.suivre_le_recensement();
    assert!(
        app.dock_manager.domains.connus.ailleurs.is_empty(),
        "fermé : rien"
    );

    app.dock_manager.top_tabs = vec![TabId::Domains];
    app.suivre_le_recensement();
    assert_eq!(
        app.dock_manager.domains.connus.ailleurs.len(),
        1,
        "rouvert : à jour"
    );
}

/// **Un sigle tiré du nom suit le nom** : « Domaine 1 » montre « DOM », renommé « Eau » il montre
/// « EAU ». Un sigle choisi au clic reste, quel que soit le nom.
#[test]
fn le_sigle_tire_du_nom_suit_le_nom() {
    let mut app = app_au_panneau();
    app.apply_domain_intent(DomainIntent::Create);
    let id = app.store.project.domains[0].id.clone();
    assert_eq!(app.store.domain(&id).unwrap().icon, "DOM");
    effacer(&mut app, 20);
    taper_le_nom(&mut app, "Eau");
    assert_eq!(app.store.domain(&id).unwrap().icon, "EAU");

    app.apply_domain_intent(DomainIntent::CycleSigil(id.clone()));
    let choisi = app.store.domain(&id).unwrap().icon.clone();
    assert_ne!(choisi, "EAU");
    app.apply_domain_intent(DomainIntent::StartRename(id.clone()));
    effacer(&mut app, 20);
    taper_le_nom(&mut app, "Eau pure");
    assert_eq!(
        app.store.domain(&id).unwrap().icon,
        choisi,
        "choisi : il reste"
    );
}
