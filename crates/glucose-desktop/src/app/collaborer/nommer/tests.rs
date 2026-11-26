//! IDENTITE-2 et PASTILLES-1 — son nom et sa couleur, demandés une fois (Q19, fiche 140).

use super::super::{lien, CanevasPartage, Entree};
use super::*;
use crate::epreuve::Dossier;
use crate::interactions::question::tests::repondre;
use glucose_core::salle::code::Code;
use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};

/// Une application dont l'identité vit dans un dossier d'épreuve — jamais dans la sienne.
fn app_dans(d: &Dossier) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.disque.identite = d.join(super::super::identite::FICHIER);
    app
}

/// Touche la pastille `i` de la question posée, comme la main : appui puis relâchement dessus.
fn toucher_la_pastille(app: &mut GlucoseApp, i: usize) {
    let (question, _) = app.ui.question.clone().expect("une question");
    let (w, h) = app.taille_de_la_fenetre();
    let placee = crate::ui::question::placer(
        &question,
        &app.renderer.typography,
        (w, h),
        app.ui.scale_factor,
    );
    let (x, y, cw, ch) = placee.pastilles.expect("des pastilles").cases[i];
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(
        f64::from(x + cw / 2.0),
        f64::from(y + ch / 2.0),
    ));
    app.handle_mouse_down(MouseButton::Left, w, h);
    app.handle_mouse_up(MouseButton::Left);
}

fn ecrire_le_nom(app: &mut GlucoseApp, nom: &str) {
    let champ = app.ui.question.as_mut().and_then(|(q, _)| q.champ.as_mut());
    champ.expect("le champ du nom").texte = nom.into();
}

fn sienne(app: &GlucoseApp) -> Identite {
    let texte = std::fs::read_to_string(&app.disque.identite).expect("le fichier");
    Identite::lire(&texte).expect("une identité")
}

/// **La première fois, partager demande d'abord son nom et sa couleur** — le champ, les dix
/// pastilles — et rien ne se partage avant sa réponse. À l'envers : une identité qu'il a choisie
/// ne se redemande plus, « Rejoindre… » ouvre aussitôt le champ du code.
#[test]
fn identite_2_le_premier_partage_demande_d_abord_son_nom() {
    let d = Dossier::nouveau("identite-2-premier");
    let mut app = app_dans(&d);
    app.repondre_a_collaborer(super::super::PARTAGER);
    let Some((q, Suite::Identite(Some(Puis::Partager)))) = &app.ui.question else {
        panic!("la question du nom, avant le partage");
    };
    assert!(q.champ.is_some());
    assert_eq!(q.pastilles.as_ref().map(|p| p.couleurs.len()), Some(10));
    assert!(app.lancement.agents.partage.is_none(), "rien n'est partagé");
    let choisie = Identite {
        nom: "Shaza".into(),
        couleur: 4,
        choisie: true,
    };
    choisie.garder(&app.disque.identite);
    app.repondre_a_collaborer(super::super::REJOINDRE);
    assert!(
        matches!(app.ui.question, Some((_, Suite::Rejoindre))),
        "choisie, elle ne se redemande pas"
    );
}

/// **La pastille touchée et le nom écrit se gardent, choisis** — par la souris sur la pastille,
/// et « Garder » : le fichier les porte, et Collaborer revient. Les flèches gauche et droite
/// changent la couleur, pas la réponse en évidence.
#[test]
fn identite_2_la_pastille_touchee_et_le_nom_ecrit_se_gardent() {
    let d = Dossier::nouveau("identite-2-garder");
    let mut app = app_dans(&d);
    app.repondre_a_collaborer(super::super::NOMMER);
    toucher_la_pastille(&mut app, 4);
    // Dans le champ, la flèche seule déplace le curseur du nom ; avec Alt, elle change la couleur.
    let droite = Key::Named(NamedKey::ArrowRight);
    assert!(app.touche_de_la_question(&droite, true, None));
    let choisie = |app: &GlucoseApp| {
        app.ui
            .question
            .as_ref()?
            .0
            .pastilles
            .as_ref()
            .map(|p| p.choisie)
    };
    assert_eq!(choisie(&app), Some(4), "la flèche seule est au champ");
    app.modifiers = winit::keyboard::ModifiersState::ALT;
    assert!(app.touche_de_la_question(&droite, true, None));
    assert_eq!(choisie(&app), Some(5));
    let (q, _) = app.ui.question.as_ref().expect("la question");
    assert_eq!(q.focus, 0, "la réponse en évidence n'a pas bougé");
    let gauche = Key::Named(NamedKey::ArrowLeft);
    assert!(app.touche_de_la_question(&gauche, true, None));
    app.modifiers = winit::keyboard::ModifiersState::empty();
    ecrire_le_nom(&mut app, "  Shaza ");
    repondre(&mut app, Reponse::Oui);
    let attendue = Identite {
        nom: "Shaza".into(),
        couleur: 4,
        choisie: true,
    };
    assert_eq!(sienne(&app), attendue);
    assert!(matches!(app.ui.question, Some((_, Suite::Collaborer))));
}

/// **Un nom vide se redemande**, la couleur choisie gardée ; rien n'est retenu.
#[test]
fn identite_2_un_nom_vide_se_redemande() {
    let d = Dossier::nouveau("identite-2-vide");
    let mut app = app_dans(&d);
    app.repondre_a_collaborer(super::super::NOMMER);
    toucher_la_pastille(&mut app, 7);
    ecrire_le_nom(&mut app, "   ");
    repondre(&mut app, Reponse::Oui);
    let Some((q, Suite::Identite(None))) = &app.ui.question else {
        panic!("la question, encore");
    };
    assert!(q.texte.starts_with("Un nom vide"), "{}", q.texte);
    assert_eq!(q.pastilles.as_ref().map(|p| p.choisie), Some(7));
    assert!(!sienne(&app).choisie, "rien n'est retenu");
}

/// **Dans un canevas partagé** : la question dit qui a déjà la couleur touchée ; « Garder » la
/// donne aux lettres suivantes — elles lisent le nom et la couleur du partage.
#[test]
fn identite_2_dans_un_partage_la_couleur_prise_se_dit_et_le_nom_part() {
    let d = Dossier::nouveau("identite-2-partage");
    let mut app = app_dans(&d);
    let code = Code::neuf([9; 32]);
    let lien = lien::lancer(
        "http://127.0.0.1:9",
        code.clone(),
        0,
        crate::agents::Boite::nouvelle(std::sync::Arc::new(|| {})),
    );
    let qui = ("b0b".into(), "Bob".into(), 0);
    let objets = std::sync::Arc::clone(&app.disque.objets);
    let mut cp = CanevasPartage::neuf(lien, code, qui, Entree::Partage, objets);
    let loup = super::super::Pair {
        nom: "Loup Vert".into(),
        couleur: 6,
        curseur: None,
        regard: None,
        vu: std::time::Instant::now(),
    };
    cp.pairs.insert("l0up".into(), loup);
    app.lancement.agents.partage = Some(cp);
    app.repondre_a_collaborer(super::super::NOMMER);
    toucher_la_pastille(&mut app, 6);
    let (q, _) = app.ui.question.as_ref().expect("la question");
    assert!(
        q.texte.contains("« Loup Vert » a déjà cette couleur"),
        "{}",
        q.texte
    );
    toucher_la_pastille(&mut app, 2);
    let (q, _) = app.ui.question.as_ref().expect("la question");
    assert!(!q.texte.contains("a déjà cette couleur"), "{}", q.texte);
    ecrire_le_nom(&mut app, "Shaza");
    repondre(&mut app, Reponse::Oui);
    let cp = app.lancement.agents.partage.as_ref().expect("le partage");
    assert_eq!((cp.nom.as_str(), cp.couleur), ("Shaza", 2));
}
