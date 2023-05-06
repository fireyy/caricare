mod app;
mod globals;
mod pages;
mod state;
mod widgets;
use app::App;
use cc_core::init_core;
use globals::global;

fn main() -> Result<(), eframe::Error> {
    init_core();

    let wait_for_shutdown = cc_runtime::start();

    eframe::run_native(
        "Caricare",
        eframe::NativeOptions {
            drag_and_drop_support: true,
            initial_window_size: Some([900.0, 450.0].into()),
            default_theme: eframe::Theme::Dark,
            min_window_size: Some([600.0, 450.0].into()),
            #[cfg(target_os = "macos")]
            fullsize_content: true,

            // Maybe hide the OS-specific "chrome" around the window:
            decorated: !cc_ui::CUSTOM_WINDOW_DECORATIONS,
            // To have rounded corners we need transparency:
            transparent: cc_ui::CUSTOM_WINDOW_DECORATIONS,

            follow_system_theme: false,
            ..Default::default()
        },
        Box::new(|cc: &eframe::CreationContext| Box::new(App::new(cc))),
    )?;

    wait_for_shutdown();

    Ok(())
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
