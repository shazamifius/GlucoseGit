//! DOMAINE-1 : regarder un domaine par le bouton « voir » du panneau, sur le **vrai** `GlucoseApp`.

use super::tests::{app_with_panel, create_domain};
use super::*;
use crate::commandes::Commande;
use crate::dock::domains::{hit_domains_panel, layout_domains_panel, DomainIntent};
use crate::dock::{compute_panel_layouts, TabId};
use crate::params::{Pointer, ScaledRect};

/// Le cadre du panneau Domaines ouvert, à l'écran de 1 440 × 900.
fn cadre(app: &GlucoseApp) -> ScaledRect {
    let boites = compute_panel_layouts(&app.dock_manager, 1440.0, 900.0, 78.0, 1.0);
    let b = boites
        .iter()
        .find(|b| b.tab == TabId::Domains)
        .expect("onglet ouvert");
    ScaledRect {
        x: b.x,
        y: b.y,
        w: b.width,
        h: b.height,
        scale: 1.0,
    }
}

/// Le clic au milieu du bouton « voir » de la première rangée, traduit par le panneau lui-même.
fn cliquer_voir(app: &mut GlucoseApp) {
    let layout = layout_domains_panel(cadre(app), &app.store, &app.dock_manager.domains);
    let voir = layout.rows[0].voir;
    let pointeur = Pointer {
        x: voir.x + voir.w / 2.0,
        y: voir.y + voir.h / 2.0,
    };
    let geste = hit_domains_panel(&layout, &app.store, pointeur, false).expect("« voir » répond");
    assert!(matches!(geste, DomainIntent::Regarder(_)), "{geste:?}");
    app.apply_domain_intent(geste);
}

/// **« voir » regarde le domaine, un second clic le rend** — sans sélection (regarder n'assigne
/// rien), sans geste au journal (une lentille, comme la frise) ; la palette
/// offre « Ne plus regarder le domaine » tant qu'il est regardé, et l'exécuter le retire ;
/// **supprimer le domaine regardé retire la lentille** (NAV-1 : sinon tout pâlirait), et
/// `Ctrl+Z` rend le domaine, pas la lentille.
#[test]
fn domaine_1_voir_regarde_le_domaine() {
    let mut app = app_with_panel();
    let id = create_domain(&mut app);
    let profondeur = app.store.undo_depth();
    assert!(
        !app.s_applique(Commande::NePlusRegarder),
        "rien n'est regardé"
    );

    cliquer_voir(&mut app);
    assert_eq!(app.store.domaine_regarde.as_deref(), Some(id.as_str()));
    assert_eq!(app.store.undo_depth(), profondeur, "aucun geste au journal");
    assert!(app.s_applique(Commande::NePlusRegarder));

    cliquer_voir(&mut app);
    assert_eq!(app.store.domaine_regarde, None, "un second clic le rend");

    cliquer_voir(&mut app);
    assert!(app.executer(Commande::NePlusRegarder));
    assert_eq!(app.store.domaine_regarde, None, "la palette le retire");

    cliquer_voir(&mut app);
    app.apply_domain_intent(DomainIntent::ConfirmDelete(id.clone()));
    assert!(app.store.project.domains.iter().all(|d| d.id != id));
    assert_eq!(
        app.store.domaine_regarde, None,
        "un domaine parti n'est plus regardé"
    );
    assert!(app.store.undo());
    assert!(app.store.project.domains.iter().any(|d| d.id == id));
    assert_eq!(
        app.store.domaine_regarde, None,
        "Ctrl+Z rend le domaine, pas la lentille"
    );
}
