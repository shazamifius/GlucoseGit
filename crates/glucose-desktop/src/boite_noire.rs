//! **La boîte noire** : ce qui se passe, écrit au fil de l'eau, et qui survit à la fin qu'elle
//! doit expliquer (fiche 45).
//!
//! # Pourquoi, en plus de la chronique
//!
//! La chronique agrège une session et l'écrit dans **un seul** fichier, que la session suivante
//! écrase : une session qui avait mal fini se perdait, sauf à copier son fichier avant de
//! relancer. Et une agrégation ne montre pas le temps : un téléphone qui chauffe coûte de plus en
//! plus cher pour le même travail, ce qu'aucun centile de fin de session ne dit. Sa demande du
//! 29/09 : voir *« où en est Glucose sur les très anciens téléphones, lorsqu'il commence à
//! chauffer, lorsqu'il s'éteint à cause de la batterie »*.
//!
//! # L'épisode, sans horloge
//!
//! L'unité est l'**épisode** : une suite d'images d'un même geste. Il s'écrit quand le geste
//! change — aucune période n'est choisie. L'état de la machine se relève chaque fois que le fil
//! d'écriture se réveille, et ne s'écrit que s'il a changé.
//!
//! # Survivre à la fin
//!
//! Chaque session a son fichier, auquel on ne fait qu'ajouter. Le fil d'écriture suit la règle du
//! scribe ([`crate::persist::scribe`]) : il dort tant que rien n'arrive, écrit tout ce qui attend,
//! **synchronise le disque**, et se rendort — la fréquence des synchronisations suit la vitesse du
//! disque, sans constante. Un programme qui plante, un appareil qui s'éteint : ce qui a été
//! synchronisé reste.
//!
//! # Ce qu'elle n'enregistre jamais
//!
//! Comme la chronique : aucun contenu ([`enregistrement`]). Et elle n'existe que dans
//! l'application : les épreuves ne l'ouvrent que dans un dossier à elles.

pub mod bilan;
pub mod enregistrement;
pub mod sondes;

use crate::chronique::{Chronique, Geste, Histogramme};
use enregistrement::Enregistrement;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::time::Instant;

/// Le dossier de la boîte noire, dans celui de l'application.
pub const DOSSIER: &str = "boite-noire";

/// Ce qu'on confie au fil d'écriture.
enum Ordre {
    Ligne(String),
    /// Répondre quand tout ce qui précède est sur le disque.
    Synchroniser(Sender<()>),
}

/// La boîte noire d'une session.
pub struct BoiteNoire {
    vers: Option<Sender<Ordre>>,
    fil: Option<JoinHandle<()>>,
    depart: Instant,
    episode: Option<Episode>,
    /// La pire image de la session jusqu'ici.
    pire_us: u32,
    chemin: PathBuf,
}

/// L'épisode en cours : un geste, depuis quand, et ce que ses images coûtent.
struct Episode {
    geste: Geste,
    debut_ms: u64,
    durees: Histogramme,
}

impl BoiteNoire {
    /// **Ouvre la boîte noire d'une session** dans `dossier` — celui de l'application —, après
    /// avoir relu la session d'avant, dont le bilan est rendu.
    pub fn ouvrir(dossier: &Path) -> std::io::Result<(Self, Option<bilan::Bilan>)> {
        let boite = dossier.join(DOSSIER);
        std::fs::create_dir_all(&boite)?;
        let demarrage = sondes::demarrage_de_l_appareil_ms();
        let precedente = bilan::ranger(&boite, demarrage);
        let epoque = sondes::maintenant_ms();
        let nom = format!("session-{epoque:020}-{}.jsonl", std::process::id());
        let chemin = boite.join(nom);
        let mut o = OpenOptions::new();
        o.create(true).append(true);
        // Tenue seule : un autre Glucose qui se lance la sait vivante, et ne la prend pas pour
        // une session interrompue.
        let fichier =
            crate::persist::verrou::ouvrir_seul(&mut o, &chemin).map_err(std::io::Error::other)?;
        let depart = Instant::now();
        let (vers, recu) = channel();
        let fil = std::thread::Builder::new()
            .name("boite-noire".into())
            .spawn(move || ecrire(fichier, recu, depart))?;
        let b = Self {
            vers: Some(vers),
            fil: Some(fil),
            depart,
            episode: None,
            pire_us: 0,
            chemin,
        };
        b.envoyer(Enregistrement::Debut {
            epoque_ms: epoque,
            version: env!("CARGO_PKG_VERSION"),
            systeme: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
            demarrage_appareil_ms: demarrage,
        });
        if let Some(p) = &precedente {
            b.envoyer(Enregistrement::Precedente {
                instant_ms: 0,
                fin: p.fin.nom(),
                appareil_redemarre: p.appareil_redemarre,
                duree_ms: p.duree_ms,
                batterie_pct: p.machine.batterie_pct,
                en_charge: p.machine.en_charge,
            });
        }
        Ok((b, precedente))
    }

    /// Le fichier de la session.
    pub fn chemin(&self) -> &Path {
        &self.chemin
    }

    fn instant_ms(&self) -> u64 {
        self.depart.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
    }

    fn envoyer(&self, e: Enregistrement) {
        if let Some(v) = &self.vers {
            // Un fil tombé ne peut plus rien écrire : l'envoi raté n'a rien de plus à dire.
            let _ = v.send(Ordre::Ligne(e.ligne()));
        }
    }

    /// **Une image rendue**, pendant ce geste, qui a coûté `duree_us`.
    pub fn image(&mut self, geste: Geste, duree_us: u32) {
        let maintenant = self.instant_ms();
        if self.episode.as_ref().is_some_and(|e| e.geste != geste) {
            self.clore_l_episode(maintenant);
        }
        self.episode
            .get_or_insert_with(|| Episode {
                geste,
                debut_ms: maintenant,
                durees: Histogramme::nouveau(),
            })
            .durees
            .ajouter(duree_us);
        if duree_us > self.pire_us {
            self.pire_us = duree_us;
            self.envoyer(Enregistrement::Pire {
                instant_ms: maintenant,
                duree_us,
                geste: geste.nom(),
            });
        }
    }

    fn clore_l_episode(&mut self, maintenant: u64) {
        if let Some(e) = self.episode.take() {
            self.envoyer(Enregistrement::Episode {
                instant_ms: e.debut_ms,
                duree_ms: maintenant.saturating_sub(e.debut_ms),
                geste: e.geste.nom(),
                images: e.durees.compte(),
                median_us: e.durees.centile(0.5),
                p99_us: e.durees.centile(0.99),
                pire_us: e.durees.pire(),
            });
        }
    }

    /// Attend que tout ce qui a été confié soit sur le disque.
    pub fn synchroniser(&self) {
        let (reponse, attente) = channel();
        if let Some(v) = &self.vers {
            if v.send(Ordre::Synchroniser(reponse)).is_ok() {
                let _ = attente.recv();
            }
        }
    }

    /// **La fin propre** : l'épisode en cours, puis la ligne qui dit que la session a fini
    /// comme on la ferme.
    pub fn clore(mut self) {
        let maintenant = self.instant_ms();
        self.clore_l_episode(maintenant);
        self.envoyer(Enregistrement::Fin {
            instant_ms: maintenant,
        });
    }

    /// **Le témoin des paniques** : chacune — de n'importe quel fil — laisse une ligne qui dit
    /// où elle a eu lieu. Pour l'application seulement : le crochet vaut pour tout le processus.
    pub fn temoigner_des_paniques(&self) {
        let Some(v) = &self.vers else {
            return;
        };
        if let Ok(mut t) = TEMOIN.lock() {
            *t = Some((v.clone(), self.depart));
        }
        static UNE_FOIS: std::sync::Once = std::sync::Once::new();
        UNE_FOIS.call_once(|| {
            let precedent = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                temoigner(info);
                precedent(info);
            }));
        });
    }
}

/// Ce que le témoin des paniques tient : où écrire, et depuis quand la session dure.
static TEMOIN: Mutex<Option<(Sender<Ordre>, Instant)>> = Mutex::new(None);

fn temoigner(info: &std::panic::PanicHookInfo<'_>) {
    let Ok(t) = TEMOIN.lock() else {
        return;
    };
    let (Some((v, depart)), Some(lieu)) = (t.as_ref(), info.location()) else {
        return;
    };
    let instant = depart.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let principal = std::thread::current().name() == Some("main");
    let ligne = enregistrement::ligne_de_panique(instant, lieu.file(), lieu.line(), principal);
    let _ = v.send(Ordre::Ligne(ligne));
}

impl Drop for BoiteNoire {
    fn drop(&mut self) {
        let maintenant = self.instant_ms();
        self.clore_l_episode(maintenant);
        // Le témoin tient une copie de l'envoyeur : tant qu'elle vit, le fil ne s'arrêterait
        // jamais. On la reprend d'abord.
        if let Ok(mut t) = TEMOIN.lock() {
            if t.as_ref().is_some_and(|(_, d)| *d == self.depart) {
                *t = None;
            }
        }
        // Lâcher l'envoyeur réveille le fil une dernière fois : il écrit ce qui reste,
        // synchronise, et s'arrête.
        self.vers = None;
        if let Some(f) = self.fil.take() {
            let _ = f.join();
        }
    }
}

/// **Le fil d'écriture.** Il dort tant que rien n'arrive ; réveillé, il écrit tout ce qui
/// attend, l'état de la machine s'il a changé, puis synchronise le disque.
fn ecrire(mut fichier: File, recu: Receiver<Ordre>, depart: Instant) {
    let mut machine = None;
    while let Ok(premier) = recu.recv() {
        let mut lot = String::new();
        let mut attendent = Vec::new();
        for ordre in std::iter::once(premier).chain(recu.try_iter()) {
            match ordre {
                Ordre::Ligne(l) => lot.push_str(&l),
                Ordre::Synchroniser(r) => attendent.push(r),
            }
        }
        let etat = sondes::etat_de_la_machine();
        if machine != Some(etat) {
            machine = Some(etat);
            let instant_ms = depart.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            lot.push_str(
                &Enregistrement::Machine {
                    instant_ms,
                    batterie_pct: etat.batterie_pct,
                    en_charge: etat.en_charge,
                }
                .ligne(),
            );
        }
        if fichier.write_all(lot.as_bytes()).is_err() {
            return;
        }
        let _ = fichier.sync_data();
        for r in attendent {
            let _ = r.send(());
        }
    }
}

impl Chronique {
    /// **Ouvre la boîte noire** de la session dans le dossier de l'application, et rend le bilan
    /// de la session d'avant. Appelée par l'application seule : les épreuves créent des
    /// centaines de chroniques, et aucune ne doit écrire chez l'utilisateur.
    pub fn ouvrir_la_boite_noire(&mut self, dossier: &Path) -> Option<bilan::Bilan> {
        match BoiteNoire::ouvrir(dossier) {
            Ok((b, precedente)) => {
                self.boite = Some(b);
                precedente
            }
            Err(e) => {
                eprintln!("[Glucose] boîte noire non ouverte : {e}");
                None
            }
        }
    }

    /// La boîte noire de la session, si elle est ouverte.
    pub fn boite_noire(&self) -> Option<&BoiteNoire> {
        self.boite.as_ref()
    }

    /// **La fin propre** de la boîte noire, à la fermeture de l'application.
    pub fn clore_la_boite_noire(&mut self) {
        if let Some(b) = self.boite.take() {
            b.clore();
        }
    }
}

#[cfg(test)]
mod tests;
