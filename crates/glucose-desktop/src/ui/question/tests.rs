//! Ce que garantit la question dessinée (QUESTION-1).

use super::*;

fn question(texte: &str) -> Question {
    Question {
        titre: "Nouveau document".into(),
        texte: texte.into(),
        choix: vec![
            ("Créer".into(), Reponse::Oui),
            ("Annuler".into(), Reponse::Non),
        ],
        ..Default::default()
    }
}

/// Son téléphone : 720 × 1600 pixels à 200 %, soit 360 × 800 points.
const TELEPHONE: (f32, f32) = (720.0, 1600.0);

/// **Tout tient dans l'écran**, la carte comme chaque ligne : sur un téléphone, une ligne qui
/// déborde est une phrase qu'on ne lit pas.
#[test]
fn test_la_question_tient_dans_l_ecran_d_un_telephone() {
    let typo = Typography::new();
    let long = crate::telemetrie::QUESTION;
    let p = placer(&question(long), &typo, TELEPHONE, 2.0);
    let (x, y, w, h) = p.carte;
    assert!(
        x >= 32.0 && x + w <= TELEPHONE.0 - 32.0,
        "la carte garde sa marge : {:?}",
        p.carte
    );
    assert!(
        y >= 0.0 && y + h <= TELEPHONE.1,
        "la carte tient en hauteur : {:?}",
        p.carte
    );
    // Chaque ligne tient dans la marge intérieure — la même de chaque côté.
    let droite = x + w - (p.gauche - x);
    let mesurer = |lignes: &[(String, f32)], corps: f32, face: Face| {
        for (ligne, _) in lignes {
            let (lw, _) = typo.measure_text(ligne, corps, face);
            assert!(
                p.gauche + lw <= droite + 0.5,
                "la ligne « {ligne} » déborde de la carte"
            );
        }
    };
    mesurer(&p.titre, p.corps.0, Face::Bold);
    mesurer(&p.texte, p.corps.1, Face::Regular);
    assert!(p.texte.len() > 5, "un long texte se coupe en lignes");
}

/// **Les blancs du texte source ne s'affichent pas** : une ligne vide sépare deux
/// paragraphes, une seule fois ; une suite de blancs se lit comme un espace.
#[test]
fn test_les_blancs_du_texte_se_lisent_comme_un_espace() {
    let typo = Typography::new();
    let p = placer(
        &question("Un    deux\n\n\n   trois  "),
        &typo,
        (1440.0, 900.0),
        1.0,
    );
    let lignes: Vec<&str> = p.texte.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(lignes, vec!["Un deux", "", "trois"]);
}

/// **Chaque réponse fait la cible d'un doigt**, et répond en son centre — la première est
/// celle qu'on lit d'abord.
#[test]
fn test_chaque_reponse_fait_quarante_huit_points_et_repond_en_son_centre() {
    let typo = Typography::new();
    let p = placer(
        &question("Créer un nouveau document ?"),
        &typo,
        TELEPHONE,
        2.0,
    );
    assert_eq!(p.boutons.len(), 2);
    for ((x, y, w, h), _, reponse) in &p.boutons {
        assert_eq!(*h, 96.0, "48 points à 200 %");
        assert_eq!(reponse_sous(&p, x + w / 2.0, y + h / 2.0), Some(*reponse));
    }
    assert_eq!(p.boutons[0].2, Reponse::Oui);
    let (x, y, _, _) = p.carte;
    assert_eq!(
        reponse_sous(&p, x + 1.0, y + 1.0),
        None,
        "le texte ne répond rien"
    );
}

/// **La part visible du champ** montre toujours la tête, sur des caractères entiers.
#[test]
fn test_documents_2_le_champ_montre_la_tete() {
    let largeur = |a: usize, b: usize| (b - a) as f32;
    assert_eq!(
        fenetre("abcdefghij", 10, 4.0, largeur),
        (6, 10),
        "la fin, tête au bout"
    );
    assert_eq!(
        fenetre("abcdefghij", 2, 4.0, largeur),
        (0, 4),
        "le début, tête au début"
    );
    assert_eq!(fenetre("abc", 1, 40.0, largeur), (0, 3), "tout tient");
    assert_eq!(
        fenetre("ééé", 6, 2.0, largeur),
        (4, 6),
        "jamais un octet coupé"
    );
}

/// **Les réponses restent à l'écran** (POPUP-1) : des notes de mise à jour de deux cents lignes
/// se coupent, et le disent ; la dernière réponse tient dans la fenêtre.
#[test]
fn test_popup_1_les_reponses_restent_a_l_ecran() {
    let typo = Typography::new();
    let long = "Une ligne des notes de la version.\n".repeat(200);
    let ecran = (1280.0, 720.0);
    let p = placer(&question(&long), &typo, ecran, 1.0);
    let ((_, y, _, h), _, _) = p.boutons.last().expect("une réponse");
    assert!(
        y + h <= ecran.1,
        "la dernière réponse sort de l'écran : {}",
        y + h
    );
    assert_eq!(
        p.texte.last().map(|(l, _)| l.as_str()),
        Some("…"),
        "le texte coupé le dit"
    );
    let court = placer(&question("Deux lignes."), &typo, ecran, 1.0);
    assert_eq!(court.texte.len(), 1, "un texte qui tient ne se coupe pas");
}

/// **La réponse en évidence se voit** : le filet blanc cerne celle qu'Entrée donnerait, et
/// suit quand elle change.
#[test]
fn test_popup_1_la_reponse_en_evidence_se_voit() {
    let typo = Typography::new();
    let theme = Theme::default();
    let rendre = |focus: usize| {
        let mut q = question("Un texte.");
        q.focus = focus;
        let p = placer(&q, &typo, (800.0, 600.0), 1.0);
        let mut pixmap = tiny_skia::Pixmap::new(800, 600).expect("une image");
        dessiner(&mut pixmap.as_mut(), &p, (&typo, &theme), (-1.0, -1.0), 1.0);
        let ((x, y, w, _), _, _) = p.boutons[0];
        let au_bord = |px: f32, py: f32| pixmap.pixel(px as u32, py as u32).expect("dedans");
        (
            au_bord(x + w / 2.0, y + 3.0),
            au_bord(x + w / 2.0, y + 10.0),
        )
    };
    let (filet, dedans) = rendre(0);
    let (sans, _) = rendre(1);
    assert_ne!(filet, dedans, "le filet cerne la première réponse");
    assert_eq!(
        sans, dedans,
        "il la quitte quand l'évidence passe à la suivante"
    );
}
