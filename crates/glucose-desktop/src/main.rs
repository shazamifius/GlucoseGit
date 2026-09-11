//! Point d'entrée de Glucose Desktop Native (PureRef en Rust pur).

mod app;
mod canvas;
pub mod dock;
pub mod error;
mod icons;
pub mod interactions;
pub mod params;
pub mod perf;
pub mod persist;
mod renderer;
mod typography;
pub mod theme;
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
