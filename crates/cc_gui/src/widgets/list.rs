use super::item_ui;
use crate::state::{NavgatorType, State, Update};
use crate::theme::icon;
use crate::widgets::confirm::ConfirmAction;
use crate::{THUMB_LIST_HEIGHT, THUMB_LIST_WIDTH};
use cc_storage::ObjectType;
use egui::{self, style::Margin, vec2, Color32, Frame, Sense, WidgetInfo, WidgetType};

macro_rules! handle_click {
    ($state:ident, $data:ident) => {{
        match $data.obj_type() {
            ObjectType::File => {
                $state
                    .update_tx
                    .send(Update::ViewObject($data.clone()))
                    .unwrap();
            }
            ObjectType::Folder => {
                $state
                    .update_tx
                    .send(Update::Navgator(NavgatorType::New($data.key().to_string())))
                    .unwrap();
            }
        };
    }};
}

pub fn list_ui(state: &mut State, ui: &mut egui::Ui, row_range: std::ops::Range<usize>) {
    egui::Grid::new("list".to_string())
        .num_columns(1)
        .striped(true)
        .show(ui, |ui| {
            for data in state.list[row_range].iter_mut() {
                let row_height = ui.text_style_height(&egui::TextStyle::Body);
                let initial_size = vec2(
                    ui.available_width(),
                    row_height, // Assume there will be
                );
                let (rect, response) = ui.allocate_exact_size(initial_size, Sense::click());
                response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ""));

                if ui.is_rect_visible(rect) {
                    let visuals = ui.style().interact(&response);
                    let mut fill_color = if response.hovered() {
                        visuals.bg_fill
                    } else {
                        Color32::TRANSPARENT
                    };
                    if data.selected {
                        fill_color = state.cc_ui.design_tokens.selection_color;
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
                                        state.confirm.show(
                                            format!(
                                                "Do you confirm to delete this item: {}?",
                                                data.key()
                                            ),
                                            ConfirmAction::RemoveFile(data.clone()),
                                        );
                                    }
                                    // download file
                                    if ui
                                        .button(icon::DOWNLOAD)
                                        .on_hover_text("Download")
                                        .clicked()
                                    {
                                        state
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
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::TOP),
                                    |ui| {
                                        ui.checkbox(&mut data.selected, "");
                                        ui.vertical(|ui| {
                                            ui.add(
                                                egui::Label::new(
                                                    state
                                                        .cc_ui
                                                        .text_ellipsis(data.name().as_ref(), 1),
                                                )
                                                .sense(egui::Sense::click()),
                                            );
                                        });
                                    },
                                );
                            })
                        });
                    });
                }
                if response.on_hover_text(data.name()).clicked() {
                    handle_click!(state, data);
                }
                ui.end_row();
            }
        });
}

pub fn thumb_ui(
    state: &mut State,
    ui: &mut egui::Ui,
    row_range: std::ops::Range<usize>,
    num_cols: usize,
    col_width: f32,
) {
    egui::Grid::new("grid".to_string())
        .num_columns(num_cols)
        .max_col_width(col_width - 9.0)
        .min_col_width(THUMB_LIST_WIDTH - 9.0)
        .min_row_height(THUMB_LIST_HEIGHT)
        .spacing(egui::Vec2::new(9.0, 0.0))
        .start_row(row_range.start)
        .show(ui, |ui| {
            for i in row_range {
                for j in 0..num_cols {
                    if let Some(d) = state.list.get_mut(j + i * num_cols) {
                        let data = d.clone();
                        let response = item_ui(ui, d);
                        if response.on_hover_text(d.name()).clicked() {
                            handle_click!(state, data);
                        }
                    }
                }
                ui.end_row();
            }
        });
}
