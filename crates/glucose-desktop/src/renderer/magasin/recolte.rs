//! **La récolte** : ce que l'atelier a fini entre dans le magasin — les images décodées (et ce
//! qu'elles sont : fixes, animées, vidéos), les niveaux revenus de chez le système.

use super::{Entree, Magasin};
use crate::renderer::atelier::{Fait, Sorte};
use crate::renderer::photo::Pyramide;
use std::time::Duration;

impl Magasin {
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
                Fait::Decodee((src, Some(pyramide), cout, sorte)) => {
                    self.arrives.remove(&src);
                    match sorte {
                        Sorte::Animee => drop(self.gifs.insert(src.clone())),
                        // Sa forme, que ses nœuds prennent (VIDEO-1, `Store::mesurer_les_videos`).
                        Sorte::Video => {
                            let (l, h) = pyramide.dimensions_natives();
                            self.mesurees.insert(src.clone(), (l, h));
                            self.mesurees_neuves = true;
                        }
                        Sorte::Fixe => {}
                    }
                    self.accueillir(src, pyramide, cout, false);
                }
                // Absent, illisible, ou trop grand pour tenir en mémoire : les trois se
                // constatent de la même façon, et aucun ne se redemande.
                Fait::Decodee((src, None, ..)) => {
                    self.changements += 1;
                    if !self.arrives.remove(&src) {
                        self.echoue(src);
                    }
                }
                Fait::Deplace(d) => self.rentrer(d),
            }
        }
        self.avancer_les_gifs(std::time::Instant::now());
        self.avancer_les_videos();
    }

    /// Une pyramide neuve : entièrement tenue, et à revoir — la fermeture offrira ce que
    /// l'écran ne demande pas. Née d'un décodage entier, elle garde sa vue d'ensemble sur le
    /// disque ; née d'un aperçu, elle en vient.
    pub(super) fn accueillir(
        &mut self,
        src: String,
        pyramide: Pyramide,
        cout: Duration,
        toile: bool,
    ) {
        self.generations += 1;
        self.changements += 1;
        // L'aperçu d'un GIF reste sa première toile : les suivantes ne le réécrivent pas.
        let a_ecrire = !toile && self.atelier.garde_les_apercus() && pyramide.entiere();
        let orientation = pyramide.orientation();
        if orientation != image::metadata::Orientation::NoTransforms {
            let droite = (orientation.to_exif(), pyramide.dimensions_natives());
            self.tournees.insert(src.clone(), droite);
            self.tournees_neuves = true;
        }
        let mut entree = Entree::nouvelle(pyramide, cout, self.image, self.generations);
        entree.apercu_a_ecrire = a_ecrire;
        // Une image redécodée garde ce qu'on savait de sa vue d'ensemble ; une toile, en plus,
        // la dernière image qui l'a vue — se poser n'est pas être vue : sinon un GIF sorti de
        // l'écran se gardait lui-même en vie, de toile en toile (GIF-2). Ses vignettes, elles,
        // montrent l'ancienne pyramide — l'aperçu flou, la toile d'avant : elles s'oublient.
        if let Some(ancienne) = self.cache.get(&src) {
            (entree.queue, entree.queue_vue) = (ancienne.queue, ancienne.queue_vue);
            if toile {
                entree.vue = ancienne.vue;
            }
            self.vignettes.oublier_la_source(&src);
        }
        let numero = self.cache.numero(&src);
        if let Some(ancienne) = self.cache.insert(src, entree) {
            self.jeter(ancienne.pyramide);
        }
        self.remettre_a_revoir(numero);
    }
}
