//! **CASCADE-2 — ce que la carte rend, et dans quel ordre.**
//!
//! Extrait de [`super`], qui portait à la fois ce que la carte **détient** et ce qu'elle
//! **rend** : deux raisons de changer, et six cent sept lignes là où la fiche 05 en admet six
//! cents quand le repli des tuiles (DE-PRES-1) y est entré. La coupure tombe là où la question
//! change — là-bas la mémoire de la carte, ici le temps qu'on lui donne.

use super::SceneGpu;
use crate::renderer::voies::APoser;
use std::time::{Duration, Instant};
use tiny_skia::Pixmap;

impl SceneGpu {
    /// **Televerse ce que la carte ne connait pas encore**, et rien d'autre.
    ///
    /// `source` n'est appelee que pour les photos absentes : une photo ne traverse le bus
    /// qu'une fois dans sa vie, au premier affichage.
    pub fn assurer(
        &mut self,
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        (a_poser, budget): (&[APoser], Duration),
        source: &dyn Fn(&str) -> Option<Pixmap>,
    ) {
        let mut tranche = Tranche {
            debut: Instant::now(),
            budget,
            faites: 0.0,
            reportees: 0.0,
            surface: 0.0,
        };
        // Les replis des tuiles (DE-PRES-1), une fois chacun : les cent tuiles d'une carte vue
        // de près partagent le même.
        let mut vus = std::collections::HashSet::new();
        let replis: Vec<&APoser> = a_poser
            .iter()
            .filter_map(|t| t.repli.as_deref())
            .filter(|r| vus.insert(r.cle.as_str()))
            .collect();
        // **Deux tours, et leur ordre EST la priorité.** Au premier, ce que la carte n'a pas
        // du tout : sans texture, un composant ne se dessine pas, et un trou se voit plus
        // qu'un flou. Au second, ce qu'elle détient mais qui a vieilli. L'urgent mange donc
        // le budget en premier, et le périmé prend ce qui reste — sans qu'aucun second
        // budget ait eu à être choisi.
        //
        // **Dans chaque tour, les replis passent après tout le reste**, et seulement sur le
        // temps qui reste : ils ne montrent rien par eux-mêmes, ils préparent le prochain zoom.
        // Mais ils sont toujours demandés tant qu'une carte est découpée — les réserver aux
        // tuiles absentes les laissait sans jamais de temps, puisque ces tuiles passent avant
        // eux, et une carte ouverte de près n'en aurait jamais eu.
        for urgent in [true, false] {
            for t in a_poser {
                self.rendre_si_besoin((peripherique, file), (t, urgent), source, &mut tranche);
            }
            for repli in &replis {
                if tranche.debut.elapsed() < tranche.budget {
                    self.rendre_si_besoin(
                        (peripherique, file),
                        (repli, urgent),
                        source,
                        &mut tranche,
                    );
                }
            }
        }
        crate::perf::compteur("textures_kpx", tranche.surface);
        crate::perf::compteur("textures_faites", tranche.faites);
        crate::perf::compteur("textures_reportees", tranche.reportees);
    }

    /// Rend et téléverse `t` si ce tour le concerne et que le budget le permet.
    fn rendre_si_besoin(
        &mut self,
        (peripherique, file): (&wgpu::Device, &wgpu::Queue),
        (t, urgent): (&APoser, bool),
        source: &dyn Fn(&str) -> Option<Pixmap>,
        tranche: &mut Tranche,
    ) {
        if self.connait(&t.identite, &t.cle) || self.detient(&t.identite) == urgent {
            return;
        }
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
        if tranche.faites > 0.0 && tranche.debut.elapsed() >= tranche.budget {
            tranche.reportees += 1.0;
            return;
        }
        if let Some(pixels) = source(&t.cle) {
            // **Ce qu'une texture pese vraiment**, en kilopixels. Les chroniques du 21/09
            // montrent UNE texture a dix-neuf millisecondes, ce qu'aucune carte de texte
            // ordinaire ne peut couter : il faut donc savoir laquelle, et la surface est la
            // seule grandeur qui puisse l'expliquer.
            tranche.surface += f64::from(pixels.width()) * f64::from(pixels.height()) / 1000.0;
            self.televerser(peripherique, file, (&t.identite, &t.cle), &pixels);
            tranche.faites += 1.0;
        }
    }
}

/// Ce qu'un appel à [`SceneGpu::assurer`] a le droit de dépenser, et ce qu'il a fait.
struct Tranche {
    debut: Instant,
    budget: Duration,
    faites: f64,
    reportees: f64,
    surface: f64,
}
