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
