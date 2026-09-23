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

/// **Ce que le processeur confie à la carte** pour une image.
///
/// Quatre listes, et rien d'autre : ce sont les seules choses que la voie graphique sait
/// produire aujourd'hui. Elles voyagent ensemble parce qu'elles viennent du **même** cadrage
/// — même vue, même culling — et que les séparer laisserait croire qu'on peut les calculer
/// à des instants différents.
#[derive(Debug, Default, Clone)]
pub struct Confie {
    /// Le fond, ou `None` quand la couche du dessous le porte encore.
    pub fond: Option<crate::present::fond_gpu::Fond>,
    /// Les lueurs des cartes visibles, dans l'ordre où le processeur les peindrait.
    pub lueurs: Vec<crate::present::lueurs_gpu::Lueur>,
    /// Où chaque photo visible se pose, à son rang — décodée ou **en chemin**.
    pub photos: Vec<(String, Pose)>,
    /// Où chaque carte de texte visible se pose — **après** les photos, puisque les
    /// annotations passent au-dessus (COMPOSANT-1).
    pub cartes: Vec<(String, Pose)>,
    /// Ce qui se rend **à la demande**, quand la carte graphique ne connaît pas la clé : les
    /// cartes de texte, les photos en chemin.
    pub composants: Vec<super::composants::Composant>,
    /// **Les lignes que la couche du dessus porte**, relevées après que tout y a été dessiné.
    ///
    /// Elles décident de ce qui s'efface et de ce qui part sur le bus (BANDE-1) : la chrome
    /// et les ornements n'occupent qu'un huitième de l'écran, et le reste n'a aucune raison
    /// d'être touché. Le relevé appartient à la peinture et non au renderer, parce que la
    /// chrome se dessine **après** lui — un relevé pris ici manquerait les docks.
    pub bandes_du_dessus: crate::present::bandes::Bandes,
    /// La couche du dessous a-t-elle reçu de l'encre ?
    ///
    /// Quand elle n'en a pas — ni membrane, ni dossier, le fond étant sur la carte — elle est
    /// entièrement transparente, et **rien ne part** : ni ses quinze mébioctets, ni le dessin
    /// qui composerait du vide. C'est la passe elle-même qui répond, en comptant ce qu'elle
    /// dessine ; balayer les pixels coûterait un écran entier pour la même réponse.
    pub dessous_porte_quelque_chose: bool,
}

/// Une texture que la carte doit poser : ce qu'elle est, ce qu'elle montre, et où.
///
/// La **clé** change dès qu'un pixel change ; l'**identité** ne change jamais tant que c'est
/// le même composant. Les séparer est ce qui permet de poser l'ancien palier d'une carte
/// pendant que le nouveau se rend (CASCADE-2).
#[derive(Debug, Clone)]
pub struct APoser {
    /// Ce que la texture montre : une empreinte nouvelle est une texture nouvelle.
    pub cle: String,
    /// Ce que le composant **est** : stable d'un palier à l'autre, d'une frappe à l'autre.
    pub identite: String,
    /// Où la poser, à l'échelle de la vue.
    pub pose: Pose,
}

impl Confie {
    /// **Tout ce que la carte pose comme texture**, dans l'ordre du modèle : les photos, puis
    /// les cartes de texte par-dessus.
    ///
    /// Une photo est sa propre identité : ses octets ne changent pas, donc sa clé non plus.
    /// Une carte de texte porte les deux, et elles diffèrent dès qu'elle change de palier.
    pub fn textures(&self) -> Vec<APoser> {
        // **Les identités se relèvent une fois, et se lisent ensuite.**
        //
        // La première version de CASCADE-2 cherchait l'identité de chaque clé par un parcours
        // linéaire des composants. Sur le document de l'utilisateur — quatre cent
        // quatre-vingt-deux cartes — cela fait deux cent trente-deux mille comparaisons de
        // chaînes par image, et la chronique du terrain les a chiffrées : le poste `textures`
        // restait à **treize millisecondes** sur les images de zoom alors que son budget en
        // vaut moins de deux, et que le rendu des textures, lui, était bien borné.
        //
        // C'est exactement ce que la fiche 05 interdit — la géométrie calculée deux fois —
        // sous une autre forme : une correspondance recalculée à chaque élément.
        let par_cle: std::collections::HashMap<&str, &str> = self
            .composants
            .iter()
            .map(|c| (c.cle.as_str(), c.identite.as_str()))
            .collect();
        self.photos
            .iter()
            .chain(self.cartes.iter())
            .map(|(cle, pose)| APoser {
                // Une photo est sa propre identité : ses octets ne changent pas, donc sa clé
                // non plus, et elle n'est dans aucun composant.
                identite: par_cle
                    .get(cle.as_str())
                    .map_or_else(|| cle.clone(), |identite| (*identite).to_string()),
                cle: cle.clone(),
                pose: *pose,
            })
            .collect()
    }

    /// Le composant qui porte cette clé, s'il y en a un : c'est lui qui sait se rendre.
    pub fn composant(&self, cle: &str) -> Option<&super::composants::Composant> {
        self.composants.iter().find(|c| c.cle == cle)
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
) -> (Vec<(String, Pose)>, Vec<super::composants::Composant>) {
    let Some(board) = store.active_board() else {
        return (Vec::new(), Vec::new());
    };
    let mut posees = Vec::new();
    let mut composants = Vec::new();
    let mut en_chemin = 0.0f64;
    for img in Visibles::nouvelles(rangs, board).images() {
        let presente = img.src.as_deref().is_some_and(|src| magasin.reclamer(src));
        if !presente {
            if let Some(c) = regime.photo_en_chemin(img) {
                posees.push((c.cle.clone(), c.pose));
                composants.push(c);
                en_chemin += 1.0;
            }
            continue;
        }
        let Some(src) = img.src.as_deref() else {
            continue;
        };
        // Ce que le filtre a le droit de lire se decide sur la texture native, celle que la
        // carte recoit (BORDURES-4).
        let bornes = magasin.cache.get(src).map_or(Pose::PARTOUT, |e| {
            let n = e.pyramide.native();
            Pose::bornes_de(img.crop, (n.width(), n.height()))
        });
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
                angle: img.rotation as f32,
                // Le recadrage du modele, lu et jamais calcule : la carte montre la fenetre
                // de la photo que le document dit (RECADRAGE-1).
                fenetre: Pose::fenetre_de(img.crop),
                bornes,
            },
        ));
    }
    crate::perf::compteur("photos_en_chemin", en_chemin);
    (posees, composants)
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
        let Some(c) = regime.carte(
            kit,
            ann.id(),
            (*x, *y, w as f32, h as f32),
            (corps, teinte, selectionnee),
            saisie,
        ) else {
            continue;
        };
        if saisie.is_some() {
            en_saisie = Some(c);
        } else {
            posees.push((c.cle.clone(), c.pose));
            composants.push(c);
        }
    }
    if let Some(c) = en_saisie {
        posees.push((c.cle.clone(), c.pose));
        composants.push(c);
    }
    (posees, composants)
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
        let (photos, en_chemin) =
            poses_des_photos(&regime, &mut self.magasin, (&vp, &rangs, store));
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
            cartes,
            composants,
            // Le releve appartient a la peinture : la chrome se dessine APRES le renderer,
            // donc rien de juste ne peut etre dit ici.
            bandes_du_dessus: crate::present::bandes::Bandes::default(),
            dessous_porte_quelque_chose: true,
        }
    }
}
