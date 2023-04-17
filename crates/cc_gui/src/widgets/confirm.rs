use cc_core::Session;
use cc_storage::Object;

#[derive(Clone, Debug)]
pub enum ConfirmAction {
    Logout,
    RemoveSession(Session),
    RemoveFile(Object),
    CreateFolder(String),
    RemoveFiles,
    GenerateUrl(u64),
    RenameObject((String, String)),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ConfirmType {
    Message,
    Prompt,
}

#[derive(Clone, Debug)]
pub struct Confirm {
    is_show: bool,
    title: String,
    message: String,
    c_type: ConfirmType,
    prompt: String,
    tx: crossbeam_channel::Sender<ConfirmAction>,
    action: Option<ConfirmAction>,
}

impl Confirm {
    pub fn new(tx: crossbeam_channel::Sender<ConfirmAction>) -> Self {
        Self {
            is_show: false,
            title: "Confirm".into(),
            message: "".into(),
            c_type: ConfirmType::Message,
            prompt: "".into(),
            tx,
            action: None,
        }
    }

    pub fn init(&mut self, ctx: &egui::Context) {
        if self.is_show {
            egui::Area::new("confirm_mask")
                .interactable(true)
                .fixed_pos(egui::Pos2::ZERO)
                .show(ctx, |ui| {
                    let screen_rect = ui.ctx().input(|i| i.screen_rect);
                    let area_response =
                        ui.allocate_response(screen_rect.size(), egui::Sense::click());
                    if area_response.clicked() {
                        self.close();
                    }
                    ui.painter().rect_filled(
                        screen_rect,
                        egui::Rounding::none(),
                        egui::Color32::from_rgba_premultiplied(0, 0, 0, 200),
                    );
                });
            let response = egui::Window::new(&self.title)
                .resizable(false)
                .title_bar(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .inner_margin(egui::vec2(10., 10.))
                        .show(ui, |ui| {
                            ui.label(&self.message);
                            if self.c_type == ConfirmType::Prompt {
                                if let Some(action) = &self.action {
                                    let hint_text = match action {
                                        ConfirmAction::CreateFolder(d) => d.to_string(),
                                        ConfirmAction::GenerateUrl(d) => d.to_string(),
                                        ConfirmAction::RenameObject((name, _)) => name.to_string(),
                                        _ => String::new(),
                                    };
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.prompt)
                                            .hint_text(hint_text),
                                    );
                                }
                                // ui.text_edit_singleline(&mut self.prompt);
                            }
                            ui.add_space(10.);
                            ui.horizontal(|ui| {
                                if ui.button("Ok").clicked() {
                                    self.close();
                                    if let Some(action) = &self.action {
                                        let mut final_action = action.clone();
                                        match action {
                                            ConfirmAction::CreateFolder(_) => {
                                                final_action = ConfirmAction::CreateFolder(
                                                    self.prompt.clone(),
                                                );
                                            }
                                            ConfirmAction::GenerateUrl(_) => {
                                                final_action = ConfirmAction::GenerateUrl(
                                                    self.prompt.parse::<u64>().unwrap(),
                                                );
                                            }
                                            ConfirmAction::RenameObject((src, _)) => {
                                                final_action = ConfirmAction::RenameObject((
                                                    src.to_string(),
                                                    self.prompt.clone(),
                                                ));
                                            }
                                            _ => {}
                                        }
                                        self.tx.send(final_action).unwrap()
                                    }
                                }
                                if ui.button("Cancel").clicked() {
                                    self.close();
                                }
                            });
                        });
                });

            if let Some(inner_response) = response {
                ctx.move_to_top(inner_response.response.layer_id);
            }
        }
    }

    pub fn show(&mut self, message: impl Into<String>, action: ConfirmAction) {
        self.c_type = ConfirmType::Message;
        self.message = message.into();
        self.action = Some(action);
        self.is_show = true;
    }

    pub fn prompt(&mut self, message: impl Into<String>, action: ConfirmAction) {
        self.c_type = ConfirmType::Prompt;
        self.message = message.into();
        self.action = Some(action);
        self.is_show = true;
    }

    pub fn close(&mut self) {
        self.is_show = false;
    }
}
