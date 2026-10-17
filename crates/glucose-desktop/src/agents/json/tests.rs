use super::*;

#[test]
fn un_message_du_protocole_se_lit_et_se_reecrit_a_l_identique() {
    let ligne = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"glucose_lire","arguments":{"x":-12.5,"ok":true,"rien":null,"l":[1,2,"trois"]}}}"#;
    let v = Json::lire(ligne).expect("lu");
    assert_eq!(v.ecrire(), ligne);
    assert_eq!(v.get("id").and_then(Json::as_f64), Some(7.0));
    let args = v
        .get("params")
        .and_then(|p| p.get("arguments"))
        .expect("arguments");
    assert_eq!(args.get("x").and_then(Json::as_f64), Some(-12.5));
}

#[test]
fn les_echappements_se_decodent_et_un_saut_de_ligne_ne_coupe_jamais_le_message() {
    // Les échappements se construisent ici : un outil d'écriture qui les décoderait en
    // caractères rendrait l'épreuve aveugle au décodage (c'est arrivé). `bs` est la barre.
    let bs = char::from(92u8);
    let entree = format!(r#""à « é »{bs}n{bs}t{bs}"{bs}{bs}{bs}/{bs}u00e9{bs}ud83d{bs}ude00""#);
    assert!(entree.contains("ud83d"), "l'entrée garde ses échappements");
    let v = Json::lire(&entree).expect("lu");
    let sourire = char::from_u32(0x1F600).expect("un caractère");
    let (saut, tab) = (char::from(10u8), char::from(9u8));
    let attendu = format!("à « é »{saut}{tab}\"{bs}/é{sourire}");
    assert_eq!(v.as_str(), Some(attendu.as_str()));
    let ecrit = Json::texte("ligne 1\nligne 2\u{1}").ecrire();
    assert!(!ecrit.contains('\n'), "un message par ligne : {ecrit}");
    assert_eq!(ecrit, r#""ligne 1\nligne 2\u0001""#);
    assert_eq!(
        Json::lire(&ecrit).expect("relu").as_str(),
        Some("ligne 1\nligne 2\u{1}")
    );
}

#[test]
fn un_entier_revient_sans_virgule_et_un_non_nombre_devient_null() {
    assert_eq!(Json::Nombre(42.0).ecrire(), "42");
    assert_eq!(Json::Nombre(0.25).ecrire(), "0.25");
    assert_eq!(Json::Nombre(f64::NAN).ecrire(), "null");
}

#[test]
fn json_1_une_entree_hostile_rend_une_erreur_sans_tomber() {
    for mauvais in [
        "",
        "{",
        "[1,",
        r#"{"a" 1}"#,
        r#""non fermée"#,
        "tru",
        "1 2",
        r#""\u12""#,
        r#""\ud83d""#,
        r#""\x""#,
        "{\"a\":\"\u{1}\"}",
        "-",
        "[1,]",
    ] {
        assert!(
            Json::lire(mauvais).is_err(),
            "« {mauvais} » devait être refusé"
        );
    }
    let profond = "[".repeat(100_000);
    assert!(Json::lire(&profond).is_err(), "l'imbrication est bornée");
    let juste = format!(
        "{}{}",
        "[".repeat(PROFONDEUR_MAX),
        "]".repeat(PROFONDEUR_MAX)
    );
    assert!(
        Json::lire(&juste).is_ok(),
        "la borne laisse passer ce qu'elle annonce"
    );
}
