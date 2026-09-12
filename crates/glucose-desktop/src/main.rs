//! Point d'entrée de Glucose Desktop Native (PureRef en Rust pur).
//!
//! Tout le programme vit dans la bibliothèque du même crate ([`glucose_desktop`]) ; il ne reste
//! ici que l'ouverture de la fenêtre. Voir l'en-tête de `lib.rs` pour la raison : un binaire pur
//! ne peut être ni mesuré, ni capturé, ni instrumenté de l'extérieur.

use glucose_desktop::app::GlucoseApp;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = GlucoseApp::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
