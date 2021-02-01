extern crate flexi_logger;
extern crate log;
extern crate vulkano;
extern crate vulkano_shaders;
extern crate vulkano_win;
extern crate winit;
extern crate vk_sys;

mod application;
mod display;

use application::Application;
use flexi_logger::DeferredNow;
use flexi_logger::Logger;
use flexi_logger::Record;
use std::error::Error;
use std::fmt::Write as FmtWrite;

/// A human-readable formatter for multiline logs with flexi logger.
pub fn multiline_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    writeln!(
        w,
        "┌ {} [{}] [{}:{}:{}]",
        record.level(),
        now.now().format("%H:%M:%S%.6f"),
        record.module_path().unwrap_or("<unnamed>"),
        record.file().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
    )?;

    let mut full_line = String::new();
    write!(&mut full_line, "{}", &record.args())
        .expect("unable to format log!");

    let split: Vec<&str> = full_line.split('\n').collect();
    for i in 0..(split.len() - 1) {
        writeln!(w, "│ {}", split[i])?;
    }

    if split.len() > 0 {
        writeln!(w, "└ {}", split[split.len() - 1])?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    Logger::with_env_or_str("info")
        .format(multiline_format)
        .start()?;
    let app = Application::initialize()?;
    app.main_loop();
    Ok(())
}
