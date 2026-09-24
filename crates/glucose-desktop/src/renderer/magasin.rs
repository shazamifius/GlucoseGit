//! Le magasin d'images : ce qui est décodé, ce qui est en chemin, ce qui ne viendra jamais.
//!
//! # Pourquoi ce type existe
//!
//! Quatre choses répondaient jusqu'ici séparément à la même question — « puis-je poser cette
//! image, et sous quelle forme ? » : le cache des pyramides, celui des vignettes, la liste des
//! fichiers illisibles, et l'atelier de décodage. Elles voyageaient en quatre paramètres, et
//! la passe des images en comptait huit à elle seule.
//!
//! Ce n'est pas qu'un compte : quatre paramètres qui vont toujours ensemble **sont** un type,
//! et tant qu'ils n'en forment pas un, chaque nouvel appelant doit se souvenir de l'ordre et
//! de ce qu'il faut emprunter en écriture. Le compilateur ne peut rien pour lui.
//!
//! # Les champs restent publics, et c'est délibéré
//!
//! Poser une image demande la pyramide **et** la vignette au même instant, l'une pour lire,
//! l'autre pour écrire. Passer par des méthodes emprunterait le magasin entier et rendrait la
//! chose impossible ; des champs distincts s'empruntent séparément, ce que le compilateur sait
//! vérifier. Le type gagne la cohésion sans rien perdre.

use super::atelier::{Atelier, Deplacement, Fait};
use super::photo::{Etat, Pyramide};
use super::vignette::Vignettes;
use crate::memoire::Memoire;
use glucose_core::store::Store;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

/// Une image décodée, et ce qu'on sait d'elle.
pub struct Entree {
    pub pyramide: Pyramide,
    /// Ce que sa reconstruction coûterait — **mesuré** à son décodage, jamais estimé.
    cout: Duration,
    /// La dernière image du rendu où elle a servi.
    vue: u64,
    /// Quelle pyramide : une image redécodée en a une neuve, et ce qui revient de chez un
    /// ouvrier pour l'ancienne ne s'y pose pas (ETAGES-1).
    generation: u64,
    /// Le premier rang de sa **vue d'ensemble** : ce rang et les plus réduits restent tenus
    /// quoi qu'il arrive, pour qu'un dézoom jusqu'au tableau entier ne demande rien à personne.
    queue: usize,
    /// L'image du rendu où la vue d'ensemble a été fixée : plusieurs nœuds peuvent montrer le
    /// même fichier, et c'est le plus grand d'entre eux qui compte.
    queue_vue: u64,
}

impl Entree {
    fn nouvelle(pyramide: Pyramide, cout: Duration, vue: u64, generation: u64) -> Self {
        let queue = pyramide.niveaux_construits();
        Self {
            pyramide,
            cout,
            vue,
            generation,
            queue,
            queue_vue: 0,
        }
    }

    /// Une entree fabriquee de toutes pieces, pour les tests.
    ///
    /// Le cout et la date de derniere vue ne changent rien a ce qui se dessine : seuls
    /// l'opacite et la taille comptent pour l'occlusion, et elles viennent de la pyramide.
    #[cfg(test)]
    pub fn pour_test(pyramide: Pyramide) -> Self {
        Self::nouvelle(pyramide, Duration::from_millis(1), 0, 0)
    }
}

/// **Ce que l'étagement a fait depuis le début** : combien de niveaux sont partis chez le
/// système, combien en sont revenus intacts, combien il avait jetés (ETAGES-1).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Mouvements {
    pub offerts: u64,
    pub repris: u64,
    pub perdus: u64,
}

/// Tout ce qu'il faut pour poser une image sur le canevas.
#[derive(Default)]
pub struct Magasin {
    /// Les images décodées, par chemin de fichier — plusieurs nœuds partagent la même.
    pub cache: HashMap<String, Entree>,
    /// Les images prêtes à reporter sans transformation, par nœud (MIP-2).
    pub vignettes: Vignettes,
    /// Les fichiers dont on sait qu'ils ne donneront rien : absents, corrompus, trop grands.
    /// Cache négatif (R-29) — on ne les redemande jamais.
    pub echecs: HashSet<String>,
    /// Les fils qui décodent, reprennent et offrent pendant que la scène se dessine.
    pub atelier: Atelier,
    /// L'image du rendu en cours — celle par rapport à laquelle « vue » se comprend.
    image: u64,
    /// Ce que le cache a le droit d'occuper, relu sur la machine. `None` tant qu'on n'a pas
    /// encore eu besoin de le savoir, ou sur une plateforme qui ne le dit pas.
    borne: Option<u64>,
    /// Combien d'images ont été évincées depuis le début. Rend la règle observable.
    evincees: usize,
    /// **L'échelle où le tableau entier tient dans l'écran** (ETAGES-1). Zéro tant que le
    /// rendu ne l'a pas dite : aucune vue d'ensemble n'est alors retenue.
    echelle_ensemble: f32,
    /// Les images à revoir à la fermeture : vues à cette image, voulues à la précédente, ou
    /// dont un niveau vient de rentrer. Les autres n'ont pas bougé, et ne se relisent pas.
    a_revoir: HashSet<String>,
    /// Le compteur des pyramides, pour dater chacune.
    generations: u64,
    /// Ce que l'étagement a fait depuis le début.
    mouvements: Mouvements,
}

impl Magasin {
    pub fn nouveau() -> Self {
        Self::default()
    }

    /// Avance le chantier des vignettes pendant au plus `budget` (CASCADE-1).
    ///
    /// Appelé **après** que l'image est présentée, avec ce que la période de l'écran laisse
    /// encore. Le rendu n'attend donc jamais une vignette : il se contente de dire lesquelles
    /// lui feraient gagner du temps, et celles qui ne sont pas prêtes se dessinent par le
    /// chemin général, qui donne exactement les mêmes pixels.
    ///
    /// Les deux champs s'empruntent séparément — les pyramides en lecture, les vignettes en
    /// écriture — ce qu'une méthode prenant `&mut self` entier interdirait.
    pub fn avancer_les_vignettes(&mut self, budget: std::time::Duration) -> usize {
        let Self {
            cache, vignettes, ..
        } = self;
        vignettes.avancer_le_chantier(budget, |src| cache.get(src).map(|e| &e.pyramide))
    }

    /// Ouvre une image du rendu.
    pub fn ouvrir(&mut self) {
        self.image += 1;
        self.vignettes.ouvrir();
    }

    /// **L'échelle où le tableau entier tiendrait dans l'écran** — ce qu'une vue d'ensemble
    /// montrerait de chaque image (ETAGES-1).
    ///
    /// Ce n'est pas un réglage : c'est la géométrie du document et celle de l'écran. Un
    /// tableau de dix millions de nœuds donne une échelle où chaque image tient dans un pixel,
    /// et sa vue d'ensemble ne coûte presque rien ; un tableau de trois photos garde de quoi
    /// les montrer en grand.
    ///
    /// Les bornes du contenu sont gardées tant que le document ne bouge pas (BORNES-1) : les
    /// relire à chaque image ne reparcourt rien. Zéro quand il n'y a rien à cadrer.
    pub fn regler_la_vue_d_ensemble(&mut self, store: &Store, (largeur, hauteur): (u32, u32)) {
        self.echelle_ensemble = store
            .active_board()
            .and_then(|b| store.content_bounds(&b.id))
            .filter(|r| r.width > 0.0 && r.height > 0.0)
            .map_or(0.0, |r| {
                (f64::from(largeur) / r.width).min(f64::from(hauteur) / r.height) as f32
            });
    }

    /// Ferme l'image du rendu : décide ce que chaque niveau devient, puis rend la mémoire que
    /// la machine réclame.
    pub fn fermer(&mut self) {
        self.vignettes.fermer();
        self.etager();
        self.ramener_sous_la_borne();
    }

    /// L'image déjà décodée pour ce chemin, ou rien — et une demande partie à l'atelier.
    ///
    /// # INVARIANT DECODE-1 — le rendu n'attend jamais un décodage
    ///
    /// Cette méthode ne décode pas. Elle consulte, et si l'image manque elle la **demande**
    /// (une seule fois : l'atelier refuse les doublons) puis rend `None` immédiatement.
    /// L'appelant dessine alors l'image comme ce qu'elle est à cet instant — en chemin.
    ///
    /// Mesuré avant que ce soit vrai : une photo de douze mégapixels coûtait 351 ms dans la
    /// boucle de rendu, soit cent quarante images perdues pour une seule photo. C'est ce que
    /// l'utilisateur décrivait par « importer une image fige tout », et c'était exact.
    pub fn pyramide(&mut self, src: &str) -> Option<&Pyramide> {
        if !self.reclamer(src, 0.0) {
            return None;
        }
        self.cache.get(src).map(|e| &e.pyramide)
    }

    /// Réclame cette image pour l'image du rendu en cours, et dit si elle est là.
    ///
    /// Séparée de l'accès à la pyramide pour une raison d'emprunt : poser une image demande la
    /// pyramide et les vignettes au même instant. Marquer d'abord, lire ensuite, laisse les
    /// deux champs s'emprunter séparément.
    ///
    /// C'est aussi ce marquage qui protège de l'éviction : **une image à l'écran ne s'évince
    /// jamais**, parce que l'évincer obligerait à la redemander dans la seconde.
    ///
    /// `largeur_monde` est la largeur du nœud dans le document : elle dit, avec l'échelle
    /// d'ensemble, quelle réduction de l'image reste tenue quoi qu'il arrive (ETAGES-1).
    pub fn reclamer(&mut self, src: &str, largeur_monde: f64) -> bool {
        if self.echecs.contains(src) {
            return false;
        }
        let image = self.image;
        let Some(entree) = self.cache.get_mut(src) else {
            self.atelier.demander(src);
            return false;
        };
        entree.vue = image;
        let queue = entree
            .pyramide
            .rang_pour(largeur_monde as f32 * self.echelle_ensemble);
        if entree.queue_vue != image {
            entree.queue_vue = image;
            entree.queue = queue;
        } else {
            entree.queue = entree.queue.min(queue);
        }
        if !self.a_revoir.contains(src) {
            self.a_revoir.insert(src.to_string());
        }
        true
    }

    /// **L'original de cette image est-il tenu ?** `None` si l'image n'est pas dans le magasin.
    ///
    /// S'il ne l'est pas, il est redemandé : le lire le marque voulu, et l'image est revue à la
    /// fermeture — même hors de l'écran, où rien d'autre ne la ferait revoir. C'est ce qu'un
    /// `Ctrl+B` sur une sélection qui dépasse l'écran attend (ETAGES-1).
    pub fn original_tenu(&mut self, src: &str) -> Option<bool> {
        let tenu = self.cache.get(src)?.pyramide.native().is_some();
        if !tenu && !self.a_revoir.contains(src) {
            self.a_revoir.insert(src.to_string());
        }
        Some(tenu)
    }

    /// Verse dans les caches tout ce que l'atelier a fini : les images décodées, et les
    /// niveaux revenus de chez le système.
    ///
    /// Appelée au début de chaque image, et là seulement : le rendu voit ainsi un cache qui ne
    /// bouge pas sous ses pieds pendant qu'il dessine.
    pub fn recolter(&mut self) {
        for fait in self.atelier.recolter() {
            match fait {
                // La pyramide arrive **faite** : il ne reste ici qu'un déplacement de
                // pointeurs. La construire ici coûtait 73 ms en pleine image (DECODE-1).
                Fait::Decodee((src, Some(pyramide), cout)) => self.accueillir(src, pyramide, cout),
                // Absent, illisible, ou trop grand pour tenir en mémoire : les trois se
                // constatent de la même façon, et aucun ne se redemande.
                Fait::Decodee((src, None, _)) => {
                    self.echecs.insert(src);
                }
                Fait::Deplace(d) => self.rentrer(d),
            }
        }
    }

    /// Une pyramide neuve : entièrement tenue, et à revoir — la fermeture offrira ce que
    /// l'écran ne demande pas.
    fn accueillir(&mut self, src: String, pyramide: Pyramide, cout: Duration) {
        self.generations += 1;
        let mut entree = Entree::nouvelle(pyramide, cout, self.image, self.generations);
        // Une image redécodée garde ce qu'on savait de sa vue d'ensemble.
        if let Some(ancienne) = self.cache.get(&src) {
            (entree.queue, entree.queue_vue) = (ancienne.queue, ancienne.queue_vue);
        }
        self.a_revoir.insert(src.clone());
        self.cache.insert(src, entree);
    }

    /// Un niveau rentre de chez un ouvrier, s'il appartient encore à la même pyramide.
    fn rentrer(&mut self, d: Deplacement<super::photo::Retour>) {
        let Some(entree) = self.cache.get_mut(&d.src) else {
            return;
        };
        if entree.generation != d.generation {
            return;
        }
        match &d.charge {
            super::photo::Retour::Offerts(_) => self.mouvements.offerts += 1,
            super::photo::Retour::Repris(Some(_)) => self.mouvements.repris += 1,
            super::photo::Retour::Repris(None) => self.mouvements.perdus += 1,
        }
        entree.pyramide.rentrer(d.rang, d.charge);
        self.a_revoir.insert(d.src);
    }

    /// **ETAGES-1 — tenu ce que l'écran montre, offert tout le reste.**
    ///
    /// Pour chaque image à revoir, chaque niveau : voulu à cette image ou dans la vue
    /// d'ensemble, il doit être tenu — offert, on le reprend ; perdu, on redécode. Ni l'un ni
    /// l'autre, et tenu, on l'offre. La borne de ce qui reste tenu n'est donc pas un nombre :
    /// c'est ce que l'écran demande, comme pour les tuiles et les vignettes.
    ///
    /// Une image voulue à cette image se revoit à la suivante : c'est là qu'on saura si elle
    /// a quitté l'écran, et qu'elle s'offrira.
    fn etager(&mut self) {
        let Self {
            cache,
            atelier,
            a_revoir,
            ..
        } = self;
        for src in std::mem::take(a_revoir) {
            let Some(entree) = cache.get_mut(&src) else {
                continue;
            };
            let voulus = entree.pyramide.prendre_les_voulus();
            let mut redecoder = false;
            for rang in 0..entree.pyramide.niveaux_construits() {
                let garder = voulus & (1 << rang) != 0 || rang >= entree.queue;
                let geste = match entree.pyramide.etat(rang) {
                    Etat::Offert => garder,
                    Etat::Tenu => !garder && entree.pyramide.offrable(rang),
                    Etat::Perdu => {
                        redecoder |= garder;
                        false
                    }
                    Etat::EnChemin => false,
                };
                if let Some(transit) = geste.then(|| entree.pyramide.sortir(rang)).flatten() {
                    atelier.deplacer(Deplacement {
                        src: src.clone(),
                        generation: entree.generation,
                        rang,
                        charge: transit,
                    });
                }
            }
            if redecoder {
                atelier.demander(&src);
            }
            if voulus != 0 {
                a_revoir.insert(src);
            }
        }
    }

    /// Ramène le cache sous ce que la machine autorise, en rendant d'abord le moins utile.
    ///
    /// # ADAPT-1 — la borne se lit sur la machine, elle ne se choisit pas
    ///
    /// Une borne écrite en dur est fausse deux fois : ridicule sur une machine à trente-deux
    /// gigaoctets, mortelle sur un téléphone à deux. Celle-ci vaut la moitié de ce qui est
    /// **disponible maintenant** ([`Memoire::part_pour_un_cache`]), donc elle se contracte
    /// d'elle-même quand une autre application réclame la mémoire.
    ///
    /// Depuis ETAGES-1, elle n'est plus la première ligne : ce que l'écran ne montre pas est
    /// offert, et le système le reprend seul. Elle reste celle qui borne **l'ensemble** — tenu
    /// et offert —, parce qu'une page offerte compte encore dans ce que le système a promis.
    ///
    /// # Ce qu'on rend quand il faut choisir
    ///
    /// Pas « le plus ancien » : ce que sa reconstruction coûterait, rapporté à ce qu'il
    /// occupe. Une vignette de cinq cents kilooctets décodée en deux millisecondes vaut moins
    /// qu'une photo de douze mégapixels qui en a coûté trois cents — et les deux termes sont
    /// **mesurés**, l'un par l'atelier, l'autre par la pyramide.
    ///
    /// Ce qui a servi à l'image en cours n'est jamais candidat : l'évincer obligerait à le
    /// redemander aussitôt, et le cache se mettrait à battre.
    fn ramener_sous_la_borne(&mut self) {
        if let Some(borne) = self.borne_courante() {
            self.ramener_sous(borne);
        }
    }

    /// La regle elle-meme, appliquee a une borne donnee.
    ///
    /// Separee de la lecture de la machine pour une raison de preuve : une regle qui depend de
    /// la memoire libre du moment ne se teste pas deux fois pareil. Ici, la borne est un
    /// argument, et le comportement se verifie sur des valeurs choisies -- le meme test dit la
    /// meme chose sur un telephone et sur un serveur.
    fn ramener_sous(&mut self, borne: u64) {
        let mut occupe = self.octets() as u64;
        if occupe <= borne {
            return;
        }

        let image = self.image;
        let mut candidats: Vec<(f64, String, u64)> = self
            .cache
            .iter()
            .filter(|(_, e)| e.vue != image)
            .map(|(src, e)| {
                let octets = e.pyramide.octets().max(1) as u64;
                (e.cout.as_secs_f64() / octets as f64, src.clone(), octets)
            })
            .collect();
        // Le moins utile part le premier. `total_cmp` plutôt qu'un `partial_cmp` déplié : il
        // ordonne tous les flottants, y compris ceux qu'une division par un coût nul produit.
        candidats.sort_by(|a, b| a.0.total_cmp(&b.0));

        for (_, src, octets) in candidats {
            if occupe <= borne {
                break;
            }
            self.cache.remove(&src);
            self.evincees += 1;
            occupe = occupe.saturating_sub(octets);
        }
    }

    /// Ce que le cache a le droit d'occuper, relu sur la machine.
    ///
    /// Relu à chaque image, et c'est délibéré : une borne qui ne se relit pas cesse d'être
    /// adaptative dès qu'une autre application démarre. Le coût est un appel système, mesuré
    /// à moins d'une microseconde — un millième du budget d'une image à 400 fps.
    fn borne_courante(&mut self) -> Option<u64> {
        self.borne = Memoire::du_systeme().map(|m| m.part_pour_un_cache());
        self.borne
    }

    /// Combien de décodages et de reprises sont encore en chantier.
    pub fn en_travail(&self) -> usize {
        self.atelier.en_travail()
    }

    /// Combien d'images ont été rendues à la machine depuis le début.
    pub fn evincees(&self) -> usize {
        self.evincees
    }

    /// Ce que l'étagement a fait depuis le début.
    pub fn mouvements(&self) -> Mouvements {
        self.mouvements
    }

    /// Attend que tout le chantier soit rentré.
    ///
    /// # Le seul endroit où attendre un décodage est juste
    ///
    /// Les **témoins visuels et les bancs**, qui produisent une image de référence et non une
    /// cadence : un témoin rendu avant l'arrivée des photos ne montrerait que des cadres, et
    /// ne prouverait rien. Partout ailleurs, l'invariant DECODE-1 l'interdit — le rendu
    /// n'attend jamais, il dessine ce qu'il a.
    ///
    /// Ne coûte rien quand le chantier est vide, ce qui est le cas de toutes les images sauf
    /// la première. La borne de temps n'est pas un réglage : c'est un garde-fou pour qu'un
    /// fil perdu fasse échouer un test au lieu de le faire tourner sans fin.
    pub fn attendre_le_chantier(&mut self) {
        let depart = std::time::Instant::now();
        while self.atelier.en_route() > 0
            && depart.elapsed() < std::time::Duration::from_secs(60)
        {
            self.recolter();
            std::thread::yield_now();
        }
        self.recolter();
    }

    /// Les octets que les images décodées représentent, où qu'ils soient.
    pub fn octets(&self) -> usize {
        self.cache.values().map(|e| e.pyramide.octets()).sum()
    }

    /// Les octets de leurs niveaux dans cet état — tenus, offerts, en chemin.
    pub fn octets_en(&self, etat: Etat) -> usize {
        self.cache.values().map(|e| e.pyramide.octets_en(etat)).sum()
    }
}

#[cfg(test)]
mod tests;
