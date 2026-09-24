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

mod confie;
pub mod cran;
pub use confie::{APoser, Confie};

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
    hue_cache: &mut SymbioticHueCache,
    pixmap: &mut PixmapMut,
    (store, pass, kit): (&Store, ViewPass<'_>, PaintKit<'_>),
    (ui, overlay): (&UiState, SceneOverlay<'_>),
    cadrage: Cadrage,
) {
    // 6. Annotations (cartes de texte, pense-betes, fleches + edition in-place). Sur la voie
    // graphique, les cartes de texte sont des textures que la carte pose : seuls leurs
    // ornements se dessinent ici (COMPOSANT-1).
    let cartes_par_la_carte = !cadrage.couche.porte_les_photos();
    pass::draw_annotations(
        hue_cache,
        kit,
        pixmap,
        store,
        (overlay.editing, cartes_par_la_carte),
        pass,
    );
    crate::perf::stage("annotations");
    // 6 bis. Les ornements des photos -- cadre de selection, poignees, reglette -- passent
    // au-dessus de TOUT, sur les deux voies (ORNEMENTS-1). Ils vivaient au bout de la pose de
    // chaque photo, donc sous les cartes qui la recouvraient : une poignee cachee par une
    // carte ne s'attrape pas, et les deux voies ne pouvaient pas se ressembler tant que l'une
    // les posait a un rang et l'autre au-dessus.
    grille::dessiner_les_ornements(kit, pixmap, store, pass);
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

/// **Ou chaque photo visible se pose a l'ecran**, en pixels et en radians -- et, pour
/// celles dont les octets ne sont pas encore la, le composant qui les dessinera.
///
/// # Ce que cette fonction est, et ce qu'elle n'est pas
///
/// Elle ne dessine rien. Elle traduit ce que le **modele** dit en ce que la carte attend :
/// un rectangle d'ecran et un angle par photo. C'est exactement la frontiere que la fiche 21
/// trace entre le socle -- le *quoi* -- et les executants -- le *comment*.
///
/// Le culling a deja fait son travail, donc `rangs` ne designe que ce qui touche l'ecran.
///
/// **Reclamer, sinon rien n'est jamais decode.** Le magasin ne decode que ce qu'on lui
/// demande, et il oublie ce qu'on ne lui redemande pas. C'est ici que chaque photo visible se
/// reclame, et `reclamer` dit du meme coup si elle est la : sinon, elle se pose comme un
/// cadre en chemin, **a son rang** -- ni dessous ni dessus les autres, exactement ou la voie
/// processeur la dessine.
///
/// L'angle est celui du modele, en **radians** : le multiplier par pi sur cent quatre-vingts
/// posait droite une photo penchee de pi sur huit, et aucune epreuve ne le voyait parce que
/// la photo penchee du temoin est en chemin.
pub(super) fn poses_des_photos(
    regime: &super::composants::Regime,
    magasin: &mut super::magasin::Magasin,
    (vp, rangs, store): (&Viewport, &[u32], &Store),
    cran: u32,
) -> PhotosAPoser {
    let mut photos = PhotosAPoser::default();
    let Some(board) = store.active_board() else {
        return photos;
    };
    let PhotosAPoser {
        posees,
        composants,
        niveaux,
        replis,
    } = &mut photos;
    let mut en_chemin = 0.0f64;
    for img in Visibles::nouvelles(rangs, board).images() {
        match pose_tenue(magasin, img, (vp, cran)) {
            Some(tenue) => {
                let cle = format!("{}@{}", tenue.src, tenue.facteur);
                if let Some((facteur, pose)) = tenue.repli {
                    let cle_du_repli = format!("{}@{facteur}", tenue.src);
                    niveaux.insert(cle_du_repli.clone(), (tenue.src.clone(), facteur));
                    replis.insert(cle.clone(), (cle_du_repli, pose));
                }
                niveaux.insert(cle.clone(), (tenue.src, tenue.facteur));
                posees.push((cle, tenue.pose));
            }
            None => {
                if let Some(pieces) = regime.photo_en_chemin(img) {
                    ranger(pieces, posees, composants);
                    en_chemin += 1.0;
                }
            }
        }
    }
    crate::perf::compteur("photos_en_chemin", en_chemin);
    photos
}

/// **Ce que la carte doit savoir d'une photo que le magasin tient** : son fichier, le niveau
/// voulu et sa pose — et, si ce niveau n'est pas tenu, celui qui le remplace en attendant.
struct PhotoTenue {
    src: String,
    facteur: u32,
    pose: Pose,
    repli: Option<(u32, Pose)>,
}

/// **Où la carte posera cette photo, et à quel niveau** — ou rien si le magasin ne tient
/// encore rien d'elle : elle est alors en chemin, et se dessine comme telle.
fn pose_tenue(
    magasin: &mut super::magasin::Magasin,
    img: &glucose_core::types::BoardImage,
    (vp, cran): (&Viewport, u32),
) -> Option<PhotoTenue> {
    let src = img.src.as_deref()?;
    if !magasin.reclamer(src, img.width) {
        return None;
    }
    let entree = magasin.cache.get(src)?;
    // Le modele place une photo par son CENTRE : le coin s'en deduit, et c'est le piege
    // que `ce_que_porte` avait deja paye une fois.
    let (x, y) =
        crate::canvas::world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, vp);
    let boite = (x, y, img.width * vp.scale, img.height * vp.scale);
    // **NIVEAU-GPU-1 : la carte reçoit le niveau qui couvre encore la taille posée**, et
    // non la texture native. La règle est celle de la voie processeur (MIP-1), lue au même
    // endroit : sur la taille à laquelle la SOURCE entière se pose, qu'un recadrage rend
    // plus grande que la boîte. Une épingle de 27 Mo posée en vignette partait entière sur
    // le bus — treize millisecondes de processeur pour un envoi, d'où les photos qui
    // arrivaient en vagues —, et le filtre lisait un texel sur dix : du crénelage.
    // ETAGES-3 : le cran commun, quand la carte ne tient pas l'écran entier.
    let largeur_source = cran::largeur_source(img, vp) / 2f64.powi(cran as i32);
    // **ETAGES-1 : la carte demande toujours le niveau VOULU.** S'il n'est pas tenu — offert
    // au système, il revient —, la carte pose ce qu'elle détient déjà pour cette photo, net
    // s'il était resté dans son cache. Et seulement si elle ne détient rien, le meilleur
    // niveau tenu, sous une identité à lui : poser le repli sous l'identité de la photo
    // remplacerait une texture nette par une floue, le temps d'une reprise.
    let pyramide = &entree.pyramide;
    let voulu = pyramide.facteur_pour(largeur_source as f32);
    let (tenu, _) = pyramide.meilleur_pour(largeur_source as f32)?;
    let pose = |facteur: u32| Pose {
        x: x as f32,
        y: y as f32,
        largeur: boite.2 as f32,
        hauteur: boite.3 as f32,
        opacite: 1.0,
        angle: img.rotation as f32,
        // Le recadrage du modele, lu et jamais calcule : la carte montre la fenetre de la
        // photo que le document dit (RECADRAGE-1).
        fenetre: Pose::fenetre_de(img.crop),
        // Ce que le filtre a le droit de lire se decide sur ce niveau-la (BORDURES-4).
        bornes: Pose::bornes_de(
            img.crop,
            pyramide.dimensions_natives(),
            (facteur, pyramide.dimensions(facteur)),
        ),
    };
    Some(PhotoTenue {
        src: src.to_string(),
        facteur: voulu,
        pose: pose(voulu),
        repli: (tenu != voulu).then(|| (tenu, pose(tenu))),
    })
}

/// **Ce que la passe des photos confie à la carte** : où chacune se pose, ce qui sait se
/// rendre pour celles en chemin, et le niveau de chacune de celles qui sont là.
#[derive(Default)]
pub(super) struct PhotosAPoser {
    posees: Vec<(String, Pose)>,
    composants: Vec<super::composants::Composant>,
    niveaux: std::collections::HashMap<String, (String, u32)>,
    replis: std::collections::HashMap<String, (String, Pose)>,
}

/// **Les cartes de texte que l'ecran montre**, comme composants (COMPOSANT-1).
///
/// La teinte symbiotique se calcule ici, comme dans `draw_annotations` : c'est le meme
/// parcours, au meme endroit, et c'est elle qui entre dans l'empreinte.
///
/// # La carte qu'on edite en est une aussi (COMPOSANT-2)
///
/// Elle en etait exclue, et elle se redessinait donc en entier a chaque image : le terrain du
/// 22/09 la chiffre a 9,74 ms en median sur le geste « editer du texte », dont l'image
/// mediane coute 19,48 ms. **Cinquante et une images par seconde pendant qu'on ecrit**, la ou
/// la charte en demande cent -- et ce n'est pas la frappe qui coute, c'est de refaire a
/// l'identique entre deux touches.
///
/// Ce qu'elle montre ne change qu'a la frappe et deux fois par seconde pour le curseur : son
/// empreinte le dit, et la texture ne se refait que la. Elle se pose **en dernier**, donc
/// au-dessus des autres cartes, ce qui est exactement le rang que le processeur lui donnait
/// en la dessinant dans sa seconde passe.
fn composants_de_texte(
    regime: &super::composants::Regime,
    hue_cache: &mut SymbioticHueCache,
    kit: PaintKit<'_>,
    (store, pass, edition): (&Store, ViewPass<'_>, Option<&super::TextEditSession>),
) -> (Vec<(String, Pose)>, Vec<super::composants::Composant>) {
    let Some(board) = store.active_board() else {
        return (Vec::new(), Vec::new());
    };
    let mut posees = Vec::new();
    let mut composants = Vec::new();
    // Ce qu'on edite passe au-dessus des autres cartes : on le met de cote et on l'ajoute
    // apres, plutot que de trier une liste dont l'ordre est deja celui du modele.
    let mut en_saisie = None;
    for ann in Visibles::nouvelles(pass.visibles, board).annotations() {
        let glucose_core::types::Annotation::Text {
            x, y, text, color, ..
        } = ann
        else {
            continue;
        };
        let saisie = edition.filter(|e| e.ann_id == ann.id());
        let Some((w, h)) = ann.size() else {
            continue;
        };
        let (_, symbiose) = hue_cache.get_or_compute(ann, pass.index, board);
        let teinte = color
            .as_deref()
            .map(|c| super::parse_hex_color(c, symbiose.0, symbiose.1, symbiose.2))
            .unwrap_or(symbiose);
        let selectionnee = store.selected_annotation_ids.iter().any(|s| s == ann.id());
        // Le tampon de saisie remplace le texte enregistre : c'est ce qu'on voit a l'ecran
        // pendant qu'on tape, et c'est ce que `carte_de` fait deja sur la voie processeur.
        let corps = saisie.map_or(text.as_str(), |e| e.buffer.as_str());
        let Some(pieces) = regime.carte(
            kit,
            ann.id(),
            (*x, *y, w as f32, h as f32),
            (corps, teinte, selectionnee),
            saisie,
        ) else {
            continue;
        };
        if saisie.is_some() {
            en_saisie = Some(pieces);
        } else {
            ranger(pieces, &mut posees, &mut composants);
        }
    }
    if let Some(pieces) = en_saisie {
        ranger(pieces, &mut posees, &mut composants);
    }
    (posees, composants)
}

/// **Range les pièces d'un composant** : ce qui se pose, à son rang, et tout ce qui sait se
/// rendre — le repli compris, qui ne se pose jamais pour lui-même (DE-PRES-1).
fn ranger(
    pieces: super::composants::Pieces,
    posees: &mut Vec<(String, Pose)>,
    composants: &mut Vec<super::composants::Composant>,
) {
    for c in pieces.posees {
        posees.push((c.cle.clone(), c.pose));
        composants.push(c);
    }
    composants.extend(pieces.repli);
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
        self.synchroniser_les_caches(store, taille);
        let debut = std::time::Instant::now();

        let plein = Cadrage::plein().sous_le_regard(regard);
        let sous = Cadrage {
            couche: Couche::Dessous,
            ..plein
        };
        let encre = self.rendre_la_region(dessous, store, ui, overlay, header_h, sous);
        let mut confie =
            self.confier_a_la_carte(store, taille, header_h, (sous, overlay.editing, regard));
        confie.dessous_porte_quelque_chose = encre;

        let sur = Cadrage {
            couche: Couche::Dessus,
            ..plein
        };
        self.rendre_la_region(dessus, store, ui, overlay, header_h, sur);
        noter_le_cout_de_la_scene(debut);
        render_ui(dessus, store, ui, &self.typography, &self.theme, pointer);
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
        (cadrage, edition, regard): (Cadrage, Option<&super::TextEditSession>, Regard),
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
        // Le regime des composants -- echelle de rendu, phase -- se decide une fois pour
        // tous : deux composants voisins se rendent au meme palier.
        let regime = super::composants::Regime::de(vp, regard, taille, header_h);
        let PhotosAPoser {
            posees: photos,
            composants: en_chemin,
            niveaux,
            replis,
        } = {
            self.carte.juger(&self.magasin, store, (&vp, &rangs));
            let cran = self.carte.cran;
            poses_des_photos(&regime, &mut self.magasin, (&vp, &rangs, store), cran)
        };
        let kit = PaintKit {
            typography: &self.typography,
            math: &self.math,
            tints: &self.domain_tints,
            theme: &self.theme,
        };
        let (cartes, de_texte) =
            composants_de_texte(&regime, &mut self.hue_cache, kit, (store, pass, edition));
        crate::perf::stage("composants");
        let mut composants = en_chemin;
        composants.extend(de_texte);
        Confie {
            fond: Some(fond_a_peindre(&self.theme, &vp, header_h)),
            lueurs,
            photos,
            niveaux,
            replis,
            cartes,
            composants,
            // Le releve appartient a la peinture : la chrome se dessine APRES le renderer,
            // donc rien de juste ne peut etre dit ici.
            bandes_du_dessus: crate::present::bandes::Bandes::default(),
            dessous_porte_quelque_chose: true,
        }
    }
}
