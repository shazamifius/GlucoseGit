//! **CLIENT-1 — participer à un canevas partagé** (fiche 117), le greffier aveugle de la fiche 113
//! vu du client.
//!
//! # Ce que le client montre : la salle, plus ses propres gestes
//!
//! La salle numérote les gestes sans les lire ; chaque Glucose les rejoue dans son ordre. Un geste
//! s'applique pourtant **tout de suite** chez celui qui le fait, puis part, et revient numéroté —
//! entre-temps il est **en attente**. Le canevas montré est donc toujours *la salle, plus mes gestes
//! en attente* : la « server reconciliation » (Replicache, Weidner 2024-2025). Quand le geste d'un
//! autre arrive, le client défait ses gestes en attente, applique celui de l'autre, puis refait les
//! siens — tout **par le nom** (PARTAGE-1), dans l'ordre que la salle donne à tous : tous convergent.
//! Quand c'est le sien qui revient, il n'y a rien à faire : il était déjà là.
//!
//! # `Ctrl+Z` défait **mon** dernier geste
//!
//! Le dernier geste de la pile peut être celui d'un autre (Liveblocks, Figma, Google Slides : chacun
//! ne défait que les siens). La pile est **une pile par main, par le nom** : défaire applique
//! l'inverse de mon geste, qui part à son tour comme un geste ; ce que l'annulation a **vraiment**
//! fait va sur la pile de rétablissement (Figma : *« an undo operation modifies redo history at the
//! time of the undo »*) — défaire, puis tout rétablir, ne change rien. Un nœud qu'un autre a retiré
//! entre-temps : **rien** ne se passe (Figma, Google Slides).
//!
//! # Jamais pendant un geste ouvert, jamais avant ce qui part
//!
//! Un glisser écrit dans le document à chaque mouvement, par les rangs, et doit pouvoir s'abandonner
//! (`Échap`) : ce qui arrive pendant ce temps attend qu'il se ferme. Et **appliquer ce qui arrive
//! commence par faire partir ce qui part** : un geste fait ici mais pas encore en attente ne serait
//! pas rejoué par-dessus celui de l'autre — la salle le mettrait après, ce Glucose avant, et les deux
//! divergeraient. Même règle pour le disque ([`Store::ecrits_a_consigner`]) : aucun geste ne quitte
//! la file sans être parti.

use super::journal::partage::{GestePartage, Rencontre};
use super::journal::{Ecrit, Main, Transaction};
use super::Store;
use crate::types::Project;
use std::collections::VecDeque;

/// **Ce qui part** : la base du canevas, une fois, puis des gestes.
#[derive(Debug, Clone, PartialEq)]
pub enum Envoi {
    Base(Box<Project>),
    Geste(GestePartage),
}

/// **Une lettre à envoyer** : son nom (`<marque>-<compteur>`), sa main, et ce qu'elle porte.
#[derive(Debug, Clone, PartialEq)]
pub struct Sortant {
    pub id: String,
    pub main: Main,
    pub envoi: Envoi,
}

/// **Un geste arrivé de la salle**, entier : son numéro dans l'ordre de la salle, son nom, sa main
/// (déjà telle qu'elle signe ici — un participant, ou l'agent d'un participant).
#[derive(Debug, Clone, PartialEq)]
pub struct Arrivee {
    pub n: u64,
    pub id: String,
    pub main: Main,
    pub geste: GestePartage,
}

/// **L'état d'un participant.**
#[derive(Debug, Clone, Default)]
pub struct Participation {
    /// Sa marque (ID-2), dans le nom de ses nœuds et de ses lettres.
    pub marque: String,
    /// Peut-il écrire ? Le code « lire » ne fait que regarder.
    pub ecrivain: bool,
    /// Le numéro de la salle du dernier geste appliqué ici.
    pub confirme: u64,
    compteur: u64,
    /// Mes lettres parties et pas encore revenues numérotées ; la base n'a pas de geste.
    en_attente: VecDeque<(String, Option<GestePartage>)>,
    sortants: Vec<Sortant>,
    /// Ce qui défait mes gestes, et ce qui refait ce que j'ai défait — par le nom.
    defaire: Vec<GestePartage>,
    refaire: Vec<GestePartage>,
    /// L'étiquette du sommet de `defaire` (FONDRE-145).
    sommet: Option<u64>,
    arrivees: VecDeque<Arrivee>,
    /// Les gestes d'un lecteur, défaits aussitôt depuis la dernière lecture : ce qu'on lui dit.
    pub defaits_au_lecteur: usize,
}

impl Participation {
    fn nouvelle(marque: &str, ecrivain: bool, confirme: u64) -> Self {
        Self {
            marque: marque.to_string(),
            ecrivain,
            confirme,
            ..Self::default()
        }
    }

    /// Ce qui défait mon geste entre dans la pile — fondu dans le sommet s'il est un pas de la même
    /// saisie (FONDRE-145, fiche 145) : on défait alors le dernier pas d'abord, son inverse devant.
    fn empiler(&mut self, inverse: GestePartage, etiquette: Option<u64>) {
        let meme = etiquette.is_some() && etiquette == self.sommet;
        match self.defaire.last_mut().filter(|_| meme) {
            Some(sommet) => {
                let anciens = std::mem::replace(&mut sommet.editions, inverse.editions);
                sommet.editions.extend(anciens);
            }
            None => {
                self.defaire.push(inverse);
                self.sommet = etiquette;
            }
        }
        self.refaire.clear();
    }

    /// Le nom d'une lettre neuve : la marque, et un compteur qui ne revient jamais en arrière.
    fn nouvel_id(&mut self) -> String {
        self.compteur += 1;
        format!("{}-{}", self.marque, self.compteur)
    }

    /// Un geste fait ici part : en attente jusqu'à son retour, puis à la salle.
    fn envoyer(&mut self, main: Main, geste: GestePartage) {
        let id = self.nouvel_id();
        self.en_attente.push_back((id.clone(), Some(geste.clone())));
        self.sortants.push(Sortant {
            id,
            main,
            envoi: Envoi::Geste(geste),
        });
    }

    /// Combien de mes gestes attendent leur numéro.
    pub fn en_attente(&self) -> usize {
        self.en_attente.len()
    }
}

/// **Rejoue l'ordre de la salle** : mes gestes en attente défaits, le sien appliqué, les miens
/// refaits — par le nom. Rend tout ce qui a été appliqué, en une transaction, et les rencontres.
fn rebaser(
    projet: &mut Project,
    en_attente: &VecDeque<(String, Option<GestePartage>)>,
    geste: &GestePartage,
) -> (Transaction, Vec<Rencontre>) {
    let miens = || en_attente.iter().filter_map(|(_, g)| g.as_ref());
    let mut tx = Transaction::default();
    for g in miens().rev() {
        tx.edits
            .extend(g.inverse().appliquer(projet).realisee.edits);
    }
    let a = geste.appliquer(projet);
    tx.edits.extend(a.realisee.edits);
    let mut rencontres = a.rencontres;
    for g in miens() {
        let a = g.appliquer(projet);
        tx.edits.extend(a.realisee.edits);
        rencontres.extend(a.rencontres);
    }
    (tx, rencontres)
}

impl Store {
    /// **Partage ce canevas** sous cette marque : la base part la première, puis chaque geste.
    pub fn partager_ce_canevas(&mut self, marque: &str) -> bool {
        if !self.participer(marque) {
            return false;
        }
        let mut p = Participation::nouvelle(marque, true, 0);
        let id = p.nouvel_id();
        p.en_attente.push_back((id.clone(), None));
        p.sortants.push(Sortant {
            id,
            main: Main::Humain,
            envoi: Envoi::Base(Box::new(self.project.clone())),
        });
        self.entrer(p);
        true
    }

    /// **Rejoint un canevas** dont la base vient d'être chargée (`load_project`), arrivée au numéro
    /// `n` de la salle ; les gestes suivants viendront.
    pub fn rejoindre(&mut self, marque: &str, ecrivain: bool, n: u64) -> bool {
        if !self.participer(marque) {
            return false;
        }
        self.entrer(Participation::nouvelle(marque, ecrivain, n));
        true
    }

    /// **Reprend une participation mise de côté** (le voyage de la Time Machine) : rien ne s'est
    /// perdu, ce qui est arrivé entre-temps attend dans le fil.
    pub fn reprendre_le_partage(&mut self, p: Participation) {
        self.participant = Some(p.marque.clone());
        self.entrer(p);
    }

    fn entrer(&mut self, p: Participation) {
        self.journal.partager(true);
        self.partage = Some(p);
    }

    /// **Quitte le partage** : le canevas redevient à soi seul, sa pile repart de rien.
    pub fn quitter_le_partage(&mut self) {
        if self.partage.take().is_some() {
            self.journal.partager(false);
            self.participant = None;
        }
    }

    /// **La file à écrire au disque** (JRN-5), une fois parti ce qui part.
    pub fn ecrits_a_consigner(&mut self) -> Vec<Ecrit> {
        self.partager_les_gestes();
        self.journal.prendre_les_ecrits_signes()
    }

    /// **Nomme les gestes faits ici depuis la dernière fois** et les fait partir. Un lecteur (son
    /// code ne fait que regarder) les voit se défaire aussitôt.
    pub fn partager_les_gestes(&mut self) {
        let Store {
            partage: Some(p),
            journal,
            project,
            ..
        } = self
        else {
            return;
        };
        let mut refuses = 0;
        for (geste, main, etiquette) in journal.nommer_ce_qui_part(project) {
            if geste.editions.is_empty() {
                continue;
            }
            if !p.ecrivain {
                // Un filet : le journal refuse déjà ce qu'un lecteur ferait (LECTEUR-1) ; ce qui
                // y échapperait ne part pas, et se défait aussitôt.
                let defait = geste.inverse().appliquer(project).realisee;
                journal.publier_tel(defait, Main::Humain);
                refuses += 1;
                continue;
            }
            p.empiler(geste.inverse(), etiquette);
            p.envoyer(main, geste);
        }
        if refuses > 0 {
            p.defaits_au_lecteur += refuses;
            self.settle_navigation();
            self.bump_version();
        }
    }

    /// Les lettres à sceller et envoyer, dans l'ordre.
    pub fn prendre_les_sortants(&mut self) -> Vec<Sortant> {
        self.partage
            .as_mut()
            .map(|p| std::mem::take(&mut p.sortants))
            .unwrap_or_default()
    }

    /// **Un geste est arrivé** de la salle : il attend [`Store::appliquer_les_arrivees`].
    pub fn recevoir(&mut self, arrivee: Arrivee) {
        if let Some(p) = self.partage.as_mut() {
            p.arrivees.push_back(arrivee);
        }
    }

    /// **Applique ce qui est arrivé**, dans l'ordre de la salle — sauf pendant un geste ouvert. Rend
    /// les rencontres : où un autre et moi avons touché la même chose.
    pub fn appliquer_les_arrivees(&mut self) -> Vec<Rencontre> {
        self.partager_les_gestes();
        let Store {
            partage: Some(p),
            journal,
            project,
            ..
        } = self
        else {
            return Vec::new();
        };
        if journal.is_open() {
            return Vec::new();
        }
        let mut rencontres = Vec::new();
        let mut change = false;
        let mut retires = Transaction::default();
        while let Some(a) = p.arrivees.pop_front() {
            if a.n <= p.confirme {
                continue; // Déjà appliqué : la salle le répète, on ne le rejoue pas.
            }
            p.confirme = a.n;
            if p.en_attente.front().is_some_and(|(id, _)| *id == a.id) {
                p.en_attente.pop_front(); // Le mien, revenu : il était déjà là.
                continue;
            }
            let (tx, r) = rebaser(project, &p.en_attente, &a.geste);
            rencontres.extend(r);
            retires.edits.extend(tx.edits.iter().cloned());
            journal.publier_tel(tx, a.main);
            change = true;
        }
        super::undo::oublier_les_retires(self, &retires);
        if change {
            self.settle_navigation();
            self.bump_version();
        }
        rencontres
    }

    /// **`Ctrl+Z` et `Ctrl+Maj+Z` partagés** : défaire mon dernier geste, ou refaire ce que j'ai
    /// défait. Rend `false` s'il n'y a rien — ou si ce qu'il touchait n'est plus là.
    pub(super) fn pas_partage(&mut self, en_arriere: bool) -> bool {
        // Le geste qui vient d'être fait doit être sur la pile avant qu'on la lise.
        self.partager_les_gestes();
        let Store {
            partage: Some(p),
            journal,
            project,
            ..
        } = self
        else {
            return false;
        };
        if journal.is_open() {
            return false;
        }
        let pris = if en_arriere {
            p.defaire.pop()
        } else {
            p.refaire.pop()
        };
        let Some(geste) = pris else {
            return false;
        };
        p.sommet = None;
        let realisee = geste.appliquer(project).realisee;
        if realisee.is_empty() {
            return false;
        }
        let fait = realisee.a_partager(project);
        let retour = fait.inverse();
        if en_arriere {
            p.refaire.push(retour);
        } else {
            p.defaire.push(retour);
        }
        journal.publier_tel(realisee.clone(), Main::Humain);
        p.envoyer(Main::Humain, fait);
        super::undo::oublier_les_retires(self, &realisee);
        self.settle_navigation();
        self.bump_version();
        true
    }

    /// Combien de mes gestes je peux défaire, et refaire, sur ce canevas partagé.
    pub(super) fn profondeurs_partagees(&self) -> Option<(usize, usize)> {
        self.partage
            .as_ref()
            .map(|p| (p.defaire.len(), p.refaire.len()))
    }
}

#[cfg(test)]
#[path = "participation_tests.rs"]
mod tests;
