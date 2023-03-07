use cc_core::Session;
use cc_oss::object::Object as OssObject;

#[derive(Clone)]
pub enum ConfirmAction {
    Logout,
    RemoveSession(Session),
    RemoveFile(OssObject),
}

pub struct Confirm {
    is_show: bool,
    title: String,
    message: String,
    tx: std::sync::mpsc::SyncSender<ConfirmAction>,
    action: Option<ConfirmAction>,
}

impl Confirm {
    pub fn new(tx: std::sync::mpsc::SyncSender<ConfirmAction>) -> Self {
        Self {
            is_show: false,
            title: "Confirm".into(),
            message: "".into(),
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
                            ui.add_space(10.);
                            ui.horizontal(|ui| {
                                if ui.button("Ok").clicked() {
                                    self.close();
                                    if let Some(action) = &self.action {
                                        self.tx.send(action.clone()).unwrap();
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
        self.message = message.into();
        self.action = Some(action);
        self.is_show = true;
    }

    pub fn close(&mut self) {
        self.is_show = false;
    }
}
