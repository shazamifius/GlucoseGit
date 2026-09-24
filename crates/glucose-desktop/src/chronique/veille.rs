//! Ce que Glucose coûte à la machine, et surtout ce qu'il coûte **quand on ne le touche pas**
//! (EMPREINTE-1).
//!
//! # La question que cinq sessions de mesure n'ont jamais posée
//!
//! Tout le reste de la chronique mesure ce qu'une **image** coûte. Rien n'y disait ce que le
//! **processus** coûte : la mémoire qu'il occupe, et le processeur qu'il consomme pendant que
//! personne ne s'en sert. *« L'idée c'est que ce soit un logiciel ultra rapide et économe »* —
//! et de cette seconde moitié, rien n'était mesuré.
//!
//! # Deux régimes, et c'est leur séparation qui répond
//!
//! Une part de processeur sur toute une session ne dit rien : elle mélange le temps où l'on
//! zoomait et le temps où la fenêtre était derrière une autre. Les deux sont donc comptés à
//! part, et le critère est **factuel, pas déclaré** : un intervalle pendant lequel **la main
//! n'a rien demandé** est un intervalle sans utilisateur. Personne n'a à s'annoncer, et c'est
//! ce qui rend la mesure impossible à fausser par oubli — la faute que la fiche 17 § 3.1
//! raconte.
//!
//! # Le critère que la première mesure a démenti en vingt secondes
//!
//! J'avais écrit « un intervalle pendant lequel aucune **image** ne s'est rendue ». Lancée sans
//! qu'on y touche, l'application a rendu vingt-six images en vingt secondes et la section a
//! annoncé *« 1,4 % d'un cœur pendant qu'on s'en sert »* — alors que personne ne s'en servait.
//! Une application qui se réveille toute seule rend des images, et compter des images revenait
//! donc à appeler « usage » ce qu'on cherchait justement à mesurer.
//!
//! Le critère est donc **ce que la main a demandé**, et ce que l'application rend pendant ce
//! temps devient un résultat au lieu d'être la question : *combien d'images par seconde
//! Glucose dessine-t-il quand personne ne le regarde ?* Zéro est la seule bonne réponse, et
//! c'est ce nombre-là qui dit si un portable peut se rendormir.
//!
//! # Pourquoi on ne relève pas à chaque image
//!
//! Le système compte le temps processeur par **quantum d'ordonnancement**, soit une quinzaine
//! de millisecondes sur Windows. Une différence prise sur un intervalle plus court ne mesure
//! que l'arrondi de ce compteur : elle vaut zéro ou quinze millisecondes, jamais ce qui s'est
//! passé. Une seconde donne une résolution d'un pour cent et demi, ce qui est la finesse utile
//! pour une grandeur dont on veut savoir si elle est proche de zéro.
//!
//! Et cette cadence n'a rien à voir avec la boucle d'images : au repos il ne s'en rend aucune,
//! donc un relevé accroché aux images ne mesurerait jamais le repos — précisément le cas qui
//! intéresse.

use crate::plateforme::empreinte::{relever, Empreinte};
use std::time::{Duration, Instant};

/// L'intervalle minimal entre deux relevés.
///
/// Il vient de la **granularité du compteur du système**, pas d'une préférence : en dessous,
/// la différence de temps processeur ne mesure que l'arrondi du quantum d'ordonnancement.
const INTERVALLE: Duration = Duration::from_secs(1);

/// Ce que la session a observé de l'empreinte du processus.
#[derive(Debug, Default)]
pub struct Veille {
    /// Le dernier relevé, et l'instant où il a été pris.
    dernier: Option<(Instant, Empreinte)>,
    /// Ce que la main avait demandé au dernier relevé — c'est lui qui dit si on dormait.
    sous_la_main: u64,
    /// Le nombre d'images rendues au dernier relevé, et celles qui l'ont été sans la main.
    rendues_au_dernier: u64,
    rendues_sans_la_main: u64,
    /// La mémoire du dernier relevé, et la plus grande vue de la session.
    memoire_octets: u64,
    memoire_pire: u64,
    /// Le temps mural et le temps processeur cumulés, dans chacun des deux régimes.
    endormi: (u64, u64),
    eveille: (u64, u64),
    /// Ce que la carte graphique porte pour Glucose, si la plateforme sait le dire (VRAM-1).
    carte: Option<Carte>,
    /// La mémoire vive, étage par étage (ETAGES-1).
    pub etages: super::etages::Etages,
}

/// **Ce que la carte graphique a porté pour Glucose pendant la session**, en octets, et ce que
/// le système lui accordait (VRAM-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Carte {
    pub utilisee: u64,
    pub utilisee_pire: u64,
    pub budget: u64,
    pub budget_plus_bas: u64,
    /// Ce que le cache des textures hors de l'écran a gardé, au plus.
    pub cache_pire: u64,
}

/// Où en est la session quand on la relève.
///
/// Deux nombres et non un, parce que la question et sa réponse ne se lisent pas au même
/// endroit : ce sont **les événements de la main** qui disent s'il y avait un utilisateur, et
/// **les images** qui disent ce que l'application faisait pendant ce temps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Compte {
    /// Combien d'**événements** la main a envoyés depuis le début de la session : souris,
    /// molette, clavier, focus. Des événements et non des images classées sous un geste — un
    /// survol n'est classé sous aucun, et c'est pourtant la main (fiche 29 § 4.4).
    pub sous_la_main: u64,
    /// Combien d'images ont été rendues, toutes causes confondues.
    pub rendues: u64,
}

/// Une part de cœur observée sur une durée.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part {
    /// La part d'un cœur : 1,00 est un cœur entier, 0,00 est le sommeil.
    pub coeurs: f64,
    /// Sur combien de temps mural elle a été observée.
    pub sur: Duration,
}

impl Veille {
    /// **Relève l'empreinte si l'intervalle est écoulé**, range ce qu'elle dit, et rend vrai
    /// quand elle l'a fait — l'appelant relève alors les étages au même rythme.
    ///
    /// `compte` dit où en est la session : ce que la main a demandé — s'il n'a pas bougé,
    /// l'intervalle qui vient de s'écouler s'est passé sans utilisateur — et ce qui a été
    /// rendu, qui dira ce que l'application dessine alors que personne ne la regarde.
    pub fn observer(&mut self, maintenant: Instant, compte: Compte) -> bool {
        if let Some((quand, _)) = self.dernier {
            if maintenant.duration_since(quand) < INTERVALLE {
                return false;
            }
        }
        let Some(vu) = relever() else {
            return false;
        };
        self.noter(maintenant, compte, vu);
        true
    }

    /// **Range un relevé**, sans l'avoir pris.
    ///
    /// Séparée d'[`Self::observer`] pour une seule raison, et c'est la bonne : ce qui décide —
    /// quel régime reçoit cet intervalle, ce qu'on garde de la mémoire — se teste alors sans
    /// système d'exploitation, sur n'importe quelle machine, avec des relevés choisis. Une
    /// fonction qui interroge le système et qui conclut dans le même souffle ne se teste que
    /// là où elle tourne.
    pub(crate) fn noter(&mut self, maintenant: Instant, compte: Compte, vu: Empreinte) {
        self.memoire_octets = vu.memoire_octets;
        self.memoire_pire = self.memoire_pire.max(vu.memoire_octets);
        if let Some((quand, avant)) = self.dernier {
            let mural = maintenant.duration_since(quand).as_micros() as u64;
            let cpu = vu.processeur_us.saturating_sub(avant.processeur_us);
            let sans_la_main = compte.sous_la_main == self.sous_la_main;
            let regime = if sans_la_main {
                self.rendues_sans_la_main += compte.rendues - self.rendues_au_dernier;
                &mut self.endormi
            } else {
                &mut self.eveille
            };
            regime.0 += mural;
            regime.1 += cpu;
        }
        self.dernier = Some((maintenant, vu));
        self.sous_la_main = compte.sous_la_main;
        self.rendues_au_dernier = compte.rendues;
    }

    /// **Range un relevé de la carte graphique** : ce qu'elle porte, ce que le système accorde,
    /// et ce que le cache de textures en garde.
    pub fn noter_la_carte(&mut self, vu: crate::memoire::MemoireGraphique, en_cache: u64) {
        let avant = self.carte.unwrap_or(Carte {
            utilisee: 0,
            utilisee_pire: 0,
            budget: vu.budget,
            budget_plus_bas: vu.budget,
            cache_pire: 0,
        });
        self.carte = Some(Carte {
            utilisee: vu.utilisee,
            utilisee_pire: avant.utilisee_pire.max(vu.utilisee),
            budget: vu.budget,
            budget_plus_bas: avant.budget_plus_bas.min(vu.budget),
            cache_pire: avant.cache_pire.max(en_cache),
        });
    }

    /// Ce que la carte a porté, ou rien si la plateforme ne l'a jamais dit.
    pub fn carte(&self) -> Option<Carte> {
        self.carte
    }

    /// Y a-t-il quelque chose à dire ? Sur une plateforme sans relevé, non — et le rapport se
    /// tait plutôt que d'écrire des zéros.
    pub fn a_mesure(&self) -> bool {
        self.dernier.is_some()
    }

    /// La mémoire du dernier relevé, et la plus grande de la session, en octets.
    pub fn memoire(&self) -> (u64, u64) {
        (self.memoire_octets, self.memoire_pire)
    }

    /// Ce que le processeur a donné pendant que personne ne touchait à rien.
    pub fn au_repos(&self) -> Option<Part> {
        Self::part(self.endormi)
    }

    /// **Combien d'images par seconde Glucose dessine pendant que personne ne le regarde.**
    ///
    /// Zéro est la seule bonne réponse : chaque image rendue sans raison empêche le processeur
    /// de descendre dans ses états de sommeil profond, et c'est de l'autonomie en moins sur un
    /// portable. Rien quand aucun intervalle sans main n'a été observé.
    pub fn images_sans_la_main(&self) -> Option<f64> {
        let secondes = Duration::from_micros(self.endormi.0).as_secs_f64();
        (secondes > 0.0).then(|| self.rendues_sans_la_main as f64 / secondes)
    }

    /// Ce qu'il a donné pendant qu'on s'en servait.
    pub fn a_l_usage(&self) -> Option<Part> {
        Self::part(self.eveille)
    }

    /// La part d'un cœur, ou rien si ce régime n'a jamais été observé.
    fn part((mural_us, cpu_us): (u64, u64)) -> Option<Part> {
        (mural_us > 0).then(|| Part {
            coeurs: cpu_us as f64 / mural_us as f64,
            sur: Duration::from_micros(mural_us),
        })
    }
}

#[cfg(test)]
mod tests;
