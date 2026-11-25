//! **Le joueur d'un GIF visible** (GIF-2, fiche 159 § 6) : un fil qui lit ses octets, en tire les
//! toiles une à une, en fait la pyramide, la tend au magasin et réveille la boucle — trois en
//! mémoire au plus : la prête, celle du canal, celle qu'il fait. Le magasin le lâche quand l'image
//! quitte l'écran : son envoi suivant échoue, et le fil s'arrête.

use super::Lecteur;
use crate::persist::objets::Objets;
use crate::plateforme::Reveil;
use crate::renderer::atelier::admission;
use crate::renderer::photo::Pyramide;
use std::sync::mpsc::{sync_channel, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Le joueur : ce que son fil tend, la toile prête, et quand elle doit se montrer.
pub(crate) struct Joueur {
    recu: Receiver<(Pyramide, Duration)>,
    prete: Option<(Pyramide, Duration)>,
    echeance: Instant,
    /// Aucune toile encore montrée : la première reste son délai entier dès qu'elle paraît.
    premiere: bool,
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
            let Some(octets) = octets else {
                return;
            };
            let encodes = octets.len() as u64;
            let Some(mut lecteur) = Lecteur::nouveau(octets.into()) else {
                return;
            };
            // Chaque toile entre par l'admission, comme un décodage : sa pyramide ne se construit
            // que si elle tient dans ce que la machine laisse (MEMOIRE-3, fiche 159 § 7).
            let porte = admission::de_la_machine();
            while let Some(t) = lecteur.suivante() {
                let pic = admission::pic(encodes, (t.largeur, t.hauteur));
                let pyramide =
                    porte.pendant(pic, || Pyramide::depuis_rgba(t.largeur, t.hauteur, &t.rgba));
                let Some(p) = pyramide else {
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
            premiere: true,
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
        // **La cadence du GIF, pas celle de l'écran** : la toile suivante est due un délai après
        // l'heure où celle-ci l'était. Comptée depuis l'heure où elle paraît, l'image d'écran de
        // retard (jusqu'à 16,7 ms à 60 Hz) s'ajoutait à chaque toile — un GIF à 20 ms jouait aux
        // deux tiers de sa vitesse. Un retard plus long qu'un délai (une fenêtre cachée, un fil en
        // retard) repart de maintenant : rien ne se rattrape en rafale.
        let en_cadence = !self.premiere && maintenant.duration_since(self.echeance) < delai;
        let base = if en_cadence {
            self.echeance
        } else {
            maintenant
        };
        self.echeance = base + delai;
        self.premiere = false;
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

#[cfg(test)]
mod epreuves {
    use super::*;

    fn toile() -> Pyramide {
        Pyramide::depuis_rgba(1, 1, &[0, 0, 0, 255]).unwrap()
    }

    #[test]
    fn gif_5_la_cadence_ne_derive_pas_avec_le_retard_de_l_ecran() {
        let (envoi, recu) = sync_channel(4);
        let t0 = Instant::now();
        let ms = |n: u64| t0 + Duration::from_millis(n);
        let cent = Duration::from_millis(100);
        let mut joueur = Joueur {
            recu,
            prete: None,
            echeance: t0,
            premiere: true,
            fini: false,
        };
        // La première paraît quand elle est prête (30 ms), et reste son délai entier.
        envoi.send((toile(), cent)).unwrap();
        assert!(joueur.a_poser(ms(30)).is_some());
        assert_eq!(joueur.echeance, ms(130));
        // La deuxième, posée 15 ms en retard : la troisième reste due à 230, pas à 245.
        envoi.send((toile(), cent)).unwrap();
        assert!(joueur.a_poser(ms(129)).is_none());
        assert!(joueur.a_poser(ms(145)).is_some());
        assert_eq!(joueur.echeance, ms(230));
        // Un retard de plus d'un délai (la fenêtre cachée) repart de maintenant.
        envoi.send((toile(), cent)).unwrap();
        assert!(joueur.a_poser(ms(500)).is_some());
        assert_eq!(joueur.echeance, ms(600));
    }
}
