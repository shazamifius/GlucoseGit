//! Les images prêtes à poser, une par nœud du canevas (MIP-2), construites en cascade.
//!
//! # Pourquoi par nœud, et non par fichier
//!
//! Le décodage et les niveaux de réduction appartiennent au **fichier** : deux nœuds qui
//! montrent la même photo la décodent une fois. La taille à l'écran et la phase sous-pixel,
//! elles, appartiennent au **nœud** : deux copies d'une même image posées à deux endroits ne
//! sont pas les mêmes pixels.
//!
//! Avoir mis la vignette dans la pyramide revenait donc à n'en garder qu'une pour tout un
//! groupe de copies, qui se la reprenaient l'une à l'autre à chaque image. Le banc l'a montré
//! tout de suite : trente-six nœuds sur un même fichier, et la vignette ne servait jamais.
//!
//! # Ce que ce cache promet de ne pas faire
//!
//! Une vignette n'est mise en chantier qu'à la **deuxième** image consécutive où la même forme
//! est demandée. Pendant un zoom, aucune forme ne se répète : rien n'est mis en chantier, et le
//! chemin général sert.
//!
//! Et une vignette dont le nœud n'a pas été dessiné à la dernière image est oubliée. Le cache
//! est ainsi borné par ce qui est à l'écran, sans qu'aucun nombre ait été choisi.
//!
//! # CASCADE-1 — construire ne bloque plus une image
//!
//! La version précédente construisait **pendant** le rendu, à l'instant où la forme se
//! répétait. La chronique de terrain a dit ce que cela coûtait :
//!
//! ```text
//!   478.00ms  repos  89 photos  ecrans 1.3x  vign 89
//!             dont vignettes 471.05ms, report 1.60ms
//! ```
//!
//! Quatre-vingt-neuf vignettes dans la même image : un gel d'une demi-seconde à chaque fois
//! qu'un geste s'arrête, pendant que le dessin lui-même ne coûtait plus que 1,60 ms.
//!
//! Ce module ne construit donc plus rien de lui-même : il **note** ce qui mériterait d'être
//! construit, et l'atelier vient ensuite, dans le temps libre de l'image, en prendre autant
//! qu'il peut. C'est la promesse de la charte, mot pour mot : une action lourde a le droit de
//! durer, mais par tranches qui tiennent dans le temps libre — le rendu ne l'attend jamais.
//!
//! # Ce qui passe en premier se calcule, il ne se choisit pas
//!
//! Une vignette fait gagner, à chaque image, la différence entre échantillonner et recopier,
//! **multipliée par la surface qu'on voit d'elle**. L'ordre du chantier est donc celui des
//! surfaces visibles décroissantes : c'est la quantité de travail épargné, pas une préférence.
//!
//! Une photo recouverte à quatre-vingt-dix-neuf pour cent se retrouve ainsi en fin de file, et
//! si la vue rebouge avant son tour, elle n'aura jamais rien coûté. Le gaspillage disparaît
//! sans qu'aucun seuil n'ait eu à être posé.

use super::photo::{Forme, Pyramide};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tiny_skia::Pixmap;

/// Ce qu'on garde pour un nœud donné.
struct Entree {
    /// La forme demandée à l'image précédente, honorée ou non : c'est elle qui dit si une
    /// forme se répète, donc si elle mérite une vignette.
    demandee: Option<Forme>,
    prete: Option<(Forme, Pixmap)>,
    /// L'image à laquelle ce nœud a été dessiné pour la dernière fois.
    vue: u64,
    /// Le fichier d'où sa pyramide se tire — l'atelier en a besoin, et le nœud seul ne le dit
    /// pas : deux nœuds peuvent montrer la même photo.
    src: String,
    /// Ce qu'on voit de ce nœud, en pixels d'écran. Ordonne le chantier.
    surface: f64,
    /// La forme à construire, quand elle s'est répétée sans être prête.
    en_chantier: Option<Forme>,
}

/// Les vignettes de tous les nœuds dessinés récemment.
#[derive(Default)]
pub struct Vignettes {
    par_noeud: HashMap<String, Entree>,
    image: u64,
    faites: usize,
    /// Les pixels de vignette produits depuis le début, et ce qu'ils ont coûté.
    ///
    /// Leur rapport est le **débit observé** de cette machine, et c'est lui qui dit si la
    /// prochaine vignette tient dans le temps libre. On n'interroge donc pas le matériel sur
    /// ce qu'il vaut : on regarde ce qu'il vient de faire. Un processeur qui se bride en
    /// chauffant fait baisser ce débit de lui-même, et les tranches se raccourcissent sans que
    /// rien n'ait eu à s'en apercevoir.
    pixels_produits: u64,
    nanos_passees: u64,
    /// Combien de fois, dans l'image en cours, une vignette existait pour ce nœud **mais pour
    /// une autre forme**.
    ///
    /// C'est le chiffre qui sépare deux causes qu'aucune durée ne distingue : une vignette qui
    /// n'a pas encore été construite, et une vignette construite pour une forme que la vue a
    /// déjà quittée. La seconde est un travail fait pour rien, et elle se corrige tout
    /// autrement que la première.
    perimees: usize,
    /// Combien de vignettes ont été achevées **pour un nœud qui n'existait plus**.
    ///
    /// Du travail intégralement perdu, et le seul cas où l'atelier peut coûter cher sans rien
    /// rapporter. Il se compte depuis le début de la session : s'il monte, le chantier survit
    /// à ce qu'il construit.
    orphelines: usize,
    /// Combien de nœuds ont dû être **recréés** pendant l'image en cours, faute d'entrée.
    ///
    /// Une entrée recréée naît sans vignette. Si ce nombre égale celui des photos à chaque
    /// image, c'est que la table est vidée entre deux images — et alors rien de ce que
    /// l'atelier construit ne peut survivre assez longtemps pour servir.
    recreees: usize,
    /// Combien de chantiers ont été abandonnés parce que leur forme n'était plus demandée.
    ///
    /// S'il monte sans cesse, c'est que la vue change plus vite que l'atelier ne construit :
    /// aucune vignette n'a le temps de servir, et il faut alors s'attaquer au **coût** d'une
    /// vignette, pas à l'ordre dans lequel on les fait.
    abandonnes: usize,
    /// La vignette en cours de construction, et jusqu'où elle est remplie.
    ///
    /// Une seule à la fois : on finit avant d'en commencer une autre, sinon des tampons à
    /// demi-remplis s'accumuleraient sans que rien ne soit prêt.
    en_cours: Option<Chantier>,
}

/// Une vignette à demi construite.
struct Chantier {
    noeud: String,
    /// Le fichier d'où tirer les pixels, retenu à l'ouverture : le relire dans la table à
    /// chaque tranche coûterait une recherche et une copie par tranche.
    src: String,
    forme: Forme,
    vignette: Pixmap,
    /// La première ligne qui reste à remplir.
    ligne: u32,
}

impl Vignettes {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ouvre une image : ce qui sera demandé ensuite appartient à celle-ci.
    pub fn ouvrir(&mut self) {
        self.image += 1;
        self.perimees = 0;
        self.recreees = 0;
    }

    /// Combien de nœuds ont été recréés pendant l'image en cours.
    pub fn recreees(&self) -> usize {
        self.recreees
    }

    /// Combien de chantiers ont été abandonnés depuis le début, leur forme ayant été quittée.
    pub fn abandonnes(&self) -> usize {
        self.abandonnes
    }

    /// Combien de vignettes se sont révélées périmées pendant l'image en cours.
    pub fn perimees(&self) -> usize {
        self.perimees
    }

    /// Combien de vignettes achevées n'ont trouvé personne à qui appartenir.
    pub fn orphelines(&self) -> usize {
        self.orphelines
    }

    /// Combien de nœuds ont une vignette prête, quelle que soit sa forme.
    pub fn pretes(&self) -> usize {
        self.par_noeud
            .values()
            .filter(|e| e.prete.is_some())
            .count()
    }

    /// Y a-t-il une vignette prête pour ce nœud **à cette forme exacte** ?
    ///
    /// Sans effet de bord, contrairement à [`Vignettes::pour`] : elle sert à **prévoir** le
    /// chemin qu'une photo prendra avant de le prendre, et prévoir ne doit rien changer.
    pub fn a_prete(&self, noeud: &str, forme: Forme) -> bool {
        self.par_noeud
            .get(noeud)
            .and_then(|e| e.prete.as_ref())
            .is_some_and(|(faite, _)| *faite == forme)
    }

    /// Ferme l'image et oublie les nœuds qui n'ont pas été dessinés.
    pub fn fermer(&mut self) {
        let courante = self.image;
        self.par_noeud.retain(|_, e| e.vue == courante);
    }

    /// Combien de vignettes ont été construites depuis le début. Rend MIP-2 observable.
    pub fn faites(&self) -> usize {
        self.faites
    }

    /// Combien de nœuds sont actuellement suivis.
    pub fn suivis(&self) -> usize {
        self.par_noeud.len()
    }

    /// Combien de vignettes attendent encore leur tour.
    ///
    /// C'est la longueur de la file du chantier : tant qu'elle n'est pas nulle, une part des
    /// photos se dessine par le chemin général, plus cher.
    pub fn en_chantier(&self) -> usize {
        self.par_noeud
            .values()
            .filter(|e| e.en_chantier.is_some())
            .count()
    }

    /// Les octets que les vignettes gardées occupent.
    pub fn octets(&self) -> usize {
        self.par_noeud
            .values()
            .filter_map(|e| e.prete.as_ref())
            .map(|(_, p)| p.width() as usize * p.height() as usize * 4)
            .sum()
    }

    /// La vignette prête pour ce nœud à cette forme, s'il y en a une.
    ///
    /// **Ne construit jamais.** Rend `None` tant que la vignette n'est pas prête, et l'appelant
    /// dessine alors par le chemin général — qui donne exactement les mêmes pixels, plus
    /// lentement. Quand la forme se répète, la demande entre au chantier.
    ///
    /// `surface_visible` est ce que l'occlusion laisse voir de ce nœud, en pixels : elle ne
    /// sert qu'à ordonner le chantier.
    pub fn pour(
        &mut self,
        noeud: &str,
        src: &str,
        forme: Forme,
        surface_visible: f64,
    ) -> Option<&Pixmap> {
        let image = self.image;
        if !self.par_noeud.contains_key(noeud) {
            self.recreees += 1;
        }
        let entree = match self.par_noeud.get_mut(noeud) {
            Some(e) => e,
            None => self.par_noeud.entry(noeud.to_string()).or_insert(Entree {
                demandee: None,
                prete: None,
                vue: image,
                src: src.to_string(),
                surface: 0.0,
                en_chantier: None,
            }),
        };
        let repetee = entree.demandee == Some(forme);
        entree.demandee = Some(forme);
        entree.vue = image;
        entree.surface = surface_visible;
        if entree.src != src {
            entree.src.clear();
            entree.src.push_str(src);
            entree.prete = None;
        }

        let prete = matches!(&entree.prete, Some((faite, _)) if *faite == forme);
        if entree.prete.is_some() && !prete {
            self.perimees += 1;
        }
        if prete {
            entree.en_chantier = None;
        } else if repetee {
            entree.en_chantier = Some(forme);
        } else {
            // **La forme vient de changer : il n'y a plus rien a construire.** Laisser la
            // demande en place la faisait survivre a ce qu'elle demandait : l'atelier ouvrait
            // un chantier sur une forme que plus personne n'attendait, l'abandonnait des la
            // tranche suivante, en rouvrait un autre, sans fin. Mesure chez l'utilisateur :
            // 324 ms d'atelier par image, deux cent quarante-cinq abandons, et pas une seule
            // photo servie par une vignette.
            entree.en_chantier = None;
        }
        if !prete {
            return None;
        }
        self.par_noeud
            .get(noeud)
            .and_then(|e| e.prete.as_ref())
            .map(|(_, p)| p)
    }

    /// Le prochain nœud du chantier : celui dont on voit la plus grande surface.
    /// Le prochain nœud du chantier : celui dont on voit la plus grande surface.
    ///
    /// **La forme retenue est la DERNIÈRE demandée, pas celle notée à la mise en chantier.**
    /// Une vignette met plusieurs images à sortir ; construire la forme d'il y a vingt images
    /// revient à la livrer déjà périmée. Mesuré : trois cent quarante-trois vignettes prêtes,
    /// et les trois cent quarante-trois à une forme que la vue avait quittée.
    fn prochain(&self) -> Option<(String, String, Forme, f64)> {
        self.par_noeud
            .iter()
            .filter_map(|(id, e)| {
                e.en_chantier
                    .and(e.demandee)
                    .map(|forme| (id.clone(), e.src.clone(), forme, e.surface))
            })
            .max_by(|a, b| a.3.total_cmp(&b.3))
    }

    /// La forme en cours de construction est-elle encore celle qu'on demande ?
    ///
    /// Un chantier qu'on sait déjà périmé ne mérite pas d'être fini : le poursuivre coûte
    /// autant qu'il ne rapportera rien. Mieux vaut jeter la tranche entamée et repartir de la
    /// forme du moment.
    fn chantier_encore_bon(&self) -> bool {
        let Some(chantier) = self.en_cours.as_ref() else {
            return false;
        };
        self.par_noeud
            .get(&chantier.noeud)
            .is_some_and(|e| e.demandee == Some(chantier.forme))
    }

    /// Avance le chantier pendant au plus `budget`, et rend combien de vignettes en sont
    /// sorties.
    ///
    /// # Pourquoi on estime avant d'entamer
    ///
    /// Commencer une vignette qu'on ne finira pas dans le temps libre ferait rater l'image —
    /// et rater une image se voit, là où retarder une tranche ne se voit pas. On compare donc
    /// la taille de la prochaine au débit **observé** sur cette machine, et on s'arrête quand
    /// elle ne tient plus.
    pub fn avancer_le_chantier<'p>(
        &mut self,
        budget: Duration,
        pyramide_de: impl Fn(&str) -> Option<&'p Pyramide>,
    ) -> usize {
        let debut = Instant::now();
        let mut sorties = 0;
        loop {
            if self.en_cours.is_none() && !self.ouvrir_un_chantier() {
                break;
            }
            if !self.chantier_encore_bon() {
                // La vue a bouge depuis l'ouverture : ce qui est commence ne servira plus.
                self.abandonnes += 1;
                self.en_cours = None;
                continue;
            }
            let Some(chantier) = self.en_cours.as_mut() else {
                break;
            };
            let Some(pyramide) = pyramide_de(&chantier.src) else {
                // La photo a quitté le cache : sa vignette n'a plus d'objet. Il faut retirer
                // la demande ELLE-MÊME, et pas seulement le chantier ouvert — sans quoi le
                // tour suivant rouvrirait exactement le même, sans fin.
                let noeud = std::mem::take(&mut chantier.noeud);
                self.en_cours = None;
                if let Some(e) = self.par_noeud.get_mut(&noeud) {
                    e.en_chantier = None;
                }
                continue;
            };

            let restantes = chantier.vignette.height() - chantier.ligne;
            let largeur = u64::from(chantier.vignette.width());
            let combien = lignes_possibles(
                budget
                    .checked_sub(debut.elapsed())
                    .unwrap_or(Duration::ZERO),
                largeur,
                nanos_par_pixel(self.pixels_produits, self.nanos_passees),
            )
            .min(restantes);

            let t = Instant::now();
            let fin = chantier.ligne + combien;
            pyramide.rendre_bande(chantier.forme, &mut chantier.vignette, chantier.ligne, fin);
            let passees = t.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
            chantier.ligne = fin;
            self.pixels_produits = self
                .pixels_produits
                .saturating_add(largeur * u64::from(combien));
            self.nanos_passees = self.nanos_passees.saturating_add(passees);

            if chantier.ligne >= chantier.vignette.height() {
                let fini = self.en_cours.take().expect("le chantier vient d'etre lu");
                self.faites += 1;
                sorties += 1;
                match self.par_noeud.get_mut(&fini.noeud) {
                    Some(e) => {
                        e.prete = Some((fini.forme, fini.vignette));
                        e.en_chantier = None;
                    }
                    None => self.orphelines += 1,
                }
            }

            if debut.elapsed() >= budget {
                break;
            }
        }
        sorties
    }

    /// Ouvre le chantier du nœud dont on voit la plus grande surface. Faux s'il n'y a rien.
    fn ouvrir_un_chantier(&mut self) -> bool {
        let Some((noeud, src, forme, _)) = self.prochain() else {
            return false;
        };
        let Some(vignette) = Pyramide::vignette_vide(forme) else {
            // La forme ne tient pas en mémoire : elle ne tiendra pas davantage plus tard.
            if let Some(e) = self.par_noeud.get_mut(&noeud) {
                e.en_chantier = None;
            }
            return true;
        };
        self.en_cours = Some(Chantier {
            noeud,
            src,
            forme,
            vignette,
            ligne: 0,
        });
        true
    }
}

/// Ce qu'un pixel de vignette a coûté jusqu'ici, ou `None` tant que rien n'a été produit.
fn nanos_par_pixel(pixels: u64, nanos: u64) -> Option<f64> {
    (pixels > 0).then(|| nanos as f64 / pixels as f64)
}

/// Combien de lignes tiennent dans `reste`, et **jamais moins d'une**.
///
/// # Pourquoi au moins une, et pourquoi ce n'est pas un seuil arbitraire
///
/// Sans ce plancher, une machine dont les images mangent déjà leur période ne construirait
/// jamais rien — donc ses images resteraient chères, donc le temps libre resterait nul. Le
/// cercle se refermerait sur elle, et c'est exactement la machine qu'il fallait aider.
///
/// Une ligne est le **grain indivisible** du travail, pas un réglage : quelques microsecondes,
/// soit un millième d'une période à soixante hertz. Un dépassement de cet ordre ne peut pas
/// faire rater une image, là où une vignette entière le pouvait cent fois.
fn lignes_possibles(reste: Duration, largeur: u64, par_pixel: Option<f64>) -> u32 {
    let Some(par_pixel) = par_pixel.filter(|v| *v > 0.0) else {
        // Rien de mesuré encore : une ligne, dont le coût servira de première estimation.
        return 1;
    };
    let par_ligne = largeur as f64 * par_pixel;
    let tiennent = reste.as_nanos() as f64 / par_ligne.max(f64::MIN_POSITIVE);
    (tiennent as u64).clamp(1, u64::from(u32::MAX)) as u32
}

#[cfg(test)]
mod tests;
