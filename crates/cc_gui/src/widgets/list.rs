use super::item_ui;
use crate::state::{NavgatorType, State, Update};
use crate::theme::text_ellipsis;
use crate::{THUMB_LIST_HEIGHT, THUMB_LIST_WIDTH};
use cc_core::{OssObject, OssObjectType};
use chrono::DateTime;

pub fn list_ui(state: &mut State, ui: &mut egui::Ui, row_range: std::ops::Range<usize>) {
    for i in row_range {
        if let Some(data) = state.list.get(i) {
            let data = data.clone();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                egui::Frame::none().show(ui, |ui| {
                    ui.set_width(120.);
                    ui.label(if data.last_modified.is_empty() {
                        "-".into()
                    } else {
                        match DateTime::parse_from_rfc3339(&data.last_modified) {
                            Ok(date) => date.format("%Y-%m-%d %H:%M:%S").to_string(),
                            Err(_) => "_".into(),
                        }
                    });
                });
                egui::Frame::none().show(ui, |ui| {
                    ui.set_width(60.);
                    ui.label(if data.size.eq(&0) {
                        "Folder".into()
                    } else {
                        data.size_string()
                    });
                });
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.vertical(|ui| {
                        if ui
                            .add(
                                egui::Label::new(text_ellipsis(ui, &data.name(), 1))
                                    .sense(egui::Sense::click()),
                            )
                            .on_hover_text(data.name())
                            .clicked()
                        {
                            handle_click(&data, ui, state);
                        }
                    });
                });
            });
        }
    }
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
                        let url = state.get_oss_url(&d.path);
                        let resp = item_ui(ui, d.clone(), url.clone(), &mut state.images);
                        if resp.on_hover_text(d.name()).clicked() {
                            handle_click(&d.clone(), ui, state);
                        }
                    }
                }
                ui.end_row();
            }
        });
}

fn handle_click(data: &OssObject, ui: &mut egui::Ui, state: &mut State) {
    match data.obj_type {
        OssObjectType::File => {
            state.current_img = data.clone();
            state.is_preview = true;
            ui.ctx().request_repaint();
        }
        OssObjectType::Folder => {
            state
                .update_tx
                .send(Update::Navgator(NavgatorType::New(data.path.clone())))
                .unwrap();
        }
    }
}
