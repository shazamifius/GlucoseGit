//! **Ce qui separe les deux voies** : ce que le processeur dessine autour des photos, et ce
//! que la carte a besoin de savoir pour les poser.
//!
//! # Pourquoi ces deux fonctions vivent ensemble
//!
//! Elles sont les deux moities d'une meme frontiere (fiche 21). L'une produit ce qui entoure
//! les photos quand la carte les pose ; l'autre traduit ce que le MODELE dit en ce que la
//! carte attend. Aucune des deux n'existe pour la voie processeur seule, et les separer
//! ferait perdre de vue qu'elles doivent rester d'accord.

use super::arrow::Contournement;
use super::{
    folder, grille, halo, noter_le_cout_de_la_scene, pass, scene, Couche, PaintKit, Regard,
    Renderer, SymbioticHueCache,
};
use crate::params::ViewPass;
use crate::params::{Pointer, SceneOverlay};
use crate::renderer::Cadrage;
use crate::ui::UiState;
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::types::Viewport;
use tiny_skia::PixmapMut;

mod confie;
pub mod cran;
mod recolte;
mod tenue;
pub use confie::{APoser, Confie};
use recolte::{composants_de_texte, poses_des_photos, PhotosAPoser};

/// **Ce qui passe sous les photos** : le fond, les membranes, les dossiers.
///
/// Tous des CONTENANTS, et c'est ce qui fait la frontière : quand la voie graphique pose
/// les photos, cette part se rend à part et lui sert de fond (voir [`Couche`]). Les
/// mélanger mettrait une membrane par-dessus la photo qu'elle contient.
///
/// # La frontière se déplace, et elle se lit ici
///
/// Le fond ne se peint ici que sur la voie **processeur**. Sur la voie graphique, la carte le
/// produit elle-même ([`crate::present::fond_gpu`]) et cette couche ne porte plus que les
/// membranes et les dossiers — donc, sur un document qui n'en a pas, **rien du tout**.
///
/// Les lueurs des cartes n'y sont plus (LUEUR-3) : elles passent devant les photos, comme
/// l'ombre d'un bloc de Tauri passait devant la toile ([`dessiner_sur_les_photos`]).
///
/// Une fonction libre et non une methode : `pass` tient deja `&self.spatial_hash`, donc un
/// `&mut self` par-dessus ne compilerait pas. Le moteur se prete en pieces, comme pour
/// l'atelier -- le compilateur autorise des emprunts disjoints sur des champs distincts,
/// jamais a travers `&mut self`.
pub(super) fn dessiner_sous_les_photos(
    fond: grille::Fond<'_>,
    pixmap: &mut PixmapMut,
    (store, pass, kit): (&Store, ViewPass<'_>, PaintKit<'_>),
    cadrage: Cadrage,
    header_h: f32,
) -> bool {
    if cadrage.couche.porte_le_fond() {
        grille::poser_le_fond(fond, pixmap, store, pass, cadrage, header_h);
        // 3 bis. Le corps des zones d'un gabarit, sous tout le contenu — le premier rang de
        // Tauri (PRESET-1). Sur la voie graphique, la carte pose les mêmes rectangles au même
        // rang, avant ses membranes.
        let ecran = (pixmap.width() as f32, pixmap.height() as f32);
        super::zone::remplir(pixmap, &premier_rang(store, pass, ecran));
        crate::perf::stage("zones");
    }
    // 4. Membranes. Leur FORME se peint ici quand le processeur porte le fond, et sur la
    // carte sinon (MEMB-FORME-1) ; leurs titres, poignées et réglettes, ici sur les deux voies.
    let membranes = if cadrage.couche.porte_le_fond() {
        scene::draw_membranes(kit, pixmap, store, pass)
    } else {
        scene::draw_membrane_ornaments(kit, pixmap, store, pass)
    };
    crate::perf::stage("membranes");
    // 4 ter. Le nom et la phrase des zones, le numéro et la description des cases du
    // storyboard, sur les deux voies — comme les titres des membranes.
    let zones = super::zone::ecrire_les_textes(kit, pixmap, store, pass)
        | super::storyboard::ecrire_les_textes(kit, pixmap, store, pass);
    // 4 bis. Dossiers — des portails vers un autre tableau, donc dessinés AVEC les autres
    // conteneurs et sous leur contenu.
    let dossiers = folder::draw_folders(kit, pixmap, store, pass);
    crate::perf::stage("folders");
    // Le fond couvre tout : quand il est là, la couche porte forcément de l'encre.
    cadrage.couche.porte_le_fond() || zones || membranes || dossiers
}

/// **Le premier rang du monde** : le corps des zones d'un gabarit, puis les cases du storyboard
/// — des rectangles unis, que le processeur remplit et que la carte pose juste après le fond
/// (PRESET-1, STORYBOARD-1). Une seule liste pour les deux voies.
pub(super) fn premier_rang(
    store: &Store,
    pass: ViewPass<'_>,
    ecran: (f32, f32),
) -> Vec<super::zone::Rectangle> {
    let mut rang = super::zone::rectangles(store, pass, ecran);
    rang.extend(super::storyboard::rectangles(store, pass, ecran));
    rang
}

/// **Ce qui passe sur les photos** : les ornements, les annotations, les repères du geste.
///
/// La jumelle de [`dessiner_sous_les_photos`], et elles vivent ensemble pour la même raison :
/// ce sont les deux moitiés d'une frontière, et une frontière dont les deux bords sont écrits
/// à deux endroits finit par ne plus être la même des deux côtés.
///
/// Une fonction libre, comme sa jumelle : `pass` tient déjà `&self.spatial_hash`, donc un
/// `&mut self` par-dessus ne compilerait pas.
pub(super) fn dessiner_sur_les_photos(
    hue_cache: &mut SymbioticHueCache,
    pixmap: &mut PixmapMut,
    (store, pass, kit): (&Store, ViewPass<'_>, PaintKit<'_>),
    (ui, overlay): (&UiState, SceneOverlay<'_>),
    cadrage: Cadrage,
) {
    // 5 bis. Les lueurs des cartes, devant les photos et sous les cartes (LUEUR-3). Sur la
    // voie graphique, la carte les pose au meme rang, entre ses photos et ses cartes.
    if cadrage.couche.porte_le_fond() {
        halo::draw_halos(hue_cache, pixmap, store, (pass, overlay.designees));
        crate::perf::stage("halos");
    }
    // 6. Annotations (cartes de texte, pense-betes, fleches + edition in-place). Sur la voie
    // graphique, les cartes de texte sont des textures que la carte pose : seuls leurs
    // ornements se dessinent ici (COMPOSANT-1).
    let cartes_par_la_carte = !cadrage.couche.porte_les_photos();
    pass::draw_annotations(
        hue_cache,
        kit,
        pixmap,
        store,
        (
            overlay.editing,
            cartes_par_la_carte,
            pass::Survols {
                eclairages: overlay.eclairages,
                badges: overlay.badges,
            },
        ),
        pass,
    );
    // Ce qu'une flèche survolée désigne : son fond, son liseré et ses lettres sont dans la
    // carte ; sa lueur passe par-dessus (FLECHE-4, PASSAGE-2).
    super::passages::eclairer(
        hue_cache,
        (kit.typography, kit.math, kit.mises_en_page),
        pixmap,
        (store, overlay.eclairages, pass),
    );
    crate::perf::stage("annotations");
    // 6 bis. Les ornements des photos -- cadre de selection, poignees, reglette -- passent
    // au-dessus de TOUT, sur les deux voies (ORNEMENTS-1). Ils vivaient au bout de la pose de
    // chaque photo, donc sous les cartes qui la recouvraient : une poignee cachee par une
    // carte ne s'attrape pas, et les deux voies ne pouvaient pas se ressembler tant que l'une
    // les posait a un rang et l'autre au-dessus.
    // Pendant une lecture, aucune prise : rien ne se prend, la case se regarde (fiche 67).
    if ui.lecture.is_none() {
        grille::dessiner_les_ornements(kit, pixmap, store, pass);
        super::handles::dessiner_le_cadre_du_groupe(kit.theme, pixmap, store, pass);
        // La zone de gabarit choisie : ses prises, comme celles d'un nœud choisi (PRESET-1).
        let choisie = ui.zones.choisie.as_deref();
        super::zone::orner_la_zone_choisie(pixmap, kit.theme, (store, pass), choisie);
        let case = ui.storyboard.choisie.as_deref();
        super::storyboard::orner_la_case_choisie(pixmap, kit.theme, (store, pass), case);
    }
    super::lecture::voiler(kit, pixmap, store, (ui, pass));
    crate::perf::stage("ornements");
    let taille = (pixmap.width(), pixmap.height());
    dessiner_les_reperes_du_geste(kit.theme, pixmap, (ui, overlay), pass, taille);
    // 9. Les images qui arrivent d'un depot web : au-dessus de tout, comme un toast, mais la
    // ou elles se poseront.
    super::arrivage::dessiner_les_arrivages(
        kit,
        pixmap,
        overlay.arrivages,
        (&pass.vp, ui.scale_factor),
    );
    // **Une marque ici, et c'est la cinquieme du genre.** Sans elle, tout ce qui suit
    // `ornements` -- les guides, la boite de selection, les marqueurs de telechargement, puis
    // le debut de l'interface -- se facturait a `bande`, qui a monte a 14,95, 32,28 et
    // 20,66 ms sur le terrain alors que `bench_bande` la chiffre a 0,49 ms refaite.
    crate::perf::stage("reperes");
}

/// Les reperes du geste en cours : les guides d'alignement et la boite de selection.
///
/// Ils appartiennent a la scene parce qu'ils suivent la vue, mais pas au contenu : ils
/// n'existent que pendant un geste, ne sont dans aucun document, et disparaitront sans
/// laisser de trace. C'est aussi ce qui les distingue pour A.1 -- ils salissent l'ecran a
/// chaque mouvement de la main, et rien d'autre ne le fait pour eux.
fn dessiner_les_reperes_du_geste(
    theme: &super::Theme,
    pixmap: &mut PixmapMut,
    (ui, overlay): (&UiState, SceneOverlay<'_>),
    pass: ViewPass<'_>,
    taille: (u32, u32),
) {
    // 7. Guides d'alignement intelligents (SNAP-1)
    if ui.smart_align {
        scene::draw_guides(
            theme,
            pixmap,
            overlay.guides,
            &pass.vp,
            taille,
            pass.header_h,
        );
    }

    // 7 bis. L'élément à naître sous un outil de création armé (PLACEMENT-1).
    if let Some(fantome) = overlay.fantome {
        scene::dessiner_le_fantome(pixmap, theme, fantome, &pass.vp);
    }
    // Et le gabarit qui suit le curseur, une boîte par zone (PRESET-1).
    for &zone in overlay.gabarit {
        scene::dessiner_le_fantome(pixmap, theme, zone, &pass.vp);
    }

    // 8. Boite de selection elastique (Marquee)
    if let Some((x1, y1, x2, y2)) = overlay.selection_box {
        scene::draw_selection_box(pixmap, theme, (x1, y1), (x2, y2));
    }
}

/// **Le fond de cette image**, tel que la carte a besoin de le connaître.
///
/// Rien n'est décidé ici : le pas de la grille, le rayon d'un point, son opacité et son
/// extinction viennent du socle ([`super::scene::grid`]), qui les tient de la fiche 06.
pub(super) fn fond_a_peindre(
    theme: &super::Theme,
    vp: &Viewport,
    header_h: f32,
) -> crate::present::fond_gpu::Fond {
    let canvas = theme.bg_canvas;
    let (pas, rayon, opacite) = super::scene::grid::grid_params(vp).unwrap_or((0.0, 0.0, 0.0));
    crate::present::fond_gpu::Fond {
        rouge: canvas.red(),
        vert: canvas.green(),
        bleu: canvas.blue(),
        echelle: vp.scale as f32,
        vue_x: vp.x as f32,
        vue_y: vp.y as f32,
        pas: pas as f32,
        rayon,
        opacite,
        gris: f32::from(super::scene::grid::grid_grey()) / 255.0,
        header: header_h,
    }
}

/// **Les lueurs que l'écran montre**, et où chacune se pose.
///
/// # Ce que cette fonction emporte avec elle, et qu'il ne faut pas perdre
///
/// `draw_halos` faisait **deux** choses sans rapport : calculer la teinte symbiotique de
/// chaque carte — qui dépend du voisinage, coûte cher, et sert aussi aux annotations — et
/// peindre. Déplacer la peinture sur la carte sans garder le calcul laisserait les teintes
/// périmées ; c'est exactement le piège qui a fait disparaître les poignées quand la pose des
/// photos est descendue sur la carte (fiche 21). Le calcul reste donc ici, au même endroit du
/// même parcours.
pub(super) fn lueurs_a_poser(
    hue_cache: &mut SymbioticHueCache,
    store: &Store,
    (pass, designees): (ViewPass<'_>, &[(String, f32)]),
    ecran: (f32, f32),
) -> Vec<crate::present::lueurs_gpu::Lueur> {
    let Some(board) = store.active_board() else {
        return Vec::new();
    };
    let mut lueurs = Vec::new();
    for ann in Visibles::nouvelles(pass.visibles, board).annotations() {
        let eclat = halo::Eclat::de(ann, designees);
        let cadre = (ecran.0, ecran.1, pass.header_h);
        let Some(boite) = halo::halo_geometry(ann, (&pass.vp, pass.echelle()), cadre, eclat) else {
            continue;
        };
        let (_hue, teinte) = hue_cache.get_or_compute(ann, pass.index, board);
        lueurs.push(crate::present::lueurs_gpu::Lueur::de(
            boite,
            teinte,
            eclat.alpha(),
        ));
    }
    lueurs
}

/// Les deux entrees de la voie graphique, posees ici pour que le moteur reste lisible.
impl Renderer {
    /// **Rend la scene en deux couches, et dit ou les photos se posent.**
    ///
    /// C'est l'entree de la voie graphique, la ou [`Renderer::render`] est celle de la voie
    /// processeur. Elle produit ce qui entoure les photos -- dessous et dessus -- et laisse
    /// la carte les poser entre les deux.
    ///
    /// Le dessus part **transparent** : tout ce qui n'y est pas dessine laisse voir les
    /// photos, et c'est ce qui fait que la composition est juste sans qu'aucune region ne
    /// soit calculee.
    pub fn rendre_les_couches(
        &mut self,
        dessous: &mut PixmapMut,
        dessus: &mut PixmapMut,
        store: &Store,
        chrome: (&mut UiState, Pointer),
        overlay: SceneOverlay<'_>,
        regard: Regard,
    ) -> Confie {
        let (ui, pointer) = chrome;
        let taille = (dessous.width(), dessous.height());
        self.magasin.ouvrir();
        self.synchroniser_les_caches(store, ui, taille);
        let header_h = ui.header_height();
        let debut = std::time::Instant::now();

        let plein = Cadrage::plein().sous_le_regard(regard);
        let sous = Cadrage {
            couche: Couche::Dessous,
            ..plein
        };
        let encre = self.rendre_la_region(dessous, store, ui, overlay, header_h, sous);
        let mut confie = self.confier_a_la_carte(store, taille, header_h, (sous, overlay, regard));
        confie.dessous_porte_quelque_chose = encre;

        let sur = Cadrage {
            couche: Couche::Dessus,
            ..plein
        };
        self.rendre_la_region(dessus, store, ui, overlay, header_h, sur);
        noter_le_cout_de_la_scene(debut);
        // **La chrome ne passe plus par la couche du dessus** (CHROME-1) : la carte la pose
        // d'après ses caches, et la couche ne porte plus que ce qui suit la vue.
        let ecran = (taille.0 as f32, taille.1 as f32);
        let kit = (&self.typography, &self.theme);
        confie.chrome = crate::ui::chrome::ChromeConfiee::de(store, ui, kit, (ecran, pointer));
        self.magasin.fermer();
        confie
    }

    /// **Tout ce que la carte a besoin de savoir** pour cette image : le fond, les lueurs,
    /// les photos, les composants.
    ///
    /// Tous viennent du **même** cadrage, donc du même culling : les calculer ensemble
    /// n'est pas un regroupement de confort, c'est ce qui interdit qu'une passe voie une vue
    /// et une autre passe une autre.
    fn confier_a_la_carte(
        &mut self,
        store: &Store,
        taille: (u32, u32),
        header_h: f32,
        (cadrage, overlay, regard): (Cadrage, SceneOverlay<'_>, Regard),
    ) -> Confie {
        let edition = overlay.editing;
        let (vp, rangs, densite) = self.cadrer(store, taille, header_h, cadrage);
        let pass = ViewPass {
            vp,
            visibles: &rangs,
            index: &self.spatial_hash,
            contournement: Some(Contournement::de(&self.itineraires, &self.suivi_du_geste)),
            header_h,
            densite,
        };
        let ecran = (taille.0 as f32, taille.1 as f32);
        let lueurs = lueurs_a_poser(&mut self.hue_cache, store, (pass, overlay.designees), ecran);
        crate::perf::stage("lueurs");
        let membranes = scene::formes_des_membranes(store, pass, taille);
        let zones = premier_rang(store, pass, ecran);
        crate::perf::stage("membranes");
        let fleches = super::arrow::fleches_a_poser(
            &mut self.hue_cache,
            (store, pass),
            (&self.typography, &self.math),
            ecran,
        );
        crate::perf::stage("fleches");
        // Le regime des composants -- echelle de rendu, phase -- se decide une fois pour
        // tous : deux composants voisins se rendent au meme palier.
        let regime = super::composants::Regime::de((vp, densite), regard, taille, header_h);
        let kit = PaintKit {
            typography: &self.typography,
            math: &self.math,
            mises_en_page: &self.mises_en_page,
            tints: &self.domain_tints,
            theme: &self.theme,
        };
        let PhotosAPoser {
            recolte: des_photos,
            niveaux,
            replis,
        } = poses_des_photos(
            (&regime, kit),
            (&mut self.magasin, &mut self.carte),
            (&vp, &rangs, store),
        );
        // Ce qu'une carte ne cache pas : le fond, et les membranes qui la contiennent (LUEUR-3).
        let contenants = super::card::Contenants::nouveaux(&self.theme, &membranes);
        let de_texte = composants_de_texte(
            (&regime, &contenants, overlay.eclairages),
            &mut self.hue_cache,
            kit,
            (store, pass, edition),
        );
        crate::perf::stage("composants");
        let confie = Confie {
            fond: Some(fond_a_peindre(&self.theme, &vp, header_h)),
            lueurs,
            zones,
            membranes,
            fleches,
            niveaux,
            replis,
            dessous_porte_quelque_chose: true,
            // Les panneaux, le releve des bandes et le lisere appartiennent a la peinture : la
            // chrome se dessine APRES le renderer, donc rien de juste ne peut en etre dit ici.
            ..Confie::default()
        };
        des_photos.puis_les_cartes(de_texte, confie)
    }
}
