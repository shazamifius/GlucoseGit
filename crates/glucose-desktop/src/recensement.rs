//! **Lire les fichiers connus, sans jamais y écrire** (fiche 88), pour savoir ce que les domaines
//! traversent. La vue elle-même est pure et vit dans le noyau (`glucose_core::recensement`) ; ici,
//! un fil lit le disque et la lui donne.
//!
//! # Ce que Glucose connaît
//!
//! Les documents déjà ouverts ou enregistrés (une ligne par chemin, dans [`FICHIER`]), leurs voisins
//! de dossier, ses brouillons et ses documents rangés. Rien d'autre : Glucose ne fouille pas le
//! disque. Ce fichier n'est qu'une liste de chemins : le perdre ne perd aucun domaine, puisque
//! chaque fichier garde les siens.
//!
//! Un fichier déplacé emporte donc ses domaines. Déplacé dans un dossier connu, il se retrouve à la
//! passe suivante ; déplacé ailleurs, il sort de la vue jusqu'à ce qu'on le rouvre.
//!
//! # RECENSE-2 — ce fil ne gêne jamais
//!
//! Il cède au fil qui dessine, et ne relit un fichier que si sa date ou sa taille a changé : une passe
//! qui ne trouve rien de neuf ne lit que des dates. La boucle n'est réveillée que s'il y a du neuf.
//! Lire ne prend aucun verrou : un fichier qu'un autre Glucose écrit se lit jusqu'à sa dernière
//! entrée saine (`histoire::ouvrir`). Un fichier illisible (Glucose Tauri, une version plus récente)
//! n'a rien à dire, et ne dit rien.
//!
//! **Le fichier ouvert ne se lit jamais** : il est vivant dans Glucose, et c'est de là que la vue le
//! prend. Le relire à chaque geste écrit reviendrait à lire 2,3 Go pour rien (mesuré : 2,6 s, sur
//! le tableau de 8 000 images). Quand on en ouvre un autre, il redevient un fichier comme les autres,
//! et la passe qui suit le lit tel qu'on l'a laissé.

use crate::plateforme::Reveil;
use glucose_core::persist::{histoire, FILE_EXTENSION};
use glucose_core::recensement::{recenser, DocumentRecense, Recensement};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::SystemTime;

/// Le fichier des documents connus, dans le dossier de Glucose.
pub const FICHIER: &str = "documents-connus.txt";

/// Ce qu'une passe doit regarder.
struct Passe {
    /// Son rang : la passe finie qui porte le dernier rang demandé dit que tout est lu.
    rang: u64,
    /// Le fichier que Glucose écrit : vivant, il ne se lit pas.
    ouvert: Option<PathBuf>,
    connus: Vec<PathBuf>,
    dossiers: Vec<PathBuf>,
}

/// Ce que le fil rapporte.
enum Arrivee {
    Lu(DocumentRecense),
    Parti(String),
    Fin(u64),
}

struct Fil {
    demandes: Sender<Passe>,
    arrivees: Receiver<Arrivee>,
}

/// **Le recenseur** : l'état déduit des fichiers, et le fil qui les lit. Sans [`Recenseur::demarrer`]
/// (une épreuve, un banc), il ne lit rien.
#[derive(Default)]
pub struct Recenseur {
    fil: Option<Fil>,
    /// Le fichier des connus, et les dossiers que Glucose range lui-même.
    fichier: Option<PathBuf>,
    dossiers: Vec<PathBuf>,
    /// Ce qu'on a lu jusqu'ici.
    pub etat: Recensement,
    /// Une passe est-elle en route ? Ce qu'on rend à un agent le dit.
    pub en_cours: bool,
    /// Le rang de la dernière passe demandée.
    demandee: u64,
    /// Le fichier que Glucose écrit en ce moment.
    ouvert: Option<PathBuf>,
    /// Le document et sa version au dernier calcul de ce que le panneau montre.
    pub(crate) vu: Option<(String, u64)>,
    /// Où aller quand le document visé sera ouvert.
    pub(crate) destination: Option<crate::dock::domains::LigneOu>,
}

impl Recenseur {
    /// **Démarre le fil**, au vrai lancement : `dossier` est celui de Glucose, et `reveil` réveille
    /// la boucle quand il y a du neuf.
    pub fn demarrer(&mut self, dossier: &Path, dossiers: Vec<PathBuf>, reveil: Reveil) {
        let (demandes, recues) = mpsc::channel();
        let (envoi, arrivees) = mpsc::channel();
        let lance = std::thread::Builder::new()
            .name("recensement".into())
            .spawn(move || servir(&recues, &envoi, &reveil));
        if lance.is_err() {
            return;
        }
        self.fil = Some(Fil { demandes, arrivees });
        self.fichier = Some(dossier.join(FICHIER));
        self.dossiers = dossiers;
        self.relancer();
    }

    fn connus(&self) -> Vec<PathBuf> {
        self.fichier
            .as_deref()
            .and_then(|f| std::fs::read_to_string(f).ok())
            .map(|texte| lire_les_connus(&texte))
            .unwrap_or_default()
    }

    /// **Retient ce document**, en tête de la liste, et le fait lire.
    pub fn connaitre(&mut self, chemin: &Path) {
        let Some(fichier) = self.fichier.clone() else {
            return;
        };
        let liste = avec_en_tete(self.connus(), chemin);
        let texte: String = liste.iter().map(|p| format!("{}\n", p.display())).collect();
        // Perdue, cette liste se refait : on ne fait pas d'une écriture ratée une faute.
        let _ = crate::persist::atomic::write_atomic(&fichier, texte.as_bytes());
        self.relancer();
    }

    /// **Demande une passe.** Elle ne bloque rien : ce qu'elle trouve arrive plus tard.
    pub fn relancer(&mut self) {
        if self.fil.is_none() {
            return;
        }
        self.demandee += 1;
        let passe = Passe {
            rang: self.demandee,
            ouvert: self.ouvert.clone(),
            connus: self.connus(),
            dossiers: self.dossiers.clone(),
        };
        self.en_cours = self
            .fil
            .as_ref()
            .is_some_and(|fil| fil.demandes.send(passe).is_ok());
    }

    /// **Le fichier que Glucose écrit** : il ne se lit pas tant qu'il est ouvert. Quand il change, celui
    /// qu'on quitte se lit, tel qu'on l'a laissé.
    pub fn suivre_le_fichier_ouvert(&mut self, ouvert: Option<PathBuf>) {
        if self.ouvert != ouvert {
            self.ouvert = ouvert;
            self.relancer();
        }
    }

    /// **Ce que le fil a rapporté depuis la dernière fois.** Rend vrai si la vue a pu changer.
    pub fn relever(&mut self) -> bool {
        let Some(fil) = &self.fil else {
            return false;
        };
        let mut neuf = false;
        while let Ok(arrivee) = fil.arrivees.try_recv() {
            match arrivee {
                Arrivee::Lu(document) => {
                    self.etat.poser(document);
                    neuf = true;
                }
                Arrivee::Parti(chemin) => {
                    self.etat.oublier(&chemin);
                    neuf = true;
                }
                // Une passe plus ancienne qui finit ne dit rien de la dernière demandée.
                Arrivee::Fin(rang) => self.en_cours = rang != self.demandee,
            }
        }
        neuf
    }
}

/// La liste écrite : une ligne par chemin.
fn lire_les_connus(texte: &str) -> Vec<PathBuf> {
    texte
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(PathBuf::from)
        .collect()
}

/// Ce chemin en tête, sans doublon.
fn avec_en_tete(liste: Vec<PathBuf>, chemin: &Path) -> Vec<PathBuf> {
    std::iter::once(chemin.to_path_buf())
        .chain(liste.into_iter().filter(|p| p != chemin))
        .collect()
}

/// Un document de Glucose, à son extension.
fn est_un_document(chemin: &Path) -> bool {
    chemin
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(FILE_EXTENSION))
}

/// **Les fichiers à regarder** : les connus qui sont encore là, leurs voisins, et ce que les dossiers
/// de Glucose portent.
fn fichiers_a_regarder(passe: &Passe) -> BTreeSet<PathBuf> {
    let voisins = passe.connus.iter().filter_map(|p| p.parent());
    let dossiers = passe.dossiers.iter().map(PathBuf::as_path).chain(voisins);
    let mut fichiers: BTreeSet<PathBuf> = dossiers
        .collect::<BTreeSet<&Path>>()
        .into_iter()
        .filter_map(|d| std::fs::read_dir(d).ok())
        .flat_map(|entrees| entrees.flatten().map(|e| e.path()))
        .filter(|p| est_un_document(p) && p.is_file())
        .collect();
    fichiers.extend(passe.connus.iter().filter(|p| p.is_file()).cloned());
    fichiers
}

/// Ce qu'un fichier dit de ses domaines, ou rien s'il n'est pas lisible.
fn lire_un_document(chemin: &Path) -> Option<DocumentRecense> {
    let fichier = std::fs::File::open(chemin).ok()?;
    let ouvert = histoire::ouvrir(&mut std::io::BufReader::new(fichier)).ok()?;
    Some(recenser(&chemin.display().to_string(), &ouvert.projet))
}

/// Ce qui dit qu'un fichier a changé, sans le lire.
fn empreinte_du_fichier(chemin: &Path) -> Option<(SystemTime, u64)> {
    let m = std::fs::metadata(chemin).ok()?;
    Some((m.modified().ok()?, m.len()))
}

fn servir(demandes: &Receiver<Passe>, arrivees: &Sender<Arrivee>, reveil: &Reveil) {
    crate::plateforme::priorite::ceder_au_fil_qui_dessine();
    let mut vus: HashMap<PathBuf, (SystemTime, u64)> = HashMap::new();
    while let Ok(mut passe) = demandes.recv() {
        // Plusieurs passes demandées pendant qu'on lisait : seule la dernière compte.
        while let Ok(plus_recente) = demandes.try_recv() {
            passe = plus_recente;
        }
        let fichiers = fichiers_a_regarder(&passe);
        let partis: Vec<PathBuf> = vus
            .keys()
            .filter(|p| !fichiers.contains(*p))
            .cloned()
            .collect();
        let mut neuf = !partis.is_empty();
        for chemin in partis {
            vus.remove(&chemin);
            if arrivees
                .send(Arrivee::Parti(chemin.display().to_string()))
                .is_err()
            {
                return;
            }
        }
        for chemin in fichiers {
            if passe.ouvert.as_ref() == Some(&chemin) {
                continue;
            }
            let Some(empreinte) = empreinte_du_fichier(&chemin) else {
                continue;
            };
            if vus.get(&chemin) == Some(&empreinte) {
                continue;
            }
            vus.insert(chemin.clone(), empreinte);
            let Some(document) = lire_un_document(&chemin) else {
                continue;
            };
            if arrivees.send(Arrivee::Lu(document)).is_err() {
                return;
            }
            neuf = true;
            reveil();
        }
        if arrivees.send(Arrivee::Fin(passe.rang)).is_err() {
            return;
        }
        if neuf {
            reveil();
        }
    }
}

#[cfg(test)]
mod tests;
