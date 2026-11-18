//! ZEN-1 par les vrais gestes : `Maj+F11`, le clic sur l'indicateur, `Échap`, le menu du vide et la
//! palette.

use super::*;
use crate::commandes::Commande;
use crate::ui::context_menu::MenuAction;
use winit::event::{ElementState, MouseButton};
use winit::keyboard::{Key, NamedKey};

const ECRAN: (f32, f32) = (1440.0, 900.0);

fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    app
}

fn touche(app: &mut GlucoseApp, n: NamedKey) {
    app.handle_shortcut_input(&Key::Named(n), ElementState::Pressed);
}

/// `Maj+F11`, le geste du mode zen depuis FENETRE-1 (fiche 120).
fn maj_f11(app: &mut GlucoseApp) {
    app.modifiers = winit::keyboard::ModifiersState::SHIFT;
    touche(app, NamedKey::F11);
    app.modifiers = winit::keyboard::ModifiersState::empty();
}

fn cliquer(app: &mut GlucoseApp, (x, y): (f32, f32)) {
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(
        f64::from(x),
        f64::from(y),
    ));
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Left);
}

/// **`Maj+F11` cache toute l'interface, et la rend** : la bande et les onglets partent (le canevas
/// prend leur place), la couche des décisions se peint (l'indicateur, sur la carte graphique
/// aussi) ; la fenêtre, elle, n'est pas touchée.
#[test]
fn zen_1_maj_f11_cache_l_interface_et_la_rend() {
    let mut app = app();
    assert!(app.ui.header_height() > 0.0);
    maj_f11(&mut app);
    assert!(app.ui.zen && app.ui.sans_interface());
    assert_eq!(app.ui.header_height(), 0.0, "plus de bande ni d'onglets");
    assert!(
        app.ui.attend_une_decision(&app.store),
        "l'indicateur se peint"
    );
    maj_f11(&mut app);
    assert!(!app.ui.zen && !app.ui.sans_interface());
    assert!(app.ui.header_height() > 0.0, "la bande revient");
}

/// **L'indicateur se clique** — par le vrai chemin de la souris, au milieu de sa prise : le mode
/// se referme ; à côté de lui, le clic va au canevas et le mode reste.
#[test]
fn zen_1_l_indicateur_referme_le_mode() {
    let mut app = app();
    maj_f11(&mut app);
    let i = crate::ui::zen::placer(&app.ui, &app.renderer.typography, ECRAN);
    let (x, y, l, h) = i.prise;
    cliquer(&mut app, (x - 40.0, y + h + 40.0));
    assert!(app.ui.zen, "à côté de l'indicateur, le mode reste");
    cliquer(&mut app, (x + l / 2.0, y + h / 2.0));
    assert!(!app.ui.zen, "l'indicateur le referme");
}

/// **`Échap` referme le mode zen**, comme dans Tauri — après ce qui attend plus près (un menu
/// ouvert se ferme d'abord) ; et **le menu du vide et la palette** le basculent aussi.
#[test]
fn zen_1_echap_le_menu_et_la_palette() {
    let mut app = app();
    maj_f11(&mut app);
    app.ui.context_menu_at = Some((200.0, 200.0));
    touche(&mut app, NamedKey::Escape);
    assert!(app.ui.zen, "le menu d'abord");
    touche(&mut app, NamedKey::Escape);
    assert!(!app.ui.zen, "puis le mode zen");
    assert_eq!(MenuAction::ModeZen.commande(), Some(Commande::ModeZen));
    assert!(app.s_applique(Commande::ModeZen), "toujours offerte");
    assert!(app.executer(Commande::ModeZen));
    assert!(app.ui.zen);
}

/// **FENETRE-1 — `F11` est le plein écran, et se combine au zen** : la touche de tous les
/// logiciels de bureau passe la fenêtre en plein écran, l'interface restant ; `Maj+F11` cache
/// l'interface, la fenêtre restant en plein écran ; `F11` encore rend la fenêtre normale.
#[test]
fn fenetre_1_f11_est_le_plein_ecran_et_se_combine_au_zen() {
    use crate::ui::fenetre::ModeDeFenetre;
    let mut app = app();
    touche(&mut app, NamedKey::F11);
    assert_eq!(app.ui.fenetre, ModeDeFenetre::PleinEcran);
    assert!(
        !app.ui.zen && app.ui.header_height() > 0.0,
        "l'interface reste"
    );
    maj_f11(&mut app);
    assert!(
        app.ui.zen && app.ui.fenetre == ModeDeFenetre::PleinEcran,
        "les deux ensemble"
    );
    touche(&mut app, NamedKey::F11);
    assert_eq!(app.ui.fenetre, ModeDeFenetre::Normale);
    assert!(app.ui.zen, "le zen ne dépend pas de la fenêtre");
}
