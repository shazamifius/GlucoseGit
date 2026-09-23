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
//! # Les quatre postes, et pourquoi ils se somment exactement
//!
//! Entre deux images, le contrôle est à tout instant à **un seul** endroit :
//!
//! | poste | qui tient le contrôle |
//! |---|---|
//! | `systeme` | winit et Windows : le sommeil demandé, et la file de messages |
//! | `main` | nos gestionnaires d'événements — souris, clavier, molette |
//! | `depot` | ce qu'un glisser-déposer a apporté, et qu'on pose dans le document |
//! | `entretien` | le reste de `about_to_wait` : l'empreinte, la chronique, les réveils |
//!
//! **Leur somme vaut l'entracte entier, au bit près**, et un test l'exige. C'est la garantie
//! qui manquait aux quatre marques mal posées de ce dépôt — `occlusion`, `recolte`, `blit`,
//! `minimap` —, dont chacune absorbait ce qui la précédait et a désigné le mauvais coupable
//! pendant plusieurs sessions. Ici, ce qu'un poste ne prend pas, un autre le porte : rien ne
//! peut disparaître, et rien ne peut compter deux fois.
//!
//! # Chaque gel est gardé décomposé, et pas seulement le pire (ENTRACTE-2)
//!
//! Une distribution dit ce qui arrive d'ordinaire ; elle ne dit pas ce qu'**un** gel de sept
//! dixièmes de seconde contenait. La première version gardait donc le pire entracte avec ses
//! cinq parts — et la première session réelle a montré pourquoi ce n'était pas assez : le pire
//! était le gel du **démarrage**, 607 ms à la 0,6ᵉ seconde, connu depuis la fiche 19, et il
//! cachait tous les autres. Le verdict comptait 2 314 ms perdues ; la section n'en expliquait
//! qu'un quart.
//!
//! Sont gardées désormais **toutes les attentes qui ont coûté au moins une image entière** —
//! exactement celles que le constat du gel additionne, avec le même plancher. La liste et le
//! constat se recoupent donc par construction, et aucun seuil n'a été choisi.

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
    /// Le reste de `about_to_wait` : l'empreinte du processus, la chronique sauvée, le calcul
    /// du prochain réveil.
    Entretien,
}

impl Poste {
    /// Tous les postes, dans l'ordre de leur indice.
    pub const TOUS: [Poste; 4] = [Poste::Systeme, Poste::Main, Poste::Depot, Poste::Entretien];

    /// Combien il y en a.
    pub const COMBIEN: usize = Self::TOUS.len();

    /// Sa place dans les tableaux.
    fn indice(self) -> usize {
        match self {
            Self::Systeme => 0,
            Self::Main => 1,
            Self::Depot => 2,
            Self::Entretien => 3,
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
            Self::Entretien => "entretien",
        }
    }
}

/// Ce que chaque poste a pris pendant un entracte.
type Parts = [Duration; Poste::COMBIEN];

/// **Une attente qui a coûté au moins une image entière**, datée et décomposée.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gel {
    /// Depuis la première image de la session.
    pub a: Duration,
    /// Ce que l'attente a duré en tout.
    pub total: Duration,
    parts: Parts,
}

impl Gel {
    /// Ce que chaque poste y a pris, du plus gros au plus petit, sans les postes vides.
    pub fn parts(&self) -> Vec<(Poste, Duration)> {
        let mut parts: Vec<(Poste, Duration)> = Poste::TOUS
            .iter()
            .map(|p| (*p, self.parts[p.indice()]))
            .filter(|(_, d)| !d.is_zero())
            .collect();
        parts.sort_by_key(|(_, d)| std::cmp::Reverse(*d));
        parts
    }
}

/// Où va le temps entre deux images, sur toute une session.
#[derive(Debug)]
pub struct Entracte {
    /// L'instant où l'entracte en cours s'est ouvert, ou `None` hors d'un entracte.
    ouverture: Option<Instant>,
    /// Le début de la tranche en cours.
    marque: Option<Instant>,
    /// Le poste auquel la tranche en cours s'impute.
    poste: Poste,
    /// **Les tranches de l'entracte en cours, horodatées** (GEL-1).
    ///
    /// Horodatées, parce que ce qui compte n'est connu qu'à la fin : l'instant où l'image est
    /// devenue **due**. Ce qui précède est du repos — l'utilisateur regardait, ou était dans
    /// un autre logiciel — et ne se range nulle part. Le tampon se vide à chaque entracte et
    /// garde sa capacité : aucune allocation une fois la session lancée.
    tranches: Vec<(Poste, Instant, Instant)>,
    /// La distribution de chaque poste, en microsecondes.
    par_poste: [Histogramme; Poste::COMBIEN],
    /// La distribution de l'entracte entier, pour vérifier que les parts s'y retrouvent.
    totaux: Histogramme,
    /// **Les gels de la session**, du plus long au plus court — bornés comme les images
    /// lentes de la chronique, pour que la mémoire reste constante.
    gels: Vec<Gel>,
    /// Combien de gels la borne a laissés de côté : un rapport qui en tait se doit de le dire.
    gels_non_gardes: u64,
    /// L'instant de la première présentation, pour dater les gels.
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
            ouverture: None,
            marque: None,
            poste: Poste::Systeme,
            tranches: Vec::new(),
            par_poste: std::array::from_fn(|_| Histogramme::nouveau()),
            totaux: Histogramme::nouveau(),
            gels: Vec::with_capacity(super::PIRES + 1),
            gels_non_gardes: 0,
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
        self.tranches.clear();
        self.ouverture = Some(maintenant);
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
        if maintenant > depuis {
            self.tranches.push((self.poste, depuis, maintenant));
        }
        self.marque = Some(maintenant);
        self.poste = poste;
    }

    /// **Le rendu commence** : l'entracte se ferme, et seule la part qui suit l'échéance se
    /// range.
    ///
    /// `due` est l'instant où l'image est devenue nécessaire (GEL-1). Avant lui, l'application
    /// n'avait rien à montrer : c'était du repos, et le ranger ferait du repos le pire gel de
    /// la session — ce que la première version faisait, onze secondes passées dans un
    /// navigateur comptées comme un gel. Sans échéance, l'image n'était attendue par personne
    /// et rien ne se range. La leçon est de la fiche 19 § 5, et elle a coûté deux fois.
    pub fn fermer(&mut self, maintenant: Instant, due: Option<Instant>) {
        // **Un entracte qui n'a jamais ete ouvert n'a rien a ranger.** Deux tests l'ont
        // exige : une image qui suit un dialogue natif, et la toute premiere de la session.
        // Sans cette garde, chacune rangeait cinq parts nulles et comptait un entracte qui
        // n'a pas eu lieu -- des zeros qui tirent la mediane vers le bas et font paraitre
        // saine une distribution qu'on n'a pas mesuree.
        let ouverture = self.ouverture.take();
        self.imputer(maintenant, Poste::Systeme);
        self.marque = None;
        let (Some(ouverture), Some(due)) = (ouverture, due) else {
            self.tranches.clear();
            return;
        };
        let depart = due.max(ouverture);
        let mut parts: Parts = [Duration::ZERO; Poste::COMBIEN];
        for (poste, debut, fin) in self.tranches.drain(..) {
            parts[poste.indice()] += fin.saturating_duration_since(debut.max(depart));
        }
        self.ranger(maintenant, parts);
    }

    /// Range les parts d'un entracte fermé : les distributions, et le gel s'il en est un.
    fn ranger(&mut self, maintenant: Instant, parts: Parts) {
        let total: Duration = parts.iter().sum();
        for (poste, part) in self.par_poste.iter_mut().zip(parts) {
            poste.ajouter(micros(part));
        }
        self.totaux.ajouter(micros(total));
        // Le plancher de la charte : une attente plus longue a mange au moins une image
        // entiere, et c'est la meme borne que le constat du gel emploie pour les compter.
        if total > crate::cadence::BUDGET_TOTAL {
            let a = self
                .origine
                .map(|o| maintenant.saturating_duration_since(o))
                .unwrap_or_default();
            self.garder_le_gel(Gel { a, total, parts });
        }
    }

    /// Range un gel parmi les autres, du plus long au plus court, dans la borne.
    fn garder_le_gel(&mut self, gel: Gel) {
        let place = self.gels.partition_point(|g| g.total >= gel.total);
        self.gels.insert(place, gel);
        if self.gels.len() > super::PIRES {
            self.gels.truncate(super::PIRES);
            self.gels_non_gardes += 1;
        }
    }

    /// **La boucle a été tenue hors de tout rendu** — un dialogue natif, le plus souvent.
    ///
    /// Le rythme oublie son intervalle pour la même raison, et au même instant : sans cela une
    /// ouverture de fichier se lit « le pire gel : 19 836 ms », vrai et sans aucun intérêt.
    pub fn oublier(&mut self) {
        self.ouverture = None;
        self.marque = None;
        self.tranches.clear();
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

    /// **Les gels de la session**, du plus long au plus court, et combien la borne en a tus.
    ///
    /// Une distribution dit ce qui arrive d'ordinaire ; elle ne dit pas ce qu'une attente de
    /// sept dixièmes de seconde contenait, et c'est pourtant la seule question que trois
    /// sessions ont posée.
    pub fn gels(&self) -> (&[Gel], u64) {
        (&self.gels, self.gels_non_gardes)
    }
}

/// Une durée en microsecondes, bornée au type qui la porte.
fn micros(d: Duration) -> u32 {
    u32::try_from(d.as_micros()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
