//! **Remettre l'image à l'écran**, et noter ce que l'œil en a reçu.
//!
//! # Pourquoi ce fichier existe à part
//!
//! [`super::peinture`] décide de ce qu'il faut repeindre et le peint ; celui-ci ne peint rien.
//! Il attend l'heure que le tempo fixe, confie l'image à la présentation, et note ce qu'elle
//! en a fait — trois choses qui n'ont de sens qu'une fois le dessin terminé. Les garder
//! ensemble faisait passer `peinture.rs` les six cents lignes que la fiche 05 admet.
//!
//! # Ce que la présentation répond, et pourquoi ce n'est pas un détail
//!
//! Elle rendait `Ok(())` aussi bien quand l'image partait à l'écran que quand la surface la
//! refusait. L'application effaçait alors la salissure, croyait l'image faite, et s'endormait
//! — **sur un canevas figé, pendant que l'interface continuait de répondre**. C'est
//! [`crate::present::Issue`] qui sépare les deux, et c'est ici qu'on en tient compte.

use super::GlucoseApp;

impl GlucoseApp {
    /// l'arbre de possibilites preparera les tuiles de la trajectoire (fiche 18, etape 3).
    pub(super) fn attendre_l_heure_de_soumettre(&mut self) {
        // Le tempo ne regle que ce qui BOUGE : c'est la seule situation ou un intervalle
        // irregulier se voit. Ailleurs -- un decodage, un curseur qui clignote -- attendre
        // ferait tourner le processeur a vide pour une regularite que personne ne regarde.
        if !(self.elan.en_cours() || self.vol.en_cours()) {
            self.tempo.oublier();
            crate::perf::compteur("tempo_balayages", 0.0);
            crate::perf::compteur("tempo_attente_us", 0.0);
            return;
        }
        let maintenant = std::time::Instant::now();
        let attente = self.tempo.attente_avant_de_soumettre(maintenant);
        crate::perf::compteur("tempo_balayages", f64::from(self.tempo.balayages()));
        crate::perf::compteur("tempo_attente_us", attente.as_micros() as f64);
        if attente.is_zero() {
            return;
        }
        let cible = maintenant + attente;
        // Le travail de fond d'abord, dans ce que l'attente laisse, marge deduite.
        let budget = attente.saturating_sub(crate::cadence::MARGE);
        let faites = self.renderer.magasin.avancer_les_vignettes(budget);
        crate::perf::compteur(
            "vign_atelier",
            f64::from(u32::try_from(faites).unwrap_or(u32::MAX)),
        );
        crate::perf::stage("atelier");
        // **Dormir, et ne tourner a vide que la marge.** La premiere version tournait a vide
        // jusqu'a la cible -- seize millisecondes par image sur le terrain. Un coeur a cent
        // pour cent chauffe, la frequence baisse, et tout le rendu ralentit trois fois :
        // `clear` passait de une a quatre millisecondes, `blit` de une a six. Le tempo ratait
        // alors ses cibles, montait, attendait plus, chauffait plus. Un cercle, et la charte
        // le nomme : le bridage thermique est le regime normal d'un portable.
        //
        // `thread::sleep` est a haute resolution sur Windows 10 depuis Rust 1.77 -- un
        // minuteur de quelques dizaines de microsecondes, sans toucher a la periode du
        // systeme. Il dort jusqu'a la marge ; la marge seule se passe a vide, pour la
        // precision.
        let reveil = cible - crate::cadence::MARGE;
        let maintenant = std::time::Instant::now();
        if reveil > maintenant {
            std::thread::sleep(reveil - maintenant);
        }
        while std::time::Instant::now() < cible {
            std::hint::spin_loop();
        }
        crate::perf::stage("tempo");
    }

    /// Met l'image a l'ecran, et note l'instant ou elle y arrive (RYTHME-1).
    ///
    /// # Pourquoi la mesure se prend ICI et nulle part ailleurs
    ///
    /// C'est le seul instant de la boucle qui corresponde a quelque chose que l'oeil recoive.
    /// Le debut du rendu, sa fin, le reveil de la boucle sont des faits internes : deux
    /// d'entre eux peuvent varier du simple au decuple sans que l'ecran change de rythme, et
    /// inversement. Toute la chronique a mesure ces faits internes, et c'est pourquoi elle ne
    /// **Ce que la surface a fait de l'image qu'on lui a donnée**, noté et, s'il le faut,
    /// redemandé.
    ///
    /// # Le défaut que cette fonction ferme
    ///
    /// La présentation rendait `Ok(())` dans deux cas qui n'ont rien à voir : l'image est
    /// partie à l'écran, ou la surface l'a refusée et elle a été jetée. L'application ne
    /// pouvait pas les distinguer : elle effaçait la salissure, considérait l'image comme
    /// faite et s'endormait — **sur un canevas figé, pendant que l'interface continuait de
    /// répondre**.
    ///
    /// C'est exactement ce que l'utilisateur a décrit — *« le canva ça a freeze, mais Ctrl+O,
    /// Ctrl+S et tout ça fonctionnent parfaitement »* — et ce que sa chronique montrait sans
    /// pouvoir le nommer : **treize secondes « à ne pas dessiner »**, parce que le rythme ne
    /// mesure un intervalle qu'entre deux images PRÉSENTÉES, et qu'une rafale de refus n'en
    /// laisse qu'un seul, très long.
    ///
    /// Une fenêtre **cachée** ne se redemande pas : dessiner pour personne est du travail
    /// perdu, et c'est le seul cas où dormir est juste.
    fn noter_ce_que_la_surface_a_fait(
        &mut self,
        issue: crate::error::DesktopResult<crate::present::Issue>,
    ) {
        use crate::present::Issue;
        match issue {
            Ok(Issue::Presentee) => {}
            Ok(Issue::Cachee) => crate::perf::compteur("images_cachees", 1.0),
            Ok(Issue::Perdue) => {
                crate::perf::compteur("images_perdues", 1.0);
                self.mark_dirty();
            }
            Err(e) => {
                eprintln!("[GlucoseDesktop] presentation du framebuffer impossible : {e}");
            }
        }
    }

    /// pouvait pas voir le tressaut dont l'utilisateur parle depuis des semaines.
    pub(super) fn presenter_et_noter_le_rythme(&mut self, debut_du_rendu: std::time::Instant) {
        if let (Some(pixmap), Some(presenter)) = (&self.pixmap, &mut self.presenter) {
            // On presente meme quand rien n'a ete redessine : la demande peut venir du
            // systeme -- une fenetre recouverte puis degagee -- et non de nous.
            let issue = match self
                .tampon_dessus
                .as_ref()
                .filter(|_| presenter.pose_les_photos())
            {
                // La voie graphique : le dessous, les photos, le dessus.
                Some(dessus) => {
                    let (renderer, confie) = (&self.renderer, &self.confie);
                    // **Ce qui sépare cette image du plancher de la charte**, et c'est tout
                    // le budget que le rendu des textures manquantes a le droit de prendre.
                    //
                    // Il ne se choisit pas : l'image a déjà coûté ce qu'elle a coûté, et la
                    // charte dit qu'elle et son travail de fond tiennent ensemble dans dix
                    // millisecondes. Ce qui ne rentre pas attend l'image suivante, en gardant
                    // son ancien palier posé (CASCADE-2). Sur une image déjà en retard, le
                    // budget est nul : on ne creuse pas un trou en le remplissant.
                    let budget = self.cadence.tranche_de_fond(debut_du_rendu.elapsed());
                    // Ce que la carte ne connait pas encore : une photo vient du magasin,
                    // un composant -- carte de texte, photo en chemin -- se rend a la
                    // demande, une fois, puis plus jamais tant que son empreinte ne change
                    // pas (COMPOSANT-1).
                    presenter.presenter_en_couches(
                        pixmap,
                        (confie, budget),
                        &|cle| match confie.composant(cle) {
                            Some(composant) => composant.rendre(renderer.kit()),
                            None => renderer
                                .magasin
                                .cache
                                .get(cle)
                                .map(|e| e.pyramide.native().clone()),
                        },
                        dessus,
                    )
                }
                None => presenter.present(pixmap),
            };
            self.noter_ce_que_la_surface_a_fait(issue);
        }
        let maintenant = std::time::Instant::now();
        let (pas, vitesse) = self.pas_et_vitesse;
        // Un geste arrive pendant un sommeil demande aussi une image : elle est attendue au
        // sens ou la precedente n'avait rien demande, mais la main, si. Ce qui compte est de ne
        // pas mesurer un SOMMEIL comme un gel ; un geste qui reveille est bien un intervalle
        // que l'oeil vit -- entre la derniere image et celle que sa main vient de demander.
        let attendue = self.image_attendue || self.chronique.navigation.en_attente();
        // **L'entracte se ferme ici, et pas au debut du rendu** (ENTRACTE-1). Les deux
        // mesures decoupent le meme intervalle par ses deux bouts : leur donner le meme
        // instant et le meme verdict est ce qui les empeche de diverger. Le rythme dit
        // COMBIEN l'application a passe a ne pas dessiner ; l'entracte dit OU il est alle.
        self.chronique.entracte.fermer(debut_du_rendu, attendue);
        self.rythme_de_l_image =
            self.chronique
                .rythme
                .presentee(maintenant, debut_du_rendu, pas, vitesse, attendue);
        self.chronique.entracte.ouvrir(maintenant);
        // **L'horloge de la trajectoire avance ICI**, au meme instant que la mesure : les deux
        // parlent de la meme chose, et les separer les ferait diverger.
        self.horloge.presentee(maintenant, self.image_attendue);
    }
}
