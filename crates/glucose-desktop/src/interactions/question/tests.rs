//! POPUP-1 — la question dessinée partout, et au clavier (fiche 58).

use crate::app::GlucoseApp;
use crate::persist::close::Apres;
use crate::telemetrie::Telemetrie;
use crate::ui::question::{Reponse, Suite};
use winit::event::MouseButton;
use winit::keyboard::{Key, ModifiersState, NamedKey};

/// Touche la réponse `r` de la question posée, comme la main : appui puis relâchement dessus.
pub(crate) fn repondre(app: &mut GlucoseApp, r: Reponse) {
    let (question, _) = app.ui.question.clone().expect("une question");
    let (w, h) = app.taille_de_la_fenetre();
    let placee = crate::ui::question::placer(
        &question,
        &app.renderer.typography,
        (w, h),
        app.ui.scale_factor,
    );
    let ((x, y, bw, bh), _, _) = placee
        .boutons
        .iter()
        .find(|(_, _, reponse)| *reponse == r)
        .expect("la réponse")
        .clone();
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(
        f64::from(x + bw / 2.0),
        f64::from(y + bh / 2.0),
    ));
    app.handle_mouse_down(MouseButton::Left, w, h);
    app.handle_mouse_up(MouseButton::Left);
}

/// Frappe une touche nommée, enfoncée puis relâchée : la question prend-elle l'appui ? Le
/// relâchement va à la question si elle est encore là — une réponse l'a peut-être retirée.
fn frapper(app: &mut GlucoseApp, touche: NamedKey) -> bool {
    let touche = Key::Named(touche);
    let prise = app.touche_de_la_question(&touche, true);
    let encore = app.ui.question.is_some();
    assert_eq!(app.touche_de_la_question(&touche, false), encore);
    prise
}

/// Un dossier d'épreuve, effacé à la fin.
struct Dossier(std::path::PathBuf);

impl Drop for Dossier {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Une application dont le journal technique vit dans un dossier d'épreuve, et sa question
/// posée — « Oui », « Non », sans « Annuler ».
fn avec_la_question_du_journal(nom: &str) -> (GlucoseApp, Dossier) {
    let d = std::env::temp_dir().join(format!("glucose-popup-{nom}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dossier d'épreuve");
    let mut app = GlucoseApp::new();
    app.lancement.telemetrie = Telemetrie::habiter(d.clone(), None);
    app.revoir_la_telemetrie();
    assert!(app.ui.question.is_some(), "la question est posée");
    (app, Dossier(d))
}

/// **Entrée donne la réponse en évidence**, la première ; **Tab** la déplace, et revient au
/// début après la dernière ; **Maj + Tab** recule.
#[test]
fn test_popup_1_entree_donne_la_reponse_en_evidence_et_tab_la_deplace() {
    let (mut app, d) = avec_la_question_du_journal("entree");
    assert!(frapper(&mut app, NamedKey::Enter));
    assert!(app.ui.question.is_none(), "la question s'en va");
    assert_eq!(app.lancement.telemetrie.accorde(), Some(true), "« Oui »");

    app.revoir_la_telemetrie();
    frapper(&mut app, NamedKey::Tab);
    frapper(&mut app, NamedKey::Enter);
    assert_eq!(
        Telemetrie::habiter(d.0.clone(), None).accorde(),
        Some(false),
        "Tab, puis Entrée : « Non »"
    );

    app.revoir_la_telemetrie();
    app.modifiers = ModifiersState::SHIFT;
    frapper(&mut app, NamedKey::Tab);
    app.modifiers = ModifiersState::empty();
    let (question, _) = app.ui.question.as_ref().expect("la question");
    assert_eq!(
        question.focus, 1,
        "Maj + Tab recule, de la première à la dernière"
    );
    frapper(&mut app, NamedKey::Tab);
    let (question, _) = app.ui.question.as_ref().expect("la question");
    assert_eq!(question.focus, 0, "Tab revient au début");
}

/// **Échap ne répond jamais à sa place** : sans « Annuler », la question se retire, et rien ne
/// s'écrit — elle se reposera (VUE-1).
#[test]
fn test_popup_1_echap_sans_annuler_ne_repond_rien() {
    let (mut app, d) = avec_la_question_du_journal("echap");
    assert!(frapper(&mut app, NamedKey::Escape));
    assert!(app.ui.question.is_none(), "la question se retire");
    assert_eq!(
        app.lancement.telemetrie.accorde(),
        None,
        "rien n'est répondu"
    );
    assert_eq!(
        Telemetrie::habiter(d.0.clone(), None).accorde(),
        None,
        "rien ne s'est écrit"
    );
}

/// **Échap donne « Annuler »** quand la question l'a : la croix sur un travail modifié, puis
/// Échap — la fenêtre reste, le travail aussi. Et le retour d'Android fait de même.
#[test]
fn test_popup_1_echap_donne_annuler() {
    let mut app = GlucoseApp::new();
    crate::persist::disque::tests::noter(&mut app, "n", "du travail");
    for touche in [NamedKey::Escape, NamedKey::BrowserBack] {
        app.fermer_puis(Apres::Quitter);
        assert!(matches!(app.ui.question, Some((_, Suite::Fermer(_)))));
        frapper(&mut app, touche);
        assert!(app.ui.question.is_none(), "{touche:?} retire la question");
        assert!(!app.ui.fermer_la_fenetre, "{touche:?} ne ferme pas");
        assert!(app.is_dirty(), "le travail reste");
    }
}

/// **Rien ne passe sous le voile** : une touche, un clic droit, la molette — le canevas
/// attend la réponse. Sans question, la touche repart au canevas.
#[test]
fn test_popup_1_rien_ne_passe_sous_le_voile() {
    let (mut app, _d) = avec_la_question_du_journal("voile");
    assert!(frapper(&mut app, NamedKey::Delete), "la touche est prise");
    let (w, h) = app.taille_de_la_fenetre();
    app.handle_mouse_down(MouseButton::Right, w, h);
    assert!(
        !app.is_panning,
        "le bouton droit ne fait pas glisser le canevas"
    );
    app.handle_mouse_up(MouseButton::Right);
    assert!(
        app.ui.context_menu_at.is_none(),
        "aucun menu sous la question"
    );
    let vue = app.store.viewport();
    app.handle_mouse_wheel(winit::event::MouseScrollDelta::LineDelta(0.0, 3.0));
    app.appliquer_l_elan(w as u32, h as u32);
    assert_eq!(app.store.viewport(), vue, "la molette ne bouge rien");
    frapper(&mut app, NamedKey::Escape);
    assert!(
        !frapper(&mut app, NamedKey::Delete),
        "sans question, la touche repart"
    );
}

/// **« Supprimer » laisse l'évidence à « Annuler »** : Entrée, frappée par réflexe, ne
/// détruit rien (le motif de dialogue du W3C).
#[test]
fn test_popup_1_entree_ne_supprime_rien() {
    let mut app = GlucoseApp::new();
    let chemin = std::path::PathBuf::from("inexistant.glucose");
    app.gerer_un_document(chemin.clone());
    repondre(&mut app, Reponse::Choix(2));
    let Some((question, Suite::Supprimer(_))) = &app.ui.question else {
        panic!("la question de la suppression");
    };
    assert_eq!(
        crate::ui::question::reponse_en_evidence(question),
        Some(Reponse::Annuler)
    );
}

/// **Un clic droit commencé avant la question n'ouvre pas de menu sous elle** : le bouton se
/// relève une fois la question posée — le geste se termine, le menu ne paraît pas.
#[test]
fn test_popup_1_un_clic_droit_d_avant_n_ouvre_rien_dessous() {
    let mut app = GlucoseApp::new();
    let (w, h) = app.taille_de_la_fenetre();
    app.handle_mouse_down(MouseButton::Right, w, h);
    app.revoir_la_telemetrie();
    app.handle_mouse_up(MouseButton::Right);
    assert!(
        app.ui.context_menu_at.is_none(),
        "aucun menu sous la question"
    );
    assert!(!app.right_or_middle_down, "le geste s'est terminé");
}

/// **BROUILLON-1, dessiné** : `Ctrl+N` sur un travail sans nom demande ce qu'il devient.
/// « Annuler » le garde tel quel ; « Ne pas enregistrer » le laisse, et le vierge le remplace.
#[test]
fn test_popup_1_le_travail_sans_nom_qu_on_quitte_pose_la_question() {
    let mut app = GlucoseApp::new();
    crate::persist::disque::tests::noter(&mut app, "n", "du travail sans nom");
    app.nouveau_document();
    assert!(
        matches!(
            app.ui.question,
            Some((_, Suite::Laisser(crate::persist::close::Ensuite::Vierge)))
        ),
        "la question du travail est posée"
    );
    repondre(&mut app, Reponse::Annuler);
    assert!(app.is_dirty(), "Annuler garde le travail");
    assert!(!app
        .store
        .active_board()
        .expect("un tableau")
        .annotations
        .is_empty());

    app.nouveau_document();
    repondre(&mut app, Reponse::Non);
    assert!(!app.is_dirty(), "le vierge a remplacé le travail");
    assert!(app
        .store
        .active_board()
        .expect("un tableau")
        .annotations
        .is_empty());
}
