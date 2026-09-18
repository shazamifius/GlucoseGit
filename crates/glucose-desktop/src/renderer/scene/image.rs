//! Poser une image sur le canevas — et ce que cela coûte selon la façon de la poser.
//!
//! Le sujet a grandi jusqu'à mériter son module : une image ne se dessine pas, elle se
//! **reporte**, depuis un niveau de réduction (MIP-1) ou depuis une vignette déjà à la forme
//! voulue (MIP-2), en composant ou en remplaçant selon ce que son opacité autorise.

use super::super::domain::draw_domain_gauge;
use super::super::handles::draw_rotated_handles;
use super::super::magasin::Magasin;
use super::super::pass::Clip;
use super::super::scale::WorldScale;
use super::super::{photo, vignette, PaintKit};
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::cout::{finesse_pour, Cout, Finesse};
use glucose_core::occlusion::{self, Calque};
use glucose_core::quadtree::Visibles;
use glucose_core::report;
use glucose_core::resize::Handle;
use glucose_core::store::Store;
use prevision::{calque_de, prevoir_la_scene, Chemin};
use tiny_skia::{
    BlendMode, Color, FilterQuality, Paint, PathBuilder, PixmapMut, PixmapPaint, Rect, Stroke,
    Transform,
};

pub(in crate::renderer) fn draw_images(
    magasin: &mut Magasin,
    cout: &mut Cout,
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let PaintKit {
        typography,
        tints,
        theme,
        ..
    } = kit;
    let scale = WorldScale::new(pass.vp.scale);
    let clip = Clip {
        width: pixmap.width() as f32,
        height: pixmap.height() as f32,
        top: pass.header_h,
    };

    // Combien d'images sont réellement posées, et quelle surface d'écran elles couvrent au
    // total. Le rapport des deux à la surface de la fenêtre dit tout de suite si le coût vient
    // du nombre ou de la SURCOUVERTURE — trente-six images empilées repeignent trente-six fois
    // le même écran, et une durée seule ne le distingue pas de trente-six images chères.
    let mut posees = 0.0f64;
    let mut pixels = 0.0f64;
    let mut cachees = 0.0f64;
    // Combien de photos ont pu passer par une vignette. Les deux chemins different d'un
    // facteur dix, et la duree seule les confond : une image chere ne dit pas si elle l'est
    // parce qu'elle dessine beaucoup ou parce qu'elle dessine MAL.
    let mut par_vignette = 0.0f64;

    // OCCLUSION-2 : on ne dessine pas ce qui sera recouvert.
    //
    // La chronique de terrain a mesure cent cinq photos couvrant cinquante fois la surface de
    // l'ecran. La premiere version ne savait traiter qu'un cas -- une seule photo couvrant
    // toute la fenetre -- et ne servait jamais : les photos se chevauchent PARTIELLEMENT, et
    // aucune ne cache seule ce que plusieurs cachent ensemble.
    //
    // Le calcul vit dans le noyau parce qu'il est geometrique : les memes rectangles donnent
    // la meme reponse sur un processeur, sur une carte graphique et sur un telephone.
    //
    // Combien de vignettes seront sorties du chantier pendant cette image. Une construction
    // reechantillonne la photo ENTIERE, meme si l'occlusion n'en laisse voir qu'une bande :
    // c'est le seul poste qui ne suive pas la surface visible, donc le seul qui puisse couter
    // cher sans que la surcouverture ne le montre. Depuis CASCADE-1 il ne devrait plus jamais
    // monter ici, et s'il montait, ce compteur le dirait.
    let vignettes_avant = magasin.vignettes.faites();
    let visibles: Vec<&glucose_core::types::BoardImage> =
        Visibles::nouvelles(pass.visibles, board).images().collect();
    let calques: Vec<Calque> = visibles
        .iter()
        .map(|img| calque_de(img, &pass, magasin))
        .collect();
    let mut caches = occlusion::Visibles::default();
    occlusion::ce_qui_se_voit(
        &calques,
        occlusion::Boite::nouvelle(0.0, clip.top, clip.width, clip.height - clip.top),
        &mut caches,
    );
    crate::perf::stage("occlusion");

    // COUT-1 : on sait ce que la scene coutera AVANT de la dessiner, donc on decide une fois
    // -- et non apres avoir rate. Sous cent images par seconde, les photos se pixelisent.
    let prevu = prevoir_la_scene(&visibles, &caches, &pass, magasin, cout);
    let finesse = finesse_pour(prevu);
    // Le prevu entre dans la trace pour qu'on puisse lire le RESIDU -- mesure moins prevu --
    // qui est la seule grandeur de tout ceci qui apprenne quelque chose de neuf.
    if let Some(prevu) = prevu {
        crate::perf::compteur("cout_prevu_us", prevu.as_micros() as f64);
    }
    crate::perf::compteur(
        "img_pixelise",
        f64::from(u8::from(finesse == Finesse::Pixelisee)),
    );
    let filtre = match finesse {
        Finesse::Lisse => report::Filtre::Lisse,
        Finesse::Pixelisee => report::Filtre::PlusProche,
    };
    // Ce que chaque chemin a REELLEMENT coute : c'est ainsi que la machine se fait comprendre.
    let mut appris: [(u64, u64); 3] = [(0, 0); 3];

    for (rang, img) in visibles.into_iter().enumerate() {
        // Une liste de parties vide veut dire : entierement recouvert, rien a peindre.
        let parts = caches.parts(rang);
        if parts.is_empty() {
            cachees += 1.0;
            continue;
        }
        let (wx, wy) = world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, &pass.vp);
        let (sx, sy) = (wx as f32, wy as f32);
        let sw = (img.width * pass.vp.scale) as f32;
        let sh = (img.height * pass.vp.scale) as f32;
        if clip.rejects(sx, sy, sw, sh) {
            continue;
        }

        posees += 1.0;
        // Le report rend LUI-MEME sa duree : entourer la pose entiere attribuait aux pixels
        // le cout fixe d'une photo -- recherche, forme, consultation des vignettes -- et le
        // modele surestimait alors d'un facteur deux, ce que son propre residu a revele.
        match poser_ou_demander(magasin, pixmap, img, (sx, sy, sw, sh), parts, filtre) {
            // REPORT-1 : ce qu'une photo coute est ce qu'elle ECRIT, et non la surface
            // qu'elle occupe. Recouverte a quatre-vingt-dix-neuf pour cent, elle en ecrit un
            // centieme -- et c'est ce centieme que la trace doit montrer.
            Some((ecrits, chemin, passees)) => {
                pixels += ecrits as f64;
                let passees = passees.as_nanos().min(u128::from(u64::MAX)) as u64;
                if chemin == Chemin::Vignette {
                    par_vignette += 1.0;
                }
                let poste = &mut appris[chemin.indice()];
                poste.0 += ecrits;
                poste.1 += passees;
            }
            None => draw_missing_image(
                typography,
                theme,
                pixmap,
                (sx, sy),
                (sw, sh),
                &img.id,
                img.rotation,
            ),
        }
        if store.selected_image_ids.contains(&img.id) {
            draw_image_adornments(pixmap, theme, scale, img, (sx, sy, sw, sh));
        }
        draw_domain_gauge(typography, tints, pixmap, scale, (sx, sy), &img.domains);
    }

    // Une observation par nature et par image : plus stable qu'une par photo, et c'est la
    // seule facon pour la machine d'apprendre ce qu'elle vaut sans qu'on le lui demande.
    for (chemin, (unites, nanos)) in Chemin::TOUS.iter().zip(appris) {
        if let Some(travail) = chemin.travail(finesse) {
            cout.observer(travail, unites, std::time::Duration::from_nanos(nanos));
        }
    }

    let fenetre = (pixmap.width() as f64) * (pixmap.height() as f64);
    crate::perf::compteur("img_n", posees);
    crate::perf::compteur("img_ecrans", pixels / fenetre.max(1.0));
    crate::perf::compteur("img_mo", magasin.octets() as f64 / 1_048_576.0);
    crate::perf::compteur("vign_mo", magasin.vignettes.octets() as f64 / 1_048_576.0);
    // Combien d'images sont encore en chemin : c'est ce qui distingue « la scene est lente »
    // de « la scene attend », et la trace ne savait pas les separer.
    crate::perf::compteur("img_attente", magasin.en_travail() as f64);
    // Combien d'images l'occlusion a evitees : le gain d'OCCLUSION-1, mesure plutot qu'annonce.
    crate::perf::compteur("img_cachees", cachees);
    crate::perf::compteur("img_par_vignette", par_vignette);
    // Une vignette existait, mais pour une forme que la vue a deja quittee : du travail fait
    // pour rien, et qui ne se corrige pas comme une vignette simplement pas encore construite.
    crate::perf::compteur("vign_perimees", magasin.vignettes.perimees() as f64);
    crate::perf::compteur("vign_pretes", magasin.vignettes.pretes() as f64);
    crate::perf::compteur("vign_orphelines", magasin.vignettes.orphelines() as f64);
    crate::perf::compteur("vign_recreees", magasin.vignettes.recreees() as f64);
    crate::perf::compteur("vign_abandonnes", magasin.vignettes.abandonnes() as f64);
    crate::perf::compteur(
        "vign_faites",
        magasin.vignettes.faites().saturating_sub(vignettes_avant) as f64,
    );
    // Combien d'images le cache a rendues à la machine : si ce nombre monte pendant qu'on
    // travaille, c'est que la mémoire se tend et que la borne se contracte.
    crate::perf::compteur("img_rendues", magasin.evincees() as f64);
}

/// Pose cette image si elle est décodée ; sinon la demande, et le dit.
///
/// # INVARIANT DECODE-1 — le rendu n'attend jamais un décodage
///
/// Rendre `None` n'est pas un échec : c'est l'état normal d'une image qui vient d'arriver
/// sur le canevas. L'appelant dessine alors son cadre, et la photo paraîtra d'elle-même à
/// l'image où l'atelier la rendra — sans qu'aucune frame ait eu à l'attendre. `Some` porte le
/// nombre de pixels écrits, qui est ce que la photo a réellement coûté.
///
/// Les deux caches s'empruntent séparément : la pyramide en écriture pour construire un
/// niveau, les vignettes pour en garder une. C'est ce que des champs distincts autorisent, et
/// ce qu'une méthode du magasin interdirait.
fn poser_ou_demander(
    magasin: &mut Magasin,
    pixmap: &mut PixmapMut,
    img: &glucose_core::types::BoardImage,
    ecran: (f32, f32, f32, f32),
    parts: &[occlusion::Boite],
    filtre: report::Filtre,
) -> Option<(u64, Chemin, std::time::Duration)> {
    let src = img.src.as_deref().filter(|s| !s.is_empty())?;
    // Réclamer marque l'image comme servie à cette passe, ce qui la met hors d'atteinte de
    // l'éviction : ce qui est à l'écran ne se rend jamais à la machine (ADAPT-1).
    if !magasin.reclamer(src) {
        return None;
    }
    let entree = magasin.cache.get(src)?;
    Some(poser(
        &entree.pyramide,
        &mut magasin.vignettes,
        pixmap,
        img,
        ecran,
        parts,
        filtre,
    ))
}

/// Pose une image sur le canevas, **restreinte aux morceaux d'elle qui atteignent l'œil**.
///
/// # Ce que REPORT-1 change ici
///
/// L'occlusion savait déjà dire quels morceaux d'une photo se voient ; ce module n'en lisait
/// qu'une chose — la liste est-elle vide — parce que le rastériseur ne sait pas peindre une
/// image restreinte à un rectangle sans un masque plein écran. Une photo recouverte à
/// quatre-vingt-dix-neuf pour cent était donc peinte à cent.
///
/// Le report du noyau, lui, **itère sur le rectangle visible** : le clip n'y coûte rien,
/// puisqu'il est le domaine du parcours. Les deux chemins historiques se retrouvent alors
/// sans être écrits deux fois :
///
/// * la **vignette**, déjà à la taille et à la phase voulues (MIP-2) — le report y constate
///   qu'un pixel vaut un pixel et se réduit à un déplacement de mémoire par ligne ;
/// * le niveau de pyramide adéquat (MIP-1), rééchantillonné à la volée.
///
/// Une image **tournée** garde le rastériseur général : sa boîte n'est plus ce qu'elle couvre,
/// donc ses morceaux visibles ne sont pas des rectangles. L'occlusion continue de la sauter
/// quand elle est entièrement cachée, ce qui reste le gain principal sur ce cas.
///
/// Rend le nombre de pixels écrits.
fn poser(
    pyramide: &photo::Pyramide,
    vignettes: &mut vignette::Vignettes,
    pixmap: &mut PixmapMut,
    img: &glucose_core::types::BoardImage,
    ecran: (f32, f32, f32, f32),
    parts: &[occlusion::Boite],
    filtre: report::Filtre,
) -> (u64, Chemin, std::time::Duration) {
    let (sx, sy, sw, sh) = ecran;
    let opaque = pyramide.opaque();

    if img.rotation != 0.0 {
        return (
            poser_en_tournant(pyramide, pixmap, img, ecran),
            Chemin::Tournee,
            std::time::Duration::ZERO,
        );
    }

    let melange = if opaque {
        report::Melange::Remplacer
    } else {
        report::Melange::Composer
    };

    // Une vignette n'a de sens que si elle tient dans la fenetre : son role est d'eviter un
    // reechantillonnage au moment de poser, or d'une image plus grande que l'ecran on ne voit
    // qu'un morceau. En demander une en zoom proche allouait des dizaines de gigaoctets, et
    // l'application plantait.
    if let Some((ecrits, passees)) =
        poser_depuis_une_vignette(vignettes, pixmap, img, ecran, parts, melange)
    {
        return (ecrits, Chemin::Vignette, passees);
    }

    // MIP-1 : on part du niveau qui couvre encore la taille posée, jamais de la résolution
    // native. Le filtre lit alors des texels voisins au lieu d'en sauter neuf sur dix.
    let loaded = pyramide.niveau_pour(sw);
    let pose = report::Pose {
        x: sx,
        y: sy,
        largeur: sw,
        hauteur: sh,
    };
    crate::perf::stage("images");
    let debut = std::time::Instant::now();
    let ecrits = reporter_les_parts(pixmap, loaded, pose, parts, melange, filtre);
    let passees = debut.elapsed();
    crate::perf::stage("report");
    (ecrits, Chemin::Echantillon, passees)
}

/// Pose une image **tournée**, par le rastériseur général.
///
/// Sa boîte n'est plus ce qu'elle couvre : ses morceaux visibles ne sont pas des rectangles,
/// donc le report ne saurait pas les clipper, et elle ne peut rien cacher non plus. Elle
/// reste néanmoins soumise à l'occlusion, qui la saute quand elle est entièrement recouverte
/// — ce qui est le gain principal sur ce cas.
fn poser_en_tournant(
    pyramide: &photo::Pyramide,
    pixmap: &mut PixmapMut,
    img: &glucose_core::types::BoardImage,
    ecran: (f32, f32, f32, f32),
) -> u64 {
    let (sx, sy, sw, sh) = ecran;
    let paint = PixmapPaint {
        quality: FilterQuality::Bilinear,
        blend_mode: mode_de_report(pyramide.opaque(), img.rotation),
        ..Default::default()
    };
    let loaded = pyramide.niveau_pour(sw);
    let ts = Transform::from_scale(sw / loaded.width() as f32, sh / loaded.height() as f32)
        .post_translate(sx, sy)
        .post_rotate_at(
            img.rotation.to_degrees() as f32,
            sx + sw / 2.0,
            sy + sh / 2.0,
        );
    pixmap.draw_pixmap(0, 0, loaded.as_ref(), &paint, ts, None);
    (f64::from(sw) * f64::from(sh)) as u64
}

/// Pose la photo depuis sa vignette, si elle en a une de prête à cette forme exacte.
///
/// Rend `None` quand la photo **déborde de la fenêtre** — une vignette n'a alors aucun sens,
/// puisqu'on n'en verrait qu'un morceau et qu'en demander une en zoom proche allouait des
/// dizaines de gigaoctets — ou quand le chantier ne l'a pas encore sortie.
fn poser_depuis_une_vignette(
    vignettes: &mut vignette::Vignettes,
    pixmap: &mut PixmapMut,
    img: &glucose_core::types::BoardImage,
    ecran: (f32, f32, f32, f32),
    parts: &[occlusion::Boite],
    melange: report::Melange,
) -> Option<(u64, std::time::Duration)> {
    let (sx, sy, sw, sh) = ecran;
    if sw > pixmap.width() as f32 || sh > pixmap.height() as f32 {
        return None;
    }
    let forme = photo::Forme::posee(sx, sy, sw, sh);
    // Le chemin du fichier se relit sur l'image : l'appelant l'a deja valide.
    let src = img.src.as_deref().unwrap_or_default();
    // Les trois postes se ferment l'un l'autre : ce qui precede la vignette est de la
    // geometrie, ce qui la suit est du report. Sans cette separation, « images » reste un bloc
    // opaque -- et c'est ce qui a empeche de voir que six cent soixante-cinq millisecondes
    // partaient ailleurs que dans le dessin.
    crate::perf::stage("images");
    // Ce qu'on voit de cette photo ordonne le chantier des vignettes : construire d'abord
    // celle qui epargne le plus de travail a chaque image.
    let visible: f64 = parts
        .iter()
        .map(|b| f64::from(b.largeur) * f64::from(b.hauteur))
        .sum();
    let vignette = vignettes.pour(&img.id, src, forme, visible);
    crate::perf::stage("vignettes");
    let vignette = vignette?;

    // La phase est deja dans la vignette : il ne reste qu'une position entiere, et le report
    // constate alors qu'un pixel vaut un pixel -- le filtre n'a rien a y faire.
    let pose = report::Pose {
        x: sx.floor(),
        y: sy.floor(),
        largeur: vignette.width() as f32,
        hauteur: vignette.height() as f32,
    };
    let debut = std::time::Instant::now();
    let ecrits = reporter_les_parts(
        pixmap,
        vignette,
        pose,
        parts,
        melange,
        report::Filtre::Lisse,
    );
    let passees = debut.elapsed();
    crate::perf::stage("report");
    Some((ecrits, passees))
}

/// Reporte la source une fois par morceau que l'occlusion laisse voir, et dit combien de
/// pixels ont été écrits.
///
/// Les deux images sont des tampons compacts de quatre octets par pixel, prémultipliés : les
/// convertir en tranches de `[u8; 4]` ne copie rien et ne suppose aucun boutisme.
fn reporter_les_parts(
    pixmap: &mut PixmapMut,
    source: &tiny_skia::Pixmap,
    pose: report::Pose,
    parts: &[occlusion::Boite],
    melange: report::Melange,
    filtre: report::Filtre,
) -> u64 {
    let (largeur, hauteur) = (pixmap.width(), pixmap.height());
    let (texels, _) = source.data().as_chunks::<4>();
    let Some(vue) = report::Vue::nouvelle(texels, source.width(), source.height()) else {
        return 0;
    };
    let (pixels, _) = pixmap.data_mut().as_chunks_mut::<4>();
    let Some(mut cible) = report::VueMut::nouvelle(pixels, largeur, hauteur) else {
        return 0;
    };
    parts
        .iter()
        .map(|part| report::reporter(&mut cible, &vue, pose, *part, melange, filtre))
        .sum()
}

/// Comment reporter une image sur le canevas : en remplaçant, ou en composant.
///
/// Le remplacement est plus rapide, mais il écrit **tous** les pixels de la zone couverte. Il
/// n'est donc licite qu'à deux conditions réunies :
///
/// * l'image est opaque — sinon le fond devrait transparaître à travers elle ;
/// * elle n'est pas tournée — sinon la zone couverte est un parallélogramme, et les coins du
///   rectangle qui l'entoure seraient remplacés par du vide, laissant quatre trous.
///
/// Les deux se constatent, aucune ne s'estime.
fn mode_de_report(opaque: bool, rotation: f64) -> BlendMode {
    if opaque && rotation == 0.0 {
        BlendMode::Source
    } else {
        BlendMode::SourceOver
    }
}

/// Ce qu'une image **sélectionnée** porte en plus : son cadre, et ses prises.
///
/// Une image verrouillée se signale par la couleur de son cadre et par l'absence de ses
/// poignées (fiche 06 § 4.3) : les deux disent le même fait, l'un de loin, l'autre au moment
/// où la main cherche une prise. Les deux suivent la rotation du nœud, comme lui.
fn draw_image_adornments(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    scale: WorldScale,
    img: &glucose_core::types::BoardImage,
    screen_box: (f32, f32, f32, f32),
) {
    let (sx, sy, sw, sh) = screen_box;
    let ink = if img.locked {
        theme.alert
    } else {
        theme.selection_frame
    };
    draw_image_selection(pixmap, ink, (sx, sy), (sw, sh), img.rotation);
    if !img.locked {
        draw_rotated_handles(pixmap, theme, scale, screen_box, &Handle::ALL, img.rotation);
    }
}

/// Une image dont les octets ne sont pas (encore) là : un cadre gris de la chrome, son
/// identifiant dedans. Monochrome — ce n'est pas du contenu, c'est son absence.
fn draw_missing_image(
    typography: &Typography,
    theme: &Theme,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    size: (f32, f32),
    id: &str,
    rotation: f64,
) {
    let Some(rect) = Rect::from_xywh(at.0, at.1, size.0, size.1) else {
        return;
    };
    // Le carré de remplacement tourne comme tournerait la texture : sans cela, une image
    // introuvable et penchée se dessinerait droite dans un cadre incliné.
    let ts = rotation_at(rotation, at, size);
    let path = PathBuilder::from_rect(rect);
    let mut fill = Paint {
        anti_alias: true,
        ..Default::default()
    };
    fill.set_color(theme.bg_hover);
    pixmap.fill_path(&path, &fill, tiny_skia::FillRule::Winding, ts, None);

    let mut border = Paint {
        anti_alias: true,
        ..Default::default()
    };
    border.set_color(theme.border_accent);
    let stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &border, &stroke, ts, None);

    typography.draw_text(
        pixmap,
        &format!("Image [{id}]"),
        at.0 + 10.0,
        at.1 + size.1 / 2.0 - 6.0,
        TextStyle {
            size: 12.0,
            color: theme.text_muted,
            face: Face::Regular,
        },
    );
}

/// La transformation qui fait tourner une boîte écran autour de son propre centre.
///
/// Un seul endroit où l'angle devient une matrice : la texture, son carré de remplacement et
/// son cadre de sélection tournent donc exactement pareil, par construction.
fn rotation_at(rotation: f64, at: (f32, f32), size: (f32, f32)) -> Transform {
    if rotation == 0.0 {
        return Transform::identity();
    }
    Transform::from_rotate_at(
        rotation.to_degrees() as f32,
        at.0 + size.0 / 2.0,
        at.1 + size.1 / 2.0,
    )
}

/// Débord du cadre de sélection autour de la texture, en pixels écran (fiche 06 § 4.2).
const IMAGE_SELECTION_INSET: f32 = 3.0;
/// Épaisseur du cadre de sélection, en pixels écran (fiche 06 § 4.2).
const IMAGE_SELECTION_STROKE: f32 = 1.25;

/// Fiche 06 § 4.2 — cadre hairline blanc pur à 0,80, débordant de 3 px, épais de 1,25 px à
/// l'écran quel que soit le zoom. Aucun néon : le contour se lit sur une image claire par le
/// liseré des poignées, sur le fond noir par le blanc.
fn draw_image_selection(
    pixmap: &mut PixmapMut,
    ink: Color,
    at: (f32, f32),
    size: (f32, f32),
    rotation: f64,
) {
    let d = IMAGE_SELECTION_INSET;
    let Some(rect) = Rect::from_xywh(at.0 - d, at.1 - d, size.0 + 2.0 * d, size.1 + 2.0 * d) else {
        return;
    };
    // Le cadre épouse le nœud : il tourne avec lui, autour du même centre.
    let ts = rotation_at(
        rotation,
        (at.0 - d, at.1 - d),
        (size.0 + 2.0 * d, size.1 + 2.0 * d),
    );
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(ink);
    let stroke = Stroke {
        width: IMAGE_SELECTION_STROKE,
        ..Default::default()
    };
    pixmap.stroke_path(&PathBuilder::from_rect(rect), &paint, &stroke, ts, None);
}

// ── Guides et boîte de sélection — taille écran constante ───────────────────

mod prevision;

#[cfg(test)]
mod tests;
