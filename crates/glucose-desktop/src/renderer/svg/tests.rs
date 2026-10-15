//! Le peintre vectoriel : ses briques (P5, fiche 69).

use super::*;

/// **Le base64 de la RFC 4648**, ses vecteurs d'épreuve (§ 10), remplissage compris.
#[test]
fn test_le_base64_de_la_rfc_4648() {
    for (clair, code) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(base64(clair.as_bytes()), code, "{clair:?}");
    }
    assert_eq!(base64(&[0xff, 0xfe, 0xfd]), "//79");
}

/// **Ce qu'on écrit dans du XML ne le casse pas** : les cinq signes, et les caractères de
/// contrôle, qui n'y ont pas de place, partent.
#[test]
fn test_un_texte_s_echappe() {
    assert_eq!(
        echapper(r#"<a href="x">&</a>"#),
        "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;"
    );
    assert_eq!(echapper("a\u{0007}b\tc"), "ab\tc");
    assert_eq!(echapper("été — « ok »"), "été — « ok »");
}

/// **Un nombre s'écrit sans décimales inutiles**, à trois chiffres après la virgule au plus.
#[test]
fn test_un_nombre_s_ecrit_court() {
    assert_eq!(n(12.0), "12");
    assert_eq!(n(12.5), "12.5");
    assert_eq!(n(1.0 / 3.0), "0.333");
    assert_eq!(n(-0.0004), "0");
}

/// **Un coin ne dépasse jamais la moitié du côté le plus court** (`rayon_tenu`, fiche 70) : le
/// canevas borne ainsi chaque coin ; un `rx` seul donnerait des coins d'ellipse à un rectangle
/// bas.
#[test]
fn test_un_coin_se_borne_au_plus_petit_demi_cote() {
    let mut svg = Svg::new();
    svg.rectangle((0.0, 0.0, 100.0, 10.0), 20.0, ("#ffffff", 1.0));
    let ecrit = svg.finir((100, 10));
    assert!(ecrit.contains(r#"rx="5""#), "{ecrit}");
    assert_eq!(rayon_tenu(3.0, (100.0, 10.0)), 3.0);
    assert_eq!(rayon_tenu(-1.0, (10.0, 10.0)), 0.0);
}

/// **Un glyphe a la taille que le canevas lui donne**, mesurée par un autre que lui : la boîte
/// que `fontdue` rastérise pour le canevas, contre le contour du SVG à son échelle. La
/// superposition ne voit pas un glyphe grossi de 5 % — à quatorze pixels, moins d'un pixel
/// (fiche 71) ; à deux cents, sept.
#[test]
fn test_un_glyphe_a_la_taille_du_canevas() {
    use crate::typography::{Face, Typography};
    let typo = Typography::new();
    let corps = 200.0;
    let (police, indice) = typo.source('H', Face::Regular);
    let mut svg = Svg::new();
    svg.glyphe((police, indice), (0.0, corps), corps);
    let ecrit = svg.finir((400, 400));
    let debut = ecrit.find("matrix(").expect("un glyphe posé") + "matrix(".len();
    let s: f32 = ecrit[debut..]
        .split(' ')
        .next()
        .and_then(|v| v.parse().ok())
        .expect("l'échelle du glyphe");
    let face = ttf_parser::Face::parse(police.octets, 0).expect("la police");
    let boite = face
        .glyph_bounding_box(ttf_parser::GlyphId(indice))
        .expect("le contour de H");
    let large = f32::from(boite.x_max - boite.x_min) * s;
    let rasterise = typo.font(Face::Regular).metrics('H', corps).width as f32;
    assert!(
        (large - rasterise).abs() <= 1.5,
        "le SVG : {large} px ; le canevas : {rasterise} px"
    );
}

/// **Un rectangle net tombe sur les mêmes pixels sur les deux surfaces** (`remplir_net`,
/// fiche 70) : le canevas le remplit aux colonnes entières de ses bords (`fill_crisp`), le SVG
/// doit écrire ces mêmes colonnes — jugé par `resvg`, au pixel près.
#[test]
fn test_un_rectangle_net_tombe_sur_les_memes_pixels() {
    use crate::renderer::surface::{Pixels, Surface, Vecteurs};
    use crate::typography::Typography;
    let typo = Typography::new();
    let rect = tiny_skia::Rect::from_xywh(10.6, 3.2, 5.5, 4.4).expect("un rectangle");
    let rouge = Color::from_rgba8(200, 40, 40, 255);
    let mut canevas = tiny_skia::Pixmap::new(32, 16).expect("une image");
    let mut pixels = canevas.as_mut();
    Pixels {
        pixmap: &mut pixels,
        typographie: &typo,
    }
    .remplir_net(rect, rouge);
    let mut svg = Svg::new();
    Vecteurs {
        svg: &mut svg,
        typographie: &typo,
    }
    .remplir_net(rect, rouge);
    let arbre = resvg::usvg::Tree::from_str(&svg.finir((32, 16)), &Default::default())
        .expect("un SVG lisible");
    let mut lu = tiny_skia::Pixmap::new(32, 16).expect("une image");
    resvg::render(&arbre, tiny_skia::Transform::identity(), &mut lu.as_mut());
    assert_eq!(canevas.data(), lu.data());
}
