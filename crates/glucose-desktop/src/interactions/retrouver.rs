//! **ABSENT-1 — « Retrouver le fichier… »** (fiche 176 ; la fiche 08 § 8 de Tauri : « cliquer le
//! badge ouvre un sélecteur pour réassocier le chemin »). Un nœud lié dont le fichier a bougé dit
//! « Introuvable : clip.mp4 », en rouge pointillé (`renderer/magasin/cadre.rs`) ; choisi seul, son
//! menu et la palette proposent de le retrouver. Le fichier choisi, le nœud le suit d'un geste
//! annulable (`Store::relier_au_fichier`) : sa place, ses flèches, ses domaines restent.

use crate::app::GlucoseApp;
use crate::persist::choix::Demande;
use glucose_core::store::Store;
use glucose_core::types::{AssetRef, BoardImage};
use std::path::Path;

/// **Le nœud choisi dont le fichier lié est introuvable** : seul choisi, lié, son fichier absent
/// de cette machine — constaté ici, à l'ouverture du menu ou de la palette, jamais à chaque image.
/// Un dossier du disque suit ses fichiers lui-même (DISQUE-2) : ses nœuds n'en ont pas. Au
/// téléphone, aucun sélecteur ne rend un chemin.
pub(crate) fn a_retrouver(store: &Store) -> Option<&BoardImage> {
    let seul = store.selected_image_ids.len() == 1 && store.selected_annotation_ids.is_empty();
    if cfg!(target_os = "android") || !seul {
        return None;
    }
    let board = store.project.active_board_id.as_str();
    let disque = glucose_core::miroir_disque::tableaux_du_disque(&store.project);
    let img = store.image(board, &store.selected_image_ids[0])?;
    match &img.asset {
        Some(AssetRef::Link { href, .. }) if !disque.contains_key(board) => {
            (!Path::new(href).exists()).then_some(img)
        }
        _ => None,
    }
}

/// Un fichier du même genre : une vidéo pour une vidéo, une image pour une image. Au téléphone,
/// rien n'est à retrouver ([`a_retrouver`]) ni ne s'ouvre : pas de filtre.
fn du_meme_genre(video: bool) -> crate::dialogue::Fichier {
    let dialogue = crate::dialogue::fichier();
    #[cfg(target_os = "android")]
    let _ = video;
    #[cfg(not(target_os = "android"))]
    let dialogue = match video {
        true => dialogue.filtre("Vidéos", &crate::renderer::video::signature::EXTENSIONS),
        false => dialogue.filtre("Images", &crate::interactions::clipboard::IMAGE_EXTENSIONS),
    };
    dialogue
}

impl GlucoseApp {
    /// **Retrouver le fichier…** : un sélecteur d'un fichier du même genre ; le choix revenu, le
    /// nœud le suit ([`Self::relier_au_fichier`]).
    pub(crate) fn retrouver_le_fichier(&mut self) {
        let Some(img) = a_retrouver(&self.store) else {
            return;
        };
        let fichier = du_meme_genre(img.is_video);
        let demande = Demande::Retrouver {
            board: self.store.project.active_board_id.clone(),
            id: img.id.clone(),
        };
        self.demander_un_fichier(fichier, crate::dialogue::Mode::Un, demande);
    }

    /// **Le fichier retrouvé** : le nœud le suit, avec sa taille, d'un geste annulable.
    pub(crate) fn relier_au_fichier(&mut self, board: &str, id: &str, chemin: &Path) {
        let Some(texte) = chemin.to_str() else {
            return;
        };
        let taille = std::fs::metadata(chemin).ok().map(|m| m.len());
        self.store.relier_au_fichier(board, id, texte, taille);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commandes::Commande;

    /// Une vidéo gardée à côté, son fichier parti ; puis il le retrouve ailleurs.
    #[test]
    fn absent_1_retrouver_le_fichier_relie_la_video_et_s_annule() {
        let dossier = std::env::temp_dir().join(format!("absent-1-app-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).expect("le dossier");
        let (parti, ici) = (dossier.join("parti.mp4"), dossier.join("ici.mp4"));
        std::fs::write(&ici, b"12345").expect("le fichier retrouve");

        let mut app = GlucoseApp::new();
        let board = app.store.project.active_board_id.clone();
        // Loin du nœud d'accueil, qui prendrait le clic au-dessus d'elle.
        let mut img = BoardImage::new("v", 3000.0, 3000.0, 100.0, 100.0);
        let chemin = parti.to_string_lossy().to_string();
        img.src = Some(chemin.clone());
        img.asset = Some(AssetRef::Link {
            href: chemin,
            sha256: None,
            size_bytes: None,
        });
        (img.is_video, img.video.a_cote) = (true, true);
        app.store.add_image(&board, img);
        app.store.selected_image_ids = Vec::<String>::new().into();
        assert!(
            !app.s_applique(Commande::RetrouverLeFichier),
            "rien de choisi"
        );
        app.store.selected_image_ids = (vec!["v".into()]).into();
        assert!(
            app.s_applique(Commande::RetrouverLeFichier),
            "seule, introuvable"
        );
        let dans = Commande::Video(crate::interactions::video::Choix::DansLeDocument);
        assert!(!app.s_applique(dans), "introuvable, rien à faire entrer");

        // En échec, son ▶ ne se dessine ni ne se touche : elle ne peut pas jouer. Une image d'abord,
        // qui dresse l'index des nœuds : sans lui, rien ne se toucherait, échec ou pas.
        crate::interactions::resize::tests::render_frame(&mut app);
        let src = parti.to_string_lossy().to_string();
        app.renderer.magasin.echecs.insert(src.clone());
        app.mouse_pos = crate::canvas::world_to_screen(3000.0, 3000.0, &app.store.viewport());
        assert!(
            !app.toucher_une_video(),
            "pas de ▶ sur une vidéo introuvable"
        );
        app.renderer.magasin.echecs.remove(&src);
        crate::dialogue::epreuve::choisir(Some(vec![ici.clone()]));
        app.executer(Commande::RetrouverLeFichier);
        app.suivre_les_fichiers_choisis();
        let v = app.store.image(&board, "v").expect("là");
        assert_eq!(
            v.src.as_deref(),
            ici.to_str(),
            "elle suit le fichier retrouvé"
        );
        assert!(
            matches!(
                v.asset,
                Some(AssetRef::Link {
                    size_bytes: Some(5),
                    ..
                })
            ),
            "avec sa taille"
        );
        assert!(v.video.a_cote, "toujours à côté");
        assert!(app.s_applique(dans), "retrouvée, elle peut entrer");
        assert!(
            !app.s_applique(Commande::RetrouverLeFichier),
            "retrouvée : plus rien à retrouver"
        );

        app.annuler();
        assert!(
            app.s_applique(Commande::RetrouverLeFichier),
            "introuvable à nouveau"
        );
        // Le même nœud, sur le tableau d'un dossier du disque : le miroir suit ses fichiers.
        let mut dossier_du_disque = glucose_core::types::CanvasFolder::new("d", "Photos", &board);
        dossier_du_disque.mirror_source = Some(glucose_core::miroir_disque::source_en_attente(
            &dossier.to_string_lossy(),
            None,
        ));
        app.store.project.boards[0].folders.push(dossier_du_disque);
        assert!(
            !app.s_applique(Commande::RetrouverLeFichier),
            "pas dans un miroir"
        );
        let v = app.store.image(&board, "v").expect("là");
        assert_eq!(
            v.src.as_deref(),
            parti.to_str(),
            "un Ctrl+Z la rend introuvable"
        );
        std::fs::remove_dir_all(&dossier).ok();
    }
}
