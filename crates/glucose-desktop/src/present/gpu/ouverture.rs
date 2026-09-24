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
        // **L'allocateur au plus juste** (ETAGES-1). Réglé sur `Performance`, il réservait
        // 198 Mo de mémoire graphique dès l'ouverture, pour rien, et en gardait ~60 de plus
        // sur les 243 photos de l'utilisateur. `bench_memoire`, deux passages chacun : créer
        // et remplir 243 textures coûte 176 à 219 ms contre 179 à 188, dix changements de
        // palier 3,1 s contre 2,7 à 3,1 — l'écart est dans le bruit. La place, elle, revient
        // aux applications d'à côté.
        memory_hints: wgpu::MemoryHints::MemoryUsage,
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

/// ── Ce que la surface accepte, et sous quel format ────────────────────────────────────
///
/// Ces deux-là s'exécutent **une fois**, au démarrage, comme le reste de ce fichier : elles
/// accordent la surface à ce que la fenêtre et l'adaptateur offrent, et ce qu'elles décident
/// ne change plus. Elles vivaient dans `gpu.rs`, qui présente des images et n'avait aucune
/// raison de porter aussi cela — le cliquet des six cents lignes l'a dit.
/// **Ce que cette surface accepte**, une fois choisis son format, sa cadence et sa profondeur.
///
/// Extraite d'`avec_cadence`, qui ouvrait le périphérique, accordait la surface ET bâtissait
/// les quatre passes : trois raisons de changer, et quatre-vingt-six lignes là où la fiche 05
/// en admet quatre-vingts. La coupure tombe là où la nature du travail change — ici on
/// s'accorde à ce que la fenêtre offre, là-bas on construit ce qui dessinera dessus.
pub(super) fn accorder_la_surface(
    surface: &wgpu::Surface<'static>,
    adapter: &wgpu::Adapter,
    (width, height): (NonZeroU32, NonZeroU32),
    cadence: Option<wgpu::PresentMode>,
) -> DesktopResult<wgpu::SurfaceConfiguration> {
    let mut config = surface
        .get_default_config(adapter, width.get(), height.get())
        .ok_or_else(|| {
            DesktopError::WindowError(
                "présentation graphique : la surface n'accepte aucun format".into(),
            )
        })?;
    config.format =
        format_sans_conversion(&surface.get_capabilities(adapter).formats).unwrap_or(config.format);
    let mut config = super::succession::cadencer(surface, adapter, config, cadence);
    // **Ce que la chaîne garde en vol.** `get_default_config` met deux ; la chronique dit que
    // `get_current_texture` attend alors vingt millisecondes au repos, ce qui veut dire
    // qu'aucune image n'est libre quand on la demande.
    if let Some(n) = super::succession::images_demandees() {
        config.desired_maximum_frame_latency = n;
    }
    Ok(config)
}

/// Un format de surface qui n'impose **aucune** conversion, s'il en existe un.
///
/// # Le défaut que cette fonction répare
///
/// `get_default_config` retient volontiers `Bgra8UnormSrgb`. Écrire dedans les octets de
/// `tiny-skia` — qui sont déjà du sRGB — en les ayant déclarés linéaires fait appliquer une
/// conversion linéaire → sRGB de trop, et **toute l'interface pâlit**. C'est exactement ce
/// qu'on a vu : un fond censé être presque noir rendu en gris moyen, et les lueurs délavées.
///
/// Les deux réponses possibles étaient de déclarer la texture en sRGB — le sampler
/// reconvertit alors dans l'autre sens, et les deux conversions s'annulent — ou de refuser
/// l'espace sRGB des deux côtés. La seconde est meilleure : elle ne compense pas une
/// conversion par une autre, elle n'en fait aucune. C'est aussi ce que ce module promet.
fn format_sans_conversion(proposes: &[wgpu::TextureFormat]) -> Option<wgpu::TextureFormat> {
    proposes.iter().copied().find(|f| !f.is_srgb())
}
