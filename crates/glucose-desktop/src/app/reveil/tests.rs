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

const ECRAN: (u32, u32) = (800, 600);

/// Une application neuve, sans le message d'accueil qui s'efface et demande ses images.
fn au_calme() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    app
}

/// **Ce que la boucle fait en s'endormant** : la peinture a consommé la salissure, puis chaque
/// raison dit ce qu'elle attend. Rend vrai si l'une d'elles a demandé une image à repeindre.
fn demande_t_on_une_image_a_repeindre(app: &mut GlucoseApp) -> bool {
    let _ = app.prendre_la_salissure();
    let _ = app.prochain_reveil();
    !app.prendre_la_salissure().est_propre()
}

/// **NETTETE-1 — la dernière image d'un geste appelle son image d'arrêt.**
///
/// À la molette, l'élan s'éteint dans l'image même qui montre le cran (NAV-4) : cette image
/// est une image de **mouvement** — les textes rendus au palier dyadique le plus proche et
/// agrandis par la carte jusqu'à √2, la scène que la voie processeur a le droit d'abîmer. Rien
/// ne demandait l'image suivante, et Glucose s'endormait sur elle : le texte restait flou à
/// tous les zooms éloignés d'une puissance de deux, jusqu'au prochain geste (son retour du
/// 12/10 : « des niveaux de zoom qui restent ultra floutés, même pour du texte »).
#[test]
fn test_nettete_1_la_derniere_image_d_un_geste_appelle_son_image_d_arret() {
    let mut app = au_calme();
    app.handle_mouse_wheel(winit::event::MouseScrollDelta::LineDelta(0.0, 1.0));
    app.bouger_la_camera(ECRAN.0, ECRAN.1);
    assert!(
        app.la_vue_bouge() && !app.elan.en_cours(),
        "le montage : un cran montre en entier, dans une image de mouvement"
    );
    assert!(
        demande_t_on_une_image_a_repeindre(&mut app),
        "la derniere image d'un geste s'endort sur son regime de mouvement"
    );

    app.bouger_la_camera(ECRAN.0, ECRAN.1);
    assert!(!app.la_vue_bouge(), "l'image demandee est a l'arret");
    assert!(
        !demande_t_on_une_image_a_repeindre(&mut app),
        "apres l'image d'arret, le repos : plus aucune image"
    );
}

/// Une finesse réduite, comme la voie processeur la laisse après un geste lourd.
fn une_finesse_reduite() -> crate::resolution::Resolution {
    let mut r = crate::resolution::Resolution::nette();
    // Une scène de 30 ms pour un budget de 8 : la loi demande le palier 4.
    r.observer(
        crate::resolution::Mesure {
            image: std::time::Duration::from_millis(32),
            scene: std::time::Duration::from_millis(30),
        },
        std::time::Duration::from_millis(8),
        true,
        4,
    );
    assert!(r.reduite(), "le montage part d'une scene reduite");
    r
}

/// **NETTETE-1 — sur la voie processeur, la finesse remonte jusqu'à être nette.**
///
/// À l'arrêt, le facteur de réduction redescend **par moitiés**, une par image, pour ne pas
/// payer d'un coup l'image chère au moment où l'œil se pose (`resolution.rs`). Mais aucune
/// raison ne demandait ces images : la scène restait agrandie au plus proche — des blocs de
/// deux, quatre ou huit pixels, les « carrés qui ne se défloutent pas » — jusqu'au geste
/// suivant. Chaque image en demande une autre tant qu'il reste à remonter, et plus après.
#[test]
fn test_nettete_1_la_finesse_reduite_remonte_jusqu_a_etre_nette() {
    let mut app = au_calme();
    app.presenter = None;
    app.resolution = une_finesse_reduite();
    let mut images = 0;
    while app.resolution.reduite() {
        assert!(
            demande_t_on_une_image_a_repeindre(&mut app),
            "la scene reste reduite au facteur {} et rien ne demande l'image qui la remonte",
            app.resolution.facteur()
        );
        app.accorder_la_finesse(std::time::Duration::from_millis(1));
        images += 1;
        assert!(images <= 3, "un facteur de quatre remonte en deux images");
    }
    assert!(
        !demande_t_on_une_image_a_repeindre(&mut app),
        "nette, la scene ne demande plus rien"
    );
}

/// Une présentation qui pose les photos et dit si sa scène est finie — la seule chose que la
/// boucle lit d'elle ici.
struct Carte {
    inachevee: std::rc::Rc<std::cell::Cell<bool>>,
}

impl crate::present::Presenter for Carte {
    fn resize(
        &mut self,
        _: std::num::NonZeroU32,
        _: std::num::NonZeroU32,
    ) -> crate::error::DesktopResult<()> {
        Ok(())
    }
    fn present(
        &mut self,
        _: &tiny_skia::Pixmap,
    ) -> crate::error::DesktopResult<crate::present::Issue> {
        Ok(crate::present::Issue::Presentee)
    }
    fn pose_les_photos(&self) -> bool {
        true
    }
    fn scene_inachevee(&self) -> bool {
        self.inachevee.get()
    }
    fn nom(&self) -> &'static str {
        "carte factice"
    }
}

/// **NETTETE-1 — une scène que la carte n'a pas finie redemande une image, sans rien
/// repeindre.**
///
/// La cascade rend au moins une texture par image et reporte ce que le budget ne permet pas
/// (CASCADE-2, 3) : les tuiles d'un texte vu de près, les cartes qui changent de palier. Elle
/// posait leur repli — l'ancien palier, ou le texte entier à un palier bien plus bas — et
/// rien ne la rappelait : des carrés flous restaient posés. Il faut l'image suivante, mais
/// **pas** une peinture : les couches du processeur sont justes, seules des textures manquent,
/// et les repeindre coûterait à chaque image ce que le téléphone n'a pas.
#[test]
fn test_nettete_1_une_scene_inachevee_redemande_une_image_sans_rien_repeindre() {
    let mut app = au_calme();
    let inachevee = std::rc::Rc::new(std::cell::Cell::new(true));
    app.presenter = Some(Box::new(Carte {
        inachevee: std::rc::Rc::clone(&inachevee),
    }));
    let avant = app.provenance.demandes();
    assert!(
        !demande_t_on_une_image_a_repeindre(&mut app),
        "rien n'est a repeindre : seules des textures manquent"
    );
    assert!(
        app.provenance.demandes() > avant,
        "une scene inachevee doit redemander une image"
    );
    une_image_apres_l_endormissement(&mut app);
    assert_eq!(
        images_de(&app, Raison::Nettete),
        1,
        "et la chronique doit dire a qui l'image est due"
    );

    inachevee.set(false);
    let avant = app.provenance.demandes();
    let _ = app.prochain_reveil();
    assert_eq!(
        app.provenance.demandes(),
        avant,
        "une scene finie ne demande plus rien"
    );
}
