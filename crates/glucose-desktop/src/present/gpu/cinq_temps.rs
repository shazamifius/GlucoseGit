//! **La scene en cinq temps** : le fond, les lueurs, la couche du dessous, les photos, la
//! couche du dessus.
//!
//! # Pourquoi ce fichier existe a part
//!
//! [`super`] decrit la presentation d'une image **finie** : le processeur a tout compose, la
//! carte pose le resultat. Celle-ci est l'inverse -- la carte compose, et le processeur ne
//! fournit que ce qu'elle ne sait pas encore faire. Les deux logiques n'ont rien de commun
//! sauf la surface, et les garder dans le meme fichier lui faisait passer les six cents
//! lignes que la fiche 05 admet.
//!
//! # Ce que chaque temps coute, et pourquoi l'ordre ne se negocie pas
//!
//! Le fond REMPLACE, puisqu'il est opaque et couvre tout. Les lueurs se composent dessus. La
//! couche du dessous porte les membranes et les dossiers -- des CONTENANTS -- et **ne part
//! que si elle porte quelque chose**. Les photos viennent ensuite, et le dessus se compose en
//! dernier, transparent partout ou les photos doivent se voir.
//!
//! Les intervertir mettrait le fond par-dessus tout, ou une membrane par-dessus la photo
//! qu'elle contient.

use super::{DesktopResult, GpuPresenter, Pixmap};
use crate::present::couches;

/// Presente une image en cinq temps.
///
/// Une fonction libre plutot qu'une methode : elle touche cinq champs du presentateur en
/// meme temps, et le compilateur autorise des emprunts disjoints sur des champs distincts,
/// jamais a travers `&mut self`.
pub(super) fn presenter(
    p: &mut GpuPresenter,
    (dessous, dessus): (&Pixmap, &Pixmap),
    (confie, budget): (&crate::renderer::Confie, std::time::Duration),
    source: &crate::present::scene_gpu::Source<'_>,
) -> DesktopResult<crate::present::Issue> {
    let retenues = preparer_la_scene(p, (dessous, confie, budget), source);
    // La couche du dessous ne part que si elle porte quelque chose. Quand la carte peint
    // le fond et les lueurs, elle ne reste que les membranes et les dossiers -- et sur un
    // document qui n'en a pas, quinze mebioctets par image cessent de traverser le bus.
    // **Ce qui part sur le bus** : ce que cette image a ecrit, plus ce que la precedente
    // avait ecrit et qu'elle n'ecrit plus -- sans quoi la texture garderait l'ancien, la
    // couche n'ayant pas d'anneau qui la renouvellerait.
    let a_televerser = p.bandes_envoyees.union(&confie.bandes_du_dessus);
    p.bandes_envoyees = confie.bandes_du_dessus.clone();
    // Le dessous de même (BANDE-2) -- sauf quand il porte le fond : alors tout part.
    let du_dessous = if confie.fond.is_none() {
        crate::present::bandes::Bandes::tout(dessous.height())
    } else {
        confie.bandes_du_dessous.clone()
    };
    let dessous_utile = !du_dessous.vides();
    let dessous_a_televerser = p.bandes_du_dessous_envoyees.union(&du_dessous);
    // Un dessous qui ne se pose pas n'envoie rien : sa texture garde ce qu'elle avait, et ce
    // sont ces bandes-là qu'il faudra effacer le jour où il se reposera.
    if dessous_utile {
        p.bandes_du_dessous_envoyees = du_dessous;
    }
    crate::perf::compteur("dessous_televerse", f64::from(u8::from(dessous_utile)));
    p.couches.televerser(
        &p.device,
        &p.queue,
        (dessous, dessous_utile, &dessous_a_televerser),
        (dessus, &a_televerser),
    );
    crate::perf::stage("blit");

    let frame = match p.acquerir()? {
        Ok(frame) => frame,
        // La surface a refusé l'image : on ferme la scène comme si on l'avait posée — ce qui
        // est vrai, elle a été préparée — et on dit à l'appelant ce qui s'est passé. Sans
        // cette distinction, il croirait l'image faite et s'endormirait sur un canevas figé.
        Err(issue) => {
            fermer_la_scene(p);
            return Ok(issue);
        }
    };
    crate::perf::stage("acquerir");
    let cible = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encodeur = p
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("glucose-couches"),
        });
    couches::composer(
        &mut encodeur,
        &cible,
        couches::Temps {
            fond: &p.fond,
            lueurs: &p.lueurs,
            membranes: &p.membranes,
            couches: &p.couches,
            scene: &p.scene,
            retenues: &retenues,
        },
    );
    crate::perf::stage("encoder");
    p.queue.submit(Some(encodeur.finish()));
    crate::perf::stage("soumettre");
    drop(cible);
    p.queue.present(frame);
    crate::perf::stage("present");
    fermer_la_scene(p);
    Ok(crate::present::Issue::Presentee)
}

/// **Le premier temps : donner à la carte ce qu'elle ne connaît pas, puis poser les quads.**
///
/// Extraite de [`presenter`], qui enchaînait la préparation, la composition et la
/// présentation — trois raisons de changer, et quatre-vingt-huit lignes là où la fiche 05 en
/// admet quatre-vingts. La coupure tombe à l'endroit où la nature du travail change : ici on
/// téléverse et on écrit des tampons, après on dessine.
///
/// Rend les clés que la carte saura poser.
fn preparer_la_scene(
    p: &mut GpuPresenter,
    (dessous, confie, budget): (&Pixmap, &crate::renderer::Confie, std::time::Duration),
    source: &crate::present::scene_gpu::Source<'_>,
) -> Vec<String> {
    // Les photos, puis les cartes de texte : l'ordre du modele, et celui de la pose.
    let textures = confie.textures();
    p.scene.ouvrir();
    p.scene
        .assurer(&p.device, &p.queue, (&textures, budget), source);
    // **Ce que la carte ne connaissait pas encore**, et il fallait le séparer du reste.
    //
    // `assurer` crée et téléverse les textures que la scène réclame et que la carte n'a pas :
    // les photos qui viennent d'être décodées, et surtout les **composants** qui changent de
    // palier ou entrent à l'écran. La fiche 22 § 11.2 chiffre ce pic à trente millisecondes
    // sur une image, et il tombait jusqu'ici dans `blit` — une marque qui absorbait tout ce
    // qui la précédait, exactement comme `occlusion` (fiche 19 § 4.4) et `recolte` (fiche 22
    // § 5.4). Trois fois le même piège, et trois fois il a désigné le mauvais coupable.
    crate::perf::stage("textures");
    let ecran = (dessous.width() as f32, dessous.height() as f32);
    let retenues = p.scene.preparer(&p.device, &p.queue, ecran, &textures);
    let photos = &confie.photos[..];
    p.fond.preparer(&p.queue, ecran, confie.fond);
    p.lueurs
        .preparer(&p.device, &p.queue, ecran, &confie.lueurs);
    p.membranes
        .preparer(&p.device, &p.queue, ecran, &confie.membranes);
    // Les poses, les uniformes et les sommets : de quoi dessiner, pas de quoi téléverser
    // une image. Ce poste doit rester petit ; s'il grandit, c'est que la scène a trop de
    // quads, et ce n'est pas le même chantier que le bus.
    crate::perf::stage("poses");
    // Le diagnostic qui dit OU la chaine se rompt : combien de photos la scene demande,
    // et combien la carte sait poser.
    crate::perf::compteur("photos_vues", photos.len() as f64);
    crate::perf::compteur("photos_posees", retenues.len() as f64);
    crate::perf::compteur("lueurs_posees", confie.lueurs.len() as f64);
    crate::perf::compteur("cartes_posees", confie.cartes.len() as f64);
    crate::perf::compteur("composants", confie.composants.len() as f64);
    retenues
}

/// **Ferme la scène en gardant, de ce qui a quitté l'écran, ce que le budget de la carte
/// permet** (VRAM-1), et dit à la chronique ce que la carte porte.
pub(super) fn fermer_la_scene(p: &mut GpuPresenter) {
    let memoire = p
        .sonde
        .as_ref()
        .and_then(crate::plateforme::graphique::Sonde::lire);
    let gardable = memoire.map_or(0, |m| m.part_pour_un_cache(p.scene.octets_en_cache()));
    p.scene.fermer(gardable);
    if let Some(m) = memoire {
        crate::perf::compteur("vram_utilisee", m.utilisee as f64);
        crate::perf::compteur("vram_budget", m.budget as f64);
        crate::perf::compteur("vram_cache", p.scene.octets_en_cache() as f64);
    }
}
