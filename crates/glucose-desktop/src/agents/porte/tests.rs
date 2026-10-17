//! La lecture bornée, le service d'une connexion, et la vraie porte du système.

use super::*;
use crate::agents::json::Json;
use std::io::{BufRead, BufReader, Cursor, Write};
use std::sync::Arc;

#[test]
fn une_ligne_se_lit_sans_son_retour_et_la_derniere_sans_fin_aussi() {
    let mut l = Cursor::new(b"un\r\ndeux\ntrois".to_vec());
    assert_eq!(lire_une_ligne(&mut l).expect("lu").as_deref(), Some("un"));
    assert_eq!(lire_une_ligne(&mut l).expect("lu").as_deref(), Some("deux"));
    assert_eq!(
        lire_une_ligne(&mut l).expect("lu").as_deref(),
        Some("trois")
    );
    assert_eq!(lire_une_ligne(&mut l).expect("lu"), None);
}

#[test]
fn une_ligne_sans_fin_ne_remplit_pas_la_memoire() {
    // Fini, pour qu'une borne retirée fasse tomber l'épreuve au lieu de la bloquer.
    let mut l = BufReader::new(std::io::Read::take(
        std::io::repeat(b'x'),
        LIGNE_MAX as u64 + 2,
    ));
    assert!(lire_une_ligne(&mut l).is_err());
}

/// Une boîte dont un fil répond à chaque appel ce qu'il a reçu — la boucle de Glucose, en
/// plus simple.
fn boite_qui_repond() -> Boite {
    let boite = Boite::nouvelle(Arc::new(|| {}));
    let b = boite.clone();
    std::thread::spawn(move || loop {
        for d in b.relever() {
            let r = Json::objet([("outil", Json::from(d.appel.nom.as_str()))]);
            let _ = d
                .reponse
                .send(crate::agents::protocole::reponse_d_outil(&d.appel, Ok(r)));
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    });
    boite
}

const DECOUVRIR: &str = r#"{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28"}}}"#;
const APPELER: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"glucose_lire","_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28"}}}"#;

#[test]
fn une_connexion_repond_dans_l_ordre_et_confie_les_outils_a_la_boucle() {
    let entree = format!(
        "{DECOUVRIR}\n\n{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/x\"}}\n{APPELER}\n"
    );
    let mut sortie = Vec::new();
    servir(
        Cursor::new(entree.into_bytes()),
        &mut sortie,
        &boite_qui_repond(),
    );
    let lignes: Vec<Json> = String::from_utf8(sortie)
        .expect("UTF-8")
        .lines()
        .map(|l| Json::lire(l).expect("JSON"))
        .collect();
    assert_eq!(lignes.len(), 2, "la notification ne reçoit rien");
    assert_eq!(lignes[0].get("id").and_then(Json::as_f64), Some(1.0));
    let texte = lignes[1]
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(Json::as_liste)
        .and_then(|c| c[0].get("text"))
        .and_then(Json::as_str)
        .expect("le texte de l'outil");
    assert_eq!(texte, r#"{"outil":"glucose_lire"}"#);
}

#[test]
fn quand_glucose_se_ferme_la_connexion_se_termine_au_lieu_d_attendre() {
    // Une boîte que personne ne relève, et dont la demande est jetée : la boucle est partie.
    let boite = Boite::nouvelle(Arc::new(|| {}));
    let b = boite.clone();
    let jeteur = std::thread::spawn(move || loop {
        if !b.relever().is_empty() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    });
    let entree = format!("{APPELER}\n{DECOUVRIR}\n");
    let mut sortie = Vec::new();
    servir(Cursor::new(entree.into_bytes()), &mut sortie, &boite);
    jeteur.join().expect("le jeteur");
    assert!(sortie.is_empty(), "rien ne suit un appel resté sans boucle");
}

/// Un nom de porte à cette épreuve seule.
fn nom_d_epreuve() -> (String, Option<crate::epreuve::Dossier>) {
    #[cfg(windows)]
    {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static RANG: AtomicUsize = AtomicUsize::new(0);
        let rang = RANG.fetch_add(1, Ordering::Relaxed);
        (
            format!(r"\\.\pipe\glucose-epreuve-{}-{rang}", std::process::id()),
            None,
        )
    }
    #[cfg(not(windows))]
    {
        let d = crate::epreuve::Dossier::nouveau("porte");
        (d.join("p.sock").to_string_lossy().into_owned(), Some(d))
    }
}

#[test]
fn agents_2_la_vraie_porte_sert_un_agent_et_ne_s_ouvre_qu_une_fois() {
    let (nom, _dossier) = nom_d_epreuve();
    let porte = Porte::ouvrir(&nom, boite_qui_repond()).expect("la porte s'ouvre");
    assert_eq!(porte.nom(), nom);
    assert!(
        Porte::ouvrir(&nom, boite_qui_repond()).is_none(),
        "un second Glucose ne prend pas la porte du premier"
    );
    // Deux agents à la fois : chacun sa connexion.
    for _ in 0..2 {
        let flux = connecter(&nom).expect("la porte répond, et elle est de ce compte");
        let mut ecrit = flux.try_clone().expect("un second bout");
        writeln!(ecrit, "{DECOUVRIR}\n{APPELER}").expect("écrit");
        let mut lu = BufReader::new(flux);
        let mut ligne = String::new();
        lu.read_line(&mut ligne).expect("la découverte");
        assert!(ligne.contains("supportedVersions"), "{ligne}");
        ligne.clear();
        lu.read_line(&mut ligne).expect("l'outil");
        assert!(ligne.contains("glucose_lire"), "{ligne}");
    }
}

/// **TUYAU-1** : une lecture en attente n'empêche pas d'écrire. Le fil qui lit est déjà bloqué
/// quand l'autre écrit la première requête, puis la seconde après la première réponse — l'ordre
/// même d'un relais. En mode synchrone, Windows figeait la seconde (17/10). L'épreuve tombe par
/// un délai au lieu de bloquer.
#[test]
fn tuyau_1_lire_et_ecrire_en_meme_temps_ne_se_bloquent_pas() {
    use std::sync::mpsc::channel;
    use std::time::Duration;
    let (nom, _dossier) = nom_d_epreuve();
    let _porte = Porte::ouvrir(&nom, boite_qui_repond()).expect("la porte s'ouvre");
    let flux = connecter(&nom).expect("la porte répond");
    let mut ecrit = flux.try_clone().expect("un second bout");
    let (envoi, reponses) = channel();
    std::thread::spawn(move || {
        let mut lu = BufReader::new(flux);
        loop {
            let mut ligne = String::new();
            if lu.read_line(&mut ligne).unwrap_or(0) == 0 {
                return;
            }
            let _ = envoi.send(ligne);
        }
    });
    // L'écrivain a son fil à lui : sur un tuyau qui se fige, c'est l'écriture qui bloque, et le fil
    // de l'épreuve ne fait qu'attendre — avec un délai, pour tomber au lieu de se figer.
    let (aller, ordres) = channel::<&'static str>();
    std::thread::spawn(move || {
        for requete in ordres {
            if writeln!(ecrit, "{requete}").is_err() {
                return;
            }
        }
    });
    // La lecture est en attente avant toute écriture.
    std::thread::sleep(Duration::from_millis(200));
    for requete in [DECOUVRIR, APPELER] {
        aller.send(requete).expect("l'écrivain attend");
        let r = reponses
            .recv_timeout(Duration::from_secs(10))
            .expect("la réponse arrive pendant que l'autre fil lit");
        assert!(r.contains("\"result\""), "{r}");
    }
}
