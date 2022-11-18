use cc_core::setup_tracing;
use cc_image_cache::runtime;

mod app;
mod theme;
mod widgets;

#[derive(Clone, Default)]
pub struct OssFile {
    pub name: String,
    pub key: String,
    pub url: String,
    pub size: String,
    pub last_modified: String,
}

fn main() {
    setup_tracing();

    runtime::start().unwrap();

    eframe::run_native(
        "Caricare",
        eframe::NativeOptions {
            drag_and_drop_support: true,
            initial_window_size: Some(egui::vec2(800.0, 400.0)),
            min_window_size: Some(egui::vec2(400.0, 300.0)),
            ..Default::default()
        },
        Box::new(|cc: &eframe::CreationContext| Box::new(app::App::new(cc))),
    );
}
