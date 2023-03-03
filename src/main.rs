use cc_core::init_core;
use cc_gui::App;

fn main() -> Result<(), eframe::Error> {
    init_core();

    eframe::run_native(
        "Caricare",
        eframe::NativeOptions {
            drag_and_drop_support: true,
            initial_window_size: Some([800.0, 400.0].into()),
            default_theme: eframe::Theme::Dark,
            min_window_size: Some([400.0, 300.0].into()),
            #[cfg(target_os = "macos")]
            fullsize_content: true,

            // Maybe hide the OS-specific "chrome" around the window:
            decorated: true,
            // To have rounded corners we need transparency:
            transparent: false,

            follow_system_theme: false,
            ..Default::default()
        },
        Box::new(|cc: &eframe::CreationContext| Box::new(App::new(cc))),
    )?;

    Ok(())
}
