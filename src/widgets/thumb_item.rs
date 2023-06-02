use crate::global;
use cc_storage::Object;
use cc_ui::text_ellipsis;
use cc_ui::THUMB_LIST_HEIGHT;
use egui::{self, vec2, RichText, Sense, WidgetInfo, WidgetType};

pub fn thumb_item_ui(ui: &mut egui::Ui, data: &mut Object, is_current: bool) -> egui::Response {
    let initial_size = vec2(
        ui.available_width(),
        THUMB_LIST_HEIGHT, // Assume there will be
    );
    let (rect, response) = ui.allocate_exact_size(initial_size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ""));

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let mut fill_color = if response.hovered() || data.selected {
            visuals.bg_fill
        } else {
            global().cc_ui.design_tokens.bottom_bar_color
        };
        if is_current {
            fill_color = global().cc_ui.design_tokens.selection_color;
        }
        ui.allocate_ui_at_rect(rect, |ui| {
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
                            RichText::new(data.size_string())
                                .color(ui.style().visuals.weak_text_color()),
                        );
                    });
                });
            });
            // .response
            // .interact(Sense::click())
        });
    }

    response
}
