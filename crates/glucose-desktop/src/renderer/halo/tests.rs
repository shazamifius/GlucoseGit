//! Tests de la passe de lueurs : exactitude de la factorisation (HALO-1), portée déduite
//! du huit bits (HALO-2), et fidélité de forme à l'ombre CSS de Glucose Tauri.

use super::*;
use std::collections::HashSet;
use tiny_skia::{Color, Pixmap};

/// Fond de référence des tests : la couleur de toile du thème sombre.
fn background() -> Color {
    Color::from_rgba8(13, 14, 18, 255)
}

fn bench_card(id: &str, x: f64, y: f64) -> Annotation {
    Annotation::Text {
        id: id.into(),
        x,
        y,
        width: Some(200.0),
        height: Some(50.0),
        text: String::new(),
        font_size: Some(14.0),
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

/// Une carte de texte à l'origine du monde ; `None` prend la dimension par défaut du modèle.
fn sized_card(width: Option<f64>, height: Option<f64>) -> Annotation {
    let mut card = bench_card("halo", 0.0, 0.0);
    if let Annotation::Text {
        width: w,
        height: h,
        ..
    } = &mut card
    {
        *w = width;
        *h = height;
    }
    card
}

/// Un viewport qui centre une carte de `(w, h)` sur un écran de 1440 × 900, à ce zoom.
fn centered(zoom: f64, (w, h): (f64, f64)) -> Viewport {
    Viewport {
        x: 720.0 - w * zoom / 2.0,
        y: 450.0 - h * zoom / 2.0,
        scale: zoom,
    }
}

// ── HALO-1 — la factorisation est exacte ──────────────────────────────────────

/// La lueur d'un rectangle, calculée **sans** la factorisation.
///
/// Elle somme le noyau à deux dimensions sur les pixels du rectangle, un par un. La
/// référence n'est pas indépendante de la mathématique — un noyau gaussien à deux
/// dimensions *est* un produit de deux noyaux à une dimension, c'est précisément le
/// théorème qu'on exploite. Elle est indépendante de son **implémentation** : elle ne
/// connaît ni table cumulée, ni interpolation, ni indice de frontière. C'est là que les
/// fautes vivent, et c'est ce que ce test attrape.
fn reference_2d(x: i32, y: i32, (a, b, c, d): (i32, i32, i32, i32), sigma: f32) -> f64 {
    let radius = (4.0 * sigma).ceil() as i32;
    let two_sigma_squared = 2.0 * sigma * sigma;
    let mut inside = 0.0f64;
    let mut total = 0.0f64;
    for i in -radius..=radius {
        for j in -radius..=radius {
            let squared = (i * i + j * j) as f32;
            let weight = f64::from((-squared / two_sigma_squared).exp());
            total += weight;
            let (sx, sy) = (x - i, y - j);
            if sx >= a && sx < b && sy >= c && sy < d {
                inside += weight;
            }
        }
    }
    inside / total
}

/// HALO-1 — le produit de deux profils rend la convolution à deux dimensions, au flottant près.
///
/// Le rectangle est aligné sur la grille de pixels : les bornes tombent alors sur des
/// frontières de la table, l'interpolation ne joue pas, et la comparaison est exacte au
/// lieu d'être approchée. C'est ce qui permet une tolérance de 10⁻⁵ plutôt qu'un seuil
/// choisi pour que ça passe.
#[test]
fn test_halo_1_the_product_of_two_profiles_is_the_two_dimensional_convolution() {
    let rect = (12, 26, 14, 20);
    for sigma in [0.75_f32, 2.0, 5.5] {
        let profile = EdgeProfile::new(sigma);
        let mut worst = 0.0f64;
        for y in 0..34 {
            for x in 0..40 {
                let separable = f64::from(
                    profile.band(x as f32 + 0.5, rect.0 as f32, rect.1 as f32)
                        * profile.band(y as f32 + 0.5, rect.2 as f32, rect.3 as f32),
                );
                let direct = reference_2d(x, y, rect, sigma);
                worst = worst.max((separable - direct).abs());
            }
        }
        assert!(
            worst < 1e-5,
            "sigma {sigma} : le produit s'écarte de {worst} de la convolution directe"
        );
    }
}

/// HALO-1 — un écart-type nul donne exactement un bord net, sans aucune garde.
///
/// Le noyau se réduit alors à un seul échantillon : la table vaut la marche, et la lueur
/// est le rectangle lui-même. Le cas ne demande pas de branche, il tombe de la
/// construction — et ce test est là pour que personne n'en rajoute une.
#[test]
fn test_halo_1_a_zero_sigma_is_a_hard_edge_and_needs_no_guard() {
    let profile = EdgeProfile::new(0.0);
    assert_eq!(profile.band(5.5, 0.0, 10.0), 1.0, "dedans : pleine opacité");
    assert_eq!(profile.band(-0.5, 0.0, 10.0), 0.0, "dehors : rien");
    assert_eq!(profile.band(10.5, 0.0, 10.0), 0.0, "au-delà du bord : rien");
}

/// HALO-1 — le profil est une partition : ce qui sort d'un côté entre de l'autre.
#[test]
fn test_halo_1_the_profile_conserves_its_mass() {
    let profile = EdgeProfile::new(3.0);
    for t in [-20.0_f32, -3.5, 0.0, 4.25, 30.0] {
        let sum = profile.cumulative(t) + (1.0 - profile.cumulative(t));
        assert!((sum - 1.0).abs() < 1e-6, "en {t}");
    }
    // Et elle croît : une cumulée qui redescendrait rendrait des bandes négatives.
    let mut previous = -1.0;
    for step in -40..=40 {
        let value = profile.cumulative(step as f32 * 0.5);
        assert!(value >= previous, "la cumulée recule en {step}");
        previous = value;
    }
}

// ── La forme : une lueur qui épouse sa carte, pas une bulle ───────────────────

/// Le débord de la lueur, en unités **monde**, au-dessus et à gauche d'une carte.
fn overhang(card: &Annotation) -> (f64, f64) {
    let rect = card.rect().expect("une carte a une boîte");
    let vp = centered(1.0, (rect.width, rect.height));
    let halo = halo_geometry(card, &vp, 1440.0, 900.0, 0.0).expect("la carte est à l'écran");
    let reach = f64::from(EdgeProfile::new(halo.sigma).reach(HALO_ALPHA));
    let (sx, sy) = world_to_screen(rect.left, rect.top, &vp);
    (
        sx - f64::from(halo.left) + reach,
        sy - f64::from(halo.top) + reach,
    )
}

/// La lueur épouse la carte : son débord ne dépend **pas** des dimensions de celle-ci.
///
/// C'est le défaut corrigé, dit en une ligne. Le disque avait pour rayon
/// `max(w, h) × 1,5 + 50` : sur une carte de 240 × 60 il montait à 380 unités au-dessus
/// d'une carte qui n'en fait que 60 de haut, et le débord *vertical* grandissait avec la
/// largeur — une bulle, pas une lueur. Ici, les quatre débords sont la même longueur, et
/// c'est celle de l'ombre CSS de Glucose Tauri : `spread` plus la portée du flou.
#[test]
fn test_the_glow_hugs_its_card_instead_of_ballooning() {
    let large = overhang(&sized_card(Some(240.0), Some(60.0)));
    let square = overhang(&sized_card(Some(60.0), Some(60.0)));
    let tall = overhang(&sized_card(Some(60.0), Some(400.0)));

    for (nom, (dx, dy)) in [("large", large), ("carrée", square), ("haute", tall)] {
        assert!(
            (dx - dy).abs() < 1e-6,
            "carte {nom} : le débord vaut {dx} en largeur et {dy} en hauteur"
        );
    }
    assert!(
        (large.0 - square.0).abs() < 1e-6 && (tall.0 - square.0).abs() < 1e-6,
        "le débord dépend encore de la taille de la carte : {} / {} / {}",
        large.0,
        square.0,
        tall.0
    );

    // Et il reste près du bord : le disque d'avant montait à 410 sur la carte large.
    let ancien_rayon = 240.0 * 1.5 + 50.0;
    assert!(
        large.1 * 4.0 < ancien_rayon,
        "le débord vertical vaut {}, pas assez loin du disque de {ancien_rayon}",
        large.1
    );
}

/// La lueur grandit avec sa carte (SCALE-1) : elle n'a pas d'exception écran.
///
/// Le conteneur de Glucose Tauri porte `scale(…)`, donc son `box-shadow` est mis à
/// l'échelle comme le reste. Doubler le zoom double la boîte de lueur, sans plafond —
/// le plafond écran de l'ancien disque a disparu avec lui.
#[test]
fn test_the_glow_scales_with_the_zoom_without_any_cap() {
    let card = sized_card(Some(240.0), Some(60.0));
    let width_at = |zoom: f64| {
        let vp = centered(zoom, (240.0, 60.0));
        let halo = halo_geometry(&card, &vp, 1440.0, 900.0, 0.0).expect("à l'écran");
        f64::from(halo.right - halo.left)
    };
    let un = width_at(1.0);
    for zoom in [0.5_f64, 2.0, 3.0, 8.0] {
        let attendu = un * zoom;
        let mesure = width_at(zoom);
        assert!(
            (mesure - attendu).abs() < 1e-3,
            "à x{zoom} la lueur fait {mesure} au lieu de {attendu}"
        );
    }
}

/// Hors du cadre, la carte n'est pas parcourue du tout.
#[test]
fn test_a_card_far_off_screen_is_culled() {
    let card = sized_card(Some(240.0), Some(60.0));
    let loin = Viewport {
        x: -100_000.0,
        y: 0.0,
        scale: 1.0,
    };
    assert!(halo_geometry(&card, &loin, 1440.0, 900.0, 0.0).is_none());

    let brise = Viewport {
        x: f64::NAN,
        y: 0.0,
        scale: 1.0,
    };
    assert!(halo_geometry(&card, &brise, 1440.0, 900.0, 0.0).is_none());
}

// ── HALO-2 — la portée se déduit du huit bits ─────────────────────────────────

/// Peint une lueur seule sur un fond uni et rend la pixmap.
fn render(halo: HaloBox, alpha: u8, side: u32) -> Pixmap {
    let mut pixmap = Pixmap::new(side, side).expect("pixmap de test");
    pixmap.fill(background());
    draw_halo(&mut pixmap.as_mut(), halo, (255, 128, 64), alpha);
    pixmap
}

/// La colonne la plus éloignée du centre qui a été touchée, en pixels.
fn farthest_touched(pixmap: &Pixmap, center: f32) -> f32 {
    let width = pixmap.width() as usize;
    let mut farthest = 0.0f32;
    for (index, pixel) in pixmap.pixels().iter().enumerate() {
        if (pixel.red(), pixel.green(), pixel.blue()) != (13, 14, 18) {
            let x = (index % width) as f32 + 0.5;
            farthest = farthest.max((x - center).abs());
        }
    }
    farthest
}

/// HALO-2 — au-delà de la portée mesurée, pas un pixel n'est touché ; juste avant, si.
///
/// C'est la portée elle-même qui est mise à l'épreuve, pas une constante : le test lit
/// ce que [`EdgeProfile::reach`] annonce, et vérifie que le dessin s'y tient des deux
/// côtés. Une portée trop courte couperait la lueur, une trop longue peindrait des
/// pixels qui s'arrondissent à zéro.
#[test]
fn test_halo_2_the_reach_is_exactly_where_the_glow_dies() {
    let side = 400u32;
    let center = side as f32 / 2.0;
    let sigma = 12.0;
    let halo = HaloBox {
        left: center - 30.0,
        top: center - 30.0,
        right: center + 30.0,
        bottom: center + 30.0,
        sigma,
    };
    let reach = f64::from(EdgeProfile::new(sigma).reach(HALO_ALPHA));
    let pixmap = render(halo, HALO_ALPHA, side);
    let touched = f64::from(farthest_touched(&pixmap, center));

    // Le bord de la boîte est à 30 du centre ; la lueur meurt donc à `30 + reach`.
    let limite = 30.0 + reach;
    assert!(
        touched <= limite,
        "la lueur atteint {touched} px alors que la portée annonce {limite}"
    );
    assert!(
        touched > limite - 2.0,
        "la lueur meurt à {touched} px, bien avant les {limite} annoncés : la portée surestime"
    );
}

/// HALO-2 — une lueur plus discrète porte moins loin, et c'est le huit bits qui le décide.
///
/// La portée n'est pas un réglage : elle sort de `½ / alpha`. Diviser l'opacité par
/// quatre rapproche donc le point où la queue de la gaussienne cesse de pouvoir changer
/// un pixel.
#[test]
fn test_halo_2_a_fainter_glow_reaches_less_far() {
    let profile = EdgeProfile::new(20.0);
    let franche = profile.reach(102); // les 40 % de l'état survolé de Tauri
    let discrete = profile.reach(HALO_ALPHA);
    let murmure = profile.reach(4);
    assert!(
        murmure < discrete && discrete < franche,
        "portées : {murmure} / {discrete} / {franche}"
    );
    // Et aucune ne dépasse le majorant d'allocation de la table.
    assert!(franche <= 4.0 * 20.0);
}

/// Une opacité nulle ne dessine rien du tout — et surtout pas un carré transparent.
#[test]
fn test_a_zero_opacity_draws_nothing() {
    let halo = HaloBox {
        left: 10.0,
        top: 10.0,
        right: 50.0,
        bottom: 50.0,
        sigma: 6.0,
    };
    let pixmap = render(halo, 0, 80);
    assert_eq!(farthest_touched(&pixmap, 40.0), 0.0);
}

/// Une boîte dégénérée ne panique pas et ne boucle pas : rien n'est dessiné.
#[test]
fn test_degenerate_geometry_is_bounded_and_safe() {
    let side = 64u32;
    for halo in [
        HaloBox {
            left: f32::NAN,
            top: 0.0,
            right: 10.0,
            bottom: 10.0,
            sigma: 3.0,
        },
        HaloBox {
            left: 0.0,
            top: 0.0,
            right: f32::INFINITY,
            bottom: 10.0,
            sigma: 3.0,
        },
        HaloBox {
            left: 30.0,
            top: 30.0,
            right: 10.0,
            bottom: 10.0,
            sigma: 3.0,
        },
        HaloBox {
            left: 10.0,
            top: 10.0,
            right: 20.0,
            bottom: 20.0,
            sigma: f32::NAN,
        },
    ] {
        let mut pixmap = Pixmap::new(side, side).expect("pixmap de test");
        pixmap.fill(background());
        draw_halo(&mut pixmap.as_mut(), halo, (255, 0, 0), HALO_ALPHA);
    }
}

/// `div255` rend exactement l'arrondi au plus proche de `x / 255`.
#[test]
fn test_div255_is_exact_rounding() {
    for x in 0..=(255u32 * 255) {
        let expected = ((f64::from(x) / 255.0) + 0.5) as u32;
        assert_eq!(div255(x), expected, "div255({x})");
    }
}

// ── Le coût ──────────────────────────────────────────────────────────────────

/// Nombre de cartes du banc de performance de la passe de lueurs.
const BENCH_CARD_COUNT: usize = 40;

/// Budget de la passe pour [`BENCH_CARD_COUNT`] cartes, en 1440 × 900, en debug.
///
/// Loi L2 — le coût d'une frame ne suit pas la taille du document. Mesuré sur la machine
/// de développement : 364 ms avec le `RadialGradient` reconstruit par carte, 58 ms avec
/// le disque en anneaux. L'ombre séparable touche cinq fois moins de pixels ; le budget
/// reste large pour le matériel le plus lent, et échoue franchement si le dégradé par
/// frame revenait.
const HALO_PASS_BUDGET_MS: u128 = 250;

/// Banc de performance : la passe reste dans son budget pour 40 cartes.
///
/// # Pourquoi il est ignoré par défaut
///
/// Il mesure un temps, et `cargo test` lance ses tests **en parallèle**. Ce qu'il
/// chronomètre est donc la passe *plus* la contention avec tout ce qui tourne à côté, et
/// le résultat dépend de la machine et du moment. Un test qui échoue au hasard ne dit
/// plus rien, et il abîme la valeur des autres. La mesure a sa place dans le banc
/// ([`crate::bench`]), qui mesure une frame entière sans rien d'autre en vol. Le lancer à
/// la main reste possible :
///
/// ```text
/// cargo test -p glucose-desktop --lib halo_pass -- --ignored --test-threads=1
/// ```
#[test]
#[ignore = "mesure un temps : sensible à la charge de la machine, voir la documentation"]
fn test_halo_pass_stays_within_budget_for_a_dense_board() {
    let mut pixmap = Pixmap::new(1440, 900).expect("pixmap 1440x900");
    let mut store = Store::new("Banc lueurs");
    let board_id = store.project.active_board_id.clone();
    for i in 0..BENCH_CARD_COUNT {
        let card = bench_card(
            &format!("halo-bench-{i}"),
            ((i % 8) as f64) * 180.0,
            ((i / 8) as f64) * 180.0,
        );
        store.add_annotation(&board_id, card);
    }

    let mut hue_cache = SymbioticHueCache::new();
    let vp = Viewport::default();
    let board = store.active_board().expect("le board vient d'être rempli");
    let visible: HashSet<&str> = board.annotations.iter().map(|a| a.id()).collect();
    hue_cache.update_positions_and_invalidate(&board.annotations);

    let pass = ViewPass {
        vp,
        visible_ids: &visible,
        header_h: 40.0,
    };
    // Frame de chauffe : remplit le cache de teintes symbiotiques.
    draw_halos(&mut hue_cache, &mut pixmap.as_mut(), &store, pass);

    let started = std::time::Instant::now();
    draw_halos(&mut hue_cache, &mut pixmap.as_mut(), &store, pass);
    let elapsed = started.elapsed().as_millis();

    assert!(
        elapsed < HALO_PASS_BUDGET_MS,
        "passe de lueurs pour {BENCH_CARD_COUNT} cartes : {elapsed} ms (budget {HALO_PASS_BUDGET_MS} ms)"
    );
}
