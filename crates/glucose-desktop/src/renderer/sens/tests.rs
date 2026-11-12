//! Les épreuves de la grammaire des couleurs du sens (PARENTE-2).

use super::SensDesTextes;
use crate::parente::{Avis, AvisDUnNoeud};
use crate::renderer::scene::membranes::{sans_couleur, BLEU_DE_TAURI};
use glucose_core::couleur::Oklch;
use glucose_core::types::Annotation;
use std::f64::consts::TAU;

fn avis(liste: &[(&str, Option<f64>, f32)]) -> Avis {
    liste
        .iter()
        .map(|&(id, teinte, surete)| {
            let famille = teinte.map(|_| 0);
            let avis = AvisDUnNoeud {
                famille,
                teinte,
                surete,
            };
            (id.to_string(), avis)
        })
        .collect()
}

/// L'écart de deux teintes sur le cercle, en radians.
fn ecart(a: f64, b: f64) -> f64 {
    let e = (a - b).rem_euclid(TAU);
    e.min(TAU - e)
}

/// **PARENTE-2 — une seule grammaire** : chaque famille a la clarté du bleu de Tauri — celle d'une
/// membrane sans couleur — et, sûre, la vivacité du plus grand cercle de teintes ; moins sûre,
/// moins vive, à proportion ; la famille d'origine prend la teinte du bleu ; un texte seul prend le
/// gris d'une membrane sans couleur. Les écarts tolérés sont ceux d'un octet par canal.
#[test]
fn parente_2_les_familles_ont_la_clarte_du_gris_neutre() {
    let mut sens = SensDesTextes::default();
    sens.poser(&avis(&[
        ("a", Some(0.0), 1.0),
        ("b", Some(2.0), 0.8),
        ("c", Some(4.0), 0.6),
        ("seul", None, 0.1),
    ]));
    let bleu = Oklch::des_octets(BLEU_DE_TAURI);
    let cercle = Oklch::cercle_des_teintes(bleu.l);
    let texte = |id: &str| Annotation::text(id, 0.0, 0.0, "x");
    for (id, attendue, surete) in [
        ("a", bleu.h, 1.0),
        ("b", bleu.h + 2.0, 0.8),
        ("c", bleu.h + 4.0, 0.6),
    ] {
        let (rgb, _) = sens.lueur(&texte(id));
        let lue = Oklch::des_octets(rgb);
        assert!((lue.l - bleu.l).abs() < 0.005, "{id} : clarté {}", lue.l);
        assert!(
            (lue.c - cercle * surete).abs() < 0.01,
            "{id} : vivacité {}",
            lue.c
        );
        assert!(ecart(lue.h, attendue) < 0.06, "{id} : teinte {}", lue.h);
    }
    assert_eq!(sens.lueur(&texte("seul")), (sans_couleur(), 0.1));
    assert_eq!(
        sens.lueur(&texte("b")).1,
        0.8,
        "la sûreté passe telle quelle"
    );
}

/// **PARENTE-2 — la couleur choisie passe devant, sûre ; un texte inconnu est gris, sans sûreté** :
/// une couleur qui ne se lit pas laisse parler l'IA.
#[test]
fn parente_2_la_couleur_choisie_puis_l_ia_puis_le_gris() {
    let mut sens = SensDesTextes::default();
    sens.poser(&avis(&[("a", Some(1.0), 0.7)]));
    let colore = |id: &str, c: Option<&str>| {
        let mut t = Annotation::text(id, 0.0, 0.0, "x");
        if let Annotation::Text { color, .. } = &mut t {
            *color = c.map(str::to_string);
        }
        t
    };
    assert_eq!(
        sens.lueur(&colore("a", Some("#f472b6"))),
        ((0xf4, 0x72, 0xb6), 1.0)
    );
    let par_l_ia = sens.lueur(&colore("a", None));
    assert_eq!(par_l_ia.1, 0.7);
    assert_eq!(sens.lueur(&colore("a", Some("pas une couleur"))), par_l_ia);
    assert_eq!(sens.lueur(&colore("inconnu", None)), (sans_couleur(), 0.0));
    assert_eq!(sens.teinte(&colore("a", None)), par_l_ia.0);
}

/// **Le dégradé retenu est celui de la loi, et s'oublie** (fiche 108) : la mémoire rend, pour
/// chaque paire, ce que rend `aspect::teintes` — l'ordre compte —, garde ce que l'image d'avant a
/// demandé, et oublie le reste.
#[test]
fn le_degrade_retenu_est_celui_de_la_loi_et_s_oublie() {
    use glucose_core::arrow::aspect::teintes;
    let sens = SensDesTextes::default();
    let (bleu, orange, gris) = ((96, 165, 250), (233, 178, 122), sans_couleur());
    for (a, b) in [(bleu, orange), (orange, bleu), (gris, bleu), (bleu, bleu)] {
        assert_eq!(sens.degrade(a, b), teintes(a, b), "{a:?} → {b:?}");
        assert_eq!(sens.degrade(a, b), teintes(a, b), "{a:?} → {b:?}, retenu");
    }
    assert_ne!(
        sens.degrade(bleu, orange),
        sens.degrade(orange, bleu),
        "le sens de la flèche compte"
    );
    assert_eq!(sens.degrades_retenus(), 4);
    sens.nouvelle_image();
    sens.degrade(bleu, orange);
    sens.nouvelle_image();
    assert_eq!(
        sens.degrades_retenus(),
        1,
        "ce que l'image d'avant n'a pas demandé s'oublie"
    );
    sens.nouvelle_image();
    assert_eq!(sens.degrades_retenus(), 0);
}

/// **Chaque image du rendu commence par oublier les dégradés que la précédente n'a pas
/// demandés** : sans cet appel, la mémoire ne ferait que grossir. Une image du témoin et ses
/// flèches en retient ; deux images d'un tableau vide n'en gardent aucun.
#[test]
fn le_rendu_oublie_les_degrades_d_une_image_a_l_autre() {
    use crate::params::{Pointer, SceneOverlay};
    use crate::renderer::{Regard, Renderer};
    let mut renderer = Renderer::new();
    let mut ui = crate::ui::UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    let mut image = |renderer: &mut Renderer, store: &glucose_core::store::Store| {
        let mut pixmap = tiny_skia::Pixmap::new(800, 600).expect("un pixmap");
        renderer.render(
            &mut pixmap.as_mut(),
            store,
            &mut ui,
            SceneOverlay::sans_rien(&guides),
            Pointer { x: 0.0, y: 0.0 },
            Regard::immobile(),
        );
    };
    image(&mut renderer, &glucose_core::synth::witness());
    assert!(
        renderer.sens_des_textes.degrades_retenus() > 0,
        "le témoin a des flèches : leurs dégradés sont retenus"
    );
    let vide = glucose_core::store::Store::new("Vide");
    image(&mut renderer, &vide);
    image(&mut renderer, &vide);
    assert_eq!(renderer.sens_des_textes.degrades_retenus(), 0);
}
