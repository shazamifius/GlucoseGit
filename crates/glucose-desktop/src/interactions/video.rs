//! **Un clic sur une vidéo** (VIDEO-1, fiche 169) : sur ▶, elle joue ou s'arrête ; sur l'icône du
//! son, il se rallume ou se coupe. Un clic qui n'a rien déplacé, et sur le nœud du dessus
//! seulement : un glisser déplace la vidéo, comme toute photo, et la sélectionne toujours.

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use crate::renderer::video::commandes::{touche, Touche};
use crate::ui::question::{Question, Reponse, Suite};
use glucose_core::fixed::Fx;
use glucose_core::hit_priority::PickOwner;
use std::path::{Path, PathBuf};

/// Une vidéo qui attend sa réponse : son fichier, et où elle a été lâchée — en virgule fixe,
/// comme l'arène range les positions (une question se compare).
pub(crate) type Pose = (PathBuf, (Fx, Fx));

/// **La forme d'une vidéo avant sa première image** : 16:9, le format le plus courant ; elle
/// prend la sienne quand l'atelier l'a décodée (`Store::mesurer_les_videos`).
const FORME_D_ATTENTE: (f64, f64) = (1920.0, 1080.0);

/// **Un choix sur les vidéos choisies** (VIDEO-2, fiche 170), une commande chacun.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choix {
    /// Qu'elles jouent seules, ou à leur ▶ : une bascule, comme le verrou.
    LectureAutomatique,
    /// Que celles gardées à côté entrent dans le document.
    DansLeDocument,
}

impl Choix {
    /// Les deux, dans l'ordre de la palette.
    pub const TOUS: [Choix; 2] = [Choix::LectureAutomatique, Choix::DansLeDocument];

    /// La fin de son identifiant, `video.<id>` : stable, une couche de raccourcis l'écrit.
    pub fn id(self) -> &'static str {
        match self {
            Choix::LectureAutomatique => "lecture-auto",
            Choix::DansLeDocument => "dans-le-document",
        }
    }

    /// Son nom à l'écran.
    pub fn nom(self) -> &'static str {
        match self {
            Choix::LectureAutomatique => "Vidéo : lecture automatique, ou au bouton ▶",
            Choix::DansLeDocument => "Vidéo : la mettre dans le document",
        }
    }
}

impl GlucoseApp {
    /// **Une vidéo déposée** — si ce fichier en est une et que ce système sait la lire : la
    /// question d'abord, dans le document ou à côté (VIDEO-2 ; sa réponse du 25/11 : « que ce soit
    /// l'utilisateur qui choisit à chaque fois »). Le scribe scelle une vidéo dès qu'elle est
    /// posée : le choix vient avant. Des vidéos déposées ensemble partagent une question. Sinon
    /// `false`, et elle reste un lanceur, comme avant VIDEO-1.
    pub(crate) fn poser_une_video(&mut self, board: &str, path: &Path, ou: (f64, f64)) -> bool {
        use crate::renderer::video::{lisible_ici, signature};
        use std::io::Read;
        let mut debut = Vec::new();
        let lu = std::fs::File::open(path)
            .and_then(|f| f.take(signature::LONGUEUR as u64).read_to_end(&mut debut));
        if lu.is_err() || !signature::est_video(&debut) || !lisible_ici() || path.to_str().is_none()
        {
            return false;
        }
        let mut poses = match self.ui.question.take() {
            Some((_, Suite::Videos { board: b, poses })) if b == board => poses,
            _ => Vec::new(),
        };
        poses.push((path.to_path_buf(), (Fx::from_f64(ou.0), Fx::from_f64(ou.1))));
        let question = question_des_videos(&poses);
        let suite = Suite::Videos {
            board: board.to_string(),
            poses,
        };
        self.demander(question, suite);
        true
    }

    /// Combien de vidéos déposées attendent leur question.
    pub(crate) fn videos_en_question(&self) -> usize {
        match &self.ui.question {
            Some((_, Suite::Videos { poses, .. })) => poses.len(),
            _ => 0,
        }
    }

    /// **La réponse** : les vidéos se posent, dans le document ou à côté, d'un seul geste ;
    /// « Annuler » n'en pose aucune.
    pub(crate) fn repondre_aux_videos(&mut self, board: &str, poses: &[Pose], reponse: Reponse) {
        let a_cote = match reponse {
            Reponse::Oui => false,
            Reponse::Non => true,
            _ => return,
        };
        self.store.begin_live_edit();
        for (path, (x, y)) in poses {
            self.poser_la_video(board, path, (x.to_f64(), y.to_f64()), a_cote);
        }
        self.store.end_live_edit();
        self.mark_dirty();
    }

    fn poser_la_video(&mut self, board: &str, path: &Path, ou: (f64, f64), a_cote: bool) {
        let Some(src) = path.to_str().map(str::to_string) else {
            return;
        };
        let taille = crate::interactions::clipboard::taille_posee(FORME_D_ATTENTE);
        let id = self.store.generate_id("img");
        let mut img = glucose_core::types::BoardImage::new(id, ou.0, ou.1, taille.0, taille.1);
        // À côté, son lien le dit, avec sa taille (comme une photo d'un dossier miroir).
        if a_cote {
            img.asset = Some(glucose_core::types::AssetRef::Link {
                href: src.clone(),
                sha256: None,
                size_bytes: std::fs::metadata(path).ok().map(|m| m.len()),
            });
        }
        img.src = Some(src);
        (img.original_width, img.original_height) = FORME_D_ATTENTE;
        img.is_video = true;
        img.video.a_cote = a_cote;
        self.store.add_image(board, img);
    }

    /// **Un choix sur les vidéos choisies**, d'un seul geste annulable : basculer leur lecture
    /// automatique, ou faire entrer dans le document celles gardées à côté (VIDEO-2).
    pub(crate) fn choisir_pour_les_videos(&mut self, choix: Choix) {
        let board = self.store.project.active_board_id.clone();
        let ids = self.store.selected_image_ids.to_vec();
        self.store.begin_live_edit();
        match choix {
            Choix::LectureAutomatique => _ = self.store.basculer_la_lecture_auto(&board, &ids),
            Choix::DansLeDocument => _ = self.store.mettre_dans_le_document(&board, &ids),
        }
        self.store.end_live_edit();
    }

    /// **Le clic relâché touche-t-il une commande de vidéo ?** Rend `true` s'il en a actionné une.
    pub(crate) fn toucher_une_video(&mut self) -> bool {
        let vp = self.store.viewport();
        let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
        let Some(dessus) = self.pick_candidates_at(wx, wy).into_iter().next() else {
            return false;
        };
        if dessus.owner != PickOwner::Image {
            return false;
        }
        let board = self.store.project.active_board_id.clone();
        let Some(img) = self.store.image(&board, &dessus.id).filter(|i| i.is_video) else {
            return false;
        };
        let Some(src) = img.src.clone() else {
            return false;
        };
        let magasin = &mut self.renderer.magasin;
        let lancee = magasin.etat_video(&src).is_some();
        match touche(img, self.mouse_pos, &vp, lancee) {
            Some(Touche::Lecture) => magasin.basculer_la_video(&src),
            Some(Touche::Son) => drop(magasin.basculer_le_son(&src)),
            None => return false,
        }
        self.mark_dirty();
        true
    }
}

/// **La question d'une ou de plusieurs vidéos déposées** : ce que chaque réponse coûte, en mots.
fn question_des_videos(poses: &[Pose]) -> Question {
    let octets: u64 = (poses.iter())
        .filter_map(|(p, _)| std::fs::metadata(p).ok())
        .map(|m| m.len())
        .sum();
    let (titre, texte) = match poses.len() {
        1 => (
            "Une vidéo".to_string(),
            format!(
                "Dans le document, elle voyage avec lui ({} de plus). À côté, il reste léger, \
                 mais elle disparaît si son fichier bouge.",
                taille_lisible(octets)
            ),
        ),
        n => (
            format!("{n} vidéos"),
            format!(
                "Dans le document, elles voyagent avec lui ({} de plus). À côté, il reste \
                 léger, mais chacune disparaît si son fichier bouge.",
                taille_lisible(octets)
            ),
        ),
    };
    Question {
        titre,
        texte,
        choix: vec![
            ("Dans le document".into(), Reponse::Oui),
            ("À côté".into(), Reponse::Non),
            ("Annuler".into(), Reponse::Annuler),
        ],
        ..Default::default()
    }
}

/// Une taille comme on la lit : en Ko sous le mégaoctet, en Mo au-delà.
fn taille_lisible(octets: u64) -> String {
    match octets {
        0..1_000_000 => format!("{} Ko", octets.div_ceil(1000)),
        _ => format!("{} Mo", (octets + 500_000) / 1_000_000),
    }
}

/// **La question au dépôt** (VIDEO-2, fiche 170), jouée sur le vrai `GlucoseApp` avec la vraie
/// vidéo de téléphone en miniature.
#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use glucose_core::types::AssetRef;

    const ROUGE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/donnees/rouge-64x48.mp4");

    /// Deux vidéos déposées ensemble, par le chemin de la fenêtre : une seule question, rien de
    /// posé avant la réponse — et rien ne se dit « posé ».
    fn deux_videos_deposees() -> Option<GlucoseApp> {
        if !crate::renderer::video::lisible_ici() {
            return None;
        }
        let mut app = GlucoseApp::new();
        app.ui.current_toast = None; // l'accueil
        app.drop_files(&[PathBuf::from(ROUGE), PathBuf::from(ROUGE)]);
        let toast = app.ui.current_toast.as_ref().map(|t| t.message.clone());
        assert_eq!(toast, None, "la question parle pour elles");
        let (question, _) = app.ui.question.as_ref().expect("la question");
        assert_eq!(
            question.titre, "2 vidéos",
            "une seule question pour les deux"
        );
        assert!(videos(&app).is_empty(), "rien n'est posé avant la réponse");
        Some(app)
    }

    fn videos(app: &GlucoseApp) -> Vec<glucose_core::types::BoardImage> {
        app.store.active_board().expect("le tableau").images.clone()
    }

    #[test]
    fn video_2_a_cote_les_videos_se_posent_d_un_seul_geste_et_restent_dehors() {
        let Some(mut app) = deux_videos_deposees() else {
            return;
        };
        app.repondre_a_la_question(Some(Reponse::Non));
        let posees = videos(&app);
        assert_eq!(posees.len(), 2);
        for v in &posees {
            assert!(v.is_video && v.video.a_cote, "à côté");
            assert!(
                matches!(v.asset, Some(AssetRef::Link { .. })),
                "son lien le dit"
            );
        }
        app.annuler();
        assert!(videos(&app).is_empty(), "un seul Ctrl+Z les retire toutes");
    }

    #[test]
    fn video_2_dans_le_document_ou_annuler() {
        let Some(mut app) = deux_videos_deposees() else {
            return;
        };
        app.repondre_a_la_question(Some(Reponse::Oui));
        assert!(videos(&app).iter().all(|v| v.is_video && !v.video.a_cote));
        let Some(mut app) = deux_videos_deposees() else {
            return;
        };
        app.repondre_a_la_question(Some(Reponse::Annuler));
        assert!(videos(&app).is_empty(), "Annuler n'en pose aucune");
    }
}
