use cc_core::{runtime, setup_tracing};

mod app;
mod theme;
mod widgets;

pub static SUPPORT_EXTENSIONS: [&str; 4] = ["png", "gif", "jpg", "svg"];
pub static THUMB_LIST_WIDTH: f32 = 200.0;
pub static THUMB_LIST_HEIGHT: f32 = 50.0;

fn main() -> Result<(), eframe::Error> {
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
    )?;

    Ok(())
}
