//! **Le cycle d'une mise à jour** : chercher, préparer, lancer (fiche 48).
//!
//! Le cœur ([`super`]) lit le fichier des versions et vérifie une signature, sans réseau. Ici,
//! ce qui touche au réseau et au système — exactement ce que fait le programme de mise à jour
//! de Glucose Tauri, lu dans sa source (`tauri-plugin-updater` 2.10.1) :
//!
//! 1. lire `latest.json` — **le même fichier que Glucose Tauri** : la bascule est sans couture ;
//! 2. télécharger l'installeur de la version proposée, et **vérifier sa signature avant de
//!    l'écrire** : rien de non signé ne touche le disque ;
//! 3. le lancer avec `/P /R /UPDATE` — sans question, relancer Glucose ensuite, c'est une mise
//!    à jour —, puis rendre la main : l'application se ferme, son document écrit.
//!
//! # L'adresse et la clé se choisissent à la compilation
//!
//! Une construction ordinaire lit le fichier de ce dépôt et vérifie par la clé de Glucose. Une
//! construction d'**épreuve** — la vérification « la version N devient N+1 toute seule » sur les
//! machines de GitHub — y met un serveur local et une clé d'essai, par `GLUCOSE_MISE_A_JOUR` et
//! `GLUCOSE_CLE_PUBLIQUE`. Rien ne se change à l'exécution : un programme installé ne se laisse
//! pas convaincre, par son environnement, d'accepter une autre clé.

use super::version::Version;
use super::{proposition, verifier, Proposition};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

/// Où Glucose lit ses versions : la release « latest » de ce dépôt — celle que Glucose Tauri lit.
pub const ADRESSE: &str = match option_env!("GLUCOSE_MISE_A_JOUR") {
    Some(a) => a,
    None => "https://github.com/shazamifius/GlucoseGit/releases/latest/download/latest.json",
};

/// La clé qui vérifie ce qu'on installe : celle de Glucose, sauf dans une construction d'épreuve.
pub const CLE: &str = match option_env!("GLUCOSE_CLE_PUBLIQUE") {
    Some(c) => c,
    None => super::CLE_PUBLIQUE,
};

/// Ce qu'on accepte d'un seul téléchargement : la borne des dépôts, qu'un installeur (une
/// dizaine de mégaoctets) ne frôle pas, et qu'une réponse sans fin ne dépasse pas.
const OCTETS_MAX: usize = crate::plateforme::moisson::OCTETS_MAX;

/// **Ce que le fichier des versions propose à ce programme**, ou rien.
pub fn chercher(adresse: &str, courante: &Version) -> Result<Option<Proposition>, String> {
    let octets = crate::plateforme::telecharger(adresse, OCTETS_MAX)?;
    let texte = String::from_utf8(octets)
        .map_err(|_| "le fichier des versions n'est pas du texte".to_string())?;
    proposition(&texte, courante, &super::plateforme()).map_err(|r| r.to_string())
}

/// **Télécharge l'installeur, le vérifie, puis le pose** dans `dossier` : rien ne touche le
/// disque avant que sa signature soit vérifiée. Rend son chemin.
pub fn preparer(p: &Proposition, cle: &str, dossier: &Path) -> Result<PathBuf, String> {
    let octets = crate::plateforme::telecharger(&p.url, OCTETS_MAX)?;
    verifier(&octets, &p.signature, cle).map_err(|r| r.to_string())?;
    std::fs::create_dir_all(dossier).map_err(|e| e.to_string())?;
    let chemin = dossier.join(nom_de_l_installeur(&p.version));
    crate::persist::atomic::ecrire_d_un_bloc(&chemin, &octets).map_err(|e| e.to_string())?;
    Ok(chemin)
}

/// Le nom d'un installeur posé : sa version s'y lit, pour qu'on sache le ranger ensuite.
fn nom_de_l_installeur(version: &Version) -> String {
    format!(
        "Glucose_{version}_installeur{}",
        std::env::consts::EXE_SUFFIX
    )
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

/// **Lance l'installeur**, et rend la main : l'application doit se fermer pour qu'il la
/// remplace. Il n'a pas besoin qu'on lui dise qui attendre : Windows lui dit quels processus
/// tiennent `glucose.exe`, et il attend exactement leur fin (`outils/installeur/attendre.nsh`).
pub fn lancer(installeur: &Path, arguments: &[&str]) -> Result<(), String> {
    std::process::Command::new(installeur)
        .args(arguments)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("l'installeur ne démarre pas : {e}"))
}

/// La version de ce programme.
pub fn courante() -> Version {
    Version::courante()
}

/// **Ce que la ligne de commande demande**, sans fenêtre — les épreuves de GitHub : `--version`
/// dit la version ; `--mettre-a-jour` cherche, prépare et lance l'installeur **sans question
/// et sans relance** (`/S /UPDATE`). Rend le code de sortie, ou rien pour un lancement
/// ordinaire.
pub fn commande(argument: Option<&str>, installeurs: &Path) -> Option<i32> {
    match argument? {
        "--version" => {
            println!("Glucose {}", courante());
            Some(0)
        }
        "--mettre-a-jour" => {
            let lance = chercher(ADRESSE, &courante()).and_then(|p| {
                let p = p.ok_or("aucune version plus récente")?;
                let installeur = preparer(&p, CLE, installeurs)?;
                lancer(&installeur, &["/S", "/UPDATE"])?;
                Ok(p.version)
            });
            match lance {
                Ok(v) => {
                    println!("Glucose {v} : l'installeur est lancé");
                    Some(0)
                }
                Err(e) => {
                    println!("pas de mise à jour : {e}");
                    Some(1)
                }
            }
        }
        _ => None,
    }
}

/// Ce que la veille fait parvenir à la boucle.
#[derive(Debug)]
pub enum Nouvelle {
    /// Une version plus récente est proposée.
    Proposee(Proposition),
    /// Son installeur est vérifié et posé : il ne reste qu'à fermer, puis à le lancer.
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
}

impl Veille {
    /// Commence à chercher, sur un fil à part : le lancement ne l'attend pas.
    pub fn commencer(dossier: PathBuf, reveil: crate::plateforme::Reveil) -> Self {
        let (envoyer, recevoir) = channel();
        let veille = Self {
            envoyer,
            recevoir,
            reveil,
            dossier,
        };
        veille.en_fond(|| match chercher(ADRESSE, &Version::courante()) {
            Ok(Some(p)) => Some(Nouvelle::Proposee(p)),
            // Rien de neuf, ou pas de réseau : rien à dire — ce n'est pas une panne.
            Ok(None) | Err(_) => None,
        });
        veille
    }

    /// Prépare cette version sur un fil à part : télécharger, vérifier, poser.
    pub fn preparer(&self, p: Proposition) {
        let dossier = self.dossier.clone();
        self.en_fond(move || {
            Some(match preparer(&p, CLE, &dossier) {
                Ok(chemin) => Nouvelle::Prete(chemin),
                Err(e) => Nouvelle::Echec(e),
            })
        });
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
