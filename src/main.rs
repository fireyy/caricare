use cc_core::init_core;
use cc_gui::App;

fn main() -> Result<(), eframe::Error> {
    init_core();

    eframe::run_native(
        "Caricare",
        eframe::NativeOptions {
            drag_and_drop_support: true,
            initial_window_size: Some([800.0, 400.0].into()),
            min_window_size: Some([400.0, 300.0].into()),
            ..Default::default()
        },
        Box::new(|cc: &eframe::CreationContext| Box::new(App::new(cc))),
    )?;

    Ok(())
}
