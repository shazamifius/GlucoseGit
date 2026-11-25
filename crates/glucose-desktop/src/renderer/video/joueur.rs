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

/// Ce que le magasin peut dire au fil. Hors Linux et Android, aucun lecteur ne l'écoute encore
/// (leur porte suit, fiche 169) : l'attente tombera le jour où l'un l'écoutera.
#[cfg_attr(
    not(any(target_os = "linux", target_os = "android")),
    expect(dead_code)
)]
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
    en_boucle: bool,
}

impl JoueurVideo {
    /// **Lance le fil** de cette vidéo : elle joue tout de suite, le son coupé (sa réponse du
    /// 25/11 : coupé par défaut, rallumé par son icône) ; `en_boucle`, elle repart du début au
    /// bout (la lecture automatique, VIDEO-2).
    pub(crate) fn lancer(
        src: String,
        objets: Option<Arc<Objets>>,
        reveil: Option<Reveil>,
        en_boucle: bool,
    ) -> Self {
        let (envoi, recu) = sync_channel(1);
        let (ordres, a_faire) = channel();
        std::thread::spawn(move || {
            crate::plateforme::priorite::ceder_au_fil_qui_dessine();
            jouer(
                (&src, objets.as_deref(), en_boucle),
                &envoi,
                &a_faire,
                reveil.as_ref(),
            );
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
            en_boucle,
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

    /// Elle joue en boucle (la lecture automatique).
    pub(crate) fn en_boucle(&self) -> bool {
        self.en_boucle
    }

    /// **Son fil a fini** (la vidéo est allée au bout, ou ne se lit pas ici) et rien n'attend.
    pub(crate) fn fini(&self) -> bool {
        self.fini && self.prete.is_none()
    }
}

/// La vie du fil : ouvrir, jouer, obéir, tendre chaque image — jusqu'à la fin.
fn jouer(
    (src, objets, en_boucle): (&str, Option<&Objets>, bool),
    envoi: &std::sync::mpsc::SyncSender<Pyramide>,
    a_faire: &Receiver<Ordre>,
    reveil: Option<&Reveil>,
) {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        use crate::renderer::atelier::admission;
        use crate::renderer::video::{Fin, LecteurVideo, Lieu};
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
                Err(Fin::AuBout) if en_boucle && lecteur.revenir_au_debut() => continue,
                Err(_) => return,
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
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    {
        let _ = (src, objets, en_boucle, envoi, a_faire, reveil);
    }
}

/// **La lecture automatique joue en boucle** (VIDEO-2, fiche 170) : sur la vraie vidéo de
/// téléphone en miniature (dix images en une seconde), le joueur en boucle ne finit jamais ; l'autre
/// finit au bout.
#[cfg(all(test, target_os = "linux"))]
mod epreuves {
    use super::*;
    use std::time::{Duration, Instant};

    const ROUGE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/donnees/rouge-64x48.mp4");

    /// Joue pendant `duree` ; rend le nombre d'images tendues, et si le fil a fini.
    fn jouer_pendant(en_boucle: bool, duree: Duration) -> Option<(usize, bool)> {
        let gst = crate::renderer::video::gst::gst().is_some();
        assert!(
            gst || crate::renderer::video::gst::GREFFONS.is_none(),
            "GStreamer manque alors que nix develop le porte"
        );
        if !gst {
            return None;
        }
        let mut joueur = JoueurVideo::lancer(ROUGE.to_string(), None, None, en_boucle);
        let (debut, mut images) = (Instant::now(), 0);
        while debut.elapsed() < duree && !joueur.fini() {
            images += usize::from(joueur.a_poser().is_some());
            std::thread::sleep(Duration::from_millis(5));
        }
        Some((images, joueur.fini()))
    }

    #[test]
    fn video_2_en_boucle_le_joueur_repart_au_bout() {
        let Some((images, fini)) = jouer_pendant(true, Duration::from_millis(2600)) else {
            return;
        };
        assert!(!fini, "la boucle a fini");
        assert!(
            images > 12,
            "{images} images en plus de deux tours d'une seconde"
        );
    }

    #[test]
    fn video_2_sans_boucle_le_joueur_finit_au_bout() {
        let Some((_, fini)) = jouer_pendant(false, Duration::from_secs(5)) else {
            return;
        };
        assert!(fini, "jamais finie");
    }
}
