//! **VIDEO-1 — une vidéo prend sa forme à l'arrivée de sa première image** (fiche 169).
//!
//! Une photo dit sa taille dans son en-tête, lu au dépôt ; une vidéo ne la dit qu'au décodeur du
//! système, qui ne répond pas dans le temps d'une image d'écran. Elle se pose donc en 16:9, et
//! prend sa forme quand l'atelier a sa première image : un seul geste, comme le redressement
//! d'une photo couchée (EXIF-1), signé « Glucose », défaisable.

use super::journal::Main;
use super::redresser::GLUCOSE;
use super::Store;
use std::collections::HashMap;

impl Store {
    /// **Donne leur forme aux nœuds vidéo dont la taille d'origine n'est pas celle de leurs
    /// images** — `mesurees` les nomme par leur fichier (`src`). Sa largeur reste, sa hauteur suit,
    /// autour de son centre. Rend combien ont changé, ou `None` si un geste est ouvert.
    pub fn mesurer_les_videos(&mut self, mesurees: &HashMap<String, (u32, u32)>) -> Option<usize> {
        if self.in_live_edit() {
            return None;
        }
        let a_mesurer: Vec<(String, String, (f64, f64))> = (self.project.boards.iter())
            .flat_map(|b| {
                b.images.iter().filter_map(|i| {
                    let &(l, h) = mesurees.get(i.src.as_deref()?)?;
                    let vraie = (f64::from(l), f64::from(h));
                    (i.is_video && l > 0 && h > 0 && (i.original_width, i.original_height) != vraie)
                        .then(|| (b.id.clone(), i.id.clone(), vraie))
                })
            })
            .collect();
        if a_mesurer.is_empty() {
            return Some(0);
        }
        let main = std::mem::replace(&mut self.journal.main, Main::Agent(GLUCOSE.into()));
        self.begin_live_edit();
        for (board, id, (l, h)) in &a_mesurer {
            self.update_image(board, id, |img| {
                (img.original_width, img.original_height) = (*l, *h);
                img.height = img.width * h / l;
            });
        }
        self.end_live_edit();
        self.journal.main = main;
        Some(a_mesurer.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoardImage;

    fn video(id: &str, src: &str) -> BoardImage {
        let mut img = BoardImage::new(id, 0.0, 0.0, 480.0, 270.0);
        img.src = Some(src.into());
        img.is_video = true;
        (img.original_width, img.original_height) = (1920.0, 1080.0);
        img
    }

    #[test]
    fn video_1_la_video_debout_prend_sa_forme_d_un_seul_geste_signe() {
        let mut store = Store::new("mesurer");
        let board = store.project.active_board_id.clone();
        store.add_image(&board, video("a", "debout.mov"));
        store.add_image(&board, video("b", "couchee.mp4"));
        let mut photo = video("c", "debout.mov");
        photo.is_video = false;
        store.add_image(&board, photo);
        store.journal.clear();
        let _ = store.journal.prendre_les_ecrits_signes();

        let mesurees = HashMap::from([
            ("debout.mov".to_string(), (1080, 1920)),
            ("couchee.mp4".to_string(), (1920, 1080)),
        ]);
        assert_eq!(store.mesurer_les_videos(&mesurees), Some(1));
        let boite = |s: &Store, id: &str| {
            let i = s.image(&board, id).expect("là");
            (i.x, i.y, i.width, i.height, i.original_width)
        };
        assert_eq!(
            boite(&store, "a"),
            (0.0, 0.0, 480.0, 853.333_333_333_333_4, 1080.0)
        );
        assert_eq!(boite(&store, "b").3, 270.0, "déjà à sa forme");
        assert_eq!(
            boite(&store, "c").3,
            270.0,
            "une photo n'est pas mesurée ici"
        );
        let ecrits = store.journal.prendre_les_ecrits_signes();
        assert_eq!(ecrits.len(), 1, "un seul geste");
        assert_eq!(ecrits[0].main, Main::Agent(GLUCOSE.into()));
        assert_eq!(store.journal.main, Main::Humain);
        assert_eq!(store.mesurer_les_videos(&mesurees), Some(0), "une fois");

        assert!(store.undo());
        assert_eq!(boite(&store, "a").3, 270.0, "défait");
        store.begin_live_edit();
        assert_eq!(
            store.mesurer_les_videos(&mesurees),
            None,
            "attend le geste ouvert"
        );
        store.end_live_edit();
    }
}
