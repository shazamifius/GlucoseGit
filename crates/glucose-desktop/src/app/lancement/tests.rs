//! La question de la mise à jour, dessinée (POPUP-1) : sans veille, rien ne se télécharge
//! ni ne se lance — une application d'épreuve ne touche jamais au réseau.

use crate::app::GlucoseApp;
use crate::interactions::question::tests::repondre;
use crate::mise_a_jour::version::Version;
use crate::mise_a_jour::Proposition;
use crate::ui::question::{echappatoire, Reponse, Suite};

fn proposition(notes: &str) -> Proposition {
    Proposition {
        version: Version::lire("9.9.9").expect("une version"),
        notes: notes.into(),
        url: String::new(),
        signature: String::new(),
    }
}

/// **La mise à jour se propose par la question dessinée** : « Installer » d'abord, « Plus
/// tard » pour Échap ; les notes s'y lisent. « Installer » la prépare et le dit.
#[test]
fn test_popup_1_la_mise_a_jour_se_propose_dessinee() {
    let mut app = GlucoseApp::new();
    app.proposer_la_mise_a_jour(proposition("Les popups se dessinent."));
    let Some((question, Suite::MiseAJour(p))) = &app.ui.question else {
        panic!("la question de la mise à jour");
    };
    assert_eq!(p.version, Version::lire("9.9.9").expect("une version"));
    assert!(question.texte.contains("9.9.9") && question.texte.contains("Les popups"));
    let libelles: Vec<&str> = question.choix.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(libelles, ["Installer", "Plus tard"]);
    assert_eq!(echappatoire(question), Some(Reponse::Annuler));

    repondre(&mut app, Reponse::Oui);
    let toast = app.ui.current_toast.as_ref().expect("elle le dit");
    assert!(toast.message.contains("téléchargement de Glucose 9.9.9"));
}

/// **« Plus tard » ne prépare rien** : aucun message, aucune question.
#[test]
fn test_popup_1_plus_tard_ne_prepare_rien() {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    app.proposer_la_mise_a_jour(proposition(""));
    repondre(&mut app, Reponse::Annuler);
    assert!(app.ui.question.is_none());
    assert!(app.ui.current_toast.is_none(), "rien ne se télécharge");
}

/// **Prête, elle ferme le document puis la fenêtre** ; sans veille — une épreuve —, rien ne
/// se ferme : il n'y a rien à installer.
#[test]
fn test_popup_1_prete_elle_ferme_la_fenetre() {
    let mut app = GlucoseApp::new();
    let installeur = std::path::Path::new("glucose-9.9.9.exe");
    app.installer(installeur);
    assert!(!app.ui.fermer_la_fenetre, "sans veille, rien à installer");
    app.relancer_pour_installer(installeur);
    assert!(
        app.ui.fermer_la_fenetre,
        "la relance partie, la fenêtre part"
    );
}
