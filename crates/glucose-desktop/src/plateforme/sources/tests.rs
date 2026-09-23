//! Ce que la source promet : la meilleure qualité d'abord, et jamais une page d'erreur prise
//! pour une image.

use super::*;

#[test]
fn test_une_adresse_se_decoupe_et_les_schemas_etrangers_sont_refuses() {
    assert_eq!(
        decouper("https://i.pinimg.com/236x/ab/cd/ef/h.jpg?x=1#frag"),
        Some(Adresse {
            securise: true,
            hote: "i.pinimg.com".into(),
            port: 443,
            chemin: "/236x/ab/cd/ef/h.jpg?x=1".into(),
        })
    );
    assert_eq!(
        decouper("http://exemple.fr:8080").map(|a| (a.port, a.chemin)),
        Some((8080, "/".into()))
    );
    assert_eq!(
        decouper("https://exemple.fr?q=1").map(|a| a.chemin),
        Some("/?q=1".into())
    );
    for refuse in [
        "file:///C:/x.jpg",
        "javascript:alert(1)",
        "ftp://x/y",
        "https://moi@hote/x",
        "https://",
    ] {
        assert_eq!(decouper(refuse), None, "{refuse}");
    }
}

/// **L'original de Pinterest passe avant la copie reçue** — c'est la « qualité optimale » que
/// l'utilisateur demande, et Pinterest la garde sous le même chemin.
#[test]
fn test_une_image_de_pinterest_se_decline_de_l_original_a_la_copie() {
    let v = variantes_pinterest("https://i.pinimg.com/236x/ab/cd/ef/hash.jpg").expect("pinterest");
    assert_eq!(v[0], "https://i.pinimg.com/originals/ab/cd/ef/hash.jpg");
    assert!(v.contains(&"https://i.pinimg.com/originals/ab/cd/ef/hash.png".to_string()));
    assert_eq!(
        v[v.len() - 2],
        "https://i.pinimg.com/736x/ab/cd/ef/hash.jpg"
    );
    assert_eq!(
        v[v.len() - 1],
        "https://i.pinimg.com/236x/ab/cd/ef/hash.jpg"
    );
    assert_eq!(variantes_pinterest("https://exemple.fr/236x/a.jpg"), None);
}

#[test]
fn test_les_candidats_vont_de_la_meilleure_image_a_la_page() {
    let c = candidats(&[
        "https://fr.pinterest.com/pin/288441551156395185/".to_string(),
        "https://exemple.fr/photo.PNG?taille=grande".to_string(),
        "https://i.pinimg.com/236x/ab/cd/ef/h.jpg".to_string(),
        "javascript:rien".to_string(),
        "https://i.pinimg.com/236x/ab/cd/ef/h.jpg".to_string(),
    ]);
    assert_eq!(
        c.first(),
        Some(&Candidat::Image(
            "https://i.pinimg.com/originals/ab/cd/ef/h.jpg".into()
        ))
    );
    assert_eq!(
        c.last(),
        Some(&Candidat::Page(
            "https://fr.pinterest.com/pin/288441551156395185/".into()
        )),
        "la page vient en dernier recours"
    );
    assert!(c.contains(&Candidat::Image(
        "https://exemple.fr/photo.PNG?taille=grande".into()
    )));
    let uniques: std::collections::HashSet<_> = c.iter().collect();
    assert_eq!(uniques.len(), c.len(), "aucun candidat en double");
}

/// **La page dit laquelle est son image, et les autres ne comptent pas.**
///
/// Une page d'épingle porte des dizaines d'adresses de `i.pinimg.com` — icônes, avatars,
/// épingles voisines. La première version les prenait avant `og:image`, et a rapporté une
/// icône sur la première épingle réelle. Ici l'icône paraît AVANT la balise, comme dans une
/// vraie page, et ne doit pas être retenue.
#[test]
fn test_une_page_dit_son_image_et_les_autres_ne_comptent_pas() {
    let html = r#"<html><head>
        <link rel="icon" href="https://i.pinimg.com/75x75_RS/ic/on/e/icone.png">
        <meta property="og:image:width" content="736">
        <meta property="og:image" content="https://i.pinimg.com/736x/ab/cd/ef/h.jpg">
        <meta name='twitter:image' content='https://exemple.fr/a.jpg?x=1&amp;y=2'>
        </head><script>{"voisine":"https://i.pinimg.com/236x/zz/zz/zz/autre.jpg"}</script>"#;
    assert_eq!(
        images_de_la_page(html),
        vec![
            "https://i.pinimg.com/736x/ab/cd/ef/h.jpg".to_string(),
            "https://exemple.fr/a.jpg?x=1&y=2".to_string(),
        ]
    );
    // Et c'est l'original de CETTE image qu'on essaiera d'abord.
    assert_eq!(
        candidats(&images_de_la_page(html)).first(),
        Some(&Candidat::Image(
            "https://i.pinimg.com/originals/ab/cd/ef/h.jpg".into()
        ))
    );
}

/// **Une page d'erreur n'est pas une image** : un serveur qui refuse répond souvent `200` avec
/// du HTML, et l'écrire en `.jpg` poserait une image cassée.
#[test]
fn test_seuls_des_octets_d_image_passent() {
    assert!(est_une_image(b"\x89PNG\r\n\x1a\n...."));
    assert!(est_une_image(&[0xFF, 0xD8, 0xFF, 0xE0]));
    assert!(est_une_image(b"RIFF\x00\x00\x00\x00WEBPVP8 "));
    assert!(est_une_image(b"\x00\x00\x00\x1cftypavif"));
    assert!(!est_une_image(b"<!DOCTYPE html><html>"));
    assert!(!est_une_image(b""));
}

#[test]
fn test_le_nom_vient_du_dernier_segment() {
    assert_eq!(
        nom_pour("https://i.pinimg.com/originals/ab/cd/h.jpg?x=1"),
        "h.jpg"
    );
    assert_eq!(nom_pour("https://exemple.fr/"), "image");
}
