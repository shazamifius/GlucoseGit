//! Tests de la passe de lueurs : exactitude de la factorisation (HALO-1), portée déduite
//! du huit bits (HALO-2), et fidélité de forme à l'ombre CSS de Glucose Tauri.

use super::*;
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
    let halo = halo_geometry(
        card,
        (&vp, WorldScale::new(vp.scale, 1.0)),
        (1440.0, 900.0, 0.0),
        Eclat::REPOS,
    )
    .expect("la carte est à l'écran");
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
        let halo = halo_geometry(
            &card,
            (&vp, WorldScale::new(vp.scale, 1.0)),
            (1440.0, 900.0, 0.0),
            Eclat::REPOS,
        )
        .expect("à l'écran");
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
    assert!(halo_geometry(
        &card,
        (&loin, WorldScale::new(loin.scale, 1.0)),
        (1440.0, 900.0, 0.0),
        Eclat::REPOS
    )
    .is_none());

    let brise = Viewport {
        x: f64::NAN,
        y: 0.0,
        scale: 1.0,
    };
    assert!(halo_geometry(
        &card,
        (&brise, WorldScale::new(brise.scale, 1.0)),
        (1440.0, 900.0, 0.0),
        Eclat::REPOS
    )
    .is_none());
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
        carte: None,
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
        carte: None,
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
            carte: None,
        },
        HaloBox {
            left: 0.0,
            top: 0.0,
            right: f32::INFINITY,
            bottom: 10.0,
            sigma: 3.0,
            carte: None,
        },
        HaloBox {
            left: 30.0,
            top: 30.0,
            right: 10.0,
            bottom: 10.0,
            sigma: 3.0,
            carte: None,
        },
        HaloBox {
            left: 10.0,
            top: 10.0,
            right: 20.0,
            bottom: 20.0,
            sigma: f32::NAN,
            carte: None,
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
    hue_cache.suivre(1, &board.id);

    // Tous les nœuds sont visibles : le test mesure la passe entière, pas le culling.
    let rangs = glucose_core::quadtree::tous_les_rangs(board);
    let mut index = glucose_core::quadtree::SpatialHash::new(1000.0);
    index.index_board(board);
    let pass = ViewPass {
        vp,
        visibles: &rangs,
        index: &index,
        header_h: 40.0,
        densite: 1.0,
    };
    // Frame de chauffe : remplit le cache de teintes symbiotiques.
    draw_halos(&mut hue_cache, &mut pixmap.as_mut(), &store, (pass, &[]));

    let started = std::time::Instant::now();
    draw_halos(&mut hue_cache, &mut pixmap.as_mut(), &store, (pass, &[]));
    let elapsed = started.elapsed().as_millis();

    assert!(
        elapsed < HALO_PASS_BUDGET_MS,
        "passe de lueurs pour {BENCH_CARD_COUNT} cartes : {elapsed} ms (budget {HALO_PASS_BUDGET_MS} ms)"
    );
}

/// **Les deux voies rendent les memes bits.** La division par 255 sur deux canaux a la fois
/// doit valoir exactement celle qu'on ecrivait canal par canal, sur toute la plage que la
/// composition peut produire -- `c x a + d x (255 - a)` ne depasse jamais `255 x 255`.
///
/// Le champ bas se balaie en entier ; le champ haut prend les valeurs ou un report d'un champ
/// sur l'autre se verrait. Les croiser tous les deux couterait quatre milliards de cas pour la
/// meme garantie, puisque les deux champs subissent le meme traitement.
#[test]
fn la_division_par_255_sur_deux_canaux_vaut_celle_sur_un() {
    for bas in 0..=65_025u32 {
        for haut in [0u32, 1, 127, 128, 254, 255, 32_640, 65_025] {
            let attendu = super::div255(bas) | (super::div255(haut) << 16);
            let obtenu = super::div255_swar(bas | (haut << 16));
            assert_eq!(
                obtenu, attendu,
                "bas {bas} et haut {haut} : {obtenu:#010x} au lieu de {attendu:#010x}"
            );
        }
    }
}

/// La composition d'un niveau de lueur, canal par canal, doit valoir ce que le mot entier
/// produit -- sinon une teinte deriverait sans que rien ne le dise.
#[test]
fn la_composition_swar_vaut_la_composition_canal_par_canal() {
    for alpha in [0u8, 1, 17, 64, 128, 200, 255] {
        let source = super::LevelSource::new((200, 40, 120), alpha);
        for fond in [
            [0u8, 0, 0, 0],
            [255, 255, 255, 255],
            [10, 200, 90, 255],
            [7, 7, 7, 9],
        ] {
            let mut obtenu = fond;
            super::blend_pixel(&mut obtenu, source);

            let inv = 255 - u32::from(alpha);
            let canal = |c: usize, s: u32| super::div255(s + u32::from(fond[c]) * inv) as u8;
            let poids = u32::from(alpha);
            let attendu = [
                canal(0, 200 * poids),
                canal(1, 40 * poids),
                canal(2, 120 * poids),
                canal(3, 255 * poids),
            ];
            assert_eq!(obtenu, attendu, "alpha {alpha} sur {fond:?}");
        }
    }
}

/// Ce que coute une lueur qui couvre l'ecran entier -- le cas mesure sur une vraie session,
/// ou une seule carte zoomee faisait passer la passe a 14,7 ms de facon tres reproductible.
///
/// Ce n'est pas une preuve, c'est un repere : il dit ce que la passe coute sur CETTE machine,
/// pour que l'effet d'une optimisation se lise au lieu de s'annoncer.
#[test]
#[ignore = "mesure un temps : sensible à la charge de la machine"]
fn banc_une_lueur_qui_couvre_l_ecran() {
    let mut pixmap = Pixmap::new(2560, 1440).expect("un ecran de 2560x1440");
    pixmap.fill(background());
    let halo = HaloBox {
        left: 900.0,
        top: 500.0,
        right: 1660.0,
        bottom: 950.0,
        // Le sigma suit le zoom : de pres, la lueur deborde largement de l'ecran.
        sigma: 300.0,
        carte: None,
    };
    let teinte = (220, 120, 180);
    draw_halo(&mut pixmap.as_mut(), halo, teinte, HALO_ALPHA);

    const IMAGES: u32 = 20;
    let debut = std::time::Instant::now();
    for _ in 0..IMAGES {
        draw_halo(&mut pixmap.as_mut(), halo, teinte, HALO_ALPHA);
    }
    let par_image = debut.elapsed() / IMAGES;
    println!(
        "  lueur plein ecran (2560x1440) : {:.2} ms par image",
        par_image.as_secs_f64() * 1000.0
    );
}

/// Ou va le temps de la lueur : dans le melange, ou dans le calcul du niveau ?
///
/// Les deux se mesurent separement, parce qu'ils appellent des remedes opposes -- l'un demande
/// d'ecrire moins de pixels, l'autre d'en calculer moins. Optimiser le mauvais des deux est
/// exactement ce qu'on vient de faire.
#[test]
#[ignore = "mesure un temps : sensible à la charge de la machine"]
fn banc_ou_va_le_temps_de_la_lueur() {
    const PIXELS: usize = 2560 * 1440;
    let mut image = vec![[13u8, 14, 18, 255]; PIXELS];
    let source = LevelSource::new((220, 120, 180), HALO_ALPHA);

    let debut = std::time::Instant::now();
    for pixel in &mut image {
        blend_pixel(pixel, source);
    }
    let melange = debut.elapsed();

    // Le calcul du niveau, seul : une multiplication flottante, un arrondi, une conversion.
    let colonnes: Vec<f32> = (0..PIXELS).map(|i| (i % 1000) as f32 / 1000.0).collect();
    let poids = f32::from(HALO_ALPHA);
    let debut = std::time::Instant::now();
    let mut somme = 0usize;
    for colonne in &colonnes {
        somme += ((poids * colonne).round() as usize).min(38);
    }
    let calcul = debut.elapsed();
    assert!(somme > 0);

    println!(
        "  melange seul : {:.2} ms   calcul du niveau seul : {:.2} ms",
        melange.as_secs_f64() * 1000.0,
        calcul.as_secs_f64() * 1000.0
    );
}

/// **Les segments donnent exactement les memes pixels que le calcul par pixel.**
///
/// C'est la garantie qui autorise le remplacement : la dichotomie ne s'appuie sur la monotonie
/// du profil que pour trouver des frontieres, jamais pour approcher une valeur. Le test compare
/// une ligne entiere, sur les deux cotes du sommet et pour des poids qui tombent de part et
/// d'autre des demi-niveaux.
#[test]
fn les_segments_rendent_les_memes_pixels_que_le_calcul_par_pixel() {
    let profil = EdgeProfile::new(120.0);
    let colonnes: Vec<f32> = (0..1400)
        .map(|x| profil.band(x as f32 + 0.5, 400.0, 1000.0))
        .collect();
    let levels: Vec<LevelSource> = (0..=HALO_ALPHA)
        .map(|a| LevelSource::new((220, 120, 180), a))
        .collect();
    let sommet = index_du_sommet(&colonnes);
    let dernier = levels.len() - 1;

    for poids in [0.6f32, 1.4, 7.5, 19.0, 37.4, f32::from(HALO_ALPHA)] {
        let fond = [13u8, 14, 18, 255];

        // La voie de reference : un calcul et un melange par pixel.
        let mut attendu = vec![fond; colonnes.len()];
        for (pixel, colonne) in attendu.iter_mut().zip(&colonnes) {
            let k = ((poids * colonne).round() as usize).min(dernier);
            if k > 0 {
                blend_pixel(pixel, levels[k]);
            }
        }

        let mut obtenu = vec![fond; colonnes.len()];
        let (gauche, droite) = obtenu.split_at_mut(sommet);
        peindre_par_segments(gauche, poids, &colonnes[..sommet], &levels, Sens::Montant);
        peindre_par_segments(
            droite,
            poids,
            &colonnes[sommet..],
            &levels,
            Sens::Descendant,
        );

        let ecart = obtenu
            .iter()
            .zip(&attendu)
            .position(|(o, a)| o != a)
            .map(|i| (i, obtenu[i], attendu[i]));
        assert!(ecart.is_none(), "poids {poids} : ecart au pixel {ecart:?}");
    }
}

/// **Le nombre de bandes ne change aucun pixel d'une lueur.**
///
/// La garantie que la charte exige de toute adaptation : elle change *comment* on arrive au
/// resultat, jamais le resultat. Ici les voies sont « un fil » et « n fils », et c'est le
/// seul moyen de savoir qu'une bande ne decale pas sa part d'une ligne -- le defaut typique
/// d'un decoupage, et celui qui se verrait le moins sur un degrade.
///
/// Une lueur est justement le pire cas pour un tel defaut : son profil est continu, donc un
/// decalage d'une ligne ne fait pas de trou franc, seulement une bavure qu'on prendrait pour
/// du flou.
#[test]
fn test_les_bandes_ne_changent_aucun_pixel_d_une_lueur() {
    let (largeur, hauteur) = (320u32, 260u32);
    // Trois lueurs qui se CHEVAUCHENT et debordent de l'ecran : c'est la composition
    // successive qui doit rester identique, pas seulement une lueur isolee.
    let lueurs = [
        (
            HaloBox {
                left: 40.0,
                top: 30.0,
                right: 200.0,
                bottom: 90.0,
                sigma: 14.0,
                carte: None,
            },
            (220u8, 90u8, 60u8),
        ),
        (
            HaloBox {
                left: 120.0,
                top: 100.5,
                right: 300.0,
                bottom: 170.5,
                sigma: 9.0,
                carte: None,
            },
            (60u8, 200u8, 180u8),
        ),
        (
            HaloBox {
                left: -30.0,
                top: 190.0,
                right: 90.0,
                bottom: 300.0,
                sigma: 20.0,
                carte: None,
            },
            (120u8, 120u8, 240u8),
        ),
    ];

    let peindre = |fils: usize| -> Vec<u8> {
        let mut pixmap = Pixmap::new(largeur, hauteur).expect("l'ecran");
        pixmap.fill(background());
        // Le chemin de production, exactement : ce sont `LueurPrete` et `peindre_en` qui
        // portent le decalage, donc c'est eux qu'il faut eprouver -- une boucle ecrite dans
        // le test prouverait que le test sait decaler, pas que le rendu sait.
        let pretes: Vec<LueurPrete> = lueurs
            .iter()
            .filter_map(|(halo, rgb)| {
                LueurPrete::nouvelle(*halo, *rgb, HALO_ALPHA, largeur as i32, hauteur as i32)
            })
            .collect();
        peindre_en(fils, &mut pixmap.as_mut(), &pretes);
        pixmap.data().to_vec()
    };

    let temoin = peindre(1);
    // Des nombres premiers entre eux avec la hauteur : les frontieres ne retombent jamais
    // deux fois au meme endroit.
    for fils in [2, 3, 5, 7, 16, 64] {
        let vu = peindre(fils);
        let ecarts = temoin.iter().zip(&vu).filter(|(a, b)| a != b).count();
        assert_eq!(ecarts, 0, "{fils} bandes changent {ecarts} octets");
    }
}

/// **La découpe par segments et frange rend exactement la loi pixel par pixel** (LUEUR-1).
///
/// Le peintre ne calcule pixel par pixel que la frange d'un pixel au bord de la carte, saute
/// son intérieur, et garde ses segments de niveau constant partout ailleurs. La référence
/// ci-dessous calcule chaque pixel : `arrondi(poids × colonne × (1 − couverture))`, composé
/// par la même source. L'égalité est stricte.
#[test]
fn test_lueur_1_la_decoupe_rend_la_loi_au_bit_pres() {
    use glucose_core::membrane_forme::{couverture_d_un_plein, Arrondi};
    let (l, h) = (360u32, 240u32);
    // Des bords entiers, et un balayage d'un pixel en dixièmes : la portée tombe tantôt sur
    // une frontière de pixel, tantôt entre deux, et seule la seconde éprouve son interpolation.
    let balayage = (0..10).map(|k| (40.0 + k as f32 * 0.1, 30.0 + k as f32 * 0.13, 32.0));
    for (x, y, r) in [
        (60.3f32, 70.6f32, 32.0f32),
        (50.0, 40.0, 0.0),
        (80.7, 90.2, 30.0),
    ]
    .into_iter()
    .chain(balayage)
    {
        let carte = Arrondi::nouveau(x, y, 200.0, 70.0, r);
        let halo = HaloBox {
            left: x - 30.0,
            top: y - 30.0,
            right: x + 230.0,
            bottom: y + 100.0,
            sigma: 30.0,
            carte: Some(carte),
        };
        let rgb = (200, 120, 60);
        let mut obtenu = tiny_skia::Pixmap::new(l, h).expect("pixmap");
        obtenu.fill(tiny_skia::Color::BLACK);
        draw_halo(&mut obtenu.as_mut(), halo, rgb, HALO_ALPHA);

        let mut attendu = tiny_skia::Pixmap::new(l, h).expect("pixmap");
        attendu.fill(tiny_skia::Color::BLACK);
        let profil = EdgeProfile::new(halo.sigma);
        let niveaux: Vec<LevelSource> =
            (0..=HALO_ALPHA).map(|a| LevelSource::new(rgb, a)).collect();
        let (pixels, _) = attendu.data_mut().as_chunks_mut::<4>();
        for py in 0..h {
            let cy = py as f32 + 0.5;
            let poids = profil.band(cy, halo.top, halo.bottom) * f32::from(HALO_ALPHA);
            if poids < 0.5 {
                continue;
            }
            for px in 0..l {
                let cx = px as f32 + 0.5;
                let colonne = profil.band(cx, halo.left, halo.right);
                let reste = 1.0 - couverture_d_un_plein(carte.distance(cx, cy));
                let k = ((poids * colonne * reste).round() as usize).min(niveaux.len() - 1);
                if k > 0 {
                    blend_pixel(&mut pixels[(py * l + px) as usize], niveaux[k]);
                }
            }
        }
        let differents = obtenu
            .data()
            .iter()
            .zip(attendu.data())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(differents, 0, "carte en ({x}, {y}), rayon {r}");
    }
}

/// **La géométrie donne à la lueur la carte qui la découpe** : sa boîte à l'écran, et le
/// rayon de ses coins, mis à l'échelle comme elle (LUEUR-1).
#[test]
fn test_lueur_1_la_geometrie_porte_la_carte() {
    let carte = crate::renderer::card::tests::probe_card("c", 10.0, 20.0);
    let vp = glucose_core::types::Viewport {
        x: 100.0,
        y: 50.0,
        scale: 2.0,
    };
    let halo = halo_geometry(
        &carte,
        (&vp, WorldScale::new(vp.scale, 1.0)),
        (1440.0, 900.0, 0.0),
        Eclat::REPOS,
    )
    .expect("visible");
    let attendue = glucose_core::membrane_forme::Arrondi::nouveau(120.0, 90.0, 400.0, 100.0, 64.0);
    assert_eq!(halo.carte, Some(attendue));
    let vive = halo_geometry(
        &carte,
        (&vp, WorldScale::new(vp.scale, 1.0)),
        (1440.0, 900.0, 0.0),
        Eclat::DESIGNEE,
    )
    .expect("visible");
    assert!(
        vive.left < halo.left && vive.sigma > halo.sigma,
        "désignée, elle s'étale"
    );
}

/// **L'éclat d'une carte suit sa vivacité** (LUEUR-2) : au repos, les nombres de repos ;
/// désignée à moitié, entre les deux ; pleinement, ceux de `isHighlightBox`.
#[test]
fn test_lueur_2_l_eclat_suit_la_vivacite() {
    let carte = crate::renderer::card::tests::probe_card("c", 0.0, 0.0);
    let alpha = |v: f32| Eclat::de(&carte, &[("c".to_string(), v)]).alpha();
    assert_eq!(alpha(0.0), HALO_ALPHA);
    assert_eq!(alpha(1.0), 102);
    assert!(alpha(0.5) > HALO_ALPHA && alpha(0.5) < 102);
    assert_eq!(
        Eclat::de(&carte, &[("autre".to_string(), 1.0)]),
        Eclat::REPOS,
        "une autre carte désignée n'avive pas celle-ci"
    );
}
