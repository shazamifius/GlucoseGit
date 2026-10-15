//! **LIBERE-1 — un brouillon tenu au lancement attend qu'on le lâche** (fiche 72).
//!
//! # Ce qu'il a vu
//!
//! Le 15/10, au téléphone, un tableau de plus de 6 000 images a « juste dépop » : le canevas
//! vide, la carte de bienvenue — Glucose venait de redémarrer, et la reprise n'avait rien
//! rouvert. Android relance l'application qu'on regardait quand son processus meurt. Or la
//! reprise saute tout brouillon qu'un autre Glucose **tient** encore (son verrou) : au bureau,
//! c'est une autre fenêtre qui l'a ouvert, et c'est juste. Au téléphone, le seul autre Glucose
//! possible est **celui qui est en train de mourir** — et un processus qui tient des gigaoctets
//! d'images met du temps à finir de mourir. S'il tient encore son verrou quand le nouveau
//! cherche, le travail est sauté. C'est une hypothèse, pas une preuve : la ligne `reprise` le
//! dira désormais (`tenus`).
//!
//! # La règle, sans délai choisi
//!
//! Chaque brouillon tenu au lancement a son fil, qui **attend son verrou** (`File::lock`) : il
//! s'ouvre quand l'autre Glucose le lâche — sa mort, ou sa fermeture —, et c'est elle qui le
//! réveille, pas une horloge. Libéré, le brouillon se rouvre si le document d'accueil est encore
//! intact (aucun geste, aucun fichier) ; sinon, un message dit qu'il attend le prochain
//! lancement. Au bureau, c'est aussi la bonne conduite : le brouillon d'une autre fenêtre qui
//! plante revient dans celle-ci.
//!
//! **Sous Windows**, le verrou n'est pas un `flock` mais un mode de partage, qu'on ne peut pas
//! attendre : un brouillon tenu y appartient à une fenêtre vivante, et y reste.

use crate::app::GlucoseApp;
use crate::plateforme::Reveil;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

/// Ce que la reprise a trouvé dans le dossier des brouillons (REPRISE-1).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Trouve {
    /// Les brouillons qu'on pouvait rouvrir, celui qu'on rouvre compris.
    pub brouillons: usize,
    /// Ceux qu'un autre Glucose tenait encore.
    pub tenus: usize,
    /// Ce que pèsent tous les brouillons du dossier.
    pub octets: u64,
}

/// **Les brouillons libérés**, que leurs fils envoient.
pub struct Liberes(Receiver<PathBuf>);

impl GlucoseApp {
    /// **Ce que le lancement a rouvert, pour la boîte noire** (REPRISE-1, fiche 72) : rien, un
    /// brouillon, un document, un échec ; ce qu'il a trouvé et ce qui était tenu ; et la place
    /// du disque. Des nombres et un nom fermé, jamais un chemin.
    pub(super) fn noter_la_reprise(
        &mut self,
        rouvert: &'static str,
        trouve: Trouve,
        chemin: Option<&Path>,
    ) {
        let mo = chemin
            .and_then(|c| std::fs::metadata(c).ok())
            .map_or(0, |m| m.len() >> 20);
        let images = self.store.project.toutes_les_images().count();
        let place = crate::plateforme::place::lire();
        self.chronique
            .noter_la_reprise(crate::boite_noire::enregistrement::Reprise {
                rouvert,
                brouillons: trouve.brouillons as u64,
                tenus: trouve.tenus as u64,
                brouillons_mo: trouve.octets >> 20,
                images: images as u64,
                mo,
                libre_mo: place.map(|p| p.libre >> 20),
                total_mo: place.map(|p| p.total >> 20),
            });
    }

    /// **Attend chaque brouillon tenu**, chacun sur son fil, qui réveille la boucle quand il
    /// est libre.
    pub(super) fn attendre_les_brouillons_tenus(&mut self, tenus: Vec<PathBuf>) {
        if tenus.is_empty() {
            return;
        }
        let (vers, recu) = channel();
        let reveil = self.reveil_de_l_export();
        for chemin in tenus {
            attendre_un(chemin, vers.clone(), reveil.clone());
        }
        self.attentes.liberes = Some(Liberes(recu));
    }

    /// **Les brouillons qui se sont libérés depuis l'image d'avant** : chacun se rouvre si le
    /// document d'accueil est intact, ou attend le prochain lancement.
    pub(crate) fn suivre_les_brouillons_liberes(&mut self) {
        let Some(Liberes(recu)) = &self.attentes.liberes else {
            return;
        };
        let liberes: Vec<PathBuf> = recu.try_iter().collect();
        for chemin in liberes {
            self.reprendre_un_brouillon_libere(&chemin);
        }
    }

    fn reprendre_un_brouillon_libere(&mut self, chemin: &Path) {
        // Fermé proprement par l'autre — rangé sous un nom, ou abandonné : il n'attend rien.
        if !chemin.exists() {
            return;
        }
        let intact = self.disque.ecriture.is_none() && self.project_path.is_none();
        let dit = if !intact {
            "Un brouillon s'est libéré : il se rouvrira au prochain lancement".to_string()
        } else {
            let (dit, rouvert) = match self.ouvrir_un_brouillon(chemin) {
                Ok(m) => (format!("Ton travail est revenu — {m}"), "brouillon"),
                Err(e) => (
                    format!(
                        "Un brouillon n'a pas pu se rouvrir : {e} — il reste dans {}",
                        chemin.display()
                    ),
                    "echec",
                ),
            };
            let octets = std::fs::metadata(chemin).map_or(0, |m| m.len());
            let trouve = Trouve {
                brouillons: 1,
                tenus: 0,
                octets,
            };
            self.noter_la_reprise(rouvert, trouve, Some(chemin));
            dit
        };
        self.dire_la_reprise(&[dit]);
        self.mark_dirty();
    }
}

/// Le fil d'un brouillon tenu : il attend son verrou, le lâche aussitôt, et le dit.
#[cfg(not(windows))]
fn attendre_un(chemin: PathBuf, vers: Sender<PathBuf>, reveil: Option<Reveil>) {
    let _ = std::thread::Builder::new()
        .name("brouillon-tenu".into())
        .spawn(move || {
            if let Ok(f) = std::fs::File::open(&chemin) {
                // Bloque jusqu'à ce que l'autre Glucose le lâche : sa mort, ou sa fermeture.
                let _ = f.lock();
            }
            if vers.send(chemin).is_ok() {
                if let Some(r) = reveil {
                    r();
                }
            }
        });
}

/// Sous Windows, un brouillon tenu l'est par une fenêtre vivante, et le mode de partage ne
/// s'attend pas : il reste à elle.
#[cfg(windows)]
fn attendre_un(chemin: PathBuf, vers: Sender<PathBuf>, reveil: Option<Reveil>) {
    let _ = (chemin, vers, reveil);
}
