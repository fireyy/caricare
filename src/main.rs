use cc_core::init_core;
use cc_gui::App;

fn main() -> Result<(), eframe::Error> {
    init_core();

    let wait_for_shutdown = cc_runtime::start();

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
            decorated: !cc_gui::CUSTOM_WINDOW_DECORATIONS,
            // To have rounded corners we need transparency:
            transparent: cc_gui::CUSTOM_WINDOW_DECORATIONS,

            follow_system_theme: false,
            ..Default::default()
        },
        Box::new(|cc: &eframe::CreationContext| Box::new(App::new(cc))),
    )?;

    wait_for_shutdown();

    Ok(())
}
