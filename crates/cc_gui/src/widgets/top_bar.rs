use super::location_bar_ui;
use crate::state::{NavgatorType, Route, State, Status, Update};
use crate::SUPPORT_EXTENSIONS;
use cc_core::ShowType;

pub fn top_bar_ui(ctx: &egui::Context, state: &mut State) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        egui::Frame::none()
            .inner_margin(egui::style::Margin::symmetric(0.0, 5.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.add_enabled_ui(state.navigator.can_go_back(), |ui| {
                        if ui.button("\u{2b05}").on_hover_text("Back").clicked() {
                            state
                                .update_tx
                                .send(Update::Navgator(NavgatorType::Back))
                                .unwrap();
                        }
                    });
                    ui.add_enabled_ui(!state.navigator.location().is_empty(), |ui| {
                        if ui.button("\u{2b06}").on_hover_text("Go Parent").clicked() {
                            let mut parent = String::from("");
                            let mut current = state.navigator.location();
                            if current.ends_with('/') {
                                current.pop();
                            }
                            if let Some(index) = current.rfind('/') {
                                current.truncate(index);
                                parent = current;
                            }
                            state
                                .update_tx
                                .send(Update::Navgator(NavgatorType::New(parent)))
                                .unwrap();
                        }
                    });
                    ui.add_enabled_ui(state.navigator.can_go_forward(), |ui| {
                        if ui.button("\u{27a1}").on_hover_text("Forward").clicked() {
                            state
                                .update_tx
                                .send(Update::Navgator(NavgatorType::Forward))
                                .unwrap();
                        }
                    });

                    if ui
                        .button("\u{2795}")
                        .on_hover_text("Upload file...")
                        .clicked()
                    {
                        if let Some(paths) = rfd::FileDialog::new()
                            .add_filter("image", &SUPPORT_EXTENSIONS)
                            .pick_files()
                        {
                            state.picked_path = paths;
                        }
                    }
                    ui.horizontal(|ui| {
                        ui.set_width(25.0);
                        let enabled = state.status != Status::Busy(Route::List)
                            && state.status != Status::Busy(Route::Upload)
                            && !state.loading_more;

                        ui.add_enabled_ui(enabled, |ui| {
                            if ui.button("\u{1f503}").clicked() {
                                state.refresh(ui.ctx());
                            }
                        });
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        egui::Frame {
                            fill: ui.style().visuals.widgets.inactive.bg_fill,
                            rounding: egui::Rounding::same(2.0),
                            ..egui::Frame::default()
                        }
                        .show(ui, |ui| {
                            ui.style_mut().spacing.item_spacing.x = 0.0;
                            ui.style_mut().visuals.button_frame = false;
                            ui.style_mut().visuals.widgets.active.rounding =
                                egui::Rounding::same(2.0);
                            if ui
                                .selectable_value(
                                    &mut state.setting.show_type,
                                    ShowType::Thumb,
                                    "\u{25a3}",
                                )
                                .clicked()
                            {
                                state.scroll_top = true;
                            }
                            if ui
                                .selectable_value(
                                    &mut state.setting.show_type,
                                    ShowType::List,
                                    "\u{2630}",
                                )
                                .clicked()
                            {
                                state.scroll_top = true;
                            }
                        });
                        location_bar_ui(ui, state);
                    });
                });
            });
    });
}
