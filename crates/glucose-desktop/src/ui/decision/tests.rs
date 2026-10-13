//! **Ce qui attend une décision se sait d'avance** (CHROME-1) : la voie graphique ne peint ni
//! ne relève son tampon quand [`UiState::attend_une_decision`] dit non. Ces épreuves lient la
//! condition au dessin : sans rien à décider, le groupe ne dessine rien ; chaque élément, seul,
//! dessine — et la condition le voit.

use super::*;
use crate::renderer::Renderer;
use glucose_core::types::Annotation;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (800, 600);

/// Combien de pixels le groupe dessine, sur un tampon transparent.
fn encre(store: &Store, ui: &UiState) -> usize {
    let mut renderer = Renderer::new();
    let mut tampon = Pixmap::new(ECRAN.0, ECRAN.1).expect("un tampon");
    let pointeur = Pointer { x: -1.0, y: -1.0 };
    renderer.poser_ce_qui_attend_une_decision(&mut tampon.as_mut(), store, ui, pointeur);
    tampon.pixels().iter().filter(|p| p.alpha() > 0).count()
}

/// Un document d'une carte de texte, rien de choisi.
fn document() -> Store {
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    store.add_annotation(
        &board,
        Annotation::text("c", 100.0, 100.0, "Bonjour le monde"),
    );
    store.clear_selection();
    store
}

/// **Sans rien à décider, le groupe ne dessine rien** — c'est ce qui permet à la carte de ne
/// pas le relever.
#[test]
fn test_rien_a_decider_ne_dessine_rien() {
    let store = document();
    let ui = UiState::new();
    assert!(!ui.attend_une_decision(&store));
    assert_eq!(
        encre(&store, &ui),
        0,
        "le groupe dessine sans rien à décider"
    );
}

/// **Chaque élément du groupe, seul, dessine — et la condition le voit.** Retirer l'une de ses
/// clauses ferait disparaître cet élément sur la voie graphique.
#[test]
fn test_chaque_element_qui_dessine_attend_une_decision() {
    let base = document();
    let mut cas: Vec<(&str, Store, UiState)> = Vec::new();

    let mut choisi = base.clone();
    choisi.select_annotation("c".into(), false);
    cas.push(("une sélection", choisi, UiState::new()));

    // Un message neuf naît transparent (son fondu d'entrée) : on le prend sur son plateau. Et
    // sa vie est longue : sous la charge de la suite entière, plusieurs secondes passent avant
    // son dessin, et un message de 2,4 s y était déjà en train de s'effacer.
    let mut ui = UiState::new();
    let mut message = toast::Toast::new("Enregistré");
    message.created_at = std::time::Instant::now() - std::time::Duration::from_millis(1000);
    message.duration = std::time::Duration::from_secs(3600);
    ui.current_toast = Some(message);
    cas.push(("un message", base.clone(), ui));

    let mut ui = UiState::new();
    ui.context_menu_at = Some((200.0, 200.0));
    cas.push(("un menu", base.clone(), ui));

    let mut ui = UiState::new();
    let question = question::Question {
        titre: "Journal technique".into(),
        texte: "Aider à améliorer Glucose ?".into(),
        choix: vec![("Oui".into(), question::Reponse::Oui)],
        ..Default::default()
    };
    ui.question = Some((question, question::Suite::Telemetrie));
    cas.push(("une question", base.clone(), ui));

    // L'éditeur d'ancres s'ouvre sur une flèche choisie, mais rien ne garantit qu'elle le
    // reste : sa clause est la sienne.
    let mut ui = UiState::new();
    ui.ancrage = Some(super::super::ancrage::Ancrage {
        fleche: "f".into(),
        etape: super::super::ancrage::Etape::Source,
        cartes: (Some("c".into()), None),
        source: Vec::new(),
        cible: Vec::new(),
        glisse: None,
        defilement: 0.0,
    });
    cas.push(("l'éditeur d'ancres", base.clone(), ui));

    for (quoi, store, ui) in &cas {
        assert!(
            encre(store, ui) > 0,
            "{quoi} ne dessine rien : l'épreuve est aveugle"
        );
        assert!(
            ui.attend_une_decision(store),
            "{quoi} dessine, et la condition ne le voit pas : il disparaîtrait sur la carte"
        );
    }
}
