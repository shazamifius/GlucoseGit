//! Ce que la moisson promet : un nom venu d'une page web ne peut désigner qu'un fichier, et
//! deux fichiers du même nom ne s'effacent pas.

use super::*;

/// **Un nom qui remonte ne remonte pas.**
///
/// `cFileName` est choisi par la page, pas par l'utilisateur. Ne garder que ce qui suit le
/// dernier séparateur rend `..\..\x` inoffensif **sans avoir à reconnaître `..`** — une
/// vérification par liste de motifs finit toujours par oublier une écriture.
#[test]
fn test_un_nom_venu_d_une_page_ne_peut_pas_designer_un_dossier() {
    for propose in [
        r"..\..\demarrage\x.exe",
        "../../etc/passwd",
        r"C:\Windows\System32\x.dll",
        r"dossier\sous\image.png",
        "C:image.png",
    ] {
        let nom = nom_sur(propose);
        assert!(
            !nom.contains(['/', '\\', ':']),
            "{propose} a laisse passer un separateur : {nom}"
        );
        assert_eq!(
            Path::new(&nom).components().count(),
            1,
            "{propose} donne plus d'une composante : {nom}"
        );
    }
    assert_eq!(nom_sur(r"..\..\demarrage\x.exe"), "x.exe");
    assert_eq!(nom_sur(r"dossier\sous\image.png"), "image.png");
}

/// **Un nom qui n'est fait que de points retombe sur un nom neutre.**
///
/// C'est le seul cas que le filtre du séparateur laisse passer : la dernière composante de
/// `..` est `..` lui-même, et un chemin qui s'appelle `..` remonte d'un dossier.
#[test]
fn test_les_noms_qui_ne_nomment_rien_retombent_sur_un_nom_neutre() {
    for propose in ["..", ".", "...", "", "   ", r"chemin\.."] {
        assert_eq!(nom_sur(propose), SANS_NOM, "pour « {propose} »");
    }
}

/// **Les caractères qu'un nom de fichier ne peut pas porter sont remplacés**, y compris ceux
/// qui ne se voient pas.
///
/// Un caractère de contrôle est invisible à l'écran et parfaitement lisible pour un
/// programme : c'est exactement la forme d'un nom qui ment.
#[test]
fn test_ce_qu_un_nom_ne_peut_pas_porter_est_remplace() {
    assert_eq!(nom_sur("ima<ge>.png"), "ima_ge_.png");
    assert_eq!(nom_sur("a\u{0}b.png"), "a_b.png");
    assert_eq!(nom_sur("photo\u{1b}[31m.jpg"), "photo_[31m.jpg");
    // Ce qui est licite ne bouge pas : un accent, un espace, un tiret.
    assert_eq!(nom_sur("Ma photo-été (1).jpeg"), "Ma photo-été (1).jpeg");
}

/// **Deux fichiers du même nom, dans le même lot, ne s'effacent pas.**
///
/// Une page qui porte deux `image.png` est banale ; la seconde qui écraserait la première
/// donnerait deux fois la même image sur le canevas, ce qui se voit et ne s'explique pas.
#[test]
fn test_deux_fichiers_du_meme_nom_ne_s_effacent_pas() {
    let d = Path::new("racine");
    let a = chemin_pour(d, "image.png", 0);
    let b = chemin_pour(d, "image.png", 1);
    assert_ne!(a, b);
    // Et le nom d'origine reste lisible à la fin : c'est lui que l'utilisateur reconnaît.
    assert!(a.to_string_lossy().ends_with("image.png"));
    assert!(b.to_string_lossy().ends_with("image.png"));
}

/// **Un dépôt vide ne pose pas de fichier.**
///
/// Une page qui promet un contenu et n'en donne pas produirait un lanceur vide — ce qui
/// ressemble à un bug plutôt qu'à un refus.
#[test]
fn test_un_depot_vide_ne_pose_rien() {
    let d = std::env::temp_dir();
    assert_eq!(poser(&d, "vide.png", 0, &[]), None);
}

/// **Ce qui s'écrit se relit**, octet pour octet — c'est tout ce que le reste de
/// l'application attend d'un dépôt.
#[test]
fn test_ce_qui_s_ecrit_se_relit_octet_pour_octet() {
    let d = dossier().expect("le repertoire temporaire du systeme doit etre accessible");
    let octets = b"\x89PNG\r\n\x1a\n-- pas une vraie image, et c'est sans importance";
    let chemin = poser(&d, "essai.png", 0, octets).expect("l'ecriture doit reussir");
    assert_eq!(std::fs::read(&chemin).ok().as_deref(), Some(&octets[..]));
    let _ = std::fs::remove_file(&chemin);
}

/// **Une adresse déposée est cliquable**, et une parenthèse ne casse pas sa syntaxe.
///
/// `…/wiki/Paris_(homonymie)` n'a rien d'exotique, et ses parenthèses cassent le lien
/// Markdown au milieu de l'adresse. **Ce test a démenti ma première version**, qui n'encodait
/// que la fermante : l'ouvrante suffit à ce que la tranche cesse d'être un lien, et
/// `Ctrl`+clic ne suivait plus rien.
///
/// C'est pour cela que la preuve va jusqu'à `url_at`, celui que le clic emploie, au lieu de
/// s'arrêter à la forme de la chaîne : une chaîne bien formée en apparence ne prouve pas
/// qu'un lien s'ouvre.
#[test]
fn test_une_adresse_deposee_est_cliquable_parentheses_comprises() {
    for adresse in [
        "https://exemple.org/photo.png",
        "https://fr.wikipedia.org/wiki/Paris_(homonymie)",
        "http://exemple.org/a(b)c(d)",
    ] {
        let source = lien_markdown(adresse);
        // Le curseur posé sur le premier caractère du texte montré, soit juste après le `[`.
        let suivi = crate::interactions::links::url_at(&source, 1);
        assert_eq!(
            suivi.as_deref(),
            Some(adresse.replace('(', "%28").replace(')', "%29").as_str()),
            "pour {adresse}, la source etant {source}"
        );
        // Et ce qui est montré garde ses parenthèses : c'est ce que l'utilisateur relit.
        assert!(source.starts_with(&format!("[{adresse}]")));
    }
}

/// **Une moisson sans chemin est vide**, quoi qu'elle porte par ailleurs : une position sans
/// fichier n'est pas un dépôt.
#[test]
fn test_une_position_sans_fichier_n_est_pas_un_depot() {
    let m = Moisson {
        chemins: Vec::new(),
        liens: Vec::new(),
        ou: Some((100.0, 200.0)),
    };
    assert!(m.est_vide());
}
