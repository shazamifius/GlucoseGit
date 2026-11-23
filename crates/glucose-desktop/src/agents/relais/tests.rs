//! RELAIS-1 — le relais répond seul jusqu'au premier outil.

use super::*;
use std::io::Cursor;

const INITIALISER: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"claude-code"}}}"#;
const INITIALISE: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LISTER: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
const APPELER: &str =
    r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"glucose_lire"}}"#;

fn reponses(sortie: Vec<u8>) -> Vec<Json> {
    String::from_utf8(sortie)
        .expect("UTF-8")
        .lines()
        .map(|l| Json::lire(l).expect("JSON"))
        .collect()
}

#[test]
fn relais_1_la_poignee_de_main_et_la_liste_se_repondent_sans_glucose() {
    let entree = [INITIALISER, INITIALISE, LISTER].join("\n");
    let mut sortie = Vec::new();
    let suite = repondre_seul(&mut Cursor::new(entree.into_bytes()), &mut sortie);
    assert!(
        suite.is_none(),
        "le client a fermé avant tout outil : rien à brancher"
    );
    let r = reponses(sortie);
    assert_eq!(r.len(), 2);
    assert!(r[1].ecrire().contains("glucose_poser_page"));
}

#[test]
fn relais_1_le_premier_outil_rend_la_poignee_a_rejouer() {
    let entree = [INITIALISER, INITIALISE, LISTER, APPELER, LISTER].join("\n");
    let mut sortie = Vec::new();
    let (prelude, premier) =
        repondre_seul(&mut Cursor::new(entree.into_bytes()), &mut sortie).expect("un outil appelé");
    assert_eq!(
        prelude,
        [INITIALISER, INITIALISE],
        "la liste ne se rejoue pas"
    );
    assert_eq!(premier, APPELER);
    assert_eq!(
        reponses(sortie).len(),
        2,
        "l'appel part à Glucose, pas au relais"
    );
    assert_eq!(
        prelude
            .iter()
            .filter(|l| Json::lire(l).is_ok_and(|m| m.get("id").is_some()))
            .count(),
        1,
        "une réponse à taire"
    );
}

/// **RELAIS-2 — le relais survit à Glucose** (fiche 131), devant une fausse porte : une prise Unix
/// qui répond `{"n": rang}` à chaque requête, puis se ferme comme Glucose qu'on ferme.
#[cfg(unix)]
mod relais_2 {
    use super::*;
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    /// La sortie du relais, lisible par l'épreuve.
    #[derive(Clone, Default)]
    struct Tampon(Arc<Mutex<Vec<u8>>>);

    impl Write for Tampon {
        fn write(&mut self, o: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(o);
            Ok(o.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Tampon {
        fn lignes(&self) -> Vec<Json> {
            reponses(self.0.lock().unwrap().clone())
        }
        /// La réponse à cette requête, attendue sur un fil, avec un délai (MÉTHODE § 4.4).
        fn reponse(&self, id: f64) -> Json {
            let debut = Instant::now();
            loop {
                let trouvee = self
                    .lignes()
                    .into_iter()
                    .find(|r| r.get("id").and_then(Json::as_f64) == Some(id));
                if let Some(r) = trouvee {
                    return r;
                }
                assert!(
                    debut.elapsed() < Duration::from_secs(10),
                    "aucune réponse à {id}"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }

    /// Glucose, ouvert : il répond `n = rang` aux requêtes, et se ferme après la `derniere` —
    /// sans lui répondre si `muette`.
    fn glucose(
        chemin: &Path,
        rang: f64,
        derniere: f64,
        muette: bool,
    ) -> std::thread::JoinHandle<()> {
        let _ = std::fs::remove_file(chemin);
        let ecoute = UnixListener::bind(chemin).expect("la prise");
        let chemin = chemin.to_path_buf();
        std::thread::spawn(move || {
            let (flux, _) = ecoute.accept().expect("le relais se branche");
            let mut lecteur = BufReader::new(flux.try_clone().expect("clone"));
            let mut ecrivain = flux;
            while let Ok(Some(ligne)) = porte::lire_une_ligne(&mut lecteur) {
                let Some(id) = Json::lire(&ligne).ok().and_then(|m| m.get("id").cloned()) else {
                    continue;
                };
                let fin = id.as_f64() == Some(derniere);
                if !(fin && muette) {
                    let resultat = Json::objet([("n", Json::Nombre(rang))]);
                    let r = Json::objet([
                        ("jsonrpc", Json::from("2.0")),
                        ("id", id),
                        ("result", resultat),
                    ]);
                    writeln!(ecrivain, "{}", r.ecrire()).expect("répondre");
                }
                if fin {
                    break;
                }
            }
            // Glucose se ferme : sa prise disparaît avec lui (`Porte::drop`).
            drop(ecoute);
            let _ = std::fs::remove_file(&chemin);
        })
    }

    fn appel(id: u32) -> String {
        format!(
            r#"{{"jsonrpc":"2.0","id":{id},"method":"tools/call","params":{{"name":"glucose_lire"}}}}"#
        )
    }

    /// Le relais, branché sur cette prise ; le client écrit dans le tuyau rendu.
    fn relais(
        chemin: &Path,
        tampon: &Tampon,
    ) -> (std::io::PipeWriter, std::thread::JoinHandle<i32>) {
        let (lecteur, mut client) = std::io::pipe().expect("tuyau");
        writeln!(client, "{INITIALISER}\n{INITIALISE}").expect("client");
        let sortie: Sortie = Arc::new(Mutex::new(Box::new(tampon.clone())));
        let nom = chemin.display().to_string();
        let fil = std::thread::spawn(move || {
            relayer_par(&mut BufReader::new(lecteur), &sortie, &nom, false)
        });
        (client, fil)
    }

    fn prise(nom: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("glucose-relais-2-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("dossier");
        d.join("porte.sock")
    }

    #[test]
    fn relais_2_glucose_relance_entre_deux_appels_le_client_ne_voit_rien() {
        let chemin = prise("relance");
        let tampon = Tampon::default();
        let premier = glucose(&chemin, 1.0, 3.0, false);
        let (mut client, fil) = relais(&chemin, &tampon);
        writeln!(client, "{}", appel(3)).expect("client");
        assert_eq!(
            tampon
                .reponse(3.0)
                .get("result")
                .and_then(|r| r.get("n"))
                .and_then(Json::as_f64),
            Some(1.0)
        );
        premier.join().expect("le premier Glucose fermé");
        let second = glucose(&chemin, 2.0, 4.0, false);
        writeln!(client, "{}", appel(4)).expect("client");
        let r = tampon.reponse(4.0);
        assert_eq!(
            r.get("result")
                .and_then(|r| r.get("n"))
                .and_then(Json::as_f64),
            Some(2.0),
            "{}",
            r.ecrire()
        );
        second.join().expect("le second Glucose fermé");
        drop(client);
        assert_eq!(fil.join().expect("le relais"), 0);
        let lignes = tampon.lignes();
        assert!(
            lignes.iter().all(|r| r.get("error").is_none()),
            "aucune erreur : {lignes:?}"
        );
        let poignees = lignes
            .iter()
            .filter(|r| r.get("id").and_then(Json::as_f64) == Some(1.0))
            .count();
        assert_eq!(poignees, 1, "la poignée de main rejouée se tait");
    }

    #[test]
    fn relais_2_un_appel_que_la_fermeture_emporte_recoit_une_erreur_qui_le_dit() {
        let chemin = prise("emporte");
        let tampon = Tampon::default();
        let premier = glucose(&chemin, 1.0, 3.0, true);
        let (mut client, fil) = relais(&chemin, &tampon);
        writeln!(client, "{}", appel(3)).expect("client");
        let r = tampon.reponse(3.0);
        let message = r
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(Json::as_str);
        assert_eq!(message, Some(lien::FERMEE), "{}", r.ecrire());
        premier.join().expect("fermé");
        let second = glucose(&chemin, 2.0, 4.0, false);
        writeln!(client, "{}", appel(4)).expect("client");
        assert_eq!(
            tampon
                .reponse(4.0)
                .get("result")
                .and_then(|r| r.get("n"))
                .and_then(Json::as_f64),
            Some(2.0)
        );
        second.join().expect("fermé");
        drop(client);
        assert_eq!(fil.join().expect("le relais"), 0);
    }
}
