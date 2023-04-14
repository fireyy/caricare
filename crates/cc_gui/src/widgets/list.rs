use super::{list_item_ui, thumb_item_ui};
use crate::global;
use crate::state::{NavgatorType, State, Update};
use crate::{THUMB_LIST_HEIGHT, THUMB_LIST_WIDTH};
use cc_storage::{Object, ObjectType};

pub fn list_ui(state: &mut State, ui: &mut egui::Ui, row_range: std::ops::Range<usize>) {
    egui::Grid::new("list".to_string())
        .num_columns(1)
        .striped(true)
        .show(ui, |ui| {
            for data in state.list[row_range].iter_mut() {
                let response = list_item_ui(ui, data);
                if response.on_hover_text(data.name()).clicked() {
                    handle_click(data);
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
) {
    egui::Grid::new("grid".to_string())
        .num_columns(num_cols)
        .max_col_width(THUMB_LIST_WIDTH - 9.0)
        .min_col_width(THUMB_LIST_WIDTH - 9.0)
        .min_row_height(THUMB_LIST_HEIGHT)
        .spacing(egui::Vec2::new(9.0, 0.0))
        .start_row(row_range.start)
        .show(ui, |ui| {
            for i in row_range {
                for j in 0..num_cols {
                    if let Some(data) = state.list.get_mut(j + i * num_cols) {
                        egui::Frame::none().show(ui, |ui| {
                            let response = thumb_item_ui(ui, data);
                            if response.on_hover_text(data.name()).clicked() {
                                handle_click(data);
                            }
                        });
                    }
                }
                ui.end_row();
            }
        });
}

fn handle_click(data: &Object) {
    match data.obj_type() {
        ObjectType::File => {
            global()
                .update_tx
                .send(Update::ViewObject(data.clone()))
                .unwrap();
        }
        ObjectType::Folder => {
            global()
                .update_tx
                .send(Update::Navgator(NavgatorType::New(data.key().to_string())))
                .unwrap();
        }
    };
}
