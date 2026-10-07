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

/// **Un dépôt vide ne livre rien** : une page qui promet un contenu et n'en donne pas
/// produirait un lanceur vide — ce qui ressemble à un bug plutôt qu'à un refus. Et le nom
/// livré est toujours un nom sûr.
#[test]
fn test_un_depot_vide_ne_livre_rien() {
    assert_eq!(Recu::nouveau("vide.png", Vec::new()), None);
    let r = Recu::nouveau(r"..\..\x.png", vec![1]).expect("un reçu");
    assert_eq!(r.nom, "x.png");
}

/// **Un raccourci livré par une page rend son adresse**, et le reste du lot demeure : c'est ce
/// que Chrome promet quand on glisse une épingle depuis la grille de Pinterest.
#[test]
fn test_un_raccourci_livre_rend_son_adresse_et_le_reste_demeure() {
    let raccourci = Recu::nouveau("epingle.url", RACCOURCI_DU_TERRAIN.as_bytes().to_vec());
    let image = Recu::nouveau("photo.png", vec![1, 2, 3]).expect("un reçu");
    let (gardes, adresses) =
        separer_les_raccourcis(vec![raccourci.expect("un reçu"), image.clone()]);
    assert_eq!(gardes, vec![image]);
    assert_eq!(
        adresses,
        vec!["https://fr.pinterest.com/pin/288441551156395185/".to_string()]
    );
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
        recus: Vec::new(),
        liens: Vec::new(),
        ou: Some((100.0, 200.0)),
        lot: None,
        echec: Some("fr.pinterest.com : délai dépassé".into()),
        apercu: false,
    };
    assert!(m.est_vide());
    let avec_un_lot = Moisson {
        lot: Some(vec![1, 2, 3]),
        ..m
    };
    assert!(!avec_un_lot.est_vide(), "un lot de nœuds est un dépôt");
}

/// Les octets **exacts** du raccourci que Chrome a promis quand l'utilisateur a glissé une
/// épingle depuis la grille de Pinterest, le 23/09 — relus sur son disque, fins de ligne
/// Windows comprises.
const RACCOURCI_DU_TERRAIN: &str =
    "[InternetShortcut]\r\nURL=https://fr.pinterest.com/pin/288441551156395185/\r\n";

#[test]
fn test_un_raccourci_du_terrain_rend_son_adresse() {
    assert_eq!(
        adresse_du_raccourci(RACCOURCI_DU_TERRAIN).as_deref(),
        Some("https://fr.pinterest.com/pin/288441551156395185/")
    );
}

/// Seule la clé `URL` de la section `[InternetShortcut]` désigne ce qu'on a déposé : une autre
/// section peut porter sa propre clé `URL`, et un raccourci sans adresse n'en invente pas une.
#[test]
fn test_seule_l_adresse_du_raccourci_compte() {
    let brouille = "[DEFAULT]\r\nBASEURL=https://ailleurs.example/\r\n[Autre]\r\nURL=https://piege.example/\r\n\
                    [InternetShortcut]\r\nIconIndex=0\r\nurl = https://vrai.example/x \r\n";
    assert_eq!(
        adresse_du_raccourci(brouille).as_deref(),
        Some("https://vrai.example/x")
    );
    assert_eq!(adresse_du_raccourci("[InternetShortcut]\r\nURL=\r\n"), None);
    assert_eq!(adresse_du_raccourci("du texte quelconque"), None);
}

#[test]
fn test_un_raccourci_se_reconnait_a_son_extension_quelle_que_soit_sa_casse() {
    assert!(est_un_raccourci(Path::new("epingle.url")));
    assert!(est_un_raccourci(Path::new(r"C:\bureau\Lien.URL")));
    assert!(!est_un_raccourci(Path::new("image.jpg")));
    assert!(!est_un_raccourci(Path::new("url")));
}

/// **DEPOT-WEB-3** — une adresse se retrouve, qu'elle soit écrite en UTF-8 ou en chaîne large.
///
/// Chromium range les données qu'une page pose dans son glisser en chaînes larges ; un
/// fragment HTML, lui, est en UTF-8. L'instrument doit voir les deux, sinon il dirait
/// « aucune adresse » sur exactement le format qu'on veut lire.
#[test]
fn test_une_adresse_se_lit_en_utf8_comme_en_chaine_large() {
    let html = br#"<a href="https://fr.pinterest.com/pin/1/"><img src="https://i.pinimg.com/236x/ab/cd.jpg" alt=x></a>"#;
    assert_eq!(
        adresses_dans(html),
        vec![
            "https://fr.pinterest.com/pin/1/".to_string(),
            "https://i.pinimg.com/236x/ab/cd.jpg".to_string(),
        ]
    );

    // La meme adresse en chaine large, precedee d'un octet : l'alignement impair aussi.
    let mut large = vec![0x07u8];
    for c in "type\0https://i.pinimg.com/originals/ef.png\0".encode_utf16() {
        large.extend_from_slice(&c.to_le_bytes());
    }
    assert_eq!(
        adresses_dans(&large),
        vec!["https://i.pinimg.com/originals/ef.png".to_string()]
    );
    assert!(adresses_dans(b"rien d'utile, http seul, httpx://non").is_empty());
}

/// La ponctuation qui suit une adresse dans un texte ne lui appartient pas — sauf la
/// parenthèse qu'elle ouvre elle-même.
#[test]
fn test_la_ponctuation_du_texte_n_appartient_pas_a_l_adresse() {
    assert_eq!(
        adresses_dans(b"fond: url(https://i.pinimg.com/originals/d5/h.png); voir https://a.fr/x."),
        vec![
            "https://i.pinimg.com/originals/d5/h.png".to_string(),
            "https://a.fr/x".to_string(),
        ]
    );
    assert_eq!(
        adresses_dans(b"https://fr.wikipedia.org/wiki/Paris_(homonymie)"),
        vec!["https://fr.wikipedia.org/wiki/Paris_(homonymie)".to_string()]
    );
}

/// **Un nœud qui n'est qu'un lien rend son adresse, et rien d'autre ne la rend**
/// (DEPOT-WEB-6) : la pose et sa lecture sont l'inverse l'une de l'autre, parenthèses comprises,
/// et un texte de l'utilisateur n'est jamais pris pour un repli de dépôt.
#[test]
fn test_un_lien_pose_se_relit_et_une_note_ne_se_relit_pas() {
    for adresse in [
        "https://fr.pinterest.com/pin/104427285106027754/",
        "https://fr.wikipedia.org/wiki/Paris_(homonymie)",
        "http://exemple.org/a(b)c(d)",
    ] {
        assert_eq!(
            adresse_du_lien(&lien_markdown(adresse)).as_deref(),
            Some(adresse),
            "aller-retour de {adresse}"
        );
    }
    for note in [
        "https://fr.pinterest.com/pin/1/",
        "voir [https://a.org/](https://a.org/)",
        "[ma référence](https://a.org/)",
        "[https://a.org/](https://b.org/)",
        "[file:///C:/x.png](file:///C:/x.png)",
        "[javascript:x](javascript:x)",
        "",
    ] {
        assert_eq!(
            adresse_du_lien(note),
            None,
            "« {note} » n'est pas un lien seul"
        );
    }
}
