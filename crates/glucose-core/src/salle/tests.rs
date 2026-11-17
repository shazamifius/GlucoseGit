//! Les codes (CODE-1), la lettre et les morceaux scellés (SCELLE-1) — sans réseau.

use super::code::Code;
use super::lettre::{Corps, Lettre};
use super::morceaux::{self, Reassemblage, MORCEAU};
use crate::persist::DOCUMENT_VERSION;
use crate::store::journal::partage::{EditionPartagee, GestePartage};
use crate::store::journal::{Edit, Main, Slot};
use crate::types::{BoardImage, Project};

fn hasard(graine: u8) -> [u8; 32] {
    std::array::from_fn(|i| graine.wrapping_mul(31).wrapping_add(i as u8))
}

/// **CODE-1** : le code « écrire » donne tout ; le code « lire » donne la salle et la clé, jamais le
/// secret d'écriture ; chaque dérivée est distincte des autres ; un code se relit à l'identique.
#[test]
fn code_1_le_code_ecrire_donne_tout_le_code_lire_seulement_de_regarder() {
    let ecrire = Code::neuf(hasard(7));
    let texte = ecrire.texte_ecrire().expect("il écrit");
    assert!(texte.starts_with("glucose-ecrire-") && texte.len() == 15 + 43);
    let relu = Code::lire_le_texte(&format!("  {texte}\n")).expect("un copier-coller se relit");
    assert_eq!(relu, ecrire);
    let lire = Code::lire_le_texte(&ecrire.texte_lire()).expect("le code lire");
    assert!(!lire.peut_ecrire() && lire.texte_ecrire().is_none());
    assert!(lire.secret_d_ecriture().is_none(), "lire n'a pas le secret");
    assert_eq!((lire.salle(), lire.cle()), (ecrire.salle(), ecrire.cle()));
    assert_eq!(lire.texte_lire(), ecrire.texte_lire());
    let salle = ecrire.salle();
    assert!(salle.len() == 32 && salle.bytes().all(|o| o.is_ascii_hexdigit()));
    let secret = ecrire.secret_d_ecriture().expect("le secret");
    let octets = crate::encodage::base64(&secret).expect("du base64");
    assert_eq!(octets.len(), 32, "la salle prend de 16 à 64 octets");
    // Chaque dérivée a son domaine : aucune ne vaut une autre, ni le code lui-même.
    let cle = ecrire.cle();
    assert_ne!(octets, cle.to_vec());
    assert_ne!(cle.to_vec(), crate::encodage::base64(&texte[15..]).unwrap());
    assert_ne!(
        Code::neuf(hasard(8)).salle(),
        salle,
        "deux partages, deux salles"
    );
}

/// **CODE-1** : ce qui n'est pas un code exact se refuse — on ne devine pas.
#[test]
fn code_1_un_texte_qui_n_est_pas_un_code_se_refuse() {
    let bon = Code::neuf(hasard(3)).texte_ecrire().unwrap();
    let corps = &bon[15..];
    for faux in [
        String::new(),
        corps.to_string(),
        format!("glucose-ecrire-{}", &corps[1..]),
        format!("glucose-ecrire-{corps}A"),
        format!("glucose-ecrire-{}=", &corps[..42]),
        format!("glucose-ecrire-{} {}", &corps[..20], &corps[20..42]),
        format!("glucose-lire-{}+", &corps[..42]),
        format!("glucose-ecrir-{corps}"),
    ] {
        assert!(Code::lire_le_texte(&faux).is_none(), "{faux}");
    }
}

fn un_geste() -> GestePartage {
    GestePartage {
        editions: vec![EditionPartagee::Telle(Edit::Image {
            board: "main".into(),
            slot: Slot::inserted(0, BoardImage::new("img-a7-3", 1.5, 2.0, 10.0, 10.0)),
        })],
    }
}

/// **La lettre se relit à l'identique**, quelle que soit sa sorte ; chez celui qui la reçoit, un
/// humain devient un participant, un agent reste un agent.
#[test]
fn lettre_une_lettre_se_relit_et_signe_chez_l_autre() {
    let mut p = Project::new("partagé");
    p.boards[0]
        .images
        .push(BoardImage::new("img-1", 0.0, 0.0, 5.0, 5.0));
    for (main, corps) in [
        (Main::Humain, Corps::Base(Box::new(p))),
        (
            Main::Agent("Claude (claude.ai)".into()),
            Corps::Geste(un_geste()),
        ),
        (
            Main::Humain,
            Corps::Curseur {
                tableau: "main".into(),
                x: -3.25,
                y: 1e6,
            },
        ),
    ] {
        let l = Lettre {
            marque: "a7".into(),
            nom: "Alice".into(),
            main,
            corps,
        };
        assert_eq!(Lettre::lire(&l.ecrire()).expect("relue"), l);
    }
    let humaine = Lettre {
        marque: "a7".into(),
        nom: "Alice".into(),
        main: Main::Humain,
        corps: Corps::Geste(un_geste()),
    };
    assert_eq!(
        humaine.main_chez_moi(),
        Main::Participant {
            marque: "a7".into(),
            nom: "Alice".into()
        }
    );
    let agent = Lettre {
        main: Main::Agent("claude".into()),
        ..humaine.clone()
    };
    assert_eq!(agent.main_chez_moi(), Main::Agent("claude".into()));
    // Une lettre d'une version plus neuve se refuse en le disant.
    let mut octets = humaine.ecrire();
    octets[..2].copy_from_slice(&(DOCUMENT_VERSION + 1).to_le_bytes());
    let e = Lettre::lire(&octets).expect_err("un schéma inconnu");
    assert!(e.to_string().contains("plus neuve"), "{e}");
}

fn nonces() -> impl FnMut() -> [u8; 12] {
    let mut n = 0u8;
    move || {
        n += 1;
        [n; 12]
    }
}

/// **SCELLE-1** : une grosse lettre part en morceaux, s'entrelace avec une autre dans l'ordre de la
/// salle, et chacune se reforme entière au numéro de son dernier morceau.
#[test]
fn scelle_1_les_morceaux_de_deux_lettres_entrelaces_se_reforment() {
    let code = Code::neuf(hasard(1));
    let (cle, salle) = (code.cle(), code.salle());
    let grosse: Vec<u8> = (0..MORCEAU * 2 + 77).map(|i| (i * 7) as u8).collect();
    let mut n = nonces();
    let a = morceaux::sceller(&cle, &salle, "a7-1", &grosse, &mut n);
    let b = morceaux::sceller(&cle, &salle, "k3-1", b"petite", &mut n);
    assert_eq!((a.len(), b.len()), (3, 1));
    assert!(
        a.iter().all(|(_, o)| o.len() <= 1_500_000),
        "sous GESTE_MAX"
    );
    let ordre = [&a[0], &b[0], &a[1], &a[2]];
    let mut r = Reassemblage::default();
    let recu: Vec<_> = ordre
        .iter()
        .map(|(l, o)| r.recevoir(&cle, &salle, l, o).expect("ouvert"))
        .collect();
    assert_eq!(recu[0], None);
    assert_eq!(recu[1], Some(("k3-1".to_string(), b"petite".to_vec())));
    assert_eq!(recu[2], None);
    assert_eq!(recu[3], Some(("a7-1".to_string(), grosse)));
    // Une lettre vide est un morceau vide, pas rien.
    let vide = morceaux::sceller(&cle, &salle, "a7-2", &[], &mut n);
    let (l, o) = &vide[0];
    assert_eq!(
        r.recevoir(&cle, &salle, l, o).unwrap(),
        Some(("a7-2".into(), vec![]))
    );
}

/// **SCELLE-1** : la salle ne peut ni lire, ni déplacer, ni renommer un morceau — une autre clé,
/// une autre salle, un autre nom local ou un rang qui ment ne s'ouvrent pas.
#[test]
fn scelle_1_un_morceau_deplace_renomme_ou_menteur_se_refuse() {
    let code = Code::neuf(hasard(2));
    let (cle, salle) = (code.cle(), code.salle());
    let mut n = nonces();
    let (l, o) = morceaux::sceller(&cle, &salle, "a7-1", b"geste", &mut n).remove(0);
    let autre = Code::neuf(hasard(9));
    let mut r = Reassemblage::default();
    assert!(
        r.recevoir(&autre.cle(), &salle, &l, &o).is_err(),
        "autre clé"
    );
    assert!(
        r.recevoir(&cle, &autre.salle(), &l, &o).is_err(),
        "autre salle"
    );
    assert!(r.recevoir(&cle, &salle, "a7-9_0", &o).is_err(), "autre nom");
    // Un morceau bien scellé sous son nom, mais qui se dit d'un autre rang.
    let mut w = crate::persist::bytes::Writer::new();
    w.count(1);
    w.count(2);
    w.raw(b"x");
    let menteur = morceaux::sceller_un(
        &cle,
        [5; 12],
        &morceaux::associees(&salle, "a7-1_0"),
        &w.into_bytes(),
    );
    let e = r.recevoir(&cle, &salle, "a7-1_0", &menteur).unwrap_err();
    assert!(e.contains("rang"), "{e}");
    assert!(
        r.recevoir(&cle, &salle, &l, &o).unwrap().is_some(),
        "le vrai passe"
    );
}

/// **Un curseur** se scelle à part : il s'ouvre comme curseur, jamais comme morceau d'une lettre.
#[test]
fn scelle_1_un_curseur_n_est_pas_un_morceau() {
    let code = Code::neuf(hasard(4));
    let (cle, salle) = (code.cle(), code.salle());
    let o = morceaux::sceller_curseur(&cle, &salle, b"x=3", [1; 12]);
    assert_eq!(
        morceaux::ouvrir_curseur(&cle, &salle, &o).as_deref(),
        Some(&b"x=3"[..])
    );
    assert!(Reassemblage::default()
        .recevoir(&cle, &salle, "curseur", &o)
        .is_err());
    assert!(morceaux::ouvrir_curseur(&Code::neuf(hasard(5)).cle(), &salle, &o).is_none());
}
