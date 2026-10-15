//! **Ce qui part, et comment** : les sessions closes de la boîte noire, une à une, sur un fil à
//! part, au lancement — jamais pendant qu'on dessine (fiche 49 § 2.4).
//!
//! Une session partie se retient dans un fichier à côté de l'accord ; une session que le serveur
//! refuse aussi — la renvoyer à chaque lancement ne la ferait pas accepter. Un réseau absent, en
//! revanche, n'est qu'un report : on réessaiera au lancement suivant.
//!
//! # Une longue session part toujours (POIDS-1, fiche 72)
//!
//! Le serveur refuse une session de plus de 512 Kio, et une session de trois minutes au
//! téléphone en pèse 72 : chaque longue session — celles qui comptent, celle de ses 6 000 images
//! qui s'est fermée seule — était refusée, retenue « refusée », et ne repartait jamais. Une
//! session trop lourde perd désormais ses **plus anciennes mesures** (épisodes, postes, records,
//! signes du pavé) jusqu'à tenir : ce qui précède une mort reste, et une ligne `coupee` dit
//! combien sont parties. Celles qu'un Glucose d'avant s'est vu refuser pour leur poids repartent
//! une fois : la liste ne disait pas pourquoi elle les retenait, leur poids le dit.
//!
//! Et la même leçon pour demain : une session refusée se retient **avec la version qui l'a
//! envoyée**. Une version suivante la retente une fois — le serveur aura peut-être appris la
//! ligne qui la faisait refuser, et il ignore ce qu'il a déjà reçu.

use crate::plateforme::Verbe;
use std::path::{Path, PathBuf};

/// Les sessions déjà parties (ou refusées), une par ligne, à côté de l'accord : sa clé, puis
/// le verdict (`partie`, ou `refusee` et la version qui l'a envoyée) — seule, sans verdict, la
/// clé vient d'un Glucose d'avant la fiche 72, qui retenait de la même façon une session reçue
/// et une session refusée.
pub const ENVOYEES: &str = "telemetrie-envoyees.txt";

/// **Le poids le plus lourd qu'une session peut avoir** : celui que le serveur accepte
/// (`POIDS_MAX` de `outils/telemetrie/src/valider.js`, qu'une épreuve lie à celui-ci).
pub const POIDS_MAX: usize = 512 * 1024;

/// Les lignes qui mesurent le temps qui passe : c'est d'elles que croît une longue session, et
/// ce sont elles qui partent, les plus anciennes d'abord.
const MESURES: [&str; 4] = ["episode", "poste", "pire", "pave"];

/// Une session du journal : le nom de son fichier, et sa clé ([`crate::boite_noire::fichiers::cle`]).
pub type Session = (String, String);

/// **Les sessions à envoyer** : celles du journal, sauf celle qui s'écrit et celles déjà
/// parties — reconnues par leur clé, que le nom du fichier peut changer sans elle —, des plus
/// anciennes aux plus récentes. Une session retenue par un Glucose d'avant la fiche 72 repart si
/// elle est `lourde` : il l'a envoyée entière, et le serveur l'a refusée pour son poids. Une
/// session refusée par une autre `version` que celle-ci repart aussi.
pub fn a_envoyer(
    sessions: &[Session],
    (courante, version): (Option<&str>, &str),
    envoyees: &str,
    lourde: impl Fn(&Session) -> bool,
) -> Vec<Session> {
    let mut restent: Vec<Session> = sessions
        .iter()
        .filter(|(nom, _)| Some(nom.as_str()) != courante)
        .filter(|s| match deja(envoyees, &s.1, version) {
            Deja::Non => true,
            Deja::Retenue => false,
            Deja::RetenueAvant => lourde(s),
        })
        .cloned()
        .collect();
    restent.sort_by(|a, b| a.1.cmp(&b.1));
    restent
}

/// Ce que la liste dit d'une session.
enum Deja {
    Non,
    /// Retenue avec son verdict.
    Retenue,
    /// Retenue sans verdict, par un Glucose d'avant la fiche 72.
    RetenueAvant,
}

fn deja(envoyees: &str, cle: &str, version: &str) -> Deja {
    let mut d = Deja::Non;
    for ligne in envoyees.lines() {
        let mut mots = ligne.split_whitespace();
        if mots.next() != Some(cle) {
            continue;
        }
        match (mots.next(), mots.next()) {
            (None, _) => d = Deja::RetenueAvant,
            // Refusée par une autre version : celle-ci la retente, à moins de l'avoir déjà fait.
            (Some("refusee"), Some(par)) if par != version => {}
            (Some(_), _) => return Deja::Retenue,
        }
    }
    d
}

/// **Les lignes entières d'une session**, et le type de chacune : une session coupée par un
/// plantage finit sur une ligne déchirée, que le serveur refuserait avec tout le reste. `None`
/// si rien ne commence par la ligne de début.
fn lignes_entieres(texte: &str) -> Option<Vec<(&str, String)>> {
    use glucose_core::persist::tauri::json;
    let lignes: Vec<(&str, String)> = texte
        .lines()
        .filter_map(|l| {
            let v = json::lire(l).ok()?;
            Some((l, v.texte("type").unwrap_or_default().to_string()))
        })
        .collect();
    (lignes.first()?.1 == "debut").then_some(lignes)
}

/// **Le poids d'une session telle qu'un Glucose d'avant la fiche 72 l'envoyait** : ses lignes
/// entières, sans rien retirer.
pub fn poids_entier(texte: &str) -> usize {
    lignes_entieres(texte).map_or(0, |l| l.iter().map(|(l, _)| l.len() + 1).sum())
}

/// La ligne qui dit qu'une session a perdu ses `lignes` premières mesures, jusqu'à `instant_ms`.
pub fn ligne_coupee(instant_ms: u64, lignes: u64) -> String {
    format!("{{\"type\":\"coupee\",\"instant_ms\":{instant_ms},\"lignes\":{lignes}}}")
}

/// **Ce qui part d'une session** : ses lignes entières, et si elles pèsent plus que `max`,
/// sans leurs plus anciennes mesures — la ligne `coupee` le dit, juste après le début. Ce qu'il
/// faut retirer compte la ligne qui le dira, à son plus long : aucune marge. `None` si rien ne
/// commence par la ligne de début.
pub fn a_envoyer_dans(texte: &str, max: usize) -> Option<String> {
    let lignes = lignes_entieres(texte)?;
    let poids = |l: &str| l.len() + 1;
    let total: usize = lignes.iter().map(|(l, _)| poids(l)).sum();
    let mut garde = vec![true; lignes.len()];
    let coupee = (total > max).then(|| {
        let a_retirer = total - max + poids(&ligne_coupee(u64::MAX, u64::MAX));
        retirer(&lignes, &mut garde, a_retirer)
    });
    let mut tout = String::new();
    for (i, (l, _)) in lignes.iter().enumerate().filter(|(i, _)| garde[*i]) {
        tout.push_str(l);
        tout.push('\n');
        if let (0, Some(c)) = (i, &coupee) {
            tout.push_str(c);
            tout.push('\n');
        }
    }
    Some(tout)
}

/// **Retire au moins `a_retirer` octets**, les plus anciennes mesures d'abord ; puis, si elles
/// ne suffisaient pas, tout ce qui suit le début. Rend la ligne qui le dit.
fn retirer(lignes: &[(&str, String)], garde: &mut [bool], mut a_retirer: usize) -> String {
    let (mut retirees, mut jusqu_a) = (0u64, 0u64);
    for mesures_seules in [true, false] {
        for (i, (l, t)) in lignes.iter().enumerate().skip(1) {
            let mesure = MESURES.contains(&t.as_str());
            if a_retirer == 0 || !garde[i] || (mesures_seules && !mesure) {
                continue;
            }
            garde[i] = false;
            a_retirer = a_retirer.saturating_sub(l.len() + 1);
            retirees += 1;
            jusqu_a = jusqu_a.max(instant_de(l));
        }
    }
    ligne_coupee(jusqu_a, retirees)
}

fn instant_de(ligne: &str) -> u64 {
    glucose_core::persist::tauri::json::lire(ligne)
        .ok()
        .and_then(|v| v.entier("instant_ms"))
        .and_then(|n| u64::try_from(n).ok())
        .unwrap_or(0)
}

/// **L'identifiant d'une session** pour le serveur : rien de la session ne part (sa clé porte
/// l'heure et le numéro du processus) — son empreinte avec l'installation, 32 chiffres.
pub fn identifiant_de_session(installation: &str, cle: &str) -> String {
    let empreinte = glucose_core::hash::sha256(format!("{installation}/{cle}").as_bytes());
    glucose_core::hash::hex_of(&empreinte)[..32].to_string()
}

/// **Le corps envoyé** : l'installation, la session, et ses lignes — **seulement** celles qui
/// sont du JSON entier, tenues dans le poids du serveur ([`a_envoyer_dans`]). `None` si rien
/// ne commence par la ligne de début.
pub fn corps(installation: &str, cle: &str, texte: &str) -> Option<String> {
    let tout = a_envoyer_dans(texte, POIDS_MAX)?;
    Some(format!(
        "{{\"installation\":\"{installation}\",\"session\":\"{}\",\"lignes\":\"{}\"}}",
        identifiant_de_session(installation, cle),
        crate::boite_noire::enregistrement::echapper(&tout)
    ))
}

/// **Envoie, sur un fil à part, ce qui attend.**
pub fn envoyer_en_fond(
    adresse: String,
    dossier: PathBuf,
    installation: String,
    courante: Option<PathBuf>,
) {
    let _ = std::thread::Builder::new()
        .name("telemetrie".into())
        .spawn(move || envoyer_tout(&adresse, &dossier, &installation, courante.as_deref()));
}

pub(super) fn envoyer_tout(
    adresse: &str,
    dossier: &Path,
    installation: &str,
    courante: Option<&Path>,
) {
    let journal = dossier.join(crate::boite_noire::DOSSIER);
    let sessions = sessions_du_journal(&journal);
    let courante = courante
        .and_then(Path::file_name)
        .map(|n| n.to_string_lossy().into_owned());
    let envoyees = elaguer(dossier, &sessions);
    let url = format!("{adresse}/v1/sessions");
    let lourde = |(nom, _): &Session| {
        std::fs::read_to_string(journal.join(nom)).is_ok_and(|t| poids_entier(&t) > POIDS_MAX)
    };
    let version = env!("CARGO_PKG_VERSION");
    for (nom, cle) in a_envoyer(&sessions, (courante.as_deref(), version), &envoyees, lourde) {
        let Ok(texte) = std::fs::read_to_string(journal.join(&nom)) else {
            continue;
        };
        let reponse = match corps(installation, &cle, &texte) {
            Some(c) => crate::plateforme::envoyer(&url, Verbe::Deposer, c.as_bytes()),
            None => Ok(0),
        };
        match reponse {
            Ok(200 | 201) => retenir(dossier, &cle, "partie"),
            Ok(code) => {
                println!("[Glucose] journal technique : {nom} refuse ({code})");
                retenir(dossier, &cle, &format!("refusee {version}"));
            }
            // Le réseau manque : la suite attendra le lancement suivant.
            Err(e) => {
                println!("[Glucose] journal technique : envoi remis ({e})");
                return;
            }
        }
    }
}

/// **Les sessions du journal** : chaque fichier de session, et sa clé — son début, lu dans sa
/// première ligne, et son processus. Un fichier sans début lisible n'a rien à envoyer.
fn sessions_du_journal(journal: &Path) -> Vec<Session> {
    use crate::boite_noire::fichiers::{cle, debut_du_fichier, processus_de};
    std::fs::read_dir(journal)
        .map(|d| {
            d.flatten()
                .filter_map(|e| {
                    let nom = e.file_name().to_string_lossy().into_owned();
                    let processus = processus_de(&nom)?;
                    let debut = debut_du_fichier(&e.path())?;
                    Some((nom, cle(debut, processus)))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Retient qu'une session est partie (ou refusée) : elle ne repartira plus — par sa clé, et le
/// verdict.
fn retenir(dossier: &Path, nom: &str, comment: &str) {
    use std::io::Write;
    let ecrit = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dossier.join(ENVOYEES))
        .and_then(|mut f| writeln!(f, "{nom} {comment}"));
    if ecrit.is_ok() {
        println!("[Glucose] journal technique : {nom} {comment}");
    }
}

/// **Efface, sur un fil à part, tout ce que cette installation a envoyé.**
pub fn effacer_en_fond(adresse: String, installation: String) {
    let _ = std::thread::Builder::new()
        .name("telemetrie".into())
        .spawn(move || {
            let url = format!("{adresse}/v1/installations/{installation}");
            match crate::plateforme::envoyer(&url, Verbe::Effacer, &[]) {
                Ok(200) => println!("[Glucose] journal technique : ce qui est parti est efface"),
                autre => println!("[Glucose] journal technique : effacement rate ({autre:?})"),
            }
        });
}

/// **La liste de ce qui est parti, sans ce que le journal a déjà effacé** : elle ne grandit
/// pas plus que le dossier. Elle ne s'oublie jamais autrement — une session partie sous un
/// ancien identifiant ne doit pas repartir sous le nouveau. Elle retient des clés, qui sont
/// les anciens noms des fichiers : celle d'avant le renommage vaut telle quelle (fiche 58).
fn elaguer(dossier: &Path, sessions: &[Session]) -> String {
    let chemin = dossier.join(ENVOYEES);
    let envoyees = std::fs::read_to_string(&chemin).unwrap_or_default();
    let gardees: String = envoyees
        .lines()
        .filter(|l| {
            let cle = l.split_whitespace().next().unwrap_or_default();
            sessions.iter().any(|(_, c)| c == cle)
        })
        .map(|l| format!("{}\n", l.trim()))
        .collect();
    if gardees != envoyees {
        let _ = std::fs::write(&chemin, &gardees);
    }
    gardees
}
