//! **CASCADE-2 — ce que la carte rend, et dans quel ordre.**
//!
//! Extrait de [`super`], qui portait à la fois ce que la carte **détient** et ce qu'elle
//! **rend** : deux raisons de changer, et six cent sept lignes là où la fiche 05 en admet six
//! cents quand le repli des tuiles (DE-PRES-1) y est entré. La coupure tombe là où la question
//! change — là-bas la mémoire de la carte, ici le temps qu'on lui donne.

use super::{SceneGpu, Source};
use crate::renderer::voies::APoser;
use std::time::{Duration, Instant};

impl SceneGpu {
    /// **Televerse ce que la carte ne connait pas encore**, et rien d'autre.
    ///
    /// `source` n'est appelee que pour les photos absentes : une photo ne traverse le bus
    /// qu'une fois dans sa vie, au premier affichage.
    ///
    /// `a_preparer` se rend comme un repli, sur le temps qui reste, et **se garde** sans se
    /// poser : la texture d'un composant que la carte dessine en formes (fiche 73).
    pub fn assurer(
        &mut self,
        peripherique: &wgpu::Device,
        envoi: &mut crate::present::envoi::Envoi,
        (a_poser, a_preparer, budget): (&[APoser], &[APoser], Duration),
        source: &Source<'_>,
    ) {
        let mut tranche = Tranche {
            debut: Instant::now(),
            budget,
            faites: 0.0,
            reportees: 0.0,
            surface: 0.0,
            rendu: Duration::ZERO,
        };
        // Les replis — le niveau tenu d'une photo dont le voulu ne l'est pas (ETAGES-1), et la
        // texture de ce que la carte dessine en formes (fiche 73) —, une fois chacun.
        let mut vus = std::collections::HashSet::new();
        let replis: Vec<&APoser> = a_poser
            .iter()
            .filter_map(|t| t.repli.as_deref())
            .chain(a_preparer)
            .filter(|r| vus.insert(&r.cle))
            .collect();
        // **Deux tours, et leur ordre EST la priorité.** Au premier, ce que la carte n'a pas
        // du tout : sans texture, un composant ne se dessine pas, et un trou se voit plus
        // qu'un flou. Au second, ce qu'elle détient mais qui a vieilli. L'urgent mange donc
        // le budget en premier, et le périmé prend ce qui reste — sans qu'aucun second
        // budget ait eu à être choisi.
        //
        // **Dans chaque tour, les replis passent après tout le reste**, et seulement sur le
        // temps qui reste : ils ne montrent rien par eux-mêmes, ils préparent le prochain zoom.
        // Mais ils sont toujours demandés tant qu'ils peuvent servir — les réserver à ce qui
        // manque les laissait sans jamais de temps, puisque ce qui manque passe avant eux.
        //
        // **Ce qui est à jour ne se regarde qu'une fois** (PHOTO-1) : à huit mille photos à
        // l'écran, tout est à jour presque toujours, et le second tour revoyait chacune pour rien.
        let en_retard: Vec<&APoser> = a_poser
            .iter()
            .filter(|t| !self.connait(&t.identite, &t.cle))
            .collect();
        for urgent in [true, false] {
            for t in &en_retard {
                self.rendre_si_besoin((peripherique, envoi), (t, urgent), source, &mut tranche);
            }
            for repli in &replis {
                if tranche.debut.elapsed() < tranche.budget {
                    self.rendre_si_besoin(
                        (peripherique, envoi),
                        (repli, urgent),
                        source,
                        &mut tranche,
                    );
                }
            }
        }
        // **Ce qui est reporté se fera à l'image suivante — encore faut-il qu'elle vienne**
        // (NETTETE-1). La cascade le dit, et la boucle la redemande tant qu'elle n'a pas fini.
        // Seul compte ce que la règle « au moins une par image » couvre : c'est elle qui fait
        // finir la scène en un nombre fini d'images. Un repli ne force jamais une image ; le
        // compter garderait éveillée, sans fin, une machine dont chaque image dépasse déjà le
        // plancher — le téléphone.
        self.inachevee = tranche.reportees > 0.0;
        // Ce qui se prépare se garde, comme ce qui se pose : oublié à la fin de chaque image où
        // il ne sert pas, il ne serait jamais là quand il sert.
        let image = self.image;
        for t in a_preparer {
            if let Some(gardee) = self.photos.get_mut(&t.identite) {
                gardee.vue = image;
            }
        }
        self.clore(&tranche);
    }

    /// **La cascade se ferme** : l'instant d'où la fin de l'image se mesure (QUEUE-1), et ce
    /// qu'elle a fait, pour la mesure.
    fn clore(&mut self, tranche: &Tranche) {
        self.fin_de_cascade = Some((Instant::now(), tranche.faites as u64));
        // Ce que la fin de cette image devrait coûter : nul tant que la présentation ne l'a
        // jamais apprise — c'est la mesure réelle qui dit qu'elle l'apprend.
        let fin_prevue = self.queue.prevoir(tranche.faites as u64);
        crate::perf::compteur(
            "fin_prevue_us",
            fin_prevue.unwrap_or_default().as_secs_f64() * 1e6,
        );
        crate::perf::compteur("textures_kpx", tranche.surface);
        crate::perf::compteur("textures_rendu_us", tranche.rendu.as_secs_f64() * 1e6);
        crate::perf::compteur("textures_faites", tranche.faites);
        crate::perf::compteur("textures_reportees", tranche.reportees);
    }

    /// **La dernière cascade a-t-elle reporté du contenu ?** (NETTETE-1)
    pub fn inachevee(&self) -> bool {
        self.inachevee
    }

    /// **L'image est à l'écran** : ce que sa fin a coûté depuis la cascade s'apprend (QUEUE-1) —
    /// moins `attente`, le temps passé à attendre que l'écran rende une image. Attendre n'est pas
    /// travailler : comptée, l'attente de l'écran (jusqu'à vingt millisecondes quand la souris
    /// bouge, sur les images qui ne font aucune texture) tordait la droite, qui prévoyait quatre
    /// millisecondes de fin pour une texture et zéro pour deux cent quatre-vingts (fiche 110).
    pub fn noter_la_fin_de_l_image(&mut self, attente: Duration) {
        if let Some((fin, faites)) = self.fin_de_cascade.take() {
            self.queue
                .noter(faites, fin.elapsed().saturating_sub(attente));
        }
    }

    /// Rend et téléverse `t` si ce tour le concerne et que le budget le permet.
    fn rendre_si_besoin(
        &mut self,
        (peripherique, envoi): (&wgpu::Device, &mut crate::present::envoi::Envoi),
        (t, urgent): (&APoser, bool),
        source: &Source<'_>,
        tranche: &mut Tranche,
    ) {
        if self.connait(&t.identite, &t.cle) || self.detient(&t.identite) == urgent {
            return;
        }
        // **CASCADE-3 : on prévoit avant d'entamer.** Vérifier qu'il reste du temps ne suffit
        // pas : une carte de texte grande comme l'écran, commencée avec une milliseconde de
        // marge, en prend quinze — l'image de zoom de 35 ms du 24/09, dont 28 en textures.
        // Ce qu'elle coûtera se lit sur ce qu'ont coûté les précédentes, par pixel posé : la
        // seule grandeur connue avant de la rendre. Rien n'est choisi, et tant que rien n'est
        // mesuré, la règle d'avant.
        let surface =
            (f64::from(t.pose.largeur.max(1.0)) * f64::from(t.pose.hauteur.max(1.0))) as u64;
        let prevu = self.debit.prevoir(surface).unwrap_or_default();
        // **QUEUE-1 : l'image ne finit pas avec la cascade** (fiche 110). Après elle viennent
        // les poses, l'envoi à la carte et la présentation — de 1 à 4 ms sur sa carte de
        // Wikipédia, d'autant plus que la cascade a fait de textures. Le budget est ce qui
        // reste de la période : la fin de l'image y tient aussi, ou l'écran saute un battement
        // et la netteté d'un cran de molette arrive une image plus tard (fiche 109 § 2).
        let fin = self
            .queue
            .prevoir(tranche.faites as u64 + 1)
            .unwrap_or_default();
        // **Le budget vaut pour les deux tours**, et la première version se trompait ici.
        // Elle n'en exemptait que l'urgent, au motif qu'un trou est pire qu'un flou — vrai
        // pour une carte isolée, faux pour quatre cent quatre-vingts. Le terrain a tranché :
        // les douze images les plus lentes de la session du 21/09 au soir sont toutes des
        // dézooms et des vols de caméra, où toutes les cartes entrent à l'écran ensemble, et
        // `textures` y coûte jusqu'à 57,6 ms. Geler une image d'un vingtième de seconde se
        // voit bien plus que deux cents cartes qui paraissent une image plus tard.
        //
        // **Au moins une par image, toujours.** C'est ce qui garantit qu'une scène finit par
        // se compléter, même quand chaque image dépasse déjà le plancher et que le budget est
        // nul — la même raison qui fait que `tranche_de_fond` ne rend jamais zéro à l'atelier
        // de décodage.
        if tranche.faites > 0.0 && tranche.debut.elapsed() + prevu + fin >= tranche.budget {
            tranche.reportees += 1.0;
            return;
        }
        let debut = Instant::now();
        if let Some(pixels) = source(&t.cle) {
            tranche.rendu += debut.elapsed();
            let pixels = pixels.vue();
            // **Ce qu'une texture pese vraiment**, en kilopixels. Les chroniques du 21/09
            // montrent UNE texture a dix-neuf millisecondes, ce qu'aucune carte de texte
            // ordinaire ne peut couter : il faut donc savoir laquelle, et la surface est la
            // seule grandeur qui puisse l'expliquer.
            tranche.surface += f64::from(pixels.width()) * f64::from(pixels.height()) / 1000.0;
            self.televerser(peripherique, envoi, (&t.identite, &t.cle), pixels);
            // Ce que la clé montre, et non d'où viennent les pixels (PHOTO-2).
            if t.cle.est_une_photo() {
                self.marquer_photo(&t.identite);
            }
            self.debit.noter(surface, debut.elapsed());
            tranche.faites += 1.0;
        }
    }
}

/// **Une durée qui suit une grandeur, apprise sur cette machine** : un prix fixe, plus un prix par
/// unité — la droite des moindres carrés sur tout ce qui a été mesuré depuis le début. Elle sert
/// deux fois : **une texture** coûte selon les pixels qu'elle couvre à l'écran (CASCADE-3, puis
/// CASCADE-4, fiche 109) ; **la fin d'une image** — poser, envoyer, présenter — selon les textures
/// que la cascade vient de faire (QUEUE-1, fiche 110).
///
/// **Par pixel seulement, la prévision d'une texture se trompait dans les deux sens** : le prix fixe
/// d'une texture (préparer son tampon, la rendre, l'envoyer), réparti sur les pixels de toutes celles
/// mesurées, gonflait le prix d'une grande texture après beaucoup de petites — de loin, sur sa
/// carte de Wikipédia, deux cent soixante-quinze textures de quelques centaines de pixels — et
/// sous-estimait une petite après de grandes. La droite apprend les deux prix de ce que la machine
/// montre : aucun nombre n'est choisi.
///
/// Cumulée et non glissante — glisser demanderait une fenêtre, donc un nombre à choisir —, tenue
/// par les moyennes et les écarts (Welford), que de grandes sommes n'abîment pas. Photos et cartes
/// de texte y sont mêlées : le rôle de la prévision est d'écarter la texture de quinze
/// millisecondes et de borner la rafale, pas de compter à la microseconde.
#[derive(Debug, Default, Clone, Copy)]
pub(super) struct Droite {
    n: f64,
    /// La moyenne de la grandeur, et celle des durées en secondes.
    x: f64,
    duree: f64,
    /// La somme des carrés des écarts de la grandeur à sa moyenne, et celle de leurs produits avec
    /// les écarts des durées.
    ecarts: f64,
    ensemble: f64,
}

impl Droite {
    pub(super) fn noter(&mut self, x: u64, duree: Duration) {
        let (v, t) = (x as f64, duree.as_secs_f64());
        self.n += 1.0;
        let dx = v - self.x;
        self.x += dx / self.n;
        self.duree += (t - self.duree) / self.n;
        self.ecarts += dx * (v - self.x);
        self.ensemble += dx * (t - self.duree);
    }

    /// Ce que coûtera `x`, ou rien tant que rien n'est mesuré. Tant que tout ce qui a été mesuré
    /// avait la même grandeur, la pente ne se lit pas : la durée moyenne.
    pub(super) fn prevoir(&self, x: u64) -> Option<Duration> {
        (self.n > 0.0).then(|| {
            let pente = if self.ecarts > 0.0 {
                self.ensemble / self.ecarts
            } else {
                0.0
            };
            let t = self.duree + pente * (x as f64 - self.x);
            Duration::from_secs_f64(t.max(0.0))
        })
    }
}

/// Ce qu'un appel à [`SceneGpu::assurer`] a le droit de dépenser, et ce qu'il a fait.
struct Tranche {
    debut: Instant,
    budget: Duration,
    faites: f64,
    reportees: f64,
    surface: f64,
    /// Le temps passé à **obtenir** les pixels — rendre un composant au processeur, ou lire
    /// une photo déjà décodée —, à part de leur téléversement.
    rendu: Duration,
}
