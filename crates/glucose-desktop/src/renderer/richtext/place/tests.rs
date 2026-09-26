//! La place d'un cadre : exactement ce qui manque, et rien de plus (PASSAGE-2).

use super::*;
use crate::renderer::card::{card_text_layout, text_box};
use crate::renderer::math::MathRenderer;
use crate::renderer::richtext::hit::x_du_caractere;
use crate::renderer::richtext::{TextLayout, TextMode};
use crate::typography::Face;

/// L'étendue d'un cadre dans ces épreuves : celle du survol d'une flèche.
const E: f32 = 5.75;

fn poser(texte: &str, largeur: f32, plages: &[(usize, usize)]) -> (TextLayout, TextLayout) {
    let typo = Typography::new();
    let brute = card_text_layout(
        &typo,
        &MathRenderer::new(),
        texte,
        largeur,
        TextMode::Rendered,
    );
    let ouverte = ouvrir_la_place(&brute, &typo, (texte, text_box(largeur)), plages, E);
    (brute, ouverte)
}

/// L'abscisse où se dessine le caractère à l'octet `o`, sur la première ligne.
fn x(mise_en_page: &TextLayout, texte: &str, o: usize) -> f32 {
    let typo = Typography::new();
    x_du_caractere(&typo, mise_en_page, &mise_en_page.lines[0], texte, o, 14.0)
}

fn avance(texte: &str) -> f32 {
    Typography::new().measure_text(texte, 14.0, Face::Regular).0
}

/// **Au milieu d'un mot — sa capture du 26/09 —, les voisins s'écartent de l'étendue du cadre,
/// exactement** : l'avant du passage recule d'une étendue, l'après aussi, et le passage garde sa
/// largeur.
#[test]
fn test_passage_2_au_milieu_d_un_mot_la_place_est_l_etendue() {
    let texte = "testetsetetstetsetes";
    let (a, b) = (3, 16);
    let (brute, ouverte) = poser(texte, 400.0, &[(a, b)]);
    let avant = x(&brute, texte, a);
    assert!(
        (x(&ouverte, texte, a) - (avant + E)).abs() < 1e-3,
        "le passage commence a {} et non a {}",
        x(&ouverte, texte, a),
        avant + E
    );
    // Après le passage : sa largeur, puis l'étendue de son bord droit, contre lequel la
    // voisine vient s'appuyer.
    let largeur = avance(&texte[a..b]);
    let apres = x(&ouverte, texte, b);
    assert!(
        (apres - (avant + E + largeur + E)).abs() < 1e-3,
        "la voisine d'apres est a {apres}"
    );
    // Et la suite de la ligne garde ses écarts : rien ne s'accumule.
    assert!(
        (x(&ouverte, texte, texte.len()) - x(&brute, texte, texte.len()) - 2.0 * E).abs() < 1e-3,
        "la ligne s'ouvre de deux etendues, pas plus"
    );
}

/// **Entre deux mots, l'espace est une place libre** : le cadre la prend avant d'écarter quoi
/// que ce soit, et la ligne ne s'ouvre que de ce qui manque.
#[test]
fn test_passage_2_une_espace_est_une_place_libre() {
    let texte = "le chat mange";
    let (a, b) = (3, 7);
    let espace = avance(" ");
    let (brute, ouverte) = poser(texte, 400.0, &[(a, b)]);
    let manque = (E - espace).max(0.0);
    assert!(
        (x(&ouverte, texte, a) - (x(&brute, texte, a) + manque)).abs() < 1e-3,
        "le mot recule de {} au lieu de {manque}",
        x(&ouverte, texte, a) - x(&brute, texte, a)
    );
    assert!(
        (x(&ouverte, texte, b + 1) - (x(&brute, texte, b + 1) + 2.0 * manque)).abs() < 1e-3,
        "apres le mot, la ligne s'ouvre de ce qui manque des deux cotes"
    );
}

/// **Un passage au début de sa ligne ne l'ouvre pas à gauche** : la marge de la carte est là
/// pour son cadre.
#[test]
fn test_passage_2_un_debut_de_ligne_est_une_place_libre() {
    let texte = "bonjour tout le monde";
    let (_, ouverte) = poser(texte, 400.0, &[(0, 7)]);
    assert_eq!(x(&ouverte, texte, 0), 0.0);
}

/// **Deux passages qui se touchent écartent chacun leur part** : deux cadres, deux étendues.
#[test]
fn test_passage_2_deux_cadres_voisins_ont_chacun_leur_place() {
    let texte = "aaaabbbbcccc";
    let (brute, ouverte) = poser(texte, 400.0, &[(0, 4), (4, 8)]);
    assert!(
        (x(&ouverte, texte, 4) - (x(&brute, texte, 4) + 2.0 * E)).abs() < 1e-3,
        "entre deux cadres, deux etendues"
    );
}

/// **La coupe des lignes ne change jamais** : survoler une flèche ne fait pas gagner une ligne
/// à une carte — ni changer sa hauteur, sa boîte, ses flèches.
#[test]
fn test_passage_2_la_coupe_des_lignes_ne_change_pas() {
    let texte = "Une phrase assez longue pour se couper en plusieurs lignes, avec un passage \
                 au milieu, collé à ses voisins, et encore une fin qui déborde un peu.";
    let debut = texte.find("passage").expect("le passage");
    let (brute, ouverte) = poser(texte, 240.0, &[(debut, debut + 7)]);
    let coupe = |m: &TextLayout| -> Vec<(usize, usize)> {
        m.lines.iter().map(|l| (l.start, l.end)).collect()
    };
    assert!(brute.line_count() > 2, "le texte se coupe");
    assert_eq!(coupe(&brute), coupe(&ouverte));
}

/// **Un fragment est dedans ou dehors, jamais à moitié** : la teinte suit les lettres. C'est ce
/// qui manquait — une lettre voisine sortait à moitié rose d'un rectangle découpé.
#[test]
fn test_passage_2_la_teinte_suit_les_lettres() {
    let texte = "tes**tetse**tetstetsetes et la suite";
    let (a, b) = (5, 20);
    let (_, ouverte) = poser(texte, 400.0, &[(a, b)]);
    for f in &ouverte.fragments {
        let dedans = a <= f.start && f.end <= b;
        let dehors = f.end <= a || b <= f.start;
        assert!(dedans || dehors, "fragment a cheval : {:?}", f.start..f.end);
        assert_eq!(f.eclaire, dedans, "fragment {:?}", f.start..f.end);
    }
    assert!(ouverte.fragments.iter().any(|f| f.eclaire));
}

/// **Rien d'éclairé, rien d'ouvert** : sans plage, la mise en page est celle de la carte.
#[test]
fn test_passage_2_sans_plage_rien_ne_bouge() {
    let texte = "le chat mange\nune souris";
    let (brute, ouverte) = poser(texte, 240.0, &[]);
    assert_eq!(brute, ouverte);
}
