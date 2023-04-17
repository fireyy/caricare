use super::confirm::ConfirmAction;
use crate::global;
use crate::state::{Route, State, Status};
use crate::theme::icon;

pub fn status_bar_ui(ctx: &egui::Context, state: &mut State, _frame: &mut eframe::Frame) {
    let frame = egui::Frame {
        fill: global().cc_ui.design_tokens.bottom_bar_color,
        inner_margin: egui::Vec2::new(10.0, 3.0).into(),
        ..Default::default()
    };
    egui::TopBottomPanel::bottom("status_bar")
        .frame(frame)
        .show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                // egui::widgets::global_dark_light_mode_switch(ui);

                if state.loading_more {
                    ui.add(egui::Spinner::new().size(12.0));
                }

                ui.label(format!(
                    "Selected: {}/{}",
                    state.selected_item,
                    state.list.len()
                ));

                if state.next_query.is_none() && !state.loading_more {
                    // ui.label("No More Data.");
                }

                match &mut state.status {
                    Status::Idle(_) => (),
                    Status::Busy(route) => match route {
                        Route::Upload => {
                            ui.label("Uploading file...");
                        }
                        Route::List => {
                            ui.label("Getting file list...");
                        }
                        _ => {}
                    },
                }

                let style = &ui.style().visuals;
                let n_color = style.text_color();
                let color = if state.is_show_result {
                    style.hyperlink_color
                } else {
                    n_color
                };
                let transfer_color = if state.transfer_manager.is_show {
                    style.hyperlink_color
                } else {
                    n_color
                };

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui
                        .button(egui::RichText::new(icon::USER).color(n_color))
                        .on_hover_text("Logout")
                        .clicked()
                    {
                        state
                            .confirm
                            .show("Do you confirm to logout?", ConfirmAction::Logout);
                    }
                    if ui
                        .button(egui::RichText::new(icon::LOG).color(color))
                        .on_hover_text("Logs")
                        .clicked()
                    {
                        state.is_show_result = !state.is_show_result;
                    }
                    // transfer toggle
                    if ui
                        .button(
                            egui::RichText::new(format!(
                                "{} {}/{} , {} {}/{}",
                                icon::DOWNLOAD,
                                0,
                                0,
                                icon::UPLOAD,
                                0,
                                0
                            ))
                            .color(transfer_color),
                        )
                        .on_hover_text("Transfer")
                        .clicked()
                    {
                        state.transfer_manager.is_show = !state.transfer_manager.is_show;
                    }
                });
            });
        });
}
