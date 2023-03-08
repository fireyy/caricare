use crate::state::State;
use crate::widgets::{confirm::ConfirmAction, password};
use egui_extras::{Column, TableBuilder};

const SAVE_NOTIF_DURATION: Option<std::time::Duration> = Some(std::time::Duration::from_secs(4));

pub fn auth_page(ctx: &egui::Context, state: &mut State) {
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::Frame::none()
            .inner_margin(egui::style::Margin::symmetric(0.0, 20.0))
            .show(ui, |ui| {
                egui::Grid::new("auth_form_grid")
                    .spacing([10.0; 2])
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Endpoint:");
                        ui.text_edit_singleline(&mut state.session.endpoint);
                        ui.end_row();
                        ui.label("AccessKeyId:");
                        ui.text_edit_singleline(&mut state.session.key_id);
                        ui.end_row();
                        ui.label("AccessKeySecret:");
                        ui.add(password(&mut state.session.key_secret));
                        ui.end_row();
                        ui.label("Bucket:");
                        ui.text_edit_singleline(&mut state.session.bucket);
                        ui.end_row();
                        ui.label("Note:");
                        ui.text_edit_singleline(&mut state.session.note);
                    });

                ui.add_space(20.0);

                if ui.button("Login").clicked() {
                    match state.login(ui.ctx()) {
                        Ok(_) => {
                            state
                                .toasts
                                .success("Success")
                                .set_duration(SAVE_NOTIF_DURATION);
                        }
                        Err(err) => {
                            state
                                .toasts
                                .error(err.to_string())
                                .set_duration(SAVE_NOTIF_DURATION);
                        }
                    }
                }

                ui.separator();

                ui.heading("History");

                // auth_history_table(ui, &mut state);
                let text_height = crate::theme::CCUi::table_line_height();

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
                    .header(crate::theme::CCUi::table_header_height(), |mut header| {
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
                                    state.confirm.show(
                                        "Do you confirm to remove this item?",
                                        ConfirmAction::RemoveSession(d.clone()),
                                    )
                                }
                            });
                        });
                    });
            });
    });
}
