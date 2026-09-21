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
    confie: &crate::renderer::Confie,
    source: &dyn Fn(&str) -> Option<Pixmap>,
) -> DesktopResult<()> {
    let photos = &confie.photos[..];
    p.scene.ouvrir();
    p.scene.assurer(&p.device, &p.queue, photos, source);
    let ecran = (dessous.width() as f32, dessous.height() as f32);
    let retenues = p.scene.preparer(&p.device, &p.queue, ecran, photos);
    p.fond.preparer(&p.queue, ecran, confie.fond);
    p.lueurs
        .preparer(&p.device, &p.queue, ecran, &confie.lueurs);
    // Le diagnostic qui dit OU la chaine se rompt : combien de photos la scene demande,
    // et combien la carte sait poser.
    crate::perf::compteur("photos_vues", photos.len() as f64);
    crate::perf::compteur("photos_posees", retenues.len() as f64);
    crate::perf::compteur("lueurs_posees", confie.lueurs.len() as f64);
    // La couche du dessous ne part que si elle porte quelque chose. Quand la carte peint
    // le fond et les lueurs, elle ne reste que les membranes et les dossiers -- et sur un
    // document qui n'en a pas, quinze mebioctets par image cessent de traverser le bus.
    let dessous_utile = confie.fond.is_none() || confie.dessous_porte_quelque_chose;
    crate::perf::compteur("dessous_televerse", f64::from(u8::from(dessous_utile)));
    p.couches
        .televerser(&p.device, &p.queue, (dessous, dessous_utile), dessus);
    crate::perf::stage("blit");

    let Some(frame) = p.acquerir()? else {
        p.scene.fermer();
        return Ok(());
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
    p.scene.fermer();
    Ok(())
}
