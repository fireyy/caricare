use crate::images::NetworkImages;
use crate::theme::text_ellipsis;
use crate::OssFile;
use egui::RichText;

pub fn item_ui(ui: &mut egui::Ui, data: OssFile, images: &NetworkImages) -> egui::Response {
    let initial_size = egui::vec2(200.0, 50.0);
    let (rect, response) = ui.allocate_exact_size(initial_size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        ui.allocate_ui_at_rect(rect, |ui| {
            egui::Frame {
                inner_margin: egui::style::Margin::same(5.0),
                stroke: egui::Stroke::new(2.0, ui.style().visuals.weak_text_color()),
                ..egui::Frame::default()
            }
            .show(ui, |ui| {
                ui.set_height(32.0);
                ui.horizontal_centered(|ui| {
                    egui::Frame {
                        ..egui::Frame::default()
                    }
                    .show(ui, |ui| {
                        ui.set_width(32.0);
                        ui.set_height(32.0);
                        if let Some(img) = images.get_image(data.url) {
                            let size = egui::vec2(32.0, 32.0);
                            img.show_size(ui, size);
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label(text_ellipsis(&data.name, 1));
                        ui.label(
                            RichText::new(data.size).color(ui.style().visuals.weak_text_color()),
                        );
                    });
                });
            });
        });
    }

    response
}
