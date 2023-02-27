use crate::state::{NavgatorType, Route, ShowType, State, Status, Update};
use crate::theme::text_ellipsis;
use crate::widgets::{auth_history_table, confirm::ConfirmAction, item_ui, password};
use crate::{SUPPORT_EXTENSIONS, THUMB_LIST_HEIGHT, THUMB_LIST_WIDTH};
use cc_core::{OssObject, OssObjectType};
use chrono::DateTime;
use egui_notify::Toasts;

const SAVE_NOTIF_DURATION: Option<std::time::Duration> = Some(std::time::Duration::from_secs(4));

pub struct App {
    state: State,
    toasts: Toasts,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = State::new(&cc.egui_ctx);
        let mut this = Self {
            state,
            toasts: Toasts::new(),
        };

        if this.state.oss.is_some() {
            this.state.get_list(&cc.egui_ctx);
        }

        this
    }

    fn render_auth(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            // .inner_margin(egui::style::Margin::same(0.0))
            .show(ui, |ui| {
                egui::Grid::new("auth_form_grid")
                    .spacing([10.0; 2])
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Endpoint:");
                        ui.text_edit_singleline(&mut self.state.session.endpoint);
                        ui.end_row();
                        ui.label("AccessKeyId:");
                        ui.text_edit_singleline(&mut self.state.session.key_id);
                        ui.end_row();
                        ui.label("AccessKeySecret:");
                        ui.add(password(&mut self.state.session.key_secret));
                        ui.end_row();
                        ui.label("Bucket:");
                        ui.text_edit_singleline(&mut self.state.session.bucket);
                        ui.end_row();
                        ui.label("Note:");
                        ui.text_edit_singleline(&mut self.state.session.note);
                    });

                ui.add_space(20.0);
                if ui.button("Save").clicked() {
                    match self.state.save_auth(ui.ctx()) {
                        Ok(_) => {
                            self.toasts
                                .success("Success")
                                .set_duration(SAVE_NOTIF_DURATION);
                        }
                        Err(err) => {
                            self.toasts
                                .error(err.to_string())
                                .set_duration(SAVE_NOTIF_DURATION);
                        }
                    }
                }

                ui.separator();

                ui.heading("History");

                auth_history_table(ui, &mut self.state);
            });
    }

    fn render_content(&mut self, ui: &mut egui::Ui) {
        if let Some(err) = &self.state.err {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new(err).color(egui::Color32::RED))
            });
            return;
        }
        if self.state.list.is_empty() {
            ui.centered_and_justified(|ui| ui.heading("Nothing Here."));
            return;
        }
        let num_cols = match self.state.show_type {
            ShowType::List => 1,
            ShowType::Thumb => {
                let w = ui.ctx().input(|i| i.screen_rect().size());
                (w.x / THUMB_LIST_WIDTH) as usize
            }
        };
        let num_rows = match self.state.show_type {
            ShowType::List => self.state.list.len(),
            ShowType::Thumb => (self.state.list.len() as f32 / num_cols as f32).ceil() as usize,
        };
        // tracing::info!("num_rows: {}", num_rows);
        let col_width = match self.state.show_type {
            ShowType::List => 1.0,
            ShowType::Thumb => {
                let w = ui.ctx().input(|i| i.screen_rect().size());
                w.x / (num_cols as f32)
            }
        };
        let row_height = match self.state.show_type {
            ShowType::List => ui.text_style_height(&egui::TextStyle::Body),
            ShowType::Thumb => THUMB_LIST_HEIGHT,
        };

        let mut scroller = egui::ScrollArea::vertical()
            .id_source("scroller_".to_owned() + &row_height.to_string())
            .auto_shrink([false; 2])
            // .enable_scrolling(false)
            // .hscroll(self.show_type == ShowType::List)
            .id_source("content_scroll");

        if self.state.scroll_top {
            self.state.scroll_top = false;
            scroller = scroller.scroll_offset(egui::Vec2::ZERO);
        }

        let (current_scroll, max_scroll) = scroller
            .show_rows(ui, row_height, num_rows, |ui, row_range| {
                // tracing::info!("row_range: {:?}", row_range);
                match self.state.show_type {
                    ShowType::List => self.render_list(ui, row_range),
                    ShowType::Thumb => self.render_thumb(ui, row_range, num_cols, col_width),
                }
                let margin = ui.visuals().clip_rect_margin;
                let current_scroll = ui.clip_rect().top() - ui.min_rect().top() + margin;
                let max_scroll = ui.min_rect().height() - ui.clip_rect().height() + 2.0 * margin;
                (current_scroll, max_scroll)
            })
            .inner;

        // tracing::info!(
        //     "current_scroll: {}, max_scroll: {}",
        //     current_scroll,
        //     max_scroll
        // );

        if self.state.next_query.is_some()
            && current_scroll >= max_scroll
            && !self.state.loading_more
        {
            self.state.loading_more = true;
            self.state.load_more(ui.ctx());
        }
    }

    fn handle_click(&mut self, data: &OssObject, ui: &mut egui::Ui) {
        match data.obj_type {
            OssObjectType::File => {
                self.state.current_img = data.clone();
                self.state.is_preview = true;
                ui.ctx().request_repaint();
            }
            OssObjectType::Folder => {
                self.state
                    .update_tx
                    .send(Update::Navgator(NavgatorType::New(data.path.clone())))
                    .unwrap();
            }
        }
    }

    fn render_list(&mut self, ui: &mut egui::Ui, row_range: std::ops::Range<usize>) {
        for i in row_range {
            if let Some(data) = self.state.list.get(i) {
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
                                    egui::Label::new(text_ellipsis(&data.name(), 1))
                                        .sense(egui::Sense::click()),
                                )
                                .on_hover_text(data.name())
                                .clicked()
                            {
                                self.handle_click(&data, ui);
                            }
                        });
                    });
                });
            }
        }
    }

    fn render_thumb(
        &mut self,
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
                        if let Some(d) = self.state.list.get(j + i * num_cols) {
                            let url = self.state.get_oss_url(&d.path);
                            let resp = item_ui(ui, d.clone(), url.clone(), &mut self.state.images);
                            if resp.on_hover_text(d.name()).clicked() {
                                self.handle_click(&d.clone(), ui);
                            }
                        }
                    }
                    ui.end_row();
                }
            });
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(self.state.navigator.can_go_back(), |ui| {
            if ui.button("\u{2b05}").on_hover_text("Back").clicked() {
                self.state
                    .update_tx
                    .send(Update::Navgator(NavgatorType::Back))
                    .unwrap();
            }
        });
        ui.add_enabled_ui(!self.state.navigator.location().is_empty(), |ui| {
            if ui.button("\u{2b06}").on_hover_text("Go Parent").clicked() {
                let mut parent = String::from("");
                let mut current = self.state.navigator.location();
                if current.ends_with('/') {
                    current.pop();
                }
                if let Some(index) = current.rfind('/') {
                    current.truncate(index);
                    parent = current;
                }
                self.state
                    .update_tx
                    .send(Update::Navgator(NavgatorType::New(parent)))
                    .unwrap();
            }
        });
        ui.add_enabled_ui(self.state.navigator.can_go_forward(), |ui| {
            if ui.button("\u{27a1}").on_hover_text("Forward").clicked() {
                self.state
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
                self.state.picked_path = paths;
            }
        }
        ui.horizontal(|ui| {
            ui.set_width(25.0);
            let enabled = self.state.status != Status::Busy(Route::List)
                && self.state.status != Status::Busy(Route::Upload)
                && !self.state.loading_more;

            ui.add_enabled_ui(enabled, |ui| {
                if ui.button("\u{1f503}").clicked() {
                    self.state.refresh(ui.ctx());
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
                ui.style_mut().visuals.widgets.active.rounding = egui::Rounding::same(2.0);
                if ui
                    .selectable_value(&mut self.state.show_type, ShowType::Thumb, "\u{25a3}")
                    .clicked()
                {
                    self.state.scroll_top = true;
                }
                if ui
                    .selectable_value(&mut self.state.show_type, ShowType::List, "\u{2630}")
                    .clicked()
                {
                    self.state.scroll_top = true;
                }
            });
            self.location_bar(ui);
        });
    }

    fn location_bar(&mut self, ui: &mut egui::Ui) {
        let response = ui.add_sized(
            ui.available_size(),
            egui::TextEdit::singleline(&mut self.state.current_path),
        );
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if self.state.current_path != self.state.navigator.location() {
                self.state
                    .update_tx
                    .send(Update::Navgator(NavgatorType::New(
                        self.state.current_path.clone(),
                    )))
                    .unwrap();
            }
        }
    }

    fn status_bar_contents(&mut self, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_switch(ui);

        if self.state.loading_more {
            ui.add(egui::Spinner::new().size(12.0));
        }

        ui.label(format!("Count: {}", self.state.list.len()));

        if self.state.next_query.is_none() && !self.state.loading_more {
            // ui.label("No More Data.");
        }

        match &mut self.state.status {
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
        let color = if self.state.is_show_result {
            style.hyperlink_color
        } else {
            style.text_color()
        };

        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            if ui.button("\u{1f464}").on_hover_text("Logout").clicked() {
                self.state
                    .confirm("Do you confirm to logout?", ConfirmAction::Logout);
            }
            if ui
                .button(egui::RichText::new("\u{1f4ac}").color(color))
                .clicked()
            {
                self.state.is_show_result = !self.state.is_show_result;
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.init(ctx);
        match &mut self.state.status {
            Status::Idle(ref mut route) => match route {
                Route::Auth => {
                    egui::CentralPanel::default().show(ctx, |ui| self.render_auth(ui));
                    return;
                }
                _ => {}
            },
            _ => {}
        };

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::Frame::none()
                .inner_margin(egui::style::Margin::symmetric(0.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        self.bar_contents(ui);
                    });
                });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                self.status_bar_contents(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::none()
                .inner_margin(egui::style::Margin::same(0.0))
                .show(ui, |ui| {
                    match &mut self.state.status {
                        Status::Idle(ref mut route) => match route {
                            Route::Upload => {}
                            Route::List => self.render_content(ui),
                            _ => {}
                        },
                        Status::Busy(_) => {
                            ui.centered_and_justified(|ui| {
                                ui.spinner();
                            });
                        }
                    };
                });
        });

        self.toasts.show(ctx);
    }
}
