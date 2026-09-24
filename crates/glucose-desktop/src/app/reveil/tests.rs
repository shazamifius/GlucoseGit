//! Ce que la provenance des images garantit : chaque image dit ce qui l'a demandée, et le
//! masque des raisons survit jusqu'à l'image qu'il provoque — ce qu'il ne faisait pas.

use super::*;

/// Combien d'images la chronique a portées au compte de cette raison.
fn images_de(app: &GlucoseApp, raison: Raison) -> u64 {
    app.chronique
        .reveils()
        .find(|(r, _)| *r == raison)
        .map_or(0, |(_, n)| n)
}

/// L'ordre réel d'une image demandée au repos : la boucle s'endort en notant ses raisons,
/// l'image commence — et vide ses compteurs —, puis la chronique l'enregistre.
fn une_image_apres_l_endormissement(app: &mut GlucoseApp) {
    let _ = app.prochain_reveil();
    crate::perf::frame_begin();
    app.enregistrer_l_image(1_000, (800, 600));
}

/// **Le masque des raisons survit jusqu'à l'image qu'il provoque.**
///
/// Il se notait comme un compteur de l'image, entre deux images ; `frame_begin` l'effaçait
/// avant qu'il soit lu, et la chronique répondait « aucune » à toutes les sessions — y compris
/// celle qui dessinait treize images par seconde sans qu'on y touche (fiche 29 § 4.4). Ce test
/// rejoue l'ordre réel, et c'est l'ordre qui cachait le défaut.
#[test]
fn test_le_masque_des_raisons_survit_jusqu_a_l_image_qu_il_provoque() {
    let mut app = GlucoseApp::new();
    app.ui.show_toast("Un message qui s'efface");
    une_image_apres_l_endormissement(&mut app);
    assert_eq!(
        images_de(&app, Raison::Toast),
        1,
        "l'image doit au toast, et la chronique doit le dire"
    );
}

/// **Une image que rien de connu ne demande est portée au compte du système** : c'est la
/// ligne qui accuse, au lieu d'un silence.
#[test]
fn test_une_image_que_rien_de_connu_ne_demande_est_au_compte_du_systeme() {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    une_image_apres_l_endormissement(&mut app);
    assert_eq!(images_de(&app, Raison::Systeme), 1);
    assert_eq!(images_de(&app, Raison::Main), 0, "personne n'a touche");
}

/// **Un survol est la main**, et il passe par l'aiguillage des événements comme un clic.
///
/// Il était classé « repos », faute de geste en cours : la veille comptait alors les images
/// qu'il faisait redessiner comme dessinées sans que personne n'y touche.
#[test]
fn test_un_survol_est_la_main() {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    let survol = winit::event::WindowEvent::CursorMoved {
        device_id: winit::event::DeviceId::dummy(),
        position: winit::dpi::PhysicalPosition::new(400.0, 300.0),
    };
    assert!(
        app.evenement_de_la_main(&survol),
        "un survol appartient a la main"
    );
    assert_eq!(
        app.provenance.evenements_de_la_main(),
        1,
        "la veille compte les evenements, et celui-ci en est un"
    );
    une_image_apres_l_endormissement(&mut app);
    assert_eq!(
        images_de(&app, Raison::Main),
        1,
        "l'image suivante doit a la main"
    );
    assert_eq!(images_de(&app, Raison::Systeme), 0);
    // Et ce qui est arrivé ne se reporte pas sur l'image d'après.
    une_image_apres_l_endormissement(&mut app);
    assert_eq!(
        images_de(&app, Raison::Main),
        1,
        "la main ne compte qu'une fois"
    );
    assert_eq!(images_de(&app, Raison::Systeme), 1);
}

/// **Un message endormi sur son plateau ne demande pas les images des autres.**
///
/// Le masque notait toute raison active : pendant les deux secondes où un message reste
/// immobile, chaque image que l'élan ou la main dessinait lui était portée — « message à
/// l'écran : 56,9 % » sur la longue session du 23/09, pour un message qui n'en demandait
/// presque aucune. Il ne compte plus que s'il redessine lui-même : à son entrée, à sa sortie.
#[test]
fn test_un_message_endormi_ne_demande_pas_les_images_des_autres() {
    let mut app = GlucoseApp::new();
    app.ui.show_toast("Un message au repos");
    if let Some(toast) = app.ui.current_toast.as_mut() {
        toast.created_at = std::time::Instant::now() - std::time::Duration::from_millis(1000);
    }
    let _ = app.prochain_reveil();
    // Une image que la main demande, pendant que le message dort.
    app.provenance.noter_la_main();
    crate::perf::frame_begin();
    app.enregistrer_l_image(1_000, (800, 600));
    assert_eq!(
        images_de(&app, Raison::Toast),
        0,
        "le message dormait : l'image est a la main, pas a lui"
    );
    assert_eq!(images_de(&app, Raison::Main), 1);
}

