use super::confirm::ConfirmAction;
use super::location_bar_ui;
use crate::state::{FileAction, NavgatorType, Route, State, Status, Update};
use cc_core::ShowType;
use oss_sdk::util::get_name_form_path;

pub fn top_bar_ui(ctx: &egui::Context, state: &mut State, frame: &mut eframe::Frame) {
    let native_pixels_per_point = frame.info().native_pixels_per_point;
    let fullscreen = frame.info().window_info.fullscreen;
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
                        // Go to Back button
                        ui.add_enabled_ui(state.navigator.can_go_back(), |ui| {
                            if ui.button("\u{2b05}").on_hover_text("Back").clicked() {
                                state
                                    .update_tx
                                    .send(Update::Navgator(NavgatorType::Back))
                                    .unwrap();
                            }
                        });
                        // Go to Parent button
                        ui.add_enabled_ui(!state.navigator.location().is_empty(), |ui| {
                            if ui
                                .button("\u{2b06}")
                                .on_hover_text("Go to Parent")
                                .clicked()
                            {
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
                        // Go Forward button
                        ui.add_enabled_ui(state.navigator.can_go_forward(), |ui| {
                            if ui.button("\u{27a1}").on_hover_text("Forward").clicked() {
                                state
                                    .update_tx
                                    .send(Update::Navgator(NavgatorType::Forward))
                                    .unwrap();
                            }
                        });
                        // Go to Home button
                        ui.add_enabled_ui(!state.navigator.location().is_empty(), |ui| {
                            if ui.button("\u{1f3e0}").on_hover_text("Home").clicked() {
                                state
                                    .update_tx
                                    .send(Update::Navgator(NavgatorType::New("".into())))
                                    .unwrap();
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.set_width(25.0);
                            let enabled = state.status != Status::Busy(Route::List)
                                && state.status != Status::Busy(Route::Upload)
                                && !state.loading_more;

                            ui.add_enabled_ui(enabled, |ui| {
                                if ui.button("\u{1f503}").on_hover_text("Refresh").clicked() {
                                    state.refresh();
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
                                }
                            });
                            location_bar_ui(ui, state);
                        });
                    });
                    ui.horizontal(|ui| {
                        // upload button
                        if ui.button("\u{1f5c1} Upload").clicked() {
                            if let Some(paths) = rfd::FileDialog::new().pick_files() {
                                state.picked_path = paths;
                            }
                        }
                        // create folder button
                        if ui.button("\u{2795} Create Folder").clicked() {
                            state.confirm.prompt(
                                "Please enter the folder name:",
                                ConfirmAction::CreateFolder("".into()),
                            );
                        }
                        ui.separator();
                        ui.add_enabled_ui(
                            state.selected_item == 1 && state.file_action.is_none(),
                            |ui| {
                                if ui.button("\u{1f5d0} Copy").clicked() {
                                    if let Some(obj) = state.list.iter().find(|x| x.selected) {
                                        state.file_action =
                                            Some(FileAction::Copy(obj.key().to_string()));
                                    }
                                }
                                if ui.button("\u{1f4e5} Move").clicked() {
                                    if let Some(obj) = state.list.iter().find(|x| x.selected) {
                                        state.file_action =
                                            Some(FileAction::Move(obj.key().to_string()));
                                    }
                                }
                                if ui.button("\u{270f} Rename").clicked() {
                                    if let Some(obj) = state.list.iter().find(|x| x.selected) {
                                        state.confirm.prompt(
                                            "Please enter a new file name:",
                                            ConfirmAction::RenameObject((
                                                obj.key().to_string(),
                                                "".into(),
                                            )),
                                        )
                                    }
                                }
                            },
                        );
                        ui.add_enabled_ui(
                            state.selected_item > 0 && state.file_action.is_none(),
                            |ui| {
                                if ui.button("\u{1f5d1} Delete").clicked() {
                                    state.confirm.show(
                                        "Do you confirm to delete selected items?",
                                        ConfirmAction::RemoveFiles,
                                    )
                                }
                            },
                        );
                        ui.add_visible_ui(state.file_action.is_some(), |ui| {
                            let text = match &state.file_action {
                                Some(action) => match action {
                                    FileAction::Copy(_) => "Paste",
                                    FileAction::Move(_) => "Move",
                                },
                                None => "",
                            };
                            ui.horizontal(|ui| {
                                if ui
                                    .add(
                                        egui::Button::new(text)
                                            .fill(state.cc_ui.design_tokens.selection_color),
                                    )
                                    .on_hover_text("Paste to current directory")
                                    .clicked()
                                {
                                    if let Some(action) = &state.file_action {
                                        match action {
                                            FileAction::Copy(src) => {
                                                let dest = format!(
                                                    "{}{}",
                                                    state.current_path,
                                                    get_name_form_path(&src)
                                                );
                                                state.copy_object(src.to_string(), dest, false);
                                            }
                                            FileAction::Move(src) => {
                                                let dest = format!(
                                                    "{}{}",
                                                    state.current_path,
                                                    get_name_form_path(&src)
                                                );
                                                state.copy_object(src.to_string(), dest, true);
                                            }
                                        }
                                    }
                                }
                                if ui.button("\u{274c}").on_hover_text("Cancel").clicked() {
                                    state.file_action = None;
                                }
                            });
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            let response = ui.add_sized(
                                ui.available_size() - [100.0, 0.0].into(),
                                egui::TextEdit::singleline(&mut state.filter_str)
                                    .hint_text("Filter with file name")
                                    .lock_focus(false),
                            );
                            //TODO: use CommandPalette
                            if !state.filter_str.is_empty() {
                                response.request_focus();
                            }

                            if response.changed() {
                                state.filter();
                            }
                        });
                    });
                });
        });
}
