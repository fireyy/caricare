mod app;
mod globals;
mod pages;
mod state;
mod widgets;
use app::App;
use cc_core::init_core;
use globals::global;
mod util;

fn main() -> Result<(), eframe::Error> {
    init_core();

    let wait_for_shutdown = cc_runtime::start();

    eframe::run_native(
        "Caricare",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_app_id("caricare")
                .with_icon(icon_data())
                .with_drag_and_drop(true)
                .with_min_inner_size([900.0, 450.0])
                .with_inner_size([900.0, 450.0])
                .with_decorations(!cc_ui::CUSTOM_WINDOW_DECORATIONS) // Maybe hide the OS-specific "chrome" around the window
                .with_fullsize_content_view(cc_ui::FULLSIZE_CONTENT)
                .with_inner_size([1200.0, 800.0])
                .with_title_shown(!cc_ui::FULLSIZE_CONTENT)
                .with_titlebar_buttons_shown(!cc_ui::CUSTOM_WINDOW_DECORATIONS)
                .with_titlebar_shown(!cc_ui::FULLSIZE_CONTENT)
                .with_transparent(cc_ui::CUSTOM_WINDOW_DECORATIONS), // To have rounded corners without decorations we need transparency
            follow_system_theme: false,
            default_theme: eframe::Theme::Dark,
            ..Default::default()
        },
        Box::new(|cc: &eframe::CreationContext| Box::new(App::new(cc))),
    )?;

    wait_for_shutdown();

    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
fn icon_data() -> egui::IconData {
    let app_icon_png_bytes = include_bytes!("../icons/icon.png");

    // We include the .png with `include_bytes`. If that fails, things are extremely broken.
    match eframe::icon_data::from_png_bytes(app_icon_png_bytes) {
        Ok(icon_data) => icon_data,
        Err(err) => {
            #[cfg(debug_assertions)]
            panic!("Failed to load app icon: {err}");

            #[cfg(not(debug_assertions))]
            {
                tracing::warn!("Failed to load app icon: {err}");
                Default::default()
            }
        }
    }
}

macro_rules! spawn_evs {
    ($state:ident, |$ev:ident, $client:ident, $ctx:ident| $fut:tt) => {{
        let $client = $state.client().clone();
        let $ev = $state.update_tx().clone();
        let $ctx = $state.ctx.clone();
        cc_runtime::spawn(async move {
            cc_runtime::tokio::task::spawn(async move { $fut });
        });
    }};
}

macro_rules! spawn_transfer {
    ($state:ident, |$transfer:ident, $ev:ident, $client:ident, $ctx:ident| $fut:tt) => {{
        let $client = $state.client().clone();
        let $transfer = $state.transfer_manager.progress_tx.clone();
        let $ev = $state.update_tx().clone();
        let $ctx = $state.ctx.clone();
        cc_runtime::spawn(async move {
            cc_runtime::tokio::task::spawn(async move { $fut });
        });
    }};
}

pub(crate) use spawn_evs;
pub(crate) use spawn_transfer;
