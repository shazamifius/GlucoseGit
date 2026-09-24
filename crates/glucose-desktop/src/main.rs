//! Point d'entrée de Glucose Desktop Native (PureRef en Rust pur).
//!
//! Tout le programme vit dans la bibliothèque du même crate ([`glucose_desktop`]) ; il ne reste
//! ici que l'ouverture de la fenêtre. Voir l'en-tête de `lib.rs` pour la raison : un binaire pur
//! ne peut être ni mesuré, ni capturé, ni instrumenté de l'extérieur.

use glucose_desktop::app::GlucoseApp;
use glucose_desktop::interactions::pincement;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut constructeur = EventLoop::builder();
    // Ce que le pavé tactile dit du pincement passe à côté de `winit` : le pont doit être en
    // place avant que la boucle existe, sinon le premier geste est déjà perdu.
    pincement::brancher(&mut constructeur);
    let event_loop = constructeur.build()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = GlucoseApp::new();
    // Le travail qu'un plantage a laissé sans nom se rouvre (HISTOIRE-1). Ici et non dans
    // `new` : les épreuves créent des centaines d'applications, et aucune ne doit rouvrir le
    // brouillon de l'utilisateur.
    app.retrouver_un_brouillon();
    // Ce qui réveillera la boucle quand le système changera le budget de la carte, même si
    // Glucose dort (ETAGES-2).
    app.reveil = Some(event_loop.create_proxy());
    event_loop.run_app(&mut app)?;

    Ok(())
}
