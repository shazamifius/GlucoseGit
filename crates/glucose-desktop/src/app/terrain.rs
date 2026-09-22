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
use std::num::NonZeroU32;

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
        // La vue se déplace, que la main la tienne encore ou qu'elle glisse sur son élan :
        // pour l'œil c'est le même mouvement, et c'est lui que la chronique juge. Sans l'élan
        // ici, chaque glissement au pavé tactile se comptait « repos » -- deux mille sept cents
        // images d'une session réelle, sous le nom de ce qui ne bouge pas.
        if self.is_panning || self.elan.en_cours() {
            return Geste::DeplacerLaVue;
        }
        if self.vol.en_cours() {
            return Geste::Animer;
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

    /// Ce qui suit la présentation : la latence vécue, le travail de fond, et la trace.
    ///
    /// Séparé du rendu parce que rien ici ne retarde l'image — elle est déjà à l'écran. Ce
    /// bloc occupe le temps qu'on aurait passé à attendre la suivante.
    pub(super) fn clore_l_image(
        &mut self,
        debut: std::time::Instant,
        (largeur, hauteur): (u32, u32),
    ) {
        // NAV-3 : l'age du plus ancien geste que cette image montre enfin. C'est **la**
        // grandeur qui dit « fluide », et aucune duree d'image ne l'explique.
        if let Some(l) = self.chronique.navigation.image_presentee() {
            crate::perf::compteur("nav_latence_us", l.as_micros() as f64);
        }

        // CASCADE-1 : le travail de fond a deja pris l'attente du tempo. Il ne prend ici que
        // ce que le plancher de la charte laisse encore quand l'image etait en retard -- sans
        // quoi la machine reste enfermee dans son regime degrade : images cheres faute de
        // vignettes, vignettes jamais construites faute de temps.
        if crate::perf::valeur_du_compteur("tempo_attente_us").unwrap_or(0.0) <= 0.0 {
            let faites = self
                .renderer
                .magasin
                .avancer_les_vignettes(self.cadence.tranche_de_fond(debut.elapsed()));
            crate::perf::compteur(
                "vign_atelier",
                f64::from(u32::try_from(faites).unwrap_or(u32::MAX)),
            );
            crate::perf::stage("atelier");
        }
        crate::perf::compteur(
            "vign_attente",
            self.renderer.magasin.vignettes.en_chantier() as f64,
        );

        // **Ce que l'image a coute, et non ce qu'elle a attendu.** Depuis le tempo, une image
        // attend jusqu'a l'heure de partir ; cette attente est du temps libre, pas du travail.
        // La compter faisait lire « 98 % des images au-dessus de dix millisecondes » sur une
        // session ou elles en coutaient cinq et en attendaient seize -- et faisait rapetisser
        // la scene pour un budget qu'elle tenait.
        let attente = crate::perf::valeur_du_compteur("tempo_attente_us")
            .map_or(std::time::Duration::ZERO, |us| {
                std::time::Duration::from_micros(us.max(0.0) as u64)
            });
        let ecoule = debut.elapsed().saturating_sub(attente);
        // **La resolution ne s'observe que sur la voie qu'elle commande.** Sur la voie
        // graphique, la scene ne se reduit jamais -- la carte filtre sans rien payer, donc
        // abimer l'image n'achete rien. Laisser le modele observer quand meme faisait trois
        // choses fausses a la fois : il lisait des images qui portent les gels du pilote
        // (`present` a 300 ms) et concluait qu'il fallait reduire ; il faisait allouer un
        // tampon reduit que personne ne lisait, et ce tampon neuf forcait un rendu complet ;
        // et la chronique annoncait « 26 % des images se rendent plus petites » sur une voie
        // qui n'en rend aucune. Un mecanisme qui s'adapte a un cout qu'il ne commande plus
        // est la forme exacte du cercle vicieux, et la chronique en avait deja quatre.
        if self.la_carte_pose_les_photos() {
            self.resolution = crate::resolution::Resolution::nette();
        } else {
            self.accorder_la_finesse(ecoule);
        }
        crate::perf::compteur("img_reduction", f64::from(self.resolution.facteur()));
        crate::perf::frame_end();
        // La chronique lit les postes APRES `frame_end` : celui-ci ne les efface pas, il se
        // contente de les afficher quand la trace est demandee.
        self.enregistrer_l_image(
            ecoule.as_micros().min(u128::from(u32::MAX)) as u32,
            (largeur, hauteur),
        );
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

        let present_us = self.relever_les_postes(&mut vu);
        self.arbitrer_la_carte(present_us);

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
        vu.tuiles_peintes = lire("tuiles_peintes").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.tuiles_reprises = lire("tuiles_reprises").clamp(0.0, f64::from(u16::MAX)) as u16;
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
        vu.textures_faites = lire("textures_faites").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.textures_kpx = lire("textures_kpx").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.surfaces_refaites = lire("surfaces_refaites").clamp(0.0, f64::from(u16::MAX)) as u16;
        // **Les images que la surface a refusees** : elles ne sont dans aucune autre mesure,
        // et une rafale de refus est exactement ce qui fige un canevas sans que rien ne le
        // dise.
        self.chronique
            .noter_un_refus(lire("images_cachees") as u64, lire("images_perdues") as u64);
        vu.dock_rendus = lire("dock_rendus").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.bande_refaite = lire("bande_refaite").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.cartes_entieres = lire("cartes_entieres").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.textures_reportees = lire("textures_reportees").clamp(0.0, f64::from(u16::MAX)) as u16;
        // Sans la passe des images, la region n'est pas declaree : une image qui n'a rien
        // redessine du tout vaut zero, ce qui est exact.
        vu.region_px = lire("img_region") as u32;
        vu.reveils = lire("reveil_masque").clamp(0.0, f64::from(u16::MAX)) as u16;
        // RYTHME-1 : ce que l'ecran a MONTRE. Ces trois-la ne viennent pas des compteurs mais
        // de la mesure prise a la presentation elle-meme : un compteur f64 perdrait le type,
        // et c'est precisement le genre de perte qui a fait lire des zeros pour des mesures.
        vu.tempo_balayages = lire("tempo_balayages").clamp(0.0, f64::from(u16::MAX)) as u16;
        vu.tempo_attente_us = lire("tempo_attente_us").clamp(0.0, f64::from(u32::MAX)) as u32;
        vu.intervalle_us = self.rythme_de_l_image.intervalle_us;
        vu.saut_px = self.rythme_de_l_image.saut_px;
        vu.fidelite_millieme = self.rythme_de_l_image.fidelite_millieme;
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

impl GlucoseApp {
    /// **Range les postes de cette image dans son enregistrement**, et rend ce que `present` a
    /// coute.
    ///
    /// Extraite d'`enregistrer_l_image`, qui lisait les postes, les compteurs, le rythme et la
    /// navigation dans la meme fonction : le cliquet des quatre-vingts lignes a raison, et
    /// `present` est le seul poste dont un autre mecanisme -- l'arbitre -- a besoin.
    fn relever_les_postes(&mut self, vu: &mut Instantane) -> u32 {
        let mut present_us = 0u32;
        for (nom, ms) in crate::perf::postes() {
            let us = (ms * 1000.0).clamp(0.0, f64::from(u32::MAX)) as u32;
            if nom == "present" {
                present_us = us;
            }
            let Some(i) = self.chronique.poste(nom) else {
                continue;
            };
            vu.postes_us[i] = us;
        }
        present_us
    }

    /// **Note ce que la presentation vient de couter, et rouvre la carte si l'arbitre le
    /// demande** (ARBITRE-1).
    ///
    /// La decision se prend ici, la reouverture a la fin de l'image : detruire la chaine
    /// pendant qu'une image est detenue arrache le sol sous ses pieds, et c'est exactement le
    /// plantage que la fiche 17 § 2.3 raconte.
    pub(super) fn arbitrer_la_carte(&mut self, present_us: u32) {
        use crate::present::arbitre::Verdict;
        let Some(arbitre) = self.arbitre.as_mut() else {
            return;
        };
        match arbitre.observer(present_us) {
            Verdict::Continuer => {}
            Verdict::Essayer(p) | Verdict::Revenir(p) => self.carte_a_rouvrir = Some(p),
        }
    }

    /// **Rouvre la presentation sur la carte que l'arbitre a designee.**
    ///
    /// A appeler hors du rendu, quand aucune image n'est detenue. Un echec n'est pas une
    /// panne : on garde celle qui marche, et on le dit -- une carte qui refuse de s'ouvrir
    /// vaut mieux qu'une application qui se ferme.
    pub(super) fn rouvrir_la_carte_si_demande(&mut self) {
        let Some(voulue) = self.carte_a_rouvrir.take() else {
            return;
        };
        let Some(window) = self.window.clone() else {
            return;
        };
        let taille = window.inner_size();
        let (Some(w), Some(h)) = (
            NonZeroU32::new(taille.width.max(1)),
            NonZeroU32::new(taille.height.max(1)),
        ) else {
            return;
        };
        // L'ancienne part AVANT que la nouvelle ne s'ouvre : deux chaines sur la meme fenetre
        // ne coexistent pas, et la surface appartient a celle qui l'a creee.
        let depart = std::time::Instant::now();
        self.presenter = None;
        let lachee = depart.elapsed();
        let carte = crate::present::gpu::succession::pour_wgpu(voulue);
        match crate::present::GpuPresenter::sur_la_carte(window, w, h, carte) {
            Ok(neuf) => {
                println!(
                    "[Glucose] arbitre : la presentation passe sur la carte {} -- {}                      (ancienne lachee en {:.0} ms, nouvelle ouverte en {:.0} ms)",
                    voulue.nom(),
                    neuf.adaptateur(),
                    lachee.as_secs_f64() * 1000.0,
                    (depart.elapsed() - lachee).as_secs_f64() * 1000.0
                );
                let neuf: Box<dyn crate::present::Presenter> = Box::new(neuf);
                // La chronique doit savoir comment les images se succedent sur CETTE carte :
                // l'Arc n'offre pas `mailbox`, la RTX si, et les memes durees ne veulent pas
                // dire la meme chose selon que la presentation attendait un balayage.
                self.chronique
                    .rythme
                    .observer_la_machine(self.cadence.periode(), neuf.rythme());
                self.presenter = Some(neuf);
            }
            Err(e) => {
                eprintln!(
                    "[Glucose] arbitre : la carte {} ne s'ouvre pas ({e})",
                    voulue.nom()
                );
                // On ne reste pas sans presentation : on rouvre celle d'avant.
                if let Some(window) = self.window.clone() {
                    if let Ok(reprise) = crate::present::GpuPresenter::new(window, w, h) {
                        self.presenter = Some(Box::new(reprise));
                    }
                }
            }
        }
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::present::Presenter;
    use crate::resolution::{Mesure, Resolution};
    use std::time::Duration;

    /// Une presentation qui ne presente rien, mais qui DIT poser les photos : c'est la seule
    /// chose que cette decision lit.
    struct Carte;

    impl Presenter for Carte {
        fn resize(
            &mut self,
            _: std::num::NonZeroU32,
            _: std::num::NonZeroU32,
        ) -> crate::error::DesktopResult<()> {
            Ok(())
        }
        fn present(
            &mut self,
            _: &tiny_skia::Pixmap,
        ) -> crate::error::DesktopResult<crate::present::Issue> {
            Ok(crate::present::Issue::Presentee)
        }
        fn pose_les_photos(&self) -> bool {
            true
        }
        fn nom(&self) -> &'static str {
            "carte factice"
        }
        fn rythme(&self) -> &'static str {
            "aucun"
        }
    }

    /// Une resolution deja reduite, comme le modele la laisse apres un pic.
    fn reduite() -> Resolution {
        let mut r = Resolution::nette();
        // Une scene de 30 ms pour un budget de 8 : la loi demande la racine de leur
        // rapport, soit le palier 4 -- en mouvement, et avec un oeil qui tolere jusqu'a 4.
        r.observer(
            Mesure {
                image: Duration::from_millis(32),
                scene: Duration::from_millis(30),
            },
            Duration::from_millis(8),
            true,
            4,
        );
        assert!(
            r.reduite(),
            "le montage doit partir d'une resolution reduite"
        );
        r
    }

    /// **Sur la voie graphique, la resolution reste nette quoi que l'image ait coute.**
    ///
    /// # Le cercle que ce test ferme
    ///
    /// Le modele de resolution observait toutes les images, y compris celles de la voie
    /// graphique -- qui ne reduit jamais. Il y lisait les gels du pilote (`present` a
    /// 300 ms), concluait qu'il fallait reduire, faisait allouer un tampon que personne ne
    /// lisait, et ce tampon neuf forcait un rendu complet. La chronique annoncait alors
    /// « 26 % des images se rendent plus petites » sur une voie qui n'en rend aucune.
    ///
    /// Le contraste avec la voie processeur est la preuve : le meme montage, la meme image
    /// lente, et la resolution y reste reduite -- parce que la, elle commande vraiment.
    #[test]
    fn test_la_resolution_ne_s_observe_que_sur_la_voie_qu_elle_commande() {
        let lente = std::time::Instant::now() - Duration::from_millis(300);

        let mut par_la_carte = GlucoseApp::new();
        par_la_carte.presenter = Some(Box::new(Carte));
        par_la_carte.resolution = reduite();
        par_la_carte.clore_l_image(lente, (800, 600));
        assert_eq!(
            par_la_carte.resolution.facteur(),
            1,
            "la carte pose les photos : rien ne se reduit, quoi que l'image ait coute"
        );

        let mut par_le_processeur = GlucoseApp::new();
        par_le_processeur.presenter = None;
        par_le_processeur.resolution = reduite();
        par_le_processeur.clore_l_image(lente, (800, 600));
        assert!(
            par_le_processeur.resolution.reduite(),
            "sur la voie processeur, la meme image lente laisse la resolution reduite : la \
             decision distingue bien les deux voies"
        );
    }
}
