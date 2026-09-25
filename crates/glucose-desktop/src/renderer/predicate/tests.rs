//! Les six sigles : qu'ils se dessinent, qu'ils se distinguent, et qu'ils ne dépendent
//! d'aucune fonte.

use super::*;
use crate::typography::coverage::missing_from_interface;
use tiny_skia::{Color, Pixmap};

const FOND: (u8, u8, u8) = (13, 14, 18);

/// Trace un sigle seul sur un fond uni, au centre d'un carré.
fn rendu(predicate: ArrowPredicate, extent: f32) -> Pixmap {
    let mut pixmap = Pixmap::new(64, 64).expect("pixmap de test");
    pixmap.fill(Color::from_rgba8(FOND.0, FOND.1, FOND.2, 255));
    draw_sigil(
        &mut pixmap.as_mut(),
        predicate,
        (32.0, 32.0),
        extent,
        Color::from_rgba8(255, 255, 255, 255),
    );
    pixmap
}

/// Les pixels peints d'une pixmap, comme empreinte de forme.
fn marque(pixmap: &Pixmap) -> Vec<bool> {
    pixmap
        .pixels()
        .iter()
        .map(|p| (p.red(), p.green(), p.blue()) != FOND)
        .collect()
}

fn peints(pixmap: &Pixmap) -> usize {
    marque(pixmap).iter().filter(|p| **p).count()
}

/// **PRED-1** — quatre des six sigles de Glucose Tauri sont absents des polices embarquées.
///
/// C'est la mesure qui a décidé de les tracer plutôt que de les écrire, et ce test la
/// refait à chaque compilation. Le jour où une fonte les couvrirait toutes, il échouera —
/// et ce sera une bonne nouvelle à instruire, pas une régression.
#[test]
fn test_pred_1_the_unicode_sigils_of_glucose_tauri_are_not_all_available() {
    // Les six caractères de `ArrowSvgLayer.tsx`, écrits en points de code pour que le
    // scanner de couverture ne les exige pas des polices (`typography::coverage`).
    let tauri = "\u{2192}\u{2717}\u{2282}\u{2726}\u{2295}\u{25CE}";
    let manquants = missing_from_interface(tauri);
    assert_eq!(
        manquants.len(),
        4,
        "les polices couvrent maintenant {} des six sigles de Glucose Tauri : {manquants:?}",
        6 - manquants.len()
    );
}

/// Chacun des six sigles dessine vraiment quelque chose.
#[test]
fn test_every_predicate_draws_a_sigil() {
    for predicate in ArrowPredicate::ALL {
        let compte = peints(&rendu(predicate, 12.0));
        assert!(
            compte > 20,
            "{} ne dessine que {compte} pixels",
            predicate.as_str()
        );
    }
}

/// Les six se distinguent deux à deux : aucun n'est le dessin d'un autre.
///
/// Un prédicat qui aurait le sigle d'un autre serait pire qu'un sigle absent : il dirait
/// une relation fausse, et rien ne le signalerait.
#[test]
fn test_the_six_sigils_are_all_distinct() {
    let marques: Vec<Vec<bool>> = ArrowPredicate::ALL
        .iter()
        .map(|p| marque(&rendu(*p, 12.0)))
        .collect();
    for (i, a) in marques.iter().enumerate() {
        for (j, b) in marques.iter().enumerate().skip(i + 1) {
            assert_ne!(
                a,
                b,
                "{} et {} dessinent la même chose",
                ArrowPredicate::ALL[i].as_str(),
                ArrowPredicate::ALL[j].as_str()
            );
        }
    }
}

/// Un sigle grandit avec sa pastille : il est tracé, donc net à toute taille.
///
/// C'est le gain du tracé sur le glyphe : une fonte rastérisée à neuf pixels est floue, et
/// la même à cent l'est encore, en plus gros.
#[test]
fn test_a_sigil_scales_with_its_badge() {
    for predicate in ArrowPredicate::ALL {
        let petit = peints(&rendu(predicate, 6.0));
        let grand = peints(&rendu(predicate, 20.0));
        assert!(
            grand > petit,
            "{} : {petit} pixels à 6 et {grand} à 20",
            predicate.as_str()
        );
    }
}

/// Un sigle tient **dans** son carré : il ne déborde pas de la pastille qui l'encadre.
#[test]
fn test_a_sigil_stays_inside_its_square() {
    let extent = 12.0f32;
    for predicate in ArrowPredicate::ALL {
        let pixmap = rendu(predicate, extent);
        let w = pixmap.width() as usize;
        for (index, peint) in marque(&pixmap).iter().enumerate() {
            if !peint {
                continue;
            }
            let (x, y) = ((index % w) as f32 + 0.5, (index / w) as f32 + 0.5);
            let debord = (x - 32.0).abs().max((y - 32.0).abs());
            // La demi-épaisseur du trait déborde légitimement du carré nominal.
            assert!(
                debord <= extent * (1.0 + SIGIL_STROKE),
                "{} déborde de {debord} px pour un carré de {extent}",
                predicate.as_str()
            );
        }
    }
}

/// Une taille dégénérée ne dessine rien, et ne panique pas.
#[test]
fn test_a_degenerate_extent_is_safe() {
    for extent in [0.0f32, -3.0, f32::NAN, f32::INFINITY] {
        let pixmap = rendu(ArrowPredicate::Contredit, extent);
        if extent.is_finite() && extent > 0.0 {
            continue;
        }
        assert_eq!(
            peints(&pixmap),
            0,
            "extent {extent} a dessiné quelque chose"
        );
    }
}

/// Le rang d'un prédicat désigne **la même** chose partout : sa couleur, son sigle, sa touche.
///
/// Trois tables qui numéroteraient les mêmes six valeurs finiraient par les numéroter
/// différemment, et c'est alors le sigle d'une relation qui s'afficherait sur une autre.
#[test]
fn test_the_rank_is_the_single_numbering_of_a_predicate() {
    for (index, predicate) in ArrowPredicate::ALL.iter().enumerate() {
        assert_eq!(predicate.rank(), index);
    }
    let theme = crate::theme::Theme::dark();
    let couleurs: std::collections::HashSet<[u8; 4]> = ArrowPredicate::ALL
        .iter()
        .map(|p| {
            let c = theme.predicate_color(*p);
            [
                (c.red() * 255.0) as u8,
                (c.green() * 255.0) as u8,
                (c.blue() * 255.0) as u8,
                (c.alpha() * 255.0) as u8,
            ]
        })
        .collect();
    assert_eq!(couleurs.len(), 6, "deux prédicats partagent une couleur");
}
