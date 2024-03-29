use crate::state::Update;
use once_cell::sync::OnceCell;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Globals {
    pub cc_ui: cc_ui::CCUi,
    pub update_tx: crossbeam_channel::Sender<Update>,
    pub update_rx: crossbeam_channel::Receiver<Update>,
}

static INSTANCE: OnceCell<Globals> = OnceCell::new();

impl Globals {
    pub fn new(ctx: &egui::Context) -> &'static Globals {
        let cc_ui = cc_ui::CCUi::load_and_apply(ctx);
        let (update_tx, update_rx) = crossbeam_channel::unbounded();
        let globals = Self {
            cc_ui,
            update_tx,
            update_rx,
        };
        INSTANCE.get_or_init(|| globals)
    }
}

pub fn global() -> &'static Globals {
    INSTANCE.get().expect("global is not initialized")
}
