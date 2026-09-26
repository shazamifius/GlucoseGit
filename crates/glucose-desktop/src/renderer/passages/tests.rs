//! Où un passage se trouve dans une carte.

use super::*;
use crate::renderer::math::MathRenderer;

const SA_CARTE: &str = "bonjours\ntest\ntest\nbonjours";

fn rects(plages: &[(usize, usize)]) -> Vec<(f32, f32, f32, f32)> {
    rectangles(
        (&Typography::new(), &MathRenderer::new()),
        SA_CARTE,
        240.0,
        plages,
    )
}

/// **Le second « bonjours » est sur la quatrième ligne, et seulement là** ; sa largeur est
/// celle du mot, sa hauteur celle de la police — de sa montante à sa descente, autour de la
/// base que la typographie pose à `haut + corps`.
#[test]
fn test_fleche_4_le_passage_est_la_ou_est_le_mot() {
    let second = SA_CARTE.rfind("bonjours").expect("le second");
    let r = rects(&[(second, second + 8)]);
    assert_eq!(r.len(), 1, "une ligne");
    let boite = text_box(240.0);
    let typo = Typography::new();
    let m = typo
        .font(crate::typography::Face::Regular)
        .horizontal_line_metrics(boite.body)
        .expect("les métriques d'Inter");
    let haut = TEXT_ORIGIN.1 + 3.0 * boite.line_height;
    let (x, y, w, h) = r[0];
    assert_eq!(x, TEXT_ORIGIN.0);
    assert_eq!(y, haut + boite.body - m.ascent);
    assert_eq!(h, m.ascent - m.descent);
    assert!(
        h < boite.line_height,
        "le cadre est la police, pas la ligne"
    );
    let (mot, _) = typo.measure_text("bonjours", boite.body, crate::typography::Face::Regular);
    assert!((w - mot).abs() < 0.01, "{w} contre {mot}");
}

/// **Un passage à cheval sur deux lignes rend deux rectangles**, un par ligne.
#[test]
fn test_fleche_4_un_passage_sur_deux_lignes() {
    let r = rects(&[(2, 12)]);
    assert_eq!(r.len(), 2, "{r:?}");
    assert!(r[1].1 > r[0].1);
}

/// **Le cadre épouse l'encre du mot, à tout zoom** : à un pixel et demi près de chaque côté,
/// et la police entière dedans — sans la hauteur de ligne qui le faisait paraître décalé vers
/// le bas. Mesuré sur l'image, pas sur les nombres.
#[test]
fn test_fleche_4_le_cadre_epouse_l_encre_a_tout_zoom() {
    use glucose_core::store::Store;
    use glucose_core::types::Viewport;
    let texte = "testetsetetstetsetest\n\ntestes";
    let debut = texte.find("testes").expect("testes");
    for zoom in [0.6f64, 0.9, 1.0, 1.35, 2.0] {
        let mut store = Store::new("encre");
        let board = store.project.active_board_id.clone();
        store.add_annotation(&board, Annotation::text("c", 0.0, 0.0, texte));
        store.clear_selection();
        store.set_viewport(
            &board,
            Viewport {
                x: 100.0,
                y: 100.0,
                scale: zoom,
            },
        );
        let store = crate::bench::ouvert(store);
        let mut renderer = crate::renderer::Renderer::new();
        let mut ui = crate::ui::UiState::new();
        let image = crate::bench::render_frame(&mut renderer, &mut ui, &store, 900, 700);
        let r = rectangles(
            (&Typography::new(), &MathRenderer::new()),
            texte,
            240.0,
            &[(debut, debut + 6)],
        )[0];
        let (x0, y0) = (100.0 + f64::from(r.0) * zoom, 100.0 + f64::from(r.1) * zoom);
        let (x1, y1) = (x0 + f64::from(r.2) * zoom, y0 + f64::from(r.3) * zoom);
        let mut encre = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for y in (y0 as u32).saturating_sub(12)..(y1 as u32 + 12) {
            for x in (x0 as u32).saturating_sub(12)..(x1 as u32 + 12) {
                let c = image.pixel(x, y).expect("dans l'image");
                if c.red().min(c.green()).min(c.blue()) > 110 {
                    let (x, y) = (f64::from(x), f64::from(y));
                    encre = (
                        encre.0.min(x),
                        encre.1.min(y),
                        encre.2.max(x + 1.0),
                        encre.3.max(y + 1.0),
                    );
                }
            }
        }
        assert!(
            (encre.0 - x0).abs() <= 1.5 && (x1 - encre.2).abs() <= 1.5,
            "zoom {zoom} : cadre [{x0}, {x1}], encre [{}, {}]",
            encre.0,
            encre.2
        );
        assert!(
            encre.1 >= y0 - 0.5 && encre.3 <= y1 + 1.0,
            "zoom {zoom} : l'encre [{}, {}] tient dans [{y0}, {y1}]",
            encre.1,
            encre.3
        );
        let ligne = f64::from(text_box(240.0).line_height) * zoom;
        assert!(
            y1 - y0 < 0.9 * ligne,
            "zoom {zoom} : le cadre n'est pas la ligne"
        );
    }
}

/// **Une formule est un atome** : un passage qui la touche prend la boîte de la formule
/// dessinée — sa largeur rendue, et non celle de sa source, qu'on ne voit pas.
#[test]
fn test_ancre_ux_une_formule_se_designe_par_sa_boite_dessinee() {
    let texte = r"avant
$$\frac{a+b}{c}$$
apres";
    let math = MathRenderer::new();
    let typo = Typography::new();
    let debut = texte.find("$$").expect("la formule");
    let r = rectangles((&typo, &math), texte, 240.0, &[(debut + 3, debut + 5)]);
    assert_eq!(r.len(), 1, "une boîte pour la formule : {r:?}");
    let (l, h, d) = math
        .measure(
            r"\frac{a+b}{c}",
            crate::renderer::richtext::mode_of(true),
            text_box(240.0).body,
        )
        .expect("la formule se compose");
    assert!(
        (r[0].2 - l).abs() < 0.01,
        "la largeur dessinée {l}, pas celle de la source"
    );
    assert!((r[0].3 - (h + d)).abs() < 0.01, "sa hauteur dessinée");
}

/// Une carte de `texte`, large de 400 unités, posée en (10, 10) et vue à ×2, son passage
/// `plage` éclairé en rouge pur — le contenu de la carte seul, comme sa texture le porte (sans
/// la lueur, qui passe au-dessus).
fn carte_eclairee(texte: &str, plage: (usize, usize)) -> tiny_skia::Pixmap {
    use glucose_core::types::Viewport;
    let renderer = crate::renderer::Renderer::new();
    let kit = renderer.kit();
    let vp = Viewport {
        x: 10.0,
        y: 10.0,
        scale: 2.0,
    };
    let mut image = tiny_skia::Pixmap::new(1000, 200).expect("pixmap");
    image.fill(tiny_skia::Color::BLACK);
    let plages = [plage];
    crate::renderer::card::peindre_le_texte_seul(
        kit,
        &mut image.as_mut(),
        (texte, 400.0, (255, 255, 255)),
        (vp, 1.0),
        Some(Eclaires {
            plages: &plages,
            teinte: (255, 0, 0),
            style: &SUR_LA_CARTE,
        }),
    );
    image
}

/// L'étendue horizontale de l'encre du caractère à l'octet `o`, à l'écran de [`carte_eclairee`] :
/// là où la mise en page ouverte le pose, plus ce que son glyphe couvre.
fn encre_du_caractere(texte: &str, plage: (usize, usize), o: usize) -> (u32, u32) {
    let typo = Typography::new();
    let math = MathRenderer::new();
    let eclaires = Eclaires {
        plages: &[plage],
        teinte: (255, 0, 0),
        style: &SUR_LA_CARTE,
    };
    let ouverte = mise_en_page(
        (&typo, &math),
        (texte, 400.0),
        TextMode::Rendered,
        Some(&eclaires),
    );
    let x = crate::renderer::richtext::hit::x_du_caractere(
        &typo,
        &ouverte,
        &ouverte.lines[0],
        texte,
        o,
        14.0,
    );
    let ch = texte[o..].chars().next().expect("un caractere");
    let m = typo
        .font(crate::typography::Face::Regular)
        .metrics(ch, 28.0);
    let gauche = 10.0 + (TEXT_ORIGIN.0 + x) * 2.0 + m.xmin as f32;
    (
        gauche.floor() as u32,
        (gauche + m.width as f32).ceil() as u32,
    )
}

/// **PASSAGE-2 — le cadre ne touche aucune lettre voisine**, mesuré sur l'image : sa capture du
/// 26/09, un passage pris au milieu d'un mot. Là où l'encre des deux voisines se pose, aucun
/// pixel n'est teinté — ni par le fond, ni par le liseré, ni par les lettres ; et les lettres du
/// passage, elles, le sont.
#[test]
fn test_passage_2_le_cadre_ne_touche_aucune_lettre_voisine() {
    let texte = "testetsetetstetsetes";
    let plage = (3, 16);
    let image = carte_eclairee(texte, plage);
    let (haut, bas) = (40u32, 80u32);
    for voisine in [plage.0 - 1, plage.1] {
        let (x0, x1) = encre_du_caractere(texte, plage, voisine);
        for y in haut..bas {
            for x in x0..x1 {
                let c = image.pixel(x, y).expect("dans l'image");
                // L'encre du texte est un blanc bleuté, le noir est gris : un rouge plus fort
                // que le vert ne peut venir que du cadre ou de la teinte du passage.
                assert!(
                    c.red() <= c.green().saturating_add(2),
                    "la voisine {voisine} est teintee en ({x}, {y}) : {c:?}"
                );
            }
        }
    }
    let (x0, x1) = encre_du_caractere(texte, plage, plage.0 + 1);
    let teinte = (x0..x1)
        .flat_map(|x| (haut..bas).map(move |y| (x, y)))
        .filter_map(|(x, y)| image.pixel(x, y))
        .any(|c| c.red() > 200 && c.green() < 60);
    assert!(teinte, "les lettres du passage ne sont pas teintees");
}

/// **Le texte d'un passage prend la teinte de sa carte**, comme le `<mark>` de Tauri — ses
/// lettres, et pas un rectangle : la plus claire est teintée.
#[test]
fn test_ancre_ux_le_texte_du_passage_prend_la_teinte() {
    let texte = "un mot ici";
    let plage = (3, 6);
    let image = carte_eclairee(texte, plage);
    let (x0, x1) = (
        encre_du_caractere(texte, plage, 3).0,
        encre_du_caractere(texte, plage, 5).1,
    );
    let mut plus_clair = (0u8, 0u8, 0u8);
    for y in 30..90u32 {
        for x in x0..x1 {
            let c = image.pixel(x, y).expect("dans l'image");
            if c.red() > plus_clair.0 {
                plus_clair = (c.red(), c.green(), c.blue());
            }
        }
    }
    assert!(
        plus_clair.0 > 200 && plus_clair.1 < 60,
        "l'encre du mot est teintee : {plus_clair:?}"
    );
}
