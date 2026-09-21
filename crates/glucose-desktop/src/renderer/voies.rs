//! **Ce qui separe les deux voies** : ce que le processeur dessine autour des photos, et ce
//! que la carte a besoin de savoir pour les poser.
//!
//! # Pourquoi ces deux fonctions vivent ensemble
//!
//! Elles sont les deux moities d'une meme frontiere (fiche 21). L'une produit ce qui entoure
//! les photos quand la carte les pose ; l'autre traduit ce que le MODELE dit en ce que la
//! carte attend. Aucune des deux n'existe pour la voie processeur seule, et les separer
//! ferait perdre de vue qu'elles doivent rester d'accord.

use super::{
    folder, grille, halo, noter_le_cout_de_la_scene, pass, render_ui, scene, Couche, PaintKit,
    Regard, Renderer, SymbioticHueCache,
};
use crate::params::ViewPass;
use crate::params::{Pointer, SceneOverlay};
use crate::present::scene_gpu::Pose;
use crate::renderer::Cadrage;
use crate::ui::UiState;
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::types::Viewport;
use tiny_skia::PixmapMut;

/// **Ce qui passe sous les photos** : le fond, les lueurs, les membranes, les dossiers.
///
/// Tous des CONTENANTS, et c'est ce qui fait la frontière : quand la voie graphique pose
/// les photos, cette part se rend à part et lui sert de fond (voir [`Couche`]). Les
/// mélanger mettrait une membrane par-dessus la photo qu'elle contient.
///
/// # La frontière se déplace, et elle se lit ici
///
/// Le fond et les lueurs ne se peignent que sur la voie **processeur**. Sur la voie
/// graphique, la carte les produit elle-même ([`crate::present::fond_gpu`],
/// [`crate::present::lueurs_gpu`]) et cette couche ne porte plus que les membranes et les
/// dossiers — donc, sur un document qui n'en a pas, **rien du tout**.
///
/// Une fonction libre et non une methode : `pass` tient deja `&self.spatial_hash`, donc un
/// `&mut self` par-dessus ne compilerait pas. Le moteur se prete en pieces, comme pour
/// l'atelier -- le compilateur autorise des emprunts disjoints sur des champs distincts,
/// jamais a travers `&mut self`.
pub(super) fn dessiner_sous_les_photos(
    fond: grille::Fond<'_>,
    hue_cache: &mut SymbioticHueCache,
    pixmap: &mut PixmapMut,
    (store, pass, kit): (&Store, ViewPass<'_>, PaintKit<'_>),
    cadrage: Cadrage,
    header_h: f32,
) -> bool {
    if cadrage.couche.porte_le_fond() {
        grille::poser_le_fond(fond, pixmap, store, pass, cadrage, header_h);
        // 3. Halos symbiotiques d'ambiance (Biome 2D + composition par anneaux)
        halo::draw_halos(hue_cache, pixmap, store, pass);
        crate::perf::stage("halos");
    }
    // 4. Membranes (pointillés, titre protecteur en haut à gauche)
    let membranes = scene::draw_membranes(kit, pixmap, store, pass);
    crate::perf::stage("membranes");
    // 4 bis. Dossiers — des portails vers un autre tableau, donc dessinés AVEC les autres
    // conteneurs et sous leur contenu.
    let dossiers = folder::draw_folders(kit, pixmap, store, pass);
    crate::perf::stage("folders");
    // Le fond couvre tout : quand il est là, la couche porte forcément de l'encre.
    cadrage.couche.porte_le_fond() || membranes || dossiers
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
    (hue_cache, magasin): (&mut SymbioticHueCache, &mut super::magasin::Magasin),
    pixmap: &mut PixmapMut,
    (store, pass, kit): (&Store, ViewPass<'_>, PaintKit<'_>),
    (ui, overlay): (&UiState, SceneOverlay<'_>),
    cadrage: Cadrage,
) {
    // Le cadre de selection, les poignees et la jauge : ils vivent au bout de la pose des
    // photos pour la voie processeur, donc la couche du dessus doit les appeler quand la
    // carte pose a sa place -- sans quoi ils disparaissent purement.
    //
    // Les photos EN CHEMIN viennent avant eux : leurs poignees se dessinent par-dessus leur
    // cadre, comme sur la voie processeur.
    if !cadrage.couche.porte_les_photos() {
        dessiner_les_photos_en_chemin(magasin, kit, pixmap, store, pass);
        grille::dessiner_les_ornements(kit, pixmap, store, pass);
    }
    // 6. Annotations (cartes de texte, pense-betes, fleches + edition in-place)
    pass::draw_annotations(hue_cache, kit, pixmap, store, overlay.editing, pass);
    crate::perf::stage("annotations");
    let taille = (pixmap.width(), pixmap.height());
    dessiner_les_reperes_du_geste(kit.theme, pixmap, (ui, overlay), pass, taille);
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

    // 8. Boite de selection elastique (Marquee)
    if let Some((x1, y1, x2, y2)) = overlay.selection_box {
        scene::draw_selection_box(pixmap, theme, (x1, y1), (x2, y2));
    }
}

/// **Les photos dont les octets ne sont pas encore là**, dessinées comme ce qu'elles sont.
///
/// # La régression que ceci répare, et pourquoi aucun test ne pouvait la voir
///
/// Sur la voie processeur, une photo qu'on n'a pas encore décodée se dessine comme un cadre
/// gris portant son identifiant — c'est `draw_missing_image`, appelée au moment où la pose
/// échoue. Quand les photos sont descendues sur la carte, cette pose a cessé d'avoir lieu :
/// la carte ne connaît pas la photo, donc elle ne dessine rien, **et plus rien ne la
/// dessinait**. Un commentaire de ce module promettait pourtant que « le processeur porte le
/// cadre en chemin dans la couche du dessus » ; personne ne l'avait écrit.
///
/// Cela ne se voit que dans les deux secondes qui suivent l'ouverture d'un document — le
/// temps que l'atelier décode — ou sur une photo dont le fichier a disparu. Les tests
/// d'aspect, eux, montent leurs scènes avec des photos déjà là. C'est la capture de la voie
/// graphique (`examples/capture_voie_gpu.rs`) qui l'a montrée, au premier coup d'œil, en
/// comparant les deux voies côte à côte.
///
/// # L'écart d'ordre, assumé et borné
///
/// Le cadre se pose dans la couche du **dessus**, donc après les photos que la carte a
/// posées. Sur la voie processeur il se pose à son rang. Deux photos qui se chevauchent,
/// dont celle **du dessous** est en chemin, montrent donc son cadre par-dessus sa voisine
/// pendant le temps du décodage.
///
/// La réponse exacte serait que la carte pose elle-même ce cadre, à son rang : c'est un quad
/// uni et une bordure, donc l'étape 3 de la fiche 21 pour les formes. Le libellé, lui,
/// demandera un atlas de glyphes. Tant que ce n'est pas fait, un artefact transitoire vaut
/// mieux qu'une photo invisible.
pub(super) fn dessiner_les_photos_en_chemin(
    magasin: &mut super::magasin::Magasin,
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let clip = super::pass::Clip {
        width: pixmap.width() as f32,
        height: pixmap.height() as f32,
        top: pass.header_h,
    };
    let mut en_chemin = 0.0f64;
    for img in Visibles::nouvelles(pass.visibles, board).images() {
        // `reclamer` fait les deux d'un coup : elle demande la photo a l'atelier si elle
        // manque, et dit si elle est la. Une photo sans source, elle, ne viendra jamais.
        let presente = img.src.as_deref().is_some_and(|src| magasin.reclamer(src));
        if presente {
            continue;
        }
        // Le modele place une photo par son CENTRE : le coin s'en deduit.
        let (wx, wy) = crate::canvas::world_to_screen(
            img.x - img.width / 2.0,
            img.y - img.height / 2.0,
            &pass.vp,
        );
        let (sx, sy) = (wx as f32, wy as f32);
        let sw = (img.width * pass.vp.scale) as f32;
        let sh = (img.height * pass.vp.scale) as f32;
        if clip.rejects(sx, sy, sw, sh) {
            continue;
        }
        en_chemin += 1.0;
        super::scene::image::ornement::draw_missing_image(
            kit.typography,
            kit.theme,
            pixmap,
            (sx, sy),
            (sw, sh),
            &img.id,
            img.rotation,
        );
    }
    crate::perf::compteur("photos_en_chemin", en_chemin);
}

/// **Ce que le processeur confie à la carte** pour une image.
///
/// Trois listes, et rien d'autre : ce sont les seules choses que la voie graphique sait
/// produire aujourd'hui. Elles voyagent ensemble parce qu'elles viennent du **même** cadrage
/// — même vue, même culling — et que les séparer laisserait croire qu'on peut les calculer
/// à des instants différents.
#[derive(Debug, Default, Clone)]
pub struct Confie {
    /// Le fond, ou `None` quand la couche du dessous le porte encore.
    pub fond: Option<crate::present::fond_gpu::Fond>,
    /// Les lueurs des cartes visibles, dans l'ordre où le processeur les peindrait.
    pub lueurs: Vec<crate::present::lueurs_gpu::Lueur>,
    /// Où chaque photo visible se pose.
    pub photos: Vec<(String, Pose)>,
    /// La couche du dessous a-t-elle reçu de l'encre ?
    ///
    /// Quand elle n'en a pas — ni membrane, ni dossier, le fond étant sur la carte — elle est
    /// entièrement transparente, et **rien ne part** : ni ses quinze mébioctets, ni le dessin
    /// qui composerait du vide. C'est la passe elle-même qui répond, en comptant ce qu'elle
    /// dessine ; balayer les pixels coûterait un écran entier pour la même réponse.
    pub dessous_porte_quelque_chose: bool,
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
    pass: ViewPass<'_>,
    ecran: (f32, f32),
) -> Vec<crate::present::lueurs_gpu::Lueur> {
    let Some(board) = store.active_board() else {
        return Vec::new();
    };
    let mut lueurs = Vec::new();
    for ann in Visibles::nouvelles(pass.visibles, board).annotations() {
        let Some(boite) = halo::halo_geometry(ann, &pass.vp, ecran.0, ecran.1, pass.header_h)
        else {
            continue;
        };
        let (_hue, (r, v, b)) = hue_cache.get_or_compute(ann, pass.index, board);
        lueurs.push(crate::present::lueurs_gpu::Lueur {
            gauche: boite.left,
            haut: boite.top,
            droite: boite.right,
            bas: boite.bottom,
            sigma: boite.sigma,
            alpha: f32::from(halo::HALO_ALPHA) / 255.0,
            portee: halo::portee_du_flou(boite.sigma),
            rouge: f32::from(r) / 255.0,
            vert: f32::from(v) / 255.0,
            bleu: f32::from(b) / 255.0,
        });
    }
    lueurs
}

/// **Ou chaque photo visible se pose a l'ecran**, en pixels et en radians.
///
/// # Ce que cette fonction est, et ce qu'elle n'est pas
///
/// Elle ne dessine rien. Elle traduit ce que le **modele** dit en ce que la carte attend :
/// un rectangle d'ecran et un angle par photo. C'est exactement la frontiere que la fiche 21
/// trace entre le socle -- le *quoi* -- et les executants -- le *comment*.
///
/// Le culling a deja fait son travail, donc `rangs` ne designe que ce qui touche l'ecran.
///
/// Une photo sans source n'y figure pas : elle n'a pas de texture, et se dessine comme un
/// cadre « en chemin » que le processeur porte dans la couche du dessus.
pub(super) fn poses_des_photos(vp: &Viewport, rangs: &[u32], store: &Store) -> Vec<(String, Pose)> {
    let Some(board) = store.active_board() else {
        return Vec::new();
    };
    let mut posees = Vec::new();
    for img in Visibles::nouvelles(rangs, board).images() {
        let Some(src) = img.src.as_deref() else {
            continue;
        };
        // Le modele place une photo par son CENTRE : le coin s'en deduit, et c'est le piege
        // que `ce_que_porte` avait deja paye une fois.
        let (x, y) =
            crate::canvas::world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, vp);
        posees.push((
            src.to_string(),
            Pose {
                x: x as f32,
                y: y as f32,
                largeur: (img.width * vp.scale) as f32,
                hauteur: (img.height * vp.scale) as f32,
                opacite: 1.0,
                angle: img.rotation.to_radians() as f32,
            },
        ));
    }
    posees
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
        let header_h = ui.header_height();
        let taille = (dessous.width(), dessous.height());
        self.magasin.ouvrir();
        self.synchroniser_les_caches(store);
        let debut = std::time::Instant::now();

        let plein = Cadrage::plein().sous_le_regard(regard);
        let sous = Cadrage {
            couche: Couche::Dessous,
            ..plein
        };
        let encre = self.rendre_la_region(dessous, store, ui, overlay, header_h, sous);
        let mut confie = self.confier_a_la_carte(store, taille, header_h, sous);
        confie.dessous_porte_quelque_chose = encre;
        // **Reclamer, sinon rien n'est jamais decode.** Le magasin ne decode que ce qu'on lui
        // demande, et il oublie ce qu'on ne lui redemande pas. La voie processeur le faisait
        // dans `poser_les_images` ; l'oublier ici laissait le cache vide, donc aucune texture
        // a televerser -- et un ecran noir ou seules les cartes de texte se voyaient.
        for (src, _) in &confie.photos {
            self.magasin.reclamer(src);
        }

        let sur = Cadrage {
            couche: Couche::Dessus,
            ..plein
        };
        self.rendre_la_region(dessus, store, ui, overlay, header_h, sur);
        noter_le_cout_de_la_scene(debut);
        render_ui(dessus, store, ui, &self.typography, &self.theme, pointer);
        crate::perf::stage("ui");
        self.magasin.fermer();
        confie
    }

    /// **Tout ce que la carte a besoin de savoir** pour cette image : le fond, les lueurs,
    /// les photos.
    ///
    /// Les trois viennent du **même** cadrage, donc du même culling : les calculer ensemble
    /// n'est pas un regroupement de confort, c'est ce qui interdit qu'une passe voie une vue
    /// et une autre passe une autre.
    fn confier_a_la_carte(
        &mut self,
        store: &Store,
        taille: (u32, u32),
        header_h: f32,
        cadrage: Cadrage,
    ) -> Confie {
        let (vp, rangs) = self.cadrer(store, taille, header_h, cadrage);
        let pass = ViewPass {
            vp,
            visibles: &rangs,
            index: &self.spatial_hash,
            header_h,
        };
        let ecran = (taille.0 as f32, taille.1 as f32);
        let lueurs = lueurs_a_poser(&mut self.hue_cache, store, pass, ecran);
        crate::perf::stage("lueurs");
        Confie {
            fond: Some(fond_a_peindre(&self.theme, &vp, header_h)),
            lueurs,
            photos: poses_des_photos(&vp, &rangs, store),
            dessous_porte_quelque_chose: true,
        }
    }
}
