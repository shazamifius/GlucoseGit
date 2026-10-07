//! **Ce que les autres applications partagent vers Glucose** (PARTAGE-1, fiche 56).
//!
//! # Le chemin
//!
//! Sous Android, « Partager » depuis la galerie, un navigateur, Pinterest ou Instagram lance
//! Glucose avec une intention : des **fichiers** (`EXTRA_STREAM`, des adresses `content://`
//! qu'on n'ouvre que par le système) ou un **texte** (`EXTRA_TEXT`, le plus souvent le lien
//! d'une épingle). L'activité Java ne fait que ce que seul Java peut faire — ouvrir ces
//! fichiers — et confie à Rust un descripteur par fichier ; tout le reste est ici.
//!
//! Un partage devient une [`Moisson`], et suit **le chemin d'un dépôt** : des images posées en
//! un geste, ou des adresses que le rapatriement va chercher ([`super::rapatrier`]) — l'image
//! entière d'une épingle, exactement comme un glisser depuis Pinterest sur un bureau.
//!
//! # La boîte aux lettres
//!
//! Le partage arrive sur un fil de Java, **quand il veut** : avant même que la fenêtre existe,
//! quand le partage lance Glucose. Il attend alors dans la boîte, et le pont de la fenêtre le
//! relève en se branchant. Une fois branché, il part aussitôt, et réveille la boucle.
//!
//! # Ce qui décide est pur
//!
//! [`lire`], [`adresses`] et [`router`] ne touchent ni Android ni le réseau : ils s'éprouvent
//! sur n'importe quelle machine.

use super::moisson::{Depot, Moisson, Recu, OCTETS_MAX};
use super::{rapatrier, sources, Reveil};
use std::io::{Read, Seek, SeekFrom};
use std::sync::mpsc::Sender;
use std::sync::{Mutex, PoisonError};

/// Le nom d'un fichier partagé : le système en donne un qu'on ne lit pas, parce que c'est
/// l'en-tête des octets qui dit ce qu'ils sont, jamais un nom.
const NOM: &str = "partage";

/// Ce que dit un partage qui n'apporte ni image ni adresse — un texte nu.
pub const RIEN: &str = "Glucose reçoit les images et les liens : ce partage n'en porte aucun";

/// **Un partage**, tel que la frontière le confie.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Partage {
    /// Les fichiers lus, dans l'ordre du partage.
    pub recus: Vec<Recu>,
    /// Les fichiers que le système n'a pas laissé ouvrir, ou qui dépassaient la borne : ils se
    /// comptent dans le compte-rendu, au lieu de disparaître.
    pub illisibles: usize,
    /// Le texte partagé, s'il y en a un.
    pub texte: Option<String>,
}

/// **Lit un fichier partagé** : à partir de `debut`, `longueur` octets quand le système les
/// dit, jamais plus de [`OCTETS_MAX`]. Rien si la lecture échoue, si le fichier est vide ou
/// trop gros.
///
/// Le début et la longueur sont ceux de l'`AssetFileDescriptor` d'Android : un fournisseur
/// peut ne donner qu'un **morceau** d'un fichier, et lire du début lirait autre chose.
pub fn lire(mut fichier: impl Read + Seek, debut: u64, longueur: Option<u64>) -> Option<Recu> {
    if debut > 0 {
        fichier.seek(SeekFrom::Start(debut)).ok()?;
    }
    let borne = longueur.unwrap_or(u64::MAX).min(OCTETS_MAX as u64 + 1);
    let mut octets = Vec::new();
    fichier.take(borne).read_to_end(&mut octets).ok()?;
    if octets.len() > OCTETS_MAX {
        return None;
    }
    Recu::nouveau(NOM, octets)
}

/// **Les adresses web d'un texte partagé**, dans l'ordre, sans doublon.
///
/// Pinterest partage `https://pin.it/…` seul, d'autres l'entourent d'une phrase ou de
/// ponctuation : un mot est une adresse si, débarrassé de ce qui l'entoure, [`sources::decouper`]
/// l'accepte — `http` ou `https`, rien d'autre.
pub fn adresses(texte: &str) -> Vec<String> {
    let mut trouvees: Vec<String> = Vec::new();
    for mot in texte.split_whitespace() {
        let mot = mot.trim_matches(|c: char| "<>()[]{}\"'«»,;!?".contains(c));
        let mot = mot.trim_end_matches('.');
        if sources::decouper(mot).is_some() && !trouvees.iter().any(|a| a == mot) {
            trouvees.push(mot.to_string());
        }
    }
    trouvees
}

/// **Où va un partage.**
#[derive(Debug, PartialEq)]
pub enum Route {
    /// Ce qu'il porte se pose tel quel.
    Poser(Moisson),
    /// Ses adresses partent chercher leur image ; la moisson est le repli, les liens.
    Rapatrier(Vec<String>, Moisson),
}

/// **Décide d'un partage** : des images se posent — le texte qui les accompagne n'est qu'une
/// légende ; sans image, ses adresses vont chercher la leur ; sans rien, il le dit.
pub fn router(partage: Partage) -> Route {
    let Partage {
        recus,
        illisibles,
        texte,
    } = partage;
    let liens = texte.as_deref().map(adresses).unwrap_or_default();
    if !recus.is_empty() || liens.is_empty() {
        let rien = recus.is_empty() && illisibles == 0;
        return Route::Poser(Moisson {
            recus,
            illisibles,
            echec: rien.then(|| RIEN.to_string()),
            ..Moisson::default()
        });
    }
    let repli = Moisson {
        liens: liens.clone(),
        illisibles,
        ..Moisson::default()
    };
    Route::Rapatrier(liens, repli)
}

/// La boîte aux lettres : les partages arrivés avant la fenêtre, et le pont de celle-ci.
struct Boite {
    attente: Vec<Partage>,
    pont: Option<(Sender<Depot>, Reveil)>,
}

static BOITE: Mutex<Boite> = Mutex::new(Boite {
    attente: Vec::new(),
    pont: None,
});

/// **Reçoit un partage**, de n'importe quel fil : il part tout de suite si la fenêtre est
/// là, et l'attend sinon.
pub fn recevoir(partage: Partage) {
    let mut boite = BOITE.lock().unwrap_or_else(PoisonError::into_inner);
    match &boite.pont {
        Some(pont) => livrer(partage, pont.clone()),
        None => boite.attente.push(partage),
    }
}

/// **Le pont de la fenêtre se branche** : ce qui attendait part, et ce qui viendra suivra. Il
/// se débranche quand la fenêtre s'en va ([`Branchement`]) — sous Android, une activité
/// fermée puis rouverte par un partage en fait naître une autre, et le partage doit
/// l'attendre, pas partir vers la précédente.
#[cfg(any(target_os = "android", test))]
pub(super) fn brancher(vers: Sender<Depot>, reveil: Reveil) -> Branchement {
    let mut boite = BOITE.lock().unwrap_or_else(PoisonError::into_inner);
    let pont = (vers, reveil);
    for partage in std::mem::take(&mut boite.attente) {
        livrer(partage, pont.clone());
    }
    boite.pont = Some(pont);
    Branchement
}

/// **Tant qu'il vit, le pont est branché.** Il vit avec le pont de la fenêtre.
pub struct Branchement;

impl Drop for Branchement {
    fn drop(&mut self) {
        BOITE.lock().unwrap_or_else(PoisonError::into_inner).pont = None;
    }
}

/// **Ce qui ouvre le sélecteur de photos du système**, quand la plateforme en donne un
/// (fiche 56) : sous Android, l'activité Java, que `glucose-android` branche au lancement.
type Selecteur = Box<dyn Fn() + Send>;

static SELECTEUR: Mutex<Option<Selecteur>> = Mutex::new(None);

/// **Branche le sélecteur de photos** de cette plateforme ; le dernier branché sert.
pub fn installer_le_selecteur(ouvrir: Selecteur) {
    *SELECTEUR.lock().unwrap_or_else(PoisonError::into_inner) = Some(ouvrir);
}

/// **Ouvre le sélecteur de photos** : ce qu'on y choisit revient comme un partage. Rend `false`
/// quand aucun n'est branché.
pub fn choisir_des_images() -> bool {
    let selecteur = SELECTEUR.lock().unwrap_or_else(PoisonError::into_inner);
    selecteur.as_ref().map(|ouvrir| ouvrir()).is_some()
}

/// Fait partir un partage par ce pont, et réveille la boucle : elle dort peut-être.
fn livrer(partage: Partage, (vers, reveil): (Sender<Depot>, Reveil)) {
    match router(partage) {
        Route::Poser(moisson) => {
            if vers
                .send(Depot::Pose {
                    numero: None,
                    moisson,
                })
                .is_err()
            {
                return;
            }
        }
        // L'annonce part avant le fil du téléchargement : le réveil la fait paraître.
        Route::Rapatrier(liens, repli) => {
            rapatrier::rapatrier(liens, repli, (vers, reveil.clone()));
        }
    }
    reveil();
}

#[cfg(test)]
mod tests;
