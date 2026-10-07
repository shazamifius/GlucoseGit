//! La boîte noire qui voyage (fiche 54) : l'accord, ce qui part et sous quelle forme — sans
//! réseau, et dans un dossier d'épreuve, jamais celui de l'utilisateur.

use super::accord::{est_un_identifiant, nouvel_identifiant, Accord};
use super::envoi::{a_envoyer, corps, identifiant_de_session};
use super::Telemetrie;
use glucose_core::persist::tauri::json;
use std::path::PathBuf;

/// Un dossier d'épreuve, effacé à la fin.
struct Dossier(PathBuf);

impl Dossier {
    fn nouveau(nom: &str) -> Self {
        let p =
            std::env::temp_dir().join(format!("glucose-telemetrie-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).expect("dossier d'épreuve");
        Self(p)
    }
}

impl Drop for Dossier {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const INSTALLATION: &str = "0123456789abcdef0123456789abcdef";

/// **L'accord se relit tel qu'il s'écrit** ; sans fichier, personne n'a répondu ; un identifiant
/// qui manque ou ne se reconnaît pas se tire au hasard.
#[test]
fn test_l_accord_se_relit_et_personne_n_a_repondu_sans_fichier() {
    let a = Accord {
        envoyer: Some(true),
        installation: INSTALLATION.into(),
    };
    assert_eq!(Accord::lire(&a.ecrire()), a);
    let vide = Accord::lire("");
    assert_eq!(vide.envoyer, None);
    assert!(est_un_identifiant(&vide.installation));
    let abime = Accord::lire("envoyer=peut-etre\ninstallation=marie\n");
    assert_eq!(abime.envoyer, None);
    assert!(est_un_identifiant(&abime.installation));
}

/// **Deux identifiants tirés ne se ressemblent pas**, et le serveur accepte leur forme.
#[test]
fn test_deux_identifiants_tires_different() {
    let (a, b) = (nouvel_identifiant(), nouvel_identifiant());
    assert!(est_un_identifiant(&a) && est_un_identifiant(&b), "{a} {b}");
    assert_ne!(a, b);
}

/// **Ce qui part** : les sessions de la boîte noire, sauf celle qui s'écrit et celles déjà
/// parties, des plus anciennes aux plus récentes.
#[test]
fn test_ce_qui_part_sauf_la_courante_et_les_parties() {
    let noms: Vec<String> = [
        "session-3.jsonl",
        "session-1.jsonl",
        "session-2.jsonl",
        "carte.txt",
        "session-4.jsonl",
    ]
    .map(String::from)
    .to_vec();
    assert_eq!(
        a_envoyer(&noms, Some("session-4.jsonl"), "session-2.jsonl\n"),
        vec!["session-1.jsonl".to_string(), "session-3.jsonl".to_string()]
    );
}

/// **Le corps porte les lignes entières, et rien du nom du fichier** : une ligne déchirée par un
/// plantage reste en arrière (le serveur refuserait tout le reste), et l'identifiant de la
/// session est une empreinte — ni l'heure ni le processus que le nom portait.
#[test]
fn test_le_corps_porte_les_lignes_entieres_et_rien_du_nom() {
    let texte = "{\"type\":\"debut\",\"epoque_ms\":1,\"version\":\"2.0.2-dev\",\"systeme\":\"windows\",\"architecture\":\"x86_64\",\"demarrage_appareil_ms\":null}\n\
                 {\"type\":\"pave\",\"instant_ms\":5,\"quoi\":\"contact\",\"valeur\":0}\n\
                 {\"type\":\"episode\",\"instant_ms\":9,\"dur";
    let nom = "session-00000001791386404045-29380.jsonl";
    let c = corps(INSTALLATION, nom, texte).expect("un corps");
    let v = json::lire(&c).expect("du JSON");
    assert_eq!(v.texte("installation"), Some(INSTALLATION));
    let session = v.texte("session").expect("une session");
    assert!(est_un_identifiant(session), "{session}");
    assert_eq!(session, identifiant_de_session(INSTALLATION, nom));
    assert!(!c.contains("1791386404045") && !c.contains("29380"), "{c}");
    let lignes = v.texte("lignes").expect("les lignes");
    assert_eq!(
        lignes.lines().count(),
        2,
        "la ligne déchirée reste : {lignes}"
    );
    assert!(lignes.lines().all(|l| json::lire(l).is_ok()));
}

/// **Une session sans début ne part pas** : le serveur la refuserait.
#[test]
fn test_une_session_sans_debut_ne_part_pas() {
    let texte = "{\"type\":\"fin\",\"instant_ms\":3}\n";
    assert_eq!(corps(INSTALLATION, "s.jsonl", texte), None);
}

/// **Une vraie session de la boîte noire fait un corps** dont chaque ligne est d'un type que le
/// serveur connaît — écrite par la boîte noire elle-même, pas recopiée à la main.
#[test]
fn test_une_vraie_session_fait_un_corps() {
    let d = Dossier::nouveau("vraie");
    let (mut b, _) = crate::boite_noire::BoiteNoire::ouvrir(&d.0).expect("ouverte");
    b.image(crate::chronique::Geste::Zoomer, 900);
    b.image(crate::chronique::Geste::Repos, 100);
    let chemin = b.chemin().to_path_buf();
    b.clore();
    let texte = std::fs::read_to_string(&chemin).expect("la session");
    let c = corps(INSTALLATION, "s.jsonl", &texte).expect("un corps");
    let lignes = json::lire(&c)
        .ok()
        .and_then(|v| v.texte("lignes").map(str::to_string))
        .expect("les lignes");
    let serveur = include_str!("../../../../outils/telemetrie/src/valider.js");
    for l in lignes.lines() {
        let t = json::lire(l)
            .ok()
            .and_then(|v| v.texte("type").map(str::to_string));
        let t = t.expect("un type");
        assert!(
            serveur.contains(&format!("  {t}: {{")),
            "type inconnu du serveur : {t}"
        );
    }
}

/// **La réponse se retient, et « non » après « oui » change d'identifiant** : ce qui partira un
/// jour ne se reliera plus à ce qui a été effacé.
#[test]
fn test_la_reponse_se_retient_et_non_change_d_identifiant() {
    let d = Dossier::nouveau("reponse");
    let mut t = Telemetrie::habiter(d.0.clone(), None);
    assert_eq!(t.accorde(), None, "personne n'a répondu");
    t.repondre(true).expect("écrit");
    let premier = Accord::charger(&d.0);
    assert_eq!(premier.envoyer, Some(true));
    t.repondre(false).expect("écrit");
    let second = Accord::charger(&d.0);
    assert_eq!(second.envoyer, Some(false));
    assert_ne!(second.installation, premier.installation);
    assert_eq!(
        Telemetrie::habiter(d.0.clone(), None).accorde(),
        Some(false)
    );
}

/// **Par le menu, à la vraie boucle** : la question s'ouvre (l'épreuve y répond, DIAL-3) et la
/// réponse se retient ; une application d'épreuve, sans dossier, ne retient rien.
#[test]
fn test_par_le_menu_la_reponse_se_retient() {
    let d = Dossier::nouveau("menu");
    let mut app = crate::app::GlucoseApp::new();
    app.lancement.telemetrie = Telemetrie::habiter(d.0.clone(), None);
    crate::dialogue::epreuve::repondre(true);
    app.revoir_la_telemetrie();
    assert_eq!(app.lancement.telemetrie.accorde(), Some(true));
    crate::dialogue::epreuve::repondre(false);
    app.revoir_la_telemetrie();
    assert_eq!(Accord::charger(&d.0).envoyer, Some(false));

    let mut sans = crate::app::GlucoseApp::new();
    crate::dialogue::epreuve::repondre(true);
    sans.revoir_la_telemetrie();
    assert_eq!(sans.lancement.telemetrie.accorde(), None);
}

/// Ce qu'un serveur local a reçu : le verbe, le chemin, le type, le corps.
type Recu = (String, String, String, Vec<u8>);

/// **Un serveur local d'une requête** : il note ce qu'il reçoit et répond `code`. Jamais
/// Internet : 127.0.0.1, sur un port libre.
fn serveur_local(code: u16) -> (String, std::sync::mpsc::Receiver<Recu>) {
    use std::io::{BufRead, BufReader, Read, Write};
    let ecoute = std::net::TcpListener::bind("127.0.0.1:0").expect("un port libre");
    let racine = format!(
        "http://127.0.0.1:{}",
        ecoute.local_addr().expect("port").port()
    );
    let (vers, recu) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let Some(Ok(flux)) = ecoute.incoming().next() else {
            return;
        };
        let mut lecteur = BufReader::new(&flux);
        let mut premiere = String::new();
        let _ = lecteur.read_line(&mut premiere);
        let (mut longueur, mut genre) = (0usize, String::new());
        let mut ligne = String::new();
        while lecteur.read_line(&mut ligne).is_ok_and(|n| n > 2) {
            let (cle, valeur) = ligne.split_once(':').unwrap_or(("", ""));
            match cle.to_ascii_lowercase().as_str() {
                "content-length" => longueur = valeur.trim().parse().unwrap_or(0),
                "content-type" => genre = valeur.trim().to_string(),
                _ => {}
            }
            ligne.clear();
        }
        let mut corps = vec![0u8; longueur];
        let _ = lecteur.read_exact(&mut corps);
        let mut mots = premiere.split_whitespace();
        let (verbe, chemin) = (mots.next().unwrap_or(""), mots.next().unwrap_or(""));
        let _ = vers.send((verbe.into(), chemin.into(), genre, corps));
        let _ = (&flux).write_all(
            format!("HTTP/1.1 {code} X\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        );
    });
    (racine, recu)
}

/// **L'envoi atteint le serveur tel qu'il l'attend** (fiche 54), par la vraie porte réseau —
/// WinHTTP sous Windows, `ureq` ailleurs : `POST` du corps en JSON, et le code rendu ; puis
/// `DELETE` d'une installation.
#[test]
fn test_l_envoi_atteint_le_serveur_tel_qu_il_l_attend() {
    use crate::plateforme::{envoyer, Verbe};
    let (racine, recu) = serveur_local(201);
    let corps = br#"{"installation":"x"}"#;
    let code = envoyer(&format!("{racine}/v1/sessions"), Verbe::Deposer, corps);
    assert_eq!(code, Ok(201));
    let (verbe, chemin, genre, recu_corps) = recu.recv().expect("une requête");
    assert_eq!((verbe.as_str(), chemin.as_str()), ("POST", "/v1/sessions"));
    assert_eq!(genre, "application/json");
    assert_eq!(recu_corps, corps);

    let (racine, recu) = serveur_local(200);
    let url = format!("{racine}/v1/installations/{INSTALLATION}");
    assert_eq!(envoyer(&url, Verbe::Effacer, &[]), Ok(200));
    let (verbe, chemin, _, _) = recu.recv().expect("une requête");
    assert_eq!(verbe, "DELETE");
    assert_eq!(chemin, format!("/v1/installations/{INSTALLATION}"));
}

/// **« Non » après « oui » efface ce qui est parti, sur le serveur** : la requête d'effacement
/// part vers lui, pour l'identifiant d'avant — un serveur local, jamais Internet.
#[test]
fn test_non_apres_oui_efface_sur_le_serveur() {
    let d = Dossier::nouveau("effacer");
    let (racine, recu) = serveur_local(200);
    let mut t = Telemetrie::habiter_avec(d.0.clone(), None, Some(racine));
    t.repondre(true).expect("écrit");
    let avant = Accord::charger(&d.0).installation;
    t.repondre(false).expect("écrit");
    let (verbe, chemin, _, _) = recu
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("l'effacement est parti");
    assert_eq!(verbe, "DELETE");
    assert_eq!(chemin, format!("/v1/installations/{avant}"));
}

/// **Une épreuve n'a jamais le serveur du programme** : aucune n'atteint Internet.
#[test]
fn test_une_epreuve_n_a_jamais_le_serveur_du_programme() {
    assert_eq!(super::serveur(), None);
    assert!(super::ADRESSE.starts_with("https://"));
}
