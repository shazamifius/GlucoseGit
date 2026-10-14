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
