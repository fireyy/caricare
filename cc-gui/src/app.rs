use crate::theme::text_ellipsis;
use crate::widgets::item_ui;
use crate::{SUPPORT_EXTENSIONS, THUMB_LIST_HEIGHT, THUMB_LIST_WIDTH};
use cc_core::{
    tokio, tracing, util::get_extension, ImageCache, ImageFetcher, MemoryHistory, OssBucket,
    OssClient, OssError, OssObject, OssObjectType, Query, UploadResult,
};
use chrono::DateTime;
use std::{path::PathBuf, sync::mpsc, vec};

enum NavgatorType {
    Back,
    Forward,
    New(String),
}

enum Update {
    Uploaded(Result<Vec<UploadResult>, OssError>),
    List(Result<OssBucket, OssError>),
    Navgator(NavgatorType),
}

#[derive(Clone, Copy, PartialEq)]
enum ShowType {
    List,
    Thumb,
}

#[derive(PartialEq)]
enum State {
    Idle(Route),
    Busy(Route),
}

#[derive(Clone, Copy, PartialEq)]
enum Route {
    Upload,
    List,
}

pub struct App {
    oss: OssClient,
    list: Vec<OssObject>,
    current_img: OssObject,
    update_tx: mpsc::SyncSender<Update>,
    update_rx: mpsc::Receiver<Update>,
    state: State,
    err: Option<String>,
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Vec<PathBuf>,
    show_type: ShowType,
    is_preview: bool,
    loading_more: bool,
    next_query: Option<Query>,
    scroll_top: bool,
    images: ImageCache,
    upload_result: Vec<UploadResult>,
    is_show_result: bool,
    current_path: String,
    navigator: MemoryHistory,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let oss = OssClient::new().expect("env variable not found");
        let (update_tx, update_rx) = mpsc::sync_channel(1);

        let current_path = oss.get_path().to_string();
        let navigator = MemoryHistory::new();

        let images = ImageCache::new(ImageFetcher::spawn(cc.egui_ctx.clone()));

        let mut this = Self {
            oss,
            current_img: OssObject::default(),
            list: vec![],
            update_tx,
            update_rx,
            state: State::Idle(Route::List),
            err: None,
            dropped_files: vec![],
            picked_path: vec![],
            show_type: ShowType::List,
            is_preview: false,
            loading_more: false,
            next_query: Some(build_query(current_path.clone())),
            scroll_top: false,
            images,
            upload_result: vec![],
            is_show_result: false,
            current_path: current_path.clone(),
            navigator,
        };

        this.get_list(&cc.egui_ctx);
        this.navigator.push(current_path);

        this
    }

    fn upload_file(&mut self, ctx: &egui::Context) {
        if self.picked_path.is_empty() {
            return;
        }
        let picked_path = self.picked_path.clone();
        self.picked_path = vec![];
        self.state = State::Busy(Route::Upload);

        let update_tx = self.update_tx.clone();
        let ctx = ctx.clone();
        let oss = self.oss.clone();

        cc_core::runtime::spawn(async move {
            tokio::spawn(async move {
                let res = oss.put_multi(picked_path).await;
                update_tx.send(Update::Uploaded(res)).unwrap();
                ctx.request_repaint();
            });
        });
    }

    fn get_list(&mut self, ctx: &egui::Context) {
        if let Some(query) = &self.next_query {
            if !self.loading_more {
                self.state = State::Busy(Route::List);
            }

            let update_tx = self.update_tx.clone();
            let ctx = ctx.clone();
            let query = query.clone();
            let oss = self.oss.clone();

            cc_core::runtime::spawn(async move {
                tokio::spawn(async move {
                    let res = oss.get_list(query).await;
                    update_tx.send(Update::List(res)).unwrap();
                    ctx.request_repaint();
                });
            });
        }
    }

    fn set_list(&mut self, obj: OssBucket) {
        let mut dirs = obj.common_prefixes;
        let mut files: Vec<OssObject> = obj
            .files
            .into_iter()
            .filter(|x| !x.path.ends_with('/'))
            .collect();
        self.list.append(&mut dirs);
        self.list.append(&mut files);
    }

    fn load_more(&mut self, ctx: &egui::Context) {
        tracing::info!("load more!");
        if self.next_query.is_some() {
            self.get_list(ctx);
        } else {
            //no more
        }
    }

    fn refresh(&mut self, ctx: &egui::Context) {
        self.err = None;
        self.scroll_top = true;
        let current_path = self.navigator.location();
        self.next_query = Some(build_query(current_path));
        self.list = vec![];
        self.get_list(ctx);
    }

    fn render_content(&mut self, ui: &mut egui::Ui) {
        if let Some(err) = &self.err {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new(err).color(egui::Color32::RED))
            });
            return;
        }
        if self.list.is_empty() {
            ui.centered_and_justified(|ui| ui.heading("Nothing Here."));
            return;
        }
        let num_cols = match self.show_type {
            ShowType::List => 1,
            ShowType::Thumb => {
                let w = ui.ctx().input().screen_rect().size();
                (w.x / THUMB_LIST_WIDTH) as usize
            }
        };
        let num_rows = match self.show_type {
            ShowType::List => self.list.len(),
            ShowType::Thumb => (self.list.len() as f32 / num_cols as f32).ceil() as usize,
        };
        // tracing::info!("num_rows: {}", num_rows);
        let col_width = match self.show_type {
            ShowType::List => 1.0,
            ShowType::Thumb => {
                let w = ui.ctx().input().screen_rect().size();
                w.x / (num_cols as f32)
            }
        };
        let row_height = match self.show_type {
            ShowType::List => ui.text_style_height(&egui::TextStyle::Body),
            ShowType::Thumb => THUMB_LIST_HEIGHT,
        };

        let mut scroller = egui::ScrollArea::vertical()
            .id_source("scroller_".to_owned() + &row_height.to_string())
            .auto_shrink([false; 2])
            // .enable_scrolling(false)
            // .hscroll(self.show_type == ShowType::List)
            .id_source("content_scroll");

        if self.scroll_top {
            self.scroll_top = false;
            scroller = scroller.scroll_offset(egui::Vec2::ZERO);
        }

        let (current_scroll, max_scroll) = scroller
            .show_rows(ui, row_height, num_rows, |ui, row_range| {
                // tracing::info!("row_range: {:?}", row_range);
                match self.show_type {
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

        if self.next_query.is_some() && current_scroll >= max_scroll && !self.loading_more {
            self.loading_more = true;
            self.load_more(ui.ctx());
        }
    }

    fn handle_click(&mut self, data: &OssObject, ui: &mut egui::Ui) {
        match data.obj_type {
            OssObjectType::File => {
                self.current_img = data.clone();
                self.is_preview = true;
                ui.ctx().request_repaint();
            }
            OssObjectType::Folder => {
                self.update_tx
                    .send(Update::Navgator(NavgatorType::New(data.path.clone())))
                    .unwrap();
            }
        }
    }

    fn render_list(&mut self, ui: &mut egui::Ui, row_range: std::ops::Range<usize>) {
        for i in row_range {
            if let Some(data) = self.list.get(i) {
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
                        if let Some(d) = self.list.get(j + i * num_cols) {
                            let url = self.oss.get_file_url(&d.path);
                            let resp = item_ui(ui, d.clone(), url.clone(), &mut self.images);
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
        ui.add_enabled_ui(self.navigator.can_go_back(), |ui| {
            if ui.button("\u{2b05}").on_hover_text("Back").clicked() {
                self.update_tx
                    .send(Update::Navgator(NavgatorType::Back))
                    .unwrap();
            }
        });
        ui.add_enabled_ui(!self.navigator.location().is_empty(), |ui| {
            if ui.button("\u{2b06}").on_hover_text("Go Parent").clicked() {
                let mut parent = String::from("");
                let mut current = self.navigator.location();
                if current.ends_with('/') {
                    current.pop();
                }
                if let Some(index) = current.rfind('/') {
                    current.truncate(index);
                    parent = current;
                }
                self.update_tx
                    .send(Update::Navgator(NavgatorType::New(parent)))
                    .unwrap();
            }
        });
        ui.add_enabled_ui(self.navigator.can_go_forward(), |ui| {
            if ui.button("\u{27a1}").on_hover_text("Forward").clicked() {
                self.update_tx
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
                self.picked_path = paths;
            }
        }
        ui.horizontal(|ui| {
            ui.set_width(25.0);
            let enabled = self.state != State::Busy(Route::List)
                && self.state != State::Busy(Route::Upload)
                && !self.loading_more;

            ui.add_enabled_ui(enabled, |ui| {
                if ui.button("\u{1f503}").clicked() {
                    self.refresh(ui.ctx());
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
                    .selectable_value(&mut self.show_type, ShowType::Thumb, "\u{25a3}")
                    .clicked()
                {
                    self.scroll_top = true;
                }
                if ui
                    .selectable_value(&mut self.show_type, ShowType::List, "\u{2630}")
                    .clicked()
                {
                    self.scroll_top = true;
                }
            });
            self.location_bar(ui);
        });
    }

    fn location_bar(&mut self, ui: &mut egui::Ui) {
        let response = ui.add_sized(
            ui.available_size(),
            egui::TextEdit::singleline(&mut self.current_path),
        );
        if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
            if self.current_path != self.navigator.location() {
                self.update_tx
                    .send(Update::Navgator(NavgatorType::New(
                        self.current_path.clone(),
                    )))
                    .unwrap();
            }
        }
    }

    fn status_bar_contents(&mut self, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_switch(ui);

        if self.loading_more {
            ui.add(egui::Spinner::new().size(12.0));
        }

        ui.label(format!("Count: {}", self.list.len()));

        if self.next_query.is_none() && !self.loading_more {
            // ui.label("No More Data.");
        }

        match &mut self.state {
            State::Idle(_) => (),
            State::Busy(route) => match route {
                Route::Upload => {
                    ui.label("Uploading file...");
                }
                Route::List => {
                    ui.label("Getting file list...");
                }
            },
        }

        let style = &ui.style().visuals;
        let color = if self.is_show_result {
            style.hyperlink_color
        } else {
            style.text_color()
        };

        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            if ui
                .button(egui::RichText::new("\u{1f4ac}").color(color))
                .clicked()
            {
                self.is_show_result = !self.is_show_result;
            }
        });
    }

    fn show_image(&mut self, ctx: &egui::Context) {
        let Self {
            mut is_preview,
            current_img,
            ..
        } = self;

        let url = self.oss.get_file_url(&current_img.path);

        if url.is_empty() {
            return;
        }

        if is_preview {
            egui::Area::new("preview_area")
                // .order(egui::Order::Foreground)
                // .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .fixed_pos(egui::Pos2::ZERO)
                .show(ctx, |ui| {
                    let screen_rect = ui.ctx().input().screen_rect;
                    let area_response =
                        ui.allocate_response(screen_rect.size(), egui::Sense::click());
                    if area_response.clicked() {
                        self.is_preview = false;
                    }
                    ui.painter().rect_filled(
                        screen_rect,
                        egui::Rounding::none(),
                        egui::Color32::from_rgba_premultiplied(0, 0, 0, 200),
                    );
                    let win_size = screen_rect.size();
                    let response = egui::Window::new("")
                        .id(egui::Id::new("preview_win"))
                        .open(&mut is_preview)
                        .title_bar(false)
                        .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
                        .resizable(false)
                        .show(&ctx, |ui| {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false; 2])
                                .max_height(win_size.y - 100.0)
                                .show(ui, |ui| {
                                    if let Some(img) = self.images.get(&url) {
                                        let mut size = img.size_vec2();
                                        size *= (ui.available_width() / size.x).min(1.0);
                                        img.show_size(ui, size);
                                    }
                                });
                            ui.vertical_centered_justified(|ui| {
                                let mut url = self.oss.get_file_url(&current_img.path);
                                let resp = ui.add(egui::TextEdit::singleline(&mut url));
                                if resp.on_hover_text("Click to copy").clicked() {
                                    ui.output().copied_text = url;
                                }
                                ui.horizontal(|ui| {
                                    ui.label(format!("size: {}", current_img.size));
                                    ui.label(&current_img.last_modified);
                                });
                            });
                        });
                    if let Some(inner_response) = response {
                        inner_response.response.request_focus();
                        ctx.move_to_top(inner_response.response.layer_id);
                    }
                });
        }
    }
    fn show_result(&mut self, ctx: &egui::Context) {
        if self.is_show_result {
            egui::Area::new("result")
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(ui.style().visuals.extreme_bg_color)
                        .inner_margin(ui.style().spacing.window_margin)
                        .show(ui, |ui| {
                            ui.set_width(400.0);
                            ui.heading("Result");
                            ui.spacing();
                            for path in &self.upload_result {
                                match path {
                                    UploadResult::Success(str) => ui.label(
                                        egui::RichText::new(format!("\u{2714} {str}"))
                                            .color(egui::Color32::GREEN),
                                    ),
                                    UploadResult::Error(str) => ui.label(
                                        egui::RichText::new(format!("\u{2716} {str}"))
                                            .color(egui::Color32::RED),
                                    ),
                                };
                            }
                        });
                });
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.images.poll();
        while let Ok(update) = self.update_rx.try_recv() {
            match update {
                Update::Uploaded(result) => match result {
                    Ok(str) => {
                        self.state = State::Idle(Route::List);
                        self.upload_result = str;
                        self.is_show_result = true;
                    }
                    Err(err) => {
                        self.state = State::Idle(Route::Upload);
                        self.err = Some(err.message());
                    }
                },
                Update::List(result) => match result {
                    Ok(str) => {
                        self.next_query = str.next_query();
                        self.set_list(str);
                        self.loading_more = false;
                        self.state = State::Idle(Route::List);
                    }
                    Err(err) => {
                        self.state = State::Idle(Route::List);
                        self.err = Some(err.message());
                    }
                },
                Update::Navgator(nav) => {
                    match nav {
                        NavgatorType::Back => {
                            if self.navigator.can_go_back() {
                                self.navigator.go(-1);
                            }
                        }
                        NavgatorType::Forward => {
                            if self.navigator.can_go_forward() {
                                self.navigator.go(1);
                            }
                        }
                        NavgatorType::New(path) => {
                            self.navigator.push(path);
                        }
                    }
                    self.current_path = self.navigator.location();
                    self.refresh(ctx);
                }
            }
        }

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
                    match &mut self.state {
                        State::Idle(ref mut route) => match route {
                            Route::Upload => {}
                            Route::List => self.render_content(ui),
                        },
                        State::Busy(_) => {
                            ui.centered_and_justified(|ui| {
                                ui.spinner();
                            });
                        }
                    };
                });
        });

        self.show_image(ctx);
        self.show_result(ctx);

        if !self.dropped_files.is_empty() {
            let mut files = vec![];
            let dropped_files = self.dropped_files.clone();
            self.dropped_files = vec![];
            for file in dropped_files {
                if let Some(path) = &file.path {
                    if SUPPORT_EXTENSIONS.contains(&get_extension(path.clone()).as_str()) {
                        files.push(path.clone());
                    }
                }
            }

            self.picked_path = files;
        }

        self.upload_file(ctx);

        if !ctx.input().raw.dropped_files.is_empty() {
            self.dropped_files = ctx.input().raw.dropped_files.clone();
        }
    }
}

fn build_query(path: String) -> Query {
    let mut path = path.clone();
    if !path.ends_with('/') && !path.is_empty() {
        path.push_str("/");
    }
    let mut query = Query::new();
    query.insert("prefix", path);
    query.insert("delimiter", "/");
    query.insert("max-keys", "40");
    query
}
