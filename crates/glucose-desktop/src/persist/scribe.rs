//! **Le scribe** : le fil qui écrit l'histoire du document sur le disque.
//!
//! Le fil qui dessine ne touche jamais au disque pour enregistrer (charte, règle de la
//! cascade) : il confie au scribe des entrées déjà encodées, et continue. Le scribe les ajoute
//! à la suite du fichier, dans l'ordre reçu.
//!
//! # Quand le disque est synchronisé — sans constante
//!
//! Le scribe dort tant que rien n'arrive. Réveillé, il écrit tout ce qui attend, puis
//! synchronise le fichier (`sync_data`), puis se rendort. Pendant qu'il synchronise, ce qui
//! arrive s'accumule, et s'écrira d'un bloc au tour suivant. La fréquence des
//! synchronisations suit donc **la vitesse du disque** : chaque geste sur un disque rapide,
//! par paquets sur un disque lent. Aucune période n'est choisie (Redis en choisit une : une
//! seconde).
//!
//! # Sceller une image
//!
//! Une image qui arrive dans le document vient d'un fichier — souvent dans le dossier
//! temporaire de Windows — ou de pixels collés. Le scribe lit ses octets, calcule leur
//! empreinte (15 ms pour 4,7 Mo : jamais sur le fil qui dessine), ajoute un **objet** si ces
//! octets ne sont pas déjà dans le fichier, puis un **lien** de sa clé vers l'empreinte, et
//! pose la tranche dans le registre des [`Objets`]. Dès lors l'image ne dépend plus de rien.
//!
//! # Garder le texte en cours de frappe
//!
//! Le texte d'une carte en édition n'est pas encore un geste ([`histoire::saisie`]). Le scribe
//! le garde dans un petit fichier à côté, **après** avoir synchronisé les entrées qui le
//! précèdent, et l'accroche à la chaîne après le dernier geste qu'il a écrit. Passant par la
//! même file que les gestes, l'ordre est celui du travail : la validation s'écrit, se
//! synchronise, et alors seulement le fichier de la saisie s'efface.
//!
//! Le fichier du document s'ouvre pour y écrire **seul** ([`super::verrou`]).

use super::objets::{Objets, Source};
use super::verrou;
use glucose_core::hash::sha256;
use glucose_core::persist::histoire::{self, nature, ouvrir::Tranche, Chaine, Saisie};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// D'où viennent les octets d'une image à sceller.
pub enum Octets {
    Chemin(PathBuf),
    Memoire(Vec<u8>),
    /// Une tranche d'un autre document, vérifiée par son empreinte (BOARDS-2).
    Tranche {
        fichier: PathBuf,
        empreinte: [u8; 32],
        offset: u64,
        longueur: u64,
    },
}

/// Ce qu'on demande au scribe.
pub enum Ordre {
    /// Une entrée déjà encodée : geste, vue, jalon, instantané.
    Entree { nature: u8, contenu: Vec<u8> },
    /// Mettre les octets d'une image dans le document, sous cette clé.
    Sceller { cle: String, octets: Octets },
    /// Répondre quand tout ce qui précède est sur le disque.
    Synchroniser(Sender<Result<(), String>>),
    /// Continuer dans un autre fichier, copie exacte de celui-ci (« Enregistrer sous »).
    /// `oublier` : supprimer l'ancien une fois copié (un brouillon).
    Deplacer {
        vers: PathBuf,
        oublier: bool,
        reponse: Sender<Result<(), String>>,
    },
    /// Garder ce texte en cours de frappe, ou l'oublier (`None`). Le document qu'il porte est
    /// posé par le scribe, qui sait où il écrit.
    Saisie(Option<Saisie>),
}

/// Où commence l'écriture.
pub struct Depart {
    pub chemin: PathBuf,
    /// La base à écrire d'abord, si le fichier n'existe pas encore.
    pub base: Option<Vec<u8>>,
    /// Où s'écrit la prochaine entrée, et la chaîne à y reprendre.
    pub fin: u64,
    pub chaine: Chaine,
    /// La version lue dans l'en-tête : une base v2 passe à 3 avant sa première entrée.
    pub version: u16,
    /// Les octets déjà dans le fichier, par empreinte.
    pub objets: HashMap<[u8; 32], Tranche>,
    /// La chaîne après le dernier geste : là où une saisie s'accroche.
    pub dernier_geste: Chaine,
    /// Le dossier où vivent les saisies : celui des brouillons.
    pub saisies: PathBuf,
}

impl Depart {
    /// Un fichier neuf, qui commencera par cette base.
    pub fn neuf(chemin: PathBuf, base: Vec<u8>, saisies: PathBuf) -> Result<Self, String> {
        let chaine = Chaine::de_la_base(&base).map_err(|e| e.to_string())?;
        Ok(Self {
            chemin,
            fin: base.len() as u64,
            chaine,
            version: glucose_core::persist::container::CONTAINER_VERSION,
            objets: HashMap::new(),
            base: Some(base),
            dernier_geste: chaine,
            saisies,
        })
    }
}

/// La poignée du fil d'écriture. Le lâcher attend que tout soit écrit et synchronisé.
pub struct Scribe {
    vers: Option<Sender<Ordre>>,
    fil: Option<JoinHandle<()>>,
    /// La dernière erreur d'écriture, que le fil qui dessine doit dire.
    erreur: Arc<Mutex<Option<String>>>,
}

impl Scribe {
    /// Ouvre le fichier — ou le crée avec sa base — **ici**, puis lance le fil.
    ///
    /// Ouvrir sur le fil qui appelle, et non sur celui du scribe, est ce qui rend l'échec
    /// honnête : un chemin impossible, un fichier en lecture seule se disent avant que le
    /// moindre geste ait été confié. Confiés à un fil déjà mort, ils seraient perdus pour tout
    /// fichier suivant. La base d'un document neuf ne pèse que son document (quelques
    /// kilo-octets) : l'écrire ici ne coûte rien de visible.
    pub fn commencer(depart: Depart, objets: Arc<Objets>) -> Result<Self, String> {
        let plume = Plume::ouvrir(depart)?;
        objets.porter(Some(plume.chemin.clone()));
        let (vers, recu) = channel();
        let erreur = Arc::new(Mutex::new(None));
        let e = Arc::clone(&erreur);
        let fil = std::thread::Builder::new()
            .name("glucose-scribe".into())
            .spawn(move || vie(plume, &recu, &objets, &e))
            .map_err(|e| format!("le fil d'écriture n'a pas pu naître : {e}"))?;
        Ok(Self {
            vers: Some(vers),
            fil: Some(fil),
            erreur,
        })
    }

    pub fn envoyer(&self, ordre: Ordre) {
        if let Some(v) = &self.vers {
            // Un fil tombé a déjà laissé son erreur : l'envoi raté n'a rien de plus à dire.
            let _ = v.send(ordre);
        }
    }

    /// Attend que tout ce qui a été confié soit sur le disque.
    pub fn synchroniser(&self) -> Result<(), String> {
        let (r, attente) = channel();
        self.envoyer(Ordre::Synchroniser(r));
        attente
            .recv()
            .map_err(|_| "le fil d'écriture s'est arrêté".to_string())?
    }

    /// Continue l'écriture dans `vers`, copie exacte du fichier courant.
    pub fn deplacer(&self, vers: PathBuf, oublier: bool) -> Result<(), String> {
        let (reponse, attente) = channel();
        self.envoyer(Ordre::Deplacer {
            vers,
            oublier,
            reponse,
        });
        attente
            .recv()
            .map_err(|_| "le fil d'écriture s'est arrêté".to_string())?
    }

    /// L'erreur d'écriture survenue depuis le dernier appel, s'il y en a une.
    pub fn prendre_l_erreur(&self) -> Option<String> {
        self.erreur.lock().ok()?.take()
    }
}

impl Drop for Scribe {
    fn drop(&mut self) {
        // Lâcher l'envoyeur réveille le fil une dernière fois : il écrit ce qui reste,
        // synchronise, et s'arrête.
        self.vers = None;
        if let Some(fil) = self.fil.take() {
            let _ = fil.join();
        }
    }
}

/// L'état du fil d'écriture.
struct Plume {
    chemin: PathBuf,
    fichier: File,
    fin: u64,
    chaine: Chaine,
    /// Vrai tant que l'en-tête dit encore « version 2 » : il passe à 3 avant la première
    /// entrée, pour qu'une build plus ancienne refuse le fichier au lieu d'en montrer une
    /// base périmée.
    a_relever: bool,
    objets: HashMap<[u8; 32], Tranche>,
    dernier_geste: Chaine,
    saisies: PathBuf,
    /// Le fichier de la saisie de ce document, et ce qu'il garde.
    fichier_de_saisie: PathBuf,
    saisie: Option<(Chaine, Saisie)>,
}

/// **La preuve que ce qui a été confié au fichier est sur le disque.** Seul
/// [`Plume::synchroniser`] en fabrique une, et toucher au fichier d'une saisie la demande :
/// effacer un texte en cours avant que sa validation soit écrite ne se compile pas. Aucune
/// épreuve ne saurait le voir — il y faudrait une coupure de courant à la microseconde.
struct Synchronise(());

/// Ce qu'un tour du scribe doit faire une fois ses entrées sur le disque.
#[derive(Default)]
struct Tour {
    attentes: Vec<Sender<Result<(), String>>>,
    /// La dernière demande de saisie du tour, accrochée au point où elle est arrivée.
    saisie: Option<Option<(Chaine, Saisie)>>,
}

fn vie(
    mut plume: Plume,
    recu: &Receiver<Ordre>,
    objets: &Arc<Objets>,
    erreur: &Mutex<Option<String>>,
) {
    while let Ok(premier) = recu.recv() {
        let mut tour = Tour::default();
        let mut ordre = Some(premier);
        while let Some(o) = ordre.take() {
            if let Err(e) = plume.traiter(o, objets, &mut tour) {
                signaler(erreur, &e);
            }
            ordre = recu.try_recv().ok();
        }
        let fin = plume.synchroniser();
        if let Err(e) = &fin {
            signaler(erreur, e);
        }
        // La saisie après la synchronisation, et seulement si elle a réussi : une validation
        // est sur le disque avant que le texte qu'elle remplace ne s'efface.
        if let (Ok(jeton), Some(s)) = (&fin, tour.saisie.take()) {
            if let Err(e) = plume.garder(s, jeton) {
                signaler(erreur, &e);
            }
        }
        let fin = fin.map(|_| ());
        for r in tour.attentes {
            let _ = r.send(fin.clone());
        }
    }
    let _ = plume.fichier.sync_data();
}

fn signaler(erreur: &Mutex<Option<String>>, e: &str) {
    if let Ok(mut place) = erreur.lock() {
        *place = Some(e.to_string());
    }
}

impl Plume {
    fn ouvrir(depart: Depart) -> Result<Self, String> {
        let dire = |e: std::io::Error| verrou::dire(&depart.chemin, &e);
        let fichier = match &depart.base {
            Some(base) => {
                // Aucun dossier n'est créé ici : un chemin choisi par l'utilisateur existe, et
                // enregistrer ne doit jamais en inventer un. Seul le dossier des brouillons,
                // qui appartient à l'application, se crée — par qui l'y met.
                let mut f = verrou::ouvrir_seul(OpenOptions::new().create(true), &depart.chemin)?;
                // Vidé une fois tenu, jamais avant : c'est peut-être le document d'un autre.
                f.set_len(0).map_err(dire)?;
                f.write_all(base).map_err(dire)?;
                f
            }
            None => verrou::ouvrir_seul(&mut OpenOptions::new(), &depart.chemin)?,
        };
        // Une fin déchirée (écriture interrompue) ne suit pas la chaîne : on écrit par-dessus.
        if fichier.metadata().map_err(dire)?.len() > depart.fin {
            fichier.set_len(depart.fin).map_err(dire)?;
        }
        Ok(Self {
            a_relever: depart.version < glucose_core::persist::container::CONTAINER_VERSION,
            fichier_de_saisie: chemin_de_saisie(&depart.saisies, &depart.chemin),
            chemin: depart.chemin,
            fichier,
            fin: depart.fin,
            chaine: depart.chaine,
            objets: depart.objets,
            dernier_geste: depart.dernier_geste,
            saisies: depart.saisies,
            saisie: None,
        })
    }

    fn traiter(&mut self, ordre: Ordre, objets: &Objets, tour: &mut Tour) -> Result<(), String> {
        match ordre {
            Ordre::Entree { nature, contenu } => {
                let e = self.chaine.encadrer(nature, &contenu);
                self.ajouter(&[&e])?;
                if nature == nature::GESTE {
                    self.dernier_geste = self.chaine;
                }
                Ok(())
            }
            Ordre::Sceller { cle, octets } => self.sceller(&cle, octets, objets),
            Ordre::Synchroniser(r) => {
                tour.attentes.push(r);
                Ok(())
            }
            Ordre::Saisie(s) => {
                tour.saisie = Some(s.map(|s| (self.dernier_geste, s)));
                Ok(())
            }
            Ordre::Deplacer {
                vers,
                oublier,
                reponse,
            } => {
                let fait = self.deplacer(vers, oublier, objets);
                let _ = reponse.send(fait.clone());
                fait
            }
        }
    }

    /// Ajoute ces morceaux à la suite du fichier.
    fn ajouter(&mut self, morceaux: &[&[u8]]) -> Result<(), String> {
        let dire = |e: std::io::Error| format!("{} : {e}", self.chemin.display());
        if self.a_relever {
            self.fichier.seek(SeekFrom::Start(8)).map_err(dire)?;
            self.fichier
                .write_all(&glucose_core::persist::container::CONTAINER_VERSION.to_le_bytes())
                .map_err(dire)?;
            self.a_relever = false;
        }
        self.fichier.seek(SeekFrom::Start(self.fin)).map_err(dire)?;
        for m in morceaux {
            self.fichier.write_all(m).map_err(dire)?;
            self.fin += m.len() as u64;
        }
        Ok(())
    }

    fn sceller(&mut self, cle: &str, octets: Octets, objets: &Objets) -> Result<(), String> {
        let octets = match octets {
            Octets::Memoire(o) => o,
            Octets::Chemin(p) => std::fs::read(&p)
                .map_err(|e| format!("image non incorporée, {} : {e}", p.display()))?,
            Octets::Tranche {
                fichier,
                empreinte,
                offset,
                longueur,
            } => super::objets::lire_une_tranche(&fichier, &empreinte, offset, longueur)
                .ok_or_else(|| {
                    format!(
                        "image non incorporée : sa tranche de {} est illisible ou abîmée",
                        fichier.display()
                    )
                })?,
        };
        let empreinte = sha256(&octets);
        let tranche = match self.objets.get(&empreinte) {
            Some(t) => *t,
            None => {
                let entete = self
                    .chaine
                    .entete_d_objet(&empreinte, octets.len() as u64)
                    .map_err(|e| e.to_string())?;
                let debut = self.fin + entete.len() as u64;
                self.ajouter(&[&entete, &octets])?;
                let t = Tranche {
                    offset: debut,
                    longueur: octets.len() as u64,
                };
                self.objets.insert(empreinte, t);
                t
            }
        };
        let lien = self
            .chaine
            .encadrer(nature::LIEN, &histoire::contenu_lien(cle, &empreinte));
        self.ajouter(&[&lien])?;
        objets.poser(
            cle,
            Source::Tranche {
                empreinte,
                offset: tranche.offset,
                longueur: tranche.longueur,
            },
        );
        Ok(())
    }

    fn synchroniser(&mut self) -> Result<Synchronise, String> {
        self.fichier
            .sync_data()
            .map(|()| Synchronise(()))
            .map_err(|e| e.to_string())
    }

    fn deplacer(&mut self, vers: PathBuf, oublier: bool, objets: &Objets) -> Result<(), String> {
        let dire = |e: std::io::Error| format!("{} : {e}", vers.display());
        let jeton = self.synchroniser()?;
        // Copier à côté, puis renommer : un « Enregistrer sous » interrompu ne laisse jamais
        // un fichier cible à moitié copié.
        let a_cote = vers.with_extension("glucose.tmp");
        // Hors de Windows, renommer par-dessus un document qu'un autre écrit réussirait : il
        // écrirait ensuite dans un fichier que plus personne ne voit.
        #[cfg(not(windows))]
        if vers.exists() && verrou::tenu_ailleurs(&vers) {
            return Err(verrou::deja_tenu(&vers));
        }
        std::fs::copy(&self.chemin, &a_cote).map_err(dire)?;
        if let Err(e) = std::fs::rename(&a_cote, &vers) {
            let _ = std::fs::remove_file(&a_cote);
            return Err(verrou::dire(&vers, &e));
        }
        let nouveau = verrou::ouvrir_seul(&mut OpenOptions::new(), &vers)?;
        nouveau.sync_data().map_err(dire)?;
        let ancien = std::mem::replace(&mut self.chemin, vers);
        self.fichier = nouveau;
        objets.porter(Some(self.chemin.clone()));
        if oublier {
            // Le brouillon a fini son travail : il vit désormais sous son vrai nom.
            let _ = std::fs::remove_file(ancien);
        }
        // La saisie suit le document : elle se nomme d'après lui.
        let saisie = self.saisie.take();
        let _ = self.garder(None, &jeton);
        self.fichier_de_saisie = chemin_de_saisie(&self.saisies, &self.chemin);
        match saisie {
            Some(s) => self.garder(Some(s), &jeton),
            None => Ok(()),
        }
    }

    /// Écrit la saisie, ou l'efface. Écrite à côté puis renommée : le fichier d'une saisie
    /// est toujours entier — l'ancienne ou la nouvelle, jamais un mélange.
    fn garder(&mut self, s: Option<(Chaine, Saisie)>, _: &Synchronise) -> Result<(), String> {
        let f = &self.fichier_de_saisie;
        let dire = |e: std::io::Error| format!("texte en cours, {} : {e}", f.display());
        let Some((point, mut saisie)) = s else {
            self.saisie = None;
            return match std::fs::remove_file(f) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(dire(e)),
                _ => Ok(()),
            };
        };
        saisie.document = self.chemin.to_string_lossy().into_owned();
        std::fs::create_dir_all(&self.saisies).map_err(dire)?;
        let a_cote = f.with_extension("saisie.tmp");
        let mut t = File::create(&a_cote).map_err(dire)?;
        t.write_all(&histoire::saisie::ecrire(point, &saisie))
            .map_err(dire)?;
        t.sync_data().map_err(dire)?;
        drop(t);
        std::fs::rename(&a_cote, f).map_err(dire)?;
        self.saisie = Some((point, saisie));
        Ok(())
    }
}

/// Le fichier de la saisie d'un document : nommé par l'empreinte de son chemin, dans le
/// dossier des saisies. Un document n'en a qu'un, qu'il retrouve en s'ouvrant.
pub fn chemin_de_saisie(dossier: &Path, document: &Path) -> PathBuf {
    let canonique = std::fs::canonicalize(document).unwrap_or_else(|_| document.to_path_buf());
    let e = sha256(canonique.as_os_str().as_encoded_bytes());
    let nom: String = e[..8].iter().map(|o| format!("{o:02x}")).collect();
    dossier.join(format!("{nom}.saisie"))
}
