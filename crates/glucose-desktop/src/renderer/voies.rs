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
    folder, grille, halo, noter_le_cout_de_la_scene, render_ui, scene, Couche, PaintKit, Regard,
    Renderer, SymbioticHueCache,
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
/// **Ce qui passe sous les photos** : le fond, les lueurs, les membranes, les dossiers.
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
) {
    grille::poser_le_fond(fond, pixmap, store, pass, cadrage, header_h);
    // 3. Halos symbiotiques d'ambiance (Biome 2D + composition par anneaux)
    halo::draw_halos(hue_cache, pixmap, store, pass);
    crate::perf::stage("halos");
    // 4. Membranes (pointillés, titre protecteur en haut à gauche)
    scene::draw_membranes(kit, pixmap, store, pass);
    crate::perf::stage("membranes");
    // 4 bis. Dossiers — des portails vers un autre tableau, donc dessinés AVEC les autres
    // conteneurs et sous leur contenu.
    folder::draw_folders(kit, pixmap, store, pass);
    crate::perf::stage("folders");
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
    /// **Les photos que l'écran montre, et où chacune se pose** — pour la voie graphique.
    ///
    /// Le cadrage vit ici, parce qu'il touche l'index spatial du moteur ; la traduction en
    /// poses vit dans [`voies`], parce qu'elle ne dépend que du modèle et de la vue.
    pub fn photos_a_poser(
        &mut self,
        store: &Store,
        taille: (u32, u32),
        header_h: f32,
        cadrage: Cadrage,
    ) -> Vec<(String, crate::present::scene_gpu::Pose)> {
        let (vp, rangs) = self.cadrer(store, taille, header_h, cadrage);
        poses_des_photos(&vp, &rangs, store)
    }

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
    ) -> Vec<(String, crate::present::scene_gpu::Pose)> {
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
        self.rendre_la_region(dessous, store, ui, overlay, header_h, sous);
        let photos = self.photos_a_poser(store, taille, header_h, sous);

        let sur = Cadrage {
            couche: Couche::Dessus,
            ..plein
        };
        self.rendre_la_region(dessus, store, ui, overlay, header_h, sur);
        noter_le_cout_de_la_scene(debut);
        render_ui(dessus, store, ui, &self.typography, &self.theme, pointer);
        crate::perf::stage("ui");
        self.magasin.fermer();
        photos
    }
}
