//! **Les vidéos, dans le magasin** (VIDEO-1, fiche 169) : un joueur par vidéo qu'on a lancée,
//! tant qu'elle est à l'écran ; chaque image qu'il tend entre comme une pyramide neuve, sous la
//! même identité — comme une toile de GIF. Une vidéo qui quitte l'écran s'arrête ; allée au bout,
//! elle reste sur sa dernière image, et ▶ la relance du début.

use super::Magasin;
use crate::renderer::video::joueur::JoueurVideo;

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
        let joueur = JoueurVideo::lancer(src.to_string(), objets, reveil);
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
            self.videos.remove(&src);
            self.changements += 1;
        }
    }
}
