use crate::state::Update;
use crate::theme;
use crate::widgets::confirm::{Confirm, ConfirmAction};
use crate::widgets::toasts::{Toast, ToastKind, ToastOptions, Toasts};
use once_cell::sync::OnceCell;
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;

#[derive(Debug)]
pub struct Globals {
    /// Is this the first run?
    pub first_run: AtomicBool,
    /// Whether or not we are shutting down. For the UI (minions will be signaled and
    /// waited for by the overlord)    
    pub shutting_down: AtomicBool,
    pub confirm: Confirm,
    pub confirm_rx: crossbeam_channel::Receiver<ConfirmAction>,
    pub toasts: Toasts,
    pub cc_ui: theme::CCUi,
    pub update_tx: crossbeam_channel::Sender<Update>,
    pub update_rx: crossbeam_channel::Receiver<Update>,
}

static INSTANCE: OnceCell<Globals> = OnceCell::new();

impl Globals {
    pub fn new(ctx: &egui::Context) {
        let cc_ui = theme::CCUi::load_and_apply(ctx);
        let (update_tx, update_rx) = crossbeam_channel::unbounded();
        let (confirm_tx, confirm_rx) = crossbeam_channel::bounded(1);
        let globals = Self {
            first_run: AtomicBool::new(false),
            shutting_down: AtomicBool::new(false),
            confirm: Confirm::new(confirm_tx),
            confirm_rx,
            toasts: Default::default(),
            cc_ui,
            update_tx,
            update_rx,
        };
        INSTANCE.set(globals).unwrap();
    }
    pub fn global() -> &'static Globals {
        INSTANCE.get().expect("global is not initialized")
    }
    pub fn success(msg: &str) {
        Globals::global().toasts.add(Toast {
            kind: ToastKind::Success,
            text: msg.to_string(),
            options: ToastOptions::with_ttl_in_seconds(4.0),
        });
    }
    pub fn error(msg: &str) {
        Globals::global().toasts.add(Toast {
            kind: ToastKind::Error,
            text: msg.to_string(),
            options: ToastOptions::with_ttl_in_seconds(4.0),
        });
    }
}
