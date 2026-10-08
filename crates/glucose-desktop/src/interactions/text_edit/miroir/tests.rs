//! CLAVIER-1 à 3 — le clavier du système, miroir de la saisie.
//!
//! Le faux clavier fait ce que fait `GameActivity` : ce qu'on lui écrit part dans une file, et
//! n'est tenu que quand l'épreuve la **sert** — c'est ce qui rend CLAVIER-2 éprouvable.

use super::*;
use crate::app::GlucoseApp;
use crate::plateforme::clavier::{Clavier, EtatDuClavier};
use glucose_core::types::{Annotation, Viewport};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Interieur {
    tient: EtatDuClavier,
    file: Vec<EtatDuClavier>,
    sorti: bool,
}

/// Un clavier dont les écritures attendent qu'on serve sa file.
#[derive(Clone, Default)]
struct Faux(Arc<Mutex<Interieur>>);

impl Clavier for Faux {
    fn montrer(&self, etat: &EtatDuClavier) {
        let mut c = self.0.lock().unwrap();
        c.file.push(etat.clone());
        c.sorti = true;
    }
    fn ecrire(&self, etat: &EtatDuClavier) {
        self.0.lock().unwrap().file.push(etat.clone());
    }
    fn cacher(&self) {
        self.0.lock().unwrap().sorti = false;
    }
    fn lire(&self) -> EtatDuClavier {
        self.0.lock().unwrap().tient.clone()
    }
}

impl Faux {
    /// Le fil de l'interface d'Android passe : tout ce qui attendait est tenu, dans l'ordre.
    fn servir(&self) {
        let mut c = self.0.lock().unwrap();
        if let Some(dernier) = std::mem::take(&mut c.file).pop() {
            c.tient = dernier;
        }
    }
    /// L'utilisateur tape, ou une suggestion réécrit : le clavier tient un autre état.
    fn taper(&self, texte: &str, curseur: usize) {
        self.0.lock().unwrap().tient = EtatDuClavier {
            texte: texte.into(),
            selection: (curseur, curseur),
            composition: None,
        };
    }
    fn tient(&self) -> EtatDuClavier {
        self.0.lock().unwrap().tient.clone()
    }
    fn sorti(&self) -> bool {
        self.0.lock().unwrap().sorti
    }
}

/// Une carte de texte en saisie, à `y` dans le monde, avec un clavier de téléphone branché.
fn au_telephone(texte: &str, y: f64) -> (GlucoseApp, Faux) {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    let vue = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    app.store.set_viewport(&board, vue);
    let mut carte = Annotation::text("c", 0.0, y, texte);
    if let Annotation::Text { width, height, .. } = &mut carte {
        *width = Some(400.0);
        *height = Some(200.0);
    }
    app.store.add_annotation(&board, carte);
    let faux = Faux::default();
    app.lancement.clavier.brancher(Box::new(faux.clone()));
    app.start_text_edit("c".into(), texte.into());
    (app, faux)
}

fn texte_de(app: &GlucoseApp) -> &str {
    &app.editing_session.as_ref().expect("une saisie").buffer
}

// ── La frontière des unités ─────────────────────────────────────────────────

/// Java compte en `char` (UTF-16), Glucose en octets : « é » fait deux octets et une unité,
/// un emoji (U+1F600) quatre octets et deux unités. Il s'écrit échappé : la police d'Inter n'en
/// a pas le dessin, et FONT-1 refuse toute chaîne du crate qu'elle ne sait pas dessiner.
#[test]
fn test_clavier_1_les_positions_se_convertissent_entre_utf16_et_utf8() {
    let texte = "é\u{1F600}a";
    for (octet, unite) in [(0, 0), (2, 1), (6, 3), (7, 4)] {
        assert_eq!(utf8_vers_utf16(texte, octet), unite, "octet {octet}");
        assert_eq!(utf16_vers_utf8(texte, unite), octet, "unité {unite}");
    }
    assert_eq!(
        utf16_vers_utf8(texte, 2),
        2,
        "au milieu de l'emoji : le début du caractère, jamais un octet coupé"
    );
    assert_eq!(utf16_vers_utf8(texte, 99), texte.len(), "au-delà : la fin");
}

/// La plus petite réécriture, sur des caractères entiers.
#[test]
fn test_clavier_1_l_epissure_est_la_plus_petite_reecriture() {
    assert_eq!(epissure("abc", "abc"), (3..3, ""));
    assert_eq!(epissure("aa", "aaa"), (2..2, "a"), "l'ajout va à la fin");
    assert_eq!(epissure("bonjoue", "bonjour "), (6..7, "r "));
    assert_eq!(epissure("abc", "ac"), (1..2, ""));
    assert_eq!(epissure("le chat dort", "le chien dort"), (5..7, "ien"));
    assert_eq!(
        epissure("é", "è"),
        (0..2, "è"),
        "deux caractères qui partagent leur premier octet ne se coupent pas"
    );
    assert_eq!(epissure("", "salut"), (0..0, "salut"));
}

// ── Le miroir ───────────────────────────────────────────────────────────────

/// Une saisie qui s'ouvre sort le clavier sur son texte, le curseur compté en UTF-16.
#[test]
fn test_clavier_1_une_saisie_sort_le_clavier_sur_son_texte() {
    let (mut app, faux) = au_telephone("café", 0.0);
    app.suivre_le_clavier();
    faux.servir();
    assert!(faux.sorti());
    assert_eq!(faux.tient().texte, "café");
    assert_eq!(
        faux.tient().selection,
        (4, 4),
        "quatre unités, pas cinq octets"
    );
    app.commit_editing();
    app.suivre_le_clavier();
    assert!(!faux.sorti(), "la saisie finie rentre le clavier");
    assert!(!app.lancement.clavier.est_ouvert());
}

/// Ce que le clavier réécrit devient la saisie — et s'annule comme une frappe.
#[test]
fn test_clavier_1_une_reecriture_du_clavier_devient_la_saisie_et_s_annule() {
    let (mut app, faux) = au_telephone("le chat dort", 0.0);
    app.suivre_le_clavier();
    faux.servir();
    app.suivre_le_clavier();
    // Une suggestion remplace « chat » par « chien » ; le curseur reste après le mot.
    faux.taper("le chien dort", 8);
    app.suivre_le_clavier();
    assert_eq!(texte_de(&app), "le chien dort");
    let session = app.editing_session.as_ref().unwrap();
    assert_eq!(session.selection, Selection::at(8));
    // `Ctrl+Z` défait la réécriture, et le clavier reçoit le texte d'avant.
    app.apply_text_command(Command::Undo, false);
    assert_eq!(texte_de(&app), "le chat dort");
    app.suivre_le_clavier();
    faux.servir();
    assert_eq!(faux.tient().texte, "le chat dort");
    // Le curseur que rend le clavier compte en UTF-16 : après « à », quinze unités, seize octets.
    faux.taper("le chat dort là", 15);
    app.suivre_le_clavier();
    let session = app.editing_session.as_ref().unwrap();
    assert_eq!(session.selection, Selection::at("le chat dort là".len()));
}

/// **CLAVIER-2** : l'état que le clavier tenait avant un envoi ne défait pas ce que Glucose
/// vient d'écrire — ni à l'ouverture, ni quand un toucher déplace le curseur.
#[test]
fn test_clavier_2_l_etat_d_avant_l_envoi_ne_defait_rien() {
    let (mut app, faux) = au_telephone("abc", 0.0);
    app.suivre_le_clavier();
    // La file n'est pas servie : le clavier rend encore son texte vide d'avant.
    app.suivre_le_clavier();
    assert_eq!(
        texte_de(&app),
        "abc",
        "le vide d'avant n'efface pas la saisie"
    );
    faux.servir();
    app.suivre_le_clavier();
    assert_eq!(texte_de(&app), "abc");
    // Glucose pose le curseur au début ; le clavier rend encore la fin.
    app.editing_session.as_mut().unwrap().selection = Selection::at(0);
    app.suivre_le_clavier();
    app.suivre_le_clavier();
    assert_eq!(app.editing_session.as_ref().unwrap().selection.head, 0);
    faux.servir();
    assert_eq!(faux.tient().selection, (0, 0));
    // Et la frappe suivante est bien lue.
    faux.taper("xabc", 1);
    app.suivre_le_clavier();
    assert_eq!(texte_de(&app), "xabc");
}

/// Toucher le texte en saisie ressort le clavier, que le geste retour a rentré.
#[test]
fn test_clavier_1_toucher_le_texte_ressort_le_clavier() {
    let (mut app, faux) = au_telephone("le chat dort", 0.0);
    app.suivre_le_clavier();
    faux.servir();
    faux.0.lock().unwrap().sorti = false;
    assert!(app.click_text_at((30.0, 20.0), 1, false));
    app.suivre_le_clavier();
    assert!(faux.sorti());
}

// ── CLAVIER-3 ───────────────────────────────────────────────────────────────

/// La ligne qu'on écrit remonte juste au-dessus du bas de la fenêtre — du moins possible ; une
/// ligne visible ne bouge rien.
#[test]
fn test_clavier_3_la_ligne_ecrite_reste_au_dessus_du_bas() {
    let (mut app, _) = au_telephone("une ligne", 2000.0);
    app.garder_la_ligne_en_vue();
    let vue = app.store.active_board().unwrap().viewport;
    let hauteur = f64::from(text_box(400.0).line_height);
    let bas = 2000.0 + f64::from(TEXT_ORIGIN.1) + hauteur + vue.y;
    let plancher = f64::from(app.taille_de_la_fenetre().1);
    assert!(
        (bas - plancher).abs() < 1e-9,
        "le bas de la ligne au bas : {bas}"
    );
    // Au milieu de la fenêtre, rien ne bouge : le décalage le plus proche de zéro est zéro.
    let (mut app, _) = au_telephone("une ligne", 300.0);
    let vue = app.store.active_board().unwrap().viewport;
    app.garder_la_ligne_en_vue();
    assert_eq!(
        app.store.active_board().unwrap().viewport,
        vue,
        "une ligne déjà visible ne déplace rien"
    );
}

/// Une ligne passée au-dessus — sous la bande — redescend juste sous elle.
#[test]
fn test_clavier_3_la_ligne_ecrite_redescend_sous_la_bande() {
    let (mut app, _) = au_telephone("une ligne", -500.0);
    app.garder_la_ligne_en_vue();
    let vue = app.store.active_board().unwrap().viewport;
    let haut = -500.0 + f64::from(TEXT_ORIGIN.1) + vue.y;
    let plafond = f64::from(app.ui.header_height());
    assert!(
        (haut - plafond).abs() < 1e-9,
        "le haut de la ligne sous la bande : {haut}"
    );
}

/// **Le champ d'une question passe devant le nœud** (DOCUMENTS-2) : le clavier y écrit le nom
/// tout sélectionné, ce qu'on tape le remplace, et la question partie, il revient au nœud.
#[test]
fn test_clavier_1_le_champ_d_une_question_prend_le_clavier() {
    use crate::ui::question::{Champ, Question, Suite};
    let (mut app, faux) = au_telephone("le chat dort", 0.0);
    app.suivre_le_clavier();
    faux.servir();
    let question = Question {
        titre: "Renommer".into(),
        champ: Some(Champ {
            texte: "Canevas 1".into(),
            selection: Selection::all("Canevas 1"),
        }),
        ..Default::default()
    };
    app.demander(question, Suite::Renommer("x".into()));
    app.suivre_le_clavier();
    faux.servir();
    assert_eq!(faux.tient().texte, "Canevas 1");
    assert_eq!(faux.tient().selection, (0, 9), "tout sélectionné");
    faux.taper("Mary", 4);
    app.suivre_le_clavier();
    let (question, _) = app.ui.question.as_ref().expect("la question");
    let champ = question.champ.as_ref().expect("le champ");
    assert_eq!(champ.texte, "Mary");
    assert_eq!(champ.selection, Selection::at(4));
    assert_eq!(texte_de(&app), "le chat dort", "le nœud n'a rien reçu");
    app.ui.question = None;
    app.suivre_le_clavier();
    faux.servir();
    assert_eq!(
        faux.tient().texte,
        "le chat dort",
        "le clavier revient au nœud"
    );
}

/// **Toucher le champ d'une question ressort le clavier**, que le geste retour a rentré.
#[test]
fn test_clavier_1_toucher_le_champ_ressort_le_clavier() {
    use crate::ui::question::{placer, Champ, Question, Suite};
    let (mut app, faux) = au_telephone("le chat dort", 0.0);
    let question = Question {
        titre: "Renommer".into(),
        champ: Some(Champ {
            texte: "Canevas 1".into(),
            selection: Selection::at(9),
        }),
        ..Default::default()
    };
    app.demander(question.clone(), Suite::Renommer("x".into()));
    app.suivre_le_clavier();
    faux.servir();
    faux.0.lock().unwrap().sorti = false;
    let (w, h) = app.taille_de_la_fenetre();
    let placee = placer(
        &question,
        &app.renderer.typography,
        (w, h),
        app.ui.scale_factor,
    );
    let ((x, y, cw, ch), _) = placee.champ.expect("le champ");
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(
        f64::from(x + cw / 2.0),
        f64::from(y + ch / 2.0),
    ));
    app.handle_mouse_down(winit::event::MouseButton::Left, w, h);
    app.handle_mouse_up(winit::event::MouseButton::Left);
    app.suivre_le_clavier();
    assert!(faux.sorti(), "le clavier ressort");
    assert!(app.ui.question.is_some(), "et la question attend toujours");
}
