//! Les lois du dessin des formules. Elles vérifient ce que la géométrie seule ne peut pas :
//! que les fontes sont là, que l'encre tombe au bon endroit, et que le cache ne ment pas.

use super::*;
use tiny_skia::Pixmap;

/// Combien de pixels d'un pixmap ne sont pas transparents — la mesure d'encre la plus simple
/// et la plus sûre.
fn encre(p: &Pixmap) -> usize {
    p.pixels().iter().filter(|px| px.alpha() > 0).count()
}

/// La plume des tests : une ligne de base et un corps, en pixels.
fn plume(x: f32, y: f32, font_size: f32) -> Pen {
    Pen { x, y, font_size }
}

const BLANC: Color = Color::WHITE;

/// **Les vingt fontes de KaTeX sont là et se chargent.** Une seule manquante, et des glyphes
/// disparaîtraient en silence — le pire des défauts, parce qu'une formule presque complète a
/// l'air d'une formule.
#[test]
fn test_les_vingt_fontes_de_katex_se_chargent() {
    let r = MathRenderer::new();
    assert_eq!(r.font_count(), 20, "fontes chargées : {}", r.font_count());
}

/// Chaque combinaison de famille et de style que la mise en page peut produire trouve une
/// fonte. C'est le test qui attrape un nom de fichier mal formé.
#[test]
fn test_toute_famille_demandee_trouve_sa_fonte() {
    let r = MathRenderer::new();
    let familles = [
        Family::Main,
        Family::Math,
        Family::Ams,
        Family::Caligraphic,
        Family::Fraktur,
        Family::SansSerif,
        Family::Script,
        Family::Typewriter,
        Family::Size1,
        Family::Size2,
        Family::Size3,
        Family::Size4,
    ];
    for f in familles {
        for style in [
            Style::ROMAN,
            Style::BOLD,
            Style::ITALIC,
            Style {
                bold: true,
                italic: true,
            },
        ] {
            let nom = nom_de_fonte(f, style);
            assert!(
                r.fonts.contains_key(nom.as_str()),
                "{f:?} en {style:?} demande « {nom} », qui n'est pas embarquée"
            );
        }
    }
}

/// **Une formule met de l'encre sur le pixmap.** Le piège serait une mise en page juste et un
/// dessin vide : tous les tests de géométrie passeraient, et l'écran resterait noir.
#[test]
fn test_une_formule_met_vraiment_de_l_encre() {
    let r = MathRenderer::new();
    let mut p = Pixmap::new(400, 200).expect("pixmap");

    assert_eq!(encre(&p), 0, "le pixmap part vide");
    let source = r"\int_0^\infty e^{-x^2}\,dx = \frac{\sqrt{\pi}}{2}";
    let ok = r.draw(
        &mut p.as_mut(),
        source,
        Mode::Display,
        plume(20.0, 120.0, 28.0),
        BLANC,
    );
    assert!(ok, "la formule est valide");
    assert!(encre(&p) > 200, "seulement {} pixels encrés", encre(&p));
}

/// **Une fraction met de l'encre des deux côtés de sa barre.** C'est le test qui attrape une
/// conversion de repère fautive : si l'on oubliait que `y` descend à l'écran, tout tomberait
/// du même côté.
#[test]
fn test_une_fraction_encre_au_dessus_et_en_dessous_de_sa_ligne() {
    let r = MathRenderer::new();
    let mut p = Pixmap::new(200, 200).expect("pixmap");
    let ligne = 100.0_f32;

    r.draw(
        &mut p.as_mut(),
        r"\frac{a}{b}",
        Mode::Display,
        plume(40.0, ligne, 40.0),
        BLANC,
    );

    let mut au_dessus = 0;
    let mut en_dessous = 0;
    for (i, px) in p.pixels().iter().enumerate() {
        if px.alpha() == 0 {
            continue;
        }
        let y = (i / p.width() as usize) as f32;
        if y < ligne - 2.0 {
            au_dessus += 1;
        } else if y > ligne + 2.0 {
            en_dessous += 1;
        }
    }
    assert!(au_dessus > 20, "le numérateur : {au_dessus} pixels");
    assert!(en_dessous > 20, "le dénominateur : {en_dessous} pixels");
}

/// **Une rature se trace.** Une ligne n'a pas d'aire : remplie, elle ne laisserait aucune
/// encre. Le contenu est un fantôme, pour que la rature soit seule à encrer.
#[test]
fn test_une_rature_se_trace() {
    let r = MathRenderer::new();
    let mut p = Pixmap::new(200, 200).expect("pixmap");
    let ok = r.draw(
        &mut p.as_mut(),
        r"\cancel{\phantom{x}}",
        Mode::Display,
        plume(40.0, 120.0, 60.0),
        BLANC,
    );
    assert!(ok, "valide");
    assert!(encre(&p) > 40, "la rature encre : {} pixels", encre(&p));
}

/// **L'avance d'un caractère est celle de la fonte qui le dessine** : l'intégrale double
/// (U+222C) y est large de 1,084 em (0,556 selon les métriques de KaTeX), un `a` italique de
/// 0,529 dans les deux.
#[test]
fn test_l_avance_vient_de_la_fonte() {
    const DOUBLE: char = '\u{222C}';
    let r = MathRenderer::new();
    let double = r
        .avance(DOUBLE, Family::Size2, Style::ROMAN)
        .expect("dans Size2");
    assert!((double - 1.084).abs() < 1e-9, "intégrale double : {double}");
    let a = r
        .avance('a', Family::Math, Style::ITALIC)
        .expect("a dans Math");
    assert!((a - 0.529).abs() < 1e-3, "a : {a}");
    assert_eq!(
        r.avance(DOUBLE, Family::Math, Style::ITALIC),
        None,
        "absent de Math"
    );
    let (large, _, _) = r.measure(r"\iint_D", Mode::Display, 100.0).expect("valide");
    assert!(
        large > 150.0,
        "la formule compte le signe réel : {large} px"
    );
}

/// Une formule fausse ne dessine rien et le dit, plutôt que de paniquer ou de dessiner du bruit.
#[test]
fn test_une_formule_fausse_ne_dessine_rien_et_le_dit() {
    let r = MathRenderer::new();
    let mut p = Pixmap::new(200, 100).expect("pixmap");

    let ok = r.draw(
        &mut p.as_mut(),
        r"\frac{",
        Mode::Inline,
        plume(10.0, 50.0, 20.0),
        BLANC,
    );
    assert!(!ok, "la source est fausse");
    assert_eq!(encre(&p), 0, "et rien n'a été dessiné");
}

/// **Le cache ne change pas le résultat.** Deux dessins de la même formule, l'un après l'autre,
/// donnent le même pixmap : la seconde passe lit le cache, et doit lire la même chose.
#[test]
fn test_le_cache_ne_change_pas_le_resultat() {
    let r = MathRenderer::new();
    let source = r"\sum_{i=1}^{n} \frac{1}{i^2}";

    let dessiner = |r: &MathRenderer| {
        let mut p = Pixmap::new(300, 200).expect("pixmap");
        r.draw(
            &mut p.as_mut(),
            source,
            Mode::Display,
            plume(20.0, 120.0, 24.0),
            BLANC,
        );
        p
    };

    let premiere = dessiner(&r);
    let seconde = dessiner(&r);
    assert_eq!(
        premiere.data(),
        seconde.data(),
        "le cache a changé le rendu"
    );
}

/// La mesure d'une formule est proportionnelle à la taille de police : doubler le corps double
/// la boîte. C'est ce qui garantit qu'une formule suit le zoom au lieu d'être figée par paliers.
#[test]
fn test_la_mesure_suit_la_taille_de_police() {
    let r = MathRenderer::new();
    let source = r"x^2 + y^2 = z^2";
    let (w1, h1, d1) = r.measure(source, Mode::Inline, 16.0).expect("mesurable");
    let (w2, h2, d2) = r.measure(source, Mode::Inline, 32.0).expect("mesurable");

    assert!((w2 - 2.0 * w1).abs() < 1e-3, "largeur : {w1} puis {w2}");
    assert!((h2 - 2.0 * h1).abs() < 1e-3, "hauteur : {h1} puis {h2}");
    assert!((d2 - 2.0 * d1).abs() < 1e-3, "profondeur : {d1} puis {d2}");
    assert!(w1 > 0.0 && h1 > 0.0);
}

/// Mesurer une formule fausse ne rend rien — l'appelant sait alors qu'il doit réserver la place
/// de la source, pas celle d'une formule.
#[test]
fn test_mesurer_une_formule_fausse_ne_rend_rien() {
    let r = MathRenderer::new();
    assert!(r.measure(r"\sqrt{", Mode::Inline, 16.0).is_none());
}

/// Une taille de police minuscule ne dessine rien et ne panique pas : c'est le cas du dézoom
/// extrême, où une formule devient plus petite qu'un pixel.
#[test]
fn test_une_taille_minuscule_ne_panique_pas() {
    let r = MathRenderer::new();
    let mut p = Pixmap::new(50, 50).expect("pixmap");
    for taille in [0.0_f32, 0.01, 0.3, -4.0] {
        r.draw(
            &mut p.as_mut(),
            r"\frac{a}{b}",
            Mode::Inline,
            plume(10.0, 25.0, taille),
            BLANC,
        );
    }
}

/// Dessiner hors du pixmap ne déborde pas : une formule à moitié sortie de l'écran est le cas
/// normal d'un canva qu'on déplace.
#[test]
fn test_dessiner_hors_du_pixmap_ne_deborde_pas() {
    let r = MathRenderer::new();
    let mut p = Pixmap::new(60, 60).expect("pixmap");
    for (x, y) in [(-200.0, 30.0), (300.0, 30.0), (30.0, -200.0), (30.0, 300.0)] {
        r.draw(
            &mut p.as_mut(),
            r"\sum_{i=1}^{n} i",
            Mode::Display,
            plume(x, y, 24.0),
            BLANC,
        );
    }
}
