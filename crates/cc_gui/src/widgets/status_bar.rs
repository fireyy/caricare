use super::confirm::ConfirmAction;
use crate::state::{Route, State, Status};

pub fn status_bar_ui(ctx: &egui::Context, state: &mut State, _frame: &mut eframe::Frame) {
    let frame = egui::Frame {
        fill: state.cc_ui.design_tokens.bottom_bar_color,
        inner_margin: egui::Vec2::splat(3.0).into(),
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

                ui.label(format!("Count: {}", state.list.len()));

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
                let color = if state.is_show_result {
                    style.hyperlink_color
                } else {
                    style.text_color()
                };

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("\u{1f464}").on_hover_text("Logout").clicked() {
                        state.confirm("Do you confirm to logout?", ConfirmAction::Logout);
                    }
                    if ui
                        .button(egui::RichText::new("\u{1f4ac}").color(color))
                        .on_hover_text("Logs")
                        .clicked()
                    {
                        state.is_show_result = !state.is_show_result;
                    }
                });
            });
        });
}
