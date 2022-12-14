use crate::theme::text_ellipsis;
use crate::widgets::item_ui;
use crate::OssFile;
use bytesize::ByteSize;
use cc_core::{
    tokio, tracing, util::get_extension, GetObjectInfo, ImageCache, ImageFetcher, ObjectList,
    OssConfig, OssError, Query, UploadResult,
};
use std::{path::PathBuf, sync::mpsc, vec};

static THUMB_LIST_WIDTH: f32 = 200.0;
static THUMB_LIST_HEIGHT: f32 = 50.0;
static SUPPORT_EXTENSIONS: [&str; 4] = ["png", "gif", "jpg", "svg"];

enum Update {
    Uploaded(Result<Vec<UploadResult>, OssError>),
    List(Result<ObjectList, OssError>),
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
    oss: OssConfig,
    list: Vec<OssFile>,
    current_img: OssFile,
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
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let oss = OssConfig::new();
        let (update_tx, update_rx) = mpsc::sync_channel(1);

        let query = Self::build_query(oss.path.clone());

        let images = ImageCache::new(ImageFetcher::spawn(cc.egui_ctx.clone()));

        let mut this = Self {
            oss,
            current_img: OssFile::default(),
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
            next_query: Some(query),
            scroll_top: false,
            images,
            upload_result: vec![],
            is_show_result: false,
        };

        this.get_list(&cc.egui_ctx);

        this
    }

    fn build_query(path: String) -> Query {
        let mut query = Query::new();
        query.insert("prefix", path);
        query.insert("max-keys", "40");
        query
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
        if let Some(query) = self.next_query.clone() {
            if !self.loading_more {
                self.state = State::Busy(Route::List);
            }
            let update_tx = self.update_tx.clone();
            let ctx = ctx.clone();
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

    fn set_list(&mut self, obj: ObjectList) {
        let mut list = vec![];
        for data in obj.object_list {
            let (base, last_modified, _etag, _typ, size, _storage_class) = data.pieces();
            let key = base.path().to_string();
            let url = self.oss.get_file_url(key.clone());
            let name = key.replace(&self.oss.path, "").replace("/", "");

            list.push(OssFile {
                name,
                key,
                url,
                size: format!("{}", ByteSize(size)),
                last_modified: last_modified.format("%Y-%m-%d %H:%M:%S").to_string(),
            });
        }
        self.list.append(&mut list);
    }

    fn load_more(&mut self, ctx: &egui::Context) {
        tracing::info!("load more!");
        if self.next_query.is_some() {
            self.get_list(ctx);
        } else {
            //no more
        }
    }

    fn render_content(&mut self, ui: &mut egui::Ui) {
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
                    ShowType::Thumb => self.render_thumb(ui, row_range, num_cols),
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

    fn render_list(&mut self, ui: &mut egui::Ui, row_range: std::ops::Range<usize>) {
        for i in row_range {
            let data = self.list.get(i).unwrap();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                egui::Frame::none().show(ui, |ui| {
                    ui.set_width(120.);
                    ui.label(&data.last_modified);
                });
                egui::Frame::none().show(ui, |ui| {
                    ui.set_width(60.);
                    ui.label(&data.size);
                });
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.vertical(|ui| {
                        if ui
                            .add(
                                egui::Label::new(text_ellipsis(&data.name, 1))
                                    .sense(egui::Sense::click()),
                            )
                            .on_hover_text(&data.url)
                            .clicked()
                        {
                            self.current_img = data.clone();
                            self.is_preview = true;
                            ui.ctx().request_repaint();
                        }
                    });
                });
            });
        }
    }

    fn render_thumb(
        &mut self,
        ui: &mut egui::Ui,
        row_range: std::ops::Range<usize>,
        num_cols: usize,
    ) {
        egui::Grid::new(format!("grid"))
            .num_columns(num_cols)
            .max_col_width(THUMB_LIST_WIDTH)
            .min_col_width(THUMB_LIST_WIDTH)
            .min_row_height(THUMB_LIST_HEIGHT)
            .spacing(egui::Vec2::ZERO)
            .start_row(row_range.start)
            .show(ui, |ui| {
                for i in row_range {
                    for j in 0..num_cols {
                        if let Some(d) = self.list.get(j + i * num_cols) {
                            let url = d.url.clone();
                            let resp = item_ui(ui, d.clone(), &mut self.images);
                            if resp.on_hover_text(url).clicked() {
                                self.current_img = d.clone();
                                self.is_preview = true;
                                ui.ctx().request_repaint();
                            }
                        }
                    }
                    ui.end_row();
                }
            });
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui) {
        if ui.button("Upload file...").clicked() {
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
                    self.scroll_top = true;
                    let query = Self::build_query(self.oss.path.clone());
                    self.next_query = Some(query);
                    self.list = vec![];
                    self.get_list(ui.ctx());
                }
            });
        });
        ui.label(format!("List({})", self.list.len()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            ui.style_mut().spacing.item_spacing.x = 0.0;
            egui::Frame {
                fill: ui.style().visuals.widgets.inactive.bg_fill,
                rounding: egui::Rounding::same(2.0),
                ..egui::Frame::default()
            }
            .show(ui, |ui| {
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
        });
    }
    fn status_bar_contents(&mut self, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_switch(ui);

        if self.loading_more {
            ui.add(egui::Spinner::new().size(12.0));
        }

        if self.next_query.is_none() && !self.loading_more {
            ui.label("No More Data.");
        }

        if let Some(err) = &self.err {
            ui.label(egui::RichText::new(err).color(egui::Color32::RED));
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

        if current_img.url.is_empty() {
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
                                    if let Some(img) = self.images.get(&current_img.url) {
                                        let mut size = img.size_vec2();
                                        size *= (ui.available_width() / size.x).min(1.0);
                                        img.show_size(ui, size);
                                    }
                                });
                            ui.vertical_centered_justified(|ui| {
                                let mut url = format!("{}/{}", self.oss.url, current_img.key);
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
                        State::Busy(route) => {
                            ui.centered_and_justified(|ui| match route {
                                Route::Upload => {
                                    ui.spinner();
                                    ui.heading("Uploading file...");
                                }
                                Route::List => {
                                    ui.spinner();
                                    ui.heading("Getting file list...");
                                }
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
