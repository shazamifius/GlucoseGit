//! Point d'entrée de Glucose Desktop Native (PureRef en Rust pur).
#![allow(
    clippy::too_many_arguments,
    clippy::field_reassign_with_default,
    clippy::manual_strip,
    clippy::manual_is_multiple_of,
    clippy::manual_range_contains,
    clippy::unnecessary_cast,
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::collapsible_match,
    clippy::single_match,
    clippy::chunks_exact_to_as_chunks,
    clippy::derivable_impls,
    clippy::new_without_default,
    clippy::type_complexity,
    clippy::unwrap_or_default
)]

mod app;
mod canvas;
pub mod dock;
pub mod error;
mod icons;
pub mod interactions;
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
