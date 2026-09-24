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

use super::objets::{Objets, Source};
use glucose_core::hash::sha256;
use glucose_core::persist::histoire::{self, nature, ouvrir::Tranche, Chaine};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// D'où viennent les octets d'une image à sceller.
pub enum Octets {
    Chemin(PathBuf),
    Memoire(Vec<u8>),
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
}

impl Depart {
    /// Un fichier neuf, qui commencera par cette base.
    pub fn neuf(chemin: PathBuf, base: Vec<u8>) -> Result<Self, String> {
        let chaine = Chaine::de_la_base(&base).map_err(|e| e.to_string())?;
        Ok(Self {
            chemin,
            fin: base.len() as u64,
            chaine,
            version: glucose_core::persist::container::CONTAINER_VERSION,
            objets: HashMap::new(),
            base: Some(base),
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
}

fn vie(
    mut plume: Plume,
    recu: &Receiver<Ordre>,
    objets: &Arc<Objets>,
    erreur: &Mutex<Option<String>>,
) {
    while let Ok(premier) = recu.recv() {
        let mut attentes = Vec::new();
        let mut ordre = Some(premier);
        while let Some(o) = ordre.take() {
            if let Err(e) = plume.traiter(o, objets, &mut attentes) {
                signaler(erreur, &e);
            }
            ordre = recu.try_recv().ok();
        }
        let fin = plume.fichier.sync_data().map_err(|e| e.to_string());
        if let Err(e) = &fin {
            signaler(erreur, e);
        }
        for r in attentes {
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
        let dire = |e: std::io::Error| format!("{} : {e}", depart.chemin.display());
        let fichier = match &depart.base {
            Some(base) => {
                // Aucun dossier n'est créé ici : un chemin choisi par l'utilisateur existe, et
                // enregistrer ne doit jamais en inventer un. Seul le dossier des brouillons,
                // qui appartient à l'application, se crée — par qui l'y met.
                let mut f = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&depart.chemin)
                    .map_err(dire)?;
                f.write_all(base).map_err(dire)?;
                f
            }
            None => OpenOptions::new()
                .read(true)
                .write(true)
                .open(&depart.chemin)
                .map_err(dire)?,
        };
        // Une fin déchirée (écriture interrompue) ne suit pas la chaîne : on écrit par-dessus.
        if fichier.metadata().map_err(dire)?.len() > depart.fin {
            fichier.set_len(depart.fin).map_err(dire)?;
        }
        Ok(Self {
            a_relever: depart.version < glucose_core::persist::container::CONTAINER_VERSION,
            chemin: depart.chemin,
            fichier,
            fin: depart.fin,
            chaine: depart.chaine,
            objets: depart.objets,
        })
    }

    fn traiter(
        &mut self,
        ordre: Ordre,
        objets: &Objets,
        attentes: &mut Vec<Sender<Result<(), String>>>,
    ) -> Result<(), String> {
        match ordre {
            Ordre::Entree { nature, contenu } => {
                let e = self.chaine.encadrer(nature, &contenu);
                self.ajouter(&[&e])
            }
            Ordre::Sceller { cle, octets } => self.sceller(&cle, octets, objets),
            Ordre::Synchroniser(r) => {
                attentes.push(r);
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

    fn deplacer(&mut self, vers: PathBuf, oublier: bool, objets: &Objets) -> Result<(), String> {
        let dire = |e: std::io::Error| format!("{} : {e}", vers.display());
        self.fichier.sync_data().map_err(dire)?;
        // Copier à côté, puis renommer : un « Enregistrer sous » interrompu ne laisse jamais
        // un fichier cible à moitié copié.
        let a_cote = vers.with_extension("glucose.tmp");
        std::fs::copy(&self.chemin, &a_cote).map_err(dire)?;
        std::fs::rename(&a_cote, &vers).map_err(dire)?;
        let nouveau = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&vers)
            .map_err(dire)?;
        nouveau.sync_data().map_err(dire)?;
        let ancien = std::mem::replace(&mut self.chemin, vers);
        self.fichier = nouveau;
        objets.porter(Some(self.chemin.clone()));
        if oublier {
            // Le brouillon a fini son travail : il vit désormais sous son vrai nom.
            let _ = std::fs::remove_file(ancien);
        }
        Ok(())
    }
}
