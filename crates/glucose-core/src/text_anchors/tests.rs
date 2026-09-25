//! Les ancres sur du texte **réel** : des accents, des positions venues d'ailleurs, et le
//! défaut que son utilisateur a relevé dans Tauri.

use super::*;

/// Sa carte, telle qu'il l'a décrite : deux fois « bonjours », entourés de « test ».
const SA_CARTE: &str = "bonjours\ntest\ntest\nbonjours";

/// **Le défaut de Tauri** : sélectionner le second « bonjours » ne désigne que lui — et après
/// une édition devant, c'est encore lui qu'on retrouve, par son contexte.
#[test]
fn test_fleche_4_le_second_bonjours_reste_le_second() {
    let second = SA_CARTE.rfind("bonjours").expect("le second");
    let ancre = create_anchor(SA_CARTE, second, second + 8).expect("une ancre");
    assert_eq!(
        resolve_anchors(SA_CARTE, std::slice::from_ref(&ancre)),
        vec![ResolvedRange {
            start: second,
            end: second + 8
        }]
    );
    let edite = format!("Élève, écoute :\n{SA_CARTE}");
    let decale = second + "Élève, écoute :\n".len();
    assert_eq!(
        resolve_anchors(&edite, &[ancre]),
        vec![ResolvedRange {
            start: decale,
            end: decale + 8
        }]
    );
}

/// **Des accents partout, et rien ne se découpe au milieu d'une lettre** : la création, le
/// contexte de trente-deux caractères, la résolution.
#[test]
fn test_fleche_4_les_accents_ne_cassent_rien() {
    let texte = "Été, hiver, été : l'élève répète « été » à l'école.";
    let i = texte.find("« été »").expect("la citation") + "« ".len();
    let ancre = create_anchor(texte, i, i + "été".len()).expect("une ancre");
    assert_eq!(ancre.quote, "été");
    assert_eq!(ancre.prefix.as_deref().map(|p| p.chars().count()), Some(32));
    assert!(ancre.prefix.as_deref().is_some_and(|p| p.ends_with("« ")));
    // Le texte change devant : la position ne tient plus, le contexte la retrouve.
    let edite = format!("Ça commence ainsi. {texte}");
    let r = resolve_anchors(&edite, &[ancre]);
    assert_eq!(r.len(), 1);
    assert_eq!(&edite[r[0].start..r[0].end], "été");
    assert!(edite[..r[0].start].ends_with("« "), "la bonne occurrence");
}

/// **Une position qui n'est pas une frontière de caractère** — celles de Tauri comptent des
/// unités UTF-16 d'un texte rendu — ne plante pas : elle est fausse, et l'ancre se retrouve par
/// sa citation.
#[test]
fn test_fleche_4_une_position_venue_de_tauri_ne_plante_pas() {
    let texte = "éé bonjours éé";
    let ancre = TextAnchor {
        start: 1,
        end: 9,
        quote: "bonjours".into(),
        prefix: Some("éé ".into()),
        suffix: Some(" éé".into()),
    };
    let r = resolve_anchors(texte, &[ancre]);
    assert_eq!(&texte[r[0].start..r[0].end], "bonjours");
    // Et une création aux positions tordues se ramène aux frontières.
    let a = create_anchor(texte, 1, 3).expect("une ancre");
    assert_eq!(a.quote, "éé");
}

/// **Sans tenir compte de la casse, les positions restent celles du texte** : « ÉTÉ » trouve
/// « été », et la plage désigne bien ces trois lettres.
#[test]
fn test_fleche_4_la_casse_ne_deplace_rien() {
    let texte = "le mot été, ici";
    let ancre = TextAnchor {
        start: -1,
        end: -1,
        quote: "ÉTÉ".into(),
        prefix: None,
        suffix: None,
    };
    let r = resolve_anchors(texte, &[ancre]);
    assert_eq!(&texte[r[0].start..r[0].end], "été");
}

/// **Deux occurrences qui se chevauchent sont deux occurrences** : l'ancre peut viser la
/// seconde.
#[test]
fn test_fleche_4_les_occurrences_qui_se_chevauchent() {
    let texte = "aaaa";
    let ancre = TextAnchor {
        start: 7,
        end: 10,
        quote: "aaa".into(),
        prefix: Some("a".into()),
        suffix: Some(String::new()),
    };
    assert_eq!(
        resolve_anchors(texte, &[ancre]),
        vec![ResolvedRange { start: 1, end: 4 }]
    );
}
