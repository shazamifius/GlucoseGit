//! Les lois des courbes et des tweens. Toutes se vérifient sans attendre une milliseconde :
//! c'est l'intérêt d'un module qui ne connaît pas l'heure.

use super::*;

/// Toutes les courbes tiennent le contrat : 0 en 0, 1 en 1, et jamais de `NaN`.
#[test]
fn test_toute_courbe_part_de_zero_et_arrive_a_un() {
    for c in [
        Curve::Linear,
        Curve::EaseOutCubic,
        Curve::DOCK_GRAB,
        Curve::DOCK_REORDER,
    ] {
        assert!((c.at(0.0) - 0.0).abs() < 1e-9, "{c:?} en 0");
        assert!((c.at(1.0) - 1.0).abs() < 1e-9, "{c:?} en 1");
        for i in 0..=100 {
            let v = c.at(i as f64 / 100.0);
            assert!(v.is_finite(), "{c:?} rend {v} en {i}%");
        }
    }
}

/// Hors de `[0, 1]`, une courbe rend ses bornes plutôt que de continuer sa course — et un
/// temps qui n'est pas un nombre rend le départ, jamais un `NaN` qui contaminerait la scène.
#[test]
fn test_hors_du_temps_normal_la_courbe_rend_ses_bornes() {
    for c in [Curve::Linear, Curve::EaseOutCubic, Curve::DOCK_GRAB] {
        assert_eq!(c.at(-5.0), c.at(0.0), "{c:?} avant le début");
        assert_eq!(c.at(42.0), c.at(1.0), "{c:?} après la fin");
        assert_eq!(c.at(f64::NAN), c.at(0.0), "{c:?} hors du temps");
        assert!(c.at(f64::INFINITY).is_finite());
    }
}

/// L'amorti universel est bien `1 − (1 − t)³`, et il est **décroissant en vitesse** : chaque
/// dixième avance moins que le précédent. C'est ce qui fait le « départ vif, arrivée posée ».
#[test]
fn test_l_amorti_universel_ralentit_du_debut_a_la_fin() {
    let c = Curve::EaseOutCubic;
    for i in 0..=100 {
        let t = i as f64 / 100.0;
        let attendu = 1.0 - (1.0 - t).powi(3);
        assert!((c.at(t) - attendu).abs() < 1e-12, "en {t}");
    }
    // À mi-course, l'amorti a déjà fait 87,5 % du chemin.
    assert!((c.at(0.5) - 0.875).abs() < 1e-12);

    let mut precedent = f64::INFINITY;
    for i in 0..10 {
        let (a, b) = (i as f64 / 10.0, (i + 1) as f64 / 10.0);
        let avance = c.at(b) - c.at(a);
        assert!(avance < precedent, "le dixième {i} avance plus que le précédent");
        assert!(avance > 0.0, "et il avance quand même");
        precedent = avance;
    }
}

/// **Le rebond du dock dépasse vraiment.** Un point de contrôle d'ordonnée 1,56 fait sortir la
/// courbe au-dessus de 1 avant qu'elle ne se cale — c'est la fonctionnalité, pas un défaut, et
/// une implémentation qui rabattrait la sortie dans `[0, 1]` la détruirait en silence.
#[test]
fn test_le_rebond_du_dock_depasse_avant_de_se_caler() {
    let c = Curve::DOCK_GRAB;
    let maximum = (0..=1000)
        .map(|i| c.at(i as f64 / 1000.0))
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(maximum > 1.0, "le rebond doit dépasser : maximum {maximum}");
    assert!(maximum < 1.2, "sans partir trop loin : {maximum}");
    assert!((c.at(1.0) - 1.0).abs() < 1e-9, "et il se cale exactement");
}

/// **La Bézier donne ce qu'un navigateur donne.** Les valeurs de référence viennent de la
/// définition de `cubic-bezier` : l'abscisse et l'ordonnée sont deux Béziers du même paramètre,
/// et l'ordonnée se lit à l'abscisse voulue — ce n'est pas `y(t)`.
///
/// Le témoin est construit ici par une recherche indépendante de celle du module : une
/// bissection à mille pas, lente mais sans Newton. Les deux doivent coïncider.
#[test]
fn test_la_bezier_donne_ce_qu_un_navigateur_donne() {
    fn temoin(x: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let (mut bas, mut haut) = (0.0_f64, 1.0_f64);
        for _ in 0..200 {
            let milieu = (bas + haut) / 2.0;
            if bezier(milieu, x1, x2) < x {
                bas = milieu;
            } else {
                haut = milieu;
            }
        }
        bezier((bas + haut) / 2.0, y1, y2)
    }

    for (x1, y1, x2, y2) in [
        (0.34, 1.56, 0.64, 1.0),   // le rebond du dock
        (0.22, 1.0, 0.36, 1.0),    // le réordonnancement
        (0.25, 0.1, 0.25, 1.0),    // « ease » de CSS
        (0.42, 0.0, 1.0, 1.0),     // « ease-in »
        (0.0, 0.0, 0.58, 1.0),     // « ease-out »
        (0.0, 0.0, 1.0, 1.0),      // la diagonale : doit redonner l'identité
    ] {
        for i in 0..=50 {
            let x = i as f64 / 50.0;
            let obtenu = Curve::CubicBezier(x1, y1, x2, y2).at(x);
            let attendu = temoin(x, x1, y1, x2, y2);
            assert!(
                (obtenu - attendu).abs() < 1e-6,
                "bezier({x1}, {y1}, {x2}, {y2}) en {x} : {obtenu} au lieu de {attendu}"
            );
        }
    }
}

/// Une Bézier dont les points de contrôle sont sur la diagonale est l'identité — le cas qui
/// attrape une confusion entre le paramètre et l'abscisse.
#[test]
fn test_la_bezier_diagonale_est_l_identite() {
    let c = Curve::CubicBezier(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0);
    for i in 0..=100 {
        let t = i as f64 / 100.0;
        assert!((c.at(t) - t).abs() < 1e-6, "en {t} : {}", c.at(t));
    }
}

// ── Les tweens ───────────────────────────────────────────────────────────────

/// Un tween part de sa valeur de départ, arrive à celle d'arrivée, et y reste.
#[test]
fn test_un_tween_part_d_ou_il_part_et_arrive_ou_il_arrive() {
    let t = Tween::new(100.0, 500.0, 400, Curve::EaseOutCubic);
    assert_eq!(t.at(0.0), 100.0);
    assert_eq!(t.at(400.0), 500.0);
    assert_eq!(t.at(10_000.0), 500.0, "et il ne repart pas");
    assert_eq!(t.at(-50.0), 100.0, "ni ne recule");

    // Une valeur qui descend marche aussi bien qu'une qui monte.
    let descente = Tween::new(1.0, 0.0, 200, Curve::Linear);
    assert_eq!(descente.at(0.0), 1.0);
    assert!((descente.at(100.0) - 0.5).abs() < 1e-12);
    assert_eq!(descente.at(200.0), 0.0);
}

/// Une animation de durée nulle rend son arrivée, plutôt que de diviser par zéro.
#[test]
fn test_une_animation_de_duree_nulle_est_deja_finie() {
    let t = Tween::new(0.0, 42.0, 0, Curve::EaseOutCubic);
    assert_eq!(t.at(0.0), 42.0);
    assert!(!t.running(0.0));
    assert_eq!(t.remaining_ms(0.0), 0.0);
}

/// Un tween dit quand il a fini et combien de temps il lui reste : c'est ce dont la boucle
/// d'événements a besoin pour redemander une frame, et rien de plus.
#[test]
fn test_un_tween_dit_ce_qu_il_lui_reste() {
    let t = Tween::new(0.0, 1.0, timing::FOLDER_TRANSITION_MS, Curve::EaseOutCubic);
    assert!(t.running(0.0));
    assert!(t.running(399.9));
    assert!(!t.running(400.0), "à l'échéance exacte, c'est fini");
    assert!(!t.running(1_000.0));

    assert_eq!(t.remaining_ms(0.0), 400.0);
    assert_eq!(t.remaining_ms(150.0), 250.0);
    assert_eq!(t.remaining_ms(400.0), 0.0);
    assert_eq!(t.remaining_ms(9_999.0), 0.0, "jamais négatif");
}

/// **La table chronométrique de la fiche 07 § 1**, et l'absence de seconde définition : les
/// durées déjà posées ailleurs sont réexportées, donc changer la source change la table.
#[test]
fn test_la_table_chronometrique_est_celle_de_la_fiche() {
    use timing::*;
    assert_eq!(MEMBRANE_TWEEN_MS, 200);
    assert_eq!(FOLDER_TRANSITION_MS, 400);
    assert_eq!(MIRROR_TELEPORT_MS, 400);
    assert_eq!(PANEL_DISMISS_MS, 200);
    assert_eq!(MINIMAP_SLIDE_MS, 180);
    assert_eq!(ARROW_PANEL_IN_MS, 180);
    assert_eq!(AUTOSAVE_DEBOUNCE_MS, 2_000);
    assert_eq!(DOCK_REORDER_MS, 250);

    // Réexportées : la valeur vient d'ailleurs, et le test le dit en la comparant à sa source.
    assert_eq!(DOUBLE_CLICK_MS, crate::hit_priority::pick_consts::DBLCLICK_MS);
    assert_eq!(DOUBLE_CLICK_MS, 350);
    assert_eq!(CYCLE_TTL_MS, 2_500);
    assert_eq!(MEMBRANE_FOCUS_MS, 320);
    assert_eq!(FOCUS_COOLDOWN_MS, 400);
}

/// Les deux Béziers nommées sont celles que la fiche écrit.
#[test]
fn test_les_deux_beziers_nommees_sont_celles_de_la_fiche() {
    assert_eq!(Curve::DOCK_GRAB, Curve::CubicBezier(0.34, 1.56, 0.64, 1.0));
    assert_eq!(Curve::DOCK_REORDER, Curve::CubicBezier(0.22, 1.0, 0.36, 1.0));
}
