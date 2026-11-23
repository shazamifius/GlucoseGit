//! **RELAIS-2 — le relais survit à Glucose** (ACCÈS-H24, fiche 131).
//!
//! Le relais recopiait les octets d'un flux à l'autre : quand Glucose se fermait — un service qui
//! le relance, une mise à jour, un plantage —, la porte se fermait, le relais mourait avec elle,
//! et c'est le client qui voyait la panne (« Connection closed ») à son appel suivant. Vu deux fois
//! le 23/11 à 0 h 05, son Glucose relancé sous les sessions du relais.
//!
//! Le relais relaie désormais **ligne par ligne**, et sait ce qu'il doit encore au client :
//!
//! * une demande envoyée à une porte qui se ferme avant d'y répondre reçoit **une erreur qui le
//!   dit** — jamais un silence qu'un client attendrait pour toujours ; son effet est incertain, et
//!   le message le dit aussi ;
//! * la demande suivante **se rebranche** : la poignée de main est rejouée devant la nouvelle
//!   porte, et ses réponses se taisent, comme au premier branchement (RELAIS-1).
//!
//! **Jamais deux Glucose** : au rebranchement, Glucose était là un instant plus tôt — un service,
//! une mise à jour ou l'utilisateur le relancent peut-être déjà. On lui laisse le temps d'un
//! lancement ([`super::LANCEMENT_MAX`]) avant de l'ouvrir soi-même : l'ouvrir tout de suite en
//! ferait naître un second, à côté de celui que son service relance (son Glucose est un service
//! depuis le 23/11, qui revient en dix secondes).

use super::super::json::Json;
use super::super::porte;
use super::super::protocole;
use std::io::{BufReader, Write};
use std::sync::{Arc, Mutex};

/// La sortie vers le client, partagée par le fil du relais et celui de chaque porte : une ligne
/// s'y écrit d'un coup, jamais mêlée à une autre.
pub type Sortie = Arc<Mutex<Box<dyn Write + Send>>>;

/// Écrit une ligne au client ; `false` s'il est parti.
pub fn dire(sortie: &Sortie, ligne: &str) -> bool {
    let Ok(mut s) = sortie.lock() else {
        return false;
    };
    writeln!(s, "{ligne}").and_then(|()| s.flush()).is_ok()
}

/// L'identifiant d'une ligne qui attend une réponse : une requête.
fn identifiant(ligne: &str) -> Option<Json> {
    Json::lire(ligne).ok()?.get("id").cloned()
}

/// Ce qu'une porte doit encore au client.
#[derive(Default)]
struct Dette {
    /// La porte s'est fermée : plus rien n'y part.
    fermee: bool,
    /// Les requêtes sans réponse encore — et si leur envoi est confirmé (`true`), ou encore en vol :
    /// la fermeture ne répond qu'aux premières ; des autres, c'est l'envoyeur qui décide. Chaque
    /// requête reçoit ainsi **une** réponse, jamais deux, jamais aucune.
    attendues: Vec<(Json, bool)>,
}

impl Dette {
    /// Retire cette requête ; `Some(partie)` si elle y était.
    fn retirer(&mut self, id: &Json) -> Option<bool> {
        let rang = self.attendues.iter().position(|(x, _)| x == id)?;
        Some(self.attendues.remove(rang).1)
    }
}

/// Ce qu'est devenue une ligne envoyée.
#[derive(Debug, PartialEq, Eq)]
pub enum Envoi {
    /// Partie à la porte.
    Partie,
    /// La porte est fermée et la ligne n'y est pas entrée : à renvoyer à la suivante.
    Coupee,
    /// La porte s'est fermée après l'avoir reçue : le client a reçu l'erreur qui le dit.
    Repondue,
}

/// **Une connexion à la porte**, et ce qu'elle doit au client.
pub struct Lien {
    aller: porte::Flux,
    dette: Arc<Mutex<Dette>>,
    sortie: Sortie,
}

impl Lien {
    /// Se branche sur ce flux : la poignée de main rejouée, ses réponses tues.
    pub fn sur(flux: porte::Flux, prelude: &[String], sortie: &Sortie) -> Result<Lien, String> {
        let retour = flux.try_clone().map_err(|e| e.to_string())?;
        let dette = Arc::new(Mutex::new(Dette::default()));
        let a_taire = prelude.iter().filter(|l| identifiant(l).is_some()).count();
        let (d, s) = (dette.clone(), sortie.clone());
        std::thread::Builder::new()
            .name("glucose-relais".into())
            .spawn(move || ecouter(retour, a_taire, &d, &s))
            .map_err(|e| e.to_string())?;
        let mut lien = Lien {
            aller: flux,
            dette,
            sortie: sortie.clone(),
        };
        for ligne in prelude {
            writeln!(lien.aller, "{ligne}").map_err(|e| e.to_string())?;
        }
        Ok(lien)
    }

    /// **Envoie une ligne du client.** Sa requête entre dans la dette, en vol, **avant** de partir :
    /// sa réponse ne peut pas la devancer.
    pub fn envoyer(&mut self, ligne: &str) -> Envoi {
        let id = identifiant(ligne);
        {
            let Ok(mut dette) = self.dette.lock() else {
                return Envoi::Coupee;
            };
            if dette.fermee {
                return Envoi::Coupee;
            }
            dette.attendues.extend(id.clone().map(|id| (id, false)));
        }
        let parti = writeln!(self.aller, "{ligne}")
            .and_then(|()| self.aller.flush())
            .is_ok();
        let Ok(mut dette) = self.dette.lock() else {
            return Envoi::Coupee;
        };
        let Some(id) = id else {
            dette.fermee |= !parti;
            return if parti { Envoi::Partie } else { Envoi::Coupee };
        };
        match (parti, dette.fermee, dette.retirer(&id)) {
            // Déjà répondue par Glucose : rien de plus à faire.
            (_, _, None) => Envoi::Partie,
            (true, false, Some(_)) => {
                dette.attendues.push((id, true));
                Envoi::Partie
            }
            // Partie, mais la porte s'est fermée pendant ce temps : son effet est incertain.
            (true, true, Some(_)) => {
                dire(&self.sortie, &protocole::erreur_de_fermeture(&id, FERMEE));
                Envoi::Repondue
            }
            (false, _, Some(_)) => {
                dette.fermee = true;
                Envoi::Coupee
            }
        }
    }
}

/// **Le fil d'une porte** : ses réponses au client, la poignée de main rejouée tue ; puis, à sa
/// fermeture, une erreur pour chaque requête partie restée sans réponse.
fn ecouter(retour: porte::Flux, a_taire: usize, dette: &Mutex<Dette>, sortie: &Sortie) {
    let mut retour = BufReader::new(retour);
    let mut a_taire = a_taire;
    while let Ok(Some(ligne)) = porte::lire_une_ligne(&mut retour) {
        if a_taire > 0 {
            a_taire -= 1;
            continue;
        }
        if let (Some(id), Ok(mut d)) = (identifiant(&ligne), dette.lock()) {
            d.retirer(&id);
        }
        if !dire(sortie, &ligne) {
            return;
        }
    }
    let orphelines: Vec<Json> = match dette.lock() {
        Ok(mut d) => {
            d.fermee = true;
            let (parties, en_vol) = std::mem::take(&mut d.attendues)
                .into_iter()
                .partition(|(_, partie)| *partie);
            d.attendues = en_vol;
            parties.into_iter().map(|(id, _)| id).collect()
        }
        Err(_) => Vec::new(),
    };
    for id in orphelines {
        dire(sortie, &protocole::erreur_de_fermeture(&id, FERMEE));
    }
}

/// Ce que reçoit une requête que la fermeture de Glucose a emportée.
pub const FERMEE: &str = "Glucose s'est fermé pendant cet appel, sans y répondre : son effet est \
                          incertain. La demande suivante attendra qu'il revienne.";

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    struct Tampon(Arc<Mutex<Vec<u8>>>);

    impl Write for Tampon {
        fn write(&mut self, o: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("tampon").extend_from_slice(o);
            Ok(o.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// **La fermeture ne répond qu'aux requêtes parties** : celle encore en vol attend son
    /// envoyeur, qui saura si elle est entrée — sinon elle recevrait deux réponses, ou une erreur
    /// pour une demande que Glucose n'a jamais lue.
    #[test]
    fn relais_2_la_fermeture_ne_repond_qu_aux_requetes_parties() {
        let (glucose, relais) = std::os::unix::net::UnixStream::pair().expect("paire");
        drop(glucose);
        let (partie, en_vol) = (Json::Nombre(1.0), Json::Nombre(2.0));
        let dette = Mutex::new(Dette {
            fermee: false,
            attendues: vec![(partie.clone(), true), (en_vol.clone(), false)],
        });
        let tampon = Arc::new(Mutex::new(Vec::new()));
        let sortie: Sortie = Arc::new(Mutex::new(Box::new(Tampon(tampon.clone()))));
        ecouter(relais, 0, &dette, &sortie);
        let dit = String::from_utf8(tampon.lock().expect("tampon").clone()).expect("UTF-8");
        let ids: Vec<Json> = dit
            .lines()
            .filter_map(|l| Json::lire(l).ok()?.get("id").cloned())
            .collect();
        assert_eq!(
            ids,
            [partie],
            "une erreur, pour la partie seulement : {dit}"
        );
        let d = dette.lock().expect("dette");
        assert!(d.fermee);
        assert_eq!(
            d.attendues,
            [(en_vol, false)],
            "celle en vol attend son envoyeur"
        );
    }

    /// **Une ligne qui n'est pas entrée se renvoie** : la porte fermée avant que son fil ne l'ait
    /// vue, l'envoi échoue — la requête sort de la dette sans erreur au client, et repartira par la
    /// porte suivante.
    #[test]
    fn relais_2_une_ligne_qui_n_est_pas_entree_se_renvoie() {
        let (glucose, relais) = std::os::unix::net::UnixStream::pair().expect("paire");
        drop(glucose);
        let tampon = Arc::new(Mutex::new(Vec::new()));
        let mut lien = Lien {
            aller: relais,
            dette: Arc::new(Mutex::new(Dette::default())),
            sortie: Arc::new(Mutex::new(Box::new(Tampon(tampon.clone())))),
        };
        let ligne = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call"}"#;
        assert_eq!(lien.envoyer(ligne), Envoi::Coupee);
        let d = lien.dette.lock().expect("dette");
        assert!(d.fermee && d.attendues.is_empty());
        assert!(
            tampon.lock().expect("tampon").is_empty(),
            "aucune erreur : elle n'est pas perdue"
        );
    }
}
