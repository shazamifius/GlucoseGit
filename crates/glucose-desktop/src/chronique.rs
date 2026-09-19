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

pub mod histogramme;
pub mod instantane;
pub mod navigation;
pub mod rythme;

pub use histogramme::Histogramme;
pub use instantane::Instantane;
pub use rythme::Rythme;

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

/// Ce qu'on sait d'un geste : combien d'images, et comment elles se distribuent.
#[derive(Debug, Clone, Default)]
struct Poste {
    /// La distribution des durées de ses images, en microsecondes.
    durees: Histogramme,
    /// La somme des durées de chaque poste de rendu, pour savoir **où** va le temps de ce
    /// geste — un geste lent ne l'est pas pour la même raison qu'un autre.
    postes_us: [u64; POSTES],
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
    /// Ce que la navigation vit : ce que le doigt demande, et le temps qu'il faut pour que
    /// l'écran le montre (NAV-3).
    pub navigation: navigation::Navigation,
    /// Toutes les photos posées de la session, et celles qui l'ont été depuis une vignette.
    ///
    /// # Pourquoi un cumul, et pas la liste des pires
    ///
    /// Les images les plus lentes sont **biaisées par construction** : une image dont les
    /// photos ont leur vignette devient rapide, donc elle quitte la liste. En n'y lisant que
    /// des `mip 0`, on conclut que le mécanisme ne sert jamais — alors qu'on ne regarde que
    /// les cas où il n'a pas servi. Une part calculée sur **toutes** les images n'a pas ce
    /// défaut.
    photos_posees: u64,
    photos_par_vignette: u64,
    /// Les recréations de nœuds cumulées : si elles suivent le nombre de photos, la table est
    /// vidée entre deux images et rien de ce que l'atelier construit ne peut survivre.
    noeuds_recrees: u64,
    /// Les tuiles peintes et reprises, cumulées : le rapport des deux dit si la grille sert.
    tuiles_peintes: u64,
    tuiles_reprises: u64,
    /// Les images où le modèle savait prévoir : combien, ce qu'il avait prévu, ce qu'elles ont
    /// vraiment coûté, et combien se sont pixelisées.
    ///
    /// Le rapport des deux durées est **le résidu**, et c'est la seule grandeur de tout ce
    /// module qui apprenne quelque chose de neuf : nul, la machine est comprise et le budget
    /// peut se tenir par le calcul ; élevé, il désigne ce que le modèle ne compte pas encore.
    prevues: u64,
    prevu_us: u64,
    mesure_us: u64,
    pixelisees: u64,
    /// La distribution des durées de **toutes** les images, tous gestes confondus.
    ///
    /// Sans elle, « combien d'images ont raté le plancher » se lisait sur la liste bornée des
    /// trente-deux plus lentes, et une session qui en ratait mille annonçait trente-deux.
    durees: Histogramme,
    /// Ce que l'écran a montré, par opposition à ce que les images ont coûté (RYTHME-1).
    pub rythme: Rythme,
    /// Combien d'images n'ont **rien** redessiné du tout.
    evitees: u64,
    /// Combien d'images chaque raison de réveil a tenues éveillées, dans l'ordre des bits.
    reveils: [u64; 16],
    /// La somme des facteurs de reduction : sa moyenne dit a quel point la scene a du ceder
    /// sur sa finesse pour tenir la cadence.
    reductions: u64,
    reduites: u64,
}

impl Default for Chronique {
    fn default() -> Self {
        Self::nouvelle()
    }
}

impl Chronique {
    /// La part des photos de la session qui se sont posées depuis une vignette, entre 0 et 1.
    ///
    /// Se lit sur **toutes** les images, et pas sur les plus lentes, qui sont justement celles
    /// où la vignette a manqué.
    pub fn part_par_vignette(&self) -> Option<f64> {
        (self.photos_posees > 0)
            .then(|| self.photos_par_vignette as f64 / self.photos_posees as f64)
    }

    /// Le rapport entre ce que le modèle avait prévu et ce que les images ont coûté.
    ///
    /// Un vaut « le modèle voit juste ». Au-dessous, il sous-estime — et une sous-estimation
    /// fait tenir un budget qu'on dépasse, ce qui est le défaut le plus grave possible ici.
    ///
    /// Ne se lit que sur les images rendues **au plus fin** : ailleurs, la prévision porte sur
    /// un travail qui n'a pas été exécuté.
    pub fn justesse_du_modele(&self) -> Option<f64> {
        (self.prevues > 0 && self.mesure_us > 0)
            .then(|| self.prevu_us as f64 / self.mesure_us as f64)
    }

    /// La part des images qui se sont **pixelisées** pour tenir le budget, entre 0 et 1.
    /// La part des images dont la scene s'est rendue plus petite, et le facteur moyen.
    pub fn part_reduite(&self) -> Option<(f64, f64)> {
        (self.rendues > 0).then(|| {
            (
                self.reduites as f64 / self.rendues as f64,
                self.reductions as f64 / self.rendues as f64,
            )
        })
    }

    /// Combien d'images ont dépassé ce budget, sur **toutes** celles de la session.
    ///
    /// Se lisait auparavant sur la liste des trente-deux plus lentes, qui est bornée par
    /// construction : une session ratant mille images et une en ratant trente-trois
    /// annonçaient le même nombre.
    pub fn images_au_dessus(&self, budget_us: u32) -> u64 {
        self.durees.au_dessus(budget_us)
    }

    /// La part des images qui n'ont **rien** redessiné, entre 0 et 1.
    ///
    /// C'est la seule mesure qui dise si le travail de l'image précédente a servi deux fois.
    /// Proche de zéro, elle veut dire que quelque chose salit tout à chaque image — et le
    /// coût d'une image ne dira jamais laquelle.
    pub fn part_evitee(&self) -> Option<f64> {
        (self.rendues > 0).then(|| self.evitees as f64 / self.rendues as f64)
    }

    /// Combien d'images chaque raison de réveil a tenues éveillées.
    pub fn reveils(&self) -> impl Iterator<Item = (crate::app::reveil::Raison, u64)> + '_ {
        crate::app::reveil::Raison::TOUTES
            .into_iter()
            .map(|r| (r, self.reveils[r.bit().trailing_zeros() as usize]))
    }

    pub fn part_pixelisee(&self) -> Option<f64> {
        (self.rendues > 0).then(|| self.pixelisees as f64 / self.rendues as f64)
    }

    /// La part des tuiles qui ont servi telles quelles, entre 0 et 1 — le travail épargné.
    ///
    /// `None` si la grille n'a jamais servi : une vue toujours entre deux niveaux et à
    /// l'arrêt, par exemple.
    pub fn part_de_tuiles_reprises(&self) -> Option<f64> {
        let total = self.tuiles_peintes + self.tuiles_reprises;
        (total > 0).then(|| self.tuiles_reprises as f64 / total as f64)
    }

    /// Combien de tuiles ont été peintes en tout — le travail fait.
    pub fn tuiles_peintes(&self) -> u64 {
        self.tuiles_peintes
    }

    /// Combien de tuiles ont servi telles quelles — le travail épargné.
    pub fn tuiles_reprises(&self) -> u64 {
        self.tuiles_reprises
    }

    /// La part des poses qui ont dû **recréer** l'entrée du nœud, entre 0 et 1.
    ///
    /// Proche de 1, elle dit que la table est vidée entre deux images : une entrée recréée
    /// naît sans vignette, donc rien de ce que l'atelier construit ne peut jamais servir.
    pub fn part_recreee(&self) -> Option<f64> {
        (self.photos_posees > 0).then(|| self.noeuds_recrees as f64 / self.photos_posees as f64)
    }

    pub fn nouvelle() -> Self {
        Self {
            debut: std::time::Instant::now(),
            par_geste: vec![Poste::default(); Geste::TOUS.len()],
            pires: Vec::with_capacity(PIRES + 1),
            noms_des_postes: Vec::new(),
            rendues: 0,
            du_neuf: false,
            navigation: navigation::Navigation::nouvelle(),
            photos_posees: 0,
            photos_par_vignette: 0,
            noeuds_recrees: 0,
            tuiles_peintes: 0,
            tuiles_reprises: 0,
            prevues: 0,
            prevu_us: 0,
            mesure_us: 0,
            pixelisees: 0,
            durees: Histogramme::nouveau(),
            rythme: Rythme::nouveau(),
            evitees: 0,
            reveils: [0; 16],
            reductions: 0,
            reduites: 0,
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
        self.photos_posees += u64::from(vu.photos);
        self.photos_par_vignette += u64::from(vu.par_vignette);
        self.noeuds_recrees += u64::from(vu.vignettes_recreees);
        self.tuiles_peintes += u64::from(vu.tuiles_peintes);
        self.tuiles_reprises += u64::from(vu.tuiles_reprises);
        self.reductions += u64::from(vu.reduction.max(1));
        if vu.reduction > 1 {
            self.reduites += 1;
        }
        if vu.pixelise > 0 {
            self.pixelisees += 1;
        }
        if vu.evitee() {
            self.evitees += 1;
        }
        for (bit, compte) in self.reveils.iter_mut().enumerate() {
            if vu.reveils & (1 << bit) != 0 {
                *compte += 1;
            }
        }
        // **Seules les images rendues au plus fin comptent.** La prevision porte sur le
        // rendu lisse ; sur une image pixelisee, on a execute autre chose, et comparer les
        // deux fait paraitre le modele cinq fois trop pessimiste alors qu'il prevoit un
        // travail qu'on n'a simplement pas fait.
        if vu.prevu_us > 0 && vu.report_us > 0 && vu.pixelise == 0 {
            self.prevues += 1;
            self.prevu_us += u64::from(vu.prevu_us);
            self.mesure_us += u64::from(vu.report_us);
        }

        self.durees.ajouter(vu.duree_us);
        let poste = &mut self.par_geste[vu.geste().indice()];
        poste.durees.ajouter(vu.duree_us);
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
        self.par_geste[geste.indice()].durees.compte()
    }

    /// La durée sous laquelle tombe la part `p` des images de ce geste, en microsecondes.
    pub fn centile_du_geste(&self, geste: Geste, p: f64) -> u32 {
        self.par_geste[geste.indice()].durees.centile(p)
    }

    /// La pire image de ce geste, en microsecondes.
    pub fn pire_du_geste(&self, geste: Geste) -> u32 {
        self.par_geste[geste.indice()].durees.pire()
    }

    /// La durée moyenne d'une image de ce geste, en microsecondes.
    pub fn moyenne_du_geste(&self, geste: Geste) -> u64 {
        self.par_geste[geste.indice()].durees.moyenne() as u64
    }

    /// Où va le temps de ce geste, poste par poste. `None` s'il n'a jamais eu lieu.
    pub fn parts_du_geste(&self, geste: Geste) -> Option<Vec<(&'static str, u64)>> {
        let poste = &self.par_geste[geste.indice()];
        if poste.durees.compte() == 0 {
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
mod verdict;

pub use verdict::Constat;

#[cfg(test)]
mod tests;
