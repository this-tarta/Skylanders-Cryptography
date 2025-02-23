mod skyutils;
mod skyfigures;
mod skyvariants;
mod skyhats;
mod app;
mod character;

use app::SkyApp;

fn main() -> eframe::Result {
    let native_opts = eframe::NativeOptions::default();
    eframe::run_native("Skylander Analyzer", native_opts, 
        Box::new(
            |cc| {Ok(Box::new(SkyApp::new(cc)))}
    ))
}