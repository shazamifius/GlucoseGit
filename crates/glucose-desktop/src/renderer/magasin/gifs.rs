//! **Les GIF qui bougent, dans le magasin** (GIF-2, fiche 159 § 6) : un joueur par GIF vu à
//! l'image d'avant ; chaque toile dont l'heure est venue y entre comme une pyramide neuve — une
//! génération de plus, que la carte graphique réécrit sous la même identité.

use super::Magasin;
use crate::renderer::gif::joueur::Joueur;
use std::time::Instant;

impl Magasin {
    /// **Avance les GIF** : lâche ceux qui ont quitté l'écran, lance ceux qui y sont, pose les
    /// toiles dont l'heure est venue. Un GIF dont le fil a tout joué s'arrête sur sa dernière
    /// toile, et ne se relance plus.
    pub(super) fn avancer_les_gifs(&mut self, maintenant: Instant) {
        // Moins de mouvement demandé : chaque GIF reste sur la toile qu'il montre (GIF-3).
        if self.mouvement_reduit {
            self.joueurs.clear();
            return;
        }
        let image = self.image;
        let vu = |m: &Magasin, src: &str| m.cache.get(src).is_some_and(|e| e.vue + 1 >= image);
        let partis: Vec<String> = (self.joueurs.keys())
            .filter(|src| !vu(self, src))
            .cloned()
            .collect();
        for src in partis {
            self.joueurs.remove(&src);
        }
        let a_lancer: Vec<String> = (self.gifs.iter())
            .filter(|src| vu(self, src) && !self.joueurs.contains_key(*src))
            .cloned()
            .collect();
        for src in a_lancer {
            let (objets, reveil) = (self.atelier.objets(), self.reveil.clone());
            let joueur = Joueur::lancer(src.clone(), objets, reveil, maintenant);
            self.joueurs.insert(src, joueur);
        }
        let mut finis = Vec::new();
        let mut toiles = Vec::new();
        for (src, joueur) in &mut self.joueurs {
            if let Some(pyramide) = joueur.a_poser(maintenant) {
                toiles.push((src.clone(), pyramide));
            } else if joueur.fini() {
                finis.push(src.clone());
            }
        }
        for (src, pyramide) in toiles {
            self.accueillir(src, pyramide, std::time::Duration::ZERO, true);
        }
        for src in finis {
            self.joueurs.remove(&src);
            self.gifs.remove(&src);
        }
    }

    /// **Quand la toile suivante d'un GIF doit se montrer**, parmi celles que leurs fils ont
    /// tendues ; `None` si aucune n'attend — un fil qui prépare la sienne réveillera la boucle.
    pub fn prochaine_toile(&mut self) -> Option<Instant> {
        (self.joueurs.values_mut())
            .filter_map(|joueur| {
                joueur.recevoir();
                joueur.echeance()
            })
            .min()
    }

    /// **Le système demande moins de mouvement** (GIF-3) : les GIF se figent à la prochaine
    /// récolte, et repartent quand il n'en demande plus.
    pub fn reduire_le_mouvement(&mut self, oui: bool) {
        self.mouvement_reduit = oui;
    }

    /// Le mouvement que le magasin tient pour demandé.
    pub fn mouvement_reduit(&self) -> bool {
        self.mouvement_reduit
    }

    /// **Ce qui réveille la boucle** quand un joueur tend sa toile.
    pub fn brancher_le_reveil(&mut self, reveil: crate::plateforme::Reveil) {
        self.reveil = Some(reveil);
    }
}
