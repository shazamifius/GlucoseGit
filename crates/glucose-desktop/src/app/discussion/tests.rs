//! La discussion vue de la boucle (fiche 129) : le clavier va au champ et le rend, ce que le fil
//! rapporte arrive dans le tiroir — par les vrais chemins (la touche, l'appui de souris, le tiroir
//! rangé dans l'écran). Aucune clé : une épreuve ne lit jamais celle de l'utilisateur.

use crate::app::GlucoseApp;
use crate::dock::discussion::{layout_discussion_panel, Message};
use crate::dock::TabId;
use crate::ia::discussion::{Conversation, EnRoute, Evenement};
use crate::params::Pointer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Un Glucose dont le tiroir de la discussion est ouvert, le clavier au champ.
fn app_ouverte() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.dock_manager.toggle_tab(TabId::Discussion);
    app.dock_manager.discussion.tape = true;
    app
}

/// Frappe une touche comme le vrai clavier : une lettre arrive avec son texte composé, que la loi
/// de l'édition insère (et non la touche, CHAMP-QUESTION-1) ; une touche nommée, sans texte.
fn taper(app: &mut GlucoseApp, touche: Key) -> bool {
    let texte = match &touche {
        Key::Character(c) => Some(c.to_string()),
        _ => None,
    };
    app.prendre_la_touche(&touche, true, texte.as_deref())
}

fn lettre(c: &str) -> Key {
    Key::Character(c.into())
}

/// Une question en route, que l'épreuve nourrit à la place du fil.
fn en_route(app: &mut GlucoseApp) -> (Sender<Evenement>, Arc<AtomicBool>) {
    let (tx, evenements) = channel();
    let arret = Arc::new(AtomicBool::new(false));
    app.lancement.agents.discussion = Some(EnRoute {
        evenements,
        arret: arret.clone(),
    });
    (tx, arret)
}

#[test]
fn test_discussion_le_clavier_va_au_champ_puis_le_rend() {
    let mut app = app_ouverte();
    assert!(taper(&mut app, lettre("é")));
    assert!(taper(&mut app, lettre("t")));
    assert!(taper(&mut app, Key::Named(NamedKey::Backspace)));
    assert_eq!(app.dock_manager.discussion.entree.texte, "é");
    // Échap rend le clavier : la touche suivante n'est plus au champ.
    assert!(taper(&mut app, Key::Named(NamedKey::Escape)));
    assert!(!app.dock_manager.discussion.tape);
    assert!(!app.frapper_dans_la_discussion(&lettre("x"), None));
    assert_eq!(app.dock_manager.discussion.entree.texte, "é");
    // Tiroir fermé, le champ ne prend rien, même armé.
    app.dock_manager.discussion.tape = true;
    app.dock_manager.toggle_tab(TabId::Discussion);
    assert!(!app.frapper_dans_la_discussion(&lettre("x"), None));
}

/// **Un accent de touche morte s'écrit** : sur un clavier AZERTY, « ê » est `^` puis `e`, et c'est le
/// texte composé qui compte, pas la touche — la loi de l'édition d'un nœud (CHAMP-QUESTION-1).
#[test]
fn test_discussion_un_accent_de_touche_morte_s_ecrit() {
    let mut app = app_ouverte();
    assert!(app.prendre_la_touche(&lettre("e"), true, Some("ê")));
    assert!(app.prendre_la_touche(&lettre("t"), true, Some("t")));
    assert_eq!(app.dock_manager.discussion.entree.texte, "êt");
}

/// **Un appui ailleurs rend le clavier, un appui dans le champ le reprend** — par le vrai appui de
/// souris, sur le tiroir tel que le dock le range dans l'écran.
#[test]
fn test_discussion_un_appui_ailleurs_rend_le_clavier_le_champ_le_reprend() {
    let mut app = app_ouverte();
    app.mouse_pos = (720.0, 450.0);
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    assert!(
        !app.dock_manager.discussion.tape,
        "le canevas a pris l'appui"
    );
    let ecran = app.screen_frame(ECRAN.0, ECRAN.1);
    let boite = crate::dock::places(&app.dock_manager, &app.store, ecran)
        .into_iter()
        .find(|b| b.tab == TabId::Discussion)
        .expect("le tiroir est rangé");
    let dedans = Pointer {
        x: boite.x + boite.width / 2.0,
        y: boite.y + 10.0,
    };
    let (tab, cadre) =
        crate::dock::panneau_sous(&app.dock_manager, &app.store, ecran, dedans).expect("visé");
    assert_eq!(tab, TabId::Discussion);
    let champ = layout_discussion_panel(cadre, &app.dock_manager.discussion).champ;
    app.mouse_pos = (
        f64::from(champ.x + champ.w / 2.0),
        f64::from(champ.y + champ.h / 2.0),
    );
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    assert!(
        app.dock_manager.discussion.tape,
        "le champ a repris le clavier"
    );
}

/// **Ce que le fil rapporte arrive dans le tiroir**, dans l'ordre ; à la fin, la conversation
/// revient à la boucle, et le tiroir descend jusqu'au dernier message.
#[test]
fn test_discussion_ce_que_le_fil_rapporte_arrive_dans_le_tiroir() {
    let mut app = app_ouverte();
    let (tx, _) = en_route(&mut app);
    tx.send(Evenement::Demande("Gemini 3.8 Flash".into()))
        .unwrap();
    app.suivre_la_discussion();
    assert_eq!(
        app.dock_manager.discussion.en_cours.as_deref(),
        Some("Gemini 3.8 Flash")
    );
    for e in [
        Evenement::Geste {
            outil: "glucose_poser_texte".into(),
            echec: None,
        },
        Evenement::Texte {
            modele: "Gemini 3.8 Flash".into(),
            texte: "J'ai posé le texte.".into(),
        },
        Evenement::Avertissement("Gemini 3.7 Flash prend le relais.".into()),
        Evenement::Fin {
            conversation: Conversation::nouvelle("k".into()),
            erreur: None,
        },
    ] {
        tx.send(e).unwrap();
    }
    app.suivre_la_discussion();
    let ui = &app.dock_manager.discussion;
    assert_eq!(
        ui.messages,
        [
            Message::Geste("a posé un texte".into()),
            Message::Ia {
                modele: "Gemini 3.8 Flash".into(),
                texte: "J'ai posé le texte.".into()
            },
            Message::Note("Gemini 3.7 Flash prend le relais.".into()),
        ]
    );
    assert_eq!(ui.en_cours, None);
    assert!(!ui.lignes.is_empty(), "les lignes sont coupées");
    assert!(app.lancement.agents.discussion.is_none());
    assert!(app.lancement.agents.conversation.is_some());
    assert_eq!(app.dock_manager.defile(TabId::Discussion), f32::MAX);
}

/// Un fil qui meurt sans finir laisse une réception orpheline : c'est dit, et la suivante repart.
#[test]
fn test_discussion_un_fil_perdu_se_dit() {
    let mut app = app_ouverte();
    let (tx, _) = en_route(&mut app);
    drop(tx);
    app.suivre_la_discussion();
    assert!(app.lancement.agents.discussion.is_none());
    assert!(matches!(
        app.dock_manager.discussion.messages.last(),
        Some(Message::Erreur(_))
    ));
    assert_eq!(app.dock_manager.discussion.en_cours, None);
}

/// **Sans clé, aucun fil ne part** : la question reste, et le tiroir dit où la donner.
#[test]
fn test_discussion_sans_cle_dit_ou_la_donner() {
    let mut app = app_ouverte();
    for c in ["b", "o", "n"] {
        taper(&mut app, lettre(c));
    }
    assert!(taper(&mut app, Key::Named(NamedKey::Enter)));
    assert!(app.lancement.agents.discussion.is_none());
    assert_eq!(app.dock_manager.discussion.entree.texte, "bon");
    let Some(Message::Erreur(e)) = app.dock_manager.discussion.messages.last() else {
        panic!("une erreur dite");
    };
    assert!(e.contains("clé Google AI Studio"), "{e}");
}

#[test]
fn test_discussion_l_arret_se_demande_au_fil() {
    let mut app = app_ouverte();
    let (_tx, arret) = en_route(&mut app);
    app.agir_dans_la_discussion(crate::dock::discussion::DiscussionIntent::Arreter);
    assert!(arret.load(Ordering::Relaxed));
}
