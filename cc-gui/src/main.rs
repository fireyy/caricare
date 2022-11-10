use cc_core::setup_tracing;

mod app;

use crate::app::App;

fn main() {
    setup_tracing();

    eframe::run_native(
        "Caricare",
        eframe::NativeOptions {
            drag_and_drop_support: true,
            initial_window_size: Some(egui::vec2(400.0, 400.0)),
            ..Default::default()
        },
        Box::new(|_cc: &eframe::CreationContext| Box::new(App::new())),
    );
}
