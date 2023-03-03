use super::location_bar_ui;
use crate::state::{NavgatorType, Route, State, Status, Update};
use crate::SUPPORT_EXTENSIONS;
use cc_core::log::LogItem;
use cc_core::ShowType;

pub fn top_bar_ui(ctx: &egui::Context, state: &mut State, frame: &mut eframe::Frame) {
    let native_pixels_per_point = frame.info().native_pixels_per_point;
    let fullscreen = {
        #[cfg(target_arch = "wasm32")]
        {
            false
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            frame.info().window_info.fullscreen
        }
    };
    let top_bar_style = state
        .cc_ui
        .top_bar_style(native_pixels_per_point, fullscreen);

    egui::TopBottomPanel::top("top_bar")
        .frame(state.cc_ui.top_panel_frame())
        .exact_height(top_bar_style.height)
        .show(ctx, |ui| {
            egui::Frame::none()
                .inner_margin(egui::style::Margin::symmetric(0.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.add_space(top_bar_style.indent);
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
                            egui::Frame::none().show(ui, |ui| {
                                ui.style_mut().visuals.selection.bg_fill =
                                    egui::Color32::TRANSPARENT;
                                let active_color = ui.visuals().widgets.inactive.fg_stroke.color;
                                let normal_color = egui::Color32::from_gray(100);
                                let show_type = state.setting.show_type;

                                if ui
                                    .selectable_value(
                                        &mut state.setting.show_type,
                                        ShowType::Thumb,
                                        egui::RichText::new("\u{25a3}").color(
                                            if show_type == ShowType::Thumb {
                                                active_color
                                            } else {
                                                normal_color
                                            },
                                        ),
                                    )
                                    .clicked()
                                {
                                    state.scroll_top = true;
                                    state
                                        .logs
                                        .push(LogItem::unknow().with_info("dooooooo".into()));
                                }
                                if ui
                                    .selectable_value(
                                        &mut state.setting.show_type,
                                        ShowType::List,
                                        egui::RichText::new("\u{2630}").color(
                                            if show_type == ShowType::List {
                                                active_color
                                            } else {
                                                normal_color
                                            },
                                        ),
                                    )
                                    .clicked()
                                {
                                    state.scroll_top = true;
                                    state.logs.push(
                                        LogItem::unknow().with_success("fffffffffffff".into()),
                                    );
                                }
                            });
                            location_bar_ui(ui, state);
                        });
                    });
                });
        });
}
