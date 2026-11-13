//! **APPLIQUE-1** (fiche 113) : la palette, le menu et l'exécution lisent une seule loi, et cette
//! loi dit vrai — une commande offerte fait quelque chose, une commande tue n'aurait rien fait.

use crate::app::GlucoseApp;
use crate::commandes::palette::{trouvees_pour, But};
use crate::commandes::Commande;
use crate::ui::context_menu::{layout_context_menu, MenuAction, MenuRow};
use glucose_core::types::{Annotation, BoardImage, Project, Viewport};
use winit::keyboard::{Key, SmolStr};

const ECRAN: (u32, u32) = (1200, 800);

/// Deux textes liés par une flèche, un post-it dans une membrane, une image ; rien de choisi, rien
/// à défaire.
fn scene() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    for (id, x) in [("a", 0.0), ("b", 2000.0)] {
        app.store
            .add_annotation(&board, Annotation::text(id, x, 0.0, "un bloc"));
    }
    let mut lien = Annotation::arrow("f", 0.0, 0.0, 2000.0, 0.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut lien
    {
        *source_id = Some("a".into());
        *target_id = Some("b".into());
    }
    app.store.add_annotation(&board, lien);
    app.store.add_annotation(
        &board,
        Annotation::membrane("m", -100.0, 600.0, 600.0, 500.0),
    );
    app.store
        .add_annotation(&board, Annotation::sticky("p", 0.0, 800.0, "un post-it"));
    app.store
        .add_image(&board, BoardImage::new("i", 3000.0, 0.0, 100.0, 100.0));
    app.store.clear_selection();
    app.store.journal.clear();
    app.ui.current_toast = None;
    app
}

/// La scène, ces nœuds choisis.
fn choisis(ids: &[&str]) -> GlucoseApp {
    let mut app = scene();
    let (images, notes): (Vec<String>, Vec<String>) =
        ids.iter().map(|s| s.to_string()).partition(|id| id == "i");
    app.store.set_selected_image_ids(images);
    app.store.set_selected_annotation_ids(notes);
    app
}

/// La scène, la fenêtre des liens de « a » ouverte, et une de ses copies choisie — ou rien.
fn dans_la_fenetre(copie_choisie: bool) -> GlucoseApp {
    let mut app = choisis(&["a"]);
    assert!(
        app.executer(Commande::MontrerLesLiens),
        "la fenêtre s'ouvre"
    );
    if copie_choisie {
        let copie = app.store.active_board().expect("la fenêtre").annotations[0]
            .id()
            .to_string();
        app.store.set_selected_annotation_ids(vec![copie]);
    }
    app
}

/// Toutes les scènes de ces épreuves, chacune nommée.
fn scenes() -> Vec<(&'static str, GlucoseApp)> {
    vec![
        ("rien", choisis(&[])),
        ("un texte", choisis(&["a"])),
        ("la flèche", choisis(&["f"])),
        ("le post-it", choisis(&["p"])),
        ("la membrane", choisis(&["m"])),
        ("l'image", choisis(&["i"])),
        ("un texte et la flèche", choisis(&["a", "f"])),
        ("la fenêtre, rien", dans_la_fenetre(false)),
        ("la fenêtre, une copie", dans_la_fenetre(true)),
    ]
}

/// **Ce qu'un geste peut changer** : le document, le choix, la vue et le vol de la caméra, la
/// fenêtre des liens, une question, la roue, le focus, un message, un lot de bordures en
/// attente, les lentilles.
#[derive(Debug, PartialEq)]
struct Etat {
    document: Project,
    choix: (Vec<String>, Vec<String>, Option<String>),
    vue: (String, Viewport, Option<Viewport>),
    fenetre: Option<String>,
    ouvert: (bool, bool, Option<String>, bool, bool),
    /// Les lentilles : la frise ouverte et la période qu'elle garde (FRISE-1), le domaine regardé
    /// (DOMAINE-1).
    lentilles: (
        bool,
        Option<glucose_core::timeline::TemporalFilter>,
        Option<String>,
    ),
}

fn etat(app: &GlucoseApp) -> Etat {
    Etat {
        document: app.store.project.clone(),
        choix: (
            app.store.selected_image_ids.to_vec(),
            app.store.selected_annotation_ids.to_vec(),
            app.store.selected_folder_id.clone(),
        ),
        vue: (
            app.store.project.active_board_id.clone(),
            app.store.viewport(),
            app.vol.cible(),
        ),
        fenetre: app.store.fenetre_des_liens().map(|f| f.centre.clone()),
        ouvert: (
            app.ui.question.is_some(),
            app.ui.roue.is_some(),
            app.focus.membrane().map(str::to_string),
            app.ui.current_toast.is_some(),
            app.attentes.bordures.is_some(),
        ),
        lentilles: (
            app.ui.frise.is_some(),
            app.store.temporal_filter,
            app.store.domaine_regarde.clone(),
        ),
    }
}

/// **Le défaut rapporté** (fiche 111 § 4) : une flèche ne se colore pas — la palette ne propose
/// plus « Couleur » sur elle, ni la roue ; l'exécution ne fait rien. Pour changer un raccourci,
/// toutes restent ; sur un texte, elles reviennent. Par la vraie palette, écrite au clavier.
#[test]
fn test_applique_1_la_palette_ne_propose_pas_couleur_sur_une_fleche() {
    let rangees = |app: &mut GlucoseApp| -> Vec<String> {
        app.ouvrir_la_palette(But::Lancer);
        for c in ["c", "o", "u", "l"] {
            assert!(app.prendre_la_touche(&Key::Character(SmolStr::new(c)), true, Some(c)));
            app.une_image_sans_fenetre(ECRAN);
        }
        let (q, _) = app.ui.question.take().expect("la palette");
        q.choix.into_iter().map(|(l, _)| l).collect()
    };
    let mut app = choisis(&["f"]);
    let vues = rangees(&mut app);
    assert!(
        !vues
            .iter()
            .any(|l| l.starts_with("Couleur") || l.contains("roue")),
        "sur une flèche : {vues:?}"
    );
    let avant = etat(&app);
    assert!(!app.executer(Commande::Couleur(Some(1))));
    assert!(!app.executer(Commande::RoueDeCouleur));
    assert_eq!(etat(&app), avant, "rien n'a changé");
    let pour_le_clavier = trouvees_pour("coul", But::ChangerLeRaccourci, &|c| app.s_applique(c));
    assert!(pour_le_clavier.contains(&Commande::Couleur(Some(1))));
    let mut app = choisis(&["a"]);
    assert!(
        rangees(&mut app).iter().any(|l| l.starts_with("Couleur")),
        "sur un texte, elle revient"
    );
}

/// **Une seule loi** : dans chaque scène, la palette offre exactement ce qui s'applique (sauf
/// elle-même), et le menu ne montre rien qui ne s'applique pas.
#[test]
fn test_applique_1_la_palette_et_le_menu_lisent_la_loi() {
    let mut menus = 0;
    for (nom, app) in scenes() {
        let offertes = trouvees_pour("", But::Lancer, &|c| app.s_applique(c));
        for c in Commande::toutes() {
            let attendue = c != Commande::Palette && app.s_applique(c);
            assert_eq!(offertes.contains(&c), attendue, "{nom} : {c:?}");
        }
        let menu = layout_context_menu(
            &app.store,
            (&app.renderer.typography, &app.ui),
            ((100.0, 100.0), None),
            (ECRAN.0 as f32, ECRAN.1 as f32),
            1.0,
            false,
        )
        .expect("un menu");
        for row in &menu.rows {
            if let MenuRow::Item { action, .. } = row {
                let c = action.commande().expect("une commande");
                assert!(app.s_applique(c), "{nom} : le menu montre {action:?}");
                menus += 1;
            }
        }
        let filets_de_suite = menu.rows.windows(2).any(|w| {
            matches!(w[0], MenuRow::Separator { .. }) && matches!(w[1], MenuRow::Separator { .. })
        });
        assert!(!filets_de_suite, "{nom} : deux filets de suite");
    }
    assert!(menus > 40, "les menus ont été lus : {menus}");
}

/// **Les commandes que la loi dit toujours offertes** — écrites ici, et non lues dans la loi :
/// une commande qu'elle rendrait « toujours » par erreur reste éprouvée dans chaque scène.
fn toujours(c: Commande) -> bool {
    use Commande as C;
    matches!(
        c,
        C::NouveauDocument
            | C::OuvrirUnDocument
            | C::Enregistrer
            | C::EnregistrerSous
            | C::Exporter
            | C::ImporterDesImages
            | C::TimeMachine
            | C::Coller
            | C::ToutChoisir
            | C::ToutCadrer
            | C::CadrerLaSelection
            | C::Naviguer
            | C::Echap
            | C::PoserLeSignet(_)
            | C::AllerAuSignet(_)
            | C::Outil(_)
            | C::Signets
            | C::NouvelOnglet
            | C::ImporterUnDocument
            | C::Chercher
            | C::ResultatSuivant
            | C::ResultatPrecedent
            | C::Palette
            | C::ChangerUnRaccourci
            | C::RetablirLesRaccourcis
            | C::OublierLesAgents
            | C::ModeZen
    )
}

/// Ce qui tient au système plutôt qu'à Glucose : le presse-papiers, l'explorateur, la fenêtre
/// elle-même, et le serveur du programme, qu'aucune épreuve n'a (`telemetrie::serveur`).
fn systeme(c: Commande) -> bool {
    matches!(
        c,
        Commande::Copier
            | Commande::Couper
            | Commande::VoirCeQuiPart
            | Commande::JournalTechnique
            | Commande::ModeReference
    )
}

/// **La loi dit vrai** : sans elle — la fenêtre des liens fermée d'abord, puis le geste nu —, une
/// commande qu'elle tait ne change rien, et une commande qu'elle offre change quelque chose ; et
/// l'exécution, elle, refuse ce qu'elle tait. Sauf l'ordre, les colonnes et la relation d'une
/// flèche, qui s'appliquent dès qu'ils ont un objet : un ordre ou une disposition déjà atteints
/// ne se calculent qu'en les faisant, et une relation déjà portée reste offerte pour que son
/// chiffre ne vole pas au signet.
#[test]
fn test_applique_1_ce_qu_elle_offre_fait_quelque_chose_et_rien_d_autre() {
    let idempotentes = |c: Commande| {
        matches!(
            c,
            Commande::PremierPlan
                | Commande::ArrierePlan
                | Commande::DisposerEnColonnes
                | Commande::Relation(_)
        )
    };
    let (mut tues, mut offerts) = (0, 0);
    for c in Commande::toutes() {
        if toujours(c) || systeme(c) {
            continue;
        }
        for (nom, mut app) in scenes() {
            let applique = app.s_applique(c);
            if !c.regarde_la_fenetre() {
                app.fermer_la_fenetre_des_liens();
            }
            let avant = etat(&app);
            if !applique {
                assert!(!app.executer(c), "{nom} : {c:?} est tue, et s'exécute");
                assert_eq!(etat(&app), avant, "{nom} : {c:?} refusée a changé");
            }
            app.faire(c);
            let change = etat(&app) != avant;
            if !applique {
                assert!(
                    !change,
                    "{nom} : {c:?} est tue, et le geste change quelque chose"
                );
                tues += 1;
            } else if !idempotentes(c) {
                assert!(
                    change,
                    "{nom} : {c:?} est offerte, et le geste ne fait rien"
                );
                offerts += 1;
            }
        }
    }
    assert!(tues > 50 && offerts > 20, "{tues} tues, {offerts} offerts");
}

/// **Le clavier n'y perd rien** : sans flèche choisie, le chiffre vole au signet ; sur une
/// flèche, il pose la relation, même déjà portée.
#[test]
fn test_applique_1_les_chiffres_gardent_leur_loi() {
    let mut app = choisis(&["f"]);
    assert!(app.s_applique(Commande::Relation(Some(1))));
    app.faire(Commande::Relation(Some(1)));
    assert!(
        app.s_applique(Commande::Relation(Some(1))),
        "déjà portée, toujours offerte"
    );
    let app = choisis(&["a"]);
    assert!(!app.s_applique(Commande::Relation(Some(1))));
    assert!(app.s_applique(Commande::AllerAuSignet(1)));
}

/// Le menu d'une sélection de deux textes ne montre ni « Suivre le lien » ni les opérateurs :
/// la loi les a retirés, avec leurs filets.
#[test]
fn test_applique_1_le_menu_d_une_selection_sans_fleche_ni_post_it() {
    let app = choisis(&["a", "b"]);
    let menu = layout_context_menu(
        &app.store,
        (&app.renderer.typography, &app.ui),
        ((100.0, 100.0), None),
        (ECRAN.0 as f32, ECRAN.1 as f32),
        1.0,
        false,
    )
    .expect("un menu");
    let actions: Vec<MenuAction> = menu
        .rows
        .iter()
        .filter_map(|r| match r {
            MenuRow::Item { action, .. } => Some(*action),
            _ => None,
        })
        .collect();
    assert!(actions.contains(&MenuAction::Copier));
    assert!(actions.contains(&MenuAction::Grouper));
    assert!(!actions.contains(&MenuAction::SuivreLeLien));
    assert!(!actions
        .iter()
        .any(|a| matches!(a, MenuAction::Operateur(_))));
    assert!(!matches!(
        menu.rows.first(),
        Some(MenuRow::Separator { .. })
    ));
    assert!(!matches!(menu.rows.last(), Some(MenuRow::Separator { .. })));
}
