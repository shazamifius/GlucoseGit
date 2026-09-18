//! L'élan de la caméra : ce que la main conduit, et ce qui continue quand elle lâche.
//!
//! # Deux régimes, et la frontière entre eux est explicite
//!
//! À tout instant la caméra est dans l'un de deux états, et c'est la **seule** structure de ce
//! module :
//!
//! * **conduite** — la main pousse. Le contenu suit *exactement* ce qu'elle demande, sans
//!   retard ni lissage : c'est la seule chose qu'un pavé tactile doive garantir. On mesure la
//!   vitesse au passage ;
//! * **libre** — la main a lâché. La vitesse mesurée subsiste et décroît.
//!
//! **Reprendre la main tue la glissade en cours, immédiatement.** C'est le « frein » qui
//! manquait : une version précédente gardait une vitesse lissée qui survivait au changement
//! d'intention, si bien qu'en repartant à gauche pendant une glissade vers la droite, la
//! première image sans événement relançait la vue vers la droite. Le geste ne pouvait pas
//! interrompre ce qu'il contredisait.
//!
//! # Mesurer un geste et l'amortir sont deux métiers
//!
//! Les confondre était l'erreur de fond. La version précédente estimait la vitesse par un
//! lissage exponentiel de la **même** constante de temps que l'amortissement : réagir vite
//! demandait une constante courte, glisser longtemps une constante longue, et une seule valeur
//! ne pouvait pas faire les deux.
//!
//! La vitesse se lit donc sur une **fenêtre glissante** : la somme des déplacements récents
//! divisée par le temps qu'ils ont pris. C'est la définition d'une vitesse moyenne, rien de
//! plus, et elle a la propriété qui manquait — deux déplacements opposés s'y **annulent**,
//! donc un changement de sens est vu à l'instant où il se produit.
//!
//! # L'amortissement, et pourquoi l'exponentielle
//!
//! `v(t) = v₀·e^(−t/τ)` — le frottement visqueux. Choisi pour une raison qui n'a rien d'un
//! goût : l'exponentielle est la **seule** décroissance qui se compose exactement. Deux pas
//! d'une demi-image donnent le même résultat qu'un pas d'une image entière, donc la glissade
//! est identique à 30 images par seconde et à 240. Une décroissance par image, elle, aurait
//! empiré le mouvement précisément quand la machine peine.
//!
//! Le pas se calcule par l'intégrale exacte, `v₀·τ·(1 − e^(−dt/τ))`, jamais par `v·dt` : une
//! image longue ne dépasse donc pas.
//!
//! Et `τ` n'est pas un coefficient sans visage : la distance totale d'une glissade vaut
//! exactement **`v₀·τ`**. Lâcher à mille pixels par seconde emporte `1000·τ` pixels.
//!
//! # Le seuil d'arrêt n'est pas une constante
//!
//! On s'arrête quand ce qui **reste** à parcourir tient sous le demi-pixel — la limite de ce
//! qu'un écran peut montrer. Ce n'est pas un epsilon choisi : c'est la définition de
//! « invisible », et elle vaut pour le déplacement comme pour le zoom, ramené aux pixels que
//! le bord de l'écran parcourrait.

use std::time::{Duration, Instant};

/// Le temps caractéristique de la glissade du **zoom**.
const TAU_ZOOM: f64 = 0.28;

/// Le temps caractéristique de la glissade du **déplacement**.
///
/// Plus long que celui du zoom, et la raison est mesurable : Windows amortit **déjà** le
/// glissement à deux doigts — le pilote continue d'envoyer des défilements décroissants
/// pendant environ un tiers de seconde après le lever — alors que le pincement s'arrête net
/// avec les doigts. Notre élan se compose donc avec celui du pilote d'un côté et avec rien de
/// l'autre, et deux amortissements en série décroissent plus vite que chacun.
const TAU_PAN: f64 = 0.45;

/// Sur quelle durée se lit la vitesse d'un geste.
///
/// Un dixième de seconde : assez long pour que le bruit d'un pavé tactile s'annule, assez
/// court pour qu'un changement de direction soit vu tout de suite. C'est une propriété du
/// geste humain, pas de la machine — elle ne dépend donc ni de l'écran ni de la cadence.
const FENETRE: Duration = Duration::from_millis(100);

/// Combien d'images la fenêtre peut couvrir.
///
/// À 240 Hz, un dixième de seconde en compte vingt-quatre ; trente-deux laisse de la marge
/// sans qu'aucune allocation n'ait lieu. Au-delà, les plus anciennes sortent — ce qui est
/// exactement leur destin, puisqu'elles seraient hors fenêtre de toute façon.
const ECHANTILLONS: usize = 32;

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
    /// Ce mouvement change-t-il quelque chose ?
    fn existe(&self) -> bool {
        self.pan != (0.0, 0.0) || self.octaves != 0.0
    }
}

/// Une vitesse de caméra : pixels écran par seconde, et octaves par seconde.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Vitesse {
    pan: (f64, f64),
    octaves: f64,
}

impl Vitesse {
    fn nulle(&self) -> bool {
        self.pan == (0.0, 0.0) && self.octaves == 0.0
    }

    /// Ce qui reste à parcourir avant l'arrêt, en pixels écran.
    ///
    /// La distance totale d'une glissade vaut `v·τ`, et un reste d'octaves se ramène aux
    /// pixels que le bord de l'écran parcourrait : les deux mouvements s'arrêtent donc sur le
    /// même critère, l'invisible.
    fn reste_en_pixels(&self, diagonale: f64) -> f64 {
        let deplacement = (self.pan.0 * TAU_PAN).hypot(self.pan.1 * TAU_PAN);
        let echelle = diagonale / 2.0 * ((self.octaves * TAU_ZOOM).exp2() - 1.0).abs();
        deplacement.max(echelle)
    }
}

/// Dans quel régime se trouve la caméra.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Etat {
    /// La main pousse : le contenu suit exactement, et rien ne survit de l'élan précédent.
    Conduite,
    /// La main a lâché : cette vitesse décroît jusqu'à l'invisible.
    Libre(Vitesse),
}

/// La demande en cours et l'élan qui lui survit.
#[derive(Debug)]
pub struct Elan {
    /// Ce que la main a demandé et que l'image n'a pas encore montré.
    demande: Mouvement,
    /// Le point d'écran autour duquel l'échelle tourne.
    ///
    /// **Séparé de la demande**, parce qu'une ancre est un lieu et non une quantité : elle ne
    /// se consomme pas. Quand elle vivait dans la demande, la vider la remettait à `(0, 0)` et
    /// toute la glissade de zoom tournait autour du coin de la fenêtre.
    ancre: (f64, f64),
    etat: Etat,
    /// Les déplacements récents, d'où se lit la vitesse du geste.
    recents: Fenetre,
    /// Quand la dernière image a pris sa part.
    dernier: Option<Instant>,
}

impl Default for Elan {
    fn default() -> Self {
        Self {
            demande: Mouvement::default(),
            ancre: (0.0, 0.0),
            // Au repos : aucune vitesse, donc rien à montrer et rien à éteindre.
            etat: Etat::Libre(Vitesse::default()),
            recents: Fenetre::default(),
            dernier: None,
        }
    }
}

impl Elan {
    /// La main demande un déplacement de tant de pixels écran.
    pub fn pousser_pan(&mut self, dx: f64, dy: f64) {
        self.demande.pan.0 += dx;
        self.demande.pan.1 += dy;
    }

    /// La main demande un changement d'échelle de tant d'octaves, autour de ce point.
    pub fn pousser_zoom(&mut self, octaves: f64, ancre: (f64, f64)) {
        self.demande.octaves += octaves;
        self.ancre = ancre;
    }

    /// Y a-t-il encore quelque chose à montrer ? C'est ce qui décide de redemander une image.
    pub fn en_cours(&self) -> bool {
        match self.etat {
            Etat::Conduite => true,
            Etat::Libre(v) => self.demande.existe() || !v.nulle(),
        }
    }

    /// Ce que cette image doit appliquer, et rien de plus.
    ///
    /// `diagonale` est la diagonale de la fenêtre en pixels : elle sert à ramener un reste de
    /// zoom à ce qu'il déplacerait à l'écran.
    pub fn avancer(&mut self, maintenant: Instant, diagonale: f64) -> Option<Mouvement> {
        let precedent = self.dernier.replace(maintenant);
        let dt = precedent
            .map(|t| maintenant.saturating_duration_since(t).min(PAS_MAX))
            .unwrap_or(PAS_MAX)
            .as_secs_f64();

        if self.demande.existe() {
            return Some(self.conduire(dt));
        }
        self.laisser_filer(dt, diagonale)
    }

    /// La main pousse : on applique exactement sa demande, et on la mesure.
    ///
    /// **C'est ici que la glissade meurt.** Reprendre la main annule l'élan précédent sans
    /// délai : c'est ce qui permet de repartir dans l'autre sens sans que la vue continue un
    /// instant dans l'ancien. La fenêtre de mesure se vide du même geste — les déplacements
    /// d'avant appartenaient à un mouvement que l'utilisateur vient de contredire.
    fn conduire(&mut self, dt: f64) -> Mouvement {
        if self.etat != Etat::Conduite {
            self.etat = Etat::Conduite;
            self.recents.vider();
        }
        let demande = std::mem::take(&mut self.demande);
        self.recents.noter(dt, &demande);
        Mouvement {
            ancre: self.ancre,
            ..demande
        }
    }

    /// La main a lâché : ce qui reste s'écoule et s'éteint.
    fn laisser_filer(&mut self, dt: f64, diagonale: f64) -> Option<Mouvement> {
        if self.etat == Etat::Conduite {
            // Le passage de la conduite au libre : la vitesse du geste devient celle de la
            // glissade, une fois pour toutes. Elle ne sera plus jamais modifiée sans qu'un
            // nouveau geste ne l'efface.
            self.etat = Etat::Libre(self.recents.vitesse());
        }
        let Etat::Libre(vitesse) = &mut self.etat else {
            return None;
        };
        if vitesse.reste_en_pixels(diagonale) < 0.5 {
            *vitesse = Vitesse::default();
            return None;
        }
        // L'intégrale exacte de la décroissance sur ce pas : jamais plus que ce qui reste.
        let pas_pan = TAU_PAN * (1.0 - (-dt / TAU_PAN).exp());
        let pas_zoom = TAU_ZOOM * (1.0 - (-dt / TAU_ZOOM).exp());
        let mouvement = Mouvement {
            pan: (vitesse.pan.0 * pas_pan, vitesse.pan.1 * pas_pan),
            octaves: vitesse.octaves * pas_zoom,
            ancre: self.ancre,
        };
        vitesse.pan.0 *= (-dt / TAU_PAN).exp();
        vitesse.pan.1 *= (-dt / TAU_PAN).exp();
        vitesse.octaves *= (-dt / TAU_ZOOM).exp();
        mouvement.existe().then_some(mouvement)
    }
}

/// Les déplacements des dernières images, pour lire la vitesse du geste.
///
/// Un anneau de taille fixe : aucune allocation, aucun parcours qui grandisse, et la plus
/// ancienne entrée disparaît d'elle-même — ce qui est exactement son destin, puisqu'elle
/// serait hors fenêtre de toute façon.
#[derive(Debug, Default)]
struct Fenetre {
    entrees: [(f64, Mouvement); ECHANTILLONS],
    /// Où écrire la prochaine.
    curseur: usize,
    /// Combien d'entrées sont valides, au plus [`ECHANTILLONS`].
    remplies: usize,
}

impl Fenetre {
    fn vider(&mut self) {
        self.curseur = 0;
        self.remplies = 0;
    }

    fn noter(&mut self, dt: f64, mouvement: &Mouvement) {
        self.entrees[self.curseur] = (dt, *mouvement);
        self.curseur = (self.curseur + 1) % ECHANTILLONS;
        self.remplies = (self.remplies + 1).min(ECHANTILLONS);
    }

    /// La vitesse moyenne sur la fenêtre : la somme des déplacements sur le temps qu'ils ont pris.
    ///
    /// # Pourquoi une moyenne, et pas la dernière valeur
    ///
    /// Deux déplacements opposés s'y **annulent**. C'est ce qui rend un changement de direction
    /// visible à l'instant où il se produit, là où un lissage exponentiel le ferait traîner sur
    /// toute sa constante de temps.
    ///
    /// Et c'est robuste au découpage : les événements d'un pavé tactile arrivent par paquets
    /// irréguliers, mais leur somme sur un dixième de seconde ne dépend pas de la façon dont
    /// les images les ont récoltés.
    fn vitesse(&self) -> Vitesse {
        let mut duree = 0.0;
        let mut pan = (0.0, 0.0);
        let mut octaves = 0.0;
        for (dt, m) in self.recentes() {
            duree += dt;
            pan.0 += m.pan.0;
            pan.1 += m.pan.1;
            octaves += m.octaves;
        }
        if duree <= 0.0 {
            return Vitesse::default();
        }
        Vitesse {
            pan: (pan.0 / duree, pan.1 / duree),
            octaves: octaves / duree,
        }
    }

    /// Les entrées de la fenêtre, de la plus récente vers la plus ancienne, et pas au-delà.
    ///
    /// # Aucune entrée ne dépasse la fenêtre, sauf la première
    ///
    /// La première est toujours retenue : sans elle, un geste plus court qu'un dixième de
    /// seconde n'aurait aucune vitesse du tout, et une chiquenaude ne lancerait rien.
    ///
    /// Les suivantes s'arrêtent **avant** de déborder, et c'est important : une version qui
    /// gardait l'entrée débordante laissait un seul vieil échantillon rapide dans une fenêtre
    /// d'échantillons lents — et comme il pesait cent fois les autres, il multipliait la
    /// vitesse mesurée par dix. Un geste qui ralentit avant de lâcher partait alors en
    /// glissade comme s'il n'avait pas ralenti.
    fn recentes(&self) -> impl Iterator<Item = &(f64, Mouvement)> {
        let fenetre = FENETRE.as_secs_f64();
        let mut cumul = 0.0;
        (0..self.remplies)
            .map(move |n| {
                let i = (self.curseur + ECHANTILLONS - 1 - n) % ECHANTILLONS;
                &self.entrees[i]
            })
            .take_while(move |(dt, _)| {
                let premiere = cumul == 0.0;
                cumul += dt;
                premiere || cumul <= fenetre
            })
    }
}

#[cfg(test)]
mod tests;
