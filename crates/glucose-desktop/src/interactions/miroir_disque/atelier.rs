//! **L'atelier des miroirs** (DISQUE-2, fiche 164 § 9) : un fil qui lit le disque, pour que le fil
//! de l'image ne l'attende jamais — 600 photos coûtaient 63 ms à froid sur un NVMe (§ 8), des
//! secondes sur un disque dur, une clé ou un partage réseau.
//!
//! Il fait, dans l'ordre où on les lui confie, deux travaux : **lire un niveau** (ses entrées, le
//! Markdown des textes qui ont changé, et ce que le miroir avait vu, relu de son fichier la première
//! fois) et **écrire ce qu'un miroir a vu**. Un seul fil : une écriture et la lecture qui la suit ne
//! se croisent jamais. Il naît à la première lecture, et meurt avec l'application.

use super::lire_un_niveau;
use glucose_core::file_kind;
use glucose_core::miroir_disque::reconcilier::{lire_les_vus, Vus};
use glucose_core::miroir_disque::{Entree, Sorte};
use glucose_core::store::miroir_disque::MiroirOuvert;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

/// **Une lecture demandée** : le miroir, ce qu'on sait de ce qu'il a vu, et où cela se garde.
#[derive(Debug, Clone)]
pub(crate) struct Demande {
    /// Le document qui l'a demandée : une lecture qui revient après qu'on en a ouvert un autre
    /// ne s'y applique pas.
    pub document: String,
    pub miroir: MiroirOuvert,
    /// Ce que le miroir a vu, s'il est en mémoire ; sinon l'atelier le relit de son fichier.
    pub vus: Option<Vus>,
    /// Le fichier où ce miroir garde ce qu'il a vu (aucun : un document sans fichier).
    pub fichier: Option<PathBuf>,
}

/// **Une lecture faite.**
#[derive(Debug)]
pub(crate) struct Lu {
    pub demande: Demande,
    /// Ce que le miroir avait vu : la mémoire, ou son fichier.
    pub vus: Option<Vus>,
    pub entrees: Result<Vec<Entree>, String>,
    /// Le Markdown de chaque texte relu, par chemin : ceux qui ont changé, ou qu'on ne connaissait pas.
    pub textes: HashMap<String, String>,
}

enum Travail {
    Lire(Box<Demande>),
    Ecrire(PathBuf, String),
}

/// **L'atelier**, vu du fil de l'image.
#[derive(Debug)]
pub(crate) struct Atelier {
    envoi: Sender<Travail>,
    pub recu: Receiver<Lu>,
}

impl Atelier {
    /// Ouvre le fil ; `reveil` réveille la boucle quand une lecture revient.
    pub fn ouvrir(reveil: Option<winit::event_loop::EventLoopProxy<()>>) -> Option<Self> {
        let (envoi, travaux) = channel::<Travail>();
        let (rendu, recu) = channel();
        std::thread::Builder::new()
            .name("glucose-miroirs".into())
            .spawn(move || {
                for t in travaux {
                    match t {
                        Travail::Lire(d) => {
                            if rendu.send(lire(*d)).is_err() {
                                return;
                            }
                            if let Some(proxy) = &reveil {
                                let _ = proxy.send_event(());
                            }
                        }
                        Travail::Ecrire(fichier, texte) => ecrire(&fichier, &texte),
                    }
                }
            })
            .ok()?;
        Some(Self { envoi, recu })
    }

    pub fn lire(&self, d: Demande) -> bool {
        self.envoi.send(Travail::Lire(Box::new(d))).is_ok()
    }

    pub fn ecrire(&self, fichier: PathBuf, texte: String) {
        let _ = self.envoi.send(Travail::Ecrire(fichier, texte));
    }
}

/// **Lit un niveau pour le réconcilier.** Un miroir « à lire » n'a rien vu qui compte : jamais lu,
/// ou sa première lecture défaite — ce qu'il avait vu alors ferait prendre chaque fichier pour un
/// nœud retiré.
pub(crate) fn lire(demande: Demande) -> Lu {
    let vus = if demande.miroir.source.pending_scan {
        None
    } else {
        demande.vus.clone().or_else(|| {
            let texte = std::fs::read_to_string(demande.fichier.as_ref()?).ok()?;
            Some(lire_les_vus(&texte))
        })
    };
    let racine = Path::new(&demande.miroir.source.root_path);
    let entrees = lire_un_niveau(racine, demande.miroir.source.sort_by).map_err(|e| e.to_string());
    let textes = match &entrees {
        Ok(es) => relire_les_textes(es, vus.as_ref()),
        Err(_) => HashMap::new(),
    };
    Lu {
        demande,
        vus,
        entrees,
        textes,
    }
}

/// Les textes qui ont changé depuis la lecture d'avant, ou qu'on ne connaît pas : relus.
fn relire_les_textes(entrees: &[Entree], vus: Option<&Vus>) -> HashMap<String, String> {
    entrees
        .iter()
        .filter(|e| e.sorte == Sorte::Lisible)
        .filter(|e| {
            vus.and_then(|v| v.get(&e.chemin))
                .is_none_or(|v| (v.taille, v.date) != (e.taille, e.date))
        })
        .filter_map(|e| {
            Some((
                e.chemin.clone(),
                markdown_du_fichier(Path::new(&e.chemin), &e.nom)?,
            ))
        })
        .collect()
}

/// **Le Markdown d'un fichier lisible** : ses premiers [`file_kind::INLINE_MAX_BYTES`] octets, et
/// jamais plus — un journal de deux gigaoctets dans un dossier ne se lit pas en entier pour en
/// montrer cent kilo-octets. Ce qui n'est pas de l'UTF-8 valide (un `.txt` d'une autre époque) se
/// remplace, sans refuser le fichier.
pub(crate) fn markdown_du_fichier(chemin: &Path, nom: &str) -> Option<String> {
    let mut octets = Vec::new();
    let borne = file_kind::INLINE_MAX_BYTES as u64 + 1;
    std::fs::File::open(chemin)
        .ok()?
        .take(borne)
        .read_to_end(&mut octets)
        .ok()?;
    let tronque = octets.len() > file_kind::INLINE_MAX_BYTES;
    let texte = String::from_utf8_lossy(&octets);
    let contenu = file_kind::truncate_on_char_boundary(&texte, file_kind::INLINE_MAX_BYTES);
    Some(file_kind::as_markdown(nom, contenu, tronque))
}

/// Écrit ce qu'un miroir a vu ; un échec se tait — perdu, les nœuds retirés reviendront une fois.
fn ecrire(fichier: &Path, texte: &str) {
    if let Some(dossier) = fichier.parent() {
        let _ = std::fs::create_dir_all(dossier);
    }
    let _ = std::fs::write(fichier, texte);
}
