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
//! Une seule définition, que la présentation et le banc des épreuves appellent tous deux : une
//! copie dans le banc ne verrait pas un moteur ajouté ici (la leçon de la fiche 64).

/// Les moteurs, dans l'ordre : le premier qui offre une carte est le bon.
pub const MOTEURS: [wgpu::Backends; 2] = [wgpu::Backends::PRIMARY, wgpu::Backends::GL];

/// **Une instance qui n'ouvre que ces moteurs-là.**
pub fn instance(moteurs: wgpu::Backends) -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: moteurs,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    })
}
