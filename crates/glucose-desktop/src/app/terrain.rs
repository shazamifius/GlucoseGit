//! Ce que l'application enregistre de son propre usage (CHRONIQUE-1).
//!
//! # Le geste se deduit, il ne se declare pas
//!
//! Un geste qui devrait penser a s'annoncer finirait par oublier -- et c'est exactement le
//! genre d'oubli qui a coute une journee de recherche cette semaine. L'etat de l'application
//! dit deja tout : une session de redimensionnement est ouverte, un glisser est en cours, la
//! vue a change d'echelle. Il suffit de le lire.
//!
//! Le zoom est le seul qui n'ait pas d'etat persistant -- c'est un evenement de molette, pas
//! une session. Il se lit donc sur le document lui-meme : **si l'echelle de la vue a change
//! depuis l'image precedente, on zoome**. C'est exact, et aucune duree d'attente arbitraire
//! n'a besoin d'etre choisie.
//!
//! # Ce qui est enregistre, et ce qui ne peut pas l'etre
//!
//! Des durees et des nombres. Le type `Instantane` ne porte que des entiers : aucun chemin,
//! aucun nom de fichier, aucun texte de carte ne peut traverser ce chemin, quelle que soit la
//! bonne ou mauvaise volonte d'un appelant futur.

use super::GlucoseApp;
use crate::chronique::{Geste, Instantane};

impl GlucoseApp {
    /// Ce que l'utilisateur est en train de faire.
    ///
    /// L'ordre compte : un zoom pendant un glisser **est** un zoom, parce que c'est le zoom
    /// qui refait toute l'image alors que le glisser n'en refait qu'un morceau.
    pub(super) fn geste_courant(&self, echelle_precedente: f64) -> Geste {
        let vp = self.store.viewport();
        if vp.scale != echelle_precedente {
            return Geste::Zoomer;
        }
        if self.is_panning {
            return Geste::DeplacerLaVue;
        }
        if self.resize_session.is_some() {
            return Geste::Redimensionner;
        }
        if self.is_dragging_item {
            return Geste::GlisserUnNoeud;
        }
        if self.draw_session.is_some() {
            return Geste::Dessiner;
        }
        if self.selection_box.is_some() {
            return Geste::Selectionner;
        }
        if self.editing_session.is_some() {
            return Geste::EditerDuTexte;
        }
        if self.renderer.magasin.en_travail() > 0 {
            return Geste::Decoder;
        }
        Geste::Repos
    }

    /// Enregistre l'image qui vient de se dessiner.
    ///
    /// Les postes viennent de la trace, qui les mesure desormais toujours : la variable
    /// d'environnement ne decide plus que de l'affichage.
    pub(super) fn enregistrer_l_image(&mut self, duree_us: u32, fenetre: (u32, u32)) {
        let echelle = self
            .store
            .active_board()
            .map(|b| b.viewport.scale)
            .unwrap_or(1.0);
        let geste = self.geste_courant(self.echelle_precedente);
        self.echelle_precedente = echelle;

        let mut vu = Instantane {
            duree_us,
            geste: Geste::TOUS.iter().position(|g| *g == geste).unwrap_or(0) as u8,
            fenetre_px: fenetre.0.saturating_mul(fenetre.1),
            ..Default::default()
        };

        for (nom, ms) in crate::perf::postes() {
            let Some(i) = self.chronique.poste(nom) else {
                continue;
            };
            vu.postes_us[i] = (ms * 1000.0).clamp(0.0, f64::from(u32::MAX)) as u32;
        }

        let lire = |nom: &str| crate::perf::valeur_du_compteur(nom).unwrap_or(0.0);
        vu.photos = lire("img_n") as u32;
        vu.images_mo = lire("img_mo") as u32;
        vu.en_decodage = lire("img_attente").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.vignettes = lire("vign_atelier").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.vignettes_en_attente = lire("vign_attente").clamp(0.0, f64::from(u16::MAX)) as u16;
        // Un compteur declare mais jamais lu ici vaut ZERO dans la trace, et un zero se lit
        // comme une mesure. Quatre l'ont ete pendant une session entiere, et tout un
        // raisonnement s'est bati dessus : « aucune photo ne passe par une vignette » ne
        // mesurait rien d'autre que l'absence de cette ligne.
        vu.par_vignette = lire("img_par_vignette").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.vignettes_perimees = lire("vign_perimees").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.vignettes_pretes = lire("vign_pretes").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.vignettes_orphelines = lire("vign_orphelines").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.vignettes_recreees = lire("vign_recreees").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.vignettes_abandonnees = lire("vign_abandonnes").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.prevu_us = lire("cout_prevu_us") as u32;
        // Le modele prevoit le REPORT : c'est a lui que sa prevision se compare, et non a la
        // duree entiere, qui porte aussi la presentation et l'interface.
        vu.report_us = crate::perf::postes()
            .iter()
            .find(|(nom, _)| *nom == "report")
            .map(|(_, ms)| (ms * 1000.0).clamp(0.0, f64::from(u32::MAX)) as u32)
            .unwrap_or(0);
        vu.pixelise = lire("img_pixelise") as u16;
        vu.reduction = lire("img_reduction").clamp(1.0, f64::from(u16::MAX)) as u16;
        vu.blit_mo = lire("blit_mo").clamp(0.0, f64::from(u16::MAX)) as u16;
        // Sans la passe des images, la region n'est pas declaree : une image qui n'a rien
        // redessine du tout vaut zero, ce qui est exact.
        vu.region_px = lire("img_region") as u32;
        vu.reveils = lire("reveil_masque").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.saut_pct = lire("nav_saut_pct").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.surcouverture = (lire("img_ecrans") * 100.0) as u32;
        vu.noeuds = self.renderer.spatial_hash.len() as u32;

        self.chronique.enregistrer(vu);
    }

    /// Ecrit le rapport de la session a cote du journal de l'application.
    ///
    /// Rend le chemin, pour que l'appelant puisse le dire a l'utilisateur -- un rapport qu'on
    /// ne sait pas retrouver ne sert a personne.
    pub fn ecrire_la_chronique(&self) -> std::io::Result<std::path::PathBuf> {
        let chemin = Self::chemin_de_la_chronique();
        if let Some(dossier) = chemin.parent() {
            std::fs::create_dir_all(dossier)?;
        }
        std::fs::write(&chemin, self.chronique.rapport())?;
        Ok(chemin)
    }

    /// Le fichier ou la chronique s'ecrit.
    ///
    /// Toujours le meme, et dit au demarrage : un rapport qu'on ne sait pas retrouver ne sert
    /// a personne, et le chercher apres coup dans un dossier temporaire est decourageant.
    pub fn chemin_de_la_chronique() -> std::path::PathBuf {
        std::env::temp_dir()
            .join("glucose-chronique")
            .join("derniere-session.txt")
    }

    /// Sauve la chronique si elle a du neuf a dire.
    ///
    /// # Pourquoi sauver en cours de route, et pas seulement a la fermeture
    ///
    /// La premiere version n'ecrivait qu'a la croix. L'utilisateur a lance, utilise, ferme --
    /// et rien n'a ete ecrit. **Une mesure qui ne survit qu'a une sortie parfaite ne mesure
    /// rien**, parce que les sessions qui interessent sont justement celles qui finissent mal.
    ///
    /// Le moment de sauver n'est pas une horloge : c'est **l'apparition d'une image plus lente
    /// que toutes les precedentes**. C'est exactement l'instant ou le fichier a quelque chose
    /// de plus a dire, et rien n'a eu a etre choisi.
    ///
    /// L'ecriture a lieu hors du rendu : une entree-sortie sur le fil qui tient la cadence
    /// serait precisement le defaut qu'on cherche a mesurer.
    pub(super) fn sauver_la_chronique_si_besoin(&mut self) {
        if !self.chronique.du_neuf() {
            return;
        }
        let _ = self.ecrire_la_chronique();
    }

    /// Ecrit la chronique et la dit, au moment de fermer.
    ///
    /// Sur la console **et** dans un fichier : la console sert a qui lance depuis un terminal
    /// et veut voir tout de suite ; le fichier sert a qui lance l'application normalement, et
    /// pourra le retrouver ensuite.
    pub(super) fn clore_la_chronique(&self) {
        // Le fichier s'ecrit TOUJOURS, meme pour une poignee d'images : c'est la trace, et
        // elle ne coute rien. Seul l'affichage sur la console attend d'avoir quelque chose a
        // dire, pour ne pas noyer un demarrage rate sous un tableau vide.
        let ecrit = self.ecrire_la_chronique();
        if self.chronique.rendues() >= 30 {
            println!("\n{}", self.chronique.rapport());
        }
        match ecrit {
            Ok(chemin) => println!(
                "[Glucose] chronique de {} images ecrite dans {}",
                self.chronique.rendues(),
                chemin.display()
            ),
            Err(e) => eprintln!("[Glucose] chronique non ecrite : {e}"),
        }
    }
}
