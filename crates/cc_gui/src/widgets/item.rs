use crate::theme::text_ellipsis;
use cc_storage::Object;
use egui::{self, RichText, Sense};

pub fn item_ui(ui: &mut egui::Ui, data: &mut Object) -> egui::Response {
    let fill_color = if data.selected {
        egui::Color32::from_gray(60)
    } else {
        egui::Color32::from_gray(20)
    };
    egui::Frame {
        inner_margin: egui::style::Margin::same(5.0),
        outer_margin: egui::style::Margin::same(0.0),
        fill: fill_color,
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
                // TODO: show file type icon
                ui.checkbox(&mut data.selected, "");
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
    .interact(Sense::click())
}
