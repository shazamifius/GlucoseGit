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
        card.rect().expect("une carte a une boîte"),
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

    // Et il est celui de l'ombre CSS, l'étalement plus la portée du flou, à la force d'une lueur
    // sûre (PARENTE-2) — le disque d'avant montait à 410 sur la carte large.
    let ombre = f64::from(debord_au_repos(1.0));
    assert!(
        (large.1 - ombre).abs() < 1e-3,
        "le débord vertical vaut {}, l'ombre {ombre}",
        large.1
    );
    assert!(large.1 < 240.0 * 1.5 + 50.0, "loin du disque d'avant");
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
            card.rect().expect("une carte a une boîte"),
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
        card.rect().expect("une carte a une boîte"),
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
        card.rect().expect("une carte a une boîte"),
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

    let vp = Viewport::default();
    let board = store.active_board().expect("le board vient d'être rempli");

    // Tous les nœuds sont visibles : le test mesure la passe entière, pas le culling.
    let rangs = glucose_core::quadtree::tous_les_rangs(board);
    let mut index = glucose_core::quadtree::SpatialHash::new(1000.0);
    index.index_board(board);
    let pass = ViewPass {
        vp,
        visibles: &rangs,
        index: &index,
        contournement: None,
        header_h: 40.0,
        densite: 1.0,
    };
    // Frame de chauffe ; chaque texte sûr, pour que chaque lueur se peigne.
    let trousse = Trousse::sure_de(&store);
    let kit = trousse.kit();
    draw_halos(&mut pixmap.as_mut(), (&store, kit), (pass, &[], None));

    let started = std::time::Instant::now();
    draw_halos(&mut pixmap.as_mut(), (&store, kit), (pass, &[], None));
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
        carte.rect().expect("une carte a une boîte"),
        (&vp, WorldScale::new(vp.scale, 1.0)),
        (1440.0, 900.0, 0.0),
        Eclat::REPOS,
    )
    .expect("visible");
    let attendue = glucose_core::membrane_forme::Arrondi::nouveau(120.0, 90.0, 400.0, 100.0, 64.0);
    assert_eq!(halo.carte, Some(attendue));
    let vive = halo_geometry(
        carte.rect().expect("une carte a une boîte"),
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
    let alpha = |v: f32| Eclat::de(&carte, &[("c".to_string(), v)]).alpha(1.0);
    assert_eq!(alpha(0.0), HALO_ALPHA);
    assert_eq!(alpha(1.0), 102);
    assert!(alpha(0.5) > HALO_ALPHA && alpha(0.5) < 102);
    assert_eq!(
        Eclat::de(&carte, &[("autre".to_string(), 1.0)]),
        Eclat::REPOS,
        "une autre carte désignée n'avive pas celle-ci"
    );
}

// ── Fiche 100 — la lueur épouse la carte qu'on écrit ──────────────────────────

/// De quoi mesurer et peindre une carte dans une épreuve : la police, les formules, les mises en
/// page, les teintes de domaine, le thème.
struct Trousse {
    typo: crate::typography::Typography,
    math: crate::renderer::math::MathRenderer,
    mises: crate::renderer::card::MisesEnPage,
    teintes: crate::renderer::domain::DomainTints,
    theme: crate::theme::Theme,
    sens: crate::renderer::sens::SensDesTextes,
}

impl Trousse {
    fn new() -> Self {
        Self {
            typo: crate::typography::Typography::new(),
            math: crate::renderer::math::MathRenderer::new(),
            mises: Default::default(),
            teintes: crate::renderer::domain::DomainTints::new(),
            theme: crate::theme::Theme::dark(),
            sens: Default::default(),
        }
    }

    /// La même, chaque texte de ce document sûr : chaque lueur se peint, pleine (PARENTE-2).
    fn sure_de(store: &Store) -> Self {
        let mut trousse = Self::new();
        trousse.sens = crate::renderer::sens::SensDesTextes::tous_surs(store);
        trousse
    }

    fn kit(&self) -> PaintKit<'_> {
        PaintKit {
            typography: &self.typo,
            math: &self.math,
            mises_en_page: &self.mises,
            tints: &self.teintes,
            theme: &self.theme,
            sens: &self.sens,
        }
    }
}

/// Ce que les deux voies font de la lueur de ce tableau, à l'écran de 1440 × 900 : les pixels du
/// processeur, et les bords de la boîte que la carte graphique pose.
fn lueurs_des_deux_voies(
    store: &Store,
    trousse: &Trousse,
    edition: Option<&TextEditSession>,
) -> (Vec<u8>, Vec<[f32; 4]>) {
    let board = store.active_board().expect("un tableau actif");
    let rangs = glucose_core::quadtree::tous_les_rangs(board);
    let mut index = glucose_core::quadtree::SpatialHash::new(1000.0);
    index.index_board(board);
    let pass = ViewPass {
        vp: Viewport {
            x: 400.0,
            y: 150.0,
            scale: 1.5,
        },
        visibles: &rangs,
        index: &index,
        contournement: None,
        header_h: 0.0,
        densite: 1.0,
    };
    let mut pixmap = Pixmap::new(1440, 900).expect("pixmap");
    pixmap.fill(background());
    let kit = trousse.kit();
    draw_halos(&mut pixmap.as_mut(), (store, kit), (pass, &[], edition));
    let posees =
        crate::renderer::voies::lueurs_a_poser((store, kit), (pass, &[], edition), (1440.0, 900.0));
    let bords = posees
        .iter()
        .map(|l| [l.gauche, l.haut, l.droite, l.bas])
        .collect();
    (pixmap.data().to_vec(), bords)
}

/// **Pendant une saisie qui passe à la ligne, la lueur épouse la carte qui grandit** (fiche 100).
///
/// Son essai du 9/11 sous niri : on écrit, la ligne passe, et le contour blanc se décroche d'une
/// forme sombre en haut à gauche. La carte se dessine à la hauteur que pousse le texte qu'on
/// écrit ; le document ne l'apprend qu'à la validation (TEXT-FIT-1), et la lueur lisait le
/// document. Sur les deux voies, la lueur **pendant** la saisie doit être, au pixel près, celle
/// de la carte **après** — le même texte validé, la hauteur écrite.
#[test]
fn test_la_lueur_suit_la_carte_qu_on_ecrit_sur_les_deux_voies() {
    let ecrit = "Un texte assez long pour passer a la ligne tout seul et voir le contour, puis \
                 encore une ligne, et une autre, pour que la carte grandisse vraiment.";
    let mut store = Store::new("saisie");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, sized_card(Some(240.0), Some(60.0)));
    // Sûre : une lueur que l'IA laisserait éteinte ne dirait rien de sa forme.
    let trousse = Trousse::sure_de(&store);
    let saisie = TextEditSession {
        ann_id: "halo".into(),
        buffer: ecrit.into(),
        selection: glucose_core::text::Selection::at(ecrit.len()),
        goal_x: None,
        blink_timer: std::time::Instant::now(),
        curseur_visible: true,
    };
    let pendant = lueurs_des_deux_voies(&store, &trousse, Some(&saisie));

    // La validation : le texte s'écrit, et la hauteur qu'il pousse avec lui.
    let haute =
        crate::renderer::card::text_card_fit_height(&trousse.typo, &trousse.math, ecrit, 240.0);
    assert!(haute > 120.0, "le texte doit pousser la carte : {haute}");
    let mut validee = sized_card(Some(240.0), Some(haute));
    if let Annotation::Text { text, .. } = &mut validee {
        *text = ecrit.into();
    }
    let mut apres = Store::new("validee");
    let board = apres.project.active_board_id.clone();
    apres.add_annotation(&board, validee);
    let apres = lueurs_des_deux_voies(&apres, &trousse, None);

    assert_eq!(pendant.1, apres.1, "la boîte que la carte graphique pose");
    assert!(
        pendant.0 == apres.0,
        "les pixels du processeur : la lueur pendant la saisie n'est pas celle de la carte validée"
    );
}

/// Ce que les deux entrées du rendu font de l'application telle qu'elle est — sa saisie en
/// cours comprise : l'image du processeur ([`crate::renderer::Renderer::render`]) et les lueurs
/// que la voie graphique confie ([`crate::renderer::Renderer::rendre_les_couches`]).
fn rendus_de_l_application(
    app: &mut crate::app::GlucoseApp,
) -> (Pixmap, Vec<crate::present::lueurs_gpu::Lueur>) {
    let (l, h) = (1200, 800);
    let overlay = crate::params::SceneOverlay {
        editing: app.editing_session.as_ref(),
        ..crate::params::SceneOverlay::sans_rien(&app.active_guides)
    };
    let pointeur = crate::params::Pointer { x: 0.0, y: 0.0 };
    let mut image = Pixmap::new(l, h).expect("pixmap");
    app.renderer.render(
        &mut image.as_mut(),
        &app.store,
        &mut app.ui,
        overlay,
        pointeur,
        crate::renderer::Regard::immobile(),
    );
    let (mut dessous, mut dessus) = (
        Pixmap::new(l, h).expect("pixmap"),
        Pixmap::new(l, h).expect("pixmap"),
    );
    let confie = app.renderer.rendre_les_couches(
        &mut dessous.as_mut(),
        &mut dessus.as_mut(),
        &app.store,
        (&mut app.ui, pointeur),
        overlay,
        crate::renderer::Regard::immobile(),
    );
    (image, confie.lueurs)
}

/// **La même chose par les entrées du rendu**, celles que Glucose emploie à chaque image : une
/// épreuve qui appelle la lueur directement ne verrait pas une saisie oubliée en chemin.
///
/// Une saisie s'ouvre dans une vraie application, sur un texte court ; le texte qu'on écrit fait
/// grandir la carte. On rend, on valide, on rend encore. La voie graphique confie la même lueur ;
/// le processeur peint les mêmes pixels **autour** de la carte — dedans, le curseur et le mode
/// d'édition changent ce qui s'y lit, et c'est voulu (MODE-1).
#[test]
fn test_la_lueur_suit_la_saisie_par_les_entrees_du_rendu() {
    const VUE: (f64, f64) = (200.0, 150.0);
    let mut app = crate::app::GlucoseApp::new();
    let tableau = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        b.folders.clear();
    }
    app.store.set_viewport(
        &tableau,
        Viewport {
            scale: 1.0,
            x: VUE.0,
            y: VUE.1,
        },
    );
    let mut carte = Annotation::text("t", 0.0, 0.0, "court");
    if let Annotation::Text {
        width,
        height,
        color,
        ..
    } = &mut carte
    {
        (*width, *height) = (Some(240.0), Some(60.0));
        // Une couleur choisie : sa lueur est pleine — l'IA, seule, laisserait un texte seul éteint.
        *color = Some("#f472b6".into());
    }
    app.store.add_annotation(&tableau, carte);
    app.ui.current_toast = None;
    let ecrit = "Un texte assez long pour passer a la ligne tout seul et voir le contour, puis \
                 encore une ligne, et une autre, pour que la carte grandisse vraiment.";
    app.start_text_edit("t".into(), ecrit.into());
    let pendant = rendus_de_l_application(&mut app);
    app.commit_editing();
    let apres = rendus_de_l_application(&mut app);

    let boite = app
        .store
        .active_board()
        .and_then(|b| b.annotations.first())
        .and_then(Annotation::rect)
        .expect("la carte validée");
    assert!(
        boite.height > 120.0,
        "le texte a poussé la carte : {}",
        boite.height
    );
    assert_eq!(pendant.1, apres.1, "la lueur que la voie graphique pose");

    // Autour de la carte : au-delà de son anneau et de ses poignées, en deçà de la portée de sa
    // lueur.
    let (x0, y0) = (boite.left + VUE.0, boite.top + VUE.1);
    let (x1, y1) = (x0 + boite.width, y0 + boite.height);
    let (anneau, portee) = (8.0, 60.0);
    let mut ecarts = 0;
    for (i, (a, b)) in pendant.0.pixels().iter().zip(apres.0.pixels()).enumerate() {
        let (x, y) = ((i % 1200) as f64, (i / 1200) as f64);
        let autour = x > x0 - portee && x < x1 + portee && y > y0 - portee && y < y1 + portee;
        let dedans = x > x0 - anneau && x < x1 + anneau && y > y0 - anneau && y < y1 + anneau;
        ecarts += usize::from(autour && !dedans && a != b);
    }
    assert_eq!(
        ecarts, 0,
        "pixels autour de la carte qui changent à la validation"
    );
}

/// La même trousse, chaque texte de ce document de cette sûreté — dans une famille de cette teinte,
/// ou dans aucune.
fn trousse_a(store: &Store, (teinte, surete): (Option<f64>, f32)) -> Trousse {
    let mut trousse = Trousse::new();
    let avis = store
        .active_board()
        .map(|b| {
            glucose_core::sens::textes_du_tableau(b)
                .map(|(id, _)| {
                    let avis = crate::parente::AvisDUnTexte {
                        famille: teinte.map(|_| 0),
                        teinte,
                        surete,
                    };
                    (id.to_string(), avis)
                })
                .collect()
        })
        .unwrap_or_default();
    trousse.sens.poser(&avis);
    trousse
}

/// **PARENTE-2 — l'opacité dit la sûreté, et chaque bloc garde son contour** : sans aucune sûreté,
/// la lueur du bloc (les 38 de Tauri) ; à moitié sûre, elle brille plus ; sûre, plus encore. Sur
/// les deux voies. Sans famille, pour que seule l'opacité change : la vivacité suit aussi la
/// sûreté, et suffisait à faire passer une opacité figée (fiche 106). **La preuve à l'envers** : un
/// tableau sans texte n'a aucune lueur — celle d'un texte seul n'est pas le fond.
#[test]
fn parente_2_l_opacite_dit_la_surete_sur_les_deux_voies() {
    let mut store = Store::new("surete");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, sized_card(Some(240.0), Some(60.0)));
    let (rien, _) = lueurs_des_deux_voies(&Store::new("vide"), &Trousse::new(), None);
    let energie = |p: &[u8]| -> u64 {
        p.iter()
            .zip(&rien)
            .map(|(a, b)| u64::from(a.abs_diff(*b)))
            .sum()
    };
    let mesure = |surete: f32| {
        let trousse = trousse_a(&store, (None, surete));
        let (pixels, boites) = lueurs_des_deux_voies(&store, &trousse, None);
        assert_eq!(
            boites.len(),
            1,
            "la carte graphique pose la lueur ({surete})"
        );
        energie(&pixels)
    };
    let (e0, e_moitie, e1) = (mesure(0.0), mesure(0.5), mesure(1.0));
    assert!(e0 > 0, "sans sûreté, le bloc garde son contour");
    assert!(
        e0 < e_moitie && e_moitie < e1,
        "la lueur suit la sûreté : {e0} < {e_moitie} < {e1}"
    );
}
