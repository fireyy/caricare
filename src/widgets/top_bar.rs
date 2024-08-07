use super::confirm::ConfirmAction;
use super::location_bar_ui;
use crate::global;
use crate::state::{FileAction, NavgatorType, Route, State, Status, Update};
use cc_core::ShowType;
use cc_storage::util::get_name_form_path;
use cc_ui::icon;

pub fn top_bar_ui(ctx: &egui::Context, state: &mut State, _frame: &mut eframe::Frame) {
    let top_bar_style = global().cc_ui.top_bar_style(false);

    egui::TopBottomPanel::top("top_bar")
        .frame(global().cc_ui.top_panel_frame())
        .exact_height(top_bar_style.height)
        .show(ctx, |ui| {
            egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(0.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.add_space(top_bar_style.indent);
                        // Go to Back button
                        ui.add_enabled_ui(state.navigator.can_go_back(), |ui| {
                            if ui.button(icon::BACK).on_hover_text("Back").clicked() {
                                global()
                                    .update_tx
                                    .send(Update::Navgator(NavgatorType::Back))
                                    .unwrap();
                            }
                        });
                        // Go to Parent button
                        ui.add_enabled_ui(!state.navigator.location().is_empty(), |ui| {
                            if ui.button(icon::TOP).on_hover_text("Go to Parent").clicked() {
                                let mut parent = String::from("");
                                let mut current = state.navigator.location();
                                if current.ends_with('/') {
                                    current.pop();
                                }
                                if let Some(index) = current.rfind('/') {
                                    current.truncate(index);
                                    parent = current;
                                }
                                global()
                                    .update_tx
                                    .send(Update::Navgator(NavgatorType::New(parent)))
                                    .unwrap();
                            }
                        });
                        // Go Forward button
                        ui.add_enabled_ui(state.navigator.can_go_forward(), |ui| {
                            if ui.button(icon::FORWARD).on_hover_text("Forward").clicked() {
                                global()
                                    .update_tx
                                    .send(Update::Navgator(NavgatorType::Forward))
                                    .unwrap();
                            }
                        });
                        // Go to Home button
                        ui.add_enabled_ui(!state.navigator.location().is_empty(), |ui| {
                            if ui.button(icon::HOME).on_hover_text("Home").clicked() {
                                global()
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
                                if ui.button(icon::REFRESH).on_hover_text("Refresh").clicked() {
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
                                        egui::RichText::new(icon::THUMB).color(
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
                                        egui::RichText::new(icon::LIST).color(
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
                        if ui.button(format!("{} Upload", icon::UPLOAD)).clicked() {
                            if let Some(paths) = rfd::FileDialog::new().pick_files() {
                                state.picked_path = paths;
                            }
                        }
                        // create folder button
                        if ui
                            .button(format!("{} Create Folder", icon::CREATE_FOLDER))
                            .clicked()
                        {
                            global()
                                .update_tx
                                .send(Update::Prompt((
                                    "Please enter the folder name:".to_string(),
                                    ConfirmAction::CreateFolder("".into()),
                                )))
                                .unwrap();
                        }
                        ui.separator();
                        ui.add_enabled_ui(
                            state.selected_item == 1 && state.file_action.is_none(),
                            |ui| {
                                if ui.button(format!("{} Copy", icon::COPY)).clicked() {
                                    if let Some(obj) = state.list.iter().find(|x| x.selected) {
                                        state.file_action =
                                            Some(FileAction::Copy(obj.key().to_string()));
                                    }
                                }
                                if ui.button(format!("{} Move", icon::MOVE)).clicked() {
                                    if let Some(obj) = state.list.iter().find(|x| x.selected) {
                                        state.file_action =
                                            Some(FileAction::Move(obj.key().to_string()));
                                    }
                                }
                                if ui.button(format!("{} Rename", icon::RENAME)).clicked() {
                                    if let Some(obj) = state.list.iter().find(|x| x.selected) {
                                        global()
                                            .update_tx
                                            .send(Update::Prompt((
                                                "Please enter a new file name:".to_string(),
                                                ConfirmAction::RenameObject((
                                                    obj.key().to_string(),
                                                    "".into(),
                                                )),
                                            )))
                                            .unwrap();
                                    }
                                }
                            },
                        );
                        ui.add_enabled_ui(
                            state.selected_item > 0 && state.file_action.is_none(),
                            |ui| {
                                if ui.button(format!("{} Delete", icon::DELETE)).clicked() {
                                    global()
                                        .update_tx
                                        .send(Update::Confirm((
                                            "Do you confirm to delete selected items?".to_string(),
                                            ConfirmAction::RemoveFiles,
                                        )))
                                        .unwrap();
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
                                            .fill(global().cc_ui.design_tokens.selection_color),
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
                                                    get_name_form_path(src)
                                                );
                                                state.copy_object(src.to_string(), dest, false);
                                            }
                                            FileAction::Move(src) => {
                                                let dest = format!(
                                                    "{}{}",
                                                    state.current_path,
                                                    get_name_form_path(src)
                                                );
                                                state.copy_object(src.to_string(), dest, true);
                                            }
                                        }
                                    }
                                }
                                if ui.button(icon::CLOSE).on_hover_text("Cancel").clicked() {
                                    state.file_action = None;
                                }
                            });
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            let response = ui.add_sized(
                                ui.available_size() - [20.0, 0.0].into(),
                                egui::TextEdit::singleline(&mut state.filter_str)
                                    .hint_text("Filter with file name")
                                    .lock_focus(false),
                            );

                            if response.changed() {
                                response.request_focus();
                                state.filter();
                            }

                            ui.label(icon::FILTER);
                        });
                    });
                });
        });
}
