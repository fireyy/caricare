use cc_core::setup_tracing;

mod app;
mod images;
mod widgets;

#[derive(Clone)]
pub struct OssFile {
    pub name: String,
    pub url: String,
    pub size: String,
    pub last_modified: String,
}

fn main() {
    setup_tracing();

    eframe::run_native(
        "Caricare",
        eframe::NativeOptions {
            drag_and_drop_support: true,
            initial_window_size: Some(egui::vec2(800.0, 400.0)),
            ..Default::default()
        },
        Box::new(|cc: &eframe::CreationContext| Box::new(app::App::new(cc))),
    );
}
