//! La chronique : ce que Glucose fait, ce que ça coûte, et ce qui a gelé (CHRONIQUE-1).
//!
//! # Pourquoi ce module existe
//!
//! Toute cette session l'a montré : mes bancs mesurent ce que je leur demande, et l'utilisateur
//! vit autre chose. Une photo importée coûtait 351 ms dans la boucle de rendu sans qu'aucun
//! banc ne puisse le voir ; trois cent cinquante-huit images se sont empilées sans qu'aucune
//! mesure ne le dise. **Ce qui manque n'est pas un banc de plus, c'est le terrain.**
//!
//! Ce module enregistre en permanence, chez l'utilisateur, pendant l'usage réel.
//!
//! # Ce qu'il n'enregistre jamais
//!
//! Aucun contenu. Pas un mot d'une carte, pas un nom de fichier, pas un chemin, pas une
//! dimension de document. Uniquement des **durées**, des **nombres** et des **noms de gestes**.
//! C'est une contrainte de conception, pas une promesse : le type [`Instantane`] ne porte que
//! des entiers, donc il ne *peut* pas transporter de contenu.
//!
//! # Pourquoi une distribution et non une moyenne
//!
//! Une moyenne ment sur la fluidité. Cent images à 2 ms et une à 200 ms donnent une moyenne de
//! 4 ms — excellente — alors que l'utilisateur a vu un gel. **Ce qui se ressent est le pire
//! centile**, pas le centre.
//!
//! D'où un histogramme par geste, à échelle logarithmique : quatre tranches par octave, ce qui
//! borne l'erreur sur un centile à 19 %. Le coût est constant en mémoire quelle que soit la
//! durée de la session — on peut donc enregistrer des heures sans rien accumuler.
//!
//! # Et les pires images, en entier
//!
//! Un histogramme dit *combien* d'images ont gelé, jamais *pourquoi*. Les plus lentes sont donc
//! gardées entières, avec leur geste, leurs postes et leurs quantités — c'est là que se lit la
//! cause. Leur nombre est borné, donc la mémoire aussi.

use std::time::Duration;

/// Ce que l'utilisateur est en train de faire quand l'image se dessine.
///
/// Déduit de l'état de l'application, jamais déclaré : un geste qui devrait penser à
/// s'annoncer finirait par oublier, et c'est précisément le genre d'oubli qui a coûté une
/// journée de recherche cette semaine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Geste {
    /// Rien en cours : l'image vient d'une animation, d'un survol ou du système.
    Repos,
    /// La vue se déplace — clic droit ou molette latérale.
    DeplacerLaVue,
    /// La vue change d'échelle.
    Zoomer,
    /// Un ou plusieurs nœuds suivent la main.
    GlisserUnNoeud,
    /// Un nœud change de taille par une poignée.
    Redimensionner,
    /// Un objet naît sous la main.
    Dessiner,
    /// Le rectangle de sélection élastique s'étire.
    Selectionner,
    /// Du texte s'écrit dans une carte.
    EditerDuTexte,
    /// Des images finissent de se décoder en arrière-plan.
    Decoder,
    /// La caméra vole vers une cible.
    Animer,
}

impl Geste {
    /// Tous les gestes, dans l'ordre de leur indice.
    pub const TOUS: [Geste; 10] = [
        Geste::Repos,
        Geste::DeplacerLaVue,
        Geste::Zoomer,
        Geste::GlisserUnNoeud,
        Geste::Redimensionner,
        Geste::Dessiner,
        Geste::Selectionner,
        Geste::EditerDuTexte,
        Geste::Decoder,
        Geste::Animer,
    ];

    fn indice(self) -> usize {
        Self::TOUS.iter().position(|g| *g == self).unwrap_or(0)
    }

    /// Le nom court qui paraît dans le rapport.
    pub fn nom(self) -> &'static str {
        match self {
            Geste::Repos => "repos",
            Geste::DeplacerLaVue => "deplacer la vue",
            Geste::Zoomer => "zoomer",
            Geste::GlisserUnNoeud => "glisser un noeud",
            Geste::Redimensionner => "redimensionner",
            Geste::Dessiner => "dessiner",
            Geste::Selectionner => "selectionner",
            Geste::EditerDuTexte => "editer du texte",
            Geste::Decoder => "decoder des images",
            Geste::Animer => "animer la camera",
        }
    }
}

/// Combien de tranches d'histogramme par doublement de durée.
///
/// Quatre : l'erreur relative sur un centile est alors d'au plus `2^(1/4) − 1`, soit 19 %.
/// Ce n'est pas un réglage de confort — c'est le compromis entre la finesse et la mémoire, et
/// il se calcule. Huit tranches donneraient 9 % pour deux fois plus de compteurs.
const PAR_OCTAVE: usize = 4;

/// De 1 µs à 2^20 µs, soit un peu plus d'une seconde. Au-delà, tout tombe dans la dernière
/// tranche — et une image d'une seconde est de toute façon dans les « pires », gardée entière.
const OCTAVES: usize = 21;

/// Le nombre de compteurs d'un histogramme.
const TRANCHES: usize = PAR_OCTAVE * OCTAVES;

/// Combien d'images lentes on garde en entier.
///
/// Assez pour voir un motif se répéter, assez peu pour que la mémoire soit constante. Elles
/// sont tenues triées, donc en garder davantage coûterait surtout du tri.
const PIRES: usize = 32;

/// Combien de postes distincts une image peut déclarer.
///
/// Le rendu en compte quatorze aujourd'hui. La borne existe pour que l'enregistrement d'une
/// image reste de taille fixe — condition pour qu'une session de plusieurs heures n'accumule
/// rien.
pub const POSTES: usize = 24;

/// Ce qu'une image a coûté, et dans quel contexte.
///
/// **Que des entiers.** Aucun texte, aucun chemin, aucun contenu : c'est ce qui rend la
/// promesse de confidentialité vérifiable par le type plutôt que par la relecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Instantane {
    /// Depuis le début de la session.
    pub instant_ms: u32,
    /// Ce que l'image entière a coûté.
    pub duree_us: u32,
    /// L'indice du geste, au sens de [`Geste::TOUS`].
    pub geste: u8,
    /// La durée de chaque poste, dans l'ordre où ils se sont déclarés.
    pub postes_us: [u32; POSTES],
    /// Combien de nœuds le culling a retenus.
    pub noeuds: u32,
    /// Combien d'images ont été posées.
    pub photos: u32,
    /// L'aire redessinée, en pixels — le cœur d'A.1.
    pub region_px: u32,
    /// Combien de fois la surface de la fenêtre les images posees couvrent, en centiemes.
    ///
    /// **Le chiffre qui distingue une image chere d'une image repetee.** Soixante-dix photos
    /// qui se recouvrent repeignent soixante-dix fois le meme ecran ; une duree seule ne le
    /// separe pas de soixante-dix photos couteuses, et les deux ne se corrigent pas pareil.
    pub surcouverture: u32,
    /// L'aire de la fenêtre, pour que la précédente soit lisible en proportion.
    pub fenetre_px: u32,
    /// Ce que les images décodées occupent, en mébioctets.
    pub images_mo: u32,
    /// Combien d'images attendent encore leur décodage.
    pub en_decodage: u16,
}

impl Instantane {
    /// La part de la fenêtre qui a été redessinée, entre 0 et 1.
    pub fn part_redessinee(&self) -> f64 {
        if self.fenetre_px == 0 {
            return 1.0;
        }
        f64::from(self.region_px) / f64::from(self.fenetre_px)
    }

    fn geste(&self) -> Geste {
        Geste::TOUS
            .get(self.geste as usize)
            .copied()
            .unwrap_or(Geste::Repos)
    }
}

/// Ce qu'on sait d'un geste : combien d'images, et comment elles se distribuent.
#[derive(Debug, Clone)]
struct Poste {
    rendues: u64,
    total_us: u64,
    pire_us: u32,
    histogramme: [u32; TRANCHES],
    /// La somme des durées de chaque poste de rendu, pour savoir **où** va le temps de ce
    /// geste — un geste lent ne l'est pas pour la même raison qu'un autre.
    postes_us: [u64; POSTES],
}

impl Default for Poste {
    fn default() -> Self {
        Self {
            rendues: 0,
            total_us: 0,
            pire_us: 0,
            histogramme: [0; TRANCHES],
            postes_us: [0; POSTES],
        }
    }
}

impl Poste {
    /// La durée sous laquelle tombe la part `p` des images, en microsecondes.
    ///
    /// Lue sur l'histogramme, donc juste à 19 % près — ce qui suffit largement pour distinguer
    /// une image de 2 ms d'une image de 80 ms, qui est la question posée.
    fn centile(&self, p: f64) -> u32 {
        if self.rendues == 0 {
            return 0;
        }
        let cible = (self.rendues as f64 * p).ceil() as u64;
        let mut cumul = 0u64;
        for (i, n) in self.histogramme.iter().enumerate() {
            cumul += u64::from(*n);
            if cumul >= cible {
                return borne_haute(i);
            }
        }
        self.pire_us
    }

    fn moyenne_us(&self) -> u64 {
        self.total_us.checked_div(self.rendues).unwrap_or(0)
    }
}

/// L'indice d'histogramme d'une durée.
fn tranche(us: u32) -> usize {
    if us == 0 {
        return 0;
    }
    // `ilog2` donne l'octave ; la partie fractionnaire se découpe en `PAR_OCTAVE` en comparant
    // le reste à des puissances intermédiaires, ce qui évite un logarithme flottant sur un
    // chemin parcouru à chaque image.
    let octave = us.ilog2() as usize;
    let base = 1u64 << octave;
    let reste = u64::from(us) - base;
    let sous = (reste * PAR_OCTAVE as u64 / base) as usize;
    (octave * PAR_OCTAVE + sous).min(TRANCHES - 1)
}

/// La durée maximale que contient cette tranche.
fn borne_haute(i: usize) -> u32 {
    let octave = i / PAR_OCTAVE;
    let sous = i % PAR_OCTAVE;
    let base = 1u64 << octave;
    let borne = base + base * (sous as u64 + 1) / PAR_OCTAVE as u64;
    borne.min(u64::from(u32::MAX)) as u32
}

/// Tout ce que la session a observé.
pub struct Chronique {
    debut: std::time::Instant,
    par_geste: Vec<Poste>,
    /// Les images les plus lentes, triées de la pire à la moins pire.
    pires: Vec<Instantane>,
    /// Les noms des postes, dans l'ordre où ils se sont déclarés.
    noms_des_postes: Vec<&'static str>,
    rendues: u64,
    /// Une image est entree dans les pires depuis la derniere sauvegarde.
    du_neuf: bool,
}

impl Default for Chronique {
    fn default() -> Self {
        Self::nouvelle()
    }
}

impl Chronique {
    pub fn nouvelle() -> Self {
        Self {
            debut: std::time::Instant::now(),
            par_geste: vec![Poste::default(); Geste::TOUS.len()],
            pires: Vec::with_capacity(PIRES + 1),
            noms_des_postes: Vec::new(),
            rendues: 0,
            du_neuf: false,
        }
    }

    /// Depuis combien de temps la session dure.
    pub fn duree(&self) -> Duration {
        self.debut.elapsed()
    }

    /// Combien d'images ont été enregistrées.
    pub fn rendues(&self) -> u64 {
        self.rendues
    }

    /// Retient l'indice d'un poste, et le crée s'il est nouveau.
    ///
    /// Les noms sont des littéraux du code, donc en nombre fini et connus d'avance : la table
    /// se remplit dans les toutes premières images puis ne bouge plus.
    pub fn poste(&mut self, nom: &'static str) -> Option<usize> {
        if let Some(i) = self.noms_des_postes.iter().position(|n| *n == nom) {
            return Some(i);
        }
        if self.noms_des_postes.len() >= POSTES {
            return None;
        }
        self.noms_des_postes.push(nom);
        Some(self.noms_des_postes.len() - 1)
    }

    /// Enregistre une image.
    pub fn enregistrer(&mut self, mut vu: Instantane) {
        vu.instant_ms = self.debut.elapsed().as_millis().min(u128::from(u32::MAX)) as u32;
        self.rendues += 1;

        let poste = &mut self.par_geste[vu.geste().indice()];
        poste.rendues += 1;
        poste.total_us += u64::from(vu.duree_us);
        poste.pire_us = poste.pire_us.max(vu.duree_us);
        poste.histogramme[tranche(vu.duree_us)] += 1;
        for (somme, us) in poste.postes_us.iter_mut().zip(vu.postes_us.iter()) {
            *somme += u64::from(*us);
        }

        // Les pires se tiennent triées : une image plus rapide que la dernière ne coûte qu'une
        // comparaison, ce qui est le cas de la quasi-totalité d'entre elles.
        if self.pires.len() == PIRES && vu.duree_us <= self.pires[PIRES - 1].duree_us {
            return;
        }
        let place = self.pires.partition_point(|p| p.duree_us > vu.duree_us);
        self.pires.insert(place, vu);
        self.pires.truncate(PIRES);
        self.du_neuf = true;
    }

    /// Les images les plus lentes, de la pire à la moins pire.
    pub fn pires(&self) -> &[Instantane] {
        &self.pires
    }

    /// Y a-t-il du neuf depuis la derniere fois qu'on a demande ?
    ///
    /// # Pourquoi cette question plutot qu'une horloge
    ///
    /// Sauvegarder la chronique "toutes les N secondes" demanderait de choisir N. Sauvegarder
    /// **quand une image plus lente que toutes les precedentes est apparue** ne demande rien :
    /// c'est exactement l'instant ou le fichier a quelque chose de plus a dire.
    ///
    /// Et c'est aussi l'instant qui compte : une session qui se termine mal aura au moins
    /// garde la trace de son pire moment.
    pub fn du_neuf(&mut self) -> bool {
        std::mem::take(&mut self.du_neuf)
    }

    /// Le nom d'un poste, s'il a été vu.
    pub fn nom_du_poste(&self, i: usize) -> Option<&'static str> {
        self.noms_des_postes.get(i).copied()
    }

    /// Combien d'images ce geste a produites.
    pub fn rendues_du_geste(&self, geste: Geste) -> u64 {
        self.par_geste[geste.indice()].rendues
    }

    /// La durée sous laquelle tombe la part `p` des images de ce geste, en microsecondes.
    pub fn centile_du_geste(&self, geste: Geste, p: f64) -> u32 {
        self.par_geste[geste.indice()].centile(p)
    }

    /// La pire image de ce geste, en microsecondes.
    pub fn pire_du_geste(&self, geste: Geste) -> u32 {
        self.par_geste[geste.indice()].pire_us
    }

    /// La durée moyenne d'une image de ce geste, en microsecondes.
    pub fn moyenne_du_geste(&self, geste: Geste) -> u64 {
        self.par_geste[geste.indice()].moyenne_us()
    }

    /// Où va le temps de ce geste, poste par poste. `None` s'il n'a jamais eu lieu.
    pub fn parts_du_geste(&self, geste: Geste) -> Option<Vec<(&'static str, u64)>> {
        let poste = &self.par_geste[geste.indice()];
        if poste.rendues == 0 {
            return None;
        }
        Some(
            poste
                .postes_us
                .iter()
                .enumerate()
                .filter_map(|(i, us)| (*us > 0).then_some(*us).zip(self.nom_du_poste(i)))
                .map(|(us, nom)| (nom, us))
                .collect(),
        )
    }
}

mod rapport;

#[cfg(test)]
mod tests;
