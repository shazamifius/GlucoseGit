//! Le cycle d'une mise à jour, de bout en bout : un vrai serveur HTTP sur cette machine sert le
//! fichier des versions et un installeur signé par la clé d'essai ; le cycle les télécharge par
//! le vrai téléchargeur, vérifie, pose — ou refuse sans rien écrire.

use super::*;

fn dossier(nom: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("glucose-cycle-{nom}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// **Les installeurs qui ont fait leur travail se retirent** — ceux d'une version déjà
/// atteinte ; un installeur plus récent, préparé et pas encore lancé, reste, et rien d'autre
/// n'est touché.
#[test]
fn test_les_installeurs_d_avant_se_rangent() {
    let d = dossier("ranger");
    std::fs::create_dir_all(&d).unwrap();
    let suffixe = std::env::consts::EXE_SUFFIX;
    for v in ["0.9.0", "1.0.0", "1.0.1-beta.1", "2.0.1"] {
        std::fs::write(d.join(format!("Glucose_{v}_installeur{suffixe}")), b"x").unwrap();
    }
    std::fs::write(d.join("autre.txt"), b"x").unwrap();
    ranger_les_anciens(&d, &Version::lire("1.0.0").unwrap());
    let mut restes: Vec<String> = std::fs::read_dir(&d)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    restes.sort();
    assert_eq!(
        restes,
        vec![
            format!("Glucose_1.0.1-beta.1_installeur{suffixe}"),
            format!("Glucose_2.0.1_installeur{suffixe}"),
            "autre.txt".to_string(),
        ]
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **De bout en bout, par le vrai téléchargeur de chaque système** — WinHTTP sous Windows,
/// `ureq` ailleurs (notes `decisions/02` et `05`).
mod de_bout_en_bout {
    use super::super::super::tests::{CLE_D_ESSAI, MESSAGE, SIG_PREHACHEE};
    use super::super::*;
    use super::dossier;

    /// La forme d'installation des épreuves : sa clé retombe sur celle de la plateforme, sur
    /// chaque système.
    const NSIS: Installation = Installation::Nsis;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    /// **Un serveur HTTP d'un après-midi** : il rend, pour chaque chemin demandé, les octets que
    /// `pages` lui donne — une fonction de sa propre racine, pour qu'un manifeste puisse désigner
    /// un installeur sur le même serveur. Rend cette racine.
    fn serveur(pages: impl FnOnce(&str) -> Vec<(&'static str, Vec<u8>)>) -> String {
        let ecoute = TcpListener::bind("127.0.0.1:0").expect("un port libre");
        let racine = format!(
            "http://127.0.0.1:{}",
            ecoute.local_addr().expect("son port").port()
        );
        let pages = pages(&racine);
        std::thread::spawn(move || {
            for flux in ecoute.incoming().flatten() {
                let mut lecteur = BufReader::new(&flux);
                let mut premiere = String::new();
                let _ = lecteur.read_line(&mut premiere);
                // Le reste de la requête : jusqu'à la ligne vide.
                let mut ligne = String::new();
                while lecteur.read_line(&mut ligne).is_ok_and(|n| n > 2) {
                    ligne.clear();
                }
                let chemin = premiere.split_whitespace().nth(1).unwrap_or("/");
                let corps = pages
                    .iter()
                    .find(|(c, _)| *c == chemin)
                    .map(|(_, o)| o.clone());
                let mut flux = &flux;
                let _ = match corps {
                    Some(o) => flux
                        .write_all(
                            format!(
                                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                o.len()
                            )
                            .as_bytes(),
                        )
                        .and_then(|()| flux.write_all(&o)),
                    None => flux.write_all(
                        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    ),
                };
            }
        });
        racine
    }

    /// Le fichier des versions : `version`, pour cette plateforme, l'installeur à `url`.
    fn manifeste(version: &str, url: &str) -> Vec<u8> {
        format!(
            "{{\"version\":\"{version}\",\"notes\":\"essai\",\"platforms\":{{\"{}\":\
             {{\"signature\":\"{SIG_PREHACHEE}\",\"url\":\"{url}\"}}}}}}",
            super::super::super::plateforme()
        )
        .into_bytes()
    }

    /// **De bout en bout** : le fichier des versions propose, l'installeur se télécharge, sa
    /// signature se vérifie par la clé, et il se pose — octet pour octet.
    #[test]
    fn test_une_version_se_cherche_se_telecharge_se_verifie_et_se_pose() {
        let racine = serveur(|racine| {
            vec![
                (
                    "/latest.json",
                    manifeste("9.9.9", &format!("{racine}/installeur.exe")),
                ),
                ("/installeur.exe", MESSAGE.to_vec()),
            ]
        });
        let p = chercher(
            &format!("{racine}/latest.json"),
            &Version::lire("1.0.0").unwrap(),
            &NSIS,
        )
        .expect("lu")
        .expect("proposée");
        assert_eq!(p.version.to_string(), "9.9.9");
        let d = dossier("pose");
        let chemin = preparer(&p, CLE_D_ESSAI, &d, &NSIS).expect("vérifié et posé");
        assert_eq!(std::fs::read(&chemin).unwrap(), MESSAGE);
        assert!(chemin
            .to_string_lossy()
            .contains("Glucose_9.9.9_installeur"));
        let _ = std::fs::remove_dir_all(&d);
    }

    /// **Un installeur altéré ne touche jamais le disque** : un octet changé en route, et la
    /// signature le refuse avant toute écriture.
    #[test]
    fn test_un_installeur_altere_ne_touche_jamais_le_disque() {
        let mut altere = MESSAGE.to_vec();
        altere[3] ^= 1;
        let racine = serveur(|_| vec![("/installeur.exe", altere)]);
        let p = Proposition {
            version: Version::lire("9.9.9").unwrap(),
            notes: String::new(),
            url: format!("{racine}/installeur.exe"),
            signature: SIG_PREHACHEE.to_string(),
        };
        let d = dossier("altere");
        assert!(preparer(&p, CLE_D_ESSAI, &d, &NSIS).is_err());
        assert!(!d.exists(), "rien n'a été écrit");
        // Et la clé de Glucose, elle, n'a jamais signé l'essai.
        let racine = serveur(|_| vec![("/installeur.exe", MESSAGE.to_vec())]);
        let p = Proposition {
            url: format!("{racine}/installeur.exe"),
            ..p
        };
        assert!(preparer(&p, super::super::super::CLE_PUBLIQUE, &d, &NSIS).is_err());
        assert!(!d.exists());
    }

    /// **Une version déjà atteinte ne se propose pas**, et un fichier absent se dit.
    #[test]
    fn test_rien_de_neuf_ne_propose_rien() {
        let racine = serveur(|_| vec![("/latest.json", manifeste("1.0.0", "http://x/y.exe"))]);
        let fichier = format!("{racine}/latest.json");
        assert_eq!(
            chercher(&fichier, &Version::lire("1.0.0").unwrap(), &NSIS),
            Ok(None)
        );
        assert!(chercher(&format!("{racine}/absent.json"), &courante(), &NSIS).is_err());
    }
}
