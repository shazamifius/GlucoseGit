//! **VIDEO-2 — les choix d'une vidéo** (fiche 170 ; sa réponse du 25/11, `relais-q-videos`) : la
//! lecture automatique, et la vidéo gardée à côté du document qu'on y fait entrer.

use super::Store;

impl Store {
    /// **Bascule la lecture automatique** des vidéos nommées, d'un seul geste : toutes la
    /// prennent, sauf si toutes l'avaient — alors toutes la quittent (comme le verrou). Rend le
    /// nouvel état, ou `None` s'il n'y avait aucune vidéo.
    pub fn basculer_la_lecture_auto(&mut self, board_id: &str, ids: &[String]) -> Option<bool> {
        let videos: Vec<String> = (ids.iter())
            .filter(|id| self.image(board_id, id).is_some_and(|i| i.is_video))
            .cloned()
            .collect();
        let toutes = videos.iter().all(|id| {
            self.image(board_id, id)
                .is_some_and(|i| i.video.lecture_auto)
        });
        let auto = !toutes;
        if videos.is_empty() {
            return None;
        }
        for id in &videos {
            self.update_image(board_id, id, |i| i.video.lecture_auto = auto);
        }
        Some(auto)
    }

    /// **Fait entrer dans le document les vidéos nommées gardées à côté** : le scribe scelle
    /// leurs octets au geste suivant. Rend combien sont entrées.
    pub fn mettre_dans_le_document(&mut self, board_id: &str, ids: &[String]) -> usize {
        let a_cote: Vec<&String> = (ids.iter())
            .filter(|id| self.image(board_id, id).is_some_and(|i| i.video.a_cote))
            .collect();
        for id in &a_cote {
            self.update_image(board_id, id, |i| i.video.a_cote = false);
        }
        a_cote.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoardImage;

    fn store() -> (Store, String) {
        let mut store = Store::new("videos");
        let board = store.project.active_board_id.clone();
        for (id, video) in [("a", true), ("b", true), ("photo", false)] {
            let mut img = BoardImage::new(id, 0.0, 0.0, 100.0, 100.0);
            img.is_video = video;
            store.add_image(&board, img);
        }
        (store, board)
    }

    #[test]
    fn video_2_la_lecture_auto_se_bascule_pour_toutes_les_videos_choisies() {
        let (mut store, board) = store();
        let ids = ["a", "b", "photo"].map(String::from);
        let auto = |s: &Store, id| s.image(&board, id).expect("là").video.lecture_auto;
        store.update_image(&board, "a", |i| i.video.lecture_auto = true);
        assert_eq!(store.basculer_la_lecture_auto(&board, &ids), Some(true));
        assert!(auto(&store, "a") && auto(&store, "b"), "toutes la prennent");
        assert!(!auto(&store, "photo"), "une photo n'a pas de lecture");
        assert_eq!(store.basculer_la_lecture_auto(&board, &ids), Some(false));
        assert!(
            !auto(&store, "a") && !auto(&store, "b"),
            "toutes la quittent"
        );
        assert_eq!(store.basculer_la_lecture_auto(&board, &ids[2..]), None);
    }

    #[test]
    fn video_2_une_video_a_cote_entre_dans_le_document() {
        let (mut store, board) = store();
        store.update_image(&board, "a", |i| i.video.a_cote = true);
        let ids = ["a", "b"].map(String::from);
        assert_eq!(store.mettre_dans_le_document(&board, &ids), 1);
        assert!(!store.image(&board, "a").expect("là").video.a_cote);
        assert_eq!(store.mettre_dans_le_document(&board, &ids), 0);
    }

    #[test]
    fn video_2_une_video_a_cote_ne_se_scelle_pas() {
        let mut img = BoardImage::new("v", 0.0, 0.0, 100.0, 100.0);
        img.is_video = true;
        img.src = Some("/films/v.mp4".into());
        let disque = std::collections::HashMap::new();
        let dehors = |i: &BoardImage| crate::miroir_disque::reste_dehors(&disque, "main", i);
        assert!(!dehors(&img), "dans le document : elle se scelle");
        img.video.a_cote = true;
        assert!(dehors(&img), "à côté : elle reste sur le disque");
    }
}
