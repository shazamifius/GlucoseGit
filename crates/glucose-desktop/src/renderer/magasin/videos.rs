//! **Les vidéos, dans le magasin** (VIDEO-1, fiche 169) : un joueur par vidéo qu'on a lancée,
//! tant qu'elle est à l'écran ; chaque image qu'il tend entre comme une pyramide neuve, sous la
//! même identité — comme une toile de GIF. Une vidéo qui quitte l'écran s'arrête ; allée au bout,
//! elle reste sur sa dernière image, et ▶ la relance du début.

use super::{Magasin, Numero};
use crate::renderer::video::joueur::JoueurVideo;
use glucose_core::types::BoardImage;

/// **Où en est une vidéo**, pour ce que son nœud montre (▶, l'icône du son).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EtatVideo {
    pub joue: bool,
    pub son: bool,
}

impl Magasin {
    /// **▶ ou ⏸** : lance la vidéo de ce fichier, l'arrête sur son image, ou la reprend.
    pub fn basculer_la_video(&mut self, src: &str) {
        if let Some(joueur) = self.videos.get_mut(src) {
            joueur.basculer();
            return;
        }
        let (objets, reveil) = (self.atelier.objets(), self.reveil.clone());
        let joueur = JoueurVideo::lancer(src.to_string(), objets, reveil, false);
        self.videos.insert(src.to_string(), joueur);
    }

    /// **Réclame la photo de ce nœud, qui est à l'écran** ([`Magasin::reclamer_par_numero`]) — et
    /// une vidéo en lecture automatique s'y met à jouer (VIDEO-2). Les trois chemins du dessin (la
    /// grille, la passe directe, la carte graphique) réclament par ici : aucun ne l'oublie.
    ///
    /// Seulement quand sa première image est tenue : c'est elle qui dit le joueur encore à l'écran
    /// ([`Magasin::avancer_les_videos`]). Sans elle — un fichier gardé à côté qui a bougé —, le
    /// joueur serait lâché à l'image suivante, puis relancé : un fil par image, sans fin.
    pub fn reclamer_le_noeud(&mut self, img: &BoardImage) -> Option<Numero> {
        let src = img.src.as_deref().filter(|s| !s.is_empty())?;
        let numero = self.reclamer_par_numero(src, img.width)?;
        if img.is_video && img.video.lecture_auto {
            self.jouer_seule(src);
        }
        Some(numero)
    }

    /// **Une vidéo en lecture automatique est à l'écran** : elle joue seule, en boucle, le son
    /// coupé — sauf déjà lancée (arrêtée par ⏸, elle le reste), ou muette ici (elle ne s'est pas
    /// lue : la relancer à chaque image ne ferait que la rater encore).
    fn jouer_seule(&mut self, src: &str) {
        if self.videos.contains_key(src) || self.videos_muettes.contains(src) {
            return;
        }
        let (objets, reveil) = (self.atelier.objets(), self.reveil.clone());
        let joueur = JoueurVideo::lancer(src.to_string(), objets, reveil, true);
        self.videos.insert(src.to_string(), joueur);
    }

    /// **Le son de cette vidéo**, si elle est lancée : rallumé ou coupé.
    pub fn basculer_le_son(&mut self, src: &str) -> bool {
        self.videos
            .get_mut(src)
            .map(JoueurVideo::basculer_le_son)
            .is_some()
    }

    /// **Où en est la vidéo de ce fichier** ; `None` si elle n'est pas lancée.
    pub fn etat_video(&self, src: &str) -> Option<EtatVideo> {
        self.videos.get(src).map(|j| EtatVideo {
            joue: j.joue(),
            son: j.son(),
        })
    }

    /// **Une image de vidéo attend d'être posée**, ou une vidéo est allée au bout : la boucle
    /// redessine (le fil l'a réveillée).
    pub(super) fn une_image_de_video_attend(&mut self) -> bool {
        (self.videos.values_mut()).any(|j| j.prete() || j.fini())
    }

    /// **Avance les vidéos** : lâche celles qui ont quitté l'écran ou sont allées au bout, pose
    /// l'image que chaque fil a tendue.
    pub(super) fn avancer_les_videos(&mut self) {
        let image = self.image;
        let vu = |m: &Magasin, src: &str| m.cache.get(src).is_some_and(|e| e.vue + 1 >= image);
        let partis: Vec<String> = (self.videos.keys())
            .filter(|src| !vu(self, src))
            .cloned()
            .collect();
        for src in partis {
            self.videos.remove(&src);
        }
        let mut images = Vec::new();
        let mut finies = Vec::new();
        for (src, joueur) in &mut self.videos {
            match joueur.a_poser() {
                Some(p) => images.push((src.clone(), p)),
                None if joueur.fini() => finies.push(src.clone()),
                None => {}
            }
        }
        for (src, pyramide) in images {
            self.accueillir(src, pyramide, std::time::Duration::ZERO, true);
        }
        for src in finies {
            // Une boucle ne finit qu'en faute : muette ici, jusqu'au prochain lancement.
            if self.videos.remove(&src).is_some_and(|j| j.en_boucle()) {
                self.videos_muettes.insert(src);
            }
            self.changements += 1;
        }
    }
}
