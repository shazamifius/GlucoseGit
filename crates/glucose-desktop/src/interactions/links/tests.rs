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

// ── La chaîne complète : d'une position écran jusqu'à l'adresse ─────────────
//
// `url_at` savait déjà lire une source. Ce qu'aucun test ne disait, c'est si un **clic** y
// arrive : entre les deux il y a la caméra, l'arbitre de clic, et la traduction d'un point
// écran en offset d'octet dans une carte lue **au repos**, là où l'adresse est masquée. Trois
// occasions de se tromper, et l'essai à la main dit que le lien ne s'ouvre pas.

mod chaine {
    use crate::app::GlucoseApp;
    use crate::interactions::resize::tests::{render_frame, text_card};

    const LIEN: &str = "voir [ici](https://exemple.fr) maintenant";

    fn app_avec_lien() -> GlucoseApp {
        let mut app = GlucoseApp::new();
        if let Some(b) = app.store.active_board_mut() {
            b.annotations.clear();
            b.images.clear();
            b.viewport.x = 400.0;
            b.viewport.y = 300.0;
            b.viewport.scale = 1.0;
        }
        let board = app.store.project.active_board_id.clone();
        app.store
            .add_annotation(&board, text_card("l", 0.0, 0.0, 300.0, LIEN));
        render_frame(&mut app);
        app
    }

    /// Il existe un point de la carte rendue où le clic trouve l'adresse.
    ///
    /// Le test balaie la première ligne plutôt que de viser un pixel : viser demanderait de
    /// recalculer ici la mise en page, donc de refaire le travail qu'on veut justement
    /// vérifier. Ce qui compte est l'existence, pas la coordonnée exacte.
    #[test]
    fn test_a_click_somewhere_on_the_link_finds_its_address() {
        let app = app_avec_lien();
        let y = 300.0 + 12.0;
        let trouve = (400..700)
            .map(f64::from)
            .filter_map(|x| app.link_under((x, y)))
            .next();
        assert_eq!(
            trouve.as_deref(),
            Some("https://exemple.fr"),
            "aucun point de la ligne du lien ne rend son adresse"
        );
    }

    /// Et le texte autour, lui, n'en rend aucune : le lien a des bords.
    #[test]
    fn test_the_words_around_the_link_open_nothing() {
        let app = app_avec_lien();
        let y = 300.0 + 12.0;
        // Très à droite de la carte : hors du texte, donc hors du lien.
        assert_eq!(app.link_under((399.0, y)), None);
    }
}
