//! Peindre une image : ce qu'on redessine, et ce qu'on se garde de redessiner (A.1).
//!
//! # Trois issues, et une seule decision
//!
//! Une image demandee ne veut pas dire une image a peindre. Selon ce que la salissure dit,
//! trois chemins s'ouvrent :
//!
//! * **rien** -- l'image precedente est encore exacte, on se contente de la representer ;
//! * **une region** -- seule une zone a change, et elle ne touche pas la chrome : on redessine
//!   cette zone seule, dans une image a sa taille, puis on la reporte ;
//! * **tout** -- le cas par defaut, celui d'hier, et celui de tout ce qu'on ne sait pas encore
//!   localiser.
//!
//! La regle de surete ne bouge jamais : **on ne reduit le travail que quand on peut le
//! prouver**. Redessiner tout coute ce que ca coutait ; oublier une zone laisse un morceau
//! d'image perime a l'ecran, et c'est le pire defaut possible -- il se voit, et on ne sait pas
//! d'ou il vient.
//!
//! # Pourquoi la chrome force le rendu complet
//!
//! La scene suit la vue : la decaler de `-x0` deplace l'origine de l'ecran d'autant, et aucune
//! passe n'a besoin de le savoir. La chrome, elle, se place sur la taille de la fenetre -- un
//! bandeau en haut, une minimap en bas. Elle ne sait pas se rendre decalee, donc une zone sale
//! qui la touche retombe sur le chemin complet.
//!
//! C'est aussi pourquoi les deux ont ete separees : elles ne se salissent pas dans le meme
//! repere -- la scene en coordonnees monde, la chrome en coordonnees ecran.

use super::GlucoseApp;
use crate::dock::{render_docks, DockCache, DockManager, DockPass};
use crate::params::{Pointer, SceneOverlay, ScreenFrame};
use crate::renderer::Renderer;
use crate::ui::UiState;
use glucose_core::store::Store;
use tiny_skia::Pixmap;

impl GlucoseApp {
    /// **La carte pose-t-elle les photos ?** -- la question dont dependent trois decisions de
    /// cette boucle, et qui merite un nom plutot que trois copies du meme predicat.
    pub(super) fn la_carte_pose_les_photos(&self) -> bool {
        self.presenter.as_ref().is_some_and(|p| p.pose_les_photos())
            && !self.renderer.carte.debordee
    }

    /// **Ce que la carte laisse aux photos, dit au rendu avant l'image** (ETAGES-3) ; et,
    /// quand elle ne tenait plus l'écran, le rejugement depuis la voie du processeur — c'est
    /// lui qui dira qu'elle peut reprendre.
    pub(super) fn preparer_la_carte(&mut self, fenetre: (u32, u32)) {
        self.renderer.carte.part_des_photos = self
            .presenter
            .as_ref()
            .and_then(|p| p.part_pour_les_photos());
        if self.renderer.carte.debordee {
            let header_h = self.ui.header_height();
            self.renderer
                .rejuger_la_carte(&self.store, fenetre, header_h);
        }
    }

    /// La region d'ecran a repeindre, ou `None` quand il faut tout refaire.
    ///
    /// Quatre raisons de tout refaire, et chacune est une impossibilite, pas une prudence :
    ///
    /// * la salissure ne sait pas se localiser -- le cas par defaut ;
    /// * la zone touche la **chrome**, qui se place sur la taille de la fenetre et ne sait
    ///   donc pas se rendre decalee ;
    /// * la scene se rend **plus petite** : une region se declare en coordonnees d'ecran
    ///   plein, et ces coordonnees ne designent plus rien dans le tampon reduit ;
    /// * **la carte pose les photos** -- et c'est la raison la moins evidente des quatre.
    ///
    /// # Le defaut que la derniere garde repare, et pourquoi il ne s'etait jamais vu
    ///
    /// Sur la voie graphique, le tampon principal n'est plus l'image : il est la **couche du
    /// dessous**, qui ne porte ni les photos ni la chrome. Y repeindre une region au cadrage
    /// `Tout` y aurait pose un carre d'image complete -- fond opaque, photos, annotations --
    /// au milieu d'une couche transparente, et la presentation l'aurait compose sous les vraies
    /// photos.
    ///
    /// Rien ne l'a montre parce que la salissure ne sait presque jamais se localiser : la
    /// chronique porte « 100 % des images redessinent » a chaque session. Le defaut attendait
    /// le jour ou l'etape 1 du plan 18 aboutirait -- c'est-a-dire le jour ou l'on croirait
    /// avoir gagne. C'est exactement la forme que prend « deplacer une fonctionnalite emporte
    /// ce qui l'entoure ».
    fn region_a_repeindre(
        &self,
        sale: crate::salissure::Salissure,
        vp: glucose_core::types::Viewport,
        fenetre: (u32, u32),
        header_h: f32,
    ) -> Option<crate::salissure::Region> {
        sale.region(&vp, fenetre, debord_des_passes(vp.scale))
            .filter(|r| !r.touche_le_haut(header_h))
            .filter(|_| !self.resolution.reduite())
            .filter(|_| !self.la_carte_pose_les_photos())
    }

    /// Attend l'instant que le tempo fixe pour cette image, en faisant avancer le travail de
    /// fond pendant ce temps.
    ///
    /// # Pourquoi l'attente est ici, apres le rendu, et non avant
    ///
    /// Attendre avant de rendre reduirait la latence -- l'image montrerait un etat plus
    /// recent -- mais demanderait de savoir combien le rendu va couter, et un rendu plus long
    /// que prevu raterait le balayage. Attendre apres ne rate jamais : l'image est la, on
    /// sait exactement combien de temps il reste. La regularite passe avant la latence, parce
    /// que c'est elle qui se voit.
    ///
    /// Le temps d'attente n'est pas perdu : l'atelier y avance (CASCADE-1), et c'est la que
    /// Ce que l'oeil et la main font en ce moment, pour le rendu.
    fn regard(&self) -> crate::renderer::Regard {
        crate::renderer::Regard {
            degradation_permise: self.perception.autorise_a_degrader(),
            en_mouvement: self.elan.en_cours() || self.vol.en_cours(),
        }
    }

    /// Redessine ce qui doit l'etre, et rien de plus. Ne presente pas.
    ///
    /// Le tampon est **sorti** de l'application le temps de la peinture : sans cela, peindre
    /// emprunterait `self` en ecriture par le tampon et en lecture par le document, et il
    /// faudrait passer chaque champ un par un pour convaincre le compilateur.
    /// Ce qu'il faut repeindre, ou `None` quand l'image d'avant suffisait.
    ///
    /// La decision vit a part de la peinture : un ordonnanceur qui peint aussi finit par ne
    /// plus laisser voir la decision -- et c'est elle qui dit, dans la trace, qu'une image a
    /// ete EVITEE. Un compteur de plus pour le meme fait n'aurait servi qu'a diverger.
    fn ce_quil_faut_repeindre(&mut self, tampon_neuf: bool) -> Option<crate::salissure::Salissure> {
        let sale = self.salissure.replace(crate::salissure::Salissure::Rien);
        // Un tampon neuf ne contient rien : il n'y a aucune image precedente a menager.
        let sale = if tampon_neuf {
            crate::salissure::Salissure::Tout
        } else {
            sale
        };
        (!sale.est_propre()).then_some(sale)
    }

    /// **Une image de l'application, sans fenêtre** : ce que `redraw` peint, par la voie
    /// processeur, sur un tampon de cette taille. Pour les épreuves qui doivent faire passer
    /// le temps comme l'application le fait — image après image.
    #[cfg(test)]
    pub(crate) fn une_image_sans_fenetre(&mut self, (largeur, hauteur): (u32, u32)) {
        if self.pixmap.is_none() {
            self.pixmap = Pixmap::new(largeur, hauteur);
        }
        self.mark_dirty();
        self.poursuivre_les_bordures();
        self.peindre_ce_qui_a_change((largeur, hauteur), false);
    }

    /// **La voie processeur vient d'écrire ce tampon en entier** : c'est l'image complète,
    /// et il est aussi la couche du dessous de la voie graphique. Si l'arbitre repasse sur la
    /// carte, la première image devra tout effacer — ses bandes d'avant ne disent plus rien.
    fn dessous_entierement_sali(&mut self, hauteur: u32) {
        self.confie.bandes_du_dessous = crate::present::bandes::Bandes::tout(hauteur);
    }

    /// Ou la main pointe, en pixels d'ecran.
    fn pointeur(&self) -> Pointer {
        Pointer {
            x: self.mouse_pos.0 as f32,
            y: self.mouse_pos.1 as f32,
        }
    }

    pub(super) fn peindre_ce_qui_a_change(&mut self, fenetre: (u32, u32), tampon_neuf: bool) {
        // La salissure est **consommee** : ce qui est redessine maintenant cesse d'etre sale,
        // et une nouvelle demande arrivee pendant le rendu appartient a l'image suivante.
        let Some(sale) = self.ce_quil_faut_repeindre(tampon_neuf) else {
            return;
        };

        let Some(mut pixmap) = self.pixmap.take() else {
            return;
        };
        let regard = self.regard();
        let mut reduit = self.tampon_reduit.take();
        let (header_h, vp) = (self.ui.header_height(), self.store.viewport());
        let eclairages = self.eclairages();
        let designees = self.cartes_designees();
        let overlay = SceneOverlay {
            guides: &self.active_guides,
            selection_box: self.selection_box,
            editing: self.editing_session.as_ref(),
            arrivages: &self.depot.en_chemin,
            eclairages: &eclairages,
            designees: &designees,
        };
        let pointer = self.pointeur();
        // Lu AVANT d'emprunter l'interface : un emprunt disjoint ne se prouve qu'a travers
        // des champs, jamais a travers une methode.
        let par_la_carte = self.la_carte_pose_les_photos();
        match self.region_a_repeindre(sale, vp, fenetre, header_h) {
            Some(r) => repeindre_la_region(
                &mut pixmap,
                r,
                &mut self.renderer,
                (&self.store, &self.ui),
                overlay,
                regard,
            ),
            None => {
                crate::perf::compteur("img_region", f64::from(fenetre.0) * f64::from(fenetre.1));
                let echelle = self.ui.scale_factor;
                let scene = self.resolution.scene_reduite(reduit.as_mut());
                let chrome = Chrome {
                    ui: &mut self.ui,
                    dock_manager: &self.dock_manager,
                    dock_cache: &self.dock_cache,
                    pointer,
                    echelle,
                };
                // **La voie graphique, quand la presentation sait poser les photos.**
                //
                // On ne reduit alors PAS : la carte filtre en bilineaire sans rien payer,
                // donc abimer l'image n'achete plus rien. C'est tout le but de l'etape 1.
                if par_la_carte {
                    let (tampon, confie) = peindre_par_la_carte(
                        (&mut pixmap, self.tampon_dessus.take()),
                        &mut self.renderer,
                        (&self.store, &self.confie),
                        chrome,
                        (overlay, regard),
                    );
                    self.tampon_dessus = tampon;
                    self.confie = confie;
                } else {
                    peindre_tout(
                        &mut pixmap,
                        scene,
                        &mut self.renderer,
                        &self.store,
                        chrome,
                        overlay,
                        regard,
                    );
                }
            }
        }
        if !par_la_carte {
            self.dessous_entierement_sali(pixmap.height());
        }
        self.pixmap = Some(pixmap);
        self.tampon_reduit = reduit;
    }
}

/// Ce qui ne suit pas la vue : l'interface et ses panneaux, plus ce qu'il faut pour les poser.
///
/// Regroupees parce qu'elles voyagent toujours ensemble, et qu'une fonction de peinture qui
/// les recevrait une par une aurait dix arguments dont l'ordre serait la seule protection.
struct Chrome<'a> {
    ui: &'a mut UiState,
    dock_manager: &'a DockManager,
    dock_cache: &'a DockCache,
    pointer: Pointer,
    echelle: f32,
}

/// Ce qu'une passe peut dessiner **au-delà** du rectangle d'un nœud, en pixels.
///
/// C'est la portée du flou des halos, seule passe qui déborde vraiment : `bench_zone` a
/// mesuré que l'écart entre un rendu par région et un rendu complet s'éteint entre seize
/// et quarante-huit pixels à l'échelle 1, ce qui est exactement cette valeur.
///
/// Elle ne se choisit donc pas : elle se lit sur le modèle de halo et suit le zoom, comme
/// le halo lui-même. Le double couvre le flou de part et d'autre de la bordure.
fn debord_des_passes(echelle: f64) -> f32 {
    (crate::renderer::halo::HALO_SPREAD as f64 * echelle * 2.0) as f32
}

/// Redessine **une région** de la scène, et la reporte dans le tampon (A.1).
///
/// La scène se rend dans une image à la taille de la région, la vue décalée d'autant, puis
/// le résultat se recopie en place. `bench_zone` mesure que ces pixels sont identiques au
/// bit près à ceux d'un rendu complet.
///
/// La chrome n'y figure pas, et c'est l'appelant qui s'en assure : elle se place en
/// coordonnées écran et ne sait pas se rendre décalée. Une région qui la toucherait
/// retombe sur le rendu complet.
fn repeindre_la_region(
    pixmap: &mut Pixmap,
    region: crate::salissure::Region,
    renderer: &mut Renderer,
    (store, ui): (&Store, &UiState),
    overlay: SceneOverlay<'_>,
    regard: crate::renderer::Regard,
) {
    // La region se mesure ici, vide ou non : c'est elle qui dit a la chronique combien
    // l'image a evite de repeindre.
    crate::perf::compteur("img_region", region.aire() as f64);
    if region.est_vide() {
        return;
    }
    let Some(mut morceau) = Pixmap::new(region.largeur, region.hauteur) else {
        return;
    };
    let origine = (region.x as f32, region.y as f32);
    let cadrage = crate::renderer::Cadrage::region(origine).sous_le_regard(regard);
    renderer.rendre_la_region(
        &mut morceau.as_mut(),
        store,
        ui,
        overlay,
        ui.header_height() - origine.1,
        cadrage,
    );
    // `Remplacer` et non `Composer` : on ÉCRASE les pixels périmés. Composer redoublerait
    // tout ce qui n'est pas opaque, et l'erreur serait invisible sur un fond sombre.
    crate::composition::poser(
        &mut pixmap.as_mut(),
        &morceau,
        (region.x as f32, region.y as f32),
        glucose_core::report::Melange::Remplacer,
    );
    crate::perf::stage("region");
}

/// **Peint les deux couches du processeur**, celles qui entourent les photos, et dit ce que
/// la carte doit poser entre les deux -- le fond, les lueurs, les photos.
///
/// Le dessus part transparent : tout ce qui n'y est pas dessine laisse voir les photos, et
/// c'est ce qui rend la composition juste sans qu'aucune region ne soit calculee.
///
/// Une fonction libre et non une methode, pour la meme raison que `peindre_tout` : `overlay`
/// emprunte deja l'application, et un `&mut self` par-dessus ne compilerait pas.
fn peindre_par_la_carte(
    (pixmap, tampon): (&mut Pixmap, Option<Pixmap>),
    renderer: &mut Renderer,
    (store, precedente): (&Store, &crate::renderer::Confie),
    chrome: Chrome<'_>,
    (overlay, regard): (SceneOverlay<'_>, crate::renderer::Regard),
) -> (Option<Pixmap>, crate::renderer::Confie) {
    let (largeur, hauteur) = (pixmap.width(), pixmap.height());
    // Le tampon du dessus suit la fenetre : il se refait quand elle change de taille, et
    // jamais autrement.
    let mut tampon = tampon.filter(|t| t.width() == largeur && t.height() == hauteur);
    if tampon.is_none() {
        tampon = Pixmap::new(largeur, hauteur);
    }
    let Some(dessus) = tampon.as_mut() else {
        return (None, crate::renderer::Confie::default());
    };
    let Chrome {
        ui,
        dock_manager,
        dock_cache,
        pointer,
        echelle,
    } = chrome;
    // **On n'efface que ce que l'image précédente avait écrit.**
    //
    // Remplir seize mébioctets de transparent coûtait 2,05 ms par image sur le terrain, pour
    // une couche qui n'en porte qu'un huitième. Les bandes de l'image d'avant suffisent :
    // partout ailleurs, le tampon est transparent depuis qu'il existe et le rester.
    //
    // La première image, elle, hérite d'un tampon dont on ne sait rien -- il vient d'être
    // alloué, ou la fenêtre a changé de taille : ses bandes d'avant valent alors toute la
    // hauteur, et c'est l'appelant qui le sait.
    precedente.bandes_du_dessus.effacer(dessus);
    // **Le dessous de même : seulement là où il avait écrit** (BANDE-2).
    //
    // Ce qui y reste -- titres, poignées et réglettes des membranes, dossiers -- se compose
    // PAR-DESSUS ce que la carte a peint : sans effacement, l'image d'avant resterait. Mais
    // il n'occupe que quelques bandes, et un document sans membrane ni dossier n'en a aucune :
    // effacer l'écran entier écrivait seize mébioctets de zéros sur des zéros.
    precedente.bandes_du_dessous.effacer(pixmap);
    crate::perf::stage("effacer");
    let mut confie = renderer.rendre_les_couches(
        &mut pixmap.as_mut(),
        &mut dessus.as_mut(),
        store,
        (ui, pointer),
        overlay,
        regard,
    );
    poser_les_docks(
        dessus,
        (dock_manager, dock_cache),
        store,
        (renderer, ui, pointer),
        (largeur, hauteur, echelle),
    );
    // **Ce que cette image a écrit dans la couche du dessus**, relevé une fois que tout y
    // est : la chrome se dessine après le renderer, donc un relevé pris plus tôt manquerait
    // les docks -- et une bande manquée est un pixel qui ne s'efface jamais.
    // Le liseré du passé se confie à la carte : peint ici, il touchait toutes les lignes.
    confie.lisere = dock_manager
        .temps
        .regarde
        .map(|_| crate::dock::lisere(&renderer.theme, crate::theme::clamp_ui_scale(echelle)));
    confie.bandes_du_dessus = crate::present::bandes::Bandes::relever(dessus);
    crate::perf::compteur("dessus_lignes", f64::from(confie.bandes_du_dessus.lignes()));
    // Le dessous ne se relève que s'il a reçu de l'encre : le socle sait ce qu'il y a dessiné,
    // et un balayage de l'écran pour découvrir qu'il est vide ne coûterait que du temps.
    if confie.dessous_porte_quelque_chose {
        confie.bandes_du_dessous = crate::present::bandes::Bandes::relever(pixmap);
    }
    crate::perf::stage("relever");
    (tampon, confie)
}

/// Pose les panneaux déroulants dans la couche du dessus.
///
/// Une fonction à part parce que la peinture par la carte a désormais un relevé à faire après
/// elle, et que le cliquet des quatre-vingts lignes a raison : ce qui **dessine** et ce qui
/// **mesure** ne sont pas la même chose.
fn poser_les_docks(
    dessus: &mut Pixmap,
    (dock_manager, dock_cache): (&DockManager, &DockCache),
    store: &Store,
    (renderer, ui, pointer): (&Renderer, &UiState, Pointer),
    (largeur, hauteur, echelle): (u32, u32, f32),
) {
    // **Combien de panneaux ont ete REELLEMENT redessines**, et non combien sont ouverts.
    //
    // Le cache du dock garde un tampon par panneau ; sa propre documentation dit que le gain
    // est de 1,1x seulement, parce que composer un tampon coute presque ce que coute le
    // dessin qu'il remplace. La question que ce compteur tranche est donc la suivante : le
    // poste `docks`, qui vaut 1,02 ms en median et jusqu'a 7,70 au pire sur une image de
    // zoom, paie-t-il des panneaux qui se REFONT, ou seulement leur composition ? Les deux
    // n'appellent pas la meme reponse -- une cle trop large d'un cote, une couche a part de
    // l'autre -- et aucune duree ne les distingue.
    let avant = dock_cache.rendus();
    render_docks(
        &mut dessus.as_mut(),
        dock_manager,
        store,
        &DockPass {
            typo: &renderer.typography,
            theme: &renderer.theme,
            screen: ScreenFrame {
                width: largeur as f32,
                height: hauteur as f32,
                header_h: ui.header_height(),
                scale: echelle,
            },
            pointer,
            cache: Some(dock_cache),
        },
    );
    crate::perf::compteur(
        "dock_rendus",
        dock_cache.rendus().saturating_sub(avant) as f64,
    );
    crate::perf::compteur("dock_pourquoi", f64::from(dock_cache.prendre_les_raisons()));
    crate::perf::stage("docks");
}

/// Peint la scène et la chrome dans le tampon.
///
/// Extraite de `redraw` parce que celle-ci a désormais une décision à prendre avant de
/// peindre -- y a-t-il seulement quelque chose à redessiner -- et qu'un ordonnanceur qui
/// peint aussi finit par ne plus laisser voir la décision.
///
/// `scene` est le tampon réduit, quand la scène ne se rend pas à la taille de la fenêtre. La
/// chrome, elle, se pose toujours à pleine résolution par-dessus : elle ne suit pas la vue,
/// elle ne coûte pas la surface de l'écran, et une barre d'outils floue se remarque bien plus
/// qu'un canevas grossier pendant un geste.
fn peindre_tout(
    pixmap: &mut Pixmap,
    scene: Option<crate::renderer::SceneReduite<'_>>,
    renderer: &mut Renderer,
    store: &Store,
    chrome: Chrome<'_>,
    overlay: SceneOverlay<'_>,
    regard: crate::renderer::Regard,
) {
    let (width, height) = (pixmap.width(), pixmap.height());
    let mut vue = pixmap.as_mut();
    let Chrome {
        ui,
        dock_manager,
        dock_cache,
        pointer,
        echelle,
    } = chrome;

    match scene {
        Some(reduite) => renderer.rendre_reduit(&mut vue, reduite, store, ui, overlay, pointer),
        None => renderer.render(&mut vue, store, ui, overlay, pointer, regard),
    }

    // Rendu des panneaux déroulants & flottants (Top & Bottom Docks).
    // `scale` et les coordonnées de la souris sont désormais portés par
    // deux types distincts : les intervertir ne compile plus (R-44).
    let pass = DockPass {
        typo: &renderer.typography,
        theme: &renderer.theme,
        screen: ScreenFrame {
            width: width as f32,
            height: height as f32,
            header_h: ui.header_height(),
            scale: echelle,
        },
        pointer,
        cache: Some(dock_cache),
    };
    render_docks(&mut vue, dock_manager, store, &pass);
    // Le liseré du passé : sur cette voie, c'est le processeur qui le peint.
    if dock_manager.temps.regarde.is_some() {
        crate::dock::lisere_du_passe(&mut vue, &pass, crate::theme::clamp_ui_scale(echelle));
    }
    crate::perf::stage("docks");
}

#[cfg(test)]
mod tests;
