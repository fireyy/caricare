use crate::global;
use crate::state::Update;
use crate::widgets::confirm::ConfirmAction;
use cc_storage::Object;
use cc_ui::icon;
use egui::{self, style::Margin, vec2, Color32, Frame, Sense, WidgetInfo, WidgetType};

pub fn list_item_ui(ui: &mut egui::Ui, data: &mut Object, is_current: bool) -> egui::Response {
    let row_height = ui.text_style_height(&egui::TextStyle::Body);
    let initial_size = vec2(
        ui.available_width(),
        row_height, // Assume there will be
    );
    let (rect, response) = ui.allocate_exact_size(initial_size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ""));

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let mut fill_color = if response.hovered() || data.selected {
            visuals.bg_fill
        } else {
            Color32::TRANSPARENT
        };
        if is_current {
            fill_color = global().cc_ui.design_tokens.selection_color;
        }
        ui.allocate_ui_at_rect(rect, |ui| {
            Frame {
                fill: fill_color,
                inner_margin: Margin::same(0.0),
                ..Frame::default()
            }
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    egui::Frame::none().show(ui, |ui| {
                        ui.set_width(60.);
                        if ui.button(icon::DELETE).on_hover_text("Delete").clicked() {
                            global()
                                .update_tx
                                .send(Update::Confirm((
                                    format!("Do you confirm to delete this item: {}?", data.key()),
                                    ConfirmAction::RemoveFile(data.clone()),
                                )))
                                .unwrap();
                        }
                        // download file
                        if ui
                            .button(icon::DOWNLOAD)
                            .on_hover_text("Download")
                            .clicked()
                        {
                            global()
                                .update_tx
                                .send(Update::DownloadObject(data.key().to_string()))
                                .unwrap();
                        }
                    });
                    egui::Frame::none().show(ui, |ui| {
                        ui.set_width(120.);
                        ui.label(data.date_string());
                    });
                    egui::Frame::none().show(ui, |ui| {
                        ui.set_width(60.);
                        ui.label(data.size_string());
                    });
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        ui.checkbox(&mut data.selected, "");
                        ui.vertical(|ui| {
                            ui.add(egui::Label::new(
                                global().cc_ui.text_ellipsis(data.name().as_ref(), 1),
                            ));
                        });
                    });
                })
            });
        });
    }

    response
}
