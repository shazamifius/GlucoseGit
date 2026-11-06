//! **Les moteurs de la carte, dans l'ordre où Glucose les ouvre** (FENETRES-1, fiche 82) :
//! ceux de chaque système d'abord — Vulkan, Metal, Direct3D 12 —, OpenGL seulement quand aucun
//! d'eux n'offre de carte : les très vieilles machines, que la charte interdit d'exclure.
//!
//! # Pourquoi OpenGL n'est plus ouvert d'office
//!
//! `wgpu` préfère déjà Vulkan quand il existe : ouvrir tous les moteurs ne changeait rien à la
//! carte choisie, et coûtait ceci sous Windows. Le moteur WGL de `wgpu-hal` (30.0.1,
//! `gles/wgl.rs`) pose une **fenêtre cachée de premier niveau** sur un fil qui attend la fin
//! de l'instance sans jamais lire ses messages. Une fenêtre sourde, toute la session : le
//! Restart Manager ne pouvait plus fermer Glucose (`RmShutdown` → 351), l'extinction de Windows
//! l'attendait, et un programme qui diffuse un message à toutes les fenêtres sans délai se
//! serait figé sur elle. Mesuré le 25/10 : de toutes les fenêtres de Glucose, elle seule ne
//! répondait pas.
//!
//! Sur une machine qui n'a qu'OpenGL, la fenêtre sourde reste — c'est le défaut de `wgpu`, à
//! lui dire.
//!
//! # Un moteur à la fois (MOTEUR-1, fiche 84)
//!
//! Ouvrir Vulkan et Direct3D 12 ensemble charge leurs deux pilotes, et le premier qui plante
//! emporte le processus : on ne saurait même pas lequel écarter. Chacun s'ouvre donc seul, et
//! [`super::echelle`] sait exactement lequel n'a pas survécu.
//!
//! Une seule définition, que la présentation et le banc des épreuves appellent tous deux : une
//! copie dans le banc ne verrait pas un moteur ajouté ici (la leçon de la fiche 64).

/// Les moteurs, dans l'ordre : le premier qui offre une vraie carte est le bon. Leur nom est celui
/// que `moteurs.txt` retient et que la ligne `carte` du journal dit.
pub const MOTEURS: [(wgpu::Backends, &str); 4] = [
    (wgpu::Backends::VULKAN, "vulkan"),
    (wgpu::Backends::METAL, "metal"),
    (wgpu::Backends::DX12, "dx12"),
    (wgpu::Backends::GL, "gl"),
];

/// **Ceux que ce programme sait ouvrir**, sur ce système.
pub fn ici() -> impl Iterator<Item = (wgpu::Backends, &'static str)> {
    let compiles = wgpu::Instance::enabled_backend_features();
    MOTEURS
        .into_iter()
        .filter(move |(moteur, _)| compiles.contains(*moteur))
}

/// **Une instance qui n'ouvre que ces moteurs-là**, hors de toute fenêtre : les bancs et la sonde.
pub fn instance(moteurs: wgpu::Backends) -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: moteurs,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    })
}

/// **Une instance pour dessiner dans cette fenêtre.** OpenGL doit connaître l'affichage dès sa
/// naissance : sous Wayland, sans lui, EGL ne s'ouvre que hors écran et refuse toute surface
/// (« gl not compatible with provided surface », MOTEUR-1) — son barreau était mort. Les autres
/// moteurs reçoivent l'affichage avec la surface, et ne changent pas.
pub fn instance_pour_la_fenetre(
    moteurs: wgpu::Backends,
    fenetre: &std::sync::Arc<winit::window::Window>,
) -> wgpu::Instance {
    if !moteurs.contains(wgpu::Backends::GL) {
        return instance(moteurs);
    }
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: moteurs,
        ..wgpu::InstanceDescriptor::new_with_display_handle(Box::new(std::sync::Arc::clone(
            fenetre,
        )))
    })
}
