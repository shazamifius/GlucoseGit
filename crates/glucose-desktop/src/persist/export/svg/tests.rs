//! L'export SVG (P5, fiches 69 et 70).
//!
//! **La superposition** d'abord : la vitrine de l'export ([`crate::bench::vitrine`]), rendue par
//! le canevas et par un lecteur indépendant (`resvg`, l'oracle), doit donner la même image —
//! à un pixel de place près, et à l'anticrénelage de deux rastériseurs près. Puis une épreuve
//! ciblée par ce que l'image seule dit mal : les octets d'une photo, les glyphes en tracés.

use crate::app::GlucoseApp;
use crate::bench::vitrine::vitrine_de_l_export;
use crate::epreuve::Dossier;
use crate::persist::export::png::{moteur, preparer};
use crate::ui::UiState;
use glucose_core::types::BoardImage;
use tiny_skia::{Pixmap, Transform};

/// **Le canevas et le SVG, de la même scène** : les pixels de l'export en image, et ceux que
/// `resvg` tire du SVG.
fn les_deux_images(app: &GlucoseApp) -> (Pixmap, Pixmap) {
    let scene = preparer(&app.store).expect("une scène");
    let (l, h) = (scene.largeur, scene.hauteur);
    let mut canevas = Pixmap::new(l, h).expect("une image");
    let mut ui = UiState::new();
    moteur(app.peintre_de_l_export()).rendre_pour_l_export(
        &mut canevas.as_mut(),
        &scene.store,
        &mut ui,
        scene.vue,
    );
    let svg = app.rendre_le_svg().expect("un SVG");
    let arbre = resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default())
        .expect("un SVG que l'oracle lit");
    let mut lu = Pixmap::new(l, h).expect("une image");
    resvg::render(&arbre, Transform::identity(), &mut lu.as_mut());
    (canevas, lu)
}

/// Les trois canaux de chaque pixel, lus une fois.
fn canaux(p: &Pixmap) -> Vec<[i32; 3]> {
    p.pixels()
        .iter()
        .map(|c| {
            let c = c.demultiply();
            [c.red(), c.green(), c.blue()].map(i32::from)
        })
        .collect()
}

/// L'écart de chaque pixel de `a` au plus proche de son voisinage 3 × 3 dans `b` — un décalage
/// d'un pixel ne compte pas.
fn ecart_au_voisinage(a: &[[i32; 3]], b: &[[i32; 3]], (l, h): (usize, usize)) -> Vec<i32> {
    let mut e = vec![0; l * h];
    for y in 0..h {
        for x in 0..l {
            let pa = a[y * l + x];
            let mut best = i32::MAX;
            for vy in y.saturating_sub(1)..(y + 2).min(h) {
                for vx in x.saturating_sub(1)..(x + 2).min(l) {
                    let pb = b[vy * l + vx];
                    let d = (0..3).map(|k| (pa[k] - pb[k]).abs()).max().unwrap_or(0);
                    best = best.min(d);
                }
            }
            e[y * l + x] = best;
        }
    }
    e
}

/// **La forme** : la plus grande moyenne, sur un bloc de 8 × 8 pixels, de l'écart au
/// voisinage, dans les deux sens. Un liseré d'anticrénelage, d'un pixel de large, s'y dilue ;
/// un mot absent ou une ligne décalée de deux pixels, non.
fn forme(a: &Pixmap, b: &Pixmap) -> f64 {
    let (l, h) = (a.width() as usize, a.height() as usize);
    let (ca, cb) = (canaux(a), canaux(b));
    let (ab, ba) = (
        ecart_au_voisinage(&ca, &cb, (l, h)),
        ecart_au_voisinage(&cb, &ca, (l, h)),
    );
    let (bl, bh) = (l / 8, h / 8);
    let mut pire = 0.0_f64;
    for by in 0..bh {
        for bx in 0..bl {
            let mut somme = 0;
            for y in by * 8..by * 8 + 8 {
                for x in bx * 8..bx * 8 + 8 {
                    somme += ab[y * l + x].max(ba[y * l + x]);
                }
            }
            pire = pire.max(f64::from(somme) / 64.0);
        }
    }
    pire
}

/// **La teinte** : le plus grand écart signé moyen d'un canal sur un bloc de 16 × 16 pixels.
/// L'anticrénelage s'y compense ; une teinte faussée, une lueur absente, non.
fn teinte(a: &Pixmap, b: &Pixmap) -> f64 {
    let l = a.width() as usize;
    let (bl, bh) = (l / 16, a.height() as usize / 16);
    let (ca, cb) = (canaux(a), canaux(b));
    let mut pire = 0.0_f64;
    for by in 0..bh {
        for bx in 0..bl {
            let mut somme = [0_i32; 3];
            for y in by * 16..by * 16 + 16 {
                for x in bx * 16..bx * 16 + 16 {
                    let (pa, pb) = (ca[y * l + x], cb[y * l + x]);
                    for k in 0..3 {
                        somme[k] += pa[k] - pb[k];
                    }
                }
            }
            let m = somme.map(|s| (f64::from(s) / 256.0).abs());
            pire = pire.max(m[0].max(m[1]).max(m[2]));
        }
    }
    pire
}

/// **Le canevas et le SVG se superposent** (P5, fiche 70) — chaque dessin de la vitrine : cartes
/// et leurs lueurs, formules, listes, code, barré, post-it et pilule, membrane et son titre,
/// flèches et leur libellé, badge de relation, photos recadrée et tournée, réglettes de
/// domaines, zones et leurs textes, cases du storyboard et les leurs, dossiers vide et plein.
///
/// Les deux bornes sont **mesurées** sur cette vitrine (fiche 70 § 4) : la paire juste donne
/// 26,6 en forme et 6,3 en teinte — des liserés d'anticrénelage, le canevas qui cale les photos
/// et les glyphes des formules sur la grille de pixels là où le SVG les pose exactement. Les
/// fautes simulées les plus fines donnent 56,6 (une ligne décalée de deux pixels) et 13,3 (une
/// teinte faussée de douze niveaux). Chaque borne est le milieu des deux.
#[test]
fn test_p5_le_canevas_et_le_svg_se_superposent() {
    let dossier = Dossier::nouveau("export-svg-superposition");
    let app = vitrine_de_l_export(&dossier);
    let (canevas, lu) = les_deux_images(&app);
    let (f, t) = (forme(&canevas, &lu), teinte(&canevas, &lu));
    assert!(f <= 41.6, "la forme : {f:.1} (bruit mesuré 26,6)");
    assert!(t <= 9.8, "la teinte : {t:.1} (bruit mesuré 6,3)");
}

/// La valeur de l'attribut `nom` dans un élément.
fn attribut(element: &str, nom: &str) -> f32 {
    let debut = element
        .find(&format!(" {nom}=\""))
        .unwrap_or_else(|| panic!("{nom} dans {element}"))
        + nom.len()
        + 3;
    let fin = element[debut..].find('"').expect("un attribut fermé") + debut;
    element[debut..fin].parse().expect("un nombre")
}

/// **Chaque flou ne travaille que là où sa lueur peut changer un pixel** (fiche 71) : la
/// région de chaque filtre tient dans le cadre de l'export — la boîte de la lueur élargie de sa
/// portée, que la marge de l'export (`debord_au_repos`) contient par construction. La fiche 70
/// déclarait ±100 000 unités pour tous : la superposition passait, et un lecteur floutait un
/// carré de 200 000 de côté par lueur — 28 s pour `resvg` sur la vitrine.
#[test]
fn test_p5_chaque_flou_tient_dans_le_cadre_de_l_export() {
    let dossier = Dossier::nouveau("export-svg-flous");
    let app = vitrine_de_l_export(&dossier);
    let scene = preparer(&app.store).expect("une scène");
    let (l, h) = (scene.largeur as f32, scene.hauteur as f32);
    let svg = app.rendre_le_svg().expect("un SVG");
    let filtres: Vec<&str> = svg
        .match_indices("<filter ")
        .map(|(i, _)| &svg[i..i + svg[i..].find('>').expect("un filtre fermé")])
        .collect();
    assert!(
        filtres.len() >= 5,
        "les lueurs de la vitrine : {}",
        filtres.len()
    );
    for f in filtres {
        let (x, y) = (attribut(f, "x"), attribut(f, "y"));
        let (fin_x, fin_y) = (x + attribut(f, "width"), y + attribut(f, "height"));
        assert!(
            x >= 0.0 && y >= 0.0 && fin_x <= l && fin_y <= h,
            "{f} déborde du cadre {l} × {h}"
        );
    }
}

/// **Les textes peu contrastés sont écrits**, et le dossier vide dit quoi faire sans badge
/// (fiche 71) : le nom d'une zone, bleu sur son bandeau bleu, sa phrase grise, la description
/// d'une case, « Double-clic pour ouvrir » — la superposition les dilue sous son seuil, et leur
/// absence ne la faisait pas tomber.
#[test]
fn test_p5_les_textes_peu_contrastes_sont_ecrits() {
    let dossier = Dossier::nouveau("export-svg-textes");
    let app = vitrine_de_l_export(&dossier);
    let svg = app.rendre_le_svg().expect("un SVG");
    for texte in [
        "RÉFÉRENCE",
        "SKETCH",
        "Images de référence, inspirations",
        "Croquis exploratoires",
        "Plan large, la ville au loin",
        "Double-clic pour ouvrir",
    ] {
        let ecrit = format!(r#"aria-label="{texte}""#);
        assert!(svg.contains(&ecrit), "« {texte} » manque au SVG");
    }
    assert!(
        !svg.contains(r#"aria-label="0""#),
        "un dossier vide n'a pas de badge"
    );
}

/// **Une photo porte ses octets d'origine**, en `data:`, découpée à sa boîte ; un tableau vide
/// n'a rien à écrire.
#[test]
fn test_p5_une_photo_porte_ses_octets_d_origine() {
    let dossier = Dossier::nouveau("export-svg");
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    assert_eq!(
        app.rendre_le_svg(),
        None,
        "un tableau vide n'a rien à écrire"
    );
    let fichier = dossier.join("rouge.png");
    image::RgbaImage::from_pixel(10, 10, image::Rgba([220, 30, 30, 255]))
        .save(&fichier)
        .expect("une photo d'épreuve");
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new("rouge", 0.0, 0.0, 100.0, 100.0);
    (img.original_width, img.original_height) = (10.0, 10.0);
    img.src = Some(fichier.to_string_lossy().into_owned());
    app.store.add_image(&board, img);
    app.store.clear_selection();
    let svg = app.rendre_le_svg().expect("un SVG");
    let octets = std::fs::read(&fichier).expect("la photo");
    let attendu = format!(
        "data:image/png;base64,{}",
        crate::renderer::svg::base64(&octets)
    );
    assert!(svg.contains(&attendu), "les octets d'origine");
    assert!(svg.contains("<clipPath"), "découpée à sa boîte");
    assert!(svg.starts_with("<?xml") && svg.trim_end().ends_with("</svg>"));
}

/// **Le texte ne demande aucune police au lecteur** (fiche 70 § 1) : aucun `<text>`, chaque
/// glyphe défini une seule fois et posé autant de fois qu'il revient, chaque ligne lisible par
/// une machine dans son `aria-label` — une formule par sa source.
#[test]
fn test_p5_le_texte_est_en_traces_et_reste_lisible() {
    let dossier = Dossier::nouveau("export-svg-glyphes");
    let app = vitrine_de_l_export(&dossier);
    let svg = app.rendre_le_svg().expect("un SVG");
    assert!(!svg.contains("<text"), "aucune police demandée au lecteur");
    let definis = svg.matches(r#"<path id="g-"#).count();
    let poses = svg.matches(r##"<use xlink:href="#g-"##).count();
    assert!(definis > 50, "les glyphes de la vitrine : {definis}");
    assert!(poses > 4 * definis, "{poses} poses pour {definis} glyphes");
    assert!(svg.contains(r#"aria-label="Entropie de Shannon""#));
    assert!(svg.contains(r#"aria-label="\sum_{i=1}^{n} 2^{-\ell_i} \le 1""#));
    assert!(
        !svg.contains("$$"),
        "une formule s'écrit par ses glyphes, pas par sa source"
    );
}
