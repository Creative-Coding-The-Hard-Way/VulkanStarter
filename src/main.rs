extern crate vulkano;
extern crate winit;

mod application;

use application::Application;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let app = Application::initialize()?;
    app.main_loop();
    Ok(())
}
