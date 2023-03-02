use cc_core::UploadResult;

use crate::state::State;

pub fn result_view_ui(ctx: &egui::Context, state: &mut State) {
    if state.is_show_result {
        egui::Area::new("result")
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(ui.style().visuals.extreme_bg_color)
                    .inner_margin(ui.style().spacing.window_margin)
                    .show(ui, |ui| {
                        ui.set_width(400.0);
                        ui.heading("Result");
                        ui.spacing();
                        for path in &state.upload_result {
                            match path {
                                UploadResult::Success(str) => ui.label(
                                    egui::RichText::new(format!("\u{2714} {str}"))
                                        .color(egui::Color32::GREEN),
                                ),
                                UploadResult::Error(str) => ui.label(
                                    egui::RichText::new(format!("\u{2716} {str}"))
                                        .color(egui::Color32::RED),
                                ),
                            };
                        }
                    });
            });
    }
}
