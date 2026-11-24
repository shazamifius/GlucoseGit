//! REJOINDRE-1 par les vrais gestes : Collaborer, « Rejoindre… », le champ, la frappe, Entrée,
//! `Échap`.

use super::{DEJA_COPIE, PAS_UN_CODE};
use crate::app::GlucoseApp;
use crate::persist::close::Ensuite;
use crate::ui::question::Suite;
use glucose_core::salle::code::Code;
use glucose_core::types::Annotation;
use winit::keyboard::{Key, NamedKey};

/// Une application dont le document porte un texte : le quitter se demande d'abord (BROUILLON-1),
/// et cette question dit le code qu'on rejoindrait.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    let tableau = app.store.project.active_board_id.clone();
    let texte = Annotation::text("t", 0.0, 0.0, "à garder");
    app.store.add_annotation(&tableau, texte);
    app
}

/// Collaborer, puis « Rejoindre… », par la question elle-même.
fn ouvrir(app: &mut GlucoseApp) {
    app.montrer_collaborer();
    let rejoindre = app.ui.question.as_ref().and_then(|(q, _)| {
        let choix = q.choix.iter().find(|(l, _)| l == "Rejoindre…");
        choix.map(|(_, r)| *r)
    });
    app.repondre_a_la_question(Some(rejoindre.expect("« Rejoindre… » parmi les choix")));
    // La première fois, son nom et sa couleur d'abord (IDENTITE-2) : il garde ceux qu'on propose.
    if matches!(app.ui.question, Some((_, Suite::Identite(_)))) {
        app.repondre_a_la_question(Some(crate::ui::question::Reponse::Oui));
    }
}

fn champ(app: &GlucoseApp) -> Option<String> {
    let question = app.ui.question.as_ref();
    question.and_then(|(q, _)| q.champ.as_ref().map(|c| c.texte.clone()))
}

fn texte(app: &GlucoseApp) -> String {
    app.ui
        .question
        .as_ref()
        .map(|(q, _)| q.texte.clone())
        .unwrap_or_default()
}

fn taper(app: &mut GlucoseApp, texte: &str) {
    for c in texte.chars() {
        let t = c.to_string();
        assert!(app.prendre_la_touche(&Key::Character(t.clone().into()), true, Some(&t)));
    }
}

fn touche(app: &mut GlucoseApp, n: NamedKey) {
    assert!(app.prendre_la_touche(&Key::Named(n), true, None));
}

fn au_presse_papiers(texte: &str) {
    let mut acces = crate::interactions::presse_papiers::ouvrir().expect("le presse-papiers");
    acces.ecrire(texte.to_string()).expect("écrit");
}

/// Un code qui donne d'écrire, et son texte.
fn un_code() -> (Code, String) {
    let code = Code::neuf([7; 32]);
    let texte = code.texte_ecrire().expect("un code qui écrit");
    (code, texte)
}

/// La suite que la question du départ porte : le code qu'on rejoindra.
fn rejoindrait(app: &GlucoseApp) -> Option<Ensuite> {
    match &app.ui.question {
        Some((_, Suite::Laisser(ensuite))) => Some(ensuite.clone()),
        _ => None,
    }
}

/// **Le presse-papiers vide, le code se tape** (sa note du 23/11 : *« un emplacement texte pour
/// pouvoir écrire à la main »*) : « Rejoindre… » ouvre un champ vide ; le code tapé, Entrée rejoint
/// sa salle, par la porte du document qu'on quitte. **L'envers** : la loi d'avant ne lisait que le
/// presse-papiers — vide, elle répondait « aucun code » par la question de Collaborer, sans rien
/// laisser écrire.
#[test]
fn rejoindre_1_le_code_se_tape_dans_un_champ() {
    let mut app = app();
    ouvrir(&mut app);
    assert!(
        matches!(app.ui.question, Some((_, Suite::Rejoindre))),
        "la question du code, pas Collaborer qui dit « aucun code »"
    );
    assert_eq!(
        champ(&app).as_deref(),
        Some(""),
        "le presse-papiers ne porte rien"
    );
    assert!(!texte(&app).contains(DEJA_COPIE));
    let (code, ecrit) = un_code();
    taper(&mut app, &ecrit);
    touche(&mut app, NamedKey::Enter);
    assert_eq!(rejoindrait(&app), Some(Ensuite::Rejoindre(code)));
}

/// **Un code copié est déjà dans le champ**, tout choisi : au bureau, rien n'est perdu ; ce que
/// le presse-papiers porte d'autre ne s'y affiche pas.
#[test]
fn rejoindre_1_le_code_copie_est_deja_dans_le_champ() {
    let mut app = app();
    let (code, ecrit) = un_code();
    au_presse_papiers(&format!("  {ecrit}\n"));
    ouvrir(&mut app);
    assert_eq!(champ(&app).as_deref(), Some(ecrit.as_str()), "le code, nu");
    assert!(texte(&app).starts_with(DEJA_COPIE), "et dit");
    touche(&mut app, NamedKey::Enter);
    assert_eq!(rejoindrait(&app), Some(Ensuite::Rejoindre(code)));
    let mut autre = self::app();
    au_presse_papiers("une liste de courses");
    ouvrir(&mut autre);
    assert_eq!(
        champ(&autre).as_deref(),
        Some(""),
        "rien d'autre qu'un code"
    );
}

/// **Un texte qui n'est pas un code se redemande, tel qu'on l'a écrit**, et le dit ; `Échap`
/// revient à Collaborer.
#[test]
fn rejoindre_1_un_texte_qui_n_est_pas_un_code_se_redemande() {
    let mut app = app();
    ouvrir(&mut app);
    taper(&mut app, "glucose-ecrire-oups");
    touche(&mut app, NamedKey::Enter);
    assert!(matches!(app.ui.question, Some((_, Suite::Rejoindre))));
    assert!(texte(&app).starts_with(PAS_UN_CODE), "{}", texte(&app));
    assert_eq!(champ(&app).as_deref(), Some("glucose-ecrire-oups"));
    touche(&mut app, NamedKey::Escape);
    assert!(
        matches!(app.ui.question, Some((_, Suite::Collaborer))),
        "Échap revient à Collaborer"
    );
}
