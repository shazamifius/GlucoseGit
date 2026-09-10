//! Point d'entrée de Glucose Desktop Native (PureRef en Rust pur).

mod app;
mod canvas;
pub mod dock;
mod icons;
mod renderer;
mod typography;
mod ui;

use app::GlucoseApp;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = GlucoseApp::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
