//! **L'entracte** : où va le temps quand Glucose ne dessine pas.
//!
//! # Le poste que trois sessions ont nommé sans jamais le voir
//!
//! Le rythme mesure, depuis la fiche 19, ce que l'application passe « à ne pas dessiner » :
//! l'intervalle entre une présentation et le début du rendu suivant. Il l'a chiffré à **185 ms
//! au p99 et 510 ms au pire** (fiche 24 § 12), puis **717 ms** (fiche 23 § 7), puis **748,7 ms
//! sur un gel de 763** (fiche 25 § 9.4). À chaque fois la même conclusion : *hors de tout code
//! de rendu, jamais instrumenté*.
//!
//! Deux hypothèses ont été écrites pour l'expliquer, et **la mesure a démenti les deux** — mes
//! propres instances restées vivantes, puis une surface qui refusait les images. La seconde
//! avait un compteur, et ce compteur est resté à zéro.
//!
//! Ce module ne fait aucune hypothèse. Il **découpe** cet intervalle et dit, pour chaque
//! morceau, à qui il appartient.
//!
//! # Les cinq postes, et pourquoi ils se somment exactement
//!
//! Entre deux images, le contrôle est à tout instant à **un seul** endroit :
//!
//! | poste | qui tient le contrôle |
//! |---|---|
//! | `systeme` | winit et Windows : le sommeil demandé, et la file de messages |
//! | `main` | nos gestionnaires d'événements — souris, clavier, molette |
//! | `depot` | ce qu'un glisser-déposer a apporté, et qu'on pose dans le document |
//! | `carte` | la réouverture de la carte graphique, quand l'arbitre en change |
//! | `entretien` | le reste de `about_to_wait` : l'empreinte, la chronique, les réveils |
//!
//! **Leur somme vaut l'entracte entier, au bit près**, et un test l'exige. C'est la garantie
//! qui manquait aux quatre marques mal posées de ce dépôt — `occlusion`, `recolte`, `blit`,
//! `minimap` —, dont chacune absorbait ce qui la précédait et a désigné le mauvais coupable
//! pendant plusieurs sessions. Ici, ce qu'un poste ne prend pas, un autre le porte : rien ne
//! peut disparaître, et rien ne peut compter deux fois.
//!
//! # Le pire entracte est gardé décomposé, et c'est lui qui répond
//!
//! Une distribution dit ce qui arrive d'ordinaire ; elle ne dit pas ce qu'**un** gel de sept
//! dixièmes de seconde contenait. Le pire entracte de la session garde donc ses cinq parts
//! telles quelles — c'est une ligne de rapport, et c'est la seule qui nomme un gel.

use super::histogramme::Histogramme;
use std::time::{Duration, Instant};

/// À qui appartient une tranche de temps entre deux images.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Poste {
    /// winit et Windows : le sommeil demandé, la file de messages, tout ce qu'on ne commande
    /// pas. C'est l'état par défaut d'un entracte, et il le reste tant qu'on ne nous rend pas
    /// la main.
    Systeme,
    /// Nos gestionnaires d'événements : ce qu'un mouvement de souris, une touche ou un cran
    /// de molette nous coûte avant qu'une image ne soit demandée.
    Main,
    /// Poser dans le document ce qu'un glisser-déposer a apporté — décodage compris.
    Depot,
    /// Lâcher une carte graphique et en ouvrir une autre sur la même fenêtre (ARBITRE-1).
    Carte,
    /// Le reste de `about_to_wait` : l'empreinte du processus, la chronique sauvée, le calcul
    /// du prochain réveil.
    Entretien,
}

impl Poste {
    /// Tous les postes, dans l'ordre de leur indice.
    pub const TOUS: [Poste; 5] = [
        Poste::Systeme,
        Poste::Main,
        Poste::Depot,
        Poste::Carte,
        Poste::Entretien,
    ];

    /// Combien il y en a.
    pub const COMBIEN: usize = Self::TOUS.len();

    /// Sa place dans les tableaux.
    fn indice(self) -> usize {
        match self {
            Self::Systeme => 0,
            Self::Main => 1,
            Self::Depot => 2,
            Self::Carte => 3,
            Self::Entretien => 4,
        }
    }

    /// Ce que le rapport en dit, en français et sans jargon.
    ///
    /// L'utilisateur ne sait pas ce qu'est une pompe de messages, et il l'a dit. « attendre
    /// Windows » l'aide ; `message pump` ne lui apprend rien.
    pub fn nom(self) -> &'static str {
        match self {
            Self::Systeme => "attendre Windows",
            Self::Main => "ecouter la main",
            Self::Depot => "poser un depot",
            Self::Carte => "changer de carte",
            Self::Entretien => "entretien",
        }
    }
}

/// Ce que chaque poste a pris pendant un entracte.
type Parts = [Duration; Poste::COMBIEN];

/// Où va le temps entre deux images, sur toute une session.
#[derive(Debug)]
pub struct Entracte {
    /// Le début de la tranche en cours, ou `None` hors d'un entracte.
    marque: Option<Instant>,
    /// Le poste auquel la tranche en cours s'impute.
    poste: Poste,
    /// Ce que l'entracte en cours a donné à chaque poste.
    en_cours: Parts,
    /// La distribution de chaque poste, en microsecondes.
    par_poste: [Histogramme; Poste::COMBIEN],
    /// La distribution de l'entracte entier, pour vérifier que les parts s'y retrouvent.
    totaux: Histogramme,
    /// Le pire entracte de la session, décomposé.
    pire: Parts,
    /// Ce que ce pire entracte valait en tout.
    pire_total: Duration,
    /// À quelle milliseconde de la session il est tombé.
    pire_a_ms: u32,
    /// L'instant de la première présentation, pour dater le pire.
    origine: Option<Instant>,
}

impl Default for Entracte {
    fn default() -> Self {
        Self::nouveau()
    }
}

impl Entracte {
    pub fn nouveau() -> Self {
        Self {
            marque: None,
            poste: Poste::Systeme,
            en_cours: [Duration::ZERO; Poste::COMBIEN],
            par_poste: std::array::from_fn(|_| Histogramme::nouveau()),
            totaux: Histogramme::nouveau(),
            pire: [Duration::ZERO; Poste::COMBIEN],
            pire_total: Duration::ZERO,
            pire_a_ms: 0,
            origine: None,
        }
    }

    /// **L'image vient de partir à l'écran** : l'entracte s'ouvre, et le contrôle retourne au
    /// système.
    ///
    /// C'est le même instant que celui où le rythme note sa présentation, et il le faut : les
    /// deux mesurent le même intervalle par ses deux bouts, et les séparer les ferait diverger.
    pub fn ouvrir(&mut self, maintenant: Instant) {
        self.origine.get_or_insert(maintenant);
        self.en_cours = [Duration::ZERO; Poste::COMBIEN];
        self.marque = Some(maintenant);
        self.poste = Poste::Systeme;
    }

    /// **Le contrôle passe à ce poste** : ce qui précède revient au poste qu'on quitte.
    ///
    /// Hors d'un entracte — avant la première image, ou pendant le rendu — l'appel ne fait
    /// rien : il n'y a pas d'intervalle à découper.
    pub fn imputer(&mut self, maintenant: Instant, poste: Poste) {
        let Some(depuis) = self.marque else {
            return;
        };
        self.en_cours[self.poste.indice()] += maintenant.saturating_duration_since(depuis);
        self.marque = Some(maintenant);
        self.poste = poste;
    }

    /// **Le rendu commence** : l'entracte se ferme et ses parts se rangent.
    ///
    /// `garder` dit si cet intervalle a un sens — c'est la même question que le rythme pose
    /// sous le nom `attendue`. Une application qui dormait parce que personne ne demandait
    /// rien n'a pas gelé, et ranger son sommeil ferait du repos le pire moment de chaque
    /// session. La leçon est de la fiche 19 § 5, et elle a coûté une lecture entière.
    pub fn fermer(&mut self, maintenant: Instant, garder: bool) {
        // **Un entracte qui n'a jamais ete ouvert n'a rien a ranger.** Deux tests l'ont
        // exige : une image qui suit un dialogue natif, et la toute premiere de la session.
        // Sans cette garde, chacune rangeait cinq parts nulles et comptait un entracte qui
        // n'a pas eu lieu -- des zeros qui tirent la mediane vers le bas et font paraitre
        // saine une distribution qu'on n'a pas mesuree.
        let ouvert = self.marque.is_some();
        self.imputer(maintenant, Poste::Systeme);
        self.marque = None;
        if !ouvert || !garder {
            return;
        }
        let total: Duration = self.en_cours.iter().sum();
        for (poste, part) in self.par_poste.iter_mut().zip(self.en_cours) {
            poste.ajouter(micros(part));
        }
        self.totaux.ajouter(micros(total));
        if total > self.pire_total {
            self.pire_total = total;
            self.pire = self.en_cours;
            self.pire_a_ms = self
                .origine
                .map(|o| maintenant.saturating_duration_since(o).as_millis())
                .unwrap_or(0)
                .min(u128::from(u32::MAX)) as u32;
        }
    }

    /// **La boucle a été tenue hors de tout rendu** — un dialogue natif, le plus souvent.
    ///
    /// Le rythme oublie son intervalle pour la même raison, et au même instant : sans cela une
    /// ouverture de fichier se lit « le pire gel : 19 836 ms », vrai et sans aucun intérêt.
    pub fn oublier(&mut self) {
        self.marque = None;
    }

    /// La distribution d'un poste : `(médian, p99, pire)` en microsecondes.
    pub fn poste(&self, poste: Poste) -> (u32, u32, u32) {
        let h = &self.par_poste[poste.indice()];
        (h.centile(0.50), h.centile(0.99), h.pire())
    }

    /// La distribution de l'entracte entier : `(médian, p99, pire)` en microsecondes.
    pub fn total(&self) -> (u32, u32, u32) {
        (
            self.totaux.centile(0.50),
            self.totaux.centile(0.99),
            self.totaux.pire(),
        )
    }

    /// Combien d'entractes ont été rangés.
    pub fn comptes(&self) -> u64 {
        self.totaux.compte()
    }

    /// **Le pire entracte, décomposé** : quand, combien, et ce que chaque poste y a pris.
    ///
    /// C'est la ligne qui nomme un gel. Une distribution dit ce qui arrive d'ordinaire ; elle
    /// ne dit pas ce qu'une attente de sept dixièmes de seconde contenait, et c'est pourtant
    /// la seule question que trois sessions ont posée.
    pub fn pire(&self) -> (Duration, Duration, Vec<(Poste, Duration)>) {
        let parts = Poste::TOUS
            .iter()
            .map(|p| (*p, self.pire[p.indice()]))
            .collect();
        (
            Duration::from_millis(u64::from(self.pire_a_ms)),
            self.pire_total,
            parts,
        )
    }
}

/// Une durée en microsecondes, bornée au type qui la porte.
fn micros(d: Duration) -> u32 {
    u32::try_from(d.as_micros()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
