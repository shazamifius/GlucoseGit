//! **EXIF-1 — les nœuds posés couchés se redressent à l'arrivée de leurs pixels** (fiche 165 § 3.3).
//!
//! Ce que chaque nœud devient vit dans `types/image/redresser.rs` ; ici, le geste qui les redresse
//! tous : un seul, écrit dans le document, partagé aux autres mains, défaisable — et signé
//! « Glucose » dans l'histoire, puisque personne ne l'a fait à la main.

use super::journal::Main;
use super::Store;
use std::collections::HashMap;

/// Le nom sous lequel Glucose signe ce qu'il écrit seul dans un document.
pub const GLUCOSE: &str = "Glucose";

/// Ce qu'on sait d'une photo décodée tournée : son orientation EXIF, et ses dimensions droites.
pub type Arrivee = (u8, (u32, u32));

impl Store {
    /// **Redresse, sur tous les tableaux, les nœuds qui montrent couchée une de ces photos** —
    /// `arrivees` les nomme par leur fichier (`src`). Rend combien l'ont été, ou `None` si un geste
    /// est ouvert : celui-ci attend qu'il se ferme, plutôt que de s'y fondre (un `Échap` défairait
    /// les deux).
    ///
    /// Un verrou n'y change rien : il garde un nœud de la main, et ses pixels arrivent droits de
    /// toute façon — le laisser, c'est le montrer écrasé.
    pub fn redresser_les_photos(&mut self, arrivees: &HashMap<String, Arrivee>) -> Option<usize> {
        if self.in_live_edit() {
            return None;
        }
        let couches: Vec<(String, String, Arrivee)> = self
            .project
            .boards
            .iter()
            .flat_map(|b| {
                b.images.iter().filter_map(|i| {
                    let &(exif, pixels) = arrivees.get(i.src.as_deref()?)?;
                    i.couchee(exif, pixels)
                        .then(|| (b.id.clone(), i.id.clone(), (exif, pixels)))
                })
            })
            .collect();
        if couches.is_empty() {
            return Some(0);
        }
        let main = std::mem::replace(&mut self.journal.main, Main::Agent(GLUCOSE.into()));
        self.begin_live_edit();
        for (board, id, (exif, pixels)) in &couches {
            self.update_image(board, id, |img| {
                img.redresser(*exif, *pixels);
            });
        }
        self.end_live_edit();
        self.journal.main = main;
        Some(couches.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoardImage;

    fn couchee(id: &str, src: &str) -> BoardImage {
        let mut img = BoardImage::new(id, 0.0, 0.0, 400.0, 300.0);
        img.src = Some(src.into());
        (img.original_width, img.original_height) = (4000.0, 3000.0);
        img
    }

    /// **Un seul geste, signé « Glucose », qui se défait** — et seuls les nœuds de ces photos-là,
    /// couchés, changent.
    #[test]
    fn test_les_noeuds_couches_se_redressent_d_un_seul_geste_signe() {
        let mut store = Store::new("redresser");
        let board = store.project.active_board_id.clone();
        store.add_image(&board, couchee("a", "portrait.jpg"));
        store.add_image(&board, couchee("b", "portrait.jpg"));
        store.add_image(&board, couchee("c", "autre.jpg"));
        let mut debout = couchee("d", "portrait.jpg");
        (debout.width, debout.height) = (300.0, 400.0);
        (debout.original_width, debout.original_height) = (3000.0, 4000.0);
        store.add_image(&board, debout);
        store.journal.clear();
        let _ = store.journal.prendre_les_ecrits_signes();

        let arrivees = HashMap::from([("portrait.jpg".to_string(), (6, (3000, 4000)))]);
        assert_eq!(store.redresser_les_photos(&arrivees), Some(2));
        let taille = |s: &Store, id: &str| {
            let i = s.image(&board, id).expect("là");
            (i.width, i.height)
        };
        assert_eq!(taille(&store, "a"), (300.0, 400.0));
        assert_eq!(taille(&store, "b"), (300.0, 400.0));
        assert_eq!(taille(&store, "c"), (400.0, 300.0), "une autre photo");
        assert_eq!(taille(&store, "d"), (300.0, 400.0), "déjà debout");
        let ecrits = store.journal.prendre_les_ecrits_signes();
        assert_eq!(ecrits.len(), 1, "un seul geste");
        assert_eq!(ecrits[0].main, Main::Agent(GLUCOSE.into()));
        assert_eq!(
            store.journal.main,
            Main::Humain,
            "la main revient à l'humain"
        );
        assert_eq!(store.redresser_les_photos(&arrivees), Some(0), "une fois");

        assert!(store.undo());
        assert_eq!(taille(&store, "a"), (400.0, 300.0), "défait d'un coup");
        assert_eq!(taille(&store, "b"), (400.0, 300.0));
    }

    /// **Un geste ouvert le fait attendre** : il ne s'y fond pas.
    #[test]
    fn test_un_geste_ouvert_fait_attendre_le_redressement() {
        let mut store = Store::new("redresser");
        let board = store.project.active_board_id.clone();
        store.add_image(&board, couchee("a", "portrait.jpg"));
        let arrivees = HashMap::from([("portrait.jpg".to_string(), (6, (3000, 4000)))]);
        store.begin_live_edit();
        assert_eq!(store.redresser_les_photos(&arrivees), None);
        store.end_live_edit();
        assert_eq!(store.redresser_les_photos(&arrivees), Some(1));
    }
}
