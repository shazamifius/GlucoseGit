//! L'élan de la caméra : un seul mouvement par image, et ce qui continue quand la main lâche.
//!
//! # Les deux défauts que ce module supprime
//!
//! **Une diagonale était un escalier.** Windows livre le défilement vertical et le défilement
//! horizontal dans deux messages **séparés**. Chaque message était appliqué puis redessiné :
//! un pas à droite, une image, un pas en haut, une image. Le mouvement en biais n'existait
//! donc jamais — il était joué comme une marche d'escalier, ce qui ne se voit pas image par
//! image mais se ressent dans la main.
//!
//! **Et rien ne continuait après le geste.** Ni Glucose Rust ni Glucose Tauri n'ont jamais eu
//! d'élan : ce qu'on croyait en voir était l'inertie du pilote, qui dure un tiers de seconde
//! et s'arrête net.
//!
//! # Ce qui remplace : la caméra a une vitesse
//!
//! Les événements ne déplacent plus la caméra. Ils remplissent une **demande**, que l'image
//! vide en une fois. Deux conséquences immédiates, et aucune n'est un réglage :
//!
//! * la demande est un **vecteur**, donc une diagonale est une diagonale ;
//! * il y a exactement **un mouvement par image**, quel que soit le débit d'événements.
//!
//! Tant que la main pousse, la caméra suit **exactement** ce qu'elle demande — le contenu
//! colle au doigt, ce qui est la seule chose qu'un pavé tactile doit garantir. C'est en
//! passant qu'on mesure la vitesse. Quand la main lâche, cette vitesse reste, et décroît.
//!
//! # L'amortissement, et pourquoi celui-là
//!
//! `v(t) = v₀·e^(−t/τ)` — le frottement visqueux. Choisi pour une raison qui n'a rien d'un
//! goût : l'exponentielle est la **seule** décroissance qui se compose exactement. Deux pas
//! d'une demi-image donnent le même résultat qu'un pas d'une image entière, donc la glissade
//! est identique à 30 images par seconde et à 240. Une multiplication par image, elle, aurait
//! fait dépendre le ressenti de la charge — c'est-à-dire empirer le mouvement exactement quand
//! la machine peine.
//!
//! Le pas se calcule par l'intégrale exacte, `v₀·τ·(1 − e^(−dt/τ))`, et non par `v·dt` :
//! une image longue ne dépasse donc jamais.
//!
//! Et `τ` n'est pas un coefficient sans visage : la distance totale d'une glissade vaut
//! exactement **`v₀·τ`**. Lâcher à mille pixels par seconde emporte `1000·τ` pixels, et rien
//! d'autre à savoir.
//!
//! # Le seuil d'arrêt n'est pas une constante
//!
//! On s'arrête quand ce qui **reste** à parcourir tient sous le demi-pixel — la limite de ce
//! qu'un écran peut montrer. Ce n'est pas un epsilon choisi : c'est la définition de
//! « invisible », et elle vaut pour le déplacement comme pour le zoom, ramené aux pixels que
//! le bord de l'écran parcourrait.

use std::time::{Duration, Instant};

/// Le temps caractéristique de la glissade.
///
/// Sa lecture directe : une glissade lâchée à `v` pixels par seconde parcourt encore `v × τ`
/// pixels. C'est la seule grandeur de ressenti de ce module, et elle vaut pour le déplacement
/// comme pour le zoom — une seule main, un seul élan.
const TAU: f64 = 0.28;

/// Au-delà, on ne glisse pas : on a été suspendu.
///
/// Une image qui a mis une demi-seconde ne doit pas faire bondir la caméra de la distance
/// correspondante. Borner le pas de temps est la façon honnête de dire que la mesure ne
/// décrit plus un mouvement continu.
const PAS_MAX: Duration = Duration::from_millis(100);

/// Ce qu'une image doit appliquer à la caméra.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Mouvement {
    /// Déplacement en pixels écran.
    pub pan: (f64, f64),
    /// Changement d'échelle, en **octaves** : `+1` double la taille apparente.
    pub octaves: f64,
    /// Le point d'écran autour duquel l'échelle tourne.
    pub ancre: (f64, f64),
}

impl Mouvement {
    /// Ce mouvement change-t-il quelque chose de visible ?
    fn existe(&self) -> bool {
        self.pan.0 != 0.0 || self.pan.1 != 0.0 || self.octaves != 0.0
    }
}

/// La demande en cours et l'élan qui lui survit.
#[derive(Debug, Default)]
pub struct Elan {
    /// Ce que la main a demandé et que l'image n'a pas encore montré.
    demande: Mouvement,
    /// La vitesse du geste, en pixels écran par seconde.
    vitesse: (f64, f64),
    /// La vitesse de zoom, en octaves par seconde.
    vitesse_octaves: f64,
    /// Quand la dernière image a pris sa part.
    dernier: Option<Instant>,
}

impl Elan {
    /// La main demande un déplacement de tant de pixels écran.
    pub fn pousser_pan(&mut self, dx: f64, dy: f64) {
        self.demande.pan.0 += dx;
        self.demande.pan.1 += dy;
    }

    /// La main demande un changement d'échelle de tant d'octaves, autour de ce point.
    ///
    /// L'ancre est celle du **dernier** geste : pendant un pincement le doigt ne bouge
    /// pratiquement pas, et entre deux gestes distincts c'est bien le point courant qui compte.
    pub fn pousser_zoom(&mut self, octaves: f64, ancre: (f64, f64)) {
        self.demande.octaves += octaves;
        self.demande.ancre = ancre;
    }

    /// Y a-t-il encore quelque chose à montrer ? C'est ce qui décide de redemander une image.
    pub fn en_cours(&self) -> bool {
        self.demande.existe() || self.vitesse != (0.0, 0.0) || self.vitesse_octaves != 0.0
    }

    /// Ce que cette image doit appliquer, et rien de plus.
    ///
    /// `diagonale` est la diagonale de la fenêtre en pixels : elle sert à ramener un reste de
    /// zoom à ce qu'il déplacerait à l'écran, pour que les deux mouvements s'arrêtent sur le
    /// même critère — l'invisible.
    pub fn avancer(&mut self, maintenant: Instant, diagonale: f64) -> Option<Mouvement> {
        let precedent = self.dernier.replace(maintenant);
        let dt = precedent
            .map(|t| maintenant.saturating_duration_since(t).min(PAS_MAX))
            .unwrap_or(PAS_MAX)
            .as_secs_f64();

        if self.demande.existe() {
            return Some(self.prendre_la_demande(dt));
        }
        self.glisser(dt, diagonale)
    }

    /// La main pousse : on applique exactement sa demande, et on en retient la vitesse.
    ///
    /// # Pourquoi la vitesse se lisse, et avec le même `τ`
    ///
    /// Prendre `demande / dt` tel quel la ferait exploser sur une image courte : les
    /// événements arrivent par paquets irréguliers, et un seul paquet tombé dans une image
    /// d'une milliseconde vaudrait mille fois sa valeur.
    ///
    /// Le lissage exponentiel de constante `τ` corrige cela **exactement** : la part prise
    /// vaut `1 − e^(−dt/τ) ≈ dt/τ` pour un pas court, et elle multiplie `demande/dt` — les
    /// `dt` se simplifient. Ce qui s'accumule est donc la somme des demandes divisée par `τ`,
    /// quelle que soit la façon dont elles se répartissent entre les images.
    ///
    /// Et cela règle un second problème qui n'en avait pas l'air : le pilote Windows amortit
    /// déjà, sur une durée voisine de `τ`. Suivre sa décroissance instantanée aurait fait
    /// démarrer notre glissade d'une vitesse presque nulle — c'est-à-dire n'aurait rien
    /// changé. Lissée, la vitesse retenue reste celle du geste, pas celle de sa fin.
    fn prendre_la_demande(&mut self, dt: f64) -> Mouvement {
        let demande = std::mem::take(&mut self.demande);
        if dt > 0.0 {
            let part = 1.0 - (-dt / TAU).exp();
            self.vitesse.0 += (demande.pan.0 / dt - self.vitesse.0) * part;
            self.vitesse.1 += (demande.pan.1 / dt - self.vitesse.1) * part;
            self.vitesse_octaves += (demande.octaves / dt - self.vitesse_octaves) * part;
        }
        demande
    }

    /// La main a lâché : ce qui reste s'écoule et s'éteint.
    fn glisser(&mut self, dt: f64, diagonale: f64) -> Option<Mouvement> {
        if self.eteint(diagonale) {
            self.vitesse = (0.0, 0.0);
            self.vitesse_octaves = 0.0;
            return None;
        }
        // L'intégrale exacte de la décroissance sur ce pas : jamais plus que ce qui reste.
        let parcouru = TAU * (1.0 - (-dt / TAU).exp());
        let mouvement = Mouvement {
            pan: (self.vitesse.0 * parcouru, self.vitesse.1 * parcouru),
            octaves: self.vitesse_octaves * parcouru,
            ancre: self.demande.ancre,
        };
        let reste = (-dt / TAU).exp();
        self.vitesse = (self.vitesse.0 * reste, self.vitesse.1 * reste);
        self.vitesse_octaves *= reste;
        mouvement.existe().then_some(mouvement)
    }

    /// Ce qui reste à parcourir tient-il sous le demi-pixel ?
    ///
    /// La distance totale d'une glissade vaut `v·τ` : le test porte donc sur elle, et non sur
    /// le pas d'une image — sinon une cadence élevée arrêterait le mouvement plus tôt qu'une
    /// cadence basse, ce qui serait exactement le contraire du but.
    fn eteint(&self, diagonale: f64) -> bool {
        let reste = (self.vitesse.0 * TAU).hypot(self.vitesse.1 * TAU);
        // Ce qu'un reste d'octaves déplacerait au bord de l'écran, en pixels.
        let reste_zoom = diagonale / 2.0 * ((self.vitesse_octaves * TAU).exp2() - 1.0).abs();
        reste < 0.5 && reste_zoom < 0.5
    }
}

#[cfg(test)]
mod tests;
