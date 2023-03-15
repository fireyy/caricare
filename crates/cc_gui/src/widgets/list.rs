use super::item_ui;
use crate::state::{NavgatorType, State, Update};
use crate::widgets::confirm::ConfirmAction;
use crate::{THUMB_LIST_HEIGHT, THUMB_LIST_WIDTH};
use oss_sdk::ObjectType;

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
    egui::Grid::new(format!("list"))
        .num_columns(1)
        .striped(true)
        .show(ui, |ui| {
            for data in state.list[row_range].iter_mut() {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    egui::Frame::none().show(ui, |ui| {
                        ui.set_width(60.);
                        if ui.button("\u{1f5d1}").on_hover_text("Delete").clicked() {
                            state.confirm.show(
                                format!("Do you confirm to delete this item: {}?", data.key()),
                                ConfirmAction::RemoveFile(data.clone()),
                            );
                        }
                        //TODO：download file
                        // if ui.button("\u{1f4e9}").on_hover_text("Download").clicked() {
                        //     //
                        // }
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
                            if ui
                                .add(
                                    egui::Label::new(
                                        state.cc_ui.text_ellipsis(&data.name().as_ref(), 1),
                                    )
                                    .sense(egui::Sense::click()),
                                )
                                .on_hover_text(data.name())
                                .clicked()
                            {
                                handle_click!(state, data);
                            }
                        });
                    });
                });
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
    egui::Grid::new(format!("grid"))
        .num_columns(num_cols)
        .max_col_width(col_width - 9.0)
        .min_col_width(THUMB_LIST_WIDTH - 9.0)
        .min_row_height(THUMB_LIST_HEIGHT)
        .spacing(egui::Vec2::new(9.0, 0.0))
        .start_row(row_range.start)
        .show(ui, |ui| {
            for i in row_range {
                for j in 0..num_cols {
                    if let Some(d) = state.list.get(j + i * num_cols) {
                        let url = state.get_thumb_url(&d.key(), 64);
                        let data = d.clone();
                        let resp = item_ui(ui, &data, url.clone(), &mut state.images);
                        if resp.on_hover_text(d.name()).clicked() {
                            handle_click!(state, data);
                        }
                    }
                }
                ui.end_row();
            }
        });
}
