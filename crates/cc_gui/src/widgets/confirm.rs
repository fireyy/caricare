use cc_core::Session;
use egui_modal::{Icon, Modal};

#[derive(Clone)]
pub enum ConfirmAction {
    Logout,
    RemoveSession(Session),
}

pub struct Confirm {
    modal: Option<Modal>,
    title: String,
    message: String,
    tx: std::sync::mpsc::SyncSender<ConfirmAction>,
    action: Option<ConfirmAction>,
}

impl Confirm {
    pub fn new(tx: std::sync::mpsc::SyncSender<ConfirmAction>) -> Self {
        Self {
            modal: None,
            title: "".into(),
            message: "".into(),
            tx,
            action: None,
        }
    }

    pub fn init(&mut self, ctx: &egui::Context) {
        let modal = Modal::new(ctx, "confirm_modal");
        modal.show(|ui| {
            modal.title(ui, &self.title);
            modal.frame(ui, |ui| {
                modal.body_and_icon(ui, &self.message, Icon::Warning);
            });
            modal.buttons(ui, |ui| {
                if modal.suggested_button(ui, "Cancel").clicked() {
                    //
                };
                if modal.caution_button(ui, "OK").clicked() {
                    if let Some(action) = &self.action {
                        self.tx.send(action.clone()).unwrap();
                    }
                };
            });
        });
        self.modal = Some(modal);
    }

    pub fn show(&mut self, message: impl Into<String>, action: ConfirmAction) {
        if let Some(modal) = &self.modal {
            self.message = message.into();
            self.action = Some(action);
            modal.open();
        }
    }
}
