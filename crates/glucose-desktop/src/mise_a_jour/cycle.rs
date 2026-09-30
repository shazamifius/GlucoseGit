//! **Le cycle d'une mise à jour** : chercher, préparer, poser, relancer (fiche 48).
//!
//! Le cœur ([`super`]) lit le fichier des versions et vérifie une signature, sans réseau. Ici,
//! ce qui touche au réseau et au système — exactement ce que fait le programme de mise à jour
//! de Glucose Tauri, lu dans sa source (`tauri-plugin-updater` 2.10.1) :
//!
//! 1. lire `latest.json` — **le même fichier que Glucose Tauri** : la bascule est sans couture ;
//! 2. télécharger l'installeur de la version proposée, et **vérifier sa signature avant de
//!    l'écrire** : rien de non signé ne touche le disque ;
//! 3. le poser selon la forme de l'installation ([`Installation`]) : sous Linux tout de suite,
//!    sous Windows quand Glucose se ferme ;
//! 4. fermer Glucose, son document écrit, et relancer — l'installeur avec `/P /R /UPDATE`, ou
//!    Glucose lui-même, déjà remplacé.
//!
//! # L'adresse et la clé se choisissent à la compilation
//!
//! Une construction ordinaire lit le fichier de ce dépôt et vérifie par la clé de Glucose. Une
//! construction d'**épreuve** — la vérification « la version N devient N+1 toute seule » sur les
//! machines de GitHub — y met un serveur local et une clé d'essai, par `GLUCOSE_MISE_A_JOUR` et
//! `GLUCOSE_CLE_PUBLIQUE`. Rien ne se change à l'exécution : un programme installé ne se laisse
//! pas convaincre, par son environnement, d'accepter une autre clé.

use super::installation::Installation;
use super::version::Version;
use super::{proposition, verifier, Proposition};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

/// **Où Glucose lit ses versions** : la release « latest » de ce dépôt — celle que Glucose Tauri
/// lit.
pub const ADRESSE_DE_GLUCOSE: &str =
    "https://github.com/shazamifius/GlucoseGit/releases/latest/download/latest.json";

/// L'adresse que ce programme lit : celle de Glucose, sauf dans une construction d'épreuve.
pub const ADRESSE: &str = match option_env!("GLUCOSE_MISE_A_JOUR") {
    Some(a) => a,
    None => ADRESSE_DE_GLUCOSE,
};

/// La clé qui vérifie ce qu'on installe : celle de Glucose, sauf dans une construction d'épreuve.
pub const CLE: &str = match option_env!("GLUCOSE_CLE_PUBLIQUE") {
    Some(c) => c,
    None => super::CLE_PUBLIQUE,
};

/// Ce qu'on accepte d'un seul téléchargement : la borne des dépôts, qu'un installeur (une
/// dizaine de mégaoctets) ne frôle pas, et qu'une réponse sans fin ne dépasse pas.
const OCTETS_MAX: usize = crate::plateforme::moisson::OCTETS_MAX;

/// **Ce que le fichier des versions propose à ce programme**, installé ainsi, ou rien.
pub fn chercher(
    adresse: &str,
    courante: &Version,
    installation: &Installation,
) -> Result<Option<Proposition>, String> {
    let octets = crate::plateforme::telecharger(adresse, OCTETS_MAX)?;
    let texte = String::from_utf8(octets)
        .map_err(|_| "le fichier des versions n'est pas du texte".to_string())?;
    let cles = installation.cles(&super::plateforme());
    proposition(&texte, courante, &cles).map_err(|r| r.to_string())
}

/// **Télécharge l'installeur, le vérifie, puis l'écrit** dans `dossier` : rien ne touche le
/// disque avant que sa signature soit vérifiée. Rend son chemin.
pub fn preparer(
    p: &Proposition,
    cle: &str,
    dossier: &Path,
    installation: &Installation,
) -> Result<PathBuf, String> {
    let octets = crate::plateforme::telecharger(&p.url, OCTETS_MAX)?;
    verifier(&octets, &p.signature, cle).map_err(|r| r.to_string())?;
    std::fs::create_dir_all(dossier).map_err(|e| e.to_string())?;
    let nom = format!(
        "Glucose_{}_installeur{}",
        p.version,
        installation.extension()
    );
    let chemin = dossier.join(nom);
    crate::persist::atomic::ecrire_d_un_bloc(&chemin, &octets).map_err(|e| e.to_string())?;
    Ok(chemin)
}

/// **Retire les installeurs qui ont fait leur travail** : ceux d'une version que ce programme
/// a déjà atteinte. Un installeur d'une version plus récente — préparé, pas encore lancé —
/// reste.
pub fn ranger_les_anciens(dossier: &Path, courante: &Version) {
    for entree in std::fs::read_dir(dossier).into_iter().flatten().flatten() {
        let nom = entree.file_name().to_string_lossy().into_owned();
        let version = nom
            .strip_prefix("Glucose_")
            .and_then(|r| r.split_once("_installeur"))
            .and_then(|(v, _)| Version::lire(v));
        if version.is_some_and(|v| v <= *courante) {
            let _ = std::fs::remove_file(entree.path());
        }
    }
}

/// **Lance un programme**, et rend la main — l'installeur, ou Glucose remplacé : l'application
/// doit se fermer ensuite. L'installeur n'a pas besoin qu'on lui dise qui attendre : Windows lui
/// dit quels processus tiennent `glucose.exe` (`outils/installeur/attendre.nsh`).
pub fn lancer(programme: &Path, arguments: &[&str]) -> Result<(), String> {
    std::process::Command::new(programme)
        .args(arguments)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("{} ne démarre pas : {e}", programme.display()))
}

/// La version de ce programme.
pub fn courante() -> Version {
    Version::courante()
}

/// **Ce que la ligne de commande demande**, sans fenêtre — les épreuves de GitHub : `--version`
/// dit la version ; `--mettre-a-jour` cherche, prépare et pose **sans question et sans
/// relance** (sous Windows, l'installeur en silence : `/S /UPDATE`). Rend le code de sortie, ou
/// rien pour un lancement ordinaire.
pub fn commande(argument: Option<&str>, installeurs: &Path) -> Option<i32> {
    match argument? {
        "--version" => {
            println!("Glucose {}", courante());
            Some(0)
        }
        "--mettre-a-jour" => match mettre_a_jour_en_silence(installeurs) {
            Ok(v) => {
                println!("Glucose {v} : la mise à jour est posée");
                Some(0)
            }
            Err(e) => {
                println!("pas de mise à jour : {e}");
                Some(1)
            }
        },
        _ => None,
    }
}

fn mettre_a_jour_en_silence(installeurs: &Path) -> Result<Version, String> {
    let installation = Installation::de_ce_programme()
        .ok_or("ce Glucose n'est pas installé : il ne se remplace pas")?;
    let p = chercher(ADRESSE, &courante(), &installation)?.ok_or("aucune version plus récente")?;
    let installeur = preparer(&p, CLE, installeurs, &installation)?;
    installation.poser(&installeur)?;
    if installation == Installation::Nsis {
        lancer(&installeur, &["/S", "/UPDATE"])?;
    }
    Ok(p.version)
}

/// Ce que la veille fait parvenir à la boucle.
#[derive(Debug)]
pub enum Nouvelle {
    /// Une version plus récente est proposée.
    Proposee(Proposition),
    /// Son installeur est vérifié et posé : il ne reste qu'à fermer, puis à relancer.
    Prete(PathBuf),
    /// Ce qui a échoué, en une phrase.
    Echec(String),
}

/// **La veille des mises à jour** : un fil cherche au lancement, un autre prépare quand
/// l'utilisateur a dit oui ; la boucle récolte ce qu'ils disent, sans jamais les attendre.
pub struct Veille {
    envoyer: Sender<Nouvelle>,
    recevoir: Receiver<Nouvelle>,
    reveil: crate::plateforme::Reveil,
    /// Où les installeurs se posent.
    dossier: PathBuf,
    /// Comment ce programme est installé.
    installation: Installation,
}

impl Veille {
    /// Commence à chercher, sur un fil à part : le lancement ne l'attend pas.
    pub fn commencer(
        dossier: PathBuf,
        reveil: crate::plateforme::Reveil,
        installation: Installation,
    ) -> Self {
        let (envoyer, recevoir) = channel();
        let veille = Self {
            envoyer,
            recevoir,
            reveil,
            dossier,
            installation,
        };
        let installation = veille.installation.clone();
        veille.en_fond(
            move || match chercher(ADRESSE, &Version::courante(), &installation) {
                Ok(Some(p)) => Some(Nouvelle::Proposee(p)),
                // Rien de neuf, ou pas de réseau : rien à dire — ce n'est pas une panne.
                Ok(None) | Err(_) => None,
            },
        );
        veille
    }

    /// Prépare cette version sur un fil à part : télécharger, vérifier, poser.
    pub fn preparer(&self, p: Proposition) {
        let (dossier, installation) = (self.dossier.clone(), self.installation.clone());
        self.en_fond(move || {
            let pret = preparer(&p, CLE, &dossier, &installation)
                .and_then(|chemin| installation.poser(&chemin).map(|()| chemin));
            Some(match pret {
                Ok(chemin) => Nouvelle::Prete(chemin),
                Err(e) => Nouvelle::Echec(e),
            })
        });
    }

    /// Comment ce programme est installé.
    pub fn installation(&self) -> &Installation {
        &self.installation
    }

    fn en_fond(&self, travail: impl FnOnce() -> Option<Nouvelle> + Send + 'static) {
        let (vers, reveil) = (self.envoyer.clone(), std::sync::Arc::clone(&self.reveil));
        std::thread::spawn(move || {
            if let Some(n) = travail() {
                if vers.send(n).is_ok() {
                    reveil();
                }
            }
        });
    }

    /// Tout ce qui est arrivé, sans attendre.
    pub fn recolter(&self) -> Vec<Nouvelle> {
        self.recevoir.try_iter().collect()
    }
}

#[cfg(test)]
mod tests;
