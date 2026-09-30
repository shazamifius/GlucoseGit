//! **Les preuves d'une publication** (fiche 48) : ce que `.github/workflows/publier.yml` exige
//! avant qu'une version ne devienne un brouillon de release. Chacune est faite **par le code même
//! de Glucose** — sa comparaison des versions, sa lecture de `latest.json`, sa vérification d'une
//! signature — : ce qui passe ici est ce que ses utilisateurs accepteront.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example publication -- version <nouvelle> <publiée>
//! cargo run --release -p glucose-desktop --example publication -- programme <programme> <version>
//! cargo run --release -p glucose-desktop --example publication -- signatures <fichier>...
//! cargo run --release -p glucose-desktop --example publication -- manifeste <latest.json> <version>
//! ```
//!
//! * `version` : la nouvelle version est écrite sous sa forme exacte, et monte strictement
//!   au-dessus de la dernière publiée — sinon ni Glucose Tauri ni Glucose Rust ne la
//!   proposeraient jamais. « -dev » dit une version de travail : elle ne se publie pas.
//! * `programme` : le programme construit dit cette version ; il porte l'adresse des versions de
//!   ce dépôt et la clé de Glucose, jamais celles d'une épreuve ; sous Windows, il est fenêtré.
//! * `signatures` : chaque `<fichier>` est signé, dans son `<fichier>.sig`, par la clé de Glucose
//!   (`mise_a_jour::CLE_PUBLIQUE`) — celle de Glucose Tauri.
//! * `manifeste` : sous chacune des clés que lisent les programmes de mise à jour — celle de
//!   chaque forme d'installation, puis celle de la plateforme, dans l'ordre de Tauri —,
//!   `latest.json` propose cette version, à une adresse de la release de ce dépôt ; le fichier
//!   qu'elle désigne, posé à côté de `latest.json`, se vérifie par la signature que le manifeste
//!   porte ; et le manifeste ne porte rien d'autre.
//!
//! # Pourquoi
//!
//! La clé secrète de Glucose vit dans les secrets du dépôt, où personne ne peut la relire : la
//! publication signe par elle, sans la voir. Rien ne dit, sans ces preuves, que c'est bien **la**
//! clé dont Glucose Tauri et Glucose Rust portent la moitié publique : une signature par une autre
//! clé ne se verrait que chez ses utilisateurs — une mise à jour refusée par tous. Et le
//! `latest.json` s'écrit ailleurs (`outils/publication/manifeste.py`) : c'est le lecteur de
//! l'application qui le juge, pas celui qui l'a écrit.
//!
//! Sort en erreur à la première preuve qui ne tient pas.

use glucose_core::persist::tauri::{json, Valeur};
use glucose_desktop::mise_a_jour::cycle::ADRESSE_DE_GLUCOSE;
use glucose_desktop::mise_a_jour::installation::Installation;
use glucose_desktop::mise_a_jour::version::Version;
use glucose_desktop::mise_a_jour::{plateforme_de, proposition, verifier, CLE_PUBLIQUE};
use std::path::{Path, PathBuf};

const USAGE: &str = concat!(
    "usage : publication version <nouvelle> <publiée>\n",
    "        publication programme <programme> <version>\n",
    "        publication signatures <fichier>...\n",
    "        publication manifeste <latest.json> <version>",
);

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let preuve = match arguments.as_slice() {
        ["version", nouvelle, publiee] => monte(nouvelle, publiee),
        ["programme", programme, version] => le_programme(Path::new(programme), version),
        ["signatures", fichiers @ ..] if !fichiers.is_empty() => signatures(fichiers),
        ["manifeste", fichier, version] => manifeste(Path::new(fichier), version),
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2)
        }
    };
    if let Err(e) = preuve {
        eprintln!("{e}");
        std::process::exit(1)
    }
}

/// Une version écrite sous sa forme exacte : celle que Glucose relit et réécrit à l'identique.
fn exacte(texte: &str) -> Result<Version, String> {
    Version::lire(texte)
        .filter(|v| v.to_string() == texte)
        .ok_or_else(|| {
            format!("« {texte} » n'est pas une version sous sa forme exacte (2.0.1-beta.1)")
        })
}

/// **La nouvelle version monte** au-dessus de la dernière publiée, comme la mise à jour compare.
fn monte(nouvelle: &str, publiee: &str) -> Result<(), String> {
    let v = exacte(nouvelle)?;
    if nouvelle.contains("-dev") {
        return Err(format!(
            "{v} est une version de travail (-dev) : elle ne se publie pas"
        ));
    }
    let p = Version::lire(publiee).ok_or_else(|| format!("« {publiee} » n'est pas une version"))?;
    if v <= p {
        return Err(format!(
            "{v} ne monte pas au-dessus de {p} : personne ne la recevrait"
        ));
    }
    println!("la version {v} monte au-dessus de {p}, la dernière publiée");
    Ok(())
}

/// **Le programme publié est celui de Glucose** : il dit cette version ; il porte l'adresse des
/// versions de ce dépôt et la clé de Glucose — dans le programme, seules l'adresse et la clé
/// d'usage (`cycle::ADRESSE`, `cycle::CLE`) les emploient : les y trouver, c'est savoir qu'aucune
/// construction d'épreuve ne les a remplacées ; et sous Windows, il est fenêtré.
fn le_programme(programme: &Path, version: &str) -> Result<(), String> {
    let attendue = format!("Glucose {}", exacte(version)?);
    let sortie = std::process::Command::new(programme)
        .arg("--version")
        .output()
        .map_err(|e| format!("{} : {e}", programme.display()))?;
    let dit = String::from_utf8_lossy(&sortie.stdout).trim().to_string();
    if dit != attendue {
        return Err(format!("le programme dit « {dit} », pas « {attendue} »"));
    }
    let octets = std::fs::read(programme).map_err(|e| format!("{} : {e}", programme.display()))?;
    for (quoi, texte) in [
        ("l'adresse des versions de Glucose", ADRESSE_DE_GLUCOSE),
        ("la clé de Glucose", CLE_PUBLIQUE),
    ] {
        if !octets.windows(texte.len()).any(|w| w == texte.as_bytes()) {
            return Err(format!("le programme ne porte pas {quoi} : {texte}"));
        }
    }
    if octets.starts_with(b"MZ") && sous_systeme(&octets) != Some(FENETRE) {
        return Err("le programme n'est pas fenêtré : une console noire s'ouvrirait".into());
    }
    println!("{attendue}, l'adresse et la clé de Glucose, publiable");
    Ok(())
}

/// Le sous-système d'une application fenêtrée de Windows (`IMAGE_SUBSYSTEM_WINDOWS_GUI`).
const FENETRE: u16 = 2;

/// Le sous-système qu'annonce l'en-tête d'un exécutable de Windows : à `e_lfanew` (lu à 0x3C),
/// la signature « PE\0\0 », l'en-tête du fichier (20 octets), puis l'en-tête optionnel, où il
/// est à l'octet 68 — le même pour 32 et 64 bits.
fn sous_systeme(octets: &[u8]) -> Option<u16> {
    let lire = |a: usize, n: usize| octets.get(a..a + n);
    let pe = u32::from_le_bytes(lire(0x3C, 4)?.try_into().ok()?) as usize;
    if lire(pe, 4)? != b"PE\0\0" {
        return None;
    }
    Some(u16::from_le_bytes(lire(pe + 24 + 68, 2)?.try_into().ok()?))
}

/// **Chaque fichier est signé par la clé de Glucose**, dans son `.sig`.
fn signatures(fichiers: &[&str]) -> Result<(), String> {
    for fichier in fichiers {
        let octets = std::fs::read(fichier).map_err(|e| format!("{fichier} : {e}"))?;
        let signature = std::fs::read_to_string(format!("{fichier}.sig"))
            .map_err(|e| format!("{fichier}.sig : {e}"))?;
        verifier(&octets, signature.trim(), CLE_PUBLIQUE)
            .map_err(|e| format!("{fichier} : {e}"))?;
        println!("signé par la clé de Glucose : {fichier}");
    }
    Ok(())
}

/// Les formes d'installation de chaque plateforme, dans l'ordre de Tauri. La première est aussi
/// ce que sert la clé de la plateforme seule — l'installeur NSIS, l'AppImage qui marche sur tout
/// Linux —, pour un programme de mise à jour qui ne sait pas comment il est installé.
fn formes() -> [(String, Vec<Installation>); 2] {
    [
        (plateforme_de("windows", "x86_64"), vec![Installation::Nsis]),
        (
            plateforme_de("linux", "x86_64"),
            vec![
                Installation::AppImage(PathBuf::new()),
                Installation::Deb(PathBuf::new()),
                Installation::Rpm(PathBuf::new()),
            ],
        ),
    ]
}

/// **Le manifeste propose cette version sous chaque clé, et chaque fichier qu'il désigne se
/// vérifie** par la signature qu'il porte.
fn manifeste(fichier: &Path, version: &str) -> Result<(), String> {
    let texte =
        std::fs::read_to_string(fichier).map_err(|e| format!("{} : {e}", fichier.display()))?;
    let dossier = fichier.parent().unwrap_or(Path::new("."));
    let attendue = exacte(version)?;
    let depot = ADRESSE_DE_GLUCOSE
        .strip_suffix("latest/download/latest.json")
        .ok_or_else(|| {
            format!("{ADRESSE_DE_GLUCOSE} n'est pas l'adresse d'une release de GitHub")
        })?;
    let prefixe = format!("{depot}download/v{attendue}/");
    let mut vues: Vec<String> = Vec::new();
    for (plateforme, formes) in formes() {
        let generique = formes[0].extension();
        for forme in &formes {
            let [la_sienne, la_plateforme] = forme.cles(&plateforme);
            for (cle, extension) in [(la_sienne, forme.extension()), (la_plateforme, generique)] {
                if !vues.contains(&cle) {
                    une_entree(&texte, &cle, &attendue, &prefixe, extension, dossier)?;
                    vues.push(cle);
                }
            }
        }
    }
    rien_d_autre(&texte, &vues)?;
    println!(
        "les {} clés que lisent les programmes de mise à jour proposent Glucose {attendue}, \
         et le manifeste ne porte rien d'autre",
        vues.len()
    );
    Ok(())
}

/// **Rien d'autre que ce que les preuves ont lu** : la version, et sous chaque clé une adresse
/// et une signature. L'updater de Tauri lit le fichier d'un bloc : une date qui ne serait pas au
/// format RFC 3339, une entrée de trop mal formée — et il le refuse tout entier, pour tous ses
/// utilisateurs, sans rien dire (`tauri-plugin-updater` 2.10.1, `RemoteRelease`).
fn rien_d_autre(texte: &str, cles: &[String]) -> Result<(), String> {
    let noms = |v: Option<&Valeur>| match v {
        Some(Valeur::Carte(carte)) => carte.keys().cloned().collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    let exiger = |lus: Vec<String>, mut attendus: Vec<String>, ou: &str| {
        attendus.sort();
        if lus == attendus {
            Ok(())
        } else {
            Err(format!(
                "{ou} porte {lus:?}, et rien d'autre que {attendus:?} n'est admis"
            ))
        }
    };
    let racine = json::lire(texte).map_err(|e| e.to_string())?;
    let texte_de = |noms: &[&str]| noms.iter().map(|n| n.to_string()).collect();
    exiger(
        noms(Some(&racine)),
        texte_de(&["platforms", "version"]),
        "le manifeste",
    )?;
    let plateformes = racine.champ("platforms");
    exiger(noms(plateformes), cles.to_vec(), "platforms")?;
    for cle in cles {
        let entree = plateformes.and_then(|p| p.champ(cle));
        exiger(noms(entree), texte_de(&["signature", "url"]), cle)?;
    }
    Ok(())
}

/// Une clé du manifeste : la version attendue, une adresse de la release, un fichier de la bonne
/// forme, signé par la clé de Glucose.
fn une_entree(
    texte: &str,
    cle: &str,
    attendue: &Version,
    prefixe: &str,
    extension: &str,
    dossier: &Path,
) -> Result<(), String> {
    // Depuis la plus basse des versions : le manifeste doit proposer quelque chose.
    let plus_basse = Version::lire("0.0.0").expect("une version");
    let p = proposition(texte, &plus_basse, &[cle.to_string()])
        .map_err(|e| format!("{cle} : {e}"))?
        .ok_or_else(|| format!("{cle} : rien n'est proposé"))?;
    if p.version != *attendue {
        return Err(format!("{cle} propose {}, pas {attendue}", p.version));
    }
    let nom = p
        .url
        .strip_prefix(prefixe)
        .filter(|n| !n.contains('/') && n.ends_with(extension))
        .ok_or_else(|| format!("{cle} : {} n'est pas un {extension} de {prefixe}", p.url))?;
    let octets = std::fs::read(dossier.join(nom)).map_err(|e| format!("{cle} : {nom} : {e}"))?;
    verifier(&octets, &p.signature, CLE_PUBLIQUE).map_err(|e| format!("{cle} : {nom} : {e}"))?;
    println!("{cle} : Glucose {attendue}, {nom}, signé par la clé de Glucose");
    Ok(())
}
