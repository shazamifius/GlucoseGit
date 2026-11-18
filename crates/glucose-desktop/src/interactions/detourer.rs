//! **Le détourage, vu du geste** (DETOURER-1 à 3, fiche 121) : « Détourer » (la forme principale,
//! par l'IA) et « Retirer le fond uni » (exact, sans IA). Le calcul vit dans le noyau
//! ([`glucose_core::detourage`]) ; ce module lit la sélection, attend les pixels, confie le
//! travail à un fil, et pose le résultat.
//!
//! # Trois temps, l'écran vivant
//!
//! 1. **Les pixels natifs** de chaque image choisie sont attendus comme ceux de « Retirer les
//!    bordures » (ETAGES-1) : réclamés au magasin, l'écran continue de vivre.
//! 2. **Un fil** détoure les images l'une après l'autre (chacune prend déjà tous les cœurs qu'on
//!    lui laisse), puis réveille la boucle — sans qu'elle ait à repasser voir. Le curseur dit,
//!    pendant ce temps, qu'un travail se fait (`Progress`, le signe du système) : aucun message.
//! 3. **Le résultat se pose d'un seul geste**, une seule entrée d'annulation : chaque image reçoit
//!    une clé neuve, dont le magasin adopte les pixels et promet les octets au scribe (COLLER-3).
//!    Les octets d'avant restent dans le document : `Ctrl+Z` rend l'image d'origine, et une autre
//!    pose de la même photo n'a pas changé. Le co-working envoie les octets neufs avec le geste
//!    (B1, fiche 119).
//!
//! Une image changée, retirée ou verrouillée pendant le calcul n'est pas touchée : le geste ne
//! pose que ce qu'il a calculé sur ce qui est encore là.

use crate::app::GlucoseApp;
use crate::persist::objets::{Promesse, Source};
use std::sync::mpsc;

/// Ce que le geste retire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Maniere {
    /// Tout sauf la forme principale, par le modèle (DETOURER-1, 2).
    FormePrincipale,
    /// La couleur du bord, exactement (DETOURER-3).
    FondUni,
}

/// Une image à détourer : où elle est, et la clé de ses octets au moment du geste.
struct Cible {
    id: String,
    src: String,
}

/// Une image détourée, prête à se poser.
struct Fait {
    id: String,
    src: String,
    rvba: Vec<u8>,
    taille: (u32, u32),
}

/// **Un détourage en cours** : d'abord les originaux attendus, puis le fil.
pub struct Detourage {
    board: String,
    maniere: Maniere,
    cibles: Vec<Cible>,
    fil: Option<mpsc::Receiver<Option<Fait>>>,
    faits: Vec<Fait>,
    rendus: usize,
}

impl Detourage {
    /// Le fil travaille : la boucle n'a pas à repasser, il la réveillera.
    pub fn au_travail(&self) -> bool {
        self.fil.is_some()
    }
}

impl GlucoseApp {
    /// **Détoure les images choisies**, ou retire leur fond uni — le menu d'une image, la palette.
    /// Un seul détourage à la fois : le suivant attend que celui-ci soit posé.
    pub(crate) fn detourer_la_selection(&mut self, maniere: Maniere) {
        if self.attentes.detourage.is_some() {
            return;
        }
        let board = self.store.project.active_board_id.clone();
        let cibles: Vec<Cible> = self
            .store
            .selected_image_ids
            .iter()
            .filter_map(|id| self.store.image(&board, id))
            .filter(|img| !img.locked)
            .filter_map(|img| {
                Some(Cible {
                    id: img.id.clone(),
                    src: img.src.clone()?,
                })
            })
            .collect();
        if cibles.is_empty() {
            return;
        }
        self.attentes.detourage = Some(Detourage {
            board,
            maniere,
            cibles,
            fil: None,
            faits: Vec::new(),
            rendus: 0,
        });
        self.update_cursor();
        self.poursuivre_le_detourage();
    }

    /// **Fait avancer le détourage** — à chaque image et à chaque réveil : lance le fil quand tous
    /// les originaux sont tenus, puis pose le résultat quand le fil a tout rendu.
    pub(crate) fn poursuivre_le_detourage(&mut self) {
        let Some(d) = self.attentes.detourage.as_ref() else {
            return;
        };
        if d.fil.is_none() {
            self.lancer_le_fil_du_detourage();
            return;
        }
        let Some(d) = self.attentes.detourage.as_mut() else {
            return;
        };
        let attendus = d.cibles.len();
        if let Some(fil) = &d.fil {
            while let Ok(rendu) = fil.try_recv() {
                d.rendus += 1;
                d.faits.extend(rendu);
            }
        }
        if d.rendus < attendus {
            return;
        }
        if let Some(fini) = self.attentes.detourage.take() {
            self.poser_le_detourage(fini);
        }
        self.update_cursor();
    }

    /// Les pixels natifs de chaque cible, s'ils sont tous tenus — sinon redemandés, et l'on
    /// repassera. Une image que le magasin ne connaît pas n'a pas de pixels : elle sort du lot.
    fn lancer_le_fil_du_detourage(&mut self) {
        let Some(d) = self.attentes.detourage.as_mut() else {
            return;
        };
        let magasin = &mut self.renderer.magasin;
        let etats: Vec<Option<bool>> = d
            .cibles
            .iter()
            .map(|c| magasin.original_tenu(&c.src))
            .collect();
        if etats.contains(&Some(false)) {
            return;
        }
        let mut etats = etats.into_iter();
        d.cibles.retain(|_| etats.next().flatten().is_some());
        let travail: Vec<(Cible, Vec<u8>, (u32, u32))> = std::mem::take(&mut d.cibles)
            .into_iter()
            .filter_map(|c| {
                let natif = magasin.cache.get(&c.src)?.pyramide.native()?;
                let taille = (natif.width(), natif.height());
                Some((c, natif.data().to_vec(), taille))
            })
            .collect();
        d.cibles = travail
            .iter()
            .map(|(c, ..)| Cible {
                id: c.id.clone(),
                src: c.src.clone(),
            })
            .collect();
        let (envoi, recu) = mpsc::channel();
        let reveil = self.lancement.reveil.clone();
        let maniere = d.maniere;
        std::thread::Builder::new()
            .name("glucose-detourage".into())
            .spawn(move || {
                for (cible, premul, taille) in travail {
                    let fait = detourer_une_image(maniere, cible, premul, taille);
                    if envoi.send(fait).is_err() {
                        return;
                    }
                    if let Some(proxy) = &reveil {
                        let _ = proxy.send_event(());
                    }
                }
            })
            .ok();
        d.fil = Some(recu);
        if d.cibles.is_empty() {
            self.poursuivre_le_detourage();
        }
    }

    /// **Pose les images détourées d'un seul geste.**
    fn poser_le_detourage(&mut self, d: Detourage) {
        let horloge = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        self.store.begin_live_edit();
        for (k, fait) in d.faits.into_iter().enumerate() {
            let encore = self
                .store
                .image(&d.board, &fait.id)
                .is_some_and(|img| !img.locked && img.src.as_deref() == Some(fait.src.as_str()));
            if !encore {
                continue;
            }
            let cle = format!("detouree:{horloge}:{k}");
            let (promesse, parole) = Promesse::nouvelle();
            self.disque.objets.poser(&cle, Source::Promise(promesse));
            if !self
                .renderer
                .magasin
                .adopter(&cle, fait.rvba, fait.taille, parole)
            {
                continue;
            }
            self.store
                .update_image(&d.board, &fait.id, |img| img.src = Some(cle));
        }
        self.store.end_live_edit();
        self.mark_dirty();
    }
}

/// **Une image détourée**, sur le fil : ses pixels natifs (prémultipliés) rendus droits, puis le
/// geste du noyau. `None` : rien à poser (un fond déjà transparent, un modèle illisible).
fn detourer_une_image(
    maniere: Maniere,
    cible: Cible,
    mut rvba: Vec<u8>,
    taille: (u32, u32),
) -> Option<Fait> {
    depremultiplier(&mut rvba);
    let (l, h) = (taille.0 as usize, taille.1 as usize);
    let rvba = match maniere {
        Maniere::FondUni => glucose_core::detourage::fond_uni::retirer(&rvba, (l, h))?,
        Maniere::FormePrincipale => {
            let modele = glucose_core::detourage::embarque()?;
            let fils = glucose_core::detourage::fils_disponibles();
            glucose_core::detourage::detourer(modele, &rvba, (l, h), fils)
        }
    };
    Some(Fait {
        id: cible.id,
        src: cible.src,
        rvba,
        taille,
    })
}

/// Les pixels du magasin sont prémultipliés ; le détourage lit des couleurs droites.
fn depremultiplier(rvba: &mut [u8]) {
    for p in rvba.as_chunks_mut::<4>().0 {
        let a = u16::from(p[3]);
        if a != 0 && a != 255 {
            for c in &mut p[..3] {
                *c = ((u16::from(*c) * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
}

#[cfg(test)]
#[path = "detourer/tests.rs"]
mod tests;
