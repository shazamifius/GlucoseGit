//! Ce que le geste promet : les bandes partent, le contenu ne bouge pas, et un `Ctrl+Z` rend
//! tout le lot d'un coup.

use super::*;
use crate::renderer::magasin::Entree;
use crate::renderer::photo::Pyramide;
use glucose_core::types::BoardImage;
use tiny_skia::Pixmap;

/// Une application sans rien dessus.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app.store.journal.clear();
    app
}

/// Une photo de 100 × 100 dont les vingt colonnes de gauche et les dix lignes du haut sont
/// noires, décodée dans le magasin sous ce nom.
fn decoder_a_bandes(app: &mut GlucoseApp, src: &str) {
    let mut pixmap = Pixmap::new(100, 100).expect("une image");
    let largeur = pixmap.width() as usize;
    for (i, bloc) in pixmap
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        let (x, y) = (i % largeur, i / largeur);
        let couleur = if x < 20 || y < 10 {
            [0, 0, 0, 255]
        } else if (x / 3 + y / 3) % 2 == 0 {
            [200, 60, 30, 255]
        } else {
            [30, 90, 200, 255]
        };
        bloc.copy_from_slice(&couleur);
    }
    app.renderer.magasin.cache.insert(
        src.to_string(),
        Entree::pour_test(Pyramide::nouvelle(pixmap)),
    );
}

/// Pose une image de 200 × 200 centrée en `(cx, cy)` qui porte ce fichier, et la sélectionne.
fn poser(app: &mut GlucoseApp, id: &str, src: &str, cx: f64, cy: f64) {
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new(id, cx, cy, 200.0, 200.0);
    img.src = Some(src.to_string());
    app.store.add_image(&board, img);
}

fn image<'a>(app: &'a GlucoseApp, id: &str) -> &'a BoardImage {
    let board = &app.store.project.active_board_id;
    app.store.image(board, id).expect("l'image existe")
}

/// **Les bandes partent, et ce qu'on garde ne bouge pas d'un pixel.**
///
/// Vingt colonnes sur cent à gauche, dix lignes sur cent en haut : la boîte de 200 × 200
/// perd quarante unités de largeur et vingt de hauteur, par la gauche et par le haut. Le coin
/// bas-droit, qui était du contenu, reste exactement où il était.
#[test]
fn test_les_bandes_partent_et_le_contenu_ne_bouge_pas() {
    let mut app = app();
    decoder_a_bandes(&mut app, "bandes.png");
    poser(&mut app, "i", "bandes.png", 300.0, 300.0);
    let avant = image(&app, "i").rect();
    assert_eq!((avant.left, avant.top), (200.0, 200.0));

    app.retirer_les_bordures_de_la_selection();

    let img = image(&app, "i");
    let (g, h, d, b) = img.crop.marges();
    assert!(
        (g - 0.2).abs() < 1e-9 && (h - 0.1).abs() < 1e-9,
        "marges : {g} {h}"
    );
    assert_eq!((d, b), (0.0, 0.0));
    let apres = img.rect();
    assert!(
        (apres.width - 160.0).abs() < 1e-9,
        "largeur : {}",
        apres.width
    );
    assert!(
        (apres.height - 180.0).abs() < 1e-9,
        "hauteur : {}",
        apres.height
    );
    // Le bord gauche a reculé de quarante, le haut de vingt ; le coin bas-droit n'a pas bougé.
    assert!((apres.left - 240.0).abs() < 1e-9, "gauche : {}", apres.left);
    assert!((apres.top - 220.0).abs() < 1e-9, "haut : {}", apres.top);
    assert!(
        (apres.left + apres.width - 400.0).abs() < 1e-9
            && (apres.top + apres.height - 400.0).abs() < 1e-9,
        "le coin bas-droit doit rester en (400, 400)"
    );
}

/// **Un lot est une entrée d'annulation** : trois images recadrées, un `Ctrl+Z`, et les trois
/// reviennent entières, à leur place.
#[test]
fn test_un_lot_s_annule_d_un_coup() {
    let mut app = app();
    decoder_a_bandes(&mut app, "bandes.png");
    for (i, cx) in [100.0, 400.0, 700.0].iter().enumerate() {
        poser(&mut app, &format!("i{i}"), "bandes.png", *cx, 300.0);
    }
    app.store.selected_image_ids = vec!["i0".into(), "i1".into(), "i2".into()];
    // Les trois poses sont trois entrees : on ne veut compter que celle du geste.
    app.store.journal.clear();
    let avant: Vec<BoardImage> = (0..3)
        .map(|i| image(&app, &format!("i{i}")).clone())
        .collect();

    app.retirer_les_bordures_de_la_selection();
    for i in 0..3 {
        assert!(
            !image(&app, &format!("i{i}")).crop.est_entier(),
            "i{i} doit etre cadree"
        );
    }

    assert!(app.store.undo(), "il y a quelque chose a annuler");
    for (i, attendue) in avant.iter().enumerate() {
        assert_eq!(
            image(&app, &format!("i{i}")),
            attendue,
            "i{i} doit etre revenue"
        );
    }
    assert!(!app.store.undo(), "un seul geste : une seule entree");
}

/// **Une image déjà nette ne change pas, et une image sans pixels non plus** — et le
/// compte-rendu le dit au lieu de laisser croire que rien ne s'est passé.
#[test]
fn test_ce_qui_ne_change_pas_se_dit() {
    let mut app = app();
    // Une photo sans bande : un damier, en x ET en y. La premiere version de ce test ne
    // variait qu'en x et faisait cinq colonnes unies a gauche -- le detecteur les a trouvees,
    // et il avait raison.
    let mut nette = Pixmap::new(50, 50).expect("une image");
    for (i, bloc) in nette
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        let (x, y) = (i % 50, i / 50);
        let v = if (x / 3 + y / 3) % 2 == 0 { 200 } else { 40 };
        bloc.copy_from_slice(&[v, 60, 30, 255]);
    }
    app.renderer.magasin.cache.insert(
        "nette.png".to_string(),
        Entree::pour_test(Pyramide::nouvelle(nette)),
    );
    poser(&mut app, "nette", "nette.png", 100.0, 100.0);
    poser(&mut app, "absente", "pas-decodee.png", 400.0, 100.0);
    app.store.selected_image_ids = vec!["nette".into(), "absente".into()];

    app.retirer_les_bordures_de_la_selection();
    assert!(image(&app, "nette").crop.est_entier());
    assert!(image(&app, "absente").crop.est_entier());
    assert_eq!(
        compte_rendu(0, 1, 1),
        "1 sans bordure, 1 pas encore décodée"
    );
    assert_eq!(compte_rendu(3, 0, 0), "3 images recadrées");
    assert_eq!(compte_rendu(1, 2, 0), "1 image recadrée, 2 sans bordure");
}

/// **Une image tournée recule le long de son propre axe**, pas de l'axe de l'écran.
///
/// Un quart de tour : retirer une bande « à gauche » de l'image la fait reculer vers le bas
/// de l'écran. Le décalage du centre tourne avec elle.
#[test]
fn test_une_image_tournee_recule_le_long_de_son_axe() {
    let mut app = app();
    decoder_a_bandes(&mut app, "bandes.png");
    poser(&mut app, "i", "bandes.png", 300.0, 300.0);
    let board = app.store.project.active_board_id.clone();
    app.store.update_image(&board, "i", |img| {
        img.rotation = std::f64::consts::FRAC_PI_2
    });
    app.store.journal.clear();

    app.retirer_les_bordures_de_la_selection();
    let img = image(&app, "i");
    // Sans rotation, le centre irait en (320, 310) : +20 en x, +10 en y. Tourné d'un quart
    // de tour (x → y, y → −x), il va en (300 − 10, 300 + 20) = (290, 320).
    assert!((img.x - 290.0).abs() < 1e-9, "x : {}", img.x);
    assert!((img.y - 320.0).abs() < 1e-9, "y : {}", img.y);
}

/// **Un lot dont l'original est chez le système attend, puis s'applique d'un seul bloc**
/// (ETAGES-1).
///
/// La photo est loin de l'écran, et son original a été offert : rien ne le lit. `Ctrl+B` ne
/// change donc rien tout de suite — il le redemande et attend. L'application ne fait passer
/// une image que lorsqu'elle demande à se réveiller, comme la vraie boucle : c'est ce qui
/// attrape un lot qui attendrait sans que rien ne le fasse repasser.
#[test]
fn test_un_lot_attend_ses_originaux_puis_s_applique_d_un_bloc() {
    use crate::renderer::photo::Etat;
    let mut app = app();
    decoder_a_bandes(&mut app, "bandes.png");
    poser(&mut app, "i", "bandes.png", 100_000.0, 100_000.0);
    app.store.selected_image_ids = vec!["i".into()];
    app.store.journal.clear();
    // Le mot d'accueil réveillerait l'application de lui-même, et cacherait un lot qui ne
    // la réveille pas.
    app.ui.current_toast = None;

    // Une image du rendu où rien ne lit l'original : il part chez le système. Puis une image
    // de repos : l'offre est rentrée, et plus rien ne fait revoir cette photo hors de l'écran.
    let magasin = &mut app.renderer.magasin;
    magasin.ouvrir();
    magasin.reclamer("bandes.png", 200.0);
    magasin.fermer();
    magasin.attendre_le_chantier();
    magasin.ouvrir();
    magasin.fermer();
    assert_eq!(magasin.cache["bandes.png"].pyramide.etat(0), Etat::Offert);

    app.retirer_les_bordures_de_la_selection();
    assert!(
        image(&app, "i").crop.est_entier(),
        "rien ne change tant que l'original n'est pas revenu"
    );

    for _ in 0..1000 {
        if app.bordures_en_attente.is_none() || app.prochain_reveil().is_none() {
            break;
        }
        app.une_image_sans_fenetre((800, 600));
        app.renderer.magasin.attendre_le_chantier();
    }
    assert!(
        app.bordures_en_attente.is_none(),
        "le lot attend encore : plus rien ne l'a fait repasser"
    );
    assert!(
        !image(&app, "i").crop.est_entier(),
        "les bandes sont parties"
    );
    assert!(app.store.undo(), "il y a quelque chose a annuler");
    assert!(image(&app, "i").crop.est_entier(), "et un Ctrl+Z les rend");
    assert!(!app.store.undo(), "un seul geste : une seule entree");
}
