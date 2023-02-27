use crate::state::State;
use egui_extras::{Column, TableBuilder};

use super::confirm::ConfirmAction;

pub fn auth_history_table(ui: &mut egui::Ui, state: &mut State) {
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    let table = TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto())
        .column(Column::initial(100.0))
        .column(
            Column::initial(100.0)
                .at_least(40.0)
                .resizable(true)
                .clip(true),
        )
        .column(Column::remainder())
        .min_scrolled_height(0.0);

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("ID");
            });
            header.col(|ui| {
                ui.strong("Secret");
            });
            header.col(|ui| {
                ui.strong("Note");
            });
            header.col(|ui| {
                ui.strong("Action");
            });
        })
        .body(|body| {
            let sessions = state.sessions.clone();
            body.rows(text_height, state.sessions.len(), |row_index, mut row| {
                let d = sessions.get(row_index).unwrap();
                row.col(|ui| {
                    ui.label(&d.key_id);
                });
                row.col(|ui| {
                    ui.label(&d.key_secret_mask());
                });
                row.col(|ui| {
                    ui.label(&d.note);
                });
                row.col(|ui| {
                    if ui.button("Use").clicked() {
                        state.session = d.clone();
                    }
                    if ui.button("Remove").clicked() {
                        state.confirm(
                            "Do you confirm to remove this item?",
                            ConfirmAction::RemoveSession(d.clone()),
                        )
                    }
                });
            });
        });
}
