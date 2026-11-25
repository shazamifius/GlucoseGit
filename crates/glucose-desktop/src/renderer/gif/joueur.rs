//! **Le joueur d'un GIF visible** (GIF-2, fiche 159 § 6) : un fil qui lit ses octets, en tire les
//! toiles une à une, en fait la pyramide, la tend au magasin et réveille la boucle — trois en
//! mémoire au plus : la prête, celle du canal, celle qu'il fait. Le magasin le lâche quand l'image
//! quitte l'écran : son envoi suivant échoue, et le fil s'arrête.

use super::Lecteur;
use crate::persist::objets::Objets;
use crate::plateforme::Reveil;
use crate::renderer::photo::Pyramide;
use std::sync::mpsc::{sync_channel, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Le joueur : ce que son fil tend, la toile prête, et quand elle doit se montrer.
pub(crate) struct Joueur {
    recu: Receiver<(Pyramide, Duration)>,
    prete: Option<(Pyramide, Duration)>,
    echeance: Instant,
    fini: bool,
}

impl Joueur {
    /// **Lance le fil** de ce GIF ; sa première toile se montre dès qu'elle est prête.
    pub(crate) fn lancer(
        src: String,
        objets: Option<Arc<Objets>>,
        reveil: Option<Reveil>,
        maintenant: Instant,
    ) -> Self {
        let (envoi, recu) = sync_channel(1);
        std::thread::spawn(move || {
            crate::plateforme::priorite::ceder_au_fil_qui_dessine();
            let octets = match objets {
                Some(o) => o.lire(&src),
                None => std::fs::read(&src).ok(),
            };
            let Some(mut lecteur) = octets.and_then(|o| Lecteur::nouveau(o.into())) else {
                return;
            };
            while let Some(t) = lecteur.suivante() {
                let Some(p) = Pyramide::depuis_rgba(t.largeur, t.hauteur, &t.rgba) else {
                    return;
                };
                if envoi.send((p, t.delai)).is_err() {
                    return;
                }
                if let Some(reveil) = &reveil {
                    reveil();
                }
            }
        });
        Self {
            recu,
            prete: None,
            echeance: maintenant,
            fini: false,
        }
    }

    /// **Prend la toile que son fil a tendue**, s'il n'en tient pas déjà une.
    pub(crate) fn recevoir(&mut self) {
        if self.prete.is_none() {
            match self.recu.try_recv() {
                Ok(toile) => self.prete = Some(toile),
                Err(TryRecvError::Disconnected) => self.fini = true,
                Err(TryRecvError::Empty) => {}
            }
        }
    }

    /// **La pyramide de la toile suivante**, si son heure est venue à `maintenant`.
    pub(crate) fn a_poser(&mut self, maintenant: Instant) -> Option<Pyramide> {
        self.recevoir();
        if maintenant < self.echeance {
            return None;
        }
        let (pyramide, delai) = self.prete.take()?;
        self.echeance = maintenant + delai;
        Some(pyramide)
    }

    /// **Quand revenir** : à l'heure de la toile suivante si elle est prête ; sinon (`None`), son
    /// fil réveillera la boucle en la tendant.
    pub(crate) fn echeance(&self) -> Option<Instant> {
        self.prete.as_ref().map(|_| self.echeance)
    }

    /// **Son fil a tout joué** (les tours demandés, ou un GIF d'une seule image) et rien n'attend.
    pub(crate) fn fini(&self) -> bool {
        self.fini && self.prete.is_none()
    }
}
