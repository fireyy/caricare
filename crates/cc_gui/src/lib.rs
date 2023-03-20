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
        let _evs = $state.update_tx.clone();
        let $ctx = $state.ctx.clone();
        cc_runtime::spawn(async move {
            let _ev = _evs;
            let $ev = &_ev;
            {
                $fut
            }
        });
    }};
}

pub(crate) use spawn_evs;
