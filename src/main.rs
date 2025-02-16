mod skyutils;
mod skyfigures;
mod skyvariants;
mod skyhats;
mod app;

use app::SkyApp;
use skyutils::Skylander;

fn main() -> eframe::Result {
    // let native_opts = eframe::NativeOptions::default();
    // eframe::run_native("Skylander Analyzer", native_opts, 
    //     Box::new(
    //         |cc| {Ok(Box::new(SkyApp::new(cc)))}
    // ))
    let sky1 = Skylander::from_nfc().expect("msg");
    println!("{}", sky1.get_figure().to_string());
    Ok(())
}