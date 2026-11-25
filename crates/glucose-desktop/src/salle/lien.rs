//! **Le fil relié à la salle** (CLIENT-1, fiche 117) — comme celui de la porte publique
//! (`agents/publique/lien.rs`) : il tient un WebSocket vers le relais, s'y relie encore quand le
//! réseau revient (l'attente double d'une seconde à une minute), et vérifie qu'il vit (`ping`
//! toutes les 20 s, mort sans `pong` en 10).
//!
//! Il parle à la boucle par deux files. **Ce qui part** : chaque lettre, scellée en morceaux, est
//! gardée jusqu'à ce que la salle la rende numérotée — une connexion qui tombe entre l'envoi et le
//! numéro ne la perd pas, elle repart à la suivante. **Ce qui arrive** : il compte les numéros de la
//! salle — une répétition s'ignore, un trou se redemande —, réassemble, ouvre et relit chaque
//! lettre : la boucle ne reçoit que des lettres entières, déjà lues, hors de son image.

use crate::agents::json::Json;
use crate::agents::publique::lien::EtatDuLien;
use crate::agents::Boite;
use crate::plateforme::hasard;
use crate::plateforme::websocket::{self, Canal};
use glucose_core::salle::code::Code;
use glucose_core::salle::lettre::Lettre;
use glucose_core::salle::morceaux::{self, Reassemblage, MORCEAU};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

const PING: Duration = Duration::from_secs(20);
const PONG_MAX: Duration = Duration::from_secs(10);
const PAUSE_MAX: Duration = Duration::from_secs(60);

/// **Ce que la boucle demande au fil.**
pub enum Ordre {
    /// Une lettre écrite (`Lettre::ecrire`), et son nom.
    Lettre { id: String, clair: Vec<u8> },
    /// Un curseur : il ne part que si la salle est à jour, et ne se garde pas.
    Curseur(Vec<u8>),
}

/// **Ce qui passe** (B1, fiche 117) : les octets d'une photo (`photo`), ou une demande — scellés en
/// morceaux comme une lettre, mais ni numérotés ni gardés ; perdus si la salle n'est pas à jour,
/// ils se redemandent au « bonjour » suivant. La salle ne lit pas ce qu'ils portent : la sorte se
/// dit en clair, pour qu'elle refuse les octets d'un lecteur.
///
/// Ils ont **leur propre file, d'une place**, et le fil n'en fait partir qu'un par tour, après
/// les lettres et le curseur : un geste ne fait jamais la queue derrière un lot de photos, et celui
/// qui les envoie attend que la précédente soit partie — la mémoire n'en tient que trois.
pub struct Passe {
    pub id: String,
    pub clair: Vec<u8>,
    pub photo: bool,
}

/// **Ce que le fil apporte à la boucle.**
pub enum Nouvelle {
    /// Une lettre entière, lue, à son numéro dans l'ordre de la salle.
    Lettre {
        n: u64,
        id: String,
        lettre: Box<Lettre>,
    },
    Curseur(Box<Lettre>),
    /// Une lettre qui passe — une photo, une demande —, réassemblée, lue.
    Passe(Box<Lettre>),
    /// Tout ce que la salle gardait est arrivé ; ce qui suit vient en direct.
    AJour(u64),
    /// La salle a refusé (un code qui ne fait que lire, un message trop gros).
    Refus(String),
    /// Un morceau ou une lettre qui ne se lit pas.
    Illisible(String),
}

/// **Le fil en marche** : le lâcher l'arrête.
pub struct Lien {
    arret: Arc<AtomicBool>,
    ordres: mpsc::Sender<Ordre>,
    passes: mpsc::SyncSender<Passe>,
    nouvelles: mpsc::Receiver<Nouvelle>,
    etat: Arc<Mutex<EtatDuLien>>,
}

impl Drop for Lien {
    fn drop(&mut self) {
        self.arret.store(true, Ordering::Relaxed);
    }
}

impl Lien {
    pub fn envoyer(&self, ordre: Ordre) {
        let _ = self.ordres.send(ordre);
    }

    /// La file de ce qui passe, pour le fil qui attend les octets d'une photo.
    pub fn expediteur(&self) -> mpsc::SyncSender<Passe> {
        self.passes.clone()
    }

    /// Ce qui est arrivé depuis la dernière relève, dans l'ordre.
    pub fn relever(&self) -> Vec<Nouvelle> {
        self.nouvelles.try_iter().collect()
    }

    pub fn etat(&self) -> EtatDuLien {
        self.etat
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// Ce que le fil garde d'une connexion à l'autre.
struct Fil {
    adresse: String,
    code: Code,
    salle: String,
    cle: [u8; 32],
    /// Le numéro du dernier morceau reçu : tous ceux d'avant sont arrivés, dans l'ordre.
    dernier: u64,
    reassemblage: Reassemblage,
    /// Ce qui passe se réassemble à part : ses morceaux ne sont pas numérotés.
    passes: Reassemblage,
    /// Le numéro depuis lequel on a redemandé après un trou : une fois par position, pas à chaque
    /// morceau qui suit le trou.
    redemande: Option<u64>,
    /// Mes lettres scellées, pas encore revenues numérotées.
    envoyes: VecDeque<(String, Vec<(String, String)>)>,
    ordres: mpsc::Receiver<Ordre>,
    a_passer: mpsc::Receiver<Passe>,
    nouvelles: mpsc::Sender<Nouvelle>,
    boite: Boite,
}

/// **Un lien d'épreuve, relié** : ce que la salle apporte, l'épreuve l'envoie par la file rendue.
#[cfg(test)]
pub fn d_epreuve() -> (Lien, mpsc::Sender<Nouvelle>) {
    let (envoi, nouvelles) = mpsc::channel();
    let lien = Lien {
        arret: Arc::new(AtomicBool::new(false)),
        ordres: mpsc::channel().0,
        passes: mpsc::sync_channel(1).0,
        nouvelles,
        etat: Arc::new(Mutex::new(EtatDuLien::Relie)),
    };
    (lien, envoi)
}

/// **Relie ce Glucose à la salle de ce code**, depuis le numéro `depuis` (zéro : depuis la base).
pub fn lancer(racine: &str, code: Code, depuis: u64, boite: Boite) -> Lien {
    let (lien, mut fil) = relier(racine, code, depuis, boite);
    let (arret, etat) = (Arc::clone(&lien.arret), Arc::clone(&lien.etat));
    let _ = std::thread::Builder::new()
        .name("glucose-salle".into())
        .spawn(move || fil.tourner(&arret, &etat));
    lien
}

/// Le lien que garde la boucle, et le fil qui tournera de l'autre côté de ses files.
fn relier(racine: &str, code: Code, depuis: u64, boite: Boite) -> (Lien, Fil) {
    let (ordres, entree) = mpsc::channel();
    let (passes, a_passer) = mpsc::sync_channel(1);
    let (sortie, nouvelles) = mpsc::channel();
    let lien = Lien {
        arret: Arc::new(AtomicBool::new(false)),
        ordres,
        passes,
        nouvelles,
        etat: Arc::new(Mutex::new(EtatDuLien::EnRoute)),
    };
    let fil = Fil {
        adresse: super::adresse(racine, &code.salle()),
        salle: code.salle(),
        cle: code.cle(),
        code,
        dernier: depuis,
        reassemblage: Reassemblage::default(),
        passes: Reassemblage::default(),
        redemande: None,
        envoyes: VecDeque::new(),
        ordres: entree,
        a_passer,
        nouvelles: sortie,
        boite,
    };
    (lien, fil)
}

fn message(paires: Vec<(&str, Json)>) -> String {
    Json::objet(paires).ecrire()
}

fn depuis(n: u64) -> String {
    message(vec![
        ("t", Json::texte("depuis")),
        ("n", Json::Nombre(n as f64)),
    ])
}

fn geste(l: &str, o: &str) -> String {
    message(vec![
        ("t", Json::texte("geste")),
        ("l", Json::texte(l)),
        ("o", Json::texte(o)),
    ])
}

impl Fil {
    fn dire(&self, nouvelle: Nouvelle) {
        let _ = self.nouvelles.send(nouvelle);
        self.boite.reveiller();
    }

    fn dire_l_etat(&self, etat: &Mutex<EtatDuLien>, nouveau: EtatDuLien) {
        *etat.lock().unwrap_or_else(PoisonError::into_inner) = nouveau;
        self.boite.reveiller();
    }

    fn tourner(&mut self, arret: &AtomicBool, etat: &Mutex<EtatDuLien>) {
        let mut pause = Duration::from_secs(1);
        // **Depuis quand la salle est perdue** : la sortie dit la coupure, la raison du premier
        // échec d'une série, et le retour — sans quoi « déconnecté de la sync » (sa note du 24/11)
        // ne se diagnostique pas : un lien qui ne revient jamais ne s'y lisait pas.
        let mut perdue: Option<Instant> = None;
        while !arret.load(Ordering::Relaxed) {
            let autorisation = self.code.secret_d_ecriture().map(|s| format!("Bearer {s}"));
            let entetes: Vec<(&str, &str)> = autorisation
                .as_deref()
                .map(|a| ("Authorization", a))
                .into_iter()
                .collect();
            match websocket::ouvrir(&self.adresse, &entetes) {
                Ok(canal) => {
                    if let Some(depuis) = perdue.take() {
                        let s = depuis.elapsed().as_secs();
                        crate::dire!("[Glucose] canevas partagé : relié de nouveau, après {s} s");
                    }
                    self.dire_l_etat(etat, EtatDuLien::Relie);
                    pause = Duration::from_secs(1);
                    let raison = self.servir(canal, arret);
                    crate::dire!("[Glucose] canevas partagé : {raison}");
                    perdue = Some(Instant::now());
                    self.dire_l_etat(etat, EtatDuLien::Hors(raison));
                }
                Err(e) => {
                    if perdue.is_none() {
                        crate::dire!("[Glucose] canevas partagé : pas relié ({e}) ; il réessaie");
                        perdue = Some(Instant::now());
                    }
                    self.dire_l_etat(etat, EtatDuLien::Hors(e));
                }
            }
            let debut = Instant::now();
            while debut.elapsed() < pause && !arret.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(100));
            }
            pause = (pause * 2).min(PAUSE_MAX);
        }
    }

    /// **Une connexion**, jusqu'à ce qu'elle meure ou qu'on quitte ; rend pourquoi.
    fn servir(&mut self, mut canal: Box<dyn Canal>, arret: &AtomicBool) -> String {
        let mut a_jour = false;
        self.redemande = Some(self.dernier);
        if let Err(e) = canal.envoyer(&depuis(self.dernier)) {
            return e;
        }
        let mut dernier_ping = Instant::now();
        let mut pong_attendu: Option<Instant> = None;
        loop {
            if arret.load(Ordering::Relaxed) {
                return "quitté".into();
            }
            if let Err(e) = self.un_tour(canal.as_mut(), a_jour) {
                return e;
            }
            if pong_attendu.is_some_and(|t| t.elapsed() > PONG_MAX) {
                return "la salle ne répond plus".into();
            }
            if pong_attendu.is_none() && dernier_ping.elapsed() > PING {
                if let Err(e) = canal.envoyer("ping") {
                    return e;
                }
                dernier_ping = Instant::now();
                pong_attendu = Some(dernier_ping);
            }
            let recu = match canal.recevoir(Duration::from_millis(25)) {
                Ok(Some(m)) if m == "pong" => {
                    pong_attendu = None;
                    Ok(())
                }
                Ok(Some(m)) => self.recevoir(&m, canal.as_mut(), &mut a_jour),
                Ok(None) => Ok(()),
                Err(e) => Err(e),
            };
            if let Err(e) = recu {
                return e;
            }
        }
    }

    /// **Les ordres de la boucle** : chaque lettre scellée, gardée jusqu'à son retour — envoyée tout
    /// de suite si la salle est à jour, sinon à son « à jour ».
    fn faire_partir(&mut self, canal: &mut dyn Canal, a_jour: bool) -> Result<(), String> {
        let ordres: Vec<Ordre> = self.ordres.try_iter().collect();
        for ordre in ordres {
            match ordre {
                Ordre::Lettre { id, clair } => {
                    let parts = self.sceller(&id, &clair)?;
                    if a_jour {
                        for (l, o) in &parts {
                            canal.envoyer(&geste(l, o))?;
                        }
                    }
                    self.envoyes.push_back((id, parts));
                }
                Ordre::Curseur(clair) if a_jour => {
                    let o = morceaux::sceller_curseur(
                        &self.cle,
                        &self.salle,
                        &clair,
                        hasard::octets()?,
                    );
                    canal.envoyer(&message(vec![
                        ("t", Json::texte("curseur")),
                        ("o", Json::texte(o)),
                    ]))?;
                }
                Ordre::Curseur(_) => {}
            }
        }
        Ok(())
    }

    /// **Ce qui part à chaque tour** : les lettres et le curseur d'abord, puis une seule chose qui
    /// passe.
    fn un_tour(&mut self, canal: &mut dyn Canal, a_jour: bool) -> Result<(), String> {
        self.faire_partir(canal, a_jour)?;
        self.faire_passer(canal, a_jour)
    }

    /// **Ce qui passe, un par tour** — perdu si la salle n'est pas à jour.
    fn faire_passer(&mut self, canal: &mut dyn Canal, a_jour: bool) -> Result<(), String> {
        let Ok(Passe { id, clair, photo }) = self.a_passer.try_recv() else {
            return Ok(());
        };
        if !a_jour {
            return Ok(());
        }
        let sorte = if photo { "photo" } else { "demande" };
        for (l, o) in self.sceller(&id, &clair)? {
            canal.envoyer(&message(vec![
                ("t", Json::texte(sorte)),
                ("l", Json::texte(l)),
                ("o", Json::texte(o)),
            ]))?;
        }
        Ok(())
    }

    /// Une lettre en morceaux scellés, chacun sous un nonce neuf du hasard du système.
    fn sceller(&self, id: &str, clair: &[u8]) -> Result<Vec<(String, String)>, String> {
        let k = clair.len().div_ceil(MORCEAU).max(1);
        let nonces: Vec<[u8; 12]> = (0..k).map(|_| hasard::octets()).collect::<Result<_, _>>()?;
        let mut tires = nonces.into_iter();
        // Autant de nonces que de morceaux, tirés d'avance : le défaut n'est jamais pris.
        let mut nonce = || tires.next().unwrap_or_default();
        Ok(morceaux::sceller(
            &self.cle,
            &self.salle,
            id,
            clair,
            &mut nonce,
        ))
    }

    /// **Un message de la salle.**
    fn recevoir(
        &mut self,
        texte: &str,
        canal: &mut dyn Canal,
        a_jour: &mut bool,
    ) -> Result<(), String> {
        let Ok(m) = Json::lire(texte) else {
            return Ok(());
        };
        let champ = |c: &str| m.get(c).and_then(Json::as_str);
        match champ("t") {
            Some("geste") => {
                let n = m.get("n").and_then(Json::as_f64).map_or(0, |n| n as u64);
                match (champ("l"), champ("o")) {
                    (Some(l), Some(o)) => self.un_morceau(n, l, o, canal),
                    _ => Ok(()),
                }
            }
            Some("a_jour") if m.get("complet").and_then(Json::as_bool) == Some(false) => {
                canal.envoyer(&depuis(self.dernier))
            }
            Some("a_jour") => {
                *a_jour = true;
                for (l, o) in self.envoyes.iter().flat_map(|(_, parts)| parts) {
                    canal.envoyer(&geste(l, o))?;
                }
                self.dire(Nouvelle::AJour(self.dernier));
                Ok(())
            }
            Some("curseur") => {
                let lettre = champ("o")
                    .and_then(|o| morceaux::ouvrir_curseur(&self.cle, &self.salle, o))
                    .and_then(|c| Lettre::lire(&c).ok());
                if let Some(l) = lettre {
                    self.dire(Nouvelle::Curseur(Box::new(l)));
                }
                Ok(())
            }
            Some("photo" | "demande") => {
                if let (Some(l), Some(o)) = (champ("l"), champ("o")) {
                    match self.passes.recevoir(&self.cle, &self.salle, l, o) {
                        Ok(Some((_, clair))) => match Lettre::lire(&clair) {
                            Ok(lettre) => self.dire(Nouvelle::Passe(Box::new(lettre))),
                            Err(e) => self.dire(Nouvelle::Illisible(e.to_string())),
                        },
                        Ok(None) => {}
                        Err(e) => self.dire(Nouvelle::Illisible(e)),
                    }
                }
                Ok(())
            }
            Some("refus") => {
                let pourquoi = champ("pourquoi").unwrap_or("refusé").to_string();
                self.dire(Nouvelle::Refus(pourquoi));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// **Un morceau numéroté** : dans l'ordre, sans trou ; une lettre entière part à la boucle, lue.
    fn un_morceau(
        &mut self,
        n: u64,
        l: &str,
        o: &str,
        canal: &mut dyn Canal,
    ) -> Result<(), String> {
        if n <= self.dernier {
            return Ok(()); // Une répétition.
        }
        if n > self.dernier + 1 {
            // Un trou : redemander depuis le dernier vu ; ce qui suit reviendra dans l'ordre.
            if self.redemande == Some(self.dernier) {
                return Ok(());
            }
            self.redemande = Some(self.dernier);
            return canal.envoyer(&depuis(self.dernier));
        }
        self.dernier = n;
        match self.reassemblage.recevoir(&self.cle, &self.salle, l, o) {
            Err(e) => self.dire(Nouvelle::Illisible(e)),
            Ok(None) => {}
            Ok(Some((id, clair))) => {
                if self.envoyes.front().is_some_and(|(e, _)| *e == id) {
                    self.envoyes.pop_front();
                }
                match Lettre::lire(&clair) {
                    Ok(lettre) => self.dire(Nouvelle::Lettre {
                        n,
                        id,
                        lettre: Box::new(lettre),
                    }),
                    Err(e) => self.dire(Nouvelle::Illisible(e.to_string())),
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
