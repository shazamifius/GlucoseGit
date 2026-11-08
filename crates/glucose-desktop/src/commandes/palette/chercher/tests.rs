//! `Ctrl+F` joué sur le vrai `GlucoseApp` : la touche, l'écriture, Entrée, `F3`.

use crate::app::GlucoseApp;
use crate::commandes::palette::{But, Etat};
use crate::commandes::Commande;
use crate::ui::question::{Reponse, Suite};
use glucose_core::types::Annotation;

/// Une application à deux onglets ; « loin » est loin de la vue, sur le second.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let un = app.store.project.active_board_id.clone();
    let deux = app.store.add_board("Chimie");
    app.store.add_annotation(
        &un,
        Annotation::text("pres", 0.0, 0.0, "L'eau, ça mouille."),
    );
    app.store.add_annotation(
        &deux,
        Annotation::text("loin", 9000.0, 9000.0, "Eau = H2O. Deux hydrogènes."),
    );
    app.store.journal.clear();
    app
}

/// Écrit dans le champ de la question, comme le clavier, puis laisse la palette le suivre (une image).
fn ecrire(app: &mut GlucoseApp, texte: &str) {
    if let Some((q, _)) = &mut app.ui.question {
        q.champ.as_mut().expect("un champ").texte = texte.to_string();
    }
    app.suivre_la_palette();
}

fn etat(app: &GlucoseApp) -> Etat {
    match &app.ui.question {
        Some((_, Suite::Palette(e))) => e.clone(),
        autre => panic!("pas de palette : {autre:?}"),
    }
}

/// **`Ctrl+F` ouvre la recherche**, qui trouve dans tous les onglets, sans majuscules ni accents, et
/// dit combien.
#[test]
fn ctrl_f_trouve_dans_tous_les_onglets() {
    let mut app = app();
    assert!(app.executer(Commande::Chercher));
    assert_eq!(etat(&app).but, But::Chercher);
    ecrire(&mut app, "EAU");
    let e = etat(&app);
    let ids: Vec<_> = e.trouves.iter().filter_map(|t| t.id.clone()).collect();
    assert_eq!(ids, ["pres", "loin"]);
    let (q, _) = app.ui.question.as_ref().expect("la question");
    assert!(q.texte.starts_with("2 résultats"), "{}", q.texte);
    assert!(
        q.choix
            .iter()
            .any(|(l, _)| l.contains("Eau = H2O.") && l.contains("Chimie")),
        "la phrase, et l'onglet : {:?}",
        q.choix
    );
    ecrire(&mut app, "feu");
    let (q, _) = app.ui.question.as_ref().expect("la question");
    assert!(q.texte.contains("Aucun résultat"), "{}", q.texte);
    assert!(q.choix.is_empty());
}

/// **Entrée va au résultat** (TROUVE-2) : son onglet, le nœud choisi, la caméra qui vole — et rien
/// dans le journal.
#[test]
fn entree_va_au_resultat_sans_rien_ecrire() {
    let mut app = app();
    let profondeur = app.store.undo_depth();
    app.executer(Commande::Chercher);
    ecrire(&mut app, "hydrogenes");
    app.repondre_a_la_question(Some(Reponse::Choix(0)));
    assert!(app.ui.question.is_none(), "la palette se ferme");
    let deux = app.store.project.boards[1].id.clone();
    assert_eq!(app.store.project.active_board_id, deux, "son onglet");
    assert_eq!(
        app.store.selected_annotation_ids,
        ["loin"],
        "le nœud choisi"
    );
    assert!(app.vol.en_cours(), "la caméra vole");
    assert_eq!(
        app.store.undo_depth(),
        profondeur,
        "le journal n'en sait rien"
    );
}

/// **`F3` va au suivant, `Maj+F3` au précédent**, en boucle, sans rouvrir la palette ; sans
/// recherche, `F3` l'ouvre.
#[test]
fn f3_va_au_suivant_en_boucle() {
    let mut app = app();
    assert!(app.executer(Commande::ResultatSuivant));
    assert!(
        matches!(app.ui.question, Some((_, Suite::Palette(ref e))) if e.but == But::Chercher),
        "sans recherche, F3 l'ouvre"
    );
    ecrire(&mut app, "eau");
    app.repondre_a_la_question(Some(Reponse::Choix(0)));
    assert_eq!(app.store.selected_annotation_ids, ["pres"]);
    app.executer(Commande::ResultatSuivant);
    assert!(app.ui.question.is_none(), "pas de palette");
    assert_eq!(app.store.selected_annotation_ids, ["loin"]);
    app.executer(Commande::ResultatSuivant);
    assert_eq!(app.store.selected_annotation_ids, ["pres"], "en boucle");
    app.executer(Commande::ResultatPrecedent);
    assert_eq!(app.store.selected_annotation_ids, ["loin"]);
}

/// **Le document change entre deux `F3`** : la recherche se refait sur ce qu'il est devenu.
#[test]
fn f3_cherche_dans_le_document_tel_qu_il_est() {
    let mut app = app();
    app.executer(Commande::Chercher);
    ecrire(&mut app, "eau");
    app.repondre_a_la_question(Some(Reponse::Choix(0)));
    let un = app.store.project.active_board_id.clone();
    app.store.remove_annotations(&un, &["pres"]);
    app.executer(Commande::ResultatSuivant);
    assert_eq!(
        app.store.selected_annotation_ids,
        ["loin"],
        "le seul qui reste"
    );
    let deux = app.store.project.active_board_id.clone();
    app.store.remove_annotations(&deux, &["loin"]);
    app.executer(Commande::ResultatSuivant);
    assert!(app.ui.question.is_some(), "plus rien : la palette s'ouvre");
}

/// **Au doigt** : « Chercher… » au menu du vide, qui fait ce que fait `Ctrl+F`.
#[test]
fn le_menu_du_vide_porte_chercher() {
    use crate::ui::context_menu::MenuAction;
    assert_eq!(MenuAction::Chercher.commande(), Some(Commande::Chercher));
}

/// **Un résultat rangé dans un dossier** : on y entre, la pile des dossiers se refait, et la rangée
/// dit l'onglet qui le porte. La preuve à l'envers : son tableau n'est pas un onglet.
#[test]
fn un_resultat_dans_un_dossier_s_atteint() {
    let mut app = app();
    let un = app.store.project.active_board_id.clone();
    let mut dossier = glucose_core::types::CanvasFolder::new("", "Archives", "");
    (dossier.x, dossier.y) = (-5000.0, -5000.0);
    app.store.create_folder(&un, dossier);
    let dedans = app.store.project.boards[0].folders[0]
        .child_board_id
        .clone();
    assert!(
        app.store.onglets().all(|(id, _, _)| id != dedans),
        "ce n'est pas un onglet"
    );
    app.store.add_annotation(
        &dedans,
        Annotation::text("range", 0.0, 0.0, "Une vieille note sur l'eau."),
    );
    app.executer(Commande::Chercher);
    ecrire(&mut app, "vieille");
    let (q, _) = app.ui.question.as_ref().expect("la question");
    let premier_onglet = app.store.onglets().next().map(|(_, n, _)| n.to_string());
    assert!(
        q.choix[0].0.ends_with(&premier_onglet.unwrap()),
        "l'onglet qui le porte : {:?}",
        q.choix
    );
    app.repondre_a_la_question(Some(Reponse::Choix(0)));
    assert_eq!(app.store.project.active_board_id, dedans, "on y entre");
    assert_eq!(
        app.store.folder_path().last().map(String::as_str),
        Some("Archives"),
        "par son dossier"
    );
    assert_eq!(app.store.selected_annotation_ids, ["range"]);
}

/// **Les raccourcis de la recherche** : `Ctrl+F` cherche (sa décision du 13/10 : tout cadrer est passé
/// à `Alt+F`), `F3` et `Maj+F3` vont au suivant et au précédent.
#[test]
fn ctrl_f_et_f3_sont_a_la_recherche() {
    use glucose_core::raccourcis::Accord;
    let app = app();
    let accords = |id: &str| app.ui.clavier.accords_de(id);
    assert_eq!(accords("chercher"), [Accord::lire("Ctrl+F").unwrap()]);
    assert_eq!(accords("chercher.suivant"), [Accord::lire("F3").unwrap()]);
    assert_eq!(
        accords("chercher.precedent"),
        [Accord::lire("Maj+F3").unwrap()]
    );
    assert_eq!(accords("tout-cadrer"), [Accord::lire("Alt+F").unwrap()]);
}

/// **L'index suit le document** : un nœud posé pendant qu'on cherche se trouve à la lettre suivante.
#[test]
fn l_index_suit_le_document() {
    let mut app = app();
    assert_eq!(app.chercher_dans_le_document("braise").len(), 0);
    let un = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&un, Annotation::text("feu", 0.0, 0.0, "Une braise."));
    assert_eq!(app.chercher_dans_le_document("braise").len(), 1);
}

/// **Une flèche trouvée par ce qu'elle dit** : Entrée la choisit et vole jusqu'à son encre, comme
/// `F` la cadrerait. Elle n'a pas de boîte où s'accrocher : sans son emprise, la vue restait où elle
/// était.
#[test]
fn entree_vole_jusqu_a_une_fleche() {
    let mut app = app();
    let un = app.store.project.active_board_id.clone();
    let mut fleche = Annotation::arrow("vapeur", 4000.0, 0.0, 4400.0, 100.0);
    if let Annotation::Arrow { text, .. } = &mut fleche {
        *text = Some("devient de la vapeur".to_string());
    }
    app.store.add_annotation(&un, fleche);
    app.executer(Commande::Chercher);
    ecrire(&mut app, "vapeur");
    app.repondre_a_la_question(Some(Reponse::Choix(0)));
    assert_eq!(app.store.selected_annotation_ids, ["vapeur"]);
    let vue = app.vol.cible().expect("la caméra vole");
    let (l, h) = app.taille_de_la_fenetre();
    let bandeau = f64::from(app.ui.header_height());
    let centre = (
        (f64::from(l) / 2.0 - vue.x) / vue.scale,
        (bandeau + (f64::from(h) - bandeau) / 2.0 - vue.y) / vue.scale,
    );
    assert!(
        (centre.0 - 4200.0).abs() < 1e-6 && (centre.1 - 50.0).abs() < 1e-6,
        "le centre de son encre au centre de l'écran : {centre:?}"
    );
}
