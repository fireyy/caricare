mod app;
mod pages;
mod state;
mod theme;
mod widgets;

pub use app::App;
pub use theme::CUSTOM_WINDOW_DECORATIONS;

pub static THUMB_LIST_WIDTH: f32 = 200.0;
pub static THUMB_LIST_HEIGHT: f32 = 50.0;

macro_rules! spawn_evs {
    ($state:ident, |$ev:ident, $client:ident, $ctx:ident| $fut:tt) => {{
        let $client = $state.oss().clone();
        let $ev = $state.update_tx.clone();
        let $ctx = $state.ctx.clone();
        cc_runtime::spawn(async move {
            cc_runtime::tokio::task::spawn(async move { $fut });
        });
    }};
}

macro_rules! spawn_transfer {
    ($state:ident, |$transfer:ident, $ev:ident, $client:ident, $ctx:ident| $fut:tt) => {{
        let $client = $state.oss().clone();
        let $transfer = $state.transfer_manager.progress_tx.clone();
        let $ev = $state.update_tx.clone();
        let $ctx = $state.ctx.clone();
        cc_runtime::spawn(async move {
            cc_runtime::tokio::task::spawn(async move { $fut });
        });
    }};
}

pub(crate) use spawn_evs;
pub(crate) use spawn_transfer;
