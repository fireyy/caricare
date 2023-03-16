use crate::theme::text_ellipsis;
use cc_core::util::is_vaild_img;
use cc_files::Cache as FileCache;
use egui::RichText;
use oss_sdk::Object;

pub fn item_ui(
    ui: &mut egui::Ui,
    data: &Object,
    url: String,
    file_cache: &mut FileCache,
) -> egui::Response {
    let response = egui::Frame {
        inner_margin: egui::style::Margin::same(5.0),
        outer_margin: egui::style::Margin::same(0.0),
        fill: ui.style().visuals.faint_bg_color,
        ..egui::Frame::default()
    }
    .show(ui, |ui| {
        ui.set_height(32.0);
        ui.set_width(ui.available_width());
        ui.horizontal_centered(|ui| {
            egui::Frame {
                ..egui::Frame::default()
            }
            .show(ui, |ui| {
                ui.set_width(32.0);
                ui.set_height(32.0);
                if data.is_file() && is_vaild_img(&data.key().to_string()) {
                    if let Some(file) = file_cache.get(&url) {
                        let size = egui::vec2(32.0, 32.0);
                        file.show_size(ui, size);
                    }
                }
            });
            ui.vertical(|ui| {
                ui.label(text_ellipsis(ui.style(), &data.name(), 1));
                ui.label(
                    RichText::new(data.size_string()).color(ui.style().visuals.weak_text_color()),
                );
            });
        });
    })
    .response
    .interact(egui::Sense::click());

    response
}
