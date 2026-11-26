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
            autre => {
                if let Some((_, suite)) = autre {
                    self.laisser(suite);
                }
                Vec::new()
            }
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
            _ => return self.rendre_les_copies_recues(poses),
        };
        self.store.begin_live_edit();
        for (path, (x, y)) in poses {
            self.poser_la_video(board, path, (x.to_f64(), y.to_f64()), a_cote);
        }
        self.store.end_live_edit();
        // Dans le document, une copie reçue y est deux fois : elle se retire une fois scellée.
        if !a_cote {
            let recues: Vec<PathBuf> = (poses.iter().map(|(p, _)| p))
                .filter(|p| self.est_recue(p))
                .cloned()
                .collect();
            self.depot.a_retirer.extend(recues);
        }
        self.mark_dirty();
    }

    /// **RECUE-1 — une copie reçue que plus rien ne désigne se retire** (fiche 172 § 8). Une
    /// vidéo partagée au téléphone se copie dans le dossier privé de Glucose, que l'utilisateur ne
    /// voit pas : « Annuler », ou la question fermée sans réponse, l'y laissaient pour rien.
    pub(crate) fn rendre_les_copies_recues(&mut self, poses: &[Pose]) {
        for (chemin, _) in poses.iter().filter(|(p, _)| self.est_recue(p)) {
            retirer(chemin);
        }
    }

    /// **Une suite qu'aucune réponse ne suivra** — la question fermée, ou remplacée : ce
    /// qu'elle tenait se rend.
    pub(crate) fn laisser(&mut self, suite: Suite) {
        if let Suite::Videos { poses, .. } = suite {
            self.rendre_les_copies_recues(&poses);
        }
    }

    /// Les copies mises dans le document, retirées dès que le scribe les a scellées : le
    /// document ne dépend plus d'elles. Une copie dont le document s'est fermé avant reste.
    pub(crate) fn retirer_les_copies_scellees(&mut self) {
        let objets = &self.disque.objets;
        self.depot.a_retirer.retain(|chemin| {
            let scellee = chemin.to_str().is_some_and(|cle| objets.est_scellee(cle));
            if scellee {
                retirer(chemin);
            }
            !scellee
        });
    }

    /// Ce fichier est-il une copie reçue, à Glucose seul ?
    fn est_recue(&self, chemin: &Path) -> bool {
        (self.depot.recues.as_deref()).is_some_and(|dossier| chemin.starts_with(dossier))
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

/// Retire une copie reçue ; déjà partie, rien à dire.
fn retirer(chemin: &Path) {
    match std::fs::remove_file(chemin) {
        Ok(()) => crate::dire!("[video] copie reçue retirée : {}", chemin.display()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => crate::dire_erreur!("[video] copie reçue gardée ({e}) : {}", chemin.display()),
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

    /// RECUE-1 (fiche 172 § 8) — **Une copie reçue que plus rien ne désigne se retire** :
    /// « Annuler », la question fermée sans réponse ou remplacée par une autre la retirent tout de
    /// suite ; « À côté » la garde, le document la désigne ; « Dans le document » la retire une
    /// fois scellée, pas avant. Un fichier hors du dossier des copies reçues — celui de
    /// l'utilisateur — ne se touche jamais.
    #[test]
    fn recue_1_une_copie_recue_que_rien_ne_designe_se_retire() {
        use crate::persist::disque::tests::{application, dossier, image_suivante};
        if !crate::renderer::video::lisible_ici() {
            return;
        }
        let d = dossier("recue-1");
        let recues = d.join("telechargements");
        std::fs::create_dir_all(&recues).expect("le dossier des copies");
        let copie = |dossier: &Path, nom: &str| {
            let c = dossier.join(nom);
            std::fs::write(&c, std::fs::read(ROUGE).expect("la vidéo")).expect("une copie");
            c
        };
        let mut app = application(&d);
        app.save_to(d.join("doc.glucose"));
        app.depot.recues = Some(recues.clone());
        let deposer = |app: &mut GlucoseApp, chemin: &Path, reponse| {
            app.drop_files(&[chemin.to_path_buf()]);
            assert_eq!(app.videos_en_question(), 1);
            app.repondre_a_la_question(reponse);
        };

        let a_lui = copie(&d, "a-lui.mp4");
        deposer(&mut app, &a_lui, Some(Reponse::Annuler));
        assert!(a_lui.exists(), "le sien ne se touche jamais");
        let annulee = copie(&recues, "partage");
        deposer(&mut app, &annulee, Some(Reponse::Annuler));
        assert!(!annulee.exists(), "Annuler la retire");
        let fermee = copie(&recues, "partage (1)");
        deposer(&mut app, &fermee, None);
        assert!(!fermee.exists(), "la question fermée sans réponse aussi");
        let remplacee = copie(&recues, "partage (2)");
        app.drop_files(std::slice::from_ref(&remplacee));
        app.demander(Question::default(), Suite::NouveauDocument);
        assert!(!remplacee.exists(), "la question remplacée aussi");
        app.repondre_a_la_question(None);

        let a_cote = copie(&recues, "partage (3)");
        deposer(&mut app, &a_cote, Some(Reponse::Non));
        let dedans = copie(&recues, "partage (4)");
        deposer(&mut app, &dedans, Some(Reponse::Oui));
        app.retirer_les_copies_scellees();
        assert!(dedans.exists(), "pas avant d'être scellée");
        image_suivante(&mut app);
        app.retirer_les_copies_scellees();
        assert!(!dedans.exists(), "scellée, elle se retire");
        assert!(a_cote.exists(), "à côté, le document la désigne");
        assert_eq!(videos(&app).len(), 2, "les deux vidéos sont posées");
    }
}
