//! **ABSENT-1 — un fichier lié retrouvé** (fiche 176). Un nœud lié (`AssetRef::Link` : une vidéo
//! gardée à côté, une image d'un document de Tauri) montre un fichier qui vit hors du document ;
//! déplacé ou renommé, il devient introuvable, et le nœud le dit. L'utilisateur le retrouve : le
//! nœud suit le chemin neuf.

use super::Store;
use crate::types::AssetRef;

impl Store {
    /// **Relie ce nœud au fichier retrouvé**, d'un seul geste annulable : sa clé et son lien
    /// suivent le chemin neuf, avec sa taille ; ce qu'il porte (sa place, ses flèches, ses
    /// domaines, les choix d'une vidéo) reste. Seul un nœud lié se relie — un nœud scellé n'a
    /// pas de fichier à retrouver. Rend s'il s'est relié.
    pub fn relier_au_fichier(
        &mut self,
        board_id: &str,
        id: &str,
        chemin: &str,
        taille: Option<u64>,
    ) -> bool {
        let lie = (self.image(board_id, id))
            .is_some_and(|i| matches!(i.asset, Some(AssetRef::Link { .. })));
        if lie {
            self.update_image(board_id, id, |i| {
                i.src = Some(chemin.to_string());
                i.asset = Some(AssetRef::Link {
                    href: chemin.to_string(),
                    sha256: None,
                    size_bytes: taille,
                });
            });
        }
        lie
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoardImage;

    fn lien(href: &str) -> Option<AssetRef> {
        Some(AssetRef::Link {
            href: href.into(),
            sha256: None,
            size_bytes: Some(3),
        })
    }

    #[test]
    fn absent_1_le_noeud_lie_suit_son_fichier_et_s_annule() {
        let mut store = Store::new("retrouver");
        let board = store.project.active_board_id.clone();
        let mut img = BoardImage::new("v", 10.0, 20.0, 100.0, 100.0);
        (img.src, img.asset) = (Some("/avant/clip.mp4".into()), lien("/avant/clip.mp4"));
        img.is_video = true;
        img.video.a_cote = true;
        store.add_image(&board, img);
        let mut scelle = BoardImage::new("s", 0.0, 0.0, 10.0, 10.0);
        scelle.src = Some("cle".into());
        store.add_image(&board, scelle);

        assert!(store.relier_au_fichier(&board, "v", "/apres/clip.mp4", Some(9)));
        let v = store.image(&board, "v").expect("là").clone();
        assert_eq!(v.src.as_deref(), Some("/apres/clip.mp4"), "sa clé suit");
        assert!(
            matches!(&v.asset, Some(AssetRef::Link { href, size_bytes: Some(9), .. }) if href == "/apres/clip.mp4"),
            "son lien suit, avec sa taille"
        );
        assert!(
            v.is_video && v.video.a_cote && v.x == 10.0,
            "le reste reste"
        );

        assert!(
            !store.relier_au_fichier(&board, "s", "/x.png", None),
            "un nœud scellé ne se relie pas"
        );
        assert_eq!(
            store.image(&board, "s").expect("là").src.as_deref(),
            Some("cle")
        );

        assert!(store.undo());
        let v = store.image(&board, "v").expect("là");
        assert_eq!(
            v.src.as_deref(),
            Some("/avant/clip.mp4"),
            "un seul Ctrl+Z le rend"
        );
        assert_eq!(v.asset, lien("/avant/clip.mp4"));
    }
}
