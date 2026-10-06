//! Le cœur de la mise à jour, éprouvé sans réseau.
//!
//! **Les signatures d'essai** : une clé tirée d'une graine fixe, SHA-256 de « glucose : clé
//! d'essai de la mise à jour » ; Ed25519 d'après le code de référence de la RFC 8032 § 6, au
//! format de minisign et sous la forme de Tauri (le base64 du fichier entier). Elles ne signent
//! rien d'autre que le message d'essai ; la clé de Glucose, elle, ne sort jamais de chez lui.

use super::version::Version;
use super::*;

/// La clé d'essai, telle que Tauri porte la sienne.
pub(super) const CLE_D_ESSAI: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDY4RDgxNzU5Q0UzM0M5OTkKUldTWnlUUE9XUmZZYUJ3UDBBZGVQeUI5eklUTlptaU5tTEJBTTFQUkJUcUtmeGFTUVpSSVhBVGwK";

/// Ce que signent les deux signatures d'essai : un « installeur ».
pub(super) const MESSAGE: &[u8] = b"installeur d'essai de Glucose\n";

/// Préhachée (`ED`, BLAKE2b-512) : ce que produit le signataire de Tauri.
pub(super) const SIG_PREHACHEE: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVTWnlUUE9XUmZZYUdvRmtsTFlOTVZ1UXNZeU55akcxdVk3OFBwTWNKZWNVQzdRYVVwaE9Pa25xOGlEQUM1UjNWK09jMHRjYUhlclgvKzRtV2lMQ2I4U015NFlkbEFVVVFrPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkwMDAwMDAwCWZpbGU6R2x1Y29zZV8yLjAuMV94NjQtc2V0dXAuZXhlCjhYZ2w4emVHeWUrU090dXcwV2NxdHh6NGhUaVk1bjJNZG5uMXRwUjZRREt3bHFmOVQybFJ6NlFIaUVvUzliRXE2WnB5VE1ueG9CaEVjc0hBckI1NkNBPT0K";

/// Historique (`Ed`) : le message signé tel quel.
const SIG_HISTORIQUE: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUldTWnlUUE9XUmZZYU5SQnVCSjRmclRYYTd2S2FHMkdHdGYyQ1QxZVFwN29PS0xaVWFIbnlLa0xKZ1VnVzFCWW9HSExpdUlDK3BIMkQ5aGlCaXMzTEExTmJ1ZVdZRDVmNXd3PQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkwMDAwMDAwCWZpbGU6R2x1Y29zZV8yLjAuMV94NjQtc2V0dXAuZXhlCnJNQitzZkszb3RTeENBSW4yQTcvSEZrcTQvb3hHSktyLzlzalVFakJDZndNTkY2aXhJdUtSS0Zia0lFTUFlR1VZVS9HYUZlOWFPbEZSY1hOR0dXb0RnPT0K";

fn v(texte: &str) -> Version {
    Version::lire(texte).expect(texte)
}

/// **Les versions montent comme semver le dit** — l'exemple de la spécification, puis la bascule.
#[test]
fn test_les_versions_s_ordonnent_comme_semver() {
    let chaine = [
        "1.0.0-alpha",
        "1.0.0-alpha.1",
        "1.0.0-alpha.beta",
        "1.0.0-beta",
        "1.0.0-beta.2",
        "1.0.0-beta.11",
        "1.0.0-rc.1",
        "1.0.0",
        "1.0.2-beta.1",
        "2.0.1-beta.1",
        "2.0.1",
        "2.0.10",
        "10.0.0",
    ];
    for paire in chaine.windows(2) {
        assert!(v(paire[0]) < v(paire[1]), "{} < {}", paire[0], paire[1]);
    }
    assert_eq!(v("v2.0.1"), v("2.0.1"), "le v devant est admis");
    assert_eq!(v("2.0.1+abc"), v("2.0.1"), "la construction ne compte pas");
    assert_eq!(v("2.0.1-beta.1").to_string(), "2.0.1-beta.1");
    for faux in ["", "2.0", "2.0.1.4", "a.b.c", "2.0.1-", "2.0.1-beta..1"] {
        assert_eq!(
            Version::lire(faux),
            None,
            "« {faux} » n'est pas une version"
        );
    }
}

/// **La version de travail dépasse tout ce qui a été publié.** En dessous de la dernière de Glucose
/// Tauri, elle proposerait de réinstaller Glucose Tauri ; en dessous de la dernière de Glucose Rust
/// (oubli de la 2.0.1, corrigé le 06/10/2026), elle se croirait plus vieille que ce que ses
/// utilisateurs ont déjà. À relever après chaque publication (fiche 48 § 15.4).
#[test]
fn test_la_version_depasse_tout_ce_qui_est_publie() {
    for publiee in ["1.0.2-beta.1", "2.0.1-beta.1"] {
        assert!(
            Version::courante() > v(publiee),
            "Glucose {} se croirait plus vieux que la {publiee} publiée",
            Version::courante()
        );
    }
}

/// **Les plateformes se nomment comme chez Tauri.**
#[test]
fn test_les_plateformes_se_nomment_comme_chez_tauri() {
    assert_eq!(plateforme_de("windows", "x86_64"), "windows-x86_64");
    assert_eq!(plateforme_de("macos", "aarch64"), "darwin-aarch64");
    assert_eq!(plateforme_de("linux", "aarch64"), "linux-aarch64");
    assert_eq!(plateforme_de("windows", "x86"), "windows-i686");
}

/// La clé de Windows seule.
fn windows() -> [String; 1] {
    ["windows-x86_64".to_string()]
}

/// Un `latest.json` comme Tauri les publie.
fn manifeste(version: &str) -> String {
    format!(
        concat!(
            "{{\"version\":\"{}\",\"notes\":\"la bascule\",\"pub_date\":\"2026-10-01T10:00:00Z\",",
            "\"platforms\":{{\"windows-x86_64\":{{\"signature\":\"{}\",",
            "\"url\":\"https://exemple.org/Glucose_setup.exe\"}}}}}}"
        ),
        version, SIG_PREHACHEE
    )
}

/// **Une version ne fait que monter** : proposée si elle est plus grande, rien sinon — ni la
/// même, ni une plus ancienne.
#[test]
fn test_une_version_ne_se_propose_que_si_elle_monte() {
    let texte = manifeste("2.0.1");
    let p = proposition(&texte, &v("1.0.2-beta.1"), &windows())
        .expect("lisible")
        .expect("proposée");
    assert_eq!(p.version, v("2.0.1"));
    assert_eq!(p.url, "https://exemple.org/Glucose_setup.exe");
    assert_eq!(p.signature, SIG_PREHACHEE);
    assert_eq!(p.notes, "la bascule");
    for courante in ["2.0.1", "2.0.2", "3.0.0-alpha"] {
        assert_eq!(
            proposition(&texte, &v(courante), &windows()),
            Ok(None),
            "depuis {courante}"
        );
    }
}

/// **La clé de la forme d'installation passe avant celle de la plateforme**, comme chez Tauri :
/// un paquet `.deb` prend l'entrée `-deb`, et à défaut l'entrée générale.
#[test]
fn test_la_forme_d_installation_passe_avant_la_plateforme() {
    let texte = concat!(
        "{\"version\":\"2.0.1\",\"platforms\":{",
        "\"linux-x86_64\":{\"signature\":\"s\",\"url\":\"https://x/Glucose.AppImage\"},",
        "\"linux-x86_64-deb\":{\"signature\":\"s\",\"url\":\"https://x/glucose.deb\"}}}"
    );
    let url = |cles: [String; 2]| {
        proposition(texte, &v("2.0.0"), &cles)
            .expect("lisible")
            .expect("proposée")
            .url
    };
    let deb = installation::Installation::Deb("/usr/bin/glucose".into());
    assert_eq!(url(deb.cles("linux-x86_64")), "https://x/glucose.deb");
    let rpm = installation::Installation::Rpm("/usr/bin/glucose".into());
    assert_eq!(url(rpm.cles("linux-x86_64")), "https://x/Glucose.AppImage");
    assert_eq!(
        proposition(texte, &v("2.0.0"), &rpm.cles("linux-aarch64")),
        Err(Refus::SansCettePlateforme(
            "linux-aarch64-rpm ou linux-aarch64".into()
        ))
    );
}

/// **Un fichier sans cette plateforme, ou illisible, le dit** — sans rien proposer.
#[test]
fn test_ce_qui_manque_se_dit() {
    let texte = manifeste("2.0.1");
    assert_eq!(
        proposition(&texte, &v("1.0.0"), &["darwin-aarch64".to_string()]),
        Err(Refus::SansCettePlateforme("darwin-aarch64".into()))
    );
    assert!(matches!(
        proposition("pas du json", &v("1.0.0"), &windows()),
        Err(Refus::Illisible(_))
    ));
    assert!(matches!(
        proposition("{\"version\":\"deux\"}", &v("1.0.0"), &windows()),
        Err(Refus::Illisible(_))
    ));
}

/// **Ce qui est signé par la clé passe** — préhaché comme historique, comme chez Tauri.
#[test]
fn test_ce_qui_est_signe_passe() {
    assert_eq!(verifier(MESSAGE, SIG_PREHACHEE, CLE_D_ESSAI), Ok(()));
    assert_eq!(verifier(MESSAGE, SIG_HISTORIQUE, CLE_D_ESSAI), Ok(()));
}

/// Une clé tirée par le signataire de Tauri lui-même (`tauri-cli` 2.12.0, `signer generate`),
/// et ce qu'il a signé de `MESSAGE` (`signer sign`) — le 29/09/2026, clé secrète jetée ensuite.
const CLE_DU_SIGNATAIRE_DE_TAURI: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEU0RTRCNUJFMTQzN0UxMzEKUldReDRUY1V2clhrNUNIaVoxMUdLUjFiYThZREpZYzlrZit6Slcvb2o3RlZEYXBEcnJDWHQ3bGQK";
const SIG_DU_SIGNATAIRE_DE_TAURI: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVReDRUY1V2clhrNUowS0gyQnR2V3pXdTNFTFZ6Nk1uQUUrRzM1VXU2SUR6RGVRaGdVVVJrMmN3dWE4aVVuWkhkMUx2ZkxRZ3BRNTllY1NXeDhSaWI4Nm9MUWdkNmxSN1FvPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkwNzEyNTkyCWZpbGU6bWVzc2FnZS5iaW4Kajg4UllHNkFqNVBQbjZra1E1SjNrM2Nkc3lWLzkrdmFyQ3JpcEtyckFhVVJIcmhKR1lHM1RwVmVCcGtKekdXQlJNYjVrbEFsNDJ5QXYrWGozNWMxQWc9PQo=";

/// **Ce que le vrai signataire de Tauri produit passe** — et rien d'autre. Les signatures
/// d'essai ci-dessus sont faites d'après la RFC ; celle-ci sort de l'outil même qui signera les
/// installeurs de Glucose : si Tauri change sa forme un jour, c'est ici que ça tombe.
#[test]
fn test_ce_que_signe_l_outil_de_tauri_passe() {
    assert_eq!(
        verifier(
            MESSAGE,
            SIG_DU_SIGNATAIRE_DE_TAURI,
            CLE_DU_SIGNATAIRE_DE_TAURI
        ),
        Ok(())
    );
    let mut change = MESSAGE.to_vec();
    change[0] ^= 1;
    assert!(verifier(
        &change,
        SIG_DU_SIGNATAIRE_DE_TAURI,
        CLE_DU_SIGNATAIRE_DE_TAURI
    )
    .is_err());
    assert!(verifier(MESSAGE, SIG_DU_SIGNATAIRE_DE_TAURI, CLE_D_ESSAI).is_err());
}

/// **Rien ne s'installe qui ne soit signé par la clé** : un octet changé, une signature
/// abîmée, une autre clé — la vraie clé de Glucose n'a pas signé l'essai.
#[test]
fn test_rien_ne_passe_sans_la_cle() {
    let mut change = MESSAGE.to_vec();
    change[0] ^= 1;
    assert!(matches!(
        verifier(&change, SIG_PREHACHEE, CLE_D_ESSAI),
        Err(Refus::NonSigne(_))
    ));
    let abimee = SIG_PREHACHEE.replacen('U', "V", 1);
    assert!(verifier(MESSAGE, &abimee, CLE_D_ESSAI).is_err());
    assert_eq!(
        verifier(MESSAGE, SIG_PREHACHEE, CLE_PUBLIQUE),
        Err(Refus::NonSigne("la signature ne correspond pas")),
        "la clé de Glucose se lit, et n'a pas signé l'essai"
    );
    assert!(verifier(MESSAGE, SIG_PREHACHEE, "pas une clé").is_err());
}
