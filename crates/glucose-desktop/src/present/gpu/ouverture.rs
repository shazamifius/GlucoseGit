//! **Ouvrir la carte** : choisir un adaptateur, obtenir un périphérique, et refuser
//! proprement ce qu'il ne peut pas.
//!
//! # Pourquoi ce fichier existe à part
//!
//! [`super`] présente des images ; celui-ci n'en présente aucune. Il s'exécute une fois, au
//! démarrage, et ce qu'il décide ne change plus ensuite — l'adaptateur, le périphérique, les
//! limites. Les garder ensemble faisait passer `gpu.rs` les six cents lignes que la fiche 05
//! admet, et le cliquet a eu raison de le dire.

use super::super::{DesktopError, DesktopResult};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Choisit un adaptateur, ouvre un périphérique, et refuse proprement ce qu'il ne peut pas.
///
/// Le garde-fou de taille n'est pas décoratif : demander une surface plus grande que la
/// texture maximale de l'adaptateur **fait paniquer** la couche graphique au fond de la pile,
/// et la première version du module le faisait dès qu'un écran dépassait le 1080p.
pub(super) fn ouvrir(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'static>,
    width: NonZeroU32,
    height: NonZeroU32,
    carte: wgpu::PowerPreference,
) -> DesktopResult<(wgpu::Adapter, wgpu::Device, wgpu::Queue, String)> {
    let echec = |quoi: &str, e: &dyn std::fmt::Display| {
        DesktopError::WindowError(format!("présentation graphique — {quoi} : {e}"))
    };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: carte,
        compatible_surface: Some(surface),
        ..Default::default()
    }))
    .map_err(|e| echec("adaptateur", &e))?;
    let adaptateur = adapter.get_info().name;

    // Les limites de l'adaptateur, et non les limites « de base » : celles-ci plafonnent
    // les textures à 2048 pixels, ce qui refuse d'emblée tout écran au-delà du 1080p.
    // C'est la faute qui a fait paniquer la première version sur une fenêtre de 2160 de
    // large — une garantie de portabilité transformée en refus de fonctionner.
    let limites = adapter.limits();
    let plafond = limites.max_texture_dimension_2d;
    if width.get() > plafond || height.get() > plafond {
        return Err(DesktopError::WindowError(format!(
            "présentation graphique : une fenêtre de {}×{} dépasse la texture maximale de                  cet adaptateur ({plafond})",
            width.get(),
            height.get()
        )));
    }

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("glucose"),
        required_features: wgpu::Features::empty(),
        required_limits: limites,
        memory_hints: wgpu::MemoryHints::Performance,
        ..Default::default()
    }))
    .map_err(|e| echec("périphérique", &e))?;

    // Sans cela, la moindre erreur de validation tue l'application par un `panic!` au
    // fond de la pile graphique. Une erreur de pilote n'est pas un bogue de Glucose : elle
    // se dit, et l'image suivante réessaie.
    device.on_uncaptured_error(Arc::new(|e| {
        eprintln!("[Glucose] la couche graphique a refusé une commande : {e}");
    }));

    Ok((adapter, device, queue, adaptateur))
}
