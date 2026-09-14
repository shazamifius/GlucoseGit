//! Ce qu'un `Ctrl`+clic accepte d'ouvrir — et surtout ce qu'il refuse.

use super::url_at;

#[test]
fn test_ladresse_se_retrouve_depuis_nimporte_quel_octet_du_texte() {
    let source = "va voir [Glucose](https://exemple.fr) demain";
    let debut = source.find("Glucose").expect("le texte du lien");
    for at in debut..debut + "Glucose".len() {
        assert_eq!(
            url_at(source, at).as_deref(),
            Some("https://exemple.fr"),
            "octet {at}"
        );
    }
}

#[test]
fn test_hors_du_texte_du_lien_il_ny_a_rien_a_ouvrir() {
    let source = "va voir [Glucose](https://exemple.fr) demain";
    assert_eq!(url_at(source, 0), None, "avant");
    assert_eq!(url_at(source, source.len() - 2), None, "après");
    assert_eq!(url_at("rien qu'un texte", 3), None);
}

/// Un `.glucose` se reçoit de quelqu'un. Un `Ctrl`+clic ne doit pas pouvoir faire exécuter
/// quoi que ce soit : seuls `http` et `https` partent, tout le reste est refusé.
#[test]
fn test_seuls_http_et_https_sont_ouverts() {
    for adresse in [
        "file:///C:/Windows/System32/calc.exe",
        "javascript:alert(1)",
        "vbscript:x",
        "cmd:/c dir",
        "exemple.fr",
        "ftp://exemple.fr",
        "HTTPS://exemple.fr",
    ] {
        let source = format!("[clique]({adresse})");
        let at = source.find("clique").expect("le texte");
        assert_eq!(
            url_at(&source, at),
            None,
            "{adresse:?} ne doit jamais être confié au système"
        );
    }

    for adresse in ["http://exemple.fr", "https://exemple.fr/a?b=c#d"] {
        let source = format!("[clique]({adresse})");
        let at = source.find("clique").expect("le texte");
        assert_eq!(url_at(&source, at).as_deref(), Some(adresse));
    }
}

/// Deux liens sur la même ligne : chacun rend le sien.
#[test]
fn test_chaque_lien_rend_sa_propre_adresse() {
    let source = "[a](https://un.fr) et [b](https://deux.fr)";
    let a = source.find('a').expect("a");
    let b = source.find("[b](").expect("b") + 1;
    assert_eq!(url_at(source, a).as_deref(), Some("https://un.fr"));
    assert_eq!(url_at(source, b).as_deref(), Some("https://deux.fr"));
}
