//! Les lois du banc. Elles ne portent pas sur la vitesse — une machine lente ne doit pas faire
//! échouer un test — mais sur ce que le banc garantit : rendre vraiment, et rendre la même
//! chose deux fois.

use super::*;
use glucose_core::synth::{self, Shape};

/// **Une frame rendue sans fenêtre contient vraiment quelque chose.** Le piège serait un
/// pixmap uni : tout aurait l'air de marcher, et rien ne serait dessiné.
#[test]
fn test_une_frame_hors_ecran_dessine_vraiment() {
    let store = synth::witness();
    let png = capture(&store, 800, 600);
    assert!(png.len() > 1_000, "un PNG de {} octets est vide", png.len());

    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    let pixmap = render_frame(&mut renderer, &mut ui, &store, 800, 600);
    let couleurs: std::collections::HashSet<_> =
        pixmap.pixels().iter().map(|p| (p.red(), p.green(), p.blue())).collect();
    assert!(couleurs.len() > 8, "seulement {} couleurs distinctes", couleurs.len());
}

/// Combien de pixels diffèrent entre deux images de même taille.
///
/// Comparer deux tranches d'octets avec `assert_eq!` afficherait les huit mégaoctets des deux
/// côtés au moindre échec — illisible, et assez volumineux pour noyer la sortie de test.
fn pixels_differents(a: &[u8], b: &[u8]) -> usize {
    assert_eq!(a.len(), b.len(), "deux images de tailles différentes");
    (0..a.len()).step_by(4).filter(|&i| a[i..i + 4] != b[i..i + 4]).count()
}

/// **La même scène rend le même pixmap, au bit près.** Sans cette propriété, une capture de
/// référence ne prouverait rien : elle changerait d'une exécution à l'autre.
#[test]
fn test_la_meme_scene_rend_le_meme_pixmap() {
    let store = synth::witness();
    let mut r1 = Renderer::new();
    let mut u1 = UiState::new();
    let mut r2 = Renderer::new();
    let mut u2 = UiState::new();

    let a = render_frame(&mut r1, &mut u1, &store, 640, 480);
    let b = render_frame(&mut r2, &mut u2, &store, 640, 480);
    let d = pixels_differents(a.data(), b.data());
    assert_eq!(d, 0, "deux rendus de la même scène diffèrent de {d} pixels");
}

/// Rendre deux fois avec le **même** moteur donne aussi le même résultat : les caches de
/// glyphes et de teintes ne doivent pas changer ce qui s'affiche, seulement sa vitesse.
///
/// Ce test a échoué le jour où il a été écrit, et c'est ce qui a fait découvrir que
/// `UiState::new` posait un toast — donc une horloge — dans l'état de l'interface.
#[test]
fn test_les_caches_ne_changent_pas_ce_qui_s_affiche() {
    let store = synth::witness();
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();

    let premiere = render_frame(&mut renderer, &mut ui, &store, 640, 480);
    for tour in 1..=3 {
        let suivante = render_frame(&mut renderer, &mut ui, &store, 640, 480);
        let d = pixels_differents(premiere.data(), suivante.data());
        assert_eq!(d, 0, "la frame {tour} diffère de la première de {d} pixels");
    }
}

/// Un état d'interface neuf n'affiche **aucun** message : le mot d'accueil est posé au
/// démarrage de l'application, pas par le constructeur. C'est ce qui rend toute capture
/// reproductible, puisqu'un toast porte une horloge.
#[test]
fn test_un_etat_d_interface_neuf_n_affiche_aucun_message() {
    assert!(UiState::new().current_toast.is_none());
}

/// Le cadrage met le contenu à l'écran : après un appel à `frame_document`, le barycentre du
/// document tombe au centre de la fenêtre.
#[test]
fn test_le_cadrage_met_le_document_au_centre() {
    let mut store = synth::document(200, 4_000.0, Shape::Uniform, 1);
    frame_document(&mut store, 0.5, 1920, 1080);
    let vp = store.active_board().expect("un tableau").viewport;

    let board = store.active_board().expect("un tableau");
    let (mut cx, mut cy, mut n) = (0.0, 0.0, 0.0);
    for a in &board.annotations {
        cx += a.x();
        cy += a.y();
        n += 1.0;
    }
    for img in &board.images {
        cx += img.x;
        cy += img.y;
        n += 1.0;
    }
    let (ecran_x, ecran_y) = (cx / n * vp.scale + vp.x, cy / n * vp.scale + vp.y);
    assert!((ecran_x - 960.0).abs() < 1.0, "centre en x : {ecran_x}");
    assert!((ecran_y - 540.0).abs() < 1.0, "centre en y : {ecran_y}");
}

/// La mesure rend des statistiques cohérentes, et n'inclut pas la frame de chauffe.
#[test]
fn test_la_mesure_rend_des_statistiques_coherentes() {
    let store = synth::witness();
    let s = measure(&store, 400, 300, 5);
    assert_eq!(s.frames, 5, "la chauffe n'est pas comptée");
    assert!(s.min_ms <= s.median_ms, "min {} > médiane {}", s.min_ms, s.median_ms);
    assert!(s.median_ms <= s.max_ms, "médiane {} > max {}", s.median_ms, s.max_ms);
    assert!(s.min_ms >= 0.0 && s.max_ms.is_finite());
    assert!(s.fps() > 0.0);
}

/// Une frame de durée nulle ne fait pas diviser par zéro, et le budget se lit dans les deux
/// sens.
#[test]
fn test_le_budget_se_lit_dans_les_deux_sens() {
    let rapide = Stats { frames: 1, min_ms: 1.0, median_ms: 4.0, max_ms: 9.0 };
    assert!(rapide.within_budget());
    assert!((rapide.fps() - 250.0).abs() < 1e-9);

    let lent = Stats { frames: 1, min_ms: 10.0, median_ms: 40.0, max_ms: 90.0 };
    assert!(!lent.within_budget());
    assert!((lent.fps() - 25.0).abs() < 1e-9);

    let nul = Stats { frames: 1, min_ms: 0.0, median_ms: 0.0, max_ms: 0.0 };
    assert!(nul.fps().is_infinite(), "pas de division par zéro");
}

/// Les trois définitions de référence sont là, et la 4K en fait partie — c'est une exigence
/// de la charte, pas un détail de configuration.
#[test]
fn test_les_definitions_de_reference_incluent_la_4k() {
    assert!(DEFINITIONS.iter().any(|(nom, w, h)| *nom == "4K" && *w == 3840 && *h == 2160));
    assert_eq!(BUDGET_MS, 10.0, "cent images par seconde");
}

/// **L'empreinte de la scène témoin.** Elle change dès qu'un pixel change, et c'est tout son
/// intérêt : aucun autre test ne voit une carte posée trop bas, une puce mal alignée ou un
/// titre à la mauvaise taille.
///
/// Quand ce test échoue, le geste est toujours le même, et il compte plus que la valeur :
///
/// ```text
/// cargo run -p glucose-desktop --example capture_temoin
/// ```
///
/// puis **ouvrir le PNG et le regarder**. Si le changement est celui qu'on voulait, on met
/// l'empreinte ci-dessous à jour ; sinon, on vient de trouver une régression que rien d'autre
/// n'aurait signalée.
///
/// L'empreinte est gardée ici plutôt qu'une image de référence dans le dépôt : le rendu va
/// changer souvent, et une image versionnée à chaque fois ferait grossir l'historique sans
/// rien apprendre de plus.
const EMPREINTE_TEMOIN: &str = "d02f5dcfe7a2df0e";

#[test]
fn test_l_empreinte_de_la_scene_temoin_n_a_pas_change() {
    let (w, h) = synth::WITNESS_SIZE;
    let png = capture(&synth::witness(), w, h);
    let obtenue = &glucose_core::hash::hex_of(&glucose_core::hash::sha256(&png))[..16];
    assert_eq!(
        obtenue, EMPREINTE_TEMOIN,
        "le rendu de la scène témoin a changé — lancer `cargo run -p glucose-desktop \
         --example capture_temoin`, regarder le PNG, puis mettre EMPREINTE_TEMOIN à jour si \
         le changement est voulu"
    );
}

/// Le cadrage de la scène témoin met vraiment tout son contenu dans la fenêtre, sous le
/// bandeau. C'est ce qui rend la capture utile : une scène mal cadrée ne prouverait que le
/// fond du canevas.
#[test]
fn test_le_cadrage_du_temoin_montre_tout_son_contenu() {
    let store = synth::witness();
    let (w, h) = synth::WITNESS_SIZE;
    let (x0, y0, x1, y1) = synth::WITNESS_CONTENT;
    let vp = store.active_board().expect("un tableau").viewport;
    let ui = UiState::new();

    for (wx, wy) in [(x0, y0), (x1, y1)] {
        let (sx, sy) = (wx * vp.scale + vp.x, wy * vp.scale + vp.y);
        assert!(sx >= 0.0 && sx <= w as f64, "x du contenu hors fenêtre : {sx}");
        assert!(
            sy >= ui.header_height() as f64 && sy <= h as f64,
            "y du contenu hors zone de canevas : {sy}"
        );
    }
}
