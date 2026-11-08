//! GRIS-2 — la loi du gris dans la typographie du canevas : sous la lettre lisible, aucune lettre
//! ne se rastérise, et l'encre peinte est celle des lettres — au passage du seuil, rien ne saute.

use super::super::{Face, TextStyle, Typography};
use super::LETTRE_LISIBLE;
use tiny_skia::{Color, Pixmap};

const TEXTE: &str = "Eau douce, eau salée : le cycle de l'eau";

/// Écrit `TEXTE` en blanc sur du noir, à ce corps ; rend l'encre peinte (la somme des niveaux,
/// sur 255) et le nombre de pixels touchés.
fn ecrire(typo: &Typography, corps: f32) -> (f32, usize) {
    let mut p = Pixmap::new(600, 60).expect("un tampon");
    p.fill(Color::BLACK);
    let style = TextStyle {
        size: corps,
        color: Color::WHITE,
        face: Face::Regular,
    };
    typo.draw_text(&mut p.as_mut(), TEXTE, 10.0, 20.0, style);
    let rouges = p.pixels().iter().map(|c| c.red()).filter(|&r| r > 0);
    let (somme, n) = rouges.fold((0u32, 0usize), |(s, n), r| (s + u32::from(r), n + 1));
    (somme as f32 / 255.0, n)
}

/// **Sous la lettre lisible, aucune lettre ne se rastérise, et l'encre peinte est celle des
/// lettres** ([`Typography::gris`]) — à moins d'un niveau par pixel touché, ce que garantit une
/// image de 8 bits.
#[test]
fn gris_2_sous_le_seuil_l_encre_peinte_est_celle_des_lettres() {
    let typo = Typography::new();
    let corps = LETTRE_LISIBLE * 0.9;
    let lettres = typo.cached_glyph_count();
    let (peinte, touches) = ecrire(&typo, corps);
    assert_eq!(
        typo.cached_glyph_count(),
        lettres,
        "sous le seuil, aucune lettre ne se rastérise"
    );
    let (_, attendue) = typo.gris(TEXTE.chars().map(|c| (c, Some(Face::Regular))), corps);
    eprintln!("MESURE gris-2 : peinte {peinte}, attendue {attendue}, {touches} pixels");
    assert!(attendue > 1.0, "l'épreuve doit avoir de l'encre à peindre");
    assert!(
        (peinte - attendue).abs() < touches as f32 / 255.0,
        "encre peinte {peinte}, celle des lettres {attendue}, sur {touches} pixels"
    );
}

/// **Au passage du seuil, l'encre ne saute pas** : le même texte, en lettres au corps lisible et en
/// gris juste au-dessous, porte la même encre, ramenée au carré du corps — à un niveau près par
/// pixel touché dans chacune des deux images. Sans quoi un nom changerait de teinte en dézoomant.
#[test]
fn gris_2_au_passage_du_seuil_l_encre_ne_saute_pas() {
    let typo = Typography::new();
    let (en_lettres, n1) = ecrire(&typo, LETTRE_LISIBLE);
    let juste_dessous = LETTRE_LISIBLE * (1.0 - f32::EPSILON * 8.0);
    let (en_gris, n2) = ecrire(&typo, juste_dessous);
    let en_gris = en_gris * (LETTRE_LISIBLE / juste_dessous).powi(2);
    eprintln!("MESURE gris-2 : en lettres {en_lettres} ({n1} px), en gris {en_gris} ({n2} px)");
    assert!(
        (en_lettres - en_gris).abs() < (n1 + n2) as f32 / 255.0,
        "en lettres {en_lettres}, en gris {en_gris}"
    );
}

/// **La carte graphique suit la même loi** : une lettre illisible y pose son gris — une forme, et
/// aucune courbe de lettre ; au corps lisible, ses lettres.
#[test]
fn gris_2_sur_la_carte_graphique_une_lettre_illisible_pose_son_gris() {
    use crate::present::formes_gpu::Genre;
    use crate::renderer::vectoriel::{Releve, Vectoriel};
    let typo = Typography::new();
    let poser = |corps: f32| {
        let mut r = Releve::nouveau((600.0, 60.0));
        let style = TextStyle {
            size: corps,
            color: Color::WHITE,
            face: Face::Regular,
        };
        r.texte(&typo, (10.0, 20.0), TEXTE, style);
        let lettres = r.primitives.iter();
        let courbes = lettres.filter(|p| matches!(p.genre, Genre::Courbes { .. }));
        (r.primitives.len(), courbes.count())
    };
    assert_eq!(
        poser(LETTRE_LISIBLE * 0.5),
        (1, 0),
        "une forme, aucune lettre"
    );
    assert!(
        poser(LETTRE_LISIBLE).1 > 10,
        "au corps lisible, ses lettres"
    );
}

/// **Un filet d'un dixième de pixel, vu de loin, garde un dixième de son encre** (fiche 97) : le
/// gris passe par le peintre exact des rectangles droits, et non par la loi des membranes, qui lui
/// en donnait en moyenne trois fois plus.
#[test]
fn gris_2_un_filet_d_un_dixieme_de_pixel_garde_sa_juste_encre() {
    use glucose_core::membrane_forme::Arrondi;
    for decalage in [0.0f32, 0.25, 0.45, 0.5, 0.7] {
        let mut p = Pixmap::new(60, 20).expect("un tampon");
        let filet = Arrondi::nouveau(10.0, 9.0 + decalage, 40.0, 0.1, 0.0);
        super::peindre_exact(&mut p.as_mut(), (filet, Color::WHITE));
        let encre: f32 = p.pixels().iter().map(|c| f32::from(c.alpha())).sum::<f32>() / 255.0;
        assert!(
            (encre - 4.0).abs() < 0.1,
            "décalé de {decalage} : {encre} au lieu de 4,0 (40 pixels × un dixième)"
        );
    }
}
