//! **Le joueur d'une vidéo** (VIDEO-1, fiche 169) : un fil qui tient son lecteur, en tire les
//! images à leur heure (le système les cadence et les accorde au son), en fait la pyramide et la
//! tend au magasin, puis réveille la boucle. Il obéit à deux ordres — jouer ou pas, le son ou
//! pas — et s'arrête quand le magasin le lâche (son envoi suivant échoue), à la fin de la vidéo,
//! ou à la première faute.

use crate::persist::objets::Objets;
use crate::plateforme::Reveil;
use crate::renderer::photo::Pyramide;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, TryRecvError};
use std::sync::Arc;

/// Ce que le magasin peut dire au fil. Hors Linux, aucun lecteur ne l'écoute encore (leur porte
/// suit, fiche 169) : l'attente tombera le jour où l'un l'écoutera.
#[cfg_attr(not(target_os = "linux"), expect(dead_code))]
enum Ordre {
    Jouer(bool),
    Son(bool),
}

/// **Une vidéo en cours** : ce que son fil tend, et ce qu'on lui a demandé.
pub(crate) struct JoueurVideo {
    recu: Receiver<Pyramide>,
    /// L'image reçue, pas encore posée.
    prete: Option<Pyramide>,
    ordres: Sender<Ordre>,
    joue: bool,
    son: bool,
    fini: bool,
}

impl JoueurVideo {
    /// **Lance le fil** de cette vidéo : elle joue tout de suite, le son coupé (sa réponse du
    /// 25/11 : coupé par défaut, rallumé par son icône).
    pub(crate) fn lancer(src: String, objets: Option<Arc<Objets>>, reveil: Option<Reveil>) -> Self {
        let (envoi, recu) = sync_channel(1);
        let (ordres, a_faire) = channel();
        std::thread::spawn(move || {
            crate::plateforme::priorite::ceder_au_fil_qui_dessine();
            jouer(&src, objets.as_deref(), &envoi, &a_faire, reveil.as_ref());
            // Fini : la boucle le voit, et son nœud remontre ▶.
            drop(envoi);
            if let Some(reveil) = &reveil {
                reveil();
            }
        });
        Self {
            recu,
            prete: None,
            ordres,
            joue: true,
            son: false,
            fini: false,
        }
    }

    /// **Prend l'image que le fil a tendue**, s'il n'en tient pas déjà une.
    fn recevoir(&mut self) {
        if self.prete.is_none() {
            match self.recu.try_recv() {
                Ok(p) => self.prete = Some(p),
                Err(TryRecvError::Disconnected) => self.fini = true,
                Err(TryRecvError::Empty) => {}
            }
        }
    }

    /// **Une image attend d'être posée** : la boucle doit redessiner (son fil l'a réveillée).
    pub(crate) fn prete(&mut self) -> bool {
        self.recevoir();
        self.prete.is_some()
    }

    /// **L'image que le fil a tendue**, s'il y en a une.
    pub(crate) fn a_poser(&mut self) -> Option<Pyramide> {
        self.recevoir();
        self.prete.take()
    }

    /// **Joue, ou s'arrête sur l'image où elle est.**
    pub(crate) fn basculer(&mut self) {
        self.joue = !self.joue;
        let _ = self.ordres.send(Ordre::Jouer(self.joue));
    }

    /// **Rallume ou coupe le son.**
    pub(crate) fn basculer_le_son(&mut self) {
        self.son = !self.son;
        let _ = self.ordres.send(Ordre::Son(self.son));
    }

    pub(crate) fn joue(&self) -> bool {
        self.joue
    }

    pub(crate) fn son(&self) -> bool {
        self.son
    }

    /// **Son fil a fini** (la vidéo est allée au bout, ou ne se lit pas ici) et rien n'attend.
    pub(crate) fn fini(&self) -> bool {
        self.fini && self.prete.is_none()
    }
}

/// La vie du fil : ouvrir, jouer, obéir, tendre chaque image — jusqu'à la fin.
fn jouer(
    src: &str,
    objets: Option<&Objets>,
    envoi: &std::sync::mpsc::SyncSender<Pyramide>,
    a_faire: &Receiver<Ordre>,
    reveil: Option<&Reveil>,
) {
    #[cfg(target_os = "linux")]
    {
        use crate::renderer::atelier::admission;
        use crate::renderer::video::{LecteurVideo, Lieu};
        use std::time::Duration;
        let Some(lecteur) = Lieu::de(src, objets).and_then(|l| LecteurVideo::ouvrir(l, true))
        else {
            return;
        };
        lecteur.jouer(true);
        let porte = admission::de_la_machine();
        loop {
            loop {
                match a_faire.try_recv() {
                    Ok(Ordre::Jouer(oui)) => lecteur.jouer(oui),
                    Ok(Ordre::Son(oui)) => lecteur.couper_le_son(!oui),
                    Err(TryRecvError::Empty) => break,
                    // Le magasin l'a lâché : la vidéo a quitté l'écran.
                    Err(TryRecvError::Disconnected) => return,
                }
            }
            // Un vingtième de seconde au plus sans écouter les ordres.
            let t = match lecteur.suivante(Duration::from_millis(50)) {
                Ok(Some(t)) => t,
                Ok(None) => continue,
                Err(()) => return,
            };
            let pic = admission::pic(t.rgba.len() as u64, (t.largeur, t.hauteur));
            let pyramide =
                porte.pendant(pic, || Pyramide::depuis_rgba(t.largeur, t.hauteur, &t.rgba));
            let Some(p) = pyramide else { return };
            if envoi.send(p).is_err() {
                return;
            }
            if let Some(reveil) = reveil {
                reveil();
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (src, objets, envoi, a_faire, reveil);
    }
}
